//! Tests of 3D contexts: a group with a size and a `perspective` sorts,
//! splits and clips the planes of its descendants, and the nearest plane
//! under a ray from the eye takes the input (D75).

use serde_json::{Value, json};

use super::planes::{click, close, decode, local, motions, move_to, node_events};
use crate::testing::{Core, wayland::Wayland};

/// The distance of the eye in front of the centre of the context.
const EYE: f64 = 800.0;

type Matrix = [f64; 16];

fn translate(x: f64, y: f64, z: f64) -> Matrix {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

fn turn_y(degrees: f64) -> Matrix {
    let (s, c) = degrees.to_radians().sin_cos();
    [
        c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

fn times(a: &Matrix, b: &Matrix) -> Matrix {
    std::array::from_fn(|i| {
        let (col, row) = (i / 4, i % 4);
        (0..4).map(|k| a[k * 4 + row] * b[col * 4 + k]).sum()
    })
}

fn chain(matrices: &[Matrix]) -> Matrix {
    matrices
        .iter()
        .fold(translate(0.0, 0.0, 0.0), |a, b| times(&a, b))
}

/// Where a point of a plane with a transform in a context of a size
/// shows: w = 1 + z / d around the centre of the context.
fn shown(m: &Matrix, [cx, cy]: [f64; 2], u: f64, v: f64) -> (f64, f64) {
    let at: Vec<f64> = (0..3)
        .map(|r| m[r] * u + m[4 + r] * v + m[12 + r])
        .collect();
    let w = 1.0 + at[2] / EYE;
    (cx + (at[0] - cx) / w, cy + (at[1] - cy) / w)
}

fn space(core: &mut Core, size: [f64; 2]) -> crate::testing::Client {
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(core, "grants.set", grant);
    let commands = json!({"commands": [
        {"create": {"id": "space", "kind": "group", "parent": "root",
                    "props": {"size": size, "perspective": EYE}}},
        {"create": {"id": "eye", "kind": "group", "parent": "space"}},
    ]});
    wm.ok(core, "scene.apply", commands);
    wm
}

fn rect(id: &str, size: [f64; 2], color: [f64; 4], matrix: &Matrix) -> Value {
    json!({"create": {"id": id, "kind": "rect", "parent": "eye", "props": {
        "size": size, "color": color, "transform": {"matrix": matrix}}}})
}

const RED: [u8; 4] = [255, 0, 0, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];

fn capture(
    core: &mut Core,
    wm: &mut crate::testing::Client,
) -> impl Fn(f64, f64) -> [u8; 4] + use<> {
    let picture = wm.ok(core, "screen.capture", json!({"output": "test-1"}));
    let (width, rgba) = decode(&picture);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    move |x: f64, y: f64| {
        let at = (y as usize * width + x as usize) * 4;
        rgba[at..at + 4].try_into().expect("a pixel")
    }
}

/// The rect of 200 by 100 shows inside and not outside where the math of
/// the context puts it: 4 points inside, 4 outside.
fn assert_shows(pixel: &impl Fn(f64, f64) -> [u8; 4], m: &Matrix) {
    let centre = [320.0, 240.0];
    let inside = [(20.0, 10.0), (180.0, 10.0), (180.0, 90.0), (20.0, 90.0)];
    let outside = [(-20.0, 50.0), (220.0, 50.0), (100.0, -15.0), (100.0, 115.0)];
    for ((u, v), want) in inside
        .iter()
        .map(|p| (p, RED))
        .chain(outside.iter().map(|p| (p, BLACK)))
    {
        let (x, y) = shown(m, centre, *u, *v);
        assert_eq!(pixel(x, y), want, "the point {u}, {v} at {x:.1}, {y:.1}");
    }
}

fn turned_rect() -> Matrix {
    chain(&[
        translate(320.0, 240.0, 200.0),
        turn_y(60.0),
        translate(-100.0, -50.0, 0.0),
    ])
}

#[test]
fn a_rect_at_depth_turned_about_y_shows_where_the_math_puts_it() {
    let mut core = Core::with_renderer();
    core.add_output("test-1", (640, 480));
    let mut wm = space(&mut core, [640.0, 480.0]);
    let m = turned_rect();
    let commands = json!({"commands": [rect("r", [200.0, 100.0], [1.0, 0.0, 0.0, 1.0], &m)]});
    wm.ok(&mut core, "scene.apply", commands);
    assert_shows(&capture(&mut core, &mut wm), &m);
}

#[test]
fn one_update_of_the_camera_moves_the_plane() {
    let mut core = Core::with_renderer();
    core.add_output("test-1", (640, 480));
    let mut wm = space(&mut core, [640.0, 480.0]);
    let m = turned_rect();
    let commands = json!({"commands": [rect("r", [200.0, 100.0], [1.0, 0.0, 0.0, 1.0], &m)]});
    wm.ok(&mut core, "scene.apply", commands);
    assert_shows(&capture(&mut core, &mut wm), &m);
    let camera = chain(&[translate(-60.0, 20.0, -100.0), turn_y(-10.0)]);
    let update = json!({"commands": [
        {"update": {"id": "eye", "props": {"transform": {"matrix": camera}}}},
    ]});
    wm.ok(&mut core, "scene.apply", update);
    assert_shows(&capture(&mut core, &mut wm), &times(&camera, &m));
}

/// Two rects that cross in the middle: the blue one, later in the scene,
/// is in front on the left only.
fn crossing(core: &mut Core) -> crate::testing::Client {
    core.add_output("test-1", (640, 480));
    let mut wm = space(core, [640.0, 480.0]);
    let at = |degrees| {
        chain(&[
            translate(320.0, 240.0, 0.0),
            turn_y(degrees),
            translate(-150.0, -100.0, 0.0),
        ])
    };
    let commands = json!({"commands": [
        rect("a", [300.0, 200.0], [0.0, 1.0, 0.0, 1.0], &at(30.0)),
        rect("b", [300.0, 200.0], [0.0, 0.0, 1.0, 1.0], &at(-30.0)),
    ]});
    wm.ok(core, "scene.apply", commands);
    wm
}

#[test]
fn two_crossing_planes_each_show_in_front_on_their_half() {
    let mut core = Core::with_renderer();
    let mut wm = crossing(&mut core);
    let pixel = capture(&mut core, &mut wm);
    assert_eq!(
        pixel(260.0, 240.0),
        [0, 0, 255, 255],
        "b in front on the left"
    );
    assert_eq!(
        pixel(380.0, 240.0),
        [0, 255, 0, 255],
        "a in front on the right"
    );
    assert!(core.state.assets.stats.splits >= 2, "the planes split");
}

#[test]
fn two_crossing_planes_each_take_the_input_on_their_half() {
    let mut core = Core::new();
    crossing(&mut core);
    let under = |core: &Core, x: f64, y: f64| {
        let spot = core.state.spot((x, y).into()).expect("a spot");
        crate::render::node_under(&core.state, &spot)
    };
    assert_eq!(under(&core, 260.0, 240.0).as_deref(), Some("b"));
    assert_eq!(under(&core, 380.0, 240.0).as_deref(), Some("a"));
    assert_eq!(under(&core, 20.0, 20.0), None, "the empty context");
}

#[test]
fn a_window_in_a_context_takes_input_through_the_ray() {
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
    let centre = [400.0, 300.0];
    let (sx, sy) = shown(&m, centre, 25.0, 40.0);
    let commands = json!({"commands": [
        rect("shield", [20.0, 20.0], [1.0, 1.0, 1.0, 1.0],
             &translate(sx - 10.0, sy - 10.0, 0.0)),
        {"attach": {"surface": "s1", "parent": "eye"}},
        {"update": {"id": "s1", "props": {"transform": {"matrix": m}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    wm.ok(&mut core, "events.subscribe", json!({"events": []}));
    motions(&mut wayland, &mut core);
    let (x, y) = shown(&m, centre, 75.0, 40.0);
    move_to(&mut core, (x, y).into());
    let entered = motions(&mut wayland, &mut core);
    assert!(
        entered.last().is_some_and(|at| close(*at, (75.0, 40.0))),
        "the window gets the point of the ray: {entered:?}"
    );
    click(&mut core, true);
    click(&mut core, false);
    move_to(&mut core, (sx, sy).into());
    click(&mut core, true);
    click(&mut core, false);
    let events = node_events(&mut wm, &mut core);
    let pressed: Vec<&Value> = events
        .iter()
        .filter(|e| e["event"] == "pointer.pressed")
        .collect();
    let nodes: Vec<&Value> = pressed.iter().map(|e| &e["node"]).collect();
    assert_eq!(nodes, [&json!("s1"), &json!("shield")], "the nearer plane");
    assert!(close(local(pressed[0]), (75.0, 40.0)), "{pressed:?}");
    assert!(close(local(pressed[1]), (10.0, 10.0)), "{pressed:?}");
}

#[test]
fn the_rects_of_a_query_follow_the_projection() {
    let mut core = Core::new();
    core.add_output("test-1", (640, 480));
    let mut wm = space(&mut core, [640.0, 480.0]);
    let m = turned_rect();
    let commands = json!({"commands": [rect("r", [200.0, 100.0], [1.0, 0.0, 0.0, 1.0], &m)]});
    wm.ok(&mut core, "scene.apply", commands);
    let found = wm.ok(&mut core, "scene.query", json!({"selector": "id:r"}));
    let got: Vec<f64> = found[0]["rects"]["test-1"]
        .as_array()
        .expect("a rect")
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let corners = [(0.0, 0.0), (200.0, 0.0), (200.0, 100.0), (0.0, 100.0)]
        .map(|(u, v)| shown(&m, [320.0, 240.0], u, v));
    let left = corners.iter().map(|c| c.0).fold(f64::MAX, f64::min);
    let right = corners.iter().map(|c| c.0).fold(f64::MIN, f64::max);
    let top = corners.iter().map(|c| c.1).fold(f64::MAX, f64::min);
    let bottom = corners.iter().map(|c| c.1).fold(f64::MIN, f64::max);
    for (got, want) in got.iter().zip([left, top, right - left, bottom - top]) {
        assert!((got - want).abs() < 1e-6, "{got} != {want}");
    }
}

#[test]
fn a_shader_in_a_context_reads_depth_and_facing() {
    let dir = std::env::temp_dir().join(format!("chora-facing-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shader = dir.join("facing.frag");
    std::fs::write(
        &shader,
        "precision mediump float;\nuniform float depth;\nuniform float facing;\n\
         void main() { gl_FragColor = vec4(depth * 0.5, facing, 0.0, 1.0); }\n",
    )
    .unwrap();
    let mut core = Core::with_renderer();
    core.add_output("test-1", (640, 480));
    let mut wm = space(&mut core, [640.0, 480.0]);
    let flat = translate(270.0, 190.0, 0.0);
    let commands = json!({"commands": [
        {"create": {"id": "glow", "kind": "shader", "parent": "eye", "props": {
            "size": [100, 100], "shader": shader, "transform": {"matrix": flat}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let near = |got: [u8; 4], want: [u8; 4]| {
        got.iter()
            .zip(want)
            .all(|(g, w)| (i16::from(*g) - i16::from(w)).abs() <= 3)
    };
    let got = capture(&mut core, &mut wm)(320.0, 240.0);
    assert!(near(got, [128, 255, 0, 255]), "at 1:1: {got:?}");
    let far = chain(&[
        translate(320.0, 240.0, EYE),
        turn_y(60.0),
        translate(-50.0, -50.0, 0.0),
    ]);
    let update = json!({"commands": [
        {"update": {"id": "glow", "props": {"transform": {"matrix": far}}}},
    ]});
    wm.ok(&mut core, "scene.apply", update);
    let got = capture(&mut core, &mut wm)(320.0, 240.0);
    assert!(
        near(got, [255, 128, 0, 255]),
        "twice as far, turned 60: {got:?}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

fn turn_x(degrees: f64) -> Matrix {
    let (s, c) = degrees.to_radians().sin_cos();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// A floor one pixel below the eye from 200 behind the eye to 1000 in
/// front of the centre, and a blue wall far behind it. The floor shows
/// below the horizon down to w = 1/64, at y = 240 + 64 = 304; below that
/// the near plane cuts it and the wall shows.
fn floor(core: &mut Core) -> crate::testing::Client {
    core.add_output("test-1", (640, 480));
    let mut wm = space(core, [640.0, 480.0]);
    let floor = chain(&[translate(-680.0, 241.0, -1000.0), turn_x(90.0)]);
    let commands = json!({"commands": [
        rect("wall", [4000.0, 4000.0], [0.0, 0.0, 1.0, 1.0],
             &translate(-1700.0, -1700.0, 2000.0)),
        rect("floor", [2000.0, 2000.0], [1.0, 0.0, 0.0, 1.0], &floor),
    ]});
    wm.ok(core, "scene.apply", commands);
    wm
}

#[test]
fn a_plane_through_the_near_plane_shows_and_takes_input_only_in_front_of_it() {
    let mut core = Core::with_renderer();
    let mut wm = floor(&mut core);
    let pixel = capture(&mut core, &mut wm);
    assert_eq!(
        pixel(320.0, 280.0),
        RED,
        "the floor in front of the near plane"
    );
    assert_eq!(
        pixel(320.0, 400.0),
        [0, 0, 255, 255],
        "the wall past the cut"
    );
    let under = |core: &Core, x: f64, y: f64| {
        let spot = core.state.spot((x, y).into()).expect("a spot");
        crate::render::node_hit(&core.state, &spot)
    };
    let (node, at) = under(&core, 320.0, 280.0).expect("the floor");
    assert_eq!(node, "floor");
    assert!(close((at.x, at.y), (1000.0, 220.0)), "{at:?}");
    assert_eq!(
        under(&core, 320.0, 400.0).map(|(node, _)| node).as_deref(),
        Some("wall")
    );
    let found = wm.ok(&mut core, "scene.query", json!({"selector": "id:floor"}));
    let height = found[0]["rects"]["test-1"][3].as_f64().expect("a rect");
    assert!(
        (63.0..64.0).contains(&height),
        "the box ends at the cut: {height}"
    );
}

#[test]
fn a_drag_toward_the_horizon_tells_a_clamped_local_point() {
    let mut core = Core::new();
    let mut wm = floor(&mut core);
    wm.ok(&mut core, "events.subscribe", json!({"events": []}));
    move_to(&mut core, (320.0, 280.0).into());
    click(&mut core, true);
    move_to(&mut core, (320.0, 240.000_01).into());
    click(&mut core, false);
    let events = node_events(&mut wm, &mut core);
    let moved = events
        .iter()
        .find(|e| e["event"] == "pointer.moved")
        .expect("a motion");
    assert!(close(local(moved), (1000.0, 1e6)), "{moved:?}");
}

#[test]
fn a_piece_that_the_budget_leaves_out_takes_input_and_gives_it_to_nobody() {
    let mut core = Core::with_renderer();
    let mut wm = space(&mut core, [640.0, 480.0]);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let at = |degrees| {
        chain(&[
            translate(320.0, 240.0, 0.0),
            turn_y(degrees),
            translate(-2048.0, -2048.0, 0.0),
        ])
    };
    let big = |id: &str, degrees: f64| {
        json!({"create": {"id": id, "kind": "rect", "parent": "eye", "props": {
            "size": [4096, 4096], "color": [1, 1, 1, 1], "radius": 8,
            "clip": {"radius": 8}, "transform": {"matrix": at(degrees)}}}})
    };
    let commands = json!({"commands": [big("a", 30.0), big("b", -30.0)]});
    wm.ok(&mut core, "scene.apply", commands);
    crate::backend::headless::render_all(&mut core.state);
    let left_out = core.state.assets.frame_left_out["HEADLESS-1"].clone();
    assert_eq!(left_out.len(), 1, "{left_out:?}");
    let (hidden, shown, hidden_at, shown_at) = if left_out[0] == "a" {
        ("a", "b", 380.0, 260.0)
    } else {
        ("b", "a", 260.0, 380.0)
    };
    let under = |core: &Core, x: f64| {
        let spot = core.state.spot((x, 240.0).into()).expect("a spot");
        crate::render::node_under(&core.state, &spot)
    };
    assert_eq!(under(&core, shown_at).as_deref(), Some(shown));
    assert_eq!(
        under(&core, hidden_at),
        None,
        "{hidden} shows nothing there"
    );
}
