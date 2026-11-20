//! The chords of the modifier that no binding of a window manager uses,
//! which a window must not get: with `Super`, `Super+q` types no `q` in a
//! terminal and `Super+Tab` sends no Tab. The list is one for every
//! plugin that holds them: the tiling look, and the construct while the
//! keyboard is with a window in a pane. A program keeps the keys of other
//! modifiers, such as `Alt+b` of readline and `Alt+Left` of a browser, so
//! another modifier swallows nothing.

use chora_scene::{
    Binding,
    ipc::{ClientError, RpcError},
};

use crate::runtime::Runtime;

/// The keys that chords swallow when no binding uses them: letters,
/// digits, punctuation, the function keys and the keys of navigation.
pub const KEYS: &[&str] = &[
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

/// Every chord of [`KEYS`] with `modifier`, `modifier+Shift` and
/// `modifier+Ctrl`, when `modifier` is `Super`; with another modifier
/// none.
#[must_use]
pub fn chords(modifier: &str) -> Vec<String> {
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

impl<A: Clone> Runtime<A> {
    /// The chords of [`chords`] that the plugin holds no binding for.
    fn free_chords(&self, modifier: &str) -> Result<Vec<Binding>, ClientError> {
        let mut free = Vec::new();
        for chord in chords(modifier) {
            let binding: Binding = chord.parse().map_err(|e| {
                ClientError::Rpc(RpcError::new(
                    RpcError::INVALID_PARAMS,
                    format!("{chord}: {e}"),
                ))
            })?;
            if self.action(&binding.to_string()).is_none() {
                free.push(binding);
            }
        }
        Ok(free)
    }

    /// Grabs the chords of [`chords`] that the plugin holds no binding
    /// for, with `swallow` as their action, which must do nothing. It
    /// comes after the grabs of a set and before [`Self::end_grabs`]: the
    /// first grab of a chord wins, so a chord that the plugin bound stays
    /// with its action, and one that another client holds stays with it.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core knows no such key.
    pub fn grab_swallowed(&mut self, modifier: &str, swallow: &A) -> Result<(), ClientError> {
        for binding in self.free_chords(modifier)? {
            self.grab(binding, swallow.clone())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeCore;

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
            assert!(chord.parse::<Binding>().is_ok(), "{chord}");
        }
    }

    #[test]
    fn another_modifier_keeps_the_keys_of_the_programs() {
        assert_eq!(chords("Alt"), Vec::<String>::new());
        let core = FakeCore::start();
        let grabs = core.record(|method, _| (method == "input.grab").then(String::new));
        let mut rt = core.runtime::<&str>("swallow");
        rt.grab_swallowed("Alt", &"swallow").unwrap();
        assert_eq!(grabs.lock().unwrap().len(), 0);
    }

    #[test]
    fn a_bound_chord_keeps_its_action_and_the_rest_is_swallowed() {
        let core = FakeCore::start();
        let mut rt = core.runtime::<&str>("swallow");
        rt.start_grabs();
        rt.grab("Super+q".parse().unwrap(), "close").unwrap();
        rt.grab_swallowed("Super", &"swallow").unwrap();
        rt.end_grabs().unwrap();
        assert_eq!(rt.action("Super+q"), Some(&"close"));
        assert_eq!(rt.action("Super+d"), Some(&"swallow"));
        assert_eq!(rt.action("Shift+Super+x"), Some(&"swallow"));
    }

    #[test]
    fn a_chord_that_another_client_holds_stays_with_it() {
        let core = FakeCore::start();
        core.hold("Super+d");
        let mut rt = core.runtime::<&str>("swallow");
        rt.grab_swallowed("Super", &"swallow").unwrap();
        assert_eq!(rt.action("Super+d"), None);
        assert_eq!(rt.action("Super+e"), Some(&"swallow"));
    }

    #[test]
    fn the_swallowed_chords_stay_in_the_set_of_the_old_grabs() {
        let core = FakeCore::start();
        let grabs = core.record(|method, params| {
            (method == "input.grab").then(|| params["binding"].to_string())
        });
        let mut rt = core.runtime::<&str>("swallow");
        rt.grab_swallowed("Super", &"swallow").unwrap();
        let first = grabs.lock().unwrap().len();
        assert_eq!(first, 3 * KEYS.len());
        rt.start_grabs();
        rt.grab_swallowed("Super", &"swallow").unwrap();
        rt.end_grabs().unwrap();
        assert_eq!(
            grabs.lock().unwrap().len(),
            first,
            "no chord was free in between"
        );
    }
}
