use lazy_static::lazy_static;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::model::PanelId;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyAction {
    None,
    Quit,
    MoveUp,
    MoveDown,
    NextPanel,
    PrevPanel,
    SelectPanel(usize),
}

struct Binding {
    keys: &'static str,
    description: &'static str,
    panels: Vec<PanelId>,
    matcher: fn(KeyEvent) -> bool,
    action: fn(KeyEvent) -> KeyAction,
}

lazy_static! {
    pub static ref Bindings: KeyBindings = KeyBindings::default();
}

pub struct KeyBindings {
    map: Vec<Binding>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            map: vec![
                Binding {
                    keys: "q, Esc",
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
                    action: |_| KeyAction::Quit,
                },
                Binding {
                    keys: "k, ↑",
                    description: "Move selection up",
                    panels: vec![PanelId::Projects, PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('k') | KeyCode::Up),
                    action: |_| KeyAction::MoveUp,
                },
                Binding {
                    keys: "j, ↓",
                    description: "Move selection down",
                    panels: vec![PanelId::Projects, PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('j') | KeyCode::Down),
                    action: |_| KeyAction::MoveDown,
                },
                Binding {
                    keys: "l, →",
                    description: "Select next panel",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('l') | KeyCode::Right),
                    action: |_| KeyAction::NextPanel,
                },
                Binding {
                    keys: "h, ←",
                    description: "Select previous panel",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('h') | KeyCode::Left),
                    action: |_| KeyAction::PrevPanel,
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
                        KeyCode::Char(num) => num
                            .to_digit(10)
                            .map(|d| KeyAction::SelectPanel(d as usize))
                            .unwrap_or(KeyAction::None),
                        _ => KeyAction::None,
                    },
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
            .map(|binding| (binding.action)(*key))
    }
}
