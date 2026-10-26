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
        GrantChange, Hello, Inject, InjectEvent, MethodCall, Picture, Query, Reserve, RpcError,
        SOCKET_ENV, Subscribe, admin_socket,
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
    /// Wait for a selector: poll `scene.query` every 100 ms, print the
    /// matching nodes as one JSON line and exit 0. With `--gone`, wait
    /// until nothing matches. After the timeout, exit 1.
    Wait {
        /// The selector.
        selector: String,
        /// The seconds to wait; 10 by default.
        #[arg(long, default_value_t = 10)]
        timeout: u64,
        /// Wait until nothing matches the selector.
        #[arg(long)]
        gone: bool,
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
    /// Grant a client name capabilities until the session ends or a
    /// revoke. Goes through the admin socket.
    Grant {
        /// The client name, for example `agent`.
        client: String,
        /// One or more capabilities, for example `screen.capture`.
        #[arg(required = true)]
        capabilities: Vec<Capability>,
    },
    /// Revoke grants of a client name; all of them without a capability.
    /// Goes through the admin socket.
    Revoke {
        /// The client name.
        client: String,
        /// One or more capabilities, for example `screen.capture`.
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
        /// Check the events and answer ok or the error, without sending
        /// input.
        #[arg(long)]
        dry_run: bool,
    },
    /// Serve the API as MCP tools over stdio, for an agent.
    Mcp,
    /// Start a program outside the core (D42), with an activation token
    /// of the core in `XDG_ACTIVATION_TOKEN`: in its own transient unit
    /// when a user manager runs, as its own child otherwise. Prints pid,
    /// unit and token as one JSON line.
    Run {
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
    /// Restart a plugin (D42): `systemctl --user restart` of its unit,
    /// `chora-NAME-SESSION.service` of a view in this session or
    /// `chora-NAME.service` of a flavor. Prints the unit as one JSON line.
    /// Does not call the core.
    Restart {
        /// The plugin name, for example `tiling`.
        #[arg(value_name = "NAME")]
        plugin: String,
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
    /// Open the client socket without capabilities, `clients/NAME/ipc.sock`
    /// next to `CHORA_SOCKET`, and print its path; `clients up` gives a
    /// name its capabilities.
    Create {
        /// The client name, for example `agent`.
        client: String,
    },
    /// Create the sockets of all names in a YAML file with their
    /// capabilities: `clients.create` with the list of the file. Changed
    /// capabilities recreate the socket.
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
    match run(Args::parse()) {
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
        Cmd::Restart { plugin } => return restart_program(plugin),
        _ => {}
    }
    let (commands, input) = match &args.command {
        Cmd::Apply { file } => (read_json(file.as_deref())?, Vec::new()),
        Cmd::Inject { file, .. } => (Vec::new(), read_json(file.as_deref())?),
        _ => (Vec::new(), Vec::new()),
    };
    let mut client = connect(&args.command, &hello(args.name))?;
    let result = match args.command {
        Cmd::Query { selector } => show(&mut client, "scene.query", Query::new(selector)),
        command @ Cmd::Wait { .. } => return wait(&mut client, &command),
        Cmd::Apply { .. } => client
            .call::<_, Value>("scene.apply", Apply { commands })
            .map(drop),
        Cmd::Events { names } => events(&mut client, names),
        Cmd::Outputs => show(&mut client, "output.list", Value::Null),
        command @ (Cmd::Run { .. }
        | Cmd::ConfigureOutput { .. }
        | Cmd::AddOutput { .. }
        | Cmd::RemoveOutput { .. }
        | Cmd::Reserve(_)
        | Cmd::Grant { .. }
        | Cmd::Revoke { .. }
        | Cmd::Grants
        | Cmd::Camera { .. }) => session(&mut client, command),
        Cmd::Clients { action } => return clients_command(&mut client, action),
        Cmd::Call {
            client: callee,
            method,
            params,
        } => return call(&mut client, callee, method, params.as_deref()),
        Cmd::Asset { action } => return asset(&mut client, action),
        Cmd::View { .. } | Cmd::Mcp | Cmd::Restart { .. } => {
            unreachable!("handled before the connection")
        }
        Cmd::Inject { dry_run, .. } => inject(&mut client, input, dry_run),
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

/// The `hello` of every `choractl` call: the name of `--name`, with
/// `keep_nodes`.
fn hello(name: Option<String>) -> Hello {
    Hello {
        name: name.unwrap_or_else(|| "choractl".into()),
        keep_nodes: true,
        ..Hello::default()
    }
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

/// Runs `input.inject`. A dry run checks the events and sends nothing.
fn inject(client: &mut Client, events: Vec<InjectEvent>, dry_run: bool) -> Result<(), ClientError> {
    client
        .call::<_, Value>("input.inject", Inject { events, dry_run })
        .map(drop)
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
        Cmd::Camera {
            action: CameraCmd::Release { output },
        } => show(
            client,
            "camera.release",
            serde_json::json!({"output": output}),
        ),
        Cmd::Run { argv } => {
            run_program(client, &argv, None).map(|line| print_line(std::io::stdout(), &line))
        }
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

/// `choractl clients ...`, which all go through the admin socket.
fn clients_command(client: &mut Client, action: ClientsCmd) -> Result<(), String> {
    match action {
        ClientsCmd::Create { client: name } => client_create(client, &name),
        ClientsCmd::Up { file } => clients_up(client, file.as_deref()).map_err(|e| e.to_string()),
    }
}

/// Opens the client socket of a name and prints its path. A name that has a
/// socket already gets that socket on stderr.
fn client_create(client: &mut Client, name: &str) -> Result<(), String> {
    let answer = client.call::<_, Value>("clients.create", serde_json::json!({"name": name}));
    match answer {
        Ok(path) => {
            print(&path);
            Ok(())
        }
        Err(ClientError::Rpc(err)) => {
            Err(taken_socket(name, &err).unwrap_or_else(|| err.to_string()))
        }
        Err(err) => Err(err.to_string()),
    }
}

/// The message for a name whose client socket exists already, from the
/// `path` in the error data of the core. `None` for every other error.
fn taken_socket(name: &str, err: &RpcError) -> Option<String> {
    let data = err.data.as_ref()?;
    if data.get("reason").and_then(Value::as_str) != Some("socket_exists") {
        return None;
    }
    let path = data.get("path").and_then(Value::as_str)?;
    Some(format!("{name} has a client socket already: {path}"))
}

/// `choractl run` (D42): asks the core for an activation token, starts
/// the program with it in the environment, and answers pid, unit and
/// token as one JSON value; the token lets a test follow the trip to the
/// window. With `plugin`, the unit is the one of `view_unit` and
/// restarts the program when it fails, as the unit of a flavor does.
pub(crate) fn run_program(
    client: &mut Client,
    argv: &[String],
    plugin: Option<&str>,
) -> Result<Value, ClientError> {
    if argv.is_empty() {
        return Err(ClientError::Io(std::io::Error::other("no program")));
    }
    let answer: Value = client.call("activation.token", Value::Null)?;
    let Some(token) = answer["token"].as_str().map(str::to_owned) else {
        return Err(ClientError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "the core answered no activation token",
        )));
    };
    let unit = plugin.map_or_else(
        || {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            format!(
                "chora-run-{}-{}.service",
                std::process::id(),
                now.as_nanos()
            )
        },
        view_unit,
    );
    let (pid, unit) = match systemd_unit(argv, &token, &unit, plugin.is_some()) {
        RunStart::Unit(pid) => (Some(pid), Some(unit)),
        RunStart::NoManager => (
            Some(start_child(argv, &token).map_err(|e| {
                ClientError::Io(std::io::Error::new(e.kind(), format!("{}: {e}", argv[0])))
            })?),
            None,
        ),
        // The unit started; a short program ended and was collected
        // before its pid was read.
        RunStart::Unknown => (None, Some(unit)),
        RunStart::Refused => {
            return Err(ClientError::Io(std::io::Error::other(format!(
                "systemd-run refused the unit {unit}, see its error above; a unit that runs already stops with systemctl --user stop {unit}"
            ))));
        }
    };
    Ok(serde_json::json!({ "pid": pid, "unit": unit, "token": token }))
}

/// The outcome of one start attempt through `systemd-run`.
#[derive(Debug, PartialEq, Eq)]
enum RunStart {
    /// No user manager answers: `run` starts the program as its child.
    NoManager,
    /// `systemd-run` failed, for example because the unit exists already
    /// or the systemd is older than 254; its error is on stderr. `run`
    /// fails too and never starts the program a second time.
    Refused,
    /// The unit runs the program under this pid.
    Unit(u32),
    /// The unit started, but its `MainPID` stayed unknown, for example
    /// of a program that ended at once: `run` prints `pid` null and never
    /// starts the program a second time.
    Unknown,
}

/// The pid of a unit that started: a `MainPID` that is empty, `0` or no
/// number is `Unknown`, never pid 0 as success.
fn classify_start(main_pid: &str) -> RunStart {
    match main_pid.trim().parse::<u32>() {
        Ok(pid) if pid > 0 => RunStart::Unit(pid),
        _ => RunStart::Unknown,
    }
}

/// How starting the program in the transient unit `unit` went.
/// `systemd-run` without `--scope` returns once the start job of the
/// unit is done, so the answer comes while the program runs; the pid is
/// the program's own (`MainPID`), waited for up to 2 s. The unit lives
/// until the last process in it ends, so a launcher may exit before the
/// app it started. The arguments reach the program literally. Output
/// goes to the journal, never into the line on stdout.
fn systemd_unit(argv: &[String], token: &str, unit: &str, restart: bool) -> RunStart {
    if !manager_runs() {
        return RunStart::NoManager;
    }
    let mut started = std::process::Command::new("systemd-run");
    started
        .args([
            "--user",
            "--collect",
            "--quiet",
            "--same-dir",
            "--expand-environment=no",
            "-pExitType=cgroup",
            "--unit",
            unit,
        ])
        .env("XDG_ACTIVATION_TOKEN", token)
        .args(unit_env());
    if restart {
        started.arg("-pRestart=on-failure");
    }
    // The MCP protocol of `choractl mcp` runs on stdin and stdout.
    if !started
        .arg("--")
        .args(argv)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        return RunStart::Refused;
    }
    // A unit fills in `MainPID` a moment after the start job. Wait for
    // it, then report what is known: a pid, or the error of the unknown
    // pid. A unit that ended and was collected before the first query
    // counts as unknown too; the program ran, but `run` cannot say so.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let shown = std::process::Command::new("systemctl")
            .args([
                "--user",
                "show",
                "--property=MainPID",
                "--value",
                "--unit",
                unit,
            ])
            .stderr(std::process::Stdio::null())
            .output();
        let text = match &shown {
            Ok(out) => String::from_utf8_lossy(&out.stdout).into_owned(),
            Err(_) => String::new(),
        };
        let decision = classify_start(&text);
        if decision != RunStart::Unknown || std::time::Instant::now() >= deadline {
            return decision;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// The program forked by `choractl` itself, with the token in its
/// environment; the pid of the child. It stays in the process group of
/// the caller, so that a test session that ends its group ends it too.
/// Stdout goes to the null device, so that the output of the program
/// never mixes with the JSON line or the MCP protocol on stdout; stderr
/// stays with the caller.
fn start_child(argv: &[String], token: &str) -> Result<u32, std::io::Error> {
    let mut child = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .env("XDG_ACTIVATION_TOKEN", token)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .spawn()?;
    let pid = child.id();
    // `choractl mcp` lives long: wait for the child, so it never stays a
    // zombie.
    std::thread::spawn(move || child.wait());
    Ok(pid)
}

/// The unit of a plugin that a view starts: one per plugin and session,
/// since every chora of the user shares the user manager. The session is
/// an FNV-1a hash of `CHORA_SOCKET`, the one path that no two cores
/// share; every build of choractl gives the same hash.
fn view_unit(name: &str) -> String {
    let socket = std::env::var(SOCKET_ENV).unwrap_or_default();
    let hash = socket.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("chora-{name}-{:08x}.service", hash >> 32)
}

/// Whether a user manager runs. Without one, as in a container or a
/// chroot, systemctl answers `offline` or nothing.
fn manager_runs() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "is-system-running"])
        .stderr(std::process::Stdio::null())
        .output()
        .is_ok_and(|out| {
            let state = String::from_utf8_lossy(&out.stdout);
            !matches!(state.trim(), "" | "offline")
        })
}

/// The `WAYLAND_DISPLAY` of the user manager, which a flavor sets.
fn manager_display() -> Option<String> {
    let shown = std::process::Command::new("systemctl")
        .args(["--user", "show-environment"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    String::from_utf8_lossy(&shown.stdout)
        .lines()
        .find_map(|l| l.strip_prefix("WAYLAND_DISPLAY="))
        .map(str::to_owned)
}

/// `choractl restart` (D42): restarts the systemd user unit of a plugin,
/// the unit of a view in this session or else the unit of the flavor of
/// this session, and prints the unit as one JSON line.
fn restart_program(name: &str) -> Result<(), String> {
    view::check_name(name)?;
    let mut units = vec![view_unit(name)];
    // The flavor unit belongs to the session of the user manager, not to
    // a nested chora on it.
    if manager_display() == std::env::var("WAYLAND_DISPLAY").ok() {
        units.push(format!("chora-{name}.service"));
    }
    // A plain restart would also start a plugin that does not run.
    let unit = units
        .iter()
        .find(|unit| {
            std::process::Command::new("systemctl")
                .args(["--user", "is-active", "--quiet", unit])
                .status()
                .is_ok_and(|s| s.success())
        })
        .ok_or_else(|| format!("{name}: no active unit {}", units.join(" or ")))?;
    let restarted = std::process::Command::new("systemctl")
        .args(["--user", "restart", unit])
        .status()
        .map_err(|e| format!("systemctl: {e}"))?;
    if !restarted.success() {
        return Err(format!("systemctl --user restart {unit} failed"));
    }
    print_line(std::io::stdout(), &serde_json::json!({ "unit": unit }));
    Ok(())
}

/// The environment options of a unit of `run`: the token and the
/// variables of this session, each with the value of this process. The
/// user manager may hold the variables of another session, for example
/// the host of a nested chora, so each host variable gets this session's
/// value or none. All other variables come from the user manager.
fn unit_env() -> Vec<String> {
    let mut options = vec!["--setenv=XDG_ACTIVATION_TOKEN".to_owned()];
    for key in ["PATH", "XDG_RUNTIME_DIR", "WAYLAND_DISPLAY", "CHORA_SOCKET"]
        .into_iter()
        .chain(chora_scene::ipc::HOST_VARIABLES)
    {
        options.push(if std::env::var_os(key).is_some() {
            format!("--setenv={key}")
        } else {
            format!("-pUnsetEnvironment={key}")
        });
    }
    options
}

/// Polls `scene.query` until the selector matches, or with `gone` until
/// nothing matches, and prints the matches as one JSON line. A timeout is
/// an error, so the command exits 1 with the reason on stderr.
fn wait(client: &mut Client, command: &Cmd) -> Result<(), String> {
    let Cmd::Wait {
        selector,
        timeout,
        gone,
    } = &command
    else {
        unreachable!("wait runs for the wait command only")
    };
    let nodes = wait_loop(
        || {
            client
                .call::<_, Vec<Value>>("scene.query", Query::new(selector.clone()))
                .map_err(|e| e.to_string())
        },
        selector,
        *gone,
        std::time::Duration::from_secs(*timeout),
        std::time::Duration::from_millis(100),
    )?;
    print_line(std::io::stdout(), &Value::Array(nodes));
    Ok(())
}

/// The poll loop of `wait`, split out so that a test can fake the query.
fn wait_loop(
    mut query: impl FnMut() -> Result<Vec<Value>, String>,
    selector: &str,
    gone: bool,
    timeout: std::time::Duration,
    every: std::time::Duration,
) -> Result<Vec<Value>, String> {
    let start = std::time::Instant::now();
    loop {
        let nodes = query()?;
        if nodes.is_empty() == gone {
            return Ok(nodes);
        }
        if start.elapsed() > timeout {
            let reason = if gone {
                format!("nodes still match {selector}")
            } else {
                format!("no node matches {selector}")
            };
            return Err(format!("timeout after {} s: {reason}", timeout.as_secs()));
        }
        std::thread::sleep(every);
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

/// One name of `clients up`: its capabilities; the socket itself needs no
/// more than the name.
#[derive(serde::Deserialize)]
struct ClientSpec {
    #[serde(default)]
    grants: Vec<Capability>,
}

/// Opens the socket of every name in the file with exactly its
/// capabilities (D33, D40). Equal capabilities keep the socket; changed
/// capabilities make the core recreate it. Only a name that is connected
/// gets no socket yet; `sync_grants` then carries its capabilities until
/// the create works.
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
        let params = serde_json::json!({"name": &name, "capabilities": spec.grants});
        let err = match client.call::<_, Value>("clients.create", params) {
            Ok(_) => {
                println!("{name}");
                continue;
            }
            Err(ClientError::Rpc(err)) => err,
            Err(e) => return Err(e),
        };
        match err
            .data
            .as_ref()
            .and_then(|d| d.get("reason"))
            .and_then(Value::as_str)
        {
            // Equal capabilities: the socket and its connections stay.
            Some("socket_exists") => println!("{name}"),
            // No socket yet: until the name disconnects, its capabilities
            // live in the grants, which still work for it.
            Some("connected") => {
                eprintln!("{name}: connected: socket not opened, only grants synced");
                sync_grants(client, &name, spec.grants)?;
                println!("{name}");
            }
            _ => return Err(ClientError::Rpc(err)),
        }
    }
    Ok(())
}

/// Makes the grants of a not yet opened name equal to the listed ones,
/// part by part: an unchanged grant stays, so no moment passes without
/// it. A name with a client socket gets its capabilities from the core.
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

/// Prints a value as one JSON line, for the answers scripts read. Takes the
/// writer so that a test can read the line.
fn print_line<W: Write>(mut out: W, value: &Value) {
    let line = serde_json::to_string(value).expect("values always serialize");
    // A closed stdout, for example after `| head`, is no error.
    let _ = writeln!(out, "{line}");
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use serde_json::{Value, json};
    use std::time::{Duration, Instant};

    use chora_scene::ipc::RpcError;

    use super::{
        Args, AssetCmd, Capability, ClientSpec, Cmd, grant_delta, print_line, taken_socket,
        wait_loop,
    };

    /// A line of `run` is compact JSON, so that a script reads it at once.
    #[test]
    fn the_answer_of_run_is_one_json_line() {
        let mut line = Vec::new();
        print_line(&mut line, &json!({ "pid": 7, "stdout": "a\nb" }));
        assert_eq!(
            String::from_utf8(line).unwrap(),
            "{\"pid\":7,\"stdout\":\"a\\nb\"}\n",
        );
    }

    /// The poll loop stops at the first satisfying answer; a timeout is an
    /// error with the selector.
    #[test]
    fn wait_polls_until_the_answer_satisfies_the_selector() {
        let every = Duration::from_millis(1);
        let mut answers = vec![Vec::new(), Vec::new(), vec![json!({ "id": "n1" })]];
        let got = wait_loop(
            || Ok::<Vec<Value>, String>(answers.remove(0)),
            "kind:surface",
            false,
            Duration::from_secs(5),
            every,
        )
        .unwrap();
        assert_eq!(got, vec![json!({ "id": "n1" })]);

        let gone = wait_loop(
            || Ok::<Vec<Value>, String>(Vec::new()),
            "kind:surface",
            true,
            Duration::from_secs(5),
            every,
        )
        .unwrap();
        assert!(gone.is_empty(), "--gone ends on the first empty answer");

        let err = wait_loop(
            || Ok::<Vec<Value>, String>(Vec::new()),
            "kind:surface",
            false,
            Duration::from_millis(5),
            every,
        )
        .unwrap_err();
        assert!(err.contains("timeout"), "{err}");
        let still = wait_loop(
            || Ok::<Vec<Value>, String>(vec![json!({ "id": "n1" })]),
            "kind:surface",
            true,
            Duration::from_millis(5),
            every,
        );
        assert!(still.is_err(), "--gone times out while nodes match");

        let args = Args::try_parse_from("choractl wait kind:surface --timeout 3 --gone".split(' '))
            .unwrap();
        assert!(
            matches!(args.command, Cmd::Wait { ref selector, timeout, gone } if selector == "kind:surface" && timeout == 3 && gone)
        );
    }

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

    /// A name whose client socket exists already gets that socket by name;
    /// every other error keeps the message of the core.
    #[test]
    fn a_taken_client_name_names_its_socket() {
        let taken = RpcError::new(RpcError::REJECTED, "\"agent\" has a client socket already")
            .with_data(json!({
                "reason": "socket_exists",
                "path": "/run/chora/clients/agent/ipc.sock",
            }));
        assert_eq!(
            taken_socket("agent", &taken).as_deref(),
            Some("agent has a client socket already: /run/chora/clients/agent/ipc.sock"),
        );
        let connected = RpcError::new(RpcError::REJECTED, "\"agent\" is connected")
            .with_data(json!({"reason": "connected"}));
        assert!(taken_socket("agent", &connected).is_none());
        let no_data = RpcError::new(RpcError::REJECTED, "no socket directory");
        assert!(taken_socket("agent", &no_data).is_none());
    }

    /// A unit of `run` takes no host session variable from the user
    /// manager: each one is set from this session or unset.
    #[test]
    fn a_unit_sets_or_unsets_every_host_variable() {
        let props = super::unit_env();
        for key in chora_scene::ipc::HOST_VARIABLES {
            let set = format!("--setenv={key}");
            let unset = format!("-pUnsetEnvironment={key}");
            let hits = props.iter().filter(|p| **p == set || **p == unset).count();
            assert_eq!(hits, 1, "{key} in {props:?}");
        }
    }

    /// A started unit with an unknown `MainPID` answers no pid, never
    /// pid 0.
    #[test]
    fn a_unit_answers_its_pid_or_none() {
        assert_eq!(super::classify_start("4343\n"), super::RunStart::Unit(4343));
        assert_eq!(super::classify_start("0"), super::RunStart::Unknown);
        assert_eq!(super::classify_start(""), super::RunStart::Unknown);
        assert_eq!(super::classify_start("not a pid"), super::RunStart::Unknown);
    }

    /// The fallback starts the program as a child, with the activation
    /// token in its environment, and answers a pid.
    #[test]
    fn the_fallback_starts_the_program_with_the_token() {
        let token = format!("tok-{}", std::process::id());
        let out = std::env::temp_dir().join(format!("chora-run-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&out);
        let script = format!("printf %s \"$XDG_ACTIVATION_TOKEN\" > {}", out.display());
        let program = ["sh".into(), "-c".into(), script];
        let pid = super::start_child(&program, &token).expect("the program starts");
        assert!(pid > 0);
        let deadline = Instant::now() + Duration::from_secs(5);
        while std::fs::read_to_string(&out).unwrap_or_default() != token {
            assert!(Instant::now() < deadline, "no token in {}", out.display());
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = std::fs::remove_file(&out);
    }

    /// Everything after `run` is the program.
    #[test]
    fn run_takes_the_rest_as_the_program() {
        let args = Args::try_parse_from("choractl run -- /bin/echo -n".split(' ')).unwrap();
        assert!(matches!(args.command, Cmd::Run { ref argv } if *argv == ["/bin/echo", "-n"]));
    }

    /// `restart` takes the plugin name.
    #[test]
    fn restart_takes_the_plugin_name() {
        let args = Args::try_parse_from("choractl restart tiling".split(' ')).unwrap();
        assert!(matches!(args.command, Cmd::Restart { ref plugin } if plugin == "tiling"));
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
