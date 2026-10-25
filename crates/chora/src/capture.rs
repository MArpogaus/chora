//! `screen.capture`: renders an output, or a node with its descendants,
//! offscreen and returns the picture as a PNG file (D33). It needs a grant
//! of the user and fails while the session is locked, so that no client
//! sees the lock screen or what it hides.
//!
//! An output shows as on screen, without the cursor. A node shows in its
//! own coordinates, without its transform, opacity, mesh and effects, on a
//! transparent background.

use base64::{Engine, engine::general_purpose::STANDARD};
use chora_scene::{
    NodeKind, Transform,
    ipc::{Capability, Capture, Picture, RpcError},
};
use serde_json::Value;
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, ExportMem, Offscreen, TextureMapping,
            damage::OutputDamageTracker,
            gles::{GlesRenderer, GlesTexture},
        },
    },
    output::Output,
    utils::{Buffer, Rectangle, Size},
};

use crate::{
    ipc::{params_of, rejected},
    render::{self, Parts, View},
    state::State,
};

/// The longest side of a picture.
const MAX_SIDE: f64 = 8192.0;
/// The most pixels of a picture, 64 MiB of RGBA; a larger picture
/// shrinks.
const MAX_PIXELS: f64 = 16.0 * 1024.0 * 1024.0;
/// The largest PNG file. Its base64 must fit in the output buffer of a
/// connection, 8 MiB.
const MAX_PNG: usize = 5 << 20;

/// What a capture renders.
struct Shot {
    view: View,
    what: Source,
    origin: [f64; 2],
}

enum Source {
    Output(Output),
    Node(String),
}

/// The pixels of a picture: RGBA, premultiplied, top row first.
pub struct Pixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Pixels {
    /// A part of the picture: x, y, width and height in pixels, inside it.
    pub fn crop(&self, [x, y, w, h]: [u32; 4]) -> Option<Self> {
        let inside = x.checked_add(w)? <= self.width && y.checked_add(h)? <= self.height;
        if !inside || w == 0 || h == 0 {
            return None;
        }
        let stride = self.width as usize * 4;
        let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
        for row in y..y + h {
            let start = row as usize * stride + x as usize * 4;
            rgba.extend_from_slice(&self.rgba[start..start + w as usize * 4]);
        }
        Some(Self {
            width: w,
            height: h,
            rgba,
        })
    }
}

impl State {
    /// `screen.capture`.
    pub fn ipc_capture(&mut self, conn: u64, params: Value) -> Result<Value, RpcError> {
        self.check_grant(conn, Capability::ScreenCapture)?;
        if self.lock.is_some() {
            return Err(rejected("the session is locked"));
        }
        let capture: Capture = params_of(params)?;
        let shot = self.plan(&capture)?;
        let pixels = self.render_shot(&shot).map_err(rejected)?;
        let png = encode(&pixels).map_err(rejected)?;
        if png.len() > MAX_PNG {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!(
                    "the picture has {} KiB; ask for a smaller max_side",
                    png.len() >> 10
                ),
            ));
        }
        let picture = Picture {
            width: pixels.width,
            height: pixels.height,
            scale: shot.view.scale,
            origin: shot.origin,
            png: STANDARD.encode(png),
            left_out: self.left_out_of(&shot),
        };
        Ok(serde_json::to_value(picture).expect("pictures always serialize"))
    }

    /// The nodes that the last frame left out of the output of a capture,
    /// or of any output for a capture of a node.
    fn left_out_of(&self, shot: &Shot) -> Vec<String> {
        let mut nodes: Vec<String> = match &shot.what {
            Source::Output(output) => self
                .assets
                .left_out
                .get(&output.name())
                .cloned()
                .unwrap_or_default(),
            // The node and its descendants that some output left out.
            Source::Node(id) => self
                .assets
                .left_out
                .values()
                .flatten()
                .filter(|n| self.scene.is_within(n, id))
                .cloned()
                .collect(),
        };
        nodes.sort();
        nodes.dedup();
        nodes
    }

    /// The pixels of an output as on screen, without the cursor, for the
    /// screencopy protocols.
    pub fn output_pixels(&mut self, output: &Output) -> Result<Pixels, String> {
        let shot = Shot {
            view: View::of(self, output),
            what: Source::Output(output.clone()),
            origin: [0.0, 0.0],
        };
        self.render_shot(&shot)
    }

    /// The pixels of a node with its descendants, as `screen.capture`
    /// shows it, for the screencopy protocols.
    pub fn node_pixels(&mut self, id: &str) -> Result<Pixels, String> {
        let shot = self.plan(&node_capture(id)).map_err(|e| e.message)?;
        self.render_shot(&shot)
    }

    /// The size in pixels of a capture of a node.
    pub fn node_capture_size(&self, id: &str) -> Option<(u32, u32)> {
        let size = self.plan(&node_capture(id)).ok()?.view.size;
        Some((size.w.try_into().ok()?, size.h.try_into().ok()?))
    }

    /// The view and the source of a capture.
    fn plan(&self, capture: &Capture) -> Result<Shot, RpcError> {
        if capture.max_side == Some(0) {
            return Err(rejected("max_side must be at least 1"));
        }
        // The factor that fits a size of pixels into max_side and into
        // MAX_PIXELS.
        let shrink = |w: f64, h: f64| {
            let side = capture.max_side.map_or(MAX_SIDE, f64::from).min(MAX_SIDE);
            (side / w.max(h))
                .min((MAX_PIXELS / (w * h)).sqrt())
                .min(1.0)
        };
        if let Some(id) = &capture.node {
            if capture.output.is_some() {
                return Err(rejected("name an output or a node, not both"));
            }
            let [x, y, w, h] = self
                .bounds(id)
                .ok_or_else(|| rejected(format!("{id} is no visible node with a size")))?;
            // The sharpest output decides the pixels of a logical pixel.
            let sharpest = self
                .outputs
                .iter()
                .map(|o| o.current_scale().fractional_scale())
                .fold(1.0, f64::max);
            let scale = sharpest * shrink(w * sharpest, h * sharpest);
            let view = View {
                camera: Transform::translate(-x, -y, 0.0),
                scale,
                size: pixel_size(w * scale, h * scale),
            };
            return Ok(Shot {
                view,
                what: Source::Node(id.clone()),
                origin: [x, y],
            });
        }
        let output = match &capture.output {
            Some(name) => self.outputs.iter().find(|o| o.name() == *name),
            None => self.outputs.first(),
        }
        .ok_or_else(|| rejected("no such output"))?
        .clone();
        let mut view = View::of(self, &output);
        let factor = shrink(f64::from(view.size.w), f64::from(view.size.h));
        view.scale *= factor;
        view.size = pixel_size(
            f64::from(view.size.w) * factor,
            f64::from(view.size.h) * factor,
        );
        Ok(Shot {
            view,
            what: Source::Output(output),
            origin: [0.0, 0.0],
        })
    }

    /// The area of a node and its descendants, in node coordinates: x, y,
    /// width and height. Nodes without a size do not count.
    fn bounds(&self, id: &str) -> Option<[f64; 4]> {
        let mut corners = Vec::new();
        for placed in self.scene.layer_list(id) {
            let size = match placed.node.kind {
                NodeKind::Surface => self.windows.get(&placed.node.id).map(|w| {
                    let size = w.geometry().size;
                    [f64::from(size.w), f64::from(size.h)]
                }),
                // An image without a size has the size of its file, as
                // when it draws.
                NodeKind::Image => placed.node.props.size.or_else(|| {
                    placed
                        .node
                        .props
                        .image
                        .as_deref()
                        .and_then(|p| self.assets.images.get(p))
                        .map(|i| [f64::from(i.size[0]), f64::from(i.size[1])])
                }),
                // A text node is sized by its layout, like an image
                // without a size.
                NodeKind::Text => self.assets.texts.size(placed.node),
                _ => placed.node.props.size,
            };
            let Some([w, h]) = size else { continue };
            for (x, y) in [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
                corners.push(placed.transform.apply(x, y));
            }
        }
        let fold = |pick: fn(&(f64, f64)) -> f64, start: f64, f: fn(f64, f64) -> f64| {
            corners.iter().map(pick).fold(start, f)
        };
        let (left, top) = (
            fold(|c| c.0, f64::INFINITY, f64::min),
            fold(|c| c.1, f64::INFINITY, f64::min),
        );
        let (right, bottom) = (
            fold(|c| c.0, f64::NEG_INFINITY, f64::max),
            fold(|c| c.1, f64::NEG_INFINITY, f64::max),
        );
        let (w, h) = (right - left, bottom - top);
        (w.is_finite() && h.is_finite() && w >= 1.0 && h >= 1.0).then_some([left, top, w, h])
    }

    fn render_shot(&mut self, shot: &Shot) -> Result<Pixels, String> {
        let backend = self.backend.as_mut().ok_or("no renderer yet")?;
        let mut parts = Parts {
            scene: &self.scene,
            windows: &self.windows,
            assets: &mut self.assets,
            lock: None,
            depth: 0,
        };
        // A capture renders no frame of the screen, so the layers that the
        // screen left out stay as they were.
        let screen = parts.assets.frame_left_out.clone();
        let pixels = backend.with_gles(|gles| render_pixels(gles, &mut parts, shot));
        self.assets.frame_left_out = screen;
        pixels?
    }
}

fn node_capture(id: &str) -> Capture {
    Capture {
        node: Some(id.into()),
        ..Capture::default()
    }
}

/// A size in pixels, at least 1 and at most `MAX_SIDE` a side.
fn pixel_size(w: f64, h: f64) -> smithay::utils::Size<i32, smithay::utils::Physical> {
    #[allow(clippy::cast_possible_truncation)] // clamped
    let side = |v: f64| v.ceil().clamp(1.0, MAX_SIDE) as i32;
    (side(w), side(h)).into()
}

fn render_pixels(
    gles: &mut GlesRenderer,
    parts: &mut Parts<'_>,
    shot: &Shot,
) -> Result<Pixels, String> {
    let (elements, clear) = match &shot.what {
        Source::Output(output) => (
            render::output_elements(parts, gles, output, &shot.view),
            [0.0, 0.0, 0.0, 1.0],
        ),
        Source::Node(id) => (
            render::shown_elements(parts, gles, id, &shot.view).unwrap_or_default(),
            [0.0; 4],
        ),
    };
    let size = shot.view.size;
    let buffer: Size<i32, Buffer> = (size.w, size.h).into();
    let mut texture: GlesTexture = gles
        .create_buffer(Fourcc::Abgr8888, buffer)
        .map_err(|e| e.to_string())?;
    let mut target = gles.bind(&mut texture).map_err(|e| e.to_string())?;
    let mut tracker = OutputDamageTracker::new(
        (size.w, size.h),
        shot.view.scale,
        smithay::utils::Transform::Normal,
    );
    tracker
        .render_output(gles, &mut target, 0, &elements, clear)
        .map_err(|e| format!("{e:?}"))?;
    let mapping = gles
        .copy_framebuffer(&target, Rectangle::from_size(buffer), Fourcc::Abgr8888)
        .map_err(|e| e.to_string())?;
    let mut rgba = gles
        .map_texture(&mapping)
        .map_err(|e| e.to_string())?
        .to_vec();
    let (width, height) = (size.w.unsigned_abs(), size.h.unsigned_abs());
    // A mapping that is not flipped starts with the bottom row, as GL does.
    if !mapping.flipped() {
        flip_rows(&mut rgba, width as usize * 4);
    }
    Ok(Pixels {
        width,
        height,
        rgba,
    })
}

/// Puts the bottom row first.
fn flip_rows(rgba: &mut [u8], stride: usize) {
    let rows = rgba.len() / stride;
    for row in 0..rows / 2 {
        let (top, bottom) = rgba.split_at_mut((rows - 1 - row) * stride);
        top[row * stride..(row + 1) * stride].swap_with_slice(&mut bottom[..stride]);
    }
}

/// A PNG file of premultiplied pixels, with straight alpha as PNG wants.
fn encode(pixels: &Pixels) -> Result<Vec<u8>, String> {
    let mut rgba = pixels.rgba.clone();
    for px in rgba.as_chunks_mut::<4>().0 {
        let a = u32::from(px[3]);
        if a != 0 && a != 255 {
            for c in &mut px[..3] {
                // At most 255, since a color is at most its alpha.
                #[allow(clippy::cast_possible_truncation)]
                let straight = (u32::from(*c) * 255 / a).min(255) as u8;
                *c = straight;
            }
        }
    }
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, pixels.width, pixels.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&rgba).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(png)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::testing::Core;

    /// The width and the RGBA bytes of a picture of `screen.capture`.
    fn decode(picture: &Value) -> (usize, Vec<u8>) {
        let png = STANDARD.decode(picture["png"].as_str().unwrap()).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(png))
            .read_info()
            .unwrap();
        let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut rgba).unwrap();
        (info.width as usize, rgba)
    }

    #[test]
    fn a_crop_keeps_a_part() {
        let pixels = Pixels {
            width: 2,
            height: 2,
            rgba: (0..16).collect(),
        };
        let part = pixels.crop([1, 1, 1, 1]).unwrap();
        assert_eq!(part.rgba, vec![12, 13, 14, 15]);
        assert!(pixels.crop([1, 1, 2, 1]).is_none());
        assert!(pixels.crop([0, 0, 0, 1]).is_none());
    }

    #[test]
    fn a_capture_of_an_output_shows_a_rect_of_the_scene() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut wm = core.named("wm");
        let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let props = json!({"size": [20, 10], "color": [1.0, 0.0, 0.0, 1.0],
            "transform": {"translate": [8, 4]}});
        let create = json!({"id": "r", "kind": "rect", "parent": "root", "props": props});
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"create": create}]}),
        );
        let picture = wm.ok(&mut core, "screen.capture", json!({"output": "test-1"}));
        assert_eq!(
            (picture["width"].clone(), picture["height"].clone()),
            (json!(64), json!(48))
        );
        let (width, rgba) = decode(&picture);
        let pixel = |x: usize, y: usize| &rgba[(y * width + x) * 4..][..4];
        let (red, black) = ([255, 0, 0, 255], [0, 0, 0, 255]);
        for (x, y) in [(8, 4), (27, 4), (8, 13), (27, 13)] {
            assert_eq!(pixel(x, y), red, "inside at {x},{y}");
        }
        for (x, y) in [(7, 4), (28, 4), (8, 3), (8, 14), (63, 47)] {
            assert_eq!(pixel(x, y), black, "outside at {x},{y}");
        }
    }

    #[test]
    fn a_translucent_rect_has_straight_alpha() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut wm = core.named("wm");
        let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let rect = |id: &str, x: u32, radius: u32| {
            json!({"create": {"id": id, "kind": "rect", "parent": "root", "props": {
                "size": [20, 20], "color": [1.0, 1.0, 1.0, 0.5], "radius": radius,
                "transform": {"translate": [x, 4]}}}})
        };
        let commands = json!({"commands": [rect("plain", 4, 0), rect("round", 32, 4)]});
        wm.ok(&mut core, "scene.apply", commands);
        let picture = wm.ok(&mut core, "screen.capture", json!({"output": "test-1"}));
        let (width, rgba) = decode(&picture);
        let pixel = |x: usize, y: usize| &rgba[(y * width + x) * 4..][..3];
        for (x, what) in [(14, "a rect"), (42, "a rounded rect")] {
            let gray = pixel(x, 14);
            assert!(
                gray.iter().all(|c| c.abs_diff(128) <= 2),
                "{what}: {gray:?}"
            );
        }
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // screen coordinates
    fn a_text_node_shows_its_text_inside_its_rect() {
        let mut core = Core::with_renderer();
        core.add_output("test-1", (64, 48));
        let mut wm = core.named("wm");
        let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let props = json!({"text": "Hi", "font_size": 16.0,
            "color": [1.0, 1.0, 1.0, 1.0], "transform": {"translate": [8, 8]}});
        let create = json!({"id": "hi", "kind": "text", "parent": "root", "props": props});
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"create": create}]}),
        );
        let found = wm.ok(&mut core, "scene.query", json!({"selector": "id:hi"}));
        let rect = found[0]["rects"]["test-1"].as_array().cloned();
        let Some(rect) = rect else {
            assert!(!crate::text::fonts_present(), "the text node shows no rect");
            println!("SKIP: no fonts for the layout");
            return;
        };
        let [x, y, w, h] = [
            rect[0].as_f64().unwrap(),
            rect[1].as_f64().unwrap(),
            rect[2].as_f64().unwrap(),
            rect[3].as_f64().unwrap(),
        ];
        assert_eq!((x, y), (8.0, 8.0));
        assert!(w > 4.0 && h > 8.0, "the layout of Hi is {w}x{h}");
        let picture = wm.ok(&mut core, "screen.capture", json!({"output": "test-1"}));
        let (width, rgba) = decode(&picture);
        let black = [0, 0, 0, 255];
        let mut ink = 0;
        for py in 0..48usize {
            for px in 0..width {
                let pixel = &rgba[(py * width + px) * 4..][..4];
                let (fx, fy) = (px as f64, py as f64);
                if (x..x + w).contains(&fx) && (y..y + h).contains(&fy) {
                    ink += usize::from(pixel != black);
                }
                // One pixel of slack absorbs the rounding of the box.
                let outside = fx < x - 1.0 || fx > x + w + 1.0 || fy < y - 1.0 || fy > y + h + 1.0;
                assert!(
                    !(outside && pixel != black),
                    "ink outside the rect at {px},{py}"
                );
            }
        }
        assert!(ink >= 8, "{ink} ink pixels inside {w}x{h}");
    }

    #[test]
    fn flipping_swaps_the_rows() {
        let mut rows = vec![1, 1, 2, 2, 3, 3];
        flip_rows(&mut rows, 2);
        assert_eq!(rows, vec![3, 3, 2, 2, 1, 1]);
    }

    #[test]
    fn pictures_get_straight_alpha() {
        let pixels = Pixels {
            width: 1,
            height: 1,
            rgba: vec![64, 0, 0, 128],
        };
        let png = encode(&pixels).unwrap();
        let decoder = png::Decoder::new(std::io::Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut out = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut out).unwrap();
        assert_eq!(out, vec![127, 0, 0, 128]);
    }
}
