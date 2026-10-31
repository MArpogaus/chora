//! chora-construct, steps K1 and K2a (D66): a white space with a foggy
//! horizon, a walk on a leash, and panes that face the camera: windows,
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

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod config;
mod file;
mod space;

use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Event, InputMode, NodeKind, NodeTexture, OutputInfo, PropsUpdate, ToplevelState,
    Transform, Uniform,
    ipc::{ClientError, ExportMethod, PressState, RpcError},
    overlay_id,
};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::{Action, Config, Theme},
    file::{Kind, PaneFile},
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
/// A mirror image renders at this fraction of its pane: it is blurred.
const MIRROR_SCALE: f64 = 4.0;
/// The fog shader draws the shadows of this many panes, the nearest.
const SHADOWS: usize = 8;
const PANES: &str = "construct-panes";
const WINDOWS: &str = "construct-windows";
/// A clear rect over the space: it takes the drags, clicks, the wheel
/// and, through `focus_node`, the keys of the walk.
const LOOK: &str = "construct-look";
const FOG_SHADER: &str = include_str!("../fog.frag");
const PANE_SHADER: &str = include_str!("../pane.frag");
const MIRROR_SHADER: &str = include_str!("../mirror.frag");
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
    /// The size in logical pixels.
    size: [f64; 2],
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
}

struct Construct {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    /// Beside another flavor: no windows of its own, and a space only
    /// between `construct.load` and `construct.leave`.
    overlay: bool,
    /// Whether the space shows.
    active: bool,
    /// The loaded construct and its leash radius.
    current: Option<String>,
    radius: f64,
    waiting: Vec<Waiting>,
    moving: Option<Moving>,
    fog_shader: String,
    pane_shader: String,
    mirror_shader: String,
    output: Option<OutputInfo>,
    lens: Lens,
    pose: Pose,
    glide: Option<Glide>,
    panes: Vec<Pane>,
    /// The pane nodes from the back to the front, as last sent.
    order: Vec<String>,
    flown: Option<Flown>,
    /// The press of a drag: where, the pose then, and whether it moved.
    drag: Option<([f64; 2], Pose, bool)>,
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
    let radius = settings.value.radius;
    let mut construct = Construct {
        rt,
        settings,
        overlay: args.overlay,
        active: false,
        current: None,
        radius,
        waiting: Vec::new(),
        moving: None,
        fog_shader: shader_file("fog.frag", FOG_SHADER)?,
        pane_shader: shader_file("pane.frag", PANE_SHADER)?,
        mirror_shader: shader_file("mirror.frag", MIRROR_SHADER)?,
        output: None,
        lens: Lens::new(1280.0, 800.0),
        pose: start,
        glide: None,
        panes: Vec::new(),
        order: Vec::new(),
        flown: None,
        drag: None,
    };
    construct.start()?;
    construct.serve()?;
    Ok(())
}

/// The events of the plugin.
const EVENTS: [&str; 11] = [
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

/// The uniforms of a pane; an update replaces all of them, so each
/// carries the node that the pane shows.
fn pane_uniforms(source: &str, haze: f64, fog: [f64; 3]) -> BTreeMap<String, Uniform> {
    let win = Uniform::Texture(NodeTexture {
        node: source.into(),
    });
    BTreeMap::from([
        ("win".into(), win),
        ("haze".into(), Uniform::Float(haze)),
        ("fog".into(), vec3(fog)),
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

/// The eased share of a glide after `k` of its time.
fn ease(k: f64) -> f64 {
    1.0 - (1.0 - k).powi(3)
}

impl Plugin for Construct {
    type Action = ();

    fn runtime(&mut self) -> &mut Runtime<()> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::SurfaceCreated {
                surface,
                app_id,
                title,
                to,
                token,
                ..
            } => {
                let i = (self.waiting.iter()).position(|w| w.token.is_some() && w.token == token);
                let pane = i.map(|i| self.waiting.remove(i).pane);
                // A flavor takes the free windows; an overlay only shows
                // those of its construct, which the other flavor takes.
                let free = to.as_deref().is_none_or(|t| t == NAME);
                if pane.is_some() || (free && !self.overlay) {
                    let label = app_id.or(title).unwrap_or_default();
                    return self.adopt(&surface, &label, pane.as_ref());
                }
                Ok(())
            }
            Event::SurfaceDetached { surface } if !self.overlay => {
                self.forget(&surface)?;
                self.adopt(&surface, "", None)
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
            event => self.input(event),
        }
    }

    #[allow(clippy::too_many_lines, reason = "one arm per method")]
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
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
                let [fx, fz] = self.pose.forward();
                let at = self.panes[i].at;
                let to = [
                    at[0] + fx * ahead + fz * right,
                    at[1] + fz * ahead - fx * right,
                ];
                self.move_pane(i, to).map_err(rejected)?;
                Ok(Value::Null)
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
        self.rt.ungrab_all().map_err(rejected)?;
        if self.flown.is_some() {
            self.grab_fly_out().map_err(rejected)?;
        }
        self.frame().map_err(rejected)
    }
}

impl Construct {
    /// The keys and the pointer of the walk, and the fly-out binding.
    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    fn input(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => {
                if let Some(&action) = self.settings.value.keys.get(&key) {
                    self.act(action);
                }
            }
            Event::InputKey {
                state: PressState::Pressed,
                ..
            } => return self.fly_out(),
            Event::PointerPressed {
                node,
                position,
                modifiers,
                ..
            } if node == LOOK => {
                // A window of an overlay may have taken the keyboard; a
                // press on the space takes it back.
                if self.overlay && self.flown.is_none() {
                    self.rt.conn.apply(vec![Command::FocusNode {
                        node: Some(LOOK.into()),
                    }])?;
                }
                let held = modifiers.contains(&self.settings.value.modifier);
                if let Some(i) = self.pane_at(position).filter(|_| held) {
                    let pane = &self.panes[i];
                    self.moving = Some(Moving {
                        name: pane.name.clone(),
                        from: position,
                        at: pane.at,
                    });
                } else {
                    self.drag = Some((position, self.pose, false));
                }
            }
            Event::PointerMoved { node, position, .. } if node == LOOK => {
                if self.moving.is_some() {
                    return self.drag_pane(position);
                }
                return self.look(position);
            }
            Event::PointerReleased { node, position, .. } if node == LOOK => {
                if self.moving.take().is_some() {
                    return Ok(());
                }
                let click = self.drag.take().is_some_and(|(_, _, moved)| !moved);
                if let Some(i) = self.pane_at(position).filter(|_| click) {
                    self.fly_in(i);
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
        let r = self.radius;
        self.panes[i].at = if len > r {
            [to[0] * r / len, to[1] * r / len]
        } else {
            to
        };
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
        if let Some(surface) = pane.window().filter(|_| !self.overlay) {
            commands.push(configure(surface, size));
        }
        self.rt.conn.apply(commands)?;
        self.frame()
    }

    /// Removes a pane; its window gets a close request, so a program with
    /// unsaved state can ask the user first (F13). The window of another
    /// flavor's owner gets it through `node.requested`.
    fn remove_pane(&mut self, i: usize) -> Result<(), ClientError> {
        let Some(surface) = self.panes[i].window().map(String::from) else {
            let pane = self.panes.remove(i);
            self.order.retain(|n| *n != pane.node());
            self.rt.conn.apply(pane.remove().into())?;
            return self.frame();
        };
        self.forget(&surface)?;
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
        for node in self.rt.conn.query("kind:surface")? {
            let ours = node.to.as_deref().is_none_or(|t| t == NAME);
            if node.owner.is_none() && ours {
                let info = node.surface.unwrap_or_default();
                let label = info.app_id.or(info.title).unwrap_or_default();
                self.adopt(&node.id, &label, None)?;
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
            match self.rt.conn.next_event_timeout(wait)? {
                Some(event) => runtime::dispatch(self, event)?,
                None if plugins => self.retry_plugins()?,
                None => {}
            }
            if self.glide.is_some() {
                self.step_glide()?;
            }
        }
    }

    /// The group of the space with the fog, the panes and the look rect,
    /// on the screen root of the output, at its size; the windows stay.
    /// As a flavor, it claims the output, so that new windows come to it
    /// also beside another window manager (D55).
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
            None => commands = space_nodes(&self.fog_shader, &root),
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
        self.frame()
    }

    /// A unique label from `base`: the app id or title of a window, or
    /// the scene of a plugin pane, with `-2`, `-3` and on after it.
    fn label(&self, base: &str) -> String {
        let base: String = base.trim().chars().take(32).collect();
        let base = if base.is_empty() { "pane".into() } else { base };
        let taken = |n: &str| self.panes.iter().any(|p| p.name == n);
        if !taken(&base) {
            return base;
        }
        (2..=self.panes.len() + 2)
            .map(|k| format!("{base}-{k}"))
            .find(|n| !taken(n))
            .unwrap_or_default()
    }

    /// The place of a new pane of `width` (C7), or the place of its file.
    fn place(&self, at: Option<[f64; 2]>, width: f64) -> [f64; 2] {
        at.unwrap_or_else(|| {
            let others: Vec<_> = self.panes.iter().map(|p| (p.at, p.size[0])).collect();
            let distance = self.settings.value.pane_distance;
            space::place(&self.pose, distance, width, &others)
        })
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
                pane_uniforms(&pane.source, 0.0, fog),
            ),
        ]
    }

    /// A new window becomes a pane, ahead of the camera or where its file
    /// puts it. A flavor hides the window under the space, and its pane
    /// shows it through a node texture; an overlay only shows the window
    /// of the other flavor.
    fn adopt(
        &mut self,
        surface: &str,
        label: &str,
        file: Option<&PaneFile>,
    ) -> Result<(), ClientError> {
        if !self.active || self.panes.iter().any(|p| p.window() == Some(surface)) {
            return Ok(());
        }
        let size = (file.and_then(|f| f.size)).unwrap_or([self.lens.w, self.lens.h]);
        let pane = Pane {
            name: self.label(label),
            source: surface.into(),
            content: Content::Window {
                command: file.and_then(|f| f.command.clone()),
            },
            at: self.place(file.and_then(|f| f.at), size[0]),
            size,
        };
        let mut commands = Vec::new();
        if !self.overlay {
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

    fn resize(&mut self, surface: &str, size: [f64; 2]) -> Result<(), ClientError> {
        let Some(pane) = (self.panes.iter_mut()).find(|p| p.window() == Some(surface)) else {
            return Ok(());
        };
        if pane.size.iter().zip(size).all(|(a, b)| (a - b).abs() < 0.5) {
            return Ok(());
        }
        pane.size = size;
        let mut commands = resized(pane);
        if !self.overlay && self.flown.as_ref().is_some_and(|f| f.surface == surface) {
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
        [t.x, t.z] = space::leash([t.x, t.z], step, self.radius);
        self.glide_to(t, STEP_MS, None);
    }

    /// A drag looks around: sideways turns, up and down tilts.
    fn look(&mut self, p: [f64; 2]) -> Result<(), ClientError> {
        let Some((from, pose, moved)) = &mut self.drag else {
            return Ok(());
        };
        let (dx, dy) = (p[0] - from[0], p[1] - from[1]);
        *moved |= dx.abs() + dy.abs() >= 4.0;
        if !*moved {
            return Ok(());
        }
        let mut to = *pose;
        to.yaw += dx * LOOK_RATE;
        to.pitch = (to.pitch - dy * LOOK_RATE).clamp(-1.2, 1.2);
        self.glide = None;
        self.pose = to;
        self.frame()
    }

    /// The pane under a point of the output, the nearest one.
    fn pane_at(&self, p: [f64; 2]) -> Option<usize> {
        (self.panes.iter().enumerate())
            .filter_map(|(i, pane)| {
                space::hit(&self.lens, &self.pose, p, pane.at, pane.size).map(|t| (i, t))
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

    /// Flies in front of a pane. The window of a window pane then takes
    /// over; a plugin pane stays a pane, at 1:1.
    fn fly_in(&mut self, i: usize) {
        let pane = &self.panes[i];
        let to = space::fly_in(&self.lens, [self.pose.x, self.pose.z], pane.at);
        let back = self.target();
        let into = (pane.window()).map(|surface| Flown {
            surface: surface.into(),
            back,
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
    /// An overlay leaves the window where its owner has it: the pane
    /// shows it at 1:1, and the owner gets the request for the keyboard
    /// (D31).
    fn snap(&mut self, flown: Flown) -> Result<(), ClientError> {
        let surface = flown.surface.clone();
        let Some(pane) = self.panes.iter().find(|p| p.window() == Some(&surface)) else {
            return Ok(());
        };
        let commands = if self.overlay {
            vec![Command::Focus {
                surface: Some(surface),
            }]
        } else {
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
        };
        self.rt.conn.apply(commands)?;
        self.flown = Some(flown);
        self.grab_fly_out()
    }

    fn grab_fly_out(&mut self) -> Result<(), ClientError> {
        let binding = self.settings.value.fly_out.parse().map_err(|e| {
            ClientError::Rpc(RpcError::new(RpcError::INVALID_PARAMS, format!("{e}")))
        })?;
        self.rt.grab(binding, ()).map(drop)
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
        self.rt.ungrab_all()
    }

    fn fly_out(&mut self) -> Result<(), ClientError> {
        let Some(flown) = self.flown.take() else {
            return Ok(());
        };
        self.unsnap((!self.overlay).then_some(&flown.surface))?;
        self.glide_to(flown.back, FLY_MS, None);
        self.frame()
    }

    /// The sky, the ground and the fog for the pose, with the shadows of
    /// the nearest panes, `near` first.
    fn space_frame(&self, near: &[&Pane]) -> Vec<Command> {
        let palette = self.settings.value.theme.palette();
        let edge = (self.pose.x.hypot(self.pose.z) / self.radius).min(1.0);
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
            ("dust".into(), vec3(palette.dust)),
        ]);
        for k in 0..SHADOWS {
            let shadow = near.get(k).map_or(vec![0.0; 4], |p| {
                let lift = (self.lens.ground - p.size[1] / 2.0).max(0.0);
                vec![p.at[0], p.at[1], p.size[0] / 2.0, lift]
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
    fn frame(&mut self) -> Result<(), ClientError> {
        if self.output.is_none() || !self.active {
            return Ok(());
        }
        let theme = self.settings.value.theme;
        let fog = theme.palette().horizon;
        let gloss = match theme {
            Theme::Light => 0.13,
            Theme::Dark => 0.3,
        };
        let radius = self.radius;
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
        for (pane, depth) in &sorted {
            // A pane at or behind the eye does not show.
            let shown = *depth > self.lens.focal * 0.1;
            let mut props = PropsUpdate {
                visible: Some(shown),
                ..PropsUpdate::default()
            };
            let mut mirror = props.clone();
            if shown {
                let matrix = space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size);
                let far = (depth - self.lens.focal).max(0.0) / radius;
                let haze = (1.0 - (-far * 1.5).exp()) * 0.9;
                props.transform = Some(Transform::from_matrix(matrix));
                props.uniforms = Some(pane_uniforms(&pane.source, haze, fog));
                let k = pane.size[0] / mirror_size(pane.size)[0];
                let matrix = space::mirror_matrix(&self.lens, &self.pose, pane.at, pane.size, k);
                mirror.transform = Some(Transform::from_matrix(matrix));
                mirror.uniforms = Some(mirror_uniforms(&pane.source, gloss * (1.0 - haze)));
            }
            commands.push(update(&pane.node(), props));
            commands.push(update(&pane.mirror(), mirror));
        }
        let order: Vec<String> = sorted.iter().map(|(p, _)| p.node()).collect();
        if order != self.order {
            commands.extend(order.iter().map(|id| Command::Move {
                id: id.clone(),
                parent: None,
                index: None,
            }));
            self.order = order;
        }
        self.rt.conn.apply(commands)
    }

    /// Loads a construct (F18). An overlay takes the output now; a
    /// flavor keeps the windows that it has as panes. The plugin panes
    /// and the panes of the construct before go.
    fn load(&mut self, name: &str, file: file::File) -> Result<(), ClientError> {
        if self.flown.is_some() {
            self.fly_out()?;
        }
        self.glide = None;
        let gone: Vec<Pane> = if self.overlay {
            std::mem::take(&mut self.panes)
        } else {
            let (plugins, windows) = std::mem::take(&mut self.panes)
                .into_iter()
                .partition(|p| p.window().is_none());
            self.panes = windows;
            plugins
        };
        let commands: Vec<Command> = gone.iter().flat_map(Pane::remove).collect();
        self.rt.conn.apply(commands)?;
        self.active = true;
        self.order.clear();
        self.waiting.clear();
        self.build()?;
        self.radius = file.radius.unwrap_or(self.settings.value.radius);
        self.pose = file.camera;
        self.current = Some(name.into());
        for pane in file.panes {
            match pane.kind {
                Kind::Program => {
                    let token = launch(pane.command.as_deref().unwrap_or_default());
                    self.waiting.push(Waiting { token, pane });
                }
                Kind::Plugin => {
                    if let Some(pane) = self.show_plugin(pane)? {
                        self.waiting.push(Waiting { token: None, pane });
                    }
                }
            }
        }
        self.frame()
    }

    /// Gives the output back (F20): the space goes, and the keyboard goes
    /// back to the window that had it. The windows stay with their owner.
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
        self.rt.ungrab_all()?;
        self.active = false;
        self.current = None;
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
        let pane = Pane {
            name: self.label(&scene),
            source: node.id.clone(),
            content: Content::Plugin { plugin, scene },
            at: self.place(file.at, size[0]),
            size,
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
        for Waiting { token, pane } in plugins {
            match self.show_plugin(pane)? {
                Some(pane) => self.waiting.push(Waiting { token, pane }),
                None => shown = true,
            }
        }
        if shown { self.frame() } else { Ok(()) }
    }

    /// The construct as a file: the camera, the radius, the panes and the
    /// panes that wait.
    fn snapshot(&self) -> file::File {
        let camera = self
            .flown
            .as_ref()
            .map_or_else(|| self.target(), |f| f.back);
        let panes = (self.panes.iter().filter_map(Pane::file))
            .chain(self.waiting.iter().map(|w| w.pane.clone()))
            .collect();
        file::File {
            camera,
            radius: Some(self.radius),
            panes,
        }
    }

    /// The answer of `construct.list`: the files, the loaded construct
    /// and its panes with their labels.
    fn list(&self) -> Value {
        let panes: Vec<Value> = (self.panes.iter())
            .map(|p| {
                let mut v = json!({"name": p.name, "at": p.at, "size": p.size});
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
            "Moves a pane by its name: right and ahead, in logical pixels, as the camera \
             looks; negative values move it left or nearer.",
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
            "pane.fly",
            "Flies into a pane by its name, until it covers the output at 1:1.",
            name(),
            &["name"],
        ),
    ]
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

/// Starts a shell command with `choractl run`, as mc does, and answers
/// the activation token that finds its window.
fn launch(command: &str) -> Option<String> {
    let run = std::process::Command::new("choractl")
        .args(["run", "--", "sh", "-c", command])
        .stdin(std::process::Stdio::null())
        .output();
    let out = match run {
        Ok(out) if out.status.success() => out.stdout,
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            eprintln!("chora-construct: {command}: {}", err.trim());
            return None;
        }
        Err(err) => {
            eprintln!("chora-construct: choractl run: {err}");
            return None;
        }
    };
    let answer: Value = serde_json::from_slice(&out).unwrap_or_default();
    answer["token"].as_str().map(String::from)
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
            size: [1200.0, 768.0],
        };
        let started = pane(Content::Window {
            command: Some("foot".into()),
        });
        let saved = started.file().unwrap();
        assert_eq!(saved.kind, Kind::Program);
        assert_eq!(
            (saved.at, saved.size),
            (Some(started.at), Some(started.size))
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
}
