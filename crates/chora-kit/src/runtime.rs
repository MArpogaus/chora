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

/// Whether an event counts as the person's: it did not come from
/// `input.inject`, or the plugin takes injected input as the person's
/// (the setting `test_accept_injected_keys`). A window manager lets the
/// other events go on and ignores the rest.
#[must_use]
pub fn from_person(event: &Event, accepts_injected: bool) -> bool {
    let injected = match event {
        Event::InputKey { injected, .. }
        | Event::NodeKey { injected, .. }
        | Event::PointerPressed { injected, .. }
        | Event::PointerMoved { injected, .. }
        | Event::PointerReleased { injected, .. }
        | Event::PointerEntered { injected, .. }
        | Event::PointerScroll { injected, .. }
        | Event::InputButton { injected, .. }
        | Event::InputMoved { injected, .. } => *injected,
        _ => false,
    };
    accepts_injected || !injected
}

/// Whether a plugin acts on a button grab, a drag move, a hover or a
/// scroll of the pointer: those of the person, or of `input.inject` while
/// the plugin takes it as the person's. Every other event is true here: a
/// plugin judges a key or a press itself, with [`from_person`].
#[must_use]
pub fn hears_pointer(event: &Event, accepts_injected: bool) -> bool {
    match event {
        Event::InputButton { injected, .. }
        | Event::InputMoved { injected, .. }
        | Event::PointerEntered { injected, .. }
        | Event::PointerScroll { injected, .. } => accepts_injected || !injected,
        _ => true,
    }
}

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

    /// Called before `state.get` answers, so that a plugin brings up to
    /// date what only the core knows, such as which node has the keyboard
    /// (the core tells a look that listens to `focus.node` which node of
    /// another client takes it, and no other look);
    /// nothing by default.
    fn refresh(&mut self) {}

    /// The state behind `state.get`: what the plugin shows and holds, as
    /// the shape of `choractl state`; none by default.
    fn state(&self) -> Option<Value> {
        None
    }

    /// The settings as a caller of `settings.get`, `settings.set` and
    /// `settings.reload` sees them; all of them by default. A plugin
    /// hides its secrets here.
    fn shown_settings(&self, settings: Value) -> Value {
        settings
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
    /// The bindings of the set before [`Self::start_grabs`] that the
    /// core still holds for the plugin.
    old_grabs: Vec<String>,
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
    /// The client name of the caller of the method that runs now.
    pub caller: String,
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
            old_grabs: Vec::new(),
            refused: std::collections::HashSet::new(),
            give_keys: None,
            give_tried: None,
            inject_ok: test_session(),
            confined_call: false,
            caller: String::new(),
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

    /// Pins whether the plugin takes input of `input.inject` as the
    /// person's, whatever the environment says; for the unit tests of a
    /// plugin, which must not change with `CHORA_TEST_ACCEPT_INJECTED_KEYS`.
    pub fn pin_accepts_injected(&mut self, on: bool) {
        self.inject_ok = on;
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
        self.subscribe(events)?;
        self.conn
            .call::<_, Value>("methods.export", Export { names: methods })
            .map(drop)
    }

    /// Subscribes to `events`; the subscription replaces the earlier one.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn subscribe<E: AsRef<str>>(&mut self, events: &[E]) -> Result<(), ClientError> {
        self.conn
            .call::<_, Value>(
                "events.subscribe",
                Subscribe {
                    events: events.iter().map(|e| e.as_ref().into()).collect(),
                },
            )
            .map(drop)
    }

    /// Whether `--output` leaves an output to the plugin.
    #[must_use]
    pub fn manages(&self, output: &str) -> bool {
        self.outputs.is_empty() || self.outputs.iter().any(|o| o == output)
    }

    /// Leaves the plugin these outputs from now on, as `--output` does;
    /// all when empty. A window manager of the faces of a cube takes the
    /// screens when the cube goes, and the faces again when it comes back.
    pub fn set_outputs(&mut self, outputs: Vec<String>) {
        self.outputs = outputs;
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
        let mut outputs = self.all_outputs()?;
        outputs.retain(|o| self.manages(&o.name));
        Ok(outputs)
    }

    /// All outputs of the core, whatever `--output` names.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn all_outputs(&mut self) -> Result<Vec<OutputInfo>, ClientError> {
        self.conn.call("output.list", Value::Null)
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
        if let Some(i) = self.old_grabs.iter().position(|b| *b == text) {
            self.grabs.push((self.old_grabs.swap_remove(i), action));
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
        self.start_grabs();
        self.end_grabs()
    }

    /// Starts a new set of grabs, which [`Self::end_grabs`] completes. A
    /// [`Self::grab`] of a binding of the old set keeps it with the core,
    /// so that a binding of both sets is never free between them and no
    /// key of it reaches a window. The give keys stay as in
    /// [`Self::ungrab_all`].
    pub fn start_grabs(&mut self) {
        let keep = |normal: &str, serves: bool| {
            serves
                && crate::give::KEYS
                    .iter()
                    .filter_map(|(key, _)| key.parse::<Binding>().ok())
                    .any(|b| b.to_string() == normal)
        };
        let serves = self.give_keys.is_some();
        let (kept, old): (Vec<_>, Vec<_>) = std::mem::take(&mut self.grabs)
            .into_iter()
            .partition(|(normal, _)| keep(normal, serves));
        self.grabs = kept;
        self.old_grabs
            .extend(old.into_iter().map(|(normal, _)| normal));
    }

    /// Lets the bindings go that the plugin held at [`Self::start_grabs`]
    /// and did not grab since.
    ///
    /// # Errors
    ///
    /// As [`Self::ungrab_all`].
    pub fn end_grabs(&mut self) -> Result<(), ClientError> {
        for normal in std::mem::take(&mut self.old_grabs) {
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
            caller,
            method,
            params,
            confined,
            ..
        } => {
            rt.confined_call = confined;
            rt.caller = caller;
            rt.conn.serving_confined = confined;
            let result = answer_call(plugin, &method, params);
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

/// The answer to a method call: the methods of the runtime, then those of
/// the plugin.
fn answer_call<P: Plugin>(plugin: &mut P, method: &str, params: Value) -> Result<Value, RpcError> {
    match method {
        "settings.get" => (plugin.settings().map(|s| s.get()))
            .ok_or_else(|| no_settings("settings.get"))
            .map(|s| plugin.shown_settings(s)),
        "settings.set" => set_settings(plugin, params),
        "settings.reload" => reload_settings(plugin),
        "state.get" => {
            plugin.refresh();
            (plugin.state()).ok_or_else(|| RpcError::new(RpcError::METHOD_NOT_FOUND, "state.get"))
        }
        _ => plugin.method(method, params),
    }
}

/// The export of `state.get`, for a plugin that answers [`Plugin::state`].
#[must_use]
pub fn state_method() -> ExportMethod {
    method(
        "state.get",
        "Answers what the plugin shows and holds: windows, focus and what is open.",
        json!({}),
        &[],
    )
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
    Ok(plugin.shown_settings(answer))
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
    Ok(plugin.shown_settings(answer))
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

/// One line on stderr, `chora-NAME: ERR`, for a request that failed and
/// changes nothing else: the plugin `name` keeps running, the log says
/// what went wrong.
pub fn report(name: &str, err: &dyn std::fmt::Display) {
    eprintln!("chora-{name}: {err}");
}

/// Runs sessions of the plugin `name` until one ends without an error.
/// The core may close the connection and stay: the emergency reset
/// revokes grants and disconnects, and the scene survives it. Come back
/// in that case; give up when only failures follow within the episode.
pub fn run(
    name: &str,
    mut session: impl FnMut() -> Result<(), Box<dyn std::error::Error>>,
) -> ExitCode {
    crate::log::init();
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
    fn the_outputs_of_a_plugin_change_with_set_outputs() {
        use crate::conn::fake_core::{self, Answer};
        let (path, _core) = fake_core::serve(|_, _| Answer::Result(Value::Null));
        let conn = Conn::connect_at("set-outputs", &path).unwrap();
        let mut rt: Runtime<()> = Runtime::with(conn, "set-outputs", vec!["VIRTUAL-1".into()]);
        assert!(rt.manages("VIRTUAL-1") && !rt.manages("HEADLESS-1"));
        rt.set_outputs(vec!["HEADLESS-1".into()]);
        assert!(!rt.manages("VIRTUAL-1") && rt.manages("HEADLESS-1"));
        rt.set_outputs(Vec::new());
        assert!(rt.manages("VIRTUAL-1") && !rt.filters());
    }

    /// A plugin that answers `state.get`, and one that does not.
    struct Stated(Option<Value>);

    impl Plugin for Stated {
        type Action = ();

        fn runtime(&mut self) -> &mut Runtime<()> {
            unreachable!("answer_call needs no connection")
        }

        fn event(&mut self, _: Event) -> Result<(), ClientError> {
            Ok(())
        }

        fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
            Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method))
        }

        fn state(&self) -> Option<Value> {
            self.0.clone()
        }
    }

    /// `state.get` answers the state of the plugin; a plugin without one
    /// answers no such method.
    #[test]
    fn state_get_answers_the_state_of_the_plugin() {
        let shape = json!({"windows": [], "open": []});
        let mut with = Stated(Some(shape.clone()));
        assert_eq!(answer_call(&mut with, "state.get", Value::Null), Ok(shape));
        let mut without = Stated(None);
        let err = answer_call(&mut without, "state.get", Value::Null).unwrap_err();
        assert_eq!(err.code, RpcError::METHOD_NOT_FOUND);
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

        /// Shows no opacity, as a plugin hides its keys.
        fn shown_settings(&self, mut settings: Value) -> Value {
            settings["opacity"] = "(hidden)".into();
            settings
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
        assert_eq!(
            plugin.applied, 6,
            "a file that fails to parse applies nothing"
        );
        std::fs::write(&path, "gap: 7\n").unwrap();
        plugin.fail = Some(7);
        assert!(reload_settings(&mut plugin).is_err());
        assert_eq!(plugin.settings.value.gap, 6, "a failed apply is taken back");
        assert_eq!(plugin.applied, 6, "and the old settings apply again");
        std::fs::remove_dir_all(dir).unwrap();
    }

    /// Review plugins r1, VO-B-01: the answers of `settings.set` and
    /// `settings.reload` show the settings as the plugin shows them, as
    /// `settings.get` does.
    #[test]
    fn the_settings_answers_show_what_the_plugin_shows() {
        let dir = dir("runtime-shown");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        let settings = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        let mut plugin = Fake {
            settings,
            fail: None,
            applied: 0,
        };
        let set = set_settings(&mut plugin, json!({"gap": 8})).unwrap();
        assert_eq!(
            (&set["gap"], &set["opacity"]),
            (&json!(8), &json!("(hidden)"))
        );
        std::fs::write(&path, "gap: 6\n").unwrap();
        let reloaded = reload_settings(&mut plugin).unwrap();
        assert_eq!(reloaded["opacity"], "(hidden)", "{reloaded}");
        assert_eq!(plugin.applied, 6);
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

    #[test]
    fn a_binding_of_the_old_and_the_new_set_of_grabs_is_never_free() {
        use std::sync::{Arc, Mutex};

        use crate::conn::fake_core::{self, Answer};

        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&calls);
        let (path, _core) = fake_core::serve(move |method, params| {
            if method.starts_with("input.") {
                let binding = params["binding"].as_str().unwrap_or_default();
                seen.lock().unwrap().push(format!("{method} {binding}"));
            }
            match method {
                "input.grab" => Answer::Result(params["binding"].clone()),
                _ => Answer::Result(Value::Null),
            }
        });
        let conn = Conn::connect_at("construct", &path).unwrap();
        let mut rt: Runtime<u8> = Runtime::with(conn, "construct", Vec::new());
        let binding = |key: &str| key.parse::<Binding>().unwrap();
        rt.grab(binding("Alt+Escape"), 1).unwrap();
        rt.grab(binding("Super+t"), 1).unwrap();
        rt.start_grabs();
        rt.grab(binding("Alt+Escape"), 2).unwrap();
        rt.grab(binding("Super+x"), 2).unwrap();
        rt.end_grabs().unwrap();
        assert_eq!(
            *calls.lock().unwrap(),
            [
                "input.grab Alt+Escape",
                "input.grab Super+t",
                "input.grab Super+x",
                "input.ungrab Super+t",
            ]
        );
        assert_eq!(rt.action("Alt+Escape"), Some(&2));
        assert_eq!(rt.action("Super+x"), Some(&2));
        assert_eq!(rt.action("Super+t"), None);
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
        let mut rt = Runtime::with(conn, "tiling", Vec::new());
        rt.pin_accepts_injected(false);
        Recorder {
            rt,
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
        plugin.rt.pin_accepts_injected(true);
        dispatch(&mut plugin, key(true)).unwrap();
        assert_eq!(plugin.seen.len(), 2);
        plugin.rt.pin_accepts_injected(false);
        dispatch(&mut plugin, key(true)).unwrap();
        assert_eq!(plugin.seen.len(), 2);
    }

    #[test]
    fn from_person_knows_every_event_with_an_injected_flag() {
        let events = include_str!("../../chora-scene/src/event.rs");
        let source = include_str!("runtime.rs");
        let body = &source[source.find("pub fn from_person").unwrap()..];
        let body = &body[..body.find("\n}\n").unwrap()];
        let mut names = 0;
        let mut variant = "";
        for line in events.lines() {
            if let Some(name) = line.strip_prefix("    ").and_then(|l| l.strip_suffix(" {")) {
                variant = name;
            }
            if line.trim() == "injected: bool," {
                assert!(body.contains(&format!("Event::{variant} {{")), "{variant}");
                names += 1;
            }
        }
        assert_eq!(names, 9);
    }

    #[test]
    fn hears_pointer_knows_the_events_that_a_plugin_does_not_judge_itself() {
        let events = include_str!("../../chora-scene/src/event.rs");
        let source = include_str!("runtime.rs");
        let body = &source[source.find("pub fn hears_pointer").unwrap()..];
        let body = &body[..body.find("\n}\n").unwrap()];
        let judged_by_the_plugin = [
            "InputKey",
            "NodeKey",
            "PointerPressed",
            "PointerReleased",
            "PointerMoved",
        ];
        let mut variant = "";
        let mut flagged = 0;
        for line in events.lines() {
            if let Some(name) = line.strip_prefix("    ").and_then(|l| l.strip_suffix(" {")) {
                variant = name;
            }
            if line.trim() == "injected: bool," {
                let listed = body.contains(&format!("Event::{variant} {{"));
                assert_eq!(
                    listed,
                    !judged_by_the_plugin.contains(&variant),
                    "{variant}"
                );
                flagged += 1;
            }
        }
        assert_eq!(flagged, 9);
    }

    #[test]
    fn an_injected_pointer_event_is_not_heard_unless_the_plugin_accepts_them() {
        let events = [
            json!({"event": "pointer.entered", "node": "a", "injected": true}),
            json!({"event": "pointer.scroll", "node": "a", "position": [0, 0], "dx": 0, "dy": 1,
                   "discrete": [0, 0], "source": "wheel", "injected": true}),
        ];
        for event in events {
            let event: Event = serde_json::from_value(event.clone()).expect("the event");
            assert!(!hears_pointer(&event, false), "{event:?}");
            assert!(hears_pointer(&event, true), "{event:?}");
        }
        let person: Event =
            serde_json::from_value(json!({"event": "pointer.entered", "node": "a"})).unwrap();
        assert!(hears_pointer(&person, false));
    }

    #[test]
    fn from_person_takes_the_persons_events_and_the_injected_ones_only_when_accepted() {
        let injected = key(true);
        assert!(!from_person(&injected, false));
        assert!(from_person(&injected, true));
        assert!(from_person(&key(false), false));
        let other = Event::ClientConnected {
            name: "panel".into(),
        };
        assert!(from_person(&other, false), "an event without the flag");
    }

    #[test]
    fn hears_pointer_takes_a_key_as_judged_by_the_plugin() {
        assert!(hears_pointer(&key(true), false));
    }

    /// A plugin with settings that hides nothing and applies nothing new:
    /// the defaults of [`Plugin`] for the settings methods.
    struct Bare {
        settings: File<Demo>,
    }

    impl Plugin for Bare {
        type Action = ();

        fn runtime(&mut self) -> &mut Runtime<()> {
            unreachable!("answer_call needs no connection")
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
    }

    #[test]
    fn a_plugin_without_settings_or_state_answers_no_such_method() {
        let mut plugin = recorder();
        for method in ["settings.get", "state.get"] {
            let err = answer_call(&mut plugin, method, Value::Null).unwrap_err();
            assert_eq!(err.code, RpcError::METHOD_NOT_FOUND, "{method}");
        }
        let err = set_settings(&mut plugin, json!({})).unwrap_err();
        assert_eq!(err.code, RpcError::METHOD_NOT_FOUND);
        let err = reload_settings(&mut plugin).unwrap_err();
        assert_eq!(err.code, RpcError::METHOD_NOT_FOUND);
    }

    #[test]
    fn a_plugin_with_settings_shows_them_all_and_applies_them_by_default() {
        let dir = dir("runtime-bare");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("demo.yaml");
        let settings = File::<Demo>::load(path.clone(), DEFAULT).unwrap();
        let mut plugin = Bare { settings };
        let shown = answer_call(&mut plugin, "settings.get", Value::Null).unwrap();
        assert_eq!(shown, plugin.settings.get(), "nothing is hidden");
        let set = set_settings(&mut plugin, json!({"gap": 7})).unwrap();
        assert_eq!(set["gap"], 7);
        assert!(std::fs::read_to_string(&path).unwrap().contains("gap: 7"));
        std::fs::write(&path, "gap: 3\n").unwrap();
        let reloaded = answer_call(&mut plugin, "settings.reload", Value::Null).unwrap();
        assert_eq!(reloaded["gap"], 3);
        assert_eq!(plugin.settings.value.gap, 3);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn accept_injected_sets_the_setting_unless_it_is_a_test_session() {
        let mut plugin = recorder();
        plugin.rt.accept_injected(true);
        assert!(plugin.rt.accepts_injected());
        plugin.rt.accept_injected(false);
        assert_eq!(plugin.rt.accepts_injected(), test_session());
    }

    #[test]
    fn holds_names_the_bindings_that_the_plugin_grabbed() {
        use crate::conn::fake_core::{self, Answer};

        let (path, _core) = fake_core::serve(|_, params| Answer::Result(params["binding"].clone()));
        let conn = Conn::connect_at("tiling", &path).unwrap();
        let mut rt: Runtime<u8> = Runtime::with(conn, "tiling", Vec::new());
        rt.grab("Alt+Super+1".parse::<Binding>().unwrap(), 1)
            .unwrap();
        assert!(rt.holds("Alt+Super+1"));
        assert!(!rt.holds("Alt+Super+9"), "a binding not grabbed");
        assert!(
            !rt.holds("no key of this name"),
            "a text that is no binding"
        );
    }

    #[test]
    fn announce_subscribes_to_the_events_and_exports_the_methods() {
        use std::sync::{Arc, Mutex};

        use crate::conn::fake_core::{self, Answer};

        let sent = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&sent);
        let (path, _core) = fake_core::serve(move |method, params| {
            seen.lock()
                .unwrap()
                .push((method.to_owned(), params.clone()));
            Answer::Result(Value::Null)
        });
        let conn = Conn::connect_at("panel", &path).unwrap();
        let mut rt: Runtime<()> = Runtime::with(conn, "panel", Vec::new());
        rt.announce(&["output.removed"], vec![state_method()])
            .unwrap();
        let sent = sent.lock().unwrap();
        let subscribe = sent.iter().find(|(m, _)| m == "events.subscribe").unwrap();
        assert_eq!(subscribe.1["events"], json!(["output.removed"]));
        let export = sent.iter().find(|(m, _)| m == "methods.export").unwrap();
        assert_eq!(export.1["names"][0]["name"], "state.get");
    }

    #[test]
    fn regrab_tries_the_missing_give_keys_once_per_interval() {
        use std::sync::{Arc, Mutex};

        use crate::conn::fake_core::{self, Answer};

        let grabbed: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&grabbed);
        let (path, _core) = fake_core::serve(move |method, params| {
            if method != "input.grab" {
                return Answer::Result(Value::Null);
            }
            let binding = params["binding"].as_str().unwrap_or_default().to_owned();
            seen.lock().unwrap().push(binding.clone());
            if binding == "Alt+Super+2" {
                Answer::Error("held by another client")
            } else {
                Answer::Result(params["binding"].clone())
            }
        });
        let conn = Conn::connect_at("mc", &path).unwrap();
        let mut rt: Runtime<&'static str> = Runtime::with(conn, "mc", Vec::new());
        let asked = || grabbed.lock().unwrap().len();
        let held_by_other = || {
            grabbed
                .lock()
                .unwrap()
                .iter()
                .filter(|b| b.as_str() == "Alt+Super+2")
                .count()
        };
        rt.grab_give_keys().unwrap();
        assert_eq!(asked(), 0, "no give keys, nothing asked");
        rt.regrab_give_keys().unwrap();
        assert_eq!(asked(), 0, "no give keys, nothing asked");

        rt.give_keys(Some(|to| to));
        rt.regrab_give_keys().unwrap();
        let first = asked();
        assert!(first > 1, "every give key is asked: {first}");
        assert_eq!(held_by_other(), 1);
        rt.regrab_give_keys().unwrap();
        assert_eq!(asked(), first, "a second try within the interval waits");

        rt.give_tried = Some(Instant::now().checked_sub(REGRAB * 2).unwrap());
        rt.regrab_give_keys().unwrap();
        assert_eq!(asked(), first + 1, "only the key that is still missing");
        assert_eq!(held_by_other(), 2);
    }

    #[test]
    fn a_new_set_of_grabs_lets_go_the_others_but_keeps_the_give_keys() {
        use crate::conn::fake_core::{self, Answer};

        let (path, _core) = fake_core::serve(|_, params| Answer::Result(params["binding"].clone()));
        let conn = Conn::connect_at("mc", &path).unwrap();
        let mut rt: Runtime<&'static str> = Runtime::with(conn, "mc", Vec::new());
        rt.give_keys(Some(|to| to));
        rt.grab("Alt+Super+2".parse::<Binding>().unwrap(), "xp")
            .unwrap();
        rt.grab("Super+t".parse::<Binding>().unwrap(), "tiling")
            .unwrap();
        rt.start_grabs();
        rt.grab("Alt+Super+2".parse::<Binding>().unwrap(), "xp")
            .unwrap();
        rt.end_grabs().unwrap();
        assert!(rt.holds("Alt+Super+2"), "a give key stays with the plugin");
        assert!(!rt.holds("Super+t"), "another binding goes");
    }

    #[test]
    fn dispatch_answers_a_call_and_hands_on_only_the_events_of_managed_outputs() {
        let mut plugin = recorder();
        plugin.rt.set_outputs(vec!["DP-1".into()]);
        let gone: Event =
            serde_json::from_value(json!({"event": "output.removed", "name": "HDMI-1"})).unwrap();
        dispatch(&mut plugin, gone).unwrap();
        assert!(
            plugin.seen.is_empty(),
            "an output of another is left to others"
        );
        let kept: Event =
            serde_json::from_value(json!({"event": "output.removed", "name": "DP-1"})).unwrap();
        dispatch(&mut plugin, kept).unwrap();
        assert_eq!(plugin.seen.len(), 1);
        let call: Event = serde_json::from_value(json!({
            "event": "method.called", "call": 7, "caller": "panel", "method": "ping",
            "params": {}, "confined": true
        }))
        .unwrap();
        dispatch(&mut plugin, call).unwrap();
        assert_eq!(
            (plugin.rt.caller.as_str(), plugin.rt.confined_call),
            ("panel", true)
        );
        assert_eq!(plugin.seen.len(), 1, "a call is answered, not handed on");
    }

    #[test]
    fn a_method_names_its_schema_and_privileged_marks_it() {
        let described = method(
            "run.it",
            "Runs it.",
            json!({"n": {"type": "integer"}}),
            &["n"],
        );
        let ExportMethod::Described(d) = described.clone() else {
            panic!("a described method");
        };
        assert_eq!(d.input_schema.as_ref().unwrap()["required"], json!(["n"]));
        assert!(!d.privileged);
        let ExportMethod::Described(p) = privileged(described) else {
            panic!("a described method");
        };
        assert!(p.privileged && p.description.is_some());
        let ExportMethod::Described(q) = privileged(ExportMethod::Plain("run.it".into())) else {
            panic!("a described method");
        };
        assert!(q.privileged && q.name == "run.it" && q.description.is_none());
    }

    #[test]
    fn run_ends_at_once_when_a_session_ends_without_an_error() {
        let mut sessions = 0;
        let code = run("panel", || {
            sessions += 1;
            Ok(())
        });
        assert_eq!(code, ExitCode::SUCCESS);
        assert_eq!(sessions, 1);
    }
}
