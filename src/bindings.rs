use lazy_static::lazy_static;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::model::PanelId;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Quit,
    MoveUp,
    MoveDown,
    NextPanel,
    PrevPanel,
    SelectPanel(usize),
    DockerComposeStart,
    DockerComposeStop,
    DockerComposeDown,
    DockerComposeUp,
    DockerComposeRestart,
    DockerFollowLogs,
    DockerConsole,
    ShowBindings,
    ClosePopup,
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub keys: &'static str,
    pub description: &'static str,
    pub panels: Vec<PanelId>,
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
                    panels: vec![PanelId::Projects, PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('k') | KeyCode::Up),
                    action: |_| Some(KeyAction::MoveUp),
                },
                Binding {
                    keys: "j, ↓",
                    description: "Move selection down",
                    panels: vec![PanelId::Projects, PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('j') | KeyCode::Down),
                    action: |_| Some(KeyAction::MoveDown),
                },
                Binding {
                    keys: "l, →",
                    description: "Select next panel",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('l') | KeyCode::Right),
                    action: |_| Some(KeyAction::NextPanel),
                },
                Binding {
                    keys: "h, ←",
                    description: "Select previous panel",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('h') | KeyCode::Left),
                    action: |_| Some(KeyAction::PrevPanel),
                },
                Binding {
                    keys: "1-9",
                    description: "Select panel <nr>",
                    panels: vec![],
                    matcher: |k| {
                        matches!(
                            k.code,
                            KeyCode::Char('1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')
                        )
                    },
                    action: |k| match k.code {
                        KeyCode::Char(num) => {
                            num.to_digit(10).map(|d| KeyAction::SelectPanel(d as usize))
                        }
                        _ => None,
                    },
                },
                Binding {
                    keys: "s",
                    description: "Docker compose start",
                    panels: vec![PanelId::Projects],
                    matcher: |k| matches!(k.code, KeyCode::Char('s')),
                    action: |_| Some(KeyAction::DockerComposeStart),
                },
                Binding {
                    keys: "S",
                    description: "Docker compose stop",
                    panels: vec![PanelId::Projects],
                    matcher: |k| matches!(k.code, KeyCode::Char('S')),
                    action: |_| Some(KeyAction::DockerComposeStop),
                },
                Binding {
                    keys: "u",
                    description: "Docker compose up",
                    panels: vec![PanelId::Projects],
                    matcher: |k| matches!(k.code, KeyCode::Char('u')),
                    action: |_| Some(KeyAction::DockerComposeUp),
                },
                Binding {
                    keys: "r",
                    description: "Docker compose restart",
                    panels: vec![PanelId::Projects],
                    matcher: |k| matches!(k.code, KeyCode::Char('r')),
                    action: |_| Some(KeyAction::DockerComposeRestart),
                },
                Binding {
                    keys: "d",
                    description: "Docker compose down",
                    panels: vec![PanelId::Projects],
                    matcher: |k| matches!(k.code, KeyCode::Char('d')),
                    action: |_| Some(KeyAction::DockerComposeDown),
                },
                Binding {
                    keys: "?",
                    description: "Show keybindings",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('?')),
                    action: |_| Some(KeyAction::ShowBindings),
                },
                Binding {
                    keys: "Esc",
                    description: "Close popup",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Esc),
                    action: |_| Some(KeyAction::ClosePopup),
                },
                Binding {
                    keys: "m",
                    description: "Docker follow logs",
                    panels: vec![PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('m')),
                    action: |_| Some(KeyAction::DockerFollowLogs),
                },
                Binding {
                    keys: "E",
                    description: "Docker console",
                    panels: vec![PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('E')),
                    action: |_| Some(KeyAction::DockerConsole),
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

    pub fn global(&self) -> Vec<Binding> {
        self.map
            .iter()
            .filter(|binding| binding.panels.is_empty())
            .cloned()
            .collect()
    }

    pub fn bindings_for(&self, panel: PanelId) -> Vec<Binding> {
        self.map
            .iter()
            .filter(|binding| binding.panels.contains(&panel))
            .cloned()
            .collect()
    }
}
