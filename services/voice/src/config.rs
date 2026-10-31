//! The settings of the agent (D62, docs/voice.md section 9). Each key has
//! a default, so a file needs only the keys that differ.

use std::collections::BTreeMap;

use chora_kit::settings::Settings;
use chora_scene::Binding;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A rule of the spoken confirmation: the tool (a glob), and optionally
/// the command kinds of `apply_commands`, the program of `run_program` or
/// the method (a glob) of `call_method`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Confirm {
    pub tool: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argv0: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// What the agent asks; `{program}` stands for the program of
    /// `run_program`.
    pub prompt: String,
}

/// The canned sentences of the agent, in English; `reply_language: de`
/// turns each one that keeps its default into German.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Phrases {
    /// The LLM does not answer.
    pub llm_down: String,
    /// The LLM is too slow.
    pub llm_slow: String,
    /// The final round after `tool_rounds` gave no words.
    pub tool_rounds: String,
    /// The ASR failed for a request.
    pub asr_failed: String,
    /// After the first sentence of a long answer of a sub-agent.
    pub details: String,
    /// The question of a sub-agent: `{name}` and `{what}`.
    pub agent_asks: String,
    /// A sub-agent ended by itself: `{name}`.
    pub agent_ended: String,
}

impl Default for Phrases {
    fn default() -> Self {
        Self {
            llm_down: "The language model does not answer.".into(),
            llm_slow: "The language model takes too long.".into(),
            tool_rounds: "I stopped after too many tool calls.".into(),
            asr_failed: "I did not understand you.".into(),
            details: "Details on the screen.".into(),
            agent_asks: "{name} wants to {what}. Allow?".into(),
            agent_ended: "{name} has ended.".into(),
        }
    }
}

/// The settings of the agent.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// The realtime transcription websocket of the ASR.
    pub asr_url: String,
    /// The served name of the ASR model.
    pub asr_model: String,
    /// The environment variable with the API key of the ASR; empty: none.
    pub asr_api_key_env: String,
    /// The file with the API key of the ASR; `%d` is the credentials
    /// directory of systemd, `~/` the home directory; empty: none.
    pub asr_api_key_file: String,
    /// Extra headers of the ASR requests.
    pub asr_headers: BTreeMap<String, String>,
    /// The language of the speaker, for example `de`; `auto` lets the ASR
    /// find it.
    pub asr_language: String,
    /// The words that the ASR must know; the agent adds the wake, sleep
    /// and stop words and the projects. Empty: no prompt at all.
    pub asr_prompt: String,
    /// The speech websocket of the TTS.
    pub tts_url: String,
    pub tts_model: String,
    pub tts_api_key_env: String,
    pub tts_api_key_file: String,
    pub tts_headers: BTreeMap<String, String>,
    /// A voice of the TTS server.
    pub voice: String,
    /// The language of the TTS, for example `German`; `auto` lets the
    /// model choose; empty follows `reply_language`.
    pub tts_language: String,
    /// The OpenAI-compatible base URL of the LLM.
    pub llm_url: String,
    pub llm_model: String,
    pub llm_api_key_env: String,
    pub llm_api_key_file: String,
    pub llm_headers: BTreeMap<String, String>,
    /// The language of the answers, for example `en` or `de`; `de` also
    /// turns the default phrases and confirm prompts into German.
    pub reply_language: String,
    /// The interval of the interim transcripts while the user speaks, in
    /// ms; 0 turns them off.
    pub asr_interim_ms: u32,
    /// The command that writes raw mono PCM16 at 24 kHz to stdout.
    pub capture_cmd: Vec<String>,
    /// The command that plays raw mono PCM16 at 24 kHz from stdin.
    pub playback_cmd: Vec<String>,
    /// The sound that playback holds back before it starts, unless the
    /// sentence ends first: the TTS can be slower than real time.
    pub playback_buffer_ms: u32,
    /// The push-to-talk key, for example `Super+v`.
    pub ptt_key: String,
    /// The speech probability from which speech starts.
    pub vad_threshold: f64,
    /// The speech probability under which speech ends.
    pub vad_release: f64,
    /// How long speech lasts before an utterance starts.
    pub onset_ms: u32,
    /// The quiet that ends an utterance.
    pub silence_ms: u32,
    /// An utterance with less speech is a noise.
    pub min_speech_ms: u32,
    /// The longest utterance.
    pub max_turn_ms: u32,
    /// The mic before the onset that an utterance keeps, at most.
    pub preroll_ms: u32,
    pub barge_in_vad_threshold: f64,
    pub barge_in_ms: u32,
    pub barge_in_echo_ratio: f64,
    /// A fixed level floor of the barge-in; 0: 40 % of the level of the
    /// user's last turns.
    pub barge_in_min_rms: f64,
    /// The echo level over the played level at the start; the agent
    /// learns it while it speaks. The default assumes the echo canceller
    /// of `PipeWire`.
    pub barge_in_coupling: f64,
    /// The longest utterance that can be a command word.
    pub word_max_ms: u32,
    pub sleep_words: Vec<String>,
    pub wake_words: Vec<String>,
    pub cancel_words: Vec<String>,
    pub code_cancel_words: Vec<String>,
    pub yes_words: Vec<String>,
    pub no_words: Vec<String>,
    pub filler_words: Vec<String>,
    pub confirm: Vec<Confirm>,
    pub confirm_timeout_ms: u32,
    pub earcons: bool,
    /// From 0 to 1.
    pub earcon_volume: f64,
    /// A state dot under `screen:OUTPUT`.
    pub indicator: bool,
    /// Empty: the built-in prompt.
    pub system_prompt: String,
    /// Empty: the built-in documents.
    pub context_docs: Vec<String>,
    pub prompt_file: String,
    /// 0: ask the server.
    pub context_tokens: u32,
    pub reply_reserve_tokens: u32,
    pub tool_rounds: u32,
    pub old_tool_chars: u32,
    pub saved_requests: u32,
    pub reasoning_effort: String,
    pub phrases: Phrases,
    pub asr_timeout_s: u32,
    pub tts_timeout_s: u32,
    pub tool_timeout_s: u32,
    pub llm_first_token_s: u32,
    pub llm_idle_s: u32,
    pub llm_total_s: u32,
    /// The command of `choractl`, for example in a container.
    pub chora_mcp: Vec<String>,
    /// A coding harness; empty turns the coding tools off. It runs in
    /// `sandbox_cmd`.
    pub coder_cmd: Vec<String>,
    /// The sandbox in front of `coder_cmd` and `agent_cmd`; it runs the
    /// command in the directory where it starts.
    pub sandbox_cmd: Vec<String>,
    /// Runs `coder_cmd` and `agent_cmd` without `sandbox_cmd`, with all
    /// rights of the user.
    pub unsandboxed: bool,
    /// The command of an ACP agent (docs/voice.md section 18), in
    /// `sandbox_cmd`; `{dir}` and `{project}` are the project. Empty turns
    /// the agent tools off.
    pub agent_cmd: Vec<String>,
    /// Projects for the agents besides the inventory: name and directory.
    pub agent_projects: BTreeMap<String, String>,
    /// The voices that the agents get in turn.
    pub agent_voices: Vec<String>,
    /// A longer answer of an agent speaks its first sentence and
    /// `phrases.details`.
    pub agent_said_chars: u32,
    /// A question of an agent without an answer is a no after this time.
    pub agent_permission_s: u32,
    /// The tools that the LLM does not get, as globs, for example the
    /// methods of the agent itself.
    pub skipped_tools: Vec<String>,
    /// A directory for recordings of the sessions; empty turns it off.
    pub record_dir: String,
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|&w| w.into()).collect()
}

fn pcm_cmd(program: &str) -> Vec<String> {
    let args = [
        "--raw",
        "--rate",
        "24000",
        "--channels",
        "1",
        "--format",
        "s16",
        "-",
    ];
    std::iter::once(program)
        .chain(args)
        .map(Into::into)
        .collect()
}

fn rule(tool: &str, prompt: &str) -> Confirm {
    Confirm {
        tool: tool.into(),
        commands: Vec::new(),
        argv0: Vec::new(),
        method: None,
        prompt: prompt.into(),
    }
}

/// The tools that ask first (docs/voice.md section 9).
fn default_confirm() -> Vec<Confirm> {
    let restart = "Shall I restart the plugin?";
    vec![
        Confirm {
            commands: words(&["remove", "close"]),
            ..rule("apply_commands", "Shall I remove that?")
        },
        rule("unload_view", "Shall I remove the view?"),
        rule("inject_input", "Shall I type or click?"),
        rule("code_task", "Shall I start the coding task?"),
        rule("agent_start", "Shall I start the agent?"),
        // A program can do all that the user can (D42).
        rule("run_program", "Shall I start the program {program}?"),
        rule("configure_output", "Shall I change the screen?"),
        rule("*.restart", restart),
        Confirm {
            method: Some("*.restart".into()),
            ..rule("call_method", restart)
        },
    ]
}

impl Config {
    /// The command line of a harness: in `sandbox_cmd`, unless
    /// `unsandboxed`; empty stays empty.
    pub fn spawn(&self, cmd: &[String]) -> Vec<String> {
        if cmd.is_empty() || self.unsandboxed {
            return cmd.to_vec();
        }
        self.sandbox_cmd.iter().chain(cmd).cloned().collect()
    }

    /// The language of the TTS: `tts_language`, else the one of
    /// `reply_language`. With `auto`, the TTS made short German clips hard
    /// to understand (WP6).
    pub fn tts_language(&self) -> &str {
        if !self.tts_language.is_empty() {
            return &self.tts_language;
        }
        language_name(&self.reply_language).unwrap_or("auto")
    }

    /// The ASR server, with its key and headers.
    ///
    /// # Errors
    ///
    /// The key is not there.
    pub fn asr(&self) -> Result<Backend, String> {
        let (env, file) = (&self.asr_api_key_env, &self.asr_api_key_file);
        let server = [&self.asr_url, &self.asr_model, env, file];
        backend("asr", server, &self.asr_headers)
    }

    /// The TTS server, with its key and headers.
    ///
    /// # Errors
    ///
    /// The key is not there.
    pub fn tts(&self) -> Result<Backend, String> {
        let (env, file) = (&self.tts_api_key_env, &self.tts_api_key_file);
        let server = [&self.tts_url, &self.tts_model, env, file];
        backend("tts", server, &self.tts_headers)
    }

    /// The LLM server, with its key and headers.
    ///
    /// # Errors
    ///
    /// The key is not there.
    pub fn llm(&self) -> Result<Backend, String> {
        let (env, file) = (&self.llm_api_key_env, &self.llm_api_key_file);
        let server = [&self.llm_url, &self.llm_model, env, file];
        backend("llm", server, &self.llm_headers)
    }

    /// The prompt of the ASR: `asr_prompt`, the wake, sleep and stop words
    /// and `names`, each once; none for an empty `asr_prompt`.
    pub fn asr_prompt<'a>(&'a self, names: impl IntoIterator<Item = &'a String>) -> String {
        let words = (self.wake_words.iter())
            .chain(&self.sleep_words)
            .chain(&self.cancel_words)
            .chain(names);
        let mut out = self.asr_prompt.trim().to_owned();
        if out.is_empty() {
            return out;
        }
        for word in words {
            if !out.to_lowercase().contains(&word.to_lowercase()) {
                out.push_str(", ");
                out.push_str(word);
            }
        }
        out
    }

    /// The sentence of the system prompt about the language of the
    /// answers.
    pub fn reply_rule(&self) -> String {
        let lang = language_name(&self.reply_language).unwrap_or(&self.reply_language);
        format!(
            "Always answer in {lang}, even when the user speaks another language. Change the \
             language only when the user asks for it."
        )
    }

    /// The settings as the agent uses them: with `reply_language: de`,
    /// the phrases and confirm prompts that keep their English default
    /// in German.
    #[must_use]
    pub fn localized(&self) -> Self {
        let mut out = self.clone();
        if self.reply_language != "de" {
            return out;
        }
        let p = &mut out.phrases;
        let texts = [
            &mut p.llm_down,
            &mut p.llm_slow,
            &mut p.tool_rounds,
            &mut p.asr_failed,
            &mut p.details,
            &mut p.agent_asks,
            &mut p.agent_ended,
        ];
        let prompts = out.confirm.iter_mut().map(|r| &mut r.prompt);
        for text in texts.into_iter().chain(prompts) {
            if let Some((_, de)) = GERMAN.iter().find(|(en, _)| en == text) {
                *text = (*de).into();
            }
        }
        out
    }

    /// The spoken result of a coding task in `reply_language`.
    pub fn code_said(&self, said: &str) -> String {
        if self.reply_language != "de" {
            return said.into();
        }
        // ponytail: the English sentence of `coder.rs` piece by piece; a
        // phrase per state in the settings when another language needs it.
        (CODE_GERMAN.iter()).fold(said.to_owned(), |s, (en, de)| s.replacen(en, de, 1))
    }
}

/// The default phrases and confirm prompts, and their German.
const GERMAN: [(&str, &str); 15] = [
    (
        "The language model does not answer.",
        "Das Sprachmodell antwortet nicht.",
    ),
    (
        "The language model takes too long.",
        "Das Sprachmodell braucht zu lange.",
    ),
    (
        "I stopped after too many tool calls.",
        "Ich habe nach zu vielen Werkzeugaufrufen aufgehört.",
    ),
    (
        "I did not understand you.",
        "Ich habe dich nicht verstanden.",
    ),
    ("Details on the screen.", "Details auf dem Bildschirm."),
    (
        "{name} wants to {what}. Allow?",
        "{name} möchte {what}. Erlauben?",
    ),
    ("{name} has ended.", "{name} hat sich beendet."),
    ("Shall I remove that?", "Soll ich das entfernen?"),
    (
        "Shall I remove the view?",
        "Soll ich die Ansicht entfernen?",
    ),
    ("Shall I type or click?", "Soll ich tippen oder klicken?"),
    (
        "Shall I start the coding task?",
        "Soll ich die Programmieraufgabe starten?",
    ),
    ("Shall I start the agent?", "Soll ich den Agenten starten?"),
    (
        "Shall I start the program {program}?",
        "Soll ich das Programm {program} starten?",
    ),
    (
        "Shall I change the screen?",
        "Soll ich den Bildschirm ändern?",
    ),
    (
        "Shall I restart the plugin?",
        "Soll ich das Plugin neu starten?",
    ),
];

/// The pieces of the spoken coding result of `coder.rs`, and their German.
const CODE_GERMAN: [(&str, &str); 4] = [
    ("The coding task in ", "Die Programmieraufgabe in "),
    (" is done.", " ist fertig."),
    (" failed.", " ist fehlgeschlagen."),
    (" was cancelled.", " wurde abgebrochen."),
];

/// The English name of a language code, as the TTS and the LLM take it.
fn language_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "de" => "German",
        "en" => "English",
        "fr" => "French",
        "it" => "Italian",
        "es" => "Spanish",
        "pt" => "Portuguese",
        "ru" => "Russian",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        _ => return None,
    })
}

/// A server of the agent: its URL, its model and the headers of each
/// request.
#[derive(Clone, Debug, PartialEq)]
pub struct Backend {
    pub url: String,
    pub model: String,
    pub headers: Vec<(String, String)>,
}

/// The server `name` of URL, model, key variable and key file: the key
/// goes into `Authorization: Bearer`, as `OpenAI` takes it over HTTP and on
/// its realtime websocket.
fn backend(
    name: &str,
    [url, model, env, file]: [&String; 4],
    headers: &BTreeMap<String, String>,
) -> Result<Backend, String> {
    let key = if !env.is_empty() {
        Some(std::env::var(env).map_err(|_| format!("{name}_api_key_env: no {env}"))?)
    } else if !file.is_empty() {
        let dir = std::env::var("CREDENTIALS_DIRECTORY").unwrap_or_default();
        let path = crate::expand(&file.replace("%d", &dir));
        let key = std::fs::read_to_string(&path)
            .map_err(|e| format!("{name}_api_key_file: {}: {e}", path.display()))?;
        Some(key)
    } else {
        None
    };
    let mut out: Vec<(String, String)> = headers.clone().into_iter().collect();
    if let Some(key) = key {
        out.push(("Authorization".into(), format!("Bearer {}", key.trim())));
    }
    Ok(Backend {
        url: url.into(),
        model: model.into(),
        headers: out,
    })
}

impl Default for Config {
    #[allow(clippy::too_many_lines)] // one line per key
    fn default() -> Self {
        Self {
            asr_url: "ws://127.0.0.1:8881/v1/realtime?intent=transcription".into(),
            asr_model: "gufo".into(),
            asr_api_key_env: String::new(),
            asr_api_key_file: String::new(),
            asr_headers: BTreeMap::new(),
            asr_language: "auto".into(),
            asr_prompt: "Chora, Coder".into(),
            tts_url: "ws://127.0.0.1:8882/v1/audio/speech/stream".into(),
            tts_model: "gufo".into(),
            tts_api_key_env: String::new(),
            tts_api_key_file: String::new(),
            tts_headers: BTreeMap::new(),
            voice: "aiden".into(),
            tts_language: String::new(),
            llm_url: "http://127.0.0.1:8888/v1".into(),
            llm_model: "gufo".into(),
            llm_api_key_env: String::new(),
            llm_api_key_file: String::new(),
            llm_headers: BTreeMap::new(),
            reply_language: "en".into(),
            asr_interim_ms: 0,
            capture_cmd: pcm_cmd("pw-record"),
            playback_cmd: pcm_cmd("pw-play"),
            playback_buffer_ms: 1000,
            ptt_key: "Super+v".into(),
            vad_threshold: 0.5,
            vad_release: 0.35,
            onset_ms: 96,
            silence_ms: 800,
            min_speech_ms: 200,
            max_turn_ms: 30_000,
            preroll_ms: 300,
            barge_in_vad_threshold: 0.7,
            barge_in_ms: 250,
            barge_in_echo_ratio: 2.0,
            barge_in_min_rms: 0.0,
            barge_in_coupling: 0.1,
            word_max_ms: 2000,
            sleep_words: words(&["sleep", "go to sleep", "schlaf", "schlaf jetzt"]),
            wake_words: words(&["hey chora", "listen", "hallo chora", "hör zu"]),
            cancel_words: words(&[
                "stop",
                "cancel",
                "never mind",
                "stopp",
                "abbrechen",
                "halt",
                "vergiss es",
            ]),
            code_cancel_words: words(&[
                "cancel the coding task",
                "coding-aufgabe abbrechen",
                "programmieraufgabe abbrechen",
            ]),
            yes_words: words(&[
                "yes", "okay", "ok", "do it", "please", "sure", "go ahead", "ja", "genau", "mach",
                "mach das", "bitte",
            ]),
            no_words: words(&["no", "rather not", "nein", "nicht", "lieber nicht"]),
            filler_words: words(&["please", "now", "bitte", "jetzt", "mal"]),
            confirm: default_confirm(),
            confirm_timeout_ms: 8000,
            earcons: true,
            earcon_volume: 0.3,
            indicator: true,
            system_prompt: String::new(),
            context_docs: Vec::new(),
            prompt_file: "~/.config/chora/voice-prompt.md".into(),
            context_tokens: 0,
            reply_reserve_tokens: 1024,
            tool_rounds: 10,
            old_tool_chars: 500,
            saved_requests: 20,
            reasoning_effort: "none".into(),
            phrases: Phrases::default(),
            asr_timeout_s: 15,
            tts_timeout_s: 10,
            tool_timeout_s: 30,
            llm_first_token_s: 30,
            llm_idle_s: 20,
            llm_total_s: 120,
            chora_mcp: words(&["choractl"]),
            coder_cmd: Vec::new(),
            sandbox_cmd: words(&["agent-sandbox.sh"]),
            unsandboxed: false,
            agent_cmd: Vec::new(),
            agent_projects: BTreeMap::new(),
            agent_voices: words(&["ryan", "vivian", "serena", "eric"]),
            agent_said_chars: 160,
            agent_permission_s: 120,
            skipped_tools: words(&["subscribe_events", "wait_events", "capture", "voice.*"]),
            record_dir: String::new(),
        }
    }
}

impl Config {
    /// The word lists by key, for the checks.
    fn word_lists(&self) -> [(&'static str, &[String]); 7] {
        [
            ("sleep_words", &self.sleep_words),
            ("wake_words", &self.wake_words),
            ("cancel_words", &self.cancel_words),
            ("code_cancel_words", &self.code_cancel_words),
            ("yes_words", &self.yes_words),
            ("no_words", &self.no_words),
            ("filler_words", &self.filler_words),
        ]
    }
}

impl Config {
    /// The numbers and commands.
    fn check_values(&self) -> Result<(), String> {
        for (key, value) in [
            ("vad_threshold", self.vad_threshold),
            ("vad_release", self.vad_release),
            ("barge_in_vad_threshold", self.barge_in_vad_threshold),
            ("earcon_volume", self.earcon_volume),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(format!("{key}: from 0 to 1"));
            }
        }
        if self.vad_release > self.vad_threshold {
            return Err("vad_release: at most vad_threshold".into());
        }
        if self.barge_in_echo_ratio < 1.0 || self.barge_in_min_rms < 0.0 {
            return Err("barge_in_echo_ratio: at least 1; barge_in_min_rms: at least 0".into());
        }
        if !(0.001..=10.0).contains(&self.barge_in_coupling) {
            return Err("barge_in_coupling: from 0.001 to 10".into());
        }
        for (key, ms) in [
            ("onset_ms", self.onset_ms),
            ("silence_ms", self.silence_ms),
            ("max_turn_ms", self.max_turn_ms),
            ("barge_in_ms", self.barge_in_ms),
            ("word_max_ms", self.word_max_ms),
            ("confirm_timeout_ms", self.confirm_timeout_ms),
        ] {
            if !(32..=600_000).contains(&ms) {
                return Err(format!("{key}: from 32 to 600000"));
            }
        }
        for (key, ms) in [
            ("preroll_ms", self.preroll_ms),
            ("playback_buffer_ms", self.playback_buffer_ms),
        ] {
            if ms > 5000 {
                return Err(format!("{key}: at most 5000"));
            }
        }
        if self.tool_rounds == 0 || self.tool_timeout_s == 0 {
            return Err("tool_rounds, tool_timeout_s: at least 1".into());
        }
        for (key, cmd) in [
            ("capture_cmd", &self.capture_cmd),
            ("playback_cmd", &self.playback_cmd),
            ("chora_mcp", &self.chora_mcp),
        ] {
            if cmd.first().is_none_or(String::is_empty) {
                return Err(format!("{key}: a program and its arguments"));
            }
        }
        Ok(())
    }

    /// The key, the words and the rules.
    fn check_words(&self) -> Result<(), String> {
        let key: Binding = self.ptt_key.parse().map_err(|e| format!("ptt_key: {e}"))?;
        // chora reports the release of a key with its modifier only.
        if !(key.mods.ctrl || key.mods.alt || key.mods.logo) {
            return Err("ptt_key: needs Ctrl, Alt or Super".into());
        }
        let mut seen = BTreeMap::new();
        for (key, list) in self.word_lists() {
            if list.is_empty() && key != "filler_words" {
                return Err(format!("{key}: at least one word"));
            }
            for word in list {
                let word = word.trim().to_lowercase();
                if word.is_empty() {
                    return Err(format!("{key}: an empty word"));
                }
                // A filler word may answer a confirmation ("Bitte.").
                if let Some(other) = seen.insert(word.clone(), key)
                    && other != key
                    && ![key, other].contains(&"filler_words")
                {
                    return Err(format!("{key}: {word:?} is in {other} too"));
                }
            }
        }
        if let Some(rule) = self.confirm.iter().find(|r| r.prompt.trim().is_empty()) {
            return Err(format!("confirm: the rule of {} has no prompt", rule.tool));
        }
        Ok(())
    }
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        self.check_values()?;
        self.check_harnesses()?;
        self.check_words()
    }
}

impl Config {
    /// The sandbox and the agents.
    fn check_harnesses(&self) -> Result<(), String> {
        if !self.unsandboxed
            && self.sandbox_cmd.first().is_none_or(String::is_empty)
            && !(self.coder_cmd.is_empty() && self.agent_cmd.is_empty())
        {
            return Err("sandbox_cmd: a program, or unsandboxed: true".into());
        }
        if self.agent_said_chars == 0 || self.agent_permission_s == 0 {
            return Err("agent_said_chars, agent_permission_s: at least 1".into());
        }
        for (name, env, file) in [
            ("asr", &self.asr_api_key_env, &self.asr_api_key_file),
            ("tts", &self.tts_api_key_env, &self.tts_api_key_file),
            ("llm", &self.llm_api_key_env, &self.llm_api_key_file),
        ] {
            if !env.is_empty() && !file.is_empty() {
                return Err(format!(
                    "{name}_api_key_env, {name}_api_key_file: one of them"
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::parse;

    use super::*;

    #[test]
    fn an_empty_file_gives_the_defaults() {
        assert_eq!(parse::<Config>("{}").unwrap(), Config::default());
        let some = parse::<Config>("silence_ms: 500\n").unwrap();
        assert_eq!(some.silence_ms, 500);
        assert_eq!(
            some.vad_threshold, 0.5,
            "the other keys keep their defaults"
        );
    }

    #[test]
    fn the_defaults_round_trip() {
        let text = serde_saphyr::to_string(&Config::default()).unwrap();
        assert_eq!(parse::<Config>(&text).unwrap(), Config::default());
    }

    #[test]
    fn harnesses_run_in_the_sandbox_unless_unsandboxed() {
        let mut config = Config::default();
        let cmd = words(&["opencode", "acp"]);
        assert_eq!(config.spawn(&cmd), ["agent-sandbox.sh", "opencode", "acp"]);
        assert_eq!(config.spawn(&[]), Vec::<String>::new());
        config.unsandboxed = true;
        assert_eq!(config.spawn(&cmd), cmd);
    }

    #[test]
    fn the_tts_language_follows_the_reply_language() {
        let mut config = Config::default();
        assert_eq!(config.tts_language(), "English");
        config.reply_language = "xx".into();
        assert_eq!(config.tts_language(), "auto");
        config.tts_language = "English".into();
        assert_eq!(config.tts_language(), "English");
    }

    #[test]
    fn check_rejects_mistakes() {
        for (yaml, key) in [
            ("vad_threshold: 1.5", "vad_threshold"),
            ("vad_release: 0.9", "vad_release"),
            ("silence_ms: 0", "silence_ms"),
            ("capture_cmd: []", "capture_cmd"),
            ("ptt_key: Hyper+v", "ptt_key"),
            ("ptt_key: Shift+v", "ptt_key"),
            ("sleep_words: []", "sleep_words"),
            ("wake_words: [schlaf]", "schlaf"),
            ("yes_words: [Nein]", "nein"),
            ("confirm: [{tool: x, prompt: ' '}]", "confirm"),
            ("colour: 1", "colour"),
            (
                "{sandbox_cmd: [], agent_cmd: [opencode, acp]}",
                "sandbox_cmd",
            ),
            ("{llm_api_key_env: K, llm_api_key_file: k}", "llm_api_key"),
            ("model: gufo", "model"),
        ] {
            let err = parse::<Config>(yaml).unwrap_err();
            assert!(err.contains(key), "{yaml}: {err}");
        }
    }

    #[test]
    fn a_key_goes_into_the_bearer_header() {
        let mut config = Config::default();
        assert_eq!(config.llm().unwrap().headers, []);
        let file = std::env::temp_dir().join(format!("chora-voice-key-{}", std::process::id()));
        std::fs::write(&file, "sk-1\n").unwrap();
        config.llm_api_key_file = file.to_string_lossy().into();
        config.llm_headers.insert("X-Title".into(), "chora".into());
        let llm = config.llm().unwrap();
        std::fs::remove_file(&file).unwrap();
        assert_eq!(llm.model, "gufo");
        assert_eq!(
            llm.headers,
            [
                ("X-Title".to_owned(), "chora".to_owned()),
                ("Authorization".to_owned(), "Bearer sk-1".to_owned()),
            ]
        );
        assert_eq!(config.asr().unwrap().headers, [], "per server");
        config.tts_api_key_env = "CHORA_VOICE_NO_SUCH_KEY".into();
        assert!(
            config
                .tts()
                .unwrap_err()
                .contains("CHORA_VOICE_NO_SUCH_KEY")
        );
    }

    #[test]
    fn german_replies_get_the_german_defaults_only() {
        let mut config = Config::default();
        assert_eq!(config.localized(), config, "English stays");
        config.reply_language = "de".into();
        config.phrases.llm_slow = "Zu langsam.".into();
        let de = config.localized();
        assert_eq!(de.phrases.details, "Details auf dem Bildschirm.");
        assert_eq!(de.phrases.agent_asks, "{name} möchte {what}. Erlauben?");
        assert_eq!(de.phrases.llm_slow, "Zu langsam.", "the user's own");
        assert!(de.confirm.iter().all(|r| r.prompt.starts_with("Soll ich")));
        assert_eq!(
            config.code_said("The coding task in demo-1 is done. All good."),
            "Die Programmieraufgabe in demo-1 ist fertig. All good."
        );
    }

    #[test]
    fn the_asr_prompt_adds_the_words_and_names_once() {
        let config = Config {
            asr_prompt: "Chora".into(),
            wake_words: words(&["hey chora", "listen"]),
            sleep_words: words(&["sleep"]),
            cancel_words: words(&["stop"]),
            ..Config::default()
        };
        let names = words(&["atlas", "Listen"]);
        assert_eq!(
            config.asr_prompt(&names),
            "Chora, hey chora, listen, sleep, stop, atlas"
        );
        let off = Config {
            asr_prompt: " ".into(),
            ..config
        };
        assert_eq!(off.asr_prompt(&names), "");
    }
}
