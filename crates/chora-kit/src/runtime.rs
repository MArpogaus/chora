//! The runtime that every plugin repeats: it connects, subscribes to the
//! events, exports the methods, grabs bindings, answers method calls and
//! `settings.get`, `settings.set` and `settings.reload`, keeps to the
//! outputs of `--output`, and comes back after the core closed the
//! connection.

use std::{
    process::ExitCode,
    thread::sleep,
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Event, OutputInfo,
    ipc::{ClientError, DescribedMethod, Export, ExportMethod, Grab, RpcError, Subscribe},
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::{conn::Conn, settings::Store};

/// The time between two tries to grab give keys that another client held.
const REGRAB: Duration = Duration::from_secs(2);

/// Whether the environment says that this is a test session, which takes
/// the keys of `input.inject`.
fn test_session() -> bool {
    std::env::var("CHORA_TEST_ACCEPT_INJECTED_KEYS").is_ok_and(|v| v == "1")
}

/// A plugin on the runtime: it handles the events and the calls of its
/// exported methods.
pub trait Plugin {
    /// What a grabbed binding does.
    type Action;

    /// The runtime of the plugin.
    fn runtime(&mut self) -> &mut Runtime<Self::Action>;

    /// Handles an event other than a method call. Events of outputs that
    /// `--output` leaves to others do not come here.
    ///
    /// # Errors
    ///
    /// The socket fails.
    fn event(&mut self, event: Event) -> Result<(), ClientError>;

    /// Runs an exported method; the runtime sends the answer.
    ///
    /// # Errors
    ///
    /// The method fails; the caller gets the error.
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError>;

    /// The settings behind `settings.get` and `settings.set`; none by
    /// default. The plugin exports the methods of its settings file.
    fn settings(&mut self) -> Option<&mut dyn Store> {
        None
    }

    /// Applies the settings after a `settings.set`.
    ///
    /// # Errors
    ///
    /// The change cannot be applied; the caller gets the error.
    fn settings_changed(&mut self) -> Result<(), RpcError> {
        Ok(())
    }
}

/// The connection of a plugin, with its outputs and grabs.
pub struct Runtime<A> {
    /// The connection to the core.
    pub conn: Conn,
    name: String,
    /// The outputs that `--output` leaves to the plugin; all when empty.
    outputs: Vec<String>,
    /// The grabbed bindings, as the core names them.
    grabs: Vec<(String, A)>,
    /// The bindings whose refusal was logged, so that each logs once.
    refused: std::collections::HashSet<String>,
    /// Makes the action of a give key; none when the plugin has no give
    /// keys.
    give_keys: Option<fn(&'static str) -> A>,
    /// The last try to grab give keys that another client held.
    give_tried: Option<Instant>,
    /// Whether a key of `input.inject` runs a binding (test sessions).
    inject_ok: bool,
    /// Whether the connection of the caller of the method that runs now is
    /// confined, for a method that acts in the name of the plugin.
    pub confined_call: bool,
}

impl<A> Runtime<A> {
    /// Connects as `name`, subscribes to `events` and exports `methods`.
    /// `outputs` are the names of `--output`.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core refuses the `hello`, or another
    /// instance exports the methods.
    pub fn start(
        name: &str,
        outputs: Vec<String>,
        events: &[&str],
        methods: Vec<ExportMethod>,
    ) -> Result<Self, ClientError> {
        let mut rt = Self::connect(name, outputs)?;
        rt.announce(events, methods)?;
        Ok(rt)
    }

    /// Connects as `name` without events and methods, for a plugin that
    /// builds its scene before it calls [`Self::announce`].
    ///
    /// # Errors
    ///
    /// The socket fails, or the core refuses the `hello`.
    pub fn connect(name: &str, outputs: Vec<String>) -> Result<Self, ClientError> {
        Ok(Self::with(Conn::connect(name)?, name, outputs))
    }

    /// A runtime on a connection, for example of [`Conn::connect_at`].
    #[must_use]
    pub fn with(conn: Conn, name: &str, outputs: Vec<String>) -> Self {
        Self {
            conn,
            name: name.into(),
            outputs,
            grabs: Vec::new(),
            refused: std::collections::HashSet::new(),
            give_keys: None,
            give_tried: None,
            inject_ok: test_session(),
            confined_call: false,
        }
    }

    /// Sets whether a key of `input.inject` runs a binding of the plugin;
    /// off by default, since only a person presses the keys of a
    /// window manager. The environment variable
    /// `CHORA_TEST_ACCEPT_INJECTED_KEYS=1` of the test sessions turns it
    /// on too.
    pub fn accept_injected(&mut self, setting: bool) {
        self.inject_ok = setting || test_session();
    }

    /// Whether the plugin takes input of `input.inject` as the person's:
    /// the setting of [`Self::accept_injected`] or a test session.
    #[must_use]
    pub fn accepts_injected(&self) -> bool {
        self.inject_ok
    }

    /// Sets the action of a give key of [`crate::give::KEYS`] by its
    /// target, or none for no give keys.
    pub fn give_keys(&mut self, make: Option<fn(&'static str) -> A>) {
        self.give_keys = make;
    }

    /// Whether the plugin holds a binding, such as `Super+Alt+1`.
    #[must_use]
    pub fn holds(&self, binding: &str) -> bool {
        binding
            .parse::<Binding>()
            .is_ok_and(|b| self.grabs.iter().any(|(held, _)| *held == b.to_string()))
    }

    /// Grabs the give keys that the plugin does not hold; a key that
    /// another client holds stays free for the next try.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn grab_give_keys(&mut self) -> Result<(), ClientError> {
        let Some(make) = self.give_keys else {
            return Ok(());
        };
        for (key, to) in crate::give::KEYS {
            let Ok(binding) = key.parse::<Binding>() else {
                continue;
            };
            let text = binding.to_string();
            if self.grabs.iter().any(|(b, _)| *b == text) {
                continue;
            }
            self.grab(binding, make(to))?;
        }
        Ok(())
    }

    /// As [`Self::grab_give_keys`], at most every [`REGRAB`], so that a
    /// plugin calls it per event: a client that held a key may have gone.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn regrab_give_keys(&mut self) -> Result<(), ClientError> {
        let now = Instant::now();
        if self.give_keys.is_none() || self.give_tried.is_some_and(|t| now < t + REGRAB) {
            return Ok(());
        }
        self.give_tried = Some(now);
        self.grab_give_keys()
    }

    /// Subscribes to `events` and exports `methods`. The events of the
    /// changes before, such as the detach of the surfaces of a removed
    /// group, do not come.
    ///
    /// # Errors
    ///
    /// The socket fails, or another instance exports the methods.
    pub fn announce(
        &mut self,
        events: &[&str],
        methods: Vec<ExportMethod>,
    ) -> Result<(), ClientError> {
        self.conn.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: events.iter().map(|&e| e.into()).collect(),
            },
        )?;
        self.conn
            .call::<_, Value>("methods.export", Export { names: methods })
            .map(drop)
    }

    /// Whether `--output` leaves an output to the plugin.
    #[must_use]
    pub fn manages(&self, output: &str) -> bool {
        self.outputs.is_empty() || self.outputs.iter().any(|o| o == output)
    }

    /// Whether `--output` names outputs.
    #[must_use]
    pub fn filters(&self) -> bool {
        !self.outputs.is_empty()
    }

    /// The outputs of the plugin, as the core has them.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn outputs(&mut self) -> Result<Vec<OutputInfo>, ClientError> {
        let mut outputs: Vec<OutputInfo> = self.conn.call("output.list", Value::Null)?;
        outputs.retain(|o| self.manages(&o.name));
        Ok(outputs)
    }

    /// Grabs a binding for an action. False when the core refuses it, for
    /// example while another client holds it; the first refusal of a
    /// binding logs a line.
    ///
    /// # Errors
    ///
    /// The socket fails, or the core knows no such key, so that a
    /// `settings.set` with it fails.
    pub fn grab(&mut self, binding: Binding, action: A) -> Result<bool, ClientError> {
        let text = binding.to_string();
        if let Some(held) = self.grabs.iter_mut().find(|(b, _)| *b == text) {
            held.1 = action;
            return Ok(true);
        }
        match self.conn.call::<_, String>("input.grab", Grab { binding }) {
            Ok(normal) => {
                self.grabs.push((normal, action));
                Ok(true)
            }
            Err(ClientError::Rpc(err)) if err.code == RpcError::INVALID_PARAMS => {
                Err(RpcError::new(err.code, format!("{text}: {}", err.message)).into())
            }
            Err(ClientError::Rpc(err)) => {
                if self.refused.insert(text.clone()) {
                    eprintln!("chora-{}: {text}: {err}", self.name);
                }
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    /// Lets all grabbed bindings go, except the give keys while the
    /// plugin has them (see [`Self::give_keys`]): no other client may take
    /// them between two grabs of the plugin.
    ///
    /// # Errors
    ///
    /// The socket fails, the core refuses an ungrab, or the core named a
    /// grabbed binding that does not parse.
    pub fn ungrab_all(&mut self) -> Result<(), ClientError> {
        let keep = |normal: &str, serves: bool| {
            serves
                && crate::give::KEYS
                    .iter()
                    .filter_map(|(key, _)| key.parse::<Binding>().ok())
                    .any(|b| b.to_string() == normal)
        };
        let serves = self.give_keys.is_some();
        let (kept, dropped): (Vec<_>, Vec<_>) = std::mem::take(&mut self.grabs)
            .into_iter()
            .partition(|(normal, _)| keep(normal, serves));
        self.grabs = kept;
        for (normal, _) in dropped {
            let binding = normal.parse().map_err(|err| {
                RpcError::new(RpcError::INVALID_PARAMS, format!("{normal}: {err}"))
            })?;
            self.conn
                .call::<_, Value>("input.ungrab", Grab { binding })?;
        }
        Ok(())
    }

    /// The action of a grabbed binding, as the core names it.
    #[must_use]
    pub fn action(&self, binding: &str) -> Option<&A> {
        self.grabs
            .iter()
            .find(|(b, _)| b == binding)
            .map(|(_, a)| a)
    }
}

/// Handles events until the connection fails.
///
/// # Errors
///
/// The socket fails or closes.
pub fn serve<P: Plugin>(plugin: &mut P) -> Result<(), ClientError> {
    loop {
        let event = plugin.runtime().conn.next_event()?;
        dispatch(plugin, event)?;
    }
}

/// Answers a method call, drops the events of outputs of others and hands
/// the rest to the plugin. The call may have ended before the answer, for
/// example when its caller left; that is no reason to exit. A plugin with
/// a loop of its own, for example for animations, calls it per event.
///
/// # Errors
///
/// The socket fails.
pub fn dispatch<P: Plugin>(plugin: &mut P, event: Event) -> Result<(), ClientError> {
    let rt = plugin.runtime();
    match event {
        Event::MethodCalled {
            call,
            method,
            params,
            confined,
            ..
        } => {
            rt.confined_call = confined;
            rt.conn.serving_confined = confined;
            let result = match method.as_str() {
                "settings.get" => {
                    (plugin.settings().map(|s| s.get())).ok_or_else(|| no_settings("settings.get"))
                }
                "settings.set" => set_settings(plugin, params),
                "settings.reload" => reload_settings(plugin),
                _ => plugin.method(&method, params),
            };
            let rt = plugin.runtime();
            rt.conn.serving_confined = false;
            match rt.conn.answer(call, result.map_err(capped)) {
                Err(ClientError::Rpc(err)) => {
                    eprintln!("chora-{}: {err}", rt.name);
                    Ok(())
                }
                other => other,
            }
        }
        Event::OutputChanged { output } if !rt.manages(&output.name) => Ok(()),
        Event::OutputRemoved { name } if !rt.manages(&name) => Ok(()),
        Event::InputKey { injected: true, .. } if !rt.inject_ok => Ok(()),
        event => {
            rt.regrab_give_keys()?;
            plugin.event(event)
        }
    }
}

/// `settings.set`: checks the change, has the plugin apply it, then saves
/// it. A change that the plugin cannot apply is taken back and not saved.
/// A change that cannot be saved stays applied, and the caller gets the
/// error.
///
/// # Errors
///
/// The plugin has no settings, a change is wrong, the plugin cannot apply
/// it, or the file cannot be written.
pub fn set_settings<P: Plugin>(plugin: &mut P, changes: Value) -> Result<Value, RpcError> {
    let no_settings = || no_settings("settings.set");
    let answer = plugin.settings().ok_or_else(no_settings)?.set(changes)?;
    apply(plugin)?;
    plugin
        .settings()
        .ok_or_else(no_settings)?
        .save()
        .map_err(|e| RpcError::new(e.code, format!("applied, not saved: {}", e.message)))?;
    Ok(answer)
}

/// The most bytes of the message of an error that a method answers.
const MAX_MESSAGE: usize = 512;

/// An error with its message cut to [`MAX_MESSAGE`] bytes. A parser may
/// quote a whole broken file, and an answer past the line limit of the
/// core costs the plugin its connection.
fn capped(mut err: RpcError) -> RpcError {
    if err.message.len() > MAX_MESSAGE {
        let mut end = MAX_MESSAGE;
        while !err.message.is_char_boundary(end) {
            end -= 1;
        }
        err.message.truncate(end);
        err.message.push('…');
    }
    err
}

/// The error of a settings method of a plugin without settings.
fn no_settings(method: &str) -> RpcError {
    RpcError::new(RpcError::METHOD_NOT_FOUND, method)
}

/// `settings.reload`: reads the settings file again, after an edit by
/// hand, and has the plugin apply it. A wrong file, or one that the plugin
/// cannot apply, changes nothing. Public for a binding of the plugin
/// that reloads, which has no method call to answer.
///
/// # Errors
///
/// The plugin has no settings, the file is wrong, or the plugin cannot
/// apply it.
pub fn reload_settings<P: Plugin>(plugin: &mut P) -> Result<Value, RpcError> {
    let answer = (plugin.settings())
        .ok_or_else(|| no_settings("settings.reload"))?
        .reload()?;
    apply(plugin)?;
    Ok(answer)
}

/// Has the plugin apply its settings; takes them back when it fails, and
/// has the plugin apply the old ones again, since it may have applied a
/// part, for example ungrabbed its bindings before a new one failed.
fn apply<P: Plugin>(plugin: &mut P) -> Result<(), RpcError> {
    plugin.settings_changed().inspect_err(|_| {
        if let Some(settings) = plugin.settings() {
            settings.undo();
            if let Err(err) = plugin.settings_changed() {
                eprintln!("the old settings failed to apply again: {err}");
            }
        }
    })
}

/// An exported method with its description and the schema of its
/// parameters, as `methods.list` answers it.
#[must_use]
pub fn method(name: &str, description: &str, properties: Value, required: &[&str]) -> ExportMethod {
    let mut schema = json!({"type": "object", "required": required});
    schema["properties"] = properties;
    ExportMethod::Described(DescribedMethod {
        name: name.into(),
        description: Some(description.into()),
        input_schema: Some(schema),
        privileged: false,
    })
}

/// A [`method`] that a caller on a client socket may call only with the
/// capability `methods.privileged`: one that runs a command.
#[must_use]
pub fn privileged(method: ExportMethod) -> ExportMethod {
    match method {
        ExportMethod::Described(d) => ExportMethod::Described(DescribedMethod {
            privileged: true,
            ..d
        }),
        ExportMethod::Plain(name) => ExportMethod::Described(DescribedMethod {
            name,
            description: None,
            input_schema: None,
            privileged: true,
        }),
    }
}

/// Parses the parameters of an exported method. No parameters (`null`)
/// read as an empty object, so the defaults of `T` apply.
///
/// # Errors
///
/// The parameters do not fit `T`.
pub fn params<T: DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    let read = |params| {
        serde_json::from_value(params)
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))
    };
    if params.is_null()
        && let Ok(none) = serde_json::from_value(Value::Null)
    {
        return Ok(none);
    }
    read(if params.is_null() { json!({}) } else { params })
}

/// Runs sessions of the plugin `name` until one ends without an error.
/// The core may close the connection and stay: the emergency reset
/// revokes grants and disconnects, and the scene survives it. Come back
/// in that case; give up when only failures follow within the episode.
pub fn run(
    name: &str,
    mut session: impl FnMut() -> Result<(), Box<dyn std::error::Error>>,
) -> ExitCode {
    let mut failing = Duration::ZERO;
    loop {
        let start = Instant::now();
        match session() {
            Ok(()) => return ExitCode::SUCCESS,
            Err(err) => {
                failing = next_failing(start.elapsed(), failing);
                if failing >= COME_BACK {
                    eprintln!("chora-{name}: {err}");
                    return ExitCode::FAILURE;
                }
                eprintln!("chora-{name}: {err}; I come back");
                sleep(TRY_AGAIN);
            }
        }
    }
}

/// How long a served session must last to start a new episode of errors.
const SERVED: Duration = Duration::from_millis(500);
/// How long one episode of failures may last before the plugin gives up.
const COME_BACK: Duration = Duration::from_secs(30);
/// The wait between two attempts.
const TRY_AGAIN: Duration = Duration::from_millis(500);

/// The failures of the current episode after a session of `served`.
fn next_failing(served: Duration, failing: Duration) -> Duration {
    if served >= SERVED {
        TRY_AGAIN
    } else {
        failing + served + TRY_AGAIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{
        File,
        tests::{DEFAULT, Demo, dir},
    };

    #[derive(Debug, serde::Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Faces {
        #[serde(default)]
        faces: usize,
    }

    #[test]
    fn no_params_read_as_the_defaults() {
        assert_eq!(params::<Faces>(Value::Null).unwrap(), Faces { faces: 0 });
        assert_eq!(params::<Option<u8>>(Value::Null).unwrap(), None);
        assert!(params::<Faces>(json!({"sides": 3})).is_err());
    }

    /// A plugin whose apply of settings fails on demand.
    struct Fake {
        settings: File<Demo>,
        /// The gap whose apply fails.
        fail: Option<u16>,
        applied: u16,
    }

    impl Plugin for Fake {
        type Action = ();

        fn runtime(&mut self) -> &mut Runtime<()> {
            unreachable!("set_settings needs no connection")
        }

        fn event(&mut self, _: Event) -> Result<(), ClientError> {
            Ok(())
        }

        fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
            Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method))
        }

        fn settings(&mut self) -> Option<&mut dyn Store> {
            Some(&mut self.settings)
        }

        /// Applies a part before it fails, as a plugin that ungrabs its
        /// bindings and then fails to grab a new one.
        fn settings_changed(&mut self) -> Result<(), RpcError> {
            self.applied = self.settings.value.gap;
            if self.fail == Some(self.applied) {
                return Err(RpcError::new(RpcError::REJECTED, "cannot apply"));
            }
            Ok(())
        }
    }

    /// Check, apply, save: a change that fails to apply leaves the file
    /// as it was; a change that fails to save stays applied.
    #[test]
    fn set_settings_applies_before_it_saves() {
        let dir = dir("runtime-set");
        let path = dir.join("demo.yaml");
        let settings = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        let mut plugin = Fake {
            settings,
            fail: Some(8),
            applied: 0,
        };
        assert!(set_settings(&mut plugin, json!({"gap": 8})).is_err());
        assert!(!path.exists(), "a change that fails to apply is not saved");
        assert_eq!(plugin.settings.value.gap, 4, "and taken back");
        assert_eq!(plugin.applied, 4, "the old settings apply again");
        plugin.fail = None;
        set_settings(&mut plugin, json!({"gap": 9})).unwrap();
        assert_eq!(plugin.applied, 9);
        assert!(std::fs::read_to_string(&path).unwrap().contains("gap: 9"));

        // A directory where the file should be: the save fails.
        let blocked = dir.join("blocked.yaml");
        plugin.settings = File::<Demo>::load(blocked.clone(), DEFAULT).unwrap();
        std::fs::create_dir_all(&blocked).unwrap();
        let err = set_settings(&mut plugin, json!({"gap": 10})).unwrap_err();
        assert_eq!(err.code, RpcError::REJECTED);
        assert_eq!(plugin.applied, 10, "the change stays applied");
        assert_eq!(plugin.settings.value.gap, 10);
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// An edit by hand applies with a reload; a wrong edit changes nothing.
    #[test]
    fn reload_settings_reads_the_file_and_applies_it() {
        let dir = dir("runtime-reload");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        let settings = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        let mut plugin = Fake {
            settings,
            fail: None,
            applied: 0,
        };
        std::fs::write(&path, "gap: 6\n").unwrap();
        assert_eq!(reload_settings(&mut plugin).unwrap()["gap"], 6);
        assert_eq!(plugin.applied, 6);
        std::fs::write(&path, "gap: wide\n").unwrap();
        let err = reload_settings(&mut plugin).unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
        assert_eq!(plugin.settings.value.gap, 6);
        std::fs::write(&path, "gap: 7\n").unwrap();
        plugin.fail = Some(7);
        assert!(reload_settings(&mut plugin).is_err());
        assert_eq!(plugin.settings.value.gap, 6, "a failed apply is taken back");
        assert_eq!(plugin.applied, 6, "and the old settings apply again");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// A broken file of 1 MiB made an error of 1 MiB, past the line limit
    /// of the core (W9).
    #[test]
    fn a_long_error_is_cut() {
        let long = RpcError::new(RpcError::INVALID_PARAMS, "ä".repeat(1 << 19));
        let message = capped(long).message;
        assert!(
            message.len() <= MAX_MESSAGE + '…'.len_utf8(),
            "{}",
            message.len()
        );
        assert!(message.ends_with('…'));
        assert_eq!(capped(RpcError::new(1, "short")).message, "short");
    }

    #[test]
    fn a_served_session_starts_a_new_episode_of_failures() {
        assert_eq!(next_failing(SERVED * 2, COME_BACK / 2), TRY_AGAIN);
    }

    #[test]
    fn an_episode_of_pure_failures_gives_up_after_come_back() {
        let mut failing = Duration::ZERO;
        for _ in 0..1000 {
            failing = next_failing(Duration::ZERO, failing);
            if failing >= COME_BACK {
                return;
            }
        }
        panic!("the plugin never gave up");
    }

    #[test]
    fn the_give_keys_are_grabbed_again_when_another_client_lets_them_go() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        use crate::conn::fake_core::{self, Answer};

        let released = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&released);
        let (path, _core) = fake_core::serve(move |method, params| match method {
            "input.grab" if params["binding"] == "Alt+Super+2" && !seen.load(Ordering::SeqCst) => {
                Answer::Error("Alt+Super+2 is grabbed already")
            }
            "input.grab" => Answer::Result(params["binding"].clone()),
            _ => Answer::Result(Value::Null),
        });
        let conn = Conn::connect_at("mc", &path).unwrap();
        let mut rt: Runtime<&'static str> = Runtime::with(conn, "mc", Vec::new());
        rt.give_keys(Some(|to| to));
        rt.grab_give_keys().unwrap();
        for (key, to) in [("Alt+Super+1", "tiling"), ("Alt+Super+3", "mc")] {
            assert_eq!(rt.action(key), Some(&to));
        }
        assert_eq!(rt.action("Alt+Super+4"), Some(&"construct"));
        assert_eq!(rt.action("Alt+Super+2"), None);
        released.store(true, Ordering::SeqCst);
        rt.grab_give_keys().unwrap();
        assert_eq!(rt.action("Alt+Super+2"), Some(&"xp"));
    }

    struct Recorder {
        rt: Runtime<()>,
        seen: Vec<Event>,
    }

    impl Plugin for Recorder {
        type Action = ();

        fn runtime(&mut self) -> &mut Runtime<()> {
            &mut self.rt
        }

        fn event(&mut self, event: Event) -> Result<(), ClientError> {
            self.seen.push(event);
            Ok(())
        }

        fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
            Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method))
        }
    }

    fn key(injected: bool) -> Event {
        Event::InputKey {
            binding: "Alt+Super+2".into(),
            state: chora_scene::ipc::PressState::Pressed,
            injected,
        }
    }

    fn recorder() -> Recorder {
        use crate::conn::fake_core::{self, Answer};

        let (path, _core) = fake_core::serve(|_, _| Answer::Result(Value::Null));
        let conn = Conn::connect_at("tiling", &path).unwrap();
        Recorder {
            rt: Runtime::with(conn, "tiling", Vec::new()),
            seen: Vec::new(),
        }
    }

    #[test]
    fn an_injected_key_runs_no_binding_unless_the_setting_is_on() {
        let mut plugin = recorder();
        dispatch(&mut plugin, key(true)).unwrap();
        assert_eq!(plugin.seen, []);
        dispatch(&mut plugin, key(false)).unwrap();
        assert_eq!(plugin.seen.len(), 1);
        plugin.rt.accept_injected(true);
        dispatch(&mut plugin, key(true)).unwrap();
        assert_eq!(plugin.seen.len(), 2);
        plugin.rt.accept_injected(false);
        dispatch(&mut plugin, key(true)).unwrap();
        assert_eq!(plugin.seen.len(), 2);
    }
}
