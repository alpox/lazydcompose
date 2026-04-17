use std::{collections::HashSet, sync::OnceLock};

use lazy_static::lazy_static;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::{
    cli::ProjectKind,
    model::{ContextId, Model, OverlayContextId},
    settings::Settings,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Condition {
    Panel(ContextId),
    OverlayContext(OverlayContextId),
    ComposeProject,
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

#[derive(Copy, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyAction {
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
    DockerFollowLogs,
    DockerConsole,
    ShowBindings,
    ClosePopup,
    Select,
    Deselect,
    ChooseAction,
    NextBinding,
    PreviousBinding,
}

pub struct BindingContext {
    tags: HashSet<Condition>,
}

impl From<&Model> for BindingContext {
    fn from(model: &Model) -> Self {
        let mut tags = HashSet::new();

        tags.insert(Condition::Panel(model.active_context));

        if let Some(project) = model.selected_project()
            && matches!(project.kind, ProjectKind::Compose(_))
        {
            tags.insert(Condition::ComposeProject);
        }

        Self::new(tags)
    }
}

impl BindingContext {
    pub fn new(tags: impl Into<HashSet<Condition>>) -> Self {
        Self { tags: tags.into() }
    }

    fn satisfies(&self, conditions: &[Condition]) -> bool {
        conditions.is_empty() || conditions.iter().all(|cond| self.tags.contains(cond))
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub keys: Vec<Key>,
    pub display: &'static str,
    pub description: &'static str,
    pub action: KeyAction,
    pub conditions: Vec<Condition>,
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
                // Global
                Binding {
                    keys: vec![
                        KeyCode::Char('q').into(),
                        Key::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                        Key::new(KeyCode::Char('C'), KeyModifiers::CONTROL),
                    ],
                    display: "q, Ctrl+c",
                    description: "Quit application",
                    action: KeyAction::Quit,
                    conditions: vec![],
                },
                Binding {
                    keys: vec![KeyCode::Char('?').into()],
                    display: "?",
                    description: "Show keybindings",
                    action: KeyAction::ShowBindings,
                    conditions: vec![],
                },
                // Projects panel - navigation
                Binding {
                    keys: vec![KeyCode::Char('k').into(), KeyCode::Up.into()],
                    display: "k, ↑",
                    description: "Move selection up",
                    action: KeyAction::MoveUp,
                    conditions: vec![Condition::Panel(ContextId::Projects)],
                },
                Binding {
                    keys: vec![KeyCode::Char('j').into(), KeyCode::Down.into()],
                    display: "j, ↓",
                    description: "Move selection down",
                    action: KeyAction::MoveDown,
                    conditions: vec![Condition::Panel(ContextId::Projects)],
                },
                Binding {
                    keys: vec![KeyCode::Enter.into()],
                    display: "Enter",
                    description: "Select",
                    action: KeyAction::Select,
                    conditions: vec![Condition::Panel(ContextId::Projects)],
                },
                // Projects panel - compose actions
                Binding {
                    keys: vec![KeyCode::Char('u').into()],
                    display: "u",
                    description: "Docker compose up",
                    action: KeyAction::DockerComposeUp,
                    conditions: vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ],
                },
                Binding {
                    keys: vec![KeyCode::Char('d').into()],
                    display: "d",
                    description: "Docker compose down",
                    action: KeyAction::DockerComposeDown,
                    conditions: vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ],
                },
                Binding {
                    keys: vec![KeyCode::Char('s').into()],
                    display: "s",
                    description: "Docker compose start",
                    action: KeyAction::DockerComposeStart,
                    conditions: vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ],
                },
                Binding {
                    keys: vec![KeyCode::Char('S').into()],
                    display: "S",
                    description: "Docker compose stop",
                    action: KeyAction::DockerComposeStop,
                    conditions: vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ],
                },
                Binding {
                    keys: vec![KeyCode::Char('r').into()],
                    display: "r",
                    description: "Docker compose restart",
                    action: KeyAction::DockerComposeRestart,
                    conditions: vec![
                        Condition::Panel(ContextId::Projects),
                        Condition::ComposeProject,
                    ],
                },
                // Containers panel - navigation
                Binding {
                    keys: vec![KeyCode::Char('k').into(), KeyCode::Up.into()],
                    display: "k, ↑",
                    description: "Move selection up",
                    action: KeyAction::MoveUp,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Char('j').into(), KeyCode::Down.into()],
                    display: "j, ↓",
                    description: "Move selection down",
                    action: KeyAction::MoveDown,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Enter.into()],
                    display: "Enter",
                    description: "Select",
                    action: KeyAction::Select,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Esc.into()],
                    display: "Esc",
                    description: "Deselect",
                    action: KeyAction::Deselect,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                // Containers panel - container actions
                Binding {
                    keys: vec![KeyCode::Char('s').into()],
                    display: "s",
                    description: "Docker container start",
                    action: KeyAction::DockerContainerStart,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Char('S').into()],
                    display: "S",
                    description: "Docker container stop",
                    action: KeyAction::DockerContainerStop,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Char('r').into()],
                    display: "r",
                    description: "Docker container restart",
                    action: KeyAction::DockerContainerRestart,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Char('m').into()],
                    display: "m",
                    description: "Docker follow logs",
                    action: KeyAction::DockerFollowLogs,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
                Binding {
                    keys: vec![KeyCode::Char('E').into()],
                    display: "E",
                    description: "Docker console",
                    action: KeyAction::DockerConsole,
                    conditions: vec![Condition::Panel(ContextId::Containers)],
                },
            ],
        }
    }
}

impl KeyBindings {
    pub fn resolve(&self, key: &KeyEvent, model: &Model) -> Option<KeyAction> {
        let binding_context = BindingContext::from(model);

        self.map
            .iter()
            .find(|binding| {
                let key_matches = binding
                    .keys
                    .iter()
                    .any(|k| key.code == k.code && key.modifiers.contains(k.modifiers));

                key_matches && binding_context.satisfies(binding.conditions.as_slice())
            })
            .map(|binding| binding.action)
    }

    pub fn global(&self) -> Vec<Binding> {
        self.map
            .iter()
            .filter(|binding| binding.conditions.is_empty())
            .cloned()
            .collect()
    }

    pub fn bindings_for(&self, model: &Model) -> Vec<Binding> {
        let binding_context = BindingContext::from(model);

        self.map
            .iter()
            .filter(|binding| {
                !binding.conditions.is_empty()
                    && binding_context.satisfies(binding.conditions.as_slice())
            })
            .cloned()
            .collect()
    }

    pub fn apply_config(&mut self, settings: Settings) {
        for binding in &mut self.map {
            if let Some(keys) = settings.keybindings.get(&binding.action) {
                binding.keys = keys.clone();
            }
        }
    }
}
