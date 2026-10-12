//! `choractl mcp`: the API of chora as MCP tools over stdio (D34).
//!
//! The server reads JSON-RPC messages, one per line, from stdin and writes
//! its answers to stdout. It holds one connection to the core under its
//! client name, `agent` by default, which also names the grants that
//! `capture`, `describe_screen` and `inject_input` need. No tool grants
//! anything (D40).

use std::{
    collections::HashMap,
    io::{BufRead, Write},
    time::Duration,
};

use chora_scene::{
    Node, NodeKind, OutputInfo, Transform,
    ipc::{
        Apply, Capture, Client, ClientError, Hello, Inject, MethodCall, Picture, Query, RpcError,
    },
};
use serde_json::{Value, json};

use crate::{
    events::{self, Subscription},
    view,
};

/// The versions of the protocol that the server speaks, newest first.
const VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
/// The longer side of the screenshot of `describe_screen` by default.
const DESCRIBE_SIDE: u32 = 1280;
/// The longer side of a picture of `capture` by default.
const CAPTURE_SIDE: u32 = 1920;

/// The connection to the core. After the core closed it, for example in
/// an emergency reset, the next call connects again.
pub struct Core {
    name: String,
    client: Option<Client>,
    /// The events of `subscribe_events`, on a connection of their own.
    subscription: Option<Subscription>,
}

impl Core {
    /// Calls a method of the core, and connects again once when the
    /// connection is gone.
    fn call<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, ClientError> {
        let params = serde_json::to_value(params)?;
        for _ in 0..2 {
            let client = match &mut self.client {
                Some(client) => client,
                None => self.client.insert(Client::from_env(&Hello {
                    name: self.name.clone(),
                    keep_nodes: true,
                })?),
            };
            match client.call(method, &params) {
                Err(ClientError::Closed | ClientError::Io(_)) => self.client = None,
                other => return other,
            }
        }
        Err(ClientError::Closed)
    }
}

/// Serves MCP until stdin ends.
pub fn serve(name: String) -> Result<(), String> {
    let hello = Hello {
        name: name.clone(),
        keep_nodes: true,
    };
    let client = Client::from_env(&hello).map_err(|e| e.to_string())?;
    let mut chora = Core {
        name,
        client: Some(client),
        subscription: None,
    };
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(answer) = handle_line(&mut chora, &line) {
            let mut text = answer.to_string();
            text.push('\n');
            if stdout
                .write_all(text.as_bytes())
                .and_then(|()| stdout.flush())
                .is_err()
            {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// The answer to one line: a message, or a batch of them as protocol
/// version 2025-03-26 allows. None when nothing needs an answer.
fn handle_line(chora: &mut Core, line: &str) -> Option<Value> {
    match serde_json::from_str(line) {
        Ok(Value::Array(batch)) => {
            let answers: Vec<Value> = batch.iter().filter_map(|m| handle(chora, m)).collect();
            (!answers.is_empty()).then_some(Value::Array(answers))
        }
        Ok(message) => handle(chora, &message),
        Err(e) => Some(error(&Value::Null, -32700, &format!("parse error: {e}"))),
    }
}

/// The answer to one message; none for a notification or a response.
fn handle(chora: &mut Core, message: &Value) -> Option<Value> {
    let method = message.get("method")?.as_str().unwrap_or_default();
    let id = message.get("id").cloned()?;
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => Ok(initialize(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => call_tool(chora, &params),
        other => Err((-32601, format!("no method {other:?}"))),
    };
    Some(match result {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err((code, text)) => error(&id, code, &text),
    })
}

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn initialize(params: &Value) -> Value {
    let asked = params["protocolVersion"].as_str().unwrap_or_default();
    let version = VERSIONS
        .into_iter()
        .find(|v| *v == asked)
        .unwrap_or(VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {"name": "choractl", "version": env!("CARGO_PKG_VERSION")},
        "instructions": "chora is a programmable Wayland compositor. Start with \
            describe_screen: it shows a screenshot, the windows and the plugins with \
            their methods. Ask plugins for changes with call_method, for example \
            tiling.set_layout, instead of moving their windows. Screenshots and input \
            need a grant that only the user gives with choractl grant.",
    })
}

/// A tool: its name, description and the schema of its arguments.
fn tool(name: &str, description: &str, properties: &Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": properties, "required": required},
    })
}

fn tools() -> Vec<Value> {
    let mut tools = screen_tools();
    tools.extend(scene_tools());
    tools.extend(view_tools());
    tools.extend(event_tools());
    tools
}

/// The tools of events.
fn event_tools() -> Vec<Value> {
    vec![
        tool(
            "subscribe_events",
            "Start to collect events for wait_events, and grab keys, which then arrive as input.key events. Replaces the earlier subscription. Events: surface.created, surface.changed, surface.destroyed, surface.requested, pointer.pressed, pointer.moved, pointer.released, pointer.entered, pointer.left, input.key, animation.done, output.changed and others.",
            &json!({
                "events": {"type": "array", "items": {"type": "string"}, "description": "event names; every event if empty or missing"},
                "keys": {"type": "array", "items": {"type": "string"}, "description": "bindings to grab, for example Super+F1; they need Ctrl, Alt or Super"},
            }),
            &[],
        ),
        tool(
            "wait_events",
            "The events that arrived since the last call, or the first one that arrives within the timeout. dropped counts events that a full buffer lost.",
            &json!({
                "timeout_ms": {"type": "integer", "description": "how long to wait for the first event; 5000 by default, at most 30000"},
                "max": {"type": "integer", "description": "the most events to return; 50 by default, at most 500"},
            }),
            &[],
        ),
    ]
}

/// The tools that see the screen.
fn screen_tools() -> Vec<Value> {
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
fn scene_tools() -> Vec<Value> {
    vec![
        tool(
            "query_scene",
            "The nodes that match a selector, for example kind:surface, app_id:firefox or title~:Inbox.",
            &json!({"selector": {"type": "string", "description": "the selector; every node if empty"}}),
            &[],
        ),
        tool(
            "apply_commands",
            "Apply a transaction of scene commands: create, attach, update, animate, move, remove, configure, focus, close, camera.",
            &json!({"commands": {"type": "array", "items": {"type": "object"}}}),
            &["commands"],
        ),
        tool(
            "list_plugins",
            "The connected clients and the methods that each exported.",
            &json!({}),
            &[],
        ),
        tool(
            "call_method",
            "Call a method that a plugin exported, for example client tiling, method set_layout, params {\"layout\": \"rows\"}.",
            &json!({"client": {"type": "string"}, "method": {"type": "string"}, "params": {"description": "any JSON value"}}),
            &["client", "method"],
        ),
        tool(
            "inject_input",
            "Send synthetic input: pointer_move, button, click, scroll, key, shortcut and text events. Needs a grant of input.inject.",
            &json!({"events": {"type": "array", "items": {"type": "object"}, "description": "for example [{\"text\": {\"text\": \"ls\\n\"}}]"}}),
            &["events"],
        ),
    ]
}

/// The tools of the views.
fn view_tools() -> Vec<Value> {
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

/// The result of a tool call. A tool that fails, also on invalid
/// arguments, gives a result with `isError`, so that the model can correct
/// its call, as protocol version 2025-11-25 asks. An unknown tool gives a
/// JSON-RPC error.
fn call_tool(chora: &mut Core, params: &Value) -> Result<Value, (i64, String)> {
    let name = params["name"].as_str().unwrap_or_default();
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match run_tool(chora, name, &args) {
        Ok(content) => Ok(json!({"content": content, "isError": false})),
        Err(Failure::UnknownTool(message)) => Err((-32602, message)),
        Err(Failure::Tool(message)) => Ok(json!({"content": [text(&message)], "isError": true})),
    }
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

fn text(text: &str) -> Value {
    json!({"type": "text", "text": text})
}

fn json_text(value: &Value) -> Value {
    text(&serde_json::to_string_pretty(value).expect("values always serialize"))
}

fn image(picture: &Picture) -> Value {
    json!({"type": "image", "data": picture.png, "mimeType": "image/png"})
}

fn arg<T: serde::de::DeserializeOwned>(args: &Value) -> Result<T, Failure> {
    serde_json::from_value(args.clone())
        .map_err(|e| Failure::Tool(format!("invalid arguments: {e}")))
}

fn run_tool(chora: &mut Core, name: &str, args: &Value) -> Result<Vec<Value>, Failure> {
    let string = |key: &str| -> Result<String, Failure> {
        args[key]
            .as_str()
            .map(String::from)
            .ok_or_else(|| Failure::Tool(format!("{key} is missing")))
    };
    match name {
        "describe_screen" => describe(chora, args),
        "capture" => {
            let mut capture: Capture = arg(args)?;
            capture.max_side = capture.max_side.or(Some(CAPTURE_SIDE));
            let picture: Picture = chora.call("screen.capture", capture)?;
            Ok(vec![image(&picture), json_text(&picture_info(&picture))])
        }
        "query_scene" => {
            let selector = args["selector"].as_str().unwrap_or_default().to_string();
            let nodes: Value = chora.call("scene.query", Query { selector })?;
            Ok(vec![json_text(&nodes)])
        }
        "apply_commands" => {
            let apply: Apply = arg(args)?;
            chora.call::<_, Value>("scene.apply", apply)?;
            Ok(vec![text("applied")])
        }
        "list_plugins" => {
            let clients: Value = chora.call("clients.list", Value::Null)?;
            Ok(vec![json_text(&clients)])
        }
        "call_method" => {
            let call: MethodCall = arg(args)?;
            let result: Value = chora.call("methods.call", call)?;
            Ok(vec![json_text(&result)])
        }
        "inject_input" => {
            let inject: Inject = arg(args)?;
            chora.call::<_, Value>("input.inject", inject)?;
            Ok(vec![text("injected")])
        }
        "load_view" => {
            // An agent loads views by name only, not any file by path.
            let name = string("name")?;
            view::check_name(&name)?;
            Ok(vec![json_text(&view::load(&name)?)])
        }
        "save_view" => Ok(vec![json_text(&view::save(&string("name")?, None)?)]),
        "unload_view" => Ok(vec![json_text(&view::unload(&string("name")?)?)]),
        "subscribe_events" => subscribe(chora, args),
        "wait_events" => wait(chora, args),
        other => Err(Failure::UnknownTool(format!("no tool {other:?}"))),
    }
}

/// Takes the events of the subscription.
fn wait(chora: &Core, args: &Value) -> Result<Vec<Value>, Failure> {
    let subscription = chora
        .subscription
        .as_ref()
        .ok_or_else(|| Failure::Tool("call subscribe_events first".into()))?;
    let wait = args["timeout_ms"].as_u64().unwrap_or(5000);
    let wait = Duration::from_millis(wait).min(events::MAX_WAIT);
    let max = args["max"]
        .as_u64()
        .and_then(|m| usize::try_from(m).ok())
        .unwrap_or(50)
        .min(events::MAX_TAKE);
    Ok(vec![json_text(&subscription.take(max, wait))])
}

/// Starts a subscription, which replaces the earlier one.
fn subscribe(chora: &mut Core, args: &Value) -> Result<Vec<Value>, Failure> {
    let strings = |key: &str| -> Result<Vec<String>, Failure> {
        match args.get(key) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(v) => {
                serde_json::from_value(v.clone()).map_err(|e| Failure::Tool(format!("{key}: {e}")))
            }
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
    chora.subscription = None;
    let (subscription, grabbed) = Subscription::start(&chora.name, events.clone(), keys)?;
    chora.subscription = Some(subscription);
    let events = if events.is_empty() {
        json!("all")
    } else {
        json!(events)
    };
    Ok(vec![json_text(&json!({"events": events, "keys": grabbed}))])
}

fn picture_info(picture: &Picture) -> Value {
    json!({
        "width": picture.width,
        "height": picture.height,
        "scale": picture.scale,
        "origin": picture.origin,
    })
}

/// A screenshot, if the grant allows it, and the scene in words.
fn describe(chora: &mut Core, args: &Value) -> Result<Vec<Value>, Failure> {
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
    let mut description = describe_scene(chora)?;
    match chora.call::<_, Picture>("screen.capture", capture) {
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
fn describe_scene(chora: &mut Core) -> Result<Value, Failure> {
    let outputs: Vec<OutputInfo> = chora.call("output.list", Value::Null)?;
    let clients: Value = chora.call("clients.list", Value::Null)?;
    let nodes: Vec<Node> = chora.call(
        "scene.query",
        Query {
            selector: "*".into(),
        },
    )?;
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
                "visible": n.props.visible && n.parent.is_some(),
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

    #[test]
    fn initialize_speaks_the_version_of_the_client() {
        let result = initialize(&json!({"protocolVersion": "2025-03-26"}));
        assert_eq!(result["protocolVersion"], "2025-03-26");
        let result = initialize(&json!({"protocolVersion": "1999-01-01"}));
        assert_eq!(result["protocolVersion"], VERSIONS[0]);
    }

    #[test]
    fn only_requests_get_answers() {
        let mut chora = Core {
            name: "agent".into(),
            client: None,
            subscription: None,
        };
        let mut answer = |line: &str| handle_line(&mut chora, line);
        assert!(answer(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).is_none());
        assert!(
            answer(r#"{"jsonrpc":"2.0","id":7,"result":{}}"#).is_none(),
            "a response"
        );
        let unknown = answer(r#"{"jsonrpc":"2.0","id":1,"method":"nope"}"#).unwrap();
        assert_eq!(unknown["error"]["code"], -32601);
        let batch = answer(
            r#"[{"jsonrpc":"2.0","id":2,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        )
        .unwrap();
        assert_eq!(batch, json!([{"jsonrpc": "2.0", "id": 2, "result": {}}]));
        let tools = answer(r#"{"jsonrpc":"2.0","id":3,"method":"tools/list"}"#).unwrap();
        assert!(
            tools["result"]["tools"]
                .as_array()
                .is_some_and(|t| t.len() == 12)
        );
        let bad =
            answer(r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"nope"}}"#)
                .unwrap();
        assert_eq!(bad["error"]["code"], -32602);
        let wrong = answer(
            r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"apply_commands","arguments":{}}}"#,
        )
        .unwrap();
        assert_eq!(wrong["result"]["isError"], true, "{wrong}");
    }

    #[test]
    fn no_tool_grants() {
        let names: Vec<_> = tools()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        assert!(names.iter().all(|n| !n.contains("grant")), "{names:?}");
        assert!(names.contains(&"describe_screen".to_string()));
    }
}
