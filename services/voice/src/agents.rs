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
    process::ChildStdin,
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
    permission: Duration,
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
            permission: Duration::from_secs(config.agent_permission_s.into()),
        }
    }
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
            started: Instant::now(),
            cmds: Some(cmds),
            pid: None,
        }
    }

    fn record(&self, name: &str) -> Value {
        json!({"name": name, "state": self.state, "task": self.task, "said": self.said,
            "voice": self.voice, "plan": self.plan, "question": self.question,
            "log": self.log})
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

/// A name with a capital first letter, as spoken and as mc shows it.
fn title(name: &str) -> String {
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
        let Some(dir) = self.projects(&inner.settings).remove(project) else {
            return format!("error: no project {project}; agent_list names them");
        };
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
        let (cmds, rx) = mpsc::unbounded_channel();
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
            permission: inner.settings.permission,
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

    /// Stops all agents and waits up to 5 s for their processes, then
    /// kills what is left: at the exit of the voice agent.
    pub fn shutdown(&self) {
        let pids: Vec<u32> = {
            let mut inner = self.lock();
            let subs = std::mem::take(&mut inner.subs);
            subs.into_values().filter_map(|s| s.pid).collect()
        };
        let alive = |pid: &u32| {
            std::fs::read_to_string(format!("/proc/{pid}/stat"))
                .is_ok_and(|stat| !stat.contains(") Z "))
        };
        let start = Instant::now();
        while pids.iter().any(alive) && start.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(50));
        }
        for pid in pids.iter().filter(|p| alive(p)) {
            crate::log::warn(&format!("agent pid {pid} ignored TERM; KILL"));
            let _ = std::process::Command::new("kill")
                .args(["-s", "KILL", "--", &format!("-{pid}")])
                .status();
        }
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
        let (limit, details) = {
            let inner = self.lock();
            (inner.settings.said_chars, inner.settings.details.clone())
        };
        let text = summary(text, limit, &details);
        if !text.is_empty() {
            let voice = self.voice(name);
            let _ = self.tx.send(Input::Agent(Event::Said { voice, text }));
        }
    }
}

/// A question of the agent that waits for the user.
struct Permission {
    id: Value,
    options: Vec<Value>,
    answer: oneshot::Receiver<bool>,
    deadline: tokio::time::Instant,
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

/// The answer to a permission request: the first option of an allowing
/// kind for yes, of a rejecting kind for no; cancelled without one.
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

/// What a permission request asks, short enough to speak: the title of
/// its tool call, a path as its file name, and the kind as a verb.
fn asked(tool: &Value) -> String {
    let title = tool["title"].as_str().unwrap_or_default().trim();
    let title = if title.starts_with('/') {
        Path::new(title)
            .file_name()
            .map_or(title.into(), |f| f.to_string_lossy())
    } else {
        title.into()
    };
    let title: String = title.chars().take(80).collect();
    // ponytail: German verbs, as the default phrases; a phrase per kind
    // in the settings when another language needs it.
    let verb = match tool["kind"].as_str() {
        Some("edit") => "ändern",
        Some("delete") => "löschen",
        Some("move") => "verschieben",
        Some("execute") => "ausführen",
        Some("fetch") => "abrufen",
        Some("read") => "lesen",
        _ => "",
    };
    match (title.is_empty(), verb.is_empty()) {
        (true, true) => "etwas tun".into(),
        (true, false) => verb.into(),
        (false, true) => title,
        (false, false) => format!("{title} {verb}"),
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
    permission: Duration,
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
        let _group = Group(child.id());
        let pid = child.id();
        self.agents.update(&self.name, self.id, |s| s.pid = pid);
        let mut out = BufReader::new(child.stdout.take().expect("piped"));
        let mut session = Session::default();
        loop {
            tokio::select! {
                line = read_line(&mut out) => {
                    let Some(line) = line? else { return Ok(false) };
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
            s.note(if yes { "  ja" } else { "  nein" });
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
        let what = asked(&params["toolCall"]);
        let (answer, rx) = oneshot::channel();
        session.question = Some(Permission {
            id: rpc_id,
            options: params["options"].as_array().cloned().unwrap_or_default(),
            answer: rx,
            deadline: tokio::time::Instant::now() + self.permission,
        });
        let asks = self.agents.lock().settings.asks.clone();
        let prompt = asks
            .replace("{name}", &title(name))
            .replace("{what}", &what);
        self.agents.update(name, id, |s| {
            s.state = "question";
            s.note(&format!("? {what}"));
            s.question = what;
        });
        let voice = self.agents.voice(name);
        let _ = self.agents.tx.send(Input::Agent(Event::Ask {
            name: name.to_owned(),
            voice,
            prompt,
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
                    session.message.push_str(text);
                }
            }
            Some("tool_call") => {
                self.flush(session);
                let title = update["title"].as_str().unwrap_or_default().to_owned();
                self.agents
                    .update(name, id, |s| s.note(&format!("⚙ {title}")));
            }
            Some("plan") => {
                let plan: Vec<String> = (update["entries"].as_array().into_iter().flatten())
                    .map(plan_line)
                    .collect();
                self.agents.update(name, id, |s| s.plan = plan);
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
            text.clone_into(&mut s.said);
        });
    }
}

/// The answer to the question that waits: yes, or no after its timeout
/// or when the core dropped it. Without a question, never.
async fn decided(question: &mut Option<Permission>) -> bool {
    match question {
        Some(q) => matches!(
            tokio::time::timeout_at(q.deadline, &mut q.answer).await,
            Ok(Ok(true))
        ),
        None => std::future::pending().await,
    }
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
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn a_long_answer_speaks_its_first_sentence() {
        assert_eq!(summary(" Kurz. ", 20, "Mehr."), "Kurz.");
        let long = "Erledigt: alles gut. Und dann noch sehr viel mehr Text danach.";
        assert_eq!(summary(long, 20, "Mehr."), "Erledigt: alles gut. Mehr.");
        assert_eq!(summary("v1.2 ist da und lang", 5, "X"), "v1.2 X");
    }

    #[test]
    fn a_question_is_short_to_speak() {
        let ask = |tool| asked(&tool);
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
        assert!(agents.start("demo", Some("Coder"), None).contains("starts"));
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
            answer,
        }) = events.pop()
        else {
            unreachable!()
        };
        assert_eq!((name.as_str(), voice.as_str()), ("coder", "ryan"));
        assert_eq!(prompt, "Coder fragt: notes.txt ändern. Erlauben?");
        let status = agents.call("agent_status", &json!({"name": "coder"}));
        assert!(
            status
                .as_str()
                .unwrap()
                .contains("question: notes.txt ändern"),
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
                "? notes.txt ändern",
                "ja"
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
            "Erlaubt: die Datei ist fertig. Details auf dem Bildschirm."
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
}
