//! chora-tiling: a tiling plugin with the user experience of awesome. Each
//! output has tags, each tag a layout, and the bindings come from a YAML
//! file (see `tiling.yaml`). Other clients, for example an agent, switch
//! the layout and drive the windows through the exported `tiling.*`
//! methods (see `docs/configuration.md`).

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod commands;
mod config;
mod drag;
mod dress;
mod keys;
mod layout;
mod lifecycle;
mod looks;
mod methods;
mod overlay;
mod reload;
mod state;
mod theme;
mod windows;
mod wm;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
    process::ExitCode,
};

use chora_kit::{
    actions::{
        System,
        perform::{Failure, Host, perform},
    },
    conn::Conn,
    effects::{Driver, Target},
    give::{self, MOVE},
    notice,
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
    wm::{Book, give::pass_on, kept},
};
use chora_scene::{
    Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceRequest,
    ipc::{ClientError, PressState, Query, RpcError},
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use chora_kit::wm::focus::Focus;

use crate::{
    commands::{GROUP, answers, leaves_tiling, runtime_shader},
    config::{Action, Config, Own},
    layout::Layout,
    methods::exported_methods,
    wm::{Effect, OutputTags, Placement, WINDOW_KEY, Wm},
};

/// The client name, and the group that holds the windows.
const NAME: &str = "tiling";

/// Tile the windows of chora with tags and layouts in the style of awesome.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The configuration file. Without it, chora-tiling reads
    /// $XDG_CONFIG_HOME/chora/chora-tiling.yaml, by default in ~/.config,
    /// if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default configuration and exit.
    #[arg(long)]
    print_config: bool,
    /// The app id of windows to leave to other clients; repeatable.
    #[arg(long, value_name = "APP_ID")]
    ignore: Vec<String>,
    /// Manages only this output, by name, and leaves the others to other
    /// window managers; all outputs without it. Repeatable.
    #[arg(long = "output", value_name = "NAME")]
    outputs: Vec<String>,
    /// Accepted and ignored: the panel (waybar.sh) and the wallpaper
    /// (wallpaper.sh) of the flavor start by default, each once (not when
    /// its pid file names a running one).
    #[arg(long = "with-companions", hide = true, action = clap::ArgAction::Count)]
    _with_companions: u8,
    /// Starts no panel and no wallpaper, for a session or a gate that
    /// starts them itself.
    #[arg(long)]
    no_companions: bool,
    /// Write the files of the palette (the colours of the panel, the
    /// launcher, the terminal and the lock screen) and exit.
    #[arg(long)]
    apply_theme: bool,
    /// The directory that --apply-theme writes; by default `chora-tiling/
    /// theme` in the runtime directory.
    #[arg(long, value_name = "DIR", requires = "apply_theme")]
    theme_dir: Option<PathBuf>,
}

/// Parameters of `tiling.set_layout`; a missing field keeps its value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetLayout {
    layout: Option<Layout>,
    gap: Option<u16>,
}

/// Parameters of `tiling.view_tag`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewTag {
    tag: usize,
}

/// Parameters of `tiling.move_to_tag`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveToTag {
    surface: String,
    tag: usize,
}

/// Parameters of `tiling.focus` and `tiling.swap_master`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForSurface {
    surface: String,
}

/// The answer of `tiling.set_layout` and `tiling.get_layout`.
#[derive(Serialize)]
struct LayoutInfo {
    layout: Layout,
    /// The gap of the old look, as `set_layout` sets it.
    gap: u16,
    /// The gap to the edge of the work area that the tiles have.
    gap_outer: i32,
    /// The gap between two tiles that they have.
    gap_inner: i32,
    master_width: f64,
    masters: usize,
    columns: usize,
    /// The windows that show on the current output, in their order.
    windows: Vec<String>,
}

struct Tiling {
    rt: Runtime<Action>,
    wm: Wm,
    /// Which windows are tiling's (`kit::wm::book`).
    book: chora_kit::wm::Book,
    /// The configuration, as the file has it.
    settings: settings::File<Config>,
    /// The app ids of windows to leave to other clients.
    ignore: Vec<String>,
    /// What the last transaction set, so that the next one sends changes
    /// only.
    sent: HashMap<String, Placement>,
    order: Vec<String>,
    focus: Focus,
    /// The meta that the core has, by node: what a restart rebuilds from.
    saved: HashMap<String, BTreeMap<String, String>>,
    /// The claim that the core has (D55).
    claimed: Option<[f64; 4]>,
    /// The window that the binding of `drag` or a move of its client
    /// holds, and the pointer relative to the window.
    dragged: Option<(String, [f64; 2])>,
    /// The springs of a window that came with its drag from another
    /// window manager, until its first move bends it.
    springs: Option<(String, Value)>,
    /// The window effects of the setting `effects`.
    effects: Driver,
    /// Where the effects last bent each wobbling window, for a frame after
    /// the transaction state was forgotten.
    bent: HashMap<String, Target>,
    /// The meshes of a drag that the next transaction carries.
    meshes: Vec<Command>,
    /// The indicator of the volume keys.
    volume: chora_kit::volume::Mixer,
    /// The windows that have a group in the core, by surface id.
    groups: HashSet<String>,
    /// The windows that left and whose groups the next transaction removes.
    stale: Vec<String>,
    /// The path of the shader of the borders; none without a runtime
    /// directory, and the windows then have no border.
    border: Option<String>,
    /// The path of the shader of the shadows; none without a runtime
    /// directory.
    shadow: Option<String>,
    /// The first tag of the current output when the windows were last
    /// placed, to know the direction of a tag switch.
    last_tag: usize,
    /// What the panel was last told: the layout and the tags.
    bar: Option<(Option<Layout>, Vec<OutputTags>)>,
    /// The pid of the panel and whether it was told to hide for a
    /// fullscreen window.
    bar_hidden: Option<(u32, bool)>,
    /// Whether the next placement is a rebuild, not a change a person made:
    /// nothing moves.
    still: bool,
    /// The open shell surfaces: the key list, a sheet that holds the
    /// keyboard.
    open: chora_kit::wm::open::Open,
    /// The word of `state.get`; only reads.
    voiced: chora_kit::helper::Helper,
    /// When the layout chip leaves.
    chip_until: Option<std::time::Instant>,
    /// When the hint of an unbound chord showed last.
    hinted: Option<std::time::Instant>,
    /// The notice of the last failed reload, which the next one dismisses.
    reload_notice: Option<u32>,
    /// The colours of the look, and the scheme they come from.
    colors: theme::Colors,
    scheme: theme::Scheme,
    /// When the system is asked for its colour scheme again.
    theme_check: Option<std::time::Instant>,
    /// What the system asked for at the last answer.
    system: Option<theme::Scheme>,
    /// The question to the system that a thread is asking, and answers.
    theme_query: Option<std::sync::mpsc::Receiver<Option<theme::Scheme>>>,
    /// How the system is asked, in a thread of its own.
    ask_system: fn() -> Option<theme::Scheme>,
}

#[allow(clippy::print_stdout, reason = "--print-config prints the default")]
fn main() -> ExitCode {
    let words: Vec<String> = std::env::args().skip(1).collect();
    if words.first().is_some_and(|a| a == "--launch") {
        return launch(&words[1..]);
    }
    let args = Args::try_parse().unwrap_or_else(|err| {
        if err.kind() == clap::error::ErrorKind::UnknownArgument {
            eprintln!(
                "chora-tiling: --mod, --terminal, --launcher, --gap and --layout moved into \
                 the configuration file; chora-tiling --print-config prints it"
            );
        }
        err.exit()
    });
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    if args.apply_theme {
        return apply_theme(&args);
    }
    runtime::run(NAME, || play(&args))
}

/// Shows why a program does not start as a critical notice of
/// the notification server; when the core or the server is gone, the line on stderr
/// is the only message.
fn tell_launch_error(err: &theme::LaunchError) {
    let (title, text) = err.notice();
    tell_launch(title, text);
}

/// Shows a critical notice of a launch that could not start.
fn tell_launch(title: &str, text: &str) {
    let notice = notice::Notice {
        app: "chora-tiling",
        icon: "",
        summary: title,
        body: text,
        expire_ms: 0,
        urgency: notice::Urgency::Critical,
    };
    match launch_conn() {
        Ok(mut conn) => {
            notice::show_or_log(&mut conn, None, &notice);
        }
        Err(err) => eprintln!("chora-tiling: notice {title:?}: {err}"),
    }
}

/// The connection of a launch: it only shows one notice or asks tiling to
/// close the key list, and ends, so it is no plugin.
#[allow(
    clippy::disallowed_methods,
    reason = "a launch is no plugin: it only shows one notice or asks one thing and ends"
)]
fn launch_conn() -> Result<Conn, chora_scene::ipc::ClientError> {
    Conn::connect(&launch_name(std::process::id()))
}

/// The client name of a launch. It carries the process id: the word of the
/// bar asks every 2 seconds, and a click must not meet the name of a launch
/// that is still open.
fn launch_name(pid: u32) -> String {
    format!("chora-tiling.launch-{pid}")
}

/// Asks tiling to close the key list before the launcher or the power menu
/// opens, so that one overlay shows at a time; without a session or a tiling
/// that answers it does nothing.
fn close_key_list() {
    let Ok(mut conn) = launch_conn() else {
        return;
    };
    let call = json!({"client": NAME, "method": "hide_keys", "params": {}});
    let _ = conn.call_within::<_, Value>("methods.call", call, std::time::Duration::from_secs(1));
}

/// `--launch PROGRAM [ARGS]`: starts foot, fuzzel or swaylock with the
/// config file of the flavor, in the palette; another program as it is.
/// `fuzzel` without arguments is the Apps list: the installed programs
/// and those of the flavor, sorted, as [`chora_kit::launcher`] shows them.
fn launch(argv: &[String]) -> ExitCode {
    use std::os::unix::process::CommandExt as _;
    let Some((program, rest)) = argv.split_first() else {
        eprintln!("usage: chora-tiling --launch PROGRAM [ARGS]");
        return ExitCode::from(2);
    };
    if program == "looks-word" {
        return launch_looks_word();
    }
    let dirs = theme::config_dirs();
    let line = match theme::launch_line(program, rest, &dirs, theme::written_scheme(&dirs)) {
        Ok(line) => line,
        Err(err) => {
            tell_launch_error(&err);
            return cannot_start(program, &err);
        }
    };
    if program == "power" || program == "looks" || (program == "fuzzel" && rest.is_empty()) {
        close_key_list();
    }
    if program == "looks" {
        return launch_looks(&dirs);
    }
    if program == "fuzzel" && rest.is_empty() {
        let flavor = dirs.iter().find(|d| d.join("applications").is_dir());
        let helper = chora_kit::launcher::helper_name();
        chora_kit::launcher::run_ordered(
            &line,
            "chora-tiling --launch foot",
            flavor.cloned(),
            true,
            helper.as_deref(),
        );
        return ExitCode::SUCCESS;
    }
    let mut command = std::process::Command::new(&line[0]);
    command.args(&line[1..]);
    if program == "power" {
        command
            .env("CHORA_MENU", "chora-tiling --launch fuzzel")
            .env("CHORA_LOCKER", "chora-tiling --launch swaylock -f")
            .env("CHORA_SCREENSHOT", "chora-tiling --launch screenshot");
    }
    if program == "fuzzel" {
        let current = std::env::var("XDG_DATA_DIRS").ok();
        if let Some(data) = theme::data_dirs(&dirs, current.as_deref()) {
            command.env("XDG_DATA_DIRS", data);
        }
    }
    cannot_start(program, &command.exec())
}

/// Says on stderr why a program of a launch did not start.
fn cannot_start(program: &str, err: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("chora-tiling: cannot start {program}: {err}");
    ExitCode::FAILURE
}

/// `--launch looks-word`: prints the word of the Looks door as the JSON of
/// a custom module of waybar: "Looks" and the names of the other looks that
/// run, or "Looks" alone (also when the core does not answer).
fn launch_looks_word() -> ExitCode {
    use std::io::Write as _;
    let clients = launch_conn()
        .and_then(|mut conn| {
            conn.call_within::<_, Value>(
                "clients.list",
                Value::Null,
                std::time::Duration::from_secs(1),
            )
        })
        .unwrap_or(Value::Null);
    let text = looks::word(&looks::running(&clients));
    let said = writeln!(std::io::stdout(), "{}", json!({ "text": text }));
    if said.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// `--launch looks`: the menu of the Looks word of the bar. It lists the
/// other looks that run (the clients of the core) as the rows that move the
/// focused window there, under the prompt "Move this window to ", or says
/// that no other look runs; the row that the person chooses runs `give.sh`.
fn launch_looks(dirs: &[std::path::PathBuf]) -> ExitCode {
    use std::io::Write as _;
    use std::os::unix::process::CommandExt as _;
    let language = chora_kit::words::Language::of_session();
    let clients =
        launch_conn().and_then(|mut conn| conn.call::<_, Value>("clients.list", Value::Null));
    let clients = match clients {
        Ok(clients) => clients,
        Err(err) => {
            let (title, text) = looks::unreachable_notice(language);
            tell_launch(title, text);
            return cannot_start("looks", &err);
        }
    };
    let running = looks::running(&clients);
    let scheme = theme::written_scheme(dirs);
    let fuzzel = match theme::launch_line("fuzzel", &[], dirs, scheme) {
        Ok(line) => line,
        Err(err) => {
            tell_launch_error(&err);
            return ExitCode::FAILURE;
        }
    };
    let spawned = std::process::Command::new(&fuzzel[0])
        .args(&fuzzel[1..])
        .args(["--dmenu", "--prompt", looks::prompt(language)])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => return cannot_start("fuzzel", &err),
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(looks::rows(&running, language).join("\n").as_bytes());
    }
    let Ok(out) = child.wait_with_output() else {
        return ExitCode::FAILURE;
    };
    let line = String::from_utf8_lossy(&out.stdout);
    let Some(look) = looks::chosen(&running, language, &line) else {
        return ExitCode::SUCCESS;
    };
    match theme::launch_line("give", &[look.to_owned()], dirs, scheme) {
        Ok(give) => cannot_start(
            "give.sh",
            &std::process::Command::new(&give[0]).args(&give[1..]).exec(),
        ),
        Err(err) => {
            tell_launch_error(&err);
            ExitCode::FAILURE
        }
    }
}

/// `--apply-theme`: the files of the palette of the configuration.
fn apply_theme(args: &Args) -> ExitCode {
    let result = settings::File::<Config>::path(args.config.clone(), "chora-tiling")
        .and_then(|path| settings::File::<Config>::load(path, config::DEFAULT))
        .map_err(|e| e.clone())
        .and_then(|settings| {
            let look = &settings.value.look;
            let scheme = look.theme.scheme(system_for(look.theme));
            let dir = args.theme_dir.clone().or_else(theme::default_dir);
            let dir = dir.ok_or("no directory: give --theme-dir")?;
            theme::apply(&dir, &look.palette.over(scheme), scheme).map_err(|e| e.to_string())
        });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chora-tiling: {err}");
            ExitCode::FAILURE
        }
    }
}

/// What the system asks for, when the theme follows it.
fn system_for(theme: theme::Theme) -> Option<theme::Scheme> {
    (theme == theme::Theme::System)
        .then(theme::system_scheme)
        .flatten()
}

/// One connection to the core: window management until it goes away.
fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-tiling")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let rt = Runtime::start(NAME, args.outputs.clone(), &EVENTS, methods)?;
    let book = lifecycle::book(rt.filters());
    let settings_effects = settings.value.effects.clone();
    let mut tiling = Tiling {
        rt,
        wm: Wm::new(settings.value.clone()),
        book,
        settings,
        ignore: args.ignore.clone(),
        sent: HashMap::new(),
        order: Vec::new(),
        focus: Focus::default(),
        saved: HashMap::new(),
        claimed: None,
        dragged: None,
        springs: None,
        effects: Driver::new(&settings_effects),
        bent: HashMap::new(),
        meshes: Vec::new(),
        volume: chora_kit::volume::Mixer::new(NAME),
        groups: HashSet::new(),
        stale: Vec::new(),
        border: runtime_shader("border.frag", include_str!("../border.frag")),
        shadow: runtime_shader("shadow.frag", dress::SHADOW_SHADER),
        last_tag: 0,
        bar: None,
        bar_hidden: None,
        still: true,
        open: chora_kit::wm::open::Open::default(),
        voiced: chora_kit::helper::Helper::default(),
        chip_until: None,
        hinted: None,
        reload_notice: None,
        colors: theme::Colors::of(theme::Scheme::Ink),
        scheme: theme::Scheme::Ink,
        theme_check: None,
        system: None,
        theme_query: None,
        ask_system: theme::system_scheme,
    };
    tiling.system = system_for(tiling.settings.value.look.theme);
    tiling.apply_theme(false);
    tiling.start()?;
    if !args.no_companions {
        theme::start_companions();
    }
    tiling.still = false;
    Ok(serve(&mut tiling)?)
}

/// How often the system is asked for its colour scheme.
const THEME_CHECK: std::time::Duration = std::time::Duration::from_secs(5);

/// How often the loop looks for the answer of a question to the system.
const ANSWER_CHECK: std::time::Duration = std::time::Duration::from_millis(100);

/// The units of the panel and of the wallpaper of the flavor.
const PANEL_UNIT: &str = "chora-tiling-waybar.service";
const WALLPAPER_UNIT: &str = "chora-tiling-swaybg.service";

/// How long the layout chip shows.
const CHIP_TIME: std::time::Duration = std::time::Duration::from_millis(700);

/// How long the hint of an unbound chord shows on the chip: long enough for
/// slow eyes and for a look 4.3 s after the key.
const HINT_TIME: std::time::Duration = std::time::Duration::from_secs(8);

/// How long after a hint the next one may show.
const HINT_AGAIN: std::time::Duration = std::time::Duration::from_secs(10);

/// The time between two frames of the effects, 60 Hz.
const FRAME: std::time::Duration = std::time::Duration::from_micros(16_667);

/// Handles events; while an effect runs, a frame follows each 16 ms.
fn serve(tiling: &mut Tiling) -> Result<(), ClientError> {
    let mut next = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        let wake = tiling.wake(next);
        let event = if let Some(at) = wake {
            tiling
                .rt
                .conn
                .next_event_timeout(at.saturating_duration_since(now))?
        } else {
            Some(tiling.rt.conn.next_event()?)
        };
        if let Some(event) = event {
            runtime::dispatch(tiling, event)?;
            tiling.push_bar();
        }
        let now = std::time::Instant::now();
        if tiling.effects.animating() && now >= next {
            tiling.frame(now)?;
            next = now + FRAME;
        }
        tiling.end_chip(now)?;
        tiling.follow_system(now)?;
    }
}

/// The events that tiling handles.
const EVENTS: [&str; 22] = [
    "voice.state",
    "voice.say",
    "voice.remove",
    "client.connected",
    "client.disconnected",
    "assistant.ready",
    "assistant.chat",
    "surface.created",
    "surface.destroyed",
    "surface.detached",
    "surface.changed",
    "surface.requested",
    "node.requested",
    "output.changed",
    "output.removed",
    "pointer.entered",
    "pointer.pressed",
    "focus.changed",
    "focus.node",
    "input.button",
    "input.ungrabbed",
    "node.key",
];

/// Whether tiling acts on an event: not for a key, a press, a drag or a
/// release of `input.inject` unless it takes injected input as the
/// person's.
fn hears(event: &Event, accepts_injected: bool) -> bool {
    chora_kit::runtime::from_person(event, accepts_injected)
}

impl Plugin for Tiling {
    type Action = Action;

    fn runtime(&mut self) -> &mut Runtime<Action> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if !hears(&event, self.rt.accepts_injected()) {
            return Ok(());
        }
        if matches!(&event, Event::EffectsDropped { client } if client == NAME) {
            return self.effects_dropped();
        }
        self.handle(event)
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.run(method, params)
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn state(&self) -> Option<Value> {
        Some(self.state_value())
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.reconfigure()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

/// The whole output in scene coordinates, as the default camera shows it.
fn full_area(output: &OutputInfo) -> layout::Rect {
    let ([x, y], [w, h]) = (output.position, output.size);
    [x, y, w, h]
}

impl give::Windows for Tiling {
    fn conn(&mut self) -> &mut chora_kit::conn::Conn {
        &mut self.rt.conn
    }

    fn has(&self, surface: &str) -> bool {
        self.wm.has(surface)
    }

    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        Tiling::hand(self, surface, to)
    }

    fn hand_kept(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        Tiling::hand_kept(self, surface, to)
    }
}

/// What tiling does for the actions of every look that it has: a command,
/// a close, the volume, the key list and a give. The rest has no meaning
/// here, and the settings drop its bindings.
impl Host for Tiling {
    type Error = ClientError;

    fn volume(&mut self, change: chora_kit::volume::Change) -> Result<(), ClientError> {
        self.volume.change(change, true);
        Ok(())
    }

    fn keys(&mut self) -> Result<(), ClientError> {
        self.toggle_keys()
    }

    fn chat(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    fn run(&mut self, command: &str) -> Result<(), ClientError> {
        chora_kit::launch::run(command, self.rt.conn.activation_token().as_deref());
        Ok(())
    }

    fn close(&mut self) -> Result<(), ClientError> {
        let Some(surface) = self.wm.focused().map(String::from) else {
            return Ok(());
        };
        self.apply(vec![Command::Close { surface }])
    }

    fn failed(&mut self, failure: Failure) -> Result<(), ClientError> {
        match failure {
            Failure::NoScript(script) | Failure::NotStarted(script) => {
                eprintln!("chora-tiling: {script} did not start");
            }
            Failure::NotGiven { to, why } => eprintln!("chora-tiling: give to {to}: {why}"),
        }
        Ok(())
    }
}

impl Tiling {
    /// Whether `--output` names outputs and none of them is there: the
    /// plugin then leaves all windows to other window managers.
    fn without_outputs(&self) -> bool {
        self.book.released()
    }

    fn start(&mut self) -> Result<(), ClientError> {
        self.voiced.ask(&mut self.rt.conn);
        for output in self.rt.outputs()? {
            self.wm
                .set_output(&output.name, output.layout_area(), full_area(&output));
            let _ = self.book.set_output(&output.name);
        }
        // The nodes that the core kept over a restart with resume.
        let kept = self.query(&format!("owner:{NAME}"))?;
        if kept.iter().any(|n| n["id"] == NAME) {
            self.restore(&kept);
        } else {
            let group = Command::Create {
                id: NAME.into(),
                kind: NodeKind::Group,
                parent: ROOT.into(),
                props: PropsUpdate::default(),
            };
            self.call_apply(vec![group])?;
        }
        // Free surfaces, and those the core reserved for tiling, for example
        // during the grace time of a restart.
        for node in self.query("kind:surface")? {
            if let (Some(id), None) = (node["id"].as_str(), node["owner"].as_str()) {
                let changes = self.book.claim(id, node["to"].as_str(), None);
                self.settle(changes)?;
            }
        }
        // The relayout grabs when it sets the claim: with a claim, a
        // pointer binding can be shared with another window manager (D55).
        self.relayout()?;
        if self.claimed.is_none() {
            self.grab_all()?;
        }
        Ok(())
    }

    /// Rebuilds the model from the meta of the kept nodes
    /// (`kit::wm::kept`, which reads the keys of before too). A kept
    /// window without meta comes back as a new one.
    pub(crate) fn restore(&mut self, nodes: &[Value]) {
        let kept = kept::read(nodes, NAME, wm::legacy_key);
        let windows: Vec<(String, String)> = (kept.windows.into_iter())
            .map(|(id, mut meta)| (id, meta.remove(WINDOW_KEY).unwrap_or_default()))
            .collect();
        for node in nodes {
            let surface = node["id"].as_str().and_then(|id| id.strip_prefix(GROUP));
            self.groups.extend(surface.map(str::to_owned));
        }
        self.wm.restore(&kept.root, &windows);
        for (id, _) in &windows {
            self.book.keep(id);
            if !self.wm.has(id) {
                self.wm.add_window(id);
            }
        }
    }

    /// Grabs every binding of the configuration. A binding that another
    /// client holds only logs.
    #[allow(clippy::expect_used, reason = "the configuration checks its bindings")]
    fn grab_all(&mut self) -> Result<(), ClientError> {
        let bindings = self.wm.config.common.bindings.clone();
        self.rt
            .give_keys(Some(|to| Action::System(System::Give(to.into()))));
        self.rt
            .accept_injected(self.wm.config.common.test_accept_injected_keys);
        for (text, action) in bindings {
            let binding = self
                .wm
                .config
                .binding(&text)
                .expect("the configuration checks its bindings");
            self.rt.grab(binding, action)?;
        }
        self.rt.grab_give_keys()?;
        // The title bars of the clients drag as `drag` does.
        let moves = MOVE.parse().expect("move is a binding");
        self.rt.grab(moves, Action::System(System::Drag))?;
        self.rt
            .grab_swallowed(&self.wm.config.modifier, &Action::Look(Own::Swallow))?;
        Ok(())
    }

    /// Applies the settings and grabs their bindings.
    fn reconfigure(&mut self) -> Result<(), ClientError> {
        self.check_shaders()?;
        self.rt.start_grabs();
        self.apply_theme(true);
        if self.theme_check.is_some() {
            self.theme_check = Some(std::time::Instant::now());
        }
        self.wm.reconfigure(self.settings.value.clone());
        self.effects.set(&self.settings.value.effects);
        self.sent.clear();
        self.grab_all()?;
        self.rt.end_grabs()?;
        if self.keys_open() {
            self.hide_keys()?;
            self.toggle_keys()?;
        }
        self.still = true;
        let done = self.relayout();
        self.still = false;
        done
    }

    fn query(&mut self, selector: &str) -> Result<Vec<Value>, ClientError> {
        self.rt.conn.call("scene.query", Query::new(selector))
    }

    fn handle(&mut self, event: Event) -> Result<(), ClientError> {
        self.voiced.hear_asking(&event, &mut self.rt.conn);
        if matches!(&event, Event::Emitted { event, client, .. } if event == "assistant.ready" && client == "assistant")
        {
            self.clear_panel()?;
            return self.name_theme();
        }
        if matches!(&event, Event::ClientDisconnected { name } if name == "assistant") {
            return self.clear_panel();
        }
        if let Event::Emitted {
            event: name,
            client,
            data,
        } = &event
            && name == "assistant.chat"
            && client == "assistant"
        {
            return self.helper_panel(data);
        }
        match self.overlay_event(&event) {
            Some(done) => done,
            None => self.handle_scene(event),
        }
    }

    /// The open helper panel keeps its width: the tiles shrink beside it,
    /// and the width comes back when it closes.
    fn helper_panel(&mut self, data: &Value) -> Result<(), ClientError> {
        let rect: Option<[i32; 4]> = serde_json::from_value(data["rect"].clone())
            .ok()
            .filter(|_| data["open"] == true);
        let output = data["output"].as_str().unwrap_or_default();
        if rect.is_some() && self.keys_open() {
            self.hide_keys()?;
        }
        if self.wm.set_panel(output, rect) {
            self.relayout()?;
        }
        Ok(())
    }

    /// Gives the width of the helper panel back on every output.
    fn clear_panel(&mut self) -> Result<(), ClientError> {
        if self.wm.set_panel("", None) {
            self.relayout()?;
        }
        Ok(())
    }

    fn handle_scene(&mut self, event: Event) -> Result<(), ClientError> {
        if matches!(event, Event::FocusNode { node: None, .. }) && self.focus.node() == Some(NAME) {
            self.focus.drop_node();
        }
        if self.focus.hear(&event).is_some() || matches!(event, Event::FocusNode { .. }) {
            return Ok(());
        }
        match event {
            event if Book::hears(&event) => return self.lifecycle(&event),
            Event::SurfaceChanged {
                surface,
                size: Some(size),
                ..
            } => {
                if !self.wm.set_size(&surface, size) {
                    return Ok(());
                }
            }
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                .. // injected: the runtime drops an injected binding key unless it accepts them
            } => return self.act(&binding),
            Event::InputButton {
                binding,
                state,
                position,
                surface,
                grab,
                .. // injected: hears drops an injected event unless tiling accepts them
            } => {
                self.drag(&binding, state, position, surface);
                self.take_given_drag(grab);
            }
            // A window that follows the pointer.
            Event::InputMoved {
                binding, position, .. // injected: hears drops an injected event unless tiling accepts them
            } if self.drag_to(&binding, position) => {}
            Event::PointerEntered { node, moved, .. } => { // injected: hears drops an injected entry unless tiling accepts them
                if !self.pointer_focus(&node, false, moved) {
                    return Ok(());
                }
            }
            Event::PointerPressed { node, .. } => { // injected: hears drops an injected press unless tiling accepts them
                if !self.pointer_focus(&node, true, true) {
                    return Ok(());
                }
            }
            Event::SurfaceRequested { surface, request } => {
                return self.answer_surface(&surface, request);
            }
            Event::NodeRequested {
                node, action, args, ..
            } => {
                return self.answer_request(&node, &action, &args);
            }
            _ => return Ok(()),
        }
        self.relayout()
    }

    /// Answers a `surface.requested` of one of the windows.
    fn answer_surface(
        &mut self,
        surface: &str,
        request: SurfaceRequest,
    ) -> Result<(), ClientError> {
        if !self.wm.has(surface) {
            return Ok(());
        }
        if request == SurfaceRequest::Close {
            return self.apply(vec![Command::Close {
                surface: surface.into(),
            }]);
        }
        self.wm.request(surface, request);
        self.relayout()
    }

    /// Does what a request of another client asks of one of the windows
    /// (D31). Actions that `answers` does not know stay unanswered.
    fn answer_request(
        &mut self,
        node: &str,
        action: &str,
        args: &Value,
    ) -> Result<(), ClientError> {
        if !self.wm.has(node) {
            return Ok(());
        }
        let before = self.wm.focused().map(String::from);
        let Some(commands) = answers(&mut self.wm, node, action, args) else {
            return Ok(());
        };
        if self.wm.focused() != before.as_deref() {
            self.focus.take();
        }
        if action == "close" {
            return self.apply(commands);
        }
        if !commands.is_empty() {
            let applied = self.apply_report(commands)?;
            if leaves_tiling(applied, action, args) {
                self.forget_window(node, &chora_kit::wm::Why::Detached);
            }
        }
        self.relayout()
    }

    /// Runs an exported method. Everything but `get_layout` and `get_tags` changes
    /// something, relayouts like the matching binding, and answers with
    /// the layout info of the current tag. A failed relayout fails the
    /// method.
    fn run(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        if method == give::METHOD {
            return self.give(params);
        }
        if method == "get_tags" {
            return Ok(json!({"outputs": self.wm.tags()}));
        }
        if method == "show_helper" {
            Host::run(self, chora_kit::launcher::HELPER_PROGRAM)
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
            return Ok(Value::Null);
        }
        if method != "get_layout" {
            let before = self.wm.focused().map(String::from);
            let focuses = self.tune(method, params)?;
            // A method that moves the focus of tiling takes it back, as a
            // binding does.
            if focuses || self.wm.focused() != before.as_deref() {
                self.focus.take();
            }
            let shown = self.relayout().and_then(|()| {
                if method == "next_layout" {
                    self.show_chip()
                } else {
                    Ok(())
                }
            });
            shown.map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        let (layout, params) = self
            .wm
            .current()
            .ok_or_else(|| RpcError::new(RpcError::REJECTED, "no output"))?;
        let (gap_outer, gap_inner) = self.wm.config.gaps();
        let info = LayoutInfo {
            layout,
            gap: self.wm.config.gap,
            gap_outer,
            gap_inner,
            master_width: params.master_width,
            masters: params.masters,
            columns: params.columns,
            windows: self.wm.shown_windows(),
        };
        #[allow(clippy::expect_used, reason = "a layout has string keys only")]
        let value = serde_json::to_value(info).expect("layouts always serialize");
        Ok(value)
    }

    /// Changes what a method other than `window.give` and `get_layout` names.
    /// True when the method focuses a window on its own.
    fn tune(&mut self, method: &str, params: Value) -> Result<bool, RpcError> {
        Ok(match method {
            "show_keys" => {
                self.toggle_keys()
                    .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
                false
            }
            "hide_keys" => {
                self.hide_keys_if_open()
                    .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
                false
            }
            "set_layout" | "next_layout" => {
                self.change_layout(method, params)?;
                false
            }
            "view_tag" => {
                let ViewTag { tag } = runtime::params(params)?;
                self.check_tag(tag)?;
                self.wm.view_tag(tag);
                false
            }
            "move_to_tag" => {
                let MoveToTag { surface, tag } = runtime::params(params)?;
                self.check_tag(tag)?;
                only(self.wm.move_to_tag(&surface, tag), || {
                    format!("tiling has no window {surface}")
                })?;
                false
            }
            "focus" => {
                let ForSurface { surface } = runtime::params(params)?;
                only(self.wm.has(&surface), || {
                    format!("tiling has no window {surface}")
                })?;
                // A window that shows nowhere only gets the urgent mark.
                self.wm.focus(&surface);
                self.wm.focused() == Some(surface.as_str())
            }
            "focus_next" => {
                self.wm.focus_next();
                false
            }
            "focus_prev" => {
                self.wm.focus_prev();
                false
            }
            "swap_master" => {
                let ForSurface { surface } = runtime::params(params)?;
                only(self.wm.swap_master(&surface), || {
                    format!("{surface} shows nowhere")
                })?;
                false
            }
            other => {
                return Err(RpcError::new(
                    RpcError::METHOD_NOT_FOUND,
                    format!("tiling has no method {other}"),
                ));
            }
        })
    }

    /// The events of the key list: `Esc` and a click outside hide it, and
    /// the keys that it takes go nowhere else. `None` for any other event.
    fn overlay_event(&mut self, event: &Event) -> Option<Result<(), ClientError>> {
        match event {
            Event::NodeKey {
                key,
                state,
                node: _,
                code: _,
                modifiers: _,
                text: _,
                injected: _, // injected: hears drops an injected key unless tiling accepts them
            } => {
                let escape = self.keys_open() && key == "Escape" && *state == PressState::Pressed;
                Some(if escape { self.hide_keys() } else { Ok(()) })
            }
            Event::PointerPressed { node, .. } // injected: hears drops an injected press unless tiling accepts them
                if self.keys_open() && node.starts_with(keys::KEYS) =>
            {
                Some(if node.ends_with("scrim") {
                    self.hide_keys()
                } else {
                    Ok(())
                })
            }
            _ => None,
        }
    }

    /// `set_layout` and `next_layout`.
    fn change_layout(&mut self, method: &str, params: Value) -> Result<(), RpcError> {
        if method == "next_layout" {
            self.wm.act(&Own::NextLayout);
            return Ok(());
        }
        let change: SetLayout = runtime::params(params)?;
        if let Some(layout) = change.layout {
            self.wm.set_layout(layout);
        }
        if let Some(gap) = change.gap {
            self.wm.set_gap(gap);
        }
        Ok(())
    }

    /// A tag number of the configuration.
    fn check_tag(&self, tag: usize) -> Result<(), RpcError> {
        if (1..=self.wm.config.tags).contains(&tag) {
            return Ok(());
        }
        Err(RpcError::new(
            RpcError::INVALID_PARAMS,
            format!("no tag {tag}"),
        ))
    }

    /// The watchdog of the core removed the meshes: the effects end.
    #[allow(clippy::unnecessary_wraps, reason = "an event handler")]
    fn effects_dropped(&mut self) -> Result<(), ClientError> {
        self.effects.dropped();
        self.bent.clear();
        Ok(())
    }

    /// Moves the effects one frame forward. A window sits where the last
    /// transaction put it, else where the effects last bent it.
    fn frame(&mut self, now: std::time::Instant) -> Result<(), ClientError> {
        let (sent, wm, bent) = (&self.sent, &self.wm, &self.bent);
        let frame = self.effects.tick(now, |id| {
            let window = wm.windows.iter().find(|w| w.id == id)?;
            let kept = bent.get(id)?;
            let placement = sent.get(id);
            let origin = placement.and_then(|p| p.at);
            let size = placement.and_then(|p| p.size).or(window.size);
            Some(Target {
                node: id.into(),
                origin: origin.map_or(kept.origin, |at| at.map(f64::from)),
                size: size.map_or(kept.size, |size| size.map(f64::from)),
                set_size: false,
            })
        });
        let effects = &self.effects;
        self.bent.retain(|id, _| effects.wobbles(id));
        if frame.commands.is_empty() {
            return Ok(());
        }
        self.apply(frame.commands)
    }

    /// The topmost shown window at a point in scene coordinates.
    fn window_at(&self, [px, py]: [f64; 2]) -> Option<String> {
        let rect = |id: &String| {
            let placement = self.sent.get(id)?;
            let window = self.wm.windows.iter().find(|w| &w.id == id)?;
            let [x, y] = placement.at?;
            let [w, h] = placement.size.or(window.size)?;
            Some([x, y, w, h].map(f64::from))
        };
        let hit = |id: &&String| rect(id).is_some_and(|r| inside(r, [px, py]));
        self.order.iter().rev().find(hit).cloned()
    }

    /// Focuses a window under the pointer. While another window manager
    /// has the focus, the focused window of tiling takes it again.
    fn pointer_focus(&mut self, node: &str, clicked: bool, moved: bool) -> bool {
        let lost = self.focus.lost_to_other(NAME);
        let took = self.wm.pointer_focus(node, clicked, moved, lost);
        if took && lost {
            self.focus.reclaim();
        }
        took
    }

    fn act(&mut self, binding: &str) -> Result<(), ClientError> {
        let Some(action) = self.rt.action(binding).cloned() else {
            return Ok(());
        };
        match action {
            Action::System(system) => perform(self, &system),
            Action::Look(Own::Swallow) => self.hint_unbound(binding),
            Action::Look(own) => self.turn(&own),
        }
    }

    /// Does an action of tiling: a change of the windows, the layouts or
    /// the tags.
    fn turn(&mut self, action: &Own) -> Result<(), ClientError> {
        let before = self.wm.focused().map(String::from);
        let effect = self.wm.act(action);
        // A binding that moves the focus of tiling takes it back.
        if self.wm.focused() != before.as_deref() {
            self.focus.take();
        }
        let layout_key = matches!(action, Own::NextLayout | Own::PreviousLayout);
        match effect {
            Some(Effect::Relayout) if layout_key => {
                self.relayout()?;
                self.show_chip()
            }
            Some(Effect::Relayout) => self.relayout(),
            // The same path as the method `settings.reload`.
            Some(Effect::Reload) => {
                match runtime::reload_settings(self) {
                    Ok(_) => self.dismiss_reload_notice(),
                    Err(err) => {
                        eprintln!("chora-tiling: {}", err.message);
                        self.tell_reload_failed(&err.message);
                    }
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    /// Passes a surface that the core reserved for tiling on to other
    /// window managers while tiling has no output: it takes it and lets
    /// it go at once, without `to`.
    fn pass_on_reserved(&mut self, surface: &str) -> Result<(), ClientError> {
        let reserved = self
            .query(&format!("id:{surface}"))?
            .iter()
            .any(|node| node["to"] == NAME);
        if !reserved {
            return Ok(());
        }
        self.apply(pass_on(surface, NAME))
    }
}

/// Whether a rect `[x, y, w, h]` holds a point.
fn inside([x, y, w, h]: [f64; 4], [px, py]: [f64; 2]) -> bool {
    (x..x + w).contains(&px) && (y..y + h).contains(&py)
}

/// Fails the method unless the model accepted it.
fn only(ok: bool, err: impl FnOnce() -> String) -> Result<(), RpcError> {
    ok.then_some(())
        .ok_or_else(|| RpcError::new(RpcError::REJECTED, err()))
}

#[cfg(test)]
mod tests {
    use crate::{
        commands::{changes, styled, taken},
        dress::Dress,
        wm::{Mark, States},
    };
    use std::path::Path;

    use chora_kit::testing::{FakeCore, Log};

    use super::*;

    /// rtd3-1: the pid is the only input of the name, and one process has one
    /// pid, so two real connects of one machine cannot be made in a test; the
    /// pure name function is the guard against a constant name.
    #[test]
    fn two_launches_of_one_machine_connect_under_different_names() {
        let (a, b) = (crate::launch_name(1), crate::launch_name(2));
        assert_ne!(a, b);
        assert!(a.starts_with("chora-tiling.launch") && b.starts_with("chora-tiling.launch"));
    }

    /// A tiling on a fake core that answers every request, `input.grab`
    /// with its binding, and keeps the names of the methods called.
    pub(crate) fn on_fake_core(tag: &str) -> (Tiling, Log) {
        on_fake_core_with(tag, &[])
    }

    /// A tiling on a fake core as [`on_fake_core`], with the effects
    /// `effects`. The windows of these cells are not in the scene: the fake
    /// core takes the commands as they come, and the test reads them.
    fn on_fake_core_with(tag: &str, effects: &[chora_kit::effects::Kind]) -> (Tiling, Log) {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-tiling-{tag}")).keep();
        std::fs::create_dir_all(&dir).unwrap();
        let core = FakeCore::start();
        core.respond(|method, _| (method == "scene.apply").then_some(Value::Null));
        let methods = core.record(|method, params| Some(format!("{method} {params}")));
        (tiling_on(core.runtime(NAME), &dir, effects), methods)
    }

    /// A tiling on the runtime `rt`, with its settings file in `dir`.
    pub(crate) fn tiling_on(
        rt: Runtime<Action>,
        dir: &Path,
        effects: &[chora_kit::effects::Kind],
    ) -> Tiling {
        let file = dir.join("chora-tiling.yaml");
        let mut settings = settings::File::<Config>::load(file, config::DEFAULT).unwrap();
        settings.value.effects = effects.to_vec();
        Tiling {
            rt,
            wm: Wm::new(settings.value.clone()),
            book: crate::lifecycle::book(false),
            settings,
            ignore: Vec::new(),
            sent: HashMap::new(),
            order: Vec::new(),
            focus: Focus::default(),
            saved: HashMap::new(),
            claimed: None,
            dragged: None,
            springs: None,
            effects: Driver::new(effects),
            bent: HashMap::new(),
            meshes: Vec::new(),
            volume: chora_kit::volume::Mixer::new(NAME),
            groups: HashSet::new(),
            stale: Vec::new(),
            border: None,
            shadow: None,
            last_tag: 0,
            bar: None,
            bar_hidden: None,
            still: true,
            open: chora_kit::wm::open::Open::default(),
            voiced: chora_kit::helper::Helper::default(),
            chip_until: None,
            hinted: None,
            reload_notice: None,
            colors: theme::Colors::of(theme::Scheme::Ink),
            scheme: theme::Scheme::Ink,
            theme_check: None,
            system: None,
            theme_query: None,
            ask_system: || None,
        }
    }

    /// Presses on a window, moves the pointer through the points and lets
    /// go at the last.
    fn drag(tiling: &mut Tiling, surface: &str, points: &[[f64; 2]]) {
        let press = |state, position: [f64; 2]| Event::InputButton {
            binding: give::MOVE.into(),
            state,
            position,
            output: "X".into(),
            local: position,
            surface: Some(surface.into()),
            grab: None,
            injected: false,
        };
        let moved = |position: [f64; 2]| Event::InputMoved {
            binding: give::MOVE.into(),
            position,
            output: "X".into(),
            local: position,
            injected: false,
        };
        tiling.event(press(PressState::Pressed, points[0])).unwrap();
        for point in &points[1..] {
            tiling.event(moved(*point)).unwrap();
        }
        let last = points[points.len() - 1];
        tiling.event(press(PressState::Released, last)).unwrap();
    }

    /// Drags a floating window of 40 by 20 from its place to another and
    /// lets go; the transactions that tiling sent are the answer.
    fn drag_a_floating_window(tiling: &mut Tiling, methods: &Log) -> String {
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.book.keep("s1");
        tiling.wm.act(&Own::ToggleFloating);
        assert!(tiling.wm.set_size("s1", [40, 20]));
        tiling.relayout().unwrap();
        drag(
            tiling,
            "s1",
            &[[190.0, 150.0], [230.0, 170.0], [260.0, 190.0]],
        );
        methods.lock().unwrap().join("\n")
    }

    /// The commands of the transactions that tiling sent, in order.
    fn sent_commands(methods: &Log) -> Vec<Value> {
        let calls = methods.lock().unwrap().clone();
        let applies = calls.iter().filter_map(|m| m.strip_prefix("scene.apply "));
        applies
            .flat_map(|params| {
                let params: Value = serde_json::from_str(params).unwrap();
                params["commands"].as_array().cloned().unwrap_or_default()
            })
            .collect()
    }

    /// The updates of a node that set a mesh with points.
    fn bends(methods: &Log, node: &str) -> usize {
        let meshes = sent_commands(methods);
        let bent = |c: &&Value| {
            let update = &c["update"];
            update["id"] == node
                && update["props"]["mesh"]["points"]
                    .as_array()
                    .is_some_and(|p| !p.is_empty())
        };
        meshes.iter().filter(bent).count()
    }

    /// The updates of a node that remove its mesh.
    fn unbends(methods: &Log, node: &str) -> usize {
        let unmesh = |c: &&Value| {
            let update = &c["update"];
            update["id"] == node
                && update["props"]["mesh"].is_object()
                && update["props"]["mesh"]["points"]
                    .as_array()
                    .is_none_or(Vec::is_empty)
        };
        sent_commands(methods).iter().filter(unmesh).count()
    }

    fn settle(tiling: &mut Tiling) {
        let mut now = std::time::Instant::now();
        for _ in 0..600 {
            now += std::time::Duration::from_millis(16);
            tiling.frame(now).unwrap();
            if !tiling.effects.animating() {
                return;
            }
        }
    }

    const WOBBLY: [chora_kit::effects::Kind; 1] = [chora_kit::effects::Kind::Wobbly];

    #[test]
    fn get_layout_answers_the_gaps_in_use() {
        let (mut tiling, _) = on_fake_core("gaps");
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        let info = tiling.run("get_layout", json!({})).unwrap();
        assert_eq!(
            (info["gap_outer"].as_i64(), info["gap_inner"].as_i64()),
            (Some(12), Some(12))
        );
        let info = tiling.run("set_layout", json!({"gap": 3})).unwrap();
        assert_eq!(
            (info["gap_outer"].as_i64(), info["gap_inner"].as_i64()),
            (Some(3), Some(6))
        );
        assert_eq!(info["gap"], 3);
    }

    #[test]
    fn the_panel_hears_of_tags_and_layouts_only_when_they_change() {
        let (mut tiling, _) = on_fake_core("bar-changes");
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        assert!(tiling.bar_changed(), "the first state");
        assert!(!tiling.bar_changed(), "nothing changed");
        tiling.wm.add_window("w");
        assert!(tiling.bar_changed(), "the tag is occupied");
        assert!(!tiling.bar_changed());
        tiling.run("next_layout", json!({})).unwrap();
        assert!(tiling.bar_changed(), "a new layout");
        tiling.run("set_layout", json!({"gap": 5})).unwrap();
        assert!(!tiling.bar_changed(), "a gap is no concern of the panel");
    }

    #[test]
    fn the_loop_does_not_wait_for_the_system() {
        let (mut tiling, _) = on_fake_core("slow-system");
        tiling.ask_system = || {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            Some(theme::Scheme::Paper)
        };
        let now = std::time::Instant::now();
        tiling.theme_check = Some(now);
        tiling.follow_system(now).unwrap();
        assert!(now.elapsed() < std::time::Duration::from_millis(300));
        assert!(tiling.theme_query.is_some(), "the question is out");
        assert_eq!(tiling.system, None);
        assert!(
            tiling.theme_check.is_some_and(|at| at > now),
            "the loop looks again soon"
        );
        tiling.follow_system(std::time::Instant::now()).unwrap();
        assert!(tiling.theme_query.is_some(), "still no answer");
    }

    #[test]
    fn the_method_next_layout_goes_to_the_next_layout() {
        let (mut tiling, _) = on_fake_core("next-layout");
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        let first = tiling.run("get_layout", json!({})).unwrap();
        assert_eq!(first["layout"], "tile");
        let next = tiling.run("next_layout", json!({})).unwrap();
        assert_eq!(next["layout"], "tile_left");
    }

    #[test]
    fn a_tiling_window_wobbles_with_the_opt_in() {
        let (mut tiling, methods) = on_fake_core_with("wobble", &WOBBLY);
        drag_a_floating_window(&mut tiling, &methods);
        assert!(bends(&methods, "s1") > 0, "the dragged node bends");
        assert!(tiling.effects.animating(), "and shakes after the release");
    }

    #[test]
    fn the_frames_stop_when_the_window_settles() {
        let (mut tiling, methods) = on_fake_core_with("settles", &WOBBLY);
        drag_a_floating_window(&mut tiling, &methods);
        settle(&mut tiling);
        assert!(!tiling.effects.animating());
        assert_eq!(unbends(&methods, "s1"), 1, "one unmesh at the end");
    }

    #[test]
    fn a_rejected_transaction_in_a_wobble_leaves_no_mesh_behind() {
        let (mut tiling, methods) = on_fake_core_with("rejected", &WOBBLY);
        drag_a_floating_window(&mut tiling, &methods);
        tiling.forget();
        settle(&mut tiling);
        assert_eq!(unbends(&methods, "s1"), 1, "the window is flat again");
    }

    #[test]
    fn a_window_closed_in_a_wobble_ends_it() {
        let (mut tiling, methods) = on_fake_core_with("closed", &WOBBLY);
        drag_a_floating_window(&mut tiling, &methods);
        let gone = Event::SurfaceDestroyed {
            surface: "s1".into(),
        };
        tiling.event(gone).unwrap();
        assert!(!tiling.effects.animating());
        let before = sent_commands(&methods).len();
        tiling.frame(std::time::Instant::now()).unwrap();
        assert_eq!(sent_commands(&methods).len(), before, "nothing to send");
    }

    #[test]
    fn a_tiled_window_never_wobbles() {
        let (mut tiling, methods) = on_fake_core_with("tiled", &WOBBLY);
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.relayout().unwrap();
        drag(&mut tiling, "s1", &[[190.0, 150.0], [230.0, 170.0]]);
        assert!(!tiling.effects.animating());
        assert_eq!(bends(&methods, "s1"), 0);
    }

    #[test]
    fn a_window_loses_its_mesh_before_the_hand_over() {
        let (mut tiling, methods) = on_fake_core_with("hand", &WOBBLY);
        drag_a_floating_window(&mut tiling, &methods);
        tiling.hand("s1", "other").unwrap();
        let sent = sent_commands(&methods);
        let position = |wanted: &dyn Fn(&Value) -> bool| sent.iter().position(wanted).unwrap();
        let unmesh = position(&|c| {
            c["update"]["id"] == "s1"
                && c["update"]["props"]["mesh"]["points"]
                    .as_array()
                    .is_none_or(Vec::is_empty)
                && c["update"]["props"]["mesh"].is_object()
        });
        let give = position(&|c| c["remove"]["to"] == "other");
        assert!(unmesh < give, "the mesh goes before the window");
        assert!(!tiling.effects.animating());
    }

    #[test]
    fn the_watchdog_of_the_core_ends_the_wobble() {
        let (mut tiling, methods) = on_fake_core_with("dropped", &WOBBLY);
        drag_a_floating_window(&mut tiling, &methods);
        let dropped = Event::EffectsDropped {
            client: NAME.into(),
        };
        tiling.event(dropped).unwrap();
        assert!(!tiling.effects.animating());
    }

    #[test]
    fn a_new_press_lets_go_of_the_window_before() {
        let (mut tiling, methods) = on_fake_core_with("twice", &WOBBLY);
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        for id in ["s1", "s2"] {
            tiling.wm.add_window(id);
            tiling.wm.act(&Own::ToggleFloating);
            assert!(tiling.wm.set_size(id, [40, 20]));
        }
        tiling.relayout().unwrap();
        let press = |surface: &str, position: [f64; 2]| Event::InputButton {
            binding: give::MOVE.into(),
            state: PressState::Pressed,
            position,
            output: "X".into(),
            local: position,
            surface: Some(surface.into()),
            grab: None,
            injected: false,
        };
        let moved = Event::InputMoved {
            binding: give::MOVE.into(),
            position: [230.0, 170.0],
            output: "X".into(),
            local: [230.0, 170.0],
            injected: false,
        };
        tiling.event(press("s1", [190.0, 150.0])).unwrap();
        tiling.event(moved).unwrap();
        tiling.event(press("s2", [50.0, 50.0])).unwrap();
        settle(&mut tiling);
        assert!(bends(&methods, "s1") > 0);
        assert!(!tiling.effects.animating(), "s1 shakes to its end");
    }

    /// tiling with the floating window s1 of 40 by 20 on the output X.
    fn with_a_floating_window(tag: &str) -> (Tiling, Log) {
        let (mut tiling, methods) = on_fake_core_with(tag, &WOBBLY);
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.wm.act(&Own::ToggleFloating);
        assert!(tiling.wm.set_size("s1", [40, 20]));
        tiling.relayout().unwrap();
        (tiling, methods)
    }

    /// The press of `move` on s1, with the drag of another window manager
    /// when `grab` has one, and a move of the pointer.
    fn press_and_move(tiling: &mut Tiling, grab: Option<chora_scene::GivenGrab>) {
        let press = Event::InputButton {
            binding: give::MOVE.into(),
            state: PressState::Pressed,
            position: [190.0, 150.0],
            output: "X".into(),
            local: [190.0, 150.0],
            surface: Some("s1".into()),
            grab,
            injected: false,
        };
        let moved = Event::InputMoved {
            binding: give::MOVE.into(),
            position: [230.0, 170.0],
            output: "X".into(),
            local: [230.0, 170.0],
            injected: false,
        };
        tiling.event(press).unwrap();
        tiling.event(moved).unwrap();
    }

    /// E3: `window.give` with `grab` sends the remove and the `give_grab` in
    /// one transaction, with the springs of the wobble, and tiling drags
    /// no more.
    #[test]
    fn a_give_with_the_drag_sends_the_springs_and_ends_the_drag() {
        let (mut tiling, methods) = with_a_floating_window("give-drag");
        press_and_move(&mut tiling, None);
        let give = json!({"surface": "s1", "to": "xp", "grab": {"point": [5.0, 5.0]}});
        tiling.method(give::METHOD, give).unwrap();
        let sent = sent_commands(&methods);
        let at = (sent.iter().position(|c| c.get("give_grab").is_some())).expect("a give_grab");
        assert_eq!(sent[at - 1]["remove"], json!({"id": "s1", "to": "xp"}));
        let grab = &sent[at]["give_grab"];
        assert_eq!(grab["to"], "xp");
        assert_eq!(grab["point"], json!([5.0, 5.0]));
        let offsets = grab["wobble"]["offsets"].as_array();
        assert_eq!(offsets.map(Vec::len), Some(16), "{grab}");
        assert!(tiling.dragged.is_none());
    }

    /// I12 (WM5): the held window lies on top of the order of tiling while
    /// the person drags it, also below a window that was raised later.
    #[test]
    fn the_held_window_lies_on_top_while_it_is_dragged() {
        let (mut tiling, _methods) = with_a_floating_window("held-on-top");
        tiling.wm.add_window("s2");
        tiling.wm.act(&Own::ToggleFloating);
        assert!(tiling.wm.set_size("s2", [40, 20]));
        tiling.relayout().unwrap();
        assert_eq!(tiling.order.last().map(String::as_str), Some("s2"));
        press_and_move(&mut tiling, None);
        assert_eq!(
            tiling.dragged.as_ref().map(|(id, _)| id.as_str()),
            Some("s1")
        );
        tiling.relayout().unwrap();
        assert_eq!(
            tiling.order.last().map(String::as_str),
            Some("s1"),
            "{:?}",
            tiling.order
        );
    }

    /// I12 (WM5 r2): the press of a drag raises the window as a click does,
    /// so it stays on top after the release, also over a window that was
    /// raised later.
    #[test]
    fn a_dragged_window_stays_on_top_after_the_release() {
        let (mut tiling, _methods) = with_a_floating_window("held-release");
        tiling.wm.add_window("s2");
        tiling.wm.act(&Own::ToggleFloating);
        assert!(tiling.wm.set_size("s2", [40, 20]));
        tiling.relayout().unwrap();
        assert_eq!(tiling.order.last().map(String::as_str), Some("s2"));
        drag(&mut tiling, "s1", &[[190.0, 150.0], [230.0, 170.0]]);
        tiling.relayout().unwrap();
        assert!(tiling.dragged.is_none());
        assert_eq!(
            tiling.order.last().map(String::as_str),
            Some("s1"),
            "{:?}",
            tiling.order
        );
    }

    /// I12 (WM5 r2): `state.get` names the dragged window as `held`, as the
    /// acts read it, with a floating window under another one. The state
    /// goes to the file of `CHORA_STATE_DUMP` for the cell of the acts.
    #[test]
    fn state_get_names_the_dragged_window_as_held() {
        let (mut tiling, _methods) = with_a_floating_window("held-state");
        tiling.wm.add_window("s2");
        tiling.wm.act(&Own::ToggleFloating);
        assert!(tiling.wm.set_size("s2", [40, 20]));
        tiling.relayout().unwrap();
        press_and_move(&mut tiling, None);
        tiling.relayout().unwrap();
        let state = tiling.state_value();
        if let Some(path) = std::env::var_os("CHORA_STATE_DUMP") {
            std::fs::write(path, state.to_string()).unwrap();
        }
        let held: Vec<&str> = state["windows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|w| w["held"] == json!(true))
            .filter_map(|w| w["surface"].as_str())
            .collect();
        assert_eq!(held, ["s1"], "{state}");
    }

    /// E3: the press of `move` that comes with a window and its drag puts
    /// the named point of the window under the pointer, and the window
    /// wobbles on from the springs of the giver.
    #[test]
    fn a_given_drag_goes_on_at_its_point_with_its_springs() {
        let (mut tiling, methods) = with_a_floating_window("given-drag");
        let pulled = json!({"offsets": vec![[4.0, 0.0, 0.0]; 16], "velocity": vec![[0.0; 3]; 16]});
        let grab = chora_scene::GivenGrab {
            point: [7.0, 9.0],
            velocity: [0.0; 2],
            wobble: Some(pulled),
        };
        press_and_move(&mut tiling, Some(grab));
        assert_eq!(tiling.dragged, Some(("s1".into(), [7.0, 9.0])));
        let far = chora_scene::GivenGrab {
            point: [900.0, -5.0],
            velocity: [0.0; 2],
            wobble: None,
        };
        press_and_move(&mut tiling, Some(far));
        let held = tiling.dragged.clone().map(|(_, point)| point);
        assert_eq!(held, Some([40.0, 0.0]), "inside the window of 40 by 20");
        assert!(tiling.springs.is_none(), "the first move took the springs");
        assert!(bends(&methods, "s1") > 0);
    }

    #[test]
    fn a_manager_without_the_opt_in_sends_no_mesh() {
        let (mut tiling, methods) = on_fake_core("flat");
        let sent = drag_a_floating_window(&mut tiling, &methods);
        assert!(sent.contains("scene.apply"), "the window moved: {sent}");
        assert!(!sent.contains("\"mesh\""), "{sent}");
    }

    /// With `--output X` before X exists, the grab of a pointer binding
    /// that another window manager holds fails; tiling grabs again once
    /// its claim appears (2.7).
    #[test]
    fn tiling_grabs_again_when_its_claim_appears() {
        let (mut tiling, methods) = on_fake_core("regrab");
        let grabs = || {
            methods
                .lock()
                .unwrap()
                .iter()
                .filter(|m| m.starts_with("input.grab"))
                .count()
        };
        tiling.relayout().unwrap();
        assert_eq!(grabs(), 0, "no output, no claim");
        tiling.wm.set_output("X", [0, 0, 100, 60], [0, 0, 100, 80]);
        tiling.relayout().unwrap();
        assert!(tiling.claimed.is_some());
        assert!(grabs() > 0, "the claim came, and tiling grabbed");
        let before = grabs();
        tiling.relayout().unwrap();
        assert_eq!(grabs(), before, "only when the claim appears");
    }

    /// New settings and a new claim change the grabs. A binding that
    /// stays is never free in between, where its key would reach a
    /// window: tiling lets no binding go that it grabs again.
    #[test]
    fn a_change_of_the_grabs_keeps_the_bindings_that_stay() {
        let (mut tiling, methods) = on_fake_core("kept-grabs");
        let ungrabs = || {
            let seen = methods.lock().unwrap();
            let gone = seen.iter().filter(|m| m.starts_with("input.ungrab"));
            gone.cloned().collect::<Vec<_>>()
        };
        tiling.wm.set_output("X", [0, 0, 100, 60], [0, 0, 100, 80]);
        tiling.relayout().unwrap();
        assert!(tiling.rt.action("Super+Return").is_some());
        assert_eq!(ungrabs(), [""; 0], "the claim came");
        tiling.reconfigure().unwrap();
        assert_eq!(ungrabs(), [""; 0], "the same settings again");
        assert!(tiling.rt.action("Super+Return").is_some());
    }

    #[test]
    fn a_window_passed_on_is_free_for_any_window_manager() {
        assert!(matches!(
            &pass_on("s1", NAME)[..],
            [
                Command::Attach { surface, parent },
                Command::Remove { id, to: None },
            ] if surface == "s1" && parent == NAME && id == "s1"
        ));
    }

    #[test]
    fn a_move_drags_the_window_that_asked_for_it() {
        let under = || Some("s2".to_owned());
        let ours = |id: &str| id != "x";
        assert_eq!(taken(Some("s1".into()), ours, under).as_deref(), Some("s1"));
        assert_eq!(taken(Some("x".into()), ours, under), None);
        assert_eq!(taken(None, ours, under).as_deref(), Some("s2"));
    }

    #[test]
    fn a_relayout_takes_no_focus_from_another_window_manager() {
        let mut shown = Focus::default();
        assert!(shown.sends(Some("s3")));
        shown.sent(Some("s3".into()));
        assert!(!shown.sends(Some("s3")));
        // chora-xp takes the focus: a relayout, even with a new focus of
        // tiling, as after a window exits, sends nothing.
        shown.moved(Some("s1"));
        assert!(!shown.sends(Some("s3")));
        assert!(!shown.sends(Some("s4")));
        // The pointer on s3 takes it back, though tiling sent s3 before.
        shown.take();
        assert!(shown.sends(Some("s3")));
        shown.sent(Some("s3".into()));
        // The core confirms the focus of tiling.
        shown.moved(Some("s3"));
        assert!(!shown.lost && !shown.sends(Some("s3")));
    }

    /// The window that tiling hands away leaves the keyboard to its
    /// receiver: tiling sends no focus in the relayout that follows, which
    /// would empty or move the focus that the receiver set.
    #[test]
    fn a_handed_window_leaves_the_focus_to_its_receiver() {
        let (mut tiling, methods) = on_fake_core("handed");
        tiling.wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        tiling.wm.add_window("s1");
        tiling.relayout().unwrap();
        let before = methods.lock().unwrap().len();
        tiling.hand("s1", "xp").unwrap();
        let after = methods.lock().unwrap().split_off(before);
        assert!(
            !after.is_empty(),
            "the hand and the relayout reach the core"
        );
        assert!(
            after.iter().all(|call| !call.contains("\"focus\"")),
            "tiling sent a focus after the hand: {after:?}"
        );
    }

    /// A tiling on output A (0..100) beside xp, whose claim is `xp_claim`;
    /// the windows `ids` open on A, the last has the keyboard. The core
    /// refuses a give to `gone`.
    fn beside_xp(tag: &str, xp_claim: [f64; 4], ids: &[&str]) -> (Tiling, Log) {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-tiling-{tag}")).keep();
        std::fs::create_dir_all(&dir).unwrap();
        let core = FakeCore::start();
        core.respond(|method, params| {
            let refused = params.to_string().contains(r#""to":"gone""#);
            (method == "scene.apply" && !refused).then_some(Value::Null)
        });
        let xp = chora_scene::Node {
            id: "xp".into(),
            kind: chora_scene::NodeKind::Group,
            owner: Some("xp".into()),
            parent: Some(chora_scene::ROOT.into()),
            children: Vec::new(),
            props: chora_scene::Props {
                claim: Some(xp_claim),
                ..chora_scene::Props::default()
            },
            surface: None,
            to: None,
        };
        let claim = json!([xp]);
        let mut held = chora_scene::Node {
            kind: chora_scene::NodeKind::Surface,
            owner: Some(NAME.into()),
            props: chora_scene::Props::default(),
            ..xp
        };
        held.id = ids.last().copied().unwrap_or_default().into();
        let focused = json!([held]);
        core.respond(
            move |method, params| match (method, params["selector"].as_str()) {
                ("scene.query", Some("*")) => Some(claim.clone()),
                ("scene.query", Some("kind:surface")) => Some(focused.clone()),
                _ => None,
            },
        );
        let methods = core.record(|method, params| Some(format!("{method} {params}")));
        let mut tiling = tiling_on(core.runtime(NAME), &dir, &[]);
        tiling.wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        for id in ids {
            tiling.wm.add_window(id);
            tiling.book.keep(id);
        }
        tiling.relayout().unwrap();
        (tiling, methods)
    }

    /// The commands that tiling sent from the call `from` on.
    fn commands_from(methods: &Log, from: usize) -> Vec<Value> {
        let calls = methods.lock().unwrap().clone();
        (calls[from..].iter())
            .filter_map(|m| m.strip_prefix("scene.apply "))
            .flat_map(|params| {
                let params: Value = serde_json::from_str(params).unwrap();
                params["commands"].as_array().cloned().unwrap_or_default()
            })
            .collect()
    }

    /// `window.give` of `surface` to xp with `stay`, as `give.sh` of the
    /// Looks door calls it.
    fn stay(tiling: &mut Tiling, surface: &str) {
        let params = json!({"surface": surface, "to": "xp", "stay": true});
        tiling.give(params).unwrap();
    }

    /// The holder that `state.get` of tiling tells.
    fn holder(tiling: &Tiling) -> chora_kit::wm::state::Focus {
        tiling.focus.state(NAME, |s| tiling.wm.has(s))
    }

    /// D119, ti-57: a window that goes to xp on another output leaves the
    /// keyboard with tiling, where the person looks: the window focused
    /// before it gets it, before the window goes, so that xp sees another
    /// holder when the window comes.
    #[test]
    fn a_window_handed_to_a_look_on_another_output_leaves_the_keyboard_with_tiling() {
        let (mut tiling, methods) =
            beside_xp("keep-next", [100.0, 0.0, 100.0, 80.0], &["s1", "s2"]);
        assert_eq!(tiling.focus.surface.as_deref(), Some("s2"));
        let from = methods.lock().unwrap().len();
        stay(&mut tiling, "s2");
        let sent = commands_from(&methods, from);
        let at = |wanted: &dyn Fn(&Value) -> bool| sent.iter().position(wanted);
        let give =
            at(&|c| c["remove"]["id"] == "s2" && c["remove"]["to"] == "xp").expect("the give");
        let kept = at(&|c| c["focus"]["surface"] == "s1");
        assert!(
            kept.is_some_and(|k| k < give),
            "s1 takes the keyboard before s2 goes: {sent:?}"
        );
        let held = holder(&tiling);
        assert_eq!(
            (held.holder.as_deref(), held.surface.as_deref()),
            (Some(NAME), Some("s1")),
            "holder after the move"
        );
    }

    /// The last window of tiling goes to xp on another output: tiling
    /// holds the keyboard with its own desktop, the root of its nodes.
    #[test]
    fn the_last_window_handed_to_another_output_leaves_the_keyboard_with_the_desktop() {
        let (mut tiling, methods) = beside_xp("keep-desk", [100.0, 0.0, 100.0, 80.0], &["s1"]);
        let from = methods.lock().unwrap().len();
        stay(&mut tiling, "s1");
        let sent = commands_from(&methods, from);
        let at = |wanted: &dyn Fn(&Value) -> bool| sent.iter().position(wanted);
        let give = at(&|c| c["remove"]["id"] == "s1").expect("the give");
        let kept = at(&|c| c["focus_node"]["node"] == NAME);
        assert!(
            kept.is_some_and(|k| k < give),
            "the desktop takes the keyboard before s1 goes: {sent:?}"
        );
        let held = holder(&tiling);
        assert_eq!((held.holder.as_deref(), held.surface), (Some(NAME), None));
        tiling.toggle_keys().unwrap();
        let from = methods.lock().unwrap().len();
        tiling.hide_keys().unwrap();
        let back = commands_from(&methods, from);
        assert!(
            back.iter().any(|c| c["focus_node"]["node"] == NAME),
            "the key list gives the keyboard back to the desktop: {back:?}"
        );
        tiling
            .event(Event::FocusNode {
                node: None,
                owner: None,
            })
            .unwrap();
        assert_eq!(
            holder(&tiling).holder,
            None,
            "a click elsewhere took the keyboard from the desktop"
        );
    }

    /// rtf1-1: the give key Super+Alt+2 is a give of the person, as the
    /// row of the Looks door: the window goes to xp on another output and
    /// the keyboard stays with tiling, on the window focused before it.
    #[test]
    fn the_give_key_leaves_the_keyboard_with_tiling() {
        let (mut tiling, methods) = beside_xp("key-keep", [100.0, 0.0, 100.0, 80.0], &["s1", "s2"]);
        let from = methods.lock().unwrap().len();
        give::focused(&mut tiling, "xp").unwrap();
        let sent = commands_from(&methods, from);
        let at = |wanted: &dyn Fn(&Value) -> bool| sent.iter().position(wanted);
        let give = at(&|c| c["remove"]["id"] == "s2").expect("the give");
        let kept = at(&|c| c["focus"]["surface"] == "s1");
        assert!(
            kept.is_some_and(|k| k < give),
            "s1 takes the keyboard before s2 goes: {sent:?}"
        );
        let held = holder(&tiling);
        assert_eq!(
            (held.holder.as_deref(), held.surface.as_deref()),
            (Some(NAME), Some("s1")),
            "holder after the give key"
        );
    }

    /// rtf1-2: a window that xp gives to tiling while a window of xp or
    /// the desktop of xp holds the keyboard (the person moved it from a
    /// row of xp on another output) comes without the keyboard: tiling
    /// sends no focus for it, and the keyboard stays with xp.
    #[test]
    fn a_window_given_in_while_xp_holds_the_keyboard_comes_without_it() {
        let focus = |surface: Option<&str>| Event::FocusChanged {
            surface: surface.map(String::from),
        };
        let desk = Event::FocusNode {
            node: Some("xp".into()),
            owner: Some("xp".into()),
        };
        let xp_holds = [
            vec![focus(Some("x1"))],
            vec![focus(Some("x1")), focus(None), desk],
        ];
        for (n, held) in xp_holds.into_iter().enumerate() {
            let tag = format!("given-in-{n}");
            let (mut tiling, methods) = beside_xp(&tag, [100.0, 0.0, 100.0, 80.0], &["s1"]);
            for event in held {
                tiling.event(event).unwrap();
            }
            let from = methods.lock().unwrap().len();
            tiling
                .event(Event::SurfaceDetached {
                    surface: "s5".into(),
                })
                .unwrap();
            assert!(tiling.wm.has("s5"), "tiling took the window");
            let sent = commands_from(&methods, from);
            assert!(
                sent.iter().all(|c| c.get("focus").is_none()),
                "tiling focused a window given in while xp holds the keyboard: {sent:?}"
            );
            assert_eq!(holder(&tiling).holder, None, "tiling holds the keyboard");
        }
    }

    /// rtf1-4: a give of the person that the core refuses leaves the
    /// window with tiling, and it takes the keyboard back from the window
    /// that kept it.
    #[test]
    fn a_refused_give_of_the_person_gives_the_keyboard_back() {
        let (mut tiling, methods) =
            beside_xp("keep-back", [100.0, 0.0, 100.0, 80.0], &["s1", "s2"]);
        let from = methods.lock().unwrap().len();
        let params = json!({"surface": "s2", "to": "gone", "stay": true});
        assert!(tiling.give(params).is_err(), "the core refuses the give");
        let sent = commands_from(&methods, from);
        let focus: Vec<&Value> = sent.iter().filter_map(|c| c.get("focus")).collect();
        assert_eq!(
            focus.first().map(|f| &f["surface"]),
            Some(&json!("s1")),
            "{sent:?}"
        );
        assert_eq!(
            focus.last().map(|f| &f["surface"]),
            Some(&json!("s2")),
            "{sent:?}"
        );
        assert!(tiling.wm.has("s2"), "s2 stays with tiling");
        assert_eq!(holder(&tiling).surface.as_deref(), Some("s2"));
    }

    /// A give without `stay` (automation, the voice, MCP) leaves the
    /// keyboard to the receiver, also on another output (D55).
    #[test]
    fn a_give_without_stay_leaves_the_keyboard_to_the_receiver() {
        let (mut tiling, methods) = beside_xp("no-stay", [100.0, 0.0, 100.0, 80.0], &["s1", "s2"]);
        let from = methods.lock().unwrap().len();
        tiling.hand("s2", "xp").unwrap();
        let sent = commands_from(&methods, from);
        assert!(
            sent.iter()
                .all(|c| c.get("focus").is_none() && c.get("focus_node").is_none()),
            "tiling sent a focus: {sent:?}"
        );
    }

    /// A receiver whose claim lies on the output of the window gets the
    /// keyboard with it, as before: tiling sends no focus.
    #[test]
    fn a_window_handed_to_a_look_on_the_same_output_takes_the_keyboard_along() {
        let (mut tiling, methods) = beside_xp("follow", [20.0, 0.0, 60.0, 80.0], &["s1", "s2"]);
        let from = methods.lock().unwrap().len();
        stay(&mut tiling, "s2");
        let sent = commands_from(&methods, from);
        assert!(
            sent.iter()
                .all(|c| c.get("focus").is_none() && c.get("focus_node").is_none()),
            "tiling sent a focus: {sent:?}"
        );
    }

    #[test]
    fn a_focus_emptied_by_a_closed_window_is_no_loss() {
        let mut shown = Focus::default();
        shown.sent(Some("s3".into()));
        // The focused window of tiling exits: the core empties the focus.
        shown.moved(None);
        assert!(!shown.lost, "an empty focus is not another owner");
        assert!(shown.sends(Some("s4")), "the next window gets the focus");
        // Another window manager still takes it with its own window.
        shown.moved(Some("s9"));
        assert!(shown.lost && !shown.sends(Some("s4")));
    }

    /// The policy of tiling (`kit::wm::focus`): after the focused window
    /// goes, the window focused before it gets the keyboard.
    #[test]
    fn the_window_focused_before_gets_the_keyboard_after_a_close() {
        let (mut tiling, _) = on_fake_core("after");
        tiling.wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        tiling.wm.add_window("s1");
        tiling.wm.add_window("s2");
        tiling.relayout().unwrap();
        assert_eq!(tiling.focus.surface.as_deref(), Some("s2"));
        tiling.forget_window("s2", &chora_kit::wm::Why::Destroyed);
        assert_eq!(tiling.focus.surface.as_deref(), Some("s1"));
        assert!(tiling.focus.sends(Some("s1")), "the next relayout sends it");
    }

    /// A press on a window took the focus, and Escape hid the key list,
    /// from `input.inject` and without a person (review security a r2,
    /// MINOR 5).
    type Make = fn(bool) -> Event;

    #[test]
    fn an_injected_press_or_key_does_nothing_in_tiling() {
        let press = |injected| Event::PointerPressed {
            node: "s1".into(),
            button: 272,
            position: [0.0; 2],
            local: [0.0; 2],
            modifiers: Vec::new(),
            injected,
        };
        let escape = |injected| Event::NodeKey {
            node: "tiling-keys".into(),
            key: "Escape".into(),
            code: 1,
            state: PressState::Pressed,
            modifiers: Vec::new(),
            text: None,
            injected,
        };
        let makers: [(&str, Make); 2] = [("a press", press), ("Escape", escape)];
        for (name, make) in makers {
            assert!(hears(&make(false), false), "{name} of the person");
            assert!(!hears(&make(true), false), "{name} injected");
            assert!(hears(&make(true), true), "{name} injected in a test");
        }
    }

    /// An event as the core sends it, so that a test names the field
    /// `injected` of an event that the type does not carry yet.
    fn wire(event: Value) -> Event {
        serde_json::from_value(event).expect("an event of the core")
    }

    #[test]
    fn an_injected_pointer_move_does_not_move_the_focus_with_follow_mouse() {
        let (mut tiling, _methods) = on_fake_core("injected-follow");
        tiling.rt.pin_accepts_injected(false);
        assert_eq!(tiling.wm.config.focus, config::Focus::FollowMouse);
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.wm.add_window("s2");
        tiling.wm.focus("s1");
        let entered = |injected| {
            wire(
                json!({"event": "pointer.entered", "node": "s2", "moved": true,
                "injected": injected}),
            )
        };
        tiling.event(entered(true)).unwrap();
        assert_eq!(tiling.wm.focused(), Some("s1"), "an injected entry");
        tiling.event(entered(false)).unwrap();
        assert_eq!(tiling.wm.focused(), Some("s2"), "the person's entry");
        tiling.wm.focus("s1");
        tiling.rt.pin_accepts_injected(true);
        tiling.event(entered(true)).unwrap();
        assert_eq!(
            tiling.wm.focused(),
            Some("s2"),
            "an injected entry in a test"
        );
    }

    #[test]
    fn an_injected_press_and_an_injected_drag_move_nothing_in_tiling() {
        let (mut tiling, methods) = on_fake_core("injected-press");
        tiling.grab_all().unwrap();
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.wm.add_window("s2");
        tiling.wm.focus("s1");
        let send = |tiling: &mut Tiling, event: Value| {
            tiling.rt.pin_accepts_injected(false);
            tiling.event(wire(event)).unwrap();
        };
        let press = |injected| {
            json!({"event": "pointer.pressed", "node": "s2", "button": 272,
                "position": [10.0, 10.0], "injected": injected})
        };
        send(&mut tiling, press(true));
        assert_eq!(tiling.wm.focused(), Some("s1"), "an injected press");
        send(&mut tiling, press(false));
        assert_eq!(tiling.wm.focused(), Some("s2"), "the press of the person");
        let grab = |injected| {
            json!({"event": "input.button", "binding": give::MOVE, "state": "pressed",
                "position": [10.0, 10.0], "output": "X", "local": [10.0, 10.0],
                "surface": "s2", "injected": injected})
        };
        let moved = |injected| {
            json!({"event": "input.moved", "binding": give::MOVE, "position": [90.0, 90.0],
                "output": "X", "local": [90.0, 90.0], "injected": injected})
        };
        tiling.wm.act(&Own::ToggleFloating);
        tiling.relayout().unwrap();
        send(&mut tiling, grab(false));
        assert!(
            tiling.dragged.is_some(),
            "the press of the person starts a drag"
        );
        let before = methods.lock().unwrap().len();
        send(&mut tiling, moved(true));
        assert_eq!(methods.lock().unwrap().len(), before, "an injected move");
        send(&mut tiling, moved(false));
        assert!(
            methods.lock().unwrap().len() > before,
            "the move of the person"
        );
    }

    /// The words of the hint chips that tiling drew so far.
    fn hints_shown(methods: &Log) -> Vec<String> {
        let chips = sent_commands(methods).into_iter().map(|c| c.to_string());
        chips
            .filter(|c| c.contains("tiling-chip-word") && c.contains("does nothing here"))
            .collect()
    }

    /// A tiling with one output and its grabs, as a person starts it.
    fn tiling_with_grabs(tag: &str) -> (Tiling, Log) {
        let (mut tiling, methods) = on_fake_core(tag);
        tiling.grab_all().unwrap();
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        (tiling, methods)
    }

    #[test]
    fn an_unbound_chord_shows_the_hint_on_the_chip_ti_56() {
        let (mut tiling, methods) = tiling_with_grabs("stray-hint");
        tiling.act("Super+q").unwrap();
        let hints = hints_shown(&methods);
        assert_eq!(hints.len(), 1, "{hints:?}");
        assert!(
            hints[0].contains("Super+Q does nothing here. Super+F1 shows all keys."),
            "{hints:?}"
        );
        assert!(tiling.chip_until.is_some(), "the chip has a time to go");
    }

    /// rtd1-3: the hint names the help key that the person bound.
    #[test]
    fn the_hint_names_a_rebound_help_key_rtd1_3() {
        let (mut tiling, methods) = tiling_with_grabs("stray-rebound");
        let keys = Action::System(System::Keys);
        let config = &mut tiling.wm.config.common;
        config
            .bindings
            .retain(|text, action| *action != keys || text == "MOD+slash");
        tiling.act("Super+q").unwrap();
        let hints = hints_shown(&methods);
        assert_eq!(hints.len(), 1, "{hints:?}");
        assert!(hints[0].contains("Super+/ shows all keys."), "{hints:?}");
    }

    /// rtd1-3: with no key bound to the key list the hint does not name
    /// Super+F1, which would be a swallowed chord itself.
    #[test]
    fn the_hint_names_no_help_key_when_none_is_bound_rtd1_3() {
        let (mut tiling, methods) = tiling_with_grabs("stray-no-help");
        let keys = Action::System(System::Keys);
        tiling
            .wm
            .config
            .common
            .bindings
            .retain(|_, action| *action != keys);
        tiling.act("Super+q").unwrap();
        let hints = hints_shown(&methods);
        assert_eq!(hints.len(), 1, "{hints:?}");
        assert!(hints[0].contains("Super+Q does nothing here."), "{hints:?}");
        assert!(!hints[0].contains("Super+F1"), "{hints:?}");
        assert!(!hints[0].contains("shows all keys"), "{hints:?}");
    }

    #[test]
    fn the_hint_shows_7_seconds_after_the_key_and_is_gone_at_9_ti_56() {
        let (mut tiling, _methods) = tiling_with_grabs("stray-long");
        let key = std::time::Instant::now();
        tiling.act("Super+q").unwrap();
        tiling
            .end_chip(key + std::time::Duration::from_secs(7))
            .unwrap();
        assert!(
            tiling.chip_until.is_some(),
            "the hint is gone 7 s after the key"
        );
        tiling
            .end_chip(key + std::time::Duration::from_secs(9))
            .unwrap();
        assert!(
            tiling.chip_until.is_none(),
            "the hint still shows 9 s after the key"
        );
    }

    #[test]
    fn a_second_unbound_chord_within_ten_seconds_shows_no_hint_ti_56() {
        let (mut tiling, methods) = tiling_with_grabs("stray-twice");
        tiling.act("Super+q").unwrap();
        tiling.act("Super+x").unwrap();
        assert_eq!(hints_shown(&methods).len(), 1, "once in ten seconds");
    }

    #[test]
    fn an_unbound_chord_shows_no_hint_while_the_key_list_shows_ti_56() {
        let (mut tiling, methods) = tiling_with_grabs("stray-keys");
        tiling.toggle_keys().unwrap();
        assert!(tiling.keys_open());
        tiling.act("Super+q").unwrap();
        assert_eq!(hints_shown(&methods), Vec::<String>::new());
    }

    /// rth-4: with no notification server a failed reload still shows its
    /// reason, on the chip of tiling.
    #[test]
    fn a_failed_reload_without_a_notice_server_shows_the_reason_on_the_chip() {
        let (mut tiling, methods) = tiling_with_grabs("reload-chip");
        tiling.tell_reload_failed("/x/chora-tiling.yaml: simple key expected ':' at line 24");
        let chips: Vec<String> = sent_commands(&methods)
            .iter()
            .map(ToString::to_string)
            .filter(|c| c.contains("tiling-chip-word") && c.contains("chora-tiling.yaml"))
            .collect();
        assert_eq!(chips.len(), 1, "{chips:?}");
        assert!(chips[0].contains("Settings not applied"), "{chips:?}");
        assert!(tiling.chip_until.is_some(), "the chip has a time to go");
    }

    #[test]
    fn an_injected_button_grab_drags_no_window_in_tiling() {
        let (mut tiling, _methods) = on_fake_core("injected-grab");
        tiling.grab_all().unwrap();
        tiling.rt.pin_accepts_injected(false);
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        let press = |injected| {
            wire(
                json!({"event": "input.button", "binding": give::MOVE, "state": "pressed",
                "position": [10.0, 10.0], "output": "X", "local": [10.0, 10.0],
                "surface": "s1", "injected": injected}),
            )
        };
        let send = |tiling: &mut Tiling, accepts, injected| {
            tiling.rt.pin_accepts_injected(accepts);
            tiling.event(press(injected)).unwrap();
        };
        send(&mut tiling, false, false);
        assert!(tiling.dragged.is_some(), "the person's press");
        tiling.dragged = None;
        send(&mut tiling, false, true);
        assert_eq!(tiling.dragged, None, "an injected press");
        send(&mut tiling, true, true);
        assert!(tiling.dragged.is_some(), "an injected press in a test");
    }

    fn panel_data(open: bool, rect: &Value) -> Event {
        Event::Emitted {
            event: "assistant.chat".into(),
            client: "assistant".into(),
            data: serde_json::json!({"output": "A", "open": open, "rect": rect}),
        }
    }

    /// A tiling with one output whose panel is open.
    fn with_open_panel(tag: &str) -> Tiling {
        let (mut tiling, _) = on_fake_core_with(tag, &[]);
        tiling.wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        tiling
            .handle(panel_data(true, &serde_json::json!([60, 0, 40, 60])))
            .unwrap();
        tiling
    }

    #[test]
    fn a_null_rect_or_a_closed_chat_gives_the_panel_back() {
        let mut tiling = with_open_panel("panel-null");
        assert!(!tiling.wm.set_panel("A", Some([60, 0, 40, 60])), "open");
        tiling.handle(panel_data(true, &Value::Null)).unwrap();
        assert!(!tiling.wm.set_panel("A", None), "null rect cleared it");
        tiling
            .handle(panel_data(true, &serde_json::json!([60, 0, 40, 60])))
            .unwrap();
        tiling
            .handle(panel_data(false, &serde_json::json!([60, 0, 40, 60])))
            .unwrap();
        assert!(!tiling.wm.set_panel("A", None), "open false cleared it");
    }

    #[test]
    fn the_panel_goes_with_the_assistant() {
        let gone = Event::ClientDisconnected {
            name: "assistant".into(),
        };
        let ready = Event::Emitted {
            event: "assistant.ready".into(),
            client: "assistant".into(),
            data: Value::Null,
        };
        for (tag, event) in [("panel-gone", gone), ("panel-ready", ready)] {
            let mut tiling = with_open_panel(tag);
            tiling.handle(event).unwrap();
            assert!(!tiling.wm.set_panel("A", None), "{tag}: still reserved");
        }
    }

    fn tiler(windows: &[&str]) -> Wm {
        let mut wm = Wm::new(settings::parse(crate::config::DEFAULT).unwrap());
        wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        for id in windows {
            wm.add_window(id);
        }
        wm
    }

    #[test]
    fn tiling_answers_a_requested_focus_with_the_model() {
        let mut tiler = tiler(&["s1", "s2"]);
        let commands = answers(&mut tiler, "s1", "focus", &Value::Null).expect("focus answers");
        assert_eq!(commands, Vec::<Command>::new());
        assert_eq!(tiler.focused(), Some("s1"));
    }

    #[test]
    fn tiling_answers_a_requested_move_and_close() {
        let mut tiler = tiler(&["s1"]);
        let args = serde_json::json!({"parent": "other", "index": 2});
        let commands = answers(&mut tiler, "s1", "move", &args).expect("move answers");
        assert!(
            matches!(&commands[..], [Command::Move { id, parent, index }]
                if id == "s1" && parent.as_deref() == Some("other") && *index == Some(2))
        );
        let closed = answers(&mut tiler, "s1", "close", &Value::Null).expect("close answers");
        assert!(
            matches!(&closed[..], [Command::Close { surface }] if surface == "s1"),
            "{closed:?}"
        );
    }

    #[test]
    fn a_window_leaves_tiling_only_after_an_applied_move() {
        let to = |parent| serde_json::json!({"parent": parent});
        assert!(leaves_tiling(true, "move", &to("other")));
        assert!(
            !leaves_tiling(true, "move", &to(NAME)),
            "a move inside tiling keeps the window"
        );
        assert!(
            !leaves_tiling(false, "move", &to("other")),
            "a rejected move keeps the window managed"
        );
        assert!(!leaves_tiling(true, "focus", &to("other")));
    }

    #[test]
    fn a_new_window_gets_the_style_of_the_settings() {
        let mut style = settings::parse::<Config>(config::DEFAULT).unwrap().style;
        style.opacity = 0.5;
        assert!(matches!(
            styled(&style, "s1"),
            Command::Update { id, props }
                if id == "s1" && props.opacity == Some(0.5) && props.effects == Some(Vec::new())
        ));
    }

    /// The book passes a requested look through (D58, cells in
    /// lifecycle.rs); a relayout sends no look to the window, so it keeps it
    /// (the group of the window carries the opacity 1 of a jump and is not
    /// the window).
    #[test]
    fn a_requested_look_stays_across_a_relayout() {
        let mut tiler = tiler(&["s1"]);
        let args = serde_json::json!({"opacity": 0.5});
        assert!(answers(&mut tiler, "s1", "update", &args).is_none());
        let placed = Placement {
            id: "s1".into(),
            at: Some([0, 0]),
            size: Some([10, 10]),
            states: States::default(),
            radius: 0,
            frame: [10, 10],
            border: 0,
            mark: Mark::Line,
        };
        assert!(
            changes(None, &placed, &Dress::default())
                .iter()
                .all(|c| !matches!(c,
            Command::Update { id, props }
                if id == "s1" && (props.opacity.is_some() || props.effects.is_some())))
        );
    }

    #[test]
    fn a_requested_resize_stays_unanswered() {
        let mut tiler = tiler(&["s1"]);
        assert!(answers(&mut tiler, "s1", "resize", &Value::Null).is_none());
    }

    #[test]
    fn a_window_new_to_the_core_hides_too() {
        let hidden = Placement {
            id: "s1".into(),
            at: None,
            size: None,
            states: States::default(),
            radius: 0,
            frame: [10, 10],
            border: 0,
            mark: Mark::Line,
        };
        let commands = changes(None, &hidden, &Dress::default());
        assert!(matches!(
            &commands[..],
            [Command::Update { props, .. }] if props.visible == Some(false)
        ));
        assert_eq!(changes(Some(&hidden), &hidden, &Dress::default()).len(), 0);
    }

    #[test]
    fn a_reshape_during_the_focus_fade_ends_it_at_the_mark() {
        let colors = [[0.5; 3]; 4];
        let dress = Dress {
            colors: Some(&colors),
            shadows: true,
            animate: true,
            slide: 0.0,
        };
        let focus = Placement {
            id: "s1".into(),
            at: Some([0, 0]),
            size: Some([100, 80]),
            states: States::default(),
            radius: 10,
            frame: [100, 80],
            border: 2,
            mark: Mark::Focus,
        };
        let line = Placement {
            mark: Mark::Line,
            ..focus.clone()
        };
        let reshaped = Placement {
            size: Some([120, 80]),
            frame: [120, 80],
            ..line.clone()
        };
        let fading = changes(Some(&focus), &line, &dress);
        assert!(
            fading
                .iter()
                .any(|c| matches!(c, Command::Animate { id, .. } if id == "tg-s1"))
        );
        let commands = changes(Some(&line), &reshaped, &dress);
        for node in ["tg-s1", "tf-s1"] {
            let last = commands.iter().rev().find_map(|c| match c {
                Command::Update { id, props } | Command::Animate { id, props, .. }
                    if id == node =>
                {
                    Some(props)
                }
                _ => None,
            });
            assert_eq!(
                last.and_then(|p| p.opacity),
                Some(0.0),
                "{node}: {commands:?}"
            );
        }
    }
}

/// The scripts of the panel in `flavors/tiling/waybar`, run with stand-ins
/// of `wpctl`, `choractl` and `waybar` on a PATH of their own: the tools
/// that a script needs are links there, so that a missing one is missing.
#[cfg(test)]
mod panel {
    use std::{
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        process::{Command, Output, Stdio},
        time::{Duration, Instant},
    };

    /// The tools of the scripts, linked into the PATH of a test.
    const TOOLS: [&str; 10] = [
        "sh", "awk", "timeout", "sleep", "cat", "kill", "dirname", "mkdir", "cp", "date",
    ];

    fn waybar_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling/waybar")
    }

    /// Whether jq is installed; the panel scripts need it.
    fn has_jq() -> bool {
        let found = Command::new("sh")
            .args(["-c", "command -v jq"])
            .output()
            .is_ok_and(|o| o.status.success());
        if !found {
            eprintln!("skip: jq is not installed, and the panel scripts need it");
        }
        found
    }

    /// A directory with the tools, jq when `jq`, and the stand-ins
    /// `(name, shell code)`.
    fn bin(name: &str, jq: bool, fakes: &[(&str, &str)]) -> chora_kit::testing::TempDir {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-panel-{name}"));
        std::fs::create_dir_all(&dir).expect("a directory");
        let tools = TOOLS.iter().copied().chain(jq.then_some("jq"));
        for tool in tools {
            let found = Command::new("sh")
                .args(["-c", &format!("command -v {tool}")])
                .output()
                .expect("sh runs");
            let path = String::from_utf8_lossy(&found.stdout).trim().to_owned();
            if path.starts_with('/') {
                std::os::unix::fs::symlink(path, dir.join(tool)).expect("a link");
            }
        }
        for (fake, code) in fakes {
            let path = dir.join(fake);
            std::fs::write(&path, format!("#!/bin/sh\n{code}\n")).expect("a stand-in");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("mode");
        }
        dir
    }

    fn command(bin: &Path, script: &str, args: &[&str], lang: &str) -> Command {
        let mut command = Command::new(bin.join("sh"));
        command
            .arg(waybar_dir().join(script))
            .args(args)
            .env_clear()
            .env("PATH", bin)
            .env("LANG", lang)
            .env(chora_scene::paths::RUNTIME_ENV, bin);
        command
    }

    fn run(bin: &Path, script: &str, args: &[&str], lang: &str) -> Output {
        command(bin, script, args, lang).output().expect("sh runs")
    }

    fn printed(out: &Output) -> String {
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    }

    /// What volume.sh prints with a stand-in of wpctl, or without one.
    fn volume(name: &str, wpctl: Option<&str>, lang: &str) -> String {
        let fakes: Vec<(&str, &str)> = wpctl.map(|code| ("wpctl", code)).into_iter().collect();
        printed(&run(&bin(name, false, &fakes), "volume.sh", &[], lang))
    }

    #[test]
    fn the_volume_always_has_honest_words() {
        let (en, de) = ("C.UTF-8", "de_DE.UTF-8");
        let muted = Some("echo 'Volume: 0.40 [MUTED]'");
        let none = Some("exit 1");
        let unavailable = "No sound";
        let (lautlos, keiner) = ("Kein Ton", "Kein Ausgabegerät");
        for (name, wpctl, lang, words) in [
            ("loud", Some("echo 'Volume: 0.40'"), en, ["Volume 40%"; 2]),
            ("muted", muted, en, ["Muted"; 2]),
            ("muted-de", muted, de, ["Stumm"; 2]),
            ("missing", None, en, [unavailable, "wpctl not found"]),
            ("missing-de", None, de, [lautlos, "wpctl nicht gefunden"]),
            ("none", none, en, [unavailable, "No output device"]),
            ("none-de", none, de, [lautlos, keiner]),
        ] {
            assert_eq!(said(&volume(name, wpctl, lang)), words.map(str::to_owned));
        }
    }

    /// The text and the tooltip of the JSON of a custom module.
    fn said(json: &str) -> [String; 2] {
        let value: serde_json::Value = serde_json::from_str(json).expect(json);
        ["text", "tooltip"].map(|key| value[key].as_str().unwrap_or_default().to_owned())
    }

    #[test]
    fn a_hung_wpctl_gets_no_answer_within_2_seconds() {
        let start = Instant::now();
        assert_eq!(
            said(&volume("hung", Some("sleep 10"), "C")),
            ["No sound", "No answer"].map(str::to_owned)
        );
        let took = start.elapsed();
        assert!(took < Duration::from_secs(5), "{took:?}");
    }

    #[test]
    fn the_panel_labels_the_volume() {
        let config = std::fs::read_to_string(waybar_dir().join("config.jsonc")).expect("config");
        assert!(config.contains("\"custom/volume\""), "{config}");
        assert!(
            !config.contains("\"pulseaudio\""),
            "the unlabelled module is gone"
        );
    }

    #[test]
    fn the_panel_shows_no_internal_text() {
        let config = std::fs::read_to_string(waybar_dir().join("config.jsonc")).expect("config");
        for internal in ["\"cpu\"", "\"memory\"", "{ipaddr}\","] {
            assert!(!config.contains(internal), "{internal} in {config}");
        }
    }

    #[test]
    fn the_layout_word_is_a_word_of_a_person() {
        if !has_jq() {
            return;
        }
        let said = |layout: &str, lang: &str| {
            let fake = format!("echo '{{\"layout\": \"{layout}\"}}'");
            let bin = bin("layout", true, &[("choractl", &fake)]);
            said(&printed(&run(&bin, "layout.sh", &[], lang)))
        };
        let [word, tip] = said("fair_horizontal", "C");
        assert_eq!(word, "Fair rows");
        assert!(tip.contains("Super+Shift+Space"), "{tip}");
        assert_eq!(said("tile", "de_DE.UTF-8")[0], "Kacheln");
        assert_eq!(said("nonsense", "C")[0], "");
    }

    /// The stand-in of choractl: tiling has the outputs in the file
    /// `tiling` of its directory, the core those in `outputs`.
    const CHORACTL: &str = r#"dir=$(dirname "$0")
case "$1 $2 $3" in
"call tiling get_tags") cat "$dir/tiling" ;;
"outputs  ") cat "$dir/outputs" ;;
*) exit 1 ;;
esac"#;

    fn outputs(bin: &Path, tiling: &[&str], all: &[&str]) {
        let list = |names: &[&str]| {
            let names: Vec<String> = names
                .iter()
                .map(|n| format!("{{\"name\": \"{n}\"}}"))
                .collect();
            names.join(", ")
        };
        let tags = format!("{{\"outputs\": [{}]}}", list(tiling));
        std::fs::write(bin.join("tiling"), tags).expect("tiling");
        std::fs::write(bin.join("outputs"), format!("[{}]", list(all))).expect("outputs");
    }

    #[test]
    fn the_panel_shows_only_on_the_outputs_of_tiling() {
        if !has_jq() {
            return;
        }
        let waybar = r#"while [ $# -gt 0 ]; do [ "$1" = --config ] && cat "$2"; shift; done"#;
        let bin = bin(
            "outputs",
            true,
            &[("choractl", CHORACTL), ("waybar", waybar)],
        );
        outputs(&bin, &["HEADLESS-1"], &["HEADLESS-1", "HEADLESS-2"]);
        let dir = waybar_dir();
        let out = run(&bin, "waybar.sh", &[dir.to_str().expect("a path")], "C");
        let config = printed(&out);
        let config: serde_json::Value = serde_json::from_str(&config).expect(&config);
        assert_eq!(config["output"], serde_json::json!(["!HEADLESS-2", "*"]));
        let include = dir.join("config.jsonc");
        assert_eq!(config["include"], serde_json::json!([include.to_str()]));
    }

    /// When the outputs change, waybar starts again with the outputs of
    /// tiling of then.
    #[test]
    fn the_panel_follows_a_change_of_the_outputs() {
        if !has_jq() {
            return;
        }
        let waybar = r#"while [ $# -gt 0 ]; do [ "$1" = --config ] && cat "$2" >>"$(dirname "$0")/seen" && echo >>"$(dirname "$0")/seen"; shift; done
exec sleep 30"#;
        let bin = bin(
            "follow",
            true,
            &[("choractl", CHORACTL), ("waybar", waybar)],
        );
        outputs(&bin, &["A"], &["A", "B"]);
        let dir = waybar_dir();
        let mut script = command(&bin, "waybar.sh", &[dir.to_str().expect("a path")], "C")
            .stdout(Stdio::null())
            .spawn()
            .expect("the script starts");
        let seen = |bin: &Path| std::fs::read_to_string(bin.join("seen")).unwrap_or_default();
        let wait_for = |bin: &Path, text: &str| {
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(10) && !seen(bin).contains(text) {
                std::thread::sleep(Duration::from_millis(100));
            }
            seen(bin).contains(text)
        };
        assert!(wait_for(&bin, "\"!B\""), "{}", seen(&bin));
        outputs(&bin, &["A"], &["A", "B", "C"]);
        let followed = wait_for(&bin, "\"!C\"");
        let _ = Command::new("kill").arg(script.id().to_string()).status();
        let _ = script.wait();
        assert!(followed, "{}", seen(&bin));
        assert_eq!(seen(&bin).matches("\"!B\"").count(), 2, "{}", seen(&bin));
    }

    #[test]
    fn the_signal_traps_stand_before_the_pids_are_written() {
        let flavor = waybar_dir().join("..");
        for (script, pid) in [
            ("waybar/waybar.sh", "panel.pid"),
            ("wallpaper.sh", "wallpaper.pid"),
        ] {
            let text = std::fs::read_to_string(flavor.join(script)).expect("the script");
            let trap = text
                .find("' USR2")
                .unwrap_or_else(|| panic!("{script} has no USR2 trap"));
            let written = text
                .find(&format!(">\"$pids/{pid}\""))
                .expect("the pid file");
            assert!(trap < written, "{script} writes {pid} before it traps USR2");
        }
    }

    #[test]
    fn the_log_out_names_a_session_only_when_there_is_one() {
        let marks = "echo \"$@\" >>\"$(dirname \"$0\")/marks\"";
        let bin = bin("logout", false, &[("loginctl", marks)]);
        let power = |session: Option<&str>| {
            let mut run = command(&bin, "../../shared/power.sh", &["logout"], "C");
            run.env("USER", "zed");
            if let Some(id) = session {
                run.env("XDG_SESSION_ID", id);
            }
            run.output().expect("sh runs");
            std::fs::read_to_string(bin.join("marks")).unwrap_or_default()
        };
        assert_eq!(power(Some("7")), "terminate-session 7\n");
        std::fs::remove_file(bin.join("marks")).expect("marks");
        assert_eq!(power(None), "terminate-user zed\n");
        std::fs::remove_file(bin.join("marks")).expect("marks");
        assert_eq!(power(Some("")), "terminate-user zed\n");
    }

    #[test]
    fn the_screenshot_goes_where_xdg_user_dir_says() {
        let grim = "for last; do :; done; : >\"$last\"";
        let pictures = "echo \"$(dirname \"$0\")/Bilder\"";
        let bin = bin(
            "pictures",
            false,
            &[("grim", grim), ("xdg-user-dir", pictures)],
        );
        let mut run = command(&bin, "../../shared/screenshot.sh", &[], "C");
        run.env("HOME", &bin)
            .env("XDG_PICTURES_DIR", bin.join("env"));
        assert!(run.status().expect("sh runs").success());
        assert!(bin.join("Bilder/Screenshots").is_dir());
        assert!(!bin.join("env").exists());
    }

    #[test]
    fn the_colours_of_the_panel_are_the_generated_ones_unless_the_user_file_is_newer() {
        if !has_jq() {
            return;
        }
        let bin = bin(
            "colours",
            true,
            &[("choractl", CHORACTL), ("waybar", "exit 0")],
        );
        outputs(&bin, &["A"], &["A"]);
        let (made, own) = (bin.join("chora-tiling/theme"), bin.join("cfg/chora/tiling"));
        std::fs::create_dir_all(&made).expect("theme");
        std::fs::create_dir_all(&own).expect("own");
        std::fs::write(own.join("colors.css"), "own").expect("own file");
        std::thread::sleep(Duration::from_millis(1100));
        std::fs::write(made.join("colors.css"), "made").expect("generated file");
        let dir = waybar_dir();
        let start = |bin: &Path| {
            let mut run = command(bin, "waybar.sh", &[dir.to_str().expect("a path")], "C");
            run.env("XDG_CONFIG_HOME", bin.join("cfg"))
                .output()
                .expect("sh runs");
            std::fs::read_to_string(bin.join("chora-tiling-waybar/colors.css")).expect("colors")
        };
        assert_eq!(start(&bin), "made", "the older file of the user won");
        std::thread::sleep(Duration::from_millis(1100));
        std::fs::write(own.join("colors.css"), "own").expect("own file");
        assert_eq!(start(&bin), "own", "the newer file of the user lost");
    }

    #[test]
    fn the_panel_needs_jq() {
        let bin = bin(
            "no-jq",
            false,
            &[("choractl", CHORACTL), ("waybar", "exit 0")],
        );
        let dir = waybar_dir();
        let out = run(&bin, "waybar.sh", &[dir.to_str().expect("a path")], "C");
        assert!(!out.status.success());
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(said.contains("jq"), "{said}");
    }
}
