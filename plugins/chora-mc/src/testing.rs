//! A fake core for the tests of the plugin: the scene model of the core
//! behind a socket, without a renderer and without layout.

use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixListener,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use chora_kit::{conn::Conn, runtime::Runtime, settings};
use chora_scene::{Command, OutputInfo, Transform, ipc::RpcError};
use serde_json::{Value, json};

use crate::{Mc, NAME, config};

/// The scene model of the fake core, shared with the test.
pub type Model = Arc<Mutex<Core>>;

/// What the fake core keeps of a connection.
#[derive(Default)]
pub struct Core {
    pub scene: chora_scene::Scene,
    /// The focus commands that the plugin sent.
    pub focuses: Vec<Option<String>>,
}

/// The model of each connection, in the order of the connections.
static MODELS: Mutex<Vec<Model>> = Mutex::new(Vec::new());

/// Starts the fake core once; answers its socket.
fn socket() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("chora-mc-core-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let model = Model::default();
                MODELS.lock().unwrap().push(model.clone());
                std::thread::spawn(move || serve(&stream, &model));
            }
        });
        path
    })
}

/// Answers the requests of one connection.
fn serve(stream: &std::os::unix::net::UnixStream, model: &Model) {
    let mut writer = stream.try_clone().unwrap();
    for line in BufReader::new(stream).lines().map_while(Result::ok) {
        let request: Value = serde_json::from_str(&line).unwrap();
        let answer = match answer(model, &request) {
            Ok(result) => json!({"jsonrpc": "2.0", "id": request["id"], "result": result}),
            Err((code, message)) => json!({"jsonrpc": "2.0", "id": request["id"],
                "error": {"code": code, "message": message}}),
        };
        writeln!(writer, "{answer}").unwrap();
    }
}

fn answer(model: &Model, request: &Value) -> Result<Value, (i64, String)> {
    let params = &request["params"];
    match request["method"].as_str().unwrap_or_default() {
        "hello" => Ok(json!({"version": chora_scene::ipc::VERSION})),
        "scene.apply" => {
            let commands: Vec<Command> =
                serde_json::from_value(params["commands"].clone()).unwrap();
            let mut core = model.lock().unwrap();
            core.focuses.extend(commands.iter().filter_map(|c| match c {
                Command::Focus { surface } => Some(surface.clone()),
                _ => None,
            }));
            match core.scene.apply(NAME, &commands) {
                Ok(_) => Ok(Value::Null),
                Err(e) => Err((
                    RpcError::REJECTED,
                    format!("command {}: {}", e.index, e.error),
                )),
            }
        }
        "scene.query" => {
            let id = params["selector"]
                .as_str()
                .and_then(|s| s.strip_prefix("id:"));
            let core = model.lock().unwrap();
            let found = id
                .and_then(|id| core.scene.node(id))
                .map(|n| json!({"id": n.id}));
            Ok(found.into_iter().collect())
        }
        _ => Ok(Value::Null),
    }
}

/// A plugin on the fake core with the default settings, no scene files,
/// one output of `size` and its nodes built; and the model.
pub fn mc(dir: &str, size: [i32; 2]) -> (Mc, Model) {
    // One connection at a time, so that the last model is this one.
    static CONNECT: Mutex<()> = Mutex::new(());
    let one = CONNECT.lock().unwrap();
    let conn = Conn::connect_at(NAME, socket()).unwrap();
    let rt = Runtime::with(conn, NAME, Vec::new());
    let model = MODELS.lock().unwrap().last().unwrap().clone();
    drop(one);
    let dir = std::env::temp_dir().join(format!("chora-mc-{dir}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let file = dir.join("chora-mc.yaml");
    let settings = settings::File::load(file, config::DEFAULT).unwrap();
    let mut mc = Mc::new(
        rt,
        settings,
        dir.join("scenes"),
        [PathBuf::new(), PathBuf::new()],
    );
    mc.output = Some(OutputInfo {
        name: "test".into(),
        position: [0, 0],
        size,
        scale: 1.0,
        camera: Transform::IDENTITY,
        work_area: [0, 0, size[0], size[1]],
        default_work_area: [0; 4],
    });
    mc.build().unwrap();
    (mc, model)
}

/// Whether the model has a node.
pub fn has(model: &Model, id: &str) -> bool {
    model.lock().unwrap().scene.node(id).is_some()
}

/// The text of a text node of the model.
pub fn text(model: &Model, id: &str) -> Option<String> {
    model.lock().unwrap().scene.node(id)?.props.text.clone()
}
