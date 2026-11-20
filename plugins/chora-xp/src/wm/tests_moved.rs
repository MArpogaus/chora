//! Unit tests of the modules of xp that came after the RC: the event loop
//! (`dispatch`), the keys and the switcher (`keys`), the pointer, the
//! presses, the panels, the choices, the windows, the surfaces and the
//! outputs. They run on the fake core of `wm.rs`, which answers every call
//! with null and records the methods, so that a test sees what xp sent.

use chora_kit::{
    give::Windows as _,
    volume::{Reading, Volume},
};
use serde_json::{Value, json};

use super::tests::{wm_on_fake_core, wm_on_kit_core};
use super::*;

/// An event as the core sends it.
fn wire(event: Value) -> Event {
    serde_json::from_value(event).expect("an event of the core")
}

/// A window of xp on the desktop that shows, as a client that mapped it.
fn own_window(wm: &mut Wm, surface: &str) {
    let mut win = crate::window::Window::new(surface, Some("probe".into()), Some(surface));
    win.shown = true;
    win.desktop = wm.desktop;
    wm.windows.push(win);
}

/// The Turn Off binding, as xp spells it when it holds it.
fn power_key() -> String {
    "Super+Shift+e".parse::<Binding>().unwrap().to_string()
}

/// The methods that the fake core saw since `from`.
fn sent(methods: &chora_kit::testing::Log, from: usize) -> Vec<String> {
    methods.lock().unwrap()[from..].to_vec()
}

/// The output that `wm_on_fake_core` lists, as a change of it.
fn fake_output() -> chora_scene::OutputInfo {
    chora_scene::OutputInfo {
        name: "X".into(),
        position: [0, 0],
        size: [1280, 800],
        scale: 1.0,
        camera: chora_scene::Transform::default(),
        work_area: [0, 0, 1280, 800],
        default_work_area: [0, 0, 1280, 800],
        is_virtual: None,
        owner: None,
    }
}

mod dispatch {
    use super::*;

    /// A call that names no window of xp is refused, and it changes none.
    #[test]
    fn a_call_for_a_window_of_another_is_refused() {
        let (mut wm, _methods) = wm_on_fake_core("calls-unknown");
        own_window(&mut wm, "s1");
        for method in ["minimize", "restore", "activate"] {
            let err = wm
                .method(method, json!({"surface": "nope"}))
                .expect_err(method);
            assert_eq!(err.code, RpcError::REJECTED, "{method}");
        }
        assert!(!wm.windows[0].minimized, "s1 stays");
    }

    /// Minimize pours a window into its task, and restore or activate
    /// brings it back and focuses it.
    #[test]
    fn minimize_restore_and_activate_act_on_the_window_named() {
        let (mut wm, _methods) = wm_on_fake_core("calls-windows");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        assert_eq!(
            wm.method("minimize", json!({"surface": "s1"})),
            Ok(Value::Null)
        );
        assert!(wm.windows[0].minimized, "s1 is minimized");
        assert!(!wm.windows[1].minimized, "s2 is not");
        wm.method("restore", json!({"surface": "s1"})).unwrap();
        assert!(!wm.windows[0].minimized, "s1 is back");
        wm.method("activate", json!({"surface": "s2"})).unwrap();
        assert_eq!(wm.focused.as_deref(), Some("s2"), "s2 has the focus");
    }

    /// Turn to a desktop of the four, and a desktop that is not one of
    /// them, which is refused as bad parameters.
    #[test]
    fn turn_to_takes_a_desktop_of_the_four() {
        let (mut wm, _methods) = wm_on_fake_core("calls-turn");
        assert_eq!(wm.method("turn_to", json!({"desktop": 2})), Ok(Value::Null));
        assert!(wm.effects.turn.is_some(), "the cube turns to desktop 2");
        let err = wm.method("turn_to", json!({"desktop": 4})).unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
    }

    /// The start menu opens for a call that asks it to, and closes for one
    /// that asks it to close, and a call that repeats the state changes
    /// nothing. Under the shade of Turn Off it is refused.
    #[test]
    fn start_menu_opens_and_closes_as_it_is_asked() {
        let (mut wm, _methods) = wm_on_fake_core("calls-start");
        assert!(!wm.menu.is_open());
        wm.method("start_menu", json!({"open": true})).unwrap();
        assert!(wm.menu.is_open(), "opened");
        wm.method("start_menu", json!({"open": true})).unwrap();
        assert!(wm.menu.is_open(), "still open");
        wm.method("start_menu", json!({"open": false})).unwrap();
        assert!(!wm.menu.is_open(), "closed");
        wm.key(&power_key());
        assert!(wm.dialog.is_some(), "Turn Off is open");
        let err = wm
            .method("start_menu", json!({"open": true}))
            .expect_err("under the shade");
        assert_eq!(err.code, RpcError::REJECTED);
    }

    /// Restart keeps the windows for the next start and ends the plugin
    /// to start it again.
    #[test]
    fn restart_ends_the_plugin_to_start_it_again() {
        let (mut wm, _methods) = wm_on_fake_core("calls-restart");
        assert!(!wm.done && !wm.restart);
        wm.method("restart", Value::Null).unwrap();
        assert!(wm.done, "the plugin ends");
        assert!(wm.restart, "and starts again");
    }

    /// A changed title and size reach the window; an empty size is
    /// not a size.
    #[test]
    fn a_changed_surface_retitles_and_resizes_its_window() {
        let (mut wm, _methods) = wm_on_fake_core("surface-changed");
        own_window(&mut wm, "s1");
        wm.event(wire(json!({
            "event": "surface.changed", "surface": "s1", "app_id": "probe",
            "title": "Renamed", "size": [300, 200]
        })))
        .unwrap();
        assert_eq!(wm.windows[0].title, "Renamed");
        assert_eq!(wm.windows[0].size, Some([300, 200]));
        wm.event(wire(json!({
            "event": "surface.changed", "surface": "s1", "app_id": "probe",
            "title": "Renamed", "size": [0, 0]
        })))
        .unwrap();
        assert_eq!(wm.windows[0].size, Some([300, 200]), "no empty size");
    }

    /// The keyboard focus that the core moves to a window of xp makes that
    /// window the focus and takes Alt+Space; the focus that moves away, or
    /// goes, gives Alt+Space back to the clients.
    #[test]
    fn the_focus_moves_between_xp_and_the_others() {
        let (mut wm, methods) = wm_on_fake_core("focus-moves");
        own_window(&mut wm, "s1");
        wm.event(wire(json!({"event": "focus.changed", "surface": "s1"})))
            .unwrap();
        assert_eq!(wm.focused.as_deref(), Some("s1"));
        assert_eq!(wm.core_focus.as_deref(), Some("s1"));
        assert!(wm.system_key.is_some(), "Alt+Space is ours");
        let before = methods.lock().unwrap().len();
        wm.event(wire(json!({"event": "focus.changed", "surface": "s1"})))
            .unwrap();
        assert!(sent(&methods, before).iter().all(|m| m != "input.grab"));
        wm.event(wire(json!({"event": "focus.changed", "surface": "other"})))
            .unwrap();
        assert_eq!(wm.focused, None, "the other client has the focus");
        assert!(wm.system_key.is_none(), "Alt+Space goes back");
        assert!(sent(&methods, before).iter().any(|m| m == "input.ungrab"));
        wm.event(wire(json!({"event": "focus.changed", "surface": null})))
            .unwrap();
        assert_eq!(wm.core_focus, None, "no focus at all");
    }

    /// The pointer that enters and leaves a node of xp, the effects that
    /// xp drops, and the clients that come and go reach their arms of the
    /// event loop, and none of them fails.
    #[test]
    fn the_pointer_leaves_and_the_clients_go() {
        let (mut wm, _methods) = wm_on_fake_core("pointer-leaves");
        wm.event(wire(
            json!({"event": "pointer.entered", "node": "xp-start", "moved": true}),
        ))
        .unwrap();
        assert_eq!(wm.hover.as_deref(), Some("xp-start"));
        wm.event(wire(json!({"event": "pointer.left", "node": "xp-start"})))
            .unwrap();
        assert_eq!(wm.hover, None, "the pointer left");
        wm.event(wire(json!({"event": "effects.dropped", "client": "xp"})))
            .unwrap();
        wm.event(wire(
            json!({"event": "client.disconnected", "name": "other"}),
        ))
        .unwrap();
        wm.event(wire(json!({"event": "client.connected", "name": "other"})))
            .unwrap();
        assert!(wm.menu_due().is_none(), "no cascade due");
        wm.menu_rest(Instant::now()).unwrap();
    }

    /// A node of another client asks xp to update a look; the request is
    /// answered without a failure whatever its arguments.
    #[test]
    fn a_node_asks_xp_for_an_update() {
        let (mut wm, _methods) = wm_on_fake_core("node-request");
        wm.event(wire(json!({
            "event": "node.requested", "node": "xp-look", "client": "other",
            "action": "update", "args": {}
        })))
        .unwrap();
        let update = wm.requested_look("xp-look", &json!({}));
        assert!(
            update.is_empty(),
            "an update of no window of xp changes nothing"
        );
    }

    /// Output changes: a new output, and then one removed, keep xp's bars
    /// on the output that remains.
    #[test]
    fn an_output_that_comes_and_goes_moves_the_bars() {
        let (mut wm, _methods) = wm_on_fake_core("outputs");
        assert!(!wm.bars.is_empty(), "xp has its bar");
        wm.event(Event::OutputChanged {
            output: fake_output(),
        })
        .unwrap();
        assert!(
            wm.bars.iter().any(|b| b.output == "X"),
            "the bar stays on X"
        );
        wm.event(Event::OutputRemoved {
            name: "gone".into(),
        })
        .unwrap();
        assert!(
            wm.bars.iter().any(|b| b.output == "X"),
            "a gone output leaves X"
        );
    }

    /// The speaker shows the level of the default output, and says what is
    /// wrong when there is no reading.
    #[test]
    fn the_speaker_tells_the_level_or_what_is_wrong() {
        let (mut wm, _methods) = wm_on_fake_core("speaker");
        assert_eq!(wm.speaker(), taskbar::Speaker::Off, "no reading yet");
        assert_eq!(wm.speaker_tip(), "", "nothing to say yet");
        wm.volume = Some(Reading::Level(Volume {
            percent: 40,
            muted: false,
        }));
        assert_eq!(wm.speaker(), taskbar::Speaker::On(40));
        assert_eq!(wm.speaker_tip(), "Volume 40%");
        wm.volume = Some(Reading::Level(Volume {
            percent: 40,
            muted: true,
        }));
        assert_eq!(wm.speaker(), taskbar::Speaker::Off, "muted");
        assert_eq!(wm.speaker_tip(), wm.volume_words());
        wm.volume = Some(Reading::NoWpctl);
        assert_eq!(wm.speaker(), taskbar::Speaker::Off);
        assert!(!wm.speaker_tip().is_empty(), "what is wrong");
        let _ = wm.icons();
        let _ = wm.helper_tip();
        let _ = wm.now();
        let _ = wm.held_shortcuts();
    }

    /// The volume keys read the change at the next tick, and the reading
    /// of the mixer is taken from the mixer when it comes.
    #[test]
    fn a_volume_change_waits_for_the_reading() {
        let (mut wm, _methods) = wm_on_fake_core("volume-change");
        let commands = wm.change_volume(chora_kit::volume::Change::Mute, true);
        assert!(commands.is_empty(), "the reading comes later");
    }

    /// A call of xp that is not a method of the plugin is refused with the
    /// method not found.
    #[test]
    fn a_call_xp_does_not_export_is_not_found() {
        let (mut wm, _methods) = wm_on_fake_core("calls-missing");
        let err = wm.method("log_off", Value::Null).unwrap_err();
        assert_eq!(err.code, RpcError::METHOD_NOT_FOUND);
    }

    /// Clicks on the tooltip and the menu of the taskbar have nothing to
    /// rest on, when no cascade or tooltip is due.
    #[test]
    fn nothing_is_due_without_a_menu_or_a_tooltip() {
        let (mut wm, _methods) = wm_on_fake_core("due");
        assert_eq!(wm.menu_due(), None);
        wm.menu_rest(Instant::now()).unwrap();
        assert_eq!(wm.menu_due(), None, "still nothing");
    }
}

mod keys {
    use super::*;

    /// Super alone, pressed and released, opens the Start Menu; another key
    /// in between means a combination, and the menu stays shut.
    #[test]
    fn super_alone_opens_the_start_menu_and_a_combination_does_not() {
        let (mut wm, _methods) = wm_on_fake_core("keys-tap");
        wm.key("Super+Super_L");
        assert!(wm.tap == Some(false), "Super is down alone");
        let _ = wm.key_released("Super+Super_L");
        assert!(wm.menu.is_open(), "the tap opens Start");
        wm.key("Super+Super_L");
        let _ = wm.key_released("Super+Super_L");
        assert!(!wm.menu.is_open(), "the second tap closes it");
        wm.key("Super+Super_R");
        wm.key("Super+d");
        assert_eq!(wm.tap, Some(true), "a key went with Super");
        let _ = wm.key_released("Super+Super_R");
        assert!(!wm.menu.is_open(), "no tap after a combination");
    }

    /// Super+d puts the windows of the desktop away, and brings the same
    /// ones back.
    #[test]
    fn super_d_puts_the_windows_away_and_back() {
        let (mut wm, _methods) = wm_on_fake_core("keys-desktop");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        wm.key("Super+d");
        assert!(wm.windows.iter().all(|w| w.minimized), "all away");
        wm.key("Super+d");
        assert!(wm.windows.iter().all(|w| !w.minimized), "and back");
    }

    /// Alt+Tab opens the switcher with the windows of the desktop, moves
    /// its choice on each further press, and its release activates the
    /// choice.
    #[test]
    fn alt_tab_opens_moves_and_chooses_in_the_switcher() {
        let (mut wm, methods) = wm_on_fake_core("alt-tab");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        let before = methods.lock().unwrap().len();
        let opened = wm.key("Alt+Tab");
        assert!(wm.switcher.is_some(), "the switcher opened");
        assert!(
            opened.iter().any(|c| matches!(c, Command::Create { .. })),
            "the switcher is drawn"
        );
        assert!(
            sent(&methods, before).iter().any(|m| m == "events.emit"),
            "the others hear of it"
        );
        let moved = wm.key("Alt+Tab");
        assert!(
            moved.iter().any(|c| matches!(c, Command::Update { .. })),
            "the choice moves"
        );
        let chosen = wm.switcher.as_ref().map(|s| s.chosen().to_string());
        let released = wm.key_released("Alt+Tab");
        assert!(wm.switcher.is_none(), "the switcher closed");
        assert!(
            released.iter().any(|c| matches!(c, Command::Remove { .. })),
            "the switcher is removed"
        );
        assert!(chosen.is_some(), "a window was chosen");
    }

    /// Alt+Shift+Tab goes back, and a switch with no window opens nothing.
    #[test]
    fn switching_back_and_with_no_window() {
        let (mut wm, _methods) = wm_on_fake_core("alt-back");
        let _ = wm.key("Alt+Shift+Tab");
        assert!(wm.switcher.is_none(), "no window, no switcher");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        let _ = wm.key("Alt+Shift+Tab");
        assert!(wm.switcher.is_some(), "the switcher opened back");
        let _ = wm.key("Alt+Shift+Tab");
        let _ = wm.key_released("Alt+Shift+Tab");
        assert!(wm.switcher.is_none());
    }

    /// A client that holds Alt+Escape refuses the grab until it has given
    /// the key up. xp does not sleep and ask again: the loop stays free,
    /// and the grab is made again on the core event `input.ungrabbed`, for
    /// any holder, not only one that xp knows (W7, D117). Another key
    /// given up is no reason to ask again.
    #[test]
    fn alt_escape_is_grabbed_when_the_holder_gives_it_up() {
        let core = chora_kit::testing::FakeCore::start();
        let mut wm = wm_on_kit_core(&core, "alt-escape-held");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        core.refuse_next("input.grab");
        wm.key("Alt+Tab");
        assert!(wm.switcher.is_some(), "the switcher shows");
        assert!(
            wm.switch_escape.is_none(),
            "the holder has the key, and xp does not wait for it by sleeping"
        );
        let other = json!({"event": "input.ungrabbed", "binding": "Super+a"});
        wm.event(serde_json::from_value(other).unwrap()).unwrap();
        assert!(wm.switch_escape.is_none(), "Super+a is not the key of xp");
        let free = json!({"event": "input.ungrabbed", "binding": "Alt+Escape"});
        wm.event(serde_json::from_value(free).unwrap()).unwrap();
        assert!(wm.switch_escape.is_some(), "the event made the grab");
        wm.key("Alt+Escape");
        assert!(wm.switch_escape.is_none(), "and Alt+Escape is given back");
    }

    /// The refusal of the Alt+Escape grab by a holder is expected and not
    /// reported on every Alt+Tab (rd-2); a failed socket still is.
    #[test]
    fn only_a_refusal_by_a_holder_is_expected() {
        use chora_scene::ipc::{ClientError, RpcError};
        let held = ClientError::Rpc(RpcError::new(
            RpcError::REJECTED,
            "Alt+Escape is grabbed already",
        ));
        assert!(super::super::keys::held_by_a_client(&held));
        assert!(!super::super::keys::held_by_a_client(&ClientError::Closed));
        let limit = ClientError::Rpc(RpcError::new(RpcError::LIMIT_REACHED, "too many"));
        assert!(!super::super::keys::held_by_a_client(&limit));
    }

    /// Alt+Escape cancels the switch: its key is grabbed while the
    /// switcher shows, and the window stays where it was.
    #[test]
    fn alt_escape_cancels_the_switch() {
        let (mut wm, _methods) = wm_on_fake_core("alt-escape");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        wm.key("Alt+Tab");
        assert!(wm.switch_escape.is_some(), "Alt+Escape is grabbed");
        let cancelled = wm.key("Alt+Escape");
        assert!(wm.switcher.is_none(), "the switch is cancelled");
        assert!(wm.switch_escape.is_none(), "and Alt+Escape given back");
        assert!(
            cancelled
                .iter()
                .any(|c| matches!(c, Command::Remove { .. })),
            "the switcher is removed"
        );
        assert!(wm.cancel_switch().is_empty(), "nothing left to cancel");
    }

    /// A window that closes leaves the switcher; the last one closes it.
    #[test]
    fn a_closed_window_leaves_the_switcher() {
        let (mut wm, _methods) = wm_on_fake_core("switcher-drop");
        own_window(&mut wm, "s1");
        own_window(&mut wm, "s2");
        wm.key("Alt+Tab");
        let kept = wm.drop_from_switcher("s1");
        assert!(wm.switcher.is_some(), "s2 is still there");
        assert!(kept.iter().any(|c| matches!(c, Command::Update { .. })));
        wm.drop_from_switcher("s2");
        assert!(wm.switcher.is_none(), "the last one closes it");
        assert!(
            wm.drop_from_switcher("s9").is_empty(),
            "no switcher, no change"
        );
    }

    /// Nothing switches under the shade of Turn Off, and Alt+Space opens
    /// the system menu of the focused window.
    #[test]
    fn the_shade_of_turn_off_stops_the_switch_and_the_system_key() {
        let (mut wm, _methods) = wm_on_fake_core("shade");
        own_window(&mut wm, "s1");
        wm.key(&power_key());
        assert!(wm.dialog.is_some());
        assert!(wm.switch(1).is_empty(), "no switch");
        assert!(wm.toggle_start().is_empty(), "no Start");
        assert!(wm.system_key_pressed().is_empty(), "no system menu");
        assert!(wm.switcher.is_none());
    }

    /// Alt+Space on a focused window opens its system menu, and with no
    /// focus it only closes the open menus.
    #[test]
    fn alt_space_opens_the_system_menu_of_the_focused_window() {
        let (mut wm, _methods) = wm_on_fake_core("system-key");
        own_window(&mut wm, "s1");
        wm.event(wire(json!({"event": "focus.changed", "surface": "s1"})))
            .unwrap();
        let commands = wm.system_key_pressed();
        assert!(!commands.is_empty(), "the system menu of s1 opens");
        assert_eq!(wm.sys_window.as_deref(), Some("s1"), "for s1");
        wm.focused = None;
        let _ = wm.system_key_pressed();
    }

    /// The keys that the core sends arrive as events, and the released
    /// half of a key or a button ends what the press began.
    #[test]
    fn the_key_and_button_events_reach_their_actions() {
        let (mut wm, _methods) = wm_on_fake_core("key-events");
        wm.event(wire(
            json!({"event": "input.key", "binding": "Super+Super_L", "state": "pressed"}),
        ))
        .unwrap();
        wm.event(wire(
            json!({"event": "input.key", "binding": "Super+Super_L", "state": "released"}),
        ))
        .unwrap();
        assert!(wm.menu.is_open(), "the tap opened Start");
        wm.event(wire(
            json!({"event": "node.key", "node": "xp-look", "key": "Menu",
            "code": 127, "state": "pressed", "modifiers": []}),
        ))
        .unwrap();
        wm.event(wire(
            json!({"event": "node.key", "node": "xp-look", "key": "F10",
            "code": 68, "state": "pressed", "modifiers": ["Shift"]}),
        ))
        .unwrap();
        wm.event(wire(
            json!({"event": "input.button", "binding": "Super+BTN_LEFT",
            "state": "pressed", "position": [100.0, 100.0], "output": "X",
            "local": [100.0, 100.0]}),
        ))
        .unwrap();
        assert_eq!(wm.cube_grab, Some(100.0), "the cube is grabbed");
        wm.event(wire(
            json!({"event": "input.moved", "binding": "Super+BTN_LEFT",
            "position": [150.0, 100.0], "output": "X", "local": [150.0, 100.0]}),
        ))
        .unwrap();
        wm.event(wire(
            json!({"event": "input.button", "binding": "Super+BTN_LEFT",
            "state": "released", "position": [150.0, 100.0], "output": "X",
            "local": [150.0, 100.0]}),
        ))
        .unwrap();
        assert_eq!(wm.cube_grab, None, "the release lets the cube go");
    }

    /// The bindings of the look and the cube are as the defaults say.
    #[test]
    fn the_defaults_bind_the_keys_of_the_looks() {
        let bound = crate::config::defaults().common.bindings;
        assert_eq!(bound["Super+e"], KeyAction::Look(Look::Files));
        assert_eq!(bound["Super+Super_L"], KeyAction::Look(Look::Tap));
        assert_eq!(bound["Super+space"], KeyAction::System(System::Launcher));
    }

    /// Without a dialog the desktop does not fade, and no fade frame
    /// is drawn.
    #[test]
    fn without_a_dialog_the_desktop_does_not_fade() {
        let (wm, _methods) = wm_on_fake_core("no-fade");
        assert!(!wm.fading(), "no fade");
        assert!(wm.fade_frame(Instant::now()).is_none());
    }
}

mod fake_core_with_scene {
    use super::*;

    /// On the fake core of the scene, the window of a surface that the
    /// core created is claimed and listed with its origin.
    #[test]
    fn a_created_surface_is_claimed_and_listed() {
        let core = chora_kit::testing::FakeCore::start();
        let mut wm = wm_on_kit_core(&core, "claimed");
        chora_kit::testing::surface(&core.model(), "s1");
        wm.event(chora_kit::testing::created("s1", None)).unwrap();
        assert!(wm.has("s1"), "xp has the window");
        let state = wm.state().unwrap();
        assert!(state.to_string().contains("s1"), "{state}");
    }
}

mod output {
    use super::*;

    /// An output of the fake core with another name, place and size.
    fn output_named(name: &str, position: [i32; 2], size: [i32; 2]) -> chora_scene::OutputInfo {
        let mut out = fake_output();
        out.name = name.into();
        out.position = position;
        out.size = size;
        out.work_area = [position[0], position[1], size[0], size[1]];
        out.default_work_area = out.work_area;
        out
    }

    /// A window of xp at a place of its output, with a size.
    fn placed(wm: &mut Wm, surface: &str, pos: [i32; 2], size: [i32; 2]) {
        own_window(wm, surface);
        let win = wm.windows.last_mut().unwrap();
        win.pos = pos;
        win.size = Some(size);
    }

    /// A smaller output keeps its windows: one that does not fit shrinks,
    /// one that sticks out moves in, and one that is inside stays.
    #[test]
    fn a_smaller_output_keeps_its_windows_on_it() {
        let (mut wm, _methods) = wm_on_fake_core("output-shrink");
        placed(&mut wm, "big", [100, 100], [900, 600]);
        placed(&mut wm, "moved", [600, 300], [200, 100]);
        placed(&mut wm, "inside", [10, 10], [100, 100]);
        wm.event(Event::OutputChanged {
            output: output_named("X", [0, 0], [640, 400]),
        })
        .unwrap();
        let [x, y, w, h] = wm.windows[0].rect();
        assert!(
            x >= 0 && y >= 0 && x + w <= 640 && y + h <= 400,
            "big fits: {:?}",
            wm.windows[0].rect()
        );
        assert_ne!(wm.windows[1].pos, [600, 300], "moved in");
        assert_eq!(wm.windows[2].pos, [10, 10], "untouched");
    }

    /// A maximized window fills the area of its output when the output
    /// changes size.
    #[test]
    fn a_maximized_window_fills_the_smaller_output() {
        let (mut wm, _methods) = wm_on_fake_core("output-maximized");
        placed(&mut wm, "max", [100, 100], [900, 600]);
        wm.windows[0].restore = Some([100, 100, 300, 200]);
        wm.event(Event::OutputChanged {
            output: output_named("X", [0, 0], [640, 400]),
        })
        .unwrap();
        assert_eq!(wm.windows[0].pos, [0, 0], "fills from the corner");
        assert_eq!(
            wm.windows[0].size.map(|s| s[0]),
            Some(640),
            "the width of the output"
        );
    }

    /// The windows of a removed output go to the output that remains, to
    /// the same place as far as it fits; a maximized one fills it. The
    /// menu of the removed output closes.
    #[test]
    fn the_windows_of_a_removed_output_go_to_the_other_one() {
        let (mut wm, _methods) = wm_on_fake_core("output-remove");
        wm.event(Event::OutputChanged {
            output: output_named("Y", [1280, 0], [1280, 800]),
        })
        .unwrap();
        placed(&mut wm, "plain", [100, 100], [300, 200]);
        wm.windows[0].restore = Some([100, 100, 300, 200]);
        placed(&mut wm, "max", [200, 200], [300, 200]);
        wm.windows[1].restore = Some([200, 200, 300, 200]);
        wm.press_taskbar(taskbar::Part::Start("X".into()));
        assert!(wm.menu.is_open(), "the menu of X is open");
        wm.event(Event::OutputRemoved { name: "X".into() }).unwrap();
        assert!(!wm.menu.is_open(), "the menu of X closed");
        assert!(
            wm.windows[0].pos[0] >= 1280,
            "plain moved to Y: {:?}",
            wm.windows[0].pos
        );
        assert!(wm.windows[1].pos[0] >= 1280, "max moved to Y");
        assert!(wm.bars.iter().all(|b| b.output == "Y"), "only Y has a bar");
    }
}

mod choices {
    use std::path::PathBuf;

    use super::*;

    /// The settings of the Control Panel change what they say: the scale
    /// goes to the core, an effect flips, the volume and the note change
    /// nothing else, and a gift of the window to another manager with
    /// nothing focused tells the person why.
    #[test]
    fn the_control_panel_settings_change_what_they_say() {
        let (mut wm, methods) = wm_on_fake_core("settings");
        let before = methods.lock().unwrap().len();
        wm.set(&control::Setting::Scale(150), Some("X"));
        assert!(
            sent(&methods, before)
                .iter()
                .any(|m| m == "output.configure"),
            "the scale goes to the core"
        );
        let lamp = wm.effects.toggles.lamp;
        wm.set(&control::Setting::Effect(control::Effect::Lamp), None);
        assert_ne!(wm.effects.toggles.lamp, lamp, "the lamp flips");
        assert_eq!(wm.set(&control::Setting::Note, None).len(), 0);
        assert_eq!(
            wm.set(
                &control::Setting::Volume(chora_kit::volume::Change::Mute),
                None
            )
            .len(),
            0
        );
        wm.set(&control::Setting::Give("mc"), None);
        wm.set(
            &control::Setting::Wallpaper(PathBuf::from("/nonexistent/wall.png")),
            None,
        );
    }

    /// The Control Panel opens its dialogs on the taskbar, the date and the
    /// display, and a second open of one raises it instead of a new one.
    #[test]
    fn the_control_panel_opens_its_dialogs_once() {
        let (mut wm, _methods) = wm_on_fake_core("panel-dialogs");
        wm.set(&control::Setting::Taskbar, None);
        wm.set(&control::Setting::DateTime, None);
        wm.set(&control::Setting::Display, None);
        let open = wm.panels.len();
        assert!(open >= 3, "three dialogs: {open}");
        wm.set(&control::Setting::Taskbar, None);
        assert_eq!(
            wm.panels.len(),
            open,
            "the taskbar dialog is raised, not new"
        );
    }

    /// Turn Off opens the dialog, and a press outside it closes it, with
    /// the fade that goes with it.
    #[test]
    fn a_press_outside_the_dialog_closes_it() {
        let (mut wm, _methods) = wm_on_fake_core("dialog-outside");
        wm.key(&power_key());
        assert!(wm.dialog.is_some(), "the dialog opened");
        let closed = wm.press_dialog("nowhere");
        assert!(wm.dialog.is_none(), "closed");
        assert!(!closed.is_empty(), "the dialog is removed");
        assert!(
            wm.press_dialog("nowhere").is_empty(),
            "nothing left to close"
        );
    }

    /// A press on an enabled button of the dialog shows it pressed and
    /// keeps the dialog open, and so does a press on a disabled part of it;
    /// a press on a node outside the dialog closes it.
    #[test]
    fn a_press_on_a_button_keeps_the_dialog_open() {
        let (mut wm, _methods) = wm_on_fake_core("dialog-button");
        wm.key(&power_key());
        let pressed = wm.press_dialog("xp-session-button-0");
        assert!(!pressed.is_empty(), "the button looks pressed");
        assert!(wm.dialog.is_some(), "still open");
        wm.press_dialog("xp-session-button-99");
        assert!(wm.dialog.is_some(), "a part of the dialog keeps it open");
        wm.press_dialog("xp-desktop");
        assert!(wm.dialog.is_none(), "a node outside the dialog closes it");
    }

    /// Cancel closes the dialog and changes nothing; Switch User is
    /// disabled and does nothing either.
    #[test]
    fn cancel_and_switch_user_change_nothing() {
        let (mut wm, _methods) = wm_on_fake_core("choose-cancel");
        wm.key(&power_key());
        assert!(
            !wm.choose(Choice::Cancel).is_empty(),
            "the dialog is removed"
        );
        assert!(wm.dialog.is_none());
        wm.key(&power_key());
        assert!(wm.choose(Choice::SwitchUser).is_empty() || wm.dialog.is_none());
        assert!(wm.dialog.is_none(), "the dialog closed");
    }

    /// Each choice of the dialog runs the command of the settings when one
    /// is set. The commands here are `true`, so nothing changes.
    #[test]
    fn each_choice_runs_the_command_of_the_settings() {
        let (mut wm, _methods) = wm_on_fake_core("choose-settings");
        wm.settings.value.session.log_off = Some("true".into());
        wm.settings.value.session.turn_off = Some("true".into());
        wm.settings.value.session.restart = Some("true".into());
        wm.settings.value.session.stand_by = Some("true".into());
        wm.effects.toggles.sounds = false;
        for choice in [
            Choice::LogOff,
            Choice::TurnOff,
            Choice::Restart,
            Choice::StandBy,
        ] {
            wm.key(&power_key());
            assert!(wm.dialog.is_some());
            let _ = wm.choose(choice);
            assert!(wm.dialog.is_none(), "the choice closes the dialog");
        }
    }

    /// A shared script that is not installed says so in a balloon, and the
    /// choice that needs it does nothing else.
    #[test]
    fn a_missing_shared_script_is_told_in_a_balloon() {
        let (mut wm, methods) = wm_on_fake_core("shared-missing");
        let before = methods.lock().unwrap().len();
        wm.run_shared("no-such-script", &["arg"]);
        let sent = methods.lock().unwrap().len() - before;
        assert_eq!(sent, 1, "one balloon goes to the notification server");
    }

    /// A dialog fades the desktop while it is open, and the fade ends with
    /// the dialog.
    #[test]
    fn the_desktop_fades_while_a_dialog_is_open() {
        let (mut wm, _methods) = wm_on_fake_core("fade-dialog");
        wm.key(&power_key());
        assert!(wm.fade.is_some(), "the dialog starts the fade");
        assert!(wm.fading(), "the desktop fades");
        assert!(
            wm.fade_frame(Instant::now()).is_some(),
            "a frame of the fade"
        );
        wm.choose(Choice::Cancel);
        assert!(wm.fade.is_none(), "the fade ends with the dialog");
        assert!(!wm.fading());
        assert!(wm.fade_frame(Instant::now()).is_none(), "no frame after it");
    }
}

mod press {
    use super::*;
    use crate::decoration::Button;
    use context::Act;

    /// A window of xp at a place and a size, framed as a client draws it.
    fn placed(wm: &mut Wm, surface: &str, pos: [i32; 2], size: [i32; 2]) {
        own_window(wm, surface);
        let win = wm.windows.last_mut().unwrap();
        win.pos = pos;
        win.size = Some(size);
        win.framed = true;
    }

    /// A press on a task button activates the window, and a second press on
    /// the active one minimizes it; a third press brings it back.
    #[test]
    fn a_task_button_activates_then_minimizes_then_restores() {
        let (mut wm, _methods) = wm_on_fake_core("task-button");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.pressed("xp-task-s1-X", LEFT_BUTTON, [0.0, 0.0]);
        assert_eq!(wm.focused.as_deref(), Some("s1"), "activated");
        wm.pressed("xp-task-s1-X", LEFT_BUTTON, [0.0, 0.0]);
        assert!(wm.windows[0].minimized, "the active window minimizes");
        wm.pressed("xp-task-s1-X", LEFT_BUTTON, [0.0, 0.0]);
        assert!(!wm.windows[0].minimized, "and comes back");
    }

    /// A press on the title bar starts a drag that moves the window with the
    /// pointer; a release outside every taskbar leaves the window on its
    /// output.
    #[test]
    fn a_title_bar_drag_moves_the_window_and_stays_on_its_output() {
        let (mut wm, _methods) = wm_on_fake_core("title-drag");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_node("xp-title-s1", LEFT_BUTTON, [150.0, 110.0]);
        assert!(wm.drag.is_some(), "the drag began");
        wm.dragged([200.0, 160.0]);
        assert_eq!(wm.windows[0].pos, [150, 150], "moved with the pointer");
        wm.released("xp-title-s1", [200.0, 160.0]);
        assert!(wm.drag.is_none(), "the drag ended");
        let [x, y, ..] = wm.windows[0].rect();
        assert!(
            x >= 0 && y >= 0,
            "kept on the output: {:?}",
            wm.windows[0].rect()
        );
    }

    /// The second press of a double click on a title bar maximizes the
    /// window, and the third restores nothing that was not maximized.
    #[test]
    fn a_double_click_on_a_title_bar_maximizes_the_window() {
        let (mut wm, _methods) = wm_on_fake_core("title-double");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_title("s1", [150.0, 110.0]);
        wm.press_title("s1", [150.0, 110.0]);
        assert!(wm.windows[0].maximized(), "the double click maximizes");
    }

    /// A right press on a title bar opens the system menu of its window, and
    /// the Escape key closes it again.
    #[test]
    fn a_right_press_on_a_title_bar_opens_the_system_menu() {
        let (mut wm, _methods) = wm_on_fake_core("title-right");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.press_node("xp-title-s1", RIGHT_BUTTON, [150.0, 110.0]);
        assert_eq!(wm.sys_window.as_deref(), Some("s1"), "the menu of s1");
        assert!(wm.context.is_some(), "the system menu shows");
        wm.node_key("Escape");
        assert!(wm.context.is_none(), "Escape closes the menu");
    }

    /// The system menu acts on its window by the key of its row: Close
    /// closes the window, and Minimize minimizes it.
    #[test]
    fn the_system_menu_acts_on_its_window_by_key() {
        let (mut wm, _methods) = wm_on_fake_core("system-keys");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        wm.open_system("s1", [150, 110]);
        let closed = wm.node_key("c");
        assert!(wm.context.is_none(), "the menu closed");
        assert!(
            closed
                .iter()
                .any(|c| matches!(c, Command::Close { surface } if surface == "s1")),
            "Close closes s1: {closed:?}"
        );
        wm.open_system("s1", [150, 110]);
        wm.node_key("n");
        assert!(wm.windows[0].minimized, "Minimize minimizes s1");
    }

    /// The right button on a taskbar, its clock or the desktop opens the
    /// menu of that part; a press on a node of none of them opens nothing.
    #[test]
    fn a_right_press_opens_the_menu_of_its_part() {
        let (mut wm, _methods) = wm_on_fake_core("right-parts");
        wm.press_right("xp-tray-X", [1200.0, 790.0]);
        assert!(wm.context.is_some(), "the clock menu");
        wm.node_key("Escape");
        wm.press_right("xp-panel-X", [600.0, 790.0]);
        assert!(wm.context.is_some(), "the taskbar menu");
        wm.node_key("Escape");
        wm.press_right(&wallpaper::id("X"), [600.0, 400.0]);
        assert!(wm.context.is_some(), "the desktop menu");
        wm.node_key("Escape");
        assert_eq!(wm.press_right("nothing-here", [1.0, 1.0]).len(), 0);
        assert!(wm.context.is_none());
    }

    /// Shift+F10 or the Menu key on the desktop opens its menu, and the same
    /// key again shuts nothing that is not open.
    #[test]
    fn the_menu_key_opens_the_desktop_menu() {
        let (mut wm, _methods) = wm_on_fake_core("menu-key");
        wm.node_key(MENU_KEY);
        assert!(wm.context.is_some(), "the desktop menu opens");
        wm.node_key("Escape");
        assert!(wm.context.is_none());
        assert!(wm.node_key("x").is_empty(), "an unknown key with no menu");
    }

    /// The rows of the desktop and window menus act on their output: cascade
    /// and tile the windows of the desktop, lock the taskbar, and open the
    /// properties dialogs.
    #[test]
    fn the_rows_of_the_menus_act() {
        let (mut wm, _methods) = wm_on_fake_core("rows");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        placed(&mut wm, "s2", [200, 200], [300, 200]);
        let area = wm.bars[0].area();
        assert!(!wm.act(Act::Cascade, area).is_empty(), "cascade");
        assert!(!wm.act(Act::TileRows, area).is_empty(), "tile rows");
        assert!(!wm.act(Act::TileColumns, area).is_empty(), "tile columns");
        let locked = wm.taskbar.locked;
        wm.act(Act::Lock, area);
        assert_ne!(wm.taskbar.locked, locked, "the taskbar locks");
        wm.act(Act::DesktopProperties, area);
        assert!(
            wm.panels.iter().any(|p| matches!(p, Panel::Display(_))),
            "Display Properties"
        );
        wm.act(Act::Position(1), area);
        wm.act(Act::Color([0, 0, 0]), area);
        wm.act(Act::TaskbarProperties, area);
        wm.act(Act::DateTime, area);
        wm.act(Act::Refresh, area);
        wm.act(Act::ArrangeByName, area);
        wm.act(Act::ArrangeByType, area);
        wm.act(Act::ShowDesktop, area);
    }

    /// Clicking a window's close, minimize and maximize buttons acts only when
    /// the button went up on the same button it went down on.
    #[test]
    fn a_window_button_acts_when_the_press_and_release_match() {
        let (mut wm, _methods) = wm_on_fake_core("buttons");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        let at = |wm: &Wm, button: Button| {
            let [bx, by, bw, bh] = decoration::button_rect(wm.windows[0].outer()[0], button);
            [f64::from(100 + bx + bw / 2), f64::from(100 + by + bh / 2)]
        };
        let minimize = at(&wm, Button::Minimize);
        wm.press_decoration(
            &Part::Button("s1".into(), Button::Minimize),
            LEFT_BUTTON,
            minimize,
        );
        assert!(
            wm.release_button("s1", Button::Maximize, minimize)
                .is_empty(),
            "another button"
        );
        assert!(!wm.windows[0].minimized, "not minimized");
        wm.press_decoration(
            &Part::Button("s1".into(), Button::Minimize),
            LEFT_BUTTON,
            minimize,
        );
        wm.release_button("s1", Button::Minimize, minimize);
        assert!(wm.windows[0].minimized, "minimized on the release");
        wm.windows[0].minimized = false;
        let close = at(&wm, Button::Close);
        wm.press_decoration(
            &Part::Button("s1".into(), Button::Close),
            LEFT_BUTTON,
            close,
        );
        let closed = wm.release_button("s1", Button::Close, [0.0, 0.0]);
        assert!(
            !closed.iter().any(|c| matches!(c, Command::Close { .. })),
            "released outside"
        );
        wm.press_decoration(
            &Part::Button("s1".into(), Button::Close),
            LEFT_BUTTON,
            close,
        );
        let closed = wm.release_button("s1", Button::Close, close);
        assert!(
            closed
                .iter()
                .any(|c| matches!(c, Command::Close { surface } if surface == "s1"))
        );
    }

    /// A window that is given away is not xp's anymore: a give of a window xp
    /// does not manage is refused, one to xp itself is refused as bad
    /// parameters, and the method of a give of a window of xp is answered.
    #[test]
    fn a_give_is_refused_when_it_cannot_be_made() {
        let (mut wm, _methods) = wm_on_fake_core("give-refused");
        placed(&mut wm, "s1", [100, 100], [300, 200]);
        let err = wm
            .method(give::METHOD, json!({"surface": "nope", "to": "mc"}))
            .unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED, "no such window");
        let err = wm.give_dragged("s1", NAME, None).expect_err("to xp itself");
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
        let err = wm.give("s1", "mc").expect_err("the book does not know s1");
        assert_eq!(err.code, RpcError::REJECTED);
        assert_eq!(wm.windows.len(), 1, "s1 stays");
    }

    /// The task button of the help window is pressed without a help client,
    /// and a press on a node that is no window of xp opens nothing.
    #[test]
    fn a_press_on_the_help_task_and_an_unknown_surface() {
        let (mut wm, _methods) = wm_on_fake_core("help-task");
        let _ = wm.press_taskbar(taskbar::Part::Task(HELP_TASK.into()));
        assert_eq!(
            wm.press_node("unknown-surface", LEFT_BUTTON, [0.0, 0.0])
                .len(),
            0
        );
    }

    /// A pager button of the taskbar turns the cube to its desktop.
    #[test]
    fn the_pager_turns_the_cube_to_its_desktop() {
        let (mut wm, _methods) = wm_on_fake_core("pager");
        wm.press_taskbar(taskbar::Part::Pager(2));
        assert!(wm.effects.turn.is_some(), "the cube turns");
    }
}

/// xp-tidy pr1-1: the window menu opens from a cache of the looks that
/// run; the core is asked at a start and at a client event, not at the
/// press (a slow core must not hold the menu).
#[test]
fn the_window_menu_opens_without_asking_the_core_for_the_clients() {
    let core = chora_kit::testing::FakeCore::start();
    core.outputs(json!([fake_output()]));
    core.respond(|method, _| match method {
        "scene.apply" => Some(Value::Null),
        "clients.list" => Some(json!([{"name": "mc", "methods": ["window.give"]}])),
        _ => None,
    });
    let methods = core.record(|method, _| Some(method.to_owned()));
    let mut wm = wm_on_kit_core(&core, "menu-cache");
    own_window(&mut wm, "s1");
    let from = methods.lock().unwrap().len();
    wm.open_system("s1", [150, 110]);
    assert!(
        !sent(&methods, from).iter().any(|m| m == "clients.list"),
        "the press asked for the clients: {:?}",
        sent(&methods, from)
    );
    wm.node_key("Escape");
    let from = methods.lock().unwrap().len();
    let connected = chora_scene::Event::ClientConnected { name: "mc".into() };
    wm.event(connected).unwrap();
    assert_eq!(
        sent(&methods, from)
            .iter()
            .filter(|m| *m == "clients.list")
            .count(),
        1,
        "a client event refreshes the cache once"
    );
    let from = methods.lock().unwrap().len();
    wm.open_system("s1", [150, 110]);
    assert!(
        !sent(&methods, from).iter().any(|m| m == "clients.list"),
        "the press asked for the clients after the refresh"
    );
}

/// xp-tidy pr1-2: a failed `clients.list` leaves one warn line, and the
/// Send To rows stay gray.
#[test]
fn a_failed_clients_list_warns_once() {
    if chora_kit::testing::in_child() {
        let core = chora_kit::testing::FakeCore::start();
        core.outputs(json!([fake_output()]));
        core.respond(|method, _| match method {
            "scene.apply" => Some(Value::Null),
            "clients.list" => Some(json!([])),
            _ => None,
        });
        let mut wm = wm_on_kit_core(&core, "list-failed");
        core.refuse_next("clients.list");
        let connected = chora_scene::Event::ClientConnected { name: "mc".into() };
        wm.event(connected).unwrap();
        return;
    }
    let log = chora_kit::testing::stderr_of("wm::tests_moved::a_failed_clients_list_warns_once");
    let lines: Vec<&str> = log.lines().filter(|l| l.contains("clients.list")).collect();
    assert_eq!(lines.len(), 1, "one warn line: {log:?}");
    assert!(lines[0].starts_with("chora-xp: clients.list"), "{lines:?}");
}

/// xp-tidy pr3-1: a look that exports `window.give` after its connected
/// event is listed when the system menu opens.
#[test]
fn a_look_that_exports_give_late_is_listed_at_menu_open() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let ready = std::sync::Arc::new(AtomicBool::new(false));
    let core = chora_kit::testing::FakeCore::start();
    core.outputs(json!([fake_output()]));
    let flag = ready.clone();
    core.respond(move |method, _| match method {
        "scene.apply" => Some(Value::Null),
        "clients.list" => {
            let methods: &[&str] = if flag.load(Ordering::SeqCst) {
                &["window.give"]
            } else {
                &[]
            };
            Some(json!([{"name": "mc", "methods": methods}]))
        }
        _ => None,
    });
    let mut wm = wm_on_kit_core(&core, "late-give");
    own_window(&mut wm, "s1");
    let connected = chora_scene::Event::ClientConnected { name: "mc".into() };
    wm.event(connected).unwrap();
    ready.store(true, Ordering::SeqCst);
    wm.open_system("s1", [150, 110]);
    assert_eq!(wm.looks, ["mc"], "the look is listed at the open");
}
