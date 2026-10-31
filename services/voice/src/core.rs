//! The core: one thread reads all inputs from one channel and answers
//! with effects (docs/voice.md sections 2 to 4). `Core::handle` does no
//! IO, so the tests drive it directly.

use std::{
    collections::{BTreeMap, VecDeque},
    sync::mpsc::Sender,
    time::{Duration, Instant},
};

use tokio::sync::oneshot;

use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    agents,
    asr::Transcript,
    chat, coder,
    config::Config,
    earcon::Earcon,
    gate::{self, BargeIn, ECHO_WINDOW, FRAME_MS, Frame, Gate},
    speech::Control,
    words::{self, Word},
};

/// The states of docs/voice.md section 3.
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
    /// Listening while a coding task runs.
    Working,
}

impl State {
    /// The state of an agent in `mc agent.set`.
    pub fn agent(self) -> &'static str {
        match self {
            Self::Sleeping => "asleep",
            Self::Listening | Self::Recording | Self::Working => "listening",
            Self::Transcribing | Self::Thinking => "thinking",
            Self::Tool => "working",
            Self::Confirming => "question",
            Self::Speaking => "speaking",
        }
    }
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
    /// The push-to-talk key of another client: down or up.
    Talk(bool),
}

/// What the reply of a turn tells the core.
#[derive(Debug)]
pub enum Reply {
    Chat(chat::Event),
    /// Sound went to the player: it plays from `at`.
    Played {
        at: Instant,
        pcm: Vec<i16>,
    },
    /// A sentence plays from `at` for `len`.
    Said {
        text: String,
        at: Instant,
        len: Duration,
    },
    /// The reply ended, also its sound.
    Done(String),
    /// The TTS failed for a sentence.
    Failed,
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
    /// The ASR answered for an utterance: the final text of a finished
    /// one, or an interim text of the one in progress.
    Transcript {
        turn: u64,
        interim: bool,
        result: Result<Transcript, String>,
    },
    /// The reply with this number.
    Reply(u64, Reply),
    /// The coding task.
    Code(coder::Event),
    /// An ACP agent.
    Agent(agents::Event),
    /// A signal: the agent stops its sub-agents and exits.
    Quit,
    /// The method `agent`: a tool of the ACP agents and its arguments;
    /// the text of its answer goes to the channel.
    AgentCall(String, Value, Sender<Value>),
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
    /// Starts reply `reply` to a text; `ended` is the end of the speech.
    /// With `say`, the reply is the text itself, without the LLM: the
    /// result of a coding task.
    Ask {
        reply: u64,
        text: String,
        speak: bool,
        ended: Instant,
        say: bool,
        /// The voice of a sub-agent, else that of the settings.
        voice: Option<String>,
    },
    /// Stops the reply in progress and its sound; the words that played
    /// go into its note in the history.
    Stop(String),
    /// Pauses or resumes the speech of the reply: a barge-in.
    Speech(Control),
    /// Stops the coding task.
    StopCode,
    /// A prompt for a sub-agent, said to it by name.
    AgentPrompt {
        name: String,
        text: String,
    },
    /// Cancels the request of a sub-agent.
    AgentCancel(String),
    /// A sentence for the HUD: it plays from `at` for `len`.
    Say {
        text: String,
        at: Instant,
        len: Duration,
    },
    Exit,
}

/// The mute switches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Muted {
    pub mic: bool,
    pub speaker: bool,
}

/// An utterance whose final text waits.
struct Pending {
    turn: u64,
    length_ms: u32,
    /// Awake or by the key: the text is a request, not only a wake word.
    heard: bool,
}

/// The push-to-talk key.
#[derive(Default)]
struct Key {
    down: bool,
    /// Down during the utterance in progress.
    used: bool,
}

/// A pause of the reply's sound up to this long, while the TTS catches
/// up between sentences, still counts as speaking.
const SOUND_GAP: Duration = Duration::from_millis(1500);

/// Words that played within this time count for the echo filter.
const ECHO_MEMORY: Duration = Duration::from_secs(3);

pub struct Core {
    config: Config,
    gate: Gate,
    barge: BargeIn,
    state: State,
    since: Instant,
    seq: u64,
    awake: bool,
    muted: Muted,
    key: Key,
    /// The number of the last utterance.
    turn: u64,
    last_transcript: String,
    last_reply: String,
    /// The number of the last reply.
    replies: u64,
    /// The reply that runs.
    asking: Option<u64>,
    /// The text of the reply that runs or ran last.
    request: String,
    /// What the reply does when it neither speaks nor asks: thinking or
    /// a tool.
    doing: State,
    /// The answer of the confirmation that waits, and the unclear
    /// answers so far.
    confirm: Option<oneshot::Sender<bool>>,
    unclear: u32,
    /// The utterances whose final text waits.
    pending: Vec<Pending>,
    /// A barge-in paused the speech.
    paused: bool,
    /// The level of each played frame, by its time.
    levels: VecDeque<(Instant, f64)>,
    /// When the played sound of the reply ends; none before its first
    /// sound.
    sound_end: Option<Instant>,
    /// The sentences that played, with the end of their sound.
    said: VecDeque<(Instant, String)>,
    /// The words of the reply that played.
    played: String,
    /// Texts that came while a reply ran; they start the next one.
    queue: Vec<String>,
    /// The end of the last utterance.
    ended: Instant,
    /// Frames since the onset of the utterance in progress.
    recorded: u32,
    /// The mic failed; the error earcon sounds once until it works again.
    mic_down: bool,
    /// The coding task: its text, its state for chora-mc (`working`,
    /// `done` or `blocked`; empty before the first task) and its last
    /// step or result.
    code: (String, &'static str, String),
    /// Texts to say when the agent listens: the coding results, the
    /// spoken errors, and the words and questions of the sub-agents.
    notes: VecDeque<Note>,
    /// The reply in which the TTS failed; the error earcon sounded.
    tts_failed: Option<u64>,
    /// The ACP agents by name: the parameters of `mc agent.set`.
    subs: BTreeMap<String, Value>,
    /// The question of a sub-agent that waits for yes or no.
    sub_confirm: Option<(String, oneshot::Sender<bool>)>,
}

/// A text to say when the agent listens.
struct Note {
    text: String,
    /// The voice of a sub-agent.
    voice: Option<String>,
    /// A question of a sub-agent: its name and the answer.
    ask: Option<(String, oneshot::Sender<bool>)>,
}

impl From<String> for Note {
    fn from(text: String) -> Self {
        Self {
            text,
            voice: None,
            ask: None,
        }
    }
}

impl Core {
    /// The agent starts awake.
    pub fn new(config: Config) -> Self {
        let now = Instant::now();
        Self {
            gate: Gate::new(&config),
            barge: BargeIn::new(&config),
            config,
            state: State::Listening,
            since: now,
            seq: 0,
            awake: true,
            muted: Muted::default(),
            key: Key::default(),
            turn: 0,
            last_transcript: String::new(),
            last_reply: String::new(),
            replies: 0,
            asking: None,
            request: String::new(),
            doing: State::Thinking,
            confirm: None,
            unclear: 0,
            pending: Vec::new(),
            paused: false,
            levels: VecDeque::new(),
            sound_end: None,
            said: VecDeque::new(),
            played: String::new(),
            queue: Vec::new(),
            ended: now,
            recorded: 0,
            mic_down: false,
            code: (String::new(), "", String::new()),
            notes: VecDeque::new(),
            tts_failed: None,
            subs: BTreeMap::new(),
            sub_confirm: None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// The parameters of `mc agent.set` without the name: the state, the
    /// request and the last reply.
    pub fn agent(&self) -> Value {
        json!({
            "state": self.state.agent(),
            "task": self.request,
            "said": self.last_reply,
        })
    }

    /// The parameters of `mc agent.set` for the coding task as the
    /// sub-agent `coder`; none before the first task.
    pub fn coder(&self) -> Option<Value> {
        let (task, state, said) = &self.code;
        (!state.is_empty())
            .then(|| json!({"name": "coder", "state": state, "task": task, "said": said}))
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
            "queued": self.queue.len(),
            "coder": self.coder(),
            "agents": self.subs.values().collect::<Vec<_>>(),
            "health": crate::health::record(),
        })
    }

    fn set(&mut self, state: State) {
        if state != self.state {
            self.state = state;
            self.since = Instant::now();
            self.seq += 1;
        }
    }

    /// The state from what goes on: an utterance first, then its text,
    /// then the reply.
    fn refresh(&mut self) {
        // A question of a sub-agent that timed out is gone.
        if self
            .sub_confirm
            .as_ref()
            .is_some_and(|(_, a)| a.is_closed())
        {
            self.sub_confirm = None;
        }
        let working =
            self.code.1 == "working" || self.subs.values().any(|s| s["state"] == "working");
        let state = if self.gate.pcm().is_some() {
            State::Recording
        } else if self.pending.iter().any(|p| p.heard) {
            State::Transcribing
        } else if self.asking.is_some() {
            if self.confirm.is_some() {
                State::Confirming
            } else if self.paused || self.sounding(Instant::now()) {
                State::Speaking
            } else {
                self.doing
            }
        } else if self.sub_confirm.is_some() {
            State::Confirming
        } else if self.awake && working {
            State::Working
        } else if self.awake {
            State::Listening
        } else {
            State::Sleeping
        };
        self.set(state);
    }

    fn earcon(&self, kind: Earcon) -> Option<Effect> {
        (self.config.earcons && !self.muted.speaker).then_some(Effect::Earcon(kind))
    }

    pub fn handle(&mut self, input: Input) -> Vec<Effect> {
        let mut effects = match input {
            Input::Frame(frame) => {
                self.mic_down = false;
                self.frame(&frame)
            }
            Input::MicDown if self.mic_down => Vec::new(),
            Input::MicDown => {
                self.mic_down = true;
                // The cut utterance is gone with the mic.
                self.gate.reset();
                self.earcon(Earcon::Error).into_iter().collect()
            }
            Input::PttDown => self.key(true),
            Input::PttUp => self.key(false),
            Input::Call(call, reply) => {
                let effects = self.call(call);
                self.refresh();
                // The caller may have gone after its timeout.
                let _ = reply.send(self.record());
                effects
            }
            Input::Settings(config) => {
                self.gate.configure(&config);
                self.barge.configure(&config);
                self.config = *config;
                Vec::new()
            }
            Input::ChoraGone => vec![Effect::Exit],
            Input::Transcript {
                turn,
                interim,
                result,
            } => self.transcript(turn, interim, result),
            Input::Reply(reply, event) => self.reply(reply, event),
            Input::Code(event) => self.code(event),
            Input::Agent(event) => self.sub(event),
            // The loop of the agent takes them before.
            Input::Quit | Input::AgentCall(..) => Vec::new(),
        };
        // A barge-in that came to nothing: the speech goes on.
        if self.paused && self.gate.pcm().is_none() && self.pending.is_empty() {
            self.paused = false;
            effects.push(Effect::Speech(Control::Resume { skip: false }));
        }
        self.refresh();
        // A coding result waits for the agent to listen: it cuts no
        // speech of the user.
        if matches!(self.state, State::Listening | State::Working)
            && let Some(note) = self.notes.pop_front()
        {
            let mut ask = self.ask(note.text, true);
            if let Some(Effect::Ask { voice, .. }) = ask.first_mut() {
                *voice = note.voice;
            }
            effects.extend(ask);
            // The question waits from when it is spoken.
            self.sub_confirm = note.ask;
            self.unclear = 0;
            self.refresh();
        }
        effects
    }

    /// The sub-agents by name, for the HUD.
    pub fn subs(&self) -> &BTreeMap<String, Value> {
        &self.subs
    }

    fn sub(&mut self, event: agents::Event) -> Vec<Effect> {
        match event {
            agents::Event::State(record) => {
                let name = record["name"].as_str().unwrap_or_default().to_owned();
                self.subs.insert(name, record);
            }
            agents::Event::Gone(name) => {
                self.subs.remove(&name);
                if self.sub_confirm.as_ref().is_some_and(|(n, _)| *n == name) {
                    self.sub_confirm = None;
                }
            }
            agents::Event::Said { voice, text } => self.notes.push_back(Note {
                text,
                voice: Some(voice),
                ask: None,
            }),
            agents::Event::Ask {
                name,
                voice,
                prompt,
                answer,
            } => self.notes.push_back(Note {
                text: prompt,
                voice: Some(voice),
                ask: Some((name, answer)),
            }),
        }
        self.seq += 1;
        Vec::new()
    }

    /// An utterance that starts with the name of a sub-agent goes to it
    /// without the LLM: a cancel word cancels its request, yes or no
    /// answers its question, else the rest is its prompt.
    fn address(&mut self, name: String, rest: String) -> Vec<Effect> {
        let asks = self.sub_confirm.as_ref().is_some_and(|(n, _)| *n == name);
        crate::log::info(&format!("to {name}: {rest:?}"));
        match words::matches(&self.config, &rest, 0, asks) {
            Some(Word::Cancel | Word::CodeCancel) => std::iter::once(Effect::AgentCancel(name))
                .chain(self.earcon(Earcon::Cancel))
                .collect(),
            Some(yes @ (Word::Yes | Word::No)) => self.yes_no(yes == Word::Yes),
            _ if rest.is_empty() => Vec::new(),
            _ => vec![Effect::AgentPrompt { name, text: rest }],
        }
    }

    fn code(&mut self, event: coder::Event) -> Vec<Effect> {
        match event {
            coder::Event::Started(task) => self.code = (task, "working", String::new()),
            coder::Event::Step(step) => self.code.2 = step,
            coder::Event::Done { ok, said } => {
                self.code.1 = if ok { "done" } else { "blocked" };
                self.code.2.clone_from(&said);
                self.notes.push_back(said.into());
            }
        }
        self.seq += 1;
        Vec::new()
    }

    /// The push-to-talk key: down stops a reply and records.
    fn key(&mut self, down: bool) -> Vec<Effect> {
        crate::log::debug(if down { "ptt down" } else { "ptt up" });
        self.key.down = down;
        if down && self.asking.is_some() {
            crate::log::info("ptt down: the reply stops");
            return self.stop(Some(Earcon::Cancel));
        }
        Vec::new()
    }

    /// Stops the reply in progress; its played words wait for the note
    /// in the history.
    fn stop(&mut self, earcon: Option<Earcon>) -> Vec<Effect> {
        if let Some(reply) = self.asking.take() {
            crate::log::info(&format!("reply={reply} stopped"));
        }
        self.confirm = None;
        self.paused = false;
        self.queue.clear();
        self.sound_end = None;
        std::iter::once(Effect::Stop(std::mem::take(&mut self.played)))
            .chain(earcon.and_then(|e| self.earcon(e)))
            .collect()
    }

    /// Starts a reply to the queued texts and `text`; with `say`, a
    /// reply that says `text`.
    fn ask(&mut self, text: String, say: bool) -> Vec<Effect> {
        let text = if say {
            text
        } else {
            if !text.is_empty() {
                self.queue.push(text);
            }
            std::mem::take(&mut self.queue).join(" ")
        };
        self.replies += 1;
        self.asking = Some(self.replies);
        if !say {
            self.request.clone_from(&text);
        }
        self.doing = State::Thinking;
        self.unclear = 0;
        self.played.clear();
        self.sound_end = None;
        vec![Effect::Ask {
            reply: self.replies,
            text,
            speak: !self.muted.speaker,
            ended: self.ended,
            say,
            voice: None,
        }]
    }

    /// Whether a confirmation waits: of the reply or of a sub-agent.
    fn confirming(&self) -> bool {
        self.confirm.is_some() || self.sub_confirm.is_some()
    }

    /// Answers the confirmation that waits; that of the reply first.
    fn answer(&mut self, yes: bool) {
        let said = if yes { "yes" } else { "no" };
        if let Some(answer) = self.confirm.take() {
            crate::log::info(&format!("confirm: {said}"));
            let _ = answer.send(yes);
            self.doing = if yes { State::Tool } else { State::Thinking };
        } else if let Some((name, answer)) = self.sub_confirm.take() {
            crate::log::info(&format!("confirm {name}: {said}"));
            let _ = answer.send(yes);
        }
    }

    fn sleep(&mut self) -> Vec<Effect> {
        self.gate.reset();
        self.awake = false;
        let mut effects = if self.asking.is_some() {
            self.stop(None)
        } else {
            Vec::new()
        };
        self.queue.clear();
        effects.extend(self.earcon(Earcon::Sleep));
        effects
    }

    fn wake(&mut self) -> Vec<Effect> {
        self.awake = true;
        self.earcon(Earcon::Wake).into_iter().collect()
    }

    fn reply(&mut self, reply: u64, event: Reply) -> Vec<Effect> {
        // A late event of a stopped reply; a dropped confirmation is a no.
        if self.asking != Some(reply) {
            return Vec::new();
        }
        match event {
            Reply::Chat(chat::Event::Thinking) => self.doing = State::Thinking,
            Reply::Chat(chat::Event::Tool(_)) => self.doing = State::Tool,
            Reply::Chat(chat::Event::Confirm { answer, .. }) => {
                self.confirm = Some(answer);
                self.unclear = 0;
            }
            Reply::Failed if self.tts_failed != Some(reply) => {
                self.tts_failed = Some(reply);
                return self.earcon(Earcon::Error).into_iter().collect();
            }
            Reply::Chat(chat::Event::Sentence(_)) | Reply::Failed => {}
            Reply::Played { at, pcm } => {
                let frame = Duration::from_millis(FRAME_MS.into());
                let levels = pcm.chunks(crate::audio::FRAME).map(crate::pcm::rms);
                for (k, level) in (0u32..).zip(levels) {
                    self.levels.push_back((at + frame * k, level));
                    let end = at + frame * (k + 1);
                    self.sound_end = Some(self.sound_end.map_or(end, |e| e.max(end)));
                }
            }
            Reply::Said { text, at, len } => {
                self.said.push_back((at + len, text.clone()));
                self.played.push_str(&text);
                self.played.push(' ');
                return vec![Effect::Say { text, at, len }];
            }
            Reply::Done(text) => {
                self.last_reply = text;
                self.asking = None;
                self.confirm = None;
                self.paused = false;
                self.played.clear();
                self.seq += 1;
                if !self.queue.is_empty() {
                    return self.ask(String::new(), false);
                }
            }
        }
        Vec::new()
    }

    /// Whether a text is the agent's own voice from the speaker.
    fn echo(&mut self, text: &str) -> bool {
        let now = Instant::now();
        while self
            .said
            .front()
            .is_some_and(|(end, _)| *end + ECHO_MEMORY < now)
        {
            self.said.pop_front();
        }
        let said: Vec<&str> = self.said.iter().map(|(_, t)| t.as_str()).collect();
        words::is_echo(text, &said.join(" "))
    }

    /// An interim text of the utterance in progress.
    fn interim_text(&mut self, turn: u64, result: Result<Transcript, String>) {
        // A late interim text drops.
        if self.gate.pcm().is_none() || turn != self.turn + 1 {
            return;
        }
        match result {
            Ok(t) => {
                crate::log::info(&format!(
                    "turn={turn} interim={}ms text={:?}",
                    t.after_commit.as_millis(),
                    t.text
                ));
                self.last_transcript = t.text;
                self.seq += 1;
            }
            Err(e) => crate::log::warn(&format!("turn={turn} interim: {e}")),
        }
    }

    fn transcript(
        &mut self,
        turn: u64,
        interim: bool,
        result: Result<Transcript, String>,
    ) -> Vec<Effect> {
        if interim {
            self.interim_text(turn, result);
            return Vec::new();
        }
        // The final text of an older utterance still counts: no speech is
        // dropped.
        let at = self.pending.iter().position(|p| p.turn == turn);
        let (length_ms, heard) = at.map_or((u32::MAX, self.awake), |at| {
            let p = self.pending.remove(at);
            (p.length_ms, p.heard)
        });
        let text = match result {
            Ok(t) => {
                crate::log::info(&format!(
                    "turn={turn} asr={}ms text={:?}",
                    t.after_commit.as_millis(),
                    t.text
                ));
                self.last_transcript = t.text;
                self.seq += 1;
                self.last_transcript.trim().to_owned()
            }
            Err(e) => {
                crate::log::error(&format!("turn={turn} asr: {e}"));
                // A request says so when the agent listens again.
                let phrase = &self.config.phrases.asr_failed;
                if heard && !self.notes.iter().any(|n| n.text == *phrase) {
                    self.notes.push_back(phrase.clone().into());
                }
                return self.earcon(Earcon::Error).into_iter().collect();
            }
        };
        let word = words::matches(&self.config, &text, length_ms, self.confirming());
        if let Some(word) = word {
            crate::log::info(&format!("turn={turn} word={word:?}"));
        }
        if !heard {
            return if word == Some(Word::Wake) {
                self.wake()
            } else {
                Vec::new()
            };
        }
        match word {
            Some(Word::CodeCancel) => std::iter::once(Effect::StopCode)
                .chain(self.earcon(Earcon::Cancel))
                .collect(),
            Some(Word::Cancel) if self.asking.is_some() => self.stop(Some(Earcon::Cancel)),
            Some(Word::Cancel) => {
                self.queue.clear();
                self.earcon(Earcon::Cancel).into_iter().collect()
            }
            Some(Word::Sleep) => self.sleep(),
            Some(Word::Wake) => Vec::new(),
            Some(yes @ (Word::Yes | Word::No)) => self.yes_no(yes == Word::Yes),
            None => self.text(text),
        }
    }

    /// A spoken yes or no: the cut prompt does not play again.
    fn yes_no(&mut self, yes: bool) -> Vec<Effect> {
        self.answer(yes);
        if !self.paused {
            return Vec::new();
        }
        self.paused = false;
        vec![Effect::Speech(Control::Resume { skip: true })]
    }

    /// A text without a command word.
    fn text(&mut self, text: String) -> Vec<Effect> {
        if text.is_empty() || self.echo(&text) {
            crate::log::info("empty or echo");
            return Vec::new();
        }
        if let Some((name, rest)) = words::addressed(&text, self.subs.keys()) {
            return self.address(name, rest);
        }
        if self.confirming() {
            self.unclear += 1;
            crate::log::info(&format!("confirm: unclear answer {}", self.unclear));
            if self.unclear >= 2 {
                self.answer(false);
            }
            return Vec::new();
        }
        if self.asking.is_none() {
            return self.ask(text, false);
        }
        if self.paused {
            // A barge-in with new words: the reply ends, the words start
            // the next one.
            let mut effects = self.stop(None);
            effects.extend(self.ask(text, false));
            return effects;
        }
        crate::log::info(&format!("queued: {text:?}"));
        self.queue.push(text);
        Vec::new()
    }

    fn call(&mut self, call: Call) -> Vec<Effect> {
        match call {
            Call::Confirm(yes) => self.yes_no(yes),
            Call::Cancel if self.asking.is_some() => self.stop(Some(Earcon::Cancel)),
            Call::Cancel if self.gate.pcm().is_some() => {
                self.gate.reset();
                self.earcon(Earcon::Cancel).into_iter().collect()
            }
            Call::State | Call::Cancel => Vec::new(),
            Call::Sleep => self.sleep(),
            Call::Wake => self.wake(),
            Call::Talk(down) => self.key(down),
            Call::Mute { mic, speaker } => {
                let before = self.muted;
                self.muted.mic = mic.unwrap_or(before.mic);
                self.muted.speaker = speaker.unwrap_or(before.speaker);
                if self.muted != before {
                    self.seq += 1;
                }
                Vec::new()
            }
        }
    }

    /// The loudest played frame near `now`: the echo reaches the mic a
    /// little later.
    fn played_level(&mut self, now: Instant) -> f64 {
        let window = Duration::from_millis(u64::from(FRAME_MS) * ECHO_WINDOW as u64);
        while self
            .levels
            .front()
            .is_some_and(|(at, _)| *at + window < now)
        {
            self.levels.pop_front();
        }
        self.levels
            .iter()
            .take_while(|(at, _)| *at <= now)
            .fold(0.0, |max, (_, level)| f64::max(max, *level))
    }

    /// Whether the reply speaks: its sound plays, or ran dry between
    /// two sentences a moment ago.
    fn sounding(&self, now: Instant) -> bool {
        // ponytail: a fixed gap; the speech could report its queue.
        self.sound_end.is_some_and(|end| now < end + SOUND_GAP)
    }

    fn frame(&mut self, frame: &Frame) -> Vec<Effect> {
        let now = Instant::now();
        let playing = self.asking.is_some() && !self.paused && self.sounding(now);
        if playing && self.gate.pcm().is_none() && !self.key.down {
            // Only the barge-in gate listens over the agent's sound.
            let level = self.played_level(now);
            if !self.muted.mic && self.barge.push(frame, level) {
                crate::log::info(&format!(
                    "barge-in played={level:.0} coupling={:.3}",
                    self.barge.coupling()
                ));
                self.paused = true;
                self.sound_end = Some(now);
                // The speech that made the barge-in, and the pre-roll before it.
                self.gate
                    .open(self.config.barge_in_ms + self.config.preroll_ms);
                self.recorded = 0;
                return vec![Effect::Speech(Control::Pause), Effect::Mark("barge")];
            }
            // The gate keeps its pre-roll.
            self.gate.push(frame, false, false);
            return Vec::new();
        }
        self.barge.reset();
        self.key.used |= self.key.down;
        match self.gate.push(frame, !self.muted.mic, self.key.down) {
            None => self.interim().into_iter().collect(),
            Some(gate::Event::Onset) => {
                self.recorded = 0;
                vec![Effect::Mark("onset")]
            }
            Some(gate::Event::Noise { speech_ms }) => {
                self.key.used = false;
                crate::log::debug(&format!("noise speech={speech_ms}ms"));
                vec![Effect::Mark("noise")]
            }
            Some(gate::Event::End {
                pcm,
                speech_ms,
                end_wait_ms,
            }) => {
                self.turn += 1;
                self.ended = now;
                let length_ms =
                    u32::try_from(pcm.len() * 1000 / crate::pcm::RATE as usize).unwrap_or(u32::MAX);
                self.pending.push(Pending {
                    turn: self.turn,
                    length_ms,
                    heard: self.awake || std::mem::take(&mut self.key.used),
                });
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

    /// A frame of loud speech, as over the agent's sound.
    fn loud() -> Input {
        Input::Frame(Frame {
            samples: vec![3000; 768],
            prob: 0.95,
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

    fn quiet(core: &mut Core, frames: usize) -> Vec<Effect> {
        (0..frames).flat_map(|_| core.handle(frame(0.0))).collect()
    }

    fn text(turn: u64, interim: bool, result: Result<&str, &str>) -> Input {
        Input::Transcript {
            turn,
            interim,
            result: result
                .map(|text| Transcript {
                    text: text.into(),
                    after_commit: Duration::ZERO,
                })
                .map_err(Into::into),
        }
    }

    /// One short utterance (about 1.2 s) with its final text.
    fn say(core: &mut Core, words: &str) -> Vec<Effect> {
        let mut effects = speak(core, 10);
        effects.extend(quiet(core, 26));
        let turn = effects
            .iter()
            .find_map(|e| match e {
                Effect::Utterance { turn, .. } => Some(*turn),
                _ => None,
            })
            .expect("an utterance");
        effects.extend(core.handle(text(turn, false, Ok(words))));
        effects
    }

    fn reply(id: u64, event: chat::Event) -> Input {
        Input::Reply(id, Reply::Chat(event))
    }

    /// Sound of the reply for `ms` from now, at the level 1000.
    fn sound(id: u64, ms: u32) -> Input {
        Input::Reply(
            id,
            Reply::Played {
                at: Instant::now(),
                pcm: vec![1000; (ms * 24) as usize],
            },
        )
    }

    fn asks(effects: &[Effect]) -> Vec<(u64, &str)> {
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::Ask { reply, text, .. } => Some((*reply, text.as_str())),
                _ => None,
            })
            .collect()
    }

    fn no_ask(effects: &[Effect]) -> bool {
        !effects.iter().any(|e| matches!(e, Effect::Ask { .. }))
    }

    fn stop() -> Effect {
        Effect::Stop(String::new())
    }

    /// A core whose reply 1 runs.
    fn asking() -> Core {
        let mut core = Core::new(Config::default());
        let effects = say(&mut core, "hallo");
        assert_eq!(asks(&effects), [(1, "hallo")]);
        core
    }

    /// docs/voice.md section 3: `working` while a coding task runs.
    #[test]
    fn a_coding_task_shows_as_working_and_as_the_agent_coder() {
        let mut core = Core::new(Config::default());
        assert_eq!(core.coder(), None);
        core.handle(Input::Code(coder::Event::Started("Baue es.".into())));
        assert_eq!(core.state(), State::Working);
        core.handle(Input::Code(coder::Event::Step("Edit a.rs".into())));
        assert_eq!(
            core.record()["coder"],
            json!({"name": "coder", "state": "working", "task": "Baue es.", "said": "Edit a.rs"})
        );
    }

    /// The result of a coding task waits until the user and the reply
    /// are done, then is said.
    #[test]
    fn a_coding_result_waits_for_the_agent_to_listen() {
        let mut core = Core::new(Config::default());
        speak(&mut core, 10);
        let done = coder::Event::Done {
            ok: true,
            said: "Fertig.".into(),
        };
        assert!(no_ask(&core.handle(Input::Code(done))), "no cut speech");
        quiet(&mut core, 26);
        let effects = core.handle(text(1, false, Ok("Wie spät ist es?")));
        assert_eq!(asks(&effects), [(1, "Wie spät ist es?")]);
        let effects = core.handle(Input::Reply(1, Reply::Done("Zehn.".into())));
        assert!(
            matches!(&effects[..], [Effect::Ask { reply: 2, text, say: true, .. }] if text == "Fertig."),
            "{effects:?}"
        );
        assert_eq!(core.coder().unwrap()["state"], "done");
        core.handle(Input::Reply(2, Reply::Done("Fertig.".into())));
        assert_eq!(core.state(), State::Listening);
        assert_eq!(core.record()["last_reply"], "Fertig.");
    }

    /// A core with the working ACP agent `coder`.
    fn with_coder() -> Core {
        let mut core = Core::new(Config::default());
        let record = json!({"name": "coder", "state": "working"});
        core.handle(Input::Agent(agents::Event::State(record)));
        core
    }

    /// An ACP agent: its state shows, its question is spoken in its voice
    /// when the user is done and waits for yes or no.
    #[test]
    fn a_sub_agent_asks_in_its_voice_when_the_user_is_done() {
        let mut core = with_coder();
        assert_eq!(core.state(), State::Working);
        assert_eq!(core.record()["agents"][0]["name"], "coder");
        speak(&mut core, 10);
        let (answer, mut rx) = oneshot::channel();
        let ask = agents::Event::Ask {
            name: "coder".into(),
            voice: "ryan".into(),
            prompt: "Coder fragt: Write x. Erlauben?".into(),
            answer,
        };
        assert!(no_ask(&core.handle(Input::Agent(ask))), "no cut speech");
        quiet(&mut core, 26);
        let effects = core.handle(text(1, false, Ok("Wie spät ist es?")));
        assert_eq!(asks(&effects), [(1, "Wie spät ist es?")]);
        let effects = core.handle(Input::Reply(1, Reply::Done("Zehn.".into())));
        let [
            Effect::Ask {
                reply: 2,
                voice,
                say: true,
                ..
            },
        ] = &effects[..]
        else {
            panic!("{effects:?}");
        };
        assert_eq!(voice.as_deref(), Some("ryan"));
        core.handle(Input::Reply(2, Reply::Done("Coder fragt".into())));
        assert_eq!(core.state(), State::Confirming);
        say(&mut core, "Ja.");
        assert_eq!(rx.try_recv(), Ok(true));
        assert_eq!(core.state(), State::Working);
    }

    /// The name of an ACP agent addresses it without the LLM.
    #[test]
    fn a_sub_agent_hears_its_name() {
        let mut core = with_coder();
        let effects = say(&mut core, "Koder, schreib noch Tests.");
        assert!(
            effects.contains(&Effect::AgentPrompt {
                name: "coder".into(),
                text: "schreib noch Tests.".into()
            }),
            "{effects:?}"
        );
        let effects = say(&mut core, "Coder, stopp!");
        assert!(
            effects.contains(&Effect::AgentCancel("coder".into())),
            "{effects:?}"
        );
        core.handle(Input::Agent(agents::Event::Gone("coder".into())));
        assert_eq!(core.state(), State::Listening);
        let effects = say(&mut core, "Coder, hallo");
        assert_eq!(
            asks(&effects),
            [(1, "Coder, hallo")],
            "a gone agent: the LLM"
        );
    }

    #[test]
    fn speech_records_an_utterance_and_its_text_starts_a_reply() {
        let mut core = Core::new(Config::default());
        assert_eq!(core.state(), State::Listening, "the agent starts awake");
        let effects = speak(&mut core, 10);
        assert_eq!(effects, [Effect::Mark("onset")]);
        assert_eq!(core.state(), State::Recording);
        let effects = quiet(&mut core, 25);
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
                reply, text, speak, ..
            },
        ] = &effects[..]
        else {
            panic!("{effects:?}");
        };
        assert_eq!((*reply, text.as_str(), *speak), (1, "hallo", true));
        assert_eq!(core.state(), State::Thinking);
        assert_eq!(call(&mut core, Call::State).1["last_transcript"], " hallo ");
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
        core.handle(sound(1, 2000));
        assert_eq!(core.state(), State::Speaking, "the sound plays");
        core.handle(reply(1, chat::Event::Thinking));
        assert_eq!(core.state(), State::Speaking, "the sound goes on");
        assert!(
            speak(&mut core, 20).is_empty(),
            "quiet speech is no barge-in"
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
        for input in [Input::PttDown, Input::Call(Call::Cancel, mpsc::channel().0)] {
            let mut core = asking();
            let (answer, mut rx) = oneshot::channel();
            core.handle(reply(
                1,
                chat::Event::Confirm {
                    prompt: "Wirklich?".into(),
                    answer,
                },
            ));
            let effects = core.handle(input);
            assert_eq!(effects, [stop(), Effect::Earcon(Earcon::Cancel)]);
            assert_eq!(core.state(), State::Listening);
            assert!(rx.try_recv().is_err(), "a dropped answer is a no");
            core.handle(sound(1, 1000));
            assert_eq!(core.state(), State::Listening, "a late event of reply 1");
        }
        let mut core = asking();
        let (effects, record) = call(&mut core, Call::Sleep);
        assert_eq!(effects, [stop(), Effect::Earcon(Earcon::Sleep)]);
        assert_eq!(record["state"], "sleeping");
    }

    #[test]
    fn a_cancel_word_stops_the_reply_with_the_words_that_played() {
        let mut core = asking();
        let said = Reply::Said {
            text: "Es ist zehn Uhr.".into(),
            at: Instant::now(),
            len: Duration::from_secs(1),
        };
        let effects = core.handle(Input::Reply(1, said));
        assert!(matches!(&effects[..], [Effect::Say { .. }]), "{effects:?}");
        let effects = say(&mut core, "Stopp!");
        let played = Effect::Stop("Es ist zehn Uhr. ".into());
        assert!(effects.contains(&played), "{effects:?}");
        assert!(effects.contains(&Effect::Earcon(Earcon::Cancel)));
        assert_eq!(core.state(), State::Listening);
        let effects = say(&mut core, "Wie spät ist es morgen?");
        assert_eq!(asks(&effects), [(2, "Wie spät ist es morgen?")]);
    }

    #[test]
    fn a_barge_in_pauses_the_speech_and_its_text_decides() {
        let config = Config {
            barge_in_ms: 96,
            ..Config::default()
        };
        // Echo or nothing: the speech goes on from the cut sentence.
        let mut core = Core::new(config.clone());
        say(&mut core, "hallo");
        core.handle(sound(1, 5000));
        let effects: Vec<_> = (0..3).flat_map(|_| core.handle(loud())).collect();
        assert_eq!(
            effects,
            [Effect::Speech(Control::Pause), Effect::Mark("barge")]
        );
        assert_eq!(core.state(), State::Recording);
        (0..8).for_each(|_| drop(core.handle(loud())));
        let effects = quiet(&mut core, 26);
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::Utterance { .. }))
        );
        assert_eq!(core.state(), State::Transcribing);
        let effects = core.handle(text(2, false, Ok("")));
        assert_eq!(effects, [Effect::Speech(Control::Resume { skip: false })]);
        assert_eq!(core.state(), State::Speaking, "the speech goes on");
        // New words: the reply ends and they start the next one.
        core.handle(sound(1, 5000));
        (0..11).for_each(|_| drop(core.handle(loud())));
        quiet(&mut core, 26);
        let effects = core.handle(text(3, false, Ok("Und morgen?")));
        assert_eq!(effects[0], stop());
        assert_eq!(asks(&effects), [(2, "Und morgen?")]);
        // A muted mic barges in nowhere.
        call(
            &mut core,
            Call::Mute {
                mic: Some(true),
                speaker: None,
            },
        );
        core.handle(sound(2, 5000));
        assert!((0..5).flat_map(|_| core.handle(loud())).next().is_none());
    }

    #[test]
    fn speech_during_a_reply_waits_in_the_queue_and_starts_the_next() {
        let mut core = asking();
        core.handle(reply(1, chat::Event::Tool("query".into())));
        say(&mut core, "Und mach das Fenster größer.");
        assert_eq!(core.state(), State::Tool, "the reply goes on");
        assert_eq!(call(&mut core, Call::State).1["queued"], 1);
        let effects = core.handle(Input::Reply(1, Reply::Done("Gut.".into())));
        assert_eq!(asks(&effects), [(2, "Und mach das Fenster größer.")]);
        // A cancel word drops the queue with the reply.
        say(&mut core, "Noch etwas.");
        say(&mut core, "Vergiss es.");
        assert_eq!(call(&mut core, Call::State).1["queued"], 0);
        assert_eq!(core.state(), State::Listening);
    }

    #[test]
    fn spoken_yes_and_no_answer_and_two_unclear_answers_are_a_no() {
        let confirming = |core: &mut Core| {
            let (answer, rx) = oneshot::channel();
            core.handle(reply(
                1,
                chat::Event::Confirm {
                    prompt: "Wirklich?".into(),
                    answer,
                },
            ));
            rx
        };
        let mut core = asking();
        let mut rx = confirming(&mut core);
        say(&mut core, "Ja, bitte.");
        assert_eq!(rx.try_recv(), Ok(true));
        assert_eq!(core.state(), State::Tool);
        let mut rx = confirming(&mut core);
        say(&mut core, "Lieber nicht.");
        assert_eq!(rx.try_recv(), Ok(false));
        let mut rx = confirming(&mut core);
        say(&mut core, "Was meinst du?");
        assert!(rx.try_recv().is_err(), "one unclear answer waits");
        say(&mut core, "Keine Ahnung.");
        assert_eq!(rx.try_recv(), Ok(false));
        assert_eq!(call(&mut core, Call::State).1["queued"], 0);
    }

    #[test]
    fn sleep_and_wake_words_and_the_code_cancel_word() {
        let mut core = Core::new(Config::default());
        let effects = say(&mut core, "Schlaf.");
        assert_eq!(effects.last(), Some(&Effect::Earcon(Earcon::Sleep)));
        assert_eq!(core.state(), State::Sleeping);
        let effects = say(&mut core, "Wie spät ist es?");
        assert!(no_ask(&effects), "asleep, only a wake word counts");
        assert_eq!(core.state(), State::Sleeping);
        let effects = say(&mut core, "Hallo Cora.");
        assert_eq!(effects.last(), Some(&Effect::Earcon(Earcon::Wake)));
        assert_eq!(core.state(), State::Listening);
        let effects = say(&mut core, "Coding-Aufgabe abbrechen.");
        assert!(effects.contains(&Effect::StopCode), "{effects:?}");
        assert!(no_ask(&effects), "{effects:?}");
    }

    #[test]
    fn the_agents_own_words_come_back_as_echo() {
        let mut core = asking();
        core.handle(Input::Reply(
            1,
            Reply::Said {
                text: "Heute hast du drei Termine.".into(),
                at: Instant::now(),
                len: Duration::from_secs(2),
            },
        ));
        core.handle(Input::Reply(1, Reply::Done("Heute ...".into())));
        let effects = say(&mut core, "drei Termine");
        assert!(no_ask(&effects), "{effects:?}");
    }

    #[test]
    fn the_key_records_asleep_and_its_text_gets_a_reply() {
        let mut core = Core::new(Config::default());
        call(&mut core, Call::Sleep);
        call(&mut core, Call::Talk(true));
        quiet(&mut core, 10);
        assert_eq!(core.state(), State::Recording);
        call(&mut core, Call::Talk(false));
        let effects = quiet(&mut core, 1);
        assert!(matches!(
            &effects[1],
            Effect::Utterance { awake: false, .. }
        ));
        let effects = core.handle(text(1, false, Ok("Wie spät ist es?")));
        assert_eq!(asks(&effects), [(1, "Wie spät ist es?")]);
        core.handle(Input::Reply(1, Reply::Done("Zehn.".into())));
        assert_eq!(core.state(), State::Sleeping, "and asleep again");
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
        quiet(&mut core, 30);
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
    fn a_failed_transcript_sounds_the_error_and_says_so_once() {
        let mut core = Core::new(Config::default());
        speak(&mut core, 10);
        quiet(&mut core, 30);
        let old = core.handle(text(0, false, Err("old turn")));
        assert_eq!(old, [Effect::Earcon(Earcon::Error)]);
        assert_eq!(core.state(), State::Transcribing);
        let effects = core.handle(text(1, false, Err("timeout")));
        assert_eq!(effects[0], Effect::Earcon(Earcon::Error));
        assert_eq!(
            asks(&effects[1..]),
            [(1, "I did not understand you.")]
        );
        core.handle(Input::Reply(1, Reply::Done(String::new())));
        assert_eq!(core.state(), State::Listening, "said once");
    }

    #[test]
    fn a_failing_tts_sounds_the_error_once_per_reply() {
        let mut core = asking();
        let failed = || Input::Reply(1, Reply::Failed);
        assert_eq!(core.handle(failed()), [Effect::Earcon(Earcon::Error)]);
        assert_eq!(core.handle(failed()), []);
    }

    #[test]
    fn cancel_drops_the_utterance_with_an_earcon() {
        let mut core = Core::new(Config::default());
        speak(&mut core, 10);
        let (effects, record) = call(&mut core, Call::Cancel);
        assert_eq!(effects, [Effect::Earcon(Earcon::Cancel)]);
        assert_eq!(record["state"], "listening");
        let after = quiet(&mut core, 40);
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
        let effects = quiet(&mut core, 25);
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
