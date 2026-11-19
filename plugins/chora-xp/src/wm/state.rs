//! `state.get` (design 3.1): what chora-xp shows and holds, read from its
//! model until xp uses `kit::wm`; the port deletes this adapter. Only
//! reads, besides `core_focus`. Without a focused window xp holds the
//! focus with its own nodes (the icons, a menu, a dialog) until a window
//! of another window manager takes it: a node that takes the keyboard
//! sends no `focus.changed`. Desktops count from 1, as the pager shows them. `views` and
//! `texts` stay out: xp shows no window of another look, and its drawn
//! text runs come with NET3.

use serde_json::{Value, json};

use super::{NAME, Wm};

impl Wm {
    /// The client windows bottom first, the desktops, the focus, the open
    /// menus and dialogs, and the effects that still run.
    pub(super) fn state_value(&self) -> Value {
        let windows: Vec<Value> = (self.windows.iter().enumerate())
            .map(|(z, w)| {
                json!({
                    "surface": w.surface,
                    "title": w.title,
                    "app_id": w.app_id,
                    "output": self.output_of(&w.surface),
                    "desks": [w.desktop + 1],
                    "rect": w.rect(),
                    "shown": w.shown && !w.minimized && w.desktop == self.desktop,
                    "minimized": w.minimized,
                    "maximized": w.maximized(),
                    "fullscreen": false,
                    "floating": self.floats.floats(&w.surface),
                    "z": z,
                })
            })
            .collect();
        let desks: Vec<Value> = (self.bars.iter())
            .map(|b| json!({"output": b.output, "shown": [self.desktop + 1]}))
            .collect();
        let focus = match self.focused.as_deref().filter(|s| self.index(s).is_some()) {
            Some(surface) => json!({"holder": NAME, "surface": surface}),
            None if !self.focus_away() => json!({"holder": NAME}),
            None => json!({"holder": null}),
        };
        json!({"windows": windows, "desks": desks, "focus": focus,
            "open": self.open_value(), "busy": self.busy_value(),
            "helper": self.voiced.word()})
    }

    /// A window of another window manager took the focus last.
    fn focus_away(&self) -> bool {
        (self.core_focus.as_deref()).is_some_and(|s| self.index(s).is_none())
    }

    /// The menus and dialogs that show, in the order Escape shuts them.
    fn open_value(&self) -> Vec<Value> {
        let entry =
            |id: &str, kind: &str, modal: bool| json!({"id": id, "kind": kind, "modal": modal});
        let mut open = Vec::new();
        if self.switcher.is_some() {
            open.push(entry("xp-switcher", "switcher", false));
        }
        if self.dialog.is_some() {
            open.push(entry("xp-session", "dialog", true));
        }
        if self.context.is_some() {
            let mut menu = entry("xp-ctx", "menu", false);
            if let Some(surface) = &self.sys_window {
                menu["for"] = json!(surface);
            }
            open.push(menu);
        }
        if self.menu.is_open() {
            open.push(entry("xp-menu", "menu", false));
        }
        for panel in self.panels.iter().rev() {
            open.push(entry(&panel.window_ref().group(), "dialog", false));
        }
        open
    }

    /// What still moves: quiet once this is empty.
    fn busy_value(&self) -> Vec<&'static str> {
        let running = [
            (self.effects.driver.animating(), "effects"),
            (self.effects.turn.is_some(), "cube"),
            (self.fading(), "fade"),
            (self.sliding(), "slide"),
            (self.resize_due(), "resize"),
        ];
        running
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::runtime::Plugin as _;
    use serde_json::json;

    use super::super::tests::wm_on_fake_core;
    use crate::window::Window;

    /// A window with a size on a desktop.
    fn window(surface: &str, desktop: usize) -> Window {
        let mut w = Window::new(surface, Some("notepad".into()), Some(surface));
        w.size = Some([200, 100]);
        w.shown = true;
        w.desktop = desktop;
        w
    }

    /// A fresh xp shows no window, holds the focus with its desktop and
    /// is quiet.
    #[test]
    fn a_fresh_xp_has_an_empty_state() {
        let (wm, _methods) = wm_on_fake_core("state-empty");
        let state = wm.state().expect("xp has a state");
        let expected = json!({"windows": [], "focus": {"holder": "xp"}, "open": [],
            "busy": [], "helper": "none", "desks": [{"output": "X", "shown": [1]}]});
        assert_eq!(state, expected);
    }

    /// The fields of a window that a test reads.
    fn read(state: &serde_json::Value, i: usize) -> serde_json::Value {
        let w = &state["windows"][i];
        json!([
            w["surface"],
            w["output"],
            w["z"],
            w["shown"],
            w["desks"],
            w["minimized"]
        ])
    }

    /// The windows come bottom first with their desktop, and only those
    /// of the desktop that shows show.
    #[test]
    fn the_state_lists_the_windows_bottom_first() {
        let (mut wm, _methods) = wm_on_fake_core("state-windows");
        wm.windows.push(window("s1", 0));
        wm.windows.push(window("s2", 1));
        let state = wm.state().unwrap();
        assert_eq!(read(&state, 0), json!(["s1", "X", 0, true, [1], false]));
        assert_eq!(read(&state, 1), json!(["s2", "X", 1, false, [2], false]));
        assert_eq!(state["windows"][0]["app_id"], "notepad");
        wm.windows[0].minimized = true;
        let state = wm.state().unwrap();
        assert_eq!(read(&state, 0), json!(["s1", "X", 0, false, [1], true]));
    }

    /// xp holds the focus on one of its windows, or with its own nodes
    /// (the icons, a menu, a dialog) until a window of another window
    /// manager takes it.
    #[test]
    fn xp_holds_the_focus_until_another_window_takes_it() {
        let (mut wm, _methods) = wm_on_fake_core("state-focus");
        assert_eq!(wm.state().unwrap()["focus"], json!({"holder": "xp"}));
        wm.windows.push(window("s1", 0));
        wm.event(focus("s1")).unwrap();
        let focus_s1 = json!({"holder": "xp", "surface": "s1"});
        assert_eq!(wm.state().unwrap()["focus"], focus_s1);
        wm.event(focus("other")).unwrap();
        assert_eq!(wm.state().unwrap()["focus"], json!({"holder": null}));
        wm.event(focus("s1")).unwrap();
        assert_eq!(wm.state().unwrap()["focus"], focus_s1);
    }

    /// The focused window of xp goes: its icons have the keyboard again,
    /// so xp holds the focus, as after a give it does not.
    #[test]
    fn xp_holds_the_focus_after_its_focused_window_goes() {
        let (mut wm, _methods) = wm_on_fake_core("state-gone");
        wm.windows.push(window("s1", 0));
        wm.event(focus("s1")).unwrap();
        let gone = json!({"event": "surface.destroyed", "surface": "s1"});
        wm.event(serde_json::from_value(gone).unwrap()).unwrap();
        assert_eq!(wm.state().unwrap()["focus"], json!({"holder": "xp"}));
    }

    /// A `focus.changed` of the core.
    fn focus(surface: &str) -> chora_scene::Event {
        serde_json::from_value(json!({"event": "focus.changed", "surface": surface})).unwrap()
    }

    /// The start menu is open after `start_menu {open: true}` and shut
    /// after `{open: false}`.
    #[test]
    fn the_start_menu_is_open_in_the_state() {
        let (mut wm, _methods) = wm_on_fake_core("state-menu");
        wm.method("start_menu", json!({"open": true})).unwrap();
        let open = &wm.state().unwrap()["open"];
        assert_eq!(
            *open,
            json!([{"id": "xp-menu", "kind": "menu", "modal": false}])
        );
        wm.method("start_menu", json!({"open": false})).unwrap();
        assert_eq!(wm.state().unwrap()["open"], json!([]));
    }

    /// `texts` holds the runs of text that xp painted into the image
    /// nodes it shows, each with its node, line, rect and font size.
    #[test]
    fn the_texts_are_the_runs_xp_painted() {
        let (mut wm, _methods) = wm_on_fake_core("state-texts");
        wm.method("start_menu", json!({"open": true})).unwrap();
        let state = wm.state().unwrap();
        let texts = state["texts"].as_array().expect("texts");
        assert!(
            texts
                .iter()
                .any(|t| t["node"].as_str().is_some_and(|n| n.starts_with("xp-menu"))),
            "the open Start menu has runs: {texts:?}"
        );
        for run in texts {
            assert!(run["text"].as_str().is_some_and(|t| !t.is_empty()), "{run}");
            assert_eq!(run["rect"].as_array().map(Vec::len), Some(4), "{run}");
            assert!(run["px"].as_u64().is_some_and(|px| px > 0), "{run}");
        }
    }

    /// `helper` is `none` until voice reports, follows its latest
    /// `voice.state`, and is `none` again when voice goes.
    #[test]
    fn the_helper_follows_voice() {
        let (mut x, _methods) = wm_on_fake_core("state-helper");
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
}
