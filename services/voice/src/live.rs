//! The subcommands that check the servers of the settings by hand:
//! `asr FILE`, `tts TEXT`, `roundtrip [TEXT]`, `ask TEXT` and `levels`.

#![allow(clippy::print_stdout, reason = "the subcommands print their results")]

use std::{
    path::Path,
    time::{Duration, Instant},
};

use crate::{
    asr,
    audio::{Analyzer, Mic, Speaker},
    chat::{Chat, Event},
    config::Config,
    core::{Input, Reply},
    pcm::{self, RATE},
    speech,
    tools::Tools,
    tts,
    vad::Vad,
};

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Runtime::new().map_err(|e| format!("tokio: {e}"))
}

fn ms(samples: usize) -> usize {
    samples * 1000 / RATE as usize
}

/// The sound of `text`; prints the timing of each sentence and the time
/// to the end of the session.
fn speak(config: &Config, text: &str) -> Result<Vec<i16>, String> {
    let start = Instant::now();
    let (pcm, timings) = runtime()?.block_on(tts::collect(config, text))?;
    for t in timings {
        println!(
            "tts sentence={} first_audio={}ms start={}ms length={}ms text={:?}",
            t.index,
            t.first_audio.as_millis(),
            ms(t.start),
            ms(t.samples),
            t.text
        );
    }
    println!(
        "tts total={}ms audio={}ms",
        start.elapsed().as_millis(),
        ms(pcm.len())
    );
    Ok(pcm)
}

fn transcribe(config: &Config, pcm: &[i16]) -> Result<(), String> {
    let timeout = Duration::from_secs(config.asr_timeout_s.into());
    let request = asr::Request::new(config, &crate::projects(config));
    let t = runtime()?.block_on(asr::transcribe(&request, pcm, timeout))?;
    println!(
        "asr after_commit={}ms audio={}ms text={:?}",
        t.after_commit.as_millis(),
        ms(pcm.len()),
        t.text
    );
    Ok(())
}

/// `chora-voice asr FILE`: the transcript of a WAV file at 24 kHz.
pub fn asr(config: &Config, file: &Path) -> Result<(), String> {
    let (rate, samples) = pcm::read_wav(file).map_err(|e| e.to_string())?;
    if rate != RATE {
        return Err(format!(
            "{}: {rate} Hz; the ASR takes 24000",
            file.display()
        ));
    }
    transcribe(config, &samples)
}

/// `chora-voice tts TEXT`: speaks with the playback command, or writes
/// the sound to a WAV file.
pub fn tts(config: &Config, text: &str, out: Option<&Path>) -> Result<(), String> {
    let pcm = speak(config, text)?;
    if let Some(path) = out {
        let len = u32::try_from(pcm.len() * 2).map_err(|_| "too long")?;
        let mut file = pcm::wav_header(RATE, len).to_vec();
        file.extend(pcm::bytes(&pcm));
        return std::fs::write(path, file).map_err(|e| format!("{}: {e}", path.display()));
    }
    Speaker::new(config.playback_cmd.clone())
        .play(&pcm)
        .map_err(|e| format!("playback: {e}"))?;
    // The process must live until the sound ends.
    std::thread::sleep(Duration::from_millis(ms(pcm.len()) as u64 + 500));
    Ok(())
}

/// `chora-voice roundtrip [TEXT]`: speaks the text with the TTS and
/// gives its sound to the ASR, with the latencies of both.
pub fn roundtrip(config: &Config, text: &str) -> Result<(), String> {
    let pcm = speak(config, text)?;
    transcribe(config, &pcm)
}

/// `chora-voice ask TEXT`: one turn with the LLM and the tools of chora
/// under the client name `name`, without the history file.
pub fn ask(config: &Config, name: &str, text: &str, speak: bool, yes: bool) -> Result<(), String> {
    runtime()?.block_on(async {
        let start = Instant::now();
        let mut chat = Chat::start(config.clone(), name.into(), Tools::chora(config, name), None).await?;
        println!(
            "start={}ms tools={}",
            start.elapsed().as_millis(),
            chat.tools.specs.len()
        );
        chat.warm().await;
        let speaker = speak.then(|| Speaker::new(config.playback_cmd.clone()));
        // Nothing pauses the speech here.
        let (_pause, control) = tokio::sync::mpsc::unbounded_channel();
        let start = Instant::now();
        let at = || start.elapsed().as_millis();
        let on = |reply| match reply {
            Reply::Chat(Event::Tool(name)) => println!("{:>6}ms tool {name}", at()),
            Reply::Chat(Event::Confirm { prompt, answer }) => {
                println!("{:>6}ms confirm {prompt:?}: {}", at(), if yes { "yes" } else { "no" });
                let _ = answer.send(Some(yes));
            }
            Reply::Said { text, .. } => println!("{:>6}ms say {text:?}", at()),
            _ => {}
        };
        let (report, spoken) = speech::reply(&mut chat, config, (text, false), speaker.as_ref().map(|s| (s, control)), &on).await;
        let ms = |d: Option<Duration>| d.map_or("-".into(), |d| format!("{}ms", d.as_millis()));
        println!("reply={:?}", report.reply);
        println!(
            "llm_first={} tts_first={} first_sound={} tools={}/{}ms rounds={} outcome={} total={}ms",
            ms(report.llm_first),
            ms(spoken.tts_first),
            ms(spoken.first_sound.map(|t| t.saturating_duration_since(start))),
            report.tools,
            report.tools_time.as_millis(),
            report.rounds,
            report.outcome,
            at()
        );
        Ok(())
    })
}

/// `chora-voice levels`: the loudest mic level and speech probability of
/// the capture command, twice a second, until Ctrl+C. For the check of
/// the echo canceller and of the barge-in levels.
pub fn levels(config: &Config) -> Result<(), String> {
    let vad = Vad::new().map_err(|e| format!("the VAD model: {e}"))?;
    let (tx, rx) = std::sync::mpsc::channel();
    let _mic = Mic::start(config.capture_cmd.clone(), Analyzer::new(vad), tx);
    let (mut rms, mut prob, mut n) = (0.0, 0.0f32, 0);
    for input in rx {
        match input {
            Input::Frame(frame) => {
                rms = f64::max(rms, pcm::rms(&frame.samples));
                prob = prob.max(frame.prob);
                n += 1;
                if n == 16 {
                    println!("rms={rms:.0} speech={prob:.2}");
                    (rms, prob, n) = (0.0, 0.0, 0);
                }
            }
            Input::MicDown => println!("the capture command ended"),
            _ => {}
        }
    }
    Ok(())
}
