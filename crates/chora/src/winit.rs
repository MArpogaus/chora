//! The winit backend: chora as a window inside another session.

use std::{collections::HashSet, time::Duration};

use chora_scene::Event;
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        egl::EGLDevice,
        renderer::{ImportDma, damage::OutputDamageTracker, gles::GlesRenderer},
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    desktop::space::SpaceElement,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    utils::{Rectangle, Transform},
    wayland::dmabuf::{
        DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier,
    },
};

use crate::{render, state::State};

/// The name of the one output of the winit backend.
const OUTPUT: &str = "winit";
/// The refresh rate that the winit output reports, in mHz.
const REFRESH: i32 = 60_000;

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
                    refresh: REFRESH,
                }),
                None,
                None,
                None,
            );
            for output in state.output_info() {
                state.emit(&Event::OutputChanged { output });
            }
        }
        WinitEvent::Input(event) => crate::input::process(state, event),
        WinitEvent::Redraw => redraw(state),
        WinitEvent::CloseRequested => state.loop_signal.stop(),
        WinitEvent::Focus(_) => {}
    }
}

fn redraw(state: &mut State) {
    if let Err(err) = render_frame(state) {
        tracing::warn!("{err}");
    }
    send_frames(state);
    state.popups.cleanup();
    if let Some(backend) = state.backend.as_ref() {
        backend.graphics.window().request_redraw();
    }
}

/// Renders the damaged parts of the scene into the window and shows them.
fn render_frame(state: &mut State) -> Result<(), String> {
    let State {
        backend,
        scene,
        windows,
        rects,
        ..
    } = state;
    let Some(backend) = backend.as_mut() else {
        return Ok(());
    };
    // The first query fails before the first bind; 0 means a full redraw.
    let age = backend.graphics.buffer_age().unwrap_or(0);
    let damage = {
        let (renderer, mut framebuffer) =
            backend.graphics.bind().map_err(|e| format!("bind: {e}"))?;
        let elements = render::elements(scene, windows, rects, renderer);
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

/// Sends frame callbacks: every frame to the visible toplevels, once per
/// second to the others, so that hidden clients do not draw for nothing.
fn send_frames(state: &State) {
    let Some(output) = state.output.as_ref() else {
        return;
    };
    let visible: HashSet<&str> = state
        .scene
        .draw_list()
        .iter()
        .map(|p| p.node.id.as_str())
        .collect();
    let time = state.start_time.elapsed();
    for (id, window) in &state.windows {
        let shown = visible.contains(id.as_str());
        if shown {
            window.output_enter(output, Rectangle::from_size(window.bbox().size));
        } else {
            window.output_leave(output);
        }
        let throttle = if shown {
            Duration::ZERO
        } else {
            Duration::from_secs(1)
        };
        window.send_frame(output, time, Some(throttle), |_, _| Some(output.clone()));
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
