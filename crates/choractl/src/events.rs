//! The events of `choractl mcp` (#7). A subscription opens a connection of
//! its own under the client name of the server, grabs its keys there and
//! reads its events in a thread into a bounded buffer, so that the core
//! never waits for an agent that does not read. `wait_events` takes them
//! from the buffer.

use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Event,
    ipc::{Client, ClientError, Grab, Hello, Subscribe},
};
use serde_json::{Value, json};

/// The most events that wait in the buffer; older ones go.
const MAX_BUFFERED: usize = 1000;
/// The longest wait of `wait_events`.
pub const MAX_WAIT: Duration = Duration::from_secs(30);
/// The most events of one `wait_events`.
pub const MAX_TAKE: usize = 500;

/// The buffer that the thread fills.
#[derive(Default)]
struct Buffer {
    events: VecDeque<Event>,
    /// Events that went because the buffer was full, since the last take.
    dropped: usize,
    /// Why the connection of the subscription ended, if it did.
    ended: Option<String>,
}

/// A running subscription.
pub struct Subscription {
    buffer: Arc<(Mutex<Buffer>, Condvar)>,
    /// The connection, shut down when the subscription ends.
    socket: std::os::unix::net::UnixStream,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // The thread then reads the end of the stream and stops.
        let _ = self.socket.shutdown(std::net::Shutdown::Both);
    }
}

impl Subscription {
    /// Connects under a client name, grabs the keys and subscribes to the
    /// events; every event if `events` is empty. Returns the grabbed
    /// bindings in the form of `input.key`.
    pub fn start(
        name: &str,
        events: Vec<String>,
        keys: Vec<Binding>,
    ) -> Result<(Self, Vec<String>), ClientError> {
        let path = std::env::var_os(chora_scene::ipc::SOCKET_ENV).ok_or(ClientError::NoSocket)?;
        let socket = std::os::unix::net::UnixStream::connect(path)?;
        let hello = Hello {
            name: name.into(),
            keep_nodes: true,
        };
        let mut client = Client::from_stream(socket.try_clone()?, &hello)?;
        let mut grabbed = Vec::new();
        for binding in keys {
            grabbed.push(client.call::<_, String>("input.grab", Grab { binding })?);
        }
        client.call::<_, Value>("events.subscribe", Subscribe { events })?;
        let buffer = Arc::new((Mutex::new(Buffer::default()), Condvar::new()));
        let filled = Arc::clone(&buffer);
        thread::spawn(move || read(&mut client, &filled));
        Ok((Self { buffer, socket }, grabbed))
    }

    /// Takes up to `max` events; waits up to `timeout` for the first one.
    pub fn take(&self, max: usize, timeout: Duration) -> Value {
        let (lock, ready) = &*self.buffer;
        let deadline = Instant::now() + timeout;
        let mut buffer = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while buffer.events.is_empty() && buffer.ended.is_none() {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            buffer = ready
                .wait_timeout(buffer, left)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
        let count = buffer.events.len().min(max);
        let events: Vec<Event> = buffer.events.drain(..count).collect();
        let dropped = std::mem::take(&mut buffer.dropped);
        json!({
            "events": events,
            "dropped": dropped,
            "ended": buffer.ended,
        })
    }
}

/// Reads events into the buffer until the connection ends.
fn read(client: &mut Client, buffer: &(Mutex<Buffer>, Condvar)) {
    let (lock, ready) = buffer;
    loop {
        let next = client.next_event();
        let mut buffer = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match next {
            Ok(event) => {
                if buffer.events.len() >= MAX_BUFFERED {
                    buffer.events.pop_front();
                    buffer.dropped += 1;
                }
                buffer.events.push_back(event);
            }
            Err(err) => {
                buffer.ended = Some(format!("the subscription ended: {err}"));
                ready.notify_all();
                return;
            }
        }
        ready.notify_all();
    }
}
