//! chora-construct, steps K1 and K2a (D66): a light, foggy space with a
//! horizon, a walk on a leash, and panes that stand in it: windows,
//! and node trees of other plugins such as mc scenes. A click or a key
//! flies into a pane until it covers the output at 1:1; then the window
//! itself shows, sharp, with the keyboard. A binding flies out again.
//!
//! A construct is a file of panes (F17). As the flavor, the plugin owns
//! its windows. With `--overlay` it waits beside another flavor until
//! `construct.load`, and then covers the output from its screen root
//! (`screen:NAME`), so that no camera changes hands (F20); its program
//! panes show windows of the other flavor, which gets the keyboard
//! requests for them (D31).
//!
//! Step K2b: a window list shows the windows without a visible pane, and
//! a choice flies to the pane or takes the window of another client as a
//! pane (F29, F30). Such a window only shows through a node texture; the
//! construct changes none of its props, so its owner, any window manager,
//! has it back in full when the pane or the construct goes (F31).

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod config;
mod file;
mod hud;
mod space;

use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_kit::{
    give::{self, Give},
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Event, InputMode, NodeKind, NodeTexture, OutputInfo, PropsUpdate, ToplevelState,
    Transform, Uniform,
    ipc::{ClientError, ExportMethod, Filter, PressState, Query, RpcError},
    overlay_id,
};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::{Action, Config, Theme},
    file::{Kind, PaneFile},
    hud::{Claim, LIST, LIST_LINE, LIST_ROWS, Look, Metrics, Voice},
    space::{Lens, Pose},
};

/// The client name, and the id of the group of the space.
const NAME: &str = "construct";
const FOG: &str = "construct-fog";
/// A plain rect of the colour of the space under the fog: it shows when
/// the watchdog takes the shaders away.
const SKY: &str = "construct-sky";
/// The fog renders at this fraction of the output and scales up: it is
/// soft, and a full-size shader is too slow for software rendering.
const FOG_SCALE: f64 = 4.0;
/// The mirror images of the panes in the ground, under the panes.
const MIRRORS: &str = "construct-mirrors";
/// A mirror image renders at this fraction of its pane: a little soft.
const MIRROR_SCALE: f64 = 2.0;
/// The fog shader draws the shadows of this many panes, the nearest.
const SHADOWS: usize = 8;
const PANES: &str = "construct-panes";
const WINDOWS: &str = "construct-windows";
/// The glow of the focused pane, under it in the group of the panes; it
/// renders at this fraction of its size and scales up.
const FOCUS: &str = "construct-focus";
const FOCUS_SCALE: f64 = 4.0;
/// A clear rect over the space: it takes the drags, clicks, the wheel
/// and, through `focus_node`, the keys of the walk.
const LOOK: &str = "construct-look";
/// The prefix of the rows of the window list (F29).
const LIST_ROW: &str = "construct-list-row-";
const FOG_SHADER: &str = include_str!("../fog.frag");
const PANE_SHADER: &str = include_str!("../pane.frag");
const MIRROR_SHADER: &str = include_str!("../mirror.frag");
const GLASS_SHADER: &str = include_str!("../glass.frag");
/// The notice of a window that finds no room in the view.
const NO_ROOM: &str = "No room in view: in the window list (Tab)";
/// The client name of the voice agent (docs/voice.md, section 10).
const VOICE: &str = "voice";
/// How long after the launcher starts its first new window becomes a
/// pane also beside another flavor, and how long a program pane waits
/// for its window.
const LAUNCHED: Duration = Duration::from_secs(60);
/// One step of a walk key, in logical pixels, and one turn, in radians.
const STEP: f64 = 90.0;
const TURN: f64 = 0.06;
/// The look of a drag, in radians per logical pixel.
const LOOK_RATE: f64 = 0.004;
/// The walk of the wheel, per logical pixel of scroll.
const WHEEL: f64 = 4.0;
/// The glide of a key step and of a fly-in or fly-out, in milliseconds.
const STEP_MS: f64 = 160.0;
const FLY_MS: f64 = 380.0;
/// The frame time of a glide.
const FRAME: Duration = Duration::from_millis(16);
/// How often a plugin pane looks again for the node of its plugin.
const RETRY: Duration = Duration::from_secs(1);
/// The smallest and the largest side of a pane, in logical pixels.
const PANE_MIN: f64 = 100.0;
const PANE_MAX: f64 = 8192.0;

/// A spatial flavor: walk in a foggy space with program panes.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The settings file. Without it, chora-construct reads
    /// $XDG_CONFIG_HOME/chora/chora-construct.yaml if it exists, or the
    /// default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default settings and exit.
    #[arg(long)]
    print_config: bool,
    /// Wait beside another flavor until `construct.load`, and show its
    /// windows without taking them.
    #[arg(long)]
    overlay: bool,
}

/// What a pane shows.
#[derive(Clone)]
enum Content {
    /// A window; its command if the construct started it.
    Window { command: Option<String> },
    /// The node of a plugin that it tags `meta:pane=SCENE`.
    Plugin { plugin: String, scene: String },
}

/// A pane in the space.
struct Pane {
    /// The short label for voice, unique in the space.
    name: String,
    /// The node that the pane shows: the surface of a window, or the
    /// node of a plugin.
    source: String,
    content: Content,
    /// The centre on the ground, (x, z).
    at: [f64; 2],
    /// The turn of its face about the y axis (F8): it faces the camera
    /// where the pane was placed, moved or flown into, and stays when
    /// the camera walks or looks around.
    yaw: f64,
    /// The size in logical pixels.
    size: [f64; 2],
    /// A window of another client, taken on request (F30): the pane goes
    /// without a close request, and the window stays with its owner.
    taken: bool,
    /// A window that the flavor attached and hides under the space.
    attached: bool,
}

/// A window of the window list (F29): the label, the surface, its owner
/// and where its pane is: `shown`, `behind`, `far`, `aside` or `none`.
struct Entry {
    name: String,
    surface: String,
    owner: Option<String>,
    pane: &'static str,
    size: Option<[f64; 2]>,
}

impl Entry {
    fn json(&self) -> Value {
        json!({"name": self.name, "surface": self.surface, "owner": self.owner, "pane": self.pane})
    }
}

/// How a pane holds its window.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Hold {
    /// The construct owns the window and hides it under the space.
    Own,
    /// A window that the construct started beside another flavor, which
    /// owns it; the pane only shows it.
    Shown,
    /// A window of another client, taken on request (F30): the pane only
    /// shows it, and goes without a close request.
    Taken,
}

/// A row of the window list: a window, or a window manager to give the
/// window of the list to.
struct Row {
    label: String,
    note: String,
    /// The surface, or the client name of the window manager.
    target: String,
}

/// The open window list: the selected row, and the surface to give, for
/// the rows of the window managers.
struct ListState {
    sel: usize,
    give: Option<String>,
}

/// A unique label from `base`, at most 32 characters, with `-2`, `-3`
/// and on after it.
fn unique(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let base: String = base.trim().chars().take(32).collect();
    let base = if base.is_empty() { "pane".into() } else { base };
    if !taken(&base) {
        return base;
    }
    (2..=10_000)
        .map(|k| format!("{base}-{k}"))
        .find(|n| !taken(n))
        .unwrap_or_default()
}

impl Pane {
    fn node(&self) -> String {
        match self.content {
            Content::Window { .. } => format!("construct-pane-{}", self.source),
            Content::Plugin { .. } => format!("construct-plugin-{}", self.name),
        }
    }

    fn mirror(&self) -> String {
        format!("{}-mirror", self.node())
    }

    /// The commands that remove the nodes of the pane.
    fn remove(&self) -> [Command; 2] {
        [self.node(), self.mirror()].map(|id| Command::Remove { id, to: None })
    }

    /// Puts the pane at `at`, its face to the camera at `eye`.
    fn put(&mut self, at: [f64; 2], eye: [f64; 2]) {
        self.at = at;
        self.yaw = space::facing(eye, at);
    }

    fn window(&self) -> Option<&str> {
        matches!(self.content, Content::Window { .. }).then_some(self.source.as_str())
    }

    /// The pane as a file has it; a window that the construct did not
    /// start has no command and stays out.
    fn file(&self) -> Option<PaneFile> {
        let (kind, command, plugin, scene) = match &self.content {
            Content::Window { command } => (Kind::Program, Some(command.clone()?), None, None),
            Content::Plugin { plugin, scene } => (
                Kind::Plugin,
                None,
                Some(plugin.clone()),
                Some(scene.clone()),
            ),
        };
        Some(PaneFile {
            kind,
            command,
            plugin,
            scene,
            at: Some(self.at),
            yaw: Some(self.yaw),
            size: Some(self.size),
        })
    }
}

/// A pane of a construct file that does not show yet: a program whose
/// window has not come, with the activation token that finds it, or a
/// plugin whose node is missing.
struct Waiting {
    token: Option<String>,
    pane: PaneFile,
    since: Instant,
    /// A program of a key, not of a file: its pane comes only when the
    /// view has room (C7).
    key: bool,
}

impl Waiting {
    fn new(token: Option<String>, pane: PaneFile) -> Self {
        Self {
            token,
            pane,
            since: Instant::now(),
            key: false,
        }
    }

    /// A program whose window did not come within [`LAUNCHED`]: a window
    /// without the token, such as one of X11 or of D-Bus activation,
    /// never matches. A plugin pane waits for its plugin.
    fn stale(&self) -> bool {
        self.pane.kind == Kind::Program && self.since.elapsed() >= LAUNCHED
    }

    /// The pane to save: a plugin pane only. A program that waits may
    /// already show in a window without the token.
    fn file(&self) -> Option<&PaneFile> {
        (self.pane.kind == Kind::Plugin).then_some(&self.pane)
    }
}

/// The grabbed bindings: the drag with the modifier, which the flavor
/// grabs so that it shares it with the window managers of other outputs
/// (D55), and the fly-out.
#[derive(Clone, Copy)]
enum Grab {
    Drag,
    FlyOut,
}

/// A drag with the modifier: the pane, the press and its centre then.
struct Moving {
    name: String,
    from: [f64; 2],
    at: [f64; 2],
}

/// An eased camera move; at its end it may snap into a pane.
struct Glide {
    from: Pose,
    to: Pose,
    start: Instant,
    ms: f64,
    into: Option<Flown>,
}

/// The pose with the snap: the window of the pane that the camera flew
/// into, and the pose to fly back to.
struct Flown {
    surface: String,
    back: Pose,
    /// Whether the window is attached, so that it shows itself.
    attached: bool,
}

struct Construct {
    rt: Runtime<Grab>,
    settings: settings::File<Config>,
    /// Beside another flavor: no windows of its own, and a space only
    /// between `construct.load` and `construct.leave`.
    overlay: bool,
    /// Whether the space shows.
    active: bool,
    /// The loaded construct, and the leash radius that its file sets;
    /// without one, the settings give it.
    current: Option<String>,
    radius: Option<f64>,
    waiting: Vec<Waiting>,
    moving: Option<Moving>,
    fog_shader: String,
    pane_shader: String,
    mirror_shader: String,
    glass_shader: String,
    /// The launcher: the token and the process of its own window, which
    /// is no pane,
    /// that window, and when it started.
    launcher: Option<(String, Option<u32>)>,
    launcher_window: Option<String>,
    launched: Option<Instant>,
    output: Option<OutputInfo>,
    lens: Lens,
    pose: Pose,
    glide: Option<Glide>,
    panes: Vec<Pane>,
    /// The pane nodes from the back to the front, as last sent.
    order: Vec<String>,
    flown: Option<Flown>,
    /// The press of a drag: where, the point of the last move, and
    /// whether it moved.
    drag: Option<Drag>,
    list: Option<ListState>,
    /// The window that had the keyboard when an overlay took the output;
    /// it gets it back on leave (F31).
    focus_before: Option<String>,
    /// The voice agent: its state, sentence and question (F28), and the
    /// claim of its indicator.
    voice: Voice,
    /// The sizes of the texts over the space, as the core measures them.
    metrics: Metrics,
    claim: Claim,
    glass: Glass,
    /// The last point of the pointer over the space that the core told:
    /// it tells the place only from a press to its release.
    // ponytail: the label follows the pointer from the last press, drag
    // or release; a hover needs a pointer position event of the core.
    pointer: Option<[f64; 2]>,
    /// The pane that the label names, if it shows.
    labelled: Option<String>,
}

#[allow(clippy::print_stdout, reason = "--print-config prints the default")]
fn main() -> ExitCode {
    let args = Args::parse();
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    runtime::run(NAME, || play(&args))
}

/// Writes a shader to the runtime directory when it differs.
fn shader_file(name: &str, text: &str) -> std::io::Result<String> {
    let runtime =
        std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let path = runtime.join(format!("chora-construct/{name}"));
    if chora_kit::read_text(&path, chora_kit::MAX_TEXT)
        .ok()
        .as_deref()
        != Some(text)
    {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, text)?;
    }
    Ok(path.to_string_lossy().into_owned())
}

fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-construct")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let rt = Runtime::connect(NAME, Vec::new())?;
    let start = Pose::default();
    let mut construct = Construct {
        rt,
        settings,
        overlay: args.overlay,
        active: false,
        current: None,
        radius: None,
        waiting: Vec::new(),
        moving: None,
        fog_shader: shader_file("fog.frag", FOG_SHADER)?,
        pane_shader: shader_file("pane.frag", PANE_SHADER)?,
        mirror_shader: shader_file("mirror.frag", MIRROR_SHADER)?,
        glass_shader: shader_file("glass.frag", GLASS_SHADER)?,
        launcher: None,
        launcher_window: None,
        launched: None,
        output: None,
        lens: Lens::new(1280.0, 800.0),
        pose: start,
        glide: None,
        panes: Vec::new(),
        order: Vec::new(),
        flown: None,
        drag: None,
        list: None,
        focus_before: None,
        voice: Voice::default(),
        metrics: Metrics::default(),
        claim: Claim { due: true },
        glass: Glass::default(),
        pointer: None,
        labelled: None,
    };
    construct.start()?;
    construct.serve()?;
    Ok(())
}

/// The events of the plugin.
const EVENTS: [&str; 16] = [
    "voice.*",
    "client.connected",
    "client.disconnected",
    "effects.dropped",
    "layers.dropped",
    "surface.created",
    "surface.changed",
    "surface.destroyed",
    "surface.detached",
    "output.changed",
    "output.removed",
    "node.key",
    "pointer.pressed",
    "pointer.moved",
    "pointer.released",
    "pointer.scroll",
];

fn create(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

fn update(id: &str, props: PropsUpdate) -> Command {
    Command::Update {
        id: id.into(),
        props,
    }
}

fn vec3(c: [f64; 3]) -> Uniform {
    Uniform::Vector(c.to_vec())
}

/// The nodes of the space on the screen root of an output, bottom
/// first: the sky, the fog, the mirror images, the panes, the windows and
/// the look rect, which takes the keys of the walk.
fn space_nodes(fog_shader: &str, root: &str) -> Vec<Command> {
    let group = |input| PropsUpdate {
        input,
        ..PropsUpdate::default()
    };
    vec![
        create(NAME, NodeKind::Group, root, group(None)),
        create(SKY, NodeKind::Rect, NAME, PropsUpdate::default()),
        create(
            FOG,
            NodeKind::Shader,
            NAME,
            PropsUpdate {
                shader: Some(fog_shader.into()),
                ..PropsUpdate::default()
            },
        ),
        create(MIRRORS, NodeKind::Group, NAME, group(Some(InputMode::Pass))),
        create(PANES, NodeKind::Group, NAME, group(Some(InputMode::Pass))),
        create(WINDOWS, NodeKind::Group, NAME, group(None)),
        create(
            LOOK,
            NodeKind::Rect,
            NAME,
            PropsUpdate {
                color: Some([0.0; 4]),
                ..PropsUpdate::default()
            },
        ),
        Command::FocusNode {
            node: Some(LOOK.into()),
        },
    ]
}

/// The last value of a shadow uniform of fog.frag: the lift in whole
/// pixels, and the yaw in the fraction.
// ponytail: two values in one float, as the core passes at most 16
// uniforms; about 4e-4 rad of yaw at a lift of 1000 px, with highp.
fn shadow_w(lift: f64, yaw: f64) -> f64 {
    let turn = (yaw / std::f64::consts::TAU + 0.5).rem_euclid(1.0);
    lift.floor() + turn * 0.99
}

/// The uniforms of a pane; an update replaces all of them, so each
/// carries the node that the pane shows.
fn pane_uniforms(
    source: &str,
    haze: f64,
    fog: [f64; 3],
    edge: [f64; 4],
) -> BTreeMap<String, Uniform> {
    let win = Uniform::Texture(NodeTexture {
        node: source.into(),
    });
    BTreeMap::from([
        ("win".into(), win),
        ("haze".into(), Uniform::Float(haze)),
        ("fog".into(), vec3(fog)),
        ("edge".into(), Uniform::Vector(edge.to_vec())),
    ])
}

/// The new size of a pane and of its mirror image.
fn resized(pane: &Pane) -> Vec<Command> {
    let size = |id: &str, size| {
        update(
            id,
            PropsUpdate {
                size: Some(size),
                ..PropsUpdate::default()
            },
        )
    };
    vec![
        size(&pane.node(), pane.size),
        size(&pane.mirror(), mirror_size(pane.size)),
    ]
}

/// The uniforms of a mirror image: the pane's node and the strength.
fn mirror_uniforms(source: &str, gloss: f64) -> BTreeMap<String, Uniform> {
    let win = Uniform::Texture(NodeTexture {
        node: source.into(),
    });
    BTreeMap::from([("win".into(), win), ("gloss".into(), Uniform::Float(gloss))])
}

fn mirror_size(size: [f64; 2]) -> [f64; 2] {
    size.map(|s| (s / MIRROR_SCALE).ceil())
}

/// The glow nodes of the panels.
#[derive(Default)]
struct Glass {
    /// After the watchdog took the shaders: no more glow nodes until the
    /// settings change (D57).
    plain: bool,
    /// They hide while a layer of the elements stays out.
    hidden: bool,
}

/// A drag with the button held, in output coordinates.
struct Drag {
    press: [f64; 2],
    last: [f64; 2],
    moved: bool,
}

/// Turns the pose by a drag of `dx`, `dy`, and a running glide with it,
/// so that a walk during the drag goes on. Only the yaw and the pitch
/// change.
fn turn(pose: &mut Pose, glide: Option<&mut Glide>, dx: f64, dy: f64) {
    let by = |p: &mut Pose| {
        p.yaw += dx * LOOK_RATE;
        p.pitch = (p.pitch - dy * LOOK_RATE).clamp(-1.2, 1.2);
    };
    by(pose);
    if let Some(glide) = glide {
        by(&mut glide.from);
        by(&mut glide.to);
    }
}

/// A place moved by `right` and `ahead` as a camera that looks
/// `forward` sees them.
fn moved(at: [f64; 2], [fx, fz]: [f64; 2], right: f64, ahead: f64) -> [f64; 2] {
    [
        at[0] + fx * ahead + fz * right,
        at[1] + fz * ahead - fx * right,
    ]
}

/// The eased share of a glide after `k` of its time.
fn ease(k: f64) -> f64 {
    1.0 - (1.0 - k).powi(3)
}

impl Plugin for Construct {
    type Action = Grab;

    fn runtime(&mut self) -> &mut Runtime<Grab> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::SurfaceCreated {
                surface,
                app_id,
                title,
                to,
                pid,
                token,
                ..
            } => {
                let label = app_id.or(title).unwrap_or_default();
                self.created(&surface, &label, to.as_deref(), pid, token.as_deref())
            }
            Event::SurfaceDetached { surface } => self.detached(&surface),
            Event::SurfaceDestroyed { surface }
                if self.launcher_window.as_deref() == Some(&surface) =>
            {
                self.launcher_window = None;
                self.rt.conn.apply(vec![Command::FocusNode {
                    node: Some(LOOK.into()),
                }])
            }
            Event::SurfaceDestroyed { surface } => self.forget(&surface),
            Event::SurfaceChanged {
                surface,
                size: Some([w, h]),
                ..
            } => self.resize(&surface, [f64::from(w), f64::from(h)]),
            Event::OutputChanged { output }
                if self.output.as_ref().is_none_or(|o| o.name == output.name) =>
            {
                let same = self
                    .output
                    .as_ref()
                    .is_some_and(|o| o.size == output.size && o.position == output.position);
                self.output = Some(output);
                if same { Ok(()) } else { self.build() }
            }
            Event::OutputRemoved { name }
                if self.output.as_ref().is_some_and(|o| o.name == name) =>
            {
                self.output = self.rt.outputs()?.into_iter().find(|o| o.name != name);
                if self.output.is_some() {
                    self.build()?;
                }
                Ok(())
            }
            event => self.other(event),
        }
    }

    #[allow(clippy::too_many_lines, reason = "one arm per method")]
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.waiting.retain(|w| !w.stale());
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        let invalid = |e: String| RpcError::new(RpcError::INVALID_PARAMS, e);
        match method {
            "construct.load" => {
                let Named { name } = runtime::params(params)?;
                let file = file::dir()
                    .and_then(|d| file::load(&d, &name))
                    .map_err(invalid)?;
                self.load(&name, file).map_err(rejected)?;
                Ok(self.list())
            }
            "construct.save" => {
                let Named { name } = runtime::params(params)?;
                let dir = file::dir().map_err(invalid)?;
                file::save(&dir, &name, &self.snapshot()).map_err(invalid)?;
                self.current = Some(name);
                Ok(Value::Null)
            }
            "construct.list" => Ok(self.list()),
            "construct.leave" => {
                if !self.overlay {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        "the construct is the flavor; load another construct instead",
                    ));
                }
                self.leave().map_err(rejected)?;
                Ok(Value::Null)
            }
            "pane.move" => {
                let MovePane { name, right, ahead } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                let to = moved(self.panes[i].at, self.pose.forward(), right, ahead);
                self.move_pane(i, to).map_err(rejected)?;
                Ok(json!({"at": self.panes[i].at}))
            }
            "pane.resize" => {
                let ResizePane { name, scale } = runtime::params(params)?;
                if !(scale.is_finite() && scale > 0.0) {
                    return Err(invalid(format!("scale: {scale} is not above 0")));
                }
                let i = self.pane_named(&name)?;
                self.resize_pane(i, scale).map_err(rejected)?;
                Ok(Value::Null)
            }
            "pane.remove" => {
                let Named { name } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                self.remove_pane(i).map_err(rejected)?;
                Ok(Value::Null)
            }
            "window.list" => {
                let windows = self.windows().map_err(rejected)?;
                Ok(json!({"windows": windows.iter().map(Entry::json).collect::<Vec<_>>()}))
            }
            "window.show" => {
                let Named { name } = runtime::params(params)?;
                if !self.active {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        "no construct shows; construct.load first",
                    ));
                }
                let windows = self.windows().map_err(rejected)?;
                let entry = (windows.into_iter())
                    .find(|e| e.name == name || e.surface == name)
                    .ok_or_else(|| {
                        invalid(format!("no window {name:?}; window.list names them"))
                    })?;
                let pane = self.show(&entry).map_err(rejected)?;
                Ok(json!({"pane": pane}))
            }
            give::METHOD => {
                let Give { surface, to } = runtime::params(params)?;
                self.give(&surface, &to).map_err(rejected)
            }
            "pane.fly" => {
                let Named { name } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                if self.flown.is_some() {
                    self.fly_out().map_err(rejected)?;
                }
                self.fly_in(i);
                Ok(Value::Null)
            }
            _ => Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        self.glass.plain = false;
        self.regrab().map_err(rejected)?;
        self.restyle().map_err(rejected)?;
        self.frame().map_err(rejected)
    }
}

impl Construct {
    /// The events of the voice agent and of the watchdog, then the input.
    fn other(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == VOICE => self.follow(&event, data),
            Event::ClientConnected { name } if name == VOICE => {
                self.claim.due = true;
                self.claim_indicator(false);
                Ok(())
            }
            Event::ClientDisconnected { name } if name == VOICE => {
                self.claim.due = true;
                self.voice = Voice::default();
                self.rt
                    .conn
                    .try_apply(vec![hud::fade(hud::HUD, 0.0, hud::FADE_OUT)])?;
                self.draw_ask()
            }
            Event::EffectsDropped { client } if client == NAME => {
                if !self.glass.plain {
                    eprintln!(
                        "chora-construct: the watchdog took the shaders; the panels stay \
                         plain until the settings change"
                    );
                }
                self.glass.plain = true;
                self.draw_hud()
            }
            Event::LayersDropped { nodes, .. } => {
                if !nodes.is_empty() {
                    eprintln!("chora-construct: layers stay out: {}", nodes.join(" "));
                }
                let ours = [
                    "construct-hud",
                    "construct-ask",
                    "construct-list",
                    "construct-label",
                ];
                let hide = nodes.iter().any(|n| ours.iter().any(|o| n.starts_with(o)));
                if hide != self.glass.hidden {
                    self.glass.hidden = hide;
                    self.draw_hud()?;
                    self.draw_ask()?;
                }
                Ok(())
            }
            event => self.input(event),
        }
    }

    /// The keys and the pointer of the walk, and the fly-out binding.
    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    fn input(&mut self, event: Event) -> Result<(), ClientError> {
        if self.list_input(&event)? {
            return Ok(());
        }
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => match self.settings.value.keys.get(&key) {
                Some(Action::Terminal) => self.start_terminal(),
                Some(Action::Launcher) => self.start_launcher(),
                Some(Action::Close) => return self.close_ahead(),
                Some(Action::Windows) => return self.open_list(),
                Some(&action) => self.act(action),
                None => {}
            },
            Event::InputKey {
                state: PressState::Pressed,
                ..
            } => return self.fly_out(),
            // The grabbed drag comes in scene coordinates, also over the
            // other outputs; the window flown into keeps its presses.
            Event::InputButton {
                state, position, ..
            } if self.flown.is_none() => {
                let at = self.local(position);
                return match state {
                    PressState::Pressed => self.press(at, true),
                    PressState::Released => self.release(at),
                };
            }
            Event::InputMoved { position, .. } => return self.moved(self.local(position)),
            Event::PointerPressed {
                node,
                position,
                modifiers,
                ..
            } if node == LOOK => {
                let held = modifiers.contains(&self.settings.value.modifier);
                return self.press(position, held);
            }
            Event::PointerMoved { node, position, .. } if node == LOOK => {
                return self.moved(position);
            }
            Event::PointerReleased { node, position, .. } if node == LOOK => {
                return self.release(position);
            }
            Event::PointerScroll { node, dy, .. } if node == hud::ASK_GLASS => {
                if let Some(command) = self.voice.scroll(dy) {
                    self.rt.conn.apply(vec![command])?;
                }
            }
            Event::PointerScroll {
                node,
                position,
                dy,
                modifiers,
                ..
            } if node == LOOK => {
                let held = modifiers.contains(&self.settings.value.modifier);
                match self.pane_at(position).filter(|_| held) {
                    Some(i) => return self.resize_pane(i, (-dy * 0.005).exp()),
                    None => self.walk(-dy * WHEEL, 0.0),
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// A press on the space at a point of the output: with the modifier
    /// on a pane, it starts to move the pane; else a drag looks around.
    fn press(&mut self, at: [f64; 2], held: bool) -> Result<(), ClientError> {
        self.pointer = Some(at);
        // A window of an overlay may have taken the keyboard; a press on
        // the space takes it back.
        if self.overlay && self.flown.is_none() {
            self.rt.conn.apply(vec![Command::FocusNode {
                node: Some(LOOK.into()),
            }])?;
        }
        if let Some(i) = self.pane_at(at).filter(|_| held) {
            let pane = &self.panes[i];
            self.moving = Some(Moving {
                name: pane.name.clone(),
                from: at,
                at: pane.at,
            });
        } else {
            self.drag = Some(Drag {
                press: at,
                last: at,
                moved: false,
            });
        }
        Ok(())
    }

    fn moved(&mut self, at: [f64; 2]) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if self.moving.is_some() {
            return self.drag_pane(at);
        }
        self.look(at)
    }

    /// The end of a press: a moved pane drops, a click flies in.
    fn release(&mut self, at: [f64; 2]) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if let Some(moving) = self.moving.take() {
            return self.drop_pane(&moving, at);
        }
        let click = self.drag.take().is_some_and(|d| !d.moved);
        if let Some(i) = self.pane_at(at).filter(|_| click) {
            self.fly_in(i);
            return Ok(());
        }
        let commands = self.label_commands();
        self.rt.conn.apply(commands)
    }

    /// The top left corner of the output in the scene.
    fn origin(&self) -> [f64; 2] {
        (self.output.as_ref()).map_or([0.0; 2], |o| o.position.map(f64::from))
    }

    /// A point of the scene in the coordinates of the output.
    fn local(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        let [ox, oy] = self.origin();
        [x - ox, y - oy]
    }

    /// The drop of a moved pane at a point of the output (D55). Over a
    /// plugin pane, its window goes to that plugin; off the output, the
    /// pane goes back to where it was, and its window to the window
    /// manager whose claim holds the point. A window of another client
    /// goes back to its owner instead. A refused window stays a pane.
    fn drop_pane(&mut self, moving: &Moving, at: [f64; 2]) -> Result<(), ClientError> {
        let Ok(i) = self.pane_named(&moving.name) else {
            return Ok(());
        };
        if self.overlay {
            return Ok(());
        }
        let inside = (0.0..self.lens.w).contains(&at[0]) && (0.0..self.lens.h).contains(&at[1]);
        let to = if inside {
            let plugin =
                |p: &Pane| p.name != moving.name && matches!(p.content, Content::Plugin { .. });
            match self.pane_hit(at, plugin).map(|k| &self.panes[k].content) {
                Some(Content::Plugin { plugin, .. }) => Some(plugin.clone()),
                _ => None,
            }
        } else {
            self.panes[i].at = moving.at;
            let [ox, oy] = self.origin();
            (self.rt.conn)
                .claimant_at([at[0] + ox, at[1] + oy])
                .map_err(ClientError::Rpc)?
        };
        let pane = &self.panes[i];
        if let (Some(to), Some(surface)) = (to, pane.window().map(String::from)) {
            let given = if pane.taken {
                self.forget(&surface)
            } else {
                self.give(&surface, &to).map(drop)
            };
            if let Err(err) = given {
                eprintln!("chora-construct: {surface} stays: {err}");
            }
        }
        self.frame()
    }

    /// The keys and the clicks of the open window list; whether it took
    /// the event. A press on the space closes the list and goes on.
    fn list_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        let Some(sel) = self.list.as_ref().map(|l| l.sel) else {
            return Ok(false);
        };
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => {
                let toggle = self.settings.value.keys.get(key) == Some(&Action::Windows);
                match key.as_str() {
                    "Escape" => self.close_list(),
                    _ if toggle => self.close_list(),
                    "Up" => self.select(sel.saturating_sub(1)),
                    "Down" => self.select(sel + 1),
                    "Return" | "KP_Enter" => self.choose(sel),
                    "g" => self.give_menu(sel),
                    _ => Ok(()),
                }?;
            }
            Event::PointerPressed { node, .. } if node.starts_with(LIST) => {
                let row =
                    (node.strip_prefix(LIST_ROW)).and_then(|r| r.split('-').next()?.parse().ok());
                if let Some(k) = row {
                    self.choose(k)?;
                }
            }
            Event::PointerPressed { node, .. } if node == LOOK => {
                self.close_list()?;
                return Ok(false);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// A drag with the modifier moves a pane: sideways along its face,
    /// and up the screen away from the camera, at the pace of the
    /// pointer at the pane's depth.
    fn drag_pane(&mut self, position: [f64; 2]) -> Result<(), ClientError> {
        let Some(moving) = &self.moving else {
            return Ok(());
        };
        let Ok(i) = self.pane_named(&moving.name) else {
            return Ok(());
        };
        let k = space::depth(&self.pose, moving.at).max(self.lens.focal * 0.1) / self.lens.focal;
        let (dx, dy) = (position[0] - moving.from[0], position[1] - moving.from[1]);
        let [fx, fz] = self.pose.forward();
        let (right, ahead) = (dx * k, -dy * k);
        let at = moving.at;
        let to = [
            at[0] + fx * ahead + fz * right,
            at[1] + fz * ahead - fx * right,
        ];
        self.move_pane(i, to)
    }

    /// The index of the pane with the label `name`.
    fn pane_named(&self, name: &str) -> Result<usize, RpcError> {
        (self.panes.iter().position(|p| p.name == name)).ok_or_else(|| {
            RpcError::new(
                RpcError::INVALID_PARAMS,
                format!("no pane {name:?}; construct.list names them"),
            )
        })
    }

    /// Moves a pane to `to`, inside the leash (C4).
    fn move_pane(&mut self, i: usize, to: [f64; 2]) -> Result<(), ClientError> {
        let len = to[0].hypot(to[1]);
        let r = self.radius();
        let to = if len > r {
            [to[0] * r / len, to[1] * r / len]
        } else {
            to
        };
        self.panes[i].put(to, [self.pose.x, self.pose.z]);
        self.frame()
    }

    /// Scales a pane. A window of the flavor gets the new size, so that
    /// its content stays at 1:1; other panes stretch.
    fn resize_pane(&mut self, i: usize, scale: f64) -> Result<(), ClientError> {
        let pane = &mut self.panes[i];
        let size = pane
            .size
            .map(|s| (s * scale).round().clamp(PANE_MIN, PANE_MAX));
        pane.size = size;
        let mut commands = resized(pane);
        if let Some(surface) = pane.window().filter(|_| pane.attached) {
            commands.push(configure(surface, size));
        }
        self.rt.conn.apply(commands)?;
        self.frame()
    }

    /// Removes a pane; its window gets a close request, so a program with
    /// unsaved state can ask the user first (F13). The window of another
    /// flavor's owner gets it through `node.requested`. The pane goes
    /// when the window does: a window that refuses stays a pane.
    fn remove_pane(&mut self, i: usize) -> Result<(), ClientError> {
        if self.panes[i].taken {
            // A taken window goes back to its owner as it was (F31).
            let surface = self.panes[i].source.clone();
            return self.forget(&surface);
        }
        let Some(surface) = self.panes[i].window().map(String::from) else {
            let pane = self.panes.remove(i);
            self.order.retain(|n| *n != pane.node());
            self.rt.conn.apply(pane.remove().into())?;
            return self.frame();
        };
        if let Some(err) = self.rt.conn.try_apply(vec![Command::Close { surface }])? {
            eprintln!("chora-construct: close: {err}");
        }
        Ok(())
    }

    /// Builds the space, then takes the events and the free windows.
    fn start(&mut self) -> Result<(), ClientError> {
        // A restart with resume finds its old nodes; the space starts anew.
        if !self.rt.conn.query(&format!("id:{NAME}"))?.is_empty() {
            self.rt.conn.apply(vec![Command::Remove {
                id: NAME.into(),
                to: None,
            }])?;
        }
        self.output = self.rt.outputs()?.into_iter().next();
        let mut methods = exported_methods();
        methods.extend(settings::File::<Config>::methods());
        self.rt.announce(&EVENTS, methods)?;
        if self.overlay {
            return Ok(());
        }
        self.active = true;
        self.build()?;
        self.regrab()?;
        self.claim_indicator(false);
        for node in self.rt.conn.query("kind:surface")? {
            let ours = node.to.as_deref().is_none_or(|t| t == NAME);
            if node.owner.is_none() && ours {
                let info = node.surface.unwrap_or_default();
                let label = info.app_id.or(info.title).unwrap_or_default();
                self.adopt(&node.id, &label, None, Hold::Own, false)?;
            }
        }
        let start = self.settings.value.start.clone();
        if !start.is_empty() {
            match file::dir().and_then(|d| file::load(&d, &start)) {
                Ok(file) => self.load(&start, file)?,
                Err(err) => eprintln!("chora-construct: start: {err}"),
            }
        }
        Ok(())
    }

    /// Handles events; during a glide, frames come every 16 ms.
    fn serve(&mut self) -> Result<(), ClientError> {
        loop {
            let plugins = self.waiting.iter().any(|w| w.pane.kind == Kind::Plugin);
            let wait = if self.glide.is_some() {
                FRAME
            } else if plugins {
                RETRY
            } else {
                Duration::from_secs(3600)
            };
            let wait = (self.voice.deadline()).map_or(wait, |t| {
                wait.min(t.saturating_duration_since(Instant::now()))
            });
            match self.rt.conn.next_event_timeout(wait)? {
                Some(event) => runtime::dispatch(self, event)?,
                None if plugins => self.retry_plugins()?,
                None => {}
            }
            self.tick()?;
            if self.glide.is_some() {
                self.step_glide()?;
            }
        }
    }

    /// The group of the space with the fog, the panes and the look rect,
    /// on the screen root of the output, at its size; the windows stay.
    /// As a flavor, it claims the output, so that new windows come to it
    /// also beside another window manager (D55).
    #[allow(clippy::too_many_lines, reason = "one node after the other")]
    fn build(&mut self) -> Result<(), ClientError> {
        let Some(output) = &self.output else {
            return Ok(());
        };
        if !self.active {
            return Ok(());
        }
        let [w, h] = output.size.map(f64::from);
        let [x, y] = output.position.map(f64::from);
        self.lens = Lens::new(w, h);
        let root = overlay_id(&output.name);
        let mut commands = Vec::new();
        match self.rt.conn.query(&format!("id:{NAME}"))?.first() {
            None => {
                commands = space_nodes(&self.fog_shader, &root);
                if !self.glass.plain {
                    commands.push(create(
                        FOCUS,
                        NodeKind::Shader,
                        PANES,
                        PropsUpdate {
                            shader: Some(self.glass_shader.clone()),
                            visible: Some(false),
                            ..PropsUpdate::default()
                        },
                    ));
                }
                self.order.clear();
                commands.extend(self.styled().groups(NAME));
                self.labelled = None;
            }
            Some(node) if node.parent.as_deref() != Some(root.as_str()) => {
                commands.push(Command::Move {
                    id: NAME.into(),
                    parent: Some(root),
                    index: None,
                });
            }
            Some(_) => {}
        }
        let claim = (!self.overlay).then_some([x, y, w, h]);
        commands.extend([
            update(
                NAME,
                PropsUpdate {
                    claim: Some(claim),
                    ..PropsUpdate::default()
                },
            ),
            update(
                SKY,
                PropsUpdate {
                    size: Some([w, h]),
                    ..PropsUpdate::default()
                },
            ),
            update(
                FOG,
                PropsUpdate {
                    size: Some([w / FOG_SCALE, h / FOG_SCALE]),
                    transform: Some(Transform::scale(FOG_SCALE, FOG_SCALE, 1.0)),
                    ..PropsUpdate::default()
                },
            ),
            update(
                LOOK,
                PropsUpdate {
                    size: Some([w, h]),
                    ..PropsUpdate::default()
                },
            ),
        ]);
        self.rt.conn.apply(commands)?;
        self.draw_hud()?;
        self.draw_ask()?;
        self.frame()
    }

    /// A unique label from `base`: the app id or title of a window, or
    /// the scene of a plugin pane, with `-2`, `-3` and on after it.
    fn label(&self, base: &str) -> String {
        unique(base, |n| self.panes.iter().any(|p| p.name == n))
    }

    /// The place of its file, or of a new pane of `size` in the view
    /// (C7); with `force`, ahead also when the view is full.
    fn place(&self, at: Option<[f64; 2]>, size: [f64; 2], force: bool) -> Option<[f64; 2]> {
        let distance = self.settings.value.pane_distance;
        let others: Vec<_> = self.panes.iter().map(|p| (p.at, p.size, p.yaw)).collect();
        at.or_else(|| space::place(&self.lens, &self.pose, distance, size, &others))
            .or_else(|| {
                let widths: Vec<_> = others.iter().map(|(at, size, _)| (*at, size[0])).collect();
                force.then(|| space::beside(&self.pose, distance, size[0], &widths))
            })
    }

    /// The yaw of a new pane at `at`: its file's, else facing the camera.
    fn yaw_at(&self, file: Option<&PaneFile>, at: [f64; 2]) -> f64 {
        (file.and_then(|f| f.yaw)).unwrap_or_else(|| space::facing([self.pose.x, self.pose.z], at))
    }

    /// The shader nodes of a new pane: the pane and its mirror image,
    /// both hidden until the frame.
    fn pane_nodes(&self, pane: &Pane) -> [Command; 2] {
        let fog = self.settings.value.theme.palette().horizon;
        let node = |id: &str, parent, size, shader: &String, uniforms| {
            create(
                id,
                NodeKind::Shader,
                parent,
                PropsUpdate {
                    size: Some(size),
                    shader: Some(shader.clone()),
                    uniforms: Some(uniforms),
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
            )
        };
        [
            node(
                &pane.mirror(),
                MIRRORS,
                mirror_size(pane.size),
                &self.mirror_shader,
                mirror_uniforms(&pane.source, 0.0),
            ),
            node(
                &pane.node(),
                PANES,
                pane.size,
                &self.pane_shader,
                pane_uniforms(&pane.source, 0.0, fog, [0.0; 4]),
            ),
        ]
    }

    /// A new window becomes a pane, ahead of the camera or where its file
    /// puts it. The construct hides its own window under the space, and
    /// the pane shows it through a node texture; a window of another owner
    /// only shows.
    #[allow(clippy::too_many_arguments, reason = "the window and how it comes")]
    fn adopt(
        &mut self,
        surface: &str,
        label: &str,
        file: Option<&PaneFile>,
        hold: Hold,
        force: bool,
    ) -> Result<(), ClientError> {
        if !self.active || self.panes.iter().any(|p| p.window() == Some(surface)) {
            return Ok(());
        }
        let size = (file.and_then(|f| f.size)).unwrap_or([self.lens.w, self.lens.h]);
        let Some(at) = self.place(file.and_then(|f| f.at), size, force) else {
            // No room in the view: the window waits in the window list.
            self.voice.notice(NO_ROOM, Instant::now());
            return self.draw_hud();
        };
        let pane = Pane {
            name: self.label(label),
            source: surface.into(),
            content: Content::Window {
                command: file.and_then(|f| f.command.clone()),
            },
            at,
            yaw: self.yaw_at(file, at),
            size,
            taken: hold == Hold::Taken,
            attached: hold == Hold::Own,
        };
        let mut commands = Vec::new();
        if pane.attached {
            commands = vec![
                Command::Attach {
                    surface: surface.into(),
                    parent: WINDOWS.into(),
                },
                update(
                    surface,
                    PropsUpdate {
                        opacity: Some(0.0),
                        ..PropsUpdate::default()
                    },
                ),
                configure(surface, size),
            ];
        }
        commands.extend(self.pane_nodes(&pane));
        // The surface may have closed in between; the frame then fixes
        // nothing up, and the destroy event removes the pane.
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            eprintln!("chora-construct: {surface}: {err}");
            return Ok(());
        }
        self.panes.push(pane);
        self.frame()
    }

    /// A new window: the launcher's own window, a pane of the construct,
    /// the program of the launcher, or, for the flavor, a free window.
    #[allow(clippy::too_many_arguments, reason = "the fields of surface.created")]
    fn created(
        &mut self,
        surface: &str,
        label: &str,
        to: Option<&str>,
        pid: Option<u32>,
        token: Option<&str>,
    ) -> Result<(), ClientError> {
        // A free window with the token of the launcher is the launcher
        // itself: it shows, but is no pane.
        let free = to.is_none_or(|t| t == NAME);
        self.waiting.retain(|w| !w.stale());
        let started = (self.launcher.as_ref()).filter(|(t, _)| token == Some(t.as_str()));
        if let Some(&(_, launcher_pid)) = started {
            self.launcher = None;
            if !launchers_own(pid, launcher_pid) {
                // fuzzel passes its own token on to the program it starts.
                self.launched = None;
                return self.adopt(surface, label, None, self.started(), false);
            }
            if free && !self.overlay {
                return self.show_launcher(surface);
            }
            return Ok(());
        }
        let i = (self.waiting.iter()).position(|w| token.is_some() && w.token.as_deref() == token);
        let waited = i.map(|i| self.waiting.remove(i));
        // A pane of a construct file comes also when the view is full.
        let force = waited.as_ref().is_some_and(|w| !w.key);
        let pane = waited.map(|w| w.pane);
        // The first other window after the launcher is its program.
        let launched =
            pane.is_none() && (self.launched.take()).is_some_and(|t| t.elapsed() < LAUNCHED);
        // A flavor takes the free windows; an overlay only shows those of
        // its construct, which the other flavor takes.
        if pane.is_some() || launched || (free && !self.overlay) {
            return self.adopt(surface, label, pane.as_ref(), self.started(), force);
        }
        Ok(())
    }

    /// The window of a launcher shows over the space, in the middle of
    /// the output once its size comes, with the keyboard; its close gives
    /// the keyboard back to the walk.
    fn show_launcher(&mut self, surface: &str) -> Result<(), ClientError> {
        self.launcher_window = Some(surface.into());
        let commands = vec![
            Command::Attach {
                surface: surface.into(),
                parent: NAME.into(),
            },
            Command::Focus {
                surface: Some(surface.into()),
            },
        ];
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            eprintln!("chora-construct: launcher {surface}: {err}");
        }
        Ok(())
    }

    /// Starts the program of the key `terminal`; its window becomes a
    /// pane ahead, as a program pane of a construct.
    fn start_terminal(&mut self) {
        self.waiting.retain(|w| !w.stale());
        let command = self.settings.value.terminal.clone();
        let token = chora_kit::launch(&command).map(|(t, _)| t);
        let pane = PaneFile {
            kind: Kind::Program,
            command: Some(command),
            plugin: None,
            scene: None,
            at: None,
            yaw: None,
            size: None,
        };
        self.waiting.push(Waiting {
            key: true,
            ..Waiting::new(token, pane)
        });
    }

    /// Starts the launcher. Its own window, if it has one and the
    /// token, is no pane; the next window after it is.
    fn start_launcher(&mut self) {
        self.launcher = chora_kit::launch(&self.settings.value.launcher);
        self.launched = Some(Instant::now());
    }

    /// Asks the window of the pane ahead to close; its pane goes when
    /// it does. A plugin pane goes at once.
    fn close_ahead(&mut self) -> Result<(), ClientError> {
        match self.pane_ahead() {
            Some(i) => self.remove_pane(i),
            None => Ok(()),
        }
    }

    fn resize(&mut self, surface: &str, size: [f64; 2]) -> Result<(), ClientError> {
        if self.launcher_window.as_deref() == Some(surface) {
            return self.rt.conn.apply(vec![self.snap_place(size, surface)]);
        }
        let Some(pane) = (self.panes.iter_mut()).find(|p| p.window() == Some(surface)) else {
            return Ok(());
        };
        if pane.size.iter().zip(size).all(|(a, b)| (a - b).abs() < 0.5) {
            return Ok(());
        }
        pane.size = size;
        let mut commands = resized(pane);
        if (self.flown.as_ref()).is_some_and(|f| f.surface == surface && f.attached) {
            commands.push(self.snap_place(size, surface));
        }
        self.rt.conn.apply(commands)?;
        self.frame()
    }

    /// A window closed or left: its pane goes. A pane that the camera
    /// is in flies back out first.
    fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some(i) = (self.panes.iter()).position(|p| p.window() == Some(surface)) else {
            return Ok(());
        };
        let pane = self.panes.remove(i);
        self.order.retain(|n| *n != pane.node());
        self.rt.conn.apply(pane.remove().into())?;
        if let Some(flown) = self.flown.take_if(|f| f.surface == surface) {
            self.unsnap(None)?;
            self.pose = flown.back;
        }
        if self
            .glide
            .as_ref()
            .is_some_and(|g| g.into.as_ref().is_some_and(|f| f.surface == surface))
        {
            self.glide = None;
        }
        self.frame()
    }

    /// The leash radius: the construct file's, or the settings'.
    fn radius(&self) -> f64 {
        self.radius.unwrap_or(self.settings.value.radius)
    }

    /// The target of the walk: the end of the glide, or the pose.
    fn target(&self) -> Pose {
        self.glide.as_ref().map_or(self.pose, |g| g.to)
    }

    fn glide_to(&mut self, to: Pose, ms: f64, into: Option<Flown>) {
        self.glide = Some(Glide {
            from: self.pose,
            to,
            start: Instant::now(),
            ms,
            into,
        });
    }

    fn act(&mut self, action: Action) {
        let mut t = self.target();
        match action {
            Action::Forward => return self.walk(STEP, 0.0),
            Action::Back => return self.walk(-STEP, 0.0),
            Action::Left => return self.walk(0.0, -STEP),
            Action::Right => return self.walk(0.0, STEP),
            Action::TurnLeft => t.yaw -= TURN,
            Action::TurnRight => t.yaw += TURN,
            Action::LookUp => t.pitch = (t.pitch + TURN).min(1.2),
            Action::LookDown => t.pitch = (t.pitch - TURN).max(-1.2),
            Action::Home => {
                // Back to the start point, facing the first pane (F6).
                let yaw = self.panes.first().map_or(0.0, |p| p.at[0].atan2(p.at[1]));
                let home = Pose {
                    x: 0.0,
                    z: 0.0,
                    yaw,
                    pitch: 0.0,
                };
                return self.glide_to(home, FLY_MS, None);
            }
            // input() handles them.
            Action::Terminal | Action::Launcher | Action::Close | Action::Windows => return,
            Action::FlyIn => {
                if let Some(i) = self.pane_ahead() {
                    self.fly_in(i);
                }
                return;
            }
        }
        self.glide_to(t, STEP_MS, None);
    }

    /// Walks `ahead` forward and `aside` to the right, on the leash.
    fn walk(&mut self, ahead: f64, aside: f64) {
        if self.flown.is_some() {
            return;
        }
        let mut t = self.target();
        let [fx, fz] = t.forward();
        let step = [fx * ahead + fz * aside, fz * ahead - fx * aside];
        [t.x, t.z] = space::leash([t.x, t.z], step, self.radius());
        self.glide_to(t, STEP_MS, None);
    }

    /// A drag looks around: sideways turns, up and down tilts. It turns
    /// by the move since the last event and keeps the place, so a walk
    /// during the drag stays.
    fn look(&mut self, p: [f64; 2]) -> Result<(), ClientError> {
        let Some(drag) = &mut self.drag else {
            return Ok(());
        };
        let (dx, dy) = (p[0] - drag.press[0], p[1] - drag.press[1]);
        drag.moved |= dx.abs() + dy.abs() >= 4.0;
        if !drag.moved {
            return Ok(());
        }
        let (dx, dy) = (p[0] - drag.last[0], p[1] - drag.last[1]);
        drag.last = p;
        // A fly-in ends at its pane; a drag stops it.
        self.glide.take_if(|g| g.into.is_some());
        turn(&mut self.pose, self.glide.as_mut(), dx, dy);
        self.frame()
    }

    /// The pane under a point of the output, the nearest one.
    fn pane_at(&self, p: [f64; 2]) -> Option<usize> {
        self.pane_hit(p, |_| true)
    }

    /// The nearest pane under a point of the output of those that `keep`.
    fn pane_hit(&self, p: [f64; 2], keep: impl Fn(&Pane) -> bool) -> Option<usize> {
        (self.panes.iter().enumerate())
            .filter(|(_, pane)| keep(pane))
            .filter_map(|(i, pane)| {
                space::hit(&self.lens, &self.pose, p, pane.at, pane.size, pane.yaw).map(|t| (i, t))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// The pane nearest the middle of the view, in front of the camera.
    fn pane_ahead(&self) -> Option<usize> {
        let [fx, fz] = self.pose.forward();
        (self.panes.iter().enumerate())
            .filter_map(|(i, p)| {
                let (dx, dz) = (p.at[0] - self.pose.x, p.at[1] - self.pose.z);
                let cos = (dx * fx + dz * fz) / dx.hypot(dz).max(f64::EPSILON);
                (cos > 0.3).then_some((i, cos))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// Flies in front of the face of a pane, which keeps its yaw. The
    /// window of a window pane then takes over; a plugin pane stays a
    /// pane, at 1:1. A fly-in from inside a pane, or during a fly-in,
    /// keeps the pose of the first to go back to.
    fn fly_in(&mut self, i: usize) {
        let pane = &self.panes[i];
        let to = space::fly_in(&self.lens, pane.at, pane.yaw);
        let gliding_in = self.glide.as_ref().and_then(|g| g.into.as_ref());
        let back = (self.flown.as_ref().or(gliding_in)).map_or_else(|| self.target(), |f| f.back);
        let into = (pane.window()).map(|surface| Flown {
            surface: surface.into(),
            back,
            attached: pane.attached,
        });
        self.glide_to(to, FLY_MS, into);
    }

    fn step_glide(&mut self) -> Result<(), ClientError> {
        let Some(glide) = &self.glide else {
            return Ok(());
        };
        let k = (glide.start.elapsed().as_secs_f64() * 1000.0 / glide.ms).min(1.0);
        self.pose = glide.from.lerp(&glide.to, ease(k));
        if k < 1.0 {
            return self.frame();
        }
        self.pose = glide.to;
        let into = self.glide.take().and_then(|g| g.into);
        self.frame()?;
        if let Some(flown) = into {
            self.snap(flown)?;
        }
        Ok(())
    }

    /// The transform of the window that the camera flew into: whole
    /// pixels in the middle of the output, so that it shows 1:1.
    fn snap_place(&self, size: [f64; 2], surface: &str) -> Command {
        let x = ((self.lens.w - size[0]) / 2.0).round();
        let y = ((self.lens.h - size[1]) / 2.0).round();
        update(
            surface,
            PropsUpdate {
                transform: Some(Transform::translate(x, y, 0.0)),
                ..PropsUpdate::default()
            },
        )
    }

    /// The end of a fly-in: the window itself shows over the space,
    /// takes the pointer and the keyboard, and the fly-out key is grabbed.
    /// A window of another owner stays where its owner has it: the pane
    /// shows it at 1:1, and the owner gets the request for the keyboard
    /// (D31).
    fn snap(&mut self, flown: Flown) -> Result<(), ClientError> {
        let surface = flown.surface.clone();
        let Some(pane) = self.panes.iter().find(|p| p.window() == Some(&surface)) else {
            return Ok(());
        };
        let commands = if pane.attached {
            vec![
                self.snap_place(pane.size, &surface),
                update(
                    &surface,
                    PropsUpdate {
                        opacity: Some(1.0),
                        ..PropsUpdate::default()
                    },
                ),
                Command::Move {
                    id: surface.clone(),
                    parent: None,
                    index: None,
                },
                update(
                    LOOK,
                    PropsUpdate {
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                ),
                Command::Focus {
                    surface: Some(surface.clone()),
                },
            ]
        } else {
            vec![Command::Focus {
                surface: Some(surface),
            }]
        };
        self.rt.conn.apply(commands)?;
        self.flown = Some(flown);
        self.draw_hud()?;
        // The label hides, and the focus moves to the pane flown into.
        self.frame()?;
        self.regrab()
    }

    /// Grabs the drag with the modifier as the flavor, and the fly-out
    /// binding while the camera is in a pane. Without the grab of the
    /// drag, which another client may hold or the core refuses for Shift
    /// (D45), the presses on the space still move panes.
    fn regrab(&mut self) -> Result<(), ClientError> {
        let parse = |b: String| {
            b.parse().map_err(|e| {
                ClientError::Rpc(RpcError::new(RpcError::INVALID_PARAMS, format!("{b}: {e}")))
            })
        };
        self.rt.ungrab_all()?;
        if self.active && !self.overlay {
            let drag = parse(format!("{}+BTN_LEFT", self.settings.value.modifier))?;
            if let Err(err) = self.rt.grab(drag, Grab::Drag) {
                eprintln!("chora-construct: the drag: {err}");
            }
        }
        if self.flown.is_some() {
            let fly_out = parse(self.settings.value.fly_out.clone())?;
            self.rt.grab(fly_out, Grab::FlyOut)?;
        }
        Ok(())
    }

    /// Hides the window again and gives the keys back to the walk.
    fn unsnap(&mut self, surface: Option<&str>) -> Result<(), ClientError> {
        let mut commands = vec![
            update(
                LOOK,
                PropsUpdate {
                    visible: Some(true),
                    ..PropsUpdate::default()
                },
            ),
            Command::FocusNode {
                node: Some(LOOK.into()),
            },
        ];
        if let Some(surface) = surface {
            commands.insert(
                0,
                update(
                    surface,
                    PropsUpdate {
                        opacity: Some(0.0),
                        transform: Some(Transform::translate(0.0, 0.0, 0.0)),
                        ..PropsUpdate::default()
                    },
                ),
            );
        }
        self.rt.conn.try_apply(commands)?;
        self.regrab()
    }

    fn fly_out(&mut self) -> Result<(), ClientError> {
        let Some(flown) = self.flown.take() else {
            return Ok(());
        };
        self.unsnap(flown.attached.then_some(&flown.surface))?;
        self.draw_hud()?;
        self.glide_to(flown.back, FLY_MS, None);
        self.frame()
    }

    /// The sky, the ground and the fog for the pose, with the shadows of
    /// the nearest panes, `near` first.
    fn space_frame(&self, near: &[&Pane]) -> Vec<Command> {
        let palette = self.settings.value.theme.palette();
        let edge = (self.pose.x.hypot(self.pose.z) / self.radius()).min(1.0);
        let mut uniforms = BTreeMap::from([
            (
                "view".into(),
                Uniform::Vector(vec![self.lens.w, self.lens.h]),
            ),
            (
                "pose".into(),
                Uniform::Vector(vec![
                    self.pose.yaw,
                    self.pose.pitch,
                    self.pose.x,
                    self.pose.z,
                ]),
            ),
            (
                "lens".into(),
                Uniform::Vector(vec![self.lens.focal, self.lens.ground, FOG_SCALE, edge]),
            ),
            ("sky".into(), vec3(palette.sky)),
            ("fog".into(), vec3(palette.horizon)),
            ("ground".into(), vec3(palette.ground)),
            ("sheen".into(), vec3(palette.sheen)),
        ]);
        for k in 0..SHADOWS {
            let shadow = near.get(k).map_or(vec![0.0; 4], |p| {
                let lift = (self.lens.ground - p.size[1] / 2.0).max(0.0);
                vec![p.at[0], p.at[1], p.size[0] / 2.0, shadow_w(lift, p.yaw)]
            });
            uniforms.insert(format!("s{k}"), Uniform::Vector(shadow));
        }
        let color = Some([palette.sky[0], palette.sky[1], palette.sky[2], 1.0]);
        vec![
            update(
                SKY,
                PropsUpdate {
                    color,
                    ..PropsUpdate::default()
                },
            ),
            update(
                FOG,
                PropsUpdate {
                    uniforms: Some(uniforms),
                    ..PropsUpdate::default()
                },
            ),
        ]
    }

    /// Sends the pose: the matrix and the haze of each pane and its
    /// mirror image, back to front, and the camera of the fog.
    #[allow(clippy::too_many_lines, reason = "each pane, its mirror and the focus")]
    fn frame(&mut self) -> Result<(), ClientError> {
        if self.output.is_none() || !self.active {
            return Ok(());
        }
        let theme = self.settings.value.theme;
        let fog = theme.palette().horizon;
        let gloss = match theme {
            Theme::Light => 0.4,
            Theme::Dark => 0.55,
        };
        let radius = self.radius();
        let mut sorted: Vec<(&Pane, f64)> = (self.panes.iter())
            .map(|p| (p, space::depth(&self.pose, p.at)))
            .collect();
        sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
        // ponytail: the shadows of the 8 nearest panes in front; a
        // uniform array when the core passes arrays.
        let near: Vec<&Pane> = (sorted.iter().rev())
            .filter(|(_, depth)| *depth > 0.0)
            .map(|(p, _)| *p)
            .collect();
        let mut commands = self.space_frame(&near);
        let focus = self.focus().map(|k| self.panes[k].name.clone());
        let accent = self.settings.value.style.accent;
        let mut glow = None;
        for (pane, depth) in &sorted {
            let focused = focus.as_deref() == Some(pane.name.as_str());
            // A pane at or behind the eye does not show.
            let shown = *depth > self.lens.focal * 0.1;
            let mut props = PropsUpdate {
                visible: Some(shown),
                ..PropsUpdate::default()
            };
            let mut mirror = props.clone();
            if shown {
                let matrix =
                    space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
                let far = (depth - self.lens.focal).max(0.0) / radius;
                // The focused pane shows clear, with an accent edge of
                // about 1.5 pixels on the output.
                let haze = if focused {
                    0.0
                } else {
                    (1.0 - (-far * 1.5).exp()) * 0.9
                };
                let px = depth / self.lens.focal;
                let edge = if focused {
                    [accent[0], accent[1], accent[2], (1.5 * px).max(1.0)]
                } else {
                    [0.0; 4]
                };
                props.transform = Some(Transform::from_matrix(matrix));
                props.uniforms = Some(pane_uniforms(&pane.source, haze, fog, edge));
                if focused {
                    glow = Some(self.focus_glow(pane, px));
                }
                let k = pane.size[0] / mirror_size(pane.size)[0];
                let matrix =
                    space::mirror_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw, k);
                mirror.transform = Some(Transform::from_matrix(matrix));
                // Fresnel: the floor mirrors more at a grazing view.
                let grazing = 1.0 - self.lens.ground / self.lens.ground.hypot(*depth);
                let fresnel = 0.05 + 0.95 * grazing.powi(5);
                let strength = gloss * (0.4 + 0.6 * fresnel) * (1.0 - haze);
                mirror.uniforms = Some(mirror_uniforms(&pane.source, strength));
            }
            commands.push(update(&pane.node(), props));
            commands.push(update(&pane.mirror(), mirror));
        }
        let shows_glow = glow.is_some() && self.flown.is_none();
        if !self.glass.plain {
            let props = glow.filter(|_| shows_glow).unwrap_or(PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            });
            commands.push(update(FOCUS, props));
        }
        // The glow of the focused pane lies right under it.
        let order: Vec<String> = (sorted.iter())
            .flat_map(|(p, _)| {
                let under = (shows_glow && focus.as_deref() == Some(p.name.as_str()))
                    .then(|| FOCUS.to_owned());
                under.into_iter().chain([p.node()])
            })
            .collect();
        if order != self.order {
            commands.extend(order.iter().map(|id| Command::Move {
                id: id.clone(),
                parent: None,
                index: None,
            }));
            self.order = order;
        }
        commands.extend(self.label_commands());
        self.rt.conn.apply(commands)
    }

    /// The pane in focus: the pane flown into, else the pane under the
    /// pointer or ahead, as the label picks it.
    fn focus(&self) -> Option<usize> {
        if let Some(flown) = &self.flown {
            return self
                .panes
                .iter()
                .position(|p| p.window() == Some(&flown.surface));
        }
        (self.pointer.and_then(|p| self.pane_at(p)))
            .or_else(|| self.pane_ahead())
            .filter(|&i| {
                let far = self.radius();
                space::sight(&self.lens, &self.pose, self.panes[i].at, far) == "shown"
            })
    }

    /// The soft accent glow behind the focused pane, as the glass of a
    /// panel has it: a node of a quarter of the size, scaled up, whose
    /// glow reaches 16 pixels of the output. `px` is the size of a pixel
    /// of the output in pixels of the pane.
    fn focus_glow(&self, pane: &Pane, px: f64) -> PropsUpdate {
        let reach = self.settings.value.style.glow * px;
        let [w, h] = pane.size;
        let size = [w / FOCUS_SCALE, h / FOCUS_SCALE];
        let r = reach / FOCUS_SCALE;
        let uniforms = self.styled().glow_uniforms(size, 0.0, r, [0.0; 4], 0.0);
        let matrix = space::glow_matrix(
            &self.lens,
            &self.pose,
            pane.at,
            pane.size,
            pane.yaw,
            reach,
            FOCUS_SCALE,
        );
        PropsUpdate {
            size: Some([size[0] + 2.0 * r, size[1] + 2.0 * r]),
            transform: Some(Transform::from_matrix(matrix)),
            uniforms: Some(uniforms),
            visible: Some(true),
            ..PropsUpdate::default()
        }
    }

    /// Loads a construct (F18). An overlay takes the output now; a
    /// flavor keeps the windows that it has as panes, and an overlay the
    /// windows that it owns, which were given to it: no window gets lost.
    /// The other panes of the construct before go.
    fn load(&mut self, name: &str, file: file::File) -> Result<(), ClientError> {
        if self.flown.is_some() {
            self.fly_out()?;
        }
        self.close_list()?;
        if self.overlay && !self.active {
            self.focus_before = self.focused_surface()?;
        }
        self.glide = None;
        let overlay = self.overlay;
        let (kept, gone): (Vec<Pane>, Vec<Pane>) = std::mem::take(&mut self.panes)
            .into_iter()
            .partition(|p| p.window().is_some() && (p.attached || !overlay));
        self.panes = kept;
        let commands: Vec<Command> = gone.iter().flat_map(Pane::remove).collect();
        self.rt.conn.apply(commands)?;
        self.active = true;
        self.order.clear();
        self.waiting.clear();
        self.build()?;
        self.radius = file.radius;
        self.pose = file.camera;
        self.current = Some(name.into());
        for pane in file.panes {
            match pane.kind {
                Kind::Program => {
                    let token = chora_kit::launch(pane.command.as_deref().unwrap_or_default())
                        .map(|(t, _)| t);
                    self.waiting.push(Waiting::new(token, pane));
                }
                Kind::Plugin => {
                    if let Some(pane) = self.show_plugin(pane)? {
                        self.waiting.push(Waiting::new(None, pane));
                    }
                }
            }
        }
        self.frame()
    }

    /// Gives the output back (F20, F31): the space goes, and the keyboard
    /// goes back to the window that had it, through its owner (D31). The
    /// windows stay with their owner, which the construct never changed.
    fn leave(&mut self) -> Result<(), ClientError> {
        if !self.active {
            return Ok(());
        }
        self.rt.conn.apply(vec![
            Command::FocusNode { node: None },
            Command::Remove {
                id: NAME.into(),
                to: None,
            },
        ])?;
        if let Some(surface) = self.focus_before.take() {
            // The window may have closed in between.
            self.rt.conn.try_apply(vec![Command::Focus {
                surface: Some(surface),
            }])?;
        }
        self.list = None;
        self.labelled = None;
        self.rt.ungrab_all()?;
        self.active = false;
        self.current = None;
        self.radius = None;
        self.panes.clear();
        self.waiting.clear();
        self.order.clear();
        self.flown = None;
        self.glide = None;
        self.moving = None;
        self.drag = None;
        Ok(())
    }

    /// Shows a plugin pane through the node that its plugin tags
    /// `meta:pane=SCENE`; answers the pane back when the node is missing.
    fn show_plugin(&mut self, file: PaneFile) -> Result<Option<PaneFile>, ClientError> {
        let (Some(plugin), Some(scene)) = (file.plugin.clone(), file.scene.clone()) else {
            return Ok(None);
        };
        let nodes = (self.rt.conn).query(&format!("owner:{plugin} meta:pane={scene}"))?;
        let Some(node) = nodes.first() else {
            return Ok(Some(file));
        };
        let size = (file.size)
            .or(node.props.size)
            .unwrap_or([self.lens.w, self.lens.h]);
        let at = self.place(file.at, size, true).unwrap_or_default();
        let pane = Pane {
            name: self.label(&scene),
            source: node.id.clone(),
            content: Content::Plugin { plugin, scene },
            at,
            yaw: self.yaw_at(Some(&file), at),
            size,
            taken: false,
            attached: false,
        };
        self.rt.conn.apply(self.pane_nodes(&pane).into())?;
        self.panes.push(pane);
        Ok(None)
    }

    /// Looks again for the nodes of the plugin panes that wait.
    fn retry_plugins(&mut self) -> Result<(), ClientError> {
        let (plugins, programs): (Vec<Waiting>, Vec<Waiting>) = std::mem::take(&mut self.waiting)
            .into_iter()
            .partition(|w| w.pane.kind == Kind::Plugin);
        self.waiting = programs;
        let mut shown = false;
        for waiting in plugins {
            let Waiting { pane, .. } = waiting;
            match self.show_plugin(pane)? {
                Some(pane) => self.waiting.push(Waiting { pane, ..waiting }),
                None => shown = true,
            }
        }
        if shown { self.frame() } else { Ok(()) }
    }

    /// The construct as a file: the camera, the radius, the panes and the
    /// plugin panes that wait.
    fn snapshot(&self) -> file::File {
        let camera = self
            .flown
            .as_ref()
            .map_or_else(|| self.target(), |f| f.back);
        let panes = (self.panes.iter().filter_map(Pane::file))
            .chain(self.waiting.iter().filter_map(Waiting::file).cloned())
            .collect();
        file::File {
            camera,
            radius: self.radius,
            panes,
        }
    }

    /// The answer of `construct.list`: the files, the loaded construct
    /// and its panes with their labels.
    fn list(&self) -> Value {
        let panes: Vec<Value> = (self.panes.iter())
            .map(|p| {
                let mut v = json!({"name": p.name, "at": p.at, "yaw": p.yaw, "size": p.size});
                if let Some(surface) = p.window() {
                    v["surface"] = json!(surface);
                    v["taken"] = json!(p.taken);
                }
                match &p.content {
                    Content::Window { command } => {
                        v["kind"] = json!("program");
                        v["command"] = json!(command);
                    }
                    Content::Plugin { plugin, scene } => {
                        v["kind"] = json!("plugin");
                        v["plugin"] = json!(plugin);
                        v["scene"] = json!(scene);
                    }
                }
                v
            })
            .collect();
        let waiting: Vec<&PaneFile> = self.waiting.iter().map(|w| &w.pane).collect();
        json!({
            "constructs": file::dir().map(|d| file::list(&d)).unwrap_or_default(),
            "current": self.current,
            "overlay": self.overlay,
            "shown": self.active,
            "panes": panes,
            "waiting": waiting,
        })
    }
}

impl Construct {
    /// The surface with the keyboard, as the core has it.
    fn focused_surface(&mut self) -> Result<Option<String>, ClientError> {
        let query = Query {
            selector: "kind:surface".into(),
            filter: Some(Filter {
                focused: true,
                ..Filter::default()
            }),
        };
        let nodes: Vec<Value> = self.rt.conn.call("scene.query", query)?;
        Ok((nodes.first()).and_then(|n| Some(n["id"].as_str()?.to_owned())))
    }

    /// All windows (F29): the panes first, then the windows of any
    /// owner that have no pane, but not the launcher's own window.
    fn windows(&mut self) -> Result<Vec<Entry>, ClientError> {
        let nodes = self.rt.conn.query("kind:surface")?;
        let far = self.radius();
        let mut out: Vec<Entry> = (self.panes.iter())
            .filter_map(|p| {
                let surface = p.window()?;
                let node = nodes.iter().find(|n| n.id == surface);
                Some(Entry {
                    name: p.name.clone(),
                    surface: surface.into(),
                    owner: node.and_then(|n| n.owner.clone()),
                    pane: space::sight(&self.lens, &self.pose, p.at, far),
                    size: None,
                })
            })
            .collect();
        for node in nodes {
            let paned = out.iter().any(|e| e.surface == node.id);
            if paned || self.launcher_window.as_deref() == Some(&node.id) {
                continue;
            }
            let info = node.surface.unwrap_or_default();
            let base = (info.app_id.or(info.title)).unwrap_or_else(|| node.id.clone());
            out.push(Entry {
                name: unique(&base, |n| out.iter().any(|e| e.name == n)),
                surface: node.id,
                owner: node.owner,
                pane: "none",
                size: info.size.map(|s| s.map(f64::from)),
            });
        }
        Ok(out)
    }

    /// The windows without a visible pane: the rows of the list.
    fn hidden(&mut self) -> Result<Vec<Entry>, ClientError> {
        Ok((self.windows()?.into_iter())
            .filter(|e| e.pane != "shown")
            .collect())
    }

    /// Flies to the pane of a window, or takes the window of another
    /// client as a pane at its own size first (F30). Answers the pane.
    fn show(&mut self, entry: &Entry) -> Result<String, ClientError> {
        if entry.pane == "none" {
            // A choice makes the pane also when the view is full.
            let file = PaneFile {
                kind: Kind::Program,
                command: None,
                plugin: None,
                scene: None,
                at: None,
                yaw: None,
                size: entry.size,
            };
            // A free window, such as one that found no room, becomes the
            // flavor's own.
            let hold = match entry.owner {
                None if !self.overlay => Hold::Own,
                _ => Hold::Taken,
            };
            self.adopt(&entry.surface, &entry.name, Some(&file), hold, true)?;
        }
        let Some(i) = (self.panes.iter()).position(|p| p.window() == Some(&entry.surface)) else {
            return Err(ClientError::Rpc(RpcError::new(
                RpcError::REJECTED,
                format!("the window {} did not become a pane", entry.name),
            )));
        };
        if self.flown.is_some() {
            self.fly_out()?;
        }
        self.fly_in(i);
        Ok(self.panes[i].name.clone())
    }

    fn open_list(&mut self) -> Result<(), ClientError> {
        if self.list.is_some() {
            return self.close_list();
        }
        self.list = Some(ListState { sel: 0, give: None });
        self.draw_list(true)
    }

    fn select(&mut self, sel: usize) -> Result<(), ClientError> {
        if let Some(list) = &mut self.list {
            list.sel = sel;
        }
        self.draw_list(false)
    }

    /// The rows of the window managers to give the window of row `sel`
    /// to; only a window that the construct owns can go.
    fn give_menu(&mut self, sel: usize) -> Result<(), ClientError> {
        let Some(row) = self.rows()?.into_iter().nth(sel) else {
            return Ok(());
        };
        let own = (self.panes.iter()).any(|p| p.window() == Some(&row.target) && p.attached);
        if let Some(list) = self.list.as_mut().filter(|l| own && l.give.is_none()) {
            *list = ListState {
                sel: 0,
                give: Some(row.target),
            };
            return self.draw_list(false);
        }
        Ok(())
    }

    /// The rows of the open list: the windows without a visible pane, or
    /// the window managers to give a window to.
    fn rows(&mut self) -> Result<Vec<Row>, ClientError> {
        if self.list.as_ref().is_some_and(|l| l.give.is_some()) {
            return Ok((self.managers()?.into_iter())
                .map(|m| Row {
                    label: format!("give to {m}"),
                    note: "window manager".into(),
                    target: m,
                })
                .collect());
        }
        Ok((self.hidden()?.into_iter())
            .map(|e| {
                let owner = e.owner.as_deref().unwrap_or("no owner");
                let place = if e.pane == "none" { "no pane" } else { e.pane };
                Row {
                    label: e.name,
                    note: format!("{owner} \u{b7} {place}"),
                    target: e.surface,
                }
            })
            .collect())
    }

    fn close_list(&mut self) -> Result<(), ClientError> {
        if self.list.take().is_some() {
            let mut commands = hud::list_close();
            commands.extend(self.label_commands());
            self.rt.conn.try_apply(commands)?;
        }
        Ok(())
    }

    /// Closes the list and shows the window of row `k`, or gives the
    /// window to the window manager of row `k`.
    fn choose(&mut self, k: usize) -> Result<(), ClientError> {
        let rows = self.rows()?;
        let give = self.list.as_ref().and_then(|l| l.give.clone());
        self.close_list()?;
        let Some(row) = rows.into_iter().nth(k) else {
            return Ok(());
        };
        // A window that closed in between, or refuses, stays as it is.
        let done = match give {
            Some(surface) => self.give(&surface, &row.target).map(drop),
            None => match self.hidden()?.into_iter().find(|e| e.surface == row.target) {
                Some(entry) => self.show(&entry).map(drop),
                None => Ok(()),
            },
        };
        if let Err(err) = done {
            eprintln!("chora-construct: window list: {err}");
        }
        Ok(())
    }

    /// The other window managers: the clients with a claim (D55).
    fn managers(&mut self) -> Result<Vec<String>, ClientError> {
        let mut managers: Vec<String> = (self.rt.conn.query("")?.into_iter())
            .filter(|n| n.props.claim.is_some())
            .filter_map(|n| n.owner)
            .filter(|o| o != NAME)
            .collect();
        managers.sort();
        managers.dedup();
        Ok(managers)
    }

    /// Gives an own window to the client `to`, a window manager or a
    /// plugin such as mc (D55): the core reserves the window for `to`,
    /// with its props back to the defaults, and the pane goes. The core
    /// refuses a `to` without a connection, and the pane stays.
    fn give(&mut self, surface: &str, to: &str) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        let Some(pane) = self.panes.iter().find(|p| p.window() == Some(surface)) else {
            return refuse(format!("{surface} is no pane"));
        };
        if !pane.attached {
            return refuse(format!(
                "the window {} belongs to another client, which gives it",
                pane.name
            ));
        }
        let answer = self.rt.conn.give(surface, to).map_err(ClientError::Rpc)?;
        self.forget(surface)?;
        Ok(answer)
    }

    /// How a pane holds a window that the construct started.
    fn started(&self) -> Hold {
        if self.overlay { Hold::Shown } else { Hold::Own }
    }

    /// A window that is free again, or that another window manager gives
    /// to the construct (D55). The flavor takes both; an overlay only a
    /// window given to it, so that it never takes a window on its own
    /// (F30). It owns such a window, and its leave frees it again.
    fn detached(&mut self, surface: &str) -> Result<(), ClientError> {
        let node = self.rt.conn.query(&format!("id:{surface}"))?.pop();
        let Some(node) = node else {
            return Ok(());
        };
        if self.overlay && node.to.as_deref() != Some(NAME) {
            return Ok(());
        }
        let info = node.surface.unwrap_or_default();
        let label = info.app_id.or(info.title).unwrap_or_default();
        self.forget(surface)?;
        self.adopt(surface, &label, None, Hold::Own, false)
    }

    /// The window list as a glass panel in the middle of the output
    /// (docs/construct-style.md): the label `WINDOWS` or `GIVE TO`, then
    /// one row per window with its name and, as a note in data, its owner
    /// and the place of its pane. An accent mark shows the selected row.
    /// `open` fades the panel in; a new selection only makes the rows anew.
    fn draw_list(&mut self, open: bool) -> Result<(), ClientError> {
        let Some(sel) = self.list.as_ref().map(|l| l.sel) else {
            return Ok(());
        };
        let rows = self.rows()?;
        let sel = sel.min(rows.len().saturating_sub(1));
        let giving = self.list.as_mut().is_some_and(|l| {
            l.sel = sel;
            l.give.is_some()
        });
        let unit = self.settings.value.style.unit;
        // ponytail: a window of 12 rows that follows the selection; a
        // scroll bar when lists grow long.
        let first = (sel + 1).saturating_sub(LIST_ROWS);
        let shown = rows.len().saturating_sub(first).clamp(1, LIST_ROWS);
        let size = hud::list_size(unit, shown);
        let place = [
            ((self.lens.w - size[0]) / 2.0).round(),
            ((self.lens.h - size[1]) / 2.0).round(),
        ];
        let label = if giving { "GIVE TO" } else { "WINDOWS" };
        let mut commands = vec![
            Command::Remove {
                id: hud::LIST_ROWS_GROUP.into(),
                to: None,
            },
            create(
                hud::LIST_ROWS_GROUP,
                NodeKind::Group,
                LIST,
                PropsUpdate::default(),
            ),
        ];
        let top = hud::list_top(unit);
        commands.extend(self.fitted(|s| {
            let look = s.styled();
            let mut commands = look.list(size, place, label, open);
            if rows.is_empty() {
                let none = if giving {
                    "no other window manager runs"
                } else {
                    "every window shows"
                };
                commands.extend(look.row(&format!("{LIST_ROW}none"), top, none, "", false));
            }
            for (k, row) in rows.iter().enumerate().skip(first).take(LIST_ROWS) {
                #[allow(clippy::cast_precision_loss, reason = "a dozen rows")]
                let y = top + LIST_LINE * (k - first) as f64;
                let id = format!("{LIST_ROW}{k}");
                commands.extend(look.row(&id, y, &row.label, &row.note, k == sel));
            }
            commands
        })?);
        commands.extend(self.label_commands());
        // The first draw has no rows to remove.
        if self
            .rt
            .conn
            .query(&format!("id:{}", hud::LIST_ROWS_GROUP))?
            .is_empty()
        {
            commands.remove(0);
        }
        self.rt.conn.apply(commands)
    }

    /// What the elements over the space draw with.
    fn styled(&self) -> Look<'_> {
        let style = &self.settings.value.style;
        Look {
            style,
            colors: style.colors(self.settings.value.theme),
            plain: self.glass.plain,
            shader: &self.glass_shader,
            metrics: &self.metrics,
        }
    }

    /// Builds the elements over the space with the sizes of their texts
    /// as the core lays them out: a build that asks for texts not yet
    /// measured runs again after the core measured them.
    fn fitted<T>(&mut self, mut build: impl FnMut(&mut Self) -> T) -> Result<T, ClientError> {
        let mut out = build(self);
        for _ in 0..3 {
            let misses = self.metrics.take_misses();
            if misses.is_empty() {
                break;
            }
            self.measure(misses)?;
            out = build(self);
        }
        Ok(out)
    }

    /// Measures texts with probe text nodes in a hidden group: their
    /// rects are the sizes that the core lays them out at.
    fn measure(&mut self, probes: Vec<hud::Probe>) -> Result<(), ClientError> {
        const PROBE: &str = "construct-probe";
        let meta = BTreeMap::from([("probe".to_owned(), "1".to_owned())]);
        let mut commands = vec![create(
            PROBE,
            NodeKind::Group,
            NAME,
            PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        )];
        let look = self.styled();
        commands.extend(probes.iter().enumerate().map(|(k, probe)| {
            create(
                &format!("{PROBE}-{k}"),
                NodeKind::Text,
                PROBE,
                PropsUpdate {
                    meta: Some(meta.clone()),
                    ..look.probe_props(probe)
                },
            )
        }));
        self.rt.conn.apply(commands)?;
        let nodes: Result<Vec<Value>, ClientError> = (self.rt.conn).call(
            "scene.query",
            Query::new(format!("owner:{NAME} meta:probe=1")),
        );
        self.rt.conn.apply(vec![Command::Remove {
            id: PROBE.into(),
            to: None,
        }])?;
        let sizes: BTreeMap<String, [f64; 2]> = (nodes?.iter())
            .filter_map(|n| {
                let r = n["rects"].as_object()?.values().next()?.as_array()?;
                let size = [r.get(2)?.as_f64()?, r.get(3)?.as_f64()?];
                Some((n["id"].as_str()?.to_owned(), size))
            })
            .collect();
        for (k, probe) in probes.into_iter().enumerate() {
            // A probe that the core did not lay out keeps its estimate.
            if let Some(size) = sizes.get(&format!("{PROBE}-{k}")) {
                self.metrics.insert(probe, *size);
            }
        }
        Ok(())
    }

    /// The elements anew in the style of the settings, which changed.
    fn restyle(&mut self) -> Result<(), ClientError> {
        if !self.active || self.output.is_none() {
            return Ok(());
        }
        let mut commands: Vec<Command> = [hud::LABEL, hud::HUD, LIST]
            .map(|id| Command::Remove {
                id: id.into(),
                to: None,
            })
            .into();
        commands.extend(self.styled().groups(NAME));
        self.rt.conn.try_apply(commands)?;
        self.labelled = None;
        self.draw_hud()?;
        self.draw_ask()?;
        if self.list.is_some() {
            self.draw_list(true)?;
        }
        Ok(())
    }

    /// Asks the voice agent to hide its dot while the HUD shows its
    /// state, once per connection of the agent (docs/voice.md, 10).
    fn claim_indicator(&mut self, last: bool) {
        if !self.settings.value.voice_hud || !self.active {
            return;
        }
        let conn = &mut self.rt.conn;
        self.claim.claim(last, || {
            let call = json!({"client": VOICE, "method": "indicator.claim", "params": {}});
            conn.call::<_, Value>("methods.call", call).is_ok()
        });
    }

    /// An event `voice.KIND` of the voice agent. Wrong data only log.
    fn follow(&mut self, event: &str, data: Value) -> Result<(), ClientError> {
        self.claim_indicator(true);
        let kind = event.strip_prefix("voice.").unwrap_or_default();
        match self.voice.follow(kind, data, Instant::now()) {
            Ok(changed) => {
                if changed.hud {
                    self.draw_hud()?;
                }
                if changed.ask {
                    self.draw_ask()?;
                }
                Ok(())
            }
            Err(err) => {
                eprintln!("chora-construct: {event}: {err}");
                Ok(())
            }
        }
    }

    /// The deadlines of the sentence: the waveform stops at its end, the
    /// sentence fades out 2 s later, and then the panel fits again.
    fn tick(&mut self) -> Result<(), ClientError> {
        let now = Instant::now();
        let due = |t: Option<Instant>| t.is_some_and(|t| t <= now);
        if due(self.voice.say_until) {
            self.voice.say_until = None;
            self.draw_hud()?;
        }
        if due(self.voice.fade_at) {
            self.voice.fade_at = None;
            self.voice.drop_at = Some(now + Duration::from_millis(120));
            if self.shows_hud() {
                let fade = hud::fade(hud::HUD_SAY, 0.0, hud::FADE_OUT);
                self.rt.conn.try_apply(vec![fade])?;
            }
        }
        if due(self.voice.drop_at) {
            self.voice.drop_at = None;
            self.voice.say = None;
            self.draw_hud()?;
        }
        Ok(())
    }

    fn shows_hud(&self) -> bool {
        self.settings.value.voice_hud && self.active && self.output.is_some()
    }

    /// The voice HUD: at the bottom centre, or compact at the bottom
    /// right while flown into a pane.
    fn draw_hud(&mut self) -> Result<(), ClientError> {
        if !self.shows_hud() {
            return Ok(());
        }
        let out = [self.lens.w, self.lens.h];
        let level = self.voice.level(Instant::now());
        let (compact, glow) = (self.flown.is_some(), !self.glass.hidden);
        let commands = self.fitted(|s| s.voice.hud(&s.styled(), out, compact, level, glow))?;
        if commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.apply(commands)
    }

    /// The question of the voice agent, on top, made anew.
    fn draw_ask(&mut self) -> Result<(), ClientError> {
        if !self.shows_hud() {
            return Ok(());
        }
        let out = [self.lens.w, self.lens.h];
        let glow = !self.glass.hidden;
        let mut commands = self.fitted(|s| {
            let look = Look {
                style: &s.settings.value.style,
                colors: s.settings.value.style.colors(s.settings.value.theme),
                plain: s.glass.plain,
                shader: &s.glass_shader,
                metrics: &s.metrics,
            };
            s.voice.ask(&look, out, glow)
        })?;
        if self.rt.conn.query(&format!("id:{}", hud::ASK))?.is_empty() {
            commands.remove(0);
        }
        self.rt.conn.apply(commands)
    }

    /// The pane label: the name of the pane under the pointer, else of the
    /// pane ahead, 8 below its bottom edge; hidden while flown in, while
    /// the list is open, and when the pane does not show. The text and the
    /// pill change only with the pane.
    fn label_commands(&mut self) -> Vec<Command> {
        let hidden = self.flown.is_some()
            || self.list.is_some()
            || self.glide.as_ref().is_some_and(|g| g.into.is_some());
        let pick = if hidden { None } else { self.focus() };
        let Some(k) = pick else {
            return match self.labelled.take() {
                Some(_) => Look::label_hide(),
                None => Vec::new(),
            };
        };
        let fresh = self.labelled.is_none();
        let name = self.panes[k].name.clone();
        let fitted = self.fitted(|s| s.styled().label(&name, fresh));
        let (label, pill) = fitted.unwrap_or_else(|err| {
            eprintln!("chora-construct: measure the label: {err}");
            self.styled().label(&name, fresh)
        });
        let pane = &self.panes[k];
        let matrix = space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
        let [x, y] = space::project(&matrix, pane.size[0] / 2.0, pane.size[1]);
        let unit = self.settings.value.style.unit;
        let place = Transform::translate((x - pill / 2.0).round(), (y + unit).round(), 0.0);
        let mut commands = vec![update(
            hud::LABEL,
            PropsUpdate {
                transform: Some(place),
                ..PropsUpdate::default()
            },
        )];
        if self.labelled.as_deref() != Some(pane.name.as_str()) {
            self.labelled = Some(pane.name.clone());
            commands.extend(label);
        }
        commands
    }
}

/// Parameters with a name.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Named {
    name: String,
}

/// Parameters of `pane.move`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MovePane {
    name: String,
    #[serde(default)]
    right: f64,
    #[serde(default)]
    ahead: f64,
}

/// Parameters of `pane.resize`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResizePane {
    name: String,
    scale: f64,
}

/// The methods; those that start programs or write files are privileged
/// (D62).
#[allow(clippy::too_many_lines, reason = "one entry per method")]
fn exported_methods() -> Vec<ExportMethod> {
    let name = || json!({"name": {"type": "string"}});
    vec![
        runtime::privileged(runtime::method(
            "construct.load",
            "Loads the construct NAME from ~/.config/chora/constructs/NAME.yaml and starts \
             its programs; beside another flavor, the construct takes the output.",
            name(),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "construct.save",
            "Saves the space as the construct NAME: the camera, the radius and the panes.",
            name(),
            &["name"],
        )),
        runtime::method(
            "construct.list",
            "Answers the constructs, the loaded one, and its panes with their names.",
            json!({}),
            &[],
        ),
        runtime::method(
            "construct.leave",
            "Leaves a construct that was loaded beside another flavor: the output and \
             the keyboard go back.",
            json!({}),
            &[],
        ),
        runtime::method(
            "pane.move",
            "Moves the pane NAME by an offset from where it is: right and ahead, in \
             logical pixels, as the camera looks; negative values move it left or \
             nearer. Moves add up: right 1800 and then right -1800 bring it back. \
             The leash radius stops it. Answers its new place.",
            json!({
                "name": {"type": "string"},
                "right": {"type": "number"},
                "ahead": {"type": "number"},
            }),
            &["name"],
        ),
        runtime::method(
            "pane.resize",
            "Resizes a pane by its name: scale 1.25 makes it a quarter larger.",
            json!({"name": {"type": "string"}, "scale": {"type": "number"}}),
            &["name", "scale"],
        ),
        runtime::method(
            "pane.remove",
            "Removes a pane by its name; its window gets a close request.",
            name(),
            &["name"],
        ),
        runtime::method(
            "window.list",
            "Answers all windows: the label, the surface, the owner and the pane, which \
             is shown, behind, far, aside or none. Panes that do not show and windows \
             without a pane are the window list of the key Tab.",
            json!({}),
            &[],
        ),
        runtime::method(
            "window.show",
            "Flies to the window NAME (a label or a surface of window.list); a window of \
             another client becomes a pane first, and goes back to it when the pane or \
             the construct goes.",
            name(),
            &["name"],
        ),
        give::method(),
        runtime::method(
            "pane.fly",
            "Flies into a pane by its name, until it covers the output at 1:1.",
            name(),
            &["name"],
        ),
    ]
}

/// Whether a window with the token of the launcher is the launcher's own:
/// a launcher without a window, such as fuzzel, may pass its token on to
/// the program that it starts, another process. Without both pids, the
/// window is the launcher's.
// ponytail: pids only; a launcher started through a wrapper shell with
// more than one command then has a window of another process, a pane.
fn launchers_own(window: Option<u32>, launcher: Option<u32>) -> bool {
    window.zip(launcher).is_none_or(|(w, l)| w == l)
}

fn configure(surface: &str, size: [f64; 2]) -> Command {
    #[allow(clippy::cast_possible_truncation, reason = "clamped to PANE_MAX")]
    let size = size.map(|s| s.round().clamp(1.0, PANE_MAX) as i32);
    Command::Configure {
        surface: surface.into(),
        size: Some(size),
        states: vec![ToplevelState::Maximized],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pane keeps its pose in the file, and a construct file gives it
    /// back unchanged; a window that the construct did not start has no
    /// command and stays out.
    #[test]
    fn a_pane_saves_its_pose() {
        let pane = |content| Pane {
            name: "foot".into(),
            source: "s1".into(),
            content,
            at: [-750.125, 2300.5],
            yaw: 0.25,
            size: [1200.0, 768.0],
            taken: false,
            attached: false,
        };
        let started = pane(Content::Window {
            command: Some("foot".into()),
        });
        let saved = started.file().unwrap();
        assert_eq!(saved.kind, Kind::Program);
        assert_eq!(
            (saved.at, saved.yaw, saved.size),
            (Some(started.at), Some(started.yaw), Some(started.size))
        );
        let text = serde_saphyr::to_string(&file::File {
            panes: vec![saved.clone()],
            ..file::File::default()
        })
        .unwrap();
        assert_eq!(file::parse(&text).unwrap().panes, [saved]);
        assert!(pane(Content::Window { command: None }).file().is_none());
        let plugin = pane(Content::Plugin {
            plugin: "mc".into(),
            scene: "mission".into(),
        });
        assert_eq!(plugin.node(), "construct-plugin-foot");
        assert_eq!(plugin.file().unwrap().scene.as_deref(), Some("mission"));
    }

    /// A pane keeps its yaw while the camera walks and looks around
    /// (live bug: the panes turned with the camera), and a move turns
    /// its face to the camera.
    #[test]
    fn a_walk_keeps_the_yaw_and_a_move_refaces() {
        let mut pane = Pane {
            name: "foot".into(),
            source: "s1".into(),
            content: Content::Window { command: None },
            at: [0.0, 0.0],
            yaw: 0.0,
            size: [640.0, 400.0],
            taken: false,
            attached: false,
        };
        pane.put([0.0, 2000.0], [0.0, 0.0]);
        assert!(pane.yaw.abs() < 1e-12);
        let mut pose = Pose::default();
        for _ in 0..20 {
            [pose.x, pose.z] = space::leash([pose.x, pose.z], [150.0, 40.0], 6000.0);
            turn(&mut pose, None, 30.0, 4.0);
        }
        assert!(pane.yaw.abs() < 1e-12, "the walk turned the pane");
        pane.put([1000.0, 2000.0], [pose.x, pose.z]);
        let face = space::facing([pose.x, pose.z], [1000.0, 2000.0]);
        assert!((pane.yaw - face).abs() < 1e-12);
        assert!(face.abs() > 0.1, "the move turned it: {face}");
    }

    /// The shadow uniform gives the lift and the yaw back as fog.frag
    /// reads them.
    #[test]
    fn the_shadow_packs_the_lift_and_the_yaw() {
        for (lift, yaw) in [(0.0, 0.0), (412.7, 3.0), (999.0, -3.1), (5.0, 7.0)] {
            let w = shadow_w(lift, yaw);
            assert!((w.floor() - lift.floor()).abs() < f64::EPSILON);
            let back = (w.fract() / 0.99 - 0.5) * std::f64::consts::TAU;
            let diff = (back - yaw).rem_euclid(std::f64::consts::TAU);
            assert!(
                diff.min(std::f64::consts::TAU - diff) < 1e-9,
                "{lift} {yaw} {back}"
            );
        }
    }

    /// A walk during a drag stays: the drag turns only (live bug: the
    /// camera jumped back to the press when the key `w` was released).
    #[test]
    fn a_walk_during_a_drag_keeps_its_place() {
        let mut pose = Pose::default();
        turn(&mut pose, None, 50.0, 0.0);
        // The key w: a glide forward.
        let mut to = pose;
        to.x += 90.0 * pose.yaw.sin();
        to.z += 90.0 * pose.yaw.cos();
        let mut glide = Glide {
            from: pose,
            to,
            start: Instant::now(),
            ms: STEP_MS,
            into: None,
        };
        turn(&mut pose, Some(&mut glide), 10.0, -5.0);
        pose = glide.from.lerp(&glide.to, 1.0);
        // w released, the drag moves on.
        turn(&mut pose, None, 10.0, 0.0);
        assert!(
            (pose.x - to.x).abs() < 1e-9 && (pose.z - to.z).abs() < 1e-9,
            "{pose:?}"
        );
        assert!((pose.yaw - 70.0 * LOOK_RATE).abs() < 1e-9);
        assert!((pose.pitch - 5.0 * LOOK_RATE).abs() < 1e-9);
    }

    /// The offsets of `pane.move` add up, as the camera looks.
    #[test]
    fn pane_moves_add_up() {
        let near =
            |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9;
        let start = [3150.0, 2000.0];
        let ahead = Pose::default().forward();
        let there = moved(start, ahead, 1800.0, 0.0);
        assert!(
            near(there, [4950.0, 2000.0]),
            "right is +x at yaw 0: {there:?}"
        );
        assert!(near(moved(there, ahead, -1800.0, 0.0), start));
        let twice = moved(moved(start, ahead, 900.0, 100.0), ahead, 900.0, 100.0);
        assert!(near(twice, moved(start, ahead, 1800.0, 200.0)));
        // Turned right a quarter, the camera looks along +x; its right is -z.
        let turned = Pose {
            yaw: std::f64::consts::FRAC_PI_2,
            ..Pose::default()
        };
        assert!(near(
            moved([0.0, 0.0], turned.forward(), 100.0, 0.0),
            [0.0, -100.0]
        ));
    }

    /// The launcher's token on the window of another process is the
    /// program of the launcher, as fuzzel passes it on.
    #[test]
    fn a_window_of_another_process_is_not_the_launcher() {
        assert!(launchers_own(Some(7), Some(7)));
        assert!(!launchers_own(Some(8), Some(7)));
        assert!(launchers_own(None, Some(7)));
        assert!(launchers_own(Some(8), None));
    }

    /// A program that waits longer than [`LAUNCHED`] goes, and no program
    /// that waits is saved; a plugin pane waits and saves.
    #[test]
    fn a_waiting_program_expires_and_is_not_saved() {
        let pane = |kind| PaneFile {
            kind,
            command: Some("xterm".into()),
            plugin: None,
            scene: None,
            at: None,
            yaw: None,
            size: None,
        };
        let old = |kind| Waiting {
            since: Instant::now().checked_sub(LAUNCHED).unwrap(),
            ..Waiting::new(None, pane(kind))
        };
        assert!(!Waiting::new(None, pane(Kind::Program)).stale());
        assert!(old(Kind::Program).stale());
        assert!(!old(Kind::Plugin).stale());
        assert!(Waiting::new(None, pane(Kind::Program)).file().is_none());
        assert!(old(Kind::Plugin).file().is_some());
    }
}
