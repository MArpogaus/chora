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
        GrantChange, Hello, Inject, MethodCall, Picture, ProcessWait, Query, Reserve, Restart,
        SOCKET_ENV, Spawn, Subscribe, admin_socket,
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
    /// `choractl call tiling set_layout '{"layout": "fair"}'`.
    Call {
        /// The client that exported the method.
        client: String,
        /// The method name.
        method: String,
        /// The parameters as JSON; `null` if missing.
        params: Option<String>,
    },
    /// Grant a client name `input.inject`, `screen.capture`,
    /// `process.spawn` or `input.grab-any`, until the session ends or a
    /// revoke. Goes through the admin socket.
    Grant {
        /// The client name, for example `agent`.
        client: String,
        /// One or more of `input.inject`, `screen.capture`,
        /// `process.spawn` and `input.grab-any`.
        #[arg(required = true)]
        capabilities: Vec<Capability>,
    },
    /// Revoke grants of a client name; all of them without a capability.
    /// Goes through the admin socket.
    Revoke {
        /// The client name.
        client: String,
        /// One or more of `input.inject`, `screen.capture`,
        /// `process.spawn` and `input.grab-any`.
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
        /// Wait for the end of the program, and print its output and its
        /// exit status.
        #[arg(long)]
        capture: bool,
        /// The program and its arguments.
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        argv: Vec<String>,
    },
    /// Set the scale or the transform of an output, as wlr-randr does.
    ConfigureOutput {
        /// The output; the first one if missing.
        #[arg(long)]
        output: Option<String>,
        /// The scale, from 0.1 to 10.
        #[arg(long)]
        scale: Option<f64>,
        /// The transform: normal, 90, 180, 270, flipped, flipped-90,
        /// flipped-180 or flipped-270.
        #[arg(long)]
        transform: Option<String>,
    },
    /// Add a virtual output, as a monitor plugs in, and print it: `WxH`
    /// with an optional `@SCALE` and `+X+Y`. Only for tests on the headless
    /// backend.
    AddOutput {
        /// For example `1920x1080@1.5+1280+0`.
        spec: String,
    },
    /// Remove a virtual output, as a monitor unplugs. Only for tests on the
    /// headless backend.
    RemoveOutput {
        /// The output, for example `HEADLESS-2`.
        #[arg(value_name = "NAME")]
        output: String,
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
    /// Manage the client sockets, which fix the name of their clients.
    /// Goes through the admin socket.
    Clients {
        #[command(subcommand)]
        action: ClientsCmd,
    },
    /// Take the camera of an output away from its owner, so that the view
    /// goes back to its default. Goes through the admin socket.
    Camera {
        #[command(subcommand)]
        action: CameraCmd,
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
enum CameraCmd {
    /// End the ownership; the next `camera` command of any client wins.
    Release {
        /// The output, for example `HDMI-A-1`.
        output: String,
    },
}

#[derive(Subcommand)]
enum ClientsCmd {
    /// Open the client socket of a name, `clients/NAME/ipc.sock` next to
    /// `CHORA_SOCKET`, and print its path.
    Create {
        /// The client name, for example `agent`.
        client: String,
    },
    /// Create the sockets of all names in a YAML file, with their
    /// grants: `clients.create` and `grants.set` in one step.
    Up {
        /// The file, `~/.config/chora/clients.yaml` by default, `-` for
        /// stdin.
        file: Option<String>,
    },
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
        resume: false,
    };
    let mut client = connect(&args.command, &hello)?;
    let result = match args.command {
        Cmd::Query { selector } => show(&mut client, "scene.query", Query { selector }),
        Cmd::Apply { .. } => client
            .call::<_, Value>("scene.apply", Apply { commands })
            .map(drop),
        Cmd::Events { names } => events(&mut client, names),
        Cmd::Outputs => show(&mut client, "output.list", Value::Null),
        command @ (Cmd::Spawn { .. }
        | Cmd::Restart { .. }
        | Cmd::ConfigureOutput { .. }
        | Cmd::AddOutput { .. }
        | Cmd::RemoveOutput { .. }
        | Cmd::Reserve(_)
        | Cmd::Grant { .. }
        | Cmd::Revoke { .. }
        | Cmd::Grants
        | Cmd::Clients { .. }
        | Cmd::Camera { .. }) => session(&mut client, command),
        Cmd::Call {
            client: callee,
            method,
            params,
        } => return call(&mut client, callee, method, params.as_deref()),
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

/// Connects to the core: grants and client sockets through the admin
/// socket (D40), the rest through `CHORA_SOCKET`.
fn connect(command: &Cmd, hello: &Hello) -> Result<Client, String> {
    let admin = matches!(
        command,
        Cmd::Grant { .. } | Cmd::Revoke { .. } | Cmd::Clients { .. } | Cmd::Camera { .. }
    );
    if !admin {
        return Client::from_env(hello).map_err(|e| e.to_string());
    }
    let main = std::env::var_os(SOCKET_ENV).ok_or("CHORA_SOCKET is not set")?;
    // A symlink to the socket names the directory of the core only once
    // resolved.
    let main = std::fs::canonicalize(&main).unwrap_or_else(|_| main.into());
    let path = admin_socket(&main);
    Client::connect(&path, hello).map_err(|e| format!("{}: {e}", path.display()))
}

/// The commands for grants, programs and outputs.
fn session(client: &mut Client, command: Cmd) -> Result<(), ClientError> {
    match command {
        Cmd::Reserve(args) => reserve(client, args),
        Cmd::Grant {
            client: name,
            capabilities,
        } => grants(client, "grants.set", name, capabilities),
        Cmd::Revoke {
            client: name,
            capabilities,
        } => grants(client, "grants.revoke", name, capabilities),
        Cmd::Grants => show(client, "grants.list", Value::Null),
        Cmd::Clients {
            action: ClientsCmd::Create { client: name },
        } => show(client, "clients.create", serde_json::json!({"name": name})),
        Cmd::Clients {
            action: ClientsCmd::Up { file },
        } => clients_up(client, file.as_deref()),
        Cmd::Camera {
            action: CameraCmd::Release { output },
        } => show(
            client,
            "camera.release",
            serde_json::json!({"output": output}),
        ),
        Cmd::Spawn { capture, argv } => spawn(client, argv, capture),
        Cmd::Restart { client: name } => show(client, "plugins.restart", Restart { name }),
        Cmd::ConfigureOutput {
            output,
            scale,
            transform,
        } => {
            let params =
                serde_json::json!({"output": output, "scale": scale, "transform": transform});
            show(client, "output.configure", params)
        }
        Cmd::AddOutput { spec } => show(client, "output.add", serde_json::json!({"spec": spec})),
        Cmd::RemoveOutput { output } => client
            .call::<_, Value>("output.remove", serde_json::json!({"output": output}))
            .map(drop),
        _ => unreachable!("only commands for grants, programs and outputs come here"),
    }
}

/// Starts a program. With `capture`, waits for its end on the same
/// connection, since the output goes with the last connection of the name.
fn spawn(client: &mut Client, argv: Vec<String>, capture: bool) -> Result<(), ClientError> {
    let started: Value = client.call("process.spawn", Spawn { argv, capture })?;
    let pid = started["pid"].as_u64().and_then(|p| u32::try_from(p).ok());
    let Some(pid) = pid.filter(|_| capture) else {
        print(&started);
        return Ok(());
    };
    loop {
        let wait = ProcessWait {
            pid,
            timeout_ms: 30_000,
        };
        let output: Value = client.call("process.wait", wait)?;
        if output["running"] != Value::Bool(true) {
            print(&output);
            return Ok(());
        }
    }
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

/// One name of `clients up`: its grants; the socket itself needs no
/// more than the name.
#[derive(serde::Deserialize)]
struct ClientSpec {
    #[serde(default)]
    grants: Vec<Capability>,
}

/// Creates the socket of every name in the file and sets its grants.
fn clients_up(client: &mut Client, file: Option<&str>) -> Result<(), ClientError> {
    let bad = |e: String| ClientError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
    let text = match file {
        Some("-") => {
            let mut text = String::new();
            std::io::stdin().read_to_string(&mut text)?;
            text
        }
        Some(f) => std::fs::read_to_string(f)?,
        None => {
            let mut paths = Vec::new();
            if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
                paths.push(format!("{xdg}/chora/clients.yaml"));
            }
            if let Ok(home) = std::env::var("HOME") {
                paths.push(format!("{home}/.config/chora/clients.yaml"));
            }
            let path = paths
                .iter()
                .find(|p| std::path::Path::new(p).exists())
                .cloned()
                .ok_or_else(|| bad("no clients.yaml; give the file as argument".into()))?;
            std::fs::read_to_string(&path).map_err(|e| bad(format!("{path}: {e}")))?
        }
    };
    let wanted: std::collections::BTreeMap<String, ClientSpec> =
        view::parse_yaml(&text).map_err(bad)?;
    for (name, spec) in wanted {
        view::check_name(&name).map_err(bad)?;
        let kept =
            match client.call::<_, Value>("clients.create", serde_json::json!({"name": &name})) {
                Ok(_) => None,
                Err(ClientError::Rpc(err)) => {
                    let reason = err
                        .data
                        .as_ref()
                        .and_then(|d| d.get("reason"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if reason != "socket_exists" && reason != "connected" {
                        return Err(ClientError::Rpc(err));
                    }
                    Some(reason.to_owned())
                }
                Err(e) => return Err(e),
            };
        if let Some(reason) = kept {
            eprintln!("{name}: {reason}: socket not opened, only grants synced");
        }
        sync_grants(client, &name, spec.grants)?;
        println!("{name}");
    }
    Ok(())
}

/// Makes the grants of a name equal to the listed ones, part by part:
/// an unchanged grant stays, so no client is disconnected and no moment
/// passes without it.
fn sync_grants(
    client: &mut Client,
    name: &str,
    listed: Vec<Capability>,
) -> Result<(), ClientError> {
    let grants = client.call::<_, Value>("grants.list", Value::Null)?;
    let current: std::collections::BTreeSet<Capability> = grants
        .get(name)
        .cloned()
        .map(|v| serde_json::from_value(v).map_err(ClientError::Json))
        .transpose()?
        .unwrap_or_default();
    let wanted = listed
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let (gone, added) = grant_delta(&current, &wanted);
    if !gone.is_empty() {
        client.call::<_, Value>(
            "grants.revoke",
            serde_json::json!({"client": name, "capabilities": gone}),
        )?;
    }
    if !added.is_empty() {
        client.call::<_, Value>(
            "grants.set",
            GrantChange {
                client: name.into(),
                capabilities: added,
            },
        )?;
    }
    Ok(())
}

/// The grants to revoke and the grants to set so that `current` becomes
/// `wanted`; an unchanged grant appears in neither list.
fn grant_delta(
    current: &std::collections::BTreeSet<Capability>,
    wanted: &std::collections::BTreeSet<Capability>,
) -> (Vec<Capability>, Vec<Capability>) {
    (
        current.difference(wanted).copied().collect(),
        wanted.difference(current).copied().collect(),
    )
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

    use super::{Args, AssetCmd, Capability, ClientSpec, Cmd, grant_delta};

    /// An unchanged grant is revoked by no one and set by no one.
    #[test]
    fn the_grant_delta_of_an_unchanged_file_is_empty() {
        let have = [Capability::InputInject, Capability::ScreenCapture]
            .into_iter()
            .collect();
        let (gone, added) = grant_delta(&have, &have);
        assert!(gone.is_empty() && added.is_empty());
        let wanted = [Capability::ScreenCapture].into_iter().collect();
        let (gone, added) = grant_delta(&have, &wanted);
        assert_eq!(gone, [Capability::InputInject]);
        assert!(added.is_empty() && wanted.len() == 1);
    }

    /// The clients file of `clients up` maps names to their grants.
    #[test]
    fn a_clients_file_lists_names_and_grants() {
        let args = Args::try_parse_from("choractl clients up -".split(' ')).unwrap();
        assert!(matches!(
            args.command,
            Cmd::Clients {
                action: super::ClientsCmd::Up {
                    file: Some(ref f)
                }
            } if f == "-"
        ));
        let file = "agent:\n  grants: [screen.capture, input.grab-any]\nguest: {}\n";
        let wanted: std::collections::BTreeMap<String, ClientSpec> =
            super::view::parse_yaml(file).unwrap();
        assert_eq!(wanted["agent"].grants.len(), 2);
        assert_eq!(wanted["guest"].grants.len(), 0);
        assert!(
            super::view::parse_yaml::<std::collections::BTreeMap<String, ClientSpec>>(
                "agent: { grants: [not-a-capability] }",
            )
            .is_err(),
            "an unknown capability is an error"
        );
    }

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
