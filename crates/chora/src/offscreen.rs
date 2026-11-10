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
    use serde_json::json;
    use smithay::{
        backend::{allocator::Fourcc, renderer::ExportMem},
        utils::{Buffer, Rectangle, Size},
    };

    use super::*;
    use crate::{
        backend::Backend,
        testing::{Core, wayland::Wayland},
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

    #[test]
    fn render_into_draws_an_output_outside_the_backend_list() {
        let (mut core, output) = core_with_left_half_red((64, 48));
        let mut target = OutputTexture::new(output);
        assert_eq!(render_into(&mut core.state, &mut target), Ok(true));
        let texture = target.texture().expect("a texture");
        assert_eq!(texture.size(), (64, 48).into());
        let row = first_row(&mut core.state, &target);
        assert_eq!(row[8], [255, 0, 0, 255], "the rect");
        assert_eq!(row[56], [0, 0, 0, 255], "the clear color");
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

    #[test]
    fn a_rendered_frame_sends_frame_callbacks() {
        let mut core = Core::with_renderer();
        let output = core.add_output("VIRTUAL-1", (64, 48));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "window");
        wayland.attach(&mut core, &window, (20, 20));
        let mut wm = core.named("wm");
        let attach = json!({"commands": [{"attach": {"surface": "s1", "parent": "root"}}]});
        wm.ok(&mut core, "scene.apply", attach);
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
}
