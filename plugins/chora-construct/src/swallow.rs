//! The chords of `Super` that no binding of the construct uses, which a
//! window must not get while the keyboard may be with it (in a pane or
//! on a face of the cube): `chora-tiling` holds them the same way
//! (`docs/configuration.md`, Tiling). A program keeps the keys of other
//! modifiers, such as `Alt+b` of readline.

/// The keys that `Super` chords swallow when no binding uses them: letters,
/// digits, punctuation, the function keys and the keys of navigation.
const KEYS: &[&str] = &[
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "Return",
    "Tab",
    "space",
    "Escape",
    "BackSpace",
    "Delete",
    "Insert",
    "Home",
    "End",
    "Prior",
    "Next",
    "Left",
    "Right",
    "Up",
    "Down",
    "minus",
    "equal",
    "bracketleft",
    "bracketright",
    "semicolon",
    "apostrophe",
    "grave",
    "backslash",
    "comma",
    "period",
    "slash",
    "Print",
];

/// The extra modifiers of the swallowed chords: none, Shift and Ctrl.
const WITH: [&str; 3] = ["", "Shift+", "Ctrl+"];

/// Every chord of the swallowed keys with `modifier`, `modifier+Shift`
/// and `modifier+Ctrl`, when `modifier` is `Super`; with another modifier
/// none.
pub(crate) fn chords(modifier: &str) -> Vec<String> {
    if modifier != "Super" {
        return Vec::new();
    }
    (WITH.iter())
        .flat_map(|with| {
            KEYS.iter()
                .map(move |key| format!("{modifier}+{with}{key}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super_chords_with_shift_and_ctrl_are_swallowed() {
        let chords = chords("Super");
        assert_eq!(chords.len(), 3 * KEYS.len());
        for chord in [
            "Super+d",
            "Super+q",
            "Super+Tab",
            "Super+1",
            "Super+Shift+x",
            "Super+Ctrl+z",
        ] {
            assert!(chords.contains(&chord.to_owned()), "{chord}");
        }
        for chord in &chords {
            assert!(chord.parse::<chora_scene::Binding>().is_ok(), "{chord}");
        }
    }

    #[test]
    fn another_modifier_keeps_the_keys_of_the_programs() {
        assert_eq!(chords("Alt").len(), 0);
    }
}
