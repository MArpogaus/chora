//! Tests of the texture limits on the input path: what a plane shows of
//! its content and where it takes input agree, and a node that the layer
//! budget left out of the last frame of an output takes input there and
//! gives it to nobody.

use serde_json::{Value, json};

use super::planes::{click, decode, forward, motions, move_to, node_events, transform};
use crate::testing::{Client, Core, wayland::Wayland};

fn granted(core: &mut Core) -> Client {
    let wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(core, "grants.set", grant);
    wm
}

fn shot(core: &mut Core, wm: &mut Client, output: &str) -> impl Fn(f64, f64) -> [u8; 4] + use<> {
    let picture = wm.ok(core, "screen.capture", json!({"output": output}));
    let (width, rgba) = decode(&picture);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    move |x: f64, y: f64| {
        let at = (y as usize * width + x as usize) * 4;
        rgba[at..at + 4].try_into().expect("a pixel")
    }
}

fn leave_out(core: &mut Core, output: &str, ids: &[&str]) {
    let ids: Vec<String> = ids.iter().map(|id| (*id).to_owned()).collect();
    let assets = &mut core.state.assets;
    assets.frame_left_out.insert(output.to_owned(), ids.clone());
    assets.frame_dark.insert(output.to_owned(), ids);
}

#[test]
fn a_plane_wider_than_a_texture_shows_and_takes_input_at_the_same_point() {
    let mut core = Core::with_renderer();
    let mut wm = granted(&mut core);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let matrix = [
        0.1, 0.001, 0.0, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 50.0, 0.0, 1.0,
    ];
    let spec = json!({"matrix": matrix});
    let commands = json!({"commands": [
        {"create": {"id": "wide", "kind": "group", "parent": "root",
            "props": {"size": [5000, 100], "transform": spec}}},
        {"create": {"id": "red", "kind": "rect", "parent": "wide",
            "props": {"size": [4500, 100], "color": [1, 0, 0, 1]}}},
        {"create": {"id": "green", "kind": "rect", "parent": "wide",
            "props": {"size": [500, 100], "color": [0, 1, 0, 1],
                      "transform": {"translate": [4500, 0]}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let at = forward(&transform(&spec), 4900.0, 50.0);
    let pixel = shot(&mut core, &mut wm, "HEADLESS-1");
    assert_eq!(pixel(at.x, at.y), [0, 255, 0, 255], "the drawn color");
    let spot = core.state.spot(at).expect("a spot");
    let under = crate::render::node_under(&core.state, &spot);
    assert_eq!(under.as_deref(), Some("green"), "the hit agrees");
}

fn rotated_rect(core: &mut Core, mesh: bool) -> (Client, chora_scene::Transform) {
    core.add_output("HDMI-1", (800, 600));
    let mut xp = core.named("xp");
    let spec = json!({"translate": [300, 100], "rotate": 45});
    let mut props = json!({"size": [100, 50], "color": [1, 1, 1, 1], "transform": spec});
    if mesh {
        props["mesh"] = json!({"cols": 1, "rows": 1,
            "points": [[0, 0], [100, 0], [0, 50], [100, 50]]});
    }
    let commands = json!({"commands": [
        {"create": {"id": "r", "kind": "rect", "parent": "root", "props": props}},
    ]});
    xp.ok(core, "scene.apply", commands);
    xp.ok(core, "events.subscribe", json!({"events": []}));
    (xp, transform(&spec))
}

#[test]
fn a_mesh_plane_that_the_budget_left_out_gives_input_to_nobody() {
    let mut core = Core::new();
    let (mut xp, total) = rotated_rect(&mut core, true);
    leave_out(&mut core, "HDMI-1", &["r"]);
    move_to(&mut core, forward(&total, 20.0, 10.0));
    click(&mut core, true);
    click(&mut core, false);
    assert_eq!(node_events(&mut xp, &mut core), Vec::<Value>::new());
}

#[test]
fn a_window_in_a_2d_layer_that_the_budget_left_out_gives_input_to_nobody() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let mut wm = core.named("wm");
    let commands = json!({"commands": [
        {"create": {"id": "box", "kind": "group", "parent": "root", "props": {
            "size": [200, 200], "clip": {"radius": 6},
            "transform": {"translate": [50, 50]}}}},
        {"attach": {"surface": "s1", "parent": "box"}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    leave_out(&mut core, "HDMI-1", &["box"]);
    motions(&mut wayland, &mut core);
    move_to(&mut core, (90.0, 90.0).into());
    assert_eq!(motions(&mut wayland, &mut core), Vec::<(f64, f64)>::new());
}

#[test]
fn a_plane_left_out_on_another_output_takes_input_on_this_one() {
    let mut core = Core::new();
    let (mut xp, total) = rotated_rect(&mut core, false);
    core.add_output("HDMI-2", (800, 600));
    leave_out(&mut core, "HDMI-2", &["r"]);
    move_to(&mut core, forward(&total, 20.0, 10.0));
    click(&mut core, true);
    click(&mut core, false);
    let events = node_events(&mut xp, &mut core);
    assert_eq!(events[0]["event"], "pointer.pressed", "{events:?}");
}

#[test]
fn a_window_whose_mirror_does_not_fit_takes_input() {
    let dir = std::env::temp_dir().join(format!("chora-mirror-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shader = dir.join("mirror.frag");
    std::fs::write(
        &shader,
        "precision mediump float;\nvarying vec2 v_uv;\nuniform sampler2D src;\n\
         void main() { gl_FragColor = texture2D(src, v_uv); }\n",
    )
    .unwrap();
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let filler = |id: &str, height: u32| {
        json!({"create": {"id": id, "kind": "rect", "parent": "root", "props": {
            "size": [4096, height], "color": [1, 1, 1, 1], "input": "pass",
            "transform": {"translate": [10, 10], "rotate": 10}}}})
    };
    let commands = json!({"commands": [
        {"attach": {"surface": "s1", "parent": "root"}},
        {"update": {"id": "s1", "props": {"transform": {"translate": [300, 300]}}}},
        {"create": {"id": "mirror", "kind": "shader", "parent": "root", "props": {
            "size": [100, 80], "shader": shader, "uniforms": {"src": {"node": "s1"}},
            "input": "pass", "transform": {"translate": [20, 300]}}}},
        filler("f1", 4091),
        filler("f2", 4096),
        filler("f3", 4096),
        filler("f4", 4096),
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    crate::backend::headless::render_all(&mut core.state);
    let left_out = core.state.assets.frame_left_out["HEADLESS-1"].clone();
    assert!(!left_out.contains(&"s1".to_owned()), "{left_out:?}");
    assert_eq!(left_out, ["mirror"], "the node that draws the source");
    motions(&mut wayland, &mut core);
    move_to(&mut core, (350.0, 340.0).into());
    let moved = motions(&mut wayland, &mut core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(!moved.is_empty(), "the window that drew takes input");
}

#[test]
fn a_shader_and_a_layer_that_drew_take_input_when_a_source_does_not_fit() {
    let dir = std::env::temp_dir().join(format!("chora-nested-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let shader = dir.join("mirror.frag");
    std::fs::write(
        &shader,
        "precision mediump float;\nvarying vec2 v_uv;\nuniform sampler2D src;\n\
         void main() { gl_FragColor = texture2D(src, v_uv); }\n",
    )
    .unwrap();
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let filler = |id: &str, height: u32| {
        json!({"create": {"id": id, "kind": "rect", "parent": "root", "props": {
            "size": [4096, height], "color": [1, 1, 1, 1], "input": "pass",
            "transform": {"translate": [10, 10], "rotate": 10}}}})
    };
    let commands = json!({"commands": [
        {"create": {"id": "mirror", "kind": "shader", "parent": "root", "props": {
            "size": [100, 80], "shader": shader, "uniforms": {"src": {"node": "src"}},
            "transform": {"translate": [20, 300]}}}},
        {"create": {"id": "src", "kind": "group", "parent": "root", "props": {
            "size": [20, 20], "transform": {"translate": [300, 300]}}}},
        {"create": {"id": "inner", "kind": "rect", "parent": "src", "props": {
            "size": [40, 40], "color": [0, 1, 0, 1], "radius": 6}}},
        filler("f1", 4091),
        filler("f2", 4096),
        filler("f3", 4096),
        filler("f4", 4096),
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    crate::backend::headless::render_all(&mut core.state);
    std::fs::remove_dir_all(&dir).unwrap();
    let under = |core: &Core, at: (f64, f64)| {
        let spot = core.state.spot(at.into()).expect("a spot");
        crate::render::node_under(&core.state, &spot)
    };
    assert_eq!(under(&core, (320.0, 320.0)).as_deref(), Some("inner"));
    assert_eq!(under(&core, (60.0, 340.0)).as_deref(), Some("mirror"));
}

#[test]
fn layers_dropped_names_the_window_that_keeps_the_keyboard() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (800, 600));
    let mut wayland = Wayland::connect(&mut core);
    let window = wayland.toplevel(&mut core, "window");
    wayland.attach(&mut core, &window, (100, 80));
    let mut wm = core.named("wm");
    let commands = json!({"commands": [
        {"create": {"id": "box", "kind": "group", "parent": "root", "props": {
            "size": [200, 200], "clip": {"radius": 6}}}},
        {"attach": {"surface": "s1", "parent": "box"}},
        {"focus": {"surface": "s1"}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    wm.ok(
        &mut core,
        "events.subscribe",
        json!({"events": ["layers.dropped"]}),
    );
    leave_out(&mut core, "HDMI-1", &["box"]);
    let budget = std::time::Duration::from_millis(16);
    core.state.frame_took("HDMI-1", budget, Default::default());
    let events = wm.events_now(&mut core);
    let dropped = &events[0]["params"];
    assert_eq!(dropped["nodes"], json!(["box"]), "{events:?}");
    assert_eq!(dropped["keyboard"], "s1", "{events:?}");
}
