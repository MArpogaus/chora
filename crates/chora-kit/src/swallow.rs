//! The chords of the modifier that no binding of a window manager uses,
//! which a window must not get: with `Super`, `Super+q` types no `q` in a
//! terminal and `Super+Tab` sends no Tab. The list is one for every
//! plugin that holds them: the tiling look, and the construct while the
//! keyboard is with a window in a pane. A program keeps the keys of other
//! modifiers, such as `Alt+b` of readline and `Alt+Left` of a browser, so
//! another modifier swallows nothing.

use chora_scene::{
    Binding, Event,
    ipc::{ClientError, RpcError},
};
use serde_json::{Value, json};

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

/// Whether `name` is a window manager of [`crate::give::KEYS`].
#[must_use]
pub fn manager(name: &str) -> bool {
    crate::give::KEYS.iter().any(|(_, to)| *to == name)
}

/// The event by which the window manager `name` says whether it
/// swallows: `NAME.swallow {on}` (D118).
#[must_use]
pub fn said_event(name: &str) -> String {
    format!("{name}.swallow")
}

/// Whether `event` is the event of a window manager that says whether
/// it swallows, which the runtime handles and no plugin gets.
#[must_use]
pub fn is_said(event: &Event) -> bool {
    matches!(event, Event::Emitted { event, client, .. }
        if manager(client) && *event == said_event(client))
}

impl<A> Runtime<A> {
    /// Says to the other window managers whether the plugin swallows, on
    /// a change only: a window manager that came earlier lets its swallow
    /// go while the plugin swallows (D118). A refusal of the core, such
    /// as its limit of events per second, is no error.
    pub(crate) fn say_swallow(&mut self, on: bool) -> Result<(), ClientError> {
        if self.said == on {
            return Ok(());
        }
        self.said = on;
        let said = json!({"event": said_event(&self.name), "data": {"on": on}});
        match self.conn.call::<_, Value>("events.emit", said) {
            Ok(_) | Err(ClientError::Rpc(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

impl<A: Clone> Runtime<A> {
    /// The chords of [`chords`] that the plugin holds no binding for and
    /// waits for none: a bound chord that the core refused stays bound.
    fn free_chords(&self, modifier: &str) -> Result<Vec<Binding>, ClientError> {
        let mut free = Vec::new();
        for chord in chords(modifier) {
            let binding: Binding = chord.parse().map_err(|e| {
                ClientError::Rpc(RpcError::new(
                    RpcError::INVALID_PARAMS,
                    format!("{chord}: {e}"),
                ))
            })?;
            let text = binding.to_string();
            let bound = |(waits, _, swallows): &(String, A, bool)| *waits == text && !swallows;
            if self.action(&text).is_none() && !self.waiting.iter().any(bound) {
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
    /// A refusal logs nothing.
    ///
    /// The swallow is the last claim (D118): while a window manager that
    /// connected after the plugin says that it swallows, the plugin
    /// swallows nothing, so that the joiner gets every chord that it
    /// binds and swallows the rest itself; see [`Self::swallow_event`].
    /// The plugin says so itself once it swallows, and
    /// [`Self::end_grabs`] of a set without a swallow says that it
    /// stopped.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core knows no such key.
    pub fn grab_swallowed(&mut self, modifier: &str, swallow: &A) -> Result<(), ClientError> {
        self.swallow = Some((modifier.to_owned(), swallow.clone()));
        let free = if self.joined.is_empty() {
            self.free_chords(modifier)?
        } else {
            Vec::new()
        };
        let mut asked = Vec::new();
        for binding in free {
            let text = binding.to_string();
            if let Some(i) = self.old_grabs.iter().position(|b| *b == text) {
                let normal = self.old_grabs.swap_remove(i);
                self.swallowed.insert(normal.clone());
                self.grabs.push((normal, swallow.clone()));
            } else {
                asked.push(text);
            }
        }
        self.ask_swallowed(asked, swallow)?;
        self.say_swallow(true)
    }

    /// Grabs the chords `asked` for the swallow in one batch, so that the
    /// plugin takes one round trip for about 200 chords and a press of the
    /// person meets a plugin that listens soon; a refused one waits.
    fn ask_swallowed(&mut self, asked: Vec<String>, swallow: &A) -> Result<(), ClientError> {
        if asked.is_empty() {
            return Ok(());
        }
        let calls: Vec<(&str, Value)> = (asked.iter())
            .map(|text| ("input.grab", json!({"binding": text})))
            .collect();
        let answers = self.conn.call_all(&calls)?;
        for (text, answer) in asked.into_iter().zip(answers) {
            self.waiting.retain(|(waits, ..)| *waits != text);
            match answer {
                Ok(normal) => {
                    let normal = normal.as_str().map_or_else(|| text.clone(), str::to_owned);
                    self.swallowed.insert(normal.clone());
                    self.grabs.push((normal, swallow.clone()));
                }
                Err(err) if err.code == RpcError::INVALID_PARAMS => {
                    return Err(RpcError::new(err.code, format!("{text}: {}", err.message)).into());
                }
                Err(_) => self.waiting.push((text, swallow.clone(), true)),
            }
        }
        Ok(())
    }

    /// The events of the swallow, which [`crate::runtime::dispatch`]
    /// hands here: a window manager of [`crate::give::KEYS`] that
    /// connected after the plugin and says that it swallows
    /// (`NAME.swallow {on: true}`) takes the swallow, which the plugin lets
    /// go at once; when the last one stopped or went the plugin swallows
    /// again. One that swallows nothing, as the construct with `--overlay`
    /// or without a window in a pane, takes nothing (rss-7). A binding that
    /// the core refused is asked again when the core tells that it is free
    /// (`input.ungrabbed`, D117): so the joiner gets the chords that the
    /// swallow let go after its own grab of them.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn swallow_event(&mut self, event: &Event) -> Result<(), ClientError> {
        match event {
            Event::ClientConnected { name } if manager(name) && *name != self.name => {
                self.after.insert(name.clone());
                Ok(())
            }
            Event::Emitted { client, data, .. }
                if is_said(event) && self.after.contains(client) =>
            {
                if data["on"].as_bool() == Some(true) {
                    self.joined.insert(client.clone());
                    self.let_swallow_go()
                } else {
                    self.leaves(client)
                }
            }
            Event::ClientDisconnected { name } => {
                self.after.remove(name);
                self.leaves(name)
            }
            Event::InputUngrabbed { binding } => self.ask_again(binding),
            _ => Ok(()),
        }
    }

    /// The window manager `name` swallows no more: when it was the last
    /// one that did, the plugin swallows again.
    fn leaves(&mut self, name: &str) -> Result<(), ClientError> {
        if !self.joined.remove(name) || !self.joined.is_empty() {
            return Ok(());
        }
        match self.swallow.clone() {
            Some((modifier, swallow)) => self.grab_swallowed(&modifier, &swallow),
            None => Ok(()),
        }
    }

    /// Lets every swallowed chord go, in one batch so that the joiner meets
    /// few of them, and forgets the refused ones.
    fn let_swallow_go(&mut self) -> Result<(), ClientError> {
        self.waiting.retain(|(.., swallows)| !swallows);
        let swallowed = std::mem::take(&mut self.swallowed);
        self.grabs.retain(|(normal, _)| !swallowed.contains(normal));
        let ungrabs: Vec<(&str, Value)> = (swallowed.iter())
            .filter_map(|normal| normal.parse::<Binding>().ok())
            .map(|binding| ("input.ungrab", json!({"binding": binding.to_string()})))
            .collect();
        if !ungrabs.is_empty() {
            self.conn.call_all(&ungrabs)?;
        }
        Ok(())
    }

    /// Grabs a binding that the core refused, now that another client let
    /// it go; a swallow only while no window manager joined, and not over
    /// a binding that the plugin holds by now.
    fn ask_again(&mut self, binding: &str) -> Result<(), ClientError> {
        let Ok(binding) = binding.parse::<Binding>() else {
            return Ok(());
        };
        let text = binding.to_string();
        let Some(i) = self.waiting.iter().position(|(waits, ..)| *waits == text) else {
            return Ok(());
        };
        let (_, action, swallows) = self.waiting.swap_remove(i);
        if swallows && (!self.joined.is_empty() || self.action(&text).is_some()) {
            return Ok(());
        }
        self.grab_as(binding, action, swallows).map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        runtime::{Plugin, dispatch},
        testing::{FakeCore, in_child, stderr_of},
    };

    /// A look on the kit runtime that only grabs: the runtime handles the
    /// events of the swallow.
    struct Look {
        rt: Runtime<&'static str>,
    }

    impl Plugin for Look {
        type Action = &'static str;

        fn runtime(&mut self) -> &mut Runtime<&'static str> {
            &mut self.rt
        }

        fn event(&mut self, _: Event) -> Result<(), ClientError> {
            Ok(())
        }

        fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
            Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method))
        }
    }

    fn joined(name: &str) -> Event {
        Event::ClientConnected { name: name.into() }
    }

    fn gone(name: &str) -> Event {
        Event::ClientDisconnected { name: name.into() }
    }

    /// The swallow is the last claim (D118): a window manager that joins
    /// gets every chord that it binds, also one that the look swallowed
    /// before it came (rss-1, rss-2). The look swallows again when it is
    /// the last window manager; a client that is no window manager takes
    /// nothing.
    #[test]
    fn a_swallow_yields_to_a_window_manager_that_joins_and_comes_back_when_it_goes() {
        let core = FakeCore::start();
        let mut look = Look {
            rt: core.runtime("xp"),
        };
        look.rt.grab("Super+d".parse().unwrap(), "desktop").unwrap();
        look.rt.grab_swallowed("Super", &"swallow").unwrap();
        dispatch(&mut look, joined("voice")).unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            Some(&"swallow"),
            "voice manages no windows"
        );
        dispatch(&mut look, joined("tiling")).unwrap();
        dispatch(&mut look, swallows("tiling", true)).unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            None,
            "rss-2: the swallow of xp holds Super+q against tiling, which joined"
        );
        assert_eq!(
            look.rt.action("Super+d"),
            Some(&"desktop"),
            "a bound chord stays"
        );
        let ungrabs = core.model().lock().unwrap().ungrabs.clone();
        assert!(ungrabs.contains(&json!("Super+q")), "{ungrabs:?}");
        assert!(!ungrabs.contains(&json!("Super+d")), "{ungrabs:?}");
        look.rt.start_grabs();
        look.rt.grab("Super+d".parse().unwrap(), "desktop").unwrap();
        look.rt.grab_swallowed("Super", &"swallow").unwrap();
        look.rt.end_grabs().unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            None,
            "a new set swallows nothing while tiling is there"
        );
        dispatch(&mut look, gone("tiling")).unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            Some(&"swallow"),
            "tiling went, xp swallows again"
        );
        assert_eq!(look.rt.action("Super+d"), Some(&"desktop"));
    }

    /// The event of a window manager that says whether it swallows, as
    /// the core hands it on.
    fn swallows(name: &str, on: bool) -> Event {
        Event::Emitted {
            event: format!("{name}.swallow"),
            client: name.into(),
            data: json!({"on": on}),
        }
    }

    /// A window manager that joins and swallows nothing, as the construct
    /// with `--overlay` or without a window in a pane, leaves the swallow
    /// with the look (rss-7): only a joiner that says that it swallows
    /// takes it, and when it stops the look swallows again (D118).
    #[test]
    fn a_joiner_that_swallows_nothing_leaves_the_swallow_with_the_look() {
        let core = FakeCore::start();
        let mut look = Look {
            rt: core.runtime("tiling"),
        };
        look.rt.grab_swallowed("Super", &"swallow").unwrap();
        dispatch(&mut look, joined("construct")).unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            Some(&"swallow"),
            "rss-7: tiling lets its swallow go to the construct, which swallows nothing"
        );
        dispatch(&mut look, swallows("construct", true)).unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            None,
            "the construct swallows for a window in a pane"
        );
        dispatch(&mut look, swallows("construct", false)).unwrap();
        assert_eq!(
            look.rt.action("Super+q"),
            Some(&"swallow"),
            "the construct stopped swallowing, tiling swallows again"
        );
    }

    /// A look whose swallow came first does not yield to the swallow of
    /// a look that came before it: else each yields to the other and
    /// nobody swallows.
    #[test]
    fn the_swallow_of_an_earlier_look_takes_nothing() {
        let core = FakeCore::start();
        let mut look = Look {
            rt: core.runtime("tiling"),
        };
        look.rt.grab_swallowed("Super", &"swallow").unwrap();
        dispatch(&mut look, swallows("xp", true)).unwrap();
        assert_eq!(look.rt.action("Super+q"), Some(&"swallow"));
    }

    /// A plugin says through the core whether it swallows, once when it
    /// starts and once when it stops (D118), so that a look that came
    /// earlier lets the swallow go to it and takes it back.
    #[test]
    fn a_swallow_says_when_it_starts_and_when_it_stops() {
        let core = FakeCore::start();
        let mut rt = core.runtime::<&str>("construct");
        rt.start_grabs();
        rt.grab_swallowed("Super", &"swallow").unwrap();
        rt.end_grabs().unwrap();
        rt.start_grabs();
        rt.grab_swallowed("Super", &"swallow").unwrap();
        rt.end_grabs().unwrap();
        rt.start_grabs();
        rt.end_grabs().unwrap();
        let emits = core.model().lock().unwrap().emits.clone();
        let said: Vec<_> = (emits.iter())
            .filter(|e| e["event"] == "construct.swallow")
            .map(|e| e["data"]["on"].clone())
            .collect();
        assert_eq!(said, [json!(true), json!(false)], "{emits:?}");
    }

    /// The window manager that joins asks again for a chord that it was
    /// refused, once the core tells that it is free (D117): the swallow of
    /// the other look let it go after the refusal (rss-1). Its own swallow
    /// leaves the refused binding to its action.
    #[test]
    fn a_refused_grab_is_asked_again_when_the_core_lets_it_go() {
        let core = FakeCore::start();
        core.hold("Super+1");
        let mut look = Look {
            rt: core.runtime("tiling"),
        };
        assert!(!look.rt.grab("Super+1".parse().unwrap(), "tag 1").unwrap());
        look.rt.grab_swallowed("Super", &"swallow").unwrap();
        core.model().lock().unwrap().held.clear();
        dispatch(
            &mut look,
            Event::InputUngrabbed {
                binding: "Super+1".into(),
            },
        )
        .unwrap();
        assert_eq!(
            look.rt.action("Super+1"),
            Some(&"tag 1"),
            "rss-1: Super+1 of tiling stays dead after the swallow of mc let it go"
        );
    }

    /// A chord that another client holds is no error of the swallow: it
    /// logs nothing (rss-4).
    #[test]
    fn a_refused_swallow_logs_nothing() {
        if in_child() {
            let core = FakeCore::start();
            core.hold("Super+q");
            let mut rt = core.runtime::<&str>("mc");
            rt.grab_swallowed("Super", &"swallow").unwrap();
            return;
        }
        let log = stderr_of("swallow::tests::a_refused_swallow_logs_nothing");
        assert!(!log.contains("Super+q"), "rss-4: {log}");
    }

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
