//! The conversation with the LLM: the system prompt with the context
//! documents, the history with its token budget and file, and the tool
//! loop of a turn with the confirmation gate (docs/voice.md sections 5
//! to 7).

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

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
    Call no tool. In your reply language, say in one or two short sentences what you did \
    and what is left.)";

/// The note after an answer that announced a tool but called none.
const NUDGE: &str = "(The agent: you announced an action but called no tool, and your turn \
    would end here. Call the tool now, or say in one short sentence that you will not.)";

/// The note of the round after a cut answer; it does not stay in the
/// history. The same request gave the same fragment (dump 2026-10-31).
const RETRY: &str = "[Your last answer stopped mid-sentence. Answer again in full.]";

/// Whether the last sentence of an answer announces an action in the
/// first person, in an explicit form only: "let me", "I'll", "I will"
/// or "I'm going to" before a verb of action, "ich werde", "lass mich"
/// and "ich schaue nach", "mal" or "kurz" (logs 2026-10-31). Closing
/// lines announce nothing ("Ich helfe dir gern.", "I'll be here if you
/// need anything."), nor do a question, "let me know" and a no ("I
/// won't", "nicht", "kein").
fn announces(answer: &str) -> bool {
    // ponytail: English and German forms; a setting when another
    // language needs them.
    const ENGLISH: [&str; 4] = ["let me", "i'll", "i will", "i'm going to"];
    const GERMAN: [&str; 5] = [
        "ich werde",
        "lass mich",
        "ich schaue nach",
        "ich schaue mal",
        "ich schaue kurz",
    ];
    /// Words after an English form that are no action.
    const NO_ACTION: [&str; 8] = [
        "know", "be", "not", "never", "stay", "wait", "remember", "keep",
    ];
    let text = answer.to_lowercase().replace('’', "'");
    let text = text.trim_end_matches(|c: char| c.is_whitespace() || ".!…\"»“”".contains(c));
    if text.ends_with('?') {
        return false;
    }
    let last = text.rsplit(['.', '!', '?']).next().unwrap_or_default();
    let words: Vec<&str> = last
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\''))
        .collect();
    if words.iter().any(|w| *w == "nicht" || w.starts_with("kein")) {
        return false;
    }
    let at = |form: &str, k: usize| {
        let form: Vec<&str> = form.split(' ').collect();
        words[k..]
            .starts_with(&form)
            .then(|| words.get(k + form.len()))
    };
    (0..words.len()).any(|k| {
        ENGLISH
            .iter()
            .any(|f| at(f, k).is_some_and(|next| next.is_some_and(|n| !NO_ACTION.contains(n))))
            || GERMAN.iter().any(|f| at(f, k).is_some())
    })
}

/// What a yes with an always word allows for the session: the same
/// full argv of `run_program`, or the same method `CLIENT.METHOD` with
/// the same parameters. None for every other call, such as
/// `inject_input`, `apply_commands`, `code_task`, `agent_start` or
/// `configure_output`, and for each settings change: for them, an always
/// word is a single yes.
fn session_key(tool: &str, patch: Option<&Value>, args: &Value) -> Option<String> {
    if patch.is_some() {
        return None;
    }
    // serde_json sorts the keys of an object: the text is canonical.
    let (method, params) = match tool {
        "run_program" => return Some(format!("run_program {}", args.get("argv")?)),
        "call_method" => (
            format!("{}.{}", args["client"].as_str()?, args["method"].as_str()?),
            &args["params"],
        ),
        _ if tool.contains('.') => (tool.to_owned(), args),
        _ => return None,
    };
    let settings = [".settings.set", ".settings.reload"];
    (!settings.iter().any(|s| method.ends_with(s))).then(|| format!("{method} {params}"))
}

/// Whether `allow_programs` lists the program of `run_program`, a bare
/// name equal to its `argv[0]`, or `allow_methods` the method
/// `CLIENT.METHOD` of a call.
fn listed(config: &Config, tool: &str, args: &Value) -> bool {
    let has = |list: &[String], entry: &str| list.iter().any(|e| e == entry);
    match tool {
        "run_program" => args["argv"].as_array().is_some_and(|argv| {
            let argv: Vec<&str> = argv.iter().filter_map(Value::as_str).collect();
            // An entry is the whole argv: a bare name only allows the
            // program without arguments (`foot -e sh` is a shell).
            (config.allow_programs.iter()).any(|e| e.split(' ').eq(argv.iter().copied()))
                && argv.first().is_some_and(|p| !p.contains('/'))
        }),
        "call_method" => (args["client"].as_str())
            .zip(args["method"].as_str())
            .is_some_and(|(c, m)| has(&config.allow_methods, &format!("{c}.{m}"))),
        _ if tool.contains('.') => has(&config.allow_methods, tool),
        _ => false,
    }
}

/// The full command or value of a call that asks, for the screen: the
/// settings keys and their values, the program and its arguments, the
/// method and its parameters, or the tool and its arguments. Secrets are
/// hidden as in the journal: each subscriber of `voice.*` gets it.
fn detail(tool: &str, patch: Option<&Value>, args: &Value) -> String {
    let text = |v: &Value| v.as_str().map_or_else(|| v.to_string(), str::to_owned);
    let (mut patch, mut args) = (patch.cloned(), args.clone());
    patch.iter_mut().for_each(hide);
    hide(&mut args);
    let (patch, args) = (patch.as_ref(), &args);
    if let Some(patch) = patch {
        let keys = patch.as_object().into_iter().flatten();
        return (keys.map(|(k, v)| format!("{k}: {}", text(v))))
            .collect::<Vec<_>>()
            .join("\n");
    }
    match (tool, args["argv"].as_array()) {
        ("run_program", Some(items)) => items.iter().map(text).collect::<Vec<_>>().join(" "),
        ("call_method", _) => format!(
            "{}.{} {}",
            text(&args["client"]),
            text(&args["method"]),
            args["params"]
        ),
        _ => format!("{tool} {args}"),
    }
}

/// Whether an answer stops mid-sentence: its words do not end with
/// `.!?…`, also before a closing quote. Gufo 0.27 ended rounds with
/// `stop` after 3-12 tokens ("I’m setting the", dump 2026-10-31); the
/// same request again gave the whole answer.
fn cut(answer: &str) -> bool {
    let text = answer
        .trim_end()
        .trim_end_matches(['"', '\'', '»', '«', '“', '”', '’', ')', '*', '`']);
    !text.is_empty() && !text.ends_with(['.', '!', '?', '…'])
}

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

/// The result of a tool call that the user stopped.
pub const CANCELLED: &str = "cancelled by the user; result unknown";

/// The note after an answer that the user stopped. It is a message of
/// the user of its own: in the answer, the LLM copied it into its next
/// answer.
pub const STOPPED: &str = "[Note of the agent: the user stopped the answer before this note; \
     the result of what it started is unknown. Do not repeat this note.]";

/// The notes that the LLM may copy; the agent never speaks or keeps them.
/// The longer note first.
const NOTES: [&str; 2] = [STOPPED, CANCELLED];

/// The played words up to the last whole sentence, else without the
/// last word, which may be cut: the LLM copied answers that end mid-word
/// and then ended its own there.
fn complete(played: &str) -> &str {
    let ends = |c: char| ".!?".contains(c);
    if played.ends_with(ends) {
        return played;
    }
    let sentence = played
        .char_indices()
        .rev()
        .find(|&(i, c)| ends(c) && played[i + 1..].starts_with(char::is_whitespace));
    if let Some((i, _)) = sentence {
        return &played[..=i];
    }
    played
        .rsplit_once(char::is_whitespace)
        .map_or("", |(words, _)| {
            words.trim_end_matches([',', ';', ':', '-', '–', ' '])
        })
}

/// `text` without the notes of `NOTES`.
pub fn strip_notes(text: &str) -> String {
    NOTES.iter().fold(text.to_owned(), |t, n| t.replace(n, ""))
}

/// The length of the end of `text` that may begin a note: the words that
/// stream must wait for the rest.
pub fn note_start(text: &str) -> usize {
    NOTES
        .iter()
        .flat_map(|n| (1..n.len()).filter(|&k| n.is_char_boundary(k) && text.ends_with(&n[..k])))
        .max()
        .unwrap_or(0)
}

/// What a turn tells the agent while it runs.
#[derive(Debug)]
pub enum Event {
    /// A request to the LLM starts.
    Thinking,
    /// A tool call starts.
    Tool(String),
    /// A sentence to speak.
    Sentence(String),
    /// A call waits for yes (`Some(true)`), no, or an unclear answer
    /// (`None`); the prompt is spoken too. A dropped sender is no answer.
    Confirm {
        prompt: String,
        /// The full command or value, uncut, for the screen.
        detail: String,
        /// The core sets it before a yes with an always word.
        always: Arc<AtomicBool>,
        /// Only a key of `confirm_keys` answers: a security setting.
        physical: bool,
        answer: oneshot::Sender<Option<bool>>,
    },
    /// A text of the user from the inbox went into the turn.
    Heard(String),
}

/// The note after the second failure of a call the same way (live log
/// 2026-10-31: the same `apply_commands` failed 8 times in one turn).
pub const FAILED_TWICE: &str =
    "This failed twice the same way; stop and tell the user what failed.";

/// The last failed call of a turn: its tool, its arguments, its error,
/// and how often in a row a call failed the same way.
struct Failed {
    tool: String,
    args: Value,
    error: String,
    times: u32,
}

impl Failed {
    /// Whether a call of `tool` with `args` is nearly the same: the same
    /// tool with the same form, the same keys and types, whatever the
    /// values.
    fn like(&self, tool: &str, args: &Value) -> bool {
        fn form(v: &Value) -> Value {
            match v {
                Value::Object(m) => m.iter().map(|(k, v)| (k.clone(), form(v))).collect(),
                Value::Array(a) => a.iter().map(form).collect(),
                Value::String(_) => "s".into(),
                Value::Number(_) => 0.into(),
                other => other.clone(),
            }
        }
        self.tool == tool && form(&self.args) == form(args)
    }
}

/// The tool result of a call whose question the agent skipped: the user
/// said something new while the turn ran.
pub const SAID_NEW: &str = "Not confirmed: the user said something new";

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

impl Report {
    /// Notes the first token of a round, the first of the turn only.
    fn first(&mut self, start: Instant, r: &Timed) {
        if let Some(first) = r.round.first {
            let begun = start.elapsed().saturating_sub(r.elapsed);
            self.llm_first.get_or_insert(begun + first);
        }
    }
}

pub struct Chat {
    /// The client name of the tools in chora.
    name: String,
    /// The settings file that `settings.reload` reads.
    pub settings_file: Option<PathBuf>,
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
    /// The texts that the user says while the turn runs; each goes into
    /// its next round.
    pub inbox: Option<UnboundedReceiver<String>>,
    /// The texts of the inbox that wait for the next round.
    heard: Vec<String>,
    /// The last failed call of the turn.
    failed: Option<Failed>,
    /// The actions that an always word allowed until the agent restarts
    /// or takes settings.
    allowed: HashSet<String>,
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
            settings_file: None,
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
            inbox: None,
            heard: Vec::new(),
            failed: None,
            allowed: HashSet::new(),
        };
        chat.system = chat.prompt()?;
        Ok(chat)
    }

    /// Takes new settings, also the same ones again on `settings.reload`
    /// and on each new connection to chora; answers whether the prompt
    /// changed, which asks for a new warm-up. The always words of the
    /// session end: new rules or lists may be stricter.
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
        if self.config.chora_mcp != old.chora_mcp {
            self.tools.restart(self.config.chora_mcp.clone(), &self.name);
        }
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
        self.allowed.clear();
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
            SYSTEM
                .replace("{name}", &self.name)
                .replace("{own}", &self.tools.own)
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
        parts.push(format!("# Language\n\n{}", config.reply_rule()));
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
        self.begin(text).await;
        let mut report = Report::default();
        let mut limit = self.config.tool_rounds;
        let mut nudged = false;
        // Cut answers in a row, and the sentences that they said: the
        // next round does not say them again.
        let mut cuts = 0;
        let mut skip = String::new();
        for round in 0.. {
            let last = round == limit;
            self.listen(out);
            out(Event::Thinking);
            report.rounds += 1;
            let (said, result) = self.round(last, cuts > 0, &skip, out).await;
            report.reply += &said;
            let r = match result {
                Ok(r) => r,
                Err(e) => {
                    report.outcome = self.failed(&e, out);
                    break;
                }
            };
            report.first(start, &r);
            if r.cut {
                cuts += 1;
                skip += &said;
                if self.brake(cuts, out) {
                    report.outcome = "cut";
                    break;
                }
                // A cut round is no tool round.
                limit += 1;
                continue;
            }
            cuts = 0;
            skip.clear();
            if last || r.round.calls.is_empty() {
                report.outcome = self.answered(&r.round, last, said.is_empty(), out);
                if self.more(&r.round.text, last || report.tools > 0, &mut nudged) {
                    // New words in the last round get rounds of their
                    // own now, not after the reply is said (log
                    // 2026-10-31: 49 s).
                    limit += u32::from(last) * (self.config.tool_rounds + 1);
                    continue;
                }
                break;
            }
            let tools_start = Instant::now();
            report.tools += self.run(r.round, round == 0 && said.is_empty(), out).await;
            report.tools_time += tools_start.elapsed();
        }
        self.open = None;
        report
    }

    /// Starts a turn with the text of the user.
    async fn begin(&mut self, text: &str) {
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
        self.heard.clear();
        self.failed = None;
    }

    /// Whether an answer needs one more round: the user said more while
    /// the LLM answered, also in the `last` round, or the answer announced
    /// an action and called no tool (once per turn, with `NUDGE`); `last`
    /// also when the turn already ran a tool: no nudge then.
    fn more(&mut self, answer: &str, last: bool, nudged: &mut bool) -> bool {
        if self.take_inbox() {
            crate::log::info("llm: the user said more; one more round");
            return true;
        }
        if last || *nudged || cut(answer) || !announces(answer) {
            return false;
        }
        crate::log::info("llm: an announced tool without a call; one more round");
        *nudged = true;
        self.history.push(json!({"role": "user", "content": NUDGE}));
        true
    }

    /// Moves the texts of the inbox to `heard`; answers whether texts
    /// wait there.
    fn take_inbox(&mut self) -> bool {
        if let Some(inbox) = &mut self.inbox {
            while let Ok(text) = inbox.try_recv() {
                self.heard.push(text);
            }
        }
        !self.heard.is_empty()
    }

    /// Puts the texts that the user said meanwhile into the turn, as
    /// user messages.
    fn listen(&mut self, out: &impl Fn(Event)) {
        self.take_inbox();
        if self.heard.is_empty() {
            return;
        }
        for text in std::mem::take(&mut self.heard) {
            crate::log::info(&format!("heard in the turn: {text:?}"));
            self.history.push(json!({"role": "user", "content": text}));
            self.requests.push(text.clone());
            out(Event::Heard(text));
        }
        self.save();
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
    /// played up to the last whole sentence, then the note `STOPPED` as a message of the user, or
    /// `CANCELLED` as the result of each tool call in flight. A turn whose
    /// words were all there but did not all play keeps only the played
    /// words.
    pub fn cancelled(&mut self, played: &str) {
        let played = complete(played.trim());
        let note = json!({"role": "user", "content": STOPPED});
        let calls = match (self.open.take(), self.answer.take()) {
            (Some(calls), _) if !calls.is_empty() => calls,
            (None, Some(i)) if played.is_empty() => {
                self.history[i] = note;
                return;
            }
            (None, Some(i)) => {
                self.history[i]["content"] = played.into();
                self.history.insert(i + 1, note);
                return;
            }
            _ => {
                if !played.is_empty() {
                    self.history
                        .push(json!({"role": "assistant", "content": played}));
                }
                self.history.push(note);
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

    /// After the `cuts`-th cut answer in a row: the second drops the
    /// history before the last words of the user, the third says
    /// `phrases.lost`. Answers whether the turn ends.
    fn brake(&mut self, cuts: u32, out: &impl Fn(Event)) -> bool {
        if cuts >= 3 {
            crate::log::warn("llm: three cut answers; the turn ends");
            out(Event::Sentence(self.config.phrases.lost.clone()));
            return true;
        }
        if cuts == 2 {
            let start = self.history.iter().rposition(|m| {
                m["role"] == "user" && m["content"] != NUDGE && m["content"] != STOPPED
            });
            let start = start.unwrap_or(0);
            crate::log::warn(&format!(
                "llm: two cut answers; dropped {start} messages of the history"
            ));
            self.history.drain(..start);
        }
        false
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
        let text = strip_notes(&round.text);
        if !text.trim().is_empty() {
            self.answer = Some(self.history.len());
            self.history
                .push(json!({"role": "assistant", "content": text.trim()}));
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
    /// answers the number of calls. With `ack`, says `phrases.ack` first:
    /// no silence while the tools run.
    async fn run(&mut self, round: Round, ack: bool, out: &(impl Fn(Event) + Sync)) -> u32 {
        if ack {
            out(Event::Sentence(self.config.phrases.ack.clone()));
        }
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
            "content": strip_notes(&round.text).trim(), "tool_calls": calls}));
        self.history.extend(results);
        u32::try_from(calls.len()).unwrap_or(u32::MAX)
    }

    /// One request with retries: once more after a connect error, and
    /// with half the budget after a context error. Answers the words that
    /// went out as sentences. The last words of a [`cut`] answer never go
    /// out. Sentences in `skip` went out before. A `retry` asks with
    /// `RETRY` for the whole answer.
    async fn round(
        &mut self,
        last: bool,
        retry: bool,
        skip: &str,
        out: &(impl Fn(Event) + Sync),
    ) -> (String, Result<Timed, llm::Error>) {
        let budget = self
            .context_tokens
            .saturating_sub(self.config.reply_reserve_tokens);
        let mut said = String::new();
        let mut retried = false;
        loop {
            let (body, messages, chars) =
                self.request(last, retry, if retried { budget / 2 } else { budget });
            let mut splitter = Splitter::default();
            let start = Instant::now();
            let result = self
                .llm
                .chat(&body, |words| {
                    let new = splitter.push(words).into_iter();
                    for sentence in new.filter(|s| !skip.contains(s)) {
                        said += &sentence;
                        said.push(' ');
                        out(Event::Sentence(sentence));
                    }
                })
                .await;
            let cut = matches!(&result, Ok(r) if r.finish == "stop"
                    && r.calls.is_empty() && cut(&strip_notes(&r.text)));
            let rest = splitter.finish();
            if cut {
                crate::log::info(&format!("llm: cut answer, dropped {rest:?}"));
            }
            // The words before the tool calls are spoken while they run.
            if let Some(sentence) = rest.filter(|s| !cut && !skip.contains(s)) {
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
                    return (
                        said,
                        Ok(Timed {
                            round,
                            elapsed,
                            cut,
                        }),
                    );
                }
            }
        }
    }

    /// The body of a request in `budget` tokens, its messages and their
    /// characters.
    fn request(&mut self, last: bool, retry: bool, budget: u32) -> (Value, Vec<Value>, usize) {
        let tools = self.tools.specs.clone();
        let fixed = self.system.len() + serde_json::to_string(&tools).map_or(0, |t| t.len());
        self.fit(budget, fixed);
        let mut messages = vec![json!({"role": "system", "content": self.system})];
        messages.extend(self.history.iter().cloned());
        if last {
            messages.push(json!({"role": "user", "content": LAST_ROUND}));
        }
        if retry {
            messages.push(json!({"role": "user", "content": RETRY}));
        }
        let mut body = self.llm.body(&messages, &tools);
        if last && !tools.is_empty() {
            // The tools stay in the request, so the server keeps the
            // prompt in its cache; without them it reads it again.
            body["tool_choice"] = "none".into();
        }
        let chars = fixed + size(&messages[1..]);
        (body, messages, chars)
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
            let again =
                (self.failed.as_ref()).is_some_and(|f| f.times >= 2 && f.like(&call.name, &args));
            let content = if args.is_null() {
                Value::from(format!("error: the arguments are no JSON: {}", call.args))
            } else if again {
                crate::log::info(&format!("tool {}: not run, it failed twice", call.name));
                Value::from(format!("error: not run. {FAILED_TWICE}"))
            } else if let Err(no) = self.confirmed(&call.name, &args, out).await {
                Value::from(format!("{no}; nothing happened"))
            } else {
                out(Event::Tool(call.name.clone()));
                let start = Instant::now();
                let content = self.tools.call(&call.name, &args).await;
                let (logged, result) = journal(&call.name, &args, &text_of(&content));
                crate::log::info(&format!(
                    "tool {} {logged} {}ms -> {result}",
                    call.name,
                    start.elapsed().as_millis(),
                ));
                self.guard(&call.name, &args, content)
            };
            results.push(json!({"role": "tool", "tool_call_id": call.id, "content": content}));
        }
        results
    }

    /// Notes a failed call and adds `FAILED_TWICE` to its error when the
    /// call before failed the same way: with the same error, or as the
    /// same tool with nearly the same arguments. The error stays whole,
    /// so the LLM can correct the form.
    fn guard(&mut self, tool: &str, args: &Value, content: Value) -> Value {
        let Some(error) = content.as_str().filter(|t| t.starts_with("error:")) else {
            self.failed = None;
            return content;
        };
        let times = match &self.failed {
            Some(f) if f.error == error || f.like(tool, args) => f.times + 1,
            _ => 1,
        };
        let error = error.to_owned();
        let content = if times >= 2 {
            format!("{error}\n{FAILED_TWICE}").into()
        } else {
            content
        };
        self.failed = Some(Failed {
            tool: tool.to_owned(),
            args: args.clone(),
            error,
            times,
        });
        content
    }

    /// Whether the settings file differs from the running settings in a
    /// security key. A file that cannot be read or parsed counts as
    /// different: the reload will refuse it anyway.
    fn reload_is_security(&self) -> bool {
        let Some(file) = &self.settings_file else {
            return true;
        };
        let new = std::fs::read_to_string(file)
            .map_err(|e| e.to_string())
            .and_then(|text| chora_kit::settings::parse::<Config>(&text))
            .map(|c| serde_json::to_value(c.localized()));
        let old = serde_json::to_value(&self.config);
        let (Ok(Ok(new)), Ok(old)) = (new, old) else {
            return true;
        };
        let keys = new.as_object().into_iter().flatten().map(|(k, _)| k);
        keys.filter(|k| crate::config::security_key(k))
            .any(|k| new[k] != old[k])
    }

    /// Asks the user when a rule of `confirm` matches, and before each
    /// change of the agent's own settings; else, or on a no, the reason
    /// for the LLM.
    async fn confirmed(
        &mut self,
        function: &str,
        args: &Value,
        out: &(impl Fn(Event) + Sync),
    ) -> Result<(), String> {
        let Some(name) = self.tools.mcp_name(function).map(str::to_owned) else {
            return Ok(());
        };
        let own = self.tools.own_settings_set(&name, args);
        // A security setting takes only a key press (`confirm_keys`). A
        // reload takes it when the file differs from the running
        // settings in a security key: the LLM may have written the file
        // (review rc3 round 3, 2.4).
        let reload = self.tools.own_settings_reload(&name, args) && self.reload_is_security();
        let physical = reload
            || own
                .and_then(Value::as_object)
                .is_some_and(|p| p.keys().any(|k| crate::config::security_key(k)));
        let detail = detail(&name, own, args);
        let prompt = if let Some(patch) = own {
            self.config.settings_question(patch)
        } else if let Some(rule) = tools::rule(&self.config.confirm, &name, args) {
            let (prompt, kinds) = (rule.question(&name, args), rule.commands.clone());
            if name == "apply_commands" && self.removes_own(&kinds, args).await {
                crate::log::info("confirm apply_commands: own nodes, no question");
                return Ok(());
            }
            prompt
        } else {
            return Ok(());
        };
        let key = session_key(&name, own, args);
        if own.is_none() && !reload && listed(&self.config, &name, args) {
            crate::log::info(&format!("confirm {name}: in allow, no question"));
            return Ok(());
        }
        if key.as_ref().is_some_and(|k| self.allowed.contains(k)) {
            crate::log::info(&format!("confirm {name}: allowed for the session"));
            return Ok(());
        }
        // New words of the user come first: the question would answer
        // what they no longer ask; the next round hears them.
        if self.take_inbox() {
            crate::log::info(&format!(
                "confirm {name}: skipped, the user said something new"
            ));
            return Err(SAID_NEW.into());
        }
        let (result, always) = self.ask(prompt, detail, physical, out).await;
        crate::log::info(&format!(
            "confirm {name}: {}",
            result.as_ref().map_or_else(String::as_str, |()| "yes")
        ));
        // A call without a session key takes an always word as a single
        // yes.
        if let (Ok(()), true, Some(key)) = (&result, always, key) {
            crate::log::info(&format!("confirm {name}: allowed for the session as {key}"));
            self.allowed.insert(key);
            out(Event::Sentence(self.config.phrases.allowed.clone()));
        }
        result
    }

    /// Asks the user and waits for the answer; answers whether it had
    /// an always word.
    async fn ask(
        &self,
        prompt: String,
        detail: String,
        physical: bool,
        out: &(impl Fn(Event) + Sync),
    ) -> (Result<(), String>, bool) {
        let (answer, rx) = oneshot::channel();
        let always = Arc::new(AtomicBool::new(false));
        self.tools.physical.store(physical, Ordering::Relaxed);
        out(Event::Confirm {
            prompt,
            detail,
            always: always.clone(),
            physical,
            answer,
        });
        let wait = Duration::from_millis(self.config.confirm_timeout_ms.into());
        // The core times the answer from the end of the spoken question
        // and drops it then; this is only a backstop.
        let backstop = wait + Duration::from_secs(60);
        // The agent itself asks: the LLM must not look for a dialog.
        let result = match tokio::time::timeout(backstop, rx).await {
            Ok(Ok(Some(true))) => Ok(()),
            Ok(Ok(Some(false))) => Err("The user declined the spoken confirmation".to_owned()),
            Ok(Ok(None)) => Err("The answer to the spoken confirmation was not understood".into()),
            Ok(Err(_)) | Err(_) => Err(format!(
                "The user did not answer the spoken confirmation in time ({} s)",
                wait.as_secs()
            )),
        };
        self.tools.physical.store(false, Ordering::Relaxed);
        (result, always.load(Ordering::Relaxed))
    }

    /// Whether each command of `kinds` (each command when empty) in the
    /// `apply_commands` of `args` removes a node of the agent's own client
    /// with only its own nodes under it, by the scene of `query_scene`.
    /// Any doubt answers false: the question stays.
    async fn removes_own(&mut self, kinds: &[String], args: &Value) -> bool {
        let mut targets = Vec::new();
        for command in args["commands"].as_array().into_iter().flatten() {
            for (kind, body) in command.as_object().into_iter().flatten() {
                if !kinds.is_empty() && !kinds.contains(kind) {
                    continue;
                }
                match (kind.as_str(), body["id"].as_str()) {
                    ("remove", Some(id)) => targets.push(id.to_owned()),
                    _ => return false,
                }
            }
        }
        if targets.is_empty() {
            return false;
        }
        let scene = self
            .tools
            .call("query_scene", &json!({"selector": ""}))
            .await;
        let nodes: Vec<Value> = scene
            .as_str()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();
        let node = |id: &str| nodes.iter().find(|n| n["id"] == id);
        let own = |n: &Value| n["owner"] == self.name.as_str();
        // ponytail: a walk up from each node, O(nodes × depth); fine for
        // the scenes of a desktop.
        let under = |n: &Value, target: &str| {
            let mut at = n["parent"].as_str();
            for _ in 0..nodes.len() {
                match at {
                    Some(id) if id == target => return true,
                    Some(id) => at = node(id).and_then(|p| p["parent"].as_str()),
                    None => return false,
                }
            }
            false
        };
        targets
            .iter()
            .all(|t| node(t).is_some_and(own) && nodes.iter().all(|n| own(n) || !under(n, t)))
    }

    /// Drops the oldest whole turns until the estimate fits `budget` and
    /// the history fits `history_tokens` (characters / 4); a turn starts
    /// at a user message, so a tool call keeps its results. The last turn
    /// stays (dump 2026-10-31: 220 messages of cut answers, and the LLM
    /// copied them).
    fn fit(&mut self, budget: u32, fixed: usize) {
        let limit = match self.config.history_tokens {
            0 => usize::MAX,
            n => n as usize * 4,
        };
        loop {
            let history = size(&self.history);
            #[allow(clippy::cast_precision_loss)]
            let tokens = (fixed + history) as f64 / self.chars_per_token;
            if tokens <= f64::from(budget) && history <= limit {
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
    /// The answer stopped mid-sentence; its last words did not go out,
    /// and it does not go into the history.
    cut: bool,
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

/// The arguments and the result of a tool call for the journal,
/// without secrets: of `read` only the path and the byte count, and no
/// value of a key such as `llm_headers` or `tts_api_key_file`.
fn journal(tool: &str, args: &Value, result: &str) -> (String, String) {
    if tool == "read" {
        let path = args["path"].as_str().unwrap_or_default();
        return (path.to_owned(), format!("{} bytes", result.len()));
    }
    let mut args = args.clone();
    hide(&mut args);
    // A result that is JSON, such as the answer of `settings.set`.
    let result = serde_json::from_str::<Value>(result).map_or_else(
        |_| result.to_owned(),
        |mut v| {
            hide(&mut v);
            v.to_string()
        },
    );
    (args.to_string(), short(&result, 300))
}

/// Hides the value of each key such as `llm_headers`, `token`,
/// `authorization` or one with `api_key`, `secret` or `password`.
fn hide(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let key = key.to_lowercase().replace('-', "_");
                let secret = ["api_key", "apikey", "secret", "password"];
                if key.ends_with("_headers")
                    || key.ends_with("token")
                    || key == "authorization"
                    || secret.iter().any(|s| key.contains(s))
                {
                    *value = "(hidden)".into();
                } else {
                    hide(value);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(hide),
        _ => {}
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
                    Event::Heard(s) => format!("heard {s}"),
                    Event::Confirm { prompt, answer, .. } => {
                        let _ = answer.send(Some(yes));
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
            // The LLM copied the note of a cancelled call.
            Reply::Text("Gut. Ich warte. cancelled by the user; result unknown"),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let stop = Duration::from_millis(150);
        for text in ["Schlaf ein.", "Warte."] {
            let cut = tokio::time::timeout(stop, turn(&mut chat, text, true)).await;
            assert!(cut.is_err(), "the turn hangs in the tool, then in the LLM");
        }
        chat.cancelled("Ich warte. Auf de");
        let (report, events) = turn(&mut chat, "Und?", true).await;
        assert_eq!(
            events,
            ["thinking", "say Gut.", "say Ich warte."],
            "no note"
        );
        assert_eq!(report.reply, "Gut. Ich warte. ");
        let seen = server.seen.lock().unwrap();
        let messages = &seen.last().unwrap()["messages"].as_array().unwrap()[1..];
        let roles: Vec<_> = messages
            .iter()
            .map(|m| m["role"].as_str().unwrap())
            .collect();
        assert_eq!(
            roles,
            [
                "user",
                "assistant",
                "tool",
                "user",
                "assistant",
                "user",
                "user"
            ]
        );
        assert_eq!(messages[1]["tool_calls"][0]["function"]["name"], "sleep");
        assert_eq!(messages[2]["content"], CANCELLED);
        assert_eq!(
            messages[4]["content"], "Ich warte.",
            "the whole played sentences"
        );
        assert_eq!(messages[5]["content"], STOPPED);
        assert_eq!(chat.history.last().unwrap()["content"], "Gut. Ich warte.");
    }

    #[test]
    fn a_stopped_answer_keeps_only_whole_sentences_or_words() {
        assert_eq!(complete("Gut. Die Blö"), "Gut.");
        assert_eq!(
            complete("Ich mache die Blöcke kräftiger:"),
            "Ich mache die Blöcke"
        );
        assert_eq!(complete("I'm not stuck, I"), "I'm not stuck");
        assert_eq!(complete("Fertig!"), "Fertig!");
        assert_eq!(complete("Aktual"), "");
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
                "say One moment.",
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
            system.starts_with("Du bist ein Test.\n\n# File ")
                && system.ends_with(
                    "# Language\n\nAlways answer in English, even when the user \
                                     speaks another language. Change the language only when \
                                     the user asks for it."
                ),
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
        let acks = events.iter().filter(|e| *e == "say One moment.").count();
        assert_eq!(acks, 1, "one per turn");
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
            "say I stopped after too many tool calls."
        );
    }

    /// Answers a confirmation with `answer` (yes, no or unclear), or
    /// drops it.
    fn answer_with(event: Event, answer: Result<Option<bool>, ()>) {
        if let Event::Confirm { answer: tx, .. } = event
            && let Ok(a) = answer
        {
            let _ = tx.send(a);
        }
    }

    /// Runs the turn "Echo." with an always yes to each question;
    /// answers what it said.
    async fn always(chat: &mut Chat) -> Vec<String> {
        let events = Mutex::new(Vec::new());
        chat.turn("Echo.", &|event| {
            if let Event::Confirm { always, answer, .. } = event {
                always.store(true, Ordering::SeqCst);
                let _ = answer.send(Some(true));
            } else if let Event::Sentence(s) = event {
                events.lock().unwrap().push(s);
            }
        })
        .await;
        events.into_inner().unwrap()
    }

    /// An always word allows the same method with the same parameters
    /// until the agent restarts; another tool asks again.
    #[tokio::test]
    async fn an_always_yes_allows_the_action_for_the_session() {
        let args = r#"{"layout": "grid"}"#;
        let script = [
            Reply::Tool("tiling__set_layout", args),
            Reply::Text("Gut."),
            Reply::Tool("tiling__set_layout", args),
            Reply::Text("Gut."),
            Reply::Tool("echo", "{}"),
            Reply::Text("Gut."),
            Reply::Tool("echo", "{}"),
            Reply::Text("Gut."),
        ];
        let server = fake::llm(script.into()).await;
        let (tools, calls) = tools::tests::fake(0);
        let config = Config {
            llm_url: server.url.clone(),
            confirm: ["tiling.set_layout", "echo"]
                .map(|tool| Confirm {
                    tool: tool.into(),
                    commands: Vec::new(),
                    argv0: Vec::new(),
                    method: None,
                    prompt: "Wirklich?".into(),
                })
                .into(),
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        let said = always(&mut chat).await;
        assert!(
            said.contains(&"Allowed for this session.".to_owned()),
            "{said:?}"
        );
        let (_, events) = turn(&mut chat, "Noch mal.", false).await;
        assert!(
            !events.iter().any(|e| e.starts_with("confirm")),
            "{events:?}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        // Review rc3 round 2, 1.4: a tool such as inject_input takes an
        // always word as a single yes.
        let said = always(&mut chat).await;
        assert!(!said.iter().any(|s| s.starts_with("Allowed")), "{said:?}");
        let (_, events) = turn(&mut chat, "Noch mal.", false).await;
        assert!(
            events.contains(&"confirm Wirklich?".to_owned()),
            "{events:?}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    /// Live log 2026-10-31: a call that fails the same way twice gets a
    /// note to stop, and a third one does not run. The error reaches the
    /// LLM whole.
    #[tokio::test]
    async fn a_call_that_fails_twice_the_same_way_stops() {
        let script = vec![
            Reply::Tool(
                "echo",
                r#"{"fail": 1, "commands": [{"create": {"id": "a"}}]}"#,
            ),
            Reply::Tool(
                "echo",
                r#"{"fail": 1, "commands": [{"create": {"id": "b"}}]}"#,
            ),
            Reply::Tool(
                "echo",
                r#"{"fail": 1, "commands": [{"create": {"id": "c"}}]}"#,
            ),
            Reply::Text("Das ging nicht."),
        ];
        let server = fake::llm(script).await;
        let (tools, calls) = tools::tests::fake(0);
        let config = Config {
            llm_url: server.url.clone(),
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        turn(&mut chat, "Mach einen Knoten.", true).await;
        assert_eq!(calls.load(Ordering::SeqCst), 2, "the third does not run");
        let results: Vec<String> = (chat.history.iter())
            .filter(|m| m["role"] == "tool")
            .map(|m| m["content"].as_str().unwrap().to_owned())
            .collect();
        let error = format!("error: {}", tools::tests::FAIL);
        assert_eq!(results[0], error, "whole");
        assert_eq!(results[1], format!("{error}\n{FAILED_TWICE}"));
        assert_eq!(results[2], format!("error: not run. {FAILED_TWICE}"));
    }

    /// Review rc3 round 2, 1.3: new settings, or the same ones read
    /// again, end the always words of the session.
    #[tokio::test]
    async fn settings_end_the_always_words() {
        let server = fake::llm(Vec::new()).await;
        let (tools, _) = tools::tests::fake(0);
        let config = Config {
            llm_url: server.url.clone(),
            ..Config::default()
        };
        let mut chat = Chat::start(config.clone(), "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        chat.allowed.insert("mc.app.open {}".into());
        chat.configure(config).unwrap();
        assert!(chat.allowed.is_empty());
    }

    /// A changed `chora_mcp` ends the running tool process: the next
    /// refresh starts the new command (live 2026-10-31: tools kept
    /// running on the host after the fix).
    #[tokio::test]
    async fn a_new_chora_mcp_restarts_the_tool_process() {
        let server = fake::llm(Vec::new()).await;
        let (tools, calls) = tools::tests::fake(0);
        let config = Config {
            llm_url: server.url.clone(),
            ..Config::default()
        };
        let mut chat = Chat::start(config.clone(), "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        chat.configure(config.clone()).unwrap();
        assert_eq!(chat.tools.call("echo", &json!({})).await, "{}");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let new = Config {
            chora_mcp: vec!["/nonexistent/choractl".into()],
            ..config
        };
        chat.configure(new).unwrap();
        let answer = chat.tools.call("echo", &json!({})).await;
        assert!(answer.as_str().unwrap().contains("not connected"), "{answer}");
    }

    /// Review rc3 round 2, 1.4 and 4.1: "always" covers the same full
    /// argv or the same method with the same parameters; settings
    /// changes and the other tools take it as a single yes.
    #[test]
    fn what_always_covers() {
        let key =
            |tool, patch: Option<Value>, args: Value| session_key(tool, patch.as_ref(), &args);
        let patch = |k: &str| Some(json!({ k: 1 }));
        for k in ["barge_in_ms", "allow_programs", "llm_url", "confirm"] {
            assert_eq!(key("x", patch(k), json!({})), None, "{k}");
        }
        for tool in [
            "inject_input",
            "apply_commands",
            "code_task",
            "agent_start",
            "configure_output",
        ] {
            assert_eq!(key(tool, None, json!({"text": "ls"})), None, "{tool}");
        }
        let run = json!({"argv": ["/usr/bin/foot", "-e", "sh"]});
        assert_eq!(
            key("run_program", None, run).as_deref(),
            Some(r#"run_program ["/usr/bin/foot","-e","sh"]"#)
        );
        assert_ne!(
            key("run_program", None, json!({"argv": ["foot"]})),
            key("run_program", None, json!({"argv": ["foot", "-e", "sh"]}))
        );
    }

    /// Review rc3 round 2, 1.4: a method is keyed with its canonical
    /// parameters; settings methods take an always word as a single yes.
    #[test]
    fn always_keys_methods_with_their_parameters() {
        let key =
            |tool, patch: Option<Value>, args: Value| session_key(tool, patch.as_ref(), &args);
        for method in ["settings.set", "settings.reload"] {
            let call = json!({"client": "xp", "method": method});
            assert_eq!(key("call_method", None, call), None);
            let tool = format!("xp.{method}");
            assert_eq!(session_key(&tool, None, &json!({})), None);
        }
        let open = |params: Value| json!({"client": "mc", "method": "app.open", "params": params});
        let one = key("call_method", None, open(json!({"app": "foot", "at": 1})));
        assert_eq!(one.as_deref(), Some(r#"mc.app.open {"app":"foot","at":1}"#));
        // The parameters are canonical: their order does not count.
        let other: Value = serde_json::from_str(r#"{"at": 1, "app": "foot"}"#).unwrap();
        assert_eq!(key("call_method", None, open(other)), one);
        assert_eq!(
            key("mc.app.open", None, json!({"at": 1, "app": "foot"})),
            one
        );
        assert_ne!(key("call_method", None, open(json!({"app": "sh"}))), one);
    }

    /// Review rc3 round 2, 1.2: a program matches its bare name only, and
    /// programs and methods are separate lists.
    #[test]
    fn allow_lists_programs_and_methods_apart() {
        let config = Config {
            allow_programs: vec!["foot".into()],
            allow_methods: vec!["mc.app.open".into()],
            ..Config::default()
        };
        let run = |argv0: &str| json!({"argv": [argv0]});
        let open = json!({"client": "mc", "method": "app.open"});
        assert!(listed(&config, "run_program", &run("foot")));
        assert!(listed(&config, "call_method", &open));
        assert!(listed(&config, "mc.app.open", &json!({})));
        assert!(!listed(&config, "mc.scene.save", &json!({})));
        for argv0 in [
            "/usr/bin/foot",
            "./foot",
            "/home/u/proj/foot",
            "sh",
            "mc.app.open",
        ] {
            assert!(!listed(&config, "run_program", &run(argv0)), "{argv0}");
        }
        let foot = json!({"client": "x", "method": "foot"});
        assert!(!listed(&config, "call_method", &foot));
    }

    /// Review rc3 round 3, 2.3: a bare entry allows the program with no
    /// argument; an entry with arguments matches the whole argv.
    #[test]
    fn allow_programs_matches_the_whole_argv() {
        let config = Config {
            allow_programs: vec!["foot".into(), "xterm -e htop".into()],
            ..Config::default()
        };
        let ok = |argv: Value| listed(&config, "run_program", &json!({ "argv": argv }));
        assert!(ok(json!(["foot"])));
        assert!(!ok(json!(["foot", "-e", "sh", "-c", "curl x|sh"])));
        assert!(!ok(json!(["foot", "-e"])));
        assert!(ok(json!(["xterm", "-e", "htop"])));
        assert!(!ok(json!(["xterm", "-e", "htop", "x"])));
        assert!(!ok(json!(["xterm", "-e", "sh"])));
        assert!(!ok(json!(["xterm"])));
    }

    #[tokio::test]
    async fn a_declined_call_does_not_run() {
        let script = [Reply::Tool("echo", "{}"), Reply::Text("Gut.")].repeat(4);
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
        assert_eq!(events[2], "confirm Wirklich?");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            chat.history[2]["content"],
            "The user declined the spoken confirmation; nothing happened"
        );
        let (_, events) = turn(&mut chat, "Echo.", true).await;
        assert_eq!(events[2..4], ["confirm Wirklich?", "tool echo"]);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        // Not understood, and no answer: the LLM learns which.
        for (answer, says) in [
            (
                Ok(None),
                "The answer to the spoken confirmation was not understood",
            ),
            (
                Err(()),
                "The user did not answer the spoken confirmation in time (0 s)",
            ),
        ] {
            chat.config.confirm_timeout_ms = 50;
            // A dropped answer counts as none, as a timeout.
            chat.turn("Echo.", &|event| answer_with(event, answer))
                .await;
            let last = chat.history.iter().rev().find(|m| m["role"] == "tool");
            assert_eq!(
                last.unwrap()["content"],
                format!("{says}; nothing happened")
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    /// A remove of the agent's own nodes needs no question; one of a
    /// node of another client, or over one, does.
    #[tokio::test]
    async fn the_agent_removes_its_own_nodes_without_a_question() {
        let (mut chat, _) = chat(vec![], Config::default()).await;
        let kinds = ["remove".to_owned(), "close".to_owned()];
        let remove = |id: &str| json!({"remove": {"id": id}});
        let create = json!({"create": {"id": "voice-c"}});
        for (commands, own) in [
            (json!([remove("voice-a")]), true),
            (json!([create, remove("voice-a")]), true),
            (json!([remove("voice-g")]), false),
            (json!([remove("mc-r0-drawer")]), false),
            (json!([remove("gone")]), false),
            (json!([{"close": {"surface": "voice-a"}}]), false),
            (json!([create]), false),
        ] {
            let args = json!({ "commands": commands });
            assert_eq!(chat.removes_own(&kinds, &args).await, own, "{args}");
        }
    }

    /// An answer that announces a tool but calls none gets one more
    /// round (log 2026-10-31: "let me use `put_asset` instead", then the
    /// turn ended); a plain "let me know" does not.
    #[tokio::test]
    async fn an_announced_tool_without_a_call_gets_one_more_round() {
        let script = vec![
            Reply::Text("The picture tool works too, let me use that instead."),
            Reply::Tool("echo", "{}"),
            Reply::Text("Fertig."),
            Reply::Text("Let me know if you need more."),
            Reply::Tool("echo", "{}"),
            Reply::Text("Let me check the scene."),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let (report, events) = turn(&mut chat, "Mach was.", true).await;
        assert_eq!((report.rounds, report.tools), (3, 1), "{events:?}");
        assert_eq!(last_user(&server, 1), NUDGE);
        let (report, _) = turn(&mut chat, "Danke.", true).await;
        assert_eq!(report.rounds, 1);
        // Review 0.9.0-rc.3: a turn that ran a tool gets no nudge.
        let (report, events) = turn(&mut chat, "Noch mal.", true).await;
        assert_eq!((report.rounds, report.tools), (2, 1), "{events:?}");
    }

    #[test]
    fn first_person_announcements_count_without_a_tool_name() {
        for text in [
            "Okay. I'll open the editor.",
            "I will start the loader.",
            "Let me check the scene.",
            "I'm going to open the file.",
            "Gut, ich werde das Fenster schließen.",
            "Lass mich die Ränder prüfen.",
            "Ich schaue kurz, was mit den Rändern los ist.",
            "Ich schaue nach.",
            "Ich schaue mal in die Notizen.",
            "I'll check the margins.",
        ] {
            assert!(announces(text), "{text}");
        }
        for text in [
            "Let me know if you need more.",
            "Let me know.",
            "Ich habe den Loader gestartet.",
            "I'll start it. Done, the loader runs.",
            "Willst du, dass ich starte?",
            "Dich starte ich nicht.",
            "Ich bin bereit.",
            "Ich habe das erledigt.",
            "Ich kann das nicht.",
            "Let me know if the margins come back.",
            "Ja, ich höre dich.",
            "Ich schaue nach, ja?",
            "I can't open it.",
            "I did open it.",
            "I will not open it.",
            "Sag Bescheid, wenn ich weitermachen soll.",
            // Closing lines (review 0.9.0-rc.3).
            "Ich helfe dir gern.",
            "Ich heiße Chora.",
            "Ich empfehle eine Skalierung von 1,5.",
            "I'll be here if you need anything.",
            "Ich zeige Fenster, starte Programme und lese Dateien.",
        ] {
            assert!(!announces(text), "{text}");
        }
    }

    #[test]
    fn answers_that_stop_mid_sentence_are_cut() {
        for text in [
            "I'll set the scale to 1.5 and switch",
            "I’m setting the",
            "I’ll apply",
            "Hier sind die Optionen:",
        ] {
            assert!(cut(text), "{text}");
        }
        for text in [
            "",
            "Fertig.",
            "Gute Nacht! ",
            "Soll ich?",
            "Er sagte „Ja.“",
            "Done.\"",
            "Na ja…",
            "**Fertig.**",
        ] {
            assert!(!cut(text), "{text}");
        }
    }

    /// An answer cut by the server gets one more round; its last words
    /// are not spoken, and the sentences spoken before it are not spoken
    /// again (dump 2026-10-31).
    #[tokio::test]
    async fn a_cut_answer_gets_one_more_round() {
        let script = vec![
            Reply::Text("I'll set the scale to 1.5 and switch"),
            Reply::Text("I'll set the scale to 1.5 and switch the theme to light."),
            Reply::Text("Okay. I’m setting the"),
            Reply::Text("Okay. I’m setting the theme now."),
            Reply::Text("I’ll apply"),
            Reply::Text("Done, the scale is 1.5."),
        ];
        let (mut chat, _server) = chat(script, Config::default()).await;
        let said = |events: &[String]| -> Vec<String> {
            events
                .iter()
                .filter(|e| e.starts_with("say "))
                .cloned()
                .collect()
        };
        let (report, events) = turn(&mut chat, "Stelle das Scaling auf 1,5.", true).await;
        assert_eq!(report.rounds, 2, "{events:?}");
        assert_eq!(
            said(&events),
            ["say I'll set the scale to 1.5 and switch the theme to light."]
        );
        let (_, events) = turn(&mut chat, "Das ist noch nicht passiert.", true).await;
        assert_eq!(
            said(&events),
            ["say Okay.", "say I’m setting the theme now."]
        );
        let (_, events) = turn(&mut chat, "Bitte umsetzen.", true).await;
        assert_eq!(said(&events), ["say Done, the scale is 1.5."]);
    }

    /// The words of the turn that went out as sentences.
    fn said(events: &[String]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|e| e.strip_prefix("say "))
            .collect()
    }

    /// The fragment of a cut answer is never spoken nor kept, and gets
    /// no nudge (log 2026-10-31: "cut answer, held \"I’ll send Alt\"",
    /// then "say: I’ll send Alt" and a nudge).
    #[tokio::test]
    async fn a_cut_fragment_is_not_spoken() {
        let script = vec![
            Reply::Text("I’ll send Alt"),
            Reply::Text("I’ll send Alt"),
            Reply::Text("Sent."),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let (report, events) = turn(&mut chat, "Schick Alt+Tab.", true).await;
        assert_eq!(
            (report.rounds, report.outcome),
            (3, "answered"),
            "{events:?}"
        );
        assert_eq!(said(&events), ["Sent."]);
        let texts: Vec<&Value> = chat.history.iter().map(|m| &m["content"]).collect();
        assert_eq!(texts, ["Schick Alt+Tab.", "Sent."]);
        let seen = server.seen.lock().unwrap();
        assert!(
            !seen
                .iter()
                .any(|s| s.to_string().contains("announced an action"))
        );
    }

    /// The round after a cut answer asks with a note of its own, which
    /// does not stay in the history: the same request gave the same
    /// fragment (dump 2026-10-31).
    #[tokio::test]
    async fn the_retry_of_a_cut_answer_asks_with_a_note() {
        let script = vec![Reply::Text("Because"), Reply::Text("Because it is late.")];
        let (mut chat, server) = chat(script, Config::default()).await;
        let (_, events) = turn(&mut chat, "Warum?", true).await;
        assert_eq!(said(&events), ["Because it is late."]);
        assert_eq!(last_user(&server, 0), "Warum?");
        assert_eq!(last_user(&server, 1), RETRY);
        assert!(chat.history.iter().all(|m| m["content"] != RETRY));
    }

    /// After two cut answers in a row, the third round has only the last
    /// words of the user; a third cut says `phrases.lost` and ends the
    /// turn.
    #[tokio::test]
    async fn two_cut_answers_pull_the_emergency_brake() {
        let script = vec![
            Reply::Text("Eins."),
            Reply::Text("The"),
            Reply::Text("I can't"),
            Reply::Text("The"),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        turn(&mut chat, "Zähl.", true).await;
        let (report, events) = turn(&mut chat, "Weiter.", true).await;
        assert_eq!((report.rounds, report.outcome), (3, "cut"), "{events:?}");
        assert_eq!(said(&events), [Config::default().phrases.lost.as_str()]);
        let seen = server.seen.lock().unwrap();
        let contents = |i: usize| -> Vec<String> {
            let messages = seen[i]["messages"].as_array().unwrap();
            messages
                .iter()
                .map(|m| m["content"].as_str().unwrap_or_default().into())
                .collect()
        };
        assert_eq!(contents(2).len(), 5, "{:?}", contents(2));
        assert_eq!(contents(3)[1..], ["Weiter.", RETRY]);
        assert_eq!(chat.history.len(), 1);
    }

    /// A German announcement without a tool name gets one more round,
    /// and only one per turn (log 2026-10-31, after "Du hast nichts
    /// gemacht.").
    #[tokio::test]
    async fn a_german_announcement_gets_one_more_round_once() {
        let script = vec![
            Reply::Text("Ich werde den Loader starten."),
            Reply::Text("Lass mich den Loader starten."),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let (report, events) = turn(&mut chat, "Du hast nichts gemacht.", true).await;
        assert_eq!((report.rounds, report.tools), (2, 0), "{events:?}");
        assert_eq!(last_user(&server, 1), NUDGE);
    }

    /// Words of the user during the last round get rounds of their own
    /// in the same turn, with tools (log 2026-10-31: "hallo, hast du
    /// mich?" waited 49 s for the end of a reply at the round limit).
    #[tokio::test]
    async fn words_during_the_last_round_get_rounds_of_their_own() {
        let script = vec![
            Reply::Tool("echo", "{}"),
            Reply::Text("Ich habe es versucht."),
            Reply::Tool("echo", "{}"),
            Reply::Text("Ja, ich höre dich."),
        ];
        let config = Config {
            tool_rounds: 1,
            ..Config::default()
        };
        let (mut chat, server) = chat(script, config).await;
        let events = interrupted(&mut chat, "say Ich habe es versucht.", "Hast du mich?").await;
        assert_eq!(
            events,
            [
                "thinking",
                "say One moment.",
                "tool echo",
                "thinking",
                "say Ich habe es versucht.",
                "heard Hast du mich?",
                "thinking",
                "tool echo",
                "thinking",
                "say Ja, ich höre dich."
            ]
        );
        assert_eq!(server.seen.lock().unwrap().len(), 4);
        assert_eq!(last_user(&server, 2), "Hast du mich?");
    }

    /// Runs a turn whose user says `said` when `when` matches an event;
    /// answers each confirmation with yes.
    async fn interrupted(chat: &mut Chat, when: &str, said: &str) -> Vec<String> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        chat.inbox = Some(rx);
        let events = Mutex::new(Vec::new());
        let sent = std::sync::atomic::AtomicBool::new(false);
        chat.turn("Mach was.", &|event| {
            let line = match event {
                Event::Thinking => "thinking".into(),
                Event::Tool(name) => format!("tool {name}"),
                Event::Sentence(s) => format!("say {s}"),
                Event::Heard(s) => format!("heard {s}"),
                Event::Confirm { prompt, answer, .. } => {
                    let _ = answer.send(Some(true));
                    format!("confirm {prompt}")
                }
            };
            if line == when && !sent.swap(true, Ordering::SeqCst) {
                tx.send(said.into()).unwrap();
            }
            events.lock().unwrap().push(line);
        })
        .await;
        events.into_inner().unwrap()
    }

    /// The last user message of request `i` to the LLM.
    fn last_user(server: &fake::Server, i: usize) -> Value {
        let seen = server.seen.lock().unwrap();
        let messages = seen[i]["messages"].as_array().unwrap();
        let user = messages.iter().rev().find(|m| m["role"] == "user");
        user.unwrap()["content"].clone()
    }

    /// Words of the user while the turn runs go into its next round,
    /// after a tool call and after an answer (log 2026-10-31: they waited
    /// up to 84 s for the end of the reply).
    #[tokio::test]
    async fn words_during_a_turn_go_into_its_next_round() {
        let script = vec![
            Reply::Tool("echo", "{}"),
            Reply::Text("Gut."),
            Reply::Text("Eins."),
            Reply::Text("Zwei."),
        ];
        let (mut chat, server) = chat(script, Config::default()).await;
        let events = interrupted(&mut chat, "tool echo", "Und größer.").await;
        assert_eq!(
            events,
            [
                "thinking",
                "say One moment.",
                "tool echo",
                "heard Und größer.",
                "thinking",
                "say Gut."
            ]
        );
        assert_eq!(last_user(&server, 1), "Und größer.");
        let events = interrupted(&mut chat, "say Eins.", "Nein, rot.").await;
        assert_eq!(
            events,
            [
                "thinking",
                "say Eins.",
                "heard Nein, rot.",
                "thinking",
                "say Zwei."
            ]
        );
        assert_eq!(last_user(&server, 3), "Nein, rot.");
    }

    /// New words of the user before a question skip it: the call does
    /// not run, the LLM learns why and hears the words.
    #[tokio::test]
    async fn new_words_skip_the_question_of_a_call() {
        let script = vec![Reply::Tool("echo", "{}"), Reply::Text("Gut.")];
        let config = Config {
            confirm: vec![Confirm {
                tool: "echo".into(),
                commands: Vec::new(),
                argv0: Vec::new(),
                method: None,
                prompt: "Wirklich?".into(),
            }],
            ..Config::default()
        };
        let (mut chat, server) = chat(script, config).await;
        let events = interrupted(&mut chat, "thinking", "Lass das.").await;
        assert!(
            !events.iter().any(|e| e.starts_with("confirm")),
            "{events:?}"
        );
        assert!(!events.contains(&"tool echo".to_owned()), "{events:?}");
        let result = chat.history.iter().find(|m| m["role"] == "tool").unwrap();
        assert_eq!(result["content"], format!("{SAID_NEW}; nothing happened"));
        assert_eq!(last_user(&server, 1), "Lass das.");
    }

    /// Each change of the agent's own settings asks with its key and
    /// value; a security setting says so (log 2026-10-31: "set the
    /// barge-in RMS to 15").
    #[tokio::test]
    async fn a_change_of_the_own_settings_asks_with_key_and_value() {
        let script = vec![
            Reply::Tool(
                "voice__settings__set",
                r#"{"patch": {"barge_in_min_rms": 15}}"#,
            ),
            Reply::Tool(
                "voice__settings__set",
                r#"{"patch": {"llm_url": "http://x/v1"}}"#,
            ),
            Reply::Text("Erledigt."),
        ];
        let server = fake::llm(script).await;
        let (mut tools, calls) = tools::tests::fake(0);
        tools.own = "voice".into();
        let config = Config {
            llm_url: server.url.clone(),
            confirm: Vec::new(),
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        assert!(chat.system.contains("voice.settings.set changes keys"));
        let (_, events) = turn(&mut chat, "Setz den Barge-in auf 15.", true).await;
        let asked: Vec<_> = events.iter().filter(|e| e.starts_with("confirm")).collect();
        assert_eq!(
            asked,
            [
                "confirm Shall I set barge in min rms to 15?",
                "confirm Security setting! Shall I set llm url? The full value is on the screen. \
                 Press Alt+y for yes or Alt+n for no."
            ]
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    /// Review 0.9.0-rc.3: the journal gets no secret of a tool call.
    #[test]
    fn the_journal_hides_secrets() {
        let (args, result) = journal("read", &json!({"path": "proj/.env"}), "KEY=sk-1\n");
        assert_eq!((args.as_str(), result.as_str()), ("proj/.env", "9 bytes"));
        let set = json!({"patch": {"llm_headers": {"X-Key": "sk-2"}, "llm_api_key_file": "/k",
            "asr_url": "http://x", "list": [{"token": "t"}]}});
        let (args, _) = journal("voice__settings__set", &set, "ok");
        for secret in ["sk-2", "/k", "\"t\""] {
            assert!(!args.contains(secret), "{args}");
        }
        assert!(args.contains("http://x"), "{args}");
        let answer = r#"{"llm_headers":{"X-Key":"sk-3"},"asr_url":"http://x"}"#;
        let (_, result) = journal("voice__settings__set", &set, answer);
        assert!(!result.contains("sk-3") && result.contains("http://x"), "{result}");
    }

    /// Review rc3 round 2, 4.4, the user's decision: a security setting
    /// asks for a key press, never for a spoken yes.
    #[tokio::test]
    async fn a_security_setting_asks_for_a_key() {
        let script = vec![
            Reply::Tool(
                "voice__settings__set",
                r#"{"patch": {"allow_programs": ["sh"]}}"#,
            ),
            Reply::Tool("voice__settings__set", r#"{"patch": {"barge_in_ms": 300}}"#),
            Reply::Text("Erledigt."),
        ];
        let server = fake::llm(script).await;
        let (mut tools, _) = tools::tests::fake(0);
        tools.own = "voice".into();
        let config = Config {
            llm_url: server.url.clone(),
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        let asked = Mutex::new(Vec::new());
        let open = chat.tools.physical.clone();
        chat.turn("Erlaube sh.", &|event| {
            if let Event::Confirm {
                physical, answer, ..
            } = event
            {
                asked
                    .lock()
                    .unwrap()
                    .push((physical, open.load(Ordering::SeqCst)));
                let _ = answer.send(Some(true));
            }
        })
        .await;
        assert_eq!(asked.into_inner().unwrap(), [(true, true), (false, false)]);
        assert!(!chat.tools.physical.load(Ordering::SeqCst));
        for key in [
            "indicator",
            "earcons",
            "confirm_timeout_ms",
            "ptt_key",
            "wake_words",
            "sleep_words",
        ] {
            assert!(crate::config::security_key(key), "{key}");
        }
    }

    /// Review rc3 round 3, 2.4: a reload of a file that differs in a
    /// security key asks for a key press; any other reload, a spoken yes.
    #[tokio::test]
    async fn a_reload_with_a_new_security_key_asks_for_a_key() {
        let script = [
            Reply::Tool("voice__settings__reload", "{}"),
            Reply::Text("Erledigt."),
            Reply::Tool("voice__settings__reload", "{}"),
            Reply::Text("Erledigt."),
        ];
        let server = fake::llm(script.to_vec()).await;
        let (mut tools, _) = tools::tests::fake(0);
        tools.own = "voice".into();
        let config = Config {
            llm_url: server.url.clone(),
            ..Config::default()
        };
        let mut chat = Chat::start(config, "voice-mcp".into(), tools, None)
            .await
            .unwrap();
        let file = temp("reload.yaml");
        chat.settings_file = Some(file.clone());
        let asked = Mutex::new(Vec::new());
        let url = chat.config.llm_url.clone();
        for yaml in ["allow_programs: [sh]", "barge_in_ms: 300"] {
            std::fs::write(&file, format!("{{llm_url: '{url}', {}}}", &yaml)).unwrap();
            chat.turn("Lade neu.", &|event| {
                if let Event::Confirm {
                    physical, answer, ..
                } = event
                {
                    asked.lock().unwrap().push(physical);
                    let _ = answer.send(Some(true));
                }
            })
            .await;
        }
        assert_eq!(asked.into_inner().unwrap(), [true, false]);
    }

    /// Review rc3 round 2, 4.2: the screen and the event get no secret.
    #[test]
    fn the_detail_hides_secrets() {
        let patch = json!({"llm_headers": {"Authorization": "Bearer sk-1"}, "llm_url": "http://x"});
        let shown = detail("voice.settings.set", Some(&patch), &json!({}));
        assert!(!shown.contains("sk-1"), "{shown}");
        assert!(shown.contains("llm_headers: (hidden)") && shown.contains("http://x"));
        let call = json!({"client": "mc", "method": "x", "params": {"api_key": "sk-2"}});
        let shown = detail("call_method", None, &call);
        assert!(!shown.contains("sk-2"), "{shown}");
    }

    /// Review 0.9.0-rc.3: a security value is never cut; the question
    /// names the key, and the screen shows the full value.
    #[test]
    fn a_security_value_is_never_cut() {
        let config = Config::default();
        let long = "x".repeat(200);
        for key in [
            "agent_projects",
            "yes_words",
            "no_words",
            "phrases",
            "record_dir",
        ] {
            let patch = json!({ key: long });
            let question = config.settings_question(&patch);
            assert!(question.starts_with("Security setting!"), "{question}");
            assert!(
                !question.contains('x') && !question.contains('…'),
                "{question}"
            );
            assert_eq!(
                detail("voice.settings.set", Some(&patch), &json!({})),
                format!("{key}: {long}")
            );
        }
        let short = config.settings_question(&json!({"barge_in_min_rms": long}));
        assert!(short.contains('…'), "a plain value is cut: {short}");
        let argv = json!({"argv": ["foot", "-e", "sh"]});
        assert_eq!(detail("run_program", None, &argv), "foot -e sh");
        let call = json!({"client": "mc", "method": "app.open", "params": {"app": "foot"}});
        assert_eq!(
            detail("call_method", None, &call),
            r#"mc.app.open {"app":"foot"}"#
        );
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
            context_tokens: 460,
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

    /// `history_tokens` drops the oldest whole turns, each with its tool
    /// calls and their results, also within the context budget (dump
    /// 2026-10-31: 221 messages, 33k tokens).
    #[tokio::test]
    async fn history_tokens_drops_whole_turns_with_their_tool_pairs() {
        let config = Config {
            history_tokens: 150,
            ..Config::default()
        };
        let (mut chat, _server) = chat(Vec::new(), config).await;
        for i in 0..6 {
            let id = format!("call{i}");
            chat.history.extend([
                json!({"role": "user", "content": format!("Bitte {i}.")}),
                json!({"role": "assistant", "content": "", "tool_calls": [
                    {"id": id, "type": "function",
                     "function": {"name": "echo", "arguments": "{}"}}]}),
                json!({"role": "tool", "tool_call_id": id, "content": "x".repeat(100)}),
                json!({"role": "assistant", "content": "Fertig."}),
            ]);
        }
        chat.fit(u32::MAX, 0);
        let history = &chat.history;
        assert!(history.len() < 24 && history.len() >= 4, "{history:?}");
        assert!(size(history) <= 600);
        assert_eq!(history[0]["role"], "user");
        for (i, m) in history.iter().enumerate() {
            if m["role"] == "tool" {
                let id = &m["tool_call_id"];
                let calls = history[..i]
                    .iter()
                    .filter_map(|m| m["tool_calls"].as_array());
                assert!(calls.flatten().any(|c| c["id"] == *id), "{id}");
            }
        }
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
            "say The language model does not answer."
        );
    }

    #[test]
    fn old_tool_texts_shrink_but_the_last_shader_stays() {
        let call = |id: &str, name: &str, args: &str| json!({"id": id, "type": "function", "function": {"name": name, "arguments": args}});
        let big = format!(r#"{{"name": "a.frag", "data": "{}"}}"#, "x".repeat(600));
        let old = format!(r#"{{"name": "b.frag", "data": "{}"}}"#, "y".repeat(600));
        let mut chat = Chat {
            name: String::new(),
            settings_file: None,
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
            inbox: None,
            heard: Vec::new(),
            failed: None,
            allowed: HashSet::new(),
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
