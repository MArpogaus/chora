//! `state.get` (design 3.1): what chora-xp shows and holds, in the shape
//! of `kit::wm::state`, read from its model and its book. Only reads,
//! besides `core_focus`. Without a focused window xp holds the
//! focus with its own nodes (the icons, a menu, a dialog) until a window
//! of another window manager takes it: a node that takes the keyboard
//! sends no `focus.changed`. Desktops count from 1, as the pager shows them. `views`
//! stays out: xp shows no window of another look. `texts` are the runs
//! of text that chora-draw reports for the image nodes of xp.

use chora_kit::wm::{
    Origin,
    state::{Desks, Focus, Open, State, Window},
};
use serde_json::{Map, Value, json};

use super::{NAME, Wm};

impl Wm {
    /// The client windows bottom first, the desktops, the focus, the open
    /// menus and dialogs, and the effects that still run.
    pub(super) fn state_value(&self) -> Value {
        let windows = (self.windows.iter().enumerate())
            .map(|(z, w)| Window {
                surface: w.surface.clone(),
                origin: self.book.origin(&w.surface).map(Origin::word),
                title: Some(w.title.clone()),
                app_id: w.app_id.clone(),
                output: self.output_of(&w.surface).unwrap_or_default(),
                desks: vec![w.desktop + 1],
                rect: Some(w.rect().map(f64::from)),
                shown: w.shown && !w.minimized && w.desktop == self.desktop,
                minimized: w.minimized,
                maximized: w.maximized(),
                fullscreen: false,
                floating: self.floats.floats(&w.surface),
                z: Some(z),
            })
            .collect();
        let desks = (self.bars.iter())
            .map(|b| Desks {
                output: b.output.clone(),
                shown: vec![self.desktop + 1],
            })
            .collect();
        let focus = match self.focused.as_deref().filter(|s| self.index(s).is_some()) {
            Some(surface) => Focus {
                holder: Some(NAME.into()),
                surface: Some(surface.into()),
            },
            None if !self.focus_away() => Focus {
                holder: Some(NAME.into()),
                surface: None,
            },
            None => Focus::default(),
        };
        let mut more = Map::new();
        more.insert("helper".into(), json!(self.voiced.word()));
        more.insert("texts".into(), json!(self.theme.texts()));
        let mut state = State {
            windows,
            desks,
            focus,
            open: self.open_value(),
            busy: self.busy_value(),
            texts: self.texts_value(),
            more,
        }
        .value();
        self.name_the_window_of_the_system_menu(&mut state);
        state
    }

    /// The system menu of a window tells which window it is for.
    fn name_the_window_of_the_system_menu(&self, state: &mut Value) {
        let Some(surface) = &self.sys_window else {
            return;
        };
        let menu = (state["open"].as_array_mut())
            .and_then(|open| open.iter_mut().find(|o| o["id"] == "xp-ctx"));
        if let Some(menu) = menu {
            menu["for"] = json!(surface);
        }
    }

    /// A window of another window manager took the focus last.
    fn focus_away(&self) -> bool {
        (self.core_focus.as_deref()).is_some_and(|s| self.index(s).is_none())
    }

    /// The runs of text that chora-draw reports for the image nodes of xp.
    fn texts_value(&self) -> Vec<Value> {
        (self.theme.texts().iter())
            .filter_map(|run| serde_json::to_value(run).ok())
            .collect()
    }

    /// The menus and dialogs that show, in the order Escape shuts them.
    fn open_value(&self) -> Vec<Open> {
        self.open.state()
    }

    /// What still moves: quiet once this is empty.
    fn busy_value(&self) -> Vec<String> {
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
            .map(|(_, name)| name.to_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::runtime::Plugin as _;
    use serde_json::json;

    use super::super::{Wm, tests::wm_on_fake_core};
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
        assert_eq!(
            state.get("texts").map(serde_json::Value::is_array),
            Some(true)
        );
        let mut state = state;
        state.as_object_mut().unwrap().remove("texts");
        assert_eq!(state, expected);
    }

    /// A window that xp holds, in its list and in its book.
    fn add(wm: &mut Wm, surface: &str, desktop: usize) {
        wm.windows.push(window(surface, desktop));
        wm.book.keep(surface);
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
        add(&mut wm, "s1", 0);
        add(&mut wm, "s2", 1);
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
        add(&mut wm, "s1", 0);
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
        add(&mut wm, "s1", 0);
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

    /// The ids of what is open, in the order Escape shuts them.
    fn open_ids(wm: &super::Wm) -> Vec<String> {
        let open = wm.state().unwrap()["open"].clone();
        let ids = open.as_array().unwrap().iter();
        ids.map(|o| o["id"].as_str().unwrap().to_owned()).collect()
    }

    /// I4 (sd-h2): a window that goes to another window manager, by the
    /// give keys or by a drag out of the face, shuts the Start menu and
    /// the context menu; they stayed open and covered the window.
    #[test]
    fn a_give_shuts_the_start_menu() {
        let (mut wm, _methods) = wm_on_fake_core("state-give-menu");
        wm.windows.push(window("s1", 0));
        wm.method("start_menu", json!({"open": true})).unwrap();
        assert_eq!(open_ids(&wm), ["xp-menu"]);
        wm.give("s1", "tiling")
            .expect("the fake core takes the give");
        assert_eq!(open_ids(&wm), Vec::<String>::new(), "open after the give");
    }

    /// The same for the drag that carries the window out (`pull_out`).
    #[test]
    fn a_give_with_the_drag_shuts_the_start_menu() {
        let (mut wm, _methods) = wm_on_fake_core("state-drag-menu");
        wm.windows.push(window("s1", 0));
        wm.method("start_menu", json!({"open": true})).unwrap();
        let grab = chora_scene::GivenGrab {
            point: [10.0, 10.0],
            velocity: [0.0; 2],
            wobble: None,
        };
        wm.give_dragged("s1", "tiling", Some(grab))
            .expect("the give");
        assert_eq!(open_ids(&wm), Vec::<String>::new(), "open after the drag");
    }

    /// A press on a title bar, which starts a drag, shuts the menu too.
    #[test]
    fn a_press_on_a_title_bar_shuts_the_start_menu() {
        let (mut wm, _methods) = wm_on_fake_core("state-title-menu");
        wm.windows.push(window("s1", 0));
        wm.method("start_menu", json!({"open": true})).unwrap();
        let commands = wm.pressed("xp-title-s1", 272, [100.0, 100.0]);
        wm.apply(commands).unwrap();
        assert_eq!(open_ids(&wm), Vec::<String>::new(), "open after the press");
    }

    /// Escape shuts what opened last: a Help panel, then the Start menu on
    /// top of it. The first Escape shuts the menu, the second the panel,
    /// the order `state.open` lists them in.
    #[test]
    fn escape_shuts_the_newest_surface_first() {
        let (mut wm, _methods) = wm_on_fake_core("state-escape-order");
        let commands = wm.open_help(0);
        wm.apply(commands).unwrap();
        wm.method("start_menu", json!({"open": true})).unwrap();
        let ids = open_ids(&wm);
        assert_eq!(ids.len(), 2, "open: {ids:?}");
        assert_eq!(ids[0], "xp-menu", "the menu is the newest: {ids:?}");
        let panel = ids[1].clone();
        let commands = wm.node_key("Escape");
        wm.apply(commands).unwrap();
        assert_eq!(
            open_ids(&wm),
            std::slice::from_ref(&panel),
            "the first Escape"
        );
        let commands = wm.node_key("Escape");
        wm.apply(commands).unwrap();
        assert_eq!(open_ids(&wm), Vec::<String>::new(), "the second Escape");
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
