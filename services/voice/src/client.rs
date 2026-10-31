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
    ipc::{ClientError, Emit, ExportMethod, PressState, RpcError},
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

/// `confirm.answer` of `caller`: only a client that claimed the indicator,
/// a HUD plugin, may answer, and only the open question.
fn confirm_answer(
    claims: &Claims,
    tx: &Sender<Input>,
    caller: &str,
    params: Value,
) -> Result<Value, RpcError> {
    if !claims.0.contains(caller) {
        return Err(RpcError::new(
            RpcError::REJECTED,
            "confirm.answer: only a client that holds indicator.claim",
        ));
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
                ..
            }) if method == "indicator.claim" => {
                let _ = client.tx.send(Input::Claimed(client.claims.claim(&caller)));
                client.rt.conn.answer(call, Ok(json!({})))?;
            }
            Some(Event::MethodCalled {
                call,
                caller,
                method,
                params,
            }) if method == "confirm.answer" => {
                let answer = confirm_answer(&client.claims, &client.tx, &caller, params);
                client.rt.conn.answer(call, answer)?;
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
fn without_headers(mut settings: Value) -> Value {
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

impl Plugin for Client {
    type Action = Key;

    fn runtime(&mut self) -> &mut Runtime<Key> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if let Event::ClientDisconnected { name } = &event
            && self.claims.0.contains(name)
        {
            let _ = self.tx.send(Input::Claimed(self.claims.gone(name)));
        }
        if let Event::InputKey { binding, state } = event
            && let Some(&key) = self.rt.action(&binding)
        {
            let input = match (key, state) {
                (Key::Ptt, PressState::Pressed) => Input::PttDown,
                (Key::Ptt, PressState::Released) => Input::PttUp,
                (Key::Confirm(yes), PressState::Pressed) => Input::ConfirmKey(yes),
                (Key::Confirm(_), PressState::Released) => return Ok(()),
            };
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

    use chora_scene::ipc::{ClientError, ExportMethod, RpcError};
    use serde_json::{Value, json};

    use super::{Claims, Held, RESEND, confirm_answer, without_headers};
    use crate::core::Input;

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
        let err = confirm_answer(&claims, &tx, "evil", params.clone()).unwrap_err();
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
            confirm_answer(&claims, &tx, "mc", params.clone()),
            Ok(json!({}))
        );
        let err = confirm_answer(&claims, &tx, "mc", params).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED, "stale");
        core.join().unwrap();
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
