//! Command line client for the chora IPC socket.

use std::{
    io::{Read, Write},
    process::ExitCode,
};

use base64::{Engine, engine::general_purpose::STANDARD};

use chora_scene::{
    Command, Event,
    ipc::{
        Apply, Capability, Capture, Client, ClientError, GrantChange, Hello, MethodCall, Picture,
        Query, Subscribe,
    },
};
use clap::{Parser, Subcommand};
use serde_json::Value;

/// Talk to the chora compositor in `CHORA_SOCKET`.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The client name, the owner of the nodes that this call creates.
    #[arg(long, global = true, default_value = "choractl")]
    name: String,
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
    let commands = match &args.command {
        Cmd::Apply { file } => read_commands(file.as_deref())?,
        _ => Vec::new(),
    };
    let hello = Hello {
        name: args.name,
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
        } => {
            let params = parse_params(params.as_deref())?;
            let call = MethodCall {
                client: callee,
                method,
                params,
            };
            client.call("methods.call", call).map(|v| print(&v))
        }
        Cmd::Grant {
            client: name,
            capabilities,
        } => grants(&mut client, "grants.set", name, capabilities),
        Cmd::Revoke {
            client: name,
            capabilities,
        } => grants(&mut client, "grants.revoke", name, capabilities),
        Cmd::Grants => client.call("grants.list", Value::Null).map(|v| print(&v)),
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
            let picture: Picture = client
                .call("screen.capture", capture)
                .map_err(|e| e.to_string())?;
            return save(&picture, &file);
        }
    };
    result.map_err(|e| e.to_string())
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

/// Writes the PNG file of a picture, and prints the rest of it.
fn save(picture: &Picture, file: &str) -> Result<(), String> {
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

fn read_commands(file: Option<&str>) -> Result<Vec<Command>, String> {
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
