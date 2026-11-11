//! chora-tiling: a tiling plugin with the user experience of awesome. Each
//! output has tags, each tag a layout, and the bindings come from a YAML
//! file (see `tiling.yaml`). Other clients, for example an agent, switch
//! the layout and drive the windows through the exported `tiling.*`
//! methods (see `docs/configuration.md`).

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod config;
mod dress;
mod keys;
mod layout;
mod theme;
mod wm;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
    process::ExitCode,
};

use chora_kit::{
    effects::{Driver, Target},
    give::{self, Give, MOVE},
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Clip, Command, Event, GivenGrab, InputMode, NodeKind, OutputInfo, PropsUpdate, ROOT,
    SurfaceRequest, ToplevelState, Transform, Uniform,
    ipc::{ClientError, ExportMethod, PressState, Query, RpcError},
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    config::{Action, Config},
    dress::Dress,
    layout::Layout,
    wm::{Effect, Mark, Placement, States, WINDOW_KEY, Wm},
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
    gap: u16,
    master_width: f64,
    masters: usize,
    columns: usize,
    /// The windows that show on the current output, in their order.
    windows: Vec<String>,
}

struct Tiling {
    rt: Runtime<Action>,
    wm: Wm,
    /// The configuration, as the file has it.
    settings: settings::File<Config>,
    /// The app ids of windows to leave to other clients.
    ignore: Vec<String>,
    /// What the last transaction set, so that the next one sends changes
    /// only.
    sent: HashMap<String, Placement>,
    order: Vec<String>,
    focus: ShownFocus,
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
    /// Whether the next placement is a rebuild, not a change a person made:
    /// nothing moves.
    still: bool,
    /// Whether the key list shows, and holds the keyboard.
    keys_open: bool,
    /// When the layout chip leaves.
    chip_until: Option<std::time::Instant>,
    /// The colours of the look, and the scheme they come from.
    colors: theme::Colors,
    scheme: theme::Scheme,
    /// When the system is asked for its colour scheme again.
    theme_check: Option<std::time::Instant>,
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

/// `--launch PROGRAM [ARGS]`: starts foot, fuzzel or swaylock with the
/// config file of the flavor, in the palette; another program as it is.
fn launch(argv: &[String]) -> ExitCode {
    use std::os::unix::process::CommandExt as _;
    let Some((program, rest)) = argv.split_first() else {
        eprintln!("usage: chora-tiling --launch PROGRAM [ARGS]");
        return ExitCode::from(2);
    };
    let dirs = theme::config_dirs();
    let line = theme::launch_line(program, rest, &dirs, theme::written_scheme(&dirs));
    let err = std::process::Command::new(&line[0]).args(&line[1..]).exec();
    eprintln!("chora-tiling: cannot start {program}: {err}");
    ExitCode::FAILURE
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
    let settings_effects = settings.value.effects.clone();
    let mut tiling = Tiling {
        rt,
        wm: Wm::new(settings.value.clone()),
        settings,
        ignore: args.ignore.clone(),
        sent: HashMap::new(),
        order: Vec::new(),
        focus: ShownFocus::default(),
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
        still: true,
        keys_open: false,
        chip_until: None,
        colors: theme::Colors::of(theme::Scheme::Ink),
        scheme: theme::Scheme::Ink,
        theme_check: None,
    };
    tiling.apply_theme(false);
    tiling.start()?;
    tiling.still = false;
    Ok(serve(&mut tiling)?)
}

/// How often the system is asked for its colour scheme.
const THEME_CHECK: std::time::Duration = std::time::Duration::from_secs(5);

/// The units of the panel and of the wallpaper of the flavor.
const PANEL_UNIT: &str = "chora-tiling-waybar.service";
const WALLPAPER_UNIT: &str = "chora-tiling-swaybg.service";

/// How long the layout chip shows.
const CHIP_TIME: std::time::Duration = std::time::Duration::from_millis(700);

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
const EVENTS: [&str; 13] = [
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
    "input.button",
    "node.key",
];

impl Plugin for Tiling {
    type Action = Action;

    fn runtime(&mut self) -> &mut Runtime<Action> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
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
}

impl Tiling {
    /// Whether `--output` names outputs and none of them is there: the
    /// plugin then leaves all windows to other window managers.
    fn without_outputs(&self) -> bool {
        self.rt.filters() && self.wm.outputs.is_empty()
    }

    /// Gives all windows away with a remove without `to`, so that another
    /// window manager adopts them, when the last output of `--output`
    /// goes.
    fn give_all_away(&mut self) -> Result<(), ClientError> {
        let ids: Vec<String> = self.wm.windows.iter().map(|w| w.id.clone()).collect();
        let commands = ids
            .iter()
            .map(|id| Command::Remove {
                id: id.clone(),
                to: None,
            })
            .collect();
        self.apply(commands)?;
        for id in &ids {
            self.forget_window(id);
        }
        Ok(())
    }

    fn start(&mut self) -> Result<(), ClientError> {
        for output in self.rt.outputs()? {
            self.wm
                .set_output(&output.name, output.layout_area(), full_area(&output));
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
            let ours = node["to"].is_null() || node["to"] == NAME;
            if let (Some(id), None, true) = (node["id"].as_str(), node["owner"].as_str(), ours) {
                self.adopt(id)?;
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

    /// Rebuilds the model from the meta of the kept nodes. A kept window
    /// without meta comes back as a new one.
    fn restore(&mut self, kept: &[Value]) {
        let meta = |node: &Value| {
            serde_json::from_value::<BTreeMap<String, String>>(node["meta"].clone())
                .unwrap_or_default()
        };
        let group = kept
            .iter()
            .find(|n| n["id"] == NAME)
            .map(meta)
            .unwrap_or_default();
        let windows: Vec<(String, String)> = kept
            .iter()
            .filter(|n| n["kind"] == "surface")
            .filter_map(|n| {
                let text = meta(n).remove(WINDOW_KEY).unwrap_or_default();
                Some((n["id"].as_str()?.to_owned(), text))
            })
            .collect();
        for node in kept {
            let surface = node["id"].as_str().and_then(|id| id.strip_prefix(GROUP));
            self.groups.extend(surface.map(str::to_owned));
        }
        self.wm.restore(&group, &windows);
        for (id, _) in &windows {
            if !self.wm.has(id) {
                self.wm.add_window(id);
            }
        }
    }

    /// Grabs every binding of the configuration. A binding that another
    /// client holds only logs.
    #[allow(clippy::expect_used, reason = "the configuration checks its bindings")]
    fn grab_all(&mut self) -> Result<(), ClientError> {
        let bindings = self.wm.config.bindings.clone();
        self.rt.give_keys(Some(|to| Action::Give(to.into())));
        self.rt
            .accept_injected(self.wm.config.test_accept_injected_keys);
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
        self.rt.grab(moves, Action::Drag)?;
        let swallowed: Vec<_> = self.wm.config.swallowed().collect();
        for binding in swallowed {
            if self.rt.action(&binding.to_string()).is_none() {
                self.rt.grab(binding, Action::Swallow)?;
            }
        }
        Ok(())
    }

    /// Applies the settings and grabs their bindings.
    fn reconfigure(&mut self) -> Result<(), ClientError> {
        self.check_shaders()?;
        self.rt.ungrab_all()?;
        self.apply_theme(true);
        self.wm.reconfigure(self.settings.value.clone());
        self.effects.set(&self.settings.value.effects);
        self.sent.clear();
        self.grab_all()?;
        if self.keys_open {
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
        match self.overlay_event(&event) {
            Some(done) => done,
            None => self.handle_scene(event),
        }
    }

    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    fn handle_scene(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            // The start adopts surfaces that the events then announce.
            Event::SurfaceCreated { surface, .. } if self.wm.has(&surface) => return Ok(()),
            // The core reserved it for another window manager.
            Event::SurfaceCreated { to: Some(to), .. } if to != NAME => return Ok(()),
            // A detached surface has no owner any more, so it is free to take.
            Event::SurfaceCreated { surface, .. } | Event::SurfaceDetached { surface } => {
                self.wm.remove_window(&surface);
                self.adopt(&surface)?;
            }
            Event::SurfaceDestroyed { surface } => {
                self.forget_window(&surface);
            }
            Event::SurfaceChanged {
                surface,
                size: Some(size),
                ..
            } => {
                if !self.wm.set_size(&surface, size) {
                    return Ok(());
                }
            }
            Event::OutputChanged { output } => {
                self.wm
                    .set_output(&output.name, output.layout_area(), full_area(&output));
            }
            Event::FocusChanged { surface } => {
                self.focus_changed(surface.as_deref());
                return Ok(());
            }
            Event::OutputRemoved { name } => self.output_removed(&name)?,
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                ..
            } => return self.act(&binding),
            Event::InputButton {
                binding,
                state,
                position,
                surface,
                grab,
                ..
            } => {
                self.drag(&binding, state, position, surface);
                self.take_given_drag(grab);
            }
            // A window that follows the pointer.
            Event::InputMoved {
                binding, position, ..
            } if self.drag_to(&binding, position) => {}
            Event::PointerEntered { node, moved } => {
                if !self.pointer_focus(&node, false, moved) {
                    return Ok(());
                }
            }
            Event::PointerPressed { node, .. } => {
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
                self.forget_window(node);
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
        let info = LayoutInfo {
            layout,
            gap: self.wm.config.gap,
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
            Event::NodeKey { key, state, .. } => {
                let escape = self.keys_open && key == "Escape" && *state == PressState::Pressed;
                Some(if escape { self.hide_keys() } else { Ok(()) })
            }
            Event::PointerPressed { node, .. }
                if self.keys_open && node.starts_with(keys::KEYS) =>
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

    /// When the loop must wake without an event: the next frame of an
    /// effect, or the end of the layout chip.
    fn wake(&self, frame: std::time::Instant) -> Option<std::time::Instant> {
        let frame = self.effects.animating().then_some(frame);
        [frame, self.chip_until, self.theme_check]
            .into_iter()
            .flatten()
            .min()
    }

    /// Takes the colours of the theme and writes the files of the flavor
    /// that carry them; `reload` asks the panel to read its style again,
    /// which it must when the colours changed under it. Without a user
    /// manager there is nothing to reload.
    fn apply_theme(&mut self, reload: bool) {
        let look = &self.settings.value.look;
        let scheme = look.theme.scheme(system_for(look.theme));
        let colors = look.palette.over(scheme);
        let changed = colors != self.colors;
        let new_scheme = scheme != self.scheme;
        self.theme_check =
            (look.theme == theme::Theme::System).then(|| std::time::Instant::now() + THEME_CHECK);
        self.scheme = scheme;
        self.colors = colors;
        let Some(dir) = theme::default_dir() else {
            return;
        };
        if let Err(err) = theme::apply(&dir, &self.colors, scheme) {
            eprintln!("chora-tiling: the files of the theme: {err}");
            return;
        }
        let user_unit = |verb: &str, unit: &str| {
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "--no-block", verb, unit])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        };
        if reload && changed {
            user_unit("reload", PANEL_UNIT);
        }
        if reload && new_scheme {
            user_unit("restart", WALLPAPER_UNIT);
        }
    }

    /// Asks the system for its colour scheme when it is time, and takes the
    /// new colours of the theme when it changed.
    fn follow_system(&mut self, now: std::time::Instant) -> Result<(), ClientError> {
        if self.theme_check.is_none_or(|at| now < at) {
            return Ok(());
        }
        let before = self.colors.clone();
        self.apply_theme(true);
        if self.colors != before {
            self.sent.clear();
            self.still = true;
            let done = self.relayout();
            self.still = false;
            done?;
            if self.keys_open {
                self.hide_keys()?;
                self.toggle_keys()?;
            }
        }
        Ok(())
    }

    /// Shows the chip that names the layout of the current tag for 700 ms.
    fn show_chip(&mut self) -> Result<(), ClientError> {
        let (Some((output, size)), Some((layout, _))) =
            (self.wm.current_output(), self.wm.current())
        else {
            return Ok(());
        };
        let lang = chora_kit::words::Language::of_session();
        let mut commands = Vec::new();
        if self.chip_until.is_some() {
            commands.push(Command::Remove {
                id: keys::CHIP.into(),
                to: None,
            });
        }
        commands.extend(keys::chip(
            &format!("screen:{output}"),
            size,
            keys::layout_word(layout, lang),
            &self.colors,
        ));
        self.chip_until = Some(std::time::Instant::now() + CHIP_TIME);
        if self.apply_report(commands)? {
            let guess = keys::guess(keys::layout_word(layout, lang));
            let measured = self.measured(&format!("{}-word", keys::CHIP));
            let animate = self.wm.config.look.animations;
            self.apply(keys::chip_fit(size, measured.unwrap_or(guess), animate))?;
        }
        Ok(())
    }

    /// The size that the renderer gave a text node, in logical pixels.
    fn measured(&mut self, id: &str) -> Option<[f64; 2]> {
        let nodes = self.query(&format!("id:{id}")).ok()?;
        let rect = nodes
            .first()
            .and_then(|n| n["rects"].as_object())
            .and_then(|r| r.values().next())
            .and_then(Value::as_array);
        rect.and_then(|r| Some([r.get(2)?.as_f64()?, r.get(3)?.as_f64()?]))
    }

    /// Takes the chip away when its time is up.
    fn end_chip(&mut self, now: std::time::Instant) -> Result<(), ClientError> {
        if self.chip_until.is_some_and(|at| now >= at) {
            self.chip_until = None;
            self.apply(vec![Command::Remove {
                id: keys::CHIP.into(),
                to: None,
            }])?;
        }
        Ok(())
    }

    /// Shows the key list when it is hidden, and hides it when it shows.
    fn toggle_keys(&mut self) -> Result<(), ClientError> {
        if self.keys_open {
            return self.hide_keys();
        }
        let Some((output, size)) = self.wm.current_output() else {
            return Ok(());
        };
        let lang = chora_kit::words::Language::of_session();
        let rows = keys::rows(&self.wm.config, lang);
        let mut commands =
            keys::key_list(&format!("screen:{output}"), size, &rows, &self.colors, lang);
        let fade = self.wm.config.look.animations.then_some(dress::FADE);
        commands.push(dress::change(
            keys::KEYS.into(),
            PropsUpdate {
                opacity: Some(1.0),
                ..PropsUpdate::default()
            },
            fade,
        ));
        commands.push(Command::FocusNode {
            node: Some(keys::KEYS.into()),
        });
        self.apply(commands)?;
        self.keys_open = true;
        Ok(())
    }

    /// Hides the key list and gives the keyboard back.
    fn hide_keys(&mut self) -> Result<(), ClientError> {
        self.keys_open = false;
        self.apply(vec![
            Command::FocusNode { node: None },
            Command::Remove {
                id: keys::KEYS.into(),
                to: None,
            },
        ])
    }

    /// `set_layout` and `next_layout`.
    fn change_layout(&mut self, method: &str, params: Value) -> Result<(), RpcError> {
        if method == "next_layout" {
            self.wm.act(&Action::NextLayout);
            return Ok(());
        }
        let change: SetLayout = runtime::params(params)?;
        if let Some(layout) = change.layout {
            self.wm.set_layout(layout);
        }
        if let Some(gap) = change.gap {
            self.wm.config.gap = gap;
            self.wm.config.look.gap_outer = None;
            self.wm.config.look.gap_inner = None;
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

    /// `window.give`: hands a window to another window manager, and
    /// answers with its id.
    fn give(&mut self, params: Value) -> Result<Value, RpcError> {
        let Give { surface, to, grab } = runtime::params(params)?;
        self.hand_dragged(&surface, &to, grab)
    }

    /// A press of `move` that came with a window and its drag from
    /// another window manager: the pointer holds the point of the window
    /// that the giver names, kept inside the window, and the window
    /// wobbles on from its springs.
    fn take_given_drag(&mut self, grab: Option<GivenGrab>) {
        let (Some(grab), Some((id, _))) = (grab, self.dragged.clone()) else {
            return;
        };
        let size = self.target(&id, [0.0; 2]).map_or([0.0; 2], |t| t.size);
        let point = [0, 1].map(|axis| grab.point[axis].clamp(0.0, size[axis].max(0.0)));
        self.dragged = Some((id.clone(), point));
        self.springs = grab.wobble.map(|springs| (id, springs));
    }

    /// Hands a window to the window manager `to` with a remove with `to`
    /// (D55); the receiver gives it the focus. The answer of
    /// `window.give`.
    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        self.hand_dragged(surface, to, None)
    }

    /// As [`Self::hand`]; with `grab` the drag of the window goes too,
    /// with the springs of its wobble unless the caller sent some, and
    /// the drag of tiling ends.
    fn hand_dragged(
        &mut self,
        surface: &str,
        to: &str,
        mut grab: Option<GivenGrab>,
    ) -> Result<Value, RpcError> {
        only(self.wm.has(surface), || {
            format!("tiling has no window {surface}")
        })?;
        let bent = self.bent.remove(surface);
        if let (Some(grab), Some(target)) = (&mut grab, &bent) {
            let springs = self.effects.springs(surface, target);
            grab.wobble = grab.wobble.take().or(springs);
        }
        if let Some(flat) = self.effects.unbend(surface, surface) {
            self.apply(vec![flat])
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        let dragged = grab.is_some();
        let answer = self.rt.conn.give_dragged(surface, to, grab)?;
        if dragged {
            self.dragged.take_if(|(id, _)| id == surface);
        }
        self.focus.gave(surface);
        self.forget_window(surface);
        self.relayout()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(answer)
    }

    /// The key `give`: the focused window goes to the window manager `to`.
    /// A refusal only logs.
    fn give_focused(&mut self, to: &str) {
        if let Err(err) = give::focused(self, to) {
            eprintln!("chora-tiling: give to {to}: {}", err.message);
        }
    }

    /// Whether a binding drags: the one of `drag`, or `move`, the
    /// interactive move of a client, for example from its title bar.
    fn drags(&self, binding: &str) -> bool {
        matches!(self.rt.action(binding), Some(Action::Drag))
    }

    /// A drag: the press takes the window that asked for a move, else the
    /// window under the pointer, and a release outside the claim of tiling
    /// hands it to the window manager whose claim holds the point (D55).
    /// Without one there, the window stays.
    fn drag(&mut self, binding: &str, state: PressState, point: [f64; 2], asked: Option<String>) {
        if !self.drags(binding) {
            return;
        }
        if state == PressState::Pressed {
            if let Some((old, _)) = self.dragged.take() {
                self.effects.release(&old);
            }
            let grabbed = taken(asked, |id| self.wm.has(id), || self.window_at(point));
            self.dragged = grabbed.map(|id| {
                let [x, y] = (self.sent.get(&id))
                    .and_then(|p| p.at)
                    .map_or(point, |at| at.map(f64::from));
                (id, [point[0] - x, point[1] - y])
            });
            return;
        }
        let Some((surface, _)) = self.dragged.take() else {
            return;
        };
        self.effects.release(&surface);
        if self.wm.output_at(point).is_some() {
            return;
        }
        let given = match self.rt.conn.claimant_at(point) {
            Ok(Some(to)) => self.hand(&surface, &to).map(drop),
            Ok(None) => Err(RpcError::new(RpcError::REJECTED, "no claim there")),
            Err(err) => Err(err),
        };
        if let Err(err) = given {
            eprintln!("chora-tiling: {surface} stays: {}", err.message);
            // Back from where the pointer left it.
            self.wm.unplace(&surface);
        }
    }

    /// A move of a drag: a window that does not tile follows the pointer.
    /// True when it moved.
    #[allow(clippy::cast_possible_truncation, reason = "scene coordinates")]
    fn drag_to(&mut self, binding: &str, [px, py]: [f64; 2]) -> bool {
        let Some((id, [dx, dy])) = self.dragged.clone().filter(|_| self.drags(binding)) else {
            return false;
        };
        let at = [(px - dx).round() as i32, (py - dy).round() as i32];
        let placed = self.wm.place(&id, at, [px, py]);
        if placed {
            self.bend(&id, at.map(f64::from), [px, py]);
        }
        placed
    }

    /// The window as the effects bend it: at an origin, with its size.
    fn target(&self, id: &str, origin: [f64; 2]) -> Option<Target> {
        let window = self.wm.windows.iter().find(|w| w.id == id)?;
        let size = self.sent.get(id).and_then(|p| p.size).or(window.size)?;
        Some(Target {
            node: id.into(),
            origin,
            size: size.map(f64::from),
            set_size: false,
        })
    }

    /// A dragged window wobbles: it starts to at the first move, and each
    /// move bends its mesh.
    fn bend(&mut self, id: &str, origin: [f64; 2], pointer: [f64; 2]) {
        let Some(target) = self.target(id, origin) else {
            return;
        };
        if !self.effects.wobbles(id) {
            let springs = self.springs.take_if(|(of, _)| of == id).map(|(_, s)| s);
            (self.effects).grab_given(id, &target, pointer, springs.as_ref());
        }
        let mesh = self.effects.motion(id, &target);
        self.meshes.extend(mesh);
        if self.effects.wobbles(id) {
            self.bent.insert(id.into(), target);
        }
    }

    /// The watchdog of the core removed the meshes: the effects end.
    #[allow(clippy::unnecessary_wraps, reason = "an event handler")]
    fn effects_dropped(&mut self) -> Result<(), ClientError> {
        eprintln!("chora-tiling: frames were slow, the core dropped the effects");
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

    fn focus_changed(&mut self, surface: Option<&str>) {
        self.focus.moved(surface);
    }

    /// An output goes; without one of `--output` the windows go too.
    fn output_removed(&mut self, name: &str) -> Result<(), ClientError> {
        self.wm.remove_output(name);
        if self.without_outputs() {
            self.give_all_away()?;
        }
        Ok(())
    }

    /// Focuses a window under the pointer. While another window manager
    /// has the focus, the focused window of tiling takes it again.
    fn pointer_focus(&mut self, node: &str, clicked: bool, moved: bool) -> bool {
        let took = self.wm.pointer_focus(node, clicked, moved, self.focus.lost);
        if took {
            self.focus.take();
        }
        took
    }

    fn act(&mut self, binding: &str) -> Result<(), ClientError> {
        let Some(action) = self.rt.action(binding).cloned() else {
            return Ok(());
        };
        if let Action::Give(to) = &action {
            self.give_focused(to);
            return Ok(());
        }
        if action == Action::ShowKeys {
            return self.toggle_keys();
        }
        let before = self.wm.focused().map(String::from);
        let effect = self.wm.act(&action);
        // A binding that moves the focus of tiling takes it back.
        if self.wm.focused() != before.as_deref() {
            self.focus.take();
        }
        let layout_key = matches!(action, Action::NextLayout | Action::PreviousLayout);
        match effect {
            Some(Effect::Relayout) if layout_key => {
                self.relayout()?;
                self.show_chip()
            }
            Some(Effect::Relayout) => self.relayout(),
            Some(Effect::Run(command)) => {
                chora_kit::run(&command, self.rt.conn.activation_token().as_deref());
                Ok(())
            }
            Some(Effect::Close(surface)) => self.apply(vec![Command::Close { surface }]),
            Some(Effect::Volume(change)) => {
                self.volume.change(change, true);
                Ok(())
            }
            // The same path as the method `settings.reload`.
            Some(Effect::Reload) => {
                if let Err(err) = runtime::reload_settings(self) {
                    eprintln!("chora-tiling: {}", err.message);
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
        self.apply(pass_on(surface))
    }

    /// Takes a surface without an owner. A surface that another client took
    /// first, or whose app id `--ignore` names, stays out of the layout.
    fn adopt(&mut self, surface: &str) -> Result<(), ClientError> {
        if self.without_outputs() {
            return self.pass_on_reserved(surface);
        }
        let nodes = self.query(&format!("id:{surface}"))?;
        let ignored = nodes.iter().any(|node| {
            node["surface"]["app_id"]
                .as_str()
                .is_some_and(|app| self.ignore.iter().any(|i| i == app))
        });
        // The size of a window that comes with one, as from another window
        // manager: it floats at that size, centered, and a drag hits it.
        let size = nodes
            .first()
            .and_then(|n| serde_json::from_value(n["surface"]["size"].clone()).ok());
        if ignored {
            return self.pass_on_reserved(surface);
        }
        let mut commands = Vec::new();
        if !self.groups.contains(surface) {
            commands.extend(frame_nodes(
                surface,
                self.border.as_deref(),
                self.shadow.as_deref(),
            ));
        }
        commands.push(Command::Attach {
            surface: surface.into(),
            parent: clip_of(surface),
        });
        match self.call_apply(commands) {
            Ok(()) => {
                self.groups.insert(surface.into());
                // A new window of tiling takes the focus.
                self.wm.add_window(surface);
                if let Some(size) = size {
                    self.wm.set_size(surface, size);
                }
                self.focus.take();
                self.style(surface)
            }
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {surface}: {err}");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Gives a new window the style of the settings. A shader that fails
    /// only logs, and the window stays.
    fn style(&mut self, surface: &str) -> Result<(), ClientError> {
        self.apply(vec![styled(&self.wm.config.style, surface)])
    }

    /// Sends what changed since the last transaction: sizes and states,
    /// positions, visibility, the stacking order and the focus.
    fn relayout(&mut self) -> Result<(), ClientError> {
        let placements = self.wm.arrange();
        let mut commands = Vec::new();
        commands.extend(self.window_changes(&placements));
        let order: Vec<String> = placements.iter().map(|p| p.id.clone()).collect();
        if order != self.order {
            commands.extend(order.iter().enumerate().map(|(index, id)| Command::Move {
                id: group_of(id),
                parent: None,
                index: Some(index),
            }));
        }
        let focus = self.wm.focused().map(String::from);
        let send_focus = self.focus.sends(focus.as_deref());
        if send_focus {
            commands.push(Command::Focus {
                surface: focus.clone(),
            });
        }
        let saved = self.meta();
        for (id, meta) in &saved {
            commands.extend(meta_change(self.saved.get(id), id, meta));
        }
        let claim = self.wm.claim();
        if claim != self.claimed {
            commands.push(Command::Update {
                id: NAME.into(),
                props: PropsUpdate {
                    // No outputs: no claim, so `null`.
                    claim: Some(claim),
                    ..PropsUpdate::default()
                },
            });
        }
        commands.append(&mut self.meshes);
        commands.extend(self.stale.drain(..).map(|id| Command::Remove {
            id: group_of(&id),
            to: None,
        }));
        if commands.is_empty() {
            return Ok(());
        }
        let gained = self.claimed.is_none() && claim.is_some();
        self.saved = saved;
        self.claimed = claim;
        self.sent = placements.into_iter().map(|p| (p.id.clone(), p)).collect();
        self.last_tag = self.wm.current_tag_index();
        self.order = order;
        if send_focus {
            self.focus.sent(focus);
        }
        self.apply(commands)?;
        // A grab of a pointer binding that another window manager holds
        // succeeds only with a claim, for example with `--output` for an
        // output that came later (D55).
        if gained && self.claimed.is_some() {
            self.rt.ungrab_all()?;
            self.grab_all()?;
        }
        Ok(())
    }

    /// The commands that bring the windows from what was sent to their
    /// placements.
    fn window_changes(&self, placements: &[Placement]) -> Vec<Command> {
        let c = &self.colors;
        let colors = [&c.accent, &c.accent_2, &c.line, &c.urgent].map(|h| theme::channels(h));
        let look = &self.wm.config.look;
        let width = self.wm.current_output().map_or(0.0, |(_, size)| size[0]);
        let tag = self.wm.current_tag_index();
        let dress = Dress {
            colors: self.border.is_some().then_some(&colors),
            shadows: look.shadows,
            animate: look.animations && !self.still,
            slide: match tag.cmp(&self.last_tag) {
                std::cmp::Ordering::Greater => width,
                std::cmp::Ordering::Less => -width,
                std::cmp::Ordering::Equal => 0.0,
            },
        };
        placements
            .iter()
            .flat_map(|p| changes(self.sent.get(&p.id), p, &dress))
            .collect()
    }

    /// The meta of the group and of each window, by node.
    fn meta(&self) -> HashMap<String, BTreeMap<String, String>> {
        let windows = self.wm.window_meta().into_iter();
        windows
            .map(|(id, text)| (id, BTreeMap::from([(WINDOW_KEY.to_owned(), text)])))
            .chain([(NAME.to_owned(), self.wm.group_meta())])
            .collect()
    }

    /// Forgets a window that left tiling: its place in the model, and
    /// what the core shows of it.
    fn forget_window(&mut self, id: &str) {
        self.wm.remove_window(id);
        self.sent.remove(id);
        self.saved.remove(id);
        if self.groups.remove(id) {
            self.stale.push(id.into());
        }
        self.effects.cancel_wobble(id);
        self.bent.remove(id);
        // The focus on it is no longer that of tiling: when the window
        // comes back, tiling sends it again.
        if self.focus.surface.as_deref() == Some(id) {
            self.focus.surface = None;
            self.focus.sent = false;
        }
    }

    /// Forgets what the core shows, so that the next transaction sends all.
    fn forget(&mut self) {
        self.sent.clear();
        self.saved.clear();
        self.claimed = None;
        self.order.clear();
        self.focus.sent = false;
    }

    /// Has the core compile the shaders of the style, on a node that the
    /// same transaction removes again, so that `settings.set` refuses a
    /// shader that does not compile instead of each new window.
    fn check_shaders(&mut self) -> Result<(), ClientError> {
        let effects = self.settings.value.style.effects.clone();
        if effects.is_empty() {
            return Ok(());
        }
        let id = "tiling-shader-check";
        self.call_apply(vec![
            Command::Create {
                id: id.into(),
                kind: NodeKind::Rect,
                parent: NAME.into(),
                props: PropsUpdate {
                    effects: Some(effects),
                    ..PropsUpdate::default()
                },
            },
            Command::Remove {
                id: id.into(),
                to: None,
            },
        ])
    }

    fn call_apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        match self.rt.conn.try_apply(commands)? {
            None => Ok(()),
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    /// Applies a transaction. A rejected transaction only logs, and the next
    /// one sends everything again: a surface can disappear between an event
    /// and the answer to it, and the event that follows corrects the layout.
    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        self.apply_report(commands).map(|_| ())
    }

    /// Like `apply`, but `Ok(false)` when the core rejected the transaction.
    fn apply_report(&mut self, commands: Vec<Command>) -> Result<bool, ClientError> {
        match self.call_apply(commands) {
            Err(ClientError::Rpc(err)) => {
                if !err.is_gone_surface() {
                    eprintln!("chora-tiling: {err}");
                }
                self.forget();
                Ok(false)
            }
            Ok(()) => Ok(true),
            Err(err) => Err(err),
        }
    }
}

/// The window that a drag takes: the window of tiling that asked for a
/// move, from its title bar for example, else the window `under` the
/// pointer, which may be another window where windows overlap.
fn taken(
    asked: Option<String>,
    ours: impl Fn(&str) -> bool,
    under: impl FnOnce() -> Option<String>,
) -> Option<String> {
    match asked {
        Some(id) => ours(&id).then_some(id),
        None => under(),
    }
}

/// The prefix of the id of the group that frames a window.
const GROUP: &str = "tw-";

/// The id of the node that clips a window to its corners.
fn clip_of(surface: &str) -> String {
    format!("tc-{surface}")
}

/// The id of the node that draws the border of a window.
fn border_of(surface: &str) -> String {
    format!("tb-{surface}")
}

/// The nodes of a new window: the group that carries it, the border and
/// the clip that holds the surface.
fn frame_nodes(surface: &str, border: Option<&str>, shadow: Option<&str>) -> Vec<Command> {
    let create = |id: String, kind, parent: String, props| Command::Create {
        id,
        kind,
        parent,
        props,
    };
    let hidden = PropsUpdate {
        size: Some([1.0, 1.0]),
        visible: Some(false),
        ..PropsUpdate::default()
    };
    let mut nodes = vec![create(
        group_of(surface),
        NodeKind::Group,
        NAME.into(),
        hidden.clone(),
    )];
    if let Some(shader) = shadow {
        nodes.extend(dress::shadow_nodes(surface, shader, &group_of(surface)));
    }
    if let Some(shader) = border {
        let props = PropsUpdate {
            shader: Some(shader.into()),
            input: Some(InputMode::Pass),
            ..hidden
        };
        nodes.push(create(
            border_of(surface),
            NodeKind::Shader,
            group_of(surface),
            props.clone(),
        ));
        let focus = PropsUpdate {
            opacity: Some(0.0),
            ..props
        };
        nodes.push(create(
            dress::focus_of(surface),
            NodeKind::Shader,
            group_of(surface),
            focus,
        ));
    }
    nodes.push(create(
        clip_of(surface),
        NodeKind::Group,
        group_of(surface),
        PropsUpdate {
            size: Some([1.0, 1.0]),
            ..PropsUpdate::default()
        },
    ));
    nodes
}

/// A shader of the plugin, written into the runtime directory of the user;
/// none when there is none, or it is no absolute path (the shared temporary
/// directory would let another user plant it first).
fn runtime_shader(name: &str, text: &str) -> Option<String> {
    let runtime = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?);
    let path = runtime.join("chora-tiling").join(name);
    runtime.is_absolute().then_some(())?;
    if chora_kit::read_text(&path, chora_kit::MAX_TEXT)
        .ok()
        .as_deref()
        != Some(text)
    {
        chora_kit::write_file(&path, text.as_bytes()).ok()?;
    }
    Some(path.to_string_lossy().into_owned())
}

/// The id of the group of a window: it holds the surface, clips it to the
/// rounded corners and carries its position.
fn group_of(surface: &str) -> String {
    format!("{GROUP}{surface}")
}

/// Whether an answered request takes the window out of tiling: the core
/// accepted a `move` to a parent that is not the tiling group. A rejected
/// transaction leaves the window managed (D31).
fn leaves_tiling(applied: bool, action: &str, args: &Value) -> bool {
    applied
        && action == "move"
        && args
            .get("parent")
            .and_then(Value::as_str)
            .is_some_and(|parent| parent != NAME)
}

/// The answer to a request of another client for one of the windows
/// (D31): the commands that do it. `focus` goes through the model and
/// answers without commands. `None`: the action, for example `resize`,
/// stays unanswered, as does an `update` with nothing left to change.
fn answers(wm: &mut Wm, node: &str, action: &str, args: &Value) -> Option<Vec<Command>> {
    match action {
        "focus" => {
            wm.focus(node);
            Some(Vec::new())
        }
        "close" => Some(vec![Command::Close {
            surface: node.into(),
        }]),
        "move" => Some(vec![Command::Move {
            id: node.into(),
            parent: args.get("parent").and_then(Value::as_str).map(|p| {
                if p == NAME {
                    group_of(node)
                } else {
                    p.to_owned()
                }
            }),
            index: args
                .get("index")
                .and_then(Value::as_u64)
                .and_then(|i| usize::try_from(i).ok()),
        }]),
        "update" => looked(node, args).map(|update| vec![update]),
        _ => None,
    }
}

/// The update of the look of a window that another client asks for
/// (D58): opacity, effects and meta, without the meta key of tiling. No
/// relayout sends opacity or effects, so the look stays across layouts,
/// tags and restarts with resume, and wins over the style.
fn looked(node: &str, args: &Value) -> Option<Command> {
    let asked: PropsUpdate = serde_json::from_value(args.clone()).ok()?;
    let mut meta = asked.meta;
    if let Some(meta) = &mut meta {
        meta.remove(WINDOW_KEY);
    }
    let props = PropsUpdate {
        opacity: asked.opacity,
        effects: asked.effects,
        meta: meta.filter(|m| !m.is_empty()),
        ..PropsUpdate::default()
    };
    (props != PropsUpdate::default()).then(|| Command::Update {
        id: node.into(),
        props,
    })
}

/// The update that gives a new window a style.
fn styled(style: &config::Style, surface: &str) -> Command {
    Command::Update {
        id: surface.into(),
        props: PropsUpdate {
            opacity: Some(style.opacity),
            effects: Some(style.effects.clone()),
            ..PropsUpdate::default()
        },
    }
}

/// The commands that bring a window from what was sent to its placement.
/// `colors` are the colours of the palette when the window has a border.
fn changes(sent: Option<&Placement>, placement: &Placement, dress: &Dress) -> Vec<Command> {
    let id = &placement.id;
    let Some(at) = placement.at else {
        return hide(sent, id);
    };
    let mut commands = Vec::new();
    if sent.is_none_or(|s| s.size != placement.size || s.states != placement.states) {
        commands.push(configure(id, placement.size, placement.states));
    }
    if sent.is_none_or(|s| s.at != placement.at) {
        commands.extend(dress::place(sent, placement, at, dress));
    }
    let reshaped = sent.is_none_or(|s| s.frame != placement.frame);
    let rounded = sent.is_none_or(|s| s.radius != placement.radius);
    if reshaped || rounded {
        let mut props = PropsUpdate::default();
        if reshaped {
            props.size = Some(placement.frame.map(f64::from));
        }
        if rounded {
            props.clip = Some((placement.radius > 0).then(|| Clip {
                radius: f64::from(placement.radius),
            }));
        }
        commands.push(update(clip_of(id), props));
    }
    if let Some(colors) = dress.colors {
        if sent.is_none_or(|s| {
            s.frame != placement.frame
                || s.radius != placement.radius
                || s.border != placement.border
                || s.mark != placement.mark
        }) {
            commands.extend(border_updates(placement, colors));
            commands.extend(dress::shadow_updates(placement, dress.shadows));
        }
        commands.extend(dress::fade(sent, placement, dress.animate));
    }
    commands
}

fn update(id: String, props: PropsUpdate) -> Command {
    Command::Update { id, props }
}

/// The updates of the two border nodes of a window: the ring in the line
/// colour (urgent for a window that asked for attention) and the gradient
/// of the focus, which shows through its opacity.
fn border_updates(placement: &Placement, colors: &[[f64; 3]; 4]) -> Vec<Command> {
    let line = match placement.mark {
        Mark::Urgent => colors[3],
        Mark::Line | Mark::Focus => colors[2],
    };
    vec![
        ring_update(border_of(&placement.id), placement, [line, line]),
        ring_update(
            dress::focus_of(&placement.id),
            placement,
            [colors[0], colors[1]],
        ),
    ]
}

/// The update of a border node: its size and place around the window, its
/// colours and whether it shows.
fn ring_update(node: String, placement: &Placement, [from, to]: [[f64; 3]; 2]) -> Command {
    let width = f64::from(placement.border);
    let color = |[r, g, b]: [f64; 3]| Uniform::Vector(vec![r, g, b, 1.0]);
    let [w, h] = placement.frame.map(f64::from);
    update(
        node,
        PropsUpdate {
            visible: Some(placement.border > 0),
            size: Some([w + 2.0 * width, h + 2.0 * width]),
            transform: Some(Transform::translate(-width, -width, 0.0)),
            uniforms: Some(BTreeMap::from([
                ("from".to_owned(), color(from)),
                ("to".to_owned(), color(to)),
                (
                    "radius".to_owned(),
                    Uniform::Float(f64::from(placement.radius) + width),
                ),
                ("width".to_owned(), Uniform::Float(width)),
                (
                    "logical".to_owned(),
                    Uniform::Vector(vec![w + 2.0 * width, h + 2.0 * width]),
                ),
            ])),
            ..PropsUpdate::default()
        },
    )
}

/// The update from the saved meta of a node to its new meta. A key that
/// went stays in the core; the restore skips it.
fn meta_change(
    saved: Option<&BTreeMap<String, String>>,
    id: &str,
    meta: &BTreeMap<String, String>,
) -> Option<Command> {
    (saved != Some(meta)).then(|| Command::Update {
        id: id.into(),
        props: PropsUpdate {
            meta: Some(meta.clone()),
            ..PropsUpdate::default()
        },
    })
}

/// Hides a window that showed or is new to the core, and takes its focus
/// state away.
fn hide(sent: Option<&Placement>, id: &str) -> Vec<Command> {
    if sent.is_some_and(|s| s.at.is_none()) {
        return Vec::new();
    }
    let mut commands = Vec::new();
    if let Some(sent) = sent.filter(|s| s.states.activated) {
        let states = States {
            activated: false,
            ..sent.states
        };
        commands.push(configure(id, sent.size, states));
    }
    commands.push(Command::Update {
        id: group_of(id),
        props: PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        },
    });
    commands
}

fn configure(id: &str, size: Option<[i32; 2]>, states: States) -> Command {
    let states = [
        (states.tiled, ToplevelState::Tiled),
        (states.maximized, ToplevelState::Maximized),
        (states.fullscreen, ToplevelState::Fullscreen),
        (states.activated, ToplevelState::Activated),
    ];
    Command::Configure {
        surface: id.into(),
        size,
        states: states
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, s)| s)
            .collect(),
    }
}

/// The layout values that `set_layout` accepts.
fn layout_schema() -> Value {
    json!({
        "enum": [
            "tile", "tile_left", "tile_bottom", "tile_top", "fair",
            "fair_horizontal", "spiral", "dwindle", "max", "fullscreen",
            "magnifier", "floating",
        ],
        "description": "The layout of the current tag.",
    })
}

/// The methods that other clients call, with description and schema of
/// their parameters, as `methods.list` answers them.
fn exported_methods() -> Vec<ExportMethod> {
    let mut methods = layout_methods();
    methods.extend(window_methods());
    methods
}

/// The methods about layouts, tags and the key list.
fn layout_methods() -> Vec<ExportMethod> {
    let field = |ty: &str, description: &str| json!({"type": ty, "description": description});
    vec![
        runtime::method(
            "set_layout",
            "Sets the layout and the gap of the current tag.",
            json!({
                "layout": layout_schema(),
                "gap": field("integer", "The gap in logical pixels."),
            }),
            &[],
        ),
        runtime::method(
            "show_keys",
            "Shows the list of all keys, or hides it, as the binding does.",
            json!({}),
            &[],
        ),
        runtime::method(
            "next_layout",
            "Goes to the next layout of the current tag, as the binding does.",
            json!({}),
            &[],
        ),
        runtime::method(
            "get_layout",
            "Answers the layout of the current tag.",
            json!({}),
            &[],
        ),
        runtime::method(
            "get_tags",
            "Answers the tags of each output: shown, focused, occupied and urgent.",
            json!({}),
            &[],
        ),
        give::method(),
    ]
}

/// The methods about tags and windows.
fn window_methods() -> Vec<ExportMethod> {
    let field = |ty: &str, description: &str| json!({"type": ty, "description": description});
    let window = || field("string", "The surface id of the window.");
    let tag = || field("integer", "The tag number, from 1.");
    let on_surface = json!({"surface": window()});
    vec![
        runtime::method(
            "view_tag",
            "Shows a tag on the current output.",
            json!({"tag": tag()}),
            &["tag"],
        ),
        runtime::method(
            "move_to_tag",
            "Moves a window to a tag.",
            json!({"surface": window(), "tag": tag()}),
            &["surface", "tag"],
        ),
        runtime::method(
            "focus",
            "Focuses a window.",
            on_surface.clone(),
            &["surface"],
        ),
        runtime::method("focus_next", "Focuses the next window.", json!({}), &[]),
        runtime::method("focus_prev", "Focuses the previous window.", json!({}), &[]),
        runtime::method(
            "swap_master",
            "Swaps a window into the master slot.",
            on_surface,
            &["surface"],
        ),
    ]
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

/// Takes a surface and lets it go at once, without `to`: the core then
/// announces it as detached, free for any window manager.
fn pass_on(surface: &str) -> Vec<Command> {
    vec![
        Command::Attach {
            surface: surface.into(),
            parent: NAME.into(),
        },
        Command::Remove {
            id: surface.into(),
            to: None,
        },
    ]
}

/// The keyboard focus as tiling shows it to the core.
#[derive(Debug, Default)]
struct ShownFocus {
    /// The focus of the last transaction that sent one.
    surface: Option<String>,
    /// Whether the core has `surface`.
    sent: bool,
    /// Another window manager took the focus since. Tiling sends none
    /// until one of its windows takes the focus back, so that no relayout
    /// takes it.
    lost: bool,
}

impl ShownFocus {
    /// The core moved the focus. An empty focus is no loss: it happens
    /// when the focused window of tiling itself exits, and tiling must
    /// then still focus its next window.
    fn moved(&mut self, surface: Option<&str>) {
        match surface {
            None => {}
            Some(s) if Some(s) == self.surface.as_deref() => self.lost = false,
            Some(_) => self.lost = true,
        }
    }

    /// Tiling hands the window with its focus to another window manager,
    /// which focuses it. Tiling sends no focus until one of its windows
    /// takes it back, whether the receiver set it before or after.
    fn gave(&mut self, surface: &str) {
        if self.surface.as_deref() == Some(surface) {
            self.lost = true;
        }
    }

    /// A window of tiling takes the focus back: by the pointer, a binding
    /// or as a new window. The next transaction sends it, even when it is
    /// the one sent before.
    fn take(&mut self) {
        if self.lost {
            self.lost = false;
            self.sent = false;
        }
    }

    /// Whether a transaction sends a focus.
    fn sends(&self, focus: Option<&str>) -> bool {
        !self.lost && (!self.sent || self.surface.as_deref() != focus)
    }

    /// A transaction sent a focus.
    fn sent(&mut self, focus: Option<String>) {
        self.surface = focus;
        self.sent = true;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixListener,
        sync::{Arc, Mutex},
    };

    use chora_kit::conn::Conn;

    use super::*;

    /// A tiling on a fake core that answers every request, `input.grab`
    /// with its binding, and keeps the names of the methods called.
    fn on_fake_core(tag: &str) -> (Tiling, Arc<Mutex<Vec<String>>>) {
        on_fake_core_with(tag, &[])
    }

    fn on_fake_core_with(
        tag: &str,
        effects: &[chora_kit::effects::Kind],
    ) -> (Tiling, Arc<Mutex<Vec<String>>>) {
        let dir = std::env::temp_dir().join(format!("chora-tiling-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("core.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let methods = Arc::new(Mutex::new(Vec::new()));
        let seen = methods.clone();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            for line in BufReader::new(&stream).lines().map_while(Result::ok) {
                let request: Value = serde_json::from_str(&line).unwrap();
                let method = request["method"].as_str().unwrap_or_default().to_owned();
                let result = match method.as_str() {
                    "hello" => json!({"version": chora_scene::ipc::VERSION}),
                    "input.grab" => request["params"]["binding"].clone(),
                    _ => Value::Null,
                };
                seen.lock()
                    .unwrap()
                    .push(format!("{method} {}", request["params"]));
                let answer = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
                writeln!(writer, "{answer}").unwrap();
            }
        });
        let conn = Conn::connect_at(NAME, &path).unwrap();
        let file = dir.join("chora-tiling.yaml");
        let mut settings = settings::File::<Config>::load(file, config::DEFAULT).unwrap();
        settings.value.effects = effects.to_vec();
        let tiling = Tiling {
            rt: Runtime::with(conn, NAME, Vec::new()),
            wm: Wm::new(settings.value.clone()),
            settings,
            ignore: Vec::new(),
            sent: HashMap::new(),
            order: Vec::new(),
            focus: ShownFocus::default(),
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
            still: true,
            keys_open: false,
            chip_until: None,
            colors: theme::Colors::of(theme::Scheme::Ink),
            scheme: theme::Scheme::Ink,
            theme_check: None,
        };
        (tiling, methods)
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
        };
        let moved = |position: [f64; 2]| Event::InputMoved {
            binding: give::MOVE.into(),
            position,
            output: "X".into(),
            local: position,
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
    fn drag_a_floating_window(tiling: &mut Tiling, methods: &Arc<Mutex<Vec<String>>>) -> String {
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.wm.act(&Action::ToggleFloating);
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
    fn sent_commands(methods: &Arc<Mutex<Vec<String>>>) -> Vec<Value> {
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
    fn bends(methods: &Arc<Mutex<Vec<String>>>, node: &str) -> usize {
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
    fn unbends(methods: &Arc<Mutex<Vec<String>>>, node: &str) -> usize {
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
            tiling.wm.act(&Action::ToggleFloating);
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
        };
        let moved = Event::InputMoved {
            binding: give::MOVE.into(),
            position: [230.0, 170.0],
            output: "X".into(),
            local: [230.0, 170.0],
        };
        tiling.event(press("s1", [190.0, 150.0])).unwrap();
        tiling.event(moved).unwrap();
        tiling.event(press("s2", [50.0, 50.0])).unwrap();
        settle(&mut tiling);
        assert!(bends(&methods, "s1") > 0);
        assert!(!tiling.effects.animating(), "s1 shakes to its end");
    }

    /// tiling with the floating window s1 of 40 by 20 on the output X.
    fn with_a_floating_window(tag: &str) -> (Tiling, Arc<Mutex<Vec<String>>>) {
        let (mut tiling, methods) = on_fake_core_with(tag, &WOBBLY);
        tiling
            .wm
            .set_output("X", [0, 0, 400, 300], [0, 0, 400, 300]);
        tiling.wm.add_window("s1");
        tiling.wm.act(&Action::ToggleFloating);
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
        };
        let moved = Event::InputMoved {
            binding: give::MOVE.into(),
            position: [230.0, 170.0],
            output: "X".into(),
            local: [230.0, 170.0],
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

    #[test]
    fn a_window_passed_on_is_free_for_any_window_manager() {
        assert!(matches!(
            &pass_on("s1")[..],
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
        let mut shown = ShownFocus::default();
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

    #[test]
    fn a_focus_emptied_by_a_closed_window_is_no_loss() {
        let mut shown = ShownFocus::default();
        shown.sent(Some("s3".into()));
        // The focused window of tiling exits: the core empties the focus.
        shown.moved(None);
        assert!(!shown.lost, "an empty focus is not another owner");
        assert!(shown.sends(Some("s4")), "the next window gets the focus");
        // Another window manager still takes it with its own window.
        shown.moved(Some("s9"));
        assert!(shown.lost && !shown.sends(Some("s4")));
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

    #[test]
    fn a_requested_look_applies_without_the_meta_of_tiling() {
        let mut tiler = tiler(&["s1"]);
        let args = serde_json::json!({"opacity": 0.5, "meta": {WINDOW_KEY: "x", "role": "a"}});
        let commands = answers(&mut tiler, "s1", "update", &args).expect("update answers");
        assert!(matches!(
            &commands[..],
            [Command::Update { id, props }] if id == "s1"
                && props.opacity == Some(0.5)
                && props.meta == Some(BTreeMap::from([("role".into(), "a".into())]))
        ));
        let own = serde_json::json!({"meta": {WINDOW_KEY: "x"}});
        assert!(answers(&mut tiler, "s1", "update", &own).is_none());
        // A relayout sends no look, so it keeps the requested one.
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
            Command::Update { props, .. } if props.opacity.is_some() || props.effects.is_some()))
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
    const TOOLS: [&str; 7] = ["sh", "awk", "timeout", "sleep", "cat", "kill", "dirname"];

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
    fn bin(name: &str, jq: bool, fakes: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-panel-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
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
            .env("XDG_RUNTIME_DIR", bin);
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
