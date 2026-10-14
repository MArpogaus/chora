//! A core for the unit tests: the event loop, the state, IPC clients on
//! socket pairs, and on request the headless renderer.

use std::{
    io::{ErrorKind, Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

use serde_json::{Value, json};
use smithay::{
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
};

use crate::state::State;

pub mod wayland;

/// The event loop and the state of a core without a backend.
pub struct Core {
    pub event_loop: EventLoop<'static, State>,
    pub state: State,
}

impl Core {
    pub fn new() -> Self {
        let event_loop = EventLoop::try_new().expect("an event loop");
        let display = Display::new().expect("a display");
        let mut state = State::new(display, event_loop.handle(), event_loop.get_signal());
        state.time.stop();
        Self { event_loop, state }
    }

    /// A core with the headless renderer, for captures.
    pub fn with_renderer() -> Self {
        let mut core = Self::new();
        let headless = crate::backend::headless::Headless::new().expect("a software renderer");
        core.state.backend = Some(crate::backend::Backend::Headless(Box::new(headless)));
        core
    }

    /// Runs one turn of the event loop without waiting.
    pub fn dispatch(&mut self) {
        self.event_loop
            .dispatch(Some(Duration::ZERO), &mut self.state)
            .expect("the event loop runs");
        self.state.after_events();
    }

    /// A new IPC connection.
    pub fn connect(&mut self) -> Client {
        let (ours, theirs) = UnixStream::pair().expect("a socket pair");
        self.state.ipc_accept(theirs).expect("the core accepts");
        ours.set_nonblocking(true).expect("a socket option");
        Client {
            stream: ours,
            input: Vec::new(),
            events: Vec::new(),
            next: 1,
        }
    }

    /// A new IPC connection after `hello` with a name.
    pub fn named(&mut self, name: &str) -> Client {
        let mut client = self.connect();
        client.ok(self, "hello", json!({"name": name}));
        client
    }

    /// Adds an output of a size, right of the others, with its global.
    pub fn add_output(&mut self, name: &str, size: (i32, i32)) -> Output {
        let output = Output::new(
            name.into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: "chora".into(),
                model: "test".into(),
                serial_number: String::new(),
            },
        );
        output.create_global::<State>(&self.state.display_handle);
        let mode = Mode {
            size: size.into(),
            refresh: 60_000,
        };
        output.change_current_state(Some(mode), None, None, None);
        output.set_preferred(mode);
        self.state.add_output(output.clone());
        output
    }

    /// Moves the time of the deadlines and runs what expired.
    pub fn advance(&mut self, by: Duration) {
        self.state.time.advance(by);
        self.state.expire();
        self.dispatch();
    }
}

/// The client side of an IPC connection. A drop shuts the socket down, so
/// that the core sees the end at once, also while a child that a test of
/// another thread forks holds a copy of the socket until its `exec`.
pub struct Client {
    stream: UnixStream,
    input: Vec<u8>,
    /// The events that arrived and that no test took yet.
    events: Vec<Value>,
    next: u64,
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

impl Client {
    /// Sends one raw line.
    pub fn send_line(&mut self, line: &[u8]) {
        self.stream.write_all(line).expect("the core reads");
        self.stream.write_all(b"\n").expect("the core reads");
    }

    /// Writes raw bytes as far as the socket takes them.
    pub fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.stream.write(bytes)
    }

    /// Sends a request and returns its id.
    pub fn send(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next;
        self.next += 1;
        let mut request = json!({"jsonrpc": "2.0", "id": id, "method": method});
        request["params"] = params;
        self.send_line(request.to_string().as_bytes());
        id
    }

    /// The messages that arrived, in order. `None` once the core closed
    /// the connection and all messages were read.
    pub fn read(&mut self) -> Option<Vec<Value>> {
        let mut buf = [0; 4096];
        let mut open = true;
        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => {
                    open = false;
                    break;
                }
                Ok(n) => self.input.extend_from_slice(&buf[..n]),
                Err(err) if err.kind() == ErrorKind::WouldBlock => break,
                Err(_) => {
                    open = false;
                    break;
                }
            }
        }
        let mut messages = Vec::new();
        while let Some(end) = self.input.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.input.drain(..=end).collect();
            messages.push(serde_json::from_slice(&line).expect("the core sends JSON"));
        }
        (open || !messages.is_empty()).then_some(messages)
    }

    /// Reads the messages and keeps the events. Returns the responses.
    pub fn responses(&mut self) -> Vec<Value> {
        let messages = self.read().unwrap_or_default();
        let (events, responses) = messages
            .into_iter()
            .partition(|m| m.get("method").is_some());
        let events: Vec<Value> = events;
        self.events.extend(events);
        responses
    }

    /// The answer of the request with this id, after one turn of the core.
    /// `None` while the answer waits.
    pub fn answer(&mut self, core: &mut Core, id: u64) -> Option<Result<Value, Value>> {
        core.dispatch();
        let response = self.responses().into_iter().find(|r| r["id"] == id)?;
        Some(match response.get("error") {
            Some(error) => Err(error.clone()),
            None => Ok(response["result"].clone()),
        })
    }

    /// Sends a request and returns its answer after one turn of the core.
    pub fn call(&mut self, core: &mut Core, method: &str, params: Value) -> Result<Value, Value> {
        let id = self.send(method, params);
        self.answer(core, id).expect("an answer in the same turn")
    }

    /// The result of a request that succeeds.
    pub fn ok(&mut self, core: &mut Core, method: &str, params: Value) -> Value {
        self.call(core, method, params)
            .unwrap_or_else(|error| panic!("{method} failed: {error}"))
    }

    /// The code and the message of a request that fails.
    pub fn fails(&mut self, core: &mut Core, method: &str, params: Value) -> (i64, String) {
        let error = self
            .call(core, method, params)
            .expect_err("the request fails");
        let number = error["code"].as_i64().expect("an error code");
        (
            number,
            error["message"].as_str().unwrap_or_default().to_owned(),
        )
    }

    /// The names of the events that arrived, and takes them.
    pub fn event_names(&mut self, core: &mut Core) -> Vec<String> {
        core.dispatch();
        self.responses();
        self.take_events()
            .iter()
            .map(|e| e["params"]["event"].as_str().unwrap_or_default().to_owned())
            .collect()
    }

    /// Takes the events that arrived after one turn of the core.
    pub fn events_now(&mut self, core: &mut Core) -> Vec<Value> {
        core.dispatch();
        self.responses();
        self.take_events()
    }

    /// Takes the events that arrived until now.
    pub fn take_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.events)
    }

    /// Closes the side of the client that writes, so that the core reads
    /// the end of the stream.
    pub fn shut_down_writes(&self) {
        self.stream
            .shutdown(std::net::Shutdown::Write)
            .expect("a shutdown");
    }

    /// Whether the core closed the connection.
    pub fn closed(&mut self, core: &mut Core) -> bool {
        core.dispatch();
        match self.read() {
            None => true,
            Some(messages) => {
                self.events.extend(messages);
                false
            }
        }
    }
}
