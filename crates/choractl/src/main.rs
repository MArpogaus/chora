//! Command line client for the chora IPC socket.

#[cfg(test)]
mod docs;
mod events;
mod mcp;
mod schema;
mod view;

use std::{
    io::{Read, Write},
    process::ExitCode,
};

use base64::{Engine, engine::general_purpose::STANDARD};

use chora_scene::{
    Event,
    ipc::{
        Apply, Asset, AssetPut, AssetRemove, Capability, Capture, Client, ClientError, Edges,
        GrantChange, Hello, Inject, MethodCall, Picture, Query, Reserve, Restart, Spawn, Subscribe,
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
    /// Reserve space at the edges of an output, in logical pixels: the work
    /// area leaves it out until this command ends. Prints the output, then
    /// waits.
    Reserve(ReserveArgs),
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
    /// Grant a client name `input.inject`, `screen.capture` or
    /// `process.spawn`, until the
    /// session ends or a revoke.
    Grant {
        /// The client name, for example `agent`.
        client: String,
        /// `input.inject`, `screen.capture` or `process.spawn`.
        #[arg(required = true)]
        capabilities: Vec<Capability>,
    },
    /// Revoke grants of a client name; all of them without a capability.
    Revoke {
        /// The client name.
        client: String,
        /// `input.inject`, `screen.capture` or `process.spawn`.
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
    /// Start a program with the environment of the compositor, and print
    /// its pid. Needs a grant of `process.spawn` for the client name.
    Spawn {
        /// The program and its arguments.
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        argv: Vec<String>,
    },
    /// Restart a plugin: stop the process of a client name, and start its
    /// command again. Needs a grant of `process.spawn`.
    Restart {
        /// The client name, for example `tiling`.
        #[arg(value_name = "NAME")]
        client: String,
    },
    /// Store a file for image nodes, shader nodes and effects in a
    /// directory of the core, or remove it; for a client that shares no
    /// directory with the core.
    Asset {
        #[command(subcommand)]
        action: AssetCmd,
    },
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
        #[arg(value_name = "NAME")]
        view: String,
    },
    /// Remove the nodes of a view.
    Unload {
        /// The name of the view.
        #[arg(value_name = "NAME")]
        view: String,
    },
    /// Write the nodes of a view as they are now, with the plugins of its
    /// file.
    Save {
        /// The name of the view.
        #[arg(value_name = "NAME")]
        view: String,
        /// The file; `~/.config/chora/views/NAME.yaml` if missing.
        #[arg(long)]
        file: Option<String>,
    },
}

/// The arguments of `choractl reserve`.
#[derive(clap::Args)]
struct ReserveArgs {
    /// The output; the first one if missing.
    #[arg(long)]
    output: Option<String>,
    /// The space at the top.
    #[arg(long, default_value_t = 0)]
    top: u32,
    /// The space at the right.
    #[arg(long, default_value_t = 0)]
    right: u32,
    /// The space at the bottom.
    #[arg(long, default_value_t = 0)]
    bottom: u32,
    /// The space at the left.
    #[arg(long, default_value_t = 0)]
    left: u32,
}

#[derive(Subcommand)]
enum AssetCmd {
    /// Store a file under a name and print its path. A second put of the
    /// name replaces the file.
    Put {
        /// The name, for example `glow.frag`.
        #[arg(value_name = "NAME")]
        asset: String,
        /// The file; stdin for `-`.
        file: String,
    },
    /// Remove a file of an earlier put.
    Remove {
        /// The name.
        #[arg(value_name = "NAME")]
        asset: String,
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
        Cmd::Query { selector } => show(&mut client, "scene.query", Query { selector }),
        Cmd::Apply { .. } => client
            .call::<_, Value>("scene.apply", Apply { commands })
            .map(drop),
        Cmd::Events { names } => events(&mut client, names),
        Cmd::Outputs => show(&mut client, "output.list", Value::Null),
        Cmd::Reserve(args) => reserve(&mut client, args),
        Cmd::Spawn { argv } => show(&mut client, "process.spawn", Spawn { argv }),
        Cmd::Restart { client: name } => show(&mut client, "plugins.restart", Restart { name }),
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
        Cmd::Grants => show(&mut client, "grants.list", Value::Null),
        Cmd::Asset { action } => return asset(&mut client, action),
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

/// Calls a method and prints its result.
fn show<P: serde::Serialize>(
    client: &mut Client,
    method: &str,
    params: P,
) -> Result<(), ClientError> {
    client.call(method, params).map(|v| print(&v))
}

/// Stores or removes an asset.
fn asset(client: &mut Client, action: AssetCmd) -> Result<(), String> {
    match action {
        AssetCmd::Put { asset: name, file } => {
            let data = if file == "-" {
                let mut data = Vec::new();
                std::io::stdin().read_to_end(&mut data).map(|_| data)
            } else {
                std::fs::read(&file)
            }
            .map_err(|e| format!("{file}: {e}"))?;
            let put = AssetPut {
                name,
                data: STANDARD.encode(data),
            };
            let asset: Asset = client.call("assets.put", put).map_err(|e| e.to_string())?;
            println!("{}", asset.path);
        }
        AssetCmd::Remove { asset: name } => {
            client
                .call::<_, Value>("assets.remove", AssetRemove { name })
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Runs a view command with the client of the view.
fn view_command(action: &ViewCmd) -> Result<(), String> {
    let result = match action {
        ViewCmd::Load { view } => view::load(view),
        ViewCmd::Unload { view } => view::unload(view),
        ViewCmd::Save { view, file } => view::save(view, file.as_deref()),
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

/// Reserves space and holds it until the core closes the connection or the
/// command ends.
fn reserve(client: &mut Client, args: ReserveArgs) -> Result<(), ClientError> {
    let reserve = Reserve {
        output: args.output,
        edges: Edges {
            top: args.top,
            right: args.right,
            bottom: args.bottom,
            left: args.left,
        },
    };
    let output: Value = client.call("output.reserve", reserve)?;
    print(&output);
    let _ = std::io::stdout().flush();
    loop {
        match client.next_event() {
            Err(ClientError::Closed) => return Ok(()),
            Err(err) => return Err(err),
            Ok(_) => {}
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Args, AssetCmd, Cmd};

    /// No argument of a command takes the place of `--name`.
    #[test]
    fn arguments_keep_the_client_name() {
        let parse = |line: &str| Args::try_parse_from(line.split(' ')).unwrap();
        for line in [
            "choractl --name agent asset put x.frag f",
            "choractl --name agent asset remove x.frag",
            "choractl --name agent restart tiling",
            "choractl --name agent view load v",
            "choractl --name agent grant tiling input.inject",
        ] {
            assert_eq!(parse(line).name.as_deref(), Some("agent"), "{line}");
        }
        let args = parse("choractl asset put x.frag f");
        assert!(args.name.is_none());
        assert!(
            matches!(args.command, Cmd::Asset { action: AssetCmd::Put { asset, .. } } if asset == "x.frag")
        );
    }
}
