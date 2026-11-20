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

pub use chora_scene::testing::TempDir;

type Lines = std::sync::Arc<std::sync::Mutex<Vec<u8>>>;

thread_local! {
    static LINES: std::cell::RefCell<Option<Lines>> = const { std::cell::RefCell::new(None) };
}

/// The writer of the test-wide subscriber: a line goes to the sink of the
/// thread that logged it.
struct Route;

impl Write for Route {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        LINES.with(|lines| {
            if let Some(sink) = lines.borrow().as_ref() {
                sink.lock().unwrap().extend_from_slice(buf);
            }
        });
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Sets the one subscriber of the test binary, at the first core: a span
/// that smithay makes before any subscriber exists carries the id 0xDEAD
/// of no subscriber, and the registry of a subscriber that a test sets
/// later panics on it as a parent: "tried to clone Id(57005), but no span
/// exists" (FLK-TR1). The subscriber is global because a subscriber of
/// one test thread loses to the interest in a callsite that another
/// thread cached without one. A thread with no sink for its lines
/// disables every span and event, so no test pays for the formatting.
fn install_subscriber() {
    use tracing_subscriber::{Layer, layer::SubscriberExt};
    static SUBSCRIBER: std::sync::Once = std::sync::Once::new();
    SUBSCRIBER.call_once(|| {
        let lines = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(|| Route)
            .with_filter(tracing_subscriber::filter::dynamic_filter_fn(|_, _| {
                LINES.with(|lines| lines.borrow().is_some())
            }));
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(lines))
            .expect("no other subscriber of the test binary");
    });
}

/// The log lines of every level that `run` writes in this thread.
pub fn logged(run: impl FnOnce()) -> String {
    install_subscriber();
    let lines = Lines::default();
    LINES.with(|l| *l.borrow_mut() = Some(std::sync::Arc::clone(&lines)));
    run();
    LINES.with(|l| *l.borrow_mut() = None);
    String::from_utf8_lossy(&lines.lock().unwrap()).into_owned()
}

/// The event loop and the state of a core without a backend.
pub struct Core {
    pub event_loop: EventLoop<'static, State>,
    pub state: State,
}

/// A new, empty directory in the temporary directory for one test: no
/// other test uses it, and what an earlier run left there is gone.
///
/// Made by [`TempDir`]; it hands out a `PathBuf` that
/// about 25 tests pass on, so it keeps no guard. New tests use [`TempDir`].
pub fn scratch_dir(name: &str) -> std::path::PathBuf {
    TempDir::new(name).keep()
}

impl Core {
    pub fn new() -> Self {
        install_subscriber();
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
        Client::new(ours)
    }

    /// A new IPC connection of the admin socket of `choractl`.
    pub fn admin(&mut self) -> Client {
        let (ours, theirs) = UnixStream::pair().expect("a socket pair");
        self.state
            .ipc_accept_admin(theirs)
            .expect("the core accepts");
        Client::new(ours)
    }

    /// A new IPC connection through a socket file of the core.
    pub fn connect_to(&mut self, path: &std::path::Path) -> Client {
        let ours = UnixStream::connect(path).expect("the socket listens");
        self.dispatch();
        Client::new(ours)
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

    /// Whether nothing holds the pointer: no grab of Smithay, no press on
    /// a node, no button grab and no drag of a client.
    pub fn pointer_free(&self) -> bool {
        let state = &self.state;
        !state.pointer().is_grabbed()
            && state.node_grab.is_none()
            && state.button_grab.is_none()
            && !state.client_drag
    }

    /// Moves the time of the deadlines and runs what expired.
    pub fn advance(&mut self, by: Duration) {
        self.state.time.advance(by);
        self.state.expire();
        self.dispatch();
    }
}

/// The parameters of `scene.apply` that create a red rect under the root.
pub fn rect(id: &str) -> Value {
    json!({"commands": [{"create": {"id": id, "kind": "rect", "parent": "root",
        "props": {"size": [10, 10], "color": [1, 0, 0, 1]}}}]})
}

/// The client side of an IPC connection. A drop shuts the socket down, so
/// that the core sees the end at once. This also holds while a forked child
/// of another test keeps a copy of the socket until its `exec`.
pub struct Client {
    stream: UnixStream,
    input: Vec<u8>,
    /// The events that arrived and that no test took yet.
    events: Vec<Value>,
    /// The responses that arrived and that no test took yet.
    responses: Vec<Value>,
    next: u64,
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

impl Client {
    fn new(stream: UnixStream) -> Self {
        stream.set_nonblocking(true).expect("a socket option");
        Self {
            stream,
            input: Vec::new(),
            events: Vec::new(),
            responses: Vec::new(),
            next: 1,
        }
    }

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

    /// Reads the messages and keeps the events. Takes the responses.
    pub fn responses(&mut self) -> Vec<Value> {
        let messages = self.read().unwrap_or_default();
        self.keep(messages);
        std::mem::take(&mut self.responses)
    }

    /// Keeps messages as events and responses for later.
    fn keep(&mut self, messages: Vec<Value>) {
        let (events, responses): (Vec<Value>, Vec<Value>) = messages
            .into_iter()
            .partition(|m| m.get("method").is_some());
        self.events.extend(events);
        self.responses.extend(responses);
    }

    /// The answer of the request with this id, after one turn of the core.
    /// `None` while the answer waits. Other responses stay for later.
    pub fn answer(&mut self, core: &mut Core, id: u64) -> Option<Result<Value, Value>> {
        core.dispatch();
        let messages = self.read().unwrap_or_default();
        self.keep(messages);
        let i = self.responses.iter().position(|r| r["id"] == id)?;
        let response = self.responses.remove(i);
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
                self.keep(messages);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use wayland_client::Proxy;

    use super::{
        Core,
        wayland::{PointerEvent, Wayland},
    };

    /// The spans of the seat, which smithay makes in `State::new`, are
    /// parents of the spans of the keyboard that come later. A span made
    /// under no subscriber carries the id 0xDEAD (57005), which the
    /// registry of a subscriber that a test sets afterwards does not know:
    /// "tried to clone Id(57005), but no span exists" (FLK-TR1).
    #[test]
    fn a_core_makes_its_spans_under_a_subscriber() {
        let _core = Core::new();
        let none = tracing::dispatcher::get_default(
            tracing::Dispatch::is::<tracing::subscriber::NoSubscriber>,
        );
        assert!(
            !none,
            "the seat of the core got its spans from no subscriber"
        );
    }

    #[test]
    fn an_answer_stays_while_the_client_waits_for_another() {
        let mut core = Core::new();
        let mut client = core.named("wm");
        let query = json!({"selector": "id:root"});
        let first = client.send("scene.query", query.clone());
        let second = client.send("scene.query", query);
        assert!(client.answer(&mut core, second).is_some());
        assert!(client.answer(&mut core, first).is_some());
    }

    #[test]
    fn the_client_records_pointer_and_popup_events() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "window");
        wayland.attach(&mut core, &window, (200, 200));
        let mut wm = core.named("wm");
        let place = json!({"commands": [
            {"attach": {"surface": "s1", "parent": "root"}},
            {"update": {"id": "s1", "props": {"transform": {"translate": [100, 50]}}}},
        ]});
        wm.ok(&mut core, "scene.apply", place);
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        let click = json!({"events": [
            {"pointer_move": {"position": [150.0, 80.0]}},
            {"pointer_move": {"position": [160.0, 90.0]}},
            {"button": {"state": "pressed"}},
            {"button": {"state": "released"}},
        ]});
        agent.ok(&mut core, "input.inject", click);
        wayland.roundtrip(&mut core);
        let events = wayland.take_pointer_events();
        let [first, second, press, release] = &events[..] else {
            panic!("an enter, a motion, a press and a release: {events:?}");
        };
        assert_eq!(
            *first,
            PointerEvent::Enter {
                surface: window.surface.id(),
                at: (50.0, 30.0)
            }
        );
        assert_eq!(*second, PointerEvent::Motion { at: (60.0, 40.0) });
        let (
            PointerEvent::Button {
                serial,
                pressed: true,
            },
            PointerEvent::Button { pressed: false, .. },
        ) = (press, release)
        else {
            panic!("a press and a release: {press:?} {release:?}");
        };
        let popup = wayland.popup(&mut core, &window, *serial);
        assert_eq!(wayland.popup_geometry(&popup), Some([-25, -25, 50, 50]));
    }
}
