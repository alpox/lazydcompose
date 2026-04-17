use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, Visitor},
};

use crate::bindings::{Key, KeyAction};

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(KeyVisitor)
    }
}

struct KeyVisitor;

impl<'de> Visitor<'de> for KeyVisitor {
    type Value = Key;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a key string like \"ctrl+f\" or \"shift+enter\"")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Key, E> {
        let mut parts = s.split('+');
        let mut modifiers = KeyModifiers::NONE;

        let last = loop {
            match parts.next() {
                Some("ctrl") => modifiers |= KeyModifiers::CONTROL,
                Some("alt") => modifiers |= KeyModifiers::ALT,
                Some("shift") => modifiers |= KeyModifiers::SHIFT,
                Some(other) => break other,
                None => return Err(E::custom("empty key string")),
            }
        };

        if parts.next().is_some() {
            return Err(E::custom(format!(
                "key code must be the last segment in \"{s}\""
            )));
        }

        let code = parse_key_code(last)
            .ok_or_else(|| E::custom(format!("unknown key code \"{last}\"")))?;

        Ok(Key::new(code, modifiers))
    }
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    let code = match s.to_ascii_lowercase().as_str() {
        "space" => KeyCode::Char(' '),
        "backspace" => KeyCode::Backspace,
        "enter" | "return" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "delete" | "del" => KeyCode::Delete,
        "insert" | "ins" => KeyCode::Insert,
        "null" => KeyCode::Null,
        "esc" | "escape" => KeyCode::Esc,
        "capslock" => KeyCode::CapsLock,
        "scrolllock" => KeyCode::ScrollLock,
        "numlock" => KeyCode::NumLock,
        "printscreen" => KeyCode::PrintScreen,
        "pause" => KeyCode::Pause,
        "menu" => KeyCode::Menu,
        "keypadbegin" => KeyCode::KeypadBegin,
        _ => {
            let mut chars = s.chars();
            let c = chars.next()?;
            if chars.next().is_some() {
                return None;
            }
            KeyCode::Char(c)
        }
    };
    Some(code)
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Settings {
    pub keybindings: HashMap<KeyAction, Vec<Key>>,
}
