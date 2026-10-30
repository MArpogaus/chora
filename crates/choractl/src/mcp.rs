//! `choractl mcp`: the API of chora as MCP tools over stdio (D34), on rmcp.
//!
//! The server reads JSON-RPC messages, one per line, from stdin and writes
//! its answers to stdout. rmcp answers requests in parallel and ends a
//! cancelled call through `notifications/cancelled`. The server holds one
//! connection to the core under its client name, `agent` by default, which
//! also names the grants that `capture`, `describe_screen` and
//! `inject_input` need. No tool grants anything (D40).

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use base64::{Engine, engine::general_purpose::STANDARD};
use chora_scene::{
    Node, NodeKind, OutputInfo, Transform,
    ipc::{
        Apply, AssetPut, Capture, Client, ClientError, Hello, Inject, MethodCall, OutputConfigure,
        Picture, Query, Reserve, RpcError, SOCKET_ENV,
    },
};
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ErrorCode,
        ErrorData, Implementation, JsonObject, ListToolsResult, PaginatedRequestParams,
        ServerCapabilities, ServerConfig, Tool,
    },
    service::{RequestContext, RoleServer},
    transport::io::stdio_pipes,
};
use serde_json::{Value, json};
use tokio::task::{block_in_place, spawn_blocking};

use crate::{
    events::{self, Subscription},
    schema, view,
};

/// The custom JSON-RPC code of a cancelled call, from the LSP.
const CANCELLED: ErrorCode = ErrorCode(-32800);
/// The longest single wait of `wait_events` between two looks at the
/// cancellation: a blocked `take` cannot be interrupted, so the wait runs
/// in slices, and the process ends within one slice after the last answer.
/// The slice stays well under the 100 ms of gate check 3 of 0.8, since the
/// wake, the log line and the pipe add to its end.
const SLICE: Duration = Duration::from_millis(20);
/// The longer side of the screenshot of `describe_screen` by default.
const DESCRIBE_SIDE: u32 = 1280;
/// The longer side of a picture of `capture` by default.
const CAPTURE_SIDE: u32 = 1920;
/// The instructions of `initialize`, so the model starts in the right order.
const INSTRUCTIONS: &str = "chora is a programmable Wayland compositor. Start with \
    describe_screen: it shows a screenshot, the windows and the plugins with \
    their methods. Ask plugins for changes with call_method, for example \
    tiling.set_layout, instead of moving their windows. Screenshots and input \
    need a grant that only the user gives with choractl grant.";

/// The connection to the core. After the core closed it, for example in
/// an emergency reset, the next call connects again.
pub struct Core {
    name: String,
    client: Option<Client>,
    /// The reserves of `reserve_space` by output, for a new connection.
    reserves: HashMap<String, Reserve>,
}

/// The methods that change nothing, so that a call goes again after a lost
/// connection.
const READ_ONLY: [&str; 6] = [
    "scene.query",
    "output.list",
    "clients.list",
    "grants.list",
    "methods.list",
    "screen.capture",
];

impl Core {
    /// Calls a method of the core. A lost connection connects again with
    /// the next call; a method that changes nothing goes again at once. A
    /// method with an effect does not, since it may have run.
    fn call<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, ClientError> {
        let params = serde_json::to_value(params)?;
        let tries = if READ_ONLY.contains(&method) { 2 } else { 1 };
        for _ in 0..tries {
            match self.connected()?.call(method, &params) {
                Err(ClientError::Closed | ClientError::Io(_)) => self.client = None,
                other => return other,
            }
        }
        Err(ClientError::Closed)
    }

    /// The connection. A new one reserves the space of the old one again,
    /// since a reserve ends with its connection.
    fn connected(&mut self) -> Result<&mut Client, ClientError> {
        if self.client.is_none() {
            let hello = Hello {
                name: self.name.clone(),
                keep_nodes: true,
                ..Hello::default()
            };
            let mut client = Client::from_env(&hello)?;
            // A reserve that the core refuses now, for example of an output
            // that is gone, goes; a lost connection fails the call.
            let mut refused = Vec::new();
            for (output, reserve) in &self.reserves {
                match client.call::<_, Value>("output.reserve", reserve) {
                    Err(ClientError::Rpc(_)) => refused.push(output.clone()),
                    other => drop(other?),
                }
            }
            for output in refused {
                self.reserves.remove(&output);
            }
            self.client = Some(client);
        }
        Ok(self.client.as_mut().expect("connected above"))
    }
}

/// The MCP server: the core connection and the subscription of
/// `wait_events`, each behind a lock, since rmcp runs the tools in
/// parallel. No lock is held across an await.
pub struct Mcp {
    name: String,
    /// The path of the core socket.
    socket: PathBuf,
    core: Mutex<Core>,
    /// The subscription of `wait_events`, on a connection of its own.
    subscription: Mutex<Option<Arc<Subscription>>>,
}

impl Mcp {
    fn locked(&self) -> MutexGuard<'_, Core> {
        self.core.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs one tool, or routes `client.method` to `methods.call`.
    fn dispatch(&self, name: &str, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
        if name == "subscribe_events" {
            // The subscription lives on a connection of its own.
            return self.subscribe(args);
        }
        let mut core = self.locked();
        match run_tool(&mut core, name, args) {
            Err(Failure::UnknownTool(unknown)) => match plugin_method(name) {
                Some((client, method)) => call_plugin(&mut core, client, method, args),
                None => Err(Failure::UnknownTool(unknown)),
            },
            other => other,
        }
    }

    /// The exported methods of all clients, for `list_tools`.
    fn exported(&self) -> Result<Vec<Value>, String> {
        self.locked()
            .call::<_, Vec<Value>>("methods.list", Value::Null)
            .map_err(|e| e.to_string())
    }

    /// Starts a subscription, which replaces the earlier one.
    fn subscribe(&self, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
        let strings = |key: &str| -> Result<Vec<String>, Failure> {
            match args.get(key) {
                None | Some(Value::Null) => Ok(Vec::new()),
                Some(v) => serde_json::from_value(v.clone())
                    .map_err(|e| Failure::Tool(format!("{key}: {e}"))),
            }
        };
        let events = strings("events")?;
        let keys = strings("keys")?
            .iter()
            .map(|k| {
                k.parse::<chora_scene::Binding>()
                    .map_err(|e| Failure::Tool(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        // The old subscription ends first, since its grabs block the new
        // ones; its events and keys bring it back if the new one fails.
        let old = self
            .subscription
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .map(|old| (old.events.clone(), old.keys.clone()));
        let started = Subscription::start(
            &self.socket,
            &self.name,
            events.clone(),
            keys,
            old.is_some(),
        );
        let (subscription, grabbed) = match started {
            Ok(started) => started,
            Err(e) => {
                let back = old.and_then(|(events, keys)| {
                    Subscription::start(&self.socket, &self.name, events, keys, true).ok()
                });
                *self
                    .subscription
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner) = back.map(|(s, _)| Arc::new(s));
                return Err(e.into());
            }
        };
        *self
            .subscription
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(subscription));
        let events = if events.is_empty() {
            json!("all")
        } else {
            json!(events)
        };
        Ok(vec![json_text(&json!({"events": events, "keys": grabbed}))])
    }

    /// The events of the subscription. rmcp cancels the context token on
    /// `notifications/cancelled`; the wait ends within one slice then, and
    /// also when the process shuts down.
    async fn wait(
        &self,
        args: &Value,
        context: &RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let current = self
            .subscription
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let Some(subscription) = current else {
            return Ok(tool_error("call subscribe_events first"));
        };
        let max = args["max"]
            .as_u64()
            .and_then(|m| usize::try_from(m).ok())
            .unwrap_or(50)
            .min(events::MAX_TAKE);
        let timeout = Duration::from_millis(args["timeout_ms"].as_u64().unwrap_or(5000))
            .min(events::MAX_WAIT);
        let deadline = tokio::time::Instant::now() + timeout;
        let mut collected: Vec<Value> = Vec::new();
        let mut dropped = 0;
        let mut ended = Value::Null;
        loop {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            let want = max - collected.len();
            // A `take` drains the buffer, so a `max` of 0 ends the wait
            // before it reaches one: it looks at nothing and keeps nothing.
            if want == 0 {
                break;
            }
            let filled = Arc::clone(&subscription);
            let pending = spawn_blocking(move || filled.take(want, left.min(SLICE)));
            let taken = tokio::select! {
                // First the finished `take`: it drains the buffer, so its
                // events must not go to a cancel that arrives at the same
                // instant as the last slice.
                biased;
                got = pending => got.map_err(|e| ErrorData::internal_error(e.to_string(), None))?,
                () = context.ct.cancelled() => {
                    // rmcp swallows the answer of a cancelled call, so the
                    // line on stderr is all the gate check 0.8-3 can see.
                    eprintln!("wait_events cancelled");
                    return Err(ErrorData::new(CANCELLED, "call cancelled", None));
                }
            };
            if let Some(list) = taken["events"].as_array() {
                collected.extend(list.iter().cloned());
            }
            dropped += taken["dropped"].as_u64().unwrap_or(0);
            if taken["ended"].is_string() {
                ended = taken["ended"].clone();
            }
            if !collected.is_empty() || ended.is_string() || left <= SLICE {
                break;
            }
        }
        let batch = json!({"events": collected, "dropped": dropped, "ended": ended});
        Ok(CallToolResult::success(vec![json_text(&batch)]).into())
    }
}

impl rmcp::ServerHandler for Mcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(INSTRUCTIONS)
            .with_server_info(Implementation::new("choractl", env!("CARGO_PKG_VERSION")))
    }

    /// The built-in tools plus one tool per exported plugin method. The
    /// list answers fresh on every call, so a later `methods.export` needs
    /// no restart and no list-changed notification.
    fn list_tools(
        &self,
        _page: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>>
    + rmcp::service::MaybeSendFuture
    + '_ {
        // Off the worker, as in `call_tool`: the core socket blocks.
        let exported = block_in_place(|| self.exported());
        if let Err(e) = &exported {
            // The built-ins stay, since a client that cached the list would
            // else show a dead server until it restarts.
            eprintln!("tools/list without the plugin tools: {e}");
        }
        let mut all = tools();
        all.extend(plugin_tools(&exported.unwrap_or_default()));
        std::future::ready(Ok(ListToolsResult::with_all_items(all)))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let args = request.arguments.map_or_else(|| json!({}), Value::Object);
        if request.name == "wait_events" {
            return self.wait(&args, &context).await;
        }
        // Off the worker: the core socket blocks, and a plugin method may
        // wait for seconds. The transport must still read the cancel.
        let answer = block_in_place(|| self.dispatch(&request.name, &args));
        Ok(match answer {
            Ok(blocks) => CallToolResult::success(blocks).into(),
            Err(Failure::UnknownTool(message)) => {
                return Err(ErrorData::invalid_params(message, None));
            }
            Err(Failure::Tool(message)) => tool_error(message),
        })
    }
}

/// Serves MCP over stdin and stdout until the client closes the pipe.
pub fn serve(name: String) -> Result<(), String> {
    let hello = Hello {
        name: name.clone(),
        keep_nodes: true,
        ..Hello::default()
    };
    let socket = PathBuf::from(std::env::var_os(SOCKET_ENV).ok_or("CHORA_SOCKET is not set")?);
    let client = Client::connect(&socket, &hello).map_err(|e| e.to_string())?;
    let server = Mcp {
        name: name.clone(),
        socket,
        core: Mutex::new(Core {
            name,
            client: Some(client),
            reserves: HashMap::new(),
        }),
        subscription: Mutex::new(None),
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async {
        let service = rmcp::serve_server(server, stdio_pipes())
            .await
            .map_err(|e| e.to_string())?;
        service.waiting().await.map(drop).map_err(|e| e.to_string())
    })
}

/// A tool: its name, description and the schema of its arguments.
fn tool(
    name: &'static str,
    description: &'static str,
    properties: &Value,
    required: &[&str],
) -> Tool {
    let schema = json!({
        "type": "object",
        "properties": properties,
        "required": required,
    });
    Tool::new(name, description, object(&schema))
}

/// A tool whose input schema comes from the type of its parameters, so that
/// the model sees every command and every field.
fn typed_tool<T: schemars::JsonSchema>(name: &'static str, description: &'static str) -> Tool {
    Tool::new(name, description, object(&schema::of::<T>()))
}

fn object(schema: &Value) -> JsonObject {
    schema.as_object().expect("a JSON object").clone()
}

/// One tool per exported plugin method: the name `client.method`, the
/// description and the input schema of the export. A plain export gets a
/// schema that accepts any object.
fn plugin_tools(methods: &[Value]) -> Vec<Tool> {
    methods
        .iter()
        .filter_map(|method| {
            let client = method["client"].as_str()?;
            let name = method["name"].as_str()?;
            let schema = match &method["input_schema"] {
                Value::Object(fields) => fields.clone(),
                _ => object(&json!({"type": "object"})),
            };
            let description = method["description"]
                .as_str()
                .map_or_else(|| format!("Call {client}.{name}."), str::to_owned);
            Some(Tool::new(format!("{client}.{name}"), description, schema))
        })
        .collect()
}

/// The tool name of an exported method: `tiling.set_layout` is the method
/// `set_layout` of the client `tiling`. None for a name with no dot or
/// with more than one.
fn plugin_method(name: &str) -> Option<(&str, &str)> {
    let (client, method) = name.split_once('.')?;
    (!client.is_empty() && !method.is_empty() && !method.contains('.')).then_some((client, method))
}

/// The built-in tools.
fn tools() -> Vec<Tool> {
    let mut tools = screen_tools();
    tools.extend(scene_tools());
    tools.extend(session_tools());
    tools.extend(view_tools());
    tools.extend(event_tools());
    tools
}

/// The tools of events.
fn event_tools() -> Vec<Tool> {
    vec![
        tool(
            "subscribe_events",
            "Start to collect events for wait_events, and grab keys and buttons, which then arrive as input.key events, or as input.button and input.moved events for a button such as Super+BTN_LEFT. Replaces the earlier subscription; if it fails, the earlier one stays. Events: surface.created, surface.changed, surface.destroyed, surface.requested, pointer.pressed, pointer.moved, pointer.released, pointer.scroll, pointer.entered, pointer.left, input.key, node.key, animation.done, output.changed and others.",
            &json!({
                "events": {"type": "array", "items": {"type": "string"}, "description": "event names; every event if empty or missing"},
                "keys": {"type": "array", "items": {"type": "string"}, "description": "bindings to grab, for example Super+F1 or Super+BTN_LEFT; they need Ctrl, Alt or Super"},
            }),
            &[],
        ),
        tool(
            "wait_events",
            "The events that arrived since the last call, or the first one that arrives within the timeout. dropped counts events that a full buffer lost. A notifications/cancelled ends the wait without an answer.",
            &json!({
                "timeout_ms": {"type": "integer", "description": "how long to wait for the first event; 5000 by default, at most 30000"},
                "max": {"type": "integer", "description": "the most events to return; 50 by default, at most 500, 0 takes nothing"},
            }),
            &[],
        ),
    ]
}

/// The tools that see the screen.
fn screen_tools() -> Vec<Tool> {
    let max_side =
        json!({"type": "integer", "description": "the longer side of the picture in pixels"});
    let output =
        json!({"type": "string", "description": "the output name; the first output by default"});
    vec![
        tool(
            "describe_screen",
            "A screenshot of an output, the windows with their positions in scene coordinates and sizes, the outputs, and the plugins with their methods. Needs a grant of screen.capture for the picture.",
            &json!({"output": output, "max_side": max_side}),
            &[],
        ),
        tool(
            "capture",
            "A picture of an output, or of a node with its descendants, as PNG. Needs a grant of screen.capture.",
            &json!({"output": output, "node": {"type": "string", "description": "a node id, for example a window"}, "max_side": max_side}),
            &[],
        ),
    ]
}

/// The tools of the scene, the plugins and the input.
fn scene_tools() -> Vec<Tool> {
    vec![
        tool(
            "query_scene",
            "The nodes that match a selector, for example kind:surface, app_id:firefox or title~:Inbox.",
            &json!({"selector": {"type": "string", "description": "the selector; every node if empty"}}),
            &[],
        ),
        typed_tool::<Apply>(
            "apply_commands",
            "Apply a transaction of scene commands: create, attach, update, animate, move, remove, configure, focus, focus_node, close, camera. All commands apply, or none. Warnings in the answer tell what applied but does not work as expected, for example a surface without input.",
        ),
        tool(
            "list_plugins",
            "The connected clients and the methods that each exported.",
            &json!({}),
            &[],
        ),
        tool(
            "call_method",
            "Call a method that a plugin exported, for example client tiling, method set_layout, params {\"layout\": \"fair\"}. Every export also has its own tool named client.method.",
            &json!({"client": {"type": "string"}, "method": {"type": "string"}, "params": {"description": "any JSON value"}}),
            &["client", "method"],
        ),
        typed_tool::<Inject>(
            "inject_input",
            "Send synthetic input: pointer_move, button, click, scroll, key, shortcut and text events, for example [{\"text\": {\"text\": \"ls\\n\"}}]. Needs a grant of input.inject.",
        ),
    ]
}

/// The tools of the session: programs, the work area and assets.
fn session_tools() -> Vec<Tool> {
    vec![
        tool(
            "run_program",
            "Start a program as choractl run does, on the machine of this MCP server. It runs in a systemd user unit of its own, or as its own child without a user manager. The program gets an activation token, so that its window gets the focus. The client name needs the grant activation.token on every socket. Returns the pid, the unit and the token, or the error when the start failed. The output of the program never comes back: it goes to the journal of the unit, or nowhere without a user manager.",
            &json!({
                "argv": {"type": "array", "items": {"type": "string"}, "minItems": 1, "description": "the program and its arguments, for example [\"foot\"]"},
            }),
            &["argv"],
        ),
        tool(
            "configure_output",
            "Set the scale or the transform of an output, as wlr-randr does. Returns the output.",
            &json!({
                "output": {"type": "string", "description": "the output name; the first output if missing"},
                "scale": {"type": "number", "minimum": 0.1, "maximum": 10},
                "transform": {"enum": ["normal", "90", "180", "270", "flipped", "flipped-90", "flipped-180", "flipped-270"]},
            }),
            &[],
        ),
        tool(
            "reserve_space",
            "Reserve space at the edges of an output, in logical pixels, for example for a panel of nodes: the work area, where window managers put windows, leaves it out until this server ends. A later call replaces the reserve on the output; no space removes it. Returns the output with its new work_area in scene coordinates.",
            &json!({
                "output": {"type": "string", "description": "the output name; the first output if missing"},
                "top": {"type": "integer", "minimum": 0},
                "right": {"type": "integer", "minimum": 0},
                "bottom": {"type": "integer", "minimum": 0},
                "left": {"type": "integer", "minimum": 0},
            }),
            &[],
        ),
        tool(
            "put_asset",
            "Store a file for image nodes, shader nodes and effects in a directory of chora, and get its absolute path, since chora may not see the files of the agent, for example in another container. Pass a shader as text and a PNG in base64, at most 512 KiB. A second put of a name replaces the file, and nodes that use it reload it.",
            &json!({
                "name": {"type": "string", "description": "the file name, for example glow.frag: letters, digits, '.', '_' and '-'"},
                "text": {"type": "string", "description": "the content as text, for a shader"},
                "base64": {"type": "string", "description": "the content in base64, for a PNG"},
            }),
            &["name"],
        ),
    ]
}

/// The tools of the views.
fn view_tools() -> Vec<Tool> {
    let name = json!({"type": "string", "description": "the name of the view"});
    vec![
        tool(
            "load_view",
            "Load a view: start and configure its plugins and apply its nodes.",
            &json!({"name": name}),
            &["name"],
        ),
        tool(
            "save_view",
            "Save the nodes of a view to its YAML file.",
            &json!({"name": name}),
            &["name"],
        ),
        tool(
            "unload_view",
            "Remove the nodes of a view.",
            &json!({"name": name}),
            &["name"],
        ),
    ]
}

/// Why a tool call failed.
enum Failure {
    /// An unknown tool: a JSON-RPC error.
    UnknownTool(String),
    /// The tool ran and failed.
    Tool(String),
}

impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self::Tool(message)
    }
}

impl From<ClientError> for Failure {
    fn from(err: ClientError) -> Self {
        Self::Tool(err.to_string())
    }
}

/// A tool that ran and failed: the model reads the reason and can correct
/// its call, as the protocol asks.
fn tool_error(message: impl Into<String>) -> CallToolResponse {
    CallToolResult::error(vec![ContentBlock::text(message.into())]).into()
}

fn json_text(value: &Value) -> ContentBlock {
    ContentBlock::text(serde_json::to_string_pretty(value).expect("values always serialize"))
}

fn image(picture: &Picture) -> ContentBlock {
    ContentBlock::image(picture.png.clone(), "image/png")
}

fn arg<T: serde::de::DeserializeOwned>(args: &Value) -> Result<T, Failure> {
    serde_json::from_value(args.clone())
        .map_err(|e| Failure::Tool(format!("invalid arguments: {e}")))
}

fn run_tool(core: &mut Core, name: &str, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
    let string = |key: &str| -> Result<String, Failure> {
        args[key]
            .as_str()
            .map(String::from)
            .ok_or_else(|| Failure::Tool(format!("{key} is missing")))
    };
    match name {
        "describe_screen" => describe(core, args),
        "capture" => {
            let mut capture: Capture = arg(args)?;
            capture.max_side = capture.max_side.or(Some(CAPTURE_SIDE));
            let picture: Picture = core.call("screen.capture", capture)?;
            Ok(vec![image(&picture), json_text(&picture_info(&picture))])
        }
        "query_scene" => {
            let selector = args["selector"].as_str().unwrap_or_default().to_string();
            let nodes: Value = core.call("scene.query", Query::new(selector))?;
            Ok(vec![json_text(&nodes)])
        }
        "apply_commands" => {
            let apply: Apply = arg(args)?;
            let answer: Value = core.call("scene.apply", apply)?;
            if answer["warnings"].as_array().is_some_and(|w| !w.is_empty()) {
                return Ok(vec![ContentBlock::text("applied"), json_text(&answer)]);
            }
            Ok(vec![ContentBlock::text("applied")])
        }
        "list_plugins" => {
            let clients: Value = core.call("clients.list", Value::Null)?;
            Ok(vec![json_text(&clients)])
        }
        "call_method" => {
            let call: MethodCall = arg(args)?;
            let result: Value = core.call("methods.call", call)?;
            Ok(vec![json_text(&result)])
        }
        "inject_input" => {
            let inject: Inject = arg(args)?;
            core.call::<_, Value>("input.inject", inject)?;
            Ok(vec![ContentBlock::text("injected")])
        }
        "load_view" => {
            // An agent loads views by name only, not any file by path.
            let name = string("name")?;
            view::check_name(&name)?;
            Ok(vec![json_text(&view::load(&name)?)])
        }
        "save_view" => Ok(vec![json_text(&view::save(&string("name")?, None)?)]),
        "unload_view" => Ok(vec![json_text(&view::unload(&string("name")?)?)]),
        "run_program" => run_program(core, args),
        "put_asset" => put_asset(core, args),
        "reserve_space" => reserve_space(core, args),
        "configure_output" => pass::<OutputConfigure>(core, "output.configure", args),
        other => Err(Failure::UnknownTool(format!("no tool {other:?}"))),
    }
}

/// `run_program`. A started program can do all the user can, also grant
/// itself, so the tool needs the grant on every socket.
fn run_program(core: &mut Core, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
    let grants: Value = core.call("grants.list", Value::Null)?;
    let held = grants[&core.name]
        .as_array()
        .is_some_and(|held| held.iter().any(|c| c == "activation.token"));
    if !held {
        return Err(Failure::Tool(format!(
            "run_program needs the grant activation.token for {}",
            core.name
        )));
    }
    let program: Vec<String> = arg(&args["argv"])?;
    let line = crate::run_program(core.connected()?, &program, None)?;
    Ok(vec![json_text(&line)])
}

/// Calls an exported plugin method, the body of its own tool.
fn call_plugin(
    core: &mut Core,
    client: &str,
    method: &str,
    args: &Value,
) -> Result<Vec<ContentBlock>, Failure> {
    let call = MethodCall {
        client: client.to_string(),
        method: method.to_string(),
        params: args.clone(),
    };
    let result: Value = core.call("methods.call", call)?;
    Ok(vec![json_text(&result)])
}

/// Checks the arguments of a tool as the parameters of a method, calls it
/// and answers with its result.
fn pass<P: serde::de::DeserializeOwned + serde::Serialize>(
    core: &mut Core,
    method: &str,
    args: &Value,
) -> Result<Vec<ContentBlock>, Failure> {
    let result: Value = core.call(method, arg::<P>(args)?)?;
    Ok(vec![json_text(&result)])
}

/// Reserves space at the edges of an output for the life of the server.
fn reserve_space(core: &mut Core, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
    let mut edges = if args.is_object() {
        args.clone()
    } else {
        json!({})
    };
    let output = edges.as_object_mut().and_then(|a| a.remove("output"));
    let output = match output {
        None | Some(Value::Null) => None,
        Some(Value::String(name)) => Some(name),
        Some(_) => return Err(Failure::Tool("output must be a name".into())),
    };
    let reserve = Reserve {
        output,
        edges: arg(&edges)?,
    };
    let output: Value = core.call("output.reserve", &reserve)?;
    // By the name of the output in the answer, so that the first output
    // and its name are one entry.
    let name = output["name"].as_str().unwrap_or_default().to_owned();
    if reserve.edges.is_empty() {
        core.reserves.remove(&name);
    } else {
        let reserve = Reserve {
            output: Some(name.clone()),
            ..reserve
        };
        core.reserves.insert(name, reserve);
    }
    Ok(vec![json_text(&output)])
}

/// Stores an asset from text or base64.
fn put_asset(core: &mut Core, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
    let name = args["name"].as_str().unwrap_or_default().to_string();
    let data = match (args["text"].as_str(), args["base64"].as_str()) {
        (Some(text), None) => STANDARD.encode(text),
        (None, Some(data)) => data.to_string(),
        _ => return Err(Failure::Tool("pass either text or base64".into())),
    };
    let asset: Value = core.call("assets.put", AssetPut { name, data })?;
    Ok(vec![json_text(&asset)])
}

fn picture_info(picture: &Picture) -> Value {
    let mut info = json!({
        "width": picture.width,
        "height": picture.height,
        "scale": picture.scale,
        "origin": picture.origin,
    });
    // The picture shows these nodes, but the screen does not.
    if !picture.left_out.is_empty() {
        info["left_out_of_the_screen"] = json!(picture.left_out);
        info["note"] = json!(
            "these layers did not fit into the pixel budget of the last frame: the user does not see them"
        );
    }
    info
}

/// A screenshot, if the grant allows it, and the scene in words.
fn describe(core: &mut Core, args: &Value) -> Result<Vec<ContentBlock>, Failure> {
    let capture = Capture {
        output: args["output"].as_str().map(String::from),
        node: None,
        max_side: Some(
            args["max_side"]
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .unwrap_or(DESCRIBE_SIDE),
        ),
    };
    let mut content = Vec::new();
    let mut description = describe_scene(core)?;
    match core.call::<_, Picture>("screen.capture", capture) {
        Ok(picture) => {
            content.push(image(&picture));
            description["picture"] = picture_info(&picture);
        }
        Err(chora_scene::ipc::ClientError::Rpc(RpcError { message, .. })) => {
            description["picture"] = json!(format!("no screenshot: {message}"));
        }
        Err(e) => return Err(e.into()),
    }
    content.push(json_text(&description));
    Ok(content)
}

/// The outputs, the windows and the plugins.
fn describe_scene(core: &mut Core) -> Result<Value, Failure> {
    let outputs: Vec<OutputInfo> = core.call("output.list", Value::Null)?;
    let clients: Value = core.call("clients.list", Value::Null)?;
    let nodes: Vec<Node> = core.call("scene.query", Query::new("*"))?;
    let by_id: HashMap<&str, &Node> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let windows: Vec<Value> = nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Surface)
        .map(|n| {
            let (x, y) = placement(n, &by_id).translation_2d();
            let surface = n.surface.as_ref();
            json!({
                "id": n.id,
                "app_id": surface.and_then(|s| s.app_id.clone()),
                "title": surface.and_then(|s| s.title.clone()),
                "owner": n.owner,
                "visible": shown(n, &by_id),
                "position": [x, y],
                "size": surface.and_then(|s| s.size),
            })
        })
        .collect();
    let outputs: Vec<Value> = outputs
        .iter()
        .map(|o| {
            json!({
                "name": o.name,
                "position": o.position,
                "size": o.size,
                "scale": o.scale,
                "work_area": o.work_area,
            })
        })
        .collect();
    Ok(json!({"outputs": outputs, "windows": windows, "plugins": clients}))
}

/// Whether a node is in the scene and it and all its ancestors are visible.
fn shown(node: &Node, by_id: &HashMap<&str, &Node>) -> bool {
    node.parent.is_some()
        && std::iter::successors(Some(node), |n| {
            n.parent.as_deref().and_then(|id| by_id.get(id).copied())
        })
        .all(|n| n.props.visible)
}

/// The transform of a node to scene coordinates, through its parents.
fn placement(node: &Node, by_id: &HashMap<&str, &Node>) -> Transform {
    let mut transform = node.props.transform;
    let mut parent = node.parent.as_deref();
    while let Some(p) = parent.and_then(|id| by_id.get(id)) {
        transform = p.props.transform.then(&transform);
        parent = p.parent.as_deref();
    }
    transform
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(tools: &[Tool]) -> Vec<&str> {
        tools.iter().map(|t| t.name.as_ref()).collect()
    }

    #[test]
    fn the_builtins_keep_their_names_and_take_no_grant() {
        let builtins = tools();
        let names = names(&builtins);
        assert_eq!(names.len(), 16, "{names:?}");
        assert!(names.contains(&"describe_screen"));
        assert!(names.iter().all(|n| !n.contains("grant")), "{names:?}");
    }

    #[test]
    fn every_exported_method_has_its_tool() {
        let methods = vec![
            json!({"client": "tiling", "name": "focus", "description": "Focus a window.",
                   "input_schema": {"type": "object",
                                    "properties": {"surface": {"type": "string"}}}}),
            json!({"client": "tiling", "name": "give", "description": null,
                   "input_schema": null}),
        ];
        let exported = plugin_tools(&methods);
        assert_eq!(names(&exported), ["tiling.focus", "tiling.give"]);
        assert_eq!(exported[0].description.as_deref(), Some("Focus a window."));
        assert_eq!(
            exported[0].input_schema["properties"]["surface"]["type"],
            "string"
        );
        assert_eq!(
            exported[1].description.as_deref(),
            Some("Call tiling.give.")
        );
        assert_eq!(exported[1].input_schema["type"], "object");
    }

    #[test]
    fn the_tool_list_of_the_core_goes_again_when_lost() {
        assert!(READ_ONLY.contains(&"methods.list"), "{READ_ONLY:?}");
    }

    #[test]
    fn the_wait_slice_stays_under_the_cancelled_second() {
        // Gate check 3 of 0.8 measures the end of a cancelled wait against
        // 100 ms, which one slice plus the wake must beat.
        assert!(SLICE * 2 <= Duration::from_millis(100), "{SLICE:?}");
    }

    #[test]
    fn a_window_in_a_hidden_group_is_not_visible() {
        let node = |id: &str, parent: Option<&str>, visible: bool| Node {
            id: id.into(),
            kind: NodeKind::Group,
            owner: None,
            parent: parent.map(String::from),
            children: Vec::new(),
            props: chora_scene::Props {
                visible,
                ..chora_scene::Props::default()
            },
            surface: None,
            to: None,
        };
        let nodes = [
            node("root", None, true),
            node("group", Some("root"), false),
            node("window", Some("group"), true),
            node("shown", Some("root"), true),
        ];
        let by_id: HashMap<&str, &Node> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        assert!(!shown(&nodes[2], &by_id));
        assert!(shown(&nodes[3], &by_id));
    }

    /// A core on a socket that grants every grab but `Super+k`.
    fn fake_core() -> PathBuf {
        let path = std::env::temp_dir().join(format!("choractl-test-{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let listener = std::os::unix::net::UnixListener::bind(&path).expect("bind");
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                std::thread::spawn(move || serve_fake(stream));
            }
        });
        path
    }

    fn serve_fake(stream: std::os::unix::net::UnixStream) {
        use std::io::{BufRead, BufReader, Write};
        let mut writer = stream.try_clone().expect("clone");
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            let request: Value = serde_json::from_str(&line).expect("json");
            let mut answer = if request["params"]["binding"] == "Super+k" {
                json!({"error": {"code": -32000, "message": "Super+k is taken"}})
            } else {
                json!({"result": request["params"]["binding"]})
            };
            answer["jsonrpc"] = json!("2.0");
            answer["id"] = request["id"].clone();
            let _ = writeln!(writer, "{answer}");
        }
    }

    #[test]
    fn a_failed_subscription_keeps_the_old_one() {
        let mcp = Mcp {
            name: "agent".into(),
            socket: fake_core(),
            core: Mutex::new(Core {
                name: "agent".into(),
                client: None,
                reserves: HashMap::new(),
            }),
            subscription: Mutex::new(None),
        };
        assert!(mcp.subscribe(&json!({"keys": ["Super+a"]})).is_ok());
        assert!(mcp.subscribe(&json!({"keys": ["Super+k"]})).is_err());
        assert!(mcp.subscription.lock().expect("lock").is_some());
        let _ = std::fs::remove_file(&mcp.socket);
    }

    #[test]
    fn the_layout_of_the_call_method_example_exists() {
        let call = tools()
            .into_iter()
            .find(|t| t.name == "call_method")
            .expect("call_method");
        let description = call.description.as_deref().unwrap_or_default();
        let layout = description
            .split("\"layout\": \"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("an example layout");
        let api = include_str!("../../../docs/api.md");
        let row = api
            .lines()
            .find(|l| l.starts_with("| `set_layout`"))
            .expect("the set_layout row");
        assert!(row.contains(&format!("`{layout}`")), "{layout}");
    }

    #[test]
    fn a_dotted_tool_name_routes_to_the_plugin() {
        assert_eq!(
            plugin_method("tiling.set_layout"),
            Some(("tiling", "set_layout"))
        );
        assert_eq!(plugin_method("describe_screen"), None);
        assert_eq!(plugin_method("a.b.c"), None);
        assert_eq!(plugin_method(".x"), None);
        assert_eq!(plugin_method("x."), None);
    }
}
