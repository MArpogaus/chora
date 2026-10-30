//! chora-voice: the voice agent of chora (docs/voice.md). The mic goes
//! through a speech gate with Silero VAD; the agent is a chora client
//! with methods, settings and a push-to-talk key.

mod asr;
mod audio;
mod chat;
mod client;
mod config;
mod core;
mod earcon;
#[cfg(test)]
mod fake;
mod gate;
mod live;
mod llm;
mod log;
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

use crate::{
    audio::{Analyzer, FRAME, Mic, Speaker},
    chat::Chat,
    config::Config,
    core::{Core, Effect, Input, Reply},
    gate::{FRAME_MS, Gate},
    record::Recorder,
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
}

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
        .and_then(|settings| match &args.command {
            Some(Command::Gate { file }) => gate(&settings.value, file),
            Some(Command::Replay { dirs }) => {
                let dirs: Vec<&Path> = dirs.iter().map(AsRef::as_ref).collect();
                let (lines, _) = replay::run(&settings.value, &dirs)?;
                println!("{}", lines.join("\n"));
                Ok(())
            }
            Some(Command::Asr { file }) => live::asr(&settings.value, file),
            Some(Command::Tts { text, out }) => live::tts(&settings.value, text, out.as_deref()),
            Some(Command::Roundtrip { text }) => live::roundtrip(&settings.value, text),
            Some(Command::Ask { text, speak, yes }) => {
                live::ask(&settings.value, &mcp_name(&args.name), text, *speak, *yes)
            }
            None => run(args.name, settings),
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

/// Runs the agent until chora stays away or a thread ends.
fn run(name: String, settings: settings::File<Config>) -> Result<(), String> {
    let config = settings.value.clone();
    log::info(&format!("settings: {}", settings.file().display()));
    let vad = Vad::new().map_err(|e| format!("the VAD model: {e}"))?;
    let net = tokio::runtime::Runtime::new().map_err(|e| format!("tokio: {e}"))?;
    let mcp = mcp_name(&name);
    let tools = Tools::chora(&config, &mcp);
    let chat = net.block_on(Chat::start(
        config.clone(),
        mcp,
        tools,
        chat::history_file(),
    ))?;
    let chat = Arc::new(tokio::sync::Mutex::new(chat));
    let warm = chat.clone();
    net.spawn(async move { warm.lock().await.warm().await });
    let (tx, rx) = mpsc::channel();
    let mut agent = Agent {
        mic: Mic::start(config.capture_cmd.clone(), Analyzer::new(vad), tx.clone()),
        speaker: Speaker::new(config.playback_cmd.clone()),
        recorder: start_recording(&config.record_dir),
        core: Core::new(config),
        net,
        tx: tx.clone(),
        interim_busy: Arc::default(),
        chat,
        reply: None,
    };
    client::start(name, settings, tx);
    log::info(&format!("started, {:?}", agent.core.state()));
    while let Ok(input) = rx.recv() {
        agent.input(&input);
        for effect in agent.core.handle(input) {
            agent.effect(effect)?;
        }
    }
    Err("an audio thread ended".into())
}

/// The core and the actors that run its effects.
struct Agent {
    core: Core,
    mic: Mic,
    speaker: Speaker,
    recorder: Option<Recorder>,
    /// The network actors: ASR, later TTS and LLM.
    net: tokio::runtime::Runtime,
    tx: mpsc::Sender<Input>,
    /// An interim transcription runs; the next interval waits for it.
    interim_busy: Arc<AtomicBool>,
    chat: Arc<tokio::sync::Mutex<Chat>>,
    /// The reply in progress.
    reply: Option<tokio::task::JoinHandle<()>>,
}

impl Agent {
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
            Input::Settings(new) => {
                let old = self.core.config();
                if new.capture_cmd != old.capture_cmd {
                    self.mic.restart(new.capture_cmd.clone());
                }
                if new.record_dir != old.record_dir {
                    self.recorder = start_recording(&new.record_dir);
                }
                self.speaker.set_cmd(new.playback_cmd.clone());
                let (chat, config) = (self.chat.clone(), (**new).clone());
                self.net.spawn(async move {
                    let mut chat = chat.lock().await;
                    match chat.configure(config) {
                        Ok(true) => chat.warm().await,
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
                    let _ = r.played(&pcm);
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
            Effect::Ask {
                turn,
                text,
                speak,
                ended,
            } => self.ask(turn, text, speak, ended),
            Effect::Stop => {
                if let Some(reply) = self.reply.take() {
                    reply.abort();
                }
                self.speaker.stop();
            }
            Effect::Exit => return Err("chora stays away".into()),
        }
        Ok(())
    }
}

impl Agent {
    /// Starts the reply to `text`; its events go to the core.
    fn ask(&mut self, turn: u64, text: String, speak: bool, ended: std::time::Instant) {
        let (chat, tx) = (self.chat.clone(), self.tx.clone());
        let config = self.core.config().clone();
        let speaker = speak.then(|| self.speaker.clone());
        self.reply = Some(self.net.spawn(async move {
            let mut chat = chat.lock().await;
            let on = |reply| drop(tx.send(Input::Reply(turn, reply)));
            let (report, spoken) =
                speech::reply(&mut chat, &config, &text, speaker.as_ref(), &on).await;
            let ms = |d: Option<Duration>| d.map_or("-".into(), |d| format!("{}ms", d.as_millis()));
            log::info(&format!(
                "turn={turn} llm_first={} tts_first={} first_sound={} tools={}/{}ms \
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
        }));
    }

    /// Sends `pcm` to the ASR; the text comes back as an input.
    fn transcribe(&self, turn: u64, interim: bool, pcm: Vec<i16>) {
        let config = self.core.config();
        let (url, language) = (config.asr_url.clone(), config.language.clone());
        let timeout = Duration::from_secs(config.asr_timeout_s.into());
        let (tx, busy) = (self.tx.clone(), self.interim_busy.clone());
        self.net.spawn(async move {
            let result = asr::transcribe(&url, &language, &pcm, timeout).await;
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

/// A leading `~/` is the home directory.
fn expand(path: &str) -> PathBuf {
    match (path.strip_prefix("~/"), std::env::var_os("HOME")) {
        (Some(rest), Some(home)) => Path::new(&home).join(rest),
        _ => PathBuf::from(path),
    }
}

/// `chora-voice gate FILE`: the utterances of a file, with their times.
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
    use super::*;

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
