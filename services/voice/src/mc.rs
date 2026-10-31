//! Reports the agent to a HUD such as chora-mc as client events (D65):
//! `NAME.state` with the state of the agent or of a sub-agent,
//! `NAME.remove` when a sub-agent is gone, and
//! `NAME.say` for each spoken sentence when its sound starts. The `chora`
//! thread emits them on the connection of the agent's name. Also the
//! indicator: a dot in the color of the state, in a corner of the first
//! output (`indicator`), under `screen:OUTPUT`, on a connection of its
//! own (`NAME-hud`). A thread of its own, so that it never holds up the
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

use crate::{
    config::{ConfirmKeys, Corner},
    core::State,
};

/// An event for the `chora` thread: the name after `NAME.` and the data.
pub type Emit = (&'static str, Value);

/// The wait before a new connection after a failed one.
const RETRY: Duration = Duration::from_secs(5);
/// The node of the indicator.
const DOT: &str = "voice-indicator";
/// The text node of a question on the HUD of the agent.
const QUESTION: &str = "voice-question";
/// The size of the dot.
const DOT_SIZE: i32 = 12;

/// The corner of the dot and its distance from it.
pub type Place = (Corner, [i32; 2]);

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
    /// A question waits: `voice.confirm {prompt, detail, id}`, the id as a
    /// text; with
    /// `hud`, the HUD of the agent shows it too.
    Confirm {
        id: u64,
        prompt: String,
        detail: String,
        hud: bool,
        /// Only these keys answer: `physical: true` and `keys` go out.
        keys: Option<ConfirmKeys>,
    },
    /// The question `id` is over: `voice.confirm {id, done: true}`.
    ConfirmDone(u64),
    /// The state of the agent for the indicator, and its place; none
    /// removes the dot.
    State(Option<(State, Place)>),
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

/// The top left of the dot `offset` away from `corner` of an output of
/// `size`.
fn at(corner: Corner, [x, y]: [i32; 2], [w, h]: [i64; 2]) -> [i64; 2] {
    let far = |side: i64, off: i32| side - i64::from(DOT_SIZE + off);
    match corner {
        Corner::BottomRight => [far(w, x), far(h, y)],
        Corner::BottomLeft => [x.into(), far(h, y)],
        Corner::TopRight => [far(w, x), y.into()],
        Corner::TopLeft => [x.into(), y.into()],
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
    /// The place of the dot on this connection.
    dot: Option<Place>,
    /// A question shows on this connection.
    question: bool,
}

impl Reporter {
    fn new(client: String, emits: Sender<Emit>) -> Self {
        Self {
            client,
            emits,
            conn: None,
            failed: None,
            quiet: false,
            dot: None,
            question: false,
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
                Some(confirm @ Report::Confirm { .. }) => self.confirm(confirm),
                Some(Report::ConfirmDone(id)) => {
                    self.question(None);
                    let _ = self
                        .emits
                        .send(("confirm", json!({"id": id.to_string(), "done": true})));
                }
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

    /// Shows a question on the own HUD with `hud`, and emits it.
    fn confirm(&mut self, report: Report) {
        let Report::Confirm {
            id,
            prompt,
            detail,
            hud,
            keys,
        } = report
        else {
            return;
        };
        if hud {
            let keys = (keys.as_ref())
                .map(|k| format!("\n{} = yes, {} = no", k.yes, k.no))
                .unwrap_or_default();
            self.question(Some(&format!("{prompt}\n{detail}{keys}")));
        }
        // chora-mc reads the id as a text.
        let mut data = json!({"id": id.to_string(), "prompt": prompt, "detail": detail});
        if let Some(keys) = keys {
            data["physical"] = true.into();
            data["keys"] = json!({"yes": keys.yes, "no": keys.no});
        }
        let _ = self.emits.send(("confirm", data));
    }

    /// Calls a method of the core on the connection, which it opens at
    /// most once per `RETRY`; answers none on an error.
    fn rpc(&mut self, method: &str, params: impl serde::Serialize) -> Option<Value> {
        if self.conn.is_none() && self.failed.is_none_or(|t| t.elapsed() >= RETRY) {
            match Conn::connect(&self.client) {
                Ok(conn) => {
                    (self.conn, self.failed) = (Some(conn), None);
                    (self.dot, self.question) = (None, false);
                }
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

    /// Shows the state in the dot; creates it on the first output, again
    /// at a new place.
    fn indicate(&mut self, state: Option<(State, Place)>) {
        let commands = match state {
            None if self.dot.is_none() => return,
            None => json!([{"remove": {"id": DOT}}]),
            Some((state, place)) if self.dot == Some(place) => {
                json!([{"update": {"id": DOT, "props": {"color": color(state)}}}])
            }
            Some((state, (corner, offset))) => {
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
                let create = json!({"create": {"id": DOT, "kind": "rect",
                    "parent": format!("screen:{name}"),
                    "props": {"size": [DOT_SIZE, DOT_SIZE], "radius": DOT_SIZE / 2,
                        "color": color(state), "input": "pass",
                        "transform": {"translate": at(corner, offset, [w, h])}}}});
                match self.dot {
                    Some(_) => json!([{"remove": {"id": DOT}}, create]),
                    None => json!([create]),
                }
            }
        };
        if self
            .rpc("scene.apply", json!({"commands": commands}))
            .is_some()
        {
            self.dot = state.map(|(_, place)| place);
        }
    }

    /// Shows a question and its full command on the first output, or
    /// removes it.
    fn question(&mut self, text: Option<&str>) {
        let remove = json!({"remove": {"id": QUESTION}});
        let commands = match text {
            None if !self.question => return,
            None => json!([remove]),
            Some(text) => {
                // ponytail: a text node holds 4 KiB; a longer command
                // shows its start, and voice.confirm carries it whole.
                let mut end = text.len().min(4000);
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                let Some(outputs) = self.rpc("output.list", Value::Null) else {
                    return;
                };
                let (Some(name), Some(w)) =
                    (outputs[0]["name"].as_str(), outputs[0]["size"][0].as_i64())
                else {
                    return;
                };
                let create = json!({"create": {"id": QUESTION, "kind": "text",
                    "parent": format!("screen:{name}"),
                    "props": {"text": &text[..end], "font_size": 18, "max_width": w - 48,
                        "color": [1.0, 0.85, 0.4, 1.0], "input": "pass",
                        "transform": {"translate": [24, 24]}}}});
                if self.question {
                    json!([remove, create])
                } else {
                    json!([create])
                }
            }
        };
        if self
            .rpc("scene.apply", json!({"commands": commands}))
            .is_some()
        {
            self.question = text.is_some();
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

    use super::{Corner, Report, at, start};

    #[test]
    fn the_dot_sits_in_its_corner() {
        let size = [1000, 600];
        assert_eq!(at(Corner::BottomRight, [12, 12], size), [976, 576]);
        assert_eq!(at(Corner::BottomLeft, [20, 10], size), [20, 578]);
        assert_eq!(at(Corner::TopRight, [0, 0], size), [988, 0]);
        assert_eq!(at(Corner::TopLeft, [5, 6], size), [5, 6]);
    }

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
        let detail = "x".repeat(5000);
        let confirm = Report::Confirm {
            id: 3,
            prompt: "Wirklich?".into(),
            detail: detail.clone(),
            hud: false,
            keys: None,
        };
        reports.send(confirm).unwrap();
        // Review rc3 round 2, 1.1: the id is a text, as chora-mc reads it.
        let shown = json!({"id": "3", "prompt": "Wirklich?", "detail": detail});
        assert_eq!(emitted.recv_timeout(wait).unwrap(), ("confirm", shown));
        reports.send(Report::ConfirmDone(3)).unwrap();
        let done = json!({"id": "3", "done": true});
        assert_eq!(emitted.recv_timeout(wait).unwrap(), ("confirm", done));
        // A question about a security setting names its keys.
        let confirm = Report::Confirm {
            id: 4,
            prompt: "Sicher?".into(),
            detail: "allow_programs: [sh]".into(),
            hud: false,
            keys: Some(crate::config::ConfirmKeys::default()),
        };
        reports.send(confirm).unwrap();
        let shown = json!({"id": "4", "prompt": "Sicher?", "detail": "allow_programs: [sh]",
            "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}});
        assert_eq!(emitted.recv_timeout(wait).unwrap(), ("confirm", shown));
    }
}
