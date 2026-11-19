//! The confirmation gate of the tool calls: the detail that a question
//! shows, the words that allow a call for the session, and the checks of
//! the calls that change the own settings.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use chora_kit::chat::hides;
use serde_json::{Value, json};
use tokio::sync::oneshot;

use super::{Chat, Config, Event, FAILED_TWICE, Failed, SAID_NEW, tools};

/// What a yes with an always word allows for the session: the same
/// full argv of `run_program`, or the same method `CLIENT.METHOD` with
/// the same parameters. None for every other call, such as
/// `inject_input`, `apply_commands`, `code_task`, `agent_start` or
/// `configure_output`, and for each settings change: for them, an always
/// word is a single yes. The keys of an object are sorted by `serde_json`,
/// so the text is canonical.
pub(super) fn session_key(tool: &str, patch: Option<&Value>, args: &Value) -> Option<String> {
    if patch.is_some() {
        return None;
    }
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
/// `CLIENT.METHOD` of a call. An entry is the whole argv: a bare name only
/// allows the program without arguments (`foot -e sh` is a shell).
pub(super) fn listed(config: &Config, tool: &str, args: &Value) -> bool {
    let has = |list: &[String], entry: &str| list.iter().any(|e| e == entry);
    match tool {
        "run_program" => args["argv"].as_array().is_some_and(|argv| {
            let words: Vec<&str> = argv.iter().filter_map(Value::as_str).collect();
            (config.allow_programs.iter()).any(|e| e.split(' ').eq(words.iter().copied()))
                && words.first().is_some_and(|p| !p.contains('/'))
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
/// hidden: each subscriber of `voice.*` gets it.
pub(crate) fn detail(tool: &str, patch: Option<&Value>, args: &Value) -> String {
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

/// Most lines and bytes of a detail that the agent asks about. A
/// longer one is refused, so that every screen shows the whole command
/// and no question hides its tail (review rc3 round 4, 2.2).
pub(crate) const DETAIL_LINES: usize = 400;
pub(crate) const DETAIL_BYTES: usize = 64 * 1024;

/// The answer to the LLM for a detail that does not show whole and as
/// it is, without a question: too long, or it [`hides`] text; none for
/// a detail that shows.
pub(crate) fn unshowable(detail: &str) -> Option<String> {
    if too_long(detail) {
        return Some(format!(
            "Refused: the command is too long to show the user ({DETAIL_LINES} lines or {} KiB at most); send a shorter command",
            DETAIL_BYTES / 1024
        ));
    }
    hides(detail).then(|| "Refused: the command holds characters that hide or reorder text (control, bidi or zero-width characters), so the user would see another command than the one that runs; send it without them".to_owned())
}

/// Whether a detail passes the cap.
pub(crate) fn too_long(detail: &str) -> bool {
    detail.len() > DETAIL_BYTES || detail.lines().count() > DETAIL_LINES
}

/// The own settings tool takes `{patch}`, and the LLM copies that to
/// `call_method`, where the params of `settings.set` are the patch itself:
/// `{"patch": {...}}` there becomes `{...}`.
pub(super) fn unwrap_patch(args: &mut Value) {
    if args["method"]
        .as_str()
        .is_some_and(|m| m.ends_with("settings.set"))
        && let Some(params) = args.get_mut("params")
        && let Some(patch) = params
            .as_object()
            .filter(|p| p.len() == 1)
            .and_then(|p| p.get("patch"))
        && patch.is_object()
    {
        *params = patch.clone();
    }
}

/// Hides the value of each key such as `llm_headers`, `token`,
/// `authorization` or one with `api_key`, `secret` or `password`.
pub(crate) fn hide(value: &mut Value) {
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

impl Chat {
    /// Notes a failed call and adds `FAILED_TWICE` to its error when the
    /// call before failed the same way: with the same error, or as the
    /// same tool with nearly the same arguments. The error stays whole,
    /// so the LLM can correct the form.
    pub(super) fn guard(&mut self, tool: &str, args: &Value, content: Value) -> Value {
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
    /// different: the reload will refuse it anyway. The file is
    /// localized in the running `reply_language`, so that a switch of
    /// the language alone is no security change, as in the client
    /// thread, which compares the raw files.
    fn reload_is_security(&self) -> bool {
        let Some(file) = &self.settings_file else {
            return true;
        };
        let new = std::fs::read_to_string(file)
            .map_err(|e| e.to_string())
            .and_then(|text| chora_kit::settings::parse::<Config>(&text))
            .map(|c| {
                let running = Config {
                    reply_language: self.config.reply_language.clone(),
                    ..c
                };
                serde_json::to_value(running.localized())
            });
        let old = serde_json::to_value(&self.config);
        let (Ok(Ok(new)), Ok(old)) = (new, old) else {
            return true;
        };
        let keys = new.as_object().into_iter().flatten().map(|(k, _)| k);
        keys.filter(|k| crate::config::security_key(k))
            .any(|k| new[k] != old[k])
    }

    /// The rule that asks before a call: one of `confirm`, else the one
    /// of a privileged method.
    fn rule(&self, name: &str, args: &Value) -> Option<crate::config::Confirm> {
        tools::rule(&self.config.confirm, name, args)
            .cloned()
            .or_else(|| {
                let prompt = &self.config.phrases.privileged;
                self.tools.privileged_rule(name, args, prompt)
            })
    }

    /// Asks the user when a rule of `confirm` matches, and before each
    /// change of the agent's own settings; else, or on a no, the reason
    /// for the LLM. A security setting takes only a key press
    /// (`confirm_keys`). A reload takes it when the file differs from the
    /// running settings in a security key: the LLM may have written the
    /// file (review rc3 round 3, 2.4). New words of the user come first:
    /// the question would answer what they no longer ask; the next round
    /// hears them. A call without a session key takes an always word as a
    /// single yes.
    pub(super) async fn confirmed(
        &mut self,
        function: &str,
        args: &Value,
        out: &(impl Fn(Event) + Sync),
    ) -> Result<(), String> {
        let Some(name) = self.tools.mcp_name(function).map(str::to_owned) else {
            return Ok(());
        };
        let own = self.tools.own_settings_set(&name, args);
        let reload = self.tools.own_settings_reload(&name, args) && self.reload_is_security();
        let physical = reload
            || own
                .and_then(Value::as_object)
                .is_some_and(|p| p.keys().any(|k| crate::config::security_key(k)));
        if physical {
            return Ok(());
        }
        let detail = detail(&name, own, args);
        let prompt = if let Some(patch) = own {
            self.config.settings_question(patch)
        } else if let Some(rule) = self.rule(&name, args) {
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
        if self.take_inbox() {
            crate::log::info(&format!(
                "confirm {name}: skipped, the user said something new"
            ));
            return Err(SAID_NEW.into());
        }
        if let Some(refusal) = unshowable(&detail) {
            crate::log::info(&format!("confirm {name}: {refusal}"));
            return Err(refusal);
        }
        let (result, always) = self.ask(prompt, detail, out).await;
        crate::log::info(&format!(
            "confirm {name}: {}",
            result.as_ref().map_or_else(String::as_str, |()| "yes")
        ));
        if let (Ok(()), true, Some(key)) = (&result, always, key) {
            crate::log::info(&format!("confirm {name}: allowed for the session as {key}"));
            self.allowed.insert(key);
            out(Event::Sentence(self.config.phrases.allowed.clone()));
        }
        result
    }

    /// Asks the user and waits for the answer; answers whether it had
    /// an always word. A spoken yes answers: the client asks the
    /// questions that take a key, and their mark (`Tools::physical`)
    /// stays as it is. The core times the answer from the end of the
    /// spoken question and drops it then, so the wait here is only a
    /// backstop. The agent itself asks: the LLM must not look for a dialog.
    async fn ask(
        &self,
        prompt: String,
        detail: String,
        out: &(impl Fn(Event) + Sync),
    ) -> (Result<(), String>, bool) {
        let (answer, rx) = oneshot::channel();
        let always = Arc::new(AtomicBool::new(false));
        out(Event::Confirm {
            prompt,
            detail,
            always: always.clone(),
            physical: false,
            answer,
        });
        let wait = Duration::from_millis(self.config.confirm_timeout_ms.into());
        let backstop = wait + Duration::from_secs(60);
        let result = match tokio::time::timeout(backstop, rx).await {
            Ok(Ok(Some(true))) => Ok(()),
            Ok(Ok(Some(false))) => Err("The user declined the spoken confirmation".to_owned()),
            Ok(Ok(None)) => Err("The answer to the spoken confirmation was not understood".into()),
            Ok(Err(_)) | Err(_) => Err(format!(
                "The user did not answer the spoken confirmation in time ({} s)",
                wait.as_secs()
            )),
        };
        (result, always.load(Ordering::Relaxed))
    }

    /// Whether each command of `kinds` (each command when empty) in the
    /// `apply_commands` of `args` removes a node of the agent's own client
    /// with only its own nodes under it, by the scene of `query_scene`.
    /// Any doubt answers false: the question stays. The walk up from each
    /// node is O(nodes × depth), which is fine for the scenes of a desktop.
    pub(super) async fn removes_own(&mut self, kinds: &[String], args: &Value) -> bool {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    /// parameters, so their order does not count; settings methods take an
    /// always word as a single yes.
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

    /// Review secfix c r1, 3: only characters that hide or reorder text
    /// refuse a detail; an emoji, a joiner, a non-breaking space, a mark
    /// of direction or a carriage return are text.
    #[test]
    fn hides_refuses_only_what_hides_or_reorders() {
        for text in [
            "⚠️ rm",
            "a\u{200d}b",
            "می\u{200c}خواهم",
            "10\u{a0}km",
            "a\u{200e}b",
            "a\r\nb",
            "x\ty\nz",
        ] {
            assert!(!hides(text), "{text:?}");
        }
        for text in [
            "a\u{202e}b",
            "a\u{2066}b",
            "a\u{1b}[2Jb",
            "a\u{9d}b",
            "a\u{200b}b",
            "a\u{feff}b",
            "a\u{0}b",
            "a\u{2062}b",
        ] {
            assert!(hides(text), "{text:?}");
        }
    }

    /// Review 0.9.0-rc.3: a shown call gets no secret of its arguments.
    #[test]
    fn hide_hides_the_values_of_secret_keys() {
        let mut set = json!({"patch": {"llm_headers": {"X-Key": "sk-2"}, "llm_api_key_file": "/k",
            "asr_url": "http://x", "list": [{"token": "t"}]}});
        hide(&mut set);
        let args = set.to_string();
        for secret in ["sk-2", "/k", "\"t\""] {
            assert!(!args.contains(secret), "{args}");
        }
        assert!(args.contains("http://x"), "{args}");
    }

    /// The LLM wraps the patch of another plugin's `settings.set` as in
    /// its own settings tool; the wrapper goes.
    #[test]
    fn call_method_settings_set_takes_the_patch_unwrapped() {
        let mut args = json!({"client": "mc", "method": "settings.set",
            "params": {"patch": {"font_size": 12}}});
        unwrap_patch(&mut args);
        assert_eq!(args["params"], json!({"font_size": 12}));
        let mut other =
            json!({"client": "mc", "method": "scene.set", "params": {"patch": {"a": 1}}});
        let same = other.clone();
        unwrap_patch(&mut other);
        assert_eq!(other, same);
        let mut direct = json!({"client": "mc", "method": "settings.set", "params": {"a": 1}});
        unwrap_patch(&mut direct);
        assert_eq!(direct["params"], json!({"a": 1}));
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
}
