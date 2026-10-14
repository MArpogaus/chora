//! A Wayland client in the process of the test: toplevels, layer surfaces,
//! and the configures that they get. The client and the core run in one
//! thread, so a roundtrip turns the event loop of the core.

use std::{os::unix::net::UnixStream, sync::Arc};

use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
        wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{
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
    output: Option<WlOutput>,
    synced: bool,
    /// The configures of each surface, by its index.
    surfaces: Vec<Configures>,
}

/// The configures of a surface.
#[derive(Default)]
pub struct Configures {
    /// The serial of the newest configure that the client did not ack.
    pending: Option<u32>,
    /// The size and the states of the newest toplevel configure.
    pub size: (i32, i32),
    pub states: Vec<xdg_toplevel::State>,
    /// How many configures arrived.
    pub count: usize,
}

/// A toplevel of the client.
pub struct Toplevel {
    pub surface: WlSurface,
    xdg: XdgSurface,
    role: XdgToplevel,
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

    /// Destroys a toplevel.
    pub fn destroy(&mut self, core: &mut Core, toplevel: &Toplevel) {
        toplevel.role.destroy();
        toplevel.xdg.destroy();
        toplevel.surface.destroy();
        self.roundtrip(core);
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

delegate_noop!(App: ignore WlCompositor);
delegate_noop!(App: ignore WlSurface);
delegate_noop!(App: ZwlrLayerShellV1);
