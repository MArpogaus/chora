//! A shader node with `forward` passes the pointer into the virtual
//! output that it reads: the pointer is where the pane says it is.

use serde_json::{Value, json};
use smithay::{
    backend::input::{ButtonState, InputTime, KeyState},
    input::keyboard::Keycode,
};

use super::{Faced, PANE, SIDE, SLOT_1, face_shader, faced};
use crate::{
    input,
    testing::{Client, wayland::PointerEvent},
};

const LEFT: u32 = 272;
const SUPER: u32 = 125;
/// The pane shows the face at twice its size.
const ZOOM: i32 = 2;

/// A face behind a pane of twice its size, which forwards or not.
fn paned(forward: bool) -> Faced {
    let mut faced = faced();
    let side = SIDE * ZOOM;
    let props = json!({"size": [side, side], "shader": face_shader(&faced.dir),
        "uniforms": {"face": {"output": "VIRTUAL-1"}}, "forward": forward,
        "transform": {"translate": [PANE.0, PANE.1]}});
    let create = json!({"id": "pane", "kind": "shader", "parent": "root", "props": props});
    let commands = json!({"commands": [{"create": create}]});
    faced.construct.ok(&mut faced.core, "scene.apply", commands);
    faced.frames(2);
    faced
}

/// Moves the pointer to a point of the face, through the pane.
fn point_at(faced: &mut Faced, x: i32, y: i32) {
    let on_screen = (f64::from(PANE.0 + x * ZOOM), f64::from(PANE.1 + y * ZOOM));
    input::move_pointer(&mut faced.core.state, on_screen.into(), InputTime::now());
}

fn button(faced: &mut Faced, state: ButtonState, injected: bool) {
    let time = InputTime::now();
    input::press_button(&mut faced.core.state, LEFT, state, time, injected);
}

fn click(faced: &mut Faced) {
    button(faced, ButtonState::Pressed, false);
    button(faced, ButtonState::Released, false);
}

fn events_of(client: &mut Client, faced: &mut Faced, name: &str) -> Vec<Value> {
    let events = client.events_now(&mut faced.core).into_iter();
    events
        .map(|e| e["params"].clone())
        .filter(|e| e["event"] == name)
        .collect()
}

fn subscribe(client: &mut Client, faced: &mut Faced, events: &[&str]) {
    let subscribe = json!({"events": events});
    client.ok(&mut faced.core, "events.subscribe", subscribe);
}

#[test]
fn a_click_on_a_scaled_pane_reaches_the_window_at_the_mapped_point() {
    let mut faced = paned(true);
    point_at(&mut faced, 10, 20);
    click(&mut faced);
    faced.wayland.roundtrip(&mut faced.core);
    let events = faced.wayland.take_pointer_events();
    let [enter, press, release] = &events[..] else {
        panic!("an enter, a press and a release: {events:?}");
    };
    let at_the_face = PointerEvent::Enter {
        surface: wayland_client::Proxy::id(&faced.window.surface),
        at: (10.0, 20.0),
    };
    assert_eq!(*enter, at_the_face);
    assert!(matches!(press, PointerEvent::Button { pressed: true, .. }));
    assert!(matches!(
        release,
        PointerEvent::Button { pressed: false, .. }
    ));
}

#[test]
fn the_pointer_hovers_a_node_on_the_virtual_output() {
    let mut faced = paned(true);
    let mut construct = faced.core.named("construct");
    subscribe(&mut construct, &mut faced, &["pointer.entered"]);
    point_at(&mut faced, 2, 2);
    let entered = events_of(&mut construct, &mut faced, "pointer.entered");
    assert_eq!(entered.len(), 1, "{entered:?}");
    assert_eq!(entered[0]["node"], "corner");
}

#[test]
fn a_grabbed_button_carries_the_virtual_output() {
    let mut faced = paned(true);
    let mut xp = faced.core.named("xp");
    let grab = json!({"binding": "Super+BTN_LEFT"});
    xp.ok(&mut faced.core, "input.grab", grab);
    point_at(&mut faced, 10, 20);
    let code = Keycode::new(SUPER + 8);
    let time = InputTime::now();
    input::key(&mut faced.core.state, code, KeyState::Pressed, time, false);
    button(&mut faced, ButtonState::Pressed, false);
    let pressed = events_of(&mut xp, &mut faced, "input.button");
    let at = [f64::from(SLOT_1) + 10.0, f64::from(SLOT_1) + 20.0];
    assert_eq!(pressed.len(), 1, "{pressed:?}");
    assert_eq!(pressed[0]["output"], "VIRTUAL-1");
    assert_eq!(pressed[0]["local"], json!([10.0, 20.0]));
    assert_eq!(pressed[0]["position"], json!(at));
}

#[test]
fn a_pane_without_forward_gives_the_press_to_its_owner() {
    let mut faced = paned(false);
    let mut construct = faced.core.named("construct");
    subscribe(&mut construct, &mut faced, &["pointer.pressed"]);
    point_at(&mut faced, 10, 20);
    click(&mut faced);
    let pressed = events_of(&mut construct, &mut faced, "pointer.pressed");
    assert_eq!(pressed.len(), 1, "{pressed:?}");
    assert_eq!(pressed[0]["node"], "pane");
    assert_eq!(pressed[0]["local"], json!([20.0, 40.0]));
    faced.wayland.roundtrip(&mut faced.core);
    assert_eq!(faced.wayland.take_pointer_events(), [], "the window");
}

fn spot_output(faced: &Faced) -> String {
    let state = &faced.core.state;
    let spot = state.spot(state.pointer_layout).expect("a spot");
    spot.output.name()
}

#[test]
fn a_locked_session_forwards_nothing() {
    let mut faced = paned(true);
    point_at(&mut faced, 10, 20);
    assert_eq!(spot_output(&faced), "VIRTUAL-1");
    let _lock = faced.wayland.lock(&mut faced.core);
    assert_eq!(spot_output(&faced), "screen");
}

/// The pointer of an injected event and the mark of the injection stay
/// on the screen.
#[test]
fn the_screen_spot_stays_on_the_screen() {
    let mut faced = paned(true);
    point_at(&mut faced, 10, 20);
    let state = &faced.core.state;
    let spot = state.screen_spot(state.pointer_layout).expect("a spot");
    assert_eq!(spot.output.name(), "screen");
}

#[test]
fn a_pane_of_another_client_forwards_nothing() {
    let mut faced = paned(true);
    let remove = json!({"output": "VIRTUAL-1"});
    faced.construct.ok(&mut faced.core, "output.remove", remove);
    let mut other = faced.core.named("xp");
    let add = json!({"spec": "32x32", "virtual": true});
    other.ok(&mut faced.core, "output.add", add);
    point_at(&mut faced, 10, 20);
    assert_eq!(spot_output(&faced), "screen");
}

#[test]
fn a_new_toplevel_under_a_pane_goes_to_the_manager_on_the_virtual_output() {
    let mut faced = paned(true);
    let claims = [
        ("desk", [0, 0, 128, 96]),
        ("xp", [SLOT_1, SLOT_1, SIDE, SIDE]),
    ];
    let _managers: Vec<Client> = claims
        .into_iter()
        .map(|(name, claim)| {
            let mut manager = faced.core.named(name);
            let group = json!({"commands": [{"create": {"id": name, "kind": "group",
                "parent": "root", "props": {"claim": claim}}}]});
            manager.ok(&mut faced.core, "scene.apply", group);
            manager
        })
        .collect();
    point_at(&mut faced, 10, 20);
    faced.wayland.toplevel(&mut faced.core, "probe");
    let scene = &faced.core.state.scene;
    let to = scene.node("s2").and_then(|n| n.to.clone());
    assert_eq!(to.as_deref(), Some("xp"));
}

#[test]
fn a_node_press_through_a_pane_stamps_an_act_and_an_injected_one_does_not() {
    let mut faced = paned(true);
    point_at(&mut faced, 2, 2);
    button(&mut faced, ButtonState::Pressed, true);
    button(&mut faced, ButtonState::Released, true);
    assert!(faced.core.state.last_act.is_none(), "an injected press");
    click(&mut faced);
    assert!(faced.core.state.last_act.is_some(), "the press of a person");
}

/// A press on a node of the virtual output, dragged off the pane: the
/// moves stay on the virtual output, and the release tells where the
/// pointer is on the screen, for the window manager under it.
#[test]
fn a_node_drag_stays_pinned_and_its_release_reports_the_screen_point() {
    let mut faced = paned(true);
    let mut construct = faced.core.named("construct");
    let wanted = ["pointer.pressed", "pointer.moved", "pointer.released"];
    subscribe(&mut construct, &mut faced, &wanted);
    point_at(&mut faced, 2, 2);
    button(&mut faced, ButtonState::Pressed, false);
    input::move_pointer(&mut faced.core.state, (4.0, 90.0).into(), InputTime::now());
    button(&mut faced, ButtonState::Released, false);
    let events = construct.events_now(&mut faced.core);
    let of = |name: &str| {
        let found = events.iter().find(|e| e["params"]["event"] == name);
        found.unwrap_or_else(|| panic!("{name}: {events:?}"))["params"].clone()
    };
    assert_eq!(of("pointer.pressed")["node"], "corner");
    let off_the_pane = f64::from(PANE.0 - 4) / f64::from(ZOOM);
    let below = f64::from(90 - PANE.1) / f64::from(ZOOM);
    let on_the_face = [f64::from(SLOT_1) - off_the_pane, f64::from(SLOT_1) + below];
    assert_eq!(of("pointer.moved")["position"], json!(on_the_face));
    assert_eq!(of("pointer.released")["position"], json!([4.0, 90.0]));
    assert_eq!(spot_output(&faced), "screen", "the press ended");
}

/// A press that starts on the screen stays there while it crosses a pane.
#[test]
fn a_drag_from_the_screen_does_not_enter_a_pane() {
    let mut faced = paned(true);
    input::move_pointer(&mut faced.core.state, (4.0, 4.0).into(), InputTime::now());
    button(&mut faced, ButtonState::Pressed, false);
    point_at(&mut faced, 10, 20);
    assert_eq!(spot_output(&faced), "screen", "while the button is down");
    button(&mut faced, ButtonState::Released, false);
    assert_eq!(spot_output(&faced), "VIRTUAL-1", "after the release");
}

#[test]
fn a_point_far_off_the_pane_stays_in_the_gap_of_the_slot() {
    let mut faced = paned(true);
    let small = json!({"commands": [{"update": {"id": "pane", "props": {"size": [8, 8]}}}]});
    faced.construct.ok(&mut faced.core, "scene.apply", small);
    let on_the_pane = (f64::from(PANE.0 + 2), f64::from(PANE.1 + 2));
    input::move_pointer(&mut faced.core.state, on_the_pane.into(), InputTime::now());
    button(&mut faced, ButtonState::Pressed, false);
    let state = &faced.core.state;
    let side = f64::from(SIDE);
    for (corner, clamped) in [((0.0, 0.0), -side), ((127.0, 95.0), 2.0 * side)] {
        let spot = state.spot(corner.into()).expect("a spot");
        assert_eq!(spot.output.name(), "VIRTUAL-1");
        assert_eq!((spot.local.x, spot.local.y), (clamped, clamped));
    }
}
