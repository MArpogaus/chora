//! A Wayland client in the process of the test: toplevels, layer surfaces,
//! and the configures that they get. The client and the core run in one
//! thread, so a roundtrip turns the event loop of the core.

use std::{
    os::{fd::AsFd, unix::net::UnixStream},
    sync::Arc,
};

use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_data_device::{self, WlDataDevice},
        wl_data_device_manager::{DndAction, WlDataDeviceManager},
        wl_data_offer::WlDataOffer,
        wl_data_source::{self, WlDataSource},
        wl_keyboard::{self, WlKeyboard},
        wl_output::{self, WlOutput},
        wl_pointer::{self, WlPointer},
        wl_region::WlRegion,
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat},
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};
use wayland_protocols::ext::image_capture_source::v1::client::{
    ext_image_capture_source_v1::ExtImageCaptureSourceV1,
    ext_output_image_capture_source_manager_v1::ExtOutputImageCaptureSourceManagerV1,
};
use wayland_protocols::ext::image_copy_capture::v1::client::{
    ext_image_copy_capture_manager_v1::{ExtImageCopyCaptureManagerV1, Options},
    ext_image_copy_capture_session_v1::{self, ExtImageCopyCaptureSessionV1},
};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1::ExtSessionLockManagerV1,
    ext_session_lock_surface_v1::{self, ExtSessionLockSurfaceV1},
    ext_session_lock_v1::{self, ExtSessionLockV1},
};
use wayland_protocols::wp::presentation_time::client::{
    wp_presentation::WpPresentation,
    wp_presentation_feedback::{self, WpPresentationFeedback},
};
use wayland_protocols::wp::text_input::zv3::client::{
    zwp_text_input_manager_v3::ZwpTextInputManagerV3,
    zwp_text_input_v3::{self, ZwpTextInputV3},
};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;
use wayland_protocols::xdg::shell::client::{
    xdg_popup::{self, XdgPopup},
    xdg_positioner::{self, XdgPositioner},
    xdg_surface::{self, XdgSurface},
    xdg_toplevel::{self, XdgToplevel},
    xdg_wm_base::{self, XdgWmBase},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};

use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

use super::Core;
use crate::state::ClientState;

/// A Wayland client of the core.
pub struct Wayland {
    conn: Connection,
    queue: EventQueue<App>,
    app: App,
}

/// The globals and the configures that the client got.
#[derive(Default)]
struct App {
    compositor: Option<WlCompositor>,
    wm_base: Option<XdgWmBase>,
    activation: Option<XdgActivationV1>,
    layer_shell: Option<ZwlrLayerShellV1>,
    lock_manager: Option<ExtSessionLockManagerV1>,
    idle_notifier: Option<ExtIdleNotifierV1>,
    /// The events of the idle notifications in order: true for idled,
    /// false for resumed.
    idle_events: Vec<bool>,
    presentation: Option<WpPresentation>,
    /// How many presentation feedbacks the core sent as presented.
    presented: usize,
    /// Whether the core confirmed a lock of the session.
    locked: bool,
    text_manager: Option<ZwpTextInputManagerV3>,
    screencopy: Option<ZwlrScreencopyManagerV1>,
    capture_sources: Option<ExtOutputImageCaptureSourceManagerV1>,
    capture: Option<ExtImageCopyCaptureManagerV1>,
    /// Whether the newest copy of the output got a buffer offer; `None`
    /// until the core answers.
    copy_offered: Option<bool>,
    output: Option<WlOutput>,
    seat: Option<WlSeat>,
    shm: Option<WlShm>,
    data_manager: Option<WlDataDeviceManager>,
    data_device: Option<WlDataDevice>,
    /// The drag and drop events that no test took yet.
    dnd_events: Vec<DndEvent>,
    /// The serials of the key events, press and release, in order.
    key_serials: Vec<u32>,
    /// The newest `repeat_info`: rate and delay.
    repeat_info: Option<(i32, i32)>,
    /// The pointer events that the core sent and that no test took yet.
    pointer_events: Vec<PointerEvent>,
    /// The commit strings that the core sent to the text input.
    commits: Vec<String>,
    /// The interfaces of all globals that the core offered.
    interfaces: Vec<String>,
    synced: bool,
    /// The configures of each surface, by its index.
    surfaces: Vec<Configures>,
}

/// An event of the pointer of the client, in the units of the client.
#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    /// The pointer entered a surface at a point of it.
    Enter {
        surface: wayland_client::backend::ObjectId,
        at: (f64, f64),
    },
    /// The pointer moved on the surface that it entered.
    Motion { at: (f64, f64) },
    /// A button changed, with the serial of the event.
    Button { serial: u32, pressed: bool },
    /// The pointer left its surface.
    Leave,
}

/// An event of a drag and drop of the client, in the units of the client.
#[derive(Debug, Clone, PartialEq)]
pub enum DndEvent {
    /// The drag entered a surface at a point of it.
    Enter {
        surface: wayland_client::backend::ObjectId,
        at: (f64, f64),
    },
    /// The drag moved on the surface that it entered.
    Motion { at: (f64, f64) },
    /// The drag left its surface.
    Leave,
    /// The drag dropped on the surface that it entered.
    Drop,
    /// The core cancelled the drag of the source.
    Cancelled,
    /// The core told the source that the drop happened.
    Performed,
}

/// The configures of a surface.
#[derive(Default)]
pub struct Configures {
    /// The serial of the newest configure that the client did not ack.
    pending: Option<u32>,
    /// The size of the newest toplevel or lock configure.
    pub size: (i32, i32),
    /// The states of the newest toplevel configure.
    pub states: Vec<xdg_toplevel::State>,
    /// How many configures arrived.
    pub count: usize,
    /// Whether the core closed the popup.
    pub done: bool,
    /// The newest popup configure: x, y, width and height relative to the
    /// parent geometry.
    pub popup: Option<[i32; 4]>,
    /// How many frame callbacks the core sent.
    pub frames: usize,
}

/// A toplevel of the client.
pub struct Toplevel {
    pub surface: WlSurface,
    xdg: XdgSurface,
    role: XdgToplevel,
    index: usize,
}

impl Toplevel {
    /// Sets the window geometry for the next commit, as a window with a
    /// drop shadow does: `(x, y, w, h)` in its buffer.
    pub fn set_geometry(&self, (x, y, w, h): (i32, i32, i32, i32)) {
        self.xdg.set_window_geometry(x, y, w, h);
    }
}

/// A popup of the client.
pub struct Popup {
    index: usize,
    pub surface: WlSurface,
    xdg: XdgSurface,
}

/// A lock surface of the client.
pub struct LockSurface {
    index: usize,
}

/// A layer surface of the client.
pub struct LayerSurface {
    pub surface: WlSurface,
    pub layer: ZwlrLayerSurfaceV1,
}

impl Wayland {
    /// Connects to the core and binds the globals.
    pub fn connect(core: &mut Core) -> Self {
        let (ours, theirs) = UnixStream::pair().expect("a socket pair");
        core.state
            .display_handle
            .insert_client(theirs, Arc::new(ClientState::default()))
            .expect("the core takes the client");
        Self::from_socket(core, ours)
    }

    /// Connects through a socket file of the core and binds the globals.
    pub fn connect_to(core: &mut Core, path: &std::path::Path) -> Self {
        let ours = UnixStream::connect(path).expect("the socket listens");
        core.dispatch();
        Self::from_socket(core, ours)
    }

    /// Whether the core closed the connection, within a few turns. A sync
    /// makes the core read the connection, as any request of a live client
    /// would; the core closes a killed client when it reads. The flush
    /// comes before the dispatch, so that the core really reads the sync
    /// and not an older request, which the last round would otherwise
    /// leave unprocessed under load.
    pub fn closed(&mut self, core: &mut Core) -> bool {
        for _ in 0..5 {
            self.conn.display().sync(&self.queue.handle(), ());
            if self.conn.flush().is_err() {
                return true;
            }
            core.dispatch();
            if self.conn.flush().is_err() {
                return true;
            }
            if let Some(guard) = self.queue.prepare_read() {
                match guard.read() {
                    Err(wayland_client::backend::WaylandError::Io(e))
                        if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    // EOF means the core closed the connection, which
                    // is what a killed client leaves behind.
                    Err(_) | Ok(0) => return true,
                    Ok(_) => {}
                }
            }
        }
        false
    }

    /// The interfaces of the globals that the core offered.
    pub fn interfaces(&self) -> &[String] {
        &self.app.interfaces
    }

    fn from_socket(core: &mut Core, ours: UnixStream) -> Self {
        let conn = Connection::from_socket(ours).expect("a connection");
        let queue = conn.new_event_queue();
        conn.display().get_registry(&queue.handle(), ());
        let mut wayland = Self {
            conn,
            queue,
            app: App::default(),
        };
        wayland.roundtrip(core);
        wayland.roundtrip(core);
        wayland
    }

    /// Sends the requests, turns the core, and handles the events, until
    /// the core answered all requests.
    pub fn roundtrip(&mut self, core: &mut Core) {
        self.app.synced = false;
        self.conn.display().sync(&self.queue.handle(), ());
        for _ in 0..100 {
            self.conn.flush().expect("the client sends");
            core.dispatch();
            if let Some(guard) = self.queue.prepare_read() {
                // Nothing to read is no error.
                let _ = guard.read();
            }
            self.queue
                .dispatch_pending(&mut self.app)
                .expect("the client handles the events");
            if self.app.synced {
                return;
            }
        }
        panic!("the core did not answer the client");
    }

    /// A toplevel after its first commit, which creates its surface node.
    pub fn toplevel(&mut self, core: &mut Core, app_id: &str) -> Toplevel {
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let wm_base = self.app.wm_base.as_ref().expect("xdg_wm_base");
        let xdg = wm_base.get_xdg_surface(&surface, &qh, index);
        let role = xdg.get_toplevel(&qh, index);
        role.set_app_id(app_id.into());
        surface.commit();
        self.roundtrip(core);
        Toplevel {
            surface,
            xdg,
            role,
            index,
        }
    }

    /// A toplevel that activates with the token before or with its first
    /// commit, as a program of `choractl run` does; its `surface.created` then carries
    /// the token. The activation comes before the toplevel role, or right
    /// after the first commit in the same batch, as in foot.
    pub fn activated_toplevel(
        &mut self,
        core: &mut Core,
        app_id: &str,
        token: &str,
        after_commit: bool,
    ) -> Toplevel {
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let activation = self.app.activation.as_ref().expect("xdg_activation");
        if !after_commit {
            activation.activate(token.into(), &surface);
        }
        let wm_base = self.app.wm_base.as_ref().expect("xdg_wm_base");
        let xdg = wm_base.get_xdg_surface(&surface, &qh, index);
        let role = xdg.get_toplevel(&qh, index);
        role.set_app_id(app_id.into());
        surface.commit();
        if after_commit {
            activation.activate(token.into(), &surface);
        }
        self.roundtrip(core);
        Toplevel {
            surface,
            xdg,
            role,
            index,
        }
    }

    /// Activates a layer surface with a token, as a field does after its
    /// first configure.
    pub fn activate_layer(&mut self, core: &mut Core, layer: &LayerSurface, token: &str) {
        let activation = self.app.activation.as_ref().expect("xdg_activation");
        activation.activate(token.into(), &layer.surface);
        self.roundtrip(core);
    }

    /// A child toplevel of `parent` after its first commit, for dialogs.
    pub fn child(&mut self, core: &mut Core, parent: &Toplevel) -> Toplevel {
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let wm_base = self.app.wm_base.as_ref().expect("xdg_wm_base");
        let xdg = wm_base.get_xdg_surface(&surface, &qh, index);
        let role = xdg.get_toplevel(&qh, index);
        role.set_parent(Some(&parent.role));
        surface.commit();
        self.roundtrip(core);
        Toplevel {
            surface,
            xdg,
            role,
            index,
        }
    }

    /// A popup of a toplevel that grabs the input with a serial, after its
    /// first commit.
    pub fn popup(&mut self, core: &mut Core, parent: &Toplevel, serial: u32) -> Popup {
        self.open_popup(core, parent, serial, ((50, 50), None))
    }

    /// A menu: a popup of `size` whose corner is at `at` in the geometry of
    /// the parent, which slides into it, with a buffer after its first
    /// commit.
    pub fn menu(
        &mut self,
        core: &mut Core,
        parent: &Toplevel,
        serial: u32,
        (at, size): ((i32, i32), (i32, i32)),
    ) -> Popup {
        let popup = self.open_popup(core, parent, serial, (size, Some(at)));
        if let Some(serial) = self.app.surfaces[popup.index].pending.take() {
            popup.xdg.ack_configure(serial);
        }
        self.show(
            core,
            &popup.surface,
            size,
            (wl_shm::Format::Argb8888, 0xff00_ff00),
        );
        popup
    }

    fn open_popup(
        &mut self,
        core: &mut Core,
        parent: &Toplevel,
        serial: u32,
        (size, at): ((i32, i32), Option<(i32, i32)>),
    ) -> Popup {
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let wm_base = self.app.wm_base.as_ref().expect("xdg_wm_base");
        let positioner = wm_base.create_positioner(&qh, ());
        positioner.set_size(size.0, size.1);
        match at {
            Some((x, y)) => {
                positioner.set_anchor_rect(x, y, 1, 1);
                positioner.set_anchor(xdg_positioner::Anchor::TopLeft);
                positioner.set_gravity(xdg_positioner::Gravity::BottomRight);
                positioner.set_constraint_adjustment(
                    xdg_positioner::ConstraintAdjustment::SlideX
                        | xdg_positioner::ConstraintAdjustment::SlideY,
                );
            }
            None => positioner.set_anchor_rect(0, 0, 1, 1),
        }
        let xdg = wm_base.get_xdg_surface(&surface, &qh, index);
        let popup = xdg.get_popup(Some(&parent.xdg), &positioner, &qh, index);
        popup.grab(self.app.seat.as_ref().expect("wl_seat"), serial);
        surface.commit();
        self.roundtrip(core);
        Popup {
            index,
            surface,
            xdg,
        }
    }

    /// Binds the data device of the client, so that drags can enter its
    /// surfaces.
    pub fn listen_for_drags(&mut self, core: &mut Core) {
        let qh = self.queue.handle();
        let manager = self
            .app
            .data_manager
            .as_ref()
            .expect("wl_data_device_manager");
        let seat = self.app.seat.as_ref().expect("wl_seat");
        self.app
            .data_device
            .get_or_insert_with(|| manager.get_data_device(seat, &qh, ()));
        self.roundtrip(core);
    }

    /// Starts a drag of the client from a surface with the serial of a
    /// press, which offers one type of text.
    pub fn start_drag(&mut self, core: &mut Core, origin: &Toplevel, serial: u32) {
        self.listen_for_drags(core);
        let qh = self.queue.handle();
        let manager = self
            .app
            .data_manager
            .as_ref()
            .expect("wl_data_device_manager");
        let device = self.app.data_device.as_ref().expect("a data device");
        let source = manager.create_data_source(&qh, ());
        source.offer("text/plain".into());
        source.set_actions(DndAction::Copy);
        device.start_drag(Some(&source), &origin.surface, None, serial);
        self.roundtrip(core);
    }

    /// Takes the drag and drop events that the client got, in order.
    pub fn take_dnd_events(&mut self) -> Vec<DndEvent> {
        std::mem::take(&mut self.app.dnd_events)
    }

    /// Whether the core closed a popup.
    pub fn popup_done(&self, popup: &Popup) -> bool {
        self.app.surfaces[popup.index].done
    }

    /// The geometry of a popup that the core configured: x, y, width and
    /// height relative to the parent geometry.
    pub fn popup_geometry(&self, popup: &Popup) -> Option<[i32; 4]> {
        self.app.surfaces[popup.index].popup
    }

    /// Takes the pointer events that the client got, in order.
    pub fn take_pointer_events(&mut self) -> Vec<PointerEvent> {
        std::mem::take(&mut self.app.pointer_events)
    }

    /// The newest key repeat that the core sent: rate and delay.
    pub fn repeat_info(&self) -> Option<(i32, i32)> {
        self.app.repeat_info
    }

    /// The serials of the key events that the client got.
    pub fn key_serials(&self) -> &[u32] {
        &self.app.key_serials
    }

    /// Enables a text input of the seat, as a text field does; the core
    /// then commits its injected texts to the focused surface.
    pub fn enable_text_input(&mut self, core: &mut Core) {
        let qh = self.queue.handle();
        let manager = self
            .app
            .text_manager
            .as_ref()
            .expect("zwp_text_input_manager_v3");
        let seat = self.app.seat.as_ref().expect("wl_seat");
        let text_input: ZwpTextInputV3 = manager.get_text_input(seat, &qh, ());
        text_input.enable();
        text_input.commit();
        self.roundtrip(core);
    }

    /// The commit strings that the core sent to the text input, in order.
    pub fn commits(&self) -> &[String] {
        &self.app.commits
    }

    /// A layer surface on the output, after its first configure.
    pub fn layer(
        &mut self,
        core: &mut Core,
        layer: Layer,
        keyboard: KeyboardInteractivity,
    ) -> LayerSurface {
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let shell = self.app.layer_shell.as_ref().expect("zwlr_layer_shell_v1");
        let output = self.app.output.as_ref();
        let layer = shell.get_layer_surface(&surface, output, layer, "test".into(), &qh, index);
        layer.set_size(100, 30);
        layer.set_keyboard_interactivity(keyboard);
        surface.commit();
        self.roundtrip(core);
        // The commit after the ack of the first configure.
        surface.commit();
        self.roundtrip(core);
        LayerSurface { surface, layer }
    }

    /// The configures of a toplevel.
    pub fn configures(&self, toplevel: &Toplevel) -> &Configures {
        &self.app.surfaces[toplevel.index]
    }

    /// Acks the newest configure of a toplevel and commits.
    pub fn answer(&mut self, core: &mut Core, toplevel: &Toplevel) {
        let serial = self.app.surfaces[toplevel.index]
            .pending
            .take()
            .expect("a configure to answer");
        toplevel.xdg.ack_configure(serial);
        toplevel.surface.commit();
        self.roundtrip(core);
    }

    /// Acks the newest configure of a toplevel, if any, and commits a
    /// transparent buffer of a size.
    pub fn attach(&mut self, core: &mut Core, toplevel: &Toplevel, size: (i32, i32)) {
        self.attach_filled(core, toplevel, size, (wl_shm::Format::Argb8888, 0));
    }

    /// As [`Self::attach`], with a buffer of a 32 bit format whose pixels
    /// all hold one value.
    pub fn attach_filled(
        &mut self,
        core: &mut Core,
        toplevel: &Toplevel,
        size: (i32, i32),
        (format, pixel): (wl_shm::Format, u32),
    ) {
        if let Some(serial) = self.app.surfaces[toplevel.index].pending.take() {
            toplevel.xdg.ack_configure(serial);
        }
        self.show(core, &toplevel.surface, size, (format, pixel));
    }

    /// A buffer of a size whose pixels all hold one ARGB value, attached
    /// to nothing yet.
    pub fn buffer(&mut self, size: (i32, i32), pixel: u32) -> WlBuffer {
        self.buffer_file(size, (wl_shm::Format::Argb8888, pixel)).0
    }

    /// A buffer of a size in a 32 bit format whose pixels all hold one
    /// value, and the file of its memory, which [`fill`] writes anew.
    pub fn buffer_file(
        &mut self,
        (w, h): (i32, i32),
        (format, pixel): (wl_shm::Format, u32),
    ) -> (WlBuffer, std::fs::File) {
        let file = tempfile();
        fill(&file, (w, h), pixel);
        let qh = self.queue.handle();
        let shm = self.app.shm.as_ref().expect("wl_shm");
        let pool = shm.create_pool(file.as_fd(), w * h * 4, &qh, ());
        let buffer = pool.create_buffer(0, w, h, w * 4, format, &qh, ());
        pool.destroy();
        (buffer, file)
    }

    /// Acks the newest configure of a toplevel, if any, and commits a
    /// buffer with the damage of a rectangle of it, or with none.
    pub fn attach_buffer(
        &mut self,
        core: &mut Core,
        toplevel: &Toplevel,
        buffer: &WlBuffer,
        damage: Option<(i32, i32, i32, i32)>,
    ) {
        if let Some(serial) = self.app.surfaces[toplevel.index].pending.take() {
            toplevel.xdg.ack_configure(serial);
        }
        toplevel.surface.attach(Some(buffer), 0, 0);
        if let Some((x, y, w, h)) = damage {
            toplevel.surface.damage_buffer(x, y, w, h);
        }
        toplevel.surface.commit();
        self.roundtrip(core);
    }

    /// Commits a transparent buffer of a size to a layer surface.
    pub fn attach_layer(&mut self, core: &mut Core, layer: &LayerSurface, size: (i32, i32)) {
        self.show(core, &layer.surface, size, (wl_shm::Format::Argb8888, 0));
    }

    fn show(
        &mut self,
        core: &mut Core,
        surface: &WlSurface,
        (w, h): (i32, i32),
        (format, pixel): (wl_shm::Format, u32),
    ) {
        let mut file = tempfile();
        let pixels = usize::try_from(w * h).expect("a size");
        std::io::Write::write_all(&mut file, &pixel.to_le_bytes().repeat(pixels))
            .expect("the file takes the pixels");
        let qh = self.queue.handle();
        let shm = self.app.shm.as_ref().expect("wl_shm");
        let pool = shm.create_pool(file.as_fd(), w * h * 4, &qh, ());
        let buffer = pool.create_buffer(0, w, h, w * 4, format, &qh, ());
        pool.destroy();
        surface.attach(Some(&buffer), 0, 0);
        surface.damage(0, 0, w, h);
        surface.commit();
        self.roundtrip(core);
    }

    /// Marks a rectangle of a toplevel opaque from its next commit, as a
    /// window that is not transparent does.
    pub fn opaque(&self, toplevel: &Toplevel, (w, h): (i32, i32)) {
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let region = compositor.create_region(&self.queue.handle(), ());
        region.add(0, 0, w, h);
        toplevel.surface.set_opaque_region(Some(&region));
        region.destroy();
    }

    /// Asks for a frame callback of a toplevel and commits.
    pub fn frame(&mut self, core: &mut Core, toplevel: &Toplevel) {
        toplevel.surface.frame(&self.queue.handle(), toplevel.index);
        self.commit(core, toplevel);
    }

    /// Asks for presentation feedback of a toplevel and commits.
    pub fn feedback(&mut self, core: &mut Core, toplevel: &Toplevel) {
        let presentation = self.app.presentation.as_ref().expect("wp_presentation");
        presentation.feedback(&toplevel.surface, &self.queue.handle(), ());
        self.commit(core, toplevel);
    }

    /// How many presentation feedbacks the core sent as presented.
    pub fn presented(&self) -> usize {
        self.app.presented
    }

    /// Whether the core confirmed a lock of the session.
    pub fn locked(&self) -> bool {
        self.app.locked
    }

    /// Asks for an idle notification after `timeout_ms` without input,
    /// as a screen locker does.
    pub fn idle_notification(&mut self, core: &mut Core, timeout_ms: u32) -> ExtIdleNotificationV1 {
        let notifier = self.app.idle_notifier.as_ref().expect("an idle notifier");
        let seat = self.app.seat.as_ref().expect("a seat");
        let notification =
            notifier.get_idle_notification(timeout_ms, seat, &self.queue.handle(), ());
        self.roundtrip(core);
        notification
    }

    /// The idle events that no test took yet: true for idled, false for
    /// resumed.
    pub fn take_idle_events(&mut self) -> Vec<bool> {
        std::mem::take(&mut self.app.idle_events)
    }

    /// Asks for an interactive move of a toplevel, as a title bar does.
    pub fn request_move(&mut self, core: &mut Core, toplevel: &Toplevel, serial: u32) {
        let seat = self.app.seat.as_ref().expect("wl_seat");
        toplevel.role._move(seat, serial);
        self.roundtrip(core);
    }

    /// Asks for an interactive resize of a toplevel at its bottom right
    /// corner.
    pub fn request_resize(&mut self, core: &mut Core, toplevel: &Toplevel, serial: u32) {
        let seat = self.app.seat.as_ref().expect("wl_seat");
        toplevel
            .role
            .resize(seat, serial, xdg_toplevel::ResizeEdge::BottomRight);
        self.roundtrip(core);
    }

    /// Unmaps a toplevel with a commit without a buffer.
    pub fn unmap(&mut self, core: &mut Core, toplevel: &Toplevel) {
        toplevel.surface.attach(None, 0, 0);
        self.commit(core, toplevel);
    }

    /// Commits a toplevel without a change, as its initial commit does.
    pub fn commit(&mut self, core: &mut Core, toplevel: &Toplevel) {
        toplevel.surface.commit();
        self.roundtrip(core);
    }

    /// Destroys a toplevel.
    pub fn destroy(&mut self, core: &mut Core, toplevel: &Toplevel) {
        toplevel.role.destroy();
        toplevel.xdg.destroy();
        toplevel.surface.destroy();
        self.roundtrip(core);
    }

    /// Locks the session, without lock surfaces.
    pub fn lock(&mut self, core: &mut Core) -> ExtSessionLockV1 {
        let manager = self.app.lock_manager.as_ref().expect("a lock manager");
        let lock = manager.lock(&self.queue.handle(), ());
        self.roundtrip(core);
        lock
    }

    /// Whether a screencopy of the output of the client gets a buffer
    /// offer, and not a failure.
    pub fn screencopy_offers(&mut self, core: &mut Core) -> bool {
        let manager = self.app.screencopy.as_ref().expect("a screencopy manager");
        let output = self.app.output.as_ref().expect("wl_output");
        self.app.copy_offered = None;
        manager.capture_output(0, output, &self.queue.handle(), ());
        self.roundtrip(core);
        self.app.copy_offered.expect("an answer to the copy")
    }

    /// Whether a capture session of the output of the client gets a buffer
    /// size, and is not stopped.
    pub fn capture_session_offers(&mut self, core: &mut Core) -> bool {
        let qh = self.queue.handle();
        let sources = self.app.capture_sources.as_ref().expect("a source manager");
        let output = self.app.output.as_ref().expect("wl_output");
        let source = sources.create_source(output, &qh, ());
        let manager = self.app.capture.as_ref().expect("a capture manager");
        self.app.copy_offered = None;
        manager.create_session(&source, Options::empty(), &qh, ());
        self.roundtrip(core);
        self.app.copy_offered.expect("an answer to the session")
    }

    /// Destroys the role of a layer surface and keeps its `wl_surface`.
    pub fn destroy_layer_role(&mut self, core: &mut Core, layer: &LayerSurface) {
        layer.layer.destroy();
        self.roundtrip(core);
    }

    /// Sets the keyboard interactivity of a layer surface and commits.
    pub fn set_layer_keyboard(
        &mut self,
        core: &mut Core,
        layer: &LayerSurface,
        keyboard: KeyboardInteractivity,
    ) {
        layer.layer.set_keyboard_interactivity(keyboard);
        layer.surface.commit();
        self.roundtrip(core);
    }

    /// A lock surface of a lock on the output; it acks each configure.
    pub fn lock_surface(&mut self, core: &mut Core, lock: &ExtSessionLockV1) -> LockSurface {
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let output = self.app.output.as_ref().expect("wl_output");
        lock.get_lock_surface(&surface, output, &qh, index);
        self.roundtrip(core);
        LockSurface { index }
    }

    /// The configures of a lock surface.
    pub fn lock_configures(&self, lock: &LockSurface) -> &Configures {
        &self.app.surfaces[lock.index]
    }

    /// Destroys a layer surface.
    pub fn destroy_layer(&mut self, core: &mut Core, layer: &LayerSurface) {
        layer.layer.destroy();
        layer.surface.destroy();
        self.roundtrip(core);
    }
}

impl Dispatch<WlRegistry, ()> for App {
    fn event(
        app: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        app.interfaces.push(interface.clone());
        match interface.as_str() {
            "wl_compositor" => app.compositor = Some(registry.bind(name, version, qh, ())),
            "xdg_wm_base" => app.wm_base = Some(registry.bind(name, version, qh, ())),
            "xdg_activation_v1" => app.activation = Some(registry.bind(name, version, qh, ())),
            "zwlr_layer_shell_v1" => app.layer_shell = Some(registry.bind(name, version, qh, ())),
            "ext_session_lock_manager_v1" => {
                app.lock_manager = Some(registry.bind(name, version, qh, ()));
            }
            "ext_idle_notifier_v1" => {
                app.idle_notifier = Some(registry.bind(name, 1, qh, ()));
            }
            "wp_presentation" => app.presentation = Some(registry.bind(name, version, qh, ())),
            "zwp_text_input_manager_v3" => {
                app.text_manager = Some(registry.bind(name, version, qh, ()));
            }
            "wl_data_device_manager" => {
                app.data_manager = Some(registry.bind(name, version.min(3), qh, ()));
            }
            "zwlr_screencopy_manager_v1" => {
                app.screencopy = Some(registry.bind(name, 1, qh, ()));
            }
            "ext_output_image_capture_source_manager_v1" => {
                app.capture_sources = Some(registry.bind(name, 1, qh, ()));
            }
            "ext_image_copy_capture_manager_v1" => {
                app.capture = Some(registry.bind(name, 1, qh, ()));
            }
            "wl_shm" => app.shm = Some(registry.bind(name, 1, qh, ())),
            "wl_seat" => app.seat = Some(registry.bind(name, version.min(7), qh, ())),
            "wl_output" if app.output.is_none() => {
                app.output = Some(registry.bind(name, version, qh, ()));
            }
            _ => {}
        }
    }
}

impl Dispatch<WlCallback, ()> for App {
    fn event(
        app: &mut Self,
        _: &WlCallback,
        event: wl_callback::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            app.synced = true;
        }
    }
}

/// A frame callback of the surface with this index.
impl Dispatch<WlCallback, usize> for App {
    fn event(
        app: &mut Self,
        _: &WlCallback,
        event: wl_callback::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            app.surfaces[*index].frames += 1;
        }
    }
}

impl Dispatch<XdgWmBase, ()> for App {
    fn event(
        _: &mut Self,
        wm_base: &XdgWmBase,
        event: xdg_wm_base::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<XdgSurface, usize> for App {
    fn event(
        app: &mut Self,
        _: &XdgSurface,
        event: xdg_surface::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            let configures = &mut app.surfaces[*index];
            configures.pending = Some(serial);
            configures.count += 1;
        }
    }
}

delegate_noop!(App: XdgActivationV1);

impl Dispatch<XdgToplevel, usize> for App {
    fn event(
        app: &mut Self,
        _: &XdgToplevel,
        event: xdg_toplevel::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Configure {
            width,
            height,
            states,
        } = event
        {
            let configures = &mut app.surfaces[*index];
            configures.size = (width, height);
            configures.states = states
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| u32::from_ne_bytes(*c))
                .filter_map(|s| match WEnum::from(s) {
                    WEnum::Value(state) => Some(state),
                    WEnum::Unknown(_) => None,
                })
                .collect();
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, usize> for App {
    fn event(
        app: &mut Self,
        layer: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            // A layer surface answers each configure at once.
            layer.ack_configure(serial);
            app.surfaces[*index].count += 1;
        }
    }
}

impl Dispatch<ExtSessionLockSurfaceV1, usize> for App {
    fn event(
        app: &mut Self,
        surface: &ExtSessionLockSurfaceV1,
        event: ext_session_lock_surface_v1::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_session_lock_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            surface.ack_configure(serial);
            let configures = &mut app.surfaces[*index];
            configures.size = (
                i32::try_from(width).expect("a width"),
                i32::try_from(height).expect("a height"),
            );
            configures.count += 1;
        }
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for App {
    fn event(
        app: &mut Self,
        _: &ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer { .. } => app.copy_offered = Some(true),
            zwlr_screencopy_frame_v1::Event::Failed => app.copy_offered = Some(false),
            _ => {}
        }
    }
}

impl Dispatch<ExtImageCopyCaptureSessionV1, ()> for App {
    fn event(
        app: &mut Self,
        _: &ExtImageCopyCaptureSessionV1,
        event: ext_image_copy_capture_session_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_image_copy_capture_session_v1::Event::BufferSize { .. } => {
                app.copy_offered = Some(true);
            }
            ext_image_copy_capture_session_v1::Event::Stopped => app.copy_offered = Some(false),
            _ => {}
        }
    }
}

delegate_noop!(App: ZwlrScreencopyManagerV1);
delegate_noop!(App: ExtIdleNotifierV1);

impl Dispatch<ExtIdleNotificationV1, ()> for App {
    fn event(
        app: &mut Self,
        _: &ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_idle_notification_v1::Event::Idled => app.idle_events.push(true),
            ext_idle_notification_v1::Event::Resumed => app.idle_events.push(false),
            _ => {}
        }
    }
}
delegate_noop!(App: ExtOutputImageCaptureSourceManagerV1);
delegate_noop!(App: ExtImageCopyCaptureManagerV1);
delegate_noop!(App: ExtImageCaptureSourceV1);

impl Dispatch<WlOutput, ()> for App {
    fn event(
        _: &mut Self,
        _: &WlOutput,
        _: wl_output::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSeat, ()> for App {
    fn event(
        _: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(c),
        } = event
        {
            if c.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
            if c.contains(wl_seat::Capability::Pointer) {
                seat.get_pointer(qh, ());
            }
        }
    }
}

impl Dispatch<WlPointer, ()> for App {
    fn event(
        app: &mut Self,
        _: &WlPointer,
        event: wl_pointer::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface,
                surface_x,
                surface_y,
                ..
            } => app.pointer_events.push(PointerEvent::Enter {
                surface: surface.id(),
                at: (surface_x, surface_y),
            }),
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => app.pointer_events.push(PointerEvent::Motion {
                at: (surface_x, surface_y),
            }),
            wl_pointer::Event::Button { serial, state, .. } => {
                app.pointer_events.push(PointerEvent::Button {
                    serial,
                    pressed: state == WEnum::Value(wl_pointer::ButtonState::Pressed),
                });
            }
            wl_pointer::Event::Leave { .. } => app.pointer_events.push(PointerEvent::Leave),
            _ => {}
        }
    }
}

impl Dispatch<WlKeyboard, ()> for App {
    fn event(
        app: &mut Self,
        _: &WlKeyboard,
        event: wl_keyboard::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key { serial, .. } => app.key_serials.push(serial),
            wl_keyboard::Event::RepeatInfo { rate, delay } => app.repeat_info = Some((rate, delay)),
            _ => {}
        }
    }
}

impl Dispatch<ZwpTextInputV3, ()> for App {
    fn event(
        app: &mut Self,
        _: &ZwpTextInputV3,
        event: zwp_text_input_v3::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwp_text_input_v3::Event::CommitString { text: Some(text) } = event {
            app.commits.push(text);
        }
    }
}

impl Dispatch<XdgPopup, usize> for App {
    fn event(
        app: &mut Self,
        _: &XdgPopup,
        event: xdg_popup::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            xdg_popup::Event::PopupDone => app.surfaces[*index].done = true,
            xdg_popup::Event::Configure {
                x,
                y,
                width,
                height,
            } => app.surfaces[*index].popup = Some([x, y, width, height]),
            _ => {}
        }
    }
}

impl Dispatch<WlDataDeviceManager, ()> for App {
    fn event(
        _: &mut Self,
        _: &WlDataDeviceManager,
        _: <WlDataDeviceManager as Proxy>::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlDataDevice, ()> for App {
    fn event(
        app: &mut Self,
        _: &WlDataDevice,
        event: wl_data_device::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_device::Event::Enter {
                serial,
                surface,
                x,
                y,
                id,
            } => {
                if let Some(offer) = id {
                    offer.accept(serial, Some("text/plain".into()));
                    offer.set_actions(DndAction::Copy, DndAction::Copy);
                }
                app.dnd_events.push(DndEvent::Enter {
                    surface: surface.id(),
                    at: (x, y),
                });
            }
            wl_data_device::Event::Motion { x, y, .. } => {
                app.dnd_events.push(DndEvent::Motion { at: (x, y) });
            }
            wl_data_device::Event::Leave => app.dnd_events.push(DndEvent::Leave),
            wl_data_device::Event::Drop => app.dnd_events.push(DndEvent::Drop),
            _ => {}
        }
    }

    wayland_client::event_created_child!(App, WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (WlDataOffer, ()),
    ]);
}

impl Dispatch<WlDataOffer, ()> for App {
    fn event(
        _: &mut Self,
        _: &WlDataOffer,
        _: <WlDataOffer as Proxy>::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlDataSource, ()> for App {
    fn event(
        app: &mut Self,
        _: &WlDataSource,
        event: wl_data_source::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_source::Event::Cancelled => app.dnd_events.push(DndEvent::Cancelled),
            wl_data_source::Event::DndDropPerformed => app.dnd_events.push(DndEvent::Performed),
            _ => {}
        }
    }
}

delegate_noop!(App: ignore WlCompositor);
delegate_noop!(App: XdgPositioner);
delegate_noop!(App: ignore WlShm);
delegate_noop!(App: WlShmPool);
delegate_noop!(App: ignore WlBuffer);

/// A new file that is gone from the file system, for a shm pool.
/// Writes one 32 bit value into every pixel of the memory of a buffer of
/// a size, behind the back of the compositor.
pub fn fill(file: &std::fs::File, (w, h): (i32, i32), pixel: u32) {
    let pixels = usize::try_from(w * h).expect("a size");
    std::os::unix::fs::FileExt::write_all_at(file, &pixel.to_le_bytes().repeat(pixels), 0)
        .expect("the file takes the pixels");
}

fn tempfile() -> std::fs::File {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("chora-test-{}-{n}", std::process::id()));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("a temporary file");
    std::fs::remove_file(&path).expect("the file goes");
    file
}
delegate_noop!(App: ignore WlSurface);
delegate_noop!(App: WlRegion);
delegate_noop!(App: ZwpTextInputManagerV3);
delegate_noop!(App: ZwlrLayerShellV1);
delegate_noop!(App: ExtSessionLockManagerV1);
delegate_noop!(App: ignore WpPresentation);

impl Dispatch<ExtSessionLockV1, ()> for App {
    fn event(
        app: &mut Self,
        _: &ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_session_lock_v1::Event::Locked = event {
            app.locked = true;
        }
    }
}

impl Dispatch<WpPresentationFeedback, ()> for App {
    fn event(
        app: &mut Self,
        _: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_presentation_feedback::Event::Presented { .. } = event {
            app.presented += 1;
        }
    }
}
