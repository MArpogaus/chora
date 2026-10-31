//! chora-construct, step K1 (D66): a white space with a foggy horizon,
//! a walk on a leash, and the windows of the session as panes that face
//! the camera. A click or a key flies into a pane until it covers the
//! output at 1:1; then the window itself shows, sharp, with the keyboard.
//! A binding flies out again.

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod config;
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
    Command, Event, InputMode, NodeKind, NodeTexture, OutputInfo, PropsUpdate, ROOT, ToplevelState,
    Transform, Uniform,
    ipc::{ClientError, PressState, RpcError},
};
use clap::Parser;
use serde_json::Value;

use crate::{
    config::{Action, Config},
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
const PANES: &str = "construct-panes";
const WINDOWS: &str = "construct-windows";
/// A clear rect over the space: it takes the drags, clicks, the wheel
/// and, through `focus_node`, the keys of the walk.
const LOOK: &str = "construct-look";
const FOG_SHADER: &str = include_str!("../fog.frag");
const PANE_SHADER: &str = include_str!("../pane.frag");
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
}

/// A window in the space.
struct Pane {
    surface: String,
    /// The centre on the ground, (x, z).
    at: [f64; 2],
    /// The size of the window in logical pixels.
    size: [f64; 2],
}

impl Pane {
    fn node(&self) -> String {
        format!("construct-pane-{}", self.surface)
    }
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
    fog_shader: String,
    pane_shader: String,
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
    let start = Pose {
        x: 0.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
    };
    let mut construct = Construct {
        rt,
        settings,
        fog_shader: shader_file("fog.frag", FOG_SHADER)?,
        pane_shader: shader_file("pane.frag", PANE_SHADER)?,
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

/// The nodes of the space, bottom first: the sky, the fog, the panes,
/// the windows and the look rect, which takes the keys of the walk.
fn space_nodes(fog_shader: &str) -> Vec<Command> {
    let group = |input| PropsUpdate {
        input,
        ..PropsUpdate::default()
    };
    vec![
        create(NAME, NodeKind::Group, ROOT, group(None)),
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
/// carries the window.
fn pane_uniforms(surface: &str, haze: f64, fog: [f64; 3]) -> BTreeMap<String, Uniform> {
    let win = Uniform::Texture(NodeTexture {
        node: surface.into(),
    });
    BTreeMap::from([
        ("win".into(), win),
        ("haze".into(), Uniform::Float(haze)),
        ("fog".into(), vec3(fog)),
    ])
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
            Event::SurfaceCreated { to: Some(to), .. } if to != NAME => Ok(()),
            Event::SurfaceCreated { surface, .. } => self.adopt(&surface),
            Event::SurfaceDetached { surface } => {
                self.forget(&surface)?;
                self.adopt(&surface)
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

    fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
        Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method))
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
            Event::PointerPressed { node, position, .. } if node == LOOK => {
                self.drag = Some((self.local(position), self.pose, false));
            }
            Event::PointerMoved { node, position, .. } if node == LOOK => {
                return self.look(position);
            }
            Event::PointerReleased { node, position, .. } if node == LOOK => {
                let click = self.drag.take().is_some_and(|(_, _, moved)| !moved);
                if let Some(i) = self.pane_at(self.local(position)).filter(|_| click) {
                    self.fly_in(i);
                }
            }
            Event::PointerScroll { node, dy, .. } if node == LOOK => self.walk(-dy * WHEEL, 0.0),
            _ => {}
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
        self.build()?;
        let mut methods = Vec::new();
        methods.extend(settings::File::<Config>::methods());
        self.rt.announce(&EVENTS, methods)?;
        for node in self.rt.conn.query("kind:surface")? {
            let ours = node.to.as_deref().is_none_or(|t| t == NAME);
            if node.owner.is_none() && ours {
                self.adopt(&node.id)?;
            }
        }
        Ok(())
    }

    /// Handles events; during a glide, frames come every 16 ms.
    fn serve(&mut self) -> Result<(), ClientError> {
        loop {
            let wait = if self.glide.is_some() {
                FRAME
            } else {
                Duration::from_secs(3600)
            };
            if let Some(event) = self.rt.conn.next_event_timeout(wait)? {
                runtime::dispatch(self, event)?;
            }
            if self.glide.is_some() {
                self.step_glide()?;
            }
        }
    }

    /// The point of the scene in the logical coordinates of the output.
    fn local(&self, p: [f64; 2]) -> [f64; 2] {
        let [x, y] = self.output.as_ref().map_or([0, 0], |o| o.position);
        [p[0] - f64::from(x), p[1] - f64::from(y)]
    }

    /// The group of the space with the fog, the panes and the look rect,
    /// at the size of the output; the windows stay.
    fn build(&mut self) -> Result<(), ClientError> {
        let Some(output) = &self.output else {
            return Ok(());
        };
        let [w, h] = output.size.map(f64::from);
        let [x, y] = output.position.map(f64::from);
        self.lens = Lens::new(w, h);
        let place = Some(Transform::translate(x, y, 0.0));
        let mut commands = Vec::new();
        if self.rt.conn.query(&format!("id:{NAME}"))?.is_empty() {
            commands = space_nodes(&self.fog_shader);
        }
        commands.extend([
            update(
                NAME,
                PropsUpdate {
                    transform: place,
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
        // New windows take the size of the output, so that a fly-in
        // fills it.
        let configure: Vec<Command> = (self.panes.iter())
            .map(|p| self.configure(&p.surface))
            .collect();
        self.rt.conn.apply(configure)?;
        self.frame()
    }

    fn configure(&self, surface: &str) -> Command {
        let [w, h] = self.output.as_ref().map_or([1280, 800], |o| o.size);
        Command::Configure {
            surface: surface.into(),
            size: Some([w, h]),
            states: vec![ToplevelState::Maximized],
        }
    }

    /// A new window becomes a pane ahead of the camera, its window
    /// hidden under the space; its pane shows it through a node texture.
    fn adopt(&mut self, surface: &str) -> Result<(), ClientError> {
        if self.panes.iter().any(|p| p.surface == surface) {
            return Ok(());
        }
        let size = [self.lens.w, self.lens.h];
        let others: Vec<_> = self.panes.iter().map(|p| (p.at, p.size[0])).collect();
        let at = space::place(
            &self.pose,
            self.settings.value.pane_distance,
            size[0],
            &others,
        );
        let pane = Pane {
            surface: surface.into(),
            at,
            size,
        };
        let fog = self.settings.value.theme.colors().1;
        let uniforms = pane_uniforms(surface, 0.0, fog);
        let commands = vec![
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
            self.configure(surface),
            create(
                &pane.node(),
                NodeKind::Shader,
                PANES,
                PropsUpdate {
                    size: Some(size),
                    shader: Some(self.pane_shader.clone()),
                    uniforms: Some(uniforms),
                    ..PropsUpdate::default()
                },
            ),
        ];
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
        let Some(pane) = self.panes.iter_mut().find(|p| p.surface == surface) else {
            return Ok(());
        };
        if pane.size.iter().zip(size).all(|(a, b)| (a - b).abs() < 0.5) {
            return Ok(());
        }
        pane.size = size;
        let node = pane.node();
        let mut commands = vec![update(
            &node,
            PropsUpdate {
                size: Some(size),
                ..PropsUpdate::default()
            },
        )];
        if self.flown.as_ref().is_some_and(|f| f.surface == surface) {
            commands.push(self.snap_place(size, surface));
        }
        self.rt.conn.apply(commands)?;
        self.frame()
    }

    /// A window closed or left: its pane goes. A pane that the camera
    /// is in flies back out first.
    fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some(i) = self.panes.iter().position(|p| p.surface == surface) else {
            return Ok(());
        };
        let pane = self.panes.remove(i);
        let node = pane.node();
        self.order.retain(|n| *n != node);
        self.rt
            .conn
            .apply(vec![Command::Remove { id: node, to: None }])?;
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
        [t.x, t.z] = space::leash([t.x, t.z], step, self.settings.value.radius);
        self.glide_to(t, STEP_MS, None);
    }

    /// A drag looks around: sideways turns, up and down tilts.
    fn look(&mut self, position: [f64; 2]) -> Result<(), ClientError> {
        let p = self.local(position);
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

    fn fly_in(&mut self, i: usize) {
        let pane = &self.panes[i];
        let to = space::fly_in(&self.lens, [self.pose.x, self.pose.z], pane.at);
        let surface = pane.surface.clone();
        let back = self.target();
        self.glide_to(to, FLY_MS, Some(Flown { surface, back }));
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
    fn snap(&mut self, flown: Flown) -> Result<(), ClientError> {
        let surface = flown.surface.clone();
        let Some(pane) = self.panes.iter().find(|p| p.surface == surface) else {
            return Ok(());
        };
        let commands = vec![
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
        ];
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
        self.unsnap(Some(&flown.surface))?;
        self.glide_to(flown.back, FLY_MS, None);
        self.frame()
    }

    /// The sky and the fog for the pose.
    fn space_frame(&self) -> Vec<Command> {
        let (sky, fog) = self.settings.value.theme.colors();
        let edge = (self.pose.x.hypot(self.pose.z) / self.settings.value.radius).min(1.0);
        let uniforms = BTreeMap::from([
            (
                "view".into(),
                Uniform::Vector(vec![self.lens.w, self.lens.h]),
            ),
            (
                "cam".into(),
                Uniform::Vector(vec![self.pose.yaw, self.pose.pitch]),
            ),
            ("focal".into(), Uniform::Float(self.lens.focal)),
            ("sky".into(), vec3(sky)),
            ("fog".into(), vec3(fog)),
            ("edge".into(), Uniform::Float(edge)),
        ]);
        let color = Some([sky[0], sky[1], sky[2], 1.0]);
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

    /// Sends the pose: the matrix and the haze of each pane, back to
    /// front, and the camera of the fog.
    fn frame(&mut self) -> Result<(), ClientError> {
        if self.output.is_none() {
            return Ok(());
        }
        let fog = self.settings.value.theme.colors().1;
        let radius = self.settings.value.radius;
        let mut commands = self.space_frame();
        let mut sorted: Vec<(&Pane, f64)> = (self.panes.iter())
            .map(|p| (p, space::depth(&self.pose, p.at)))
            .collect();
        sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
        for (pane, depth) in &sorted {
            // A pane at or behind the eye does not show.
            let shown = *depth > self.lens.focal * 0.1;
            let mut props = PropsUpdate {
                visible: Some(shown),
                ..PropsUpdate::default()
            };
            if shown {
                let matrix = space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size);
                let far = (depth - self.lens.focal).max(0.0) / radius;
                let haze = (1.0 - (-far * 1.5).exp()) * 0.9;
                props.transform = Some(Transform::from_matrix(matrix));
                props.uniforms = Some(pane_uniforms(&pane.surface, haze, fog));
            }
            commands.push(update(&pane.node(), props));
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
}
