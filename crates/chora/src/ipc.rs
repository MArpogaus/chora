//! The IPC socket: JSON-RPC 2.0, one message per line, served by the event
//! loop of the core (D29).
//!
//! Limits keep a buggy client from exhausting the core: a number of
//! connections, a time until `hello`, a line length, an output buffer per
//! connection, and a read size per wakeup, so that one busy client cannot
//! stall the event loop.

use std::{
    collections::{HashMap, HashSet},
    io::{self, ErrorKind, Read, Write},
    os::unix::{
        fs::{DirBuilderExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Event, Selector,
    ipc::{Apply, Edges, Grab, Hello, Message, Query, Request, RpcError, Subscribe},
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
    reexports::rustix::{net::sockopt::socket_peercred, process::Pid},
};

use crate::{captured::Processes, state::State, uploads::Uploads};

const MAX_CONNECTIONS: usize = 128;
const MAX_LINE: usize = 1 << 20;
const MAX_OUTPUT: usize = 8 << 20;
const READ_PER_WAKEUP: usize = 64 << 10;
/// How long a new connection may wait before its `hello`, so that idle
/// connections cannot take every slot.
const HELLO_TIMEOUT: Duration = Duration::from_secs(5);

/// The connections of the IPC socket.
#[derive(Default)]
pub struct Ipc {
    path: Option<PathBuf>,
    /// The files of `assets.put`.
    pub uploads: Uploads,
    /// The client names whose `plugins.restart` waits for the old process.
    pub restarting: HashSet<String>,
    /// The output of the programs of `process.spawn` with `capture`.
    pub processes: Processes,
    listener: Option<RegistrationToken>,
    conns: HashMap<u64, Conn>,
    next: u64,
    grabs: Vec<(Binding, Keysym, u64)>,
}

struct Conn {
    stream: UnixStream,
    input: Vec<u8>,
    output: Vec<u8>,
    /// The output passed `MAX_OUTPUT`; the connection closes.
    overflow: bool,
    name: Option<String>,
    /// The deadline of `hello`.
    hello_by: Instant,
    keep_nodes: bool,
    events: Option<Vec<String>>,
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
    let dir = private_dir(Path::new(&runtime))?;
    let mut path = dir.join(&state.socket_name);
    path.set_extension("sock");
    let mut assets = path.clone();
    assets.set_extension("assets");
    state.ipc.uploads.open(assets)?;
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
    /// The path of the socket, once it listens.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The connections of a client name and a process that reached the end
    /// of their streams and only wait for answers.
    pub fn ended(&self, name: &str, pid: Pid) -> Vec<u64> {
        self.conns
            .iter()
            .filter(|(_, c)| c.name.as_deref() == Some(name) && c.reader.is_none())
            .filter(|(_, c)| socket_peercred(&c.stream).is_ok_and(|p| p.pid == pid))
            .map(|(id, _)| *id)
            .collect()
    }

    /// The processes of the connections of a client name, each once; `None`
    /// for a process that the core cannot see.
    pub fn peer_pids(&self, name: &str) -> Vec<Option<Pid>> {
        let mut pids = Vec::new();
        for conn in self.conns.values() {
            if conn.name.as_deref() != Some(name) {
                continue;
            }
            let pid = socket_peercred(&conn.stream).ok().map(|c| c.pid);
            if !pids.contains(&pid) {
                pids.push(pid);
            }
        }
        pids
    }

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
    pub fn broadcast(&mut self, event: &Event) {
        let message = Message::event(event);
        let name = message.params.as_ref().and_then(|p| p["event"].as_str());
        let line = line(&message);
        for conn in self.conns.values_mut() {
            let wanted = conn
                .events
                .as_ref()
                .is_some_and(|e| e.is_empty() || e.iter().any(|n| name == Some(n.as_str())));
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

    /// The connections without a name whose deadline of `hello` passed.
    pub fn silent(&self, now: Instant) -> Vec<u64> {
        self.conns
            .iter()
            .filter(|(_, c)| c.name.is_none() && c.hello_by <= now)
            .map(|(id, _)| *id)
            .collect()
    }

    /// The client name of a connection.
    pub fn name(&self, conn: u64) -> Option<&str> {
        self.conns.get(&conn)?.name.as_deref()
    }

    /// The ids of all connections.
    pub fn conn_ids(&self) -> Vec<u64> {
        self.conns.keys().copied().collect()
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
    let mut bytes = serde_json::to_vec(message).expect("messages always serialize");
    bytes.push(b'\n');
    bytes
}

impl State {
    /// Accepts every waiting connection. When accepting fails, for example
    /// because no file descriptors are left, the listener pauses for a
    /// second instead of stopping the compositor. Returns the error to log.
    fn ipc_accept_all(&mut self, listener: &UnixListener) -> Option<String> {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(err) = self.ipc_accept(stream) {
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

    /// Registers a connection, or refuses it beyond `MAX_CONNECTIONS`.
    pub fn ipc_accept(&mut self, stream: UnixStream) -> Result<(), String> {
        fn failed(err: impl std::fmt::Display) -> String {
            format!("a connection failed: {err}")
        }
        if self.ipc.conns.len() >= MAX_CONNECTIONS {
            return Err("too many connections; refusing one".into());
        }
        stream.set_nonblocking(true).map_err(failed)?;
        // The event loop watches a copy; reads go through `Conn::stream`.
        let watched = stream.try_clone().map_err(failed)?;
        let id = self.ipc.next;
        self.ipc.next += 1;
        let hello_by = self.time.now() + HELLO_TIMEOUT;
        self.expire_at(hello_by);
        let token = self
            .loop_handle
            .insert_source(
                Generic::new(watched, Interest::READ, Mode::Level),
                move |_, _, state| Ok(state.ipc_readable(id)),
            )
            .map_err(failed)?;
        let conn = Conn {
            stream,
            input: Vec::new(),
            output: Vec::new(),
            overflow: false,
            name: None,
            hello_by,
            keep_nodes: false,
            events: None,
            reserves: HashMap::new(),
            reader: Some(token),
        };
        self.ipc.conns.insert(id, conn);
        Ok(())
    }

    /// Reads at most `READ_PER_WAKEUP` bytes and handles the complete
    /// lines. After the end of the stream, the requests that arrived still
    /// get their answers, also from waiting transactions.
    fn ipc_readable(&mut self, id: u64) -> PostAction {
        let open = self.ipc.conns.get_mut(&id).is_some_and(Conn::read);
        while let Some(line) = self.ipc.conns.get_mut(&id).and_then(Conn::next_line) {
            self.ipc_handle_line(id, &line);
        }
        let Some(conn) = self.ipc.conns.get_mut(&id) else {
            return PostAction::Remove;
        };
        let too_long = conn.input.len() > MAX_LINE;
        if open && !too_long {
            return PostAction::Continue;
        }
        conn.input = Vec::new();
        conn.reader = None;
        if too_long {
            let error = RpcError::new(RpcError::INVALID_PARAMS, "line too long");
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
            "input.ungrab" => {
                params_of::<Grab>(params).and_then(|g| self.ipc_ungrab(conn, &g.binding))
            }
            "output.list" => Ok(serde_json::json!(self.output_info())),
            "output.reserve" => params_of(params).and_then(|r| self.ipc_reserve(conn, &r)),
            "output.configure" => {
                params_of(params).and_then(|c| self.ipc_output_configure(conn, &c))
            }
            "clients.list" => Ok(self.clients()),
            "grants.set" | "grants.revoke" | "grants.list" => {
                self.ipc_grants(&request.method, params)
            }
            "screen.capture" => self.ipc_capture(conn, params),
            "input.inject" => self.ipc_inject(conn, params),
            "assets.put" | "assets.remove" => self.ipc_assets(conn, &request.method, params),
            "process.spawn" | "process.wait" | "plugins.restart" => {
                return self.ipc_process(conn, id, &request.method, params);
            }
            "methods.export" | "methods.call" | "methods.return" => {
                return self.ipc_methods(conn, id, &request.method, params);
            }
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
        let sym = xkb::keysym_from_name(&binding.key, xkb::KEYSYM_CASE_INSENSITIVE);
        if sym.raw() == 0 {
            return Err(rejected(format!("unknown key {:?}", binding.key)));
        }
        if m.ctrl && m.alt && is_safety_key(sym) {
            return Err(rejected(format!("{binding} belongs to the safety layer")));
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

    /// Ends a grab of a connection (D45).
    fn ipc_ungrab(&mut self, conn: u64, binding: &Binding) -> Result<Value, RpcError> {
        let sym = xkb::keysym_from_name(&binding.key, xkb::KEYSYM_CASE_INSENSITIVE);
        let before = self.ipc.grabs.len();
        self.ipc
            .grabs
            .retain(|(b, s, c)| !(*c == conn && b.mods == binding.mods && *s == sym));
        if self.ipc.grabs.len() == before {
            return Err(rejected(format!("{binding} is no grab of this connection")));
        }
        Ok(Value::Null)
    }

    /// Closes a connection. Without `keep_nodes`, the nodes and the assets
    /// of its name go when the last connection of the name closes.
    pub fn ipc_close(&mut self, conn: u64) {
        let Some(closed) = self.ipc.conns.remove(&conn) else {
            return;
        };
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
            let effects = self.scene.disconnect(&name);
            self.execute(effects);
            self.ipc.uploads.drop_client(&name);
        }
        self.ipc.processes.drop_client(&name);
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

pub fn rejected(message: impl Into<String>) -> RpcError {
    RpcError::new(RpcError::REJECTED, message)
}

/// Whether Ctrl+Alt with a key is a binding of the safety layer (D23):
/// `BackSpace` quits, `Escape` resets, `F1` to `F12` switch the VT.
fn is_safety_key(sym: xkb::Keysym) -> bool {
    use xkb::Keysym;
    sym == Keysym::BackSpace
        || sym == Keysym::Escape
        || (Keysym::F1.raw()..=Keysym::F12.raw()).contains(&sym.raw())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::testing::{Client, Core};

    const REJECTED: i64 = RpcError::REJECTED;

    fn rect(id: &str) -> Value {
        json!({"commands": [{"create": {"id": id, "kind": "rect", "parent": "root",
            "props": {"size": [10, 10], "color": [1, 0, 0, 1]}}}]})
    }

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

    #[test]
    fn hello_names_a_connection_once() {
        let mut core = Core::new();
        let mut client = core.named("wm");
        assert_eq!(core.state.ipc.name(1), None);
        assert_eq!(core.state.ipc.name(0), Some("wm"));
        let (error_code, message) = client.fails(&mut core, "hello", json!({"name": "other"}));
        assert_eq!(
            (error_code, message.as_str()),
            (REJECTED, "hello was sent already")
        );
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
        assert_eq!(error_code, REJECTED);
        assert_eq!(message, "\"wm\" is in use with keep_nodes true");
        second.ok(
            &mut core,
            "hello",
            json!({"name": "wm", "keep_nodes": true}),
        );
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

    #[test]
    fn a_grab_needs_a_modifier_and_spares_the_safety_layer() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let grab = |binding: &str| json!({"binding": binding});
        let normal = wm.ok(&mut core, "input.grab", grab("Super+Shift+Return"));
        assert_eq!(normal, "Shift+Super+Return");
        for (binding, message) in [
            ("Shift+a", "Shift+a needs Ctrl, Alt or Super"),
            (
                "Ctrl+Alt+BackSpace",
                "Ctrl+Alt+BackSpace belongs to the safety layer",
            ),
            ("Ctrl+Alt+F3", "Ctrl+Alt+F3 belongs to the safety layer"),
            ("Super+NoSuchKey", "unknown key \"NoSuchKey\""),
        ] {
            let (error_code, got) = wm.fails(&mut core, "input.grab", grab(binding));
            assert_eq!((error_code, got.as_str()), (REJECTED, message));
        }
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
            Some((0, "Super+a".into()))
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
    fn a_line_that_is_too_long_ends_the_connection() {
        let mut core = Core::new();
        let mut client = core.connect();
        let chunk = vec![b'x'; 32 << 10];
        let mut sent = 0;
        while sent <= MAX_LINE {
            match client.write(&chunk) {
                Ok(n) => sent += n,
                Err(err) if err.kind() == ErrorKind::WouldBlock => core.dispatch(),
                Err(err) => panic!("{err}"),
            }
        }
        let mut responses = Vec::new();
        for _ in 0..(MAX_LINE / READ_PER_WAKEUP + 2) {
            core.dispatch();
            responses.extend(client.responses());
        }
        assert_eq!(responses[0]["error"]["message"], "line too long");
        assert!(client.closed(&mut core));
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

    #[test]
    fn the_core_refuses_connections_beyond_its_limit() {
        let mut core = Core::new();
        let clients: Vec<Client> = (0..MAX_CONNECTIONS).map(|_| core.connect()).collect();
        let (_, theirs) = UnixStream::pair().unwrap();
        assert!(core.state.ipc_accept(theirs).is_err());
        drop(clients);
        core.dispatch();
        let (_, theirs) = UnixStream::pair().unwrap();
        assert!(core.state.ipc_accept(theirs).is_ok());
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
}
