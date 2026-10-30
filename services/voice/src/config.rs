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

/// The canned sentences of the agent.
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
            llm_down: "Das Sprachmodell antwortet nicht.".into(),
            llm_slow: "Das Sprachmodell braucht zu lange.".into(),
            tool_rounds: "Ich habe nach zu vielen Werkzeugaufrufen aufgehört.".into(),
            asr_failed: "Ich habe dich nicht verstanden.".into(),
            details: "Details auf dem Bildschirm.".into(),
            agent_asks: "{name} möchte {what}. Erlauben?".into(),
            agent_ended: "{name} hat sich beendet.".into(),
        }
    }
}

/// The settings of the agent.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub asr_url: String,
    pub tts_url: String,
    pub llm_url: String,
    /// The served name of the models.
    pub model: String,
    /// A voice of the TTS server.
    pub voice: String,
    /// The language of the ASR.
    pub language: String,
    /// The language of the TTS, for example `German`; `auto` lets the
    /// model choose; empty follows `language`.
    pub tts_language: String,
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
    let restart = "Soll ich das Plugin neu starten?";
    vec![
        Confirm {
            commands: words(&["remove", "close"]),
            ..rule("apply_commands", "Soll ich das entfernen?")
        },
        rule("unload_view", "Soll ich die Ansicht entfernen?"),
        rule("inject_input", "Soll ich tippen oder klicken?"),
        rule("code_task", "Soll ich die Programmieraufgabe starten?"),
        rule("agent_start", "Soll ich den Agenten starten?"),
        // A program can do all that the user can (D42).
        rule("run_program", "Soll ich das Programm {program} starten?"),
        rule("configure_output", "Soll ich den Bildschirm ändern?"),
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
    /// `language`. With `auto`, the TTS made short German clips hard to
    /// understand (WP6).
    pub fn tts_language(&self) -> &str {
        if !self.tts_language.is_empty() {
            return &self.tts_language;
        }
        match self.language.as_str() {
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
            _ => "auto",
        }
    }
}

impl Default for Config {
    #[allow(clippy::too_many_lines)] // one line per key
    fn default() -> Self {
        Self {
            asr_url: "ws://127.0.0.1:8881/v1/realtime?intent=transcription".into(),
            tts_url: "ws://127.0.0.1:8882/v1/audio/speech/stream".into(),
            llm_url: "http://127.0.0.1:8888/v1".into(),
            model: "gufo".into(),
            voice: "aiden".into(),
            language: "de".into(),
            tts_language: String::new(),
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
            barge_in_min_rms: 300.0,
            barge_in_coupling: 0.1,
            word_max_ms: 2000,
            sleep_words: words(&["schlaf", "schlaf jetzt"]),
            wake_words: words(&["hör zu", "hallo chora"]),
            cancel_words: words(&["stopp", "stop", "abbrechen", "halt", "vergiss es"]),
            code_cancel_words: words(&["coding-aufgabe abbrechen", "programmieraufgabe abbrechen"]),
            yes_words: words(&["ja", "okay", "genau", "mach das"]),
            no_words: words(&["nein", "nicht", "lieber nicht"]),
            filler_words: words(&["bitte", "jetzt", "mal"]),
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
                if let Some(other) = seen.insert(word.clone(), key)
                    && other != key
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
    fn the_tts_language_follows_the_language() {
        let mut config = Config::default();
        assert_eq!(config.tts_language(), "German");
        config.language = "xx".into();
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
        ] {
            let err = parse::<Config>(yaml).unwrap_err();
            assert!(err.contains(key), "{yaml}: {err}");
        }
    }
}
