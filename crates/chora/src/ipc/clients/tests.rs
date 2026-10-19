//! Tests of the client sockets with a raw Wayland client: screencopy
//! frames, activation tokens, popup grabs and the idle globals.

use std::{
    io::Write as _,
    os::{fd::AsFd, unix::fs::FileExt, unix::net::UnixStream},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use serde_json::json;
use smithay::backend::input::{ButtonState, InputTime, KeyState};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_keyboard::{self, WlKeyboard},
        wl_output::WlOutput,
        wl_pointer::{self, WlPointer},
        wl_registry::{self, WlRegistry},
        wl_seat::WlSeat,
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::activation::v1::client::{
    xdg_activation_token_v1::{self, XdgActivationTokenV1},
    xdg_activation_v1::XdgActivationV1,
};
use wayland_protocols_wlr::{
    layer_shell::v1::client::{
        zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
        zwlr_layer_surface_v1::{self, Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
    },
    screencopy::v1::client::{
        zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
        zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
    },
};

use crate::{
    state::ClientState,
    testing::{Core, wayland::Wayland},
};

#[derive(Default)]
struct App {
    globals: Vec<(u32, String, u32)>,
    synced: bool,
    buffer: Option<(u32, u32, u32)>,
    ready: bool,
    failed: bool,
    tokens: Vec<String>,
    button_serials: Vec<u32>,
    configure: Option<(u32, u32, u32)>,
    keys: usize,
}

struct Raw {
    conn: Connection,
    queue: EventQueue<App>,
    app: App,
}

impl Raw {
    fn new(core: &mut Core, stream: UnixStream) -> Self {
        let conn = Connection::from_socket(stream).unwrap();
        let queue = conn.new_event_queue();
        conn.display().get_registry(&queue.handle(), ());
        let mut raw = Self {
            conn,
            queue,
            app: App::default(),
        };
        raw.roundtrip(core);
        raw
    }

    fn roundtrip(&mut self, core: &mut Core) {
        self.app.synced = false;
        self.conn.display().sync(&self.queue.handle(), ());
        for _ in 0..100 {
            self.conn.flush().unwrap();
            core.dispatch();
            if let Some(guard) = self.queue.prepare_read() {
                let _ = guard.read();
            }
            self.queue.dispatch_pending(&mut self.app).unwrap();
            if self.app.synced {
                return;
            }
        }
        panic!("no answer");
    }

    fn bind<I: wayland_client::Proxy + 'static>(&self, max: u32) -> I
    where
        App: Dispatch<I, ()>,
    {
        let name = I::interface().name;
        let (id, _, version) = self
            .app
            .globals
            .iter()
            .find(|(_, i, _)| i == name)
            .unwrap_or_else(|| panic!("no {name}"));
        let registry = self.conn.display().get_registry(&self.queue.handle(), ());
        registry.bind(*id, (*version).min(max), &self.queue.handle(), ())
    }

    fn offers(&self, part: &str) -> bool {
        self.app.globals.iter().any(|(_, i, _)| i.contains(part))
    }

    /// An shm buffer in a file that only its descriptor names, filled with
    /// a byte.
    fn buffer(
        &self,
        tag: &str,
        (w, h, stride): (u32, u32, u32),
        byte: u8,
    ) -> (WlBuffer, std::fs::File) {
        static SHM_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "chora-shm-{tag}-{}-{}",
            std::process::id(),
            SHM_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        ));
        let mut shm_file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(&path).unwrap();
        shm_file
            .write_all(&vec![byte; (stride * h) as usize])
            .unwrap();
        let int = |v: u32| i32::try_from(v).unwrap();
        let shm: WlShm = self.bind(1);
        let pool = shm.create_pool(shm_file.as_fd(), int(stride * h), &self.queue.handle(), ());
        let buffer = pool.create_buffer(
            0,
            int(w),
            int(h),
            int(stride),
            wl_shm::Format::Xrgb8888,
            &self.queue.handle(),
            (),
        );
        (buffer, shm_file)
    }
}

impl Dispatch<WlRegistry, ()> for App {
    fn event(
        app: &mut Self,
        _: &WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && !app.globals.iter().any(|(n, _, _)| *n == name)
        {
            app.globals.push((name, interface, version));
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
            zwlr_screencopy_frame_v1::Event::Buffer {
                width,
                height,
                stride,
                ..
            } => {
                app.buffer = Some((width, height, stride));
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => app.ready = true,
            zwlr_screencopy_frame_v1::Event::Failed => app.failed = true,
            _ => {}
        }
    }
}
impl Dispatch<XdgActivationTokenV1, ()> for App {
    fn event(
        app: &mut Self,
        _: &XdgActivationTokenV1,
        event: xdg_activation_token_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_activation_token_v1::Event::Done { token } = event {
            app.tokens.push(token);
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
        if let wl_pointer::Event::Button { serial, .. } = event {
            app.button_serials.push(serial);
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
        if let wl_keyboard::Event::Key { serial, state, .. } = event {
            app.keys += 1;
            if state == wayland_client::WEnum::Value(wl_keyboard::KeyState::Pressed) {
                app.button_serials.push(serial);
            }
        }
    }
}
impl Dispatch<ZwlrLayerSurfaceV1, ()> for App {
    fn event(
        app: &mut Self,
        _: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure {
            serial,
            width,
            height,
        } = event
        {
            app.configure = Some((serial, width, height));
        }
    }
}
delegate_noop!(App: ignore WlOutput);
delegate_noop!(App: ignore WlShm);
delegate_noop!(App: ignore WlShmPool);
delegate_noop!(App: ignore WlBuffer);
delegate_noop!(App: ignore WlSeat);
delegate_noop!(App: ignore WlSurface);
delegate_noop!(App: WlCompositor);
delegate_noop!(App: ZwlrScreencopyManagerV1);
delegate_noop!(App: XdgActivationV1);
delegate_noop!(App: ZwlrLayerShellV1);

fn socket_dir(tag: &str) -> (Core, PathBuf) {
    let dir = std::env::temp_dir().join(format!("chora-raw-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = Core::with_renderer();
    core.state.ipc.path = Some(dir.join("wayland-1.sock"));
    (core, dir)
}

fn named(core: &mut Core, admin: &mut crate::testing::Client, grant: Option<&str>) -> Raw {
    let created = admin.ok(core, "clients.create", json!({"name": "agent"}));
    if let Some(cap) = grant {
        admin.ok(
            core,
            "grants.set",
            json!({"client": "agent", "capabilities": [cap]}),
        );
    }
    let path = PathBuf::from(created["wayland"].as_str().unwrap());
    let stream = UnixStream::connect(&path).unwrap();
    core.dispatch();
    Raw::new(core, stream)
}

/// A frame queued before a revoke gets nothing of the screen into the shm
/// of the killed client.
#[test]
fn a_revoke_drops_the_queued_frames_of_a_capturer() {
    let (mut core, dir) = socket_dir("cap");
    core.add_output("A", (64, 48));
    let mut admin = core.admin();
    let mut agent = named(&mut core, &mut admin, Some("screen.capture"));
    let manager: ZwlrScreencopyManagerV1 = agent.bind(3);
    let output: WlOutput = agent.bind(1);
    let frame = manager.capture_output(0, &output, &agent.queue.handle(), ());
    agent.roundtrip(&mut core);
    let size = agent.app.buffer.expect("a buffer offer");
    let (buffer, file) = agent.buffer("cap", size, 0xAB);
    frame.copy(&buffer);
    agent.roundtrip(&mut core);
    assert!(!agent.app.ready, "the frame waits for the tick");
    admin.ok(
        &mut core,
        "grants.revoke",
        json!({"client": "agent", "capabilities": ["screen.capture"]}),
    );
    // The client sends nothing more; the tick comes one refresh later.
    std::thread::sleep(Duration::from_millis(60));
    core.dispatch();
    core.dispatch();
    let mut bytes = vec![0; (size.2 * size.1) as usize];
    file.read_exact_at(&mut bytes, 0).unwrap();
    let written = bytes.iter().filter(|b| **b != 0xAB).count();
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        written, 0,
        "the core wrote {written} bytes of the screen after the revoke"
    );
}

/// A client without any input gets no token with serial 0.
#[test]
fn a_token_with_serial_zero_is_refused() {
    let (mut core, dir) = socket_dir("zero");
    core.add_output("A", (64, 48));
    let mut admin = core.admin();
    let mut agent = named(&mut core, &mut admin, None);
    let activation: XdgActivationV1 = agent.bind(1);
    let seat: WlSeat = agent.bind(1);
    let token = activation.get_activation_token(&agent.queue.handle(), ());
    token.set_serial(0, &seat);
    token.commit();
    agent.roundtrip(&mut core);
    let kept = core.state.activation_state.tokens().count();
    // The wiring works: a token without a serial is refused.
    let refused = activation.get_activation_token(&agent.queue.handle(), ());
    refused.commit();
    agent.roundtrip(&mut core);
    let after = core.state.activation_state.tokens().count();
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(after, kept, "the hook is wired: no serial, no token");
    assert_eq!(kept, 0, "serial 0 without any press gave a token");
}

/// A click at a point, then a tap of the key a.
fn click_and_type(core: &mut Core, at: (f64, f64)) {
    crate::input::move_pointer(&mut core.state, at.into(), InputTime::now());
    for state in [ButtonState::Pressed, ButtonState::Released] {
        crate::input::press_button(&mut core.state, 272, state, InputTime::now());
    }
    let a = smithay::input::keyboard::Keycode::new(38);
    for state in [KeyState::Pressed, KeyState::Released] {
        crate::input::key(&mut core.state, a, state, InputTime::now(), false);
    }
}

/// The objects that keep a layer surface of a raw client mapped.
type Mapped = (WlSurface, ZwlrLayerSurfaceV1, WlBuffer, std::fs::File);

/// A mapped layer surface: of a size at the top left, or over the whole
/// output without a size.
fn map_layer(
    raw: &mut Raw,
    core: &mut Core,
    layer: Layer,
    size: Option<(u32, u32)>,
    keyboard: KeyboardInteractivity,
) -> Mapped {
    let compositor: WlCompositor = raw.bind(4);
    let shell: ZwlrLayerShellV1 = raw.bind(4);
    let qh = raw.queue.handle();
    let surface = compositor.create_surface(&qh, ());
    let role = shell.get_layer_surface(&surface, None, layer, "test".into(), &qh, ());
    if let Some((w, h)) = size {
        role.set_size(w, h);
        role.set_anchor(Anchor::Top | Anchor::Left);
    } else {
        role.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
        role.set_exclusive_zone(-1);
    }
    role.set_keyboard_interactivity(keyboard);
    surface.commit();
    raw.roundtrip(core);
    let (serial, w, h) = raw.app.configure.expect("a configure");
    role.ack_configure(serial);
    let (buffer, file) = raw.buffer(&format!("layer-{w}x{h}"), (w, h, w * 4), 0x20);
    surface.attach(Some(&buffer), 0, 0);
    surface.commit();
    raw.roundtrip(core);
    (surface, role, buffer, file)
}

/// A full-screen top-layer panel of a client socket gets the keys after one
/// click; that click mints no tokens once the user moved on to another
/// client.
#[test]
fn one_old_press_mints_no_tokens_after_the_focus_moved() {
    let (mut core, dir) = socket_dir("lock");
    core.add_output("A", (64, 48));
    let mut admin = core.admin();
    let mut agent = named(&mut core, &mut admin, None);
    let seat: WlSeat = agent.bind(7);
    let _pointer = seat.get_pointer(&agent.queue.handle(), ());
    let _keyboard = seat.get_keyboard(&agent.queue.handle(), ());
    let full = KeyboardInteractivity::Exclusive;
    let _panel = map_layer(&mut agent, &mut core, Layer::Top, None, full);
    // The user clicks into the panel and types.
    click_and_type(&mut core, (30.0, 20.0));
    agent.roundtrip(&mut core);
    let typed = agent.app.keys;
    let press = *agent.app.button_serials.last().expect("a press");
    // The user moves on: a launcher of the main socket in the overlay
    // layer gets the next clicks and keys.
    let (ours, theirs) = UnixStream::pair().unwrap();
    core.state
        .display_handle
        .insert_client(theirs, Arc::new(ClientState::default()))
        .unwrap();
    let mut other = Raw::new(&mut core, ours);
    let oseat: WlSeat = other.bind(7);
    let _ok = oseat.get_keyboard(&other.queue.handle(), ());
    let demand = KeyboardInteractivity::OnDemand;
    let _launcher = map_layer(
        &mut other,
        &mut core,
        Layer::Overlay,
        Some((20, 10)),
        demand,
    );
    click_and_type(&mut core, (5.0, 5.0));
    other.roundtrip(&mut core);
    agent.roundtrip(&mut core);
    assert_eq!(agent.app.keys, typed, "the agent got no further keys");
    assert!(other.app.keys >= 1, "the other client got the keys");
    // Later, the agent mints two tokens with its old serial.
    let activation: XdgActivationV1 = agent.bind(1);
    for _ in 0..2 {
        let token = activation.get_activation_token(&agent.queue.handle(), ());
        token.set_serial(press, &seat);
        token.commit();
    }
    agent.roundtrip(&mut core);
    let tokens = core.state.activation_state.tokens().count();
    drop(other);
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    // Typed keys reach a top layer panel after a click, as documented.
    assert!(typed > 0);
    assert_eq!(
        tokens, 0,
        "an old press minted tokens after the user moved on"
    );
}

/// The main socket keeps the idle globals and the security context
/// manager, and a sandbox keeps the idle globals.
#[test]
fn a_sandbox_keeps_the_idle_globals() {
    let mut core = Core::new();
    let (ours, theirs) = UnixStream::pair().unwrap();
    let creator = core
        .state
        .display_handle
        .insert_client(theirs, Arc::new(ClientState::default()))
        .unwrap()
        .id();
    let main = Raw::new(&mut core, ours);
    for part in [
        "zwp_idle_inhibit_manager_v1",
        "ext_idle_notifier_v1",
        "wp_security_context_manager_v1",
    ] {
        assert!(main.offers(part), "main socket lacks {part}");
    }
    // An idle inhibitor of the main socket binds and works.
    let (ours, theirs) = UnixStream::pair().unwrap();
    let sandbox = ClientState {
        security_context: Some(smithay::wayland::security_context::SecurityContext {
            sandbox_engine: Some("org.flatpak".into()),
            app_id: Some("org.mozilla.firefox".into()),
            instance_id: None,
            creator_client_id: creator,
        }),
        ..ClientState::default()
    };
    core.state
        .display_handle
        .insert_client(theirs, Arc::new(sandbox))
        .unwrap();
    let flatpak = Raw::new(&mut core, ours);
    assert!(
        flatpak.offers("zwp_idle_inhibit_manager_v1"),
        "a video player in a sandbox inhibits idle"
    );
    assert!(flatpak.offers("ext_idle_notifier_v1"));
}

/// A popup grab of a client socket with serial 0 and no input takes no
/// keyboard.
#[test]
fn a_popup_grab_with_serial_zero_keeps_no_keyboard() {
    let (mut core, dir) = socket_dir("popup");
    let output = core.add_output("A", (800, 600));
    let mut admin = core.admin();
    let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
    let path = PathBuf::from(created["wayland"].as_str().unwrap());
    // The user types into a launcher of the main socket.
    let mut user = Wayland::connect(&mut core);
    user.layer(&mut core, Layer::Top, KeyboardInteractivity::OnDemand);
    let launcher = smithay::desktop::layer_map_for_output(&output)
        .layers()
        .next()
        .map(|l| l.wl_surface().clone());
    core.state.set_layer_focus(launcher.clone());
    let keyboard = core.state.seat.get_keyboard().unwrap();
    assert_eq!(keyboard.current_focus(), launcher);
    let mut agent = Wayland::connect_to(&mut core, &path);
    let top = agent.toplevel(&mut core, "agent");
    let popup = agent.popup(&mut core, &top, 0);
    let done = agent.popup_done(&popup);
    let focus = keyboard.current_focus();
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        done || focus == launcher,
        "a popup with serial 0 took the keyboard from the launcher"
    );
}

/// The agent gets one key press while it has the focus, then the user moves
/// to a launcher of the main socket. Returns the press serial.
fn agent_pressed_then_launcher(
    core: &mut Core,
    agent: &mut Wayland,
    user: &mut Wayland,
    output: &smithay::output::Output,
) -> (
    u32,
    Option<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
) {
    let id = core
        .state
        .windows
        .keys()
        .next()
        .cloned()
        .expect("the agent window");
    core.state.focus(Some(&id));
    let a = smithay::input::keyboard::Keycode::new(38);
    for state in [KeyState::Pressed, KeyState::Released] {
        crate::input::key(&mut core.state, a, state, InputTime::now(), false);
    }
    agent.roundtrip(core);
    let press = *agent.key_serials().first().expect("the agent got the key");
    user.layer(core, Layer::Top, KeyboardInteractivity::OnDemand);
    let launcher = smithay::desktop::layer_map_for_output(output)
        .layers()
        .next()
        .map(|l| l.wl_surface().clone());
    core.state.set_layer_focus(launcher.clone());
    let keyboard = core.state.seat.get_keyboard().unwrap();
    assert_eq!(keyboard.current_focus(), launcher, "the user moved on");
    (press, launcher)
}

/// An old press grabs no keyboard with a popup after the user moved on.
#[test]
fn an_old_press_grabs_no_keyboard_after_the_focus_moved() {
    let (mut core, dir) = socket_dir("old-press");
    let output = core.add_output("A", (800, 600));
    let mut admin = core.admin();
    let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
    let path = PathBuf::from(created["wayland"].as_str().unwrap());
    let mut user = Wayland::connect(&mut core);
    let mut agent = Wayland::connect_to(&mut core, &path);
    let top = agent.toplevel(&mut core, "agent");
    let (press, launcher) = agent_pressed_then_launcher(&mut core, &mut agent, &mut user, &output);
    let popup = agent.popup(&mut core, &top, press);
    let done = agent.popup_done(&popup);
    let focus = core.state.seat.get_keyboard().unwrap().current_focus();
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(
        done || focus == launcher,
        "a popup with an old press took the keyboard from the launcher"
    );
}

delegate_noop!(App: wayland_protocols::wp::pointer_constraints::zv1::client::zwp_pointer_constraints_v1::ZwpPointerConstraintsV1);
delegate_noop!(App: ignore wayland_protocols::wp::pointer_constraints::zv1::client::zwp_locked_pointer_v1::ZwpLockedPointerV1);

/// A full-screen top-layer panel of a client socket locks no pointer that
/// only passes over it, without the keyboard focus.
#[test]
fn a_hover_locks_no_pointer_of_a_client_socket() {
    use wayland_protocols::wp::pointer_constraints::zv1::client::zwp_pointer_constraints_v1::{
        Lifetime, ZwpPointerConstraintsV1,
    };
    let (mut core, dir) = socket_dir("hover");
    core.add_output("A", (64, 48));
    let mut admin = core.admin();
    let mut agent = named(&mut core, &mut admin, None);
    let seat: WlSeat = agent.bind(7);
    let pointer = seat.get_pointer(&agent.queue.handle(), ());
    let none = KeyboardInteractivity::None;
    let (surface, _role, _b, _f) = map_layer(&mut agent, &mut core, Layer::Top, None, none);
    let constraints: ZwpPointerConstraintsV1 = agent.bind(1);
    let _lock = constraints.lock_pointer(
        &surface,
        &pointer,
        None,
        Lifetime::Persistent,
        &agent.queue.handle(),
        (),
    );
    agent.roundtrip(&mut core);
    // The user only moves the pointer across the screen.
    crate::input::move_pointer(&mut core.state, (30.0, 20.0).into(), InputTime::now());
    agent.roundtrip(&mut core);
    let server = core.state.seat.get_pointer().unwrap();
    let focus = server.current_focus().expect("the panel has the pointer");
    let mut active = false;
    smithay::wayland::pointer_constraints::with_pointer_constraint(&focus, &server, |c| {
        active = c.is_some_and(|c| c.is_active());
    });
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(!active, "a hover locked the pointer for a client socket");
}

/// A core with an output `A`, the client socket of `agent`, a connection
/// through it after `hello`, and the path of its Wayland socket.
fn agent_ipc(tag: &str) -> (Core, PathBuf, crate::testing::Client, PathBuf) {
    let dir = std::env::temp_dir().join(format!("chora-a1-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut core = Core::new();
    core.state.ipc.path = Some(dir.join("wayland-1.sock"));
    core.add_output("A", (800, 600));
    let mut admin = core.admin();
    let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
    let mut agent = core.connect_to(&PathBuf::from(created["path"].as_str().unwrap()));
    agent.ok(&mut core, "hello", json!({"name": "agent"}));
    let wayland = PathBuf::from(created["wayland"].as_str().unwrap());
    (core, dir, agent, wayland)
}

/// The commands of a `scene.apply`.
fn apply(commands: &serde_json::Value) -> serde_json::Value {
    json!({ "commands": commands })
}

/// A client socket puts no node under an overlay root, directly or below
/// a group of another client there; the main socket still does.
#[test]
fn a_client_socket_draws_nothing_on_the_screen_overlay() {
    let (mut core, dir, mut agent, _) = agent_ipc("screen");
    let mut wm = core.named("wm");
    let group =
        |id: &str, parent: &str| json!([{"create": {"id": id, "kind": "group", "parent": parent}}]);
    wm.ok(&mut core, "scene.apply", apply(&group("hud", "screen:A")));
    agent.ok(&mut core, "scene.apply", apply(&group("g", "root")));
    for commands in [
        group("spy", "screen:A"),
        group("spy", "hud"),
        json!([{"move": {"id": "g", "parent": "screen:A"}}]),
    ] {
        let (_, message) = agent.fails(&mut core, "scene.apply", apply(&commands));
        assert!(message.contains("no screen:"), "{message}");
    }
    let screen = core.state.scene.node("screen:A").unwrap().children.clone();
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(screen, ["hud"], "only the main socket draws on the screen");
}

/// A client socket sets no camera; the main socket still does.
#[test]
fn a_client_socket_takes_no_camera() {
    let (mut core, dir, mut agent, _) = agent_ipc("camera");
    let camera = apply(&json!([{"camera": {"output": "A", "transform": {"scale": 2}}}]));
    let (_, message) = agent.fails(&mut core, "scene.apply", camera.clone());
    assert!(message.contains("no camera"), "{message}");
    let release = apply(&json!([{"camera": {"output": "A"}}]));
    let (_, message) = agent.fails(&mut core, "scene.apply", release);
    assert!(message.contains("no camera"), "{message}");
    core.named("wm").ok(&mut core, "scene.apply", camera);
    let owner = core.state.scene.camera("A").is_some();
    drop(core);
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(owner, "the main socket sets the camera");
}
