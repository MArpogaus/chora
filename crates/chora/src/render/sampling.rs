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
    let commands = json!({"commands": [{"create": {"id": "picture", "kind": "image",
        "parent": "root", "props": {"image": path, "size": size,
        "transform": {"translate": at}}}}]});
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

fn region(core: &mut Core, wm: &mut Client, from: [usize; 2], side: usize) -> (f64, f64) {
    let id = wm.send("screen.capture", json!({"output": OUTPUT}));
    let picture = (0..100)
        .find_map(|_| {
            render_all(&mut core.state);
            wm.answer(core, id)
        })
        .expect("a capture")
        .expect("the capture succeeds");
    let (width, rgba) = decode(&picture);
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
