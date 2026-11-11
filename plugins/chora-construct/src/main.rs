//! chora-construct, steps K1, K2a and K5 (D66, D75): a light, foggy
//! space with a horizon, a walk on a leash, and panes that stand in it:
//! windows, and node trees of other plugins such as mc scenes. The space
//! is a 3D context of the core, which sorts the panes and casts the ray
//! of the pointer, so a window of the construct takes clicks and keys in
//! its tilted pane. A key flies into a pane until it covers the output
//! at 1:1; then the window itself shows, sharp, with the keyboard. A
//! binding flies out again.
//!
//! A construct is a file of panes (F17). As the flavor, the plugin owns
//! its windows. With `--overlay` it waits beside another flavor until
//! `construct.load`, and then covers the output from its screen root
//! (`screen:NAME`), so that no camera changes hands (F20); its program
//! panes show windows of the other flavor, which gets the keyboard
//! requests for them (D31).
//!
//! Files: `plugin` (the `Plugin` impl), `input`, `panes`, `scene`,
//! `adopt`, `camera`, `frame`, `list` (window list and give menu),
//! `overlay` (HUD glue), `voice` (voice glue), `methods` (the
//! `construct.*` methods), `space` (camera math), `hud`, `config`
//! and `file`.
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

mod adopt;
mod camera;
mod config;
mod cube;
mod file;
mod frame;
mod hud;
mod input;
mod list;
mod methods;
mod modes;
mod overlay;
mod panes;
mod plugin;
mod pull;
mod scene;
mod space;
mod voice;
mod wobble;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_kit::{
    effects,
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
    camera::{moved, takes_the_camera, window_flown_into},
    config::{Action, Config},
    file::{Kind, PaneFile},
    hud::{Claim, LIST, LIST_LINE, LIST_ROWS, Look, Metrics, Voice},
    methods::{GlideTo, MovePane, Named, OpenCube, ResizePane, TurnCube, exported_methods},
    panes::{PaneLook, configure, create, handed_over, mirror_size, shadow_w, update, vec3},
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
/// The 3D context of the space: a group of the output's size with the
/// perspective of the lens (D75), and in it the camera group, whose
/// transform is the pose.
const SPACE: &str = "construct-space";
const POSE: &str = "construct-pose";
const PANES: &str = "construct-panes";
/// The windows of the construct without a pane, hidden under the sky.
const WINDOWS: &str = "construct-windows";
/// The window that the camera is in, over the space.
const FRONT: &str = "construct-front";
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
const SOURCE_SHADER: &str = include_str!("../source.frag");
const MIRROR_SHADER: &str = include_str!("../mirror.frag");
const GLASS_SHADER: &str = include_str!("../glass.frag");
/// The notice of a window that finds no room in the view.
const NO_ROOM: &str = "No room in view: in the window list (Tab)";
/// The client name of the voice agent (docs/voice.md, section 10).
const VOICE: &str = "voice";
/// The left button, as Linux names it.
const BTN_LEFT: u32 = 272;
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

    /// The shader node that shows the source of a pane through a node
    /// texture: every pane but one of a window of the construct, which
    /// holds its window itself.
    fn texture(&self) -> Option<String> {
        (!self.attached).then(|| format!("{}-source", self.node()))
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

    /// Where the pane is: centre, size and yaw, as `space::sight` wants.
    fn spot(&self) -> ([f64; 2], [f64; 2], f64) {
        (self.at, self.size, self.yaw)
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
    Give(&'static str),
    /// With a cube: the step back from its face, and a turn of it.
    StepBack,
    Turn(i32),
}

/// A drag with the modifier: the pane, the press, and its centre and
/// yaw then.
struct Moving {
    name: String,
    from: [f64; 2],
    at: [f64; 2],
    yaw: f64,
}

/// An eased camera move; at its end it may snap into a pane.
struct Glide {
    from: Pose,
    to: Pose,
    start: Instant,
    ms: f64,
    into: Option<Flown>,
    /// Whether the move also starts slowly, as a flight of
    /// `camera.glide` does.
    smooth: bool,
}

/// The pose with the snap: the window of the pane that the camera flew
/// into, and the pose to fly back to.
struct Flown {
    surface: String,
    back: Pose,
    /// Whether the window is attached, so that it shows itself.
    attached: bool,
    /// For a plugin pane: the plugin and its scene, which get the
    /// keyboard while the camera is in the pane. `surface` is then the
    /// node of the pane's source.
    plugin: Option<(String, String)>,
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
    /// Windows that found no room: attached and hidden, with no pane yet.
    hidden: Vec<String>,
    moving: Option<Moving>,
    fog_shader: String,
    pane_shader: String,
    source_shader: String,
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
    /// The look of the panes as last sent; `None` sends it again.
    sent: Option<PaneLook>,
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
    /// Whether the help panel shows.
    help: bool,
    claim: Claim,
    glass: Glass,
    /// The last point of the pointer over the space that the core told:
    /// it tells the place only from a press to its release.
    // ponytail: the label follows the pointer from the last press, drag
    // or release; a hover needs a pointer position event of the core.
    pointer: Option<[f64; 2]>,
    /// The pane that the label names, if it shows.
    labelled: Option<String>,
    /// The window of a pane under the pointer, from `pointer.entered`:
    /// the focus and the label follow it.
    hovered: Option<String>,
    /// The panes that show their back to the camera and are hidden.
    backs: std::collections::BTreeSet<String>,
    /// The focus glow as last sent.
    glow: Option<PropsUpdate>,
    /// How far each pane that wobbles is from its place, for its shadow.
    drifts: HashMap<String, [f64; 2]>,
    /// The pane whose mesh the focus glow has.
    glow_bend: wobble::GlowBend,
    /// The wobble of the panes of own windows that the modifier drags.
    effects: effects::Driver,
    /// The cube of virtual outputs and their window manager.
    cube: Option<cube::Host>,
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

/// Writes a shader to `$XDG_RUNTIME_DIR/{dir}/{name}` when it differs,
/// through a new file of mode 0600. Without an absolute runtime
/// directory it refuses: the shared temporary directory would let another
/// local user plant the shader first.
fn shader_file(
    runtime: Option<std::ffi::OsString>,
    dir: &str,
    name: &str,
    text: &str,
) -> std::io::Result<PathBuf> {
    let runtime = (runtime.map(PathBuf::from))
        .filter(|p| p.is_absolute())
        .ok_or_else(|| std::io::Error::other("no absolute $XDG_RUNTIME_DIR for the shaders"))?;
    let path = runtime.join(dir).join(name);
    if chora_kit::read_text(&path, chora_kit::MAX_TEXT)
        .ok()
        .as_deref()
        != Some(text)
    {
        chora_kit::write_file(&path, text.as_bytes())?;
    }
    Ok(path)
}

fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-construct")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let rt = Runtime::connect(NAME, Vec::new())?;
    let start = Pose::default();
    let shader = |name: &str, text: &str| {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR");
        shader_file(runtime, "chora-construct", name, text)
            .map(|p| p.to_string_lossy().into_owned())
    };
    let mut construct = Construct {
        rt,
        settings,
        overlay: args.overlay,
        active: false,
        current: None,
        radius: None,
        waiting: Vec::new(),
        hidden: Vec::new(),
        moving: None,
        fog_shader: shader("fog.frag", FOG_SHADER)?,
        pane_shader: shader("pane.frag", PANE_SHADER)?,
        source_shader: shader("source.frag", SOURCE_SHADER)?,
        mirror_shader: shader("mirror.frag", MIRROR_SHADER)?,
        glass_shader: shader("glass.frag", GLASS_SHADER)?,
        launcher: None,
        launcher_window: None,
        launched: None,
        output: None,
        lens: Lens::new(1280.0, 800.0),
        pose: start,
        glide: None,
        panes: Vec::new(),
        sent: None,
        flown: None,
        drag: None,
        list: None,
        focus_before: None,
        voice: Voice::default(),
        metrics: Metrics::default(),
        help: false,
        claim: Claim { due: true },
        glass: Glass::default(),
        pointer: None,
        labelled: None,
        hovered: None,
        backs: std::collections::BTreeSet::new(),
        glow: None,
        drifts: HashMap::new(),
        glow_bend: wobble::GlowBend::default(),
        effects: effects::Driver::default(),
        cube: None,
    };
    construct.effects.set(&construct.settings.value.effects);
    construct.start()?;
    construct.serve()?;
    Ok(())
}

/// The events of the plugin.
const EVENTS: [&str; 19] = [
    "voice.*",
    modes::SWITCHER,
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
    "pointer.entered",
    "pointer.left",
    "pointer.pressed",
    "pointer.moved",
    "pointer.released",
    "pointer.scroll",
];

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
    /// Whether the press came from `input.inject` without the test
    /// setting: its click flies nowhere.
    injected: bool,
}
#[cfg(test)]
mod tests;
