//! Reports to chora-mc when it runs: `agent.set` with the state of the
//! agent, and `hud.say` for each spoken sentence when its sound starts.
//! Also the indicator: a dot in the color of the state, at the bottom
//! right of the first output, under `screen:OUTPUT`. A thread of its own
//! with its own connection (`NAME-hud`), so a slow or missing mc never
//! holds up the agent; errors only log.

use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
};

use chora_kit::conn::Conn;
use chora_scene::ipc::{ClientError, MethodCall};
use serde_json::{Value, json};

use crate::core::State;

/// The client that has the methods.
const MC: &str = "mc";
/// The wait before a new connection after a failed one.
const RETRY: Duration = Duration::from_secs(5);
/// The node of the indicator.
const DOT: &str = "voice-indicator";
/// The size of the dot and its distance from the corner.
const DOT_SIZE: i32 = 12;
const DOT_MARGIN: i32 = 12;

pub enum Report {
    /// The parameters of `agent.set`; without a name, those of the agent.
    Agent(Value),
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

/// Starts the thread; it ends with the sender.
pub fn start(name: &str) -> Sender<Report> {
    let (tx, rx) = mpsc::channel();
    let (client, agent) = (format!("{name}-hud"), name.to_owned());
    thread::spawn(move || Reporter::new(client, agent).run(&rx));
    tx
}

struct Reporter {
    client: String,
    agent: String,
    conn: Option<Conn>,
    /// The last failed connection.
    failed: Option<Instant>,
    /// The last call failed; the next error does not log.
    quiet: bool,
    /// The dot is there on this connection.
    dot: bool,
}

impl Reporter {
    fn new(client: String, agent: String) -> Self {
        Self {
            client,
            agent,
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
                Some(Report::Agent(mut params)) => {
                    // A sub-agent, such as the coder, has its own name.
                    if params.get("name").is_none() {
                        params["name"] = self.agent.clone().into();
                    }
                    self.call("agent.set", params);
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
                    self.call("hud.say", params);
                }
            }
        }
    }

    /// Calls a method of mc.
    fn call(&mut self, method: &str, params: Value) {
        let call = MethodCall {
            client: MC.into(),
            method: method.into(),
            params,
        };
        if self.rpc("methods.call", call).is_some() {
            self.quiet = false;
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
            Ok(v) => Some(v),
            // mc is absent or refused the call: the connection is fine.
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
