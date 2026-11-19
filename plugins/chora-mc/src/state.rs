//! `state.get`: what mc shows and holds, in the shape of `choractl state`
//! (design 3.1). The fields that only the core knows (title, app id,
//! rectangle, stacking) and those of looks that mc has no concept of
//! (desks, views, texts) are left out, as is `open` until mc fills it:
//! a field that is absent is not read as empty.

use serde_json::{Value, json};

use crate::{Mc, NAME};

impl Mc {
    /// The windows of the scenes and the docks, the focus and what still
    /// moves. mc holds the focus only on one of its windows; `open` stays
    /// out until mc fills it.
    pub(crate) fn state_value(&self) -> Value {
        let output = self.output.as_ref().map(|o| o.name.as_str());
        let shown = |scene: &str| self.current.as_deref() == Some(scene);
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
        windows.sort_by_key(|w| w["surface"].as_str().map(str::to_owned));
        let mut busy: Vec<&str> = self.writing.keys().map(String::as_str).collect();
        busy.sort_unstable();
        let mut busy: Vec<String> = busy.into_iter().map(|p| format!("write:{p}")).collect();
        if self.typing.is_some() {
            busy.push("typing".to_owned());
        }
        let focus = match self
            .core_focus
            .as_ref()
            .filter(|s| windows.iter().any(|w| w["surface"] == s.as_str()))
        {
            Some(surface) => json!({"holder": NAME, "surface": surface}),
            None => json!({"holder": null}),
        };
        json!({"windows": windows, "focus": focus, "busy": busy,
            "helper": self.voiced_word.word()})
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
        mc.core_focus = Some("s1".into());
        let state = mc.state().unwrap();
        assert_eq!(state["windows"][0]["surface"], "s1");
        assert_eq!(state["windows"][0]["scene"], "agenda");
        assert_eq!(state["windows"][0]["output"], "test");
        assert_eq!(state["focus"]["surface"], "s1");
        mc.core_focus = Some("other".into());
        assert!(mc.state().unwrap()["focus"].get("surface").is_none());
    }

    /// mc holds the focus only while the core focus is on a window of
    /// mc; else no look of mc's holds it (net1 review). `open` stays out
    /// until mc fills it, so that I4 does not read an empty list as true.
    #[test]
    fn the_focus_holder_is_mc_only_on_a_window_of_mc() {
        let (mut mc, _model) = testing::mc("holder", [1280, 800]);
        let state = mc.state().unwrap();
        assert_eq!(state["focus"]["holder"], serde_json::Value::Null);
        assert!(state.get("open").is_none());
        mc.windows.insert("agenda".into(), "s1".into());
        mc.core_focus = Some("other".into());
        assert_eq!(
            mc.state().unwrap()["focus"]["holder"],
            serde_json::Value::Null
        );
        mc.core_focus = Some("s1".into());
        assert_eq!(mc.state().unwrap()["focus"]["holder"], "mc");
    }

    /// `helper` is `none` until voice reports, follows its latest
    /// `voice.state`, and is `none` again when voice goes.
    #[test]
    fn the_helper_follows_voice() {
        let (mut x, _model) = testing::mc("state-helper", [1280, 800]);
        assert_eq!(x.state().unwrap()["helper"], "none");
        let said = |event: &str, data| chora_scene::Event::Emitted {
            event: event.into(),
            client: "voice".into(),
            data,
        };
        x.event(said("voice.state", json!({"state": "idle"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "idle");
        x.event(said(
            "voice.state",
            json!({"name": "atlas", "state": "working"}),
        ))
        .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "working");
        x.event(said("voice.remove", json!({"name": "atlas"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "idle");
        let gone = json!({"event": "client.disconnected", "name": "voice"});
        x.event(serde_json::from_value(gone).unwrap()).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "none");
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
}
