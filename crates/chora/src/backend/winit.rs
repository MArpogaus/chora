//! The winit backend: chora as a window inside another session.

use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        egl::EGLDevice,
        renderer::{ImportDma, damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    desktop::layer_map_for_output,
    output::{Mode, Output, OutputModeSource, PhysicalProperties, Subpixel},
    reexports::wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
    utils::{Physical, Size, Transform},
    wayland::{dmabuf::DmabufFeedbackBuilder, presentation::Refresh},
};

use super::Backend;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};

use crate::{output::frame_duration, render, state::State};

/// The name of the one output of the winit backend.
const OUTPUT: &str = "winit";
/// The refresh rate that the winit output reports, in mHz.
const REFRESH: i32 = 60_000;
/// The answer to a mode change.
pub const FIXED_MODE: &str = "the window decides the mode of the winit output";
/// The answer to a request to turn the output off.
pub const ALWAYS_ON: &str = "the winit output cannot turn off";

/// The parts of the winit backend that outlive one event.
pub struct Winit {
    graphics: WinitGraphicsBackend<GlesRenderer>,
    /// Follows the mode, the scale and the transform of the window, see
    /// [`window_mode`].
    damage_tracker: OutputDamageTracker,
    /// The one output: the window.
    output: Output,
}

/// Opens the window, creates its output and starts the render loop.
pub fn init(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    let (mut graphics, events) = winit::init::<GlesRenderer>()?;
    let mode = Mode {
        size: graphics.window_size(),
        refresh: REFRESH,
    };
    let output = Output::new(
        OUTPUT.into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "chora".into(),
            model: "winit".into(),
            serial_number: String::new(),
        },
    );
    output.create_global::<State>(&state.display_handle);
    output.change_current_state(Some(mode), None, None, None);
    output.set_preferred(mode);

    let formats = graphics.renderer().dmabuf_formats();
    let dmabuf_state = &mut state.dmabuf_state;
    let node = EGLDevice::device_for_display(graphics.renderer().egl_context().display())
        .ok()
        .and_then(|d| d.try_get_render_node().ok().flatten());
    // The global stays when its handle drops.
    match node.and_then(|n| {
        DmabufFeedbackBuilder::new(n.dev_id(), formats.clone())
            .build()
            .ok()
    }) {
        Some(feedback) => dmabuf_state
            .create_global_with_default_feedback::<State>(&state.display_handle, &feedback),
        None => dmabuf_state.create_global::<State>(&state.display_handle, formats),
    };

    state.backend = Some(Backend::Winit(Box::new(Winit {
        graphics,
        damage_tracker: OutputDamageTracker::from_mode_source(window_mode(&output)),
        output: output.clone(),
    })));
    state.add_output(output);
    take_host_keymap(state);
    state
        .loop_handle
        .insert_source(events, |event, (), state| handle(state, event))?;
    Ok(())
}

/// Translates keys with the keymap of the host session, unless
/// `XKB_DEFAULT_LAYOUT` asks for another layout (#12).
fn take_host_keymap(state: &mut State) {
    if std::env::var_os("XKB_DEFAULT_LAYOUT").is_some() {
        return;
    }
    let keymap = super::host_keymap::fetch().ok_or("the host sent no keymap");
    let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
    let taken = keymap.and_then(|k| {
        keyboard
            .set_keymap_from_string(state, k)
            .map_err(|_| "the keymap of the host does not compile")
    });
    if let Err(err) = taken {
        tracing::warn!(err, "XKB_DEFAULT_LAYOUT sets the layout");
    }
}

fn handle(state: &mut State, event: WinitEvent) {
    match event {
        WinitEvent::Resized { size, .. } => {
            if let Some(backend) = winit(state) {
                resize(&backend.output, size);
            }
            state.output_changed();
        }
        WinitEvent::Input(event) => crate::input::process(state, event),
        WinitEvent::Redraw => redraw(state),
        WinitEvent::CloseRequested => state.loop_signal.stop(),
        // The host gets the releases of keys that went down here, for
        // example Super for its overview, so chora lets them go too (#11).
        WinitEvent::Focus(false) => crate::input::release_keys(state),
        WinitEvent::Focus(true) => {}
    }
}

/// Gives the output the size of the window as its one mode, so that output
/// configuration shows no old sizes.
fn resize(output: &Output, size: Size<i32, Physical>) {
    let old = output.current_mode();
    let mode = Mode {
        size,
        refresh: REFRESH,
    };
    output.change_current_state(Some(mode), None, None, None);
    output.set_preferred(mode);
    if let Some(old) = old.filter(|o| *o != mode) {
        output.delete_mode(old);
    }
    layer_map_for_output(output).arrange();
}

fn redraw(state: &mut State) {
    let Some(output) = winit(state).map(|w| w.output.clone()) else {
        return;
    };
    let start = std::time::Instant::now();
    let rendered = render_frame(state, &output);
    state.frame_rendered(&output.name(), frame_duration(&output), start.elapsed());
    let submitted = shown(state, &output, rendered);
    render::send_frames(state, &output);
    if submitted {
        // The swap waits for the host, which paces the next frame.
        if let Some(backend) = winit(state) {
            backend.graphics.window().request_redraw();
        }
    } else {
        redraw_later(state, frame_duration(&output));
    }
}

/// Notes a rendered frame for the lock, or logs why it failed. Returns
/// whether a frame went to the host.
fn shown(state: &mut State, output: &Output, rendered: Result<bool, String>) -> bool {
    match rendered {
        Ok(submitted) => {
            let generation = state.lock_generation();
            state.frame_shown(output, generation);
            submitted
        }
        Err(err) => {
            tracing::warn!("{err}");
            false
        }
    }
}

/// Asks for a redraw after a frame without damage, one refresh later, so
/// that the loop does not spin.
fn redraw_later(state: &mut State, delay: std::time::Duration) {
    let timer = Timer::from_duration(delay);
    let inserted = state.loop_handle.insert_source(timer, |_, (), state| {
        if let Some(backend) = winit(state) {
            backend.graphics.window().request_redraw();
        }
        TimeoutAction::Drop
    });
    if let Err(err) = inserted {
        tracing::warn!(?err, "no redraw timer");
    }
}

/// Renders the damaged parts of the scene into the window and shows them.
/// Returns whether a frame with damage went to the host.
fn render_frame(state: &mut State, output: &Output) -> Result<bool, String> {
    let view = render::View::of(state, output);
    let State {
        backend,
        scene,
        windows,
        assets,
        lock,
        ..
    } = state;
    let Some(Backend::Winit(backend)) = backend else {
        return Ok(false);
    };
    let mode = window_mode(output);
    if *backend.damage_tracker.mode() != mode {
        backend.damage_tracker = OutputDamageTracker::from_mode_source(mode);
    }
    // The age needs the surface of the window current. Layers and captures
    // render into textures and leave no surface current, so the query
    // failed with BAD_SURFACE in every frame of an animation and forced a
    // full redraw (#15). A bind makes the surface current first.
    drop(backend.graphics.bind().map_err(|e| format!("bind: {e}"))?);
    // 0 means a full redraw.
    let age = backend.graphics.buffer_age().unwrap_or(0);
    let damage = {
        let (renderer, mut framebuffer) =
            backend.graphics.bind().map_err(|e| format!("bind: {e}"))?;
        let mut parts = render::Parts {
            scene,
            windows,
            assets,
            lock: lock.as_ref(),
            depth: 0,
        };
        let elements = render::output_elements(&mut parts, renderer, output, &view);
        let result = backend
            .damage_tracker
            .render_output(
                renderer,
                &mut framebuffer,
                age,
                &elements,
                [0.0, 0.0, 0.0, 1.0],
            )
            .map_err(|e| format!("rendering failed: {e:?}"))?;
        result.damage.cloned()
    };
    let Some(damage) = damage else {
        return Ok(false);
    };
    let mut feedback = render::take_feedback(state, output);
    let Some(Backend::Winit(backend)) = &mut state.backend else {
        return Ok(false);
    };
    backend
        .graphics
        .submit(Some(&damage))
        .map_err(|e| format!("submitting the frame failed: {e}"))?;
    // The window shows the frame at an unknown time; now is the best guess.
    feedback.presented(
        state.clock.now(),
        Refresh::fixed(frame_duration(output)),
        0,
        wp_presentation_feedback::Kind::Vsync,
    );
    Ok(true)
}

/// How the window shows the output: in its mode and scale, and with its
/// transform followed by a vertical flip. GL draws the window upside down,
/// and the flip corrects that. The output itself keeps the transform that
/// clients see.
fn window_mode(output: &Output) -> OutputModeSource {
    OutputModeSource::Static {
        size: output.current_mode().map(|m| m.size).unwrap_or_default(),
        scale: output.current_scale().fractional_scale().into(),
        transform: flipped(output.current_transform()),
    }
}

/// The transform that shows an output transform in the window: the same
/// rotation and mirror, and the vertical flip that GL needs for a window.
fn flipped(transform: Transform) -> Transform {
    match transform {
        Transform::Normal => Transform::Flipped180,
        Transform::_90 => Transform::Flipped90,
        Transform::_180 => Transform::Flipped,
        Transform::_270 => Transform::Flipped270,
        Transform::Flipped => Transform::_180,
        Transform::Flipped90 => Transform::_90,
        Transform::Flipped180 => Transform::Normal,
        Transform::Flipped270 => Transform::_270,
    }
}

impl Winit {
    /// Compiles the shader of an effect, to check it.
    /// The renderer of the window.
    pub fn gles(&mut self) -> &mut GlesRenderer {
        self.graphics.renderer()
    }

    pub fn check_effect(&mut self, path: &str) -> Result<(), String> {
        crate::gl::check_effect(self.graphics.renderer(), path)
    }

    /// Whether the GPU can use a buffer of a client.
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        self.graphics.renderer().import_dmabuf(dmabuf, None).is_ok()
    }

    /// Accepts the current mode only: the window decides the size, and the
    /// output cannot turn off.
    pub fn test_output(&self, mode: Option<Mode>) -> Result<(), String> {
        match mode {
            Some(mode) if Some(mode) == self.output.current_mode() => Ok(()),
            Some(_) => Err(FIXED_MODE.into()),
            None => Err(ALWAYS_ON.into()),
        }
    }
}

fn winit(state: &State) -> Option<&Winit> {
    match &state.backend {
        Some(Backend::Winit(winit)) => Some(winit),
        _ => None,
    }
}
