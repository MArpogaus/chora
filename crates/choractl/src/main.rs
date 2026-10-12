//! Command line client for the chora IPC socket.

#[cfg(test)]
mod docs;
mod mcp;
mod view;

use std::{
    io::{Read, Write},
    process::ExitCode,
};

use base64::{Engine, engine::general_purpose::STANDARD};

use chora_scene::{
    Event,
    ipc::{
        Apply, Capability, Capture, Client, ClientError, GrantChange, Hello, Inject, MethodCall,
        Picture, Query, Subscribe,
    },
};
use clap::{Parser, Subcommand};
use serde_json::Value;

/// Talk to the chora compositor in `CHORA_SOCKET`.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The client name, the owner of the nodes that this call creates and
    /// the holder of grants: `choractl` by default, `agent` for `mcp`.
    #[arg(long, global = true)]
    name: Option<String>,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the nodes that match a selector, for example `kind:surface`.
    Query {
        /// The selector.
        #[arg(default_value = "*")]
        selector: String,
    },
    /// Apply a transaction: a JSON list of commands from a file or stdin.
    Apply {
        /// The file with the commands; stdin if missing or `-`.
        file: Option<String>,
    },
    /// Print events as JSON lines until the core closes the socket.
    Events {
        /// Event names, for example `surface.created`; every event if none.
        names: Vec<String>,
    },
    /// Print the outputs.
    Outputs,
    /// Call a method that a plugin exported, for example
    /// `choractl call tiling set_layout '{"layout": "columns"}'`.
    Call {
        /// The client that exported the method.
        client: String,
        /// The method name.
        method: String,
        /// The parameters as JSON; `null` if missing.
        params: Option<String>,
    },
    /// Grant a client name `input.inject` or `screen.capture`, until the
    /// session ends or a revoke.
    Grant {
        /// The client name, for example `agent`.
        client: String,
        /// `input.inject` or `screen.capture`.
        #[arg(required = true)]
        capabilities: Vec<Capability>,
    },
    /// Revoke grants of a client name; all of them without a capability.
    Revoke {
        /// The client name.
        client: String,
        /// `input.inject` or `screen.capture`.
        capabilities: Vec<Capability>,
    },
    /// Print the grants by client name.
    Grants,
    /// Send synthetic input: a JSON list of events from a file or stdin, for
    /// example `[{"text": {"text": "ls\n"}}]`. Needs a grant of
    /// `input.inject` for the client name.
    Inject {
        /// The file with the events; stdin if missing or `-`.
        file: Option<String>,
    },
    /// Serve the API as MCP tools over stdio, for an agent.
    Mcp,
    /// Load, unload or save a view: plugins and nodes from a YAML file.
    View {
        #[command(subcommand)]
        action: ViewCmd,
    },
    /// Save a picture of an output or of a node as a PNG file, and print
    /// its size, scale and origin. Needs a grant of `screen.capture` for
    /// the client name.
    Capture {
        /// The PNG file; stdout for `-`.
        file: String,
        /// The output; the first one if neither this nor a node is set.
        #[arg(long, conflicts_with = "node")]
        output: Option<String>,
        /// A node, for example a surface or a group, with its descendants.
        #[arg(long)]
        node: Option<String>,
        /// Shrink the picture until its longer side has at most this many
        /// pixels.
        #[arg(long)]
        max_side: Option<u32>,
    },
}

#[derive(Subcommand)]
enum ViewCmd {
    /// Start the plugins of a view that do not run, configure them and
    /// apply its nodes, which replace the nodes of an earlier load.
    Load {
        /// A view name, for `~/.config/chora/views/NAME.yaml` or
        /// `/usr/share/chora/views/NAME.yaml`, or the path of a YAML file.
        name: String,
    },
    /// Remove the nodes of a view.
    Unload {
        /// The name of the view.
        name: String,
    },
    /// Write the nodes of a view as they are now, with the plugins of its
    /// file.
    Save {
        /// The name of the view.
        name: String,
        /// The file; `~/.config/chora/views/NAME.yaml` if missing.
        #[arg(long)]
        file: Option<String>,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("choractl: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    match &args.command {
        Cmd::View { action } => return view_command(action),
        Cmd::Mcp => return mcp::serve(args.name.unwrap_or_else(|| "agent".into())),
        _ => {}
    }
    let (commands, input) = match &args.command {
        Cmd::Apply { file } => (read_json(file.as_deref())?, Vec::new()),
        Cmd::Inject { file } => (Vec::new(), read_json(file.as_deref())?),
        _ => (Vec::new(), Vec::new()),
    };
    let hello = Hello {
        name: args.name.unwrap_or_else(|| "choractl".into()),
        keep_nodes: true,
    };
    let mut client = Client::from_env(&hello).map_err(|e| e.to_string())?;
    let result = match args.command {
        Cmd::Query { selector } => client
            .call("scene.query", Query { selector })
            .map(|v| print(&v)),
        Cmd::Apply { .. } => client
            .call::<_, Value>("scene.apply", Apply { commands })
            .map(drop),
        Cmd::Events { names } => events(&mut client, names),
        Cmd::Outputs => client.call("output.list", Value::Null).map(|v| print(&v)),
        Cmd::Call {
            client: callee,
            method,
            params,
        } => return call(&mut client, callee, method, params.as_deref()),
        Cmd::Grant {
            client: name,
            capabilities,
        } => grants(&mut client, "grants.set", name, capabilities),
        Cmd::Revoke {
            client: name,
            capabilities,
        } => grants(&mut client, "grants.revoke", name, capabilities),
        Cmd::Grants => client.call("grants.list", Value::Null).map(|v| print(&v)),
        Cmd::View { .. } | Cmd::Mcp => unreachable!("handled before the connection"),
        Cmd::Inject { .. } => client
            .call::<_, Value>("input.inject", Inject { events: input })
            .map(drop),
        Cmd::Capture {
            file,
            output,
            node,
            max_side,
        } => {
            let capture = Capture {
                output,
                node,
                max_side,
            };
            return capture_to(&mut client, &capture, &file);
        }
    };
    result.map_err(|e| e.to_string())
}

/// Runs a view command with the client of the view.
fn view_command(action: &ViewCmd) -> Result<(), String> {
    let result = match action {
        ViewCmd::Load { name } => view::load(name),
        ViewCmd::Unload { name } => view::unload(name),
        ViewCmd::Save { name, file } => view::save(name, file.as_deref()),
    }?;
    print(&result);
    Ok(())
}

/// Prints events until the core closes the socket or stdout closes.
fn events(client: &mut Client, names: Vec<String>) -> Result<(), ClientError> {
    client.call::<_, Value>("events.subscribe", Subscribe { events: names })?;
    let mut stdout = std::io::stdout().lock();
    loop {
        let event: Event = match client.next_event() {
            Err(ClientError::Closed) => return Ok(()),
            other => other?,
        };
        let line = serde_json::to_string(&event)?;
        if writeln!(stdout, "{line}")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            return Ok(());
        }
    }
}

fn grants(
    client: &mut Client,
    method: &str,
    name: String,
    capabilities: Vec<Capability>,
) -> Result<(), ClientError> {
    let change = GrantChange {
        client: name,
        capabilities,
    };
    client.call::<_, Value>(method, change).map(drop)
}

/// Calls a method of a plugin and prints the result.
fn call(
    client: &mut Client,
    callee: String,
    method: String,
    params: Option<&str>,
) -> Result<(), String> {
    let call = MethodCall {
        client: callee,
        method,
        params: parse_params(params)?,
    };
    let result: Value = client
        .call("methods.call", call)
        .map_err(|e| e.to_string())?;
    print(&result);
    Ok(())
}

/// Writes the PNG file of a capture, and prints the rest of the picture.
fn capture_to(client: &mut Client, capture: &Capture, file: &str) -> Result<(), String> {
    let picture: Picture = client
        .call("screen.capture", capture)
        .map_err(|e| e.to_string())?;
    let png = STANDARD
        .decode(&picture.png)
        .map_err(|e| format!("the picture is no base64: {e}"))?;
    let written = if file == "-" {
        std::io::stdout().write_all(&png)
    } else {
        std::fs::write(file, &png)
    };
    written.map_err(|e| format!("{file}: {e}"))?;
    if file != "-" {
        print(&serde_json::json!({
            "width": picture.width,
            "height": picture.height,
            "scale": picture.scale,
            "origin": picture.origin,
        }));
    }
    Ok(())
}

fn parse_params(text: Option<&str>) -> Result<Value, String> {
    text.map_or(Ok(Value::Null), |t| {
        serde_json::from_str(t).map_err(|e| format!("the parameters are no JSON: {e}"))
    })
}

/// Reads a JSON value from a file, or from stdin for `None` and `-`.
fn read_json<T: serde::de::DeserializeOwned>(file: Option<&str>) -> Result<T, String> {
    let (name, text) = match file {
        None | Some("-") => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|e| format!("stdin: {e}"))?;
            ("stdin", text)
        }
        Some(path) => (
            path,
            std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?,
        ),
    };
    serde_json::from_str(&text).map_err(|e| format!("{name}: {e}"))
}

fn print(value: &Value) {
    let text = serde_json::to_string_pretty(value).expect("values always serialize");
    // A closed stdout, for example after `| head`, is no error.
    let _ = writeln!(std::io::stdout(), "{text}");
}
