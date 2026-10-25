use std::{collections::HashSet, fmt, sync::OnceLock};

use lazy_static::lazy_static;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::{
    data::compose::ProjectKind,
    events::{
        bindings::{
            CloseBindings, SelectBinding, SelectNextBinding, SelectPreviousBinding, ShowBindings,
        },
        container::{
            DockerContainerConsole, DockerContainerFollowLogs, DockerContainerRestart,
            DockerContainerStart, DockerContainerStop, SelectNextContainer, SelectPrevContainer,
        },
        general::QuitEvent,
        project::{
            DeselectProject, DockerComposeDown, DockerComposeRestart, DockerComposeStart,
            DockerComposeStop, DockerComposeUp, QuitInfo, ScrollDownInfo, ScrollUpInfo,
            SelectNextProject, SelectPrevProject, SelectProject, ShowInfo,
        },
        prompt::{ClosePrompt, ConfirmPrompt},
    },
    model::{ContextId, Model, OverlayContextId, ViewId},
    settings::Settings,
    system::Message,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Global,
    Panel(ContextId),
    View(ViewId),
    OverlayContext(OverlayContextId),
    ComposeProject,
    Any(Vec<Condition>),
    All(Vec<Condition>),
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Key {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn code(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }
}

impl From<KeyCode> for Key {
    fn from(value: KeyCode) -> Self {
        Key::code(value)
    }
}

impl From<&Key> for KeyEvent {
    fn from(value: &Key) -> Self {
        KeyEvent {
            code: value.code,
            modifiers: value.modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}

impl From<KeyEvent> for Key {
    fn from(value: KeyEvent) -> Self {
        Key::new(value.code, value.modifiers)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.contains(KeyModifiers::CONTROL) {
            write!(f, "Ctrl+")?;
        }
        if self.modifiers.contains(KeyModifiers::SHIFT) {
            write!(f, "Shift+")?;
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            write!(f, "Alt+")?;
        }
        match self.code {
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Backspace => write!(f, "Backspace"),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Up => write!(f, "↑"),
            KeyCode::Down => write!(f, "↓"),
            KeyCode::Left => write!(f, "←"),
            KeyCode::Right => write!(f, "→"),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::End => write!(f, "End"),
            KeyCode::PageUp => write!(f, "PageUp"),
            KeyCode::PageDown => write!(f, "PageDown"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::BackTab => write!(f, "BackTab"),
            KeyCode::Delete => write!(f, "Delete"),
            KeyCode::Insert => write!(f, "Insert"),
            KeyCode::Null => write!(f, "Null"),
            KeyCode::Esc => write!(f, "Esc"),
            KeyCode::CapsLock => write!(f, "CapsLock"),
            KeyCode::ScrollLock => write!(f, "ScrollLock"),
            KeyCode::NumLock => write!(f, "NumLock"),
            KeyCode::PrintScreen => write!(f, "PrintScreen"),
            KeyCode::Pause => write!(f, "Pause"),
            KeyCode::Menu => write!(f, "Menu"),
            KeyCode::KeypadBegin => write!(f, "KeypadBegin"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[derive(Copy, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BindingId {
    Quit,
    MoveUp,
    MoveDown,
    DockerComposeStart,
    DockerComposeStop,
    DockerComposeDown,
    DockerComposeUp,
    DockerComposeRestart,
    DockerContainerStart,
    DockerContainerStop,
    DockerContainerRestart,
    DockerContainerFollowLogs,
    DockerContainerConsole,
    ShowBindings,
    Select,
    DeselectProject,
    ShowInfo,
    QuitInfo,
    ScrollUpInfo,
    ScrollDownInfo,
}

pub struct BindingContext {
    tags: HashSet<Condition>,
    modal: bool,
}

impl From<&Model> for BindingContext {
    fn from(model: &Model) -> Self {
        match &model.overlay {
            Some(overlay) => Self {
                tags: HashSet::from([Condition::OverlayContext(overlay.id())]),
                modal: true,
            },
            None => Self::beneath_overlay(model),
        }
    }
}

impl BindingContext {
    fn beneath_overlay(model: &Model) -> Self {
        let mut tags = HashSet::new();

        tags.insert(Condition::Panel(model.active_context));
        tags.insert(Condition::View(model.active_view));

        if let Some(project) = model.selected_project()
            && matches!(project.kind, ProjectKind::Compose(_))
        {
            tags.insert(Condition::ComposeProject);
        }

        Self { tags, modal: false }
    }
}

impl BindingContext {
    fn satisfies(&self, condition: &Condition) -> bool {
        match condition {
            Condition::Global => !self.modal,
            Condition::All(conditions) => conditions.iter().all(|cond| self.satisfies(cond)),
            Condition::Any(conditions) => conditions.iter().any(|cond| self.satisfies(cond)),
            cond => self.tags.contains(cond),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub keys: Vec<Key>,
    pub description: &'static str,
    pub binding_id: BindingId,
    pub event: Option<fn(&Model) -> Option<Message>>,
    pub condition: Condition,
}

impl Binding {
    pub fn display(&self) -> String {
        let mut out = Vec::new();
        for k in &self.keys {
            let s = k.to_string();
            if !out.iter().any(|x: &String| x.eq_ignore_ascii_case(&s)) {
                out.push(s)
            }
        }
        out.join(", ")
    }
}

lazy_static! {
    pub static ref BINDINGS: OnceLock<KeyBindings> = OnceLock::new();
}

pub fn bindings() -> &'static KeyBindings {
    BINDINGS.get().expect("BINDINGS not initialized")
}

pub struct KeyBindings {
    pub map: Vec<Binding>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            map: vec![
                // Bindings panel
                Binding {
                    keys: vec![KeyCode::Char('?').into()],
                    description: "Show keybindings",
                    binding_id: BindingId::ShowBindings,
                    event: Some(|_| Some(ShowBindings.into())),
                    condition: Condition::Global,
                },
                Binding {
                    keys: vec![KeyCode::Char('k').into(), KeyCode::Up.into()],
                    description: "Select previous binding",
                    binding_id: BindingId::MoveUp,
                    event: Some(|_| Some(SelectPreviousBinding.into())),
                    condition: Condition::OverlayContext(OverlayContextId::BindingsPopup),
                },
                Binding {
                    keys: vec![KeyCode::Char('j').into(), KeyCode::Down.into()],
                    description: "Select next binding",
                    binding_id: BindingId::MoveDown,
                    event: Some(|_| Some(SelectNextBinding.into())),
                    condition: Condition::OverlayContext(OverlayContextId::BindingsPopup),
                },
                Binding {
                    keys: vec![KeyCode::Enter.into()],
                    description: "Select binding",
                    binding_id: BindingId::Select,
                    event: Some(|_| Some(SelectBinding.into())),
                    condition: Condition::OverlayContext(OverlayContextId::BindingsPopup),
                },
                Binding {
                    keys: vec![KeyCode::Char('q').into(), KeyCode::Esc.into()],
                    description: "Close bindings popup",
                    binding_id: BindingId::Quit,
                    event: Some(|_| Some(CloseBindings.into())),
                    condition: Condition::OverlayContext(OverlayContextId::BindingsPopup),
                },
                // Prompt panel
                Binding {
                    keys: vec![KeyCode::Enter.into()],
                    description: "Select binding",
                    binding_id: BindingId::Select,
                    event: Some(|_| Some(ConfirmPrompt.into())),
                    condition: Condition::OverlayContext(OverlayContextId::Prompt),
                },
                Binding {
                    keys: vec![KeyCode::Char('q').into(), KeyCode::Esc.into()],
                    description: "Close bindings popup",
                    binding_id: BindingId::Quit,
                    event: Some(|_| Some(ClosePrompt.into())),
                    condition: Condition::OverlayContext(OverlayContextId::Prompt),
                },
                // Global
                Binding {
                    keys: vec![
                        KeyCode::Char('q').into(),
                        Key::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Key::new(KeyCode::Char('C'), KeyModifiers::CONTROL),
                    ],
                    description: "Quit application",
                    binding_id: BindingId::Quit,
                    event: Some(|_| Some(QuitEvent.into())),
                    condition: Condition::Global,
                },
                // Projects panel - navigation
                Binding {
                    keys: vec![KeyCode::Char('i').into()],
                    description: "Show info",
                    binding_id: BindingId::ShowInfo,
                    event: Some(|model| {
                        Some(
                            ShowInfo {
                                project_name: model.selected_project()?.name.clone(),
                            }
                            .into(),
                        )
                    }),
                    condition: Condition::Global,
                },
                Binding {
                    keys: vec![KeyCode::Esc.into()],
                    description: "Quit info view",
                    binding_id: BindingId::QuitInfo,
                    event: Some(|_| Some(QuitInfo.into())),
                    condition: Condition::View(ViewId::Info),
                },
                Binding {
                    keys: vec![KeyCode::PageDown.into()],
                    description: "Scroll down",
                    binding_id: BindingId::ScrollDownInfo,
                    event: Some(|_| Some(ScrollDownInfo.into())),
                    condition: Condition::View(ViewId::Info),
                },
                Binding {
                    keys: vec![KeyCode::PageUp.into()],
                    description: "Scroll up",
                    binding_id: BindingId::ScrollUpInfo,
                    event: Some(|_| Some(ScrollUpInfo.into())),
                    condition: Condition::View(ViewId::Info),
                },
                Binding {
                    keys: vec![KeyCode::Char('k').into(), KeyCode::Up.into()],
                    description: "Move selection up",
                    binding_id: BindingId::MoveUp,
                    event: Some(|_| Some(SelectPrevProject.into())),
                    condition: Condition::Panel(ContextId::Projects),
                },
                Binding {
                    keys: vec![KeyCode::Char('j').into(), KeyCode::Down.into()],
                    description: "Move selection down",
                    binding_id: BindingId::MoveDown,
                    event: Some(|_| Some(SelectNextProject.into())),
                    condition: Condition::Panel(ContextId::Projects),
                },
                Binding {
                    keys: vec![KeyCode::Enter.into()],
                    description: "Select",
                    binding_id: BindingId::Select,
                    event: Some(|model| {
                        Some(SelectProject(model.selected_project()?.clone()).into())
                    }),
                    condition: Condition::Panel(ContextId::Projects),
                },
                // Projects panel - compose actions
                Binding {
                    keys: vec![KeyCode::Char('u').into()],
                    description: "Docker compose up",
                    binding_id: BindingId::DockerComposeUp,
                    event: Some(|model| {
                        Some(DockerComposeUp(model.selected_project()?.clone()).into())
                    }),
                    condition: Condition::All(vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ]),
                },
                Binding {
                    keys: vec![KeyCode::Char('d').into()],
                    description: "Docker compose down",
                    binding_id: BindingId::DockerComposeDown,
                    event: Some(|model| {
                        Some(DockerComposeDown(model.selected_project()?.clone()).into())
                    }),
                    condition: Condition::All(vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ]),
                },
                Binding {
                    keys: vec![KeyCode::Char('s').into()],
                    description: "Docker compose start",
                    binding_id: BindingId::DockerComposeStart,
                    event: Some(|model| {
                        Some(DockerComposeStart(model.selected_project()?.clone()).into())
                    }),
                    condition: Condition::All(vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ]),
                },
                Binding {
                    keys: vec![KeyCode::Char('S').into()],
                    description: "Docker compose stop",
                    binding_id: BindingId::DockerComposeStop,
                    event: Some(|model| {
                        Some(DockerComposeStop(model.selected_project()?.clone()).into())
                    }),
                    condition: Condition::All(vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ]),
                },
                Binding {
                    keys: vec![KeyCode::Char('r').into()],
                    description: "Docker compose restart",
                    binding_id: BindingId::DockerComposeRestart,
                    event: Some(|model| {
                        Some(DockerComposeRestart(model.selected_project()?.clone()).into())
                    }),
                    condition: Condition::All(vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ]),
                },
                // Containers panel - navigation
                Binding {
                    keys: vec![KeyCode::Char('k').into(), KeyCode::Up.into()],
                    description: "Move selection up",
                    binding_id: BindingId::MoveUp,
                    event: Some(|_| Some(SelectPrevContainer.into())),
                    condition: Condition::Panel(ContextId::Containers),
                },
                Binding {
                    keys: vec![KeyCode::Char('j').into(), KeyCode::Down.into()],
                    description: "Move selection down",
                    binding_id: BindingId::MoveDown,
                    event: Some(|_| Some(SelectNextContainer.into())),
                    condition: Condition::Panel(ContextId::Containers),
                },
                Binding {
                    keys: vec![KeyCode::Esc.into()],
                    description: "Deselect",
                    binding_id: BindingId::DeselectProject,
                    event: Some(|_| Some(DeselectProject.into())),
                    condition: Condition::Panel(ContextId::Containers),
                },
                // Containers panel - container actions
                Binding {
                    keys: vec![KeyCode::Char('s').into()],
                    description: "Docker container start",
                    binding_id: BindingId::DockerContainerStart,
                    event: Some(|model| {
                        Some(
                            DockerContainerStart {
                                project: model.selected_project()?.clone(),
                                container: model.selected_container()?.clone(),
                            }
                            .into(),
                        )
                    }),
                    condition: Condition::Panel(ContextId::Containers),
                },
                Binding {
                    keys: vec![KeyCode::Char('S').into()],
                    description: "Docker container stop",
                    binding_id: BindingId::DockerContainerStop,
                    event: Some(|model| {
                        Some(
                            DockerContainerStop {
                                project: model.selected_project()?.clone(),
                                container: model.selected_container()?.clone(),
                            }
                            .into(),
                        )
                    }),
                    condition: Condition::Panel(ContextId::Containers),
                },
                Binding {
                    keys: vec![KeyCode::Char('r').into()],
                    description: "Docker container restart",
                    binding_id: BindingId::DockerContainerRestart,
                    event: Some(|model| {
                        Some(
                            DockerContainerRestart {
                                project: model.selected_project()?.clone(),
                                container: model.selected_container()?.clone(),
                            }
                            .into(),
                        )
                    }),
                    condition: Condition::Panel(ContextId::Containers),
                },
                Binding {
                    keys: vec![KeyCode::Char('m').into()],
                    description: "Docker follow logs",
                    binding_id: BindingId::DockerContainerFollowLogs,
                    event: Some(|model| {
                        Some(
                            DockerContainerFollowLogs {
                                project: model.selected_project()?.clone(),
                                container: model.selected_container()?.clone(),
                            }
                            .into(),
                        )
                    }),
                    condition: Condition::Panel(ContextId::Containers),
                },
                Binding {
                    keys: vec![KeyCode::Char('E').into()],
                    description: "Docker console",
                    binding_id: BindingId::DockerContainerConsole,
                    event: Some(|model| {
                        Some(
                            DockerContainerConsole {
                                project: model.selected_project()?.clone(),
                                container: model.selected_container()?.clone(),
                            }
                            .into(),
                        )
                    }),
                    condition: Condition::Panel(ContextId::Containers),
                },
            ],
        }
    }
}

impl KeyBindings {
    fn find_matching_binding(&self, key: &KeyEvent, model: &Model) -> Option<&Binding> {
        let binding_context = BindingContext::from(model);

        self.map.iter().find(|binding| {
            let key_matches = binding
                .keys
                .iter()
                .any(|k| key.code == k.code && key.modifiers.contains(k.modifiers));

            key_matches && binding_context.satisfies(&binding.condition)
        })
    }

    pub fn resolve_event(&self, key: &KeyEvent, model: &Model) -> Option<Message> {
        self.find_matching_binding(key, model)
            .and_then(|binding| binding.event?(model))
    }

    pub fn global(&self) -> Vec<Binding> {
        self.map
            .iter()
            .filter(|binding| binding.condition == Condition::Global)
            .cloned()
            .collect()
    }

    pub fn bindings_for(&self, model: &Model) -> Vec<Binding> {
        let binding_context = BindingContext::beneath_overlay(model);

        self.map
            .iter()
            .filter(|binding| {
                binding.condition != Condition::Global
                    && binding_context.satisfies(&binding.condition)
            })
            .cloned()
            .collect()
    }

    pub fn apply_config(&mut self, settings: Settings) {
        let Some(keybinds) = settings.keybindings else {
            return;
        };

        for binding in &mut self.map {
            if let Some(keys) = keybinds.get(&binding.binding_id) {
                binding.keys = keys.clone();
            }
        }
    }
}
