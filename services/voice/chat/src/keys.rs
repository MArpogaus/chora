//! What a key of the keyboard means to the field.

use xkbcommon::xkb::keysyms::{
    KEY_BackSpace as BACKSPACE, KEY_Delete as DELETE, KEY_End as END, KEY_Escape as ESCAPE,
    KEY_Home as HOME, KEY_Insert as INSERT, KEY_KP_Enter as KP_ENTER, KEY_Left as LEFT,
    KEY_Return as RETURN, KEY_Right as RIGHT,
};

use crate::session::Key;

/// The modifiers that count.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mods {
    /// Ctrl.
    pub ctrl: bool,
    /// Shift.
    pub shift: bool,
    /// Alt.
    pub alt: bool,
}

/// What a key press asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum Decoded {
    /// A key of the draft.
    Key(Key),
    /// Ctrl+V or Shift+Insert: the clipboard.
    Paste,
    /// Nothing for the field.
    Ignore,
}

/// The meaning of a keysym with its modifiers and the text that the
/// keymap gives it. A key with Ctrl or Alt is never text: Alt+Y of a
/// question is a key of the window manager, and the field never takes it.
#[must_use]
pub fn decode(sym: u32, mods: Mods, utf8: &str) -> Decoded {
    match sym {
        RETURN | KP_ENTER if mods.shift => Decoded::Key(Key::Newline),
        RETURN | KP_ENTER => Decoded::Key(Key::Enter),
        ESCAPE => Decoded::Key(Key::Escape),
        BACKSPACE => Decoded::Key(Key::Backspace),
        DELETE => Decoded::Key(Key::Delete),
        LEFT => Decoded::Key(Key::Left),
        RIGHT => Decoded::Key(Key::Right),
        HOME => Decoded::Key(Key::Home),
        END => Decoded::Key(Key::End),
        INSERT if mods.shift => Decoded::Paste,
        0x76 | 0x56 if mods.ctrl => Decoded::Paste,
        _ if mods.ctrl || mods.alt => Decoded::Ignore,
        _ if utf8.is_empty() || utf8.chars().any(char::is_control) => Decoded::Ignore,
        _ => Decoded::Key(Key::Text(utf8.to_owned())),
    }
}

/// Whether a held key repeats: text and editing do, Enter and Esc do not.
#[must_use]
pub fn repeats(key: &Key) -> bool {
    !matches!(key, Key::Enter | Key::Newline | Key::Escape)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: Mods = Mods {
        ctrl: false,
        shift: false,
        alt: false,
    };

    #[test]
    fn enter_sends_and_shift_enter_breaks_the_line() {
        assert_eq!(decode(RETURN, NONE, "\r"), Decoded::Key(Key::Enter));
        let shift = Mods {
            shift: true,
            ..NONE
        };
        assert_eq!(decode(RETURN, shift, "\r"), Decoded::Key(Key::Newline));
    }

    #[test]
    fn ctrl_v_pastes_and_a_chord_is_no_text() {
        let ctrl = Mods { ctrl: true, ..NONE };
        assert_eq!(decode(0x76, ctrl, "\u{16}"), Decoded::Paste);
        assert_eq!(decode(0x79, ctrl, "\u{19}"), Decoded::Ignore);
        let alt = Mods { alt: true, ..NONE };
        assert_eq!(decode(0x79, alt, "y"), Decoded::Ignore);
    }

    #[test]
    fn a_character_is_text() {
        assert_eq!(decode(0xe4, NONE, "ä"), Decoded::Key(Key::Text("ä".into())));
        assert_eq!(decode(0xffe1, NONE, ""), Decoded::Ignore);
    }
}
