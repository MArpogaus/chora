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
};

use chora_scene::{
    Node, NodeKind, OutputInfo, Transform,
    ipc::{Apply, Capture, Client, Hello, Inject, MethodCall, Picture, Query, RpcError},
};
use serde_json::{Value, json};

use crate::view;

/// The versions of the protocol that the server speaks, newest first.
const VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];
/// The longer side of the screenshot of `describe_screen` by default.
const DESCRIBE_SIDE: u32 = 1280;

/// Serves MCP until stdin ends.
pub fn serve(name: String) -> Result<(), String> {
    let hello = Hello {
        name,
        keep_nodes: true,
    };
    let mut client = Client::from_env(&hello).map_err(|e| e.to_string())?;
    let stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(answer) = handle(&mut client, &line) {
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

/// The answer to one message; none for a notification.
fn handle(client: &mut Client, line: &str) -> Option<Value> {
    let message: Value = match serde_json::from_str(line) {
        Ok(m) => m,
        Err(e) => return Some(error(&Value::Null, -32700, &format!("parse error: {e}"))),
    };
    let id = message.get("id").cloned()?;
    let method = message["method"].as_str().unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => Ok(initialize(&params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => Ok(call_tool(client, &params)),
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
    tools
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

/// The result of a tool call; a failure is a result with `isError`.
fn call_tool(client: &mut Client, params: &Value) -> Value {
    let name = params["name"].as_str().unwrap_or_default();
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match run_tool(client, name, &args) {
        Ok(content) => json!({"content": content, "isError": false}),
        Err(message) => json!({"content": [text(&message)], "isError": true}),
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

fn arg<T: serde::de::DeserializeOwned>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|e| format!("invalid arguments: {e}"))
}

fn rpc(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn run_tool(client: &mut Client, name: &str, args: &Value) -> Result<Vec<Value>, String> {
    let string = |key: &str| -> Result<String, String> {
        args[key]
            .as_str()
            .map(String::from)
            .ok_or_else(|| format!("{key} is missing"))
    };
    match name {
        "describe_screen" => describe(client, args),
        "capture" => {
            let capture: Capture = arg(args)?;
            let picture: Picture = client.call("screen.capture", capture).map_err(rpc)?;
            Ok(vec![image(&picture), json_text(&picture_info(&picture))])
        }
        "query_scene" => {
            let selector = args["selector"].as_str().unwrap_or_default().to_string();
            let nodes: Value = client
                .call("scene.query", Query { selector })
                .map_err(rpc)?;
            Ok(vec![json_text(&nodes)])
        }
        "apply_commands" => {
            let apply: Apply = arg(args)?;
            client.call::<_, Value>("scene.apply", apply).map_err(rpc)?;
            Ok(vec![text("applied")])
        }
        "list_plugins" => {
            let clients: Value = client.call("clients.list", Value::Null).map_err(rpc)?;
            Ok(vec![json_text(&clients)])
        }
        "call_method" => {
            let call: MethodCall = arg(args)?;
            let result: Value = client.call("methods.call", call).map_err(rpc)?;
            Ok(vec![json_text(&result)])
        }
        "inject_input" => {
            let inject: Inject = arg(args)?;
            client
                .call::<_, Value>("input.inject", inject)
                .map_err(rpc)?;
            Ok(vec![text("injected")])
        }
        "load_view" => Ok(vec![json_text(&view::load(&string("name")?)?)]),
        "save_view" => Ok(vec![json_text(&view::save(&string("name")?, None)?)]),
        "unload_view" => Ok(vec![json_text(&view::unload(&string("name")?)?)]),
        other => Err(format!("no tool {other:?}")),
    }
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
fn describe(client: &mut Client, args: &Value) -> Result<Vec<Value>, String> {
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
    let mut description = describe_scene(client)?;
    match client.call::<_, Picture>("screen.capture", capture) {
        Ok(picture) => {
            content.push(image(&picture));
            description["picture"] = picture_info(&picture);
        }
        Err(chora_scene::ipc::ClientError::Rpc(RpcError { message, .. })) => {
            description["picture"] = json!(format!("no screenshot: {message}"));
        }
        Err(e) => return Err(e.to_string()),
    }
    content.push(json_text(&description));
    Ok(content)
}

/// The outputs, the windows and the plugins.
fn describe_scene(client: &mut Client) -> Result<Value, String> {
    let outputs: Vec<OutputInfo> = client.call("output.list", Value::Null).map_err(rpc)?;
    let clients: Value = client.call("clients.list", Value::Null).map_err(rpc)?;
    let nodes: Vec<Node> = client
        .call(
            "scene.query",
            Query {
                selector: "*".into(),
            },
        )
        .map_err(rpc)?;
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
    fn no_tool_grants() {
        let names: Vec<_> = tools()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        assert!(names.iter().all(|n| !n.contains("grant")), "{names:?}");
        assert!(names.contains(&"describe_screen".to_string()));
    }
}
