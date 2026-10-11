//! The tty backend: chora as the session on real hardware, through libseat,
//! udev, DRM/KMS and libinput.
//!
//! Adapted from `udev.rs` of Smithay's anvil, MIT License (see
//! `LICENSES/MIT-Smithay.txt`). This version has no leases, no debug
//! overlays and no presentation feedback yet.
//!
//! Each output renders when its last frame reached the screen, or one frame
//! duration after a render without damage.

use std::{collections::HashMap, path::Path, time::Duration};

use smithay::{
    backend::{
        allocator::{
            Fourcc,
            dmabuf::Dmabuf,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        },
        drm::{
            DrmDevice, DrmDeviceFd, DrmEvent, DrmNode, NodeType,
            compositor::FrameFlags,
            exporter::gbm::GbmFramebufferExporter,
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
        },
        egl::context::ContextPriority,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            ImportDma, ImportMemWl,
            gles::GlesRenderer,
            multigpu::{GpuManager, MultiRenderer, gbm::GbmGlesBackend},
        },
        session::{Event as SessionEvent, Session, libseat::LibSeatSession},
        udev::{UdevBackend, UdevEvent, all_gpus, primary_gpu},
    },
    output::{Mode, Output, PhysicalProperties},
    reexports::{
        calloop::{
            RegistrationToken,
            timer::{TimeoutAction, Timer},
        },
        drm::control::{ModeTypeFlags, connector, crtc},
        input::Libinput,
        rustix::fs::OFlags,
        wayland_server::backend::GlobalId,
    },
    utils::{DeviceFd, Logical, Point},
    wayland::dmabuf::DmabufFeedbackBuilder,
};
use smithay_drm_extras::{
    display_info,
    drm_scanner::{DrmScanEvent, DrmScanner},
};

use super::Backend;
use crate::{
    cursor::{self, Cursors},
    output::output_area,
    render::{self, SceneElement, View},
    state::State,
};

type Gles = GbmGlesBackend<GlesRenderer, DrmDeviceFd>;
type Allocator = GbmAllocator<DrmDeviceFd>;
type Exporter = GbmFramebufferExporter<DrmDeviceFd>;
type TtyRenderer<'a> = MultiRenderer<'a, 'a, Gles, Gles>;

/// The scan-out formats, in the order of preference.
const COLOR_FORMATS: [Fourcc; 2] = [Fourcc::Abgr8888, Fourcc::Argb8888];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// The session, the GPUs and their outputs.
pub struct Tty {
    session: LibSeatSession,
    libinput: Libinput,
    /// The render node that renders all outputs and imports client buffers.
    primary_gpu: DrmNode,
    gpus: GpuManager<Gles>,
    devices: HashMap<DrmNode, Device>,
    cursors: Cursors,
}

/// One GPU with its connectors.
struct Device {
    manager: DrmOutputManager<Allocator, Exporter, (), DrmDeviceFd>,
    scanner: DrmScanner,
    render_node: DrmNode,
    surfaces: HashMap<crtc::Handle, Surface>,
    token: RegistrationToken,
}

/// One output on a connector.
struct Surface {
    output: Output,
    compositor: DrmOutput<Allocator, Exporter, (), DrmDeviceFd>,
    global: GlobalId,
    /// A frame waits for its vblank.
    queued: bool,
    /// The timer of the next render after a frame without damage.
    timer: Option<RegistrationToken>,
}

/// Opens the session and the GPUs, creates the outputs and starts reading
/// the input devices.
pub fn init(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    let (session, notifier) = LibSeatSession::new()?;
    let seat = session.seat();
    let primary_gpu = find_primary_gpu(&seat).ok_or("no GPU found")?;
    tracing::info!(%primary_gpu, "the primary GPU");
    let gpus = GpuManager::new(Gles::with_context_priority(ContextPriority::High))?;
    let mut libinput =
        Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(session.clone().into());
    libinput
        .udev_assign_seat(&seat)
        .map_err(|()| "libinput cannot use the seat")?;
    let udev = UdevBackend::new(&seat)?;
    let devices: Vec<_> = udev
        .device_list()
        .map(|(id, path)| (id, path.to_owned()))
        .collect();
    state.backend = Some(Backend::Tty(Box::new(Tty {
        session,
        libinput: libinput.clone(),
        primary_gpu,
        gpus,
        devices: HashMap::new(),
        cursors: Cursors::from_env(),
    })));
    for (id, path) in devices {
        log_error("a GPU failed", add_device(state, id, &path));
    }
    init_buffers(state)?;
    let handle = state.loop_handle.clone();
    handle
        .insert_source(LibinputInputBackend::new(libinput), |event, (), state| {
            crate::input::process(state, event);
        })
        .map_err(|e| e.error)?;
    handle
        .insert_source(notifier, |event, (), state| session_event(state, event))
        .map_err(|e| e.error)?;
    handle
        .insert_source(udev, |event, _, state| udev_event(state, event))
        .map_err(|e| e.error)?;
    Ok(())
}

impl Tty {
    /// Whether the primary GPU can use a buffer of a client.
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        self.gpus
            .single_renderer(&self.primary_gpu)
            .is_ok_and(|mut renderer| renderer.import_dmabuf(dmabuf, None).is_ok())
    }

    /// Switches to another virtual terminal.
    pub fn change_vt(&mut self, vt: i32) {
        log_error("changing the VT failed", self.session.change_vt(vt));
    }
}

fn tty(state: &mut State) -> Option<&mut Tty> {
    match &mut state.backend {
        Some(Backend::Tty(tty)) => Some(tty),
        _ => None,
    }
}

fn surface(state: &mut State, node: DrmNode, crtc: crtc::Handle) -> Option<&mut Surface> {
    tty(state)?.devices.get_mut(&node)?.surfaces.get_mut(&crtc)
}

fn log_error<T, E: std::fmt::Debug>(what: &str, result: Result<T, E>) {
    if let Err(err) = result {
        tracing::warn!(?err, "{what}");
    }
}

/// The render node of a GPU, or the node itself if it has none.
fn render_node(node: DrmNode) -> DrmNode {
    node.node_with_type(NodeType::Render)
        .and_then(Result::ok)
        .unwrap_or(node)
}

/// The GPU that the system marks as primary, or the first one.
fn find_primary_gpu(seat: &str) -> Option<DrmNode> {
    let primary = primary_gpu(seat)
        .ok()
        .flatten()
        .and_then(|path| DrmNode::from_path(path).ok());
    primary
        .or_else(|| {
            all_gpus(seat)
                .ok()?
                .into_iter()
                .find_map(|path| DrmNode::from_path(path).ok())
        })
        .map(render_node)
}

/// Shares the buffer formats of the primary GPU with the clients.
fn init_buffers(state: &mut State) -> Result<(), String> {
    let State {
        backend,
        dmabuf_state,
        shm_state,
        display_handle,
        ..
    } = state;
    let Some(Backend::Tty(tty)) = backend else {
        return Ok(());
    };
    let renderer = tty
        .gpus
        .single_renderer(&tty.primary_gpu)
        .map_err(|e| format!("the primary GPU: {e:?}"))?;
    shm_state.update_formats(renderer.shm_formats());
    let feedback = DmabufFeedbackBuilder::new(tty.primary_gpu.dev_id(), renderer.dmabuf_formats())
        .build()
        .map_err(|e| format!("dmabuf feedback: {e:?}"))?;
    // The global stays when its handle drops.
    dmabuf_state.create_global_with_default_feedback::<State>(display_handle, &feedback);
    Ok(())
}

/// Opens a GPU and creates the outputs of its connectors.
fn add_device(state: &mut State, id: u64, path: &Path) -> Result<(), String> {
    let err = |e: &dyn std::fmt::Debug| format!("{}: {e:?}", path.display());
    let node = DrmNode::from_dev_id(id).map_err(|e| err(&e))?;
    let State {
        backend,
        loop_handle,
        ..
    } = state;
    let Some(Backend::Tty(tty)) = backend else {
        return Ok(());
    };
    let flags = OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK;
    let fd = tty.session.open(path, flags).map_err(|e| err(&e))?;
    let fd = DrmDeviceFd::new(DeviceFd::from(fd));
    let (drm, notifier) = DrmDevice::new(fd.clone(), true).map_err(|e| err(&e))?;
    let gbm = GbmDevice::new(fd).map_err(|e| err(&e))?;
    let render_node = render_node(node);
    tty.gpus
        .as_mut()
        .add_node(render_node, gbm.clone())
        .map_err(|e| err(&e))?;
    let formats = tty
        .gpus
        .single_renderer(&render_node)
        .map_err(|e| err(&e))?
        .as_mut()
        .egl_context()
        .dmabuf_render_formats()
        .clone();
    let manager = DrmOutputManager::new(
        drm,
        GbmAllocator::new(
            gbm.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        ),
        GbmFramebufferExporter::new(gbm.clone(), render_node.into()),
        Some(gbm),
        COLOR_FORMATS,
        formats,
    );
    let token = loop_handle
        .insert_source(notifier, move |event, _, state| match event {
            DrmEvent::VBlank(crtc) => vblank(state, node, crtc),
            DrmEvent::Error(error) => tracing::warn!(?error, "a DRM device failed"),
        })
        .map_err(|e| err(&e.error))?;
    let device = Device {
        manager,
        scanner: DrmScanner::new(),
        render_node,
        surfaces: HashMap::new(),
        token,
    };
    tty.devices.insert(node, device);
    scan(state, node);
    Ok(())
}

/// Closes a GPU that disappeared.
fn remove_device(state: &mut State, node: DrmNode) {
    let crtcs: Vec<_> = tty(state)
        .and_then(|t| t.devices.get(&node))
        .map(|d| d.surfaces.keys().copied().collect())
        .unwrap_or_default();
    for crtc in crtcs {
        disconnect(state, node, crtc);
    }
    let Some(tty) = tty(state) else { return };
    if let Some(device) = tty.devices.remove(&node) {
        tty.gpus.as_mut().remove_node(&device.render_node);
        state.loop_handle.remove(device.token);
    }
}

/// Creates and removes outputs after the connectors of a GPU changed.
fn scan(state: &mut State, node: DrmNode) {
    let Some(device) = tty(state).and_then(|t| t.devices.get_mut(&node)) else {
        return;
    };
    let result = device.scanner.scan_connectors(device.manager.device());
    let Ok(events) = result.inspect_err(|err| tracing::warn!(?err, "scanning connectors failed"))
    else {
        return;
    };
    for event in events {
        match event {
            DrmScanEvent::Connected {
                connector,
                crtc: Some(crtc),
            } => connect(state, node, &connector, crtc),
            DrmScanEvent::Disconnected {
                crtc: Some(crtc), ..
            } => disconnect(state, node, crtc),
            _ => {}
        }
    }
}

fn connect(state: &mut State, node: DrmNode, connector: &connector::Info, crtc: crtc::Handle) {
    match create_output(state, node, connector, crtc) {
        Ok(output) => {
            state.add_output(output);
            render(state, node, crtc);
        }
        Err(err) => tracing::warn!(%err, "an output failed"),
    }
}

fn create_output(
    state: &mut State,
    node: DrmNode,
    connector: &connector::Info,
    crtc: crtc::Handle,
) -> Result<Output, String> {
    let name = format!(
        "{}-{}",
        connector.interface().as_str(),
        connector.interface_id()
    );
    let modes = connector.modes();
    let mode = *modes
        .iter()
        .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
        .or(modes.first())
        .ok_or_else(|| format!("{name} has no mode"))?;
    let State {
        backend,
        display_handle,
        ..
    } = state;
    let Some(Backend::Tty(tty)) = backend else {
        return Err("no tty backend".into());
    };
    let device = tty.devices.get_mut(&node).ok_or("the GPU is gone")?;
    let output = Output::new(name, properties(device, connector));
    output.set_preferred(Mode::from(mode));
    output.change_current_state(Some(Mode::from(mode)), None, None, None);
    let planes = device.manager.device().planes(&crtc).ok();
    let mut renderer = tty
        .gpus
        .single_renderer(&device.render_node)
        .map_err(|e| format!("{e:?}"))?;
    let compositor = device
        .manager
        .lock()
        .initialize_output::<_, SceneElement<TtyRenderer<'_>>>(
            crtc,
            mode,
            &[connector.handle()],
            &output,
            planes,
            &mut renderer,
            &DrmOutputRenderElements::default(),
        )
        .map_err(|e| format!("{}: {e:?}", output.name()))?;
    let surface = Surface {
        output: output.clone(),
        compositor,
        global: output.create_global::<State>(display_handle),
        queued: false,
        timer: None,
    };
    device.surfaces.insert(crtc, surface);
    Ok(output)
}

/// The size, make, model and serial number of a monitor, from its EDID.
// The type of the EDID info is private in smithay-drm-extras, so the
// closures cannot name its methods.
#[allow(clippy::redundant_closure_for_method_calls)]
fn properties(device: &Device, connector: &connector::Info) -> PhysicalProperties {
    let info = display_info::for_connector(device.manager.device(), connector.handle());
    let text = |v: Option<String>| v.unwrap_or_else(|| "Unknown".into());
    let (w, h) = connector.size().unwrap_or((0, 0));
    let mm = |v: u32| i32::try_from(v).unwrap_or(0);
    PhysicalProperties {
        size: (mm(w), mm(h)).into(),
        subpixel: connector.subpixel().into(),
        make: text(info.as_ref().and_then(|i| i.make())),
        model: text(info.as_ref().and_then(|i| i.model())),
        serial_number: text(info.as_ref().and_then(|i| i.serial())),
    }
}

fn disconnect(state: &mut State, node: DrmNode, crtc: crtc::Handle) {
    let Some(surface) = tty(state)
        .and_then(|t| t.devices.get_mut(&node))
        .and_then(|d| d.surfaces.remove(&crtc))
    else {
        return;
    };
    if let Some(timer) = surface.timer {
        state.loop_handle.remove(timer);
    }
    state.display_handle.remove_global::<State>(surface.global);
    state.remove_output(&surface.output);
}

/// Renders an output, unless a frame of it still waits for its vblank.
fn render(state: &mut State, node: DrmNode, crtc: crtc::Handle) {
    let Some(output) = surface(state, node, crtc)
        .filter(|s| !s.queued)
        .map(|s| s.output.clone())
    else {
        return;
    };
    let view = View {
        camera: state.camera(&output),
        scale: output.current_scale().fractional_scale(),
    };
    let pointer = pointer_on(state, &output);
    let queued = draw(state, node, crtc, &view, pointer).unwrap_or_else(|err| {
        log_error::<(), _>("rendering failed", Err(err));
        false
    });
    if !queued {
        schedule(state, node, crtc, frame_duration(&output));
    }
    render::send_frames(state, &output);
}

/// The pointer position on an output, in logical output coordinates.
fn pointer_on(state: &State, output: &Output) -> Option<Point<f64, Logical>> {
    let area = output_area(output).to_f64();
    area.contains(state.pointer_layout)
        .then(|| state.pointer_layout - area.loc)
}

/// Renders the cursor and the scene. Returns whether a frame was queued.
fn draw(
    state: &mut State,
    node: DrmNode,
    crtc: crtc::Handle,
    view: &View,
    pointer: Option<Point<f64, Logical>>,
) -> Result<bool, String> {
    let State {
        backend,
        scene,
        windows,
        rects,
        cursor_status,
        lock,
        ..
    } = state;
    let Some(Backend::Tty(tty)) = backend else {
        return Ok(false);
    };
    let device = tty.devices.get_mut(&node).ok_or("the GPU is gone")?;
    let surface = device.surfaces.get_mut(&crtc).ok_or("the output is gone")?;
    let output = surface.output.clone();
    let mut renderer = if device.render_node == tty.primary_gpu {
        tty.gpus.single_renderer(&device.render_node)
    } else {
        let format = surface.compositor.format();
        tty.gpus
            .renderer(&tty.primary_gpu, &device.render_node, format)
    }
    .map_err(|e| format!("{e:?}"))?;
    let mut elements = pointer.map_or_else(Vec::new, |p| {
        cursor::elements(
            &mut renderer,
            cursor_status,
            &mut tty.cursors,
            p,
            view.scale,
        )
    });
    let mut parts = render::Parts {
        scene,
        windows,
        rects,
        lock: lock.as_ref(),
    };
    elements.extend(render::output_elements(
        &mut parts,
        &mut renderer,
        &output,
        view,
    ));
    let frame = surface
        .compositor
        .render_frame(&mut renderer, &elements, BLACK, FrameFlags::DEFAULT)
        .map_err(|e| format!("{e:?}"))?;
    if frame.is_empty {
        return Ok(false);
    }
    surface
        .compositor
        .queue_frame(())
        .map_err(|e| format!("{e:?}"))?;
    surface.queued = true;
    Ok(true)
}

/// The frame reached the screen: the output can render again.
fn vblank(state: &mut State, node: DrmNode, crtc: crtc::Handle) {
    if let Some(surface) = surface(state, node, crtc) {
        surface.queued = false;
        log_error("a frame failed", surface.compositor.frame_submitted());
    }
    render(state, node, crtc);
}

/// Renders an output after a delay, in place of an earlier timer.
fn schedule(state: &mut State, node: DrmNode, crtc: crtc::Handle, delay: Duration) {
    let timer = Timer::from_duration(delay);
    let token = state
        .loop_handle
        .insert_source(timer, move |_, (), state| {
            if let Some(surface) = surface(state, node, crtc) {
                surface.timer = None;
            }
            render(state, node, crtc);
            TimeoutAction::Drop
        })
        .ok();
    let old = surface(state, node, crtc).and_then(|s| std::mem::replace(&mut s.timer, token));
    if let Some(old) = old {
        state.loop_handle.remove(old);
    }
}

/// The time between two frames of the current mode.
fn frame_duration(output: &Output) -> Duration {
    let refresh = output.current_mode().map_or(60_000, |m| m.refresh);
    Duration::from_secs_f64(1000.0 / f64::from(refresh.max(1)))
}

/// Another session took the devices, or gave them back.
fn session_event(state: &mut State, event: SessionEvent) {
    let Some(tty) = tty(state) else { return };
    match event {
        SessionEvent::PauseSession => {
            tty.libinput.suspend();
            for device in tty.devices.values_mut() {
                device.manager.pause();
            }
        }
        SessionEvent::ActivateSession => {
            log_error("resuming libinput failed", tty.libinput.resume());
            let mut outputs = Vec::new();
            for (node, device) in &mut tty.devices {
                log_error(
                    "resuming a GPU failed",
                    device.manager.lock().activate(false),
                );
                for (crtc, surface) in &mut device.surfaces {
                    surface.queued = false;
                    outputs.push((*node, *crtc));
                }
            }
            for (node, crtc) in outputs {
                render(state, node, crtc);
            }
        }
    }
}

fn udev_event(state: &mut State, event: UdevEvent) {
    match event {
        UdevEvent::Added { device_id, path } => {
            log_error("a GPU failed", add_device(state, device_id, &path));
        }
        UdevEvent::Changed { device_id } => {
            if let Ok(node) = DrmNode::from_dev_id(device_id) {
                scan(state, node);
            }
        }
        UdevEvent::Removed { device_id } => {
            if let Ok(node) = DrmNode::from_dev_id(device_id) {
                remove_device(state, node);
            }
        }
    }
}
