//! Tests of the backdrop of a rect: it draws the composited frame behind
//! its shape blurred, under its own fill, only where it lies, and only
//! again when something behind it changed.

use std::time::Instant;

use serde_json::{Value, json};

use super::context::{EYE, chain, translate, turn_y};
use super::planes::decode;
use crate::{
    backend::headless::{render_all, screen_row},
    testing::{Client, Core},
};

const OUTPUT: &str = "HEADLESS-1";
/// The edge between black and white behind the glass, in the middle of
/// the output.
const EDGE: i32 = 320;
/// The row that the tests read, through the middle of the glass.
const ROW: i32 = 240;

fn world() -> (Core, Client) {
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(&mut core, "grants.set", grant);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let halves = json!({"commands": [
        {"create": {"id": "dark", "kind": "rect", "parent": "root",
            "props": {"size": [320, 480], "color": [0, 0, 0, 1]}}},
        {"create": {"id": "light", "kind": "rect", "parent": "root",
            "props": {"size": [320, 480], "color": [1, 1, 1, 1],
                      "transform": {"translate": [320, 0]}}}},
    ]});
    wm.ok(&mut core, "scene.apply", halves);
    (core, wm)
}

fn glass(id: &str, props: &Value) -> Value {
    let mut all = json!({"size": [200, 200], "color": [0, 0, 0, 0],
        "transform": {"translate": [220, 140]}});
    for (key, value) in props.as_object().into_iter().flatten() {
        all[key] = value.clone();
    }
    json!({"commands": [{"create": {"id": id, "kind": "rect", "parent": "root", "props": all}}]})
}

fn frames(core: &mut Core, count: usize) {
    for _ in 0..count {
        render_all(&mut core.state);
    }
}

/// The red channel of the screen along the test row, from the left of the
/// glass to its right.
fn screen(core: &mut Core) -> Vec<u8> {
    frames(core, 1);
    screen_row(&mut core.state, 0, ROW, (220, 200))
        .iter()
        .map(|p| p[0])
        .collect()
}

/// The red channel of a capture along the test row, over the glass.
fn captured(core: &mut Core, wm: &mut Client) -> Vec<u8> {
    let picture = wm.ok(core, "screen.capture", json!({"output": OUTPUT}));
    let (width, rgba) = decode(&picture);
    #[allow(clippy::cast_sign_loss)]
    let row = ROW as usize * width;
    (220..420).map(|x| rgba[(row + x) * 4]).collect()
}

/// The distance in pixels over which a row rises from 10 % to 90 %.
fn ramp(row: &[u8]) -> usize {
    let first = |level: u8| row.iter().position(|v| *v >= level).unwrap_or(row.len());
    first(230) - first(26)
}

#[allow(clippy::print_stdout)]
fn report(line: &str) {
    println!("{line}");
}

fn passes(core: &Core) -> u64 {
    core.state.assets.stats.by_output[OUTPUT].passes
}

#[test]
fn a_sharp_edge_behind_a_backdrop_blurs_to_a_gradient() {
    let (mut core, mut wm) = world();
    wm.ok(
        &mut core,
        "scene.apply",
        glass("glass", &json!({"backdrop": {"blur": 16}})),
    );
    let row = screen(&mut core);
    report(&format!("blur 16: {row:?}"));
    let at = |x: i32| row[usize::try_from(x - 220).unwrap()];
    assert!(
        at(EDGE - 2) > 40,
        "black next to the edge lightens: {}",
        at(EDGE - 2)
    );
    assert!(
        at(EDGE + 1) < 215,
        "white next to the edge darkens: {}",
        at(EDGE + 1)
    );
    assert!(
        at(EDGE).abs_diff(128) < 40,
        "grey on the edge: {}",
        at(EDGE)
    );
    assert!(row.windows(2).all(|w| w[1] + 2 >= w[0]), "it only rises");
    let width = ramp(&row);
    report(&format!("blur 16: 10 % to 90 % over {width} px"));
    assert!((12..=48).contains(&width), "a gradient of {width} px");
    assert!(
        at(230) < 8 && at(410) > 247,
        "far from the edge it keeps the halves"
    );
    let outside = screen_row(&mut core.state, 0, ROW, (EDGE - 102, 2));
    assert_eq!(
        outside[0],
        [0, 0, 0, 255],
        "left of the glass it stays sharp"
    );
    let shot = captured(&mut core, &mut wm);
    assert!(
        shot.iter().zip(&row).all(|(a, b)| a.abs_diff(*b) <= 2),
        "a capture shows the backdrop as the screen does"
    );
}

#[test]
fn a_larger_blur_gives_a_wider_gradient() {
    let (mut core, mut wm) = world();
    wm.ok(
        &mut core,
        "scene.apply",
        glass("glass", &json!({"backdrop": {"blur": 8}})),
    );
    let narrow = ramp(&screen(&mut core));
    let wide = json!({"commands": [{"update": {"id": "glass",
        "props": {"backdrop": {"blur": 32}}}}]});
    wm.ok(&mut core, "scene.apply", wide);
    let wide = ramp(&screen(&mut core));
    report(&format!("blur 8: {narrow} px, blur 32: {wide} px"));
    assert!(wide > narrow * 2, "{narrow} px, then {wide} px");
}

#[test]
fn a_node_without_backdrop_is_unchanged() {
    let (mut core, mut wm) = world();
    let tint = json!({"color": [1, 0, 0, 0.5]});
    wm.ok(&mut core, "scene.apply", glass("plain", &tint));
    let row = screen(&mut core);
    let at = |x: i32| row[usize::try_from(x - 220).unwrap()];
    assert!(
        at(EDGE - 1).abs_diff(128) <= 1,
        "half red over black: {}",
        at(EDGE - 1)
    );
    assert_eq!(at(EDGE), 255, "the edge stays sharp");
    assert_eq!(passes(&core), 0, "no pass for a plain rect");
}

#[test]
fn the_fill_of_the_glass_draws_over_its_backdrop() {
    let (mut core, mut wm) = world();
    let tinted = json!({"color": [1, 0, 0, 1], "backdrop": {"blur": 16}});
    wm.ok(&mut core, "scene.apply", glass("glass", &tinted));
    frames(&mut core, 1);
    let row = screen_row(&mut core.state, 0, ROW, (EDGE, 1));
    assert_eq!(row[0], [255, 0, 0, 255], "an opaque tint hides the blur");
}

#[test]
fn a_still_screen_blurs_once_and_a_change_behind_blurs_again() {
    let (mut core, mut wm) = world();
    let props = json!({"color": [1, 1, 1, 0.3], "radius": 12,
        "backdrop": {"blur": 24, "saturation": 1.4}});
    wm.ok(&mut core, "scene.apply", glass("glass", &props));
    frames(&mut core, 1);
    let first = passes(&core);
    assert!(first > 0, "the first frame blurs");
    frames(&mut core, 30);
    assert_eq!(passes(&core), first, "a still screen blurs nothing");
    let damaged = core.state.assets.stats.damaged_frames;
    frames(&mut core, 3);
    assert_eq!(core.state.assets.stats.damaged_frames, damaged, "no damage");
    let far = json!({"commands": [
        {"create": {"id": "far", "kind": "rect", "parent": "root",
            "props": {"size": [20, 20], "color": [0, 0, 1, 1]}}}]});
    wm.ok(&mut core, "scene.apply", far);
    frames(&mut core, 2);
    assert_eq!(
        passes(&core),
        first,
        "a change beside the glass blurs nothing"
    );
    let over = json!({"commands": [
        {"create": {"id": "label", "kind": "rect", "parent": "root",
            "props": {"size": [20, 20], "color": [0, 1, 0, 1],
                      "transform": {"translate": [300, 230]}}}}]});
    wm.ok(&mut core, "scene.apply", over);
    frames(&mut core, 2);
    assert_eq!(
        passes(&core),
        first,
        "a change on top of the glass blurs nothing"
    );
    let behind = json!({"commands": [
        {"update": {"id": "dark", "props": {"color": [0.5, 0, 0, 1]}}}]});
    wm.ok(&mut core, "scene.apply", behind);
    frames(&mut core, 1);
    let second = passes(&core);
    assert!(second > first, "a change behind blurs again");
    frames(&mut core, 10);
    assert_eq!(passes(&core), second, "and then rests");
    let row = screen(&mut core);
    assert!(row[0] > 100, "the new color shows through: {}", row[0]);
}

#[test]
fn a_backdrop_in_a_3d_context_blurs_what_lies_behind_its_plane() {
    let (mut core, mut wm) = world();
    let pane = chain(&[
        translate(320.0, 240.0, 0.0),
        turn_y(20.0),
        translate(-100.0, -100.0, 0.0),
    ]);
    let commands = json!({"commands": [
        {"create": {"id": "space", "kind": "group", "parent": "root",
            "props": {"size": [640, 480], "perspective": EYE}}},
        {"create": {"id": "eye", "kind": "group", "parent": "space"}},
        {"create": {"id": "pane", "kind": "rect", "parent": "eye", "props": {
            "size": [200, 200], "color": [0, 0, 0, 0],
            "backdrop": {"blur": 16}, "transform": {"matrix": pane}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    let row = screen(&mut core);
    let at = |x: i32| row[usize::try_from(x - 220).unwrap()];
    assert!(
        at(EDGE - 2) > 40,
        "black next to the edge lightens: {}",
        at(EDGE - 2)
    );
    assert!(
        at(EDGE + 1) < 215,
        "white next to the edge darkens: {}",
        at(EDGE + 1)
    );
    frames(&mut core, 1);
    let first = passes(&core);
    frames(&mut core, 10);
    assert_eq!(passes(&core), first, "a still context blurs nothing");
}

#[test]
fn a_backdrop_is_cheap_on_the_software_renderer() {
    let (mut core, mut wm) = world();
    let props = json!({"size": [400, 300], "color": [1, 1, 1, 0.3], "radius": 12,
        "transform": {"translate": [120, 90]}, "backdrop": {"blur": 24}});
    wm.ok(&mut core, "scene.apply", glass("glass", &props));
    frames(&mut core, 2);
    let mut colors = [0.0, 1.0].iter().cycle();
    let mut timed = |core: &mut Core, wm: &mut Client, blurred: bool| {
        let started = Instant::now();
        for _ in 0..20 {
            let shade = colors.next().copied().unwrap_or(0.0);
            let update = json!({"commands": [
                {"update": {"id": "dark", "props": {"color": [shade, 0, 0, 1]}}},
                {"update": {"id": "glass", "props": {"visible": blurred}}}]});
            wm.ok(core, "scene.apply", update);
            frames(core, 1);
        }
        started.elapsed().as_secs_f64() * 1000.0 / 20.0
    };
    let plain = timed(&mut core, &mut wm, false);
    let blurred = timed(&mut core, &mut wm, true);
    report(&format!(
        "640x480, glass 400x300 blur 24: {plain:.2} ms a frame without, {blurred:.2} ms with"
    ));
    assert!(
        blurred - plain < 12.0,
        "{blurred:.2} ms against {plain:.2} ms"
    );
}
