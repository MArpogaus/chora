//! Tests of planes outside a 3D context: nodes whose transform is not a 2D
//! translation with a positive scale render and take input through the
//! homography of their plane (D75).

use base64::{Engine, engine::general_purpose::STANDARD};
use chora_scene::Transform;
use serde_json::{Value, json};
use smithay::{
    backend::input::{ButtonState, InputTime},
    utils::Point,
};

use crate::{
    input::{move_pointer, press_button},
    testing::{
        Client, Core,
        wayland::{PointerEvent, Toplevel, Wayland},
    },
};

const TOLERANCE: f64 = 0.5;
const LEFT: u32 = 272;

fn transform(spec: &Value) -> Transform {
    serde_json::from_value(spec.clone()).expect("a transform")
}

fn forward(transform: &Transform, u: f64, v: f64) -> Point<f64, smithay::utils::Logical> {
    let m = transform.matrix();
    let w = m[3] * u + m[7] * v + m[15];
    (
        (m[0] * u + m[4] * v + m[12]) / w,
        (m[1] * u + m[5] * v + m[13]) / w,
    )
        .into()
}

pub(super) fn move_to(core: &mut Core, at: Point<f64, smithay::utils::Logical>) {
    move_pointer(&mut core.state, at, InputTime::now());
}

pub(super) fn click(core: &mut Core, down: bool) {
    let state = if down {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    };
    press_button(&mut core.state, LEFT, state, InputTime::now(), false);
}

pub(super) fn close(got: (f64, f64), want: (f64, f64)) -> bool {
    (got.0 - want.0).abs() <= TOLERANCE && (got.1 - want.1).abs() <= TOLERANCE
}

pub(super) fn motions(wayland: &mut Wayland, core: &mut Core) -> Vec<(f64, f64)> {
    wayland.roundtrip(core);
    wayland
        .take_pointer_events()
        .into_iter()
        .filter_map(|event| match event {
            PointerEvent::Enter { at, .. } | PointerEvent::Motion { at } => Some(at),
            _ => None,
        })
        .collect()
}

/// A window of 100 by 80 as `s1` under a group `rig` with the transform
/// of `rig` and the translation (10, 5) of the window below it.
fn window_under(rig: &Value) -> (Core, Wayland, Toplevel, Transform, Client) {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let mut wm = core.named("wm");
    let commands = json!({"commands": [
        {"create": {"id": "rig", "kind": "group", "parent": "root",
                    "props": {"transform": rig}}},
        {"attach": {"surface": "s1", "parent": "rig"}},
        {"update": {"id": "s1", "props": {"transform": {"translate": [10, 5]}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let total = transform(rig).then(&transform(&json!({"translate": [10, 5]})));
    (core, wayland, window, total, wm)
}

fn hover_and_drag(rig: &Value) {
    let (mut core, mut wayland, _window, total, _wm) = window_under(rig);
    let inside = (50.0, 40.0);
    move_to(&mut core, forward(&total, inside.0, inside.1));
    let entered = motions(&mut wayland, &mut core);
    assert!(
        entered.last().is_some_and(|at| close(*at, inside)),
        "the enter is at {inside:?}: {entered:?}"
    );
    click(&mut core, true);
    for (u, v) in [(80.0, 60.0), (120.0, 90.0), (-30.0, 95.0), (20.0, 10.0)] {
        move_to(&mut core, forward(&total, u, v));
        let moved = motions(&mut wayland, &mut core);
        assert!(
            moved.last().is_some_and(|at| close(*at, (u, v))),
            "the drag is at {:?}: {moved:?}",
            (u, v)
        );
    }
    click(&mut core, false);
}

#[test]
fn a_rotated_window_gets_motion_and_a_drag_at_the_expected_point() {
    hover_and_drag(&json!({"translate": [200, 100], "rotate": 30}));
}

#[test]
fn a_window_under_a_plugin_matrix_gets_motion_and_a_drag_at_the_expected_point() {
    let matrix = [
        1.0, 0.1, 0.0, 0.002, -0.2, 1.0, 0.0, 0.001, 0.0, 0.0, 1.0, 0.0, 120.0, 60.0, 0.0, 1.0,
    ];
    hover_and_drag(&json!({"matrix": matrix}));
}

#[test]
fn a_mirrored_window_gets_the_point_it_shows() {
    hover_and_drag(&json!({"translate": [500, 100], "scale": [-1, 1]}));
}

pub(super) fn node_events(client: &mut Client, core: &mut Core) -> Vec<Value> {
    client
        .events_now(core)
        .into_iter()
        .map(|e| e["params"].clone())
        .filter(|e| {
            e["event"].as_str().is_some_and(|n| {
                n.starts_with("pointer.p")
                    || n.starts_with("pointer.m")
                    || n.starts_with("pointer.r")
            })
        })
        .collect()
}

pub(super) fn local(event: &Value) -> (f64, f64) {
    let local = &event["local"];
    (
        local[0].as_f64().expect("a local u"),
        local[1].as_f64().expect("a local v"),
    )
}

#[test]
fn a_drag_through_a_plane_keeps_the_last_point_when_the_ray_misses() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    let matrix = [
        1.0, 0.0, 0.0, 0.01, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let commands = json!({"commands": [
        {"create": {"id": "r", "kind": "rect", "parent": "root",
            "props": {"size": [100, 50], "color": [1, 1, 1, 1],
                      "transform": {"matrix": matrix}}}},
    ]});
    xp.ok(&mut core, "scene.apply", commands);
    xp.ok(&mut core, "events.subscribe", json!({"events": []}));
    let total = transform(&json!({"matrix": matrix}));
    move_to(&mut core, forward(&total, 30.0, 20.0));
    click(&mut core, true);
    move_to(&mut core, forward(&total, 70.0, 20.0));
    move_to(&mut core, (200.0, 15.0).into());
    move_to(&mut core, forward(&total, 60.0, 30.0));
    click(&mut core, false);
    let events = node_events(&mut xp, &mut core);
    let got: Vec<(&str, (f64, f64))> = events
        .iter()
        .map(|e| (e["event"].as_str().unwrap(), local(e)))
        .collect();
    let names: Vec<&str> = got.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names,
        [
            "pointer.pressed",
            "pointer.moved",
            "pointer.moved",
            "pointer.moved",
            "pointer.released"
        ]
    );
    for ((name, at), want) in got.iter().zip([
        (30.0, 20.0),
        (70.0, 20.0),
        (70.0, 20.0),
        (60.0, 30.0),
        (60.0, 30.0),
    ]) {
        assert!(close(*at, want), "{name} at {at:?}, expected {want:?}");
    }
    assert_eq!(events[2]["position"], json!([200.0, 15.0]));
}

#[test]
fn a_press_on_a_rotated_rect_tells_its_local_point() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    let spec = json!({"translate": [300, 100], "rotate": 45});
    let commands = json!({"commands": [
        {"create": {"id": "r", "kind": "rect", "parent": "root",
            "props": {"size": [100, 50], "color": [1, 1, 1, 1], "transform": spec}}},
    ]});
    xp.ok(&mut core, "scene.apply", commands);
    xp.ok(&mut core, "events.subscribe", json!({"events": []}));
    let total = transform(&spec);
    move_to(&mut core, forward(&total, 20.0, 10.0));
    click(&mut core, true);
    click(&mut core, false);
    let events = node_events(&mut xp, &mut core);
    assert!(close(local(&events[0]), (20.0, 10.0)), "{events:?}");
    let outside = forward(&total, 120.0, 10.0);
    move_to(&mut core, outside);
    click(&mut core, true);
    click(&mut core, false);
    assert!(
        node_events(&mut xp, &mut core).is_empty(),
        "no press outside"
    );
}

pub(super) fn decode(picture: &Value) -> (usize, Vec<u8>) {
    let png = STANDARD.decode(picture["png"].as_str().unwrap()).unwrap();
    let mut reader = png::Decoder::new(std::io::Cursor::new(png))
        .read_info()
        .unwrap();
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut rgba).unwrap();
    (info.width as usize, rgba)
}

#[test]
fn a_rotated_group_without_size_renders_its_children() {
    let mut core = Core::with_renderer();
    core.add_output("test-1", (96, 64));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach_filled(
        &mut core,
        &window,
        (20, 10),
        (
            wayland_client::protocol::wl_shm::Format::Argb8888,
            0xFF00_00FF,
        ),
    );
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(&mut core, "grants.set", grant);
    let quarter = |x: u32| json!({"translate": [x, 10], "rotate": 90});
    let commands = json!({"commands": [
        {"create": {"id": "turned", "kind": "group", "parent": "root",
                    "props": {"transform": quarter(40)}}},
        {"create": {"id": "r", "kind": "rect", "parent": "turned",
                    "props": {"size": [20, 10], "color": [1, 0, 0, 1]}}},
        {"create": {"id": "other", "kind": "group", "parent": "root",
                    "props": {"transform": quarter(80)}}},
        {"attach": {"surface": "s1", "parent": "other"}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let picture = wm.ok(&mut core, "screen.capture", json!({"output": "test-1"}));
    let (width, rgba) = decode(&picture);
    let pixel = |x: usize, y: usize| rgba[(y * width + x) * 4..][..4].to_vec();
    assert_eq!(pixel(35, 20), [255, 0, 0, 255], "the rect turned a quarter");
    assert_eq!(pixel(45, 20), [0, 0, 0, 255], "right of it");
    assert_eq!(
        pixel(75, 20),
        [0, 0, 255, 255],
        "the window turned a quarter"
    );
    assert_eq!(pixel(85, 20), [0, 0, 0, 255], "right of it");
}

#[test]
fn the_popup_area_of_a_window_under_a_plane_is_its_geometry() {
    let (core, _wayland, _window, _total, _wm) =
        window_under(&json!({"translate": [200, 100], "rotate": 30}));
    let root = core.state.windows["s1"]
        .toplevel()
        .expect("a toplevel")
        .wl_surface()
        .clone();
    let area = core.state.popup_area(&root).expect("an area");
    assert_eq!(
        (area.loc.x, area.loc.y, area.size.w, area.size.h),
        (0, 0, 100, 80)
    );
    let (core, _wayland, _window, _total, _wm) = window_under(&json!({"translate": [200, 100]}));
    let root = core.state.windows["s1"]
        .toplevel()
        .expect("a toplevel")
        .wl_surface()
        .clone();
    let flat = core.state.popup_area(&root).expect("an area");
    assert!(
        flat.size.w > 100,
        "a flat window gets the area of the output"
    );
}

#[test]
fn a_cell_with_a_point_behind_the_eye_shows_the_part_in_front() {
    let mut core = Core::with_renderer();
    core.add_output("test-1", (1280, 800));
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(&mut core, "grants.set", grant);
    let mesh = json!({"cols": 2, "rows": 1,
        "points": [[0, 0], [150, 0], [300, 0], [0, 200], [150, 200], [300, 200]],
        "depth": [0, 0, 200, 0, 0, 200]});
    let matrix = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, -0.01, 700.0, 450.0, 0.0, 1.0,
    ];
    let commands = json!({"commands": [
        {"create": {"id": "layer", "kind": "group", "parent": "root", "props": {
            "size": [300, 200], "mesh": mesh, "transform": {"matrix": matrix}}}},
        {"create": {"id": "red", "kind": "rect", "parent": "layer", "props": {
            "size": [150, 200], "color": [1, 0, 0, 1]}}},
        {"create": {"id": "green", "kind": "rect", "parent": "layer", "props": {
            "size": [150, 200], "color": [0, 1, 0, 1], "transform": {"translate": [150, 0]}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let picture = wm.ok(&mut core, "screen.capture", json!({"output": "test-1"}));
    let (width, rgba) = decode(&picture);
    let pixel = |x: usize, y: usize| rgba[(y * width + x) * 4..][..4].to_vec();
    assert_eq!(pixel(775, 500), [255, 0, 0, 255], "the near cell");
    assert_eq!(
        pixel(925, 500),
        [0, 255, 0, 255],
        "the front part of the far cell"
    );
}

#[test]
fn a_node_in_a_rotated_group_takes_input_only_inside_the_group() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    let spec = json!({"translate": [300, 100], "rotate": 30});
    let commands = json!({"commands": [
        {"create": {"id": "card", "kind": "group", "parent": "root",
            "props": {"size": [100, 60], "transform": spec}}},
        {"create": {"id": "wide", "kind": "rect", "parent": "card",
            "props": {"size": [200, 60], "color": [1, 1, 1, 1]}}},
    ]});
    xp.ok(&mut core, "scene.apply", commands);
    xp.ok(&mut core, "events.subscribe", json!({"events": []}));
    let total = transform(&spec);
    move_to(&mut core, forward(&total, 150.0, 30.0));
    click(&mut core, true);
    click(&mut core, false);
    assert!(
        node_events(&mut xp, &mut core).is_empty(),
        "outside the group"
    );
    move_to(&mut core, forward(&total, 50.0, 30.0));
    click(&mut core, true);
    click(&mut core, false);
    let events = node_events(&mut xp, &mut core);
    assert_eq!(events[0]["node"], "wide");
    assert!(close(local(&events[0]), (50.0, 30.0)), "{events:?}");
}

#[test]
fn a_plane_shown_behind_the_eye_takes_no_input() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    let matrix = [
        -1.0, 0.0, 0.0, -0.01, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let commands = json!({"commands": [
        {"create": {"id": "r", "kind": "rect", "parent": "root",
            "props": {"size": [300, 50], "color": [1, 1, 1, 1],
                      "transform": {"matrix": matrix}}}},
    ]});
    xp.ok(&mut core, "scene.apply", commands);
    xp.ok(&mut core, "events.subscribe", json!({"events": []}));
    let total = transform(&json!({"matrix": matrix}));
    let behind = forward(&total, 150.0, 25.0);
    assert!(close((behind.x, behind.y), (300.0, 50.0)), "{behind:?}");
    move_to(&mut core, behind);
    click(&mut core, true);
    click(&mut core, false);
    let events = node_events(&mut xp, &mut core);
    assert_eq!(events, Vec::<Value>::new());
}

fn press(core: &mut Core, linux: u32, down: bool) {
    let state = if down {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    };
    press_button(&mut core.state, linux, state, InputTime::now(), false);
}

#[test]
fn a_drag_stays_mapped_through_its_plane_while_another_button_is_held() {
    let (mut core, mut wayland, _window, total, _wm) =
        window_under(&json!({"translate": [200, 100], "rotate": 30}));
    move_to(&mut core, forward(&total, 50.0, 40.0));
    press(&mut core, LEFT, true);
    press(&mut core, LEFT + 1, true);
    press(&mut core, LEFT, false);
    move_to(&mut core, forward(&total, -30.0, 95.0));
    let moved = motions(&mut wayland, &mut core);
    assert!(
        moved.last().is_some_and(|at| close(*at, (-30.0, 95.0))),
        "{moved:?}"
    );
    press(&mut core, LEFT + 1, false);
}

#[test]
fn a_plane_that_the_layer_budget_leaves_out_takes_input_and_gives_it_to_nobody() {
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    wm.ok(&mut core, "output.add", json!({"spec": "128x96"}));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let plane = |id: &str, input: &str| {
        json!({"create": {"id": id, "kind": "rect", "parent": "root", "props": {
            "size": [4096, 4096], "color": [1, 1, 1, 1], "input": input,
            "transform": {"translate": [10, 10], "rotate": 10}}}})
    };
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "root"}},
        plane("p1", "accept"),
        plane("p2", "pass"),
        plane("p3", "pass"),
        plane("p4", "pass"),
        plane("p5", "pass"),
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    wm.ok(&mut core, "events.subscribe", json!({"events": []}));
    crate::backend::headless::render_all(&mut core.state);
    let left_out = &core.state.assets.frame_left_out["HEADLESS-1"];
    assert_eq!(left_out, &["p1"], "the budget leaves the lowest plane out");
    wm.take_events();
    motions(&mut wayland, &mut core);
    move_to(&mut core, (40.0, 40.0).into());
    click(&mut core, true);
    click(&mut core, false);
    let events = node_events(&mut wm, &mut core);
    assert_eq!(events, Vec::<Value>::new());
    assert_eq!(motions(&mut wayland, &mut core), Vec::<(f64, f64)>::new());
}

fn perspective_rect(core: &mut Core, xp: &mut Client, w_slope: f64) {
    let matrix = [
        1.0, 0.0, 0.0, w_slope, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let commands = json!({"commands": [
        {"create": {"id": "r", "kind": "rect", "parent": "root",
            "props": {"size": [300, 50], "color": [1, 1, 1, 1],
                      "transform": {"matrix": matrix}}}},
    ]});
    xp.ok(core, "scene.apply", commands);
    xp.ok(core, "events.subscribe", json!({"events": []}));
}

fn injector(core: &mut Core) -> Client {
    let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
    core.admin().ok(core, "grants.set", grant);
    core.named("agent")
}

#[test]
fn an_injected_click_on_a_node_under_a_perspective_plane_lands_on_it() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    perspective_rect(&mut core, &mut xp, 0.01);
    let mut agent = injector(&mut core);
    let at = json!({"pointer_move": {"node": "r", "point": [100.0, 25.0]}});
    agent.ok(
        &mut core,
        "input.inject",
        json!({"events": [at, {"button": {"state": "pressed"}}]}),
    );
    let events = node_events(&mut xp, &mut core);
    assert_eq!(events[0]["node"], "r", "{events:?}");
    assert!(close(local(&events[0]), (100.0, 25.0)), "{events:?}");
}

#[test]
fn an_injected_move_to_a_point_behind_the_eye_fails() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    perspective_rect(&mut core, &mut xp, -0.01);
    let mut agent = injector(&mut core);
    let at = json!({"pointer_move": {"node": "r", "point": [150.0, 25.0]}});
    agent.fails(&mut core, "input.inject", json!({"events": [at]}));
}

#[test]
fn a_plane_that_crosses_the_near_plane_has_a_box_that_reaches_past_it() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (300, 80));
    let mut xp = core.named("xp");
    perspective_rect(&mut core, &mut xp, -0.01);
    let attach = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "root"}},
        {"update": {"id": "s1", "props": {"transform": {"matrix": [
            1.0, 0.0, 0.0, -0.01, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0]}}}},
    ]});
    xp.ok(&mut core, "scene.apply", attach);
    let found = xp.ok(&mut core, "scene.query", json!({"selector": "id:r"}));
    let rect = found[0]["rects"]["HDMI-1"]
        .as_array()
        .expect("the part in front of the eye shows");
    assert!(rect[2].as_f64().unwrap() >= 800.0, "{rect:?}");
    let window = core.state.windows["s1"].clone();
    let total = transform(&json!({"matrix": [
        1.0, 0.0, 0.0, -0.01, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]}));
    let rect = crate::render::window_rect(&window, &total);
    assert!(rect.size.w >= 800.0, "{rect:?}");
}
