//! A Wayland client in the process of the test: toplevels, layer surfaces,
//! and the configures that they get. The client and the core run in one
//! thread, so a roundtrip turns the event loop of the core.

use std::{
    os::{fd::AsFd, unix::net::UnixStream},
    sync::Arc,
};

use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_keyboard::{self, WlKeyboard},
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat},
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::ext::session_lock::v1::client::{
    ext_session_lock_manager_v1::ExtSessionLockManagerV1,
    ext_session_lock_surface_v1::{self, ExtSessionLockSurfaceV1},
    ext_session_lock_v1::ExtSessionLockV1,
};
use wayland_protocols::xdg::shell::client::{
    xdg_popup::{self, XdgPopup},
    xdg_positioner::XdgPositioner,
    xdg_surface::{self, XdgSurface},
    xdg_toplevel::{self, XdgToplevel},
    xdg_wm_base::{self, XdgWmBase},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, KeyboardInteractivity, ZwlrLayerSurfaceV1},
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
    layer_shell: Option<ZwlrLayerShellV1>,
    lock_manager: Option<ExtSessionLockManagerV1>,
    output: Option<WlOutput>,
    seat: Option<WlSeat>,
    shm: Option<WlShm>,
    /// The serials of the key events, press and release, in order.
    key_serials: Vec<u32>,
    synced: bool,
    /// The configures of each surface, by its index.
    surfaces: Vec<Configures>,
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
}

/// A toplevel of the client.
pub struct Toplevel {
    pub surface: WlSurface,
    xdg: XdgSurface,
    role: XdgToplevel,
    index: usize,
}

/// A popup of the client.
pub struct Popup {
    index: usize,
}

/// A lock surface of the client.
pub struct LockSurface {
    index: usize,
}

/// A layer surface of the client.
pub struct LayerSurface {
    pub surface: WlSurface,
    layer: ZwlrLayerSurfaceV1,
}

impl Wayland {
    /// Connects to the core and binds the globals.
    pub fn connect(core: &mut Core) -> Self {
        let (ours, theirs) = UnixStream::pair().expect("a socket pair");
        core.state
            .display_handle
            .insert_client(theirs, Arc::new(ClientState::default()))
            .expect("the core takes the client");
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
        let qh = self.queue.handle();
        let index = self.app.surfaces.len();
        self.app.surfaces.push(Configures::default());
        let compositor = self.app.compositor.as_ref().expect("wl_compositor");
        let surface = compositor.create_surface(&qh, ());
        let wm_base = self.app.wm_base.as_ref().expect("xdg_wm_base");
        let positioner = wm_base.create_positioner(&qh, ());
        positioner.set_size(50, 50);
        positioner.set_anchor_rect(0, 0, 1, 1);
        let xdg = wm_base.get_xdg_surface(&surface, &qh, index);
        let popup = xdg.get_popup(Some(&parent.xdg), &positioner, &qh, index);
        popup.grab(self.app.seat.as_ref().expect("wl_seat"), serial);
        surface.commit();
        self.roundtrip(core);
        Popup { index }
    }

    /// Whether the core closed a popup.
    pub fn popup_done(&self, popup: &Popup) -> bool {
        self.app.surfaces[popup.index].done
    }

    /// The serials of the key events that the client got.
    pub fn key_serials(&self) -> &[u32] {
        &self.app.key_serials
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
    pub fn attach(&mut self, core: &mut Core, toplevel: &Toplevel, (w, h): (i32, i32)) {
        if let Some(serial) = self.app.surfaces[toplevel.index].pending.take() {
            toplevel.xdg.ack_configure(serial);
        }
        let file = tempfile();
        file.set_len(u64::try_from(w * h * 4).expect("a size"))
            .expect("the file grows");
        let qh = self.queue.handle();
        let shm = self.app.shm.as_ref().expect("wl_shm");
        let pool = shm.create_pool(file.as_fd(), w * h * 4, &qh, ());
        let buffer = pool.create_buffer(0, w, h, w * 4, wl_shm::Format::Argb8888, &qh, ());
        pool.destroy();
        toplevel.surface.attach(Some(&buffer), 0, 0);
        toplevel.surface.damage(0, 0, w, h);
        toplevel.surface.commit();
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
        match interface.as_str() {
            "wl_compositor" => app.compositor = Some(registry.bind(name, version, qh, ())),
            "xdg_wm_base" => app.wm_base = Some(registry.bind(name, version, qh, ())),
            "zwlr_layer_shell_v1" => app.layer_shell = Some(registry.bind(name, version, qh, ())),
            "ext_session_lock_manager_v1" => {
                app.lock_manager = Some(registry.bind(name, version, qh, ()));
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
            && c.contains(wl_seat::Capability::Keyboard)
        {
            seat.get_keyboard(qh, ());
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
        if let wl_keyboard::Event::Key { serial, .. } = event {
            app.key_serials.push(serial);
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
        if let xdg_popup::Event::PopupDone = event {
            app.surfaces[*index].done = true;
        }
    }
}

delegate_noop!(App: ignore WlCompositor);
delegate_noop!(App: XdgPositioner);
delegate_noop!(App: ignore WlShm);
delegate_noop!(App: WlShmPool);
delegate_noop!(App: ignore WlBuffer);

/// A new file that is gone from the file system, for a shm pool.
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
delegate_noop!(App: ZwlrLayerShellV1);
delegate_noop!(App: ExtSessionLockManagerV1);
delegate_noop!(App: ignore ExtSessionLockV1);
