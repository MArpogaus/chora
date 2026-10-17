//! The tty backend: chora as the session on real hardware, through libseat,
//! udev, DRM/KMS and libinput.
//!
//! Adapted from `udev.rs` of Smithay's anvil, MIT License (see
//! `LICENSES/MIT-Smithay.txt`). This version has no leases and no debug
//! overlays.
//!
//! Each output renders when its last frame reached the screen, or one frame
//! duration after a render without damage.
//!
//! Output configuration can give a connector another of its modes, and turn
//! an output off and on again. An output that is off keeps its connector
//! and its CRTC, but has no global.

use std::{collections::HashMap, path::Path, time::Duration};

use smithay::{
    backend::{
        allocator::{
            Fourcc,
            dmabuf::Dmabuf,
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
        },
        drm::{
            DrmDevice, DrmDeviceFd, DrmError, DrmEvent, DrmEventMetadata, DrmEventTime, DrmNode,
            NodeType,
            compositor::FrameFlags,
            exporter::gbm::GbmFramebufferExporter,
            output::{DrmOutput, DrmOutputManager, DrmOutputRenderElements},
        },
        egl::context::ContextPriority,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            ImportDma, ImportMemWl, RendererSuper,
            gles::{GlesError, GlesFrame, GlesRenderer},
            multigpu::{Error as MultiError, GpuManager, MultiRenderer, gbm::GbmGlesBackend},
        },
        session::{Event as SessionEvent, Session, libseat::LibSeatSession},
        udev::{UdevBackend, UdevEvent, all_gpus, primary_gpu},
    },
    desktop::utils::OutputPresentationFeedback,
    input::pointer::CursorImageStatus,
    output::{Mode, Output, PhysicalProperties},
    reexports::{
        calloop::{
            LoopHandle, RegistrationToken,
            timer::{TimeoutAction, Timer},
        },
        drm::control::{self, ModeTypeFlags, connector, crtc},
        input::Libinput,
        rustix::fs::OFlags,
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback::Kind,
        wayland_server::{DisplayHandle, backend::GlobalId},
    },
    utils::{DeviceFd, Logical, Monotonic, Point, Time},
    wayland::{dmabuf::DmabufFeedbackBuilder, presentation::Refresh},
};
use smithay_drm_extras::{
    display_info,
    drm_scanner::{DrmScanEvent, DrmScanner},
};

use super::Backend;
use crate::{
    cursor::{self, Cursors},
    mesh::MeshRenderer,
    output::{frame_duration, output_area},
    render::{self, SceneElement, View},
    state::State,
};

type Gles = GbmGlesBackend<GlesRenderer, DrmDeviceFd>;
type Allocator = GbmAllocator<DrmDeviceFd>;
type Exporter = GbmFramebufferExporter<DrmDeviceFd>;
type TtyRenderer<'a> = MultiRenderer<'a, 'a, Gles, Gles>;
/// The tty renderer draws meshes on the GLES renderer of its render GPU.
impl MeshRenderer for TtyRenderer<'_> {
    fn gles(&mut self) -> &mut GlesRenderer {
        self.as_mut()
    }

    fn gles_frame<'a, 'f, 'b>(
        frame: &'a mut <Self as RendererSuper>::Frame<'f, 'b>,
    ) -> &'a mut GlesFrame<'f, 'b>
    where
        'b: 'f,
        Self: 'f,
    {
        frame.as_mut()
    }

    fn from_gles(err: GlesError) -> Self::Error {
        MultiError::Render(err)
    }
}

/// What a queued frame carries to its vblank.
type Feedback = Option<Frame>;
type Manager = DrmOutputManager<Allocator, Exporter, Feedback, DrmDeviceFd>;

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
    /// For the globals of outputs that turn on again.
    display: DisplayHandle,
    /// For the renders and timers of outputs that turn on or off.
    loop_handle: LoopHandle<'static, State>,
}

/// One GPU with its connectors.
struct Device {
    manager: Manager,
    scanner: DrmScanner,
    render_node: DrmNode,
    surfaces: HashMap<crtc::Handle, Surface>,
    /// The outputs that output configuration turned off, with their
    /// connectors. The CRTC of each stays reserved for it.
    disabled: HashMap<crtc::Handle, (Output, connector::Info)>,
    token: RegistrationToken,
}

/// A queued frame: the presentation feedback of its surfaces, and the
/// generation of the lock that it rendered under.
struct Frame {
    feedback: OutputPresentationFeedback,
    lock: Option<u64>,
}

/// One output on a connector.
struct Surface {
    output: Output,
    /// The connector, for its modes.
    connector: connector::Info,
    compositor: DrmOutput<Allocator, Exporter, Feedback, DrmDeviceFd>,
    global: GlobalId,
    /// A frame waits for its vblank.
    queued: bool,
    /// The generation of the lock of the frame on the screen.
    shown_lock: Option<u64>,
    /// The start of the render of the queued frame, for the watchdog.
    rendered_at: Option<std::time::Instant>,
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
        display: state.display_handle.clone(),
        loop_handle: state.loop_handle.clone(),
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
    /// Compiles the shader of an effect on the render GPU, to check it.
    pub fn check_effect(&mut self, path: &str) -> Result<(), String> {
        let mut renderer = self
            .gpus
            .single_renderer(&self.primary_gpu)
            .map_err(|e| format!("{e:?}"))?;
        crate::gl::check_effect(renderer.as_mut(), path)
    }

    /// Runs a function with the renderer of the primary GPU, while the
    /// session is active.
    pub fn with_gles<T>(&mut self, f: impl FnOnce(&mut GlesRenderer) -> T) -> Result<T, String> {
        if !self.session.is_active() {
            return Err("the session is not active".into());
        }
        let mut renderer = self
            .gpus
            .single_renderer(&self.primary_gpu)
            .map_err(|e| format!("{e:?}"))?;
        Ok(f(renderer.as_mut()))
    }

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

    /// Checks that the connector of an output has a mode. An output can
    /// always turn off. Both need an active session.
    pub fn test_output(&self, output: &Output, mode: Option<Mode>) -> Result<(), String> {
        if !self.session.is_active() {
            return Err("the session is paused".into());
        }
        let Some(mode) = mode else {
            return Ok(());
        };
        let connector = self
            .devices
            .values()
            .find_map(|d| d.connector(output))
            .ok_or_else(|| format!("{} has no connector", output.name()))?;
        drm_mode(connector, mode).map(drop)
    }

    /// Gives an output that is on a new mode. The mode set happens with
    /// the next frame.
    pub fn set_mode(&mut self, output: &Output, mode: Mode) -> Result<(), String> {
        let (node, crtc) = self.find_on(output)?;
        let device = self.devices.get_mut(&node).ok_or("the GPU is gone")?;
        let surface = device.surfaces.get_mut(&crtc).ok_or("the output is gone")?;
        let mode = drm_mode(&surface.connector, mode)?;
        let mut renderer = self
            .gpus
            .single_renderer(&device.render_node)
            .map_err(|e| format!("{e:?}"))?;
        surface
            .compositor
            .use_mode::<_, SceneElement<TtyRenderer<'_>>>(
                mode,
                &mut renderer,
                &DrmOutputRenderElements::default(),
            )
            .map_err(|e| format!("{}: {e:?}", output.name()))
    }

    /// Turns an output on again in a mode, with a new global.
    pub fn enable_output(&mut self, output: &Output, mode: Mode) -> Result<(), String> {
        let (node, crtc) = self.find_off(output)?;
        let connector = self
            .devices
            .get(&node)
            .and_then(|d| d.disabled.get(&crtc))
            .map(|(_, c)| c.clone())
            .ok_or("the output is gone")?;
        let drm = drm_mode(&connector, mode)?;
        // The new compositor takes the mode of the output.
        output.change_current_state(Some(mode), None, None, None);
        let surface = self.start((node, crtc), output, &connector, drm)?;
        let device = self.devices.get_mut(&node).ok_or("the GPU is gone")?;
        device.disabled.remove(&crtc);
        device.surfaces.insert(crtc, surface);
        self.loop_handle
            .insert_idle(move |state| render(state, node, crtc));
        Ok(())
    }

    /// Turns an output off: its CRTC stops and its global goes away. The
    /// connector stays for [`Self::enable_output`].
    pub fn disable_output(&mut self, output: &Output) -> Result<(), String> {
        let (node, crtc) = self.find_on(output)?;
        let device = self.devices.get_mut(&node).ok_or("the GPU is gone")?;
        let surface = device.surfaces.remove(&crtc).ok_or("the output is gone")?;
        if let Some(timer) = surface.timer {
            self.loop_handle.remove(timer);
        }
        retire_global(&self.display, &self.loop_handle, surface.global);
        device
            .disabled
            .insert(crtc, (surface.output, surface.connector));
        // The drop of the DrmOutput turns the CRTC off.
        drop(surface.compositor);
        Ok(())
    }

    /// The GPU and the CRTC of an output that is on.
    fn find_on(&self, output: &Output) -> Result<(DrmNode, crtc::Handle), String> {
        self.devices
            .iter()
            .find_map(|(node, device)| {
                let mut surfaces = device.surfaces.iter();
                let (crtc, _) = surfaces.find(|(_, s)| s.output == *output)?;
                Some((*node, *crtc))
            })
            .ok_or_else(|| format!("{} is not on", output.name()))
    }

    /// The GPU and the CRTC of an output that is off.
    fn find_off(&self, output: &Output) -> Result<(DrmNode, crtc::Handle), String> {
        self.devices
            .iter()
            .find_map(|(node, device)| {
                let mut disabled = device.disabled.iter();
                let (crtc, _) = disabled.find(|(_, (o, _))| o == output)?;
                Some((*node, *crtc))
            })
            .ok_or_else(|| format!("{} is not off", output.name()))
    }

    /// Drives a connector of a GPU with a CRTC in a mode, and creates the
    /// global of its output.
    fn start(
        &mut self,
        (node, crtc): (DrmNode, crtc::Handle),
        output: &Output,
        connector: &connector::Info,
        mode: control::Mode,
    ) -> Result<Surface, String> {
        let device = self.devices.get_mut(&node).ok_or("the GPU is gone")?;
        let planes = device.manager.device().planes(&crtc).ok();
        let mut renderer = self
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
                output,
                planes,
                &mut renderer,
                &DrmOutputRenderElements::default(),
            )
            .map_err(|e| format!("{}: {e:?}", output.name()))?;
        Ok(Surface {
            output: output.clone(),
            connector: connector.clone(),
            compositor,
            global: output.create_global::<State>(&self.display),
            queued: false,
            shown_lock: None,
            rendered_at: None,
            timer: None,
        })
    }
}

impl Device {
    /// The connector of an output of this GPU, on or off.
    fn connector(&self, output: &Output) -> Option<&connector::Info> {
        let on = self.surfaces.values().find(|s| s.output == *output);
        on.map(|s| &s.connector).or_else(|| {
            self.disabled
                .values()
                .find(|(o, _)| o == output)
                .map(|(_, c)| c)
        })
    }
}

/// The mode of a connector that matches a mode of its output.
fn drm_mode(connector: &connector::Info, mode: Mode) -> Result<control::Mode, String> {
    connector
        .modes()
        .iter()
        .copied()
        .find(|m| Mode::from(*m) == mode)
        .ok_or_else(|| {
            format!(
                "the connector has no mode {}x{} at {} mHz",
                mode.size.w, mode.size.h, mode.refresh
            )
        })
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
        .insert_source(notifier, move |event, event_metadata, state| match event {
            DrmEvent::VBlank(crtc) => vblank(state, node, crtc, event_metadata.as_ref()),
            DrmEvent::Error(error) => tracing::warn!(?error, "a DRM device failed"),
        })
        .map_err(|e| err(&e.error))?;
    let device = Device {
        manager,
        scanner: DrmScanner::new(),
        render_node,
        surfaces: HashMap::new(),
        disabled: HashMap::new(),
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
        .map(|d| {
            d.surfaces
                .keys()
                .chain(d.disabled.keys())
                .copied()
                .collect()
        })
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

/// A name for a new output: the name of its connector, with a number
/// when another GPU has a connector of that name.
fn unique_name(state: &State, base: &str) -> String {
    let tty = match &state.backend {
        Some(Backend::Tty(tty)) => Some(tty),
        _ => None,
    };
    let disabled = tty
        .into_iter()
        .flat_map(|t| t.devices.values())
        .flat_map(|d| d.disabled.values().map(|(o, _)| o.name()));
    let taken: Vec<String> = state
        .outputs
        .iter()
        .map(Output::name)
        .chain(disabled)
        .collect();
    let mut name = base.to_string();
    for n in 2.. {
        if !taken.contains(&name) {
            break;
        }
        name = format!("{base}-{n}");
    }
    name
}

fn create_output(
    state: &mut State,
    node: DrmNode,
    connector: &connector::Info,
    crtc: crtc::Handle,
) -> Result<Output, String> {
    let name = unique_name(
        state,
        &format!(
            "{}-{}",
            connector.interface().as_str(),
            connector.interface_id()
        ),
    );
    let modes = connector.modes();
    let mode = *modes
        .iter()
        .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
        .or(modes.first())
        .ok_or_else(|| format!("{name} has no mode"))?;
    let tty = tty(state).ok_or("no tty backend")?;
    let device = tty.devices.get(&node).ok_or("the GPU is gone")?;
    let output = Output::new(name, properties(device, connector));
    for mode in modes {
        output.add_mode(Mode::from(*mode));
    }
    output.set_preferred(Mode::from(mode));
    output.change_current_state(Some(Mode::from(mode)), None, None, None);
    let surface = tty.start((node, crtc), &output, connector, mode)?;
    if let Some(device) = tty.devices.get_mut(&node) {
        device.surfaces.insert(crtc, surface);
    }
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

/// Removes the output of a connector that went away, on or off.
fn disconnect(state: &mut State, node: DrmNode, crtc: crtc::Handle) {
    let Some(device) = tty(state).and_then(|t| t.devices.get_mut(&node)) else {
        return;
    };
    if let Some((output, _)) = device.disabled.remove(&crtc) {
        state.remove_output(&output);
        return;
    }
    let Some(surface) = device.surfaces.remove(&crtc) else {
        return;
    };
    if let Some(timer) = surface.timer {
        state.loop_handle.remove(timer);
    }
    retire_global(&state.display_handle, &state.loop_handle, surface.global);
    state.remove_output(&surface.output);
}

/// How long a disabled `wl_output` global stays before it goes: a client
/// that binds a removed global dies, and its bind can be on the way.
const GLOBAL_GRACE: Duration = Duration::from_secs(10);

/// Hides the global of an output from new clients, and removes it later.
pub(super) fn retire_global(
    display: &DisplayHandle,
    loop_handle: &LoopHandle<'static, State>,
    global: GlobalId,
) {
    display.disable_global::<State>(global.clone());
    let timer = Timer::from_duration(GLOBAL_GRACE);
    let removed = loop_handle.insert_source(timer, move |_, (), state| {
        state.display_handle.remove_global::<State>(global.clone());
        TimeoutAction::Drop
    });
    if let Err(err) = removed {
        tracing::warn!(?err, "an output global stays until chora exits");
    }
}

/// Renders an output, unless a frame of it still waits for its vblank. A
/// paused session renders nothing; its activation renders all outputs.
fn render(state: &mut State, node: DrmNode, crtc: crtc::Handle) {
    if tty(state).is_some_and(|t| !t.session.is_active()) {
        return;
    }
    let Some(output) = surface(state, node, crtc)
        .filter(|s| !s.queued)
        .map(|s| s.output.clone())
    else {
        return;
    };
    let view = View::of(state, &output);
    let pointer = pointer_on(state, &output);
    let start = std::time::Instant::now();
    let queued = draw(state, node, crtc, &view, pointer)
        .and_then(|damaged| Ok(damaged && queue(state, node, crtc, &output)?))
        .unwrap_or_else(|err| {
            log_error::<(), _>("rendering failed", Err(err));
            false
        });
    if queued && let Some(surface) = surface(state, node, crtc) {
        surface.rendered_at = Some(start);
    }
    if !queued {
        // Without damage, the screen keeps the frame of an earlier lock,
        // and that hides the session as well.
        if surface(state, node, crtc).is_some_and(|s| s.shown_lock.is_some()) {
            let generation = state.lock_generation();
            state.frame_shown(&output, generation);
        }
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

/// Renders the cursor and the scene. Returns whether the frame has damage.
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
        assets,
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
    // Over the lock, only the lock client sets the cursor.
    let fallback = CursorImageStatus::default_named();
    let cursor_status = match (&*lock, &*cursor_status) {
        (Some(lock), CursorImageStatus::Surface(s)) if !lock.owns(s) => &fallback,
        (_, status) => status,
    };
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
        assets,
        lock: lock.as_ref(),
        depth: 0,
    };
    elements.extend(render::output_elements(
        &mut parts,
        &mut renderer,
        &output,
        view,
    ));
    present(&mut device.manager, surface, &mut renderer, &elements)
}

/// Renders the elements. Returns whether the frame has damage.
fn present<'a>(
    manager: &mut Manager,
    surface: &mut Surface,
    renderer: &mut TtyRenderer<'a>,
    elements: &[SceneElement<TtyRenderer<'a>>],
) -> Result<bool, String> {
    let frame = surface
        .compositor
        .render_frame(renderer, elements, BLACK, FrameFlags::DEFAULT);
    let frame = match frame {
        Ok(frame) => frame,
        Err(err) => {
            // Another session can leave the CRTCs bound otherwise, for
            // example after a VT switch; a fresh state recovers.
            if test_failed(&err) {
                log_error("resetting a GPU failed", manager.device_mut().reset_state());
            }
            return Err(format!("{err:?}"));
        }
    };
    Ok(!frame.is_empty)
}

/// Queues a rendered frame with the presentation feedback of the surfaces
/// that it shows. Returns whether the frame was queued.
fn queue(
    state: &mut State,
    node: DrmNode,
    crtc: crtc::Handle,
    output: &Output,
) -> Result<bool, String> {
    let frame = Frame {
        feedback: render::take_feedback(state, output),
        lock: state.lock_generation(),
    };
    let Some(surface) = surface(state, node, crtc) else {
        return Ok(false);
    };
    surface
        .compositor
        .queue_frame(Some(frame))
        .map_err(|e| format!("{e:?}"))?;
    surface.queued = true;
    Ok(true)
}

/// Whether an error comes from a failed atomic test of the DRM device.
fn test_failed(err: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(err);
    while let Some(e) = current {
        if matches!(e.downcast_ref::<DrmError>(), Some(DrmError::TestFailed(_))) {
            return true;
        }
        current = e.source();
    }
    false
}

/// The frame reached the screen: its surfaces learn when, and the output
/// can render again.
fn vblank(state: &mut State, node: DrmNode, crtc: crtc::Handle, meta: Option<&DrmEventMetadata>) {
    let now = state.clock.now();
    let Some(entry) = surface(state, node, crtc) else {
        return;
    };
    entry.queued = false;
    let output = entry.output.clone();
    // The GPU work and the wait for the vblank count; one refresh of wait
    // is normal.
    if let Some(start) = entry.rendered_at.take() {
        let budget = frame_duration(&output) * 2;
        state.frame_rendered(&output.name(), budget, start.elapsed());
    }
    let Some(surface) = surface(state, node, crtc) else {
        return;
    };
    match surface.compositor.frame_submitted() {
        Ok(Some(Some(mut frame))) => {
            let (time, flags) = presentation_time(meta, now);
            let seq = meta.map_or(0, |m| u64::from(m.sequence));
            let refresh = Refresh::fixed(frame_duration(&output));
            frame.feedback.presented(time, refresh, seq, flags);
            surface.shown_lock = frame.lock;
            state.frame_shown(&output, frame.lock);
        }
        Ok(_) => {}
        Err(err) => tracing::warn!(?err, "a frame failed"),
    }
    render(state, node, crtc);
}

/// The time of a vblank: the time stamp of the kernel if it has one, else
/// now.
fn presentation_time(
    meta: Option<&DrmEventMetadata>,
    now: Time<Monotonic>,
) -> (Time<Monotonic>, Kind) {
    match meta.map(|m| m.time) {
        Some(DrmEventTime::Monotonic(time)) if !time.is_zero() => (
            time.into(),
            Kind::Vsync | Kind::HwClock | Kind::HwCompletion,
        ),
        _ => (now, Kind::Vsync),
    }
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
                // Another session can have lit the CRTCs of the outputs
                // that are off; a fresh state turns them off, and the
                // outputs that are on set their modes again.
                if !device.disabled.is_empty() {
                    log_error(
                        "resetting a GPU failed",
                        device.manager.device_mut().reset_state(),
                    );
                }
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
