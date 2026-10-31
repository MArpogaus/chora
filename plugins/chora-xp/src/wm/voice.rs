//! The voice agent in the window manager: its events (D65) drive the
//! paperclip of [`crate::assistant`], and a click on the paperclip mutes
//! or unmutes the mic.

use std::time::{Duration, Instant};

use chora_scene::{Command, ipc::ClientError};
use serde_json::{Value, json};

use super::Wm;
use crate::assistant::{self, Assistant, Pose};

/// The client name of the voice agent before its first event.
const VOICE: &str = "voice";

impl Wm {
    /// Asks the voice agent for its state, so that the paperclip shows
    /// after a start or restart of chora-xp while the agent is awake.
    pub(super) fn ask_voice(&mut self) -> Result<(), ClientError> {
        self.claim_indicator(false);
        // Without the agent, the call fails quietly.
        if let Ok(state) = self.voice_call("state", &json!({})) {
            let pose = state["state"].as_str().map(assistant::pose);
            self.voice = pose.map(|p| (VOICE.to_owned(), p));
            let commands = self.sync_assistant(Instant::now());
            self.apply(commands)?;
        }
        Ok(())
    }

    /// An event of the voice agent: `NAME.state` sets the pose,
    /// `NAME.say` shows a sentence, and `NAME.confirm` shows a question
    /// with its full detail until its answer. The events of sub-agents, which carry
    /// a name of their own, leave the paperclip alone.
    pub(super) fn voice_event(&mut self, event: &str, client: &str, data: &Value) -> Vec<Command> {
        // `voice.*` also matches a client named `voice.evil`: only the
        // agent itself drives the paperclip (review 4.9).
        if client != VOICE {
            return Vec::new();
        }
        self.claim_indicator(true);
        let kind = event.strip_prefix(client).and_then(|e| e.strip_prefix('.'));
        let name = data.get("name").and_then(Value::as_str);
        if name.is_some_and(|n| n != client) {
            return Vec::new();
        }
        let state = data.get("state").and_then(Value::as_str).unwrap_or("");
        let now = Instant::now();
        match kind {
            Some("state") => {
                self.voice = Some((client.to_owned(), assistant::pose(state)));
                self.sync_assistant(now)
            }
            Some("say") => {
                self.voice = Some((client.to_owned(), Pose::Speaking));
                let mut commands = self.sync_assistant(now);
                let text = data.get("text").and_then(Value::as_str).unwrap_or("");
                let time = Duration::from_millis(data["duration_ms"].as_u64().unwrap_or(0));
                // A confirmation keeps the balloon until its answer.
                let assistant = self.assistant.as_mut().filter(|a| !a.asking());
                if let Some(a) = assistant.filter(|_| !text.is_empty()) {
                    commands.extend(a.say(&mut self.theme, text, time, now));
                }
                commands
            }
            Some("confirm") => {
                let Some(a) = self.assistant.as_mut() else {
                    return Vec::new();
                };
                // A string or a number: the agent sent numbers before.
                let id = match data.get("id") {
                    Some(Value::String(id)) => id.clone(),
                    Some(Value::Number(n)) => n.to_string(),
                    _ => String::new(),
                };
                if let Some(text) = confirm_text(data) {
                    return a.ask(&mut self.theme, &id, &text, now);
                }
                a.answered(&id, now);
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// The voice agent connected: it shows its own state indicator until
    /// a claim.
    pub(super) fn voice_connected(&mut self, name: &str) {
        if name == VOICE {
            self.claim_due = true;
            self.claim_indicator(false);
        }
    }

    /// Asks the voice agent to hide its own state indicator, since the
    /// paperclip shows the state, while one is due and the paperclip is
    /// enabled. An error, such as no agent or an agent without the
    /// method, only leaves it due; after an event of the agent (`last`)
    /// it is no longer due.
    fn claim_indicator(&mut self, last: bool) {
        if !self.claim_due || !self.settings.value.assistant.enabled {
            return;
        }
        let done = self.voice_call("indicator.claim", &json!({})).is_ok();
        self.claim_due = !done && !last;
    }

    /// The voice agent closed its connection.
    pub(super) fn voice_gone(&mut self, name: &str) -> Vec<Command> {
        if name == VOICE {
            self.claim_due = true;
        }
        if self.voice.as_ref().is_none_or(|(client, _)| client != name) {
            return Vec::new();
        }
        self.voice = None;
        self.sync_assistant(Instant::now())
    }

    /// Shows, moves into its pose or removes the paperclip, as the agent,
    /// the settings and the outputs want it.
    pub(super) fn sync_assistant(&mut self, now: Instant) -> Vec<Command> {
        let enabled = self.settings.value.assistant.enabled;
        let want = self.voice.as_ref().map(|(_, p)| *p).filter(|_| enabled);
        match (want, self.assistant.as_mut()) {
            (None, None) => Vec::new(),
            (None, Some(_)) => {
                self.assistant = None;
                vec![Assistant::remove()]
            }
            (Some(pose), Some(a)) => a.set_pose(&mut self.theme, pose, now),
            (Some(pose), None) => {
                let Some(bar) = self.bars.first() else {
                    return Vec::new();
                };
                let (a, commands) = Assistant::new(&mut self.theme, bar, pose, now);
                self.assistant = Some(a);
                commands
            }
        }
    }

    /// Builds the paperclip again, after a change of the outputs.
    pub(super) fn replace_assistant(&mut self) -> Vec<Command> {
        if self.assistant.take().is_none() {
            return Vec::new();
        }
        let mut commands = vec![Assistant::remove()];
        commands.extend(self.sync_assistant(Instant::now()));
        commands
    }

    /// When the paperclip moves, blinks or fades its balloon next.
    pub fn assistant_due(&self) -> Option<Instant> {
        self.assistant.as_ref().map(Assistant::due)
    }

    /// Moves the paperclip on.
    pub fn assistant_step(&mut self, now: Instant) -> Result<(), ClientError> {
        let Some(a) = self.assistant.as_mut().filter(|a| a.due() <= now) else {
            return Ok(());
        };
        let commands = a.step(&mut self.theme, now);
        self.apply(commands)
    }

    /// A click on the paperclip mutes the mic, or unmutes it, through
    /// the method `mute` of the voice agent.
    pub(super) fn press_assistant(&mut self) -> Vec<Command> {
        let toggled = self.voice_call("state", &json!({})).and_then(|state| {
            let mute = !state["muted"]["mic"].as_bool().unwrap_or(false);
            self.voice_call("mute", &json!({"mic": mute})).map(|_| mute)
        });
        if let Err(err) = &toggled {
            eprintln!("chora-xp: voice: {err}");
        }
        let text = mic_text(toggled.ok());
        let now = Instant::now();
        self.assistant.as_mut().map_or_else(Vec::new, |a| {
            a.say(&mut self.theme, text, Duration::ZERO, now)
        })
    }

    /// Calls a method of the voice agent.
    fn voice_call(&mut self, method: &str, params: &Value) -> Result<Value, ClientError> {
        let client = self.voice.as_ref().map_or(VOICE, |(c, _)| c.as_str());
        let call = json!({"client": client, "method": method, "params": params});
        self.rt.conn.call("methods.call", call)
    }
}

/// The text of `voice.confirm` for the balloon: the prompt and the whole
/// detail; `None` for the event `done`.
fn confirm_text(data: &Value) -> Option<String> {
    if data.get("done").and_then(Value::as_bool).unwrap_or(false) {
        return None;
    }
    let field = |key| data.get(key).and_then(Value::as_str).unwrap_or("");
    Some(
        format!("{}\n{}", field("prompt"), field("detail"))
            .trim()
            .into(),
    )
}

/// What the balloon says after a click on the paperclip: the new state
/// of the mic, or `None` when the state or mute call failed.
fn mic_text(mute: Option<bool>) -> &'static str {
    match mute {
        Some(true) => "The microphone is off. Click me to turn it on.",
        Some(false) => "The microphone is on.",
        None => "I could not switch the microphone.",
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_confirmation_shows_the_prompt_and_the_whole_detail() {
        let long = "x".repeat(500);
        let ask = serde_json::json!({"id": "c1", "prompt": "Run it?", "detail": long});
        let text = super::confirm_text(&ask).unwrap();
        assert_eq!(text, format!("Run it?\n{long}"));
        let done = serde_json::json!({"id": "c1", "done": true});
        assert_eq!(super::confirm_text(&done), None);
    }

    #[test]
    fn a_failed_call_says_so_in_the_balloon() {
        assert_eq!(super::mic_text(None), "I could not switch the microphone.");
        assert!(super::mic_text(Some(true)).contains("off"));
    }
}
