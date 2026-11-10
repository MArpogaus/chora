//! An output that no screen shows: its frames go into a texture, with
//! damage. The headless backend renders its outputs with it.

use std::time::Instant;

use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Offscreen, Texture,
            damage::OutputDamageTracker,
            gles::{GlesRenderer, GlesTexture},
        },
    },
    output::Output,
    reexports::wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
    utils::{Buffer, Size},
    wayland::presentation::Refresh,
};

use crate::{output::frame_duration, render, state::State};

/// The texture that the frames of an output go to.
pub struct OutputTexture {
    output: Output,
    damage_tracker: OutputDamageTracker,
    texture: Option<GlesTexture>,
}

impl OutputTexture {
    /// A texture for the frames of an output; the first render makes it.
    pub fn new(output: Output) -> Self {
        Self {
            damage_tracker: OutputDamageTracker::from_output(&output),
            output,
            texture: None,
        }
    }

    /// The output that renders into the texture.
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// The texture of the last frame, at the size of the mode.
    #[cfg(test)]
    pub fn texture(&self) -> Option<&GlesTexture> {
        self.texture.as_ref()
    }

    /// Renders the damaged parts of the output into the texture. Returns
    /// whether the frame had damage. A mode change makes a new texture,
    /// which redraws in full. The render waits for the GPU, so that the
    /// watchdog measures the work of the renderer, as on a screen.
    pub fn render(
        &mut self,
        gles: &mut GlesRenderer,
        parts: &mut render::Parts<'_>,
        view: &render::View,
    ) -> Result<bool, String> {
        let mode = self
            .output
            .current_mode()
            .ok_or("the output has no mode")?
            .size;
        let size: Size<i32, Buffer> = (mode.w, mode.h).into();
        let mut age = 1;
        if self.texture.as_ref().is_none_or(|t| t.size() != size) {
            self.texture = Some(
                gles.create_buffer(Fourcc::Abgr8888, size)
                    .map_err(|e| e.to_string())?,
            );
            age = 0;
        }
        let elements = render::output_elements(parts, gles, &self.output, view);
        let texture = self.texture.as_mut().ok_or("no texture")?;
        let mut framebuffer = gles.bind(texture).map_err(|e| e.to_string())?;
        let result = self
            .damage_tracker
            .render_output(gles, &mut framebuffer, age, &elements, [0.0, 0.0, 0.0, 1.0])
            .map_err(|e| format!("rendering failed: {e:?}"))?;
        let _ = result.sync.wait();
        let damaged = result.damage.is_some();
        parts.assets.stats.frame(damaged);
        Ok(damaged)
    }
}

/// Renders a frame of an output with `render`, then tells the clients, the
/// lock and the watchdog: presentation feedback after damage, and the
/// frame callbacks.
pub fn render_and_present(
    state: &mut State,
    output: &Output,
    render: impl FnOnce(&mut State) -> Result<bool, String>,
) {
    let start = Instant::now();
    match render(state) {
        Ok(damaged) => {
            if damaged {
                let mut feedback = render::take_feedback(state, output);
                feedback.presented(
                    state.clock.now(),
                    Refresh::fixed(frame_duration(output)),
                    0,
                    wp_presentation_feedback::Kind::Vsync,
                );
            }
            let generation = state.lock_generation();
            state.frame_shown(output, generation);
        }
        Err(err) => tracing::warn!(output = output.name(), "{err}"),
    }
    state.frame_rendered(&output.name(), frame_duration(output), start.elapsed());
    render::send_frames(state, output);
}

#[cfg(test)]
mod tests {
    use chora_scene::ipc::RpcError;
    use serde_json::{Value, json};
    use smithay::{
        backend::{allocator::Fourcc, renderer::ExportMem},
        utils::{Buffer, Point, Rectangle, Size},
    };

    use super::*;
    use crate::{
        backend::Backend,
        testing::{
            Core,
            wayland::{Toplevel, Wayland},
        },
    };

    fn render_into(state: &mut State, target: &mut OutputTexture) -> Result<bool, String> {
        let view = render::View::of(state, target.output());
        let State {
            backend,
            scene,
            windows,
            assets,
            lock,
            ..
        } = state;
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
        target.render(headless.gles(), &mut parts, &view)
    }

    /// The first row of the texture, as RGBA.
    fn first_row(state: &mut State, target: &OutputTexture) -> Vec<[u8; 4]> {
        let Some(Backend::Headless(headless)) = &mut state.backend else {
            panic!("a headless backend");
        };
        let texture = target.texture().expect("a texture");
        let size: Size<i32, Buffer> = texture.size();
        let gles = headless.gles();
        let mapping = gles
            .copy_texture(texture, Rectangle::from_size(size), Fourcc::Abgr8888)
            .expect("a copy");
        let rgba = gles.map_texture(&mapping).expect("a mapping");
        let width = usize::try_from(size.w).expect("a width");
        rgba[..width * 4].as_chunks::<4>().0.to_vec()
    }

    fn core_with_left_half_red(size: (i32, i32)) -> (Core, Output) {
        let mut core = Core::with_renderer();
        let output = core.add_output("VIRTUAL-1", size);
        let mut wm = core.named("wm");
        let rect = json!({"commands": [{"create": {"id": "r", "kind": "rect",
            "parent": "root", "props": {"size": [size.0 / 2, size.1],
            "color": [1, 0, 0, 1]}}}]});
        wm.ok(&mut core, "scene.apply", rect);
        (core, output)
    }

    /// An output that the backend does not know renders its scene into a
    /// texture of the size of its mode: the left half red, the rest the
    /// clear color.
    #[test]
    fn an_output_renders_its_scene_into_a_texture_of_its_mode_size() {
        const IN_THE_RECT: usize = 8;
        const RIGHT_OF_THE_RECT: usize = 56;
        let (mut core, output) = core_with_left_half_red((64, 48));
        let mut target = OutputTexture::new(output);
        assert_eq!(render_into(&mut core.state, &mut target), Ok(true));
        let texture = target.texture().expect("a texture");
        assert_eq!(texture.size(), (64, 48).into());
        let row = first_row(&mut core.state, &target);
        assert_eq!(row[IN_THE_RECT], [255, 0, 0, 255], "the rect");
        assert_eq!(row[RIGHT_OF_THE_RECT], [0, 0, 0, 255], "the clear color");
    }

    #[test]
    fn render_into_redraws_only_damage() {
        let (mut core, output) = core_with_left_half_red((32, 32));
        let mut target = OutputTexture::new(output);
        let before = core.state.assets.stats.clone();
        let damaged: Vec<_> = (0..3)
            .map(|_| render_into(&mut core.state, &mut target))
            .collect();
        assert_eq!(damaged, [Ok(true), Ok(false), Ok(false)]);
        let after = core.state.assets.stats.clone();
        assert_eq!(after.frames - before.frames, 3);
        assert_eq!(after.damaged_frames - before.damaged_frames, 1);
    }

    /// A core with a window on an output that the scene shows.
    fn core_with_a_window() -> (Core, Output, Wayland, Toplevel) {
        let mut core = Core::with_renderer();
        let output = core.add_output("VIRTUAL-1", (64, 48));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "window");
        wayland.attach(&mut core, &window, (20, 20));
        let mut wm = core.named("wm");
        let attach = json!({"commands": [{"attach": {"surface": "s1", "parent": "root"}}]});
        wm.ok(&mut core, "scene.apply", attach);
        (core, output, wayland, window)
    }

    #[test]
    fn a_rendered_frame_sends_frame_callbacks() {
        let (mut core, output, mut wayland, window) = core_with_a_window();
        let mut target = OutputTexture::new(output.clone());
        for _ in 0..3 {
            wayland.frame(&mut core, &window);
            render_and_present(&mut core.state, &output, |state| {
                render_into(state, &mut target)
            });
            wayland.roundtrip(&mut core);
        }
        assert_eq!(wayland.configures(&window).frames, 3);
    }

    #[test]
    fn a_failed_render_still_sends_frame_callbacks() {
        let (mut core, output, mut wayland, window) = core_with_a_window();
        wayland.frame(&mut core, &window);
        render_and_present(&mut core.state, &output, |_| Err("no GPU".into()));
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.configures(&window).frames, 1);
    }

    #[test]
    fn presentation_feedback_follows_damage_only() {
        let (mut core, output, mut wayland, window) = core_with_a_window();
        wayland.feedback(&mut core, &window);
        render_and_present(&mut core.state, &output, |_| Ok(false));
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.presented(), 0, "no damage");
        render_and_present(&mut core.state, &output, |_| Ok(true));
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.presented(), 1, "damage");
    }

    /// A pending lock is confirmed once every output showed a frame.
    #[test]
    fn a_rendered_frame_counts_as_shown_for_the_lock() {
        let (mut core, output, mut wayland, _window) = core_with_a_window();
        let _lock = wayland.lock(&mut core);
        assert!(!wayland.locked(), "no frame yet");
        render_and_present(&mut core.state, &output, |_| Ok(false));
        wayland.roundtrip(&mut core);
        assert!(wayland.locked());
    }

    #[test]
    fn the_watchdog_measures_the_render() {
        let (mut core, output, _wayland, _window) = core_with_a_window();
        let over_budget = frame_duration(&output) * 2;
        render_and_present(&mut core.state, &output, |_| {
            std::thread::sleep(over_budget);
            Ok(false)
        });
        assert!(core.state.watchdog.slow(&output.name()));
        render_and_present(&mut core.state, &output, |_| Ok(false));
        assert!(!core.state.watchdog.slow(&output.name()));
    }

    /// The corner of the first slot in the layout.
    const SLOT_1: i32 = -262_144;

    fn add_virtual(core: &mut Core, owner: &mut crate::testing::Client, spec: &str) -> Value {
        owner.ok(core, "output.add", json!({"spec": spec, "virtual": true}))
    }

    /// A core without a backend, a virtual output of "construct" that came
    /// before the screen, and the screen.
    fn core_with_a_virtual_output() -> (Core, crate::testing::Client, Output) {
        let mut core = Core::new();
        let mut construct = core.named("construct");
        let added = add_virtual(&mut core, &mut construct, "32x32");
        assert_eq!(added["name"], "VIRTUAL-1");
        let screen = core.add_output("screen", (64, 48));
        (core, construct, screen)
    }

    fn names(outputs: &[Output]) -> Vec<String> {
        outputs.iter().map(Output::name).collect()
    }

    #[test]
    fn a_virtual_output_is_an_output_to_a_window_manager() {
        let (mut core, mut construct, _screen) = core_with_a_virtual_output();
        let list = construct.ok(&mut core, "output.list", json!({}));
        let listed: Vec<_> = list.as_array().expect("a list").iter().collect();
        assert_eq!(listed[0]["name"], "screen");
        assert_eq!(listed[0].get("virtual"), None, "a screen keeps its form");
        let face = listed[1];
        assert_eq!(
            (&face["name"], &face["virtual"], &face["owner"]),
            (&json!("VIRTUAL-1"), &json!(true), &json!("construct"))
        );
        assert_eq!(face["position"], json!([SLOT_1, SLOT_1]));
        assert_eq!(face["work_area"], json!([SLOT_1, SLOT_1, 32, 32]));
        let overlay = json!({"selector": "id:screen:VIRTUAL-1"});
        let found = construct.ok(&mut core, "scene.query", overlay);
        assert_eq!(found.as_array().map(Vec::len), Some(1), "an overlay root");
        let second = add_virtual(&mut core, &mut construct, "16x16@2");
        assert_eq!(second["name"], "VIRTUAL-2");
        assert_eq!(second["position"], json!([2 * SLOT_1, SLOT_1]));
        assert_eq!(second["size"], json!([8, 8]));
    }

    #[test]
    fn a_virtual_output_is_in_no_list_of_screens() {
        let (mut core, mut construct, screen) = core_with_a_virtual_output();
        assert_eq!(names(&core.state.outputs), ["screen"], "the pointer layout");
        assert_eq!(
            core.state.output_management.head_names(),
            ["screen"],
            "the heads of output management"
        );
        let grant = json!({"client": "construct", "capabilities": ["screen.capture"]});
        core.admin().ok(&mut core, "grants.set", grant);
        core.state.backend = Some(Backend::Headless(Box::new(
            crate::backend::headless::Headless::new().expect("a software renderer"),
        )));
        let picture = construct.ok(&mut core, "screen.capture", json!({}));
        assert_eq!(
            (&picture["width"], &picture["height"]),
            (&json!(64), &json!(48)),
            "the capture of the first screen"
        );
        let mut wayland = Wayland::connect(&mut core);
        let _lock = wayland.lock(&mut core);
        render_and_present(&mut core.state, &screen, |_| Ok(false));
        wayland.roundtrip(&mut core);
        assert!(wayland.locked(), "the lock waits for screens only");
    }

    #[test]
    fn the_pointer_never_enters_one() {
        let (mut core, mut construct, _screen) = core_with_a_virtual_output();
        let inside = Point::from((f64::from(SLOT_1) + 5.0, f64::from(SLOT_1) + 5.0));
        assert!(core.state.spot(inside).is_none(), "no spot");
        let clamped = core.state.clamp_to_outputs(inside);
        assert_eq!((clamped.x, clamped.y), (0.0, 0.0), "the nearest screen");
        let grant = json!({"client": "construct", "capabilities": ["input.inject"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let onto = json!({"events": [
            {"pointer_move": {"output": "VIRTUAL-1", "position": [5.0, 5.0]}}]});
        let (_, error) = construct.fails(&mut core, "input.inject", onto);
        assert!(error.contains("no such output"), "{error}");
    }

    #[test]
    fn a_side_has_1_to_4096_pixels_and_the_core_picks_the_place() {
        let mut core = Core::new();
        let mut construct = core.named("construct");
        for bad in ["4097x8", "8x4097", "0x8", "8x8+0+0", "8x8,8x8"] {
            let add = json!({"spec": bad, "virtual": true});
            let (_, error) = construct.fails(&mut core, "output.add", add);
            assert!(!error.contains("headless"), "{bad}: {error}");
        }
    }

    /// An `output.add` that fails at a limit whose message holds a word.
    fn hits_the_limit(core: &mut Core, owner: &mut crate::testing::Client, word: &str) {
        let add = json!({"spec": "1x1", "virtual": true});
        let (number, error) = owner.fails(core, "output.add", add);
        assert_eq!(number, RpcError::LIMIT_REACHED, "{error}");
        assert!(error.contains(word), "{error}");
    }

    #[test]
    fn the_17th_and_the_pixel_budget_fail() {
        let mut core = Core::new();
        let mut construct = core.named("construct");
        for n in 1..=16 {
            let added = add_virtual(&mut core, &mut construct, "8x8");
            assert_eq!(added["name"], format!("VIRTUAL-{n}"));
        }
        hits_the_limit(&mut core, &mut construct, "16");
        for n in 1..=16 {
            let remove = json!({"output": format!("VIRTUAL-{n}")});
            construct.ok(&mut core, "output.remove", remove);
        }
        for _ in 0..4 {
            add_virtual(&mut core, &mut construct, "4096x4096");
        }
        hits_the_limit(&mut core, &mut construct, "pixels");
        let list = construct.ok(&mut core, "output.list", json!({}));
        assert_eq!(list.as_array().map(Vec::len), Some(4));
    }

    #[test]
    fn only_the_owner_removes_it() {
        let (mut core, mut construct, _screen) = core_with_a_virtual_output();
        let remove = json!({"output": "VIRTUAL-1"});
        let mut other = core.named("xp");
        let (_, error) = other.fails(&mut core, "output.remove", remove.clone());
        assert!(error.contains("construct"), "{error}");
        let subscribe = json!({"events": ["output.removed"]});
        other.ok(&mut core, "events.subscribe", subscribe);
        construct.ok(&mut core, "output.remove", remove.clone());
        assert_eq!(other.event_names(&mut core), ["output.removed"]);
        let list = construct.ok(&mut core, "output.list", json!({}));
        assert_eq!(list.as_array().map(Vec::len), Some(1), "the screen stays");
        let again = add_virtual(&mut core, &mut construct, "32x32");
        assert_eq!(again["name"], "VIRTUAL-1", "the slot is free again");
        core.admin().ok(&mut core, "output.remove", remove);
        let (_, error) = construct.fails(&mut core, "output.remove", json!({"output": "screen"}));
        assert!(error.contains("headless"), "{error}");
    }
}
