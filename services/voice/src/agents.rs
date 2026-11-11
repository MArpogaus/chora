//! Sub-agents over the Agent Client Protocol (ACP, docs/voice.md section
//! 18): one long-lived agent per project, started with `agent_cmd` in the
//! sandbox of `sandbox_cmd`, spoken to by name. The client is JSON-RPC
//! over the stdin and stdout of the agent, one message per line. Each
//! agent is an actor: a task that owns its process and reads its lines,
//! its prompts and the answer to its question. The core gets the state
//! of each agent for chora-mc, its spoken lines and its questions.

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex, mpsc::Sender},
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout},
    sync::{mpsc, oneshot},
};

use crate::{
    coder::{Group, read_line},
    config::Config,
    core::Input,
};

/// The tools of the LLM.
pub const TOOLS: [&str; 5] = [
    "agent_list",
    "agent_start",
    "agent_say",
    "agent_stop",
    "agent_status",
];

/// The lines of the transcript that mc gets, and their length.
const LOG_LINES: usize = 20;
const LOG_CHARS: usize = 300;
/// The most bytes of the words of the agent between two tool calls; the
/// rest is cut.
const MESSAGE_BYTES: usize = 16 * 1024;
/// The entries of the transcript that an agent keeps, and the most text
/// that one `agent.read` answers.
const RING: usize = 500;
/// The most bytes of one entry, and of the whole ring; the rest is cut
/// or dropped, the oldest first.
const ENTRY_BYTES: usize = 16 * 1024;
const RING_BYTES: usize = 4 * 1024 * 1024;
/// A chat that no `agent.read` touches for this long ends.
const CHAT_TTL: Duration = Duration::from_secs(600);
const CHAT_SWEEP: Duration = Duration::from_secs(30);
const READ_BYTES: usize = 64 * 1024;

/// What an agent tells the core.
#[derive(Debug)]
pub enum Event {
    /// The parameters of `mc agent.set` with the name.
    State(Value),
    /// The agent is gone: mc removes it.
    Gone(String),
    /// A line to speak in the voice of the agent.
    Said { voice: String, text: String },
    /// A question for yes or no, spoken in the voice of the agent; a
    /// dropped sender is a no.
    Ask {
        name: String,
        voice: String,
        prompt: String,
        detail: String,
        physical: bool,
        answer: oneshot::Sender<bool>,
    },
}

/// What the actor of an agent gets.
enum Cmd {
    Prompt(String),
    Cancel,
}

/// The agents; clones share them.
#[derive(Clone)]
pub struct Agents {
    inner: Arc<Mutex<Inner>>,
    /// The directory of `inventory` and of the logs in `agents`.
    data: PathBuf,
    tx: Sender<Input>,
}

struct Inner {
    settings: Settings,
    subs: BTreeMap<String, Sub>,
    /// The number of the next actor.
    next: u64,
    chat_ttl: Duration,
}

/// The settings of the agents, from `Config`.
#[derive(Clone, Default)]
struct Settings {
    /// The command with the sandbox; empty turns the tools off.
    cmd: Vec<String>,
    projects: BTreeMap<String, String>,
    voices: Vec<String>,
    said_chars: usize,
    details: String,
    asks: String,
    ended: String,
    /// German verbs in a question, for `reply_language: de`.
    german: bool,
}

impl Settings {
    fn new(config: &Config) -> Self {
        Self {
            cmd: config.spawn(&config.agent_cmd),
            projects: config.agent_projects.clone(),
            voices: config.agent_voices.clone(),
            said_chars: config.agent_said_chars as usize,
            details: config.phrases.details.clone(),
            asks: config.phrases.agent_asks.clone(),
            ended: config.phrases.agent_ended.clone(),
            german: config.reply_language == "de",
        }
    }
}

/// An entry of the transcript, as `agent.read` answers it.
struct Entry {
    seq: u64,
    role: &'static str,
    text: String,
}

/// An agent as mc and `agent_status` see it.
struct Sub {
    /// The number of its actor: a late event of an older actor of the
    /// same name changes nothing.
    id: u64,
    project: String,
    dir: PathBuf,
    voice: String,
    state: &'static str,
    task: String,
    said: String,
    plan: Vec<String>,
    question: String,
    log: VecDeque<String>,
    entries: VecDeque<Entry>,
    /// The `seq` of the last entry; 0 before the first.
    seq: u64,
    /// The clients that hold a chat with this agent open, with the time
    /// of their last `agent.read`.
    chats: BTreeMap<String, Instant>,
    /// The bytes of text in `entries`.
    bytes: usize,
    started: Instant,
    /// None once its process ended.
    cmds: Option<mpsc::UnboundedSender<Cmd>>,
    pid: Option<u32>,
}

impl Inner {
    /// The first voice of the list that no agent has, else one in turn.
    fn free_voice(&self) -> String {
        let used: Vec<&String> = self.subs.values().map(|s| &s.voice).collect();
        let voices = &self.settings.voices;
        (voices.iter().find(|v| !used.contains(v)))
            .or_else(|| voices.get(self.subs.len() % voices.len().max(1)))
            .cloned()
            .unwrap_or_default()
    }
}

/// A name of 1 to 32 letters, digits, - or _, from a letter.
fn good_name(name: &str) -> bool {
    (1..=32).contains(&name.len())
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
}

impl Sub {
    fn new(
        id: u64,
        project: &str,
        dir: &Path,
        voice: &str,
        cmds: mpsc::UnboundedSender<Cmd>,
    ) -> Self {
        Self {
            id,
            project: project.to_owned(),
            dir: dir.to_owned(),
            voice: voice.to_owned(),
            state: "thinking",
            task: String::new(),
            said: String::new(),
            plan: Vec::new(),
            question: String::new(),
            log: VecDeque::new(),
            entries: VecDeque::new(),
            seq: 0,
            chats: BTreeMap::new(),
            bytes: 0,
            started: Instant::now(),
            cmds: Some(cmds),
            pid: None,
        }
    }

    fn record(&self, name: &str) -> Value {
        json!({"name": name, "state": self.state, "task": self.task, "said": self.said,
            "voice": self.voice, "plan": self.plan, "question": self.question,
            "log": self.log, "seq": self.seq, "epoch": self.id,
            "chat": !self.chats.is_empty()})
    }

    /// Adds an entry to the transcript, hidden and cut to `ENTRY_BYTES`;
    /// the ring drops the oldest beyond `RING` entries and `RING_BYTES`.
    fn entry(&mut self, role: &'static str, text: &str) {
        self.seq += 1;
        let text = cut(&redact(text), ENTRY_BYTES);
        self.bytes += text.len();
        self.entries.push_back(Entry {
            seq: self.seq,
            role,
            text,
        });
        while self.entries.len() > RING || (self.bytes > RING_BYTES && self.entries.len() > 1) {
            if let Some(old) = self.entries.pop_front() {
                self.bytes -= old.text.len();
            }
        }
    }

    /// The entries after `after`, up to `READ_BYTES` of text, the last
    /// `seq` that the answer holds, and the `epoch` of this start of the
    /// agent: a reader that sees another epoch starts its transcript
    /// again. The first entry always comes, so that a reader never
    /// stalls.
    fn read(&self, after: u64) -> Value {
        let mut bytes = 0;
        let mut last = self.seq;
        let mut entries = Vec::new();
        for e in self.entries.iter().filter(|e| e.seq > after) {
            bytes += e.text.len();
            if bytes > READ_BYTES && !entries.is_empty() {
                break;
            }
            last = e.seq;
            entries.push(json!({"seq": e.seq, "role": e.role, "text": e.text}));
        }
        json!({"entries": entries, "seq": last, "epoch": self.id})
    }

    fn note(&mut self, line: &str) {
        let line: String = line.trim().chars().take(LOG_CHARS).collect();
        if line.is_empty() {
            return;
        }
        self.log.push_back(line);
        while self.log.len() > LOG_LINES {
            self.log.pop_front();
        }
    }
}

/// The text with at most `limit` bytes, cut on a char boundary and
/// marked with an ellipsis.
fn cut(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let room = limit - '…'.len_utf8();
    let end = (text.char_indices().map(|(i, _)| i))
        .take_while(|&i| i <= room)
        .last()
        .unwrap_or(0);
    format!("{}…", &text[..end])
}

/// The text without the secrets that `chat::hide` hides: a JSON text
/// by its keys, any other text by the `key: value` and `key=value` of
/// each line.
fn redact(text: &str) -> String {
    if let Ok(mut value) = serde_json::from_str::<Value>(text)
        && (value.is_object() || value.is_array())
    {
        crate::chat::hide(&mut value);
        return value.to_string();
    }
    (text.lines().map(redact_line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn redact_line(line: &str) -> String {
    let Some(at) = line.find([':', '=']) else {
        return line.to_owned();
    };
    let key = (line[..at].split_whitespace().last())
        .unwrap_or_default()
        .trim_matches(['-', '"', '\'']);
    let mut probe = json!({ key: "" });
    crate::chat::hide(&mut probe);
    if probe[key] == "(hidden)" {
        format!("{} (hidden)", &line[..=at])
    } else {
        line.to_owned()
    }
}

/// A name with a capital first letter, as spoken and as mc shows it.
pub fn title(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The first sentence of a long text and `details`; a short text as it
/// is.
pub fn summary(text: &str, limit: usize, details: &str) -> String {
    let text = text.trim();
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let end = text
        .char_indices()
        .find(|&(i, c)| {
            c == '\n' || (".!?".contains(c) && text[i + 1..].starts_with(char::is_whitespace))
        })
        .map_or(text.len(), |(i, c)| i + c.len_utf8());
    let first: String = text[..end].chars().take(limit).collect();
    let first = first.trim();
    format!("{first} {details}").trim().to_owned()
}

impl Agents {
    pub fn new(config: &Config, data: PathBuf, tx: Sender<Input>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                settings: Settings::new(config),
                subs: BTreeMap::new(),
                next: 0,
                chat_ttl: CHAT_TTL,
            })),
            data,
            tx,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("no panic with the lock")
    }

    /// New settings; running agents go on.
    pub fn configure(&self, config: &Config) {
        self.lock().settings = Settings::new(config);
    }

    /// The function specs for the LLM; none without `agent_cmd`.
    pub fn specs(&self) -> Vec<Value> {
        if self.lock().settings.cmd.is_empty() {
            return Vec::new();
        }
        let function = |name: &str, description: &str, required: &[&str], props: Value| {
            json!({"type": "function", "function": {"name": name, "description": description,
                "parameters": {"type": "object", "required": required, "properties": props}}})
        };
        let name = json!({"type": "string", "description": "the name of the agent"});
        vec![
            function(
                "agent_list",
                "The projects that can get an agent, and the agents that run.",
                &[],
                json!({}),
            ),
            function(
                "agent_start",
                "Start a coding agent for a project. It runs until agent_stop and keeps its \
                 conversation; the user can talk to it directly by its name.",
                &["project"],
                json!({"project": {"type": "string", "description": "a project of agent_list"},
                    "name": {"type": "string", "description": "a short name; the project without it"},
                    "voice": {"type": "string", "description": "a voice of the TTS"}}),
            ),
            function(
                "agent_say",
                "Send a request to an agent. It works in the background; its answer is \
                 spoken in its own voice.",
                &["name", "text"],
                json!({"name": name, "text": {"type": "string", "description": "the request in full"}}),
            ),
            function(
                "agent_stop",
                "Stop an agent and its process.",
                &["name"],
                json!({"name": name}),
            ),
            function(
                "agent_status",
                "The state, the task, the plan and the last words of the agents, or of one.",
                &[],
                json!({"name": name}),
            ),
        ]
    }

    /// The project folders by name, the roots of `read`.
    pub fn roots(&self) -> BTreeMap<String, PathBuf> {
        let settings = self.lock().settings.clone();
        self.projects(&settings)
    }

    /// The projects: those of `agent_projects`, then the directories of
    /// the inventory.
    fn projects(&self, settings: &Settings) -> BTreeMap<String, PathBuf> {
        let mut projects: BTreeMap<String, PathBuf> =
            std::fs::read_dir(self.data.join("inventory"))
                .into_iter()
                .flatten()
                .flatten()
                .filter(|e| e.path().is_dir())
                .filter_map(|e| Some((e.file_name().into_string().ok()?, e.path())))
                .collect();
        for (name, dir) in &settings.projects {
            projects.insert(name.clone(), crate::expand(dir));
        }
        projects
    }

    /// Runs one of `TOOLS` and answers the text for the LLM.
    pub fn call(&self, tool: &str, args: &Value) -> Value {
        self.expire_chats();
        if let Some(answer) = self.chat_call(tool, args) {
            return answer;
        }
        let arg = |key: &str| args[key].as_str().map(str::trim).filter(|s| !s.is_empty());
        match (tool, arg("name")) {
            ("agent_list", _) => self.list(),
            ("agent_start", name) => match arg("project") {
                Some(project) => self.start(project, name, arg("voice")),
                None => "error: agent_start needs a project".into(),
            },
            ("agent_say", Some(name)) => match arg("text") {
                Some(text) => self.prompt(name, text),
                None => "error: agent_say needs a text".into(),
            },
            ("agent_stop", Some(name)) => self.stop(name),
            ("agent_status", name) => self.status(name),
            _ => format!("error: {tool} needs a name"),
        }
        .into()
    }

    /// The chat methods of the client, which never reach the LLM:
    /// `agent.read`, `agent.cancel`, `chat.open`, `chat.close` and
    /// `chat.gone` (a client is gone; no `caller` is a new session).
    /// A failure is `{"error": text}`.
    fn chat_call(&self, method: &str, args: &Value) -> Option<Value> {
        let name = args["name"].as_str().unwrap_or_default().to_lowercase();
        let caller = args["caller"].as_str();
        let answer = match method {
            "agent.read" => {
                let after = args["after"].as_u64().unwrap_or(0);
                self.read(&name, after, caller)
            }
            "agent.cancel" => self.cancel_running(&name).map(Value::from),
            "chat.open" | "chat.close" => match caller {
                Some(caller) if method == "chat.open" => self.chat_open(&name, caller),
                Some(caller) => self.chat_close(&name, caller),
                None => Err(format!("{method} needs a caller")),
            }
            .map(|()| json!({})),
            "chat.gone" => {
                self.chat_gone(caller);
                Ok(json!({}))
            }
            _ => return None,
        };
        Some(answer.unwrap_or_else(|error| json!({"error": error})))
    }

    /// The transcript of an agent after `after`.
    /// A read keeps the chat of `caller` alive.
    fn read(&self, name: &str, after: u64, caller: Option<&str>) -> Result<Value, String> {
        let mut inner = self.lock();
        let sub = (inner.subs.get_mut(name)).ok_or(format!("no agent {name}"))?;
        if let Some(read) = caller.and_then(|c| sub.chats.get_mut(c)) {
            *read = Instant::now();
        }
        Ok(sub.read(after))
    }

    /// Ends the chats that no read touched for the time to live.
    fn expire_chats(&self) {
        let mut inner = self.lock();
        let ttl = inner.chat_ttl;
        for (name, sub) in &mut inner.subs {
            let before = sub.chats.len();
            sub.chats.retain(|_, read| read.elapsed() < ttl);
            if sub.chats.len() != before {
                let _ = self.tx.send(Input::Agent(Event::State(sub.record(name))));
            }
        }
    }

    /// Cancels the prompt of an agent that runs.
    fn cancel_running(&self, name: &str) -> Result<String, String> {
        let name = name.to_lowercase();
        let runs = (self.lock().subs.get(&name)).is_some_and(|s| s.cmds.is_some());
        if runs {
            Ok(self.cancel(&name))
        } else {
            Err(format!("no agent {name} runs"))
        }
    }

    /// Marks a chat of `caller` with an agent as open.
    fn chat_open(&self, name: &str, caller: &str) -> Result<(), String> {
        self.chats(name, |chats| {
            chats.insert(caller.to_owned(), Instant::now());
            Ok(())
        })
    }

    /// Ends the chat that `caller` opened.
    fn chat_close(&self, name: &str, caller: &str) -> Result<(), String> {
        self.chats(name, |chats| {
            if chats.remove(caller).is_some() {
                Ok(())
            } else {
                Err(format!("{caller} has no chat open with {name}"))
            }
        })
    }

    /// Ends the chats of `caller`, or of all callers.
    fn chat_gone(&self, caller: Option<&str>) {
        let mut inner = self.lock();
        for (name, sub) in &mut inner.subs {
            let before = sub.chats.len();
            match caller {
                Some(c) => sub.chats.remove(c).is_some(),
                None => !std::mem::take(&mut sub.chats).is_empty(),
            };
            if sub.chats.len() != before {
                let _ = self.tx.send(Input::Agent(Event::State(sub.record(name))));
            }
        }
    }

    fn chats(
        &self,
        name: &str,
        change: impl FnOnce(&mut BTreeMap<String, Instant>) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut inner = self.lock();
        let sub = (inner.subs.get_mut(name)).ok_or(format!("no agent {name}"))?;
        let before = sub.chats.len();
        change(&mut sub.chats)?;
        if sub.chats.len() != before {
            let _ = self.tx.send(Input::Agent(Event::State(sub.record(name))));
        }
        Ok(())
    }

    fn list(&self) -> String {
        let inner = self.lock();
        let projects: Vec<String> = self.projects(&inner.settings).into_keys().collect();
        let agents: Vec<String> = (inner.subs.iter())
            .map(|(n, s)| format!("{n} ({}, {})", s.project, s.state))
            .collect();
        format!(
            "projects: {}; agents: {}",
            if projects.is_empty() {
                "none".into()
            } else {
                projects.join(", ")
            },
            if agents.is_empty() {
                "none".into()
            } else {
                agents.join(", ")
            }
        )
    }

    /// Starts an agent for `project` in its directory.
    pub fn start(&self, project: &str, name: Option<&str>, voice: Option<&str>) -> String {
        let mut inner = self.lock();
        if inner.settings.cmd.is_empty() {
            return "error: no agent_cmd in the settings".into();
        }
        // The ASR and the LLM may capitalize the name.
        let found = (self.projects(&inner.settings).into_iter())
            .find(|(p, _)| p.eq_ignore_ascii_case(project));
        let Some((project, dir)) = found else {
            return format!("error: no project {project}; agent_list names them");
        };
        let project = project.as_str();
        let name = name.unwrap_or(project).to_lowercase();
        if !good_name(&name) {
            return format!("error: {name}: a name has 1 to 32 letters, digits, - or _");
        }
        if inner.subs.get(&name).is_some_and(|s| s.cmds.is_some()) {
            return format!("{name} runs already");
        }
        let voice = voice.map_or_else(|| inner.free_voice(), str::to_owned);
        let place = |a: &String| {
            a.replace("{dir}", &dir.to_string_lossy())
                .replace("{project}", project)
        };
        let command: Vec<String> = inner.settings.cmd.iter().map(place).collect();
        let log = self.data.join("agents").join(format!("{name}.log"));
        let (cmds, rx) = tokio::sync::mpsc::unbounded_channel();
        inner.next += 1;
        let id = inner.next;
        let sub = Sub::new(id, project, &dir, &voice, cmds);
        let _ = self.tx.send(Input::Agent(Event::State(sub.record(&name))));
        inner.subs.insert(name.clone(), sub);
        crate::log::info(&format!("agent {name}: {command:?} in {}", dir.display()));
        let actor = Actor {
            agents: self.clone(),
            name: name.clone(),
            id,
        };
        tokio::spawn(actor.run(command, dir.clone(), log.clone(), rx));
        format!(
            "{name} starts in {} with the voice {voice}; its log: {}",
            dir.display(),
            log.display()
        )
    }

    /// Sends a prompt to an agent; a prompt waits while one runs.
    pub fn prompt(&self, name: &str, text: &str) -> String {
        let inner = self.lock();
        let name = name.to_lowercase();
        match inner.subs.get(&name) {
            None => format!("error: no agent {name}; agent_list names them"),
            Some(Sub { cmds: None, .. }) => {
                format!("{name} has ended; agent_start starts it again")
            }
            Some(Sub {
                cmds: Some(cmds),
                state,
                ..
            }) => {
                let _ = cmds.send(Cmd::Prompt(text.to_owned()));
                if *state == "working" {
                    format!("{name} works; the request waits for its turn")
                } else {
                    format!("{name} got the request")
                }
            }
        }
    }

    /// Cancels the prompt of an agent and the prompts that wait.
    pub fn cancel(&self, name: &str) -> String {
        let name = name.to_lowercase();
        match self.lock().subs.get(&name).and_then(|s| s.cmds.as_ref()) {
            Some(cmds) => {
                let _ = cmds.send(Cmd::Cancel);
                format!("{name} stops its request")
            }
            None => format!("no agent {name} runs"),
        }
    }

    /// Stops an agent: its actor ends its process group.
    pub fn stop(&self, name: &str) -> String {
        let name = name.to_lowercase();
        if self.lock().subs.remove(&name).is_none() {
            return format!("error: no agent {name}");
        }
        let _ = self.tx.send(Input::Agent(Event::Gone(name.clone())));
        format!("{name} stops")
    }

    fn status(&self, name: Option<&str>) -> String {
        let inner = self.lock();
        let lines: Vec<String> = (inner.subs.iter())
            .filter(|(n, _)| name.is_none_or(|w| w.eq_ignore_ascii_case(n)))
            .map(|(n, s)| {
                let plan = s.plan.join(" | ");
                let fields = [
                    ("task", &s.task),
                    ("question", &s.question),
                    ("said", &s.said),
                    ("plan", &plan),
                ];
                let mut parts = vec![format!(
                    "{n}: {} in {} for {} min",
                    s.state,
                    s.dir.display(),
                    s.started.elapsed().as_secs() / 60
                )];
                (fields.iter())
                    .filter(|(_, v)| !v.is_empty())
                    .for_each(|(k, v)| parts.push(format!("{k}: {v}")));
                parts.join("; ")
            })
            .collect();
        if lines.is_empty() {
            return "no agent runs".into();
        }
        lines.join("\n")
    }

    /// Stops all agents and ends their process groups and those of the
    /// coder (`coder::end_groups`): at the exit of the voice agent.
    pub fn shutdown(&self) {
        let pids: Vec<u32> = {
            let mut inner = self.lock();
            let subs = std::mem::take(&mut inner.subs);
            subs.into_values().filter_map(|s| s.pid).collect()
        };
        crate::coder::end_groups(&pids);
    }

    /// Changes the agent of actor `id` and reports it.
    fn update(&self, name: &str, id: u64, change: impl FnOnce(&mut Sub)) {
        let mut inner = self.lock();
        let Some(sub) = inner.subs.get_mut(name).filter(|s| s.id == id) else {
            return;
        };
        change(sub);
        let _ = self.tx.send(Input::Agent(Event::State(sub.record(name))));
    }

    fn voice(&self, name: &str) -> String {
        self.lock()
            .subs
            .get(name)
            .map(|s| s.voice.clone())
            .unwrap_or_default()
    }

    /// Speaks the end of a turn in the voice of the agent, short.
    fn speak(&self, name: &str, text: &str) {
        self.expire_chats();
        let (limit, details) = {
            let inner = self.lock();
            (inner.settings.said_chars, inner.settings.details.clone())
        };
        let text = summary(text, limit, &details);
        let silent = self
            .lock()
            .subs
            .get(name)
            .is_some_and(|s| !s.chats.is_empty());
        if !text.is_empty() && !silent {
            let voice = self.voice(name);
            let _ = self.tx.send(Input::Agent(Event::Said { voice, text }));
        }
    }
}

/// A question of the agent that waits for the user.
struct Permission {
    id: Value,
    options: Vec<Value>,
    /// The core drops its sender after `agent_permission_s` from when
    /// it spoke the question, or when it drops the question.
    answer: oneshot::Receiver<bool>,
}

/// The JSON-RPC side of the connection: requests out, and the log.
struct Rpc {
    stdin: ChildStdin,
    log: tokio::fs::File,
    next: u64,
    /// The method of each request that waits for its response.
    pending: HashMap<u64, &'static str>,
}

impl Rpc {
    async fn send(&mut self, message: &Value) -> Result<(), String> {
        let mut line = message.to_string();
        let _ = self.log.write_all(format!("> {line}\n").as_bytes()).await;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("the agent's stdin: {e}"))?;
        self.stdin.flush().await.map_err(|e| e.to_string())
    }

    async fn request(&mut self, method: &'static str, params: Value) -> Result<(), String> {
        self.next += 1;
        self.pending.insert(self.next, method);
        let id = self.next;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .await
    }

    async fn respond(&mut self, id: Value, result: Value) -> Result<(), String> {
        self.send(&json!({"jsonrpc": "2.0", "id": id, "result": result}))
            .await
    }
}

/// The spoken question of a permission request: `asks`, or "Always
/// allow?" when the only allowing option is `allow_always`, so that a
/// yes never allows always unheard.
fn question(asks: &str, name: &str, what: &str, options: &[Value], german: bool) -> String {
    let has = |kind: &str| options.iter().any(|o| o["kind"] == kind);
    if !has("allow_once") && has("allow_always") {
        // ponytail: English and German, as the verbs of `asked`.
        return if german {
            format!("{} möchte {what}. Immer erlauben?", title(name))
        } else {
            format!("{} wants to {what}. Always allow?", title(name))
        };
    }
    asks.replace("{name}", &title(name)).replace("{what}", what)
}

/// The answer to a permission request: the first option of an allowing
/// kind for yes, of a rejecting kind for no; cancelled without one. Yes
/// takes `allow_always` only without `allow_once`, after `question`
/// asked for it.
fn outcome(options: &[Value], yes: bool) -> Value {
    let kinds = if yes {
        ["allow_once", "allow_always"]
    } else {
        ["reject_once", "reject_always"]
    };
    kinds
        .iter()
        .find_map(|k| options.iter().find(|o| o["kind"] == *k))
        .map_or_else(
            || json!({"outcome": {"outcome": "cancelled"}}),
            |o| json!({"outcome": {"outcome": "selected", "optionId": o["optionId"]}}),
        )
}

/// Whether a permission request takes a key or a button of the HUD, as
/// a security question of the reply does: all but a read, a search and
/// a thought, so a command, a write, a move, a delete, a fetch and a
/// kind that is unknown; and any request without an `allow_once`
/// option, which a plain yes would turn into `allow_always`.
fn is_physical(tool: &Value, options: &[Value]) -> bool {
    let once = options.iter().any(|o| o["kind"] == "allow_once");
    !once || !matches!(tool["kind"].as_str(), Some("read" | "search" | "think"))
}

/// What a permission request asks, in full, for the screen.
fn detail(tool: &Value) -> String {
    let title = tool["title"].as_str().unwrap_or_default().trim();
    let mut input = tool["rawInput"].clone();
    crate::chat::hide(&mut input);
    if input.is_null() {
        title.to_owned()
    } else {
        format!("{title}\n{input}")
    }
}

/// What a permission request asks, short enough to speak: the title of
/// its tool call, a path as its file name for an edit, a read, a delete
/// and a move, and the kind as a verb.
fn asked(tool: &Value, german: bool) -> String {
    let title = tool["title"].as_str().unwrap_or_default().trim();
    let short = matches!(
        tool["kind"].as_str(),
        Some("edit" | "read" | "delete" | "move")
    );
    let title = if short && title.starts_with('/') {
        Path::new(title)
            .file_name()
            .map_or(title.into(), |f| f.to_string_lossy())
    } else {
        title.into()
    };
    let title: String = title.chars().take(80).collect();
    // ponytail: English and German verbs, as the default phrases; a
    // phrase per kind in the settings when another language needs it.
    let (en, de) = match tool["kind"].as_str() {
        Some("edit") => ("edit", "ändern"),
        Some("delete") => ("delete", "löschen"),
        Some("move") => ("move", "verschieben"),
        Some("execute") => ("run", "ausführen"),
        Some("fetch") => ("fetch", "abrufen"),
        Some("read") => ("read", "lesen"),
        _ => ("", ""),
    };
    match (title.is_empty(), en.is_empty(), german) {
        (true, true, false) => "do something".into(),
        (true, true, true) => "etwas tun".into(),
        (true, false, false) => en.into(),
        (true, false, true) => de.into(),
        (false, true, _) => title,
        (false, false, false) => format!("{en} {title}"),
        (false, false, true) => format!("{title} {de}"),
    }
}

/// A plan entry as a line: done, in progress or waiting.
fn plan_line(entry: &Value) -> String {
    let sign = match entry["status"].as_str() {
        Some("completed") => "✓",
        Some("in_progress") => "●",
        _ => "○",
    };
    format!("{sign} {}", entry["content"].as_str().unwrap_or_default())
}

/// The task of one agent.
struct Actor {
    agents: Agents,
    name: String,
    id: u64,
}

/// The state of the conversation of the actor.
#[derive(Default)]
struct Session {
    id: Option<String>,
    prompting: bool,
    queue: VecDeque<String>,
    /// The words of the agent since its last tool call.
    message: String,
    /// The last message of the turn.
    last: String,
    question: Option<Permission>,
}

impl Actor {
    async fn run(
        self,
        command: Vec<String>,
        dir: PathBuf,
        log: PathBuf,
        cmds: mpsc::UnboundedReceiver<Cmd>,
    ) {
        let end = self.drive(&command, &dir, &log, cmds).await;
        let (name, id) = (&self.name, self.id);
        crate::log::info(&format!("agent {name}: ended: {end:?}"));
        // Stopped: it is gone already.
        if end == Ok(true) {
            return;
        }
        let why = end.err().unwrap_or_else(|| "the agent ended".into());
        self.agents.update(name, id, |s| {
            s.cmds = None;
            s.pid = None;
            s.state = "blocked";
            s.question.clear();
            s.note(&format!("✕ {why}"));
            s.entry("notice", &why);
            s.said = why;
        });
        let ended = self.agents.lock().settings.ended.clone();
        self.agents
            .speak(name, &ended.replace("{name}", &title(name)));
    }

    /// Runs the agent until it ends (false) or the agent stops (true).
    async fn drive(
        &self,
        command: &[String],
        dir: &Path,
        log: &Path,
        mut cmds: mpsc::UnboundedReceiver<Cmd>,
    ) -> Result<bool, String> {
        let (mut child, mut rpc) = spawn(command, dir, log).await?;
        let _group = Group::new(child.id());
        let pid = child.id();
        self.agents.update(&self.name, self.id, |s| s.pid = pid);
        let mut lines = read_lines(child.stdout.take().expect("piped"));
        let mut session = Session::default();
        let mut sweep = tokio::time::interval(CHAT_SWEEP);
        loop {
            tokio::select! {
                _ = sweep.tick() => self.agents.expire_chats(),
                line = lines.recv() => {
                    let Some(line) = line.transpose()? else { return Ok(false) };
                    self.line(&mut rpc, &mut session, &line, dir).await?;
                }
                cmd = cmds.recv() => {
                    let Some(cmd) = cmd else { return Ok(true) };
                    command_of_user(&mut rpc, &mut session, cmd).await?;
                }
                yes = decided(&mut session.question) => self.answered(&mut rpc, &mut session, yes).await?,
            }
            self.next_prompt(&mut rpc, &mut session).await?;
        }
    }

    /// A line of the agent, into the log, and its message.
    async fn line(
        &self,
        rpc: &mut Rpc,
        session: &mut Session,
        line: &[u8],
        dir: &Path,
    ) -> Result<(), String> {
        let _ = rpc.log.write_all(b"< ").await;
        let _ = rpc.log.write_all(line).await;
        match serde_json::from_slice::<Value>(line) {
            Ok(message) => self.message(rpc, session, message, dir).await,
            Err(_) => Ok(()),
        }
    }

    /// The answer of the user to the question.
    async fn answered(
        &self,
        rpc: &mut Rpc,
        session: &mut Session,
        yes: bool,
    ) -> Result<(), String> {
        let question = session.question.take().expect("a question");
        rpc.respond(question.id, outcome(&question.options, yes))
            .await?;
        self.agents.update(&self.name, self.id, |s| {
            s.question.clear();
            s.state = "working";
            s.note(if yes { "› ja" } else { "› nein" });
        });
        Ok(())
    }

    /// Sends the next prompt that waits, once the session is there and
    /// no prompt runs.
    async fn next_prompt(&self, rpc: &mut Rpc, session: &mut Session) -> Result<(), String> {
        if session.prompting || session.question.is_some() {
            return Ok(());
        }
        let (Some(id), Some(text)) = (session.id.clone(), session.queue.pop_front()) else {
            return Ok(());
        };
        rpc.request(
            "session/prompt",
            json!({"sessionId": id, "prompt": [{"type": "text", "text": text}]}),
        )
        .await?;
        session.prompting = true;
        session.last.clear();
        self.agents.update(&self.name, self.id, |s| {
            s.state = "working";
            s.note(&format!("› {text}"));
            s.entry("user", &text);
            s.task = text;
            s.plan.clear();
        });
        Ok(())
    }

    /// One message of the agent: a response, a request or a notification.
    async fn message(
        &self,
        rpc: &mut Rpc,
        session: &mut Session,
        message: Value,
        dir: &Path,
    ) -> Result<(), String> {
        let Some(method) = message["method"].as_str() else {
            return self.response(rpc, session, &message, dir).await;
        };
        let params = &message["params"];
        match (method, message.get("id").cloned()) {
            ("session/update", None) => self.update(session, &params["update"]),
            ("session/request_permission", Some(rpc_id)) => {
                self.permission(rpc, session, rpc_id, params).await?;
            }
            // We offer no file system and no terminal.
            (_, Some(rpc_id)) => {
                rpc.send(&json!({"jsonrpc": "2.0", "id": rpc_id,
                    "error": {"code": -32601, "message": format!("no method {method}")}}))
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    /// The response to a request of ours.
    async fn response(
        &self,
        rpc: &mut Rpc,
        session: &mut Session,
        message: &Value,
        dir: &Path,
    ) -> Result<(), String> {
        let method = message["id"]
            .as_u64()
            .and_then(|i| rpc.pending.remove(&i))
            .unwrap_or_default();
        let error = message.get("error").map(|e| {
            e["message"]
                .as_str()
                .map_or_else(|| e.to_string(), str::to_owned)
        });
        match (method, error) {
            ("initialize" | "session/new", Some(e)) => Err(format!("{method}: {e}")),
            ("initialize", None) => {
                rpc.request(
                    "session/new",
                    json!({"cwd": dir.to_string_lossy(), "mcpServers": []}),
                )
                .await
            }
            ("session/new", None) => {
                session.id = message["result"]["sessionId"].as_str().map(str::to_owned);
                self.agents
                    .update(&self.name, self.id, |s| s.state = "idle");
                Ok(())
            }
            ("session/prompt", error) => {
                let reason = message["result"]["stopReason"].as_str().unwrap_or_default();
                self.ended(session, error, reason);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// The end of a prompt: done, cancelled or blocked; the last words
    /// are spoken.
    fn ended(&self, session: &mut Session, error: Option<String>, reason: &str) {
        session.prompting = false;
        self.flush(session);
        let (state, said) = match (error, reason) {
            (Some(e), _) => ("blocked", e),
            (None, "end_turn") => ("done", session.last.clone()),
            (None, "cancelled") => ("idle", String::new()),
            (None, reason) => ("blocked", format!("{}: {reason}", title(&self.name))),
        };
        let state = if session.queue.is_empty() {
            state
        } else {
            "working"
        };
        let keep = said.clone();
        self.agents.update(&self.name, self.id, |s| {
            s.state = state;
            if !keep.is_empty() {
                s.said = keep;
            }
        });
        if !said.is_empty() {
            self.agents.speak(&self.name, &said);
        }
    }

    /// A question of the agent: it waits for the user, who hears it in
    /// the voice of the agent.
    async fn permission(
        &self,
        rpc: &mut Rpc,
        session: &mut Session,
        rpc_id: Value,
        params: &Value,
    ) -> Result<(), String> {
        let (name, id) = (self.name.as_str(), self.id);
        self.flush(session);
        if let Some(old) = session.question.take() {
            rpc.respond(old.id, json!({"outcome": {"outcome": "cancelled"}}))
                .await?;
        }
        let (asks, german) = {
            let settings = &self.agents.lock().settings;
            (settings.asks.clone(), settings.german)
        };
        let what = asked(&params["toolCall"], german);
        let options = params["options"].as_array().cloned().unwrap_or_default();
        let prompt = question(&asks, name, &what, &options, german);
        let physical = is_physical(&params["toolCall"], &options);
        let detail = detail(&params["toolCall"]);
        if crate::chat::too_long(&detail) {
            rpc.respond(rpc_id, outcome(&options, false)).await?;
            let told = if german {
                format!(
                    "{} wollte etwas tun, das zu lang ist, um es zu zeigen. Abgelehnt.",
                    title(name)
                )
            } else {
                format!(
                    "{} asked for something too long to show. Rejected.",
                    title(name)
                )
            };
            self.agents.update(name, id, |s| {
                s.note(&format!("! {told}"));
                s.entry("notice", &told);
            });
            self.agents.speak(name, &told);
            return Ok(());
        }
        let (answer, rx) = oneshot::channel();
        session.question = Some(Permission {
            id: rpc_id,
            options,
            answer: rx,
        });
        self.agents.update(name, id, |s| {
            s.state = "question";
            s.note(&format!("? {what}"));
            s.entry("question", &what);
            s.question = what;
        });
        let voice = self.agents.voice(name);
        let _ = self.agents.tx.send(Input::Agent(Event::Ask {
            name: name.to_owned(),
            voice,
            prompt,
            detail,
            physical,
            answer,
        }));
        Ok(())
    }

    /// A `session/update`: the words, the tool calls and the plan.
    fn update(&self, session: &mut Session, update: &Value) {
        let (name, id) = (self.name.as_str(), self.id);
        match update["sessionUpdate"].as_str() {
            Some("agent_message_chunk") => {
                if let Some(text) = update["content"]["text"].as_str() {
                    let room = MESSAGE_BYTES.saturating_sub(session.message.len());
                    let end = (text.char_indices().map(|(i, _)| i))
                        .chain([text.len()])
                        .take_while(|&i| i <= room)
                        .last()
                        .unwrap_or(0);
                    session.message.push_str(&text[..end]);
                }
            }
            Some("tool_call") => {
                self.flush(session);
                let title = update["title"].as_str().unwrap_or_default().to_owned();
                self.agents.update(name, id, |s| {
                    s.note(&format!("⚙ {title}"));
                    s.entry("tool", &title);
                });
            }
            Some("plan") => {
                let plan: Vec<String> = (update["entries"].as_array().into_iter().flatten())
                    .map(plan_line)
                    .collect();
                self.agents.update(name, id, |s| {
                    if s.plan != plan {
                        s.entry("plan", &plan.join("\n"));
                    }
                    s.plan = plan;
                });
            }
            _ => {}
        }
    }

    /// The words since the last tool call go to the transcript.
    fn flush(&self, session: &mut Session) {
        let text = std::mem::take(&mut session.message);
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        text.clone_into(&mut session.last);
        self.agents.update(&self.name, self.id, |s| {
            s.note(text);
            s.entry("agent", text);
            text.clone_into(&mut s.said);
        });
    }
}

/// The answer to the question that waits: yes, or no when the core
/// dropped it, also after its timeout. Without a question, never.
async fn decided(question: &mut Option<Permission>) -> bool {
    match question {
        Some(q) => matches!((&mut q.answer).await, Ok(true)),
        None => std::future::pending().await,
    }
}

/// The lines of the agent from a task of their own, so that a select
/// that takes another branch loses no part of a line; the channel ends
/// after the last line or an error.
fn read_lines(stdout: ChildStdout) -> mpsc::Receiver<Result<Vec<u8>, String>> {
    let (tx, rx) = mpsc::channel(16);
    tokio::spawn(async move {
        let mut out = BufReader::new(stdout);
        while let Some(line) = read_line(&mut out).await.transpose() {
            let failed = line.is_err();
            if tx.send(line).await.is_err() || failed {
                return;
            }
        }
    });
    rx
}

/// Starts the agent in its own process group, so that a stop ends the
/// sandbox and its children, with stderr and the messages in `log`;
/// sends `initialize`.
async fn spawn(
    command: &[String],
    dir: &Path,
    log: &Path,
) -> Result<(tokio::process::Child, Rpc), String> {
    if let Some(parent) = log.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .map_err(|e| format!("{}: {e}", log.display()))?;
    let err = file.try_clone().map_err(|e| e.to_string())?;
    let (program, args) = command.split_first().ok_or("agent_cmd is empty")?;
    let mut child = tokio::process::Command::new(program)
        .args(args)
        .current_dir(dir)
        .process_group(0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(err)
        .spawn()
        .map_err(|e| format!("{program}: {e}"))?;
    let mut rpc = Rpc {
        stdin: child.stdin.take().expect("piped"),
        log: tokio::fs::File::from_std(file),
        next: 0,
        pending: HashMap::new(),
    };
    rpc.request(
        "initialize",
        json!({"protocolVersion": 1,
            "clientCapabilities": {"fs": {"readTextFile": false, "writeTextFile": false},
                "terminal": false},
            "clientInfo": {"name": "chora-voice", "version": env!("CARGO_PKG_VERSION")}}),
    )
    .await?;
    Ok((child, rpc))
}

/// A prompt waits; a cancel drops the prompts that wait, the question,
/// and the prompt that runs.
async fn command_of_user(rpc: &mut Rpc, session: &mut Session, cmd: Cmd) -> Result<(), String> {
    match cmd {
        Cmd::Prompt(text) => session.queue.push_back(text),
        Cmd::Cancel => {
            session.queue.clear();
            if let Some(q) = session.question.take() {
                rpc.respond(q.id, json!({"outcome": {"outcome": "cancelled"}}))
                    .await?;
            }
            if let (true, Some(id)) = (session.prompting, &session.id) {
                rpc.send(&json!({"jsonrpc": "2.0", "method": "session/cancel",
                    "params": {"sessionId": id}}))
                    .await?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, time::Duration};

    use super::*;

    /// A permission request takes a key unless it only reads, searches
    /// or thinks, and unless its only allowing option is `allow_once`.
    #[test]
    fn a_permission_request_is_physical_unless_it_only_looks() {
        let once = [
            json!({"kind": "allow_once"}),
            json!({"kind": "reject_once"}),
        ];
        for kind in ["execute", "edit", "delete", "move", "fetch", "other"] {
            assert!(is_physical(&json!({"kind": kind}), &once), "{kind}");
        }
        assert!(is_physical(&json!({}), &once), "no kind");
        for kind in ["read", "search", "think"] {
            assert!(!is_physical(&json!({"kind": kind}), &once), "{kind}");
        }
        let always = [json!({"kind": "allow_always"})];
        assert!(is_physical(&json!({"kind": "read"}), &always));
    }

    /// The title of a command is never cut to a file name, and the
    /// screen gets the whole title and the raw input without secrets.
    #[test]
    fn a_command_keeps_its_whole_title_for_the_user() {
        let tool = json!({
            "kind": "execute",
            "title": "/bin/true && rm -rf /home/user/x",
            "rawInput": {"command": "ls", "api_key": "s3cret"}
        });
        let said = asked(&tool, false);
        assert!(said.contains("rm -rf /home/user/x"), "{said}");
        let long = json!({"kind": "execute", "title": "x".repeat(200)});
        assert_eq!(detail(&long).matches('x').count(), 200);
        let shown = detail(&tool);
        assert!(
            shown.contains("/bin/true && rm -rf /home/user/x"),
            "{shown}"
        );
        assert!(shown.contains("\"command\":\"ls\""), "{shown}");
        assert!(!shown.contains("s3cret"), "{shown}");
        let edit = json!({"kind": "edit", "title": "/home/user/notes.txt"});
        assert_eq!(asked(&edit, false), "edit notes.txt");
        let unknown = json!({"title": "/bin/sh"});
        assert_eq!(asked(&unknown, false), "/bin/sh");
    }

    #[test]
    fn a_long_answer_speaks_its_first_sentence() {
        assert_eq!(summary(" Kurz. ", 20, "Mehr."), "Kurz.");
        let long = "Erledigt: alles gut. Und dann noch sehr viel mehr Text danach.";
        assert_eq!(summary(long, 20, "Mehr."), "Erledigt: alles gut. Mehr.");
        assert_eq!(summary("v1.2 ist da und lang", 5, "X"), "v1.2 X");
    }

    #[test]
    fn a_question_is_short_to_speak() {
        let ask = |tool| asked(&tool, true);
        let edit = json!({"title": "/a/b/notes.txt", "kind": "edit"});
        assert_eq!(asked(&edit, false), "edit notes.txt");
        assert_eq!(asked(&json!({}), false), "do something");
        assert_eq!(
            ask(json!({"title": "/a/b/notes.txt", "kind": "edit"})),
            "notes.txt ändern"
        );
        assert_eq!(
            ask(json!({"title": "cargo test", "kind": "execute"})),
            "cargo test ausführen"
        );
        assert_eq!(ask(json!({"kind": "think"})), "etwas tun");
    }

    #[test]
    fn a_yes_allows_once_and_a_no_rejects() {
        let options = [
            json!({"optionId": "a", "kind": "allow_always"}),
            json!({"optionId": "o", "kind": "allow_once"}),
            json!({"optionId": "r", "kind": "reject_once"}),
        ];
        assert_eq!(outcome(&options, true)["outcome"]["optionId"], "o");
        assert_eq!(outcome(&options, false)["outcome"]["optionId"], "r");
        assert_eq!(
            outcome(&options[..2], false)["outcome"]["outcome"],
            "cancelled"
        );
    }

    /// At the exit, the group of an agent ends also when its leader has
    /// ended already: the other processes of the group count. Not through
    /// `shutdown`, which would end the agents of the other tests.
    #[test]
    fn shutdown_ends_the_group_without_its_leader() {
        use std::os::unix::process::CommandExt;
        let mut leader = std::process::Command::new("sh")
            .args(["-c", "sleep 30 & exit 0"])
            .process_group(0)
            .spawn()
            .unwrap();
        let pgid = leader.id();
        leader.wait().unwrap();
        let group = format!("-{pgid}");
        let signal = |signal: &str| {
            std::process::Command::new("kill")
                .args(["-s", signal, "--", &group])
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success()
        };
        assert!(signal("0"), "the sleep runs");
        crate::coder::end(vec![pgid]);
        let ended = !crate::coder::group_alive(pgid);
        // Never leave the sleep behind.
        signal("KILL");
        assert!(ended, "the group runs on");
    }

    /// The words of an agent that talks on and on are cut.
    #[test]
    fn a_long_message_is_cut() {
        let (tx, _rx) = mpsc::channel();
        let agents = Agents::new(&Config::default(), std::env::temp_dir(), tx);
        let actor = Actor {
            agents,
            name: "a".into(),
            id: 1,
        };
        let mut session = Session::default();
        let chunk = json!({"sessionUpdate": "agent_message_chunk",
            "content": {"type": "text", "text": "ä".repeat(1000)}});
        for _ in 0..20 {
            actor.update(&mut session, &chunk);
        }
        assert_eq!(session.message.len(), MESSAGE_BYTES);
    }

    /// The question waits for the core, which times it out from when it
    /// is spoken: a late yes still allows.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_question_waits_until_it_is_spoken() {
        let data = std::env::temp_dir().join(format!("chora-voice-wait-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let project = data.join("demo");
        std::fs::create_dir_all(&project).unwrap();
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake-acp.sh");
        let config = Config {
            agent_cmd: vec!["sh".into(), script.display().to_string()],
            unsandboxed: true,
            agent_projects: [("demo".into(), project.display().to_string())].into(),
            agent_permission_s: 1,
            ..Config::default()
        };
        let (tx, rx) = mpsc::channel();
        let agents = Agents::new(&config, data.clone(), tx);
        agents.start("demo", None, None);
        let (rx, _) = until(rx, |e| is_state(e, "idle")).await;
        agents.prompt("demo", "Schreib notes.txt");
        let (rx, mut events) = until(rx, |e| matches!(e, Event::Ask { .. })).await;
        let Some(Event::Ask { answer, .. }) = events.pop() else {
            unreachable!()
        };
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let _ = answer.send(true);
        let (_rx, events) = until(rx, |e| matches!(e, Event::Said { .. })).await;
        let Some(Event::Said { text, .. }) = events.last() else {
            unreachable!()
        };
        assert!(text.starts_with("Erlaubt"), "{text}");
        agents.stop("demo");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// A request whose detail is too long to show whole is rejected,
    /// and the user is told, so that no part of it hides the rest.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_request_too_long_to_show_is_rejected() {
        let data = std::env::temp_dir().join(format!("chora-voice-long-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let project = data.join("demo");
        std::fs::create_dir_all(&project).unwrap();
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake-acp.sh");
        let config = Config {
            agent_cmd: vec!["sh".into(), script.display().to_string()],
            unsandboxed: true,
            agent_projects: [("demo".into(), project.display().to_string())].into(),
            ..Config::default()
        };
        let (tx, rx) = mpsc::channel();
        let agents = Agents::new(&config, data.clone(), tx);
        agents.start("demo", None, None);
        let (rx, _) = until(rx, |e| is_state(e, "idle")).await;
        agents.prompt("demo", "LONG");
        let (_rx, events) = until(
            rx,
            |e| matches!(e, Event::Said { text, .. } if text.starts_with("Abgelehnt")),
        )
        .await;
        assert!(!events.iter().any(|e| matches!(e, Event::Ask { .. })));
        let told = events
            .iter()
            .any(|e| matches!(e, Event::Said { text, .. } if text.contains("too long")));
        assert!(told, "{}", events.len());
        agents.stop("demo");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// A yes allows always only when the question said so.
    #[test]
    fn only_a_question_for_always_allows_always() {
        let asks = "{name} möchte {what}. Erlauben?";
        let both = [
            json!({"optionId": "a", "kind": "allow_always"}),
            json!({"optionId": "o", "kind": "allow_once"}),
        ];
        let ask = |options: &[Value]| question(asks, "atlas", "x ändern", options, true);
        assert_eq!(ask(&both), "Atlas möchte x ändern. Erlauben?");
        let always = [
            json!({"optionId": "a", "kind": "allow_always"}),
            json!({"optionId": "r", "kind": "reject_once"}),
        ];
        assert_eq!(ask(&always), "Atlas möchte x ändern. Immer erlauben?");
        assert_eq!(
            question("", "atlas", "edit x", &always, false),
            "Atlas wants to edit x. Always allow?"
        );
        assert_eq!(outcome(&always, true)["outcome"]["optionId"], "a");
    }

    /// The next event of the agents, within 10 s.
    fn next(rx: &mpsc::Receiver<Input>) -> Event {
        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Input::Agent(event)) => event,
            Ok(_) => panic!("not an agent event"),
            Err(e) => panic!("no event: {e}"),
        }
    }

    /// The events until `stop` says so, the last one included.
    async fn until(
        rx: mpsc::Receiver<Input>,
        stop: impl Fn(&Event) -> bool + Send + 'static,
    ) -> (mpsc::Receiver<Input>, Vec<Event>) {
        tokio::task::spawn_blocking(move || {
            let mut events = Vec::new();
            loop {
                let event = next(&rx);
                let done = stop(&event);
                events.push(event);
                if done {
                    return (rx, events);
                }
            }
        })
        .await
        .unwrap()
    }

    fn is_state(event: &Event, want: &str) -> bool {
        matches!(event, Event::State(r) if r["state"] == want)
    }

    /// A line that comes in pieces while commands arrive is read whole:
    /// a command between two pieces loses none of it.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_line_in_pieces_survives_commands() {
        let data = std::env::temp_dir().join(format!("chora-voice-split-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let project = data.join("demo");
        std::fs::create_dir_all(&project).unwrap();
        let script = r#"read -r _ || exit 0
printf '{"jsonrpc":"2.0","id":1,'; sleep 0.3; printf '"result":{"protocolVersion":1}}\n'
read -r _ || exit 0
printf '{"jsonrpc":"2.0","id":2,'; sleep 0.3; printf '"result":{"sessionId":"s1"}}\n'
exec cat >/dev/null"#;
        let config = Config {
            agent_cmd: vec!["sh".into(), "-c".into(), script.into()],
            unsandboxed: true,
            agent_projects: [("demo".into(), project.display().to_string())].into(),
            ..Config::default()
        };
        let (tx, rx) = mpsc::channel();
        let agents = Agents::new(&config, data.clone(), tx);
        assert!(agents.start("demo", Some("split"), None).contains("starts"));
        let pokes = agents.clone();
        let poke = tokio::spawn(async move {
            for _ in 0..40 {
                pokes.cancel("split");
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        });
        let _ = until(rx, |e| is_state(e, "idle")).await;
        poke.await.unwrap();
        agents.stop("split");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// The fake agent of `tests/fake-acp.sh`: a prompt, its plan, its
    /// question with a yes, its answer in short; a stop ends its
    /// process.
    #[tokio::test(flavor = "multi_thread")]
    #[allow(clippy::too_many_lines)] // one conversation, step by step
    async fn an_agent_works_asks_answers_and_stops() {
        let data = std::env::temp_dir().join(format!("chora-voice-acp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let project = data.join("demo");
        std::fs::create_dir_all(&project).unwrap();
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake-acp.sh");
        let config = Config {
            agent_cmd: vec!["sh".into(), script.display().to_string()],
            unsandboxed: true,
            agent_projects: [("demo".into(), project.display().to_string())].into(),
            ..Config::default()
        };
        let (tx, rx) = mpsc::channel();
        let agents = Agents::new(&config, data.clone(), tx);
        assert_eq!(agents.specs().len(), TOOLS.len());
        assert!(agents.start("nope", None, None).contains("no project"));
        assert!(agents.start("Demo", Some("Coder"), None).contains("starts"));
        assert!(
            agents
                .start("demo", Some("coder"), None)
                .contains("runs already")
        );
        let (rx, _) = until(rx, |e| is_state(e, "idle")).await;
        assert_eq!(
            agents.prompt("Coder", "Schreib notes.txt"),
            "coder got the request"
        );
        let (rx, mut events) = until(rx, |e| matches!(e, Event::Ask { .. })).await;
        let Some(Event::Ask {
            name,
            voice,
            prompt,
            detail: _,
            physical,
            answer,
        }) = events.pop()
        else {
            unreachable!()
        };
        assert!(physical, "a write takes a key");
        assert_eq!((name.as_str(), voice.as_str()), ("coder", "ryan"));
        assert_eq!(prompt, "Coder wants to edit notes.txt. Allow?");
        let status = agents.call("agent_status", &json!({"name": "coder"}));
        assert!(
            status
                .as_str()
                .unwrap()
                .contains("question: edit notes.txt"),
            "{status}"
        );
        answer.send(true).unwrap();
        let (rx, events) = until(rx, |e| matches!(e, Event::Said { .. })).await;
        let done = events.iter().rev().find_map(|e| match e {
            Event::State(r) if r["state"] == "done" => Some(r),
            _ => None,
        });
        let done = done.expect("done before the words");
        assert_eq!(done["plan"], json!(["✓ Code lesen", "✓ Datei schreiben"]));
        let log: Vec<&str> = (done["log"].as_array().unwrap().iter())
            .map(|l| l.as_str().unwrap())
            .collect();
        assert_eq!(
            &log[..5],
            [
                "› Schreib notes.txt",
                "Ich lese den Code.",
                "⚙ Read src/main.rs",
                "? edit notes.txt",
                "› ja"
            ]
        );
        assert!(
            log[5].starts_with("Erlaubt: die Datei ist fertig."),
            "{log:?}"
        );
        let Some(Event::Said { text, voice }) = events.last() else {
            unreachable!()
        };
        assert_eq!(voice, "ryan");
        assert_eq!(
            text,
            "Erlaubt: die Datei ist fertig. Details on the screen."
        );
        let pid = agents.lock().subs["coder"].pid.unwrap();
        assert_eq!(agents.stop("coder"), "coder stops");
        let (_rx, events) = until(rx, |e| matches!(e, Event::Gone(_))).await;
        assert!(matches!(&events[..], [Event::Gone(n)] if n == "coder"));
        let alive = || {
            std::fs::read_to_string(format!("/proc/{pid}/stat"))
                .is_ok_and(|stat| !stat.contains(") Z "))
        };
        let start = Instant::now();
        while alive() {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "the agent runs on"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let log = std::fs::read_to_string(data.join("agents/coder.log")).unwrap();
        assert!(log.contains(r#""method":"initialize""#), "{log}");
        assert!(log.contains(r#"< {"jsonrpc":"2.0","id":2,"#), "{log}");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// A project with the fake agent, started and idle.
    async fn idle_agent(tag: &str) -> (Agents, mpsc::Receiver<Input>, PathBuf) {
        let data = std::env::temp_dir().join(format!("chora-voice-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data);
        let project = data.join("demo");
        std::fs::create_dir_all(&project).unwrap();
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fake-acp.sh");
        let config = Config {
            agent_cmd: vec!["sh".into(), script.display().to_string()],
            unsandboxed: true,
            agent_projects: [("demo".into(), project.display().to_string())].into(),
            ..Config::default()
        };
        let (tx, rx) = mpsc::channel();
        let agents = Agents::new(&config, data.clone(), tx);
        agents.start("demo", Some("coder"), None);
        let (rx, _) = until(rx, |e| is_state(e, "idle")).await;
        (agents, rx, data)
    }

    fn roles(read: &Value) -> Vec<(u64, String)> {
        (read["entries"].as_array().unwrap().iter())
            .map(|e| {
                (
                    e["seq"].as_u64().unwrap(),
                    e["role"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_state_record_carries_the_epoch() {
        let (agents, _rx, _data) = idle_agent("epoch").await;
        let inner = agents.lock();
        let sub = &inner.subs["coder"];
        assert_eq!(sub.record("coder")["epoch"], sub.id);
    }

    /// `agent.read` answers the entries after a seq, in order, with the
    /// seq that the reader asks next; the ring keeps 500 entries.
    #[tokio::test(flavor = "multi_thread")]
    async fn agent_read_answers_after_seq() {
        let (agents, rx, data) = idle_agent("read").await;
        assert_eq!(agents.read("coder", 0, None).unwrap()["entries"], json!([]));
        agents.prompt("coder", "Schreib notes.txt");
        let (rx, mut events) = until(rx, |e| matches!(e, Event::Ask { .. })).await;
        let Some(Event::Ask { answer, .. }) = events.pop() else {
            unreachable!()
        };
        let first = agents.read("coder", 0, None).unwrap();
        let seen = roles(&first);
        let kinds: Vec<&str> = seen.iter().map(|(_, r)| r.as_str()).collect();
        assert_eq!(kinds, ["user", "plan", "agent", "tool", "question"]);
        assert!(seen.windows(2).all(|w| w[1].0 == w[0].0 + 1), "{seen:?}");
        assert_eq!(first["entries"][0]["text"], "Schreib notes.txt");
        assert_eq!(first["entries"][3]["text"], "Read src/main.rs");
        assert_eq!(first["seq"], seen[4].0);
        let epoch = agents.lock().subs["coder"].id;
        assert_eq!(first["epoch"], epoch);
        assert_eq!(agents.read("coder", 99, None).unwrap()["epoch"], epoch);
        let record = agents.lock().subs["coder"].record("coder");
        assert_eq!(record["seq"], seen[4].0);
        let after = agents.read("coder", seen[2].0, None).unwrap();
        let kinds: Vec<String> = roles(&after).into_iter().map(|(_, r)| r).collect();
        assert_eq!(kinds, ["tool", "question"]);
        let none = agents.read("coder", seen[4].0, None).unwrap();
        assert_eq!(none["entries"], json!([]));
        assert_eq!(none["seq"], seen[4].0);
        answer.send(true).unwrap();
        let (_rx, _) = until(rx, |e| is_state(e, "done")).await;
        let rest = agents.read("coder", seen[4].0, None).unwrap();
        let rest = roles(&rest);
        assert!(rest.iter().all(|(n, _)| *n > seen[4].0), "{rest:?}");
        assert!(rest.iter().any(|(_, r)| r == "agent"), "{rest:?}");
        assert!(agents.read("nobody", 0, None).is_err());
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn the_ring_keeps_500_entries_and_a_read_at_most_64_kib() {
        let (cmds, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut sub = Sub::new(1, "demo", Path::new("/"), "ryan", cmds);
        for i in 0..600 {
            sub.entry("agent", &format!("line {i}"));
        }
        assert_eq!(sub.entries.len(), 500);
        assert_eq!(sub.seq, 600);
        assert_eq!(sub.entries.front().unwrap().seq, 101);
        let mut sub = Sub::new(
            2,
            "demo",
            Path::new("/"),
            "ryan",
            tokio::sync::mpsc::unbounded_channel().0,
        );
        for _ in 0..8 {
            sub.entry("agent", &"x".repeat(16 * 1024));
        }
        let read = sub.read(0);
        let bytes: usize = (read["entries"].as_array().unwrap().iter())
            .map(|e| e["text"].as_str().unwrap().len())
            .sum();
        assert!(bytes <= 64 * 1024, "{bytes}");
        assert_eq!(read["entries"].as_array().unwrap().len(), 4);
        assert_eq!(read["seq"], 4);
    }

    /// A chat is open until its caller closes it or is gone; the agent
    /// record says so.
    #[tokio::test(flavor = "multi_thread")]
    async fn chat_open_drops_on_disconnect_and_on_chat_close() {
        let (agents, _rx, data) = idle_agent("chat").await;
        let open = |agents: &Agents| agents.lock().subs["coder"].record("coder")["chat"].clone();
        assert_eq!(open(&agents), json!(false));
        agents.chat_open("coder", "hud").unwrap();
        agents.chat_open("coder", "other").unwrap();
        assert_eq!(open(&agents), json!(true));
        agents.chat_gone(Some("hud"));
        assert_eq!(open(&agents), json!(true), "another chat is open");
        agents.chat_gone(Some("other"));
        assert_eq!(open(&agents), json!(false));
        agents.chat_open("coder", "hud").unwrap();
        assert!(agents.chat_close("coder", "stranger").is_err());
        assert_eq!(open(&agents), json!(true), "only its opener closes it");
        agents.chat_close("coder", "hud").unwrap();
        assert_eq!(open(&agents), json!(false));
        assert!(agents.chat_open("nobody", "hud").is_err());
        agents.chat_open("coder", "hud").unwrap();
        agents.chat_gone(None);
        assert_eq!(open(&agents), json!(false), "a new session drops all");
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// The results of an agent with an open chat are not spoken; its
    /// question is.
    #[tokio::test(flavor = "multi_thread")]
    async fn an_open_chat_mutes_results_not_questions() {
        let (agents, rx, data) = idle_agent("mute").await;
        agents.chat_open("coder", "hud").unwrap();
        agents.prompt("coder", "Schreib notes.txt");
        let (rx, mut events) = until(rx, |e| matches!(e, Event::Ask { .. })).await;
        let Some(Event::Ask { answer, .. }) = events.pop() else {
            unreachable!()
        };
        answer.send(true).unwrap();
        let (rx, events) = until(rx, |e| is_state(e, "done")).await;
        assert!(!events.iter().any(|e| matches!(e, Event::Said { .. })));
        assert!(rx.recv_timeout(Duration::from_millis(500)).is_err());
        let read = agents.read("coder", 0, None).unwrap();
        assert!(roles(&read).iter().any(|(_, r)| r == "agent"));
        agents.chat_close("coder", "hud").unwrap();
        agents.prompt("coder", "Noch eins");
        let (rx, mut events) = until(rx, |e| matches!(e, Event::Ask { .. })).await;
        let Some(Event::Ask { answer, .. }) = events.pop() else {
            unreachable!()
        };
        answer.send(true).unwrap();
        let (_rx, _) = until(rx, |e| matches!(e, Event::Said { .. })).await;
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// An open chat changes nothing about the gate: an edit still waits
    /// for a key or a button, and a chat method answers no question.
    #[tokio::test(flavor = "multi_thread")]
    async fn an_edit_request_stays_physical_with_a_chat_open() {
        let (agents, rx, data) = idle_agent("physical").await;
        agents.chat_open("coder", "hud").unwrap();
        agents.prompt("coder", "Schreib notes.txt");
        let (rx, mut events) = until(rx, |e| matches!(e, Event::Ask { .. })).await;
        let Some(Event::Ask {
            physical, answer, ..
        }) = events.pop()
        else {
            unreachable!()
        };
        assert!(physical);
        for text in ["yes", "y", "ja"] {
            agents.call("agent_say", &json!({"name": "coder", "text": text}));
        }
        agents.call("chat.open", &json!({"name": "coder", "caller": "hud"}));
        agents.call("agent.read", &json!({"name": "coder", "after": 0}));
        agents.call("agent.cancel", &json!({"name": "nobody"}));
        assert_eq!(agents.lock().subs["coder"].state, "question");
        assert!(rx.recv_timeout(Duration::from_millis(300)).is_err());
        let log = std::fs::read_to_string(data.join("agents/coder.log")).unwrap();
        assert!(!log.contains(r#""outcome""#), "{log}");
        drop(answer);
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// Cancelling an agent that runs answers at once; the lock is not
    /// held through the cancel.
    #[tokio::test(flavor = "multi_thread")]
    async fn cancelling_a_running_agent_does_not_deadlock() {
        let (agents, _rx, data) = idle_agent("cancel").await;
        let (done, answer) = std::sync::mpsc::channel();
        let again = agents.clone();
        std::thread::spawn(move || {
            let _ = done.send(again.cancel_running("coder"));
        });
        let answer = answer.recv_timeout(Duration::from_secs(5));
        assert_eq!(answer, Ok(Ok("coder stops its request".to_owned())));
        assert!(agents.cancel_running("Coder").is_ok());
        assert!(agents.cancel_running("nobody").is_err());
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }

    fn lone_sub() -> Sub {
        let (cmds, _rx) = tokio::sync::mpsc::unbounded_channel();
        Sub::new(1, "demo", Path::new("/"), "ryan", cmds)
    }

    /// An entry is cut to 16 KiB on a char boundary with an ellipsis,
    /// and the ring holds at most 4 MiB, oldest dropped first.
    #[test]
    fn an_entry_is_cut_and_the_ring_has_a_byte_limit() {
        let mut sub = lone_sub();
        sub.entry("tool", &"é".repeat(20 * 1024));
        let text = &sub.entries[0].text;
        assert!(text.len() <= 16 * 1024, "{}", text.len());
        assert!(text.len() > 16 * 1024 - 4, "{}", text.len());
        assert!(text.ends_with('…'));
        sub.entry("tool", &"x".repeat(16 * 1024));
        assert_eq!(sub.entries[1].text.len(), 16 * 1024);
        let mut sub = lone_sub();
        for _ in 0..400 {
            sub.entry("agent", &"x".repeat(100 * 1024));
        }
        let bytes: usize = sub.entries.iter().map(|e| e.text.len()).sum();
        assert!(bytes <= 4 * 1024 * 1024, "{bytes}");
        assert!(sub.entries.len() >= 250, "{}", sub.entries.len());
        assert_eq!(sub.entries.back().unwrap().seq, 400);
        assert!(sub.entries.front().unwrap().seq > 1);
    }

    /// The transcript hides what the journal hides.
    #[test]
    fn the_transcript_hides_secrets() {
        let mut sub = lone_sub();
        sub.entry("tool", r#"{"cmd": "x", "api_key": "sk-123"}"#);
        sub.entry("tool", "curl -H\nAuthorization: Bearer abc\ntoken=xyz\nls");
        sub.entry("tool", "Read src/main.rs");
        let all: String = sub.entries.iter().map(|e| e.text.as_str()).collect();
        for secret in ["sk-123", "abc", "xyz"] {
            assert!(!all.contains(secret), "{all}");
        }
        assert!(all.contains("(hidden)") && all.contains("Read src/main.rs"));
        assert!(all.contains("curl -H") && all.contains("ls"), "{all}");
    }

    /// A chat that is not read for the time to live ends; a read keeps
    /// it, and the agent record says so.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_chat_that_is_not_read_expires() {
        let (agents, rx, data) = idle_agent("expire").await;
        agents.lock().chat_ttl = Duration::from_millis(300);
        let open = |agents: &Agents| agents.lock().subs["coder"].record("coder")["chat"].clone();
        agents.call("chat.open", &json!({"name": "coder", "caller": "hud"}));
        agents.call("chat.open", &json!({"name": "coder", "caller": "idle"}));
        for _ in 0..3 {
            tokio::time::sleep(Duration::from_millis(150)).await;
            agents.call("agent.read", &json!({"name": "coder", "caller": "hud"}));
            agents.expire_chats();
        }
        let chats: Vec<String> = agents.lock().subs["coder"].chats.keys().cloned().collect();
        assert_eq!(chats, ["hud"]);
        assert_eq!(open(&agents), json!(true));
        tokio::time::sleep(Duration::from_millis(400)).await;
        agents.expire_chats();
        assert_eq!(open(&agents), json!(false));
        let (_rx, events) = until(rx, |e| matches!(e, Event::State(r) if r["chat"] == false)).await;
        assert!(!events.is_empty());
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }

    /// A chat method without a caller is an error, not the unknown-tool
    /// text.
    #[test]
    fn chat_open_and_close_need_a_caller() {
        let (tx, _rx) = mpsc::channel();
        let agents = Agents::new(&Config::default(), PathBuf::from("/"), tx);
        for method in ["chat.open", "chat.close"] {
            let answer = agents.call(method, &json!({"name": "coder"}));
            assert_eq!(answer["error"], format!("{method} needs a caller"));
        }
    }

    /// The notice of a refusal and of the end of an agent has the role
    /// `notice`, not `tool`.
    #[tokio::test(flavor = "multi_thread")]
    async fn notices_have_their_own_role() {
        let (agents, rx, data) = idle_agent("notice").await;
        agents.prompt("coder", "LONG");
        let (rx, _) = until(
            rx,
            |e| matches!(e, Event::Said { text, .. } if text.contains("too long")),
        )
        .await;
        let read = agents.read("coder", 0, None).unwrap();
        let notice = read["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["text"].as_str().unwrap().contains("too long"));
        assert_eq!(notice.unwrap()["role"], "notice");
        let pid = agents.lock().subs["coder"].pid.unwrap();
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .status();
        let (_rx, _) = until(rx, |e| is_state(e, "blocked")).await;
        let read = agents.read("coder", 0, None).unwrap();
        assert_eq!(
            read["entries"].as_array().unwrap().last().unwrap()["role"],
            "notice"
        );
        agents.stop("coder");
        let _ = std::fs::remove_dir_all(&data);
    }
}
