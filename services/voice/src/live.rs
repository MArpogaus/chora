//! The subcommands that check the servers of the settings by hand:
//! `asr FILE`, `tts TEXT` and `roundtrip [TEXT]`.

use std::{
    path::Path,
    time::{Duration, Instant},
};

use crate::{
    asr,
    audio::Speaker,
    config::Config,
    pcm::{self, RATE},
    tts,
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
    let t = runtime()?.block_on(asr::transcribe(
        &config.asr_url,
        &config.language,
        pcm,
        timeout,
    ))?;
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
