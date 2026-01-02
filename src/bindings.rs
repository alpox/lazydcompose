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
}

struct Binding {
    keys: &'static str,
    description: &'static str,
    panels: Vec<PanelId>,
    matcher: fn(KeyEvent) -> bool,
    action: KeyAction,
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
                    action: KeyAction::Quit,
                },
                Binding {
                    keys: "k, ↑",
                    description: "Move selection up",
                    panels: vec![PanelId::Projects, PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('k')),
                    action: KeyAction::MoveUp,
                },
                Binding {
                    keys: "j, ↓",
                    description: "Move selection down",
                    panels: vec![PanelId::Projects, PanelId::Containers],
                    matcher: |k| matches!(k.code, KeyCode::Char('j')),
                    action: KeyAction::MoveDown,
                },
                Binding {
                    keys: "l, →",
                    description: "Select next panel",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('l')),
                    action: KeyAction::NextPanel,
                },
                Binding {
                    keys: "h, ←",
                    description: "Select previous panel",
                    panels: vec![],
                    matcher: |k| matches!(k.code, KeyCode::Char('h')),
                    action: KeyAction::PrevPanel,
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
                    action: KeyAction::PrevPanel,
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
            .map(|binding| binding.action)
    }
}
