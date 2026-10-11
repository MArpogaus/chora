//! The IPC socket: JSON-RPC 2.0, one message per line, served by the event
//! loop of the core (D29).

use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
};

use chora_scene::{
    Binding, Event, Selector,
    ipc::{Apply, Grab, Hello, Message, Query, Request, RpcError, Subscribe},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use smithay::{
    input::keyboard::{Keysym, ModifiersState, xkb},
    reexports::calloop::{Interest, Mode, PostAction, RegistrationToken, generic::Generic},
};

use crate::state::State;

/// The connections of the IPC socket.
#[derive(Default)]
pub struct Ipc {
    path: Option<PathBuf>,
    conns: HashMap<u64, Conn>,
    next: u64,
    grabs: Vec<(Binding, Keysym, u64)>,
}

struct Conn {
    stream: UnixStream,
    input: Vec<u8>,
    output: Vec<u8>,
    name: Option<String>,
    keep_nodes: bool,
    events: Option<Vec<String>>,
    token: RegistrationToken,
}

/// Opens the socket in `$XDG_RUNTIME_DIR/chora/` and returns its path.
pub fn listen(state: &mut State) -> io::Result<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR is not set"))?;
    let dir = Path::new(&runtime).join("chora");
    std::fs::create_dir_all(&dir)?;
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    let mut path = dir.join(&state.socket_name);
    path.set_extension("sock");
    match std::fs::remove_file(&path) {
        Err(err) if err.kind() != ErrorKind::NotFound => return Err(err),
        _ => {}
    }
    let listener = UnixListener::bind(&path)?;
    listener.set_nonblocking(true)?;
    state
        .loop_handle
        .insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            |_, listener, state| {
                loop {
                    match listener.accept() {
                        Ok((stream, _)) => state.ipc_accept(stream)?,
                        Err(err) if err.kind() == ErrorKind::WouldBlock => {
                            return Ok(PostAction::Continue);
                        }
                        Err(err) => return Err(err),
                    }
                }
            },
        )
        .map_err(|err| io::Error::other(err.to_string()))?;
    state.ipc.path = Some(path.clone());
    Ok(path)
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
        let line = line(&Message::event(event));
        let name = event.name();
        for conn in self.conns.values_mut() {
            let wanted = conn
                .events
                .as_ref()
                .is_some_and(|e| e.is_empty() || e.contains(&name));
            if wanted {
                conn.output.extend_from_slice(&line);
            }
        }
    }

    /// Sends an event to one connection, subscribed or not.
    pub fn send_to(&mut self, conn: u64, event: &Event) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.output.extend_from_slice(&line(&Message::event(event)));
        }
    }

    /// Sends a response to one connection.
    pub fn respond(&mut self, conn: u64, message: &Message) {
        if let Some(c) = self.conns.get_mut(&conn) {
            c.output.extend_from_slice(&line(message));
        }
    }

    /// The connection that grabbed the pressed binding, and the binding.
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
    /// connections that failed.
    pub fn flush(&mut self) -> Vec<u64> {
        let mut failed = Vec::new();
        for (id, conn) in &mut self.conns {
            while !conn.output.is_empty() {
                match conn.stream.write(&conn.output) {
                    Ok(0) => {
                        failed.push(*id);
                        break;
                    }
                    Ok(n) => {
                        conn.output.drain(..n);
                    }
                    Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                    Err(_) => {
                        failed.push(*id);
                        break;
                    }
                }
            }
        }
        failed
    }
}

fn line(message: &Message) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(message).expect("messages always serialize");
    bytes.push(b'\n');
    bytes
}

impl State {
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
            output: Vec::new(),
            name: None,
            keep_nodes: false,
            events: None,
            token,
        };
        self.ipc.conns.insert(id, conn);
        Ok(())
    }

    fn ipc_readable(&mut self, id: u64, mut reader: &UnixStream) -> PostAction {
        let mut buf = [0; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    self.ipc_close(id, false);
                    return PostAction::Remove;
                }
                Ok(n) => self
                    .ipc
                    .conns
                    .get_mut(&id)
                    .expect("open")
                    .input
                    .extend_from_slice(&buf[..n]),
                Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => {
                    self.ipc_close(id, false);
                    return PostAction::Remove;
                }
            }
        }
        while let Some(line) = self.ipc_next_line(id) {
            self.ipc_handle_line(id, &line);
        }
        PostAction::Continue
    }

    fn ipc_next_line(&mut self, id: u64) -> Option<Vec<u8>> {
        let input = &mut self.ipc.conns.get_mut(&id)?.input;
        let end = input.iter().position(|b| *b == b'\n')?;
        let mut line: Vec<u8> = input.drain(..=end).collect();
        line.pop();
        Some(line)
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
            "hello" => params_of(params).map(|h| self.ipc_hello(conn, h)),
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
            "output.list" => Ok(serde_json::json!(
                self.output
                    .iter()
                    .map(crate::winit::output_info)
                    .collect::<Vec<_>>()
            )),
            other => Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("no method {other:?}"),
            )),
        })
    }

    fn ipc_hello(&mut self, conn: u64, hello: Hello) -> Value {
        let known = self
            .ipc
            .conns
            .values()
            .any(|c| c.name.as_deref() == Some(&hello.name));
        if let Some(c) = self.ipc.conns.get_mut(&conn) {
            c.name = Some(hello.name.clone());
            c.keep_nodes = hello.keep_nodes;
        }
        if !known {
            self.emit(&Event::ClientConnected { name: hello.name });
        }
        Value::Null
    }

    fn query(&self, selector: &str) -> Result<Value, RpcError> {
        let selector: Selector = selector
            .parse()
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, format!("{e}")))?;
        Ok(serde_json::json!(self.scene.query(&selector)))
    }

    fn ipc_grab(&mut self, conn: u64, binding: Binding) -> Result<Value, RpcError> {
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
        self.ipc.grabs.push((binding, sym, conn));
        Ok(Value::Null)
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
            for surface in self.scene.disconnect(&name) {
                self.emit(&Event::SurfaceDetached { surface });
            }
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
