//! The `helper` key of `state.get` (design 3.1): the state word of the
//! helper that the voice agent announces, or `none` while no helper runs.
//! An adapter feeds it the `voice.state` and `voice.remove` events and
//! `client.disconnected` of `voice`, and only reads it.

use chora_scene::Event;
use serde_json::Value;

/// The word of `state.get` while no helper runs.
pub const NONE: &str = "none";

/// The helpers that voice announced: the voice agent itself (no name) and
/// the named ones, the latest report last.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Helpers {
    seen: Vec<(String, String)>,
}

impl Helpers {
    /// Takes in the data of a `voice.state`.
    pub fn follow(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default().to_owned();
        let state = data["state"].as_str().unwrap_or_default().to_owned();
        self.seen.retain(|(n, _)| *n != name);
        self.seen.push((name, state));
    }

    /// Takes in the data of a `voice.remove`: the named helper is gone.
    pub fn forget(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default();
        self.seen.retain(|(n, _)| n != name);
    }

    /// Follows an event of the client `voice`: its `voice.state` and
    /// `voice.remove`, and its disconnect.
    pub fn hear(&mut self, event: &Event) {
        match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == "voice" => match event.as_str() {
                "voice.state" => self.follow(data),
                "voice.remove" => self.forget(data),
                _ => {}
            },
            Event::ClientDisconnected { name } if name == "voice" => self.clear(),
            _ => {}
        }
    }

    /// Voice ended: no helper runs without it.
    pub fn clear(&mut self) {
        self.seen.clear();
    }

    /// The state word of the helper that reported last, else `none`.
    #[must_use]
    pub fn word(&self) -> &str {
        match self.seen.last() {
            Some((_, state)) if !state.is_empty() => state,
            _ => NONE,
        }
    }
}

/// The state of the helper door, the one rule that every look and the
/// presenter `chora-assistant` read (KIT7): five states from the events of
/// voice. The figure, the chat and the doors are costume; this is the state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// No helper exists and the voice agent rests, or voice does not run.
    NoHelper,
    /// A helper exists; the latest report rests.
    Idle,
    /// A helper exists and the voice agent listens.
    Listening,
    /// The latest report thinks or works.
    Working,
    /// The latest report is an answer: spoken, a question to the person,
    /// done or blocked. Never `Working` after it until a new report works.
    Answered,
}

impl State {
    /// The word of `state.get`.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::NoHelper => "no_helper",
            Self::Idle => "idle",
            Self::Listening => "listening",
            Self::Working => "working",
            Self::Answered => "answered",
        }
    }
}

/// The state machine of the helper: the reports of voice, the latest
/// last, and whether the microphone is off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Helper {
    seen: Vec<(String, String)>,
    mic_off: bool,
}

impl Helper {
    /// Follows an event of the client `voice`; true when it changed.
    pub fn hear(&mut self, _event: &Event) -> bool {
        false
    }

    /// The state.
    #[must_use]
    pub fn state(&self) -> State {
        State::NoHelper
    }

    /// The word of `state.get`.
    #[must_use]
    pub fn word(&self) -> &'static str {
        self.state().word()
    }

    /// The words of a helper door.
    #[must_use]
    pub fn door(&self, language: crate::words::Language) -> String {
        crate::words::Word::NoHelperDoor.text(language).to_owned()
    }

    /// Whether the microphone of voice is off.
    #[must_use]
    pub fn mic_off(&self) -> bool {
        self.mic_off
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::words::Language;

    fn voice(event: &str, data: serde_json::Value) -> Event {
        Event::Emitted {
            event: event.into(),
            client: "voice".into(),
            data,
        }
    }

    fn state(name: &str, state: &str) -> Event {
        if name.is_empty() {
            voice("voice.state", json!({"state": state}))
        } else {
            voice("voice.state", json!({"name": name, "state": state}))
        }
    }

    /// Every state of the machine, the honest `no helper` among them, in
    /// the order of a turn of a helper (KIT7 red cell): each event, the
    /// state after it and the words of the door.
    #[test]
    fn the_helper_goes_through_every_state_and_says_no_helper_honestly() {
        use State::{Answered, Idle, Listening, NoHelper, Working};
        let say = voice("voice.say", json!({"state": "speaking", "text": "Hi"}));
        let remove = voice("voice.remove", json!({"name": "coder"}));
        let gone = Event::ClientDisconnected {
            name: "voice".into(),
        };
        let turn = [
            (state("", "listening"), NoHelper, "No helper"),
            (state("", "thinking"), Working, "No helper"),
            (say, Answered, "No helper"),
            (state("", "listening"), NoHelper, "No helper"),
            (state("coder", "idle"), Idle, "coder · idle"),
            (state("", "listening"), Listening, "coder · idle"),
            (state("coder", "working"), Working, "coder · working"),
            (
                state("coder", "question"),
                Answered,
                "coder · question for you",
            ),
            (state("coder", "done"), Answered, "coder · done"),
            (remove, NoHelper, "No helper"),
            (state("coder", "working"), Working, "coder · working"),
            (gone, NoHelper, "No helper"),
        ];
        let mut h = Helper::default();
        assert_eq!(
            (h.state(), h.door(Language::En)),
            (NoHelper, "No helper".into())
        );
        for (n, (event, want, door)) in turn.into_iter().enumerate() {
            assert!(h.hear(&event), "step {n} changes the machine");
            assert_eq!(
                (h.state(), h.door(Language::En)),
                (want, door.into()),
                "step {n}"
            );
        }
        assert_eq!(
            [NoHelper, Idle, Listening, Working, Answered].map(State::word),
            ["no_helper", "idle", "listening", "working", "answered"]
        );
    }

    /// After an answer event the machine never says `working`, whichever
    /// word the answer has, until a new report works (I8).
    #[test]
    fn never_working_after_the_answer_event() {
        for answer in ["done", "blocked", "question", "speaking"] {
            let mut h = Helper::default();
            h.hear(&state("coder", "working"));
            assert_eq!(h.state(), State::Working, "{answer}");
            h.hear(&state("coder", answer));
            assert_ne!(h.state(), State::Working, "{answer}");
            h.hear(&state("", "listening"));
            assert_ne!(h.state(), State::Working, "{answer}, then voice listens");
        }
        let mut h = Helper::default();
        h.hear(&state("", "working"));
        h.hear(&voice(
            "voice.say",
            json!({"state": "speaking", "text": "Fertig."}),
        ));
        assert_ne!(h.state(), State::Working, "the voice agent answered");
    }

    /// Only events of `voice` count; the microphone is its own report.
    #[test]
    fn only_voice_counts_and_the_microphone_is_heard() {
        let mut h = Helper::default();
        let other = Event::Emitted {
            event: "voice.state".into(),
            client: "mc".into(),
            data: json!({"name": "coder", "state": "working"}),
        };
        assert!(!h.hear(&other));
        assert_eq!(h.state(), State::NoHelper);
        h.hear(&voice(
            "voice.state",
            json!({"state": "listening", "muted": {"mic": true}}),
        ));
        assert!(h.mic_off());
        h.hear(&voice(
            "voice.state",
            json!({"state": "listening", "muted": {"mic": false}}),
        ));
        assert!(!h.mic_off());
    }

    #[test]
    fn no_report_is_none() {
        assert_eq!(Helpers::default().word(), "none");
    }

    #[test]
    fn the_latest_report_is_the_word() {
        let mut h = Helpers::default();
        h.follow(&json!({"state": "idle"}));
        assert_eq!(h.word(), "idle");
        h.follow(&json!({"name": "atlas", "state": "working"}));
        assert_eq!(h.word(), "working");
        h.follow(&json!({"state": "listening"}));
        assert_eq!(h.word(), "listening");
    }

    #[test]
    fn a_removed_helper_falls_back_to_the_one_before() {
        let mut h = Helpers::default();
        h.follow(&json!({"state": "idle"}));
        h.follow(&json!({"name": "atlas", "state": "working"}));
        h.forget(&json!({"name": "atlas"}));
        assert_eq!(h.word(), "idle");
        h.clear();
        assert_eq!(h.word(), "none");
    }
}
