//! Tests of the cost of planes in a 3D context (D75, section 2.4): a
//! static context renders nothing, a walk allocates nothing, the order of
//! the planes is computed once, and live windows on a ring show their
//! newest color.

use serde_json::{Value, json};
use wayland_client::protocol::wl_shm::Format;

use super::context::{
    EYE, Matrix, RED, capture, chain, rect, shown, space, translate, turn_x, turn_y,
};
use super::planes::decode;
use crate::{
    backend::headless::render_all,
    render::{RenderStats, node_hit, node_under},
    testing::{
        Client, Core,
        wayland::{Toplevel, Wayland},
    },
};

const OUTPUT: &str = "HEADLESS-1";
const SIZE: [f64; 2] = [640.0, 480.0];

fn world() -> (Core, Client) {
    let mut core = Core::with_renderer();
    let mut wm = space(&mut core, SIZE);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    (core, wm)
}

fn shot(core: &mut Core, wm: &mut Client) -> impl Fn(f64, f64) -> [u8; 4] + use<> {
    let picture = wm.ok(core, "screen.capture", json!({"output": OUTPUT}));
    let (width, rgba) = decode(&picture);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    move |x: f64, y: f64| {
        let at = (y as usize * width + x as usize) * 4;
        rgba[at..at + 4].try_into().expect("a pixel")
    }
}

#[allow(clippy::print_stdout)]
fn report(line: &str) {
    println!("{line}");
}

fn frames(core: &mut Core, count: usize) {
    for _ in 0..count {
        render_all(&mut core.state);
    }
}

fn stats(core: &Core) -> RenderStats {
    core.state.assets.stats.clone()
}

fn turned(degrees: f64, at: [f64; 3]) -> Matrix {
    chain(&[
        translate(at[0], at[1], at[2]),
        turn_y(degrees),
        translate(-50.0, -40.0, 0.0),
    ])
}

fn window(core: &mut Core, wayland: &mut Wayland, size: (i32, i32), pixel: u32) -> Toplevel {
    let toplevel = wayland.toplevel(core, "window");
    wayland.attach_filled(core, &toplevel, size, (Format::Argb8888, pixel));
    toplevel
}

fn static_scene(core: &mut Core, wm: &mut Client) -> Wayland {
    let mut wayland = Wayland::connect(core);
    window(core, &mut wayland, (100, 80), 0xFF00_00FF);
    let rounded = json!({"create": {"id": "round", "kind": "rect", "parent": "eye",
        "props": {"size": [120, 90], "color": [0, 1, 0, 1], "radius": 12,
                  "transform": {"matrix": turned(25.0, [160.0, 120.0, 0.0])}}}});
    let commands = json!({"commands": [
        rect("a", [120.0, 90.0], [1.0, 0.0, 0.0, 1.0], &turned(30.0, [480.0, 120.0, 0.0])),
        rect("b", [120.0, 90.0], [0.0, 0.0, 1.0, 1.0], &turned(-30.0, [480.0, 120.0, 0.0])),
        rounded,
        {"attach": {"surface": "s1", "parent": "eye"}},
        {"update": {"id": "s1", "props": {"transform":
            {"matrix": turned(35.0, [320.0, 340.0, 100.0])}}}},
    ]});
    wm.ok(core, "scene.apply", commands);
    wayland
}

#[test]
fn a_static_context_renders_no_pass_after_frame_1() {
    let (mut core, mut wm) = world();
    let _wayland = static_scene(&mut core, &mut wm);
    frames(&mut core, 1);
    let first = stats(&core);
    assert!(first.offscreen() > 0, "frame 1 renders the planes");
    frames(&mut core, 30);
    let after = stats(&core);
    assert_eq!(
        after.offscreen(),
        first.offscreen(),
        "no pass after frame 1"
    );
    assert_eq!(after.damaged_frames, first.damaged_frames, "no damage");
    assert_eq!(after.textures_allocated, first.textures_allocated);
}

#[test]
fn a_camera_walk_allocates_no_texture_after_the_first_frame() {
    let (mut core, mut wm) = world();
    let _wayland = static_scene(&mut core, &mut wm);
    frames(&mut core, 1);
    let first = stats(&core);
    for step in 1..=20 {
        let s = f64::from(step);
        let camera = chain(&[translate(-2.0 * s, s, -2.0 * s), turn_y(-0.3 * s)]);
        let update = json!({"commands": [
            {"update": {"id": "eye", "props": {"transform": {"matrix": camera}}}},
        ]});
        wm.ok(&mut core, "scene.apply", update);
        frames(&mut core, 1);
    }
    let after = stats(&core);
    assert_eq!(after.textures_allocated, first.textures_allocated);
    assert_eq!(
        after.offscreen(),
        first.offscreen(),
        "a walk renders no pass"
    );
}

#[test]
fn the_split_order_is_computed_once_per_scene_generation() {
    let (mut core, mut wm) = world();
    let _wayland = static_scene(&mut core, &mut wm);
    let hit = |core: &Core| {
        let spot = core.state.spot((480.0, 120.0).into()).expect("a spot");
        node_under(&core.state, &spot)
    };
    let orders = |core: &Core| core.state.assets.stats.orders.get();
    frames(&mut core, 1);
    assert!(hit(&core).is_some());
    let before = orders(&core);
    frames(&mut core, 10);
    for _ in 0..10 {
        assert!(hit(&core).is_some());
    }
    assert_eq!(orders(&core), before, "frames and hits share one order");
    let update = json!({"commands": [
        {"update": {"id": "a", "props": {"transform":
            {"matrix": turned(20.0, [480.0, 120.0, 0.0])}}}},
    ]});
    wm.ok(&mut core, "scene.apply", update);
    frames(&mut core, 10);
    for _ in 0..10 {
        assert!(hit(&core).is_some());
    }
    assert_eq!(orders(&core), before + 1, "one more for the new generation");
}

fn ring_matrix(index: u32) -> Matrix {
    let angle = f64::from(index) * 45.0;
    let (s, c) = angle.to_radians().sin_cos();
    turned(
        20.0,
        [
            320.0 + 220.0 * s,
            60.0 + f64::from(index) * 52.0,
            200.0 * (1.0 - c),
        ],
    )
}

#[test]
fn eight_live_windows_on_a_ring_show_their_last_color_and_none_drops() {
    let (mut core, mut wm) = world();
    let mut wayland = Wayland::connect(&mut core);
    let toplevels: Vec<_> = (0..8)
        .map(|_| window(&mut core, &mut wayland, (100, 80), 0xFF00_0000))
        .collect();
    let mut commands: Vec<Value> = Vec::new();
    for index in 0..8 {
        commands.push(json!({"attach": {"surface": format!("s{}", index + 1), "parent": "eye"}}));
        commands.push(
            json!({"update": {"id": format!("s{}", index + 1), "props": {
            "transform": {"matrix": ring_matrix(index)}}}}),
        );
    }
    wm.ok(&mut core, "scene.apply", json!({"commands": commands}));
    frames(&mut core, 1);
    let mut times = Vec::new();
    let color =
        |frame: u32, index: u32| 0xFF00_0000 | (frame * 20) << 16 | (index * 30) << 8 | 0x40;
    for frame in 1..=10 {
        for (index, toplevel) in (0..).zip(&toplevels) {
            wayland.attach_filled(
                &mut core,
                toplevel,
                (100, 80),
                (Format::Argb8888, color(frame, index)),
            );
        }
        let start = std::time::Instant::now();
        frames(&mut core, 1);
        times.push(start.elapsed());
        let dropped = &core.state.assets.frame_left_out[OUTPUT];
        assert!(dropped.is_empty(), "frame {frame} dropped {dropped:?}");
    }
    let mean = times.iter().sum::<std::time::Duration>() / 10;
    report(&format!(
        "ring of 8 live windows: mean frame {mean:?}, frames {times:?}"
    ));
    let start = std::time::Instant::now();
    frames(&mut core, 10);
    report(&format!(
        "ring of 8 idle windows: mean frame {:?}",
        start.elapsed() / 10
    ));
    let pixel = shot(&mut core, &mut wm);
    for index in 0..8 {
        let (x, y) = shown(&ring_matrix(index), [320.0, 240.0], 50.0, 40.0);
        let want = color(10, index);
        let want = [
            u8::try_from(want >> 16 & 255).unwrap(),
            u8::try_from(want >> 8 & 255).unwrap(),
            u8::try_from(want & 255).unwrap(),
            255,
        ];
        assert_eq!(
            pixel(x, y),
            want,
            "the centre of window {index} at {x:.0}, {y:.0}"
        );
    }
}

fn shader(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("chora-cost-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, format!("precision mediump float;\n{body}\n")).unwrap();
    path
}

#[test]
fn a_shader_plane_reads_the_scale_of_its_texture() {
    let probe = shader(
        "scale.frag",
        "uniform float scale;\nvoid main() { gl_FragColor = vec4(scale * 0.25, 0.0, 0.0, 1.0); }",
    );
    let mut core = Core::with_renderer();
    let mut wm = space(&mut core, SIZE);
    core.add_output("test-1", (640, 480));
    let near = chain(&[translate(100.0, 100.0, 0.0), turn_y(0.001)]);
    let far = chain(&[translate(400.0, 100.0, 3.0 * EYE), turn_y(0.001)]);
    let node = |id: &str, at: &Matrix| {
        json!({"create": {"id": id, "kind": "shader", "parent": "eye", "props": {
            "size": [100, 100], "shader": probe, "transform": {"matrix": at}}}})
    };
    wm.ok(
        &mut core,
        "scene.apply",
        json!({"commands": [node("near", &near), node("far", &far)]}),
    );
    let pixel = capture(&mut core, &mut wm);
    assert_eq!(pixel(150.0, 150.0)[0], 64, "1 texture pixel per pixel");
    let (x, y) = shown(&far, [320.0, 240.0], 50.0, 50.0);
    let got = pixel(x, y)[0];
    assert!(
        (15..=17).contains(&got),
        "a plane 4 times as far gets 1/4: {got}"
    );
    std::fs::remove_dir_all(probe.parent().unwrap()).unwrap();
}

fn mesh_floor(core: &mut Core) -> Client {
    core.add_output("test-1", (640, 480));
    let mut wm = space(core, SIZE);
    let floor = chain(&[translate(-680.0, 241.0, -1000.0), turn_x(90.0)]);
    let mesh = json!({"cols": 1, "rows": 1,
        "points": [[0, 0], [2000, 0], [0, 2000], [2000, 2000]]});
    let commands = json!({"commands": [
        rect("wall", [4000.0, 4000.0], [0.0, 0.0, 1.0, 1.0],
             &translate(-1700.0, -1700.0, 2000.0)),
        {"create": {"id": "floor", "kind": "rect", "parent": "eye", "props": {
            "size": [2000, 2000], "color": [1, 0, 0, 1], "mesh": mesh,
            "transform": {"matrix": floor}}}},
    ]});
    wm.ok(core, "scene.apply", commands);
    wm
}

#[test]
fn a_mesh_through_the_near_plane_clips_where_the_planes_clip() {
    let mut core = Core::with_renderer();
    let mut wm = mesh_floor(&mut core);
    let pixel = capture(&mut core, &mut wm);
    assert_eq!(pixel(320.0, 280.0), RED, "the floor in front of the cut");
    assert_eq!(
        pixel(320.0, 400.0),
        [0, 0, 255, 255],
        "the wall past the cut"
    );
    let under = |core: &Core, y: f64| {
        let spot = core.state.spot((320.0, y).into()).expect("a spot");
        node_hit(&core.state, &spot).map(|(node, _)| node)
    };
    assert_eq!(under(&core, 280.0).as_deref(), Some("floor"));
    assert_eq!(under(&core, 400.0).as_deref(), Some("wall"));
}

#[test]
fn a_mesh_in_a_context_takes_input_where_it_draws() {
    let mut core = Core::with_renderer();
    core.add_output("test-1", (640, 480));
    let mut wm = space(&mut core, SIZE);
    let mesh = json!({"cols": 1, "rows": 1,
        "points": [[0, 0], [200, 0], [0, 100], [0, 100]]});
    let at = turned(30.0, [320.0, 240.0, 100.0]);
    let commands = json!({"commands": [
        rect("back", [640.0, 480.0], [0.0, 0.0, 1.0, 1.0], &translate(0.0, 0.0, 500.0)),
        {"create": {"id": "wedge", "kind": "rect", "parent": "eye", "props": {
            "size": [200, 100], "color": [1, 0, 0, 1], "mesh": mesh,
            "transform": {"matrix": at}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let pixel = capture(&mut core, &mut wm);
    let under = |core: &Core, u: f64, v: f64| {
        let (x, y) = shown(&at, [320.0, 240.0], u, v);
        let spot = core.state.spot((x, y).into()).expect("a spot");
        (node_under(&core.state, &spot), pixel(x, y))
    };
    let (inside, drawn) = under(&core, 30.0, 20.0);
    assert_eq!((inside.as_deref(), drawn), (Some("wedge"), RED));
    let (outside, drawn) = under(&core, 180.0, 90.0);
    assert_eq!(outside.as_deref(), Some("back"), "the box is not the shape");
    assert_eq!(drawn, [0, 0, 255, 255]);
}
