//! A fake core for the tests of a plugin: the scene model of the core
//! behind a socket, without a renderer and without layout.
#![allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "a fake core for tests: a failure is a failed test"
)]

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use chora_scene::{Command, ipc::RpcError};
use serde_json::{Value, json};

use crate::conn::Conn;

mod script;
#[cfg(test)]
mod temp;

/// A temporary folder for one test (stub: named by name and pid only).
pub struct TempDir(PathBuf);

impl TempDir {
    /// Makes the folder `name` in the temporary directory.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    /// The path of the folder.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

pub use script::{
    Script, Subject, created, created_by, created_child, detach, output, owner, phases, surface,
};

/// The scene model of the fake core, shared with the test.
pub type Model = Arc<Mutex<Core>>;

/// The lines that a recorder of the fake core wrote, one per request.
pub type Log = Arc<Mutex<Vec<String>>>;

/// Gives the answer of a request, or `None` for the answer of the fake core.
type Responder = Arc<dyn Fn(&str, &Value) -> Option<Value> + Send + Sync>;

/// Gives the line that a request leaves in a log, or `None` for no line.
type Recorder = (
    Arc<dyn Fn(&str, &Value) -> Option<String> + Send + Sync>,
    Log,
);

/// What the fake core keeps of a connection.
#[derive(Default)]
pub struct Core {
    /// The scene that the plugin built.
    pub scene: chora_scene::Scene,
    /// The focus commands that the plugin sent.
    pub focuses: Vec<Option<String>>,
    /// The methods that the plugin called through `methods.call`.
    pub calls: Vec<String>,
    /// How often the plugin asked `scene.query`.
    pub queries: usize,
    /// The parameters of each `methods.call`: client, method and params.
    pub called: Vec<Value>,
    /// The events that the plugin emitted through `events.emit`.
    pub emits: Vec<Value>,
    /// The methods whose next call fails, in the order that they were planted.
    pub refusals: Vec<String>,
    /// The bindings that the plugin let go with `input.ungrab`.
    pub ungrabs: Vec<Value>,
    /// What `methods.call` answers for a method, by its name: looked up
    /// by client first, then by method, else null.
    pub answers: std::collections::HashMap<String, Value>,
    /// Whether `activation.token` mints a token; without, it answers null.
    pub mints_tokens: bool,
    /// Whether `scene.query` answers every selector; else only `id:`.
    pub selects: bool,
    /// The answers of `methods.call` by client and method: looked up by
    /// client first, then by method, else null.
    pub client_answers: Vec<((String, String), Value)>,
    /// The bindings that another client holds: their `input.grab` fails.
    pub held: Vec<String>,
    /// The bindings that the plugin grabbed with `input.grab`.
    pub grabs: Vec<Value>,
    responders: Vec<Responder>,
    recorders: Vec<Recorder>,
}

/// A fake core on a socket of its own, with one model for the
/// connections to it.
pub struct FakeCore {
    path: PathBuf,
    model: Model,
}

impl FakeCore {
    /// Starts a fake core that answers on a new socket.
    ///
    /// # Panics
    ///
    /// The socket cannot be bound.
    #[must_use]
    pub fn start() -> Self {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "chora-fake-core-{}-{}.sock",
            std::process::id(),
            COUNT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        let model = Model::default();
        let shared = model.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let model = shared.clone();
                std::thread::spawn(move || serve(&stream, &model));
            }
        });
        Self { path, model }
    }

    /// Makes `scene.query` answer every selector (`kind:`, `owner:`, ...)
    /// from the model, as the core does; by default it answers `id:` only.
    #[must_use]
    pub fn selecting(self) -> Self {
        self.model.lock().unwrap().selects = true;
        self
    }

    /// Plants a refusal: the next call of `method` fails with
    /// `REJECTED`, as when the client of a method is gone. The call after
    /// it passes again.
    pub fn refuse_next(&self, method: &str) {
        self.model.lock().unwrap().refusals.push(method.to_owned());
    }

    /// Plants the answer that every call of `method` through
    /// `methods.call` gets, as when the client that serves it is there.
    pub fn answer_with(&self, method: &str, answer: Value) {
        (self.model.lock().unwrap().answers).insert(method.to_owned(), answer);
    }

    /// Plants the answer of the method `method` of the client `client`
    /// for `methods.call`, as the state of voice; null without.
    pub fn answer_call(&self, client: &str, method: &str, answer: Value) {
        self.model
            .lock()
            .unwrap()
            .client_answers
            .push(((client.to_owned(), method.to_owned()), answer));
    }

    /// Plants a binding that another client holds: the `input.grab` of it
    /// fails with `REJECTED`, every time.
    pub fn hold(&self, binding: &str) {
        self.model.lock().unwrap().held.push(binding.to_owned());
    }

    /// Plants a responder: it gets the method and the parameters of each
    /// request, and a `Some` is the result; with `None` the fake core
    /// answers as it does without it. The first responder that answers wins.
    pub fn respond(
        &self,
        responder: impl Fn(&str, &Value) -> Option<Value> + Send + Sync + 'static,
    ) {
        self.model
            .lock()
            .unwrap()
            .responders
            .push(Arc::new(responder));
    }

    /// Plants a recorder: `line` gets the method and the parameters of each
    /// request, and a `Some` is appended to the log that this returns.
    #[must_use]
    pub fn record(
        &self,
        line: impl Fn(&str, &Value) -> Option<String> + Send + Sync + 'static,
    ) -> Log {
        let log = Log::default();
        (self.model.lock().unwrap().recorders).push((Arc::new(line), log.clone()));
        log
    }

    /// Makes `activation.token` mint a new token on each call, as the
    /// core does for a plugin that holds an act.
    pub fn mint_tokens(&self) {
        self.model.lock().unwrap().mints_tokens = true;
    }

    /// The scene model, shared with the fake core.
    #[must_use]
    pub fn model(&self) -> Model {
        self.model.clone()
    }

    /// Connects a plugin named `name`.
    ///
    /// # Panics
    ///
    /// The connection fails.
    #[must_use]
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    pub fn connect(&self, name: &str) -> Conn {
        Conn::connect_at(name, &self.path).unwrap()
    }
}

impl Drop for FakeCore {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Answers the requests of one connection.
fn serve(stream: &UnixStream, model: &Model) {
    let mut writer = stream.try_clone().unwrap();
    let mut owner = String::new();
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap();
        if request["method"] == "hello" {
            request["params"]["name"]
                .as_str()
                .unwrap_or_default()
                .clone_into(&mut owner);
        }
        let answer = match answer(model, &owner, &request) {
            Ok(result) => json!({"jsonrpc": "2.0", "id": request["id"], "result": result}),
            Err((code, message)) => json!({"jsonrpc": "2.0", "id": request["id"],
                "error": {"code": code, "message": message}}),
        };
        writeln!(writer, "{answer}").unwrap();
    }
}

/// What a request meets first: the recorders see it, a planted refusal or a
/// held binding fails it. The responders are those that may answer it.
fn admit(model: &Model, method: &str, params: &Value) -> Result<Vec<Responder>, (i64, String)> {
    let mut core = model.lock().unwrap();
    for (line, log) in &core.recorders {
        if let Some(line) = line(method, params) {
            log.lock().unwrap().push(line);
        }
    }
    if let Some(at) = core.refusals.iter().position(|m| m == method) {
        core.refusals.remove(at);
        return Err((RpcError::REJECTED, format!("{method} is refused")));
    }
    if method == "input.grab" {
        let binding = params["binding"].as_str().unwrap_or_default();
        if core.held.iter().any(|held| held == binding) {
            return Err((RpcError::REJECTED, "held by another client".to_owned()));
        }
        core.grabs.push(params["binding"].clone());
    }
    Ok(core.responders.clone())
}

fn answer(model: &Model, owner: &str, request: &Value) -> Result<Value, (i64, String)> {
    let params = &request["params"];
    let method = request["method"].as_str().unwrap_or_default();
    let responders = admit(model, method, params)?;
    if method != "hello"
        && let Some(result) = responders.iter().find_map(|r| r(method, params))
    {
        return Ok(result);
    }
    match method {
        "hello" => Ok(json!({"version": chora_scene::ipc::VERSION})),
        "scene.apply" => {
            let commands: Vec<Command> =
                serde_json::from_value(params["commands"].clone()).unwrap();
            let mut core = model.lock().unwrap();
            core.focuses.extend(commands.iter().filter_map(|c| match c {
                Command::Focus { surface } => Some(surface.clone()),
                _ => None,
            }));
            match core.scene.apply(owner, &commands) {
                Ok(_) => Ok(Value::Null),
                Err(e) => Err((
                    RpcError::REJECTED,
                    format!("command {}: {}", e.index, e.error),
                )),
            }
        }
        "activation.token" if model.lock().unwrap().mints_tokens => Ok(activation_token()),
        "input.grab" => Ok(params["binding"].clone()),
        "input.ungrab" => {
            let mut core = model.lock().unwrap();
            core.ungrabs.push(params["binding"].clone());
            Ok(Value::Null)
        }
        "methods.call" => {
            let method = params["method"].as_str().unwrap_or_default().to_owned();
            let mut core = model.lock().unwrap();
            core.calls.push(method.clone());
            core.called.push(params.clone());
            let client = params["client"].as_str().unwrap_or_default();
            let by_client = (core.client_answers.iter())
                .find(|((c, m), _)| c == client && *m == method)
                .map(|(_, answer)| answer.clone());
            Ok(by_client
                .or_else(|| core.answers.get(&method).cloned())
                .unwrap_or(Value::Null))
        }
        "output.list" => Ok(json!([])),
        "events.emit" => {
            model.lock().unwrap().emits.push(params.clone());
            Ok(Value::Null)
        }
        "scene.query" => Ok(query(
            model,
            params["selector"].as_str().unwrap_or_default(),
        )),
        _ => Ok(Value::Null),
    }
}

/// A new activation token, as the core mints one for a plugin.
fn activation_token() -> Value {
    static TOKENS: AtomicUsize = AtomicUsize::new(0);
    let n = TOKENS.fetch_add(1, Ordering::Relaxed);
    json!({"token": format!("fake-token-{}-{n}", std::process::id())})
}

/// The nodes that a selector finds: every selector when the core selects,
/// else only `id:`.
fn query(model: &Model, selector: &str) -> Value {
    let mut core = model.lock().unwrap();
    core.queries += 1;
    let found: Vec<Value> = if core.selects {
        let selector = selector.parse::<chora_scene::Selector>().unwrap();
        (core.scene.query(&selector).into_iter())
            .filter_map(|n| serde_json::to_value(n).ok())
            .collect()
    } else {
        let id = selector.strip_prefix("id:");
        (id.and_then(|id| core.scene.node(id)))
            .and_then(|n| serde_json::to_value(n).ok())
            .into_iter()
            .collect()
    };
    Value::Array(found)
}

/// The `focus.changed` of the core that focuses `surface`, or nothing,
/// for the cells of a look (`kit::wm::focus` reads it).
#[must_use]
pub fn focus_changed(surface: Option<&str>) -> chora_scene::Event {
    chora_scene::Event::FocusChanged {
        surface: surface.map(String::from),
    }
}

/// Whether the model has a node.
pub fn has(model: &Model, id: &str) -> bool {
    model.lock().unwrap().scene.node(id).is_some()
}

/// The text of a text node of the model.
pub fn text(model: &Model, id: &str) -> Option<String> {
    model.lock().unwrap().scene.node(id)?.props.text.clone()
}
