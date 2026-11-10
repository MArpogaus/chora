//! The chora client in a thread of its own: the methods of `voice`, the
//! settings (D62), the push-to-talk grab and the events of the HUD (D65),
//! on the `chora-kit` runtime.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Event,
    ipc::{Capability, ClientError, Emit, ExportMethod, PressState, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::Config,
    core::{Call, Input},
};

/// The only client name that emits the events of the HUD: subscribers
/// check `client == "voice"`, so an agent under another name emits none
/// (review rc3 round 2, 4.9).
const VOICE: &str = "voice";

/// Whether the agent under the client name `name` emits HUD events.
fn hud_client(name: &str) -> bool {
    name == VOICE
}

/// How long a method waits for the core.
const ANSWER: Duration = Duration::from_secs(1);
/// How often the thread looks for events to emit.
// ponytail: one thread cannot block on the socket and the channel at once,
// so it polls; a wake-up fd if the idle wake-ups ever matter.
const EMIT_POLL: Duration = Duration::from_millis(50);
/// The wait before a state that the core refused goes again: the core
/// counts its rate per second.
const RESEND: Duration = Duration::from_secs(1);

/// What a grabbed key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key {
    Ptt,
    /// A key of `confirm_keys`: yes or no.
    Confirm(bool),
}

struct Client {
    rt: Runtime<Key>,
    settings: settings::File<Config>,
    tx: Sender<Input>,
    claims: Claims,
    /// The keys of `confirm_keys` are grabbed.
    confirming: bool,
}

/// The clients that show the state of the agent themselves and claimed
/// the indicator: while one is connected, the dot hides. The claims of a
/// session end with it; the plugins claim again when `voice` connects.
#[derive(Default)]
struct Claims(std::collections::BTreeSet<String>);

impl Claims {
    /// A client claims the indicator; answers whether the dot hides.
    fn claim(&mut self, client: &str) -> bool {
        self.0.insert(client.to_owned());
        true
    }

    /// A client is gone; answers whether the dot still hides.
    fn gone(&mut self, client: &str) -> bool {
        self.0.remove(client);
        !self.0.is_empty()
    }
}

/// The parameters of `confirm`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
enum Answer {
    Yes,
    No,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfirmParams {
    answer: Answer,
}

/// The parameters of `confirm.answer`: the id as `voice.confirm` sends it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ButtonParams {
    id: String,
    yes: bool,
}

/// Whether a confined caller is refused `method`: the methods that act for
/// the user at the keyboard.
fn not_for_confined(method: &str) -> bool {
    matches!(method, "confirm" | "talk" | "mute") || CHAT_METHODS.contains(&method)
}

/// The methods of a chat view: they read the transcript of an agent,
/// stop its prompt and mark a chat open. None of them answers a question.
const CHAT_METHODS: [&str; 4] = ["agent.read", "agent.cancel", "chat.open", "chat.close"];

/// The parameters of `agent.read`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadParams {
    name: String,
    #[serde(default)]
    after: u64,
}

/// The parameters of `agent.cancel`, `chat.open` and `chat.close`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NameParams {
    name: String,
}

/// A chat method of `caller`: refused for a confined connection; the core
/// gets the name of the caller, and a failure of the core is a refusal.
fn chat_call(
    tx: &Sender<Input>,
    caller: &str,
    confined: bool,
    method: &str,
    params: Value,
) -> Result<Value, RpcError> {
    if confined {
        return Err(confined_refusal(method));
    }
    let mut args = if method == "agent.read" {
        let ReadParams { name, after } = runtime::params(params)?;
        json!({"name": name, "after": after})
    } else {
        let NameParams { name } = runtime::params(params)?;
        json!({"name": name})
    };
    args["caller"] = caller.into();
    let (reply, rx) = mpsc::channel();
    let gone = || RpcError::new(RpcError::REJECTED, "the agent does not answer");
    (tx.send(Input::AgentCall(method.to_owned(), args, reply))).map_err(|_| gone())?;
    let answer = rx.recv_timeout(ANSWER).map_err(|_| gone())?;
    match answer["error"].as_str() {
        Some(error) => Err(RpcError::new(
            RpcError::REJECTED,
            format!("{method}: {error}"),
        )),
        None => Ok(answer),
    }
}

/// The refusal of `method` for a confined caller.
fn confined_refusal(method: &str) -> RpcError {
    RpcError::new(
        RpcError::REJECTED,
        format!("{method}: not for a confined connection"),
    )
}

/// Whether `caller` may claim the indicator and answer (review A1): an
/// unconfined connection of a name in `indicator_clients`, or of a name
/// that holds the grant `indicator.claim`; `grants` answers `grants.list`
/// of the core, asked only for a name outside the setting.
fn may_answer(
    config: &Config,
    caller: &str,
    confined: bool,
    grants: impl FnOnce() -> Value,
) -> bool {
    let granted = |grants: Value| {
        (grants[caller].as_array())
            .is_some_and(|held| held.iter().any(|c| c == Capability::IndicatorClaim.name()))
    };
    !confined && (config.indicator_clients.iter().any(|c| c == caller) || granted(grants()))
}

/// The refusal of `indicator.claim` and `confirm.answer`.
fn not_allowed(method: &str) -> RpcError {
    RpcError::new(
        RpcError::REJECTED,
        format!(
            "{method}: only an unconfined client of indicator_clients, or with the grant \
             indicator.claim, that claimed the indicator"
        ),
    )
}

/// `indicator.claim` of `caller`: only a client that [`may_answer`]
/// claims, as a claim lets it answer the questions of the agent.
fn claim_indicator(claims: &mut Claims, caller: &str, allowed: bool) -> Result<Value, RpcError> {
    if !allowed {
        return Err(not_allowed("indicator.claim"));
    }
    claims.claim(caller);
    Ok(json!({}))
}

/// `confirm.answer` of `caller`: only a client that [`may_answer`] and
/// claimed the indicator, a HUD plugin, answers, and only the open
/// question.
fn confirm_answer(
    claims: &Claims,
    tx: &Sender<Input>,
    caller: &str,
    allowed: bool,
    params: Value,
) -> Result<Value, RpcError> {
    if !allowed || !claims.0.contains(caller) {
        return Err(not_allowed("confirm.answer"));
    }
    let ButtonParams { id, yes } = runtime::params(params)?;
    let id = (id.parse()).map_err(|_| RpcError::new(RpcError::INVALID_PARAMS, "id: a number"))?;
    let (open, rx) = mpsc::channel();
    let gone = || RpcError::new(RpcError::REJECTED, "the core does not answer");
    (tx.send(Input::ConfirmButton { id, yes, open })).map_err(|_| gone())?;
    match rx.recv_timeout(ANSWER) {
        Ok(true) => Ok(json!({})),
        Ok(false) => Err(RpcError::new(
            RpcError::REJECTED,
            format!("confirm.answer: no open question {id}"),
        )),
        Err(_) => Err(gone()),
    }
}

/// The parameters of `talk`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Talk {
    down: bool,
}

/// The parameters of `mute`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mute {
    mic: Option<bool>,
    speaker: Option<bool>,
}

/// Starts the thread. It sends the inputs of chora to the core, and
/// `ChoraGone` when the runtime gives up. It emits the events of `emits`
/// as `NAME.EVENT`.
pub fn start(
    name: String,
    settings: settings::File<Config>,
    tx: Sender<Input>,
    emits: Receiver<crate::mc::Emit>,
) {
    thread::spawn(move || {
        let path = settings.file().to_owned();
        let mut settings = Some(settings);
        runtime::run(&name, || {
            let settings = match settings.take() {
                Some(s) => s,
                None => load(path.clone())?,
            };
            session(&name, settings, &tx, &emits)
        });
        let _ = tx.send(Input::ChoraGone);
    });
}

/// Reads the settings file again; a broken file is an error, never the
/// defaults.
fn load(path: PathBuf) -> Result<settings::File<Config>, String> {
    settings::File::load(path, "{}").and_then(crate::unbroken)
}

fn session(
    name: &str,
    settings: settings::File<Config>,
    tx: &Sender<Input>,
    emits: &Receiver<crate::mc::Emit>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let rt = Runtime::start(name, Vec::new(), &["client.disconnected"], methods)?;
    let mut client = Client {
        rt,
        settings,
        tx: tx.clone(),
        claims: Claims::default(),
        confirming: false,
    };
    // The claims of the session before are gone.
    let _ = tx.send(Input::Claimed(false));
    let (reply, _) = mpsc::channel();
    let _ = tx.send(Input::AgentCall("chat.gone".into(), json!({}), reply));
    // A new session reads the file again, which an edit may have changed.
    client.apply().map_err(|e| e.message)?;
    crate::log::info(&format!("connected to chora as {name}"));
    crate::health::ok("chora");
    let result = serve(&mut client, name, emits);
    if let Err(e) = &result {
        crate::health::down("chora", &e.to_string());
    }
    // A key that nobody follows is up.
    let _ = tx.send(Input::PttUp);
    Ok(result?)
}

/// Handles the events of chora and emits the events of the HUD until the
/// connection fails.
#[allow(clippy::too_many_lines, reason = "one arm per event")]
fn serve(
    client: &mut Client,
    name: &str,
    emits: &Receiver<crate::mc::Emit>,
) -> Result<(), ClientError> {
    let mut held = Held::default();
    loop {
        match client.rt.conn.next_event_timeout(EMIT_POLL)? {
            // The runtime drops the caller, which a claim needs.
            Some(Event::MethodCalled {
                call,
                caller,
                method,
                confined,
                ..
            }) if method == "indicator.claim" => {
                let allowed = client.may_answer(&caller, confined);
                let answer = claim_indicator(&mut client.claims, &caller, allowed);
                if answer.is_ok() {
                    let _ = client.tx.send(Input::Claimed(true));
                }
                client.rt.conn.answer(call, answer)?;
            }
            Some(Event::MethodCalled {
                call,
                caller,
                method,
                params,
                confined,
            }) if method == "confirm.answer" => {
                let allowed = client.may_answer(&caller, confined);
                let answer = confirm_answer(&client.claims, &client.tx, &caller, allowed, params);
                client.rt.conn.answer(call, answer)?;
            }
            Some(Event::MethodCalled {
                call,
                caller,
                method,
                params,
                confined,
            }) if CHAT_METHODS.contains(&method.as_str()) => {
                let answer = chat_call(&client.tx, &caller, confined, &method, params);
                client.rt.conn.answer(call, answer)?;
            }
            Some(Event::MethodCalled {
                call,
                method,
                confined: true,
                ..
            }) if not_for_confined(&method) => {
                client
                    .rt
                    .conn
                    .answer(call, Err(confined_refusal(&method)))?;
            }
            // Any client may read the settings: without the header values.
            Some(Event::MethodCalled { call, method, .. }) if method == "settings.get" => {
                let settings = without_headers(client.settings.get());
                client.rt.conn.answer(call, Ok(settings))?;
            }
            Some(event) => runtime::dispatch(client, event)?,
            None => {}
        }
        let new: Vec<crate::mc::Emit> = emits.try_iter().collect();
        for (event, data) in &new {
            if *event == "confirm" {
                client.confirm_keys(data)?;
            }
        }
        let new = new.into_iter().filter(|_| hud_client(name));
        let new: Vec<crate::mc::Emit> = new.collect();
        held.emit(new.into_iter(), Instant::now(), |event, data| {
            let emit = Emit {
                event: format!("{VOICE}.{event}"),
                data: data.clone(),
            };
            client
                .rt
                .conn
                .call::<_, Value>("events.emit", emit)
                .map(drop)
        })?;
    }
}

/// The settings with each value of a `*_headers` key hidden (review rc3
/// round 2, 4.2): a header such as `Authorization` carries a key.
pub fn without_headers(mut settings: Value) -> Value {
    for (key, headers) in settings.as_object_mut().into_iter().flatten() {
        if key.ends_with("_headers") {
            headers
                .as_object_mut()
                .into_iter()
                .flatten()
                .for_each(|(_, v)| *v = "(hidden)".into());
        }
    }
    settings
}

/// The states that the core refused over its rate limit (D65): the last
/// `state` or `remove` of each name waits for `RESEND` and goes again,
/// and so do the states that come meanwhile. The last `confirm` waits
/// the same way. A refused `say` is gone, as its sound plays now; another
/// refusal only logs.
#[derive(Default)]
struct Held {
    events: BTreeMap<String, crate::mc::Emit>,
    until: Option<Instant>,
}

impl Held {
    fn emit(
        &mut self,
        new: impl Iterator<Item = crate::mc::Emit>,
        now: Instant,
        mut send: impl FnMut(&str, &Value) -> Result<(), ClientError>,
    ) -> Result<(), ClientError> {
        let mut out: Vec<crate::mc::Emit> = Vec::new();
        if self.until.is_some_and(|t| now >= t) {
            self.until = None;
            out.extend(std::mem::take(&mut self.events).into_values());
        }
        out.extend(new);
        for (event, data) in out {
            // The sub-agent of the state, or none for the agent; a NUL
            // is in no name.
            let key = match event {
                "state" | "remove" => Some(data["name"].as_str().unwrap_or_default().to_owned()),
                "confirm" => Some("\0confirm".to_owned()),
                _ => None,
            };
            if let (Some(key), Some(_)) = (&key, self.until) {
                self.events.insert(key.clone(), (event, data));
                continue;
            }
            match send(event, &data) {
                Err(ClientError::Rpc(e)) if e.code == RpcError::LIMIT_REACHED => {
                    if let Some(key) = key {
                        self.events.insert(key, (event, data));
                        self.until = Some(now + RESEND);
                    }
                }
                Err(ClientError::Rpc(e)) => crate::log::debug(&format!("events.emit: {e}")),
                other => other?,
            }
        }
        Ok(())
    }
}

#[allow(clippy::too_many_lines)] // one entry per method
fn exported_methods() -> Vec<ExportMethod> {
    let none = json!({});
    vec![
        runtime::method(
            "state",
            "Answers the state of the voice agent: state, since_ms, seq, awake, muted, turn, \
             last_transcript and last_reply.",
            none.clone(),
            &[],
        ),
        runtime::method(
            "cancel",
            "Stops the current turn; a coding task goes on.",
            none.clone(),
            &[],
        ),
        runtime::method(
            "sleep",
            "Lets the agent sleep: it hears only its wake words and the push-to-talk key.",
            none.clone(),
            &[],
        ),
        runtime::method("wake", "Wakes the agent up.", none.clone(), &[]),
        // Privileged (review rc3 round 2, 4.5): an unmuted mic, a pressed
        // key or a hidden dot lets a client record without a sign. The
        // export flag cannot depend on `mic`, so all of `mute` is.
        runtime::privileged(runtime::method(
            "mute",
            "Mutes or unmutes the mic and the speaker, each on its own; a muted mic still \
             records with the push-to-talk key.",
            json!({"mic": {"type": "boolean"}, "speaker": {"type": "boolean"}}),
            &[],
        )),
        // Privileged: a yes runs programs and allows what sub-agents ask.
        runtime::privileged(runtime::method(
            "confirm",
            "Answers the question of the agent before a tool call: yes runs the call, no \
             declines it.",
            json!({"answer": {"type": "string", "enum": ["yes", "no"]}}),
            &["answer"],
        )),
        // Privileged, and only a claimer of the indicator: a yes runs
        // programs and changes security settings.
        runtime::privileged(runtime::method(
            "confirm.answer",
            "The Yes or No button of a HUD that holds indicator.claim: answers the question \
             of voice.confirm with this id, also one that waits for a key.",
            json!({"id": {"type": "string"}, "yes": {"type": "boolean"}}),
            &["id", "yes"],
        )),
        runtime::privileged(runtime::method(
            "talk",
            "The push-to-talk key of another client: down stops a reply and records, up \
             ends the utterance.",
            json!({"down": {"type": "boolean"}}),
            &["down"],
        )),
        runtime::method(
            "health",
            "Answers the health of the servers asr, tts, llm, mcp and chora: status ok, down \
             or unknown, the last error and its Unix time error_at.",
            none.clone(),
            &[],
        ),
        // Privileged (D62): it starts programs.
        runtime::privileged(runtime::method(
            "agent",
            "Runs a tool of the ACP agents without the LLM: tool is agent_list, agent_start \
             (project, name, voice), agent_say (name, text), agent_stop (name) or \
             agent_status (name); answers its text.",
            json!({"tool": {"type": "string"}, "project": {"type": "string"},
                "name": {"type": "string"}, "voice": {"type": "string"},
                "text": {"type": "string"}}),
            &["tool"],
        )),
        runtime::privileged(runtime::method(
            "agent.read",
            "Answers the entries of the transcript of an ACP agent after the number after: \
             entries with seq, role and text, and the last seq; at most 64 KiB of text.",
            json!({"name": {"type": "string"}, "after": {"type": "integer"}}),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "agent.cancel",
            "Cancels the running prompt of an ACP agent and the prompts that wait.",
            json!({"name": {"type": "string"}}),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "chat.open",
            "Marks a chat of the caller with an ACP agent as open: the agent speaks its \
             questions but not its results. The mark ends with chat.close or when the \
             caller disconnects. A chat answers no question.",
            json!({"name": {"type": "string"}}),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "chat.close",
            "Ends the chat that the caller opened with an ACP agent.",
            json!({"name": {"type": "string"}}),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "indicator.claim",
            "Hides the state dot of the agent while the calling client is connected: the \
             caller shows the state itself. Call it again after the voice client connects.",
            none,
            &[],
        )),
    ]
}

impl Client {
    /// Grabs the push-to-talk key; a key that another client holds only
    /// logs.
    fn grab(&mut self) -> Result<(), ClientError> {
        let key = &self.settings.value.ptt_key;
        let binding = key.parse().expect("the settings check the key");
        if !self.rt.grab(binding, Key::Ptt)? {
            crate::log::warn(&format!("no push-to-talk key {key}"));
        }
        Ok(())
    }

    /// Grabs the keys of `confirm_keys` while a question waits for them
    /// (`physical`), and lets them go when a question ends.
    fn confirm_keys(&mut self, data: &Value) -> Result<(), ClientError> {
        if data["physical"] == true && !self.confirming {
            self.confirming = true;
            let keys = self.settings.value.confirm_keys.clone();
            for (key, yes) in [(&keys.yes, true), (&keys.no, false)] {
                let binding = key.parse().expect("the settings check the keys");
                if !self.rt.grab(binding, Key::Confirm(yes))? {
                    crate::log::warn(&format!("no confirm key {key}"));
                }
            }
        } else if data["done"] == true && self.confirming {
            self.confirming = false;
            self.rt.ungrab_all()?;
            self.grab()?;
        }
        Ok(())
    }

    /// Sends a call to the core and waits for the state record.
    fn ask(&self, call: Call) -> Result<Value, RpcError> {
        let (tx, rx) = mpsc::channel();
        let gone = || RpcError::new(RpcError::REJECTED, "the core does not answer");
        self.tx.send(Input::Call(call, tx)).map_err(|_| gone())?;
        rx.recv_timeout(ANSWER).map_err(|_| gone())
    }

    /// [`may_answer`] with the settings and `grants.list` of the core; a
    /// core that does not answer grants nothing.
    fn may_answer(&mut self, caller: &str, confined: bool) -> bool {
        let conn = &mut self.rt.conn;
        may_answer(&self.settings.value, caller, confined, || {
            (conn.call_within("grants.list", Value::Null, ANSWER)).unwrap_or_default()
        })
    }

    /// Hands the settings to the core and grabs the key again.
    fn apply(&mut self) -> Result<(), RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        self.rt.ungrab_all().map_err(rejected)?;
        self.grab().map_err(rejected)?;
        let config = Box::new(self.settings.value.clone());
        self.tx
            .send(Input::Settings(config))
            .map_err(|_| RpcError::new(RpcError::REJECTED, "the core does not answer"))
    }
}

/// What a grabbed key means. An injected key never answers a question
/// about a security setting: only the keyboard does.
fn key_input(key: Key, state: PressState, injected: bool) -> Option<Input> {
    match (key, state) {
        (Key::Ptt, PressState::Pressed) => Some(Input::PttDown),
        (Key::Ptt, PressState::Released) => Some(Input::PttUp),
        (Key::Confirm(yes), PressState::Pressed) if !injected => Some(Input::ConfirmKey(yes)),
        (Key::Confirm(_), _) => None,
    }
}

impl Plugin for Client {
    type Action = Key;

    fn runtime(&mut self) -> &mut Runtime<Key> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if let Event::ClientDisconnected { name } = &event {
            if self.claims.0.contains(name) {
                let _ = self.tx.send(Input::Claimed(self.claims.gone(name)));
            }
            let (reply, _) = mpsc::channel();
            let args = json!({"caller": name});
            let _ = self
                .tx
                .send(Input::AgentCall("chat.gone".into(), args, reply));
        }
        if let Event::InputKey {
            binding,
            state,
            injected,
        } = event
            && let Some(&key) = self.rt.action(&binding)
            && let Some(input) = key_input(key, state, injected)
        {
            // The core is gone only when the process ends.
            let _ = self.tx.send(input);
        }
        Ok(())
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let call = match method {
            "health" => return Ok(crate::health::record()),
            "agent" => {
                let tool = params["tool"].as_str().unwrap_or_default().to_owned();
                if !crate::agents::TOOLS.contains(&tool.as_str()) {
                    let tools = crate::agents::TOOLS.join(", ");
                    return Err(RpcError::new(
                        RpcError::INVALID_PARAMS,
                        format!("tool: one of {tools}"),
                    ));
                }
                let (tx, rx) = mpsc::channel();
                let gone = || RpcError::new(RpcError::REJECTED, "the agent does not answer");
                (self.tx.send(Input::AgentCall(tool, params, tx))).map_err(|_| gone())?;
                return rx.recv_timeout(ANSWER).map_err(|_| gone());
            }
            "state" => Call::State,
            "cancel" => Call::Cancel,
            "sleep" => Call::Sleep,
            "wake" => Call::Wake,
            "confirm" => {
                let ConfirmParams { answer } = runtime::params(params)?;
                Call::Confirm(matches!(answer, Answer::Yes))
            }
            "talk" => {
                let Talk { down } = runtime::params(params)?;
                Call::Talk(down)
            }
            "mute" => {
                let Mute { mic, speaker } = runtime::params(params)?;
                Call::Mute { mic, speaker }
            }
            _ => return Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        };
        self.ask(call)
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.apply()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{Key, PressState, key_input};

    use chora_scene::ipc::{ClientError, ExportMethod, RpcError};
    use serde_json::{Value, json};

    use super::{
        Claims, Held, RESEND, chat_call, claim_indicator, confirm_answer, may_answer,
        without_headers,
    };
    use crate::config::Config;
    use crate::core::Input;
    use std::sync::mpsc;

    #[test]
    fn only_the_client_voice_emits() {
        assert!(super::hud_client("voice"));
        for name in ["voice-test", "voice.evil", "chora"] {
            assert!(!super::hud_client(name), "{name}");
        }
    }

    #[test]
    fn settings_get_hides_the_header_values() {
        let settings = json!({"llm_headers": {"Authorization": "Bearer sk-1"}, "llm_url": "u"});
        assert_eq!(
            without_headers(settings),
            json!({"llm_headers": {"Authorization": "(hidden)"}, "llm_url": "u"})
        );
    }

    /// An injected confirm key is ignored: only the keyboard answers.
    #[test]
    fn an_injected_confirm_key_answers_nothing() {
        let key = |injected| key_input(Key::Confirm(true), PressState::Pressed, injected);
        assert!(matches!(key(false), Some(Input::ConfirmKey(true))));
        assert!(key(true).is_none());
        let ptt = key_input(Key::Ptt, PressState::Pressed, true);
        assert!(matches!(ptt, Some(Input::PttDown)), "not a security key");
    }

    /// A claim hides the dot, the claimer's disconnect shows it again,
    /// and a second claimer keeps it hidden.
    #[test]
    fn claims_hide_the_dot_while_a_claimer_is_connected() {
        let mut claims = Claims::default();
        assert!(claims.claim("mc"));
        assert!(!claims.gone("mc"), "the dot shows again");
        assert!(claims.claim("mc"));
        assert!(claims.claim("xp"));
        assert!(claims.gone("mc"), "xp still claims");
        assert!(claims.gone("other"), "not a claimer");
        assert!(!claims.gone("xp"));
    }

    /// Only a claimer of the indicator reaches the core with
    /// `confirm.answer`; the core's no to a stale id is a refusal.
    #[test]
    fn only_a_claimer_answers_by_button() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut claims = Claims::default();
        claims.claim("mc");
        let params = json!({"id": "4", "yes": true});
        let err = confirm_answer(&claims, &tx, "evil", true, params.clone()).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert!(rx.try_recv().is_err(), "the core heard nothing");
        let core = std::thread::spawn(move || {
            for open in [true, false] {
                let Ok(Input::ConfirmButton {
                    id: 4,
                    yes: true,
                    open: tx,
                }) = rx.recv()
                else {
                    panic!("no button");
                };
                tx.send(open).unwrap();
            }
        });
        assert_eq!(
            confirm_answer(&claims, &tx, "mc", true, params.clone()),
            Ok(json!({}))
        );
        let err = confirm_answer(&claims, &tx, "mc", true, params).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED, "stale");
        core.join().unwrap();
    }

    /// A confined connection neither claims the indicator nor answers by
    /// button, even when it claimed before or holds `methods.privileged`.
    #[test]
    fn a_confined_caller_cannot_claim_or_answer() {
        let config = Config {
            indicator_clients: vec!["voice-mcp".into()],
            ..Config::default()
        };
        let grants = || json!({"voice-mcp": ["indicator.claim"]});
        assert!(!may_answer(&config, "voice-mcp", true, grants));
        let (tx, rx) = std::sync::mpsc::channel();
        let mut claims = Claims::default();
        let err = claim_indicator(&mut claims, "voice-mcp", false).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert!(!claims.0.contains("voice-mcp"), "no claim");
        claims.claim("voice-mcp");
        let params = json!({"id": "4", "yes": true});
        let err = confirm_answer(&claims, &tx, "voice-mcp", false, params).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert!(rx.try_recv().is_err(), "the core heard nothing");
        assert!(claim_indicator(&mut claims, "mc", true).is_ok());
        assert!(claims.0.contains("mc"));
    }

    /// Review A1: an unconfined client of the main socket that is neither
    /// in `indicator_clients` nor holds the grant `indicator.claim`, such
    /// as a program of `run_program`, neither claims nor answers, even
    /// after a claim.
    #[test]
    fn a_client_that_is_not_named_cannot_claim_or_answer() {
        let config = Config::default();
        let grants = || json!({"evil": ["input.inject", "methods.privileged"]});
        let allowed = may_answer(&config, "evil", false, grants);
        assert!(!allowed);
        let (tx, rx) = std::sync::mpsc::channel();
        let mut claims = Claims::default();
        let err = claim_indicator(&mut claims, "evil", allowed).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert!(!claims.0.contains("evil"), "no claim");
        claims.claim("evil");
        let params = json!({"id": "4", "yes": true});
        let err = confirm_answer(&claims, &tx, "evil", allowed, params).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert!(rx.try_recv().is_err(), "the core heard nothing");
    }

    /// Review A1: the HUDs of the default `indicator_clients` claim and
    /// answer without asking the core.
    #[test]
    fn a_named_hud_claims_and_answers() {
        let config = Config::default();
        for hud in ["assistant", "construct", "mc"] {
            let allowed = may_answer(&config, hud, false, || panic!("asked the core"));
            assert!(allowed, "{hud}");
            let mut claims = Claims::default();
            assert_eq!(claim_indicator(&mut claims, hud, allowed), Ok(json!({})));
            assert!(claims.0.contains(hud));
        }
    }

    /// Review A1: a changed `indicator_clients` and a grant or revoke of
    /// `indicator.claim` decide the next call.
    #[test]
    fn a_changed_setting_or_grant_takes_effect() {
        let mut config = Config {
            indicator_clients: vec!["construct".into()],
            ..Config::default()
        };
        assert!(!may_answer(&config, "mc", false, || json!({})));
        assert!(may_answer(&config, "construct", false, || json!({})));
        config.indicator_clients.push("hud".into());
        assert!(may_answer(&config, "hud", false, || json!({})));
        let granted = || json!({"other": ["indicator.claim"]});
        assert!(may_answer(&config, "other", false, granted));
        assert!(!may_answer(&config, "other", false, || json!({})));
        assert!(!may_answer(&config, "other", false, || Value::Null));
    }

    /// A confined connection cannot answer by `confirm`, push to talk or
    /// mute, whatever it holds; the other methods stay open to it.
    #[test]
    fn a_confined_caller_cannot_confirm_talk_or_mute() {
        for method in ["confirm", "talk", "mute"] {
            assert!(super::not_for_confined(method), "{method}");
            let err = super::confined_refusal(method);
            assert_eq!(err.code, RpcError::REJECTED, "{method}");
        }
        for method in ["state", "cancel", "sleep", "wake", "health"] {
            assert!(!super::not_for_confined(method), "{method}");
        }
    }

    /// The chat methods read a transcript, stop a prompt and mark a chat
    /// open: a confined connection gets none of them and the core hears
    /// nothing; an unconfined one reaches the core with its name.
    #[test]
    fn chat_methods_refuse_a_confined_caller() {
        let (tx, rx) = mpsc::channel();
        for method in ["agent.read", "agent.cancel", "chat.open", "chat.close"] {
            assert!(super::not_for_confined(method), "{method}");
            let params = json!({"name": "atlas"});
            let err = chat_call(&tx, "voice-mcp", true, method, params).unwrap_err();
            assert_eq!(err.code, RpcError::REJECTED, "{method}");
            assert!(rx.try_recv().is_err(), "{method}: the core heard nothing");
        }
        let params = json!({"name": "atlas"});
        let asked = std::thread::spawn(move || {
            chat_call(&tx, "chora-assistant", false, "chat.open", params)
        });
        let Ok(Input::AgentCall(tool, params, reply)) = rx.recv_timeout(Duration::from_secs(2))
        else {
            panic!("the core heard nothing");
        };
        assert_eq!(tool, "chat.open");
        assert_eq!(params["caller"], "chora-assistant");
        reply.send(json!({})).unwrap();
        assert!(asked.join().unwrap().is_ok());
    }

    /// The core answers a failure as an `error` text; the caller gets a
    /// refusal, and a bad parameter is refused before the core hears it.
    #[test]
    fn a_failed_chat_call_is_a_refusal_and_bad_parameters_never_reach_the_core() {
        let (tx, rx) = mpsc::channel();
        let err = chat_call(
            &tx,
            "hud",
            false,
            "agent.read",
            json!({"name": "a", "after": "x"}),
        )
        .unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
        let err = chat_call(
            &tx,
            "hud",
            false,
            "chat.open",
            json!({"name": "a", "extra": 1}),
        )
        .unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
        assert!(rx.try_recv().is_err());
        let asked = std::thread::spawn(move || {
            chat_call(&tx, "hud", false, "agent.cancel", json!({"name": "a"}))
        });
        let Ok(Input::AgentCall(_, _, reply)) = rx.recv_timeout(Duration::from_secs(2)) else {
            panic!("the core heard nothing");
        };
        reply.send(json!({"error": "no agent a"})).unwrap();
        let err = asked.join().unwrap().unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert!(err.message.contains("no agent a"));
    }

    /// Emits `new` at `now`; the core takes them or refuses them over its
    /// rate limit. Answers what it took.
    fn emit(
        held: &mut Held,
        new: Vec<(&'static str, Value)>,
        now: Instant,
        refuse: bool,
    ) -> Vec<(String, Value)> {
        let mut sent: Vec<(String, Value)> = Vec::new();
        held.emit(new.into_iter(), now, |event, data| {
            if refuse {
                let e = RpcError::new(RpcError::LIMIT_REACHED, "too many");
                return Err(ClientError::Rpc(e));
            }
            sent.push((event.to_owned(), data.clone()));
            Ok(())
        })
        .unwrap();
        sent
    }

    /// A burst over the rate limit: the last state of each name goes
    /// again after the wait, a removal too; a refused sentence is gone.
    #[test]
    fn refused_states_go_again_after_the_wait() {
        let mut held = Held::default();
        let none: Vec<(String, Value)> = Vec::new();
        let t0 = Instant::now();
        let state = |name: &str, s: &str| ("state", json!({"name": name, "state": s}));
        let burst = vec![
            state("a", "working"),
            ("say", json!({"text": "Hallo"})),
            state("b", "idle"),
            ("remove", json!({"name": "b"})),
            ("state", json!({"state": "listening"})),
        ];
        assert_eq!(emit(&mut held, burst, t0, true), none);
        // Within the wait a new state waits behind the others.
        assert_eq!(emit(&mut held, vec![state("a", "done")], t0, false), none);
        let before = t0 + RESEND.saturating_sub(Duration::from_millis(1));
        assert_eq!(emit(&mut held, vec![], before, false), none);
        let mut sent = emit(&mut held, vec![], t0 + RESEND, false);
        sent.sort_by_key(|(_, d)| d.to_string());
        let want = [
            ("remove", json!({"name": "b"})),
            ("state", json!({"name": "a", "state": "done"})),
            ("state", json!({"state": "listening"})),
        ];
        let mut want: Vec<(String, Value)> = want.map(|(e, d)| (e.to_owned(), d)).into();
        want.sort_by_key(|(_, d)| d.to_string());
        assert_eq!(sent, want);
        assert_eq!(emit(&mut held, vec![], t0 + RESEND * 2, false), none);
    }

    /// A yes and the agent tools start programs; mute, talk and the
    /// claim of the dot can record without a sign (review rc3 round 2,
    /// 4.5): a confined client needs `methods.privileged` for them.
    #[test]
    fn the_methods_that_run_or_record_are_privileged() {
        let privileged: Vec<String> = (super::exported_methods().into_iter())
            .filter_map(|m| match m {
                ExportMethod::Described(d) if d.privileged => Some(d.name),
                _ => None,
            })
            .collect();
        assert_eq!(
            privileged,
            [
                "mute",
                "confirm",
                "confirm.answer",
                "talk",
                "agent",
                "agent.read",
                "agent.cancel",
                "chat.open",
                "chat.close",
                "indicator.claim"
            ]
        );
    }

    /// Review rc3 round 2, 3.2: no unprivileged `restart` beside the
    /// privileged `settings.reload`.
    #[test]
    fn there_is_no_restart() {
        let names: Vec<String> = (super::exported_methods().into_iter())
            .map(|m| match m {
                ExportMethod::Described(d) => d.name,
                ExportMethod::Plain(n) => n,
            })
            .collect();
        assert!(!names.iter().any(|n| n == "restart"), "{names:?}");
    }
}
