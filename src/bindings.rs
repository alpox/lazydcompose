use lazy_static::lazy_static;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::model::ContextId;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub keys: &'static str,
    pub description: &'static str,
    pub panels: Vec<ContextId>,
    global: bool,
    matcher: fn(KeyEvent) -> bool,
    action: fn(KeyEvent) -> Option<KeyAction>,
}

lazy_static! {
    pub static ref BINDINGS: KeyBindings = KeyBindings::default();
}

pub struct KeyBindings {
    pub map: Vec<Binding>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            map: vec![
                Binding {
                    keys: "q, Ctrl+c",
                    description: "Quit application",
                    panels: vec![],
                    global: true,
                    matcher: |k| {
                        matches!(
                            k,
                            KeyEvent {
                                code: KeyCode::Char('q'),
                                ..
                            } | KeyEvent {
                                code: KeyCode::Char('c' | 'C'),
                                modifiers: KeyModifiers::CONTROL,
                                ..
                            }
                        )
                    },
                    action: |_| Some(KeyAction::Quit),
                },
                Binding {
                    keys: "k, ↑",
                    description: "Move selection up",
                    panels: vec![ContextId::Projects, ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('k') | KeyCode::Up),
                    action: |_| Some(KeyAction::MoveUp),
                },
                Binding {
                    keys: "j, ↓",
                    description: "Move selection down",
                    panels: vec![ContextId::Projects, ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('j') | KeyCode::Down),
                    action: |_| Some(KeyAction::MoveDown),
                },
                Binding {
                    keys: "s",
                    description: "Docker compose start",
                    panels: vec![ContextId::Projects],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('s')),
                    action: |_| Some(KeyAction::DockerComposeStart),
                },
                Binding {
                    keys: "S",
                    description: "Docker compose stop",
                    panels: vec![ContextId::Projects],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('S')),
                    action: |_| Some(KeyAction::DockerComposeStop),
                },
                Binding {
                    keys: "u",
                    description: "Docker compose up",
                    panels: vec![ContextId::Projects],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('u')),
                    action: |_| Some(KeyAction::DockerComposeUp),
                },
                Binding {
                    keys: "r",
                    description: "Docker compose restart",
                    panels: vec![ContextId::Projects],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('r')),
                    action: |_| Some(KeyAction::DockerComposeRestart),
                },
                Binding {
                    keys: "d",
                    description: "Docker compose down",
                    panels: vec![ContextId::Projects],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('d')),
                    action: |_| Some(KeyAction::DockerComposeDown),
                },
                Binding {
                    keys: "s",
                    description: "Docker container start",
                    panels: vec![ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('s')),
                    action: |_| Some(KeyAction::DockerContainerStart),
                },
                Binding {
                    keys: "S",
                    description: "Docker container stop",
                    panels: vec![ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('S')),
                    action: |_| Some(KeyAction::DockerContainerStop),
                },
                Binding {
                    keys: "r",
                    description: "Docker container restart",
                    panels: vec![ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('r')),
                    action: |_| Some(KeyAction::DockerContainerRestart),
                },
                Binding {
                    keys: "?",
                    description: "Show keybindings",
                    panels: vec![],
                    global: true,
                    matcher: |k| matches!(k.code, KeyCode::Char('?')),
                    action: |_| Some(KeyAction::ShowBindings),
                },
                Binding {
                    keys: "m",
                    description: "Docker follow logs",
                    panels: vec![ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('m')),
                    action: |_| Some(KeyAction::DockerFollowLogs),
                },
                Binding {
                    keys: "E",
                    description: "Docker console",
                    panels: vec![ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Char('E')),
                    action: |_| Some(KeyAction::DockerConsole),
                },
                Binding {
                    keys: "Esc",
                    description: "Deselect",
                    panels: vec![ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Esc),
                    action: |_| Some(KeyAction::Deselect),
                },
                Binding {
                    keys: "Enter",
                    description: "Select",
                    panels: vec![ContextId::Projects, ContextId::Containers],
                    global: false,
                    matcher: |k| matches!(k.code, KeyCode::Enter),
                    action: |_| Some(KeyAction::Select),
                },
            ],
        }
    }
}

impl KeyBindings {
    pub fn get(&self, key: &KeyEvent) -> Option<KeyAction> {
        self.map
            .iter()
            .find(|binding| (binding.matcher)(*key))
            .and_then(|binding| (binding.action)(*key))
    }

    pub fn get_for_context(&self, key: &KeyEvent, context: ContextId) -> Option<KeyAction> {
        self.map
            .iter()
            .find(|binding| {
                (binding.global || binding.panels.contains(&context)) && (binding.matcher)(*key)
            })
            .and_then(|binding| (binding.action)(*key))
    }

    pub fn global(&self) -> Vec<Binding> {
        self.map
            .iter()
            .filter(|binding| binding.panels.is_empty())
            .cloned()
            .collect()
    }

    pub fn bindings_for(&self, panel: ContextId) -> Vec<Binding> {
        self.map
            .iter()
            .filter(|binding| binding.panels.contains(&panel))
            .cloned()
            .collect()
    }
}
