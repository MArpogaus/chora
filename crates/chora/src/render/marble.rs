//! Tests of the marble floor of chora-construct with its real shaders:
//! the floor of `fog.frag` has faint veins, micro bumps and fine
//! scratches that glint only in the light, and the mirror image of
//! `mirror.frag` is sharp where a pane meets the floor and blurred by
//! the roughness farther down, wavered by the bumps. Both are static.

use std::path::PathBuf;

use serde_json::json;

use super::planes::decode;
use crate::{
    backend::headless::render_all,
    testing::{Client, Core},
};

const OUTPUT: &str = "HEADLESS-1";

fn shader(name: &str) -> String {
    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "../../plugins/chora-construct",
        name,
    ]
    .iter()
    .collect();
    path.canonicalize()
        .expect("a shader of chora-construct")
        .display()
        .to_string()
}

fn world() -> (Core, Client) {
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(&mut core, "grants.set", grant);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    (core, wm)
}

/// The rows of a capture of the output as RGBA.
fn capture(core: &mut Core, wm: &mut Client) -> (usize, Vec<u8>) {
    let picture = wm.ok(core, "screen.capture", json!({"output": OUTPUT}));
    decode(&picture)
}

fn passes(core: &Core) -> u64 {
    core.state.assets.stats.by_output[OUTPUT].passes
}

/// The colours of a theme of chora-construct, as `config.rs` gives
/// them.
struct Theme {
    sky: [f64; 3],
    horizon: [f64; 3],
    ground: [f64; 3],
    sheen: [f64; 3],
}

const LIGHT: Theme = Theme {
    sky: [0.915, 0.93, 0.95],
    horizon: [1.0, 1.0, 1.0],
    ground: [0.93, 0.93, 0.925],
    sheen: [1.0, 1.0, 1.0],
};

const DARK: Theme = Theme {
    sky: [0.0, 0.0, 0.0],
    horizon: [0.15, 0.15, 0.16],
    ground: [0.012, 0.012, 0.014],
    sheen: [0.35, 0.35, 0.37],
};

/// The floor at full scale over the output, the eye level, looking
/// ahead.
fn floor(core: &mut Core, wm: &mut Client, theme: &Theme) {
    let mut uniforms = json!({
        "view": [640, 480], "pose": [0, 0, 0, 0], "lens": [600, 300, 1, 0],
        "sky": theme.sky, "fog": theme.horizon, "ground": theme.ground,
        "sheen": theme.sheen, "marble": [1, 1, 1, 900],
    });
    for k in 0..8 {
        uniforms[format!("s{k}")] = json!([0, 0, 0, 0]);
    }
    let commands = json!({"commands": [{"create": {"id": "floor", "kind": "shader",
        "parent": "root", "props": {"size": [640, 480], "shader": shader("fog.frag"),
            "uniforms": uniforms}}}]});
    wm.ok(core, "scene.apply", commands);
}

/// The mean of the luminance of a patch, in levels of 255.
fn luminance(rgba: &[u8], width: usize, (across, down): (usize, usize)) -> f64 {
    let at = (down * width + across) * 4;
    let [red, green, blue] = [rgba[at], rgba[at + 1], rgba[at + 2]].map(f64::from);
    0.2126 * red + 0.7152 * green + 0.0722 * blue
}

/// The fine detail of a patch: the mean of the absolute difference of
/// each pixel to the mean of its 5x5 neighbours, and the spread of the
/// patch from its darkest to its brightest pixel.
fn detail(rgba: &[u8], width: usize, (x0, y0): (usize, usize), side: usize) -> (f64, f64) {
    let mut sum = 0.0;
    let (mut low, mut high) = (f64::MAX, f64::MIN);
    for y in y0..y0 + side {
        for x in x0..x0 + side {
            let here = luminance(rgba, width, (x, y));
            let around: f64 = (0..25)
                .map(|i| luminance(rgba, width, (x + i % 5 - 2, y + i / 5 - 2)))
                .sum::<f64>()
                / 25.0;
            sum += (here - around).abs();
            low = low.min(here);
            high = high.max(here);
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let count = (side * side) as f64;
    (sum / count, high - low)
}

#[allow(clippy::print_stdout)]
fn report(line: &str) {
    println!("{line}");
}

#[test]
fn the_floor_is_calm_marble_with_scratches_in_the_light() {
    let measured: Vec<(&str, f64, f64, f64)> = [("light", LIGHT), ("dark", DARK)]
        .into_iter()
        .map(|(name, theme)| {
            let (mut core, mut wm) = world();
            floor(&mut core, &mut wm, &theme);
            let (width, rgba) = capture(&mut core, &mut wm);
            let (fine, spread) = detail(&rgba, width, (60, 380), 64);
            let (glint, _) = detail(&rgba, width, (288, 320), 64);
            report(&format!(
                "{name}: detail {fine:.2} beside the light, {glint:.2} in it, spread {spread:.1}"
            ));
            (name, fine, glint, spread)
        })
        .collect();
    for (name, fine, glint, spread) in measured {
        assert!(fine > 0.15, "{name}: the floor has a texture: {fine:.2}");
        assert!(spread < 40.0, "{name}: and stays calm: {spread:.1}");
        assert!(glint > fine * 1.3, "{name}: scratches glint in the light");
    }
}

#[test]
fn a_still_floor_renders_once() {
    let (mut core, mut wm) = world();
    floor(&mut core, &mut wm, &LIGHT);
    render_all(&mut core.state);
    let first = passes(&core);
    for _ in 0..10 {
        render_all(&mut core.state);
    }
    assert_eq!(passes(&core), first, "a still floor runs no pass");
}

/// A pane, black on the left and white on the right, hidden, and its
/// mirror image as `mirror.frag` draws it, upright, with the contact
/// line at its bottom edge.
fn mirrored(core: &mut Core, wm: &mut Client) -> (usize, Vec<u8>) {
    let commands = json!({"commands": [
        {"create": {"id": "pane", "kind": "group", "parent": "root",
            "props": {"size": [200, 160], "opacity": 0, "transform": {"translate": [400, 300]}}}},
        {"create": {"id": "left", "kind": "rect", "parent": "pane",
            "props": {"size": [100, 160], "color": [0, 0, 0, 1]}}},
        {"create": {"id": "right", "kind": "rect", "parent": "pane",
            "props": {"size": [100, 160], "color": [1, 1, 1, 1],
                      "transform": {"translate": [100, 0]}}}},
        {"create": {"id": "mirror", "kind": "shader", "parent": "root", "props": {
            "size": [200, 160], "shader": shader("mirror.frag"),
            "transform": {"translate": [100, 100]},
            "uniforms": {"win": {"node": "pane"}, "gloss": 1, "depth": 1, "reach": 0,
                         "ground": 0, "rough": [0.6, 10, 1.6, 1.2]}}}},
    ]});
    wm.ok(core, "scene.apply", commands);
    capture(core, wm)
}

/// The red channel of a row of the mirror image, from its left edge,
/// scaled from 0 at its darkest to 1 at its brightest.
fn row(rgba: &[u8], width: usize, y: usize) -> Vec<f64> {
    let red: Vec<f64> = (130..270)
        .map(|x| f64::from(rgba[(y * width + x) * 4]))
        .collect();
    let (low, high) = red
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    red.iter()
        .map(|v| (v - low) / (high - low).max(1.0))
        .collect()
}

/// Where a row first reaches a level.
#[allow(clippy::cast_precision_loss)]
fn reach(row: &[f64], level: f64) -> f64 {
    row.iter().position(|v| *v >= level).unwrap_or(row.len()) as f64
}

#[test]
fn a_reflection_blurs_with_the_distance_from_the_pane() {
    let (mut core, mut wm) = world();
    let (width, rgba) = mirrored(&mut core, &mut wm);
    let ramp = |y: usize| {
        let r = row(&rgba, width, y);
        reach(&r, 0.9) - reach(&r, 0.1)
    };
    let (near, far) = (ramp(252), ramp(215));
    report(&format!(
        "reflection: {near} px at the contact, {far} px far down"
    ));
    assert!(near <= 4.0, "sharp near the pane: {near} px");
    assert!(far >= near + 2.0, "soft far from it: {far} px");
    let middles: Vec<f64> = (205..250)
        .step_by(4)
        .map(|y| reach(&row(&rgba, width, y), 0.5))
        .collect();
    let (low, high) = middles
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    report(&format!(
        "reflection: the edge wavers over {} px",
        high - low
    ));
    assert!(
        (1.0..=4.0).contains(&(high - low)),
        "micro bumps waver the edge a little: {} px",
        high - low
    );
}

/// The red channel at a point of a capture.
fn red(rgba: &[u8], width: usize, x: usize, y: usize) -> f64 {
    f64::from(rgba[(y * width + x) * 4])
}

#[test]
fn a_reflection_ends_60_px_under_the_pane_and_keeps_the_pane_edges() {
    let (mut core, mut wm) = world();
    let (width, rgba) = mirrored(&mut core, &mut wm);
    let floor = red(&rgba, width, 20, 150);
    let at_contact = red(&rgba, width, 250, 258) - floor;
    let at_edge = red(&rgba, width, 297, 258) - floor;
    let past_60 = red(&rgba, width, 250, 195) - floor;
    report(&format!(
        "mirror: floor {floor}, contact {at_contact}, edge {at_edge}, 65 px down {past_60}"
    ));
    assert!(at_contact > 8.0, "the reflection shows: {at_contact}");
    assert!(
        past_60.abs() < 1.0,
        "it fades out by 60 px under the pane: {past_60}"
    );
    assert!(
        at_edge > at_contact * 0.6,
        "its side edge sits on the pane's edge, feathered by a few px only: {at_edge} of {at_contact}"
    );
}

#[test]
fn a_still_reflection_renders_once() {
    let (mut core, mut wm) = world();
    let _ = mirrored(&mut core, &mut wm);
    render_all(&mut core.state);
    let first = passes(&core);
    for _ in 0..10 {
        render_all(&mut core.state);
    }
    assert_eq!(passes(&core), first, "a still reflection runs no pass");
}
