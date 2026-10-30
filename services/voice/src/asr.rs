//! The ASR client (docs/voice.md section 8): one websocket per
//! transcription to the realtime endpoint, PCM16 at 24 kHz in pieces of
//! one second, then a commit; the server answers with the transcript.

use std::time::{Duration, Instant};

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

use crate::pcm::{self, RATE};

/// A transcript and the wait from the commit to it.
#[derive(Debug, PartialEq)]
pub struct Transcript {
    pub text: String,
    pub after_commit: Duration,
}

/// Transcribes `pcm`, with one retry on a new connection; each try has
/// `timeout` for all of it.
///
/// # Errors
///
/// Both tries fail: the server is away, slow, or answers an error.
pub async fn transcribe(
    url: &str,
    language: &str,
    pcm: &[i16],
    timeout: Duration,
) -> Result<Transcript, String> {
    match once(url, language, pcm, timeout).await {
        Ok(t) => Ok(t),
        Err(first) => {
            crate::log::warn(&format!("asr: {first}; one retry"));
            once(url, language, pcm, timeout).await
        }
    }
}

async fn once(
    url: &str,
    language: &str,
    pcm: &[i16],
    timeout: Duration,
) -> Result<Transcript, String> {
    tokio::time::timeout(timeout, session(url, language, pcm))
        .await
        .map_err(|_| format!("no transcript within {timeout:?}"))?
}

async fn session(url: &str, language: &str, pcm: &[i16]) -> Result<Transcript, String> {
    let (mut ws, _) = tokio_tungstenite::connect_async(url)
        .await
        .map_err(|e| format!("{url}: {e}"))?;
    let audio = json!({
        "format": {"type": "audio/pcm", "rate": RATE},
        "transcription": {"model": "gufo", "language": language},
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

    #[tokio::test]
    async fn the_transcript_comes_after_the_commit_in_pieces_of_one_second() {
        let server = fake::asr(fake::Asr::Text("hallo chora")).await;
        let pcm = vec![1; RATE as usize * 5 / 2];
        let t = transcribe(&server.url, "de", &pcm, Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(t.text, "hallo chora");
        let seen = server.seen.lock().unwrap().clone();
        assert_eq!(seen[0]["type"], "session.update");
        assert_eq!(seen[0]["session"]["audio"]["input"]["format"]["rate"], RATE);
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
        let t = transcribe(&server.url, "de", &[0; 100], Duration::from_secs(5)).await;
        assert_eq!(t.unwrap().text, "ja");
        let server = fake::asr(fake::Asr::Error).await;
        let e = transcribe(&server.url, "de", &[0; 100], Duration::from_secs(5)).await;
        assert!(e.unwrap_err().contains("invalid_audio"));
    }

    #[tokio::test]
    async fn a_silent_server_times_out() {
        let server = fake::asr(fake::Asr::Silent).await;
        let start = Instant::now();
        let e = transcribe(&server.url, "de", &[0; 100], Duration::from_millis(200)).await;
        assert!(e.unwrap_err().contains("no transcript"));
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "two tries of 200 ms"
        );
        let e = transcribe("ws://127.0.0.1:9", "de", &[0; 100], Duration::from_secs(1)).await;
        assert!(e.is_err(), "nobody listens");
    }
}
