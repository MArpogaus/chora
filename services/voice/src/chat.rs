//! The conversation with the LLM: the system prompt with the context
//! documents, the history with its token budget and file, and the tool
//! loop of a turn with the confirmation gate (docs/voice.md sections 5
//! to 7).

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tokio::sync::oneshot;

use crate::{
    config::Config,
    llm::{self, Llm, Round},
    speech::Splitter,
    tools::{self, Tools},
};

/// The built-in prompt; `{name}` is the client name of the tools.
const SYSTEM: &str = include_str!("prompt.md");

/// The built-in context documents: `FILE` or `FILE#HEADING` for one
/// `## HEADING` section.
const CONTEXT_DOCS: [&str; 12] = [
    "README.md",
    "docs/glossary.md",
    "docs/api.md#Model",
    "docs/api.md#Nodes",
    "docs/api.md#Commands",
    "docs/api.md#Window managers",
    "docs/api.md#Selectors",
    "docs/api.md#Programs",
    "docs/api.md#Input injection",
    "docs/api.md#Plugin methods",
    "docs/api.md#Plugins",
    "docs/api.md#MCP tools",
];

/// The files of the built-in documents, from the build.
const FILES: [(&str, &str); 3] = [
    ("README.md", include_str!("../../../README.md")),
    (
        "docs/glossary.md",
        include_str!("../../../docs/glossary.md"),
    ),
    ("docs/api.md", include_str!("../../../docs/api.md")),
];

/// The note of the final round when `tool_rounds` is reached; it does
/// not stay in the history.
const LAST_ROUND: &str = "(The agent: you reached the limit of tool calls for this request. \
    Call no tool. In the language of the user, say in one or two short sentences what you did \
    and what is left.)";

/// The part of the prompt for the coding tools, with a `coder_cmd`.
const CODING: &str = "# Coding tasks\n\n\
- Hand a task that needs code, a program or many steps to code_task. It runs in the \
  background in a clone of a project of list_projects, or in an empty repository, and you \
  hear its result when it ends.\n\
- Say the task in full: the coder hears nothing else. Ask it to commit its work, or the \
  clone keeps it uncommitted. One task runs at a time; code_status says how far it is, \
  code_cancel stops it.\n\
- When the user asks about a spoken coding result, call code_status.";

/// The part of the prompt for the ACP agents, with an `agent_cmd`.
const AGENTS: &str = "# Agents\n\n\
- Coding agents work in the projects of agent_list, one per project, each with its own \
  name and voice. agent_start starts one; it keeps its conversation until agent_stop.\n\
- Hand work in a project to its agent with agent_say, in full: it hears nothing else. \
  Its answer is spoken in its voice when it is done; you need not repeat it.\n\
- The user can also talk to an agent directly by its name; agent_status tells what each \
  agent does, its plan and its last words.";

/// The characters that a picture counts for in the estimate, instead of
/// its base64 text.
const PICTURE_CHARS: usize = 4000;

/// The note in the history for a turn that the user stopped.
pub const CANCELLED: &str = "cancelled by the user; result unknown";

/// What a turn tells the agent while it runs.
#[derive(Debug)]
pub enum Event {
    /// A request to the LLM starts.
    Thinking,
    /// A tool call starts.
    Tool(String),
    /// A sentence to speak.
    Sentence(String),
    /// A call waits for yes (true) or no; the prompt is spoken too. A
    /// dropped sender is a no.
    Confirm {
        prompt: String,
        answer: oneshot::Sender<bool>,
    },
}

/// What a turn did, for the log.
#[derive(Debug, Default)]
pub struct Report {
    pub reply: String,
    pub rounds: u32,
    pub tools: u32,
    pub tools_time: Duration,
    /// From the start of the turn to the first token.
    pub llm_first: Option<Duration>,
    pub outcome: &'static str,
}

pub struct Chat {
    /// The client name of the tools in chora.
    name: String,
    llm: Llm,
    pub tools: Tools,
    config: Config,
    /// The system prompt with the documents and the earlier requests.
    system: String,
    /// The messages after the system prompt.
    history: Vec<Value>,
    chars_per_token: f64,
    context_tokens: u32,
    file: Option<PathBuf>,
    /// The requests of the user, of earlier sessions and of this one.
    requests: Vec<String>,
    /// The requests of earlier sessions, for the prompt.
    earlier: Vec<String>,
    /// A turn in progress, with the tool calls of the round in flight; a
    /// stopped turn keeps it.
    open: Option<Vec<Value>>,
    /// The answer of the last turn in the history, until the next turn.
    answer: Option<usize>,
}

impl Chat {
    /// Asks the server for its context and image input, loads the tools,
    /// the history file and the prompt.
    ///
    /// # Errors
    ///
    /// A context document or one of its headings is missing.
    pub async fn start(
        config: Config,
        name: String,
        mut tools: Tools,
        file: Option<PathBuf>,
    ) -> Result<Self, String> {
        let llm = Llm::new(&config);
        let info = llm.info().await.unwrap_or_else(|e| {
            crate::log::warn(&format!("llm /models: {e}"));
            llm::Info {
                context_tokens: None,
                vision: false,
            }
        });
        let context_tokens = match config.context_tokens {
            0 => info.context_tokens.unwrap_or(32_768),
            n => n,
        };
        tools.vision = info.vision;
        tools.refresh().await;
        crate::log::info(&format!(
            "llm context={context_tokens} vision={} tools={}",
            info.vision,
            tools.specs.len()
        ));
        let requests = file.as_deref().map(load).unwrap_or_default();
        let mut chat = Self {
            name,
            llm,
            tools,
            system: String::new(),
            history: Vec::new(),
            chars_per_token: 3.0,
            context_tokens,
            file,
            earlier: requests.clone(),
            requests,
            config,
            open: None,
            answer: None,
        };
        chat.system = chat.prompt()?;
        Ok(chat)
    }

    /// Takes new settings; answers whether the prompt changed, which asks
    /// for a new warm-up.
    ///
    /// # Errors
    ///
    /// As `start`; the old settings stay.
    pub fn configure(&mut self, config: Config) -> Result<bool, String> {
        let old = std::mem::replace(&mut self.config, config);
        let system = match self.prompt() {
            Ok(system) => system,
            Err(e) => {
                self.config = old;
                return Err(e);
            }
        };
        self.llm = Llm::new(&self.config);
        if let Some(coder) = &self.tools.coder {
            coder.set_cmd(self.config.spawn(&self.config.coder_cmd));
        }
        if let Some(agents) = &self.tools.agents {
            agents.configure(&self.config);
        }
        self.tools.timeout = Duration::from_secs(self.config.tool_timeout_s.into());
        self.tools.skipped.clone_from(&self.config.skipped_tools);
        if self.config.context_tokens > 0 {
            self.context_tokens = self.config.context_tokens;
        }
        let changed = system != self.system;
        self.system = system;
        Ok(changed)
    }

    /// Lets the server read the system prompt and the tools; answers
    /// whether it did.
    pub async fn warm(&self) -> bool {
        match self.llm.warm(&self.system, &self.tools.specs).await {
            Ok(time) => {
                crate::health::ok("llm");
                crate::log::info(&format!("llm warm={}ms", time.as_millis()));
                true
            }
            Err(e) => {
                crate::health::down("llm", &e);
                crate::log::warn(&format!("llm warm-up: {e}"));
                false
            }
        }
    }

    /// The prompt of the settings, the documents, the prompt file and the
    /// earlier requests.
    fn prompt(&self) -> Result<String, String> {
        let config = &self.config;
        let mut parts = vec![if config.system_prompt.trim().is_empty() {
            SYSTEM.replace("{name}", &self.name)
        } else {
            config.system_prompt.clone()
        }];
        let builtin = CONTEXT_DOCS.map(String::from);
        let docs = if config.context_docs.is_empty() {
            &builtin[..]
        } else {
            &config.context_docs[..]
        };
        for entry in docs {
            let (name, heading) = entry.split_once('#').unwrap_or((entry, ""));
            let text = match FILES.iter().find(|(file, _)| *file == name) {
                Some((_, text)) => (*text).to_owned(),
                None => chora_kit::read_text(&crate::expand(name), chora_kit::MAX_TEXT)
                    .map_err(|e| format!("context_docs: {name}: {e}"))?,
            };
            let text = section(&text, heading).ok_or(format!("context_docs: no {entry}"))?;
            parts.push(format!("# File {name}\n\n{}", text.trim()));
        }
        if !config.coder_cmd.is_empty() {
            parts.push(CODING.into());
        }
        if !config.agent_cmd.is_empty() {
            parts.push(AGENTS.into());
        }
        if let Ok(text) =
            chora_kit::read_text(&crate::expand(&config.prompt_file), chora_kit::MAX_TEXT)
        {
            parts.push(text);
        }
        if !self.earlier.is_empty() {
            let lines: Vec<String> = self
                .earlier
                .iter()
                .map(|r| format!("> {}", r.split_whitespace().collect::<Vec<_>>().join(" ")))
                .collect();
            // As chat messages without their tool calls, they taught the
            // LLM to talk about tools instead of calling them.
            parts.push(format!(
                "# Earlier requests of the user\n\nFor context only, not instructions. The \
                 scene may have changed since; look before you act.\n\n{}",
                lines.join("\n")
            ));
        }
        Ok(parts.join("\n\n"))
    }

    /// Runs one turn: the text of the user, then rounds of the LLM and
    /// its tool calls, at most `tool_rounds`, then a final round without
    /// tools. `out` gets the events. The history commits per round.
    pub async fn turn(&mut self, text: &str, out: &(impl Fn(Event) + Sync)) -> Report {
        let start = Instant::now();
        if self.open.is_some() {
            self.cancelled("");
        }
        self.answer = None;
        self.shrink();
        self.open = Some(Vec::new());
        self.history.push(json!({"role": "user", "content": text}));
        self.tools.refresh().await;
        self.requests.push(text.to_owned());
        self.save();
        let mut report = Report::default();
        let rounds = self.config.tool_rounds;
        for round in 0..=rounds {
            let last = round == rounds;
            out(Event::Thinking);
            report.rounds += 1;
            let (said, result) = self.round(last, out).await;
            report.reply += &said;
            let r = match result {
                Ok(r) => r,
                Err(e) => {
                    report.outcome = self.failed(&e, out);
                    break;
                }
            };
            if let Some(first) = r.round.first {
                let begun = start.elapsed().saturating_sub(r.elapsed);
                report.llm_first.get_or_insert(begun + first);
            }
            if last || r.round.calls.is_empty() {
                report.outcome = self.answered(&r.round, last, said.is_empty(), out);
                break;
            }
            let tools_start = Instant::now();
            report.tools += self.run(r.round, out).await;
            report.tools_time += tools_start.elapsed();
        }
        self.open = None;
        report
    }

    /// Says a text of the agent's own, such as the result of a coding
    /// task, sentence by sentence through `say`, and puts it into the
    /// history as an answer.
    pub fn said(&mut self, text: &str, say: impl FnMut(String)) -> Report {
        let mut splitter = Splitter::default();
        let sentences = splitter.push(text);
        sentences.into_iter().chain(splitter.finish()).for_each(say);
        if self.open.is_some() {
            self.cancelled("");
        }
        self.answer = Some(self.history.len());
        self.history
            .push(json!({"role": "assistant", "content": text}));
        Report {
            reply: text.to_owned(),
            outcome: "said",
            ..Report::default()
        }
    }

    /// Closes the last turn, which the user stopped: the words that
    /// played and the note `CANCELLED`, also as the result of each tool
    /// call in flight. A turn whose words were all there but did not all
    /// play keeps only the played words.
    pub fn cancelled(&mut self, played: &str) {
        let played = played.trim();
        let note = if played.is_empty() {
            format!("({CANCELLED})")
        } else {
            format!("{played} ({CANCELLED})")
        };
        let calls = match (self.open.take(), self.answer.take()) {
            (Some(calls), _) if !calls.is_empty() => calls,
            (None, Some(i)) => {
                self.history[i]["content"] = note.into();
                return;
            }
            _ => {
                self.history
                    .push(json!({"role": "assistant", "content": note}));
                return;
            }
        };
        let ids: Vec<Value> = calls.iter().map(|c| c["id"].clone()).collect();
        self.history
            .push(json!({"role": "assistant", "content": played, "tool_calls": calls}));
        for id in ids {
            self.history
                .push(json!({"role": "tool", "tool_call_id": id, "content": CANCELLED}));
        }
    }

    /// Says that the LLM failed.
    fn failed(&self, e: &llm::Error, out: &impl Fn(Event)) -> &'static str {
        crate::log::error(&format!("llm: {e}"));
        let phrases = &self.config.phrases;
        let phrase = match e {
            llm::Error::Timeout(_) => &phrases.llm_slow,
            _ => &phrases.llm_down,
        };
        out(Event::Sentence(phrase.clone()));
        "llm_error"
    }

    /// Commits the last answer of a turn; after the round limit without
    /// words, says so.
    fn answered(
        &mut self,
        round: &Round,
        last: bool,
        silent: bool,
        out: &impl Fn(Event),
    ) -> &'static str {
        crate::log::info(&format!("llm: {} {:?}", round.finish, round.text));
        if !round.text.is_empty() {
            self.answer = Some(self.history.len());
            self.history
                .push(json!({"role": "assistant", "content": round.text}));
        }
        if !last {
            return "answered";
        }
        if silent {
            out(Event::Sentence(self.config.phrases.tool_rounds.clone()));
        }
        "tool_rounds"
    }

    /// Runs the calls of a round and commits it with the results;
    /// answers the number of calls.
    async fn run(&mut self, round: Round, out: &(impl Fn(Event) + Sync)) -> u32 {
        let calls: Vec<Value> = round
            .calls
            .iter()
            .map(|c| {
                json!({"id": c.id, "type": "function",
                    "function": {"name": c.name, "arguments": c.args}})
            })
            .collect();
        self.open = Some(calls.clone());
        let results = self.calls(&round.calls, out).await;
        self.open = Some(Vec::new());
        self.history.push(json!({"role": "assistant",
            "content": round.text, "tool_calls": calls}));
        self.history.extend(results);
        u32::try_from(calls.len()).unwrap_or(u32::MAX)
    }

    /// One request with retries: once more after a connect error, and
    /// with half the budget after a context error. Answers the words that
    /// went out as sentences.
    async fn round(
        &mut self,
        last: bool,
        out: &(impl Fn(Event) + Sync),
    ) -> (String, Result<Timed, llm::Error>) {
        let budget = self
            .context_tokens
            .saturating_sub(self.config.reply_reserve_tokens);
        let mut said = String::new();
        let mut retried = false;
        loop {
            let tools = self.tools.specs.clone();
            let fixed = self.system.len() + serde_json::to_string(&tools).map_or(0, |t| t.len());
            let budget = if retried { budget / 2 } else { budget };
            self.fit(budget, fixed);
            let mut messages = vec![json!({"role": "system", "content": self.system})];
            messages.extend(self.history.iter().cloned());
            if last {
                messages.push(json!({"role": "user", "content": LAST_ROUND}));
            }
            let mut body = self.llm.body(&messages, &tools);
            if last && !tools.is_empty() {
                // The tools stay in the request, so the server keeps the
                // prompt in its cache; without them it reads it again.
                body["tool_choice"] = "none".into();
            }
            let chars = fixed + size(&messages[1..]);
            let mut splitter = Splitter::default();
            let start = Instant::now();
            let result = self
                .llm
                .chat(&body, |words| {
                    for sentence in splitter.push(words) {
                        said += &sentence;
                        said.push(' ');
                        out(Event::Sentence(sentence));
                    }
                })
                .await;
            // The words before the tool calls are spoken while they run.
            if let Some(sentence) = splitter.finish() {
                said += &sentence;
                out(Event::Sentence(sentence));
            }
            match result {
                Err(llm::Error::Connect(_) | llm::Error::Context(_)) if !retried => {
                    retried = true;
                }
                Err(e) => {
                    crate::health::down("llm", &e.to_string());
                    return (said, Err(e));
                }
                Ok(round) => {
                    crate::health::ok("llm");
                    self.learn(&round, chars, &messages);
                    let elapsed = start.elapsed();
                    return (said, Ok(Timed { round, elapsed }));
                }
            }
        }
    }

    /// Learns the characters per token from `usage.prompt_tokens`; not
    /// from a request with a picture, whose tokens the server counts its
    /// own way.
    fn learn(&mut self, round: &Round, chars: usize, messages: &[Value]) {
        let picture = messages.iter().any(|m| m["content"].is_array());
        if let Some(tokens) = round.prompt_tokens.filter(|&t| t > 0 && !picture) {
            #[allow(clippy::cast_precision_loss)]
            let ratio = chars as f64 / tokens as f64;
            self.chars_per_token = ratio.clamp(1.0, 8.0);
            crate::log::debug(&format!(
                "llm prompt_tokens={tokens} chars_per_token={ratio:.2}"
            ));
        }
    }

    /// Runs the calls one after the other, each after its confirmation;
    /// answers their tool messages.
    async fn calls(
        &mut self,
        calls: &[llm::ToolCall],
        out: &(impl Fn(Event) + Sync),
    ) -> Vec<Value> {
        let mut results = Vec::new();
        for call in calls {
            let args: Value = if call.args.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(&call.args).unwrap_or(Value::Null)
            };
            let content = if args.is_null() {
                Value::from(format!("error: the arguments are no JSON: {}", call.args))
            } else if self.confirmed(&call.name, &args, out).await {
                out(Event::Tool(call.name.clone()));
                let start = Instant::now();
                let content = self.tools.call(&call.name, &args).await;
                crate::log::info(&format!(
                    "tool {} {} {}ms -> {}",
                    call.name,
                    call.args,
                    start.elapsed().as_millis(),
                    short(&text_of(&content), 300)
                ));
                content
            } else {
                "declined by the user; nothing happened".into()
            };
            results.push(json!({"role": "tool", "tool_call_id": call.id, "content": content}));
        }
        results
    }

    /// Asks the user when a rule of `confirm` matches; a timeout is a no.
    async fn confirmed(&self, function: &str, args: &Value, out: &(impl Fn(Event) + Sync)) -> bool {
        let Some(name) = self.tools.mcp_name(function) else {
            return true;
        };
        let Some(rule) = tools::rule(&self.config.confirm, name, args) else {
            return true;
        };
        let (answer, rx) = oneshot::channel();
        let program = tools::program(args).unwrap_or_default();
        out(Event::Confirm {
            prompt: rule.prompt.replace("{program}", program),
            answer,
        });
        let wait = Duration::from_millis(self.config.confirm_timeout_ms.into());
        let yes = matches!(tokio::time::timeout(wait, rx).await, Ok(Ok(true)));
        crate::log::info(&format!(
            "confirm {name}: {}",
            if yes { "yes" } else { "no" }
        ));
        yes
    }

    /// Drops the oldest whole turns until the estimate fits `budget`;
    /// the turn in progress stays.
    fn fit(&mut self, budget: u32, fixed: usize) {
        loop {
            #[allow(clippy::cast_precision_loss)]
            let tokens = (fixed + size(&self.history)) as f64 / self.chars_per_token;
            if tokens <= f64::from(budget) {
                return;
            }
            let Some(next) = self
                .history
                .iter()
                .skip(1)
                .position(|m| m["role"] == "user")
            else {
                return;
            };
            crate::log::info(&format!("history: dropped a turn, ~{tokens:.0} tokens"));
            self.history.drain(..=next);
        }
    }

    /// Before a new turn: old tool texts keep `old_tool_chars`, without
    /// pictures; long arguments become `{"shortened": true}`, except the
    /// last `put_asset` of each name, so the LLM can change its shader.
    fn shrink(&mut self) {
        let limit = self.config.old_tool_chars as usize;
        let mut assets = HashMap::new();
        for (i, message) in self.history.iter().enumerate() {
            for (j, call) in message["tool_calls"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                if call["function"]["name"] == "put_asset" {
                    let args: Value = serde_json::from_str(
                        call["function"]["arguments"].as_str().unwrap_or("{}"),
                    )
                    .unwrap_or_default();
                    assets.insert(args["name"].to_string(), (i, j));
                }
            }
        }
        let keep: Vec<(usize, usize)> = assets.into_values().collect();
        for (i, message) in self.history.iter_mut().enumerate() {
            if message["role"] == "tool" {
                let text = text_of(&message["content"]);
                message["content"] = if text.chars().count() > limit {
                    format!("{} [shortened]", short(&text, limit)).into()
                } else {
                    text.into()
                };
            }
            // Indexing a missing key with `[]` would add it.
            let calls = message.get_mut("tool_calls").and_then(Value::as_array_mut);
            for (j, call) in calls.into_iter().flatten().enumerate() {
                let args = &mut call["function"]["arguments"];
                if args.as_str().is_some_and(|a| a.len() > limit) && !keep.contains(&(i, j)) {
                    // Arguments stay JSON, which the chat template reads.
                    *args = json!({"shortened": true}).to_string().into();
                }
            }
        }
    }

    /// Writes the last `saved_requests` requests to the history file.
    fn save(&mut self) {
        let keep = self.config.saved_requests as usize;
        let skip = self.requests.len().saturating_sub(keep);
        self.requests.drain(..skip);
        let Some(file) = &self.file else {
            return;
        };
        // The format of the Python agent: user messages.
        let words: Vec<Value> = self
            .requests
            .iter()
            .map(|r| json!({"role": "user", "content": r}))
            .collect();
        let part = file.with_extension("part");
        let result = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&part, Value::from(words).to_string()))
            .and_then(|()| std::fs::rename(&part, file));
        if let Err(e) = result {
            crate::log::warn(&format!("{}: {e}", file.display()));
        }
    }
}

/// The time of a round, for the first token of the turn.
struct Timed {
    round: Round,
    elapsed: Duration,
}

/// `$XDG_STATE_HOME/chora/voice-history.json`.
pub fn history_file() -> Option<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".local/state")))?;
    Some(state.join("chora/voice-history.json"))
}

/// The requests in the history file; none for a missing or broken file.
fn load(file: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    let words: Value = serde_json::from_str(&text).unwrap_or_default();
    words
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["content"].as_str().map(str::to_owned))
        .collect()
}

/// The `## HEADING` section of a markdown text, or all of it for an
/// empty heading.
fn section<'a>(text: &'a str, heading: &str) -> Option<&'a str> {
    if heading.is_empty() {
        return Some(text);
    }
    let start = text.find(&format!("\n## {heading}\n"))? + 1;
    let end = text[start + 1..]
        .find("\n## ")
        .map_or(text.len(), |e| start + 1 + e);
    Some(&text[start..end])
}

/// The text of a message content, without its pictures.
fn text_of(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

/// The first `n` characters.
fn short(text: &str, n: usize) -> String {
    text.chars().take(n).collect()
}

/// The JSON length of messages, with a picture as `PICTURE_CHARS`.
fn size(messages: &[Value]) -> usize {
    messages
        .iter()
        .map(|m| {
            let pictures: usize = m["content"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|p| p["image_url"]["url"].as_str())
                .map(|url| url.len().saturating_sub(PICTURE_CHARS))
                .sum();
            m.to_string().len() - pictures
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, atomic::Ordering};

    use super::*;
    use crate::{
        config::Confirm,
        fake::{self, Reply},
    };

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-voice-chat-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    /// A chat with the fake LLM and the fake tools, a short prompt and
    /// one small document.
    async fn chat(script: Vec<Reply>, config: Config) -> (Chat, fake::Server) {
        let server = fake::llm(script).await;
        let doc = temp("doc.md");
        std::fs::write(&doc, "Ein Dokument.").unwrap();
        let config = Config {
            llm_url: server.url.clone(),
            system_prompt: "Du bist ein Test.".into(),
            context_docs: vec![doc.display().to_string()],
            prompt_file: String::new(),
            ..config
        };
        let (tools, _) = tools::tests::fake(0);
        let chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        (chat, server)
    }

    /// Runs a turn; answers each confirmation with `yes` and gives the
    /// events as text.
    async fn turn(chat: &mut Chat, text: &str, yes: bool) -> (Report, Vec<String>) {
        let events = Mutex::new(Vec::new());
        let report = chat
            .turn(text, &|event| {
                let line = match event {
                    Event::Thinking => "thinking".into(),
                    Event::Tool(name) => format!("tool {name}"),
                    Event::Sentence(s) => format!("say {s}"),
                    Event::Confirm { prompt, answer } => {
                        let _ = answer.send(yes);
                        format!("confirm {prompt}")
                    }
                };
                events.lock().unwrap().push(line);
            })
            .await;
        (report, events.into_inner().unwrap())
    }

    #[tokio::test]
    async fn a_stopped_turn_leaves_the_note_in_the_history() {
        let script = vec![
            Reply::Tool("sleep", "{}"),
            Reply::Silent,
            Reply::Text("Gut."),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let stop = Duration::from_millis(150);
        for text in ["Schlaf ein.", "Warte."] {
            let cut = tokio::time::timeout(stop, turn(&mut chat, text, true)).await;
            assert!(cut.is_err(), "the turn hangs in the tool, then in the LLM");
        }
        chat.cancelled("Ich warte");
        turn(&mut chat, "Und?", true).await;
        let seen = server.seen.lock().unwrap();
        let messages = &seen.last().unwrap()["messages"].as_array().unwrap()[1..];
        let roles: Vec<_> = messages
            .iter()
            .map(|m| m["role"].as_str().unwrap())
            .collect();
        assert_eq!(
            roles,
            ["user", "assistant", "tool", "user", "assistant", "user"]
        );
        assert_eq!(messages[1]["tool_calls"][0]["function"]["name"], "sleep");
        assert_eq!(messages[2]["content"], CANCELLED);
        assert_eq!(
            messages[4]["content"],
            "Ich warte (cancelled by the user; result unknown)"
        );
    }

    #[tokio::test]
    async fn a_turn_runs_the_tool_calls_and_speaks_the_answer() {
        let script = vec![
            Reply::Tool("echo", r#"{"a": 1}"#),
            Reply::Text("Fertig. Alles gut."),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let (report, events) = turn(&mut chat, "Mach was.", true).await;
        assert_eq!(
            events,
            [
                "thinking",
                "tool echo",
                "thinking",
                "say Fertig.",
                "say Alles gut."
            ]
        );
        assert_eq!(
            (report.outcome, report.tools, report.rounds),
            ("answered", 1, 2)
        );
        let seen = server.seen.lock().unwrap();
        let system = seen[0]["messages"][0]["content"].as_str().unwrap();
        assert!(
            system.starts_with("Du bist ein Test.\n\n# File "),
            "{system}"
        );
        assert!(seen[0]["tools"].as_array().unwrap().len() > 3);
        let messages = seen[1]["messages"].as_array().unwrap();
        assert_eq!(messages[2]["tool_calls"][0]["function"]["name"], "echo");
        assert_eq!(
            (&messages[3]["role"], &messages[3]["content"]),
            (&json!("tool"), &json!(r#"{"a":1}"#))
        );
        assert_eq!(chat.history.len(), 4, "user, call, result, answer");
    }

    #[tokio::test]
    async fn the_last_round_has_no_tools_and_asks_for_a_summary() {
        let script = vec![
            Reply::Tool("echo", "{}"),
            Reply::Tool("echo", "{}"),
            Reply::Text("Ich habe zweimal gesucht."),
        ];
        let config = Config {
            tool_rounds: 2,
            ..Config::default()
        };
        let (mut chat, server) = chat(script, config).await;
        let (report, events) = turn(&mut chat, "Such.", true).await;
        assert_eq!(report.outcome, "tool_rounds");
        assert_eq!(events.last().unwrap(), "say Ich habe zweimal gesucht.");
        {
            let seen = server.seen.lock().unwrap();
            assert_eq!(seen.len(), 3);
            assert_eq!(seen[2]["tool_choice"], "none");
            let messages = seen[2]["messages"].as_array().unwrap();
            assert_eq!(messages.last().unwrap()["content"], LAST_ROUND);
        }
        assert!(
            chat.history.iter().all(|m| m["content"] != LAST_ROUND),
            "the note stays out of the history"
        );
        // Without words, the phrase.
        let config = Config {
            tool_rounds: 1,
            ..Config::default()
        };
        let script = vec![Reply::Tool("echo", "{}"), Reply::Text("")];
        let (mut chat, _) = self::chat(script, config).await;
        let (_, events) = turn(&mut chat, "Such.", true).await;
        assert_eq!(
            events.last().unwrap(),
            "say Ich habe nach zu vielen Werkzeugaufrufen aufgehört."
        );
    }

    #[tokio::test]
    async fn a_declined_call_does_not_run() {
        let script = vec![
            Reply::Tool("echo", "{}"),
            Reply::Text("Gut."),
            Reply::Tool("echo", "{}"),
            Reply::Text("Gut."),
        ];
        let server = fake::llm(script).await;
        let (tools, calls) = tools::tests::fake(0);
        let config = Config {
            llm_url: server.url.clone(),
            confirm: vec![Confirm {
                tool: "ec*".into(),
                commands: Vec::new(),
                argv0: Vec::new(),
                method: None,
                prompt: "Wirklich?".into(),
            }],
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        let (_, events) = turn(&mut chat, "Echo.", false).await;
        assert_eq!(events[1], "confirm Wirklich?");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            chat.history[2]["content"],
            "declined by the user; nothing happened"
        );
        let (_, events) = turn(&mut chat, "Echo.", true).await;
        assert_eq!(events[1..3], ["confirm Wirklich?", "tool echo"]);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_context_error_retries_once_with_half_the_budget() {
        let script = vec![
            Reply::Text("Eins."),
            Reply::Text("Zwei."),
            Reply::Status(400, r#"{"error":{"code":"context_length_exceeded"}}"#),
            Reply::Text("Drei."),
        ];
        let config = Config {
            context_tokens: 400,
            reply_reserve_tokens: 0,
            ..Config::default()
        };
        let (mut chat, server) = chat(script, config).await;
        turn(&mut chat, &"a".repeat(300), true).await;
        turn(&mut chat, &"b".repeat(300), true).await;
        let (report, _) = turn(&mut chat, &"c".repeat(300), true).await;
        assert_eq!(report.outcome, "answered");
        let seen = server.seen.lock().unwrap();
        let users = |i: usize| {
            seen[i]["messages"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|m| m["role"] == "user")
                .count()
        };
        // The budget keeps two turns; the retry keeps one.
        assert_eq!((users(1), users(2), users(3)), (2, 2, 1));
        // The fake counts a quarter of the body as tokens.
        assert!(
            (3.0..4.5).contains(&chat.chars_per_token),
            "{}",
            chat.chars_per_token
        );
    }

    #[tokio::test]
    async fn an_llm_that_is_away_says_so() {
        let (tools, _) = tools::tests::fake(0);
        let config = Config {
            llm_url: "http://127.0.0.1:9/v1".into(),
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        let (report, events) = turn(&mut chat, "Hallo?", true).await;
        assert_eq!(report.outcome, "llm_error");
        assert_eq!(
            events.last().unwrap(),
            "say Das Sprachmodell antwortet nicht."
        );
    }

    #[test]
    fn old_tool_texts_shrink_but_the_last_shader_stays() {
        let call = |id: &str, name: &str, args: &str| json!({"id": id, "type": "function", "function": {"name": name, "arguments": args}});
        let big = format!(r#"{{"name": "a.frag", "data": "{}"}}"#, "x".repeat(600));
        let old = format!(r#"{{"name": "b.frag", "data": "{}"}}"#, "y".repeat(600));
        let mut chat = Chat {
            name: String::new(),
            llm: Llm::new(&Config::default()),
            tools: tools::tests::fake(0).0,
            config: Config::default(),
            system: String::new(),
            history: vec![
                json!({"role": "user", "content": "Shader."}),
                json!({"role": "assistant", "content": "", "tool_calls": [
                    call("1", "put_asset", &old), call("2", "put_asset", &big)]}),
                json!({"role": "tool", "tool_call_id": "1", "content": [
                    {"type": "text", "text": "z".repeat(600)},
                    {"type": "image_url", "image_url": {"url": "data:"}}]}),
                json!({"role": "assistant", "content": "", "tool_calls": [
                    call("3", "put_asset", &old)]}),
            ],
            chars_per_token: 3.0,
            context_tokens: 1000,
            file: None,
            requests: Vec::new(),
            earlier: Vec::new(),
            open: None,
            answer: None,
        };
        chat.shrink();
        let args =
            |i: usize, j: usize| chat.history[i]["tool_calls"][j]["function"]["arguments"].clone();
        assert_eq!(args(1, 0), r#"{"shortened":true}"#);
        assert_eq!(args(1, 1), big.as_str(), "the last put of a.frag");
        assert_eq!(args(3, 0), old.as_str(), "the last put of b.frag");
        let text = chat.history[2]["content"].as_str().unwrap();
        assert_eq!(text.len(), 500 + " [shortened]".len());
    }

    #[test]
    fn the_built_in_documents_have_their_sections() {
        for entry in CONTEXT_DOCS {
            let (name, heading) = entry.split_once('#').unwrap_or((entry, ""));
            let (_, text) = FILES.iter().find(|(f, _)| *f == name).unwrap();
            let part = section(text, heading).unwrap_or_else(|| panic!("{entry}"));
            if !heading.is_empty() {
                assert!(!part[3..].contains("\n## "), "{entry}: one section");
            }
        }
        assert_eq!(section("# A\n\n## B\nb\n## C\nc", "B"), Some("## B\nb"));
        assert_eq!(section("x", "B"), None);
    }

    #[tokio::test]
    async fn the_history_file_keeps_the_requests_for_the_next_prompt() {
        let file = temp("history.json");
        let _ = std::fs::remove_file(&file);
        let config = Config {
            saved_requests: 2,
            ..Config::default()
        };
        let (mut chat, server) = chat(vec![], config.clone()).await;
        chat.file = Some(file.clone());
        for text in ["eins", "zwei", "drei"] {
            turn(&mut chat, text, true).await;
        }
        assert_eq!(load(&file), ["zwei", "drei"]);
        let (tools, _) = tools::tests::fake(0);
        let config = Config {
            llm_url: server.url.clone(),
            prompt_file: String::new(),
            ..config
        };
        let chat = Chat::start(config, "voice-mcp".into(), tools, Some(file))
            .await
            .unwrap();
        assert!(chat.system.ends_with("> zwei\n> drei"));
        assert!(chat.system.contains("owner:voice-mcp"));
        assert!(chat.system.contains("# File docs/api.md\n\n## MCP tools"));
    }
}
