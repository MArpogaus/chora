//! The helper state machine (KIT7): the one rule for the state of the
//! helper. The presenter chora-assistant and every look feed it the
//! events of `voice` and read its state (the `helper` key of `state.get`)
//! and the words of a helper door. How the state shows is costume.

use std::time::Duration;

use chora_scene::Event;
use serde_json::{Value, json};

use crate::{
    conn::Conn,
    words::{self, Language, Word},
};

/// The events that the machine hears; a look subscribes them all, the
/// voice events or `voice.*`, so that its state is the one of the
/// presenter (rw-4).
pub const EVENTS: [&str; 4] = [
    "voice.state",
    "voice.say",
    "voice.remove",
    "client.connected",
];

/// The events of [`EVENTS`] that a subscription misses; `voice.*` holds
/// the voice events.
#[must_use]
pub fn unheard<'a>(subscribed: &[&str]) -> Vec<&'a str> {
    EVENTS
        .into_iter()
        .filter(|event| {
            let all_voice = event.starts_with("voice.") && subscribed.contains(&"voice.*");
            !all_voice && !subscribed.contains(event)
        })
        .collect()
}

/// How long a look waits for the answer of `state` of voice.
const ASK_WAIT: Duration = Duration::from_millis(500);

/// The state of the helper door, the one rule that every look and the
/// presenter `chora-assistant` read (KIT7): five states from the events of
/// voice. The figure, the chat and the doors are costume; this is the state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// No helper exists, or voice does not run. The voice agent alone is
    /// no helper: its listening, thinking and speaking show in the figure.
    NoHelper,
    /// A helper exists, rests, and the latest report rests.
    Idle,
    /// A helper exists and rests, and the voice agent listens.
    Listening,
    /// A named helper works: its latest report thinks or works, whoever
    /// reported after it.
    Working,
    /// No helper works, and the latest report of a helper, or else the
    /// latest report, is an answer: spoken, a question to the person, done
    /// or blocked. The voice agent listening again changes nothing. Never
    /// `Working` after it until a helper works again.
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

    /// Asks voice for its `state` and seeds the machine with the answer,
    /// for a look that starts after voice or hears voice connect (rw-4).
    /// Without an answer in time the machine stays. True when it seeded.
    pub fn ask(&mut self, conn: &mut Conn) -> bool {
        let call = json!({"client": "voice", "method": "state", "params": {}});
        match conn.call_within::<_, Value>("methods.call", call, ASK_WAIT) {
            Ok(answer) if answer["state"].is_string() => {
                self.seed(&answer);
                true
            }
            _ => false,
        }
    }

    /// Hears an event as [`Self::hear`] does, and asks voice for its
    /// state when voice connects. True when the machine changed.
    pub fn hear_asking(&mut self, event: &Event, conn: &mut Conn) -> bool {
        let was = self.clone();
        self.hear(event);
        if matches!(event, Event::ClientConnected { name } if name == "voice") {
            self.ask(conn);
        }
        *self != was
    }

    /// Takes in the data of a `voice.remove`: the named helper is gone.
    pub fn forget(&mut self, data: &Value) {
        let name = data["name"].as_str().unwrap_or_default();
        if !name.is_empty() {
            self.seen.retain(|(n, ..)| n != name);
        }
    }

    /// The helper of the door: the named helper that works and reported
    /// last, else the named helper that reported last; its name, raw state
    /// and task.
    #[must_use]
    pub fn named(&self) -> Option<(&str, &str, &str)> {
        let mut named = self.seen.iter().rev().filter(|(name, ..)| !name.is_empty());
        named
            .clone()
            .find(|(_, raw, _)| State::of(raw) == State::Working)
            .or_else(|| named.next())
            .map(|(name, state, task)| (name.as_str(), state.as_str(), task.as_str()))
    }

    /// The state from all live reports: `NoHelper` without a named
    /// helper, `Working` while any named helper works, `Answered` while
    /// the latest report of a helper is an answer, else the latest report
    /// that does not work (the voice agent thinking makes no helper work).
    #[must_use]
    pub fn state(&self) -> State {
        match self.named().map(|(_, raw, _)| State::of(raw)) {
            None => State::NoHelper,
            Some(state @ (State::Working | State::Answered)) => state,
            Some(_) => self
                .seen
                .iter()
                .rev()
                .map(|(_, raw, _)| State::of(raw))
                .find(|state| *state != State::Working)
                .unwrap_or(State::Idle),
        }
    }

    /// The word of `state.get`.
    #[must_use]
    pub fn word(&self) -> &'static str {
        self.state().word()
    }

    /// The words of a helper door: `No helper`, else the name and the
    /// state word of the helper of [`Self::named`] (the working one first),
    /// `coder · working`, and its task when the costume of the door has
    /// room for it.
    #[must_use]
    pub fn door(&self, language: Language, task: bool) -> String {
        match self.named() {
            Some((name, state, said)) => {
                words::row(language, name, state, if task { said } else { "" })
            }
            None => Word::NoHelperDoor.text(language).to_owned(),
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
            (state("", "thinking"), NoHelper, "No helper"),
            (say, NoHelper, "No helper"),
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

    /// Working wins while any helper works, whoever reported last, and the
    /// door names the working helper (rw-2); the voice agent alone is no
    /// helper and never makes the machine work (rw-3).
    #[test]
    fn working_wins_while_any_helper_works() {
        let mut h = Helper::default();
        h.hear(&state("coder", "working"));
        h.hear(&state("", "listening"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Working, "coder · working".into()),
            "coder working, then voice listening"
        );
        let mut h = Helper::default();
        h.hear(&state("atlas", "working"));
        h.hear(&state("coder", "idle"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Working, "atlas · working".into()),
            "atlas working, then coder idle"
        );
        h.hear(&state("atlas", "done"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Answered, "atlas · done".into()),
            "atlas done, coder idle"
        );
        h.hear(&state("", "thinking"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Answered, "atlas · done".into()),
            "the voice agent thinks, no helper works"
        );
        let mut h = Helper::default();
        h.hear(&state("", "thinking"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::NoHelper, "No helper".into()),
            "the voice agent thinks without a helper"
        );
    }

    /// The answer of a helper holds while the voice agent goes back to
    /// listening after the person answered its question (rw-6): the
    /// helper answered until a helper reports again.
    #[test]
    fn the_answer_of_a_helper_holds_while_voice_listens_again() {
        let mut h = Helper::default();
        h.hear(&state("coder", "idle"));
        h.hear(&state("coder", "question"));
        h.hear(&state("", "confirming"));
        h.hear(&state("coder", "working"));
        h.hear(&state("coder", "done"));
        h.hear(&state("", "listening"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Answered, "coder · done".into())
        );
        h.hear(&state("coder", "working"));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Working, "coder · working".into()),
            "a new working report clears the answer"
        );
        h.hear(&state("coder", "done"));
        h.hear(&state("coder", "idle"));
        assert_eq!(h.state(), State::Idle, "the helper reported again");
    }

    /// A new utterance of the person clears the answer of a helper: the
    /// voice agent recording or transcribing leaves the helper idle (`listening` while it records), and a helper that works still wins (rw-6). The voice
    /// agent thinking alone, as after a helper said something, does not.
    #[test]
    fn a_new_utterance_clears_the_answer_of_a_helper() {
        for (utterance, want) in [
            ("recording", State::Listening),
            ("transcribing", State::Idle),
        ] {
            let mut h = Helper::default();
            h.hear(&state("coder", "done"));
            assert_eq!(h.state(), State::Answered, "{utterance}");
            h.hear(&state("", utterance));
            assert_eq!(
                (h.state(), h.door(Language::En, false)),
                (want, "coder · idle".into()),
                "{utterance} clears the answer"
            );
        }
        let mut h = Helper::default();
        h.hear(&state("coder", "question"));
        h.hear(&state("", "thinking"));
        assert_eq!(h.state(), State::Answered, "thinking alone clears nothing");
        let mut h = Helper::default();
        h.hear(&state("coder", "working"));
        h.hear(&state("", "recording"));
        assert_eq!(h.state(), State::Working, "a helper that works still wins");
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

    /// A look that starts after voice, or hears it connect, seeds its
    /// machine from the answer of `state` of voice (rw-4); without an
    /// answer the machine stays.
    #[cfg(feature = "testing")]
    #[test]
    fn a_look_asks_voice_for_its_state_when_voice_connects() {
        let core = crate::testing::FakeCore::start();
        let mut conn = core.connect("look");
        let mut h = Helper::default();
        assert!(!h.ask(&mut conn), "voice says nothing");
        assert_eq!(h.state(), State::NoHelper);
        core.answer_call(
            "voice",
            "state",
            json!({"state": "listening", "agents": [{"name": "coder", "state": "idle"}]}),
        );
        let connected = Event::ClientConnected {
            name: "voice".into(),
        };
        assert!(h.hear_asking(&connected, &mut conn));
        assert_eq!(
            (h.state(), h.door(Language::En, false)),
            (State::Listening, "coder · idle".into())
        );
        assert_eq!(
            unheard(&["voice.*", "client.connected"]),
            Vec::<&str>::new()
        );
        assert_eq!(
            unheard(&["voice.state", "voice.remove"]),
            ["voice.say", "client.connected"]
        );
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
