//! Tests of popups and drags inside a plane and a 3D context: a popup
//! that grabs the pointer, a menu that opens on a press and takes the
//! release, a drag that leaves the plane, and a drag and drop onto a
//! second window (D75).

use serde_json::json;
use smithay::{
    backend::input::{ButtonState, InputTime},
    utils::{Logical, Point},
};
use wayland_client::Proxy;

use super::{
    context::{chain, shown, space, translate, turn_y},
    planes::{click, close, forward, move_to, window_under},
};
use crate::testing::{
    Client, Core,
    wayland::{DndEvent, PointerEvent, Popup, Toplevel, Wayland},
};

type Shown = Box<dyn Fn(f64, f64) -> Point<f64, Logical>>;

/// A window of 100 by 80 in a plane or a context, and where a point of
/// its geometry shows on the screen.
struct Rig {
    core: Core,
    wayland: Wayland,
    window: Toplevel,
    shown: Shown,
    _wm: Client,
}

fn plane_rig() -> Rig {
    let rig = json!({"translate": [200, 100], "rotate": 40});
    let (core, wayland, window, total, wm) = window_under(&rig);
    Rig {
        core,
        wayland,
        window,
        shown: Box::new(move |u, v| forward(&total, u, v)),
        _wm: wm,
    }
}

fn context_rig() -> Rig {
    let mut core = Core::new();
    core.add_output("test-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let m = chain(&[
        translate(400.0, 300.0, 200.0),
        turn_y(40.0),
        translate(-50.0, -40.0, 0.0),
    ]);
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "eye"}},
        {"update": {"id": "s1", "props": {"transform": {"matrix": m}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    Rig {
        core,
        wayland,
        window,
        shown: Box::new(move |u, v| shown(&m, [400.0, 300.0], u, v).into()),
        _wm: wm,
    }
}

impl Rig {
    fn move_to(&mut self, u: f64, v: f64) {
        let at = (self.shown)(u, v);
        move_to(&mut self.core, at);
    }

    fn events(&mut self) -> Vec<PointerEvent> {
        self.wayland.roundtrip(&mut self.core);
        self.wayland.take_pointer_events()
    }

    fn menu(&mut self, serial: u32) -> Popup {
        let window = &self.window;
        let placed = ((10, 10), (40, 30));
        self.wayland.menu(&mut self.core, window, serial, placed)
    }
}

fn pressed(events: &[PointerEvent]) -> u32 {
    events
        .iter()
        .rev()
        .find_map(|e| match e {
            PointerEvent::Button {
                serial,
                pressed: true,
            } => Some(*serial),
            _ => None,
        })
        .expect("a press")
}

fn enters(events: &[PointerEvent], surface: &wayland_client::backend::ObjectId) -> Vec<(f64, f64)> {
    events
        .iter()
        .filter_map(|e| match e {
            PointerEvent::Enter { surface: s, at } if s == surface => Some(*at),
            _ => None,
        })
        .collect()
}

fn buttons(events: &[PointerEvent]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|e| match e {
            PointerEvent::Button { pressed, .. } => Some(*pressed),
            _ => None,
        })
        .collect()
}

fn last_motion(events: &[PointerEvent]) -> Option<(f64, f64)> {
    events.iter().rev().find_map(|e| match e {
        PointerEvent::Enter { at, .. } | PointerEvent::Motion { at } => Some(*at),
        _ => None,
    })
}

fn a_popup_takes_motion_a_click_and_a_click_outside_closes_it(mut rig: Rig) {
    rig.move_to(50.0, 60.0);
    click(&mut rig.core, true);
    click(&mut rig.core, false);
    let serial = pressed(&rig.events());
    let popup = rig.menu(serial);
    assert!(!rig.wayland.popup_done(&popup), "the press opens the grab");
    rig.move_to(30.0, 30.0);
    let moved = rig.events();
    let entered = enters(&moved, &popup.surface.id());
    assert!(
        entered.last().is_some_and(|at| close(*at, (20.0, 20.0))),
        "the motion enters the popup: {moved:?}"
    );
    click(&mut rig.core, true);
    click(&mut rig.core, false);
    let clicked = rig.events();
    assert_eq!(buttons(&clicked), [true, false], "the click reaches it");
    assert!(!rig.wayland.popup_done(&popup), "a click in it keeps it");
    rig.move_to(150.0, 40.0);
    click(&mut rig.core, true);
    rig.events();
    assert!(rig.wayland.popup_done(&popup), "a click outside closes it");
}

#[test]
fn a_popup_in_a_turned_window_takes_motion_a_click_and_closes_on_a_click_outside() {
    a_popup_takes_motion_a_click_and_a_click_outside_closes_it(plane_rig());
}

#[test]
fn a_popup_in_a_window_of_a_context_takes_motion_a_click_and_closes_on_a_click_outside() {
    a_popup_takes_motion_a_click_and_a_click_outside_closes_it(context_rig());
}

fn a_menu_takes_the_press_drag_release_that_opened_it(mut rig: Rig) {
    rig.move_to(50.0, 60.0);
    click(&mut rig.core, true);
    let serial = pressed(&rig.events());
    let popup = rig.menu(serial);
    rig.move_to(30.0, 30.0);
    let moved = rig.events();
    let entered = enters(&moved, &popup.surface.id());
    assert!(
        entered.last().is_some_and(|at| close(*at, (20.0, 20.0))),
        "the drag enters the popup while the button is down: {moved:?}"
    );
    click(&mut rig.core, false);
    let released = rig.events();
    assert_eq!(buttons(&released), [false], "the release reaches the menu");
    assert!(
        !rig.wayland.popup_done(&popup),
        "the release keeps the menu"
    );
}

#[test]
fn a_menu_in_a_turned_window_takes_the_press_drag_release_that_opened_it() {
    a_menu_takes_the_press_drag_release_that_opened_it(plane_rig());
}

#[test]
fn a_menu_in_a_window_of_a_context_takes_the_press_drag_release_that_opened_it() {
    a_menu_takes_the_press_drag_release_that_opened_it(context_rig());
}

fn a_drag_that_starts_in_a_popup_keeps_sending_points_outside_the_plane(mut rig: Rig) {
    rig.move_to(50.0, 60.0);
    click(&mut rig.core, true);
    click(&mut rig.core, false);
    let serial = pressed(&rig.events());
    let popup = rig.menu(serial);
    rig.move_to(30.0, 30.0);
    rig.events();
    click(&mut rig.core, true);
    for (u, v) in [(60.0, 50.0), (130.0, 95.0), (-30.0, 95.0)] {
        rig.move_to(u, v);
        let moved = rig.events();
        assert!(
            last_motion(&moved).is_some_and(|at| close(at, (u - 10.0, v - 10.0))),
            "the popup gets {:?}: {moved:?}",
            (u - 10.0, v - 10.0)
        );
    }
    click(&mut rig.core, false);
    assert!(!rig.wayland.popup_done(&popup), "the drag keeps the popup");
}

#[test]
fn a_drag_in_a_popup_of_a_turned_window_keeps_sending_points_outside_the_plane() {
    a_drag_that_starts_in_a_popup_keeps_sending_points_outside_the_plane(plane_rig());
}

#[test]
fn a_drag_in_a_popup_of_a_window_of_a_context_keeps_sending_points_outside_the_plane() {
    a_drag_that_starts_in_a_popup_keeps_sending_points_outside_the_plane(context_rig());
}

fn two_windows_in_a_context() -> (Core, Wayland, Toplevel, Toplevel, Shown, Shown, Client) {
    let mut core = Core::new();
    core.add_output("test-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let source = wayland.toplevel(&mut core, "source");
    wayland.attach(&mut core, &source, (100, 80));
    let target = wayland.toplevel(&mut core, "target");
    wayland.attach(&mut core, &target, (100, 80));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let place = |x: f64| {
        chain(&[
            translate(x, 300.0, 200.0),
            turn_y(40.0),
            translate(-50.0, -40.0, 0.0),
        ])
    };
    let (left, right) = (place(250.0), place(550.0));
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "eye"}},
        {"update": {"id": "s1", "props": {"transform": {"matrix": left}}}},
        {"attach": {"surface": "s2", "parent": "eye"}},
        {"update": {"id": "s2", "props": {"transform": {"matrix": right}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let on_left: Shown = Box::new(move |u, v| shown(&left, [400.0, 300.0], u, v).into());
    let on_right: Shown = Box::new(move |u, v| shown(&right, [400.0, 300.0], u, v).into());
    (core, wayland, source, target, on_left, on_right, wm)
}

#[test]
fn a_drop_goes_to_the_second_window_of_the_context_and_not_to_the_source() {
    let (mut core, mut wayland, source, target, on_left, on_right, _wm) =
        two_windows_in_a_context();
    move_to(&mut core, on_left(50.0, 40.0));
    click(&mut core, true);
    wayland.roundtrip(&mut core);
    let serial = pressed(&wayland.take_pointer_events());
    wayland.start_drag(&mut core, &source, serial);
    wayland.take_dnd_events();
    for (u, v) in [(90.0, 40.0), (130.0, 30.0)] {
        move_to(&mut core, on_left(u, v));
    }
    move_to(&mut core, on_right(30.0, 50.0));
    move_to(&mut core, on_right(60.0, 40.0));
    wayland.roundtrip(&mut core);
    let moved = wayland.take_dnd_events();
    let entered: Vec<_> = moved
        .iter()
        .filter_map(|e| match e {
            DndEvent::Enter { surface, at } => Some((surface.clone(), *at)),
            _ => None,
        })
        .collect();
    assert!(
        entered
            .iter()
            .any(|(s, at)| *s == target.surface.id() && close(*at, (30.0, 50.0))),
        "the drag enters the second window at its point: {moved:?}"
    );
    assert!(
        close(
            moved
                .iter()
                .rev()
                .find_map(|e| match e {
                    DndEvent::Motion { at } => Some(*at),
                    _ => None,
                })
                .expect("a motion"),
            (60.0, 40.0)
        ),
        "{moved:?}"
    );
    wayland.roundtrip(&mut core);
    click(&mut core, false);
    wayland.roundtrip(&mut core);
    let dropped = wayland.take_dnd_events();
    assert!(dropped.contains(&DndEvent::Drop), "the drop: {dropped:?}");
    assert!(
        !dropped.contains(&DndEvent::Cancelled),
        "the drag is not cancelled: {dropped:?}"
    );
}

fn a_menu_that_overflows_the_window_slides_into_its_geometry(mut rig: Rig) {
    rig.move_to(50.0, 60.0);
    click(&mut rig.core, true);
    let serial = pressed(&rig.events());
    let window = &rig.window;
    let placed = ((80, 60), (40, 30));
    let menu = rig.wayland.menu(&mut rig.core, window, serial, placed);
    assert_eq!(
        rig.wayland.popup_geometry(&menu),
        Some([60, 50, 40, 30]),
        "the area of the popup is the window geometry"
    );
    rig.move_to(90.0, 70.0);
    let moved = rig.events();
    let entered = enters(&moved, &menu.surface.id());
    assert!(
        entered.last().is_some_and(|at| close(*at, (30.0, 20.0))),
        "the slid popup takes the point: {moved:?}"
    );
}

#[test]
fn a_menu_that_overflows_a_turned_window_slides_into_its_geometry() {
    a_menu_that_overflows_the_window_slides_into_its_geometry(plane_rig());
}

#[test]
fn a_menu_that_overflows_a_window_of_a_context_slides_into_its_geometry() {
    a_menu_that_overflows_the_window_slides_into_its_geometry(context_rig());
}

type Windows = (Core, Wayland, Toplevel, Toplevel, Shown, Shown, Client);

fn press_on_source(windows: &mut Windows, injected: bool) -> u32 {
    let (core, wayland, _, _, on_left, _, _) = windows;
    move_to(core, on_left(50.0, 40.0));
    let state = ButtonState::Pressed;
    crate::input::press_button(&mut core.state, 272, state, InputTime::now(), injected);
    wayland.roundtrip(core);
    pressed(&wayland.take_pointer_events())
}

fn cancelled(wayland: &mut Wayland, core: &mut Core) -> bool {
    wayland.roundtrip(core);
    wayland.take_dnd_events().contains(&DndEvent::Cancelled)
}

#[test]
fn a_drag_started_with_the_press_of_another_client_is_cancelled() {
    let mut windows = two_windows_in_a_context();
    let serial = press_on_source(&mut windows, false);
    let mut other = Wayland::connect(&mut windows.0);
    let window = other.toplevel(&mut windows.0, "other");
    other.attach(&mut windows.0, &window, (100, 80));
    other.start_drag(&mut windows.0, &window, serial);
    assert!(
        cancelled(&mut other, &mut windows.0),
        "the press is not its"
    );
    assert!(!windows.0.state.client_drag);
}

#[test]
fn a_drag_started_at_an_injected_press_is_cancelled() {
    let mut windows = two_windows_in_a_context();
    let serial = press_on_source(&mut windows, true);
    let source = &windows.2;
    windows.1.start_drag(&mut windows.0, source, serial);
    assert!(cancelled(&mut windows.1, &mut windows.0));
    assert!(!windows.0.state.client_drag);
}

#[test]
fn an_injected_move_during_a_real_drag_cancels_it() {
    let mut windows = two_windows_in_a_context();
    let serial = press_on_source(&mut windows, false);
    let source = &windows.2;
    windows.1.start_drag(&mut windows.0, source, serial);
    assert!(windows.0.state.client_drag, "a real press starts the drag");
    let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
    windows.0.admin().ok(&mut windows.0, "grants.set", grant);
    let mut agent = windows.0.named("agent");
    let step = json!({"pointer_move": {"position": [500.0, 300.0]}});
    agent.ok(&mut windows.0, "input.inject", json!({"events": [step]}));
    assert!(cancelled(&mut windows.1, &mut windows.0));
    assert!(!windows.0.state.client_drag);
}

#[test]
fn a_second_start_with_the_same_serial_keeps_the_drag() {
    let mut windows = two_windows_in_a_context();
    let serial = press_on_source(&mut windows, false);
    let source = &windows.2;
    windows.1.start_drag(&mut windows.0, source, serial);
    windows.1.start_drag(&mut windows.0, source, serial);
    assert!(
        windows.0.state.client_drag,
        "the new grab holds the pointer"
    );
}

#[test]
fn a_lock_during_a_drag_cancels_it() {
    let mut windows = two_windows_in_a_context();
    let serial = press_on_source(&mut windows, false);
    let source = &windows.2;
    windows.1.start_drag(&mut windows.0, source, serial);
    windows.1.take_dnd_events();
    let _lock = windows.1.lock(&mut windows.0);
    assert!(cancelled(&mut windows.1, &mut windows.0));
    assert!(!windows.0.state.client_drag);
}

#[test]
fn an_emergency_reset_ends_a_drag() {
    let mut windows = two_windows_in_a_context();
    let serial = press_on_source(&mut windows, false);
    let source = &windows.2;
    windows.1.start_drag(&mut windows.0, source, serial);
    windows.0.state.reset();
    assert!(!windows.0.state.pointer().is_grabbed());
    assert!(!windows.0.state.client_drag);
    assert!(cancelled(&mut windows.1, &mut windows.0));
}

fn with_agent_window() -> (Windows, Wayland, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!("chora-drag-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a directory");
    let mut core = Core::new();
    core.state.ipc.path = Some(dir.join("wayland-1.sock"));
    core.add_output("test-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let source = wayland.toplevel(&mut core, "source");
    wayland.attach(&mut core, &source, (100, 80));
    let created = core
        .admin()
        .ok(&mut core, "clients.create", json!({"name": "agent"}));
    let path = std::path::PathBuf::from(created["wayland"].as_str().expect("a path"));
    let mut agent = Wayland::connect_to(&mut core, &path);
    let target = agent.toplevel(&mut core, "agent");
    agent.attach(&mut core, &target, (100, 80));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let place = |x: f64| {
        chain(&[
            translate(x, 300.0, 200.0),
            turn_y(40.0),
            translate(-50.0, -40.0, 0.0),
        ])
    };
    let (left, right) = (place(250.0), place(550.0));
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "eye"}},
        {"update": {"id": "s1", "props": {"transform": {"matrix": left}}}},
        {"attach": {"surface": "s2", "parent": "eye"}},
        {"update": {"id": "s2", "props": {"transform": {"matrix": right}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let on_left: Shown = Box::new(move |u, v| shown(&left, [400.0, 300.0], u, v).into());
    let on_right: Shown = Box::new(move |u, v| shown(&right, [400.0, 300.0], u, v).into());
    let windows = (core, wayland, source, target, on_left, on_right, wm);
    (windows, agent, dir)
}

#[test]
fn a_confined_client_gets_no_offer() {
    let (mut windows, mut agent, dir) = with_agent_window();
    agent.listen_for_drags(&mut windows.0);
    let serial = press_on_source(&mut windows, false);
    let source = &windows.2;
    windows.1.start_drag(&mut windows.0, source, serial);
    let over = (windows.5)(60.0, 40.0);
    move_to(&mut windows.0, over);
    agent.roundtrip(&mut windows.0);
    let events = agent.take_dnd_events();
    let _ = std::fs::remove_dir_all(dir);
    assert!(events.is_empty(), "the confined window got {events:?}");
}

#[test]
fn a_confined_client_starts_no_drag() {
    let (mut windows, mut agent, dir) = with_agent_window();
    let over = (windows.5)(50.0, 40.0);
    move_to(&mut windows.0, over);
    click(&mut windows.0, true);
    agent.roundtrip(&mut windows.0);
    let serial = pressed(&agent.take_pointer_events());
    let own = &windows.3;
    agent.start_drag(&mut windows.0, own, serial);
    let done = cancelled(&mut agent, &mut windows.0);
    let _ = std::fs::remove_dir_all(dir);
    assert!(done, "a confined client may not drag");
}
