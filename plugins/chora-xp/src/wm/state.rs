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
                held: false,
                z: Some(z),
            })
            .collect();
        let desks = (self.bars.iter())
            .map(|b| Desks {
                output: b.output.clone(),
                shown: vec![self.desk_of(b) + 1],
            })
            .collect();
        let focus = match (self.focused.as_deref())
            .filter(|s| !self.node_away() && self.index(s).is_some())
        {
            Some(surface) => Focus {
                holder: Some(NAME.into()),
                surface: Some(surface.into()),
            },
            None if self.own_node()
                || (!self.node_away() && !self.focus_away() && !self.cube_off()) =>
            {
                Focus {
                    holder: Some(NAME.into()),
                    surface: None,
                }
            }
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

    /// A node of another client has the keyboard.
    pub(super) fn node_away(&self) -> bool {
        (self.key_node.as_deref()).is_some_and(|owner| owner != NAME)
    }

    /// A node of xp, such as its Start menu, has the keyboard.
    fn own_node(&self) -> bool {
        self.key_node.as_deref() == Some(NAME)
    }

    /// The commands for a `focus.node` of the core. xp sent the keys of its
    /// desktop icons from a view of the focus older than the core's when
    /// the window that left xp already has the focus: the core then keeps
    /// the icons as the holder of the keyboard behind that window, and the
    /// next new window goes to xp, not to the manager of the window.
    pub(super) fn node_focused(
        &mut self,
        node: Option<&str>,
        owner: Option<&str>,
    ) -> Vec<chora_scene::Command> {
        self.key_node = owner.map(String::from);
        let desktop = node == Some(crate::desktop_icons::GROUP) && owner == Some(NAME);
        if desktop && self.focus_away() {
            vec![chora_scene::Command::FocusNode { node: None }]
        } else {
            Vec::new()
        }
    }

    /// A window of another window manager took the focus last.
    pub(super) fn focus_away(&self) -> bool {
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
    use chora_kit::{
        runtime::Plugin as _,
        wm::open::{Kind, Surface},
    };
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
            "busy": [], "helper": "no_helper", "desks": [{"output": "X", "shown": [1]}]});
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

    /// The core focuses no surface, as when the construct of a hosted cube
    /// takes the keyboard with its own node: xp names no window and draws
    /// none active.
    #[test]
    fn xp_names_no_window_when_the_core_focuses_none() {
        let (mut wm, _methods) = wm_on_fake_core("state-none");
        add(&mut wm, "s1", 0);
        wm.event(focus("s1")).unwrap();
        let none = json!({"event": "focus.changed", "surface": null});
        wm.event(serde_json::from_value(none).unwrap()).unwrap();
        assert_eq!(wm.state().unwrap()["focus"], json!({"holder": "xp"}));
        assert_eq!(wm.focused, None, "the window of s1 looks inactive");
        wm.event(focus("s1")).unwrap();
        let focus_s1 = json!({"holder": "xp", "surface": "s1"});
        assert_eq!(wm.state().unwrap()["focus"], focus_s1);
    }

    /// rw-2: the first click on the desktop while a window of another look
    /// has the focus: xp sent its icons no keyboard then (the window still
    /// had it), so the `focus.changed` that follows gives it to them.
    #[test]
    fn the_desktop_icons_take_the_keys_when_the_focus_leaves_a_foreign_window() {
        let (mut wm, _methods) = wm_on_fake_core("state-icons-keys");
        wm.event(focus("t1")).unwrap();
        assert!(wm.focus_away());
        let none = wm.focus_moved(None);
        let icons = chora_scene::Command::FocusNode {
            node: Some(crate::desktop_icons::GROUP.into()),
        };
        assert!(none.contains(&icons), "the icons get no keys: {none:?}");
    }

    /// A node of another client that has the keys keeps them: the focus
    /// that leaves a foreign window does not give the icons the keyboard.
    #[test]
    fn the_desktop_icons_leave_the_keys_of_a_node_of_another_client() {
        let (mut wm, _methods) = wm_on_fake_core("state-icons-node");
        wm.event(focus("t1")).unwrap();
        let taken = json!({"event": "focus.node", "node": "keys", "owner": "tiling"});
        wm.event(serde_json::from_value(taken).unwrap()).unwrap();
        let sent = wm.focus_moved(None);
        assert!(
            sent.iter()
                .all(|c| !matches!(c, chora_scene::Command::FocusNode { .. })),
            "the keys of tiling's node are taken: {sent:?}"
        );
    }

    /// HT-rx-1: the Start menu of xp has the keys when the foreign window
    /// that had the focus goes: the icons do not take them from the menu.
    #[test]
    fn the_desktop_icons_leave_the_keys_of_the_start_menu_of_xp() {
        let (mut wm, _methods) = wm_on_fake_core("state-icons-menu");
        wm.event(focus("t1")).unwrap();
        let taken = json!({"event": "focus.node", "node": "xp-menu", "owner": "xp"});
        wm.event(serde_json::from_value(taken).unwrap()).unwrap();
        assert!(wm.own_node(), "the menu of xp has the keyboard");
        let sent = wm.focus_moved(None);
        assert!(
            sent.iter()
                .all(|c| !matches!(c, chora_scene::Command::FocusNode { .. })),
            "the icons took the keys from the open Start menu: {sent:?}"
        );
    }

    /// The core says that the icons of xp hold the keyboard while a window
    /// of another look has the focus: xp sent them the keys from a view of
    /// the focus older than the window's, so it takes them back. The icons
    /// that hold them with no window of another look focused stay.
    #[test]
    fn the_desktop_icons_give_back_keys_that_a_foreign_window_holds_by_focus() {
        let (mut wm, _methods) = wm_on_fake_core("state-icons-stale");
        let icons = Some(crate::desktop_icons::GROUP);
        assert!(
            wm.node_focused(icons, Some(super::NAME)).is_empty(),
            "no window is away"
        );
        wm.event(focus("t1")).unwrap();
        let sent = wm.node_focused(icons, Some(super::NAME));
        assert_eq!(sent, vec![chora_scene::Command::FocusNode { node: None }]);
        assert_eq!(wm.node_focused(icons, Some("tiling")), vec![]);
        assert_eq!(wm.node_focused(Some("menu"), Some(super::NAME)), vec![]);
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
        assert_eq!(*open, json!([Surface::new("xp-menu", Kind::Menu).entry()]));
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
        add(&mut wm, "s1", 0);
        wm.method("start_menu", json!({"open": true})).unwrap();
        assert_eq!(open_ids(&wm), ["xp-menu"]);
        wm.give("s1", "tiling")
            .expect("the fake core takes the give");
        assert_eq!(open_ids(&wm), Vec::<String>::new(), "open after the give");
    }

    /// sd-h2: a window that comes to xp, by a give or a drag, shuts the
    /// Start menu, which covered it.
    #[test]
    fn a_window_that_comes_shuts_the_start_menu() {
        let (mut wm, _methods) = wm_on_fake_core("state-adopt-menu");
        wm.method("start_menu", json!({"open": true})).unwrap();
        assert_eq!(open_ids(&wm), ["xp-menu"]);
        assert!(
            wm.adopt("s1", Some("foot".into()), None).unwrap(),
            "xp takes it"
        );
        assert_eq!(
            open_ids(&wm),
            Vec::<String>::new(),
            "open after the window came"
        );
    }

    /// The same for the drag that carries the window out (`pull_out`).
    #[test]
    fn a_give_with_the_drag_shuts_the_start_menu() {
        let (mut wm, _methods) = wm_on_fake_core("state-drag-menu");
        add(&mut wm, "s1", 0);
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

    /// `helper` is the word of the kit machine: `no_helper` until a helper
    /// exists, its state while it does, `no_helper` again when voice goes.
    #[test]
    fn the_helper_follows_voice() {
        let (mut x, _methods) = wm_on_fake_core("state-helper");
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

    /// A look that hears voice connect asks voice for its state, and it
    /// hears every event of the helper machine, so that it says what the
    /// presenter says (rw-4).
    #[test]
    fn the_helper_is_seeded_from_voice_and_hears_every_event() {
        assert_eq!(
            chora_kit::helper::unheard(&super::super::EVENTS),
            Vec::<&str>::new()
        );
        let core = chora_kit::testing::FakeCore::start();
        let mut x = super::super::tests::wm_on_kit_core(&core, "state-seed");
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
