//! Guards of the 2D path: what a scene without 3D content costs and draws
//! on the headless renderer. They hold the numbers of the 2D path before
//! 3D work, so that a change of the render path shows as a failing test.

use serde_json::{Value, json};
use smithay::{
    backend::renderer::{Bind, Frame, Renderer},
    output::Output,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::KeyboardInteractivity,
};

use super::{Headless, render_all};
use crate::{
    backend::Backend,
    gl,
    render::{self, RenderStats, Variant},
    testing::{Client, Core, wayland::Wayland},
};

const OUTPUT: &str = "HEADLESS-1";

fn core_with_output() -> (Core, Client, Output) {
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    wm.ok(&mut core, "output.add", json!({"spec": "128x96"}));
    let output = core
        .state
        .outputs
        .iter()
        .find(|o| o.name() == OUTPUT)
        .expect("the output")
        .clone();
    (core, wm, output)
}

fn stats(core: &Core) -> RenderStats {
    core.state.assets.stats.clone()
}

fn render_frames(core: &mut Core, frames: usize) {
    for _ in 0..frames {
        render_all(&mut core.state);
    }
}

fn png(side: u32) -> String {
    static FILES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let number = FILES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("chora-guard-{}-{number}.png", std::process::id()));
    let file = std::io::BufWriter::new(std::fs::File::create(&path).expect("a file"));
    let mut encoder = png::Encoder::new(file, side, side);
    encoder.set_color(png::ColorType::Grayscale);
    let mut writer = encoder.write_header().expect("a header");
    writer
        .write_image_data(&vec![128; (side * side) as usize])
        .expect("pixels");
    path.to_str().expect("a path").to_owned()
}

fn apply(wm: &mut Client, core: &mut Core, commands: &[Value]) {
    let sent = wm.send("scene.apply", json!({"commands": commands}));
    for _ in 0..1000 {
        if let Some(answer) = wm.answer(core, sent) {
            answer.expect("the commands apply");
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("no answer after the decode");
}

/// A rect, an image, a text, a window and a layer surface under a camera
/// that zooms. No node is a layer.
fn flat_scene(core: &mut Core, wm: &mut Client) -> (Wayland, Vec<Variant>) {
    let mut wayland = Wayland::connect(core);
    let layer = wayland.layer(core, Layer::Top, KeyboardInteractivity::None);
    wayland.attach_layer(core, &layer, (100, 30));
    let window = wayland.toplevel(core, "window");
    wayland.attach(core, &window, (40, 30));
    let path = png(8);
    let at = |x: u32, y: u32| json!({"translate": [x, y]});
    let mut commands = vec![
        json!({"camera": {"output": OUTPUT, "transform": {"scale": 2}}}),
        json!({"create": {"id": "r", "kind": "rect", "parent": "root",
            "props": {"size": [10, 10], "color": [1, 0, 0, 1], "transform": at(2, 2)}}}),
        json!({"create": {"id": "i", "kind": "image", "parent": "root",
            "props": {"image": &path, "transform": at(14, 2)}}}),
    ];
    let mut expected = vec![Variant::Surface, Variant::Scaled];
    if crate::text::fonts_present() {
        commands.push(
            json!({"create": {"id": "t", "kind": "text", "parent": "root",
            "props": {"text": "Hi", "font_size": 8.0, "color": [1, 1, 1, 1],
                      "transform": at(26, 2)}}}),
        );
        expected.push(Variant::Text);
    }
    commands.push(json!({"attach": {"surface": "s1", "parent": "root"}}));
    expected.extend([Variant::Image, Variant::Solid]);
    apply(wm, core, &commands);
    std::fs::remove_file(&path).expect("the file is there");
    (wayland, expected)
}

fn variants(core: &mut Core, output: &Output) -> Vec<Variant> {
    let view = render::View::of(&core.state, output);
    let crate::state::State {
        backend,
        scene,
        windows,
        assets,
        lock,
        ..
    } = &mut core.state;
    let Some(Backend::Headless(headless)) = backend else {
        panic!("a headless backend");
    };
    let mut parts = render::Parts {
        scene,
        windows,
        assets,
        lock: lock.as_ref(),
        depth: 0,
    };
    render::output_elements(&mut parts, headless.gles(), output, &view)
        .iter()
        .map(render::SceneElement::variant)
        .collect()
}

#[test]
fn a_2d_scene_takes_no_3d_path() {
    let (mut core, mut wm, output) = core_with_output();
    let (_wayland, expected) = flat_scene(&mut core, &mut wm);
    let before = stats(&core);
    render_frames(&mut core, 1);
    assert_eq!(variants(&mut core, &output), expected);
    let after = stats(&core);
    assert_eq!(after.elements_of(Variant::Mesh), 0);
    assert_eq!(after.offscreen(), before.offscreen());
    assert_eq!(after.offscreen(), 0, "no layer renders into a texture");
}

#[test]
fn the_output_target_has_no_depth_attachment() {
    let (mut core, mut wm, _output) = core_with_output();
    let (_wayland, _) = flat_scene(&mut core, &mut wm);
    render_frames(&mut core, 1);
    let Some(Backend::Headless(headless)) = &mut core.state.backend else {
        panic!("a headless backend");
    };
    let Headless {
        renderer, outputs, ..
    } = &mut **headless;
    let texture = outputs[0].texture.as_mut().expect("a rendered texture");
    let size = smithay::backend::renderer::Texture::size(texture);
    let mut framebuffer = renderer.bind(texture).expect("a bind");
    let mut frame = renderer
        .render(
            &mut framebuffer,
            (size.w, size.h).into(),
            smithay::utils::Transform::Normal,
        )
        .expect("a frame");
    let [color, depth, stencil] = gl::attachments(&mut frame).expect("a query");
    drop(frame.finish().expect("a finished frame"));
    drop(framebuffer);
    assert_ne!(color, 0, "the query reads the framebuffer of the output");
    assert_eq!((depth, stencil), (0, 0), "GL_NONE for depth and stencil");
}

#[test]
fn a_static_2d_scene_renders_once() {
    let (mut core, mut wm, _output) = core_with_output();
    let (_wayland, _) = flat_scene(&mut core, &mut wm);
    let before = stats(&core);
    render_frames(&mut core, 60);
    let after = stats(&core);
    assert_eq!(after.frames - before.frames, 60);
    assert_eq!(after.damaged_frames - before.damaged_frames, 1, "frame 1");
    assert_eq!(after.offscreen(), before.offscreen());
}

#[test]
fn a_rounded_rect_takes_two_offscreen_passes_per_frame() {
    let (mut core, mut wm, output) = core_with_output();
    let rect = json!({"create": {"id": "r", "kind": "rect", "parent": "root",
        "props": {"size": [40, 30], "color": [1, 0, 0, 1], "radius": 6,
                  "transform": {"translate": [4, 4]}}}});
    apply(&mut wm, &mut core, &[rect]);
    assert_eq!(variants(&mut core, &output), [Variant::Mesh]);
    for frame in 0..3 {
        let before = stats(&core);
        render_frames(&mut core, 1);
        let after = stats(&core);
        assert_eq!(
            after.textures_rendered - before.textures_rendered,
            1,
            "the content, frame {frame}"
        );
        assert_eq!(after.passes - before.passes, 1, "the shape, frame {frame}");
        assert_eq!(after.offscreen() - before.offscreen(), 2);
    }
}
