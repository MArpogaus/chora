//! Reports to chora-mc when it runs: `agent.set` with the state of the
//! agent, and `hud.say` for each spoken sentence when its sound starts.
//! A thread of its own with its own connection (`NAME-hud`), so a slow
//! or missing mc never holds up the agent; errors only log.

use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
};

use chora_kit::conn::Conn;
use chora_scene::ipc::{ClientError, MethodCall};
use serde_json::{Value, json};

/// The client that has the methods.
const MC: &str = "mc";
/// The wait before a new connection after a failed one.
const RETRY: Duration = Duration::from_secs(5);

pub enum Report {
    /// The parameters of `agent.set` without the name.
    Agent(Value),
    /// A sentence that plays from `at` for `len`.
    Say {
        text: String,
        at: Instant,
        len: Duration,
    },
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
}

impl Reporter {
    fn new(client: String, agent: String) -> Self {
        Self {
            client,
            agent,
            conn: None,
            failed: None,
            quiet: false,
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
                    params["name"] = self.agent.clone().into();
                    self.call("agent.set", params);
                }
                Some(Report::Say { text, at, len }) => says.push_back((at, text, len)),
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

    fn call(&mut self, method: &str, params: Value) {
        if self.conn.is_none() && self.failed.is_none_or(|t| t.elapsed() >= RETRY) {
            match Conn::connect(&self.client) {
                Ok(conn) => (self.conn, self.failed) = (Some(conn), None),
                Err(e) => {
                    self.failed = Some(Instant::now());
                    self.log(&format!("mc: {e}"));
                }
            }
        }
        let Some(conn) = &mut self.conn else {
            return;
        };
        let call = MethodCall {
            client: MC.into(),
            method: method.into(),
            params,
        };
        match conn.call::<_, Value>("methods.call", call) {
            Ok(_) => self.quiet = false,
            // mc is absent or refused the call: the connection is fine.
            Err(ClientError::Rpc(e)) => self.log(&format!("mc {method}: {e}")),
            Err(e) => {
                self.conn = None;
                self.log(&format!("mc {method}: {e}"));
            }
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
