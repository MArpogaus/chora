//! The TTS client (docs/voice.md section 8): one websocket per text; the
//! server splits it into sentences and sends each as `audio.start` with
//! its text, binary PCM16 at 24 kHz, and `audio.done`.

use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

use crate::{config::Config, pcm};

/// What the TTS sends, in order.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// A sentence starts; its sound follows. The HUD typewriter spreads
    /// its words over the PCM up to the next sentence.
    Sentence {
        index: u64,
        text: String,
    },
    Audio(Vec<i16>),
}

/// The timing of one sentence, for the log and the HUD.
#[derive(Debug, PartialEq)]
pub struct Timing {
    pub index: u64,
    pub text: String,
    /// From the request to the first PCM of the sentence.
    pub first_audio: Duration,
    /// Where its sound starts, in samples of the whole text.
    pub start: usize,
    pub samples: usize,
}

/// Speaks `text`: sends each sentence and its PCM to `tx` as they come.
/// Each message must come within `tts_timeout_s`.
///
/// # Errors
///
/// The server is away or slow, answers an error, or sends another rate.
pub async fn speak(config: &Config, text: &str, tx: UnboundedSender<Event>) -> Result<(), String> {
    let wait = Duration::from_secs(config.tts_timeout_s.into());
    let timeout = || format!("no answer within {wait:?}");
    let (mut ws, _) = tokio::time::timeout(wait, tokio_tungstenite::connect_async(&config.tts_url))
        .await
        .map_err(|_| timeout())?
        .map_err(|e| format!("{}: {e}", config.tts_url))?;
    let out = [
        json!({
            "type": "session.config",
            "model": config.model,
            "voice": config.voice,
            "response_format": "pcm",
            "stream_audio": true,
            "language": config.tts_language(),
        }),
        json!({"type": "input.text", "text": text}),
        json!({"type": "input.done"}),
    ];
    for message in out {
        ws.send(Message::text(message.to_string()))
            .await
            .map_err(|e| e.to_string())?;
    }
    loop {
        let message = tokio::time::timeout(wait, ws.next())
            .await
            .map_err(|_| timeout())?
            .ok_or("the server closed the socket")?
            .map_err(|e| e.to_string())?;
        let event = match message {
            Message::Binary(bytes) => Event::Audio(pcm::samples(&bytes)),
            Message::Text(text) => {
                let event: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                match event["type"].as_str().unwrap_or_default() {
                    "audio.start" => sentence(&event)?,
                    "session.done" => {
                        let _ = ws.close(None).await;
                        return Ok(());
                    }
                    kind if kind.contains("error") => {
                        return Err(format!("the server: {event}"));
                    }
                    _ => continue,
                }
            }
            Message::Close(_) => return Err("the server closed the socket".into()),
            _ => continue,
        };
        // The receiver may stop listening, for example on a barge-in.
        if tx.send(event).is_err() {
            return Ok(());
        }
    }
}

fn sentence(event: &Value) -> Result<Event, String> {
    let rate = event["sample_rate"].as_u64();
    if rate.is_some_and(|r| r != u64::from(pcm::RATE)) {
        return Err(format!("the server sends {rate:?} Hz, not 24000: {event}"));
    }
    Ok(Event::Sentence {
        index: event["sentence_index"].as_u64().unwrap_or_default(),
        text: event["sentence_text"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    })
}

/// Speaks `text` to the end: its PCM and the timing of each sentence.
///
/// # Errors
///
/// As `speak`.
pub async fn collect(config: &Config, text: &str) -> Result<(Vec<i16>, Vec<Timing>), String> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let start = Instant::now();
    let (result, (pcm, timings)) = tokio::join!(speak(config, text, tx), async {
        let (mut pcm, mut timings) = (Vec::new(), Vec::<Timing>::new());
        while let Some(event) = rx.recv().await {
            match event {
                Event::Sentence { index, text } => timings.push(Timing {
                    index,
                    text,
                    first_audio: Duration::ZERO,
                    start: pcm.len(),
                    samples: 0,
                }),
                Event::Audio(samples) => {
                    if let Some(t) = timings.last_mut() {
                        if t.samples == 0 {
                            t.first_audio = start.elapsed();
                        }
                        t.samples += samples.len();
                    }
                    pcm.extend(samples);
                }
            }
        }
        (pcm, timings)
    });
    result.map(|()| (pcm, timings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake;

    fn config(url: &str) -> Config {
        Config {
            tts_url: url.into(),
            tts_timeout_s: 1,
            ..Config::default()
        }
    }

    #[tokio::test]
    async fn each_sentence_comes_with_its_text_and_sound() {
        let server = fake::tts(fake::Tts::Speak).await;
        let (pcm, timings) = collect(&config(&server.url), "Hallo. Wie geht es?")
            .await
            .unwrap();
        // 60 ms per character: 6 and 12 characters.
        assert_eq!(pcm.len(), 18 * 1440);
        let texts: Vec<_> = timings.iter().map(|t| (t.index, t.text.as_str())).collect();
        assert_eq!(texts, [(0, "Hallo."), (1, "Wie geht es?")]);
        assert_eq!(
            (timings[1].start, timings[1].samples),
            (6 * 1440, 12 * 1440)
        );
        let seen = server.seen.lock().unwrap().clone();
        assert_eq!(
            (
                &seen[0]["type"],
                &seen[0]["voice"],
                &seen[0]["response_format"]
            ),
            (&json!("session.config"), &json!("aiden"), &json!("pcm"))
        );
    }

    #[tokio::test]
    async fn errors_a_wrong_rate_and_silence_end_the_speech() {
        let server = fake::tts(fake::Tts::Rate(16_000)).await;
        let e = collect(&config(&server.url), "Hallo.").await.unwrap_err();
        assert!(e.contains("16000"), "{e}");
        let server = fake::tts(fake::Tts::Silent).await;
        let start = Instant::now();
        let e = collect(&config(&server.url), "Hallo.").await.unwrap_err();
        assert!(e.contains("no answer"), "{e}");
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
