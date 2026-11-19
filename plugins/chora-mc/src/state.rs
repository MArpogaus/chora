//! `state.get`: what mc shows and holds, in the shape of `choractl state`
//! (design 3.1). The fields that only the core knows (title, app id,
//! rectangle, stacking) and those of looks that mc has no concept of
//! (desks, views, texts) are left out: a field that is absent is not read
//! as empty.

use serde_json::{Value, json};

use crate::{Mc, NAME};

/// The node of the key list.
const KEYS: &str = "mc-keys";

impl Mc {
    /// The windows of the scenes and the docks, the focus and what still
    /// moves. mc holds the focus only on one of its windows; the one shell
    /// surface it has is the key list with every binding.
    pub(crate) fn state_value(&self) -> Value {
        let output = self.output.as_ref().map(|o| o.name.as_str());
        let shown = |scene: &str| self.current.as_deref() == Some(scene);
        let origin = |surface: &str| self.book.origin(surface).map(chora_kit::wm::Origin::word);
        let mut windows: Vec<Value> = self
            .windows
            .iter()
            .map(|(scene, surface)| {
                json!({"surface": surface, "scene": scene, "output": output,
                    "shown": shown(scene)})
            })
            .chain(self.docks.iter().map(|(content, surface)| {
                json!({"surface": surface, "dock": content, "output": output, "shown": true})
            }))
            .collect();
        for window in &mut windows {
            let word = window["surface"].as_str().and_then(origin);
            if let Some(word) = word {
                window["origin"] = json!(word);
            }
        }
        windows.sort_by_key(|w| w["surface"].as_str().map(str::to_owned));
        let mut busy: Vec<&str> = self.writing.keys().map(String::as_str).collect();
        busy.sort_unstable();
        let mut busy: Vec<String> = busy.into_iter().map(|p| format!("write:{p}")).collect();
        if self.typing.is_some() {
            busy.push("typing".to_owned());
        }
        let own = |s: &str| windows.iter().any(|w| w["surface"] == s);
        let focus = serde_json::to_value(self.focus.state(NAME, own)).unwrap_or_default();
        json!({"windows": windows, "focus": focus, "open": self.open().state(),
            "busy": busy, "helper": self.voiced_word.word()})
    }

    /// The open shell surfaces: the key list shows all bindings after the
    /// `keys` binding, and holds the keyboard until Escape.
    fn open(&self) -> chora_kit::wm::open::Open {
        let mut open = chora_kit::wm::open::Open::default();
        if self.keys == crate::keys::List::All {
            let sheet = chora_kit::wm::open::Surface::new(KEYS, chora_kit::wm::open::Kind::Sheet);
            open.open(sheet.modal());
        }
        open
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::runtime::Plugin;
    use serde_json::json;

    use crate::testing;

    /// A window of a scene is in the state, and the focus names it once
    /// the core says that it has the keyboard.
    #[test]
    fn the_state_lists_the_windows_and_the_focus() {
        let (mut mc, _model) = testing::mc("state", [1280, 800]);
        let state = mc.state().expect("mc has a state");
        assert_eq!(state["windows"], serde_json::json!([]));
        mc.windows.insert("agenda".into(), "s1".into());
        mc.focus.moved(Some("s1"));
        let state = mc.state().unwrap();
        assert_eq!(state["windows"][0]["surface"], "s1");
        assert_eq!(state["windows"][0]["scene"], "agenda");
        assert_eq!(state["windows"][0]["output"], "test");
        assert_eq!(state["focus"]["surface"], "s1");
        mc.focus.moved(Some("other"));
        assert!(mc.state().unwrap()["focus"].get("surface").is_none());
    }

    /// mc holds the focus only while the core focus is on a window of
    /// mc; else no look of mc's holds it (net1 review).
    #[test]
    fn the_focus_holder_is_mc_only_on_a_window_of_mc() {
        let (mut mc, _model) = testing::mc("holder", [1280, 800]);
        let state = mc.state().unwrap();
        assert_eq!(state["focus"]["holder"], serde_json::Value::Null);
        assert_eq!(state["open"], serde_json::json!([]));
        mc.windows.insert("agenda".into(), "s1".into());
        mc.focus.moved(Some("other"));
        assert_eq!(
            mc.state().unwrap()["focus"]["holder"],
            serde_json::Value::Null
        );
        mc.focus.moved(Some("s1"));
        assert_eq!(mc.state().unwrap()["focus"]["holder"], "mc");
    }

    /// The key list is open while it shows every binding.
    #[test]
    fn the_key_list_is_open_while_it_shows_every_binding() {
        let (mut mc, _model) = testing::mc("open", [1280, 800]);
        mc.keys = crate::keys::List::All;
        let open = json!([{"id": "mc-keys", "kind": "sheet", "modal": true}]);
        assert_eq!(mc.state().unwrap()["open"], open);
        mc.keys = crate::keys::List::Six;
        assert_eq!(mc.state().unwrap()["open"], json!([]));
    }

    /// `helper` is the word of the kit machine: `no_helper` until a helper
    /// exists, its state while it does, `no_helper` again when voice goes.
    #[test]
    fn the_helper_follows_voice() {
        let (mut x, _model) = testing::mc("state-helper", [1280, 800]);
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
        let said = |event: &str, data| chora_scene::Event::Emitted {
            event: event.into(),
            client: "voice".into(),
            data,
        };
        x.event(said("voice.state", json!({"state": "idle"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
        x.event(said(
            "voice.state",
            json!({"name": "atlas", "state": "working"}),
        ))
        .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "working");
        x.event(said("voice.remove", json!({"name": "atlas"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
        let gone = json!({"event": "client.disconnected", "name": "voice"});
        x.event(serde_json::from_value(gone).unwrap()).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
    }

    /// I3 (known red net2-a2): with no window in the scene mc gives the
    /// keyboard to the node of its page; it holds the focus then, without
    /// a surface, until a window of another look takes it.
    #[test]
    fn mc_holds_the_focus_of_its_page_node() {
        let (mut mc, _model) = testing::mc("page-focus", [1280, 800]);
        let commands = mc.keyboard_to("agenda", None);
        let page = |c: &chora_scene::Command| {
            matches!(c, chora_scene::Command::FocusNode { node: Some(_) })
        };
        assert!(commands.iter().any(page), "{commands:?}");
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": "mc"}));
        mc.event(chora_scene::Event::FocusChanged {
            surface: Some("other".into()),
        })
        .unwrap();
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// A look that hears voice connect asks voice for its state, and it
    /// hears every event of the helper machine, so that it says what the
    /// presenter says (rw-4).
    #[test]
    fn the_helper_is_seeded_from_voice_and_hears_every_event() {
        assert_eq!(
            chora_kit::helper::unheard(&crate::EVENTS),
            Vec::<&str>::new()
        );
        let core = chora_kit::testing::FakeCore::start();
        let mut x = testing::mc_on(&core, "state-seed", [1280, 800]);
        core.answer_call(
            "voice",
            "state",
            json!({"state": "listening", "agents": [{"name": "coder", "state": "idle"}]}),
        );
        let connected = chora_scene::Event::ClientConnected {
            name: "voice".into(),
        };
        x.event(connected).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "listening");
        let said = chora_scene::Event::Emitted {
            event: "voice.say".into(),
            client: "voice".into(),
            data: json!({"state": "speaking", "text": "Hi"}),
        };
        x.event(said).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "answered");
    }
}
