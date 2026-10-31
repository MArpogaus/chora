//! The LLM client: `/chat/completions` of an OpenAI-compatible server
//! with SSE and tools (docs/voice.md sections 6 and 7).

use std::time::{Duration, Instant};

use futures_util::StreamExt;
use serde_json::{Value, json};

use crate::config::Config;

/// A tool call of the LLM, put together from its deltas.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// JSON as the LLM wrote it, maybe broken.
    pub args: String,
}

/// One answer of the LLM.
#[derive(Debug, Default, PartialEq)]
pub struct Round {
    pub text: String,
    pub calls: Vec<ToolCall>,
    pub finish: String,
    /// `usage.prompt_tokens`, the base of the token estimate.
    pub prompt_tokens: Option<u64>,
    /// From the request to the first delta with words or a call.
    pub first: Option<Duration>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    /// No connection; worth one retry.
    Connect(String),
    /// The prompt does not fit the context; trim and retry once.
    Context(String),
    /// A timeout of `llm_first_token_s`, `llm_idle_s` or `llm_total_s`.
    Timeout(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(e) | Self::Context(e) | Self::Timeout(e) | Self::Other(e) => {
                f.write_str(e)
            }
        }
    }
}

/// What the server says about its model.
#[derive(Debug, PartialEq)]
pub struct Info {
    pub context_tokens: Option<u32>,
    pub vision: bool,
}

pub struct Llm {
    http: reqwest::Client,
    url: String,
    model: String,
    /// The key and the extra headers, or why the key is missing.
    headers: Result<Vec<(String, String)>, String>,
    reasoning_effort: String,
    first: Duration,
    idle: Duration,
    total: Duration,
}

fn secs(s: u32) -> Duration {
    Duration::from_secs(s.into())
}

impl Llm {
    pub fn new(config: &Config) -> Self {
        Self {
            // The server is on the local network: no proxy.
            http: reqwest::Client::builder()
                .no_proxy()
                .connect_timeout(Duration::from_secs(5))
                .build()
                .expect("a client without TLS builds"),
            url: config.llm_url.trim_end_matches('/').to_owned(),
            model: config.llm_model.clone(),
            headers: config.llm().map(|b| b.headers),
            reasoning_effort: config.reasoning_effort.clone(),
            first: secs(config.llm_first_token_s),
            idle: secs(config.llm_idle_s),
            total: secs(config.llm_total_s),
        }
    }

    /// `request` with the key and the extra headers.
    fn request(&self, request: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder, String> {
        let headers = self.headers.as_ref()?;
        Ok((headers.iter()).fold(request, |r, (name, value)| r.header(name, value)))
    }

    /// A JSON request to `/chat/completions`.
    fn post(&self) -> Result<reqwest::RequestBuilder, String> {
        let url = format!("{}/chat/completions", self.url);
        let request = self.request(self.http.post(url))?;
        Ok(request.header("content-type", "application/json"))
    }

    /// The context and the image input of the model from `/models`.
    ///
    /// # Errors
    ///
    /// The server is away or answers no list.
    pub async fn info(&self) -> Result<Info, String> {
        let models: Value = self
            .request(self.http.get(format!("{}/models", self.url)))?
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())
            .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string()))?;
        let data = models["data"].as_array().ok_or("no model list")?;
        let model = data
            .iter()
            .find(|m| m["id"] == self.model.as_str())
            .or(data.first())
            .ok_or("no model")?;
        let modalities = &model["architecture"]["input_modalities"];
        Ok(Info {
            context_tokens: ["context_length", "max_model_len"]
                .iter()
                .find_map(|k| model[k].as_u64())
                .and_then(|n| u32::try_from(n).ok()),
            vision: modalities
                .as_array()
                .is_some_and(|m| m.iter().any(|m| m == "image")),
        })
    }

    /// The request body; no `tools` key without tools.
    pub fn body(&self, messages: &[Value], tools: &[Value]) -> Value {
        // Gufo thinks by default, which keeps each reply silent for
        // seconds.
        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,
            "stream_options": {"include_usage": true},
        });
        // Empty for a server that refuses the key.
        if !self.reasoning_effort.is_empty() {
            body["reasoning_effort"] = self.reasoning_effort.as_str().into();
        }
        if !tools.is_empty() {
            body["tools"] = tools.into();
        }
        body
    }

    /// Streams one answer; `words` gets each piece of text as it comes.
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn chat(&self, body: &Value, mut words: impl FnMut(&str)) -> Result<Round, Error> {
        let start = Instant::now();
        let deadline = start + self.total;
        let request = self.post().map_err(Error::Other)?;
        let response = tokio::time::timeout(self.first, request.body(body.to_string()).send())
            .await
            .map_err(|_| Error::Timeout(format!("no answer within {:?}", self.first)))?
            .map_err(|e| {
                if e.is_connect() {
                    Error::Connect(format!("{}: {e}", self.url))
                } else {
                    Error::Other(e.to_string())
                }
            })?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(refusal(status.as_u16(), &text));
        }
        let mut stream = response.bytes_stream();
        let (mut round, mut buffer) = (Round::default(), Vec::new());
        loop {
            let wait = if round.first.is_some() {
                self.idle
            } else {
                self.first
            };
            let wait = wait.min(deadline.saturating_duration_since(Instant::now()));
            let Some(chunk) = tokio::time::timeout(wait, stream.next())
                .await
                .map_err(|_| Error::Timeout(format!("no token within {wait:?}")))?
            else {
                return Ok(round);
            };
            buffer.extend_from_slice(&chunk.map_err(|e| Error::Other(e.to_string()))?);
            while let Some(end) = buffer.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=end).collect();
                let line = String::from_utf8_lossy(&line);
                let Some(data) = line.trim().strip_prefix("data:") else {
                    continue;
                };
                let data = data.trim();
                if data == "[DONE]" {
                    return Ok(round);
                }
                let event: Value =
                    serde_json::from_str(data).map_err(|e| Error::Other(format!("SSE: {e}")))?;
                if let Some(text) = round.add(&event)? {
                    round.first.get_or_insert_with(|| start.elapsed());
                    words(&text);
                }
            }
        }
    }

    /// Lets the server read the system prompt and the tools into its
    /// cache, so the first turn does not wait for them.
    ///
    /// # Errors
    ///
    /// The server refuses or is away.
    pub async fn warm(&self, system: &str, tools: &[Value]) -> Result<Duration, String> {
        let start = Instant::now();
        // The chat template of Qwen refuses a prompt without a user
        // message.
        let messages = [
            json!({"role": "system", "content": system}),
            json!({"role": "user", "content": "Hallo"}),
        ];
        let mut body = self.body(&messages, tools);
        body["stream"] = false.into();
        body["max_tokens"] = 1.into();
        body.as_object_mut()
            .expect("an object")
            .remove("stream_options");
        let response = self
            .post()?
            .body(body.to_string())
            .timeout(self.total)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(refusal(status.as_u16(), &text).to_string());
        }
        Ok(start.elapsed())
    }
}

/// An answer that is not 2xx; the server says why.
fn refusal(status: u16, text: &str) -> Error {
    let text: String = text.chars().take(500).collect();
    let message = format!("the LLM answered {status}: {text}");
    // Gufo: code context_length_exceeded; llama.cpp:
    // exceed_context_size_error; vLLM: "maximum context length".
    if text.contains("context") {
        Error::Context(message)
    } else {
        Error::Other(message)
    }
}

impl Round {
    /// Adds one SSE event; answers its words, or an empty string for a
    /// piece of a tool call.
    fn add(&mut self, event: &Value) -> Result<Option<String>, Error> {
        if !event["error"].is_null() {
            return Err(refusal(200, &event["error"].to_string()));
        }
        if let Some(n) = event["usage"]["prompt_tokens"].as_u64() {
            self.prompt_tokens = Some(n);
        }
        let choice = &event["choices"][0];
        if let Some(finish) = choice["finish_reason"].as_str() {
            finish.clone_into(&mut self.finish);
        }
        let delta = &choice["delta"];
        let mut seen = None;
        // A call comes in parts: the id and the name first, then the
        // arguments piece by piece.
        for part in delta["tool_calls"].as_array().into_iter().flatten() {
            let index = part["index"].as_u64().unwrap_or(0);
            let index = usize::try_from(index).unwrap_or(usize::MAX);
            if index >= self.calls.len() {
                if index > 16 {
                    return Err(Error::Other(format!("tool call index {index}")));
                }
                self.calls.resize(index + 1, ToolCall::default());
            }
            let call = &mut self.calls[index];
            if let Some(id) = part["id"].as_str().filter(|id| !id.is_empty()) {
                id.clone_into(&mut call.id);
            }
            call.name += part["function"]["name"].as_str().unwrap_or_default();
            call.args += part["function"]["arguments"].as_str().unwrap_or_default();
            seen = Some(String::new());
        }
        match delta["content"].as_str() {
            Some(text) if !text.is_empty() => {
                self.text += text;
                Ok(Some(text.to_owned()))
            }
            _ => Ok(seen),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::{self, Reply};

    fn llm(url: &str) -> Llm {
        Llm::new(&Config {
            llm_url: url.into(),
            llm_first_token_s: 1,
            llm_idle_s: 1,
            llm_total_s: 3,
            ..Config::default()
        })
    }

    #[test]
    fn tool_calls_come_together_from_their_deltas() {
        let mut round = Round::default();
        for event in [
            json!({"choices": [{"delta": {"content": "Ich schaue."}}]}),
            json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "id": "a",
                "function": {"name": "query", "arguments": "{\"x\""}}]}}]}),
            json!({"choices": [{"delta": {"tool_calls": [{"index": 0,
                "function": {"arguments": ": 1}"}}]}}]}),
            json!({"choices": [{"delta": {"tool_calls": [{"index": 1, "id": "b",
                "function": {"name": "tiling.set_layout", "arguments": "{}"}}]}}]}),
            json!({"choices": [{"delta": {}, "finish_reason": "tool_calls"}]}),
            json!({"choices": [], "usage": {"prompt_tokens": 278}}),
        ] {
            round.add(&event).unwrap();
        }
        assert_eq!(round.text, "Ich schaue.");
        assert_eq!(
            round.calls,
            [
                ToolCall {
                    id: "a".into(),
                    name: "query".into(),
                    args: "{\"x\": 1}".into()
                },
                ToolCall {
                    id: "b".into(),
                    name: "tiling.set_layout".into(),
                    args: "{}".into()
                }
            ]
        );
        assert_eq!(
            (round.finish.as_str(), round.prompt_tokens),
            ("tool_calls", Some(278))
        );
        let error = json!({"error": {"message": "prompt exceeds the context"}});
        assert!(matches!(round.add(&error), Err(Error::Context(_))));
    }

    #[tokio::test]
    async fn chat_streams_the_words_and_reads_the_model() {
        let server = fake::llm(vec![Reply::Text("Hallo. Wie geht es?")]).await;
        let llm = llm(&server.url);
        let mut words = String::new();
        let body = llm.body(&[], &[]);
        let round = llm.chat(&body, |w| words.push_str(w)).await.unwrap();
        assert_eq!(words, "Hallo. Wie geht es?");
        assert_eq!(round.text, words);
        let tokens = body.to_string().len() / 4;
        assert_eq!(round.prompt_tokens, Some(tokens as u64));
        assert_eq!(
            llm.info().await.unwrap(),
            Info {
                context_tokens: Some(4000),
                vision: true
            }
        );
        let seen = server.seen.lock().unwrap();
        assert_eq!(seen[0]["reasoning_effort"], "none");
        assert!(seen[0].get("tools").is_none(), "no tools, no key");
    }

    #[tokio::test]
    async fn refusals_and_silence_are_errors_of_their_kind() {
        let server = fake::llm(vec![
            Reply::Status(400, r#"{"error":{"code":"context_length_exceeded"}}"#),
            Reply::Status(500, "boom"),
            Reply::Silent,
        ])
        .await;
        let llm = llm(&server.url);
        let body = llm.body(&[], &[]);
        assert!(matches!(
            llm.chat(&body, |_| ()).await,
            Err(Error::Context(_))
        ));
        assert!(
            matches!(llm.chat(&body, |_| ()).await, Err(Error::Other(e)) if e.contains("boom"))
        );
        let start = Instant::now();
        assert!(matches!(
            llm.chat(&body, |_| ()).await,
            Err(Error::Timeout(_))
        ));
        assert!(start.elapsed() < Duration::from_secs(3));
        let away = self::llm("http://127.0.0.1:9");
        assert!(matches!(
            away.chat(&body, |_| ()).await,
            Err(Error::Connect(_))
        ));
    }
}
