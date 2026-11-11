//! Tests of popups and drags inside a plane and a 3D context: a popup
//! that grabs the pointer, a menu that opens on a press and takes the
//! release, a drag that leaves the plane, and a drag and drop onto a
//! second window (D75).

use serde_json::json;
use smithay::{
    backend::input::{ButtonState, InputTime, KeyState},
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

/// The corner of the first virtual output in the scene.
const FACE: f64 = -262_144.0;

/// A pane of a size in the context, turned 40 degrees, that forwards
/// into a virtual output of that size, and the matrix of the pane.
fn pane_in_a_context(core: &mut Core, wm: &mut Client, size: [f64; 2]) -> [f64; 16] {
    let spec = format!("{}x{}", size[0], size[1]);
    let added = wm.ok(core, "output.add", json!({"spec": spec, "virtual": true}));
    let m = chain(&[
        translate(400.0, 300.0, 200.0),
        turn_y(40.0),
        translate(-size[0] / 2.0, -size[1] / 2.0, 0.0),
    ]);
    let props = json!({"size": size, "shader": "/pane.frag", "forward": true,
        "uniforms": {"face": {"output": added["name"]}}, "transform": {"matrix": m}});
    let create = json!({"id": "pane", "kind": "shader", "parent": "eye", "props": props});
    wm.ok(
        core,
        "scene.apply",
        json!({"commands": [{"create": create}]}),
    );
    m
}

/// The window on a virtual output that a pane of the context shows.
fn pane_rig() -> Rig {
    let mut core = Core::new();
    core.add_output("test-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let m = pane_in_a_context(&mut core, &mut wm, [100.0, 80.0]);
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "root"}},
        {"update": {"id": "s1", "props": {"transform": {"translate": [FACE, FACE]}}}},
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

#[test]
fn a_popup_behind_a_pane_takes_motion_a_click_and_closes_on_a_click_outside() {
    a_popup_takes_motion_a_click_and_a_click_outside_closes_it(pane_rig());
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

#[test]
fn a_menu_behind_a_pane_takes_the_press_drag_release_that_opened_it() {
    a_menu_takes_the_press_drag_release_that_opened_it(pane_rig());
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

#[test]
fn a_drag_in_a_popup_behind_a_pane_keeps_sending_points_outside_the_pane() {
    a_drag_that_starts_in_a_popup_keeps_sending_points_outside_the_plane(pane_rig());
}

/// A press on the window behind the pane, dragged off the pane: the
/// window gets the points that the plane of the pane maps, also outside.
#[test]
fn a_drag_that_leaves_the_pane_keeps_sending_points() {
    let mut rig = pane_rig();
    rig.move_to(30.0, 40.0);
    let entered = rig.events();
    let at = enters(&entered, &rig.window.surface.id());
    assert!(
        at.last().is_some_and(|at| close(*at, (30.0, 40.0))),
        "{entered:?}"
    );
    click(&mut rig.core, true);
    for (u, v) in [(60.0, 50.0), (150.0, 95.0), (-30.0, 95.0)] {
        rig.move_to(u, v);
        let moved = rig.events();
        assert!(
            last_motion(&moved).is_some_and(|at| close(at, (u, v))),
            "the window gets {:?}: {moved:?}",
            (u, v)
        );
    }
    click(&mut rig.core, false);
    let released = rig.events();
    assert_eq!(buttons(&released), [false], "the release");
    assert_eq!(released.last(), Some(&PointerEvent::Leave), "off the pane");
}

/// While a press holds the pointer behind a pane that stands almost edge
/// on, a point of the screen maps to the virtual output or, behind the
/// eye, to nothing: never to the screen.
#[test]
fn a_held_press_behind_a_pane_never_falls_back_to_the_screen() {
    let mut core = Core::new();
    core.add_output("test-1", (800, 600));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let m = pane_in_a_context(&mut core, &mut wm, [100.0, 80.0]);
    move_to(&mut core, shown(&m, [400.0, 300.0], 50.0, 40.0).into());
    click(&mut core, true);
    let edge_on = chain(&[
        translate(400.0, 300.0, 200.0),
        turn_y(89.0),
        translate(-50.0, -40.0, 0.0),
    ]);
    let turned = json!({"commands": [{"update": {"id": "pane",
        "props": {"transform": {"matrix": edge_on}}}}]});
    wm.ok(&mut core, "scene.apply", turned);
    let spots: Vec<_> = (0..80)
        .map(|step| core.state.spot((f64::from(step) * 10.0, 300.0).into()))
        .collect();
    let outputs = spots.iter().flatten().map(|spot| spot.output.name());
    assert!(outputs.clone().all(|name| name == "VIRTUAL-1"));
    assert!(spots.iter().any(Option::is_none), "a point behind the eye");
}

type TwoWindows = (Core, Wayland, Toplevel, Toplevel, Shown, Shown, Client);

/// Two windows with a gap between them on a virtual output behind a pane.
fn two_windows_behind_a_pane() -> TwoWindows {
    let mut core = Core::new();
    core.add_output("test-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let source = wayland.toplevel(&mut core, "source");
    wayland.attach(&mut core, &source, (100, 80));
    let target = wayland.toplevel(&mut core, "target");
    wayland.attach(&mut core, &target, (100, 80));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let m = pane_in_a_context(&mut core, &mut wm, [250.0, 80.0]);
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "root"}},
        {"update": {"id": "s1", "props": {"transform": {"translate": [FACE, FACE]}}}},
        {"attach": {"surface": "s2", "parent": "root"}},
        {"update": {"id": "s2", "props": {"transform": {"translate": [FACE + 150.0, FACE]}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let on_left: Shown = Box::new(move |u, v| shown(&m, [400.0, 300.0], u, v).into());
    let on_right: Shown = Box::new(move |u, v| shown(&m, [400.0, 300.0], u + 150.0, v).into());
    (core, wayland, source, target, on_left, on_right, wm)
}

#[test]
fn a_drop_goes_to_the_second_window_behind_a_pane() {
    a_drop_goes_to_the_second_window_and_not_to_the_source(two_windows_behind_a_pane());
}

fn two_windows_in_a_context() -> TwoWindows {
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
    a_drop_goes_to_the_second_window_and_not_to_the_source(two_windows_in_a_context());
}

fn a_drop_goes_to_the_second_window_and_not_to_the_source(windows: TwoWindows) {
    let (mut core, mut wayland, source, target, on_left, on_right, _wm) = windows;
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
    let dir = crate::testing::scratch_dir("chora-drag");
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

#[test]
fn a_drag_at_a_key_press_through_a_popup_grab_is_cancelled() {
    let mut windows = two_windows_in_a_context();
    let focus = json!({"commands": [{"focus": {"surface": "s1"}}]});
    windows.6.ok(&mut windows.0, "scene.apply", focus);
    let (core, wayland, source) = (&mut windows.0, &mut windows.1, &windows.2);
    move_to(core, (windows.4)(50.0, 40.0));
    let key = smithay::input::keyboard::Keycode::new(76);
    crate::input::key(
        &mut core.state,
        key,
        KeyState::Pressed,
        InputTime::now(),
        false,
    );
    wayland.roundtrip(core);
    let serial = *wayland.key_serials().last().expect("a key press");
    let menu = wayland.menu(core, source, serial, ((10, 10), (40, 30)));
    assert!(!wayland.popup_done(&menu), "the key opens the grab");
    wayland.start_drag(core, source, serial);
    assert!(cancelled(wayland, core), "no button is down");
    assert!(!core.state.client_drag);
}

fn release_on(windows: &mut Windows) {
    let state = ButtonState::Released;
    crate::input::press_button(&mut windows.0.state, 272, state, InputTime::now(), false);
    windows.1.roundtrip(&mut windows.0);
}

fn plane_press(windows: &mut Windows) {
    press_on_source(windows, false);
    release_on(windows);
}

fn interactive_move(windows: &mut Windows) {
    let serial = press_on_source(windows, false);
    windows.1.request_move(&mut windows.0, &windows.2, serial);
    release_on(windows);
}

fn interactive_resize(windows: &mut Windows) {
    let serial = press_on_source(windows, false);
    windows.1.request_resize(&mut windows.0, &windows.2, serial);
    release_on(windows);
}

fn menu_closed_by_a_click_outside(windows: &mut Windows) {
    let serial = press_on_source(windows, false);
    let menu = windows
        .1
        .menu(&mut windows.0, &windows.2, serial, ((10, 10), (40, 30)));
    release_on(windows);
    move_to(&mut windows.0, (5.0, 5.0).into());
    click(&mut windows.0, true);
    release_on(windows);
    assert!(windows.1.popup_done(&menu), "the click outside closes it");
}

fn drag_and_drop(windows: &mut Windows) {
    let serial = press_on_source(windows, false);
    windows.1.start_drag(&mut windows.0, &windows.2, serial);
    assert!(windows.0.state.client_drag, "the drag starts");
    move_to(&mut windows.0, (windows.5)(60.0, 40.0));
    release_on(windows);
}

fn node_press(windows: &mut Windows) {
    let rect = json!({"commands": [{"create": {"id": "knob", "kind": "rect",
        "parent": "root", "props": {"size": [40, 40], "color": [1, 0, 0, 1],
        "transform": {"translate": [740, 540]}}}}]});
    windows.6.ok(&mut windows.0, "scene.apply", rect);
    move_to(&mut windows.0, (760.0, 560.0).into());
    click(&mut windows.0, true);
    release_on(windows);
}

fn button_grab(windows: &mut Windows) {
    let grab = json!({"binding": "Super+BTN_LEFT"});
    windows.6.ok(&mut windows.0, "input.grab", grab);
    let super_key = smithay::input::keyboard::Keycode::new(125 + 8);
    let now = InputTime::now();
    crate::input::key(
        &mut windows.0.state,
        super_key,
        KeyState::Pressed,
        now,
        false,
    );
    move_to(&mut windows.0, (windows.4)(50.0, 40.0));
    click(&mut windows.0, true);
    release_on(windows);
    crate::input::key(
        &mut windows.0.state,
        super_key,
        KeyState::Released,
        now,
        false,
    );
}

fn lock_during_a_press(windows: &mut Windows) {
    press_on_source(windows, false);
    let _lock = windows.1.lock(&mut windows.0);
    release_on(windows);
}

fn click_outside_a_menu_on_a_plane(windows: &mut Windows) {
    let mut other = Wayland::connect(&mut windows.0);
    let third = other.toplevel(&mut windows.0, "third");
    let m = chain(&[
        translate(400.0, 120.0, 200.0),
        turn_y(40.0),
        translate(-50.0, -40.0, 0.0),
    ]);
    let commands = json!({"commands": [
        {"attach": {"surface": "s3", "parent": "eye"}},
        {"update": {"id": "s3", "props": {"transform": {"matrix": m}}}},
    ]});
    windows.6.ok(&mut windows.0, "scene.apply", commands);
    other.attach(&mut windows.0, &third, (100, 80));
    let serial = press_on_source(windows, false);
    let menu = windows
        .1
        .menu(&mut windows.0, &windows.2, serial, ((10, 10), (40, 30)));
    release_on(windows);
    move_to(&mut windows.0, shown(&m, [400.0, 300.0], 50.0, 40.0).into());
    click(&mut windows.0, true);
    windows.1.roundtrip(&mut windows.0);
    assert!(windows.1.popup_done(&menu), "the click outside closes it");
    windows.1.take_pointer_events();
    move_to(&mut windows.0, (windows.5)(50.0, 40.0));
    release_on(windows);
    let events = windows.1.take_pointer_events();
    assert!(
        !buttons(&events).contains(&false),
        "the release goes to the window of the click: {events:?}"
    );
}

fn second_click_without_motion(windows: &mut Windows) {
    press_on_source(windows, false);
    windows.1.take_pointer_events();
    move_to(&mut windows.0, (windows.5)(50.0, 40.0));
    release_on(windows);
    let events = windows.1.take_pointer_events();
    assert!(
        !enters(&events, &windows.3.surface.id()).is_empty(),
        "the release gives the pointer to the window under it: {events:?}"
    );
    click(&mut windows.0, true);
    release_on(windows);
}

type Sequence = (&'static str, fn(&mut Windows));

#[test]
fn every_press_ends_with_its_release() {
    let sequences: [Sequence; 10] = [
        ("plane press", plane_press),
        ("move", interactive_move),
        ("resize", interactive_resize),
        ("popup", menu_closed_by_a_click_outside),
        ("drag and drop", drag_and_drop),
        ("node press", node_press),
        ("button grab", button_grab),
        ("lock during a press", lock_during_a_press),
        (
            "click outside a menu on a plane",
            click_outside_a_menu_on_a_plane,
        ),
        ("second click without motion", second_click_without_motion),
    ];
    let failed: Vec<String> = (sequences.into_iter())
        .filter_map(|(name, sequence)| {
            let ran = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut windows = two_windows_in_a_context();
                windows
                    .6
                    .ok(&mut windows.0, "input.grab", json!({"binding": "move"}));
                sequence(&mut windows);
                windows.0.pointer_free()
            }));
            match ran {
                Ok(true) => None,
                Ok(false) => Some(format!("{name}: leaves a grab or a press")),
                Err(_) => Some(format!("{name}: fails")),
            }
        })
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_press_on_a_window_that_leaves_the_scene_frees_the_pointer() {
    let mut windows = two_windows_in_a_context();
    press_on_source(&mut windows, false);
    let remove = json!({"commands": [{"remove": {"id": "s1"}}]});
    windows.6.ok(&mut windows.0, "scene.apply", remove);
    move_to(&mut windows.0, (windows.5)(50.0, 40.0));
    assert!(!windows.0.state.pointer().is_grabbed());
    release_on(&mut windows);
}

#[test]
fn a_lock_a_reset_and_a_lost_device_end_the_held_button() {
    let mut windows = two_windows_in_a_context();
    press_on_source(&mut windows, false);
    let _lock = windows.1.lock(&mut windows.0);
    assert_eq!(windows.0.state.last_real_button, None, "the lock");
    let mut windows = two_windows_in_a_context();
    press_on_source(&mut windows, false);
    windows.0.state.reset();
    assert_eq!(windows.0.state.last_real_button, None, "the reset");
    let mut windows = two_windows_in_a_context();
    press_on_source(&mut windows, false);
    windows.0.state.device_removed("event1");
    assert_eq!(windows.0.state.last_real_button, None, "the device");
}

#[test]
fn two_agent_windows_keep_their_own_directory() {
    let (_first, _, first) = with_agent_window();
    let (_second, _, second) = with_agent_window();
    assert_ne!(first, second, "the tests share a directory");
    let _ = std::fs::remove_dir_all(first);
    let _ = std::fs::remove_dir_all(second);
}

/// xp on the virtual output of a face that the client `wm` shows through
/// a pane. xp has the window s1 with a title bar and holds a press on the
/// title, through the pane. The client `other` holds a claim on the
/// screen too, and both `wm` and `other` grabbed `move`.
fn pressed_title_behind_a_pane() -> (Core, Wayland, Client, Client, Client) {
    let mut core = Core::new();
    core.add_output("test-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let mut wm = space(&mut core, [800.0, 600.0]);
    let m = pane_in_a_context(&mut core, &mut wm, [100.0, 80.0]);
    let desk = json!({"create": {"id": "desk", "kind": "group", "parent": "root",
        "props": {"claim": [0.0, 0.0, 400.0, 600.0]}}});
    wm.ok(&mut core, "scene.apply", json!({"commands": [desk]}));
    let mut other = core.named("other");
    let side = json!({"create": {"id": "side", "kind": "group", "parent": "root",
        "props": {"claim": [400.0, 0.0, 400.0, 600.0]}}});
    other.ok(&mut core, "scene.apply", json!({"commands": [side]}));
    for client in [&mut wm, &mut other] {
        client.ok(&mut core, "input.grab", json!({"binding": "move"}));
        client.ok(&mut core, "events.subscribe", json!({"events": []}));
    }
    let mut xp = core.named("xp");
    let face = json!({"create": {"id": "xp", "kind": "group", "parent": "root",
        "props": {"claim": [FACE, FACE, 100.0, 80.0]}}});
    let title = json!({"create": {"id": "title", "kind": "rect", "parent": "xp",
        "props": {"size": [100, 20], "transform": {"translate": [FACE, FACE]}}}});
    let commands = json!({"commands": [
        face,
        {"attach": {"surface": "s1", "parent": "xp"}},
        {"update": {"id": "s1", "props": {"transform": {"translate": [FACE, FACE]}}}},
        title,
    ]});
    xp.ok(&mut core, "scene.apply", commands);
    move_to(&mut core, shown(&m, [400.0, 300.0], 30.0, 10.0).into());
    click(&mut core, true);
    assert!(core.state.node_grab.is_some(), "the press is on the title");
    (core, wayland, wm, other, xp)
}

/// The last `input.moved` of a client after the pointer went to (700,
/// 500) of the screen: its output and its position.
fn moved_to_the_screen_corner(core: &mut Core, client: &mut Client) -> serde_json::Value {
    move_to(core, (700.0, 500.0).into());
    let got = client.events_now(core);
    let mut moved = got.iter().map(|e| &e["params"]);
    let last = moved.rfind(|p| p["event"] == "input.moved");
    let last = last.unwrap_or_else(|| panic!("no move for the receiver: {got:?}"));
    json!([last["output"], last["position"]])
}

/// The transaction of xp that gives s1 with its drag to a client.
fn give_to(to: &str) -> serde_json::Value {
    json!({"commands": [
        {"remove": {"id": "s1", "to": to}},
        {"give_grab": {"surface": "s1", "to": to, "point": [30.0, 10.0]}},
    ]})
}

/// The pull-out of E3: xp gives the window with the drag to the window
/// manager that shows its face. The drag then leaves its pane: the
/// receiver gets the pointer of the screen, not the point that the pane
/// maps.
#[test]
fn a_drag_given_out_of_a_pane_goes_on_at_the_screen_point() {
    let (mut core, _wayland, mut wm, _other, mut xp) = pressed_title_behind_a_pane();
    xp.ok(&mut core, "scene.apply", give_to("wm"));
    let at = moved_to_the_screen_corner(&mut core, &mut wm);
    assert_eq!(at, json!(["test-1", [700.0, 500.0]]));
    click(&mut core, false);
    assert!(core.pointer_free());
}

/// Review of the pull-out: only the owner of the pane may take a drag out
/// of it. A drag given to another window manager stays behind the pane,
/// so that no client moves the pointer of a face to the screen for a
/// third one.
#[test]
fn a_drag_given_to_another_client_stays_behind_its_pane() {
    let (mut core, _wayland, _wm, mut other, mut xp) = pressed_title_behind_a_pane();
    xp.ok(&mut core, "scene.apply", give_to("other"));
    let at = moved_to_the_screen_corner(&mut core, &mut other);
    assert_eq!(at[0], "VIRTUAL-1", "{at}");
}
