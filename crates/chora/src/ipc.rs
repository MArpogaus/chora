//! The IPC socket: JSON-RPC 2.0, one message per line, served by the event
//! loop of the core (D29).
//!
//! Limits keep a buggy client from exhausting the core: a number of
//! connections, a time until `hello`, a line length, an output buffer per
//! connection, and a read size per wakeup, so that one busy client cannot
//! stall the event loop.

use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read, Write},
    os::unix::{
        fs::{DirBuilderExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Event, MOVE, Modifiers, Node, NodeKind, Placed, Scene, Selector, Transform,
    ipc::{
        Apply, Capability, Edges, Emit, Grab, Hello, Message, Query, Request, RpcError, Subscribe,
    },
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use smithay::utils::{Logical, Rectangle};
use smithay::{
    input::keyboard::{Keysym, ModifiersState, xkb},
    reexports::calloop::{
        Interest, Mode, PostAction, RegistrationToken,
        generic::Generic,
        timer::{TimeoutAction, Timer},
    },
    reexports::rustix::{net::sockopt::socket_peercred, process::Pid},
};

use crate::{
    output::output_area,
    render::{location_and_scale, window_rect},
    state::State,
    uploads::Uploads,
};

mod clients;

pub use clients::window_client;

const MAX_CONNECTIONS: usize = 128;
/// The most connections of one process, so that one client cannot take
/// every slot.
const MAX_PER_PROCESS: usize = 16;
/// The last slots go only to processes without a connection, for example
/// `choractl` or a window manager that starts again.
const KEPT_FOR_NEW_PROCESSES: usize = 8;
const MAX_LINE: usize = 1 << 20;
const MAX_OUTPUT: usize = 8 << 20;
const READ_PER_WAKEUP: usize = 64 << 10;
/// How long a new connection may wait before its `hello`, so that idle
/// connections cannot take every slot.
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);
/// How long the core keeps the nodes of a client with `resume` after its
/// last connection closed, so that a restart gets them back.
const GRACE: Duration = Duration::from_secs(10);
/// The most bytes of the data of one emitted event (D65).
const MAX_EMIT: usize = 64 << 10;
/// The most events that one client name emits per `EMIT_WINDOW` (D65).
const MAX_EMITS: u32 = 32;
const EMIT_WINDOW: Duration = Duration::from_secs(1);

/// The connections of the IPC socket.
#[derive(Default)]
pub struct Ipc {
    pub(crate) path: Option<PathBuf>,
    /// The files of `assets.put`.
    pub uploads: Uploads,
    listener: Option<RegistrationToken>,
    /// The listener of the admin socket and its path.
    admin: Option<(RegistrationToken, PathBuf)>,
    conns: HashMap<u64, Conn>,
    next: u64,
    grabs: Vec<(Binding, Trigger, u64)>,
    /// The client names with `resume` without a connection, and the end of
    /// their grace time.
    graces: HashMap<String, Instant>,
    /// The client sockets by name: the source of the listener and the path.
    clients: HashMap<String, clients::ClientSocket>,
    /// A client said hello since the core started.
    pub greeted: bool,
    /// The emitted events per client name in the current window: its start
    /// and the count (D65).
    emits: HashMap<String, (Instant, u32)>,
}

/// What a grab waits for: a key or a pointer button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trigger {
    Key(Keysym),
    Button(u32),
    /// The interactive moves of the own windows (`xdg_toplevel.move`).
    Move,
}

/// The names of the pointer buttons of a binding, with their Linux codes.
const BUTTONS: [(&str, u32); 7] = [
    ("BTN_LEFT", 0x110),
    ("BTN_RIGHT", 0x111),
    ("BTN_MIDDLE", 0x112),
    ("BTN_SIDE", 0x113),
    ("BTN_EXTRA", 0x114),
    ("BTN_FORWARD", 0x115),
    ("BTN_BACK", 0x116),
];

/// The key or the button of a binding, or the moves of `move`.
fn trigger(binding: &Binding) -> Result<Trigger, RpcError> {
    if binding.key == MOVE && binding.mods == Modifiers::default() {
        return Ok(Trigger::Move);
    }
    if let Some((_, code)) = BUTTONS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&binding.key))
    {
        return Ok(Trigger::Button(*code));
    }
    let sym = xkb::keysym_from_name(&binding.key, xkb::KEYSYM_CASE_INSENSITIVE);
    if sym.raw() == 0 {
        return Err(RpcError::new(
            RpcError::INVALID_PARAMS,
            format!("unknown key {:?}", binding.key),
        ));
    }
    Ok(Trigger::Key(sym))
}

// Each bool is one fact of the connection; no state machine joins them.
#[allow(clippy::struct_excessive_bools)]
struct Conn {
    stream: UnixStream,
    /// The process of the client; `None` if the core cannot see it.
    pid: Option<Pid>,
    input: Vec<u8>,
    output: Vec<u8>,
    /// The output passed `MAX_OUTPUT`; the connection closes.
    overflow: bool,
    /// A line passed `MAX_LINE`: the core skips the rest of it, answers
    /// with an error and closes the connection.
    too_long: bool,
    name: Option<String>,
    /// The deadline of `hello`.
    hello_by: Instant,
    keep_nodes: bool,
    resume: bool,
    /// The `hello` asked for the capability rules of a client socket.
    confined: bool,
    /// The socket that the connection came through.
    socket: Socket,
    events: Option<Vec<String>>,
    /// The number of events sent to this connection; each event carries
    /// it as `seq` (D32).
    seq: u64,
    /// The edges of `output.reserve` by output name.
    reserves: HashMap<String, Edges>,
    /// The source that reads the connection. `None` after the end of the
    /// stream or a line that is too long: then the connection closes when
    /// its answers are out.
    reader: Option<RegistrationToken>,
}

/// Opens the socket in `$XDG_RUNTIME_DIR/chora/` and returns its path.
pub fn listen(state: &mut State) -> io::Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR is not set"))?;
    let dir = Path::new(&runtime).join("chora");
    private_dir(&dir)?;
    let mut path = dir.join(&state.socket_name);
    path.set_extension("sock");
    let mut assets = path.clone();
    assets.set_extension("assets");
    state.ipc.uploads.open(assets)?;
    state.ipc.listener = Some(bind(state, &path, Socket::Main)?);
    let admin = chora_scene::ipc::admin_socket(&path);
    state.ipc.admin = Some((bind(state, &admin, Socket::Admin)?, admin));
    state.ipc.path = Some(path.clone());
    Ok(path)
}

/// Where a connection came from.
#[derive(Clone, PartialEq, Eq)]
enum Socket {
    Main,
    /// The admin socket of `choractl` (D40).
    Admin,
    /// A client socket of a name (D33).
    Client(String),
}

/// Binds the main or the admin socket, with mode 0600, and accepts its
/// connections in the event loop.
fn bind(state: &State, path: &Path, socket: Socket) -> io::Result<RegistrationToken> {
    match std::fs::remove_file(path) {
        Err(err) if err.kind() != ErrorKind::NotFound => return Err(err),
        _ => {}
    }
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    state
        .loop_handle
        .insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            move |_, listener, state| {
                if let Some(error) = state.ipc_accept_all(listener, &socket) {
                    tracing::warn!("IPC socket: {error}");
                }
                Ok(PostAction::Continue)
            },
        )
        .map_err(|err| io::Error::other(err.to_string()))
}

/// Creates a directory with mode 0700, or sets that mode on an old one,
/// and checks that it is a real directory of the owner of its parent.
fn private_dir(dir: &Path) -> io::Result<()> {
    let parent = dir.parent().ok_or_else(|| io::Error::other("no parent"))?;
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(dir)?;
    let meta = std::fs::symlink_metadata(dir)?;
    if !meta.is_dir() || meta.uid() != std::fs::metadata(parent)?.uid() {
        return Err(io::Error::other(format!(
            "{} is no directory of the owner of its parent",
            dir.display()
        )));
    }
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

impl Drop for Ipc {
    fn drop(&mut self) {
        let admin = self.admin.iter().map(|(_, path)| path);
        for path in self.path.iter().chain(admin) {
            let _ = std::fs::remove_file(path);
        }
        for socket in self.clients.values() {
            socket.remove_files();
        }
    }
}

impl Ipc {
    /// The reserves of all connections on an output, added up.
    pub fn reserved(&self, output: &str) -> Edges {
        self.conns
            .values()
            .filter_map(|c| c.reserves.get(output))
            .fold(Edges::default(), |all, e| all.plus(*e))
    }

    /// Sets the reserve of a connection on an output; no space removes it.
    pub fn set_reserve(&mut self, conn: u64, output: String, edges: Edges) {
        if let Some(c) = self.conns.get_mut(&conn) {
            if edges.is_empty() {
                c.reserves.remove(&output);
            } else {
                c.reserves.insert(output, edges);
            }
        }
    }

    /// Sends an event to every connection that subscribed to it.
    /// `window_client` names the client socket of a surface, see
    /// [`Ipc::send_subscribed`].
    pub fn broadcast(&mut self, event: &Event, window_client: impl Fn(&str) -> Option<String>) {
        self.send_subscribed(None, event, window_client);
    }

    /// Sends an event of a node to the connections of a client name that
    /// subscribed to it, for example the keys of a focused node to its owner.
    pub fn send_to_client(&mut self, client: &str, event: &Event) {
        self.send_subscribed(Some(client), event, |_| None);
    }

    /// Sends an event to the subscribed connections, of a client name or
    /// all. A connection of a client socket gets a `surface.*` event only
    /// when `window_client` gives its name for the surface.
    pub fn send_subscribed(
        &mut self,
        client: Option<&str>,
        event: &Event,
        window_client: impl Fn(&str) -> Option<String>,
    ) {
        let message = Message::event(event);
        let core = !matches!(event, Event::Emitted { .. });
        let params = message.params.as_ref();
        let name = params.and_then(|p| p["event"].as_str());
        let window = name.filter(|n| n.starts_with("surface.")).map(|_| {
            params
                .and_then(|p| p["surface"].as_str())
                .and_then(window_client)
        });
        for conn in self.conns.values_mut() {
            let wanted = name.is_some_and(|n| conn.wants(n, core));
            let foreign = window
                .as_ref()
                .is_some_and(|w| conn.fixed().is_some_and(|f| w.as_deref() != Some(f)));
            if wanted && !foreign && client.is_none_or(|c| conn.name.as_deref() == Some(c)) {
                conn.seq += 1;
                conn.push(&stamped(&message, conn.seq));
            }
        }
    }

    /// Sends an event to one connection, subscribed or not.
    pub fn send_to(&mut self, conn: u64, event: &Event) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.seq += 1;
            c.push(&stamped(&Message::event(event), c.seq));
        }
    }

    /// Sends a response to one connection. A result that does not fit in
    /// the output of the connection becomes an error, so that a large
    /// answer, for example of a query of all nodes, does not close it.
    pub fn respond(&mut self, conn: u64, message: &Message) {
        let Some(c) = self.conns.get_mut(&conn) else {
            return;
        };
        let mut bytes = line(message);
        if message.result.is_some() && c.output.len() + bytes.len() > MAX_OUTPUT {
            let error = RpcError::new(
                RpcError::LIMIT_REACHED,
                format!(
                    "the answer has {} bytes and does not fit in the output of the connection, \
                     {MAX_OUTPUT} bytes; ask for less, for example with a narrower selector",
                    bytes.len()
                ),
            );
            let id = message.id.clone().unwrap_or(Value::Null);
            bytes = line(&Message::error(id, error));
        }
        c.push(&bytes);
    }

    /// The live connections of a client name.
    pub fn conns_of(&self, name: &str) -> Vec<u64> {
        self.conns
            .iter()
            .filter(|(_, c)| c.name.as_deref() == Some(name))
            .map(|(conn, _)| *conn)
            .collect()
    }

    /// Drops the registrations of a connection for bindings without Ctrl,
    /// Alt or Super, as happens when `input.grab-any` is revoked.
    pub fn drop_free_grabs(&mut self, conn: u64) {
        self.grabs.retain(|(b, t, c)| {
            !(*c == conn && *t != Trigger::Move && crate::input::free_binding(b))
        });
    }

    /// The connection that grabbed the pressed binding, and the binding.
    pub fn grab_for(&self, mods: &ModifiersState, syms: &[Keysym]) -> Option<(u64, Binding)> {
        self.grab_of(
            mods,
            |t| matches!(t, Trigger::Key(sym) if syms.contains(&sym)),
        )
    }

    /// The connection of a client name that grabbed the interactive moves.
    pub fn move_grab_of(&self, name: &str) -> Option<u64> {
        (self.grabs.iter())
            .find(|(_, t, c)| *t == Trigger::Move && self.name(*c) == Some(name))
            .map(|(_, _, c)| *c)
    }

    /// The connection that grabbed a pressed pointer button with the
    /// modifiers, and the binding. Of several grabs, the one whose client
    /// holds the smallest claim under `pointer` wins, else the first (D55).
    pub fn button_grab_for(
        &self,
        mods: &ModifiersState,
        button: u32,
        scene: &Scene,
        pointer: [f64; 2],
    ) -> Option<(u64, Binding)> {
        let first = self.grab_of(mods, |t| t == Trigger::Button(button))?;
        let grabbers: Vec<u64> = (self.grabs.iter())
            .filter(|(b, t, _)| b.mods == first.1.mods && *t == Trigger::Button(button))
            .map(|(_, _, conn)| *conn)
            .collect();
        let grabber = |name: &str| grabbers.iter().find(|c| self.name(**c) == Some(name));
        let conn = chora_scene::claimant_at(scene.nodes(), pointer, |n| grabber(n).is_some())
            .and_then(grabber)
            .map_or(first.0, |c| *c);
        Some((conn, first.1))
    }

    fn grab_of(
        &self,
        mods: &ModifiersState,
        matches: impl Fn(Trigger) -> bool,
    ) -> Option<(u64, Binding)> {
        self.grabs
            .iter()
            .find(|(b, trigger, _)| {
                let m = b.mods;
                (m.ctrl, m.alt, m.shift, m.logo) == (mods.ctrl, mods.alt, mods.shift, mods.logo)
                    && matches(*trigger)
            })
            .map(|(b, _, conn)| (*conn, b.clone()))
    }

    /// The connections without a name whose deadline of `hello` passed.
    pub fn silent(&self, now: Instant) -> Vec<u64> {
        self.conns
            .iter()
            .filter(|(_, c)| c.name.is_none() && c.hello_by <= now)
            .map(|(id, _)| *id)
            .collect()
    }

    /// The client names whose grace time ended.
    pub fn graces_over(&self, now: Instant) -> Vec<String> {
        self.graces
            .iter()
            .filter(|(_, end)| **end <= now)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Whether a connection of a client name subscribes to an event.
    pub fn subscribes(&self, client: &str, event: &str) -> bool {
        self.conns
            .values()
            .any(|c| c.name.as_deref() == Some(client) && c.wants(event, true))
    }

    /// Whether a client name has a connection.
    pub fn has_client(&self, name: &str) -> bool {
        self.conns.values().any(|c| c.name.as_deref() == Some(name))
    }

    /// Whether a client name has a connection or is in its grace time.
    pub fn live(&self, name: &str) -> bool {
        self.has_client(name) || self.graces.contains_key(name)
    }

    /// Whether a client name has a client socket.
    pub fn has_socket(&self, name: &str) -> bool {
        self.clients.contains_key(name)
    }

    /// Whether a connection came through the admin socket.
    fn admin(&self, conn: u64) -> bool {
        self.conns
            .get(&conn)
            .is_some_and(|c| c.socket == Socket::Admin)
    }

    /// The name of the client socket of a connection.
    pub(crate) fn fixed(&self, conn: u64) -> Option<&str> {
        self.conns.get(&conn)?.fixed()
    }

    /// Whether a connection follows the capability rules of a client
    /// socket: it came through one, or its `hello` confined it.
    pub(crate) fn confined(&self, conn: u64) -> bool {
        self.conns
            .get(&conn)
            .is_some_and(|c| c.confined || c.fixed().is_some())
    }

    /// The client name of a connection.
    pub fn name(&self, conn: u64) -> Option<&str> {
        self.conns.get(&conn)?.name.as_deref()
    }

    /// The ids of all connections.
    pub fn conn_ids(&self) -> Vec<u64> {
        self.conns.keys().copied().collect()
    }

    /// Why a new connection of a process cannot get a slot, if it cannot.
    fn refusal(&self, pid: Option<Pid>, fixed: Option<&str>) -> Option<String> {
        let total = self.conns.len();
        if let Some(name) = fixed {
            let socket = self.conns.values().filter(|c| c.fixed() == Some(name));
            let clients = self.conns.values().filter(|c| c.fixed().is_some()).count();
            if socket.count() >= clients::MAX_PER_SOCKET {
                return Some(format!(
                    "too many connections: this client socket holds {}",
                    clients::MAX_PER_SOCKET
                ));
            }
            if clients >= MAX_CONNECTIONS - clients::KEPT_FOR_MAIN {
                return Some(format!(
                    "too many connections: the last {} are kept for the main socket",
                    clients::KEPT_FOR_MAIN
                ));
            }
        }
        let own = pid.map_or(0, |pid| {
            self.conns.values().filter(|c| c.pid == Some(pid)).count()
        });
        if total >= MAX_CONNECTIONS {
            Some(format!(
                "too many connections: the core holds {MAX_CONNECTIONS}"
            ))
        } else if own >= MAX_PER_PROCESS {
            Some(format!(
                "too many connections: this process holds {MAX_PER_PROCESS}"
            ))
        } else if own > 0 && total >= MAX_CONNECTIONS - KEPT_FOR_NEW_PROCESSES {
            Some(format!(
                "too many connections: the last {KEPT_FOR_NEW_PROCESSES} are kept for \
                 processes without one"
            ))
        } else {
            None
        }
    }
}

impl Socket {
    /// The name of a client socket (D33).
    fn name(&self) -> Option<&str> {
        match self {
            Socket::Client(name) => Some(name),
            Socket::Main | Socket::Admin => None,
        }
    }
}

impl Conn {
    /// Whether the connection subscribes to an event; an empty list
    /// subscribes to all.
    /// Whether the connection subscribed to an event. An empty list means
    /// every core event; an emitted event needs its name or a pattern such
    /// as `voice.*` (D65).
    fn wants(&self, event: &str, core: bool) -> bool {
        (self.events.as_ref()).is_some_and(|e| {
            (core && e.is_empty())
                || e.iter().any(|n| {
                    n == event
                        || n.strip_suffix('*')
                            .is_some_and(|p| p.ends_with('.') && event.starts_with(p))
                })
        })
    }

    /// The name of the client socket of the connection (D33).
    fn fixed(&self) -> Option<&str> {
        self.socket.name()
    }

    fn push(&mut self, line: &[u8]) {
        if self.output.len() + line.len() > MAX_OUTPUT {
            self.overflow = true;
        } else {
            self.output.extend_from_slice(line);
        }
    }

    /// Writes as much output as the socket takes. `false` when the
    /// connection must close.
    fn flush(&mut self) -> bool {
        while !self.output.is_empty() && !self.overflow {
            match self.stream.write(&self.output) {
                Ok(0) => return false,
                Ok(n) => {
                    self.output.drain(..n);
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return false,
            }
        }
        !self.overflow
    }

    /// Reads at most `READ_PER_WAKEUP` bytes. `false` at the end of the
    /// stream or on an error.
    fn read(&mut self) -> bool {
        let mut buf = [0; 4096];
        let mut total = 0;
        while total < READ_PER_WAKEUP {
            match (&self.stream).read(&mut buf) {
                Ok(0) => return false,
                Ok(n) => {
                    self.input.extend_from_slice(&buf[..n]);
                    total += n;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return false,
            }
        }
        true
    }

    /// The next complete line, without its newline.
    fn next_line(&mut self) -> Option<Vec<u8>> {
        let end = self.input.iter().position(|b| *b == b'\n')?;
        let mut line: Vec<u8> = self.input.drain(..=end).collect();
        line.pop();
        Some(line)
    }
}

fn line(message: &Message) -> Vec<u8> {
    #[allow(
        clippy::expect_used,
        reason = "a message has no map with non-string keys"
    )]
    let mut bytes = serde_json::to_vec(message).expect("messages always serialize");
    bytes.push(b'\n');
    bytes
}

/// The line of an event notification with the sequence number of the
/// connection (D32): every connection numbers its events from 1.
/// Whether the first part of a name is that of core events, such as
/// `surface` or `output`.
fn core_namespace(name: &str) -> bool {
    let first = |n: &str| n.split('.').next().unwrap_or_default().to_owned();
    let head = first(name);
    chora_scene::EVENT_NAMES.iter().any(|e| first(e) == head)
}

/// Whether `events.subscribe` takes a name: a core event, the event of a
/// client, `NAME.something`, or a pattern `PREFIX.*` (D65). A typo in the
/// name of a core event is refused.
fn subscribable(name: &str) -> bool {
    use chora_scene::ipc::valid_name;
    if chora_scene::EVENT_NAMES.contains(&name) {
        return true;
    }
    if let Some(prefix) = name.strip_suffix(".*") {
        return valid_name(prefix);
    }
    !core_namespace(name)
        && (name.split_once('.')).is_some_and(|(head, rest)| valid_name(head) && valid_name(rest))
}

fn stamped(message: &Message, seq: u64) -> Vec<u8> {
    let mut message = message.clone();
    if let Some(Value::Object(params)) = message.params.as_mut() {
        params.insert("seq".into(), Value::from(seq));
    }
    line(&message)
}

impl State {
    /// Accepts every waiting connection. When accepting fails, for example
    /// because no file descriptors are left, the listener pauses for a
    /// second instead of stopping the compositor. Returns the error to log.
    fn ipc_accept_all(&mut self, listener: &UnixListener, socket: &Socket) -> Option<String> {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(err) = self.ipc_accept_as(stream, socket.clone()) {
                        break Some(err);
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => break None,
                Err(err) => {
                    let token = match socket {
                        Socket::Client(name) => self.ipc.clients.get(name).map(|s| s.ipc),
                        Socket::Admin => self.ipc.admin.as_ref().map(|(t, _)| *t),
                        Socket::Main => self.ipc.listener,
                    };
                    self.ipc_pause_listener(token);
                    break Some(format!("accepting fails, pausing for a second: {err}"));
                }
            }
        }
    }

    fn ipc_pause_listener(&mut self, token: Option<RegistrationToken>) {
        let Some(token) = token else {
            return;
        };
        self.loop_handle.disable(&token).ok();
        let timer = Timer::from_duration(Duration::from_secs(1));
        let resumed = self.loop_handle.insert_source(timer, move |_, (), state| {
            state.loop_handle.enable(&token).ok();
            TimeoutAction::Drop
        });
        if resumed.is_err() {
            self.loop_handle.enable(&token).ok();
        }
    }

    /// Registers a connection, or refuses it beyond the limits of all
    /// connections and of its process. A refused connection gets an error
    /// line before the close.
    #[cfg(test)]
    pub fn ipc_accept(&mut self, stream: UnixStream) -> Result<(), String> {
        self.ipc_accept_as(stream, Socket::Main)
    }

    /// Registers a connection of the admin socket, for tests.
    #[cfg(test)]
    pub fn ipc_accept_admin(&mut self, stream: UnixStream) -> Result<(), String> {
        self.ipc_accept_as(stream, Socket::Admin)
    }

    /// Registers a connection of the main socket, or with `fixed` of a
    /// client socket.
    fn ipc_accept_as(&mut self, stream: UnixStream, socket: Socket) -> Result<(), String> {
        fn failed(err: impl std::fmt::Display) -> String {
            format!("a connection failed: {err}")
        }
        stream.set_nonblocking(true).map_err(failed)?;
        let pid = socket_peercred(&stream).ok().map(|c| c.pid);
        if let Some(reason) = self.ipc.refusal(pid, socket.name()) {
            let error = RpcError::new(RpcError::LIMIT_REACHED, reason.clone());
            let _ = (&stream).write_all(&line(&Message::error(Value::Null, error)));
            // A close with unread bytes in the receive queue sends an RST
            // and eats the error line, so read what arrived before closing.
            let mut drain = [0; 4096];
            while matches!((&stream).read(&mut drain), Ok(n) if n > 0) {}
            return Err(format!("refusing a connection: {reason}"));
        }
        // The event loop watches a copy; reads go through `Conn::stream`.
        let watched = stream.try_clone().map_err(failed)?;
        let id = self.ipc.next;
        self.ipc.next += 1;
        let hello_by = self.time.now() + HELLO_TIMEOUT;
        self.expire_at(hello_by)?;
        let token = self
            .loop_handle
            .insert_source(
                Generic::new(watched, Interest::READ, Mode::Level),
                move |_, _, state| Ok(state.ipc_readable(id)),
            )
            .map_err(failed)?;
        let conn = Conn {
            stream,
            pid,
            input: Vec::new(),
            output: Vec::new(),
            overflow: false,
            too_long: false,
            name: None,
            hello_by,
            keep_nodes: false,
            resume: false,
            confined: false,
            socket,
            events: None,
            seq: 0,
            reserves: HashMap::new(),
            reader: Some(token),
        };
        self.ipc.conns.insert(id, conn);
        Ok(())
    }

    /// Reads at most `READ_PER_WAKEUP` bytes and handles the complete
    /// lines. After the end of the stream, the requests that arrived still
    /// get their answers, also from waiting transactions. A line longer than
    /// `MAX_LINE` is read to its end and dropped, so that the client can
    /// finish its write and read the error; then the connection closes.
    fn ipc_readable(&mut self, id: u64) -> PostAction {
        let open = self.ipc.conns.get_mut(&id).is_some_and(Conn::read);
        while let Some(line) = self
            .ipc
            .conns
            .get_mut(&id)
            .filter(|c| !c.too_long)
            .and_then(Conn::next_line)
        {
            self.ipc_handle_line(id, &line);
        }
        let Some(conn) = self.ipc.conns.get_mut(&id) else {
            return PostAction::Remove;
        };
        conn.too_long |= conn.input.len() > MAX_LINE;
        if conn.too_long {
            let ended = conn.input.contains(&b'\n');
            conn.input.clear();
            if open && !ended {
                return PostAction::Continue;
            }
        } else if open {
            return PostAction::Continue;
        }
        conn.input = Vec::new();
        conn.reader = None;
        if conn.too_long {
            let error = RpcError::new(
                RpcError::LIMIT_REACHED,
                format!(
                    "the request is longer than {MAX_LINE} bytes, the limit of a line; \
                     the core closes the connection"
                ),
            );
            self.ipc.respond(id, &Message::error(Value::Null, error));
        }
        PostAction::Remove
    }

    fn ipc_handle_line(&mut self, conn: u64, line: &[u8]) {
        let request: Request = match serde_json::from_slice(line) {
            Ok(r) => r,
            Err(err) => {
                let error = RpcError::new(-32700, format!("parse error: {err}"));
                self.ipc.respond(conn, &Message::error(Value::Null, error));
                return;
            }
        };
        let id = request.id.clone().unwrap_or(Value::Null);
        if let Some(result) = self.ipc_dispatch(conn, id.clone(), request) {
            let message = match result {
                Ok(value) => Message::result(id, value),
                Err(error) => Message::error(id, error),
            };
            self.ipc.respond(conn, &message);
        }
    }

    /// Runs one request. `None` means that the response comes later.
    fn ipc_dispatch(
        &mut self,
        conn: u64,
        id: Value,
        request: Request,
    ) -> Option<Result<Value, RpcError>> {
        let params = request.params;
        Some(match request.method.as_str() {
            "hello" => params_of(params).and_then(|h| self.ipc_hello(conn, h)),
            "scene.query" => params_of::<Query>(params).and_then(|q| self.query(conn, &q)),
            "scene.apply" => {
                let apply = match params_of::<Apply>(params) {
                    Ok(a) => a,
                    Err(e) => return Some(Err(e)),
                };
                let Some(owner) = self.ipc.name(conn).map(str::to_owned) else {
                    return Some(Err(rejected("send hello first")));
                };
                let checked = self.check_paths(conn, &apply.commands);
                if let Err(e) = checked.and_then(|()| self.confine(conn, &apply.commands)) {
                    return Some(Err(e));
                }
                self.submit_transaction(conn, id, owner, apply.commands);
                return None;
            }
            "events.subscribe" => {
                params_of::<Subscribe>(params).and_then(|s| self.ipc_subscribe(conn, s))
            }
            "events.emit" => params_of::<Emit>(params).and_then(|e| self.ipc_emit(conn, e)),
            "input.grab" => params_of::<Grab>(params).and_then(|g| self.ipc_grab(conn, g.binding)),
            "input.ungrab" => {
                params_of::<Grab>(params).and_then(|g| self.ipc_ungrab(conn, &g.binding))
            }
            "output.list" => Ok(serde_json::json!(self.output_info())),
            "output.reserve" => params_of(params).and_then(|r| self.ipc_reserve(conn, &r)),
            "output.configure" => {
                params_of(params).and_then(|c| self.ipc_output_configure(conn, &c))
            }
            "output.add" | "output.remove" => {
                self.ipc_headless_output(conn, &request.method, params)
            }
            "clients.list" => Ok(self.clients()),
            "methods.list" => Ok(self.list_methods()),
            "camera.release" => self.ipc_admin(conn, &request.method, params),
            "grants.set" | "grants.revoke" | "grants.list" | "clients.create" => {
                self.ipc_admin(conn, &request.method, params)
            }
            "screen.capture" => self.ipc_capture(conn, params),
            "input.inject" => self.ipc_inject(conn, params),
            "input.configure" => self.ipc_input_configure(conn, params),
            "assets.put" | "assets.remove" => self.ipc_assets(conn, &request.method, params),
            "activation.token" => self.ipc_activation_token(conn),
            "session.quit" => self.ipc_quit(conn),
            "methods.export" | "methods.call" | "methods.return" => {
                return self.ipc_methods(conn, id, &request.method, params);
            }
            other => Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("no method {other:?}"),
            )),
        })
    }

    /// A token for `choractl run` (D42): an xdg activation token of no
    /// client, so that the started program can activate with it. A use
    /// ends the token; an unused token of no client is forgotten after
    /// `MAX_TOKEN_AGE`, by its own deadline and by every new token. A
    /// client socket needs the capability `activation.token`, because
    /// such a token skips the focus rule of its own tokens.
    fn ipc_activation_token(&mut self, conn: u64) -> Result<Value, RpcError> {
        self.check_socket_grant(conn, Capability::ActivationToken)?;
        self.forget_unused_tokens(crate::protocols::MAX_TOKEN_AGE);
        let (token, _) = self.activation_state.create_external_token(None);
        let token = token.as_str().to_owned();
        let _ = self.expire_at(self.time.now() + crate::protocols::MAX_TOKEN_AGE);
        Ok(serde_json::json!({"token": token}))
    }

    /// Ends chora as Ctrl+Alt+Backspace does, after the answer. A client
    /// socket needs the capability `session.quit`.
    fn ipc_quit(&mut self, conn: u64) -> Result<Value, RpcError> {
        let name = self
            .ipc
            .name(conn)
            .ok_or_else(|| rejected("send hello first"))?;
        self.check_socket_grant(conn, Capability::SessionQuit)?;
        tracing::info!(client = name, "session.quit");
        self.loop_signal.stop();
        Ok(Value::Null)
    }

    /// Streams the named events, or all of them for an empty list. An
    /// unknown name is a mistake that answers, not silence. The answer is
    /// a snapshot of the scene as `scene.query` gives it (D32), so that no
    /// change between the snapshot and the first event is lost.
    fn ipc_subscribe(&mut self, conn: u64, subscription: Subscribe) -> Result<Value, RpcError> {
        if let Some(unknown) = subscription.events.iter().find(|n| !subscribable(n)) {
            return Err(rejected(format!("there is no event {unknown:?}")));
        }
        if let Some(c) = self.ipc.conns.get_mut(&conn) {
            c.events = Some(subscription.events);
        }
        self.query(conn, &Query::new(""))
    }

    /// Sends an event of the client to the connections that subscribed to
    /// its name or a pattern of it (D65). The name starts with the client
    /// name and a dot, so no client sends a core event or one of another
    /// name. The data has at most `MAX_EMIT` bytes, and a name emits at most
    /// `MAX_EMITS` events per second; the core refuses the rest with code 6.
    fn ipc_emit(&mut self, conn: u64, emit: Emit) -> Result<Value, RpcError> {
        let client = self
            .ipc
            .name(conn)
            .ok_or_else(|| rejected("send hello first"))?
            .to_owned();
        if core_namespace(&client) {
            return Err(rejected(format!(
                "the events of {client:?} would look like core events; \
                 emit as a client whose name starts otherwise"
            )));
        }
        let own = (emit.event.strip_prefix(client.as_str()))
            .and_then(|rest| rest.strip_prefix('.'))
            .is_some_and(chora_scene::ipc::valid_name);
        if !own {
            return Err(rejected(format!(
                "an event of {client:?} is named {client}.NAME, with NAME in lowercase \
                 letters, digits, _, - and ., from a letter, not {:?}",
                emit.event
            )));
        }
        let size = serde_json::to_vec(&emit.data).map_or(usize::MAX, |d| d.len());
        if size > MAX_EMIT {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("the data has {size} bytes, more than {MAX_EMIT}"),
            ));
        }
        let now = self.time.now();
        let emits = &mut self.ipc.emits;
        emits.retain(|_, (start, _)| now.duration_since(*start) < EMIT_WINDOW);
        let (_, count) = emits.entry(client.clone()).or_insert((now, 0));
        if *count >= MAX_EMITS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!(
                    "{client:?} emitted {MAX_EMITS} events in this second; the core drops \
                     this one, so send only the last state of a burst"
                ),
            ));
        }
        *count += 1;
        let event = Event::Emitted {
            event: emit.event,
            client,
            data: emit.data,
        };
        self.ipc.send_subscribed(None, &event, |_| None);
        Ok(Value::Null)
    }

    /// The methods of grants and client sockets. Only the admin socket of
    /// `choractl` changes them (D40); any connection lists the grants.
    fn ipc_admin(&mut self, conn: u64, method: &str, params: Value) -> Result<Value, RpcError> {
        if method != "grants.list" && !self.ipc.admin(conn) {
            return Err(RpcError::new(
                RpcError::ADMIN_SOCKET_ONLY,
                format!("{method} works only on the admin socket of choractl"),
            ));
        }
        if method == "clients.create" {
            return params_of(params).and_then(|c| self.ipc_client_socket(&c));
        }
        if method == "camera.release" {
            let output = params
                .get("output")
                .and_then(Value::as_str)
                .ok_or_else(|| rejected("camera.release wants {\"output\": NAME}"))?;
            return match self.scene.release_camera(output) {
                Some(owner) => {
                    tracing::warn!(output, %owner, "the admin released the camera");
                    Ok(serde_json::json!({"output": output, "was": owner}))
                }
                None => Err(rejected(format!("no client owns the camera of {output:?}"))),
            };
        }
        self.ipc_grants(method, params)
    }

    /// Names a connection. All connections of a name share one
    /// `keep_nodes` and one `resume` setting. With `resume`, a name in its
    /// grace time gets its nodes back; without, they go first.
    fn ipc_hello(&mut self, conn: u64, hello: Hello) -> Result<Value, RpcError> {
        check_protocol(hello.protocol)?;
        if self.ipc.name(conn).is_some() {
            return Err(rejected("hello was sent already"));
        }
        if !crate::methods::valid_name(&hello.name) {
            return Err(rejected(
                "a client name has lowercase letters, digits, _, - and ., from a letter",
            ));
        }
        // A client socket fixes the name (D33).
        match self.ipc.fixed(conn) {
            Some(fixed) if fixed != hello.name => {
                return Err(socket_rule(format!("this socket belongs to {fixed:?}")));
            }
            None if self.ipc.clients.contains_key(&hello.name) => {
                return Err(name_taken(format!(
                    "{:?} has a client socket and connects only through it",
                    hello.name
                )));
            }
            _ => {}
        }
        let peer = self
            .ipc
            .conns
            .values()
            .find(|c| c.name.as_deref() == Some(&hello.name));
        if let Some(peer) =
            peer.filter(|p| (p.keep_nodes, p.resume) != (hello.keep_nodes, hello.resume))
        {
            return Err(name_taken(format!(
                "{:?} is in use with keep_nodes {} and resume {}",
                hello.name, peer.keep_nodes, peer.resume
            )));
        }
        // Resume is for a name without a live connection: a second instance
        // must not take the nodes of a running one.
        if hello.resume && peer.is_some() {
            return Err(name_taken(format!(
                "{:?} is connected; resume needs a name without a live connection",
                hello.name
            )));
        }
        let known = peer.is_some();
        if hello.resume {
            self.ipc.graces.remove(&hello.name);
        } else if self.ipc.graces.contains_key(&hello.name) {
            self.drop_nodes(&hello.name);
        }
        if let Some(c) = self.ipc.conns.get_mut(&conn) {
            c.name = Some(hello.name.clone());
            (c.keep_nodes, c.resume) = (hello.keep_nodes, hello.resume);
            c.confined = hello.confined;
        }
        self.ipc.greeted = true;
        if !known {
            self.emit(&Event::ClientConnected { name: hello.name });
        }
        Ok(serde_json::json!({
            "version": chora_scene::ipc::VERSION,
            "protocol": chora_scene::ipc::PROTOCOL,
            "features": chora_scene::ipc::FEATURES,
        }))
    }

    /// Removes the nodes, the cameras and the assets of a client name, and
    /// ends its grace time.
    pub fn drop_nodes(&mut self, name: &str) {
        self.ipc.graces.remove(name);
        let effects = self.scene.disconnect(name);
        self.execute(effects);
        self.ipc.uploads.drop_client(name);
    }

    /// Ends every grace time and removes the kept nodes.
    pub fn end_graces(&mut self) {
        let names: Vec<String> = self.ipc.graces.keys().cloned().collect();
        for name in names {
            self.drop_nodes(&name);
        }
    }

    /// The nodes of a selector and a JSON filter (D32); for a client
    /// socket only its own.
    fn query(&self, conn: u64, query: &Query) -> Result<Value, RpcError> {
        let selector: Selector = query
            .selector
            .parse()
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, format!("{e}")))?;
        let fixed = self.ipc.fixed(conn);
        let mut nodes = self.scene.query(&selector);
        nodes.retain(|n| fixed.is_none_or(|f| clients::own_node(n, f)));
        // (D33): only the nodes that survived the filter can show a focus.
        let focus = [self.scene.focused_surface(), self.scene.focused_node()];
        let rects = self.screen_rects();
        if let Some(filter) = &query.filter {
            nodes.retain(|n| {
                filter.matches(
                    n,
                    focus.contains(&Some(n.id.as_str())),
                    rects.get(n.id.as_str()).unwrap_or(&Value::Null),
                )
            });
        }
        let nodes: Vec<Value> = nodes
            .into_iter()
            .map(|n| {
                let mut node = serde_json::json!(n);
                if focus.contains(&Some(n.id.as_str())) {
                    node["focused"] = Value::Bool(true);
                }
                node["rects"] = rects
                    .get(n.id.as_str())
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                node
            })
            .collect();
        Ok(Value::Array(nodes))
    }

    /// The screen rectangles by node: the output name maps to
    /// `[x, y, w, h]` in the logical pixels of that output, after the
    /// parent transforms and the camera of the output. Outputs where the
    /// node does not show are left out.
    fn screen_rects(&self) -> HashMap<&str, Value> {
        let mut found: HashMap<&str, serde_json::Map<String, Value>> = HashMap::new();
        for output in &self.outputs {
            let name = output.name();
            let screen = Rectangle::from_size(output_area(output).size).to_f64();
            // The overlay sits in the coordinates of its output, without
            // its camera (D43).
            let views = [
                (Transform::IDENTITY, self.scene.overlay_draw_list(&name)),
                (self.camera(output), self.scene.draw_list()),
            ];
            for (camera, list) in &views {
                // The absolute transform by node, for the boxes of the
                // layers in this list.
                let at: HashMap<&str, Transform> = list
                    .iter()
                    .map(|placed| (placed.node.id.as_str(), camera.then(&placed.transform)))
                    .collect();
                let shown = list.iter().filter_map(|placed| {
                    Some((
                        placed.node.id.as_str(),
                        self.screen_rect(placed, &at, screen)?,
                    ))
                });
                for (id, rect) in shown {
                    let rect = [rect.loc.x, rect.loc.y, rect.size.w, rect.size.h];
                    found
                        .entry(id)
                        .or_default()
                        .insert(name.clone(), serde_json::json!(rect));
                }
            }
        }
        found
            .into_iter()
            .map(|(id, rects)| (id, Value::Object(rects)))
            .collect()
    }

    /// The rectangle of one placed node on an output, as the renderer
    /// draws it there. A layer, and with a size also a shader, shows
    /// through its mesh, so its box is the box of the projected shape.
    /// Any other node needs a plain positive scale below its innermost
    /// ancestor layer, and every ancestor layer cuts its box like a clip
    /// does. The mesh of a layer does not move the boxes of descendants.
    fn screen_rect(
        &self,
        placed: &Placed<'_>,
        at: &HashMap<&str, Transform>,
        screen: Rectangle<f64, Logical>,
    ) -> Option<Rectangle<f64, Logical>> {
        let node = placed.node;
        let transform = at.get(node.id.as_str())?;
        let (below, layers) = self.lineage(node)?;
        let mut rect = if node.is_layer() {
            layer_box(node, transform)?
        } else {
            // A mirrored, rotated, collapsed or projected node draws
            // nothing outside a layer.
            location_and_scale(&below)?;
            self.content_box(node, transform)?
        };
        // Every ancestor layer cuts the box like a clip does.
        for ancestor in layers {
            rect = intersect(rect, layer_box(ancestor, at.get(ancestor.id.as_str())?)?)?;
        }
        rect.overlaps(screen).then_some(rect)
    }

    /// The node's transform below the innermost ancestor layer, and every
    /// ancestor layer. The renderer draws the node with that transform,
    /// also inside the texture of a layer, and the node shows only inside
    /// every layer: a layer composes into a texture of its size. A clip
    /// makes a layer and cuts through the same box.
    fn lineage(&self, node: &Node) -> Option<(Transform, Vec<&Node>)> {
        let mut below = node.props.transform;
        let mut layers = Vec::new();
        let mut parent = node.parent.as_deref();
        while let Some(id) = parent {
            let ancestor = self.scene.node(id)?;
            if ancestor.is_layer() {
                layers.push(ancestor);
            } else if layers.is_empty() {
                below = ancestor.props.transform.then(&below);
            }
            parent = ancestor.parent.as_deref();
        }
        Some((below, layers))
    }

    /// The box of a node that is not a layer itself.
    fn content_box(&self, node: &Node, transform: &Transform) -> Option<Rectangle<f64, Logical>> {
        match node.kind {
            // A surface is sized by its buffer, subsurfaces and decorations.
            NodeKind::Surface => Some(window_rect(self.windows.get(node.id.as_str())?, transform)),
            NodeKind::Rect => Some(bounding_box(transform, node.props.size?)),
            NodeKind::Image => {
                // Without the file the renderer draws nothing.
                let image = self.assets.images.get(node.props.image.as_deref()?)?;
                let [w, h] = node
                    .props
                    .size
                    .unwrap_or([f64::from(image.size[0]), f64::from(image.size[1])]);
                Some(bounding_box(transform, [w, h]))
            }
            NodeKind::Text => {
                // A text node reports its layout size, like an image
                // without a size.
                let size = self.assets.texts.size(node)?;
                Some(bounding_box(transform, size))
            }
            // A group without a layer of its own draws nothing, and
            // so does a shader without a size.
            NodeKind::Group | NodeKind::Shader => None,
        }
    }

    /// Grabs a binding of a key or a pointer button for a named connection.
    /// A binding needs Ctrl, Alt or Super, so that typing and clicking still
    /// work; `input.grab-any` lifts this (D45). Ctrl+Alt with the keys of
    /// the safety layer belongs to the safety layer either way. The result
    /// is the binding in the normal form of `input.key`.
    fn ipc_grab(&mut self, conn: u64, binding: Binding) -> Result<Value, RpcError> {
        let Some(name) = self.ipc.name(conn).map(str::to_owned) else {
            return Err(rejected("send hello first"));
        };
        let m = binding.mods;
        let trigger = trigger(&binding)?;
        let free = trigger == Trigger::Move || self.grants.allows(&name, Capability::GrabAny);
        if !(m.ctrl || m.alt || m.logo || free) {
            return Err(RpcError::new(
                RpcError::MISSING_CAPABILITY,
                format!("{binding} needs Ctrl, Alt or Super or the capability input.grab-any"),
            ));
        }
        if m.ctrl && m.alt && matches!(trigger, Trigger::Key(sym) if is_safety_key(sym)) {
            return Err(rejected(format!("{binding} belongs to the safety layer")));
        }
        // A button binding may have several grabs while each client holds a
        // claim; a press then goes by the claim under the pointer (D55).
        // Only on the main socket: a confined client could claim a little
        // less than an output and so take every press of the binding.
        let claims = |c: u64| {
            self.ipc.fixed(c).is_none()
                && (self.ipc.name(c)).is_some_and(|n| self.scene.holds_claim(n))
        };
        // Each window manager gets the moves of its own windows.
        let own = trigger == Trigger::Move;
        let mut holders = (self.ipc.grabs.iter())
            .filter(|(b, t, c)| b.mods == binding.mods && *t == trigger && (!own || *c == conn))
            .map(|(_, _, c)| *c)
            .peekable();
        let shared = matches!(trigger, Trigger::Button(_)) && claims(conn);
        if holders.peek().is_some() && !(shared && holders.all(|c| c != conn && claims(c))) {
            return Err(rejected(format!("{binding} is grabbed already")));
        }
        let normal = binding.to_string();
        self.ipc.grabs.push((binding, trigger, conn));
        Ok(Value::String(normal))
    }

    /// Ends a grab of a connection (D45).
    fn ipc_ungrab(&mut self, conn: u64, binding: &Binding) -> Result<Value, RpcError> {
        let trigger = trigger(binding)?;
        let before = self.ipc.grabs.len();
        self.ipc
            .grabs
            .retain(|(b, t, c)| !(*c == conn && b.mods == binding.mods && *t == trigger));
        if self.ipc.grabs.len() == before {
            return Err(rejected(format!("{binding} is no grab of this connection")));
        }
        Ok(Value::Null)
    }

    /// Closes a connection. Without `keep_nodes`, the nodes and the assets
    /// of its name go when the last connection of the name closes; with
    /// `resume`, they go after the grace time.
    pub fn ipc_close(&mut self, conn: u64) {
        let Some(closed) = self.ipc.conns.remove(&conn) else {
            return;
        };
        // The client sees the end at once, also while a child that the core
        // forks holds a copy of the socket until its exec.
        let _ = closed.stream.shutdown(std::net::Shutdown::Both);
        if let Some(reader) = closed.reader {
            self.loop_handle.remove(reader);
        }
        self.ipc.grabs.retain(|(_, _, c)| *c != conn);
        if !closed.reserves.is_empty() {
            self.output_changed();
        }
        self.methods_closed(conn);
        let held = self.transactions.drop_conn(conn);
        self.release(&held);
        // A dropped head lets the next transaction start.
        self.complete_transactions();
        let Some(name) = closed.name else { return };
        if self
            .ipc
            .conns
            .values()
            .any(|c| c.name.as_ref() == Some(&name))
        {
            return;
        }
        if self.scene.focused_node_owner() == Some(name.as_str()) {
            self.scene.unfocus_node();
            self.update_focus();
        }
        if closed.resume && !closed.keep_nodes {
            let end = self.time.now() + GRACE;
            let _ = self.expire_at(end);
            self.ipc.graces.insert(name.clone(), end);
        } else if !closed.keep_nodes {
            self.drop_nodes(&name);
        } else {
            // Kept nodes stay, but a surface reserved for the name is free.
            let effects = self.scene.unreserve(&name);
            self.execute(effects);
        }
        self.emit(&Event::ClientDisconnected { name });
    }

    /// Writes all pending IPC output. Closes the connections that failed or
    /// overflowed, and those without a reader once their answers are out. A
    /// close can send events to the other connections, so this repeats until
    /// no connection closes.
    pub fn ipc_flush(&mut self) {
        loop {
            let closing = self.ipc_write();
            if closing.is_empty() {
                return;
            }
            for conn in closing {
                self.ipc_close(conn);
            }
        }
    }

    /// Writes the output of every connection and returns those to close.
    fn ipc_write(&mut self) -> Vec<u64> {
        let transactions = &self.transactions;
        let methods = &self.methods;
        self.ipc
            .conns
            .iter_mut()
            .filter_map(|(id, conn)| {
                let broken = !conn.flush();
                if broken && conn.overflow {
                    tracing::warn!(
                        id,
                        name = ?conn.name,
                        "the output passed {} MiB; the connection closes",
                        MAX_OUTPUT >> 20,
                    );
                }
                let answered = conn.reader.is_none()
                    && conn.output.is_empty()
                    && !transactions.waits_for(*id)
                    && !methods.waits_for(*id);
                (broken || answered).then_some(*id)
            })
            .collect()
    }
}

pub fn params_of<T: DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    let params = if params.is_null() {
        Value::Object(serde_json::Map::new())
    } else {
        params
    };
    serde_json::from_value(params)
        .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))
}

/// Refuses a client whose protocol is newer than the core's (D29); a
/// lower or missing protocol works.
pub fn check_protocol(protocol: Option<u64>) -> Result<(), RpcError> {
    if let Some(protocol) = protocol.filter(|p| *p > chora_scene::ipc::PROTOCOL) {
        return Err(RpcError::new(
            RpcError::UNSUPPORTED_PROTOCOL,
            format!(
                "client protocol {protocol} is newer than the core protocol {}",
                chora_scene::ipc::PROTOCOL
            ),
        ));
    }
    Ok(())
}

pub fn rejected(message: impl Into<String>) -> RpcError {
    RpcError::new(RpcError::REJECTED, message)
}

fn name_taken(message: impl Into<String>) -> RpcError {
    RpcError::new(RpcError::NAME_TAKEN, message)
}

/// A refusal by a rule of client sockets (D33).
pub fn socket_rule(message: impl Into<String>) -> RpcError {
    RpcError::new(RpcError::CLIENT_SOCKET_RULE, message)
}

/// The box of a layer node on the screen: the points of its mesh, or the
/// corners of its size, through the full transform of the renderer.
fn layer_box(node: &Node, transform: &Transform) -> Option<Rectangle<f64, Logical>> {
    let size = node.props.size?;
    let Some(mesh) = &node.props.mesh else {
        return Some(bounding_box(transform, size));
    };
    Some(projected_box(transform, mesh.points.iter().copied()))
}

/// The largest rectangle inside both, or `None` when they miss each other.
fn intersect(
    a: Rectangle<f64, Logical>,
    b: Rectangle<f64, Logical>,
) -> Option<Rectangle<f64, Logical>> {
    let loc = (a.loc.x.max(b.loc.x), a.loc.y.max(b.loc.y));
    let end = (
        (a.loc.x + a.size.w).min(b.loc.x + b.size.w),
        (a.loc.y + a.size.h).min(b.loc.y + b.size.h),
    );
    (loc.0 < end.0 && loc.1 < end.1)
        .then(|| Rectangle::new(loc.into(), (end.0 - loc.0, end.1 - loc.1).into()))
}

/// The smallest axis-aligned rectangle that holds the box from `(0, 0)` to
/// `(w, h)` after the transform.
fn bounding_box(transform: &Transform, [w, h]: [f64; 2]) -> Rectangle<f64, Logical> {
    projected_box(
        transform,
        [[0.0, 0.0], [w, 0.0], [0.0, h], [w, h]].into_iter(),
    )
}

/// The smallest axis-aligned rectangle that holds the points after the
/// transform. The points go through the projection of the renderer, so
/// that a matrix which divides by w moves them as the frame does; the
/// depth of mesh points stays out, as in `Transform::apply`. A point at
/// or behind the eye is left out, and a box of no point in front overlaps
/// nothing.
fn projected_box(
    transform: &Transform,
    points: impl Iterator<Item = [f64; 2]>,
) -> Rectangle<f64, Logical> {
    let m = transform.matrix();
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for [px, py] in points {
        // The least w of a point that shows, as in the renderer.
        let hw = m[3] * px + m[7] * py + m[15];
        if hw <= 1e-6 {
            continue;
        }
        let (x, y) = (
            (m[0] * px + m[4] * py + m[12]) / hw,
            (m[1] * px + m[5] * py + m[13]) / hw,
        );
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    Rectangle::new((x0, y0).into(), (x1 - x0, y1 - y0).into())
}

/// Whether Ctrl+Alt with a key is a binding of the safety layer (D23):
/// `BackSpace` quits, `Escape` resets, `i` revokes the agents, `F1` to
/// `F12` switch the VT.
fn is_safety_key(sym: xkb::Keysym) -> bool {
    use xkb::Keysym;
    sym == Keysym::BackSpace
        || sym == Keysym::Escape
        || sym == Keysym::i
        || sym == Keysym::I
        || (Keysym::F1.raw()..=Keysym::F12.raw()).contains(&sym.raw())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use std::os::unix::fs::PermissionsExt;

    use crate::testing::{Client, Core, rect, wayland::Wayland};

    const REJECTED: i64 = RpcError::REJECTED;

    fn ids(client: &mut Client, core: &mut Core, selector: &str) -> Vec<String> {
        let nodes = client.ok(core, "scene.query", json!({"selector": selector}));
        let mut ids: Vec<String> = nodes
            .as_array()
            .expect("a list")
            .iter()
            .map(|n| n["id"].as_str().expect("an id").to_owned())
            .collect();
        ids.sort();
        ids
    }

    /// One node of a query by id, or null.
    fn query_node(client: &mut Client, core: &mut Core, id: &str) -> Value {
        let nodes = client.ok(core, "scene.query", json!({"selector": format!("id:{id}")}));
        nodes
            .as_array()
            .expect("a list")
            .first()
            .cloned()
            .unwrap_or(Value::Null)
    }

    #[test]
    fn query_reports_the_screen_rect_of_a_node() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "here", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [1, 0, 0, 1],
                          "transform": {"translate": [100, 50], "scale": 2}}}},
            {"create": {"id": "far", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [0, 1, 0, 1],
                          "transform": {"translate": [5000, 0]}}}},
            {"camera": {"output": "out", "transform": {"translate": [-30, -20]}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        // The 10x10 rect scales to 20x20 at (100, 50) and the pan moves it.
        assert_eq!(
            query_node(&mut wm, &mut core, "here")["rects"],
            json!({"out": [70.0, 30.0, 20.0, 20.0]}),
            "the rect after its transform and the camera"
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "far")["rects"],
            json!({}),
            "an output without the node is left out"
        );
    }

    #[test]
    fn query_cuts_the_rect_of_a_node_inside_a_clip() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "frame", "kind": "group", "parent": "root",
                "props": {"size": [100, 100], "clip": {},
                          "transform": {"translate": [20, 30]}}}},
            {"create": {"id": "inside", "kind": "rect", "parent": "frame",
                "props": {"size": [10, 10], "color": [0, 1, 0, 1],
                          "transform": {"translate": [10, 10]}}}},
            {"create": {"id": "half", "kind": "rect", "parent": "frame",
                "props": {"size": [20, 10], "color": [0, 0, 1, 1],
                          "transform": {"translate": [90, 10]}}}},
            {"create": {"id": "gone", "kind": "rect", "parent": "frame",
                "props": {"size": [10, 10], "color": [1, 1, 0, 1],
                          "transform": {"translate": [200, 0]}}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        assert_eq!(
            query_node(&mut wm, &mut core, "inside")["rects"],
            json!({"out": [30.0, 40.0, 10.0, 10.0]}),
            "a node inside the clip keeps its box"
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "half")["rects"],
            json!({"out": [110.0, 40.0, 10.0, 10.0]}),
            "a node over the clip edge is cut to it"
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "gone")["rects"],
            json!({}),
            "a node clipped away shows on no output"
        );
    }

    #[test]
    fn query_gives_the_box_of_the_projected_shape_for_a_mesh_or_perspective() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let mut matrix = [0.0; 16];
        for i in [0, 5, 10, 15] {
            matrix[i] = 1.0;
        }
        // The w row divides x by 1 + 0.01 * x, so x = 100 lands at 50.
        matrix[3] = 0.01;
        let setup = json!({"commands": [
            {"create": {"id": "wobbly", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [1, 0, 0, 1],
                          "transform": {"translate": [5, 5]},
                          "mesh": {"cols": 1, "rows": 1,
                                   "points": [[0, 0], [30, 0], [0, 10], [30, 10]]}}}},
            {"create": {"id": "deep", "kind": "rect", "parent": "root",
                "props": {"size": [100, 50], "color": [0, 1, 0, 1],
                          "transform": {"matrix": matrix}}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        assert_eq!(
            query_node(&mut wm, &mut core, "wobbly")["rects"],
            json!({"out": [5.0, 5.0, 30.0, 10.0]}),
            "the mesh stretches the drawn shape to x 30"
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "deep")["rects"],
            json!({"out": [0.0, 0.0, 50.0, 50.0]}),
            "the perspective moves the right edge in"
        );
    }

    #[test]
    fn query_leaves_out_mirrored_rotated_and_collapsed_nodes() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        wayland.toplevel(&mut core, "win");
        let mut wm = core.named("wm");
        let path = png("mirror", 8);
        let sent = wm.send(
            "scene.apply",
            json!({"commands": [
                {"attach": {"surface": "s1", "parent": "root"}},
                {"update": {"id": "s1", "props": {
                    "transform": {"translate": [400, 0], "scale": [-1, 1]}}}},
                // An image without a size, so no layer: the renderer
                // draws it only under a plain positive scale.
                {"create": {"id": "turn", "kind": "image", "parent": "root",
                    "props": {"image": &path,
                              "transform": {"translate": [10, 10], "rotate": 30}}}},
                {"create": {"id": "flat", "kind": "rect", "parent": "root",
                    "props": {"size": [50, 50], "transform": {"scale": [1, 0]}}}},
            ]}),
        );
        // The answer waits for the worker that decodes the file.
        for _ in 0..1000 {
            if wm.answer(&mut core, sent).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        for id in ["s1", "turn", "flat"] {
            assert_eq!(
                query_node(&mut wm, &mut core, id)["rects"],
                json!({}),
                "{id} draws nothing, so it shows on no output"
            );
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn query_reports_the_box_of_a_shader_node_with_a_size() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        // A shader with a size is a layer and draws wherever it stands,
        // also outside of any other layer.
        let setup = json!({"commands": [
            {"create": {"id": "glow", "kind": "shader", "parent": "root",
                "props": {"size": [40, 20], "shader": "/glow.glsl",
                          "transform": {"translate": [60, 70]}}}},
            {"create": {"id": "bare", "kind": "shader", "parent": "root",
                "props": {"shader": "/bare.glsl"}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        assert_eq!(
            query_node(&mut wm, &mut core, "glow")["rects"],
            json!({"out": [60.0, 70.0, 40.0, 20.0]}),
            "the shader draws as a layer and gets its box"
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "bare")["rects"],
            json!({}),
            "a shader without a size draws nothing"
        );
    }

    #[test]
    fn query_cuts_nodes_to_their_ancestor_layer() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        // The layer composes into a texture of its size: the mesh stretches
        // it to x 120, and nodes outside the size are drawn nowhere. The
        // mesh does not move the box of the node inside.
        let setup = json!({"commands": [
            {"create": {"id": "mat", "kind": "group", "parent": "root",
                "props": {"size": [100, 100], "mesh": {"cols": 1, "rows": 1,
                         "points": [[0, 0], [120, 0], [0, 100], [120, 100]]}}}},
            {"create": {"id": "on", "kind": "rect", "parent": "mat",
                "props": {"size": [10, 10], "transform": {"translate": [10, 10]}}}},
            {"create": {"id": "away", "kind": "rect", "parent": "mat",
                "props": {"size": [10, 10], "transform": {"translate": [200, 0]}}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        assert_eq!(
            query_node(&mut wm, &mut core, "on")["rects"],
            json!({"out": [10.0, 10.0, 10.0, 10.0]}),
            "a node inside the layer keeps its box"
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "away")["rects"],
            json!({}),
            "a node outside the size of the layer is drawn nowhere"
        );
    }

    #[test]
    fn query_cuts_a_clip_under_a_perspective_matrix_like_the_renderer() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let mut matrix = [0.0; 16];
        for i in [0, 5, 10, 15] {
            matrix[i] = 1.0;
        }
        // The w row divides x by 1 + 0.1 * x, so x 50 lands at 8.33.
        matrix[3] = 0.1;
        let setup = json!({"commands": [
            {"create": {"id": "win", "kind": "group", "parent": "root",
                "props": {"size": [50, 50], "clip": {},
                          "transform": {"matrix": matrix}}}},
            {"create": {"id": "bit", "kind": "rect", "parent": "win",
                "props": {"size": [10, 10], "transform": {"translate": [40, 0]}}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        let got = query_node(&mut wm, &mut core, "bit")["rects"]["out"]
            .as_array()
            .expect("a rect")
            .iter()
            .map(Value::as_f64)
            .collect::<Option<Vec<f64>>>()
            .expect("six numbers");
        // The renderer draws x 40 to 50 at 8.0 to 8.33; the affine box
        // would say 40.0 to 50.0.
        let near = |got: f64, want: f64| (got - want).abs() < 1e-9;
        assert!(
            near(got[0], 8.0) && near(got[1], 0.0) && near(got[2], 1.0 / 3.0) && near(got[3], 2.0),
            "the clip cuts at the projected edge, not the affine one: {got:?}"
        );
    }

    #[test]
    fn query_leaves_out_an_image_without_its_asset() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let path = png("asset", 4);
        let sent = wm.send(
            "scene.apply",
            json!({"commands": [
                {"create": {"id": "pic", "kind": "image", "parent": "root",
                    "props": {"size": [10, 10], "image": &path}}},
            ]}),
        );
        // The answer waits for the worker that decodes the file.
        for _ in 0..1000 {
            if wm.answer(&mut core, sent).is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            query_node(&mut wm, &mut core, "pic")["rects"],
            json!({"out": [0.0, 0.0, 10.0, 10.0]}),
            "an image with its asset shows"
        );
        core.state.assets.images.forget(&path);
        assert_eq!(
            query_node(&mut wm, &mut core, "pic")["rects"],
            json!({}),
            "without the asset the renderer draws nothing"
        );
        std::fs::remove_file(&path).unwrap();
    }

    /// Writes a gray PNG file of a size and returns its path.
    fn png(name: &str, side: u32) -> String {
        let path =
            std::env::temp_dir().join(format!("chora-ipc-{name}-{}.png", std::process::id()));
        let file = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
        let mut encoder = png::Encoder::new(file, side, side);
        encoder.set_color(png::ColorType::Grayscale);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&vec![128; (side * side) as usize])
            .unwrap();
        path.to_str().unwrap().to_owned()
    }

    #[test]
    fn query_marks_the_focused_surface_and_the_focused_node() {
        let mut core = Core::new();
        let mut wayland = Wayland::connect(&mut core);
        wayland.toplevel(&mut core, "foot");
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "menu", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10]}}},
            {"attach": {"surface": "s1", "parent": "root"}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        assert!(
            query_node(&mut wm, &mut core, "s1")
                .get("focused")
                .is_none(),
            "nothing focused yet"
        );
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"focus": {"surface": "s1"}}]}),
        );
        assert_eq!(
            query_node(&mut wm, &mut core, "s1")["focused"],
            json!(true),
            "the focused surface"
        );
        let focus_node = json!({"commands": [{"focus_node": {"node": "menu"}}]});
        wm.ok(&mut core, "scene.apply", focus_node);
        assert_eq!(
            query_node(&mut wm, &mut core, "menu")["focused"],
            json!(true),
            "the focused node"
        );
        wayland.toplevel(&mut core, "edit");
        let take = json!({"commands": [
            {"attach": {"surface": "s2", "parent": "root"}},
            {"focus": {"surface": "s2"}},
        ]});
        wm.ok(&mut core, "scene.apply", take);
        assert!(
            query_node(&mut wm, &mut core, "menu")
                .get("focused")
                .is_none(),
            "a new surface focus ends the node focus"
        );
        assert_eq!(query_node(&mut wm, &mut core, "s2")["focused"], json!(true));
    }

    /// The sorted ids of a `scene.query` with the given parameters.
    fn query_ids(client: &mut Client, core: &mut Core, params: Value) -> Vec<String> {
        let mut ids: Vec<String> = client
            .ok(core, "scene.query", params)
            .as_array()
            .expect("a list")
            .iter()
            .map(|n| n["id"].as_str().expect("an id").to_owned())
            .collect();
        ids.sort();
        ids
    }

    /// Two outputs, a foot surface and a rect owned by "wm", a rect owned
    /// by "other" that only shows on "hdmi", and an invisible rect.
    struct Fixture {
        core: Core,
        _wayland: Wayland,
        _other: Client,
        wm: Client,
    }

    fn fixture() -> Fixture {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        core.add_output("hdmi", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        wayland.toplevel(&mut core, "foot");
        let mut wm = core.named("wm");
        let mut other = core.named("other");
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"create": {"id": "here", "kind": "rect", "parent": "root",
                    "props": {"size": [10, 10]}}},
                {"create": {"id": "hidden", "kind": "rect", "parent": "root",
                    "props": {"size": [10, 10], "visible": false}}},
                {"attach": {"surface": "s1", "parent": "root"}},
                {"focus": {"surface": "s1"}},
            ]}),
        );
        other.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"create": {"id": "theirs", "kind": "group", "parent": "root"}},
                {"create": {"id": "far", "kind": "rect", "parent": "root",
                    "props": {"size": [10, 10], "transform": {"translate": [900, 0]}}}},
            ]}),
        );
        Fixture {
            core,
            _wayland: wayland,
            _other: other,
            wm,
        }
    }

    /// The ids a filter of the query on the fixture.
    fn filtered(f: &mut Fixture, filter: &Value) -> Vec<String> {
        query_ids(&mut f.wm, &mut f.core, json!({"filter": filter}))
    }

    #[test]
    fn query_filters_by_owner_kind_app_id_and_focus() {
        let mut f = fixture();
        assert_eq!(
            filtered(&mut f, &json!({"owner": "wm"})),
            ["here", "hidden", "s1"],
            "only the nodes of the owner, root has no owner"
        );
        assert_eq!(
            filtered(&mut f, &json!({"owner": "nobody"})),
            Vec::<String>::new(),
            "no node has the other owner"
        );
        assert_eq!(
            filtered(&mut f, &json!({"kind": "rect"})),
            ["far", "here", "hidden"],
            "only rects"
        );
        assert_eq!(
            filtered(&mut f, &json!({"app_id": "foot"})),
            ["s1"],
            "only the surface of that application"
        );
        assert_eq!(
            filtered(&mut f, &json!({"focused": true})),
            ["s1"],
            "only the focused surface"
        );
        assert_eq!(
            filtered(
                &mut f,
                &json!({"owner": "wm", "app_id": "foot", "focused": true})
            ),
            ["s1"],
            "all fields of a combined filter must match"
        );
    }

    #[test]
    fn query_filters_by_visibility_and_output() {
        let mut f = fixture();
        assert_eq!(
            query_ids(
                &mut f.wm,
                &mut f.core,
                json!({"selector": "*", "filter": {}})
            ),
            query_ids(&mut f.wm, &mut f.core, json!({"selector": "*"})),
            "an empty filter matches everything"
        );
        let visible = filtered(&mut f, &json!({"visible": true}));
        assert!(
            visible.contains(&"here".to_owned()) && visible.contains(&"far".to_owned()),
            "shown nodes are visible"
        );
        assert!(
            !visible.contains(&"hidden".to_owned()),
            "an invisible node shows on no output"
        );
        assert_eq!(
            filtered(&mut f, &json!({"output": "hdmi"})),
            ["far"],
            "only the node that shows on the second output"
        );
        assert_eq!(
            query_ids(
                &mut f.wm,
                &mut f.core,
                json!({"selector": "kind:rect", "filter": {"output": "out"}})
            ),
            ["here"],
            "selector and filter must both match"
        );
        let (error_code, _) = f.wm.fails(&mut f.core, "scene.query", json!({"filter": 3}));
        assert_eq!(
            error_code,
            RpcError::INVALID_PARAMS,
            "a filter that is no object fails"
        );
    }

    #[test]
    fn a_client_socket_sees_focused_only_on_its_own_nodes() {
        let (mut core, dir) = socket_dir("focused");
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let ipc = PathBuf::from(created["path"].as_str().unwrap());
        let wayland = PathBuf::from(created["wayland"].as_str().unwrap());
        let mut agent = core.connect_to(&ipc);
        agent.ok(&mut core, "hello", json!({"name": "agent"}));
        let mut windows = Wayland::connect_to(&mut core, &wayland);
        windows.toplevel(&mut core, "app");
        core.state.pressed = Some("agent".into());
        let own = json!({"commands": [
            {"attach": {"surface": "s1", "parent": "root"}},
            {"focus": {"surface": "s1"}},
        ]});
        agent.ok(&mut core, "scene.apply", own);
        let mut wm = core.named("wm");
        let foreign = json!({"commands": [
            {"create": {"id": "hidden", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10]}}},
            {"focus_node": {"node": "hidden"}},
        ]});
        wm.ok(&mut core, "scene.apply", foreign);
        let nodes = agent.ok(&mut core, "scene.query", json!({"selector": "*"}));
        let list = nodes.as_array().expect("a list");
        assert_eq!(list.len(), 1, "{list:?}");
        assert_eq!(list[0]["id"], json!("s1"), "only its own window");
        assert_eq!(list[0]["focused"], json!(true));
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The main socket takes the same names as the sockets of clients.
    #[test]
    fn hello_wants_a_name_of_letters_digits_and_dashes() {
        let mut core = Core::new();
        let mut client = core.connect();
        for name in ["", "Bad Name", "../x", "a b", "Ünicode"] {
            let (_, message) = client.fails(&mut core, "hello", json!({"name": name}));
            assert!(message.contains("lowercase letters"), "{name:?}: {message}");
        }
        client.ok(&mut core, "hello", json!({"name": "atk"}));
    }

    #[test]
    fn hello_names_a_connection_once() {
        let mut core = Core::new();
        let mut client = core.connect();
        client.ok(&mut core, "hello", json!({"name": "wm"}));
        assert_eq!(core.state.ipc.name(1), None);
        assert_eq!(core.state.ipc.name(0), Some("wm"));
        let (error_code, message) = client.fails(&mut core, "hello", json!({"name": "other"}));
        assert_eq!(
            (error_code, message.as_str()),
            (REJECTED, "hello was sent already")
        );
    }

    /// The full answer of the core, and a missing `protocol` means a
    /// client from before the versioning (D29).
    #[test]
    fn hello_without_a_protocol_gets_the_full_answer() {
        let mut core = Core::new();
        let mut client = core.connect();
        let answer = client.ok(&mut core, "hello", json!({"name": "wm"}));
        assert_eq!(
            answer,
            json!({
                "version": chora_scene::ipc::VERSION,
                "protocol": chora_scene::ipc::PROTOCOL,
                "features": [],
            })
        );
    }

    #[test]
    fn hello_with_an_equal_protocol_answers_with_the_core_own_values() {
        let mut core = Core::new();
        let mut client = core.connect();
        let answer = client.ok(
            &mut core,
            "hello",
            json!({"name": "wm", "protocol": 1, "features": ["drag"]}),
        );
        assert_eq!(answer["protocol"], json!(chora_scene::ipc::PROTOCOL));
        assert_eq!(answer["features"], json!([]), "the core tells its own");
    }

    #[test]
    fn hello_with_a_lower_protocol_works() {
        let mut core = Core::new();
        let mut client = core.connect();
        client.ok(&mut core, "hello", json!({"name": "wm", "protocol": 0}));
    }

    #[test]
    fn hello_with_a_higher_protocol_fails_with_the_protocol_code() {
        let mut core = Core::new();
        let mut client = core.connect();
        let (error_code, message) =
            client.fails(&mut core, "hello", json!({"name": "wm", "protocol": 2}));
        assert_eq!(error_code, RpcError::UNSUPPORTED_PROTOCOL);
        assert!(
            message.contains("client protocol 2") && message.contains("core protocol 1"),
            "{message}"
        );
        // Beyond u32 the answer is still the protocol code.
        let (error_code, _) = client.fails(
            &mut core,
            "hello",
            json!({"name": "wm", "protocol": 1u64 << 40}),
        );
        assert_eq!(error_code, RpcError::UNSUPPORTED_PROTOCOL);
    }

    #[test]
    fn requests_that_need_a_name_wait_for_hello() {
        let mut core = Core::new();
        let mut client = core.connect();
        for (method, params) in [
            ("scene.apply", rect("r")),
            ("input.grab", json!({"binding": "Super+a"})),
            ("methods.export", json!({"names": ["go"]})),
            ("input.inject", json!({"actions": []})),
        ] {
            let (error_code, message) = client.fails(&mut core, method, params);
            assert_eq!(
                (error_code, message.as_str()),
                (REJECTED, "send hello first"),
                "{method}"
            );
        }
    }

    #[test]
    fn a_line_that_is_no_request_gets_a_parse_error() {
        let mut core = Core::new();
        let mut client = core.connect();
        client.send_line(b"{nope");
        core.dispatch();
        let responses = client.responses();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0]["id"], Value::Null);
        assert_eq!(responses[0]["error"]["code"], -32700);
        assert!(
            !client.closed(&mut core),
            "a parse error keeps the connection"
        );
    }

    #[test]
    fn an_unknown_method_and_wrong_params_fail_with_their_codes() {
        let mut core = Core::new();
        let mut client = core.named("wm");
        let (error_code, _) = client.fails(&mut core, "scene.drop", json!({}));
        assert_eq!(error_code, RpcError::METHOD_NOT_FOUND);
        let (error_code, _) = client.fails(&mut core, "scene.query", json!({"selector": 3}));
        assert_eq!(error_code, RpcError::INVALID_PARAMS);
        let (error_code, _) =
            client.fails(&mut core, "scene.query", json!({"selector": "*", "x": 1}));
        assert_eq!(error_code, RpcError::INVALID_PARAMS, "unknown fields fail");
        let (error_code, _) = client.fails(&mut core, "scene.query", json!({"selector": "size:3"}));
        assert_eq!(
            error_code,
            RpcError::INVALID_PARAMS,
            "a wrong selector fails"
        );
    }

    #[test]
    fn the_connections_of_a_name_share_keep_nodes() {
        let mut core = Core::new();
        let mut first = core.connect();
        first.ok(
            &mut core,
            "hello",
            json!({"name": "wm", "keep_nodes": true}),
        );
        let mut second = core.connect();
        let (error_code, message) = second.fails(&mut core, "hello", json!({"name": "wm"}));
        assert_eq!(error_code, RpcError::NAME_TAKEN);
        assert_eq!(
            message,
            "\"wm\" is in use with keep_nodes true and resume false"
        );
        second.ok(
            &mut core,
            "hello",
            json!({"name": "wm", "keep_nodes": true}),
        );
    }

    /// Gate check 9 of 0.7: each error code has a test that provokes it.
    /// Code 2 has `a_gone_surface_has_its_own_code` in `transaction.rs`,
    /// code 3 `hello_with_a_higher_protocol_fails_with_the_protocol_code`
    /// above.
    #[test]
    fn a_method_without_its_grant_answers_missing_capability() {
        let mut core = Core::new();
        let mut agent = core.named("agent");
        let (error_code, _) = agent.fails(&mut core, "screen.capture", json!({}));
        assert_eq!(error_code, RpcError::MISSING_CAPABILITY);
    }

    #[test]
    fn a_command_for_a_missing_node_answers_unknown_node() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let error = wm
            .call(
                &mut core,
                "scene.apply",
                json!({"commands": [{"update": {"id": "n9", "props": {"visible": false}}}]}),
            )
            .expect_err("the node does not exist");
        assert_eq!(error["code"], RpcError::UNKNOWN_NODE);
        assert_eq!(error["data"]["command"], 0, "data names the command");
    }

    #[test]
    fn a_second_hello_of_a_live_name_answers_name_taken() {
        let mut core = Core::new();
        let _wm = core.named("wm");
        let mut second = core.connect();
        let (error_code, _) =
            second.fails(&mut core, "hello", json!({"name": "wm", "resume": true}));
        assert_eq!(error_code, RpcError::NAME_TAKEN);
    }

    #[test]
    fn an_admin_method_off_the_admin_socket_answers_its_code() {
        let mut core = Core::new();
        let mut agent = core.named("agent");
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        let (error_code, message) = agent.fails(&mut core, "grants.set", grant);
        assert_eq!(error_code, RpcError::ADMIN_SOCKET_ONLY);
        assert!(message.contains("only on the admin socket"), "{message}");
    }

    /// A client socket gets an activation token over IPC only with the
    /// capability: such a token belongs to no client and skips the focus
    /// rule of its own tokens.
    #[test]
    fn a_client_socket_gets_activation_tokens_only_with_the_capability() {
        let (mut core, dir) = socket_dir("token");
        let mut admin = core.admin();
        let (mut agent, _) = fixed(&mut core, &mut admin, "agent");
        let (error_code, _) = agent.fails(&mut core, "activation.token", Value::Null);
        assert_eq!(error_code, RpcError::MISSING_CAPABILITY);
        let create = json!({"name": "runner", "capabilities": ["activation.token"]});
        admin.ok(&mut core, "clients.create", create);
        let path = dir.join("clients/runner/ipc.sock");
        let mut runner = core.connect_to(&path);
        runner.ok(&mut core, "hello", json!({"name": "runner"}));
        let answer = runner.ok(&mut core, "activation.token", Value::Null);
        assert!(answer["token"].is_string(), "{answer}");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `session.quit` ends the main loop after its answer; a client socket
    /// needs the capability, so an agent cannot end the session.
    #[test]
    fn session_quit_stops_the_loop_and_needs_the_capability_on_a_client_socket() {
        let (mut core, dir) = socket_dir("quit");
        let mut admin = core.admin();
        let (mut agent, _) = fixed(&mut core, &mut admin, "agent");
        let (error_code, _) = agent.fails(&mut core, "session.quit", Value::Null);
        assert_eq!(error_code, RpcError::MISSING_CAPABILITY);
        let mut plugin = core.connect();
        let (error_code, _) = plugin.fails(&mut core, "session.quit", Value::Null);
        assert_eq!(error_code, RpcError::REJECTED, "hello first");
        plugin.ok(&mut core, "hello", json!({"name": "xp"}));
        let id = plugin.send("session.quit", Value::Null);
        let mut turns = 0;
        core.event_loop
            .run(Duration::from_millis(10), &mut core.state, |state| {
                state.after_events();
                turns += 1;
                if turns == 50 {
                    state.loop_signal.stop();
                }
            })
            .unwrap();
        assert!(turns < 50, "session.quit stops the loop");
        let answer = plugin.responses();
        assert_eq!(answer[0]["id"], id);
        assert_eq!(answer[0]["result"], Value::Null, "{answer:?}");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A confined agent calls no plugin method without the capability, so
    /// it cannot ask a window manager for the focus.
    #[test]
    fn a_client_socket_calls_methods_only_with_the_capability() {
        let (mut core, dir) = socket_dir("call");
        let mut admin = core.admin();
        let call = json!({"client": "wm", "method": "focus"});
        let (mut agent, _) = fixed(&mut core, &mut admin, "agent");
        let (error_code, _) = agent.fails(&mut core, "methods.call", call.clone());
        assert_eq!(error_code, RpcError::MISSING_CAPABILITY);
        let create = json!({"name": "caller", "capabilities": ["methods.call"]});
        admin.ok(&mut core, "clients.create", create);
        let mut caller = core.connect_to(&dir.join("clients/caller/ipc.sock"));
        caller.ok(&mut core, "hello", json!({"name": "caller"}));
        let (error_code, message) = caller.fails(&mut core, "methods.call", call);
        assert_eq!(
            error_code,
            RpcError::REJECTED,
            "the grant lets the call pass"
        );
        assert_eq!(message, "wm.focus is no exported method");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `choractl mcp` confines its connection of the main socket: the
    /// rules of a client socket hold until a grant of its name opens them.
    #[test]
    fn a_confined_main_socket_connection_needs_the_grants_of_a_client_socket() {
        let mut core = Core::new();
        let mut admin = core.admin();
        let mut mc = core.named("mc");
        let names = json!({"names": [{"name": "app.open", "privileged": true}]});
        mc.ok(&mut core, "methods.export", names);
        let mut mcp = core.connect();
        mcp.ok(
            &mut core,
            "hello",
            json!({"name": "voice-mcp", "confined": true}),
        );
        let open = json!({"client": "mc", "method": "app.open", "params": {}});
        for (method, params) in [
            ("methods.call", open.clone()),
            ("activation.token", Value::Null),
            ("session.quit", Value::Null),
        ] {
            let (error_code, message) = mcp.fails(&mut core, method, params);
            assert_eq!(error_code, RpcError::MISSING_CAPABILITY, "{method}");
            assert!(message.contains("choractl grant voice-mcp"), "{message}");
        }
        let grant = json!({"client": "voice-mcp", "capabilities": ["methods.call"]});
        admin.ok(&mut core, "grants.set", grant);
        let (error_code, message) = mcp.fails(&mut core, "methods.call", open.clone());
        assert_eq!(error_code, RpcError::MISSING_CAPABILITY);
        assert!(message.contains("methods.privileged"), "{message}");
        let grant = json!({"client": "voice-mcp", "capabilities": ["methods.privileged"]});
        admin.ok(&mut core, "grants.set", grant);
        mcp.send("methods.call", open);
        assert_eq!(mc.event_names(&mut core), ["method.called"]);
        // Another connection of the name without the flag stays free.
        let mut free = core.connect();
        free.ok(&mut core, "hello", json!({"name": "other"}));
        free.ok(&mut core, "activation.token", Value::Null);
    }

    /// Plugin settings hold commands that the plugin runs: a client socket
    /// with `methods.call` alone cannot write them, nor call a method that
    /// the export marks privileged (W12).
    #[test]
    fn a_client_socket_writes_settings_only_with_the_capability() {
        let (mut core, dir) = socket_dir("settings");
        let mut admin = core.admin();
        let create = json!({"name": "caller", "capabilities": ["methods.call"]});
        admin.ok(&mut core, "clients.create", create);
        let mut caller = core.connect_to(&dir.join("clients/caller/ipc.sock"));
        caller.ok(&mut core, "hello", json!({"name": "caller"}));
        let mut mc = core.named("mc");
        let names = json!({"names": ["look", {"name": "app.open", "privileged": true}]});
        mc.ok(&mut core, "methods.export", names);
        let open = json!({"client": "mc", "method": "app.open", "params": {}});
        let (error_code, message) = caller.fails(&mut core, "methods.call", open);
        assert_eq!(error_code, RpcError::MISSING_CAPABILITY, "{message}");
        let look = json!({"client": "mc", "method": "look", "params": {}});
        caller.send("methods.call", look);
        assert_eq!(
            mc.event_names(&mut core),
            ["method.called"],
            "a plain method passes"
        );
        for method in ["settings.set", "settings.reload"] {
            let call = json!({"client": "wm", "method": method, "params": {}});
            let (error_code, message) = caller.fails(&mut core, "methods.call", call);
            assert_eq!(error_code, RpcError::MISSING_CAPABILITY, "{method}");
            assert!(message.contains("methods.privileged"), "{message}");
        }
        let get = json!({"client": "wm", "method": "settings.get"});
        let (error_code, _) = caller.fails(&mut core, "methods.call", get);
        assert_eq!(
            error_code,
            RpcError::REJECTED,
            "settings.get needs no grant"
        );
        let create =
            json!({"name": "writer", "capabilities": ["methods.call", "methods.privileged"]});
        admin.ok(&mut core, "clients.create", create);
        let mut writer = core.connect_to(&dir.join("clients/writer/ipc.sock"));
        writer.ok(&mut core, "hello", json!({"name": "writer"}));
        let call = json!({"client": "wm", "method": "settings.set", "params": {}});
        let (error_code, message) = writer.fails(&mut core, "methods.call", call.clone());
        assert_eq!(
            error_code,
            RpcError::REJECTED,
            "the grant lets the call pass"
        );
        assert_eq!(message, "wm.settings.set is no exported method");
        let mut main = core.named("main");
        let (error_code, _) = main.fails(&mut core, "methods.call", call);
        assert_eq!(error_code, RpcError::REJECTED, "the main socket may always");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_name_connects_with_its_first_and_disconnects_with_its_last_connection() {
        let mut core = Core::new();
        let mut watcher = core.named("watcher");
        let events = json!({"events": ["client.connected", "client.disconnected"]});
        watcher.ok(&mut core, "events.subscribe", events);
        let first = core.named("wm");
        let second = core.named("wm");
        assert_eq!(watcher.event_names(&mut core), ["client.connected"]);
        drop(first);
        assert_eq!(watcher.event_names(&mut core), Vec::<String>::new());
        drop(second);
        assert_eq!(watcher.event_names(&mut core), ["client.disconnected"]);
    }

    /// A typo in a name is an answer, not silence until the end of time.
    #[test]
    fn subscribe_wants_known_event_names() {
        let mut core = Core::new();
        let mut client = core.named("client");
        client.ok(&mut core, "events.subscribe", json!({"events": []}));
        client.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["surface.created", "output.changed"]}),
        );
        let (_, message) = client.fails(
            &mut core,
            "events.subscribe",
            json!({"events": ["surface.created", "surface.creatd"]}),
        );
        assert!(message.contains("no event \"surface.creatd\""), "{message}");
        // Outside the core names, a name is the event of a client (D65).
        let clients = json!({"events": ["voice.state", "voice.*", "surface.*"]});
        client.ok(&mut core, "events.subscribe", clients);
        for bad in ["voice", "Voice.state", ".*", "output.state"] {
            let events = json!({"events": [bad]});
            client.fails(&mut core, "events.subscribe", events);
        }
    }

    /// D65: a client emits only under its own name, and the core events
    /// stay the core's.
    #[test]
    fn a_client_emits_events_of_its_own_name_only() {
        let mut core = Core::new();
        let mut voice = core.named("voice");
        let state = json!({"event": "voice.state", "data": {"state": "listening"}});
        voice.ok(&mut core, "events.emit", state);
        for event in ["mc.state", "voice", "voice.", "voicex.state", "voice.State"] {
            let (error_code, message) =
                voice.fails(&mut core, "events.emit", json!({"event": event}));
            assert_eq!(error_code, RpcError::REJECTED, "{event}: {message}");
        }
        let mut impostor = core.named("surface");
        let created = json!({"event": "surface.created", "data": {"surface": "s1"}});
        let (_, message) = impostor.fails(&mut core, "events.emit", created);
        assert!(message.contains("core events"), "{message}");
        let mut silent = core.connect();
        silent.fails(&mut core, "events.emit", json!({"event": "voice.state"}));
    }

    /// D65: an emitted event reaches the connections that named it or a
    /// pattern of it, not those of every core event; it carries the
    /// emitter and the data.
    #[test]
    fn an_emitted_event_reaches_its_subscribers_only() {
        let mut core = Core::new();
        let mut exact = core.named("exact");
        exact.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["voice.state"]}),
        );
        let mut pattern = core.named("pattern");
        pattern.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["voice.*"]}),
        );
        let mut all = core.named("all");
        all.ok(&mut core, "events.subscribe", json!({"events": []}));
        let mut other = core.named("other");
        other.ok(&mut core, "events.subscribe", json!({"events": ["mc.*"]}));
        let mut voice = core.named("voice");
        let state = json!({"event": "voice.state", "data": {"state": "speaking"}});
        voice.ok(&mut core, "events.emit", state);
        voice.ok(&mut core, "events.emit", json!({"event": "voice.say"}));
        let got = exact.events_now(&mut core);
        assert_eq!(got.len(), 1, "{got:?}");
        let params = &got[0]["params"];
        assert_eq!(params["event"], "voice.state");
        assert_eq!(params["client"], "voice");
        assert_eq!(params["data"]["state"], "speaking");
        assert!(params["seq"].is_u64());
        assert_eq!(pattern.event_names(&mut core), ["voice.state", "voice.say"]);
        let core_only = all.event_names(&mut core);
        assert!(
            core_only.iter().all(|e| e == "client.connected"),
            "{core_only:?}"
        );
        assert_eq!(other.event_names(&mut core), Vec::<String>::new());
    }

    /// D65: the data has at most 64 KiB, and a name emits at most 32
    /// events per second; the rest is refused with code 6.
    #[test]
    fn emitted_events_have_a_size_and_a_rate_limit() {
        let mut core = Core::new();
        core.state.time.stop();
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["voice.*"]}),
        );
        let mut voice = core.named("voice");
        let big = json!({"event": "voice.big", "data": "x".repeat(MAX_EMIT)});
        // The line is longer than one read of the core.
        let id = voice.send("events.emit", big);
        let answer = std::iter::repeat_with(|| voice.answer(&mut core, id))
            .flatten()
            .next()
            .expect("an answer");
        assert_eq!(answer.unwrap_err()["code"], RpcError::LIMIT_REACHED);
        for _ in 0..MAX_EMITS {
            voice.ok(&mut core, "events.emit", json!({"event": "voice.tick"}));
        }
        let mut twin = core.named("voice");
        let tick = json!({"event": "voice.tick"});
        let (error_code, message) = twin.fails(&mut core, "events.emit", tick.clone());
        assert_eq!(error_code, RpcError::LIMIT_REACHED, "the limit is per name");
        assert!(message.contains("last state"), "{message}");
        core.advance(EMIT_WINDOW);
        voice.ok(&mut core, "events.emit", tick);
        let ticks = watcher.event_names(&mut core);
        assert_eq!(
            ticks.len(),
            MAX_EMITS as usize + 1,
            "refused events never arrive"
        );
    }

    /// Gate check 5 of 0.8: a subscription that starts while transactions
    /// run misses no event and sees no event twice.
    #[test]
    fn a_subscription_answers_with_a_snapshot_and_numbers_its_events() {
        const TOTAL: usize = 1000;
        const BEFORE: usize = 512;
        let mut core = Core::new();
        core.add_output("DP-1", (800, 600));
        let mut watcher = core.named("watcher");
        watcher.ok(&mut core, "scene.apply", rect("kept"));
        // Every camera transaction sends one `output.changed`.
        let camera = json!({"commands": [{"camera": {"output": "DP-1"}}]});
        // The subscription lands between two transactions of a waiting
        // list of `TOTAL`. Batches keep the socket buffers free.
        let mut sub = 0;
        let mut messages: Vec<Value> = Vec::new();
        for sent in 0..TOTAL {
            watcher.send("scene.apply", camera.clone());
            if sent + 1 == BEFORE {
                sub = watcher.send("events.subscribe", json!({"events": ["output.changed"]}));
            }
            if (sent + 1) % 64 == 0 {
                core.dispatch();
                messages.extend(watcher.read().unwrap_or_default());
            }
        }
        loop {
            core.dispatch();
            let more = watcher.read().unwrap_or_default();
            if more.is_empty() {
                break;
            }
            messages.extend(more);
        }
        let events = TOTAL - BEFORE;
        assert_eq!(messages.len(), TOTAL + 1 + events, "all answers and events");
        let at = messages
            .iter()
            .position(|m| m["id"] == sub)
            .expect("the subscribe answer");
        assert!(messages[..at].iter().all(|m| m["method"] != "event"));
        let snapshot = messages[at]["result"].as_array().expect("a snapshot");
        assert!(snapshot.iter().any(|n| n["id"] == "kept"));
        let seqs: Vec<u64> = messages[at..]
            .iter()
            .filter(|m| m["method"] == "event")
            .map(|m| m["params"]["seq"].as_u64().expect("a sequence number"))
            .collect();
        assert_eq!(seqs, (1..=events as u64).collect::<Vec<u64>>());
    }

    #[test]
    fn a_subscription_names_its_events_or_takes_all() {
        let mut core = Core::new();
        let mut some = core.named("some");
        let only = json!({"events": ["client.disconnected"]});
        some.ok(&mut core, "events.subscribe", only);
        let mut quiet = core.named("quiet");
        let mut all = core.named("all");
        all.ok(&mut core, "events.subscribe", json!({"events": []}));
        drop(core.named("bar"));
        assert_eq!(some.event_names(&mut core), ["client.disconnected"]);
        assert_eq!(
            all.event_names(&mut core),
            ["client.connected", "client.disconnected"]
        );
        assert_eq!(quiet.event_names(&mut core), Vec::<String>::new());
    }

    #[test]
    fn the_nodes_of_a_name_go_with_its_last_connection() {
        let mut core = Core::new();
        let mut reader = core.named("reader");
        let mut wm = core.named("wm");
        wm.ok(&mut core, "scene.apply", rect("r"));
        let mut kept = core.connect();
        kept.ok(
            &mut core,
            "hello",
            json!({"name": "panel", "keep_nodes": true}),
        );
        kept.ok(&mut core, "scene.apply", rect("k"));
        assert_eq!(ids(&mut reader, &mut core, "kind:rect"), ["k", "r"]);
        drop(wm);
        drop(kept);
        core.dispatch();
        assert_eq!(ids(&mut reader, &mut core, "kind:rect"), ["k"]);
    }

    /// A window manager with `resume` that holds a group with meta and a
    /// transform, two rects in it in order, and the surface `s1` of a
    /// toplevel in it.
    fn resumable(core: &mut Core) -> (Client, Wayland) {
        let mut wayland = Wayland::connect(core);
        wayland.toplevel(core, "foot");
        let mut wm = hello_resume(core);
        let size = json!({"size": [1, 1]});
        let commands = json!({"commands": [
            {"create": {"id": "g", "kind": "group", "parent": "root",
                "props": {"meta": {"tag": "9"}, "transform": {"translate": [40, 50]}}}},
            {"create": {"id": "a", "kind": "rect", "parent": "g", "props": size}},
            {"create": {"id": "b", "kind": "rect", "parent": "g", "props": size}},
            {"attach": {"surface": "s1", "parent": "g"}},
        ]});
        wm.ok(core, "scene.apply", commands);
        (wm, wayland)
    }

    fn hello_resume(core: &mut Core) -> Client {
        let mut wm = core.connect();
        wm.ok(core, "hello", json!({"name": "wm", "resume": true}));
        wm
    }

    fn owned(reader: &mut Client, core: &mut Core) -> Value {
        reader.ok(core, "scene.query", json!({"selector": "owner:wm"}))
    }

    fn remove_s1_to(to: &str) -> Value {
        json!({"commands": [{"remove": {"id": "s1", "to": to}}]})
    }

    fn attach_s1(parent: &str) -> Value {
        json!({"commands": [{"attach": {"surface": "s1", "parent": parent}}]})
    }

    /// A window manager with a group that claims an area.
    fn claimer(core: &mut Core, name: &str, claim: [f64; 4]) -> Client {
        let mut wm = core.named(name);
        let group = json!({"commands": [{"create": {"id": name, "kind": "group",
            "parent": "root", "props": {"claim": claim}}}]});
        wm.ok(core, "scene.apply", group);
        wm
    }

    fn reserved_for(core: &Core, surface: &str) -> Option<String> {
        core.state.scene.node(surface).and_then(|n| n.to.clone())
    }

    #[test]
    fn a_new_toplevel_goes_to_the_focus_or_the_smallest_claim_under_the_pointer() {
        let mut core = Core::new();
        core.add_output("A", (800, 600));
        let mut a = claimer(&mut core, "a", [0.0, 0.0, 800.0, 600.0]);
        let _b = claimer(&mut core, "b", [100.0, 100.0, 200.0, 200.0]);
        let mut wayland = Wayland::connect(&mut core);
        core.state.pointer_layout = (150.0, 150.0).into();
        wayland.toplevel(&mut core, "one");
        assert_eq!(reserved_for(&core, "s1").as_deref(), Some("b"), "smallest");
        let (_, message) = a.fails(&mut core, "scene.apply", attach_s1("a"));
        assert!(message.contains("reserved"), "{message}");
        core.state.pointer_layout = (500.0, 500.0).into();
        wayland.toplevel(&mut core, "two");
        assert_eq!(reserved_for(&core, "s2").as_deref(), Some("a"));
        let take = json!({"commands": [{"attach": {"surface": "s2", "parent": "a"}},
            {"focus": {"surface": "s2"}}]});
        a.ok(&mut core, "scene.apply", take);
        core.state.pointer_layout = (150.0, 150.0).into();
        wayland.toplevel(&mut core, "three");
        assert_eq!(reserved_for(&core, "s3").as_deref(), Some("a"), "the focus");
    }

    #[test]
    fn a_reservation_for_a_keep_nodes_client_ends_with_it() {
        let mut core = Core::new();
        let (mut wm, _wayland) = resumable(&mut core);
        let mut agent = core.connect();
        agent.ok(
            &mut core,
            "hello",
            json!({"name": "agent", "keep_nodes": true}),
        );
        let group = json!({"commands": [{"create": {"id": "agent", "kind": "group",
            "parent": "root", "props": {"claim": [0, 0, 10, 10]}}}]});
        agent.ok(&mut core, "scene.apply", group);
        let mut other = core.named("other");
        other.ok(&mut core, "events.subscribe", json!({"events": []}));
        wm.ok(&mut core, "scene.apply", remove_s1_to("agent"));
        drop(agent);
        core.dispatch();
        let names = other.event_names(&mut core);
        assert!(names.contains(&"surface.detached".to_owned()), "{names:?}");
        other.ok(&mut core, "scene.apply", attach_s1("root"));
    }

    /// A window manager that never attaches a surface given to it does not
    /// hide it for good: the reservation ends after a while (D55).
    #[test]
    fn a_reservation_that_nobody_attaches_ends() {
        let mut core = Core::new();
        core.state.time.stop();
        let (mut wm, _wayland) = resumable(&mut core);
        let _wm2 = claimer(&mut core, "wm2", [0.0, 0.0, 10.0, 10.0]);
        let mut other = core.named("other");
        other.ok(&mut core, "events.subscribe", json!({"events": []}));
        wm.ok(&mut core, "scene.apply", remove_s1_to("wm2"));
        core.advance(crate::state::RESERVATION / 2);
        assert_eq!(reserved_for(&core, "s1").as_deref(), Some("wm2"));
        core.advance(crate::state::RESERVATION);
        assert_eq!(reserved_for(&core, "s1"), None);
        let names = other.event_names(&mut core);
        assert!(names.contains(&"surface.detached".to_owned()), "{names:?}");
        other.ok(&mut core, "scene.apply", attach_s1("root"));
    }

    #[test]
    fn a_surface_removed_to_a_client_goes_only_to_it() {
        let mut core = Core::new();
        let (mut wm, _wayland) = resumable(&mut core);
        let focus = json!({"commands": [{"focus": {"surface": "s1"}}]});
        wm.ok(&mut core, "scene.apply", focus);
        let mut b = core.named("b");
        b.ok(&mut core, "events.subscribe", json!({"events": []}));
        let group = json!({"commands": [{"create": {"id": "bg", "kind": "group",
            "parent": "root", "props": {"claim": [0, 0, 10, 10]}}}]});
        b.ok(&mut core, "scene.apply", group);
        let mut other = claimer(&mut core, "other", [0.0, 0.0, 20.0, 20.0]);
        other.ok(&mut core, "events.subscribe", json!({"events": []}));
        let (_, message) = wm.fails(&mut core, "scene.apply", remove_s1_to("nobody"));
        assert!(message.contains("no claim"), "{message}");
        wm.ok(&mut core, "scene.apply", remove_s1_to("b"));
        let heard = |events: Vec<Value>| {
            events.iter().any(|e| {
                e["params"]["event"] == "surface.detached" && e["params"]["surface"] == "s1"
            })
        };
        assert!(heard(b.events_now(&mut core)));
        assert!(!heard(other.events_now(&mut core)), "only b hears of it");
        assert_eq!(core.state.focused_surface(), None, "the focus stays behind");
        let (_, message) = other.fails(&mut core, "scene.apply", attach_s1("root"));
        assert!(message.contains("reserved for \"b\""), "{message}");
        b.ok(&mut core, "scene.apply", attach_s1("bg"));
        let node = core.state.scene.node("s1").unwrap();
        assert_eq!(
            (node.owner.as_deref(), node.to.as_deref()),
            (Some("b"), None)
        );
        // A reservation for a client that goes ends: anyone takes it.
        b.ok(&mut core, "scene.apply", remove_s1_to("other"));
        drop(other);
        core.dispatch();
        assert!(
            b.event_names(&mut core)
                .contains(&"surface.detached".to_owned())
        );
        b.ok(&mut core, "scene.apply", attach_s1("bg"));
    }

    /// A client changes a node of another client only by request (D31).
    #[test]
    fn a_change_of_a_foreign_node_reaches_its_owner_as_a_request() {
        let mut core = Core::new();
        let (mut wm, _wayland) = resumable(&mut core);
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        let mut b = core.named("b");
        b.ok(&mut core, "events.subscribe", json!({"events": []}));
        let mut watcher = core.named("watcher");
        watcher.ok(&mut core, "events.subscribe", json!({"events": []}));
        // The owner gets every action with the arguments of the command.
        for (command, action, args) in [
            (json!({"close": {"surface": "s1"}}), "close", json!({})),
            (json!({"focus": {"surface": "s1"}}), "focus", json!({})),
            (
                json!({"move": {"id": "s1", "parent": "root"}}),
                "move",
                json!({"parent": "root", "index": null}),
            ),
            (
                json!({"configure": {"surface": "s1", "size": [7, 9]}}),
                "resize",
                json!({"size": [7, 9], "states": []}),
            ),
        ] {
            b.ok(&mut core, "scene.apply", json!({"commands": [command]}));
            let events = wm.events_now(&mut core);
            assert!(
                node_requested(&events, "b", action, &args),
                "{action}: {events:?}"
            );
            assert!(!node_requested(
                &b.events_now(&mut core),
                "b",
                action,
                &args
            ));
            assert!(
                !node_requested(&watcher.events_now(&mut core), "b", action, &args),
                "only the owner"
            );
        }
        // Nothing happened to the window: the owner decides by doing it
        // or not.
        assert!(core.state.windows.contains_key("s1"));
        assert_eq!(
            core.state.scene.node("s1").unwrap().parent.as_deref(),
            Some("g")
        );
        assert_eq!(core.state.focused_surface(), None);
        // Changes beyond the four actions stay refused.
        let update = json!({"commands": [{"update": {"id": "s1", "props": {"visible": false}}}]});
        let (_, message) = b.fails(&mut core, "scene.apply", update);
        assert!(message.contains("belongs to"), "{message}");
        // The owner still changes its own node without a request.
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"close": {"surface": "s1"}}
            ]}),
        );
    }

    /// An update of the look of a foreign node reaches its owner after the
    /// core checked it (D58).
    #[test]
    fn an_update_of_a_foreign_node_reaches_its_owner_as_a_request() {
        let mut core = Core::new();
        let (mut wm, _wayland) = resumable(&mut core);
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        let mut b = core.named("b");
        let update = |props: Value| json!({"commands": [{"update": {"id": "s1", "props": props}}]});
        let props = json!({"opacity": 0.5, "meta": {"k": "v"}});
        b.ok(&mut core, "scene.apply", update(props.clone()));
        let events = wm.events_now(&mut core);
        assert!(node_requested(&events, "b", "update", &props), "{events:?}");
        assert_eq!(core.state.scene.node("s1").unwrap().props.opacity, 1.0);
        let (_, message) = b.fails(
            &mut core,
            "scene.apply",
            update(json!({"meta": {"k": "v".repeat(2000)}})),
        );
        assert!(message.contains("meta data"), "{message}");
        let (_, message) = b.fails(
            &mut core,
            "scene.apply",
            update(json!({"opacity": 0.5, "visible": false, "size": [1, 1]})),
        );
        assert!(
            message.contains("belongs to") && message.ends_with("not size, visible"),
            "the owner-only props: {message}"
        );
        assert_eq!(wm.events_now(&mut core), Vec::<Value>::new());
        // An owner that does not listen would lose the change.
        wm.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["surface.created"]}),
        );
        let (_, message) = b.fails(&mut core, "scene.apply", update(props));
        assert!(message.contains("belongs to"), "{message}");
    }

    /// Whether one of the events is a `node.requested` of `s1` by `by`.
    fn node_requested(events: &[Value], by: &str, action: &str, args: &Value) -> bool {
        events.iter().any(|e| {
            let p = &e["params"];
            p["event"] == "node.requested"
                && p["node"] == "s1"
                && p["client"] == by
                && p["action"] == action
                && p["args"] == *args
        })
    }

    #[test]
    fn a_client_with_resume_gets_its_nodes_back_within_the_grace_time() {
        let mut core = Core::new();
        let mut reader = core.named("reader");
        let (wm, _wayland) = resumable(&mut core);
        let before = owned(&mut reader, &mut core);
        assert_eq!(before.as_array().map(Vec::len), Some(4));
        drop(wm);
        core.dispatch();
        core.advance(GRACE.saturating_sub(Duration::from_millis(1)));
        assert_eq!(owned(&mut reader, &mut core), before);
        let mut wm = hello_resume(&mut core);
        assert_eq!(owned(&mut reader, &mut core), before);
        assert_eq!(
            core.state.scene.node("g").unwrap().children,
            ["a", "b", "s1"]
        );
        let hide = json!({"commands": [{"update": {"id": "s1", "props": {"visible": false}}}]});
        wm.ok(&mut core, "scene.apply", hide);
        core.advance(GRACE);
        assert!(
            core.state.scene.node("g").is_some(),
            "the timer of the old grace time removes nothing"
        );
    }

    /// A core whose main socket would be in a fresh directory.
    fn socket_dir(tag: &str) -> (Core, PathBuf) {
        let dir = std::env::temp_dir().join(format!("chora-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut core = Core::new();
        core.state.ipc.path = Some(dir.join("wayland-1.sock"));
        (core, dir)
    }

    /// A client socket of a name, and a client through it after hello.
    fn fixed(core: &mut Core, admin: &mut Client, name: &str) -> (Client, PathBuf) {
        let created = admin.ok(core, "clients.create", json!({"name": name}));
        let path = PathBuf::from(created["path"].as_str().unwrap());
        let mut client = core.connect_to(&path);
        client.ok(core, "hello", json!({"name": name}));
        (client, path)
    }

    #[test]
    fn a_client_wayland_socket_names_its_windows_and_hides_protocols_without_grants() {
        let (mut core, dir) = socket_dir("wayland");
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let path = PathBuf::from(created["wayland"].as_str().unwrap());
        assert_eq!(path, dir.join("clients/agent/wayland.sock"));
        let offers =
            |wayland: &Wayland, part: &str| wayland.interfaces().iter().any(|i| i.contains(part));
        let privileged = [
            "zwlr_screencopy",
            "ext_image_copy_capture",
            "input_method",
            "zwlr_output_manager",
        ];
        let plain = Wayland::connect(&mut core);
        assert!(
            privileged.iter().all(|p| offers(&plain, p)),
            "{:?}",
            plain.interfaces()
        );
        let mut agent = Wayland::connect_to(&mut core, &path);
        assert!(offers(&agent, "wl_compositor"));
        for part in privileged {
            assert!(!offers(&agent, part), "{part}: {:?}", agent.interfaces());
        }
        agent.toplevel(&mut core, "app");
        let surface = core
            .state
            .scene
            .node("s1")
            .unwrap()
            .surface
            .clone()
            .unwrap();
        assert_eq!(surface.client.as_deref(), Some("agent"));
        admin.ok(
            &mut core,
            "clients.create",
            json!({"name": "agent", "capabilities": ["screen.capture"]}),
        );
        let granted = Wayland::connect_to(&mut core, &path);
        assert!(offers(&granted, "zwlr_screencopy") && offers(&granted, "ext_image_copy_capture"));
        assert!(!offers(&granted, "input_method") && !offers(&granted, "zwlr_output_manager"));
        core.state.reset();
        assert!(!path.exists());
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_wayland_socket_offers_no_lock_and_no_window_list() {
        let (mut core, dir) = socket_dir("wayland-lock");
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let path = PathBuf::from(created["wayland"].as_str().unwrap());
        let agent = Wayland::connect_to(&mut core, &path);
        let offers = |part: &str| agent.interfaces().iter().any(|i| i.contains(part));
        let hidden = [
            "session_lock",
            "foreign_toplevel",
            "security_context",
            "idle_inhibit",
            "idle_notifier",
        ];
        for part in hidden {
            assert!(!offers(part), "{part}: {:?}", agent.interfaces());
        }
        assert!(offers("zwlr_layer_shell"), "panels in containers");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_wayland_socket_holds_at_most_16_clients() {
        let (mut core, dir) = socket_dir("wayland-cap");
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let path = PathBuf::from(created["wayland"].as_str().unwrap());
        let mut clients: Vec<Wayland> = (0..clients::MAX_PER_SOCKET)
            .map(|_| Wayland::connect_to(&mut core, &path))
            .collect();
        // The 17th closes at once: it reads the end of the stream.
        let refused = |core: &mut Core| {
            let mut extra = UnixStream::connect(&path).unwrap();
            core.dispatch();
            extra.set_nonblocking(true).unwrap();
            matches!(std::io::Read::read(&mut extra, &mut [0; 1]), Ok(0))
        };
        assert!(refused(&mut core));
        clients.pop();
        // The core itself says when the place is free.
        let held = |core: &Core| {
            usize::try_from(
                core.state.ipc.clients["agent"]
                    .wayland_count
                    .load(std::sync::atomic::Ordering::Relaxed),
            )
            .unwrap()
        };
        let mut freed = false;
        for _ in 0..50 {
            core.dispatch();
            if held(&core) == clients::MAX_PER_SOCKET - 1 {
                freed = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(freed, "a gone client frees its place");
        let _place = Wayland::connect_to(&mut core, &path);
        assert!(refused(&mut core), "the 16 places are full again");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A create with other capabilities recreates the socket (D33) and
    /// ends its Wayland clients, so none keeps a capability that the
    /// clients file took away. The reset ends all of them.
    #[test]
    fn changed_capabilities_and_the_reset_disconnect_the_wayland_clients() {
        let (mut core, dir) = socket_dir("wayland-recreate");
        let mut admin = core.admin();
        let caps = json!({"name": "agent", "capabilities": ["screen.capture"]});
        let created = admin.ok(&mut core, "clients.create", caps);
        let path = PathBuf::from(created["wayland"].as_str().unwrap());
        let mut capturer = Wayland::connect_to(&mut core, &path);
        let inject = json!({"name": "agent", "capabilities": ["input.inject"]});
        admin.ok(&mut core, "clients.create", inject);
        assert!(
            capturer.closed(&mut core),
            "the recreate ends the client that had screen.capture"
        );
        let mut plain = Wayland::connect_to(&mut core, &path);
        assert!(!plain.closed(&mut core));
        core.state.reset();
        assert!(plain.closed(&mut core), "the reset ends all of them");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_socket_gets_no_overlay_and_no_exclusive_keyboard() {
        use wayland_protocols_wlr::layer_shell::v1::client::{
            zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::KeyboardInteractivity,
        };
        let (mut core, dir) = socket_dir("wayland-overlay");
        let output = core.add_output("A", (800, 600));
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let path = PathBuf::from(created["wayland"].as_str().unwrap());
        let mut agent = Wayland::connect_to(&mut core, &path);
        let layers = |core: &Core| {
            let _ = core;
            smithay::desktop::layer_map_for_output(&output)
                .layers()
                .count()
        };
        agent.layer(&mut core, Layer::Overlay, KeyboardInteractivity::None);
        assert_eq!(layers(&core), 0, "no overlay");
        let panel = agent.layer(&mut core, Layer::Top, KeyboardInteractivity::Exclusive);
        assert_eq!(layers(&core), 1, "a panel stays");
        assert_eq!(core.state.exclusive_layer(), None, "no exclusive keyboard");
        panel.layer.set_layer(Layer::Overlay);
        panel.surface.commit();
        agent.roundtrip(&mut core);
        assert_eq!(layers(&core), 0, "no move to the overlay layer");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_socket_forgets_the_wayland_clients_that_went() {
        let (mut core, dir) = socket_dir("wayland-gone");
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let path = PathBuf::from(created["wayland"].as_str().unwrap());
        for _ in 0..3 {
            let gone = Wayland::connect_to(&mut core, &path);
            drop(gone);
            core.dispatch();
        }
        let _live = Wayland::connect_to(&mut core, &path);
        assert_eq!(core.state.ipc.clients["agent"].clients.len(), 1);
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A name that already has a socket gets its path in the error data, so
    /// that `choractl clients create` can name the socket it must reuse.
    #[test]
    fn a_taken_name_reports_its_socket_path() {
        let (mut core, dir) = socket_dir("taken-name");
        let mut admin = core.admin();
        admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let error = admin
            .call(&mut core, "clients.create", json!({"name": "agent"}))
            .expect_err("the name has a socket already");
        assert_eq!(error["code"], RpcError::NAME_TAKEN);
        assert_eq!(error["data"]["reason"], "socket_exists");
        assert_eq!(
            error["data"]["path"],
            dir.join("clients/agent/ipc.sock").to_str().unwrap(),
        );
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn clients_create_refuses_a_live_name() {
        let (mut core, dir) = socket_dir("live");
        let mut admin = core.admin();
        let squatter = core.named("agent");
        let (error_code, message) =
            admin.fails(&mut core, "clients.create", json!({"name": "agent"}));
        assert_eq!(error_code, RpcError::NAME_TAKEN);
        assert!(
            message.contains("connected or in its grace time"),
            "{message}"
        );
        drop(squatter);
        let resumed = hello_resume(&mut core);
        drop(resumed);
        core.dispatch();
        let (_, message) = admin.fails(&mut core, "clients.create", json!({"name": "wm"}));
        assert!(message.contains("grace time"), "{message}");
        admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The bound of the logical size holds for `output.configure` too.
    #[test]
    fn a_configure_makes_the_logical_side_not_grow() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        core.add_output("BIG", (16384, 64));
        let (error_code, text) = wm.fails(
            &mut core,
            "output.configure",
            json!({"output": "BIG", "scale": 0.1}),
        );
        assert_eq!(error_code, RpcError::REJECTED);
        assert!(text.contains("logical side"), "{text}");
        // The same size at scale 1 reaches the backend untouched.
        let (_, text) = wm.fails(
            &mut core,
            "output.configure",
            json!({"output": "BIG", "scale": 1.0}),
        );
        assert_eq!(text, "no backend");
    }

    #[test]
    fn the_reset_removes_the_client_sockets() {
        let (mut core, dir) = socket_dir("reset");
        let mut admin = core.admin();
        let (_agent, path) = fixed(&mut core, &mut admin, "agent");
        core.state.reset();
        core.dispatch();
        assert!(!path.exists());
        assert!(UnixStream::connect(&path).is_err());
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_socket_leaves_room_for_the_main_socket() {
        let (mut core, dir) = socket_dir("flood");
        let mut admin = core.admin();
        let (first, path) = fixed(&mut core, &mut admin, "agent");
        // Each connection as from a process of its own, as in a container.
        let connect = |core: &mut Core, path: &Path| {
            let client = core.connect_to(path);
            as_other_processes(core);
            client
        };
        // One more than the cap, with the first: the last is refused.
        let flood: Vec<Client> = (1..=clients::MAX_PER_SOCKET)
            .map(|_| connect(&mut core, &path))
            .collect();
        let fixed_conns = |core: &Core| {
            core.state
                .ipc
                .conns
                .values()
                .filter(|c| c.fixed().is_some())
                .count()
        };
        assert_eq!(fixed_conns(&core), clients::MAX_PER_SOCKET);
        drop((first, flood));
        // Many client sockets together keep room for the main socket.
        let mut all = Vec::new();
        for i in 0..8 {
            let created = admin.ok(
                &mut core,
                "clients.create",
                json!({"name": format!("c{i}")}),
            );
            let path = PathBuf::from(created["path"].as_str().unwrap());
            all.extend((0..clients::MAX_PER_SOCKET).map(|_| connect(&mut core, &path)));
        }
        core.dispatch();
        assert_eq!(fixed_conns(&core), MAX_CONNECTIONS - clients::KEPT_FOR_MAIN);
        core.named("main");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_socket_names_only_its_own_assets() {
        let (mut core, dir) = socket_dir("paths");
        core.state
            .ipc
            .uploads
            .open(dir.join("wayland-1.assets"))
            .unwrap();
        let mut admin = core.admin();
        let (mut agent, _) = fixed(&mut core, &mut admin, "agent");
        let image = |id: &str, path: &str| {
            json!({"commands": [{"create": {"id": id, "kind": "image", "parent": "root",
                "props": {"image": path}}}]})
        };
        for path in ["/etc/hostname", "/nonexistent.png"] {
            let (_, message) = agent.fails(&mut core, "scene.apply", image("i", path));
            assert!(message.contains("only its own assets"), "{message}");
        }
        let effect = json!({"commands": [{"create": {"id": "e", "kind": "group", "parent": "root",
            "props": {"size": [1, 1], "effects": [{"shader": "/etc/hostname"}]}}}]});
        let (_, message) = agent.fails(&mut core, "scene.apply", effect);
        assert!(message.contains("only its own assets"), "{message}");
        let put = json!({"name": "x.png", "data": "AAAA"});
        let own = agent.ok(&mut core, "assets.put", put)["path"]
            .as_str()
            .unwrap()
            .to_owned();
        let conn = core
            .state
            .ipc
            .conns
            .iter()
            .find(|(_, c)| c.fixed().is_some())
            .map(|(id, _)| *id);
        let own: Vec<chora_scene::Command> =
            serde_json::from_value(image("j", &own)["commands"].clone()).unwrap();
        assert!(core.state.check_paths(conn.unwrap(), &own).is_ok());
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_client_socket_dir_is_private_and_fits_a_socket_path() {
        // A long directory, so that a long name passes no socket path.
        let (mut core, dir) = socket_dir("dirs-long");
        let old = dir.join("clients/agent");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::set_permissions(&old, std::fs::Permissions::from_mode(0o777)).unwrap();
        let mut admin = core.admin();
        fixed(&mut core, &mut admin, "agent");
        let mode = std::fs::metadata(&old).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
        let long = "a".repeat(64);
        let (_, message) = admin.fails(&mut core, "clients.create", json!({"name": long}));
        assert!(message.contains("too long"), "{message}");
        assert!(!dir.join("clients").join(&long).exists());
        for i in 1..64 {
            admin.ok(
                &mut core,
                "clients.create",
                json!({"name": format!("c{i}")}),
            );
        }
        let (_, message) = admin.fails(&mut core, "clients.create", json!({"name": "c64"}));
        assert!(message.contains("64 client sockets"), "{message}");
        drop(core);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A camera owner that never comes back does not hold the view.
    #[test]
    fn the_admin_releases_the_camera_of_a_gone_owner() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut admin = core.admin();
        let mut holder = core.connect();
        holder.ok(&mut core, "hello", json!({"name": "holder"}));
        holder.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"camera": {"output": "out", "transform": {"scale": 2}}}]}),
        );
        let mut other = core.connect();
        other.ok(&mut core, "hello", json!({"name": "other"}));
        let (_, message) = other.fails(
            &mut core,
            "scene.apply",
            json!({"commands": [{"camera": {"output": "out", "transform": {"scale": 1}}}]}),
        );
        assert!(message.contains("belongs to \"holder\""), "{message}");
        let (_, message) = other.fails(&mut core, "camera.release", json!({"output": "out"}));
        assert!(message.contains("only on the admin socket"), "{message}");
        let answer = admin.ok(&mut core, "camera.release", json!({"output": "out"}));
        assert_eq!(answer["was"], "holder");
        other.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"camera": {"output": "out", "transform": {"scale": 1}}}]}),
        );
        let answer = admin.ok(&mut core, "camera.release", json!({"output": "out"}));
        assert_eq!(answer["was"], "other");
    }

    #[test]
    fn a_client_socket_fixes_the_name_and_grants_nothing() {
        let dir = std::env::temp_dir().join(format!("chora-clients-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut core = Core::new();
        core.state.ipc.path = Some(dir.join("wayland-1.sock"));
        let mut admin = core.admin();
        let created = admin.ok(&mut core, "clients.create", json!({"name": "agent"}));
        let path = PathBuf::from(created["path"].as_str().unwrap());
        assert_eq!(path, dir.join("clients/agent/ipc.sock"));
        let mut agent = core.connect_to(&path);
        let (_, message) = agent.fails(&mut core, "hello", json!({"name": "tiling"}));
        assert!(message.contains("belongs to \"agent\""), "{message}");
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        let (_, message) = agent.fails(&mut core, "grants.set", grant);
        assert!(message.contains("only on the admin socket"), "{message}");
        let (_, message) = agent.fails(&mut core, "clients.create", json!({"name": "x"}));
        assert!(message.contains("only on the admin socket"), "{message}");
        agent.ok(&mut core, "hello", json!({"name": "agent"}));
        agent.ok(&mut core, "scene.apply", rect("r"));
        assert_eq!(
            core.state.scene.node("r").unwrap().owner.as_deref(),
            Some("agent")
        );
        let mut other = core.connect();
        let (_, message) = other.fails(&mut core, "hello", json!({"name": "agent"}));
        assert!(message.contains("connects only through it"), "{message}");
        let (_, message) = admin.fails(&mut core, "clients.create", json!({"name": "agent"}));
        assert!(message.contains("already"), "{message}");
        let (_, message) = admin.fails(&mut core, "clients.create", json!({"name": "../x"}));
        assert!(message.contains("lowercase"), "{message}");
        drop(core);
        assert!(!path.exists(), "the core removes the socket");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_resume_while_the_name_is_connected_fails_and_keeps_the_nodes() {
        let mut core = Core::new();
        let mut reader = core.named("reader");
        let (_wm, _wayland) = resumable(&mut core);
        let before = owned(&mut reader, &mut core);
        let mut second = core.connect();
        let hello = json!({"name": "wm", "resume": true});
        let (_, message) = second.fails(&mut core, "hello", hello);
        assert!(message.contains("is connected"), "{message}");
        assert_eq!(owned(&mut reader, &mut core), before);
    }

    #[test]
    fn the_nodes_go_after_the_grace_time() {
        let mut core = Core::new();
        let mut reader = core.named("reader");
        let (wm, _wayland) = resumable(&mut core);
        drop(wm);
        core.dispatch();
        core.advance(GRACE);
        assert_eq!(owned(&mut reader, &mut core), json!([]));
        assert_eq!(core.state.scene.node("s1").unwrap().owner, None);
        let _wm = hello_resume(&mut core);
        assert_eq!(owned(&mut reader, &mut core), json!([]));
    }

    #[test]
    fn only_the_name_and_only_with_resume_gets_the_nodes_back() {
        let mut core = Core::new();
        let mut reader = core.named("reader");
        let (wm, _wayland) = resumable(&mut core);
        drop(wm);
        core.dispatch();
        let mut other = core.named("other");
        let hide = json!({"commands": [{"update": {"id": "g", "props": {"visible": false}}}]});
        let (error_code, _) = other.fails(&mut core, "scene.apply", hide);
        assert_eq!(error_code, REJECTED);
        let _plain = core.named("wm");
        assert_eq!(
            owned(&mut reader, &mut core),
            json!([]),
            "a hello without resume"
        );
    }

    #[test]
    fn the_reset_ends_the_grace_time() {
        let mut core = Core::new();
        let (wm, _wayland) = resumable(&mut core);
        drop(wm);
        core.dispatch();
        core.state.reset();
        assert!(core.state.ipc.graces.is_empty());
    }

    #[test]
    fn a_grab_needs_a_modifier_and_spares_the_safety_layer() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let grab = |binding: &str| json!({"binding": binding});
        let normal = wm.ok(&mut core, "input.grab", grab("Super+Shift+Return"));
        assert_eq!(normal, "Shift+Super+Return");
        for (binding, code, message) in [
            (
                "Shift+a",
                RpcError::MISSING_CAPABILITY,
                "Shift+a needs Ctrl, Alt or Super or the capability input.grab-any",
            ),
            (
                "Ctrl+Alt+BackSpace",
                REJECTED,
                "Ctrl+Alt+BackSpace belongs to the safety layer",
            ),
            (
                "Ctrl+Alt+F3",
                REJECTED,
                "Ctrl+Alt+F3 belongs to the safety layer",
            ),
            (
                "Super+NoSuchKey",
                RpcError::INVALID_PARAMS,
                "unknown key \"NoSuchKey\"",
            ),
        ] {
            let (error_code, got) = wm.fails(&mut core, "input.grab", grab(binding));
            assert_eq!((error_code, got.as_str()), (code, message));
        }
    }

    #[test]
    fn grab_any_grabs_keys_without_a_modifier() {
        let mut core = Core::new();
        let grant = json!({"client": "agent", "capabilities": ["input.grab-any"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        let grab = |binding: &str| json!({"binding": binding});
        assert_eq!(agent.ok(&mut core, "input.grab", grab("F13")), "F13");
        assert_eq!(agent.ok(&mut core, "input.grab", grab("a")), "a");
        // the safety layer stays free even with the capability
        let (_, message) = agent.fails(&mut core, "input.grab", grab("Ctrl+Alt+F3"));
        assert_eq!(message, "Ctrl+Alt+F3 belongs to the safety layer");
        core.admin()
            .ok(&mut core, "grants.revoke", json!({"client": "agent"}));
        // revoked: the next bare grab fails
        let mut late = core.named("agent");
        let (_, message) = late.fails(&mut core, "input.grab", grab("b"));
        assert_eq!(
            message,
            "b needs Ctrl, Alt or Super or the capability input.grab-any"
        );
    }

    #[test]
    fn a_binding_has_one_grab_until_its_connection_closes() {
        let mut core = Core::new();
        let mut first = core.named("wm");
        let mut second = core.named("other");
        let grab = json!({"binding": "Super+a"});
        first.ok(&mut core, "input.grab", grab.clone());
        let (_, message) = second.fails(&mut core, "input.grab", grab.clone());
        assert_eq!(message, "Super+a is grabbed already");
        let (_, message) = second.fails(&mut core, "input.ungrab", grab.clone());
        assert_eq!(message, "Super+a is no grab of this connection");
        drop(first);
        core.dispatch();
        second.ok(&mut core, "input.grab", grab.clone());
        second.ok(&mut core, "input.ungrab", grab.clone());
        second.ok(&mut core, "input.grab", grab);
    }

    #[test]
    fn a_grab_takes_its_key_with_exactly_its_modifiers() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        wm.ok(&mut core, "input.grab", json!({"binding": "Super+a"}));
        let logo = ModifiersState {
            logo: true,
            ..ModifiersState::default()
        };
        let both = ModifiersState {
            shift: true,
            ..logo
        };
        let a = [Keysym::a];
        assert_eq!(
            core.state.ipc.grab_for(&logo, &a),
            Some((0, "Super+a".parse().unwrap()))
        );
        assert_eq!(core.state.ipc.grab_for(&both, &a), None);
        assert_eq!(core.state.ipc.grab_for(&logo, &[Keysym::b]), None);
    }

    #[test]
    fn requests_before_the_end_of_the_stream_get_their_answers() {
        let mut core = Core::new();
        let mut client = core.named("wm");
        let id = client.send("clients.list", Value::Null);
        client.shut_down_writes();
        assert_eq!(
            client.answer(&mut core, id),
            Some(Ok(json!([{"methods": [], "name": "wm"}])))
        );
        assert!(client.closed(&mut core));
    }

    #[test]
    fn a_closed_connection_ends_while_a_forked_child_holds_its_socket() {
        let mut core = Core::new();
        let mut client = core.named("wm");
        let conn = *core.state.ipc.conns.keys().next().expect("a connection");
        // A child between its fork and its exec holds such a copy.
        let _copy = core.state.ipc.conns[&conn].stream.try_clone().unwrap();
        client.shut_down_writes();
        assert!(client.closed(&mut core));
    }

    /// `choractl run` asks the core for a token (D42). A toplevel that
    /// activates with it before or with its first commit shows it in
    /// `surface.created`, and its use ends the token.
    #[test]
    fn the_issued_token_reaches_the_surface_created_of_its_window() {
        for after_commit in [false, true] {
            token_reaches_surface_created(after_commit);
        }
    }

    fn token_reaches_surface_created(after_commit: bool) {
        let mut core = Core::new();
        core.add_output("A", (64, 48));
        let mut ctl = core.named("choractl");
        let answer = ctl.ok(&mut core, "activation.token", Value::Null);
        let token = answer["token"]
            .as_str()
            .expect("an activation token")
            .to_owned();
        assert!(
            core.state
                .activation_state
                .tokens()
                .any(|(t, _)| t.as_str() == token),
            "the core knows the token it answered",
        );
        let mut wm = core.named("wm");
        wm.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["surface.created"]}),
        );
        let mut client = Wayland::connect(&mut core);
        let _top = client.activated_toplevel(&mut core, "run", &token, after_commit);
        let created: Vec<Value> = wm
            .events_now(&mut core)
            .into_iter()
            .filter(|e| e["params"]["event"] == "surface.created")
            .collect();
        assert_eq!(created.len(), 1, "one toplevel was created");
        assert_eq!(
            created[0]["params"]["token"], token,
            "after commit: {after_commit}"
        );
        assert!(created[0]["params"]["pid"].as_u64().is_some());
        assert!(
            !core
                .state
                .activation_state
                .tokens()
                .any(|(t, _)| t.as_str() == token),
            "one use ends the token",
        );
    }

    /// A program that never activates leaves its token behind. The core
    /// forgets such unused tokens of no client once they grow old; a
    /// fresh token and the tokens of clients stay.
    #[test]
    fn an_unused_activation_token_is_forgotten() {
        let mut core = Core::new();
        let mut ctl = core.named("choractl");
        let answer = ctl.ok(&mut core, "activation.token", Value::Null);
        let token = answer["token"]
            .as_str()
            .expect("an activation token")
            .to_owned();
        core.state.forget_unused_tokens(Duration::from_secs(30));
        assert_eq!(
            core.state.activation_state.tokens().count(),
            1,
            "a fresh token survives the sweep",
        );
        std::thread::sleep(Duration::from_millis(5));
        core.state.forget_unused_tokens(Duration::ZERO);
        assert_eq!(
            core.state.activation_state.tokens().count(),
            0,
            "{token} of a program that never activated stayed",
        );
    }

    #[test]
    fn a_line_that_is_too_long_gets_an_error_before_the_close() {
        let mut core = Core::new();
        let mut client = core.connect();
        let mut line = vec![b'x'; MAX_LINE + (64 << 10)];
        line.push(b'\n');
        let mut sent = 0;
        let mut responses = Vec::new();
        while sent < line.len() {
            match client.write(&line[sent..]) {
                Ok(n) => sent += n,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                Err(err) => panic!("the core must read the whole line: {err}"),
            }
            core.dispatch();
            responses.extend(client.responses());
        }
        for _ in 0..3 {
            core.dispatch();
            responses.extend(client.responses());
        }
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0]["error"]["code"], RpcError::LIMIT_REACHED);
        assert_eq!(
            responses[0]["error"]["message"],
            "the request is longer than 1048576 bytes, the limit of a line; \
             the core closes the connection"
        );
        assert!(client.closed(&mut core));
    }

    #[test]
    fn an_answer_beyond_the_output_limit_is_an_error() {
        let mut core = Core::new();
        let mut c = core.named("c");
        let value = "x".repeat(1000);
        let meta: serde_json::Map<String, Value> =
            (0..4).map(|i| (format!("k{i}"), json!(value))).collect();
        for batch in 0..250 {
            let nodes: Vec<Value> = (0..10)
                .map(|i| {
                    json!({"create": {"id": format!("g{}", batch * 10 + i),
                    "kind": "group", "parent": "root", "props": {"meta": meta}}})
                })
                .collect();
            c.ok(&mut core, "scene.apply", json!({"commands": nodes}));
        }
        let mut asker = core.named("asker");
        let (error_code, message) = asker.fails(&mut core, "scene.query", json!({}));
        assert_eq!(error_code, RpcError::LIMIT_REACHED);
        assert!(message.contains("does not fit in the output"), "{message}");
        assert!(!asker.closed(&mut core));
        assert_eq!(ids(&mut asker, &mut core, "id:g7"), ["g7"]);
    }

    #[test]
    fn a_connection_without_hello_closes_at_its_deadline() {
        let mut core = Core::new();
        let mut silent = core.connect();
        let mut named = core.named("wm");
        core.advance(HELLO_TIMEOUT.saturating_sub(Duration::from_millis(1)));
        assert!(!silent.closed(&mut core));
        core.advance(Duration::from_millis(1));
        assert!(silent.closed(&mut core));
        assert!(!named.closed(&mut core));
    }

    /// The error line of a refused connection.
    fn refused(core: &mut Core) -> String {
        let (mut ours, theirs) = UnixStream::pair().unwrap();
        assert!(core.state.ipc_accept(theirs).is_err());
        let mut answer = String::new();
        ours.read_to_string(&mut answer).unwrap();
        let answer: Value = serde_json::from_str(&answer).unwrap();
        assert_eq!(answer["error"]["code"], RpcError::LIMIT_REACHED);
        answer["error"]["message"].as_str().unwrap().to_owned()
    }

    /// Gives all connections other processes, as if each came from its
    /// own.
    fn as_other_processes(core: &mut Core) {
        for (id, conn) in &mut core.state.ipc.conns {
            conn.pid = Pid::from_raw(1_000_000 + i32::try_from(*id).unwrap());
        }
    }

    #[test]
    fn the_core_refuses_connections_beyond_its_limits_with_a_reason() {
        let mut core = Core::new();
        let mut clients: Vec<Client> = (0..MAX_PER_PROCESS).map(|_| core.connect()).collect();
        assert_eq!(
            refused(&mut core),
            "too many connections: this process holds 16"
        );
        as_other_processes(&mut core);
        while clients.len() < MAX_CONNECTIONS - KEPT_FOR_NEW_PROCESSES {
            clients.push(core.connect());
            as_other_processes(&mut core);
        }
        // A process without a connection gets a kept slot, but no second.
        clients.push(core.connect());
        assert_eq!(
            refused(&mut core),
            "too many connections: the last 8 are kept for processes without one"
        );
        while clients.len() < MAX_CONNECTIONS {
            as_other_processes(&mut core);
            clients.push(core.connect());
        }
        assert_eq!(
            refused(&mut core),
            "too many connections: the core holds 128"
        );
        drop(clients);
        core.dispatch();
        let (_, theirs) = UnixStream::pair().unwrap();
        assert!(core.state.ipc_accept(theirs).is_ok());
    }

    /// A refused client has often already sent its `hello`. Closing with
    /// unread bytes sends an RST and eats the error line, so the core
    /// reads before it closes.
    #[test]
    fn a_refused_connection_reads_its_error_line_after_its_hello() {
        let mut core = Core::new();
        let _full: Vec<Client> = (0..MAX_PER_PROCESS).map(|_| core.connect()).collect();
        let (mut ours, theirs) = UnixStream::pair().unwrap();
        ours.write_all(b"hello\n").unwrap();
        assert!(core.state.ipc_accept(theirs).is_err());
        let mut answer = String::new();
        ours.read_to_string(&mut answer).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&answer).unwrap()["error"]["code"],
            RpcError::LIMIT_REACHED
        );
    }

    #[test]
    fn reserves_of_a_connection_add_up_and_go_with_it() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut panel = core.named("panel");
        let mut dock = core.named("dock");
        let top = json!({"output": "HDMI-1", "edges": {"top": 30}});
        panel.ok(&mut core, "output.reserve", top);
        let both = json!({"edges": {"top": 10, "left": 40}});
        let info = dock.ok(&mut core, "output.reserve", both);
        assert_eq!(info["name"], "HDMI-1", "the first output without a name");
        let (_, message) = dock.fails(&mut core, "output.reserve", json!({"output": "DP-1"}));
        assert_eq!(message, "no output Some(\"DP-1\")");
        let edges = core.state.ipc.reserved("HDMI-1");
        assert_eq!((edges.top, edges.left, edges.bottom), (40, 40, 0));
        drop(panel);
        core.dispatch();
        assert_eq!(core.state.ipc.reserved("HDMI-1").top, 10);
        assert_eq!(core.state.ipc.reserved("DP-1"), Edges::default());
    }

    // A claim of a keep_nodes client outlives it; new toplevels must not
    // stay reserved for a client that is gone.
    #[test]
    fn a_claim_of_a_gone_client_reserves_nothing() {
        let mut core = Core::new();
        core.add_output("A", (800, 600));
        let mut gone = core.connect();
        gone.ok(
            &mut core,
            "hello",
            json!({"name": "wmb", "keep_nodes": true}),
        );
        let group = json!({"commands": [{"create": {"id": "g", "kind": "group",
            "parent": "root", "props": {"claim": [0.0, 0.0, 800.0, 600.0]}}}]});
        gone.ok(&mut core, "scene.apply", group);
        drop(gone);
        core.dispatch();
        let mut other = core.named("xp");
        let mut wayland = Wayland::connect(&mut core);
        core.state.pointer_layout = (100.0, 100.0).into();
        wayland.toplevel(&mut core, "foot");
        assert_eq!(
            reserved_for(&core, "s1"),
            None,
            "reserved for a gone client"
        );
        other.ok(&mut core, "scene.apply", attach_s1("root"));
    }

    #[test]
    fn the_reservations_end_with_the_last_claim() {
        let mut core = Core::new();
        core.add_output("A", (800, 600));
        let mut a = claimer(&mut core, "a", [0.0, 0.0, 800.0, 600.0]);
        let mut wayland = Wayland::connect(&mut core);
        core.state.pointer_layout = (100.0, 100.0).into();
        wayland.toplevel(&mut core, "foot");
        assert_eq!(reserved_for(&core, "s1").as_deref(), Some("a"));
        let remove = json!({"commands": [{"remove": {"id": "a"}}]});
        a.ok(&mut core, "scene.apply", remove);
        assert_eq!(reserved_for(&core, "s1"), None);
    }

    // A claim goes with null.
    #[test]
    fn a_claim_goes_with_null() {
        let mut core = Core::new();
        let mut a = claimer(&mut core, "a", [0.0, 0.0, 10.0, 10.0]);
        let clear = json!({"commands": [{"update": {"id": "a", "props": {"claim": null}}}]});
        a.ok(&mut core, "scene.apply", clear);
        assert_eq!(core.state.scene.node("a").unwrap().props.claim, None);
    }

    #[test]
    fn a_claim_made_again_in_the_same_transaction_keeps_the_reservations() {
        let mut core = Core::new();
        core.add_output("A", (800, 600));
        let mut a = claimer(&mut core, "a", [0.0, 0.0, 800.0, 600.0]);
        let mut wayland = Wayland::connect(&mut core);
        core.state.pointer_layout = (100.0, 100.0).into();
        wayland.toplevel(&mut core, "foot");
        let again = json!({"commands": [{"remove": {"id": "a"}},
            {"create": {"id": "a", "kind": "group", "parent": "root",
                "props": {"claim": [0.0, 0.0, 800.0, 600.0]}}}]});
        a.ok(&mut core, "scene.apply", again);
        assert_eq!(reserved_for(&core, "s1").as_deref(), Some("a"));
    }

    #[test]
    fn a_window_manager_hears_when_another_takes_the_focus() {
        let mut core = Core::new();
        let mut wayland = Wayland::connect(&mut core);
        wayland.toplevel(&mut core, "one");
        wayland.toplevel(&mut core, "two");
        let mut a = core.named("a");
        let mut b = core.named("b");
        b.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["focus.changed"]}),
        );
        let take = |s: &str| {
            json!({"commands": [{"attach": {"surface": s, "parent": "root"}},
            {"focus": {"surface": s}}]})
        };
        a.ok(&mut core, "scene.apply", take("s1"));
        let heard = |b: &mut Client, core: &mut Core| -> Vec<Value> {
            b.events_now(core)
                .into_iter()
                .map(|e| {
                    let mut params = e["params"].clone();
                    if let Some(params) = params.as_object_mut() {
                        params.remove("seq");
                    }
                    params
                })
                .collect()
        };
        assert_eq!(
            heard(&mut b, &mut core),
            [json!({"event": "focus.changed", "surface": "s1"})]
        );
        b.ok(&mut core, "scene.apply", take("s2"));
        assert_eq!(
            heard(&mut b, &mut core),
            [json!({"event": "focus.changed", "surface": "s2"})]
        );
        let clear = json!({"commands": [{"focus": {}}]});
        b.ok(&mut core, "scene.apply", clear);
        assert_eq!(
            heard(&mut b, &mut core),
            [json!({"event": "focus.changed", "surface": null})]
        );
    }

    // A dialog goes to the window manager of its parent, not by the pointer.
    #[test]
    fn a_child_toplevel_goes_to_the_manager_of_its_parent() {
        let mut core = Core::new();
        core.add_output("A", (800, 600));
        let mut a = claimer(&mut core, "a", [0.0, 0.0, 400.0, 600.0]);
        let _b = claimer(&mut core, "b", [400.0, 0.0, 400.0, 600.0]);
        let mut wayland = Wayland::connect(&mut core);
        core.state.pointer_layout = (100.0, 100.0).into();
        let parent = wayland.toplevel(&mut core, "app");
        a.ok(&mut core, "scene.apply", attach_s1("a"));
        core.state.pointer_layout = (600.0, 100.0).into();
        wayland.child(&mut core, &parent);
        assert_eq!(reserved_for(&core, "s2").as_deref(), Some("a"));
    }
}
