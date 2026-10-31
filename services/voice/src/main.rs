//! chora-voice: the voice agent of chora (docs/voice.md). The mic goes
//! through a speech gate with Silero VAD; the agent is a chora client
//! with methods, settings and a push-to-talk key.

// Some modules still unwrap; they are cleaned in the follow-up package.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod agents;
mod asr;
mod audio;
mod chat;
mod client;
mod coder;
mod config;
mod core;
mod earcon;
#[cfg(test)]
mod fake;
mod gate;
mod health;
mod live;
mod llm;
mod log;
mod mc;
mod pcm;
mod record;
mod replay;
mod speech;
mod tools;
mod tts;
mod vad;
mod words;

use std::{
    io::IsTerminal,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use chora_kit::settings;
use clap::{Parser, Subcommand};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{
    audio::{Analyzer, FRAME, Mic, Speaker},
    chat::Chat,
    config::Config,
    core::{Core, Effect, Input, Reply},
    gate::{FRAME_MS, Gate},
    record::Recorder,
    speech::Control,
    tools::Tools,
    vad::Vad,
};

/// The voice agent of chora.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The client name in chora.
    #[arg(long, default_value = "voice")]
    name: String,
    /// The settings file; $XDG_CONFIG_HOME/chora/chora-voice.yaml without
    /// it.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default settings and exit.
    #[arg(long)]
    print_config: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run the speech gate of the settings over a WAV file (24 kHz mono
    /// PCM16) and print the utterances.
    Gate { file: PathBuf },
    /// Replay recordings (`mic.wav`, `events.jsonl`, `labels.txt`)
    /// through the speech and barge-in gates and print the metrics
    /// against the labels.
    Replay {
        #[arg(required = true)]
        dirs: Vec<PathBuf>,
    },
    /// Transcribe a WAV file (24 kHz mono PCM16) with the ASR.
    Asr { file: PathBuf },
    /// Speak a text with the TTS and print the timing of its sentences.
    Tts {
        text: String,
        /// Write the sound to this WAV file instead of playing it.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Speak a text with the TTS, transcribe its sound with the ASR, and
    /// print the latencies.
    Roundtrip {
        #[arg(default_value = "Hallo Chora. Zeig mir bitte die Agenda für heute.")]
        text: String,
    },
    /// Answer one text with the LLM and the tools of `choractl mcp`
    /// (client NAME-mcp), and print the latencies.
    Ask {
        text: String,
        /// Speak the reply with the TTS and the playback command.
        #[arg(long)]
        speak: bool,
        /// Answer each confirmation with yes; else with no.
        #[arg(long)]
        yes: bool,
    },
    /// Print the mic level and the speech probability of the capture
    /// command twice a second, until Ctrl+C.
    Levels,
}

#[allow(
    clippy::print_stdout,
    clippy::expect_used,
    reason = "the subcommands print their results; the defaults serialize"
)]
fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            // tract logs its kernels at info.
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tract_linalg=warn,rmcp=warn".into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .init();
    let args = Args::parse();
    if args.print_config {
        print!(
            "{}",
            serde_saphyr::to_string(&Config::default()).expect("the defaults serialize")
        );
        return ExitCode::SUCCESS;
    }
    let result = settings::File::<Config>::path(args.config.clone(), "chora-voice")
        .and_then(|path| settings::File::load(path, "{}"))
        .and_then(|settings: settings::File<Config>| {
            let config = settings.value.localized();
            match &args.command {
                Some(Command::Gate { file }) => gate(&config, file),
                Some(Command::Replay { dirs }) => {
                    let dirs: Vec<&Path> = dirs.iter().map(AsRef::as_ref).collect();
                    let (lines, _) = replay::run(&config, &dirs)?;
                    println!("{}", lines.join("\n"));
                    Ok(())
                }
                Some(Command::Asr { file }) => live::asr(&config, file),
                Some(Command::Tts { text, out }) => live::tts(&config, text, out.as_deref()),
                Some(Command::Roundtrip { text }) => live::roundtrip(&config, text),
                Some(Command::Ask { text, speak, yes }) => {
                    live::ask(&config, &mcp_name(&args.name), text, *speak, *yes)
                }
                Some(Command::Levels) => live::levels(&config),
                None => run(args.name, settings),
            }
        });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::error(&e);
            ExitCode::FAILURE
        }
    }
}

/// The client name of the tools: the same name as the agent would need
/// the same `hello` flags, and `choractl mcp` keeps its nodes.
fn mcp_name(name: &str) -> String {
    format!("{name}-mcp")
}

/// SIGTERM and SIGINT quit the loop of the agent, which then stops its
/// sub-agents.
async fn quit_on_signal(tx: mpsc::Sender<Input>) {
    use tokio::signal::unix::{SignalKind, signal};
    let (Ok(mut term), Ok(mut int)) = (
        signal(SignalKind::terminate()),
        signal(SignalKind::interrupt()),
    ) else {
        return;
    };
    tokio::select! {
        _ = term.recv() => {}
        _ = int.recv() => {}
    }
    let _ = tx.send(Input::Quit);
}

/// Runs the agent until chora stays away or a thread ends.
fn run(name: String, settings: settings::File<Config>) -> Result<(), String> {
    let config = settings.value.localized();
    log::info(&format!("settings: {}", settings.file().display()));
    let vad = Vad::new().map_err(|e| format!("the VAD model: {e}"))?;
    let net = tokio::runtime::Runtime::new().map_err(|e| format!("tokio: {e}"))?;
    let (tx, rx) = mpsc::channel();
    let mcp = mcp_name(&name);
    let mut tools = Tools::chora(&config, &mcp);
    tools.own.clone_from(&name);
    let coder = coder::Coder::new(
        config.spawn(&config.coder_cmd),
        coder::data_dir(),
        tx.clone(),
    );
    tools.coder = Some(coder.clone());
    let agents = agents::Agents::new(&config, coder::data_dir(), tx.clone());
    tools.agents = Some(agents.clone());
    let chat = net.block_on(Chat::start(
        config.clone(),
        mcp,
        tools,
        chat::history_file(),
    ))?;
    let mut agent = Agent::new(config.clone(), chat, net, tx.clone());
    agent.coder = Some(coder);
    agent.agents = Some(agents.clone());
    agent.warm();
    agent.net.spawn(quit_on_signal(tx.clone()));
    agent.mic = Some(Mic::start(
        config.capture_cmd,
        Analyzer::new(vad),
        tx.clone(),
    ));
    let (emits, emitted) = mpsc::channel();
    agent.mc = Some(mc::start(&name, emits));
    client::start(name, settings, tx, emitted);
    log::info(&format!("started, {:?}", agent.core.state()));
    let result = loop {
        match rx.recv() {
            Ok(Input::Quit) => break Ok(()),
            Ok(Input::AgentCall(tool, args, reply)) => {
                // agent_start spawns its actor on the runtime.
                let _runtime = agent.net.enter();
                let _ = reply.send(agents.call(&tool, &args));
            }
            Ok(input) => {
                if let Err(e) = agent.step(input) {
                    break Err(e);
                }
            }
            Err(_) => break Err("an audio thread ended".into()),
        }
    };
    // The sub-agents end with the agent.
    agents.shutdown();
    result
}

/// The core and the actors that run its effects.
struct Agent {
    core: Core,
    mic: Option<Mic>,
    speaker: Speaker,
    recorder: Option<Recorder>,
    /// The network actors: ASR, TTS, LLM and tools.
    net: tokio::runtime::Runtime,
    tx: mpsc::Sender<Input>,
    /// An interim transcription runs; the next interval waits for it.
    interim_busy: Arc<AtomicBool>,
    chat: Arc<tokio::sync::Mutex<Chat>>,
    /// The reply in progress: the control of its speech, and its stop
    /// with the words that played.
    reply: Option<(
        UnboundedSender<Control>,
        oneshot::Sender<String>,
        UnboundedSender<String>,
    )>,
    /// The reports to the HUD, and the last reports of the agent and of
    /// its coder.
    mc: Option<mpsc::Sender<mc::Report>>,
    reported: [Option<serde_json::Value>; 2],
    /// The last reports of the sub-agents.
    reported_subs: std::collections::BTreeMap<String, serde_json::Value>,
    /// The state that the indicator shows; none without it.
    dot: Option<core::State>,
    /// The ASR and the TTS for the health probes.
    probe: tokio::sync::watch::Sender<health::Probed>,
    coder: Option<coder::Coder>,
    agents: Option<agents::Agents>,
}

impl Agent {
    fn new(
        config: Config,
        chat: Chat,
        net: tokio::runtime::Runtime,
        tx: mpsc::Sender<Input>,
    ) -> Self {
        Self {
            speaker: Speaker::new(config.playback_cmd.clone()),
            recorder: start_recording(&config.record_dir),
            core: Core::new(config),
            mic: None,
            net,
            tx,
            interim_busy: Arc::default(),
            chat: Arc::new(tokio::sync::Mutex::new(chat)),
            reply: None,
            mc: None,
            reported: [None, None],
            reported_subs: std::collections::BTreeMap::new(),
            dot: None,
            probe: tokio::sync::watch::Sender::new([Err(String::new()), Err(String::new())]),
            coder: None,
            agents: None,
        }
    }

    /// Lets the LLM read the prompt; tries again with the backoff of the
    /// LLM until it answers, so that the cache is warm when it is back,
    /// and then says that the agent is ready. Starts the health probes of
    /// the ASR and the TTS.
    fn warm(&self) {
        let config = self.core.config();
        let _ = self.probe.send_replace([config.asr(), config.tts()]);
        self.net.spawn(health::probe(self.probe.subscribe()));
        let (chat, tx) = (self.chat.clone(), self.tx.clone());
        self.net.spawn(async move {
            while !chat.lock().await.warm().await {
                tokio::time::sleep(health::wait("llm")).await;
            }
            let _ = tx.send(Input::Ready);
        });
    }

    /// Runs an input through the core and its effects.
    fn step(&mut self, mut input: Input) -> Result<(), String> {
        // The agent works with the phrases of `reply_language`.
        match &mut input {
            Input::Settings(config) => **config = config.localized(),
            Input::Code(coder::Event::Done { said, .. }) => {
                *said = self.core.config().code_said(said);
            }
            _ => {}
        }
        self.input(&input);
        for effect in self.core.handle(input) {
            self.effect(effect)?;
        }
        self.report();
        Ok(())
    }

    /// Reports the changes of the agent, its coder and its state for the
    /// indicator to the HUD thread.
    fn report(&mut self) {
        let Some(mc) = &self.mc else {
            return;
        };
        let reports = [Some(self.core.agent()), self.core.coder()];
        for (report, reported) in reports.into_iter().zip(&mut self.reported) {
            if let Some(report) = report
                && Some(&report) != reported.as_ref()
            {
                let _ = mc.send(mc::Report::Agent(report.clone()));
                *reported = Some(report);
            }
        }
        let subs = self.core.subs();
        for (name, record) in subs {
            if self.reported_subs.get(name) != Some(record) {
                let _ = mc.send(mc::Report::Agent(record.clone()));
            }
        }
        for name in self.reported_subs.keys() {
            if !subs.contains_key(name) {
                let _ = mc.send(mc::Report::Gone(name.clone()));
            }
        }
        self.reported_subs.clone_from(subs);
        let dot = self.core.config().indicator.then(|| self.core.state());
        if dot != self.dot {
            let _ = mc.send(mc::Report::State(dot));
            self.dot = dot;
        }
    }

    /// What happens to an input before the core gets it.
    fn input(&mut self, input: &Input) {
        match input {
            Input::Frame(frame) => {
                if let Some(r) = &mut self.recorder
                    && let Err(e) = r.mic(&frame.samples)
                {
                    log::error(&format!("record: {e}; recording stops"));
                    self.recorder = None;
                }
            }
            Input::Reply(_, Reply::Played { at, pcm }) => {
                if let Some(r) = &mut self.recorder {
                    let ahead = at.saturating_duration_since(std::time::Instant::now());
                    let ahead = usize::try_from(ahead.as_millis()).unwrap_or(0) * 24;
                    let _ = r.played(pcm, ahead);
                }
            }
            Input::Settings(new) => {
                let old = self.core.config();
                if new.capture_cmd != old.capture_cmd
                    && let Some(mic) = &self.mic
                {
                    mic.restart(new.capture_cmd.clone());
                }
                if new.record_dir != old.record_dir {
                    self.recorder = start_recording(&new.record_dir);
                }
                self.speaker.set_cmd(new.playback_cmd.clone());
                self.probe.send_if_modified(|servers| {
                    let new = [new.asr(), new.tts()];
                    std::mem::replace(servers, new.clone()) != new
                });
                let (chat, config) = (self.chat.clone(), (**new).clone());
                self.net.spawn(async move {
                    let mut chat = chat.lock().await;
                    match chat.configure(config) {
                        Ok(true) => drop(chat.warm().await),
                        Ok(false) => {}
                        Err(e) => log::error(&e),
                    }
                });
            }
            _ => {}
        }
    }

    fn effect(&mut self, effect: Effect) -> Result<(), String> {
        match effect {
            Effect::Earcon(kind) => {
                let pcm = kind.pcm(self.core.config().earcon_volume);
                if let Err(e) = self.speaker.play(&pcm) {
                    log::error(&format!("playback: {e}"));
                }
                if let Some(r) = &mut self.recorder {
                    let _ = r.played(&pcm, 0);
                }
            }
            Effect::Mark(event) => {
                if let Some(r) = &mut self.recorder {
                    let _ = r.event(event);
                }
            }
            Effect::Utterance {
                turn,
                pcm,
                speech_ms,
                end_wait_ms,
                awake,
            } => {
                let length_ms = pcm.len() * 1000 / pcm::RATE as usize;
                log::info(&format!(
                    "turn={turn} speech={speech_ms}ms length={length_ms}ms \
                     end_wait={end_wait_ms}ms awake={awake}"
                ));
                self.transcribe(turn, false, pcm);
            }
            Effect::Interim { turn, pcm } => {
                if !self.interim_busy.swap(true, Ordering::SeqCst) {
                    self.transcribe(turn, true, pcm);
                }
            }
            ask @ Effect::Ask { .. } => self.ask(ask),
            Effect::Stop(played) => {
                if let Some((_, stop, _)) = self.reply.take() {
                    let _ = stop.send(played);
                }
                self.speaker.stop();
            }
            Effect::Inject(text) => drop(self.reply.as_ref().map(|(.., inject)| inject.send(text))),
            Effect::Speech(control) => {
                if let Some((speech, ..)) = &self.reply {
                    let _ = speech.send(control);
                }
            }
            Effect::StopCode => self.stop_code(),
            Effect::AgentPrompt { name, text } => self.to_agent(&name, Some(&text)),
            Effect::AgentCancel(name) => self.to_agent(&name, None),
            Effect::Say { text, at, len } => {
                if let Some(mc) = &self.mc {
                    let _ = mc.send(mc::Report::Say { text, at, len });
                }
            }
            Effect::Exit => return Err("chora stays away".into()),
        }
        Ok(())
    }
}

impl Agent {
    /// Starts the reply of an `Ask` effect; its events go to the core. A
    /// stop ends the turn with the note of the words that played.
    fn ask(&mut self, ask: Effect) {
        let Effect::Ask {
            reply: id,
            text,
            speak,
            ended,
            say,
            voice,
        } = ask
        else {
            return;
        };
        let (chat, tx) = (self.chat.clone(), self.tx.clone());
        let mut config = self.core.config().clone();
        if let Some(voice) = voice.filter(|v| !v.is_empty()) {
            config.voice = voice;
        }
        let speaker = speak.then(|| self.speaker.clone());
        let (speech, control) = tokio::sync::mpsc::unbounded_channel();
        let (stop, mut stopped) = oneshot::channel::<String>();
        let (inject, inbox) = tokio::sync::mpsc::unbounded_channel();
        self.net.spawn(async move {
            let mut chat = tokio::select! {
                chat = chat.lock() => chat,
                _ = &mut stopped => return,
            };
            chat.inbox = Some(inbox);
            let on = |reply| drop(tx.send(Input::Reply(id, reply)));
            let reply = speech::reply(
                &mut chat,
                &config,
                (&text, say),
                speaker.as_ref().map(|s| (s, control)),
                &on,
            );
            let (report, spoken) = tokio::select! {
                // The turn starts before a stop can drop it.
                biased;
                done = reply => done,
                played = &mut stopped => {
                    chat.cancelled(&played.unwrap_or_default());
                    return;
                }
            };
            let ms = |d: Option<Duration>| d.map_or("-".into(), |d| format!("{}ms", d.as_millis()));
            log::info(&format!(
                "reply={id} llm_first={} tts_first={} first_sound={} tools={}/{}ms \
                 rounds={} outcome={}",
                ms(report.llm_first),
                ms(spoken.tts_first),
                ms(spoken
                    .first_sound
                    .map(|t| t.saturating_duration_since(ended))),
                report.tools,
                report.tools_time.as_millis(),
                report.rounds,
                report.outcome
            ));
            on(Reply::Done(report.reply));
        });
        self.reply = Some((speech, stop, inject));
    }

    /// A prompt for a sub-agent said to it by name; none cancels.
    fn to_agent(&self, name: &str, text: Option<&str>) {
        let Some(agents) = &self.agents else { return };
        log::info(&match text {
            Some(text) => agents.prompt(name, text),
            None => agents.cancel(name),
        });
    }

    fn stop_code(&self) {
        let said = self.coder.as_ref().map(coder::Coder::cancel);
        log::info(&format!(
            "code cancel: {}",
            said.as_deref().unwrap_or("no coder")
        ));
    }

    /// Sends `pcm` to the ASR; the text comes back as an input.
    fn transcribe(&self, turn: u64, interim: bool, pcm: Vec<i16>) {
        let config = self.core.config();
        let request = asr::Request::new(config, &projects(config));
        let timeout = Duration::from_secs(config.asr_timeout_s.into());
        let (tx, busy) = (self.tx.clone(), self.interim_busy.clone());
        self.net.spawn(async move {
            let result = asr::transcribe(&request, &pcm, timeout).await;
            if interim {
                busy.store(false, Ordering::SeqCst);
            }
            let _ = tx.send(Input::Transcript {
                turn,
                interim,
                result,
            });
        });
    }
}

/// Starts a recording in `dir`; none for an empty `dir` or on an error.
fn start_recording(dir: &str) -> Option<Recorder> {
    if dir.is_empty() {
        return None;
    }
    let root = expand(dir);
    match Recorder::start(&root) {
        Ok(r) => {
            log::info(&format!("recording to {}", r.dir().display()));
            Some(r)
        }
        Err(e) => {
            log::error(&format!("record: {}: {e}", root.display()));
            None
        }
    }
}

/// The projects of the agents for the prompt of the ASR: those of
/// `agent_projects` and of the inventory.
fn projects(config: &Config) -> Vec<String> {
    let inventory = std::fs::read_dir(coder::data_dir().join("inventory"));
    let inventory = (inventory.into_iter().flatten().flatten())
        .map(|e| e.file_name().to_string_lossy().into_owned());
    config
        .agent_projects
        .keys()
        .cloned()
        .chain(inventory)
        .collect()
}

/// A leading `~/` is the home directory.
fn expand(path: &str) -> PathBuf {
    match (path.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest),
        _ => PathBuf::from(path),
    }
}

/// `chora-voice gate FILE`: the utterances of a file, with their times.
#[allow(clippy::print_stdout, reason = "the result of the subcommand")]
fn gate(config: &Config, file: &Path) -> Result<(), String> {
    for line in utterances(config, file)? {
        println!("{line}");
    }
    Ok(())
}

/// The gate events of a WAV file, one line each, with the time in s.
fn utterances(config: &Config, file: &Path) -> Result<Vec<String>, String> {
    let (rate, samples) = pcm::read_wav(file).map_err(|e| e.to_string())?;
    if rate != pcm::RATE {
        return Err(format!(
            "{}: {rate} Hz; the gate takes 24000",
            file.display()
        ));
    }
    let vad = Vad::new().map_err(|e| format!("the VAD model: {e}"))?;
    let (mut analyzer, mut gate) = (Analyzer::new(vad), Gate::new(config));
    let mut lines = Vec::new();
    for (i, chunk) in samples.as_chunks::<FRAME>().0.iter().enumerate() {
        let frame = analyzer.frame(chunk.to_vec())?;
        let at = f64::from(u32::try_from(i + 1).unwrap_or(u32::MAX) * FRAME_MS) / 1000.0;
        let line = match gate.push(&frame, true, false) {
            None => continue,
            Some(gate::Event::Onset) => format!("{at:.3} onset"),
            Some(gate::Event::Noise { speech_ms }) => format!("{at:.3} noise speech={speech_ms}ms"),
            Some(gate::Event::End {
                pcm,
                speech_ms,
                end_wait_ms,
            }) => {
                let len = u32::try_from(pcm.len()).unwrap_or(u32::MAX);
                let start = at - f64::from(len) / f64::from(pcm::RATE);
                format!("{at:.3} end from={start:.3} speech={speech_ms}ms end_wait={end_wait_ms}ms")
            }
        };
        lines.push(line);
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;
    use crate::{
        core::State::{self, Listening, Recording, Speaking, Thinking, Transcribing},
        fake,
        gate::Frame,
    };

    /// The frames of a corpus clip from `from` s on, with their speech
    /// probability.
    fn clip(name: &str, from: f64) -> Vec<Frame> {
        let file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/corpus")
            .join(name)
            .join("mic.wav");
        let (_, samples) = pcm::read_wav(&file).unwrap();
        let mut analyzer = Analyzer::new(Vad::new().unwrap());
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let skip = (from * f64::from(pcm::RATE)) as usize / FRAME;
        samples.as_chunks::<FRAME>().0[skip..]
            .iter()
            .map(|c| analyzer.frame(c.to_vec()).unwrap())
            .collect()
    }

    /// The agent with its inputs, and the states it went through.
    struct Run {
        agent: Agent,
        rx: mpsc::Receiver<Input>,
        states: Vec<State>,
    }

    impl Run {
        fn step(&mut self, input: Input) {
            self.agent.step(input).unwrap();
            let state = self.agent.core.state();
            if self.states.last() != Some(&state) {
                self.states.push(state);
            }
        }

        /// Feeds frames at full speed, with the inputs that come.
        fn feed(&mut self, frames: &[Frame]) {
            for frame in frames {
                self.step(Input::Frame(frame.clone()));
                while let Ok(input) = self.rx.try_recv() {
                    self.step(input);
                }
            }
        }

        /// Feeds quiet frames in real time until `done`.
        fn wait(&mut self, what: &str, done: impl Fn(&Core) -> bool) {
            let deadline = Instant::now() + Duration::from_secs(20);
            while !done(&self.agent.core) {
                assert!(Instant::now() < deadline, "{what}: {:?}", self.states);
                let next = Instant::now() + Duration::from_millis(FRAME_MS.into());
                while let Ok(input) = self.rx.recv_timeout(next - Instant::now().min(next)) {
                    self.step(input);
                }
                let quiet = Frame {
                    samples: vec![0; FRAME],
                    prob: 0.0,
                };
                self.step(Input::Frame(quiet));
            }
        }
    }

    /// The agent with the fake servers: the ASR hears a question, a
    /// cancel word and the next question; the LLM answers two of them.
    fn agent() -> (Run, fake::Server) {
        let net = tokio::runtime::Runtime::new().unwrap();
        let texts = &["Wie spät ist es?", "Stopp.", "Und morgen?"];
        let asr = net.block_on(fake::asr(fake::Asr::Seq(texts)));
        let tts = net.block_on(fake::tts(fake::Tts::Speak));
        let llm = net.block_on(fake::llm(vec![
            fake::Reply::Text("Es ist zehn Uhr. Schönen Tag noch."),
            fake::Reply::Text("Morgen ist frei."),
        ]));
        let config = Config {
            asr_url: asr.url,
            tts_url: tts.url,
            llm_url: llm.url.clone(),
            playback_cmd: ["sh", "-c", "cat > /dev/null"].map(Into::into).to_vec(),
            system_prompt: "Du bist ein Test.".into(),
            // The fake LLM has a context of 4000 tokens.
            context_docs: vec![
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/data/corpus/hallo-chora/labels.txt")
                    .display()
                    .to_string(),
            ],
            prompt_file: String::new(),
            // The short "Stopp." of the clip.
            barge_in_ms: 128,
            ..Config::default()
        };
        let (tools, _) = tools::tests::fake(0);
        let chat = net
            .block_on(Chat::start(config.clone(), "voice-mcp".into(), tools, None))
            .unwrap();
        let (tx, rx) = mpsc::channel();
        let run = Run {
            agent: Agent::new(config, chat, net, tx),
            rx,
            states: vec![State::Listening],
        };
        (run, llm)
    }

    /// docs/voice.md section 13: recorded speech through the whole agent
    /// with the fake servers and tools. A question, a barge-in with a
    /// cancel word while the agent speaks, and the next question, whose
    /// request holds the note of the stopped reply.
    #[test]
    fn a_replay_runs_through_the_whole_agent() {
        let (mut run, llm) = agent();
        let (question, stop) = (clip("hallo-chora", 0.0), clip("turns-noise", 3.6));
        run.feed(&question);
        run.wait("the reply speaks", |core| core.state() == State::Speaking);
        run.feed(&stop);
        run.wait("the cancel word stops it", |core| {
            core.state() == State::Listening
        });
        run.feed(&question);
        run.wait("the next reply", |core| {
            core.record()["last_reply"] == "Morgen ist frei." && core.state() == State::Listening
        });
        assert_eq!(
            run.states,
            [
                Listening,
                Recording,
                Transcribing,
                Thinking,
                Speaking,
                Recording,
                Transcribing,
                Listening,
                Recording,
                Transcribing,
                Thinking,
                Speaking,
                Listening
            ]
        );
        let seen = llm.seen.lock().unwrap();
        assert_eq!(seen.len(), 2, "two requests");
        let messages: Vec<(&str, &str)> = seen[1]["messages"].as_array().unwrap()[1..]
            .iter()
            .map(|m| (m["role"].as_str().unwrap(), m["content"].as_str().unwrap()))
            .collect();
        assert_eq!(
            messages,
            [
                ("user", "Wie spät ist es?"),
                ("assistant", "Es ist zehn Uhr."),
                ("user", chat::STOPPED),
                ("user", "Und morgen?"),
            ]
        );
    }

    #[test]
    fn the_gate_finds_the_one_utterance_of_the_tts_clip() {
        let clip =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/corpus/hallo-chora/mic.wav");
        let lines = utterances(&Config::default(), &clip).unwrap();
        assert_eq!(lines.len(), 2, "{lines:?}");
        // Speech from 1.00 s to 4.04 s; onset after 96 ms of speech.
        let time = |line: &str, key: &str| -> f64 {
            let field = line.split(' ').find_map(|f| f.strip_prefix(key)).unwrap();
            field.parse().unwrap()
        };
        let onset: f64 = lines[0].strip_suffix(" onset").unwrap().parse().unwrap();
        assert!((1.0..1.3).contains(&onset), "{lines:?}");
        let end: f64 = lines[1].split(' ').next().unwrap().parse().unwrap();
        assert!(
            (4.6..5.2).contains(&end),
            "the end after 800 ms of quiet: {lines:?}"
        );
        let from = time(&lines[1], "from=");
        assert!((0.6..1.0).contains(&from), "300 ms of pre-roll: {lines:?}");
    }
}
