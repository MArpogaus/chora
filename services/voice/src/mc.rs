//! Reports the agent to a HUD such as chora-mc as client events (D65):
//! `NAME.state` with the state of the agent or of a sub-agent,
//! `NAME.remove` when a sub-agent is gone, and
//! `NAME.say` for each spoken sentence when its sound starts. The `chora`
//! thread emits them on the connection of the agent's name. Also the
//! indicator: a dot in the color of the state, at the bottom right of the
//! first output, under `screen:OUTPUT`, on a connection of its own
//! (`NAME-hud`). A thread of its own, so that it never holds up the
//! agent; errors only log.

use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
};

use chora_kit::conn::Conn;
use chora_scene::ipc::ClientError;
use serde_json::{Value, json};

use crate::core::State;

/// An event for the `chora` thread: the name after `NAME.` and the data.
pub type Emit = (&'static str, Value);

/// The wait before a new connection after a failed one.
const RETRY: Duration = Duration::from_secs(5);
/// The node of the indicator.
const DOT: &str = "voice-indicator";
/// The size of the dot and its distance from the corner.
const DOT_SIZE: i32 = 12;
const DOT_MARGIN: i32 = 12;

pub enum Report {
    /// The state of the agent: `state`, `task` and `said`; with a `name`,
    /// that of a sub-agent such as the coder.
    Agent(Value),
    /// A sub-agent is gone.
    Gone(String),
    /// A sentence that plays from `at` for `len`.
    Say {
        text: String,
        at: Instant,
        len: Duration,
    },
    /// The state of the agent for the indicator; none removes the dot.
    State(Option<State>),
}

/// The color of the dot in a state.
fn color(state: State) -> [f64; 4] {
    match state {
        State::Sleeping => [0.5, 0.5, 0.5, 0.5],
        State::Listening => [0.35, 0.6, 1.0, 0.9],
        State::Recording => [0.95, 0.3, 0.3, 1.0],
        State::Transcribing | State::Thinking | State::Tool => [1.0, 0.75, 0.2, 1.0],
        State::Confirming => [0.8, 0.45, 1.0, 1.0],
        State::Speaking => [0.3, 0.85, 0.5, 1.0],
        State::Working => [0.2, 0.8, 0.8, 1.0],
    }
}

/// Starts the thread; it ends with the sender. The events go to `emits`.
pub fn start(name: &str, emits: Sender<Emit>) -> Sender<Report> {
    let (tx, rx) = mpsc::channel();
    let client = format!("{name}-hud");
    thread::spawn(move || Reporter::new(client, emits).run(&rx));
    tx
}

struct Reporter {
    client: String,
    emits: Sender<Emit>,
    conn: Option<Conn>,
    /// The last failed connection.
    failed: Option<Instant>,
    /// The last call failed; the next error does not log.
    quiet: bool,
    /// The dot is there on this connection.
    dot: bool,
}

impl Reporter {
    fn new(client: String, emits: Sender<Emit>) -> Self {
        Self {
            client,
            emits,
            conn: None,
            failed: None,
            quiet: false,
            dot: false,
        }
    }

    fn run(&mut self, rx: &Receiver<Report>) {
        // The sentences that wait for their sound.
        let mut says: VecDeque<(Instant, String, Duration)> = VecDeque::new();
        loop {
            let report = match says.front() {
                Some((at, ..)) => {
                    match rx.recv_timeout(at.saturating_duration_since(Instant::now())) {
                        Ok(r) => Some(r),
                        Err(RecvTimeoutError::Timeout) => None,
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
                None => match rx.recv() {
                    Ok(r) => Some(r),
                    Err(_) => return,
                },
            };
            match report {
                Some(Report::Agent(params)) => {
                    let _ = self.emits.send(("state", params));
                }
                Some(Report::Gone(name)) => {
                    let _ = self.emits.send(("remove", json!({"name": name})));
                }
                Some(Report::Say { text, at, len }) => says.push_back((at, text, len)),
                Some(Report::State(state)) => self.indicate(state),
                None => {
                    let Some((at, text, len)) = says.pop_front() else {
                        continue;
                    };
                    // A late sentence shows its words faster.
                    let left = len.saturating_sub(at.elapsed());
                    let params = json!({"state": "speaking", "text": text,
                        "duration_ms": left.as_millis()});
                    let _ = self.emits.send(("say", params));
                }
            }
        }
    }

    /// Calls a method of the core on the connection, which it opens at
    /// most once per `RETRY`; answers none on an error.
    fn rpc(&mut self, method: &str, params: impl serde::Serialize) -> Option<Value> {
        if self.conn.is_none() && self.failed.is_none_or(|t| t.elapsed() >= RETRY) {
            match Conn::connect(&self.client) {
                Ok(conn) => (self.conn, self.failed, self.dot) = (Some(conn), None, false),
                Err(e) => {
                    self.failed = Some(Instant::now());
                    self.log(&format!("mc: {e}"));
                }
            }
        }
        let conn = self.conn.as_mut()?;
        match conn.call::<_, Value>(method, params) {
            Ok(v) => {
                self.quiet = false;
                Some(v)
            }
            // The core refused the call: the connection is fine.
            Err(ClientError::Rpc(e)) => {
                self.log(&format!("{method}: {e}"));
                None
            }
            Err(e) => {
                self.conn = None;
                self.log(&format!("{method}: {e}"));
                None
            }
        }
    }

    /// Shows the state in the dot; creates it on the first output.
    fn indicate(&mut self, state: Option<State>) {
        let commands = match state {
            None if !self.dot => return,
            None => json!([{"remove": {"id": DOT}}]),
            Some(state) if self.dot => {
                json!([{"update": {"id": DOT, "props": {"color": color(state)}}}])
            }
            Some(state) => {
                // ponytail: placed once; a new output size moves it on the
                // next connection only.
                let Some(outputs) = self.rpc("output.list", Value::Null) else {
                    return;
                };
                let output = &outputs[0];
                let (Some(name), Some(w), Some(h)) = (
                    output["name"].as_str(),
                    output["size"][0].as_i64(),
                    output["size"][1].as_i64(),
                ) else {
                    return;
                };
                let at = |side: i64| side - i64::from(DOT_SIZE + DOT_MARGIN);
                json!([{"create": {"id": DOT, "kind": "rect", "parent": format!("screen:{name}"),
                    "props": {"size": [DOT_SIZE, DOT_SIZE], "radius": DOT_SIZE / 2,
                        "color": color(state), "input": "pass",
                        "transform": {"translate": [at(w), at(h)]}}}}])
            }
        };
        if self
            .rpc("scene.apply", json!({"commands": commands}))
            .is_some()
        {
            self.dot = state.is_some();
        }
    }

    /// Logs the first error of a run of errors.
    fn log(&mut self, line: &str) {
        if !self.quiet {
            crate::log::debug(line);
        }
        self.quiet = true;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    use serde_json::json;

    use super::{Report, start};

    /// The state goes out at once, a sentence when its sound starts, both
    /// as events of the agent (D65); no connection to chora is needed.
    #[test]
    fn reports_become_events() {
        let (emits, emitted) = mpsc::channel();
        let reports = start("voice", emits);
        let state = json!({"state": "listening", "task": "", "said": ""});
        reports.send(Report::Agent(state.clone())).unwrap();
        let wait = Duration::from_secs(5);
        assert_eq!(emitted.recv_timeout(wait).unwrap(), ("state", state));
        let at = Instant::now() + Duration::from_millis(50);
        let len = Duration::from_millis(1500);
        let text = "Hallo Welt".to_owned();
        reports.send(Report::Say { text, at, len }).unwrap();
        let (event, data) = emitted.recv_timeout(wait).unwrap();
        assert!(Instant::now() >= at, "not before the sound");
        assert_eq!(event, "say");
        assert_eq!(data["text"], "Hallo Welt");
        assert_eq!(data["state"], "speaking");
        assert!(data["duration_ms"].as_u64().unwrap() <= 1500);
    }
}
