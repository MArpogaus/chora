//! The IPC socket: JSON-RPC 2.0, one message per line, served by the event
//! loop of the core (D29).
//!
//! Limits keep a buggy client from exhausting the core: a number of
//! connections, a line length, an output buffer per connection, and a read
//! size per wakeup, so that one busy client cannot stall the event loop.

use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read, Write},
    os::unix::{
        fs::{DirBuilderExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    time::Duration,
};

use chora_scene::{
    Binding, Effect, Event, Selector,
    ipc::{Apply, Grab, Hello, Message, Query, Request, RpcError, Subscribe},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use smithay::{
    input::keyboard::{Keysym, ModifiersState, xkb},
    reexports::calloop::{
        Interest, Mode, PostAction, RegistrationToken,
        generic::Generic,
        timer::{TimeoutAction, Timer},
    },
};

use crate::state::State;

const MAX_CONNECTIONS: usize = 128;
const MAX_LINE: usize = 1 << 20;
const MAX_OUTPUT: usize = 8 << 20;
const READ_PER_WAKEUP: usize = 64 << 10;

/// The connections of the IPC socket.
#[derive(Default)]
pub struct Ipc {
    path: Option<PathBuf>,
    listener: Option<RegistrationToken>,
    conns: HashMap<u64, Conn>,
    next: u64,
    grabs: Vec<(Binding, Keysym, u64)>,
}

struct Conn {
    stream: UnixStream,
    input: Vec<u8>,
    /// How far `input` holds no newline.
    scanned: usize,
    output: Vec<u8>,
    /// The output passed `MAX_OUTPUT`; the connection closes.
    overflow: bool,
    name: Option<String>,
    keep_nodes: bool,
    events: Option<Vec<String>>,
    token: RegistrationToken,
}

/// Opens the socket in `$XDG_RUNTIME_DIR/chora/` and returns its path.
pub fn listen(state: &mut State) -> io::Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR is not set"))?;
    let dir = private_dir(Path::new(&runtime))?;
    let mut path = dir.join(&state.socket_name);
    path.set_extension("sock");
    match std::fs::remove_file(&path) {
        Err(err) if err.kind() != ErrorKind::NotFound => return Err(err),
        _ => {}
    }
    let listener = UnixListener::bind(&path)?;
    listener.set_nonblocking(true)?;
    let token = state
        .loop_handle
        .insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            |_, listener, state| {
                if let Some(error) = state.ipc_accept_all(listener) {
                    tracing::warn!("IPC socket: {error}");
                }
                Ok(PostAction::Continue)
            },
        )
        .map_err(|err| io::Error::other(err.to_string()))?;
    state.ipc.listener = Some(token);
    state.ipc.path = Some(path.clone());
    Ok(path)
}

/// Creates `RUNTIME/chora` with mode 0700 and checks that it is a real
/// directory of the owner of `RUNTIME`.
fn private_dir(runtime: &Path) -> io::Result<PathBuf> {
    let dir = runtime.join("chora");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .recursive(true)
        .create(&dir)?;
    let meta = std::fs::symlink_metadata(&dir)?;
    if !meta.is_dir() || meta.uid() != std::fs::metadata(runtime)?.uid() {
        return Err(io::Error::other(format!(
            "{} is no directory of the runtime dir owner",
            dir.display()
        )));
    }
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    Ok(dir)
}

impl Drop for Ipc {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl Ipc {
    /// Sends an event to every connection that subscribed to it.
    pub fn broadcast(&mut self, event: &Event) {
        let message = Message::event(event);
        let name = message.params.as_ref().map(|p| p["event"].clone());
        let line = line(&message);
        for conn in self.conns.values_mut() {
            let wanted = conn.events.as_ref().is_some_and(|e| {
                e.is_empty()
                    || e.iter()
                        .any(|n| name.as_ref().and_then(Value::as_str) == Some(n))
            });
            if wanted {
                conn.push(&line);
            }
        }
    }

    /// Sends an event to one connection, subscribed or not.
    pub fn send_to(&mut self, conn: u64, event: &Event) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.push(&line(&Message::event(event)));
        }
    }

    /// Sends a response to one connection.
    pub fn respond(&mut self, conn: u64, message: &Message) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.push(&line(message));
        }
    }

    /// The connection that grabbed the pressed binding, and the binding in
    /// its normal form.
    pub fn grab_for(&self, mods: &ModifiersState, syms: &[Keysym]) -> Option<(u64, String)> {
        self.grabs
            .iter()
            .find(|(b, sym, _)| {
                let m = b.mods;
                (m.ctrl, m.alt, m.shift, m.logo) == (mods.ctrl, mods.alt, mods.shift, mods.logo)
                    && syms.contains(sym)
            })
            .map(|(b, _, conn)| (*conn, b.to_string()))
    }

    /// The client name of a connection.
    pub fn name(&self, conn: u64) -> Option<&str> {
        self.conns.get(&conn)?.name.as_deref()
    }

    /// The ids of all connections.
    pub fn conn_ids(&self) -> Vec<u64> {
        self.conns.keys().copied().collect()
    }

    /// Writes the pending output of every connection. Returns the
    /// connections that failed or overflowed.
    fn flush(&mut self) -> Vec<u64> {
        self.conns
            .iter_mut()
            .filter_map(|(id, conn)| (!conn.flush()).then_some(*id))
            .collect()
    }
}

impl Conn {
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

    /// The next complete line, without its newline.
    fn next_line(&mut self) -> Option<Vec<u8>> {
        let Some(end) = self.input[self.scanned..].iter().position(|b| *b == b'\n') else {
            self.scanned = self.input.len();
            return None;
        };
        let mut line: Vec<u8> = self.input.drain(..=self.scanned + end).collect();
        self.scanned = 0;
        line.pop();
        Some(line)
    }
}

fn line(message: &Message) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(message).expect("messages always serialize");
    bytes.push(b'\n');
    bytes
}

/// The state of a connection after a read.
enum Stream {
    Open,
    Closed,
}

impl State {
    /// Accepts every waiting connection. When accepting fails, for example
    /// because no file descriptors are left, the listener pauses for a
    /// second instead of stopping the compositor. Returns the error to log.
    fn ipc_accept_all(&mut self, listener: &UnixListener) -> Option<String> {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(err) = self.ipc_take(stream) {
                        break Some(err);
                    }
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => break None,
                Err(err) => {
                    self.ipc_pause_listener();
                    break Some(format!("accepting fails, pausing for a second: {err}"));
                }
            }
        }
    }

    fn ipc_take(&mut self, stream: UnixStream) -> Result<(), String> {
        if self.ipc.conns.len() >= MAX_CONNECTIONS {
            return Err("too many connections; refusing one".into());
        }
        self.ipc_accept(stream)
            .map_err(|e| format!("a connection failed: {e}"))
    }

    fn ipc_pause_listener(&mut self) {
        let Some(token) = self.ipc.listener else {
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

    fn ipc_accept(&mut self, stream: UnixStream) -> io::Result<()> {
        stream.set_nonblocking(true)?;
        let reader = stream.try_clone()?;
        let id = self.ipc.next;
        self.ipc.next += 1;
        let token = self
            .loop_handle
            .insert_source(
                Generic::new(reader, Interest::READ, Mode::Level),
                move |_, reader, state| Ok(state.ipc_readable(id, reader)),
            )
            .map_err(|err| io::Error::other(err.to_string()))?;
        let conn = Conn {
            stream,
            input: Vec::new(),
            scanned: 0,
            output: Vec::new(),
            overflow: false,
            name: None,
            keep_nodes: false,
            events: None,
            token,
        };
        self.ipc.conns.insert(id, conn);
        Ok(())
    }

    /// Reads at most `READ_PER_WAKEUP` bytes and handles the complete
    /// lines. At the end of the stream, the lines that arrived still get
    /// their answers.
    fn ipc_readable(&mut self, id: u64, reader: &UnixStream) -> PostAction {
        let state = self.ipc_read(id, reader);
        while let Some(line) = self.ipc.conns.get_mut(&id).and_then(Conn::next_line) {
            self.ipc_handle_line(id, &line);
        }
        let too_long = self
            .ipc
            .conns
            .get(&id)
            .is_some_and(|c| c.input.len() > MAX_LINE);
        if too_long {
            let error = RpcError::new(RpcError::INVALID_PARAMS, "line too long");
            self.ipc.respond(id, &Message::error(Value::Null, error));
        }
        if matches!(state, Stream::Closed) || too_long {
            if let Some(conn) = self.ipc.conns.get_mut(&id) {
                conn.flush();
            }
            self.ipc_close(id, false);
            return PostAction::Remove;
        }
        PostAction::Continue
    }

    fn ipc_read(&mut self, id: u64, mut reader: &UnixStream) -> Stream {
        let mut buf = [0; 4096];
        let mut total = 0;
        while total < READ_PER_WAKEUP {
            let Some(conn) = self.ipc.conns.get_mut(&id) else {
                return Stream::Closed;
            };
            match reader.read(&mut buf) {
                Ok(0) => return Stream::Closed,
                Ok(n) => {
                    conn.input.extend_from_slice(&buf[..n]);
                    total += n;
                }
                Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return Stream::Closed,
            }
        }
        Stream::Open
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
            "scene.query" => params_of::<Query>(params).and_then(|q| self.query(&q.selector)),
            "scene.apply" => {
                let apply = match params_of::<Apply>(params) {
                    Ok(a) => a,
                    Err(e) => return Some(Err(e)),
                };
                let Some(owner) = self.ipc.name(conn).map(str::to_owned) else {
                    return Some(Err(rejected("send hello first")));
                };
                self.submit_transaction(conn, id, owner, apply.commands);
                return None;
            }
            "events.subscribe" => params_of::<Subscribe>(params).map(|s| {
                if let Some(c) = self.ipc.conns.get_mut(&conn) {
                    c.events = Some(s.events);
                }
                Value::Null
            }),
            "input.grab" => params_of::<Grab>(params).and_then(|g| self.ipc_grab(conn, g.binding)),
            "output.list" => Ok(serde_json::json!(self.output_info())),
            other => Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("no method {other:?}"),
            )),
        })
    }

    /// Names a connection. All connections of a name share one
    /// `keep_nodes` setting.
    fn ipc_hello(&mut self, conn: u64, hello: Hello) -> Result<Value, RpcError> {
        if self.ipc.name(conn).is_some() {
            return Err(rejected("hello was sent already"));
        }
        let peer = self
            .ipc
            .conns
            .values()
            .find(|c| c.name.as_deref() == Some(&hello.name));
        if let Some(peer) = peer.filter(|p| p.keep_nodes != hello.keep_nodes) {
            return Err(rejected(format!(
                "{:?} is in use with keep_nodes {}",
                hello.name, peer.keep_nodes
            )));
        }
        let known = peer.is_some();
        if let Some(c) = self.ipc.conns.get_mut(&conn) {
            c.name = Some(hello.name.clone());
            c.keep_nodes = hello.keep_nodes;
        }
        if !known {
            self.emit(&Event::ClientConnected { name: hello.name });
        }
        Ok(Value::Null)
    }

    fn query(&self, selector: &str) -> Result<Value, RpcError> {
        let selector: Selector = selector
            .parse()
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, format!("{e}")))?;
        Ok(serde_json::json!(self.scene.query(&selector)))
    }

    /// Grabs a binding for a named connection. A binding needs Ctrl, Alt or
    /// Super, so that typing still works, and Ctrl+Alt belongs to the safety
    /// layer. The result is the binding in the normal form of `input.key`.
    fn ipc_grab(&mut self, conn: u64, binding: Binding) -> Result<Value, RpcError> {
        if self.ipc.name(conn).is_none() {
            return Err(rejected("send hello first"));
        }
        let m = binding.mods;
        if !(m.ctrl || m.alt || m.logo) {
            return Err(rejected(format!("{binding} needs Ctrl, Alt or Super")));
        }
        if m.ctrl && m.alt {
            return Err(rejected(format!(
                "{binding}: Ctrl+Alt belongs to the safety layer"
            )));
        }
        let sym = xkb::keysym_from_name(&binding.key, xkb::KEYSYM_CASE_INSENSITIVE);
        if sym.raw() == 0 {
            return Err(rejected(format!("unknown key {:?}", binding.key)));
        }
        if self
            .ipc
            .grabs
            .iter()
            .any(|(b, s, _)| b.mods == binding.mods && *s == sym)
        {
            return Err(rejected(format!("{binding} is grabbed already")));
        }
        let normal = binding.to_string();
        self.ipc.grabs.push((binding, sym, conn));
        Ok(Value::String(normal))
    }

    /// Closes a connection. Without `keep_nodes`, the nodes of its name go
    /// when the last connection of the name closes. `remove_source` must be
    /// true unless the caller is the source of the connection itself.
    pub fn ipc_close(&mut self, conn: u64, remove_source: bool) {
        let Some(closed) = self.ipc.conns.remove(&conn) else {
            return;
        };
        if remove_source {
            self.loop_handle.remove(closed.token);
        }
        self.ipc.grabs.retain(|(_, _, c)| *c != conn);
        self.transactions.drop_conn(conn);
        let Some(name) = closed.name else { return };
        if self
            .ipc
            .conns
            .values()
            .any(|c| c.name.as_ref() == Some(&name))
        {
            return;
        }
        if !closed.keep_nodes {
            let detached = self.scene.disconnect(&name);
            self.execute(detached.into_iter().map(Effect::Detached).collect());
        }
        self.emit(&Event::ClientDisconnected { name });
    }

    /// Writes all pending IPC output and closes broken connections.
    pub fn ipc_flush(&mut self) {
        for conn in self.ipc.flush() {
            self.ipc_close(conn, true);
        }
    }
}

fn params_of<T: DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    let params = if params.is_null() {
        Value::Object(serde_json::Map::new())
    } else {
        params
    };
    serde_json::from_value(params)
        .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))
}

pub fn rejected(message: impl Into<String>) -> RpcError {
    RpcError::new(RpcError::REJECTED, message)
}
