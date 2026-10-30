//! Fake ASR and TTS servers for the tests: websockets on 127.0.0.1 that
//! speak the protocols of docs/voice.md section 8 from a script.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// A running fake: its URL and the text messages it got.
pub struct Server {
    pub url: String,
    pub seen: Arc<Mutex<Vec<Value>>>,
}

#[derive(Clone, Copy)]
pub enum Asr {
    /// Answers each commit with this text, in two deltas.
    Text(&'static str),
    /// Closes the first socket at the commit, then answers.
    FailFirst(&'static str),
    /// Answers each commit with the error of a wrong rate.
    Error,
    /// Never answers.
    Silent,
}

#[derive(Clone, Copy)]
pub enum Tts {
    /// One sentence per `.` or `?`, 60 ms of silence per character.
    Speak,
    /// As `Speak`, with another rate in `audio.start`.
    Rate(u32),
    /// Never answers.
    Silent,
}

type Socket = WebSocketStream<tokio::net::TcpStream>;

/// Starts a server that runs `serve` for each socket.
async fn start<F, Fut>(serve: F) -> Server
where
    F: Fn(Socket, Arc<Mutex<Vec<Value>>>) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let seen = Arc::<Mutex<Vec<Value>>>::default();
    let log = seen.clone();
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            if let Ok(ws) = tokio_tungstenite::accept_async(tcp).await {
                tokio::spawn(serve(ws, log.clone()));
            }
        }
    });
    Server { url, seen }
}

/// The next text message as JSON, noted in `seen`.
async fn next(ws: &mut Socket, seen: &Mutex<Vec<Value>>) -> Option<Value> {
    while let Some(Ok(message)) = ws.next().await {
        if let Message::Text(text) = message {
            let event: Value = serde_json::from_str(&text).unwrap();
            seen.lock().unwrap().push(event.clone());
            return Some(event);
        }
    }
    None
}

async fn send(ws: &mut Socket, event: &Value) {
    let _ = ws.send(Message::text(event.to_string())).await;
}

pub async fn asr(script: Asr) -> Server {
    let failed = Arc::new(AtomicBool::new(false));
    start(move |mut ws, seen| {
        let failed = failed.clone();
        async move {
            while let Some(event) = next(&mut ws, &seen).await {
                if event["type"] != "input_audio_buffer.commit" {
                    continue;
                }
                let text = match script {
                    Asr::Silent => continue,
                    Asr::FailFirst(_) if !failed.swap(true, Ordering::SeqCst) => return,
                    Asr::Text(text) | Asr::FailFirst(text) => text,
                    Asr::Error => {
                        let error =
                            json!({"type": "error", "error": {"code": "invalid_audio_event"}});
                        send(&mut ws, &error).await;
                        continue;
                    }
                };
                let delta = "conversation.item.input_audio_transcription.delta";
                let (a, b) = text.split_at(text.len() / 2);
                for part in [a, b] {
                    send(&mut ws, &json!({"type": delta, "delta": part})).await;
                }
                let done = "conversation.item.input_audio_transcription.completed";
                send(&mut ws, &json!({"type": done, "transcript": text})).await;
            }
        }
    })
    .await
}

pub async fn tts(script: Tts) -> Server {
    start(move |mut ws, seen| async move {
        let mut text = String::new();
        while let Some(event) = next(&mut ws, &seen).await {
            match event["type"].as_str() {
                Some("input.text") => text.push_str(event["text"].as_str().unwrap()),
                Some("input.done") => break,
                _ => {}
            }
        }
        let rate = match script {
            Tts::Silent => return std::future::pending().await,
            Tts::Speak => 24_000,
            Tts::Rate(rate) => rate,
        };
        let sentences = text
            .split_inclusive(['.', '?'])
            .map(str::trim)
            .filter(|s| !s.is_empty());
        for (index, sentence) in sentences.enumerate() {
            let start = json!({"type": "audio.start", "sentence_index": index,
                "sentence_text": sentence, "format": "pcm", "sample_rate": rate});
            send(&mut ws, &start).await;
            let pcm = vec![0u8; sentence.chars().count() * 1440 * 2];
            for piece in pcm.chunks(4800) {
                let _ = ws.send(Message::binary(piece.to_vec())).await;
            }
            send(
                &mut ws,
                &json!({"type": "audio.done", "sentence_index": index}),
            )
            .await;
        }
        send(&mut ws, &json!({"type": "session.done"})).await;
    })
    .await
}
