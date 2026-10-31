//! The ASR client (docs/voice.md section 8): one websocket per
//! transcription to the realtime endpoint, PCM16 at 24 kHz in pieces of
//! one second, then a commit; the server answers with the transcript.

use std::time::{Duration, Instant};

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

use crate::{
    config::{Backend, Config},
    pcm::{self, RATE},
};

/// A transcript and the wait from the commit to it.
#[derive(Debug, PartialEq)]
pub struct Transcript {
    pub text: String,
    pub after_commit: Duration,
}

/// What the ASR gets besides the sound.
#[derive(Clone, Debug)]
pub struct Request {
    pub backend: Result<Backend, String>,
    /// `auto` or empty: the server finds it.
    pub language: String,
    /// The words to know; empty: none.
    pub prompt: String,
}

impl Request {
    /// The request of `config`, with the agent `names` in the prompt.
    pub fn new<'a>(config: &'a Config, names: impl IntoIterator<Item = &'a String>) -> Self {
        Self {
            backend: config.asr(),
            language: config.asr_language.clone(),
            prompt: config.asr_prompt(names),
        }
    }

    /// The `transcription` of the session: without a language for
    /// `auto`, without a prompt when it is empty.
    fn transcription(&self, model: &str) -> Value {
        let mut out = json!({"model": model});
        if !matches!(self.language.as_str(), "" | "auto") {
            out["language"] = self.language.as_str().into();
        }
        if !self.prompt.is_empty() {
            out["prompt"] = self.prompt.as_str().into();
        }
        out
    }
}

/// Transcribes `pcm`, with one retry on a new connection; each try has
/// `timeout` for all of it.
///
/// # Errors
///
/// Both tries fail: the server is away, slow, or answers an error.
pub async fn transcribe(
    request: &Request,
    pcm: &[i16],
    timeout: Duration,
) -> Result<Transcript, String> {
    // Behind the echo canceller speech can be as quiet as -80 dBFS.
    let pcm = &pcm::normalized(pcm);
    let result = match once(request, pcm, timeout).await {
        Ok(t) => Ok(t),
        Err(first) => {
            crate::log::warn(&format!("asr: {first}; one retry"));
            once(request, pcm, timeout).await
        }
    };
    match &result {
        Ok(_) => crate::health::ok("asr"),
        Err(e) => drop(crate::health::down("asr", e)),
    }
    result.map(|mut t| {
        if echoes(&request.prompt, &t.text) {
            crate::log::info(&format!("asr: the prompt came back, dropped: {:?}", t.text));
            t.text.clear();
        }
        t
    })
}

/// Whether `text` is the prompt that came back, as the ASR does on short
/// or quiet sound (log 2026-10-31): it holds at least half of the words
/// of the prompt, and at least 4. A user who says a wake word and a
/// stop word holds far fewer.
fn echoes(prompt: &str, text: &str) -> bool {
    let mut words = crate::words::normalize(prompt);
    words.sort_unstable();
    words.dedup();
    let text = crate::words::normalize(text);
    let found = words.iter().filter(|w| text.contains(w)).count();
    found >= 4 && found * 2 >= words.len()
}

async fn once(request: &Request, pcm: &[i16], timeout: Duration) -> Result<Transcript, String> {
    tokio::time::timeout(timeout, session(request, pcm))
        .await
        .map_err(|_| format!("no transcript within {timeout:?}"))?
}

async fn session(request: &Request, pcm: &[i16]) -> Result<Transcript, String> {
    let backend = request.backend.as_ref()?;
    let mut ws = crate::health::connect(backend).await?;
    let audio = json!({
        "format": {"type": "audio/pcm", "rate": RATE},
        "transcription": request.transcription(&backend.model),
        "turn_detection": null,
    });
    let mut out = vec![json!({
        "type": "session.update",
        "session": {"type": "transcription", "audio": {"input": audio}},
    })];
    // The server closes a socket with 128 messages in its queue: pieces
    // of one second keep a turn of 30 s at 30.
    for piece in pcm.chunks(RATE as usize) {
        let audio = base64::engine::general_purpose::STANDARD.encode(pcm::bytes(piece));
        out.push(json!({"type": "input_audio_buffer.append", "audio": audio}));
    }
    out.push(json!({"type": "input_audio_buffer.commit"}));
    for message in out {
        ws.send(Message::text(message.to_string()))
            .await
            .map_err(|e| e.to_string())?;
    }
    let committed = Instant::now();
    let mut deltas = String::new();
    while let Some(message) = ws.next().await {
        let Message::Text(text) = message.map_err(|e| e.to_string())? else {
            continue;
        };
        let event: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let kind = event["type"].as_str().unwrap_or_default();
        if kind.ends_with("transcription.delta") {
            deltas.push_str(event["delta"].as_str().unwrap_or_default());
        } else if kind.ends_with("transcription.completed") {
            let text = event["transcript"].as_str().map_or(deltas, str::to_owned);
            let _ = ws.close(None).await;
            return Ok(Transcript {
                text: text.trim().to_owned(),
                after_commit: committed.elapsed(),
            });
        } else if kind.contains("error") || kind.ends_with("failed") {
            return Err(format!("the server: {event}"));
        }
    }
    Err("the server closed the socket".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake;

    fn req(url: &str) -> Request {
        let config = Config {
            asr_url: url.into(),
            ..Config::default()
        };
        Request::new(&config, [])
    }

    #[tokio::test]
    async fn the_transcript_comes_after_the_commit_in_pieces_of_one_second() {
        let server = fake::asr(fake::Asr::Text("hallo chora")).await;
        let pcm = vec![1; RATE as usize * 5 / 2];
        let t = transcribe(&req(&server.url), &pcm, Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(t.text, "hallo chora");
        let seen = server.seen.lock().unwrap().clone();
        assert_eq!(seen[0]["type"], "session.update");
        let input = &seen[0]["session"]["audio"]["input"];
        assert_eq!(input["format"]["rate"], RATE);
        let transcription = &input["transcription"];
        assert_eq!(transcription["model"], "gufo");
        assert!(transcription.get("language").is_none(), "auto: no language");
        assert!(
            transcription["prompt"]
                .as_str()
                .unwrap()
                .starts_with("Chora, Coder, hey chora")
        );
        let pieces: Vec<usize> = seen
            .iter()
            .filter(|e| e["type"] == "input_audio_buffer.append")
            .map(|e| {
                let audio = e["audio"].as_str().unwrap();
                base64::engine::general_purpose::STANDARD
                    .decode(audio)
                    .unwrap()
                    .len()
            })
            .collect();
        assert_eq!(pieces, [48_000, 48_000, 24_000]);
        assert_eq!(seen.last().unwrap()["type"], "input_audio_buffer.commit");
    }

    #[tokio::test]
    async fn a_failed_try_gets_one_retry_on_a_new_socket() {
        let server = fake::asr(fake::Asr::FailFirst("ja")).await;
        let t = transcribe(&req(&server.url), &[0; 100], Duration::from_secs(5)).await;
        assert_eq!(t.unwrap().text, "ja");
        let server = fake::asr(fake::Asr::Error).await;
        let e = transcribe(&req(&server.url), &[0; 100], Duration::from_secs(5)).await;
        assert!(e.unwrap_err().contains("invalid_audio"));
    }

    /// Log 2026-10-31, turn 1: the ASR answered its own prompt.
    #[tokio::test]
    async fn a_transcript_that_is_the_prompt_is_dropped() {
        let request = req("ws://unused");
        let echo = "Hey Chora, Coder, hey chora, listen, hallo chora, hör zu, sleep, go to \
                    sleep, schlaf, schlaf jetzt, stop, cancel, never mind, stopp, abbrechen, \
                    halt, vergiss";
        assert!(echoes(&request.prompt, echo));
        for said in [
            "Hey Chora, schlaf jetzt.",
            "Stopp, abbrechen, vergiss es.",
            "Chora",
        ] {
            assert!(!echoes(&request.prompt, said), "{said}");
        }
        assert!(!echoes("", "a b c d"), "no prompt, no echo");
        let server = fake::asr(fake::Asr::Text(echo)).await;
        let t = transcribe(&req(&server.url), &[0; 100], Duration::from_secs(5)).await;
        assert_eq!(t.unwrap().text, "");
    }

    #[tokio::test]
    async fn a_silent_server_times_out() {
        let server = fake::asr(fake::Asr::Silent).await;
        let start = Instant::now();
        let e = transcribe(&req(&server.url), &[0; 100], Duration::from_millis(200)).await;
        assert!(e.unwrap_err().contains("no transcript"));
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "two tries of 200 ms"
        );
        let e = transcribe(&req("ws://127.0.0.1:9"), &[0; 100], Duration::from_secs(1)).await;
        assert!(e.is_err(), "nobody listens");
    }
}
