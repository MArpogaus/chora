//! Tests of 2D sampling (#76): an image drawn smaller than its file shows
//! the mean of the covered pixels, and an image sized to whole physical
//! pixels at a fractional output scale stays sharp.

use serde_json::json;

use super::planes::decode;
use crate::{
    backend::headless::render_all,
    testing::{Client, Core},
};

const OUTPUT: &str = "HEADLESS-1";

fn checkerboard(name: &str, side: u32) -> String {
    let path =
        std::env::temp_dir().join(format!("chora-sampling-{}-{name}.png", std::process::id()));
    let file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
    let mut encoder = png::Encoder::new(file, side, side);
    encoder.set_color(png::ColorType::Grayscale);
    let mut writer = encoder.write_header().unwrap();
    let pixels: Vec<u8> = (0..side * side)
        .map(|i| {
            if (i % side + i / side).is_multiple_of(2) {
                0
            } else {
                255
            }
        })
        .collect();
    writer.write_image_data(&pixels).unwrap();
    path.to_str().unwrap().to_owned()
}

fn show(core: &mut Core, wm: &mut Client, path: &str, size: [f64; 2], at: [f64; 2]) {
    let props = json!({"image": path, "size": size, "transform": {"translate": at}});
    show_with(core, wm, &props);
}

fn show_with(core: &mut Core, wm: &mut Client, props: &serde_json::Value) {
    let commands = json!({"commands": [{"create": {"id": "picture", "kind": "image",
        "parent": "root", "props": props}}]});
    let id = wm.send("scene.apply", commands);
    for _ in 0..1000 {
        if let Some(answer) = wm.answer(core, id) {
            answer.expect("the image loads");
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("no answer after the decode");
}

fn world(scale: f64) -> (Core, Client) {
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(&mut core, "grants.set", grant);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let configure = json!({"output": OUTPUT, "scale": scale});
    wm.ok(&mut core, "output.configure", configure);
    (core, wm)
}

fn picture(core: &mut Core, wm: &mut Client) -> (usize, Vec<u8>) {
    let id = wm.send("screen.capture", json!({"output": OUTPUT}));
    let answer = (0..100)
        .find_map(|_| {
            render_all(&mut core.state);
            wm.answer(core, id)
        })
        .expect("a capture")
        .expect("the capture succeeds");
    decode(&answer)
}

fn region(core: &mut Core, wm: &mut Client, from: [usize; 2], side: usize) -> (f64, f64) {
    let (width, rgba) = picture(core, wm);
    let values: Vec<f64> = (from[1]..from[1] + side)
        .flat_map(|y| (from[0]..from[0] + side).map(move |x| (x, y)))
        .map(|(x, y)| f64::from(rgba[(y * width + x) * 4]))
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let count = values.len() as f64;
    let mean = values.iter().sum::<f64>() / count;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count;
    (mean, variance.sqrt())
}

#[test]
fn an_image_scaled_down_by_a_fraction_shows_the_mean() {
    let (mut core, mut wm) = world(1.0);
    let path = checkerboard("down", 512);
    show(&mut core, &mut wm, &path, [100.0, 100.0], [50.0, 50.0]);
    let (mean, deviation) = region(&mut core, &mut wm, [52, 52], 96);
    assert!((mean - 127.5).abs() < 4.0, "mean {mean}");
    assert!(deviation < 8.0, "standard deviation {deviation}");
}

#[test]
fn an_image_of_whole_physical_pixels_at_a_fractional_scale_stays_sharp() {
    let (mut core, mut wm) = world(1.25);
    let path = checkerboard("sharp", 512);
    show(&mut core, &mut wm, &path, [409.6, 409.6], [68.0, 20.0]);
    let (_, deviation) = region(&mut core, &mut wm, [85, 25], 400);
    assert!(deviation > 120.0, "standard deviation {deviation}");
}

fn colored(name: &str, side: u32, pixel: impl Fn(u32, u32) -> [u8; 3]) -> String {
    let path =
        std::env::temp_dir().join(format!("chora-sampling-{}-{name}.png", std::process::id()));
    let file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
    let mut encoder = png::Encoder::new(file, side, side);
    encoder.set_color(png::ColorType::Rgb);
    let mut writer = encoder.write_header().unwrap();
    let pixels: Vec<u8> = (0..side * side)
        .flat_map(|i| pixel(i % side, i / side))
        .collect();
    writer.write_image_data(&pixels).unwrap();
    path.to_str().unwrap().to_owned()
}

#[test]
fn a_sliced_image_scaled_down_shows_no_color_of_the_next_slice() {
    let (mut core, mut wm) = world(1.0);
    let path = colored(
        "slice",
        256,
        |x, _| {
            if x < 6 { [255, 0, 0] } else { [0, 0, 255] }
        },
    );
    let props = json!({"image": path, "size": [64, 64], "slice": [6, 6, 6, 6],
        "transform": {"translate": [50, 50]}});
    show_with(&mut core, &mut wm, &props);
    let (width, rgba) = picture(&mut core, &mut wm);
    for x in 56..60 {
        let at = (80 * width + x) * 4;
        assert!(rgba[at] < 8, "red {} at column {x}", rgba[at]);
    }
}

fn damaged(core: &Core) -> u64 {
    core.state.assets.stats.damaged_frames
}

#[test]
fn a_moved_image_and_one_that_switches_step_damage_the_output() {
    let (mut core, mut wm) = world(1.0);
    let path = checkerboard("damage", 512);
    show(&mut core, &mut wm, &path, [100.0, 100.0], [50.0, 50.0]);
    for _ in 0..3 {
        render_all(&mut core.state);
    }
    let settled = damaged(&core);
    render_all(&mut core.state);
    assert_eq!(damaged(&core), settled, "a still image has no damage");
    let update = |props: serde_json::Value| json!({"commands": [{"update": {"id": "picture", "props": props}}]});
    wm.ok(
        &mut core,
        "scene.apply",
        update(json!({"transform": {"translate": [60, 50]}})),
    );
    render_all(&mut core.state);
    assert_eq!(damaged(&core), settled + 1, "a move");
    wm.ok(&mut core, "scene.apply", update(json!({"size": [40, 40]})));
    render_all(&mut core.state);
    assert_eq!(damaged(&core), settled + 2, "a step switch");
    let (mean, deviation) = region(&mut core, &mut wm, [62, 52], 36);
    assert!(
        (mean - 127.5).abs() < 4.0 && deviation < 8.0,
        "{mean} {deviation}"
    );
}
