//! The winit backend: chora as a window inside another session.

use std::time::Duration;

use chora_scene::{Event, OutputInfo};
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        egl::EGLDevice,
        renderer::{ImportDma, damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    desktop::utils::output_update,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    utils::{Rectangle, Transform},
    wayland::dmabuf::{
        DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier,
    },
};

use crate::{render, state::State};

/// The name of the one output of the winit backend.
const OUTPUT: &str = "winit";

/// The parts of the winit backend that outlive one event.
pub struct Backend {
    graphics: WinitGraphicsBackend<GlesRenderer>,
    damage_tracker: OutputDamageTracker,
    dmabuf_state: DmabufState,
    _dmabuf_global: DmabufGlobal,
}

/// Opens the window, creates its output and starts the render loop.
pub fn init(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    let (mut graphics, events) = winit::init::<GlesRenderer>()?;
    let mode = Mode {
        size: graphics.window_size(),
        refresh: 60_000,
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
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    let formats = graphics.renderer().dmabuf_formats();
    let mut dmabuf_state = DmabufState::new();
    let node = EGLDevice::device_for_display(graphics.renderer().egl_context().display())
        .ok()
        .and_then(|d| d.try_get_render_node().ok().flatten());
    let dmabuf_global = match node.and_then(|n| {
        DmabufFeedbackBuilder::new(n.dev_id(), formats.clone())
            .build()
            .ok()
    }) {
        Some(feedback) => dmabuf_state
            .create_global_with_default_feedback::<State>(&state.display_handle, &feedback),
        None => dmabuf_state.create_global::<State>(&state.display_handle, formats),
    };

    state.backend = Some(Backend {
        graphics,
        damage_tracker: OutputDamageTracker::from_output(&output),
        dmabuf_state,
        _dmabuf_global: dmabuf_global,
    });
    state.output = Some(output);
    state
        .loop_handle
        .insert_source(events, |event, (), state| handle(state, event))?;
    Ok(())
}

fn handle(state: &mut State, event: WinitEvent) {
    match event {
        WinitEvent::Resized { size, .. } => {
            let output = state.output.as_ref().expect("the winit output exists");
            output.change_current_state(
                Some(Mode {
                    size,
                    refresh: 60_000,
                }),
                None,
                None,
                None,
            );
            let info = output_info(output);
            state.emit(&Event::OutputChanged { output: info });
        }
        WinitEvent::Input(event) => crate::input::process(state, event),
        WinitEvent::Redraw => redraw(state),
        WinitEvent::CloseRequested => state.loop_signal.stop(),
        WinitEvent::Focus(_) => {}
    }
}

/// The output as the API describes it.
pub fn output_info(output: &Output) -> OutputInfo {
    let size = output
        .current_mode()
        .map_or((0, 0).into(), |m| m.size.to_logical(1));
    let position = output.current_location();
    OutputInfo {
        name: output.name(),
        position: [position.x, position.y],
        size: [size.w, size.h],
        scale: output.current_scale().fractional_scale(),
    }
}

fn redraw(state: &mut State) {
    let damage = render_frame(state);
    let Some(backend) = state.backend.as_mut() else {
        return;
    };
    if let Err(err) = backend.graphics.submit(damage.as_deref()) {
        tracing::warn!(?err, "submitting the frame failed");
    }
    send_frames(state);
    state.popups.cleanup();
    if let Some(backend) = state.backend.as_ref() {
        backend.graphics.window().request_redraw();
    }
}

/// Renders the scene into the window and returns the damage.
fn render_frame(state: &mut State) -> Option<Vec<Rectangle<i32, smithay::utils::Physical>>> {
    let State {
        backend,
        scene,
        windows,
        rects,
        ..
    } = state;
    let backend = backend.as_mut()?;
    let (renderer, mut framebuffer) = backend.graphics.bind().ok()?;
    let elements = render::elements(scene, windows, rects, renderer);
    // A full redraw: winit surfaces report no buffer age before a bind.
    let result = backend.damage_tracker.render_output(
        renderer,
        &mut framebuffer,
        0,
        &elements,
        [0.0, 0.0, 0.0, 1.0],
    );
    match result {
        Ok(result) => result.damage.cloned(),
        Err(err) => {
            tracing::warn!(?err, "rendering failed");
            None
        }
    }
}

/// Sends frame callbacks and output enter events to all toplevels.
fn send_frames(state: &State) {
    let Some(output) = state.output.as_ref() else {
        return;
    };
    let time = state.start_time.elapsed();
    for window in state.windows.values() {
        if let Some(toplevel) = window.toplevel() {
            output_update(
                output,
                Some(Rectangle::from_size(window.bbox().size)),
                toplevel.wl_surface(),
            );
        }
        window.send_frame(output, time, Some(Duration::ZERO), |_, _| {
            Some(output.clone())
        });
    }
}

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self
            .backend
            .as_mut()
            .expect("the backend exists")
            .dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        let backend = self.backend.as_mut().expect("the backend exists");
        if backend
            .graphics
            .renderer()
            .import_dmabuf(&dmabuf, None)
            .is_ok()
        {
            let _ = notifier.successful::<State>();
        } else {
            notifier.failed();
        }
    }
}
