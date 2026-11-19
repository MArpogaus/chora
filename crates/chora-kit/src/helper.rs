//! The helper state machine (KIT7): the one rule for the state of the
//! helper. The presenter chora-assistant and every look feed it the
//! events of `voice` and read its state (the `helper` key of `state.get`)
//! and the words of a helper door. How the state shows is costume.

use chora_scene::Event;
use serde_json::{Value, json};

use crate::words::{self, Language, Word};

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
    /// The state of a word of voice: `listening`, `thinking`, `done` and
    /// the others of docs/voice.md.
    #[must_use]
    pub fn of(raw: &str) -> Self {
        match raw {
            "listening" | "recording" => Self::Listening,
            "transcribing" | "thinking" | "tool" | "working" => Self::Working,
            "speaking" | "question" | "confirming" | "done" | "blocked" => Self::Answered,
            _ => Self::Idle,
        }
    }

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
/// last (the voice agent itself under the empty name), each a name, a
/// state and a task, and whether the microphone is off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Helper {
    seen: Vec<(String, String, String)>,
    mic_off: bool,
}

impl Helper {
    /// Follows an event of the client `voice`: `voice.state`,
    /// `voice.say`, `voice.remove` and its disconnect. True when the
    /// machine changed.
    pub fn hear(&mut self, event: &Event) -> bool {
        let was = self.clone();
        match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == "voice" => match event.as_str() {
                "voice.state" => self.follow(data),
                "voice.say" => self.report("", "speaking", ""),
                "voice.remove" => self.forget(data),
                _ => {}
            },
            Event::ClientDisconnected { name } if name == "voice" => *self = Self::default(),
            _ => {}
        }
        *self != was
    }

    /// Takes in the data of a `voice.state`.
    pub fn follow(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default();
        if name.is_empty()
            && let Some(off) = data["muted"]["mic"].as_bool()
        {
            self.mic_off = off;
        }
        let text = |key: &str| data[key].as_str().unwrap_or_default();
        self.report(name, text("state"), text("task"));
    }

    fn report(&mut self, name: &str, state: &str, task: &str) {
        self.seen.retain(|(n, ..)| n != name);
        self.seen
            .push((name.to_owned(), state.to_owned(), task.to_owned()));
    }

    /// Takes in the answer of the method `state` of voice, for a reader
    /// that starts after voice: its helpers (`agents`), then voice itself.
    pub fn seed(&mut self, answer: &Value) {
        *self = Self::default();
        for agent in answer["agents"].as_array().into_iter().flatten() {
            if agent["name"].as_str().is_some_and(|name| !name.is_empty()) {
                self.follow(agent);
            }
        }
        if let Some(state) = answer["state"].as_str() {
            self.follow(&json!({"state": state, "muted": answer["muted"]}));
        }
    }

    /// Takes in the data of a `voice.remove`: the named helper is gone.
    pub fn forget(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default();
        if !name.is_empty() {
            self.seen.retain(|(n, ..)| n != name);
        }
    }

    /// The latest report of a named helper: its name, raw state and task.
    #[must_use]
    pub fn named(&self) -> Option<(&str, &str, &str)> {
        self.seen
            .iter()
            .rev()
            .find(|(name, ..)| !name.is_empty())
            .map(|(name, state, task)| (name.as_str(), state.as_str(), task.as_str()))
    }

    /// The state: the latest report, mapped; `NoHelper` while no named
    /// helper exists and that report rests.
    #[must_use]
    pub fn state(&self) -> State {
        let Some((_, raw, _)) = self.seen.last() else {
            return State::NoHelper;
        };
        let state = State::of(raw);
        if self.named().is_none() && matches!(state, State::Idle | State::Listening) {
            State::NoHelper
        } else {
            state
        }
    }

    /// The word of `state.get`.
    #[must_use]
    pub fn word(&self) -> &'static str {
        self.state().word()
    }

    /// The words of a helper door: `No helper`, else the name and the
    /// state word of the helper that reported last, `coder · working`, and
    /// its task when the costume of the door has room for it.
    #[must_use]
    pub fn door(&self, language: Language, task: bool) -> String {
        match self.named() {
            Some((name, state, said)) if self.state() != State::NoHelper => {
                words::row(language, name, state, if task { said } else { "" })
            }
            _ => Word::NoHelperDoor.text(language).to_owned(),
        }
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

    /// A reader that starts after voice takes the answer of its `state`.
    #[test]
    fn a_late_reader_seeds_the_machine_from_the_state_of_voice() {
        let mut h = Helper::default();
        h.seed(&json!({"state": "listening", "agents": []}));
        assert_eq!(h.state(), State::NoHelper);
        let answer = json!({"state": "listening", "muted": {"mic": true},
            "agents": [{"name": "coder", "state": "idle"}]});
        h.seed(&answer);
        assert_eq!((h.state(), h.mic_off()), (State::Listening, true));
        assert_eq!(h.door(Language::En, false), "coder · idle");
    }

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
            (h.state(), h.door(Language::En, false)),
            (NoHelper, "No helper".into())
        );
        for (n, (event, want, door)) in turn.into_iter().enumerate() {
            assert!(h.hear(&event), "step {n} changes the machine");
            assert_eq!(
                (h.state(), h.door(Language::En, false)),
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
        h.hear(&voice(
            "voice.state",
            json!({"name": "coder", "state": "working", "task": "Write"}),
        ));
        assert_eq!(h.door(Language::En, true), "coder · working · Write");
        assert_eq!(h.door(Language::En, false), "coder · working");
    }
}
