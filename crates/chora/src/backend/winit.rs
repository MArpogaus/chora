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
    utils::{Physical, Size, Transform},
    wayland::dmabuf::DmabufFeedbackBuilder,
};

use super::Backend;
use crate::{render, state::State};

/// The name of the one output of the winit backend.
const OUTPUT: &str = "winit";
/// The refresh rate that the winit output reports, in mHz.
const REFRESH: i32 = 60_000;

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
    state
        .loop_handle
        .insert_source(events, |event, (), state| handle(state, event))?;
    Ok(())
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
        WinitEvent::Focus(_) => {}
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
    if warn_on_error(&render_frame(state, &output)) {
        state.frame_shown(&output);
    }
    render::send_frames(state, &output);
    if let Some(backend) = winit(state) {
        backend.graphics.window().request_redraw();
    }
}

/// Logs an error. Returns whether there was none.
fn warn_on_error(result: &Result<(), String>) -> bool {
    if let Err(err) = result {
        tracing::warn!("{err}");
    }
    result.is_ok()
}

/// Renders the damaged parts of the scene into the window and shows them.
fn render_frame(state: &mut State, output: &Output) -> Result<(), String> {
    let view = render::View {
        camera: state.camera(output),
        scale: output.current_scale().fractional_scale(),
    };
    let State {
        backend,
        scene,
        windows,
        rects,
        lock,
        ..
    } = state;
    let Some(Backend::Winit(backend)) = backend else {
        return Ok(());
    };
    let mode = window_mode(output);
    if *backend.damage_tracker.mode() != mode {
        backend.damage_tracker = OutputDamageTracker::from_mode_source(mode);
    }
    // The first query fails before the first bind; 0 means a full redraw.
    let age = backend.graphics.buffer_age().unwrap_or(0);
    let damage = {
        let (renderer, mut framebuffer) =
            backend.graphics.bind().map_err(|e| format!("bind: {e}"))?;
        let mut parts = render::Parts {
            scene,
            windows,
            rects,
            lock: lock.as_ref(),
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
    backend
        .graphics
        .submit(damage.as_deref())
        .map_err(|e| format!("submitting the frame failed: {e}"))
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
    /// Whether the GPU can use a buffer of a client.
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        self.graphics.renderer().import_dmabuf(dmabuf, None).is_ok()
    }

    /// Accepts the current mode only: the window decides the size, and the
    /// output cannot turn off.
    pub fn test_output(&self, mode: Option<Mode>) -> Result<(), String> {
        match mode {
            Some(mode) if Some(mode) == self.output.current_mode() => Ok(()),
            Some(_) => Err("the window decides the mode of the winit output".into()),
            None => Err("the winit output cannot turn off".into()),
        }
    }
}

fn winit(state: &State) -> Option<&Winit> {
    match &state.backend {
        Some(Backend::Winit(winit)) => Some(winit),
        _ => None,
    }
}
