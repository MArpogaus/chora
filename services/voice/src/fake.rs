//! Fake servers for the tests on 127.0.0.1: ASR and TTS websockets that
//! speak the protocols of docs/voice.md section 8 from a script, and an
//! LLM with SSE.

use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
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
    /// Answers the n-th commit with the n-th text, then with the last.
    Seq(&'static [&'static str]),
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
    let commits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    start(move |mut ws, seen| {
        let (failed, commits) = (failed.clone(), commits.clone());
        async move {
            while let Some(event) = next(&mut ws, &seen).await {
                if event["type"] != "input_audio_buffer.commit" {
                    continue;
                }
                let text = match script {
                    Asr::Silent => continue,
                    Asr::FailFirst(_) if !failed.swap(true, Ordering::SeqCst) => return,
                    Asr::Text(text) | Asr::FailFirst(text) => text,
                    Asr::Seq(texts) => {
                        let n = commits.fetch_add(1, Ordering::SeqCst);
                        texts[n.min(texts.len() - 1)]
                    }
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

/// An answer of the fake LLM.
#[derive(Clone, Copy, Debug)]
pub enum Reply {
    /// Words in pieces of up to 4 characters, then `stop`.
    Text(&'static str),
    /// One tool call: the name and the arguments, in two pieces.
    Tool(&'static str, &'static str),
    /// An HTTP status with this body.
    Status(u16, &'static str),
    /// Never answers.
    Silent,
}

type Script = Arc<Mutex<VecDeque<Reply>>>;

/// An OpenAI-compatible server: `GET /models` (context 4000, images)
/// and `POST /chat/completions` with SSE, one scripted reply per request
/// and then `Text("")`. `usage.prompt_tokens` is a quarter of the length
/// of the request body. `seen` gets the request bodies.
pub async fn llm(script: Vec<Reply>) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let seen = Arc::<Mutex<Vec<Value>>>::default();
    let (log, script) = (seen.clone(), Arc::new(Mutex::new(VecDeque::from(script))));
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            tokio::spawn(answer(tcp, log.clone(), script.clone()));
        }
    });
    Server { url, seen }
}

/// The head (lowercase) and the body of one HTTP request.
async fn request(tcp: &mut TcpStream) -> Option<(String, Vec<u8>)> {
    let (mut request, mut buf) = (Vec::new(), vec![0; 65536]);
    let (head, length) = loop {
        let n = tcp.read(&mut buf).await.ok().filter(|&n| n > 0)?;
        request.extend_from_slice(&buf[..n]);
        if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&request[..end]).to_lowercase();
            let length = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .map_or(0, |n| n.trim().parse().unwrap());
            request.drain(..end + 4);
            break (head, length);
        }
    };
    while request.len() < length {
        let n = tcp.read(&mut buf).await.ok().filter(|&n| n > 0)?;
        request.extend_from_slice(&buf[..n]);
    }
    Some((head, request))
}

async fn answer(mut tcp: TcpStream, seen: Arc<Mutex<Vec<Value>>>, script: Script) {
    let Some((head, body)) = request(&mut tcp).await else {
        return;
    };
    let (status, kind, body) = if head.starts_with("get") {
        let models = json!({"data": [{"id": "gufo", "context_length": 4000,
            "architecture": {"input_modalities": ["text", "image"]}}]});
        (200, "application/json", models.to_string())
    } else {
        let tokens = body.len() / 4;
        seen.lock()
            .unwrap()
            .push(serde_json::from_slice(&body).unwrap());
        let reply = script.lock().unwrap().pop_front();
        match reply.unwrap_or(Reply::Text("")) {
            Reply::Silent => return std::future::pending().await,
            Reply::Status(status, body) => (status, "application/json", body.into()),
            reply => (200, "text/event-stream", sse(reply, tokens)),
        }
    };
    let head = format!(
        "HTTP/1.1 {status} X\r\ncontent-type: {kind}\r\ncontent-length: {}\r\n\
         connection: close\r\n\r\n",
        body.len()
    );
    let _ = tcp.write_all(head.as_bytes()).await;
    let _ = tcp.write_all(body.as_bytes()).await;
}

fn sse(reply: Reply, tokens: usize) -> String {
    let chunk = |delta: Value, finish: Value| json!({"choices": [{"index": 0, "delta": delta, "finish_reason": finish}]});
    let mut events = vec![chunk(json!({"role": "assistant"}), Value::Null)];
    let finish = match reply {
        Reply::Text(text) => {
            let chars: Vec<char> = text.chars().collect();
            for piece in chars.chunks(4) {
                let piece: String = piece.iter().collect();
                events.push(chunk(json!({"content": piece}), Value::Null));
            }
            "stop"
        }
        Reply::Tool(name, args) => {
            let (a, b) = args.split_at(args.len() / 2);
            let call = |first: bool, args: &str| {
                let mut part = json!({"index": 0, "function": {"arguments": args}});
                if first {
                    part["id"] = "call-1".into();
                    part["function"]["name"] = name.into();
                }
                json!({"tool_calls": [part]})
            };
            events.push(chunk(call(true, a), Value::Null));
            events.push(chunk(call(false, b), Value::Null));
            "tool_calls"
        }
        Reply::Status(..) | Reply::Silent => unreachable!(),
    };
    events.push(chunk(json!({}), finish.into()));
    events.push(json!({"choices": [], "usage": {"prompt_tokens": tokens}}));
    let mut body = String::new();
    for event in events
        .iter()
        .map(ToString::to_string)
        .chain(["[DONE]".into()])
    {
        body += "data: ";
        body += &event;
        body += "\n\n";
    }
    body
}
