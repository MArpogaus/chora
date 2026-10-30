//! The core: one thread reads all inputs from one channel and answers
//! with effects (docs/voice.md sections 2 and 3). `Core::handle` does no
//! IO, so the tests drive it directly.

use std::{sync::mpsc::Sender, time::Instant};

use tokio::sync::oneshot;

use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    asr::Transcript,
    chat,
    config::Config,
    earcon::Earcon,
    gate::{self, FRAME_MS, Frame, Gate},
};

/// The states of docs/voice.md section 3; `working` comes with the
/// coder (WP7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Sleeping,
    Listening,
    Recording,
    Transcribing,
    Thinking,
    Tool,
    Confirming,
    Speaking,
}

/// A method call of another client.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Call {
    State,
    Cancel,
    Sleep,
    Wake,
    Mute {
        mic: Option<bool>,
        speaker: Option<bool>,
    },
    /// The answer to a confirmation: yes or no.
    Confirm(bool),
}

/// What the reply of a turn tells the core.
#[derive(Debug)]
pub enum Reply {
    Chat(chat::Event),
    /// The first sound of the reply plays.
    Sound,
    /// The reply ended, also its sound.
    Done(String),
}

/// An input of the core.
pub enum Input {
    Frame(Frame),
    /// The capture command ended or failed.
    MicDown,
    PttDown,
    PttUp,
    /// A method call; the core sends the state record to the channel.
    Call(Call, Sender<Value>),
    Settings(Box<Config>),
    /// The runtime gave up on chora.
    ChoraGone,
    /// The ASR answered for a turn: the final text of a finished
    /// utterance, or an interim text of the utterance in progress.
    Transcript {
        turn: u64,
        interim: bool,
        result: Result<Transcript, String>,
    },
    /// The reply of the turn.
    Reply(u64, Reply),
}

/// What the core asks the other actors to do.
#[derive(Debug, PartialEq)]
pub enum Effect {
    Earcon(Earcon),
    /// A gate event for the recording, such as `onset`.
    Mark(&'static str),
    /// The utterance so far, every `asr_interim_ms`, for an interim
    /// transcript.
    Interim {
        turn: u64,
        pcm: Vec<i16>,
    },
    /// A finished utterance for the ASR.
    Utterance {
        turn: u64,
        pcm: Vec<i16>,
        speech_ms: u32,
        end_wait_ms: u32,
        awake: bool,
    },
    /// Starts the reply to a text; `ended` is the end of the speech.
    Ask {
        turn: u64,
        text: String,
        speak: bool,
        ended: Instant,
    },
    /// Stops the reply in progress and its sound.
    Stop,
    Exit,
}

/// The mute switches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Muted {
    pub mic: bool,
    pub speaker: bool,
}

pub struct Core {
    config: Config,
    gate: Gate,
    state: State,
    since: Instant,
    seq: u64,
    awake: bool,
    muted: Muted,
    ptt: bool,
    turn: u64,
    last_transcript: String,
    last_reply: String,
    /// The turn whose reply runs.
    asking: Option<u64>,
    /// The answer of the confirmation that waits.
    confirm: Option<oneshot::Sender<bool>>,
    /// The end of the last utterance.
    ended: Instant,
    /// Frames since the onset of the utterance in progress.
    recorded: u32,
    /// The mic failed; the error earcon sounds once until it works again.
    mic_down: bool,
}

impl Core {
    /// The agent starts awake.
    pub fn new(config: Config) -> Self {
        Self {
            gate: Gate::new(&config),
            config,
            state: State::Listening,
            since: Instant::now(),
            seq: 0,
            awake: true,
            muted: Muted::default(),
            ptt: false,
            turn: 0,
            last_transcript: String::new(),
            last_reply: String::new(),
            asking: None,
            confirm: None,
            ended: Instant::now(),
            recorded: 0,
            mic_down: false,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// The state record of `voice.state` (docs/voice.md section 10).
    pub fn record(&self) -> Value {
        json!({
            "state": self.state,
            "since_ms": u64::try_from(self.since.elapsed().as_millis()).unwrap_or(u64::MAX),
            "seq": self.seq,
            "awake": self.awake,
            "muted": self.muted,
            "turn": self.turn,
            "last_transcript": self.last_transcript,
            "last_reply": self.last_reply,
        })
    }

    fn set(&mut self, state: State) {
        if state != self.state {
            self.state = state;
            self.since = Instant::now();
            self.seq += 1;
        }
    }

    /// The state after an utterance.
    fn rest(&self) -> State {
        if self.awake {
            State::Listening
        } else {
            State::Sleeping
        }
    }

    fn earcon(&self, kind: Earcon) -> Option<Effect> {
        (self.config.earcons && !self.muted.speaker).then_some(Effect::Earcon(kind))
    }

    pub fn handle(&mut self, input: Input) -> Vec<Effect> {
        match input {
            Input::Frame(frame) => {
                self.mic_down = false;
                self.frame(&frame).into_iter().collect()
            }
            Input::MicDown if self.mic_down => Vec::new(),
            Input::MicDown => {
                self.mic_down = true;
                // The cut utterance is gone with the mic.
                self.gate.reset();
                let rest = self.rest();
                self.set(rest);
                self.earcon(Earcon::Error).into_iter().collect()
            }
            Input::PttDown if self.asking.is_some() => {
                crate::log::info("ptt down: the reply stops");
                self.stop(Earcon::Cancel)
            }
            Input::PttDown => {
                crate::log::debug("ptt down");
                self.ptt = true;
                Vec::new()
            }
            Input::PttUp => {
                crate::log::debug("ptt up");
                self.ptt = false;
                Vec::new()
            }
            Input::Call(call, reply) => {
                let effects = self.call(call);
                // The caller may have gone after its timeout.
                let _ = reply.send(self.record());
                effects
            }
            Input::Settings(config) => {
                self.gate.configure(&config);
                self.config = *config;
                Vec::new()
            }
            Input::ChoraGone => vec![Effect::Exit],
            Input::Transcript {
                turn,
                interim,
                result,
            } => self.transcript(turn, interim, result),
            Input::Reply(turn, reply) => self.reply(turn, reply),
        }
    }

    /// Stops the reply in progress with an earcon.
    fn stop(&mut self, earcon: Earcon) -> Vec<Effect> {
        self.asking = None;
        self.confirm = None;
        let rest = self.rest();
        self.set(rest);
        std::iter::once(Effect::Stop)
            .chain(self.earcon(earcon))
            .collect()
    }

    fn reply(&mut self, turn: u64, reply: Reply) -> Vec<Effect> {
        // A late event of a stopped reply; a dropped confirmation is a no.
        if self.asking != Some(turn) {
            return Vec::new();
        }
        let speaking = self.state == State::Speaking;
        match reply {
            Reply::Chat(chat::Event::Thinking) if !speaking => self.set(State::Thinking),
            Reply::Chat(chat::Event::Tool(_)) if !speaking => self.set(State::Tool),
            Reply::Chat(chat::Event::Confirm { answer, .. }) => {
                self.confirm = Some(answer);
                self.set(State::Confirming);
            }
            Reply::Sound => self.set(State::Speaking),
            Reply::Done(text) => {
                self.last_reply = text;
                self.asking = None;
                self.confirm = None;
                self.seq += 1;
                let rest = self.rest();
                self.set(rest);
            }
            Reply::Chat(_) => {}
        }
        Vec::new()
    }

    fn transcript(
        &mut self,
        turn: u64,
        interim: bool,
        result: Result<Transcript, String>,
    ) -> Vec<Effect> {
        // A late interim text. The final text of an older turn still
        // counts: no speech is dropped.
        if interim && (self.state != State::Recording || turn != self.turn + 1) {
            return Vec::new();
        }
        let kind = if interim { "interim" } else { "asr" };
        let effects = match result {
            Ok(t) => {
                crate::log::info(&format!(
                    "turn={turn} {kind}={}ms text={:?}",
                    t.after_commit.as_millis(),
                    t.text
                ));
                self.last_transcript = t.text;
                self.seq += 1;
                Vec::new()
            }
            Err(e) if interim => {
                crate::log::warn(&format!("turn={turn} interim: {e}"));
                Vec::new()
            }
            Err(e) => {
                crate::log::error(&format!("turn={turn} asr: {e}"));
                self.earcon(Earcon::Error).into_iter().collect()
            }
        };
        if !interim && turn == self.turn && self.state == State::Transcribing {
            // The word matcher and the queue of WP6 come before this.
            let text = self.last_transcript.trim().to_owned();
            if self.awake && !text.is_empty() {
                self.asking = Some(turn);
                self.set(State::Thinking);
                return vec![Effect::Ask {
                    turn,
                    text,
                    speak: !self.muted.speaker,
                    ended: self.ended,
                }];
            }
            let rest = self.rest();
            self.set(rest);
        }
        effects
    }

    fn call(&mut self, call: Call) -> Vec<Effect> {
        if call == Call::Cancel && self.asking.is_some() {
            return self.stop(Earcon::Cancel);
        }
        let kind = match call {
            Call::Confirm(yes) => {
                if let Some(answer) = self.confirm.take() {
                    let _ = answer.send(yes);
                    self.set(if yes { State::Tool } else { State::Thinking });
                }
                return Vec::new();
            }
            Call::Cancel if self.state == State::Recording => {
                self.gate.reset();
                let rest = self.rest();
                self.set(rest);
                Earcon::Cancel
            }
            Call::State | Call::Cancel => return Vec::new(),
            Call::Sleep => {
                self.gate.reset();
                self.awake = false;
                self.set(State::Sleeping);
                if self.asking.is_some() {
                    self.asking = None;
                    self.confirm = None;
                    return std::iter::once(Effect::Stop)
                        .chain(self.earcon(Earcon::Sleep))
                        .collect();
                }
                Earcon::Sleep
            }
            Call::Wake => {
                self.awake = true;
                if self.state == State::Sleeping {
                    self.set(State::Listening);
                }
                Earcon::Wake
            }
            Call::Mute { mic, speaker } => {
                let before = self.muted;
                self.muted.mic = mic.unwrap_or(before.mic);
                self.muted.speaker = speaker.unwrap_or(before.speaker);
                if self.muted != before {
                    self.seq += 1;
                }
                return Vec::new();
            }
        };
        self.earcon(kind).into_iter().collect()
    }

    fn frame(&mut self, frame: &Frame) -> Vec<Effect> {
        // During a reply the gate rests: barge-in comes with WP6.
        let busy = self.asking.is_some();
        let listen = !self.muted.mic && !busy;
        match self.gate.push(frame, listen, self.ptt && !busy) {
            None => self.interim().into_iter().collect(),
            Some(gate::Event::Onset) => {
                self.recorded = 0;
                self.set(State::Recording);
                vec![Effect::Mark("onset")]
            }
            Some(gate::Event::Noise { speech_ms }) => {
                crate::log::debug(&format!("noise speech={speech_ms}ms"));
                let rest = self.rest();
                self.set(rest);
                vec![Effect::Mark("noise")]
            }
            Some(gate::Event::End {
                pcm,
                speech_ms,
                end_wait_ms,
            }) => {
                self.turn += 1;
                self.ended = Instant::now();
                // Asleep, the text only looks for a wake word (WP6).
                let next = if self.awake {
                    State::Transcribing
                } else {
                    State::Sleeping
                };
                self.set(next);
                vec![
                    Effect::Mark("end"),
                    Effect::Utterance {
                        turn: self.turn,
                        pcm,
                        speech_ms,
                        end_wait_ms,
                        awake: self.awake,
                    },
                ]
            }
        }
    }

    /// The interim effect, every `asr_interim_ms` of an utterance.
    fn interim(&mut self) -> Option<Effect> {
        let every = self.config.asr_interim_ms.div_ceil(FRAME_MS);
        let pcm = self.gate.pcm()?;
        self.recorded += 1;
        (every > 0 && self.recorded.is_multiple_of(every)).then(|| Effect::Interim {
            turn: self.turn + 1,
            pcm: pcm.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    fn frame(prob: f32) -> Input {
        Input::Frame(Frame {
            samples: vec![0; 768],
            prob,
        })
    }

    fn call(core: &mut Core, call: Call) -> (Vec<Effect>, Value) {
        let (tx, rx) = mpsc::channel();
        let effects = core.handle(Input::Call(call, tx));
        (effects, rx.recv().unwrap())
    }

    fn speak(core: &mut Core, frames: usize) -> Vec<Effect> {
        (0..frames).flat_map(|_| core.handle(frame(0.9))).collect()
    }

    #[test]
    fn speech_records_an_utterance_and_its_text_starts_a_reply() {
        let mut core = Core::new(Config::default());
        assert_eq!(core.state(), State::Listening, "the agent starts awake");
        let effects = speak(&mut core, 10);
        assert_eq!(effects, [Effect::Mark("onset")]);
        assert_eq!(core.state(), State::Recording);
        let effects: Vec<_> = (0..25).flat_map(|_| core.handle(frame(0.0))).collect();
        assert!(
            matches!(
                &effects[1],
                Effect::Utterance {
                    turn: 1,
                    awake: true,
                    ..
                }
            ),
            "{effects:?}"
        );
        assert_eq!(core.state(), State::Transcribing);
        let effects = core.handle(text(1, false, Ok(" hallo ")));
        let [
            Effect::Ask {
                turn, text, speak, ..
            },
        ] = &effects[..]
        else {
            panic!("{effects:?}");
        };
        assert_eq!((*turn, text.as_str(), *speak), (1, "hallo", true));
        assert_eq!(core.state(), State::Thinking);
        assert_eq!(call(&mut core, Call::State).1["last_transcript"], " hallo ");
    }

    fn reply(turn: u64, event: chat::Event) -> Input {
        Input::Reply(turn, Reply::Chat(event))
    }

    /// A core whose turn 1 waits for its reply.
    fn asking() -> Core {
        let mut core = Core::new(Config::default());
        speak(&mut core, 10);
        (0..30).for_each(|_| drop(core.handle(frame(0.0))));
        core.handle(text(1, false, Ok("hallo")));
        core
    }

    #[test]
    fn the_reply_moves_through_its_states_and_a_confirmation() {
        let mut core = asking();
        core.handle(reply(1, chat::Event::Tool("query".into())));
        assert_eq!(core.state(), State::Tool);
        let (answer, mut rx) = oneshot::channel();
        core.handle(reply(
            1,
            chat::Event::Confirm {
                prompt: "Wirklich?".into(),
                answer,
            },
        ));
        assert_eq!(core.state(), State::Confirming);
        let (effects, record) = call(&mut core, Call::Confirm(true));
        assert_eq!(effects, []);
        assert_eq!(record["state"], "tool");
        assert_eq!(rx.try_recv(), Ok(true));
        core.handle(Input::Reply(1, Reply::Sound));
        core.handle(reply(1, chat::Event::Thinking));
        assert_eq!(core.state(), State::Speaking, "the sound goes on");
        assert!(
            speak(&mut core, 20).is_empty(),
            "the gate rests during a reply"
        );
        core.handle(Input::Reply(1, Reply::Done("Erledigt.".into())));
        let record = call(&mut core, Call::State).1;
        assert_eq!(
            (record["state"].as_str(), record["last_reply"].as_str()),
            (Some("listening"), Some("Erledigt."))
        );
    }

    #[test]
    fn cancel_and_the_key_stop_a_reply_and_its_late_events() {
        for stop in [Input::PttDown, Input::Call(Call::Cancel, mpsc::channel().0)] {
            let mut core = asking();
            let (answer, mut rx) = oneshot::channel();
            core.handle(reply(
                1,
                chat::Event::Confirm {
                    prompt: "Wirklich?".into(),
                    answer,
                },
            ));
            let effects = core.handle(stop);
            assert_eq!(effects, [Effect::Stop, Effect::Earcon(Earcon::Cancel)]);
            assert_eq!(core.state(), State::Listening);
            assert!(rx.try_recv().is_err(), "a dropped answer is a no");
            core.handle(Input::Reply(1, Reply::Sound));
            assert_eq!(core.state(), State::Listening, "a late event of turn 1");
        }
        let mut core = asking();
        let (effects, record) = call(&mut core, Call::Sleep);
        assert_eq!(effects, [Effect::Stop, Effect::Earcon(Earcon::Sleep)]);
        assert_eq!(record["state"], "sleeping");
    }

    fn text(turn: u64, interim: bool, result: Result<&str, &str>) -> Input {
        Input::Transcript {
            turn,
            interim,
            result: result
                .map(|text| Transcript {
                    text: text.into(),
                    after_commit: std::time::Duration::ZERO,
                })
                .map_err(Into::into),
        }
    }

    #[test]
    fn interim_texts_come_every_asr_interim_ms_while_recording() {
        let mut core = Core::new(Config {
            asr_interim_ms: 320,
            ..Config::default()
        });
        let effects = speak(&mut core, 25);
        let lengths: Vec<_> = effects
            .iter()
            .filter_map(|e| match e {
                Effect::Interim { turn: 1, pcm } => Some(pcm.len() / 768),
                _ => None,
            })
            .collect();
        // The utterance starts with the 3 frames of its onset.
        assert_eq!(lengths, [13, 23], "{effects:?}");
        core.handle(text(1, true, Ok("hallo")));
        assert_eq!(call(&mut core, Call::State).1["last_transcript"], "hallo");
        (0..30).for_each(|_| drop(core.handle(frame(0.0))));
        assert_eq!(core.state(), State::Transcribing);
        core.handle(text(1, true, Ok("late interim")));
        assert_eq!(call(&mut core, Call::State).1["last_transcript"], "hallo");
        core.handle(text(1, false, Ok("hallo chora")));
        assert_eq!(
            call(&mut core, Call::State).1["last_transcript"],
            "hallo chora"
        );
    }

    #[test]
    fn a_failed_transcript_sounds_the_error_and_listens_again() {
        let mut core = Core::new(Config::default());
        speak(&mut core, 10);
        (0..30).for_each(|_| drop(core.handle(frame(0.0))));
        let old = core.handle(text(0, false, Err("old turn")));
        assert_eq!(old, [Effect::Earcon(Earcon::Error)]);
        assert_eq!(core.state(), State::Transcribing);
        let effects = core.handle(text(1, false, Err("timeout")));
        assert_eq!(effects, [Effect::Earcon(Earcon::Error)]);
        assert_eq!(core.state(), State::Listening);
    }

    #[test]
    fn cancel_drops_the_utterance_with_an_earcon() {
        let mut core = Core::new(Config::default());
        speak(&mut core, 10);
        let (effects, record) = call(&mut core, Call::Cancel);
        assert_eq!(effects, [Effect::Earcon(Earcon::Cancel)]);
        assert_eq!(record["state"], "listening");
        let after: Vec<_> = (0..40).flat_map(|_| core.handle(frame(0.0))).collect();
        assert!(after.is_empty(), "no utterance after a cancel: {after:?}");
        let (effects, _) = call(&mut core, Call::Cancel);
        assert!(effects.is_empty(), "nothing to cancel");
    }

    #[test]
    fn sleep_and_wake_switch_the_state_and_count_seq() {
        let mut core = Core::new(Config::default());
        let (effects, record) = call(&mut core, Call::Sleep);
        assert_eq!(effects, [Effect::Earcon(Earcon::Sleep)]);
        assert_eq!(
            (record["state"].as_str(), record["awake"].as_bool()),
            (Some("sleeping"), Some(false))
        );
        // Asleep, speech still records: the ASR looks for the wake word.
        speak(&mut core, 10);
        let effects: Vec<_> = (0..25).flat_map(|_| core.handle(frame(0.0))).collect();
        assert!(matches!(
            &effects[1],
            Effect::Utterance { awake: false, .. }
        ));
        assert_eq!(core.state(), State::Sleeping);
        let (effects, record) = call(&mut core, Call::Wake);
        assert_eq!(effects, [Effect::Earcon(Earcon::Wake)]);
        assert_eq!(record["state"], "listening");
        assert_eq!(record["seq"], 4);
    }

    #[test]
    fn mute_switches_mic_and_speaker_separately() {
        let mut core = Core::new(Config::default());
        let (_, record) = call(
            &mut core,
            Call::Mute {
                mic: Some(true),
                speaker: None,
            },
        );
        assert_eq!(record["muted"], json!({"mic": true, "speaker": false}));
        assert!(
            speak(&mut core, 10).is_empty(),
            "a muted mic starts no turn"
        );
        core.handle(Input::PttDown);
        assert_eq!(
            speak(&mut core, 1),
            [Effect::Mark("onset")],
            "the key still records"
        );
        core.handle(Input::PttUp);
        let (_, record) = call(
            &mut core,
            Call::Mute {
                mic: Some(false),
                speaker: Some(true),
            },
        );
        assert_eq!(record["muted"], json!({"mic": false, "speaker": true}));
        let (effects, _) = call(&mut core, Call::Sleep);
        assert!(effects.is_empty(), "a muted speaker plays no earcon");
    }

    #[test]
    fn a_failing_mic_sounds_the_error_once() {
        let mut core = Core::new(Config::default());
        let down = |core: &mut Core| core.handle(Input::MicDown);
        assert_eq!(down(&mut core), [Effect::Earcon(Earcon::Error)]);
        assert_eq!(down(&mut core), Vec::<Effect>::new());
        core.handle(frame(0.0));
        assert_eq!(
            down(&mut core),
            [Effect::Earcon(Earcon::Error)],
            "again after it worked"
        );
    }
}
