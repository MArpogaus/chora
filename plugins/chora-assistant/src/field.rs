//! The host side of the chat field: it starts `chora-chat --input` with a
//! pipe on stdin and stdout and without `CHORA_SOCKET`, and relays. The
//! field reads text that it cannot trust and so holds no connection to
//! chora: it can send a text to its helper through the host and nothing
//! else. The host passes the activation token of a person's act with
//! `place`, and nowhere else: the field ignores its environment.

use std::{
    io::{BufRead, BufReader, Write},
    ops::Not,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{Receiver, channel},
    thread,
};

use chora_kit::ask::Question;

use crate::chat::Link;
use serde_json::{Value, json};

/// What the field tells the host.
#[derive(Debug, PartialEq, Eq)]
pub enum FieldEvent {
    /// The person sent this text to the helper.
    Say(String),
    /// The surface is ready.
    Shown,
    /// The field got or lost the keyboard.
    Keyboard(bool),
    /// Esc closed the field, or the compositor did.
    Closed,
}

/// A running field.
pub struct Field {
    child: Child,
    input: ChildStdin,
    events: Receiver<FieldEvent>,
}

/// The event of a line of the field.
fn parse(line: &str) -> Option<FieldEvent> {
    let value: Value = serde_json::from_str(line).ok()?;
    if let Some(text) = value["say"].as_str() {
        return Some(FieldEvent::Say(text.to_owned()));
    }
    match value["event"].as_str()? {
        "shown" => Some(FieldEvent::Shown),
        "keyboard" => Some(FieldEvent::Keyboard(value["data"]["on"] == true)),
        "closed" => Some(FieldEvent::Closed),
        _ => None,
    }
}

impl Field {
    /// Starts the field with `command`, which is `chora-chat --input NAME`.
    /// The field gets neither `CHORA_SOCKET` nor an activation token in its
    /// environment.
    ///
    /// # Errors
    ///
    /// The program does not start.
    pub fn start(mut command: Command) -> std::io::Result<Self> {
        let mut child = command
            .env_remove("CHORA_SOCKET")
            .env_remove("XDG_ACTIVATION_TOKEN")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let (Some(input), Some(output)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(std::io::Error::other("the field has no pipe"));
        };
        let (tx, events) = channel();
        thread::spawn(move || {
            for line in BufReader::new(output).lines().map_while(Result::ok) {
                if let Some(event) = parse(&line)
                    && tx.send(event).is_err()
                {
                    return;
                }
            }
        });
        Ok(Self {
            child,
            input,
            events,
        })
    }

    /// What the field told since the last call.
    pub fn events(&self) -> Vec<FieldEvent> {
        self.events.try_iter().collect()
    }

    /// Shows the field at a place: `anchor`, `margin`, `width`, `output`,
    /// `look` and `token`, the activation token of the act that opened
    /// the chat.
    pub fn place(&mut self, place: &Value) -> std::io::Result<()> {
        self.send(&json!({"place": place}))
    }

    /// Tells the field that a question waits or ended.
    pub fn question(&mut self, waiting: bool) -> std::io::Result<()> {
        self.send(&json!({"question": waiting}))
    }

    /// Tells the field that a text did not reach the helper.
    pub fn say_failed(&mut self, text: &str) -> std::io::Result<()> {
        self.send(&json!({"say_failed": text}))
    }

    /// Gives a line back that was typed before a helper was picked.
    pub fn pick_first(&mut self, text: &str) -> std::io::Result<()> {
        self.send(&json!({"pick_first": text}))
    }

    /// Passes a voice event to the field if it concerns its helper.
    pub fn voice_event(&mut self, event: &str, data: &Value, helper: &str) -> std::io::Result<()> {
        match voice_line(event, data, helper) {
            Some(line) => self.send(&line),
            None => Ok(()),
        }
    }

    fn send(&mut self, line: &Value) -> std::io::Result<()> {
        writeln!(self.input, "{line}")?;
        self.input.flush()
    }
}

impl Drop for Field {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The line for the field that a voice event makes: whether a question of
/// its helper waits, or the keys that answer the question that waits.
/// The events of other helpers make none.
#[must_use]
pub fn voice_line(event: &str, data: &Value, helper: &str) -> Option<Value> {
    match event {
        "voice.state" if data["name"].as_str()?.eq_ignore_ascii_case(helper) => {
            Some(json!({"question": data["state"] == "question"}))
        }
        "voice.confirm" if names_other(data, helper).not() => {
            let keys = Question::from_event(data, None)?.keys;
            Some(json!({"keys": {"yes": keys.yes, "no": keys.no}}))
        }
        _ => None,
    }
}

/// Whether a `voice.confirm` names a helper other than ours. Today it
/// names none, and its keys are the same for every helper.
fn names_other(data: &Value, helper: &str) -> bool {
    data["agent"]
        .as_str()
        .is_some_and(|agent| !agent.eq_ignore_ascii_case(helper))
}

/// The parameters of the call that sends a text to a helper:
/// `voice agent`.
#[must_use]
pub fn say_params(helper: &str, text: &str) -> Value {
    json!({"tool": "agent_say", "name": helper, "text": text})
}

impl Link for Field {
    fn place(&mut self, place: &Value) {
        log(Self::place(self, place));
    }

    fn question(&mut self, waiting: bool) {
        log(Self::question(self, waiting));
    }

    fn say_failed(&mut self, text: &str) {
        log(Self::say_failed(self, text));
    }

    fn pick_first(&mut self, text: &str) {
        log(Self::pick_first(self, text));
    }

    fn voice_event(&mut self, event: &str, data: &Value, helper: &str) {
        log(Self::voice_event(self, event, data, helper));
    }

    fn events(&mut self) -> Vec<FieldEvent> {
        Self::events(self)
    }

    fn alive(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    fn hide(&mut self) {
        log(self.send(&json!("hide")));
    }
}

fn log(result: std::io::Result<()>) {
    if let Err(err) = result {
        eprintln!("chora-assistant: field: {err}");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn the_field_gets_no_socket_and_no_token() {
        let mut command = Command::new("sh");
        command
            .args([
                "-c",
                r#"printf '{"say":"%s"}\n' "${CHORA_SOCKET-none}-${XDG_ACTIVATION_TOKEN-none}""#,
            ])
            .env("CHORA_SOCKET", "/run/chora.sock")
            .env("XDG_ACTIVATION_TOKEN", "stamped");
        let field = Field::start(command).expect("a field");
        let event = field.events.recv_timeout(Duration::from_secs(5));
        assert_eq!(event, Ok(FieldEvent::Say("none-none".into())));
    }

    #[test]
    fn the_events_of_the_field_are_parsed() {
        assert_eq!(parse(r#"{"say":"hi"}"#), Some(FieldEvent::Say("hi".into())));
        assert_eq!(
            parse(r#"{"event":"shown","data":{}}"#),
            Some(FieldEvent::Shown)
        );
        assert_eq!(
            parse(r#"{"event":"keyboard","data":{"on":true}}"#),
            Some(FieldEvent::Keyboard(true))
        );
        assert_eq!(parse(r#"{"event":"other"}"#), None);
        assert_eq!(parse("nonsense"), None);
    }

    #[test]
    fn only_the_events_of_its_helper_reach_the_field() {
        let state = json!({"name": "Atlas", "state": "question"});
        assert_eq!(
            voice_line("voice.state", &state, "atlas"),
            Some(json!({"question": true}))
        );
        assert_eq!(voice_line("voice.state", &state, "coder"), None);
        let confirm = json!({"agent": "coder", "id": "3", "prompt": "Edit?",
            "keys": {"yes": "Alt+y", "no": "Alt+n"}});
        assert_eq!(voice_line("voice.confirm", &confirm, "atlas"), None);
        let unnamed = json!({"id": "3", "prompt": "Edit?"});
        assert!(voice_line("voice.confirm", &unnamed, "atlas").is_some());
        assert_eq!(
            voice_line("voice.confirm", &confirm, "coder"),
            Some(json!({"keys": {"yes": "Alt+y", "no": "Alt+n"}}))
        );
    }

    #[test]
    fn a_say_is_a_voice_agent_call() {
        let params = say_params("coder", "hello");
        assert_eq!(params["tool"], "agent_say");
        assert_eq!(params["name"], "coder");
    }
}
