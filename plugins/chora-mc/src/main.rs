//! chora-mc, experimental: a voice-first desktop without window
//! management. The whole screen shows one scene, a grid of text panels or
//! one app, and a quiet HUD line shows what the voice agent does. The
//! agent and small collector scripts fill the panels through the exported
//! methods; the plugin reads no sources itself.

mod config;
mod scene;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::ExitCode,
};

use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Easing, Event, MAX_DURATION, NodeKind, OutputInfo, PropsUpdate, ROOT, ShaderEffect,
    ToplevelState, Transform, Uniform,
    ipc::{ClientError, ExportMethod, PressState, Query, RpcError},
};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::{Action, Config},
    scene::{Panel, Scene},
};

/// The client name, and the id of the group that holds the scenes.
const NAME: &str = "mc";
/// The id of the line of the HUD.
const HUD: &str = "mc-hud";
/// The id of a hidden node whose empty animations time the typewriter.
const TICK: &str = "mc-tick";
/// The cross-fade between two scenes, in milliseconds.
const FADE: f64 = 250.0;
/// The effect that writes new text.
const WRITE_SHADER: &str = include_str!("../write.frag");
/// The height of a line of a text node of the core, by its font size.
const LINE_HEIGHT: f64 = 1.2;

/// One scene at a time on the whole screen, driven by voice.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The settings file. Without it, chora-mc reads
    /// $XDG_CONFIG_HOME/chora/chora-mc.yaml if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default settings and exit.
    #[arg(long)]
    print_config: bool,
    /// The directory of the scenes; `$XDG_CONFIG_HOME/chora/mc/scenes`
    /// without it.
    #[arg(long)]
    scenes: Option<PathBuf>,
}

/// Parameters of `scene.show`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Show {
    name: String,
}

/// Parameters of `scene.save`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    name: String,
    #[serde(flatten)]
    scene: Scene,
}

/// Parameters of `panel.set`; a missing title keeps the title.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetPanel {
    scene: String,
    panel: String,
    title: Option<String>,
    lines: Vec<String>,
}

/// Parameters of `hud.set`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetHud {
    #[serde(default)]
    state: String,
    #[serde(default)]
    text: String,
}

/// Parameters of `hud.say`: the text shows word by word over the
/// duration, as the voice speaks it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Say {
    #[serde(default)]
    state: String,
    text: String,
    duration_ms: f64,
}

/// The words of `hud.say` that wait, with the time of each in
/// milliseconds.
struct Typing {
    words: Vec<(String, f64)>,
    shown: usize,
}

/// Parameters of `app.open`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Open {
    command: String,
}

struct Mc {
    rt: Runtime<Action>,
    settings: settings::File<Config>,
    /// The directory of the scene files.
    dir: PathBuf,
    scenes: BTreeMap<String, Scene>,
    /// The window of an app scene, by scene.
    windows: HashMap<String, String>,
    /// The scene of an app that starts, by its activation token.
    starting: HashMap<String, String>,
    current: Option<String>,
    /// The one output that the plugin fills.
    output: Option<OutputInfo>,
    /// Whether the nodes exist.
    built: bool,
    hud: SetHud,
    typing: Option<Typing>,
    /// The file of the effect that writes new text.
    shader: PathBuf,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    runtime::run(NAME, || play(&args))
}

fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-mc")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let dir = args
        .scenes
        .clone()
        .or_else(|| chora_kit::xdg::config_home().map(|d| d.join("chora/mc/scenes")))
        .ok_or("no $XDG_CONFIG_HOME or $HOME")?;
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let runtime =
        std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let shader = runtime.join(format!("chora-mc-{}/write.frag", std::process::id()));
    std::fs::create_dir_all(shader.parent().expect("a file in a directory"))?;
    std::fs::write(&shader, WRITE_SHADER)?;
    let rt = Runtime::start(NAME, Vec::new(), &EVENTS, methods)?;
    let mut mc = Mc {
        rt,
        settings,
        scenes: scene::load(&dir),
        dir,
        windows: HashMap::new(),
        starting: HashMap::new(),
        current: None,
        output: None,
        built: false,
        hud: SetHud {
            state: String::new(),
            text: String::new(),
        },
        typing: None,
        shader,
    };
    mc.start()?;
    Ok(runtime::serve(&mut mc)?)
}

const EVENTS: [&str; 6] = [
    "animation.done",
    "surface.created",
    "surface.destroyed",
    "surface.detached",
    "output.changed",
    "output.removed",
];

#[allow(clippy::too_many_lines)] // one entry per method
fn exported_methods() -> Vec<ExportMethod> {
    let lines = json!({"type": "array", "items": {"type": "string"}});
    let panels = json!({"type": "array", "items": {"type": "object"}, "description":
        "each with id, cell [column, row, width, height], title and lines"});
    vec![
        runtime::method(
            "scene.show",
            "Shows a scene with a short cross-fade.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::method(
            "scene.list",
            "Answers the scenes and the one that shows.",
            json!({}),
            &[],
        ),
        runtime::method(
            "scene.save",
            "Saves a scene in its file and shows the change: a grid of panels, or an app.",
            json!({
                "name": {"type": "string"},
                "grid": {"type": "array", "items": {"type": "integer"}, "description": "columns and rows"},
                "panels": panels,
                "app": {"type": "string", "description": "a shell command whose window fills the screen"},
            }),
            &["name"],
        ),
        runtime::method(
            "panel.set",
            "Sets the title and the lines of a panel of a scene, until the next start.",
            json!({
                "scene": {"type": "string"},
                "panel": {"type": "string"},
                "title": {"type": "string"},
                "lines": lines,
            }),
            &["scene", "panel", "lines"],
        ),
        runtime::method(
            "hud.set",
            "Sets the HUD line: the state of the agent, such as listening, thinking or \
             working, and a short message; both empty hide it.",
            json!({"state": {"type": "string"}, "text": {"type": "string"}}),
            &[],
        ),
        runtime::method(
            "hud.say",
            "Shows the text in the HUD word by word over duration_ms, the time that the \
             voice takes to speak it; a state as in hud.set shows before it.",
            json!({
                "state": {"type": "string"},
                "text": {"type": "string"},
                "duration_ms": {"type": "number"},
            }),
            &["text", "duration_ms"],
        ),
        runtime::method(
            "app.open",
            "Starts a shell command as an app in a full-screen scene of its own.",
            json!({"command": {"type": "string"}}),
            &["command"],
        ),
    ]
}

impl Plugin for Mc {
    type Action = Action;

    fn runtime(&mut self) -> &mut Runtime<Action> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::SurfaceCreated { to: Some(to), .. } if to != NAME => Ok(()),
            Event::SurfaceCreated {
                surface,
                app_id,
                token,
                ..
            } => self.adopt(&surface, app_id.as_deref(), token.as_deref()),
            Event::SurfaceDetached { surface } => self.adopt(&surface, None, None),
            Event::SurfaceDestroyed { surface } => self.forget(&surface),
            Event::OutputChanged { output }
                if self.output.as_ref().is_none_or(|o| o.name == output.name) =>
            {
                self.output = Some(output);
                self.build()
            }
            Event::OutputRemoved { name }
                if self.output.as_ref().is_some_and(|o| o.name == name) =>
            {
                self.output = None;
                Ok(())
            }
            Event::AnimationDone {
                node: Some(node), ..
            } if node == TICK => self.type_next(),
            Event::InputKey {
                binding,
                state: PressState::Pressed,
            } => {
                self.act(&binding);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        match method {
            "scene.show" => {
                let Show { name } = runtime::params(params)?;
                self.show(&name)?;
                Ok(Value::Null)
            }
            "scene.list" => Ok(json!({
                "scenes": self.scenes.keys().collect::<Vec<_>>(),
                "current": self.current,
            })),
            "scene.save" => {
                let Save { name, scene } = runtime::params(params)?;
                scene::save(&self.dir, &name, &scene).map_err(invalid)?;
                self.scenes.insert(name.clone(), scene);
                self.build().map_err(rejected)?;
                Ok(Value::Null)
            }
            "panel.set" => {
                let set: SetPanel = runtime::params(params)?;
                self.set_panel(set)?;
                Ok(Value::Null)
            }
            "hud.set" => {
                self.hud = runtime::params(params)?;
                self.typing = None;
                let update = self.hud_update();
                self.apply(vec![update]).map_err(rejected)?;
                Ok(Value::Null)
            }
            "hud.say" => {
                let say: Say = runtime::params(params)?;
                self.say(say).map_err(rejected)?;
                Ok(Value::Null)
            }
            "app.open" => {
                let Open { command } = runtime::params(params)?;
                let name = self.app_scene(&command)?;
                self.show(&name)?;
                Ok(json!({"scene": name}))
            }
            _ => Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.rt
            .ungrab_all()
            .and_then(|()| self.grab_all())
            .and_then(|()| self.build())
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

fn invalid(message: String) -> RpcError {
    RpcError::new(RpcError::INVALID_PARAMS, message)
}

/// The id of the group of a scene.
fn scene_id(name: &str) -> String {
    format!("mc-s-{name}")
}

/// The id of the group of the panels of a scene.
fn panels_id(name: &str) -> String {
    format!("mc-s-{name}-panels")
}

/// Adds a word to a text, with a space after other words.
fn push_word(text: &mut String, word: &str) {
    if !text.is_empty() {
        text.push(' ');
    }
    text.push_str(word);
}

/// The id of the group of a panel.
fn panel_id(scene: &str, panel: &str) -> String {
    format!("{}-{panel}", panels_id(scene))
}

/// A name for a scene from an app id or a command: its last path part,
/// with the characters that a name cannot have as `-`.
fn name_of(word: &str) -> String {
    let base = word.rsplit('/').next().unwrap_or(word);
    let name: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-".contains(c) {
                c
            } else {
                '-'
            }
        })
        .take(48)
        .collect();
    let name = name.trim_start_matches('.');
    if name.is_empty() {
        "app".into()
    } else {
        name.into()
    }
}

fn create(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

fn translate(x: f64, y: f64) -> Transform {
    Transform::translate(x, y, 0.0)
}

impl Mc {
    fn start(&mut self) -> Result<(), ClientError> {
        self.output = self.rt.outputs()?.into_iter().next();
        // The nodes that the core kept over a restart: the windows in them
        // become free and come back as scenes of their own.
        let kept: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new(format!("owner:{NAME}")))?;
        let old = kept
            .iter()
            .filter_map(|n| n["id"].as_str())
            .filter(|id| *id == NAME || *id == HUD)
            .map(|id| Command::Remove {
                id: id.into(),
                to: None,
            })
            .collect();
        self.apply(old)?;
        self.grab_all()?;
        self.build()?;
        let first = self.scenes.keys().next().cloned();
        let default = Some(self.settings.value.default_scene.clone())
            .filter(|v| self.scenes.contains_key(v))
            .or(first);
        if let Some(name) = default
            && let Err(err) = self.show(&name)
        {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        let free: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new("kind:surface"))?;
        for node in free {
            let ours = node["to"].is_null() || node["to"] == NAME;
            if let (Some(id), true, true) = (node["id"].as_str(), node["owner"].is_null(), ours) {
                let app_id = node["surface"]["app_id"].as_str();
                self.adopt(id, app_id, None)?;
            }
        }
        Ok(())
    }

    /// Grabs the bindings of the settings; one that another client holds
    /// only logs.
    fn grab_all(&mut self) -> Result<(), ClientError> {
        for (text, action) in self.settings.value.bindings.clone() {
            let binding = text.parse().expect("the settings check their bindings");
            self.rt.grab(binding, action)?;
        }
        Ok(())
    }

    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.apply(commands)
    }

    /// Builds all nodes again, for example after a change of the theme or
    /// of the output. The windows wait under the root meanwhile, all in one
    /// transaction, so nothing flickers.
    fn build(&mut self) -> Result<(), ClientError> {
        let Some(output) = self.output.clone() else {
            return Ok(());
        };
        let mut commands = if self.built {
            self.unbuild()
        } else {
            Vec::new()
        };
        self.finish_typing();
        let [w, h] = output.size.map(f64::from);
        let [x, y] = output.position.map(f64::from);
        let colors = self.settings.value.colors().clone();
        commands.push(create(
            NAME,
            NodeKind::Group,
            ROOT,
            PropsUpdate {
                transform: Some(translate(x, y)),
                ..PropsUpdate::default()
            },
        ));
        commands.push(create(TICK, NodeKind::Group, NAME, PropsUpdate::default()));
        commands.push(create(
            "mc-bg",
            NodeKind::Rect,
            NAME,
            PropsUpdate {
                size: Some([w, h]),
                color: Some(colors.background),
                ..PropsUpdate::default()
            },
        ));
        for name in self.scenes.keys() {
            let shown = self.current.as_ref() == Some(name);
            commands.push(create(
                &scene_id(name),
                NodeKind::Group,
                NAME,
                PropsUpdate {
                    visible: Some(shown),
                    opacity: Some(if shown { 1.0 } else { 0.0 }),
                    ..PropsUpdate::default()
                },
            ));
            commands.extend(self.panels(name));
            if let Some(surface) = self.windows.get(name) {
                commands.extend(self.place_window(name, surface, false));
            }
        }
        commands.push(create(
            HUD,
            NodeKind::Text,
            &format!("screen:{}", output.name),
            self.hud_props(),
        ));
        self.apply(commands)?;
        self.built = true;
        let names: Vec<String> = self.scenes.keys().cloned().collect();
        names.iter().try_for_each(|name| self.align(name))
    }

    /// Removes all nodes; the windows wait under the root.
    fn unbuild(&self) -> Vec<Command> {
        let moves = self.windows.values().map(|surface| Command::Move {
            id: surface.clone(),
            parent: Some(ROOT.into()),
            index: None,
        });
        let removes = [NAME, HUD].map(|id| Command::Remove {
            id: id.into(),
            to: None,
        });
        moves.chain(removes).collect()
    }

    /// Creates the panels of a scene, under a group of their own that the
    /// next call replaces.
    fn panels(&self, name: &str) -> Vec<Command> {
        let Some(scene) = self.scenes.get(name) else {
            return Vec::new();
        };
        let mut commands = vec![create(
            &panels_id(name),
            NodeKind::Group,
            &scene_id(name),
            PropsUpdate::default(),
        )];
        for panel in &scene.panels {
            commands.extend(self.panel(name, panel));
        }
        commands
    }

    /// The box of a panel on the screen.
    fn place(&self, scene: &Scene, panel: &Panel) -> Option<[f64; 4]> {
        let output = self.output.as_ref()?;
        let margin = self.settings.value.font.size * 3.0;
        let size = output.size.map(f64::from);
        Some(scene::place(panel.cell, scene.grid, size, margin))
    }

    /// Creates one panel: a group with a clip, its title and its lines.
    fn panel(&self, name: &str, panel: &Panel) -> Vec<Command> {
        let Some([x, y, w, h]) = self.scenes.get(name).and_then(|s| self.place(s, panel)) else {
            return Vec::new();
        };
        let config = &self.settings.value;
        let colors = config.colors();
        let size = config.font.size;
        let id = panel_id(name, &panel.id);
        let mut commands = vec![create(
            &id,
            NodeKind::Group,
            &panels_id(name),
            PropsUpdate {
                transform: Some(translate(x, y)),
                size: Some([w, h]),
                clip: Some(Some(chora_scene::Clip { radius: 0.0 })),
                ..PropsUpdate::default()
            },
        )];
        let text = |text: String, font_size: f64, color: [f64; 4], top: f64| PropsUpdate {
            text: Some(text),
            family: Some(config.font.family.clone()),
            font_size: Some(font_size),
            max_width: Some(w),
            color: Some(color),
            transform: Some(translate(0.0, top)),
            ..PropsUpdate::default()
        };
        let title_size = size * 1.5;
        if !panel.title.is_empty() {
            let props = text(panel.title.clone(), title_size, colors.accent, 0.0);
            commands.push(create(&format!("{id}-title"), NodeKind::Text, &id, props));
        }
        if let Some((times, texts)) = panel.columns() {
            // Two columns that do not wrap, so that their rows match;
            // `align` moves the second one next to the first.
            let top = title_size * 2.0;
            let times = PropsUpdate {
                max_width: None,
                ..text(times, size, colors.text, top)
            };
            let texts = PropsUpdate {
                max_width: None,
                ..text(texts, size, colors.text, top)
            };
            commands.push(create(&format!("{id}-times"), NodeKind::Text, &id, times));
            commands.push(create(&format!("{id}-lines"), NodeKind::Text, &id, texts));
        } else if !panel.lines.is_empty() {
            let props = text(panel.text(), size, colors.text, title_size * 2.0);
            commands.push(create(&format!("{id}-lines"), NodeKind::Text, &id, props));
        }
        commands
    }

    /// Puts the second column of the panels of a scene next to the
    /// first, as wide as the core draws the first.
    fn align(&mut self, name: &str) -> Result<(), ClientError> {
        let Some(scene) = self.scenes.get(name) else {
            return Ok(());
        };
        let size = self.settings.value.font.size;
        let ids: Vec<String> = (scene.panels.iter())
            .filter(|p| p.columns().is_some())
            .map(|p| panel_id(name, &p.id))
            .collect();
        let mut commands = Vec::new();
        for id in ids {
            let nodes: Vec<Value> =
                (self.rt.conn).call("scene.query", Query::new(format!("id:{id}-times")))?;
            let width = nodes
                .first()
                .and_then(|n| n["rects"].as_object()?.values().next()?.get(2)?.as_f64())
                .unwrap_or(size * 6.0);
            commands.push(Command::Update {
                id: format!("{id}-lines"),
                props: PropsUpdate {
                    transform: Some(translate(width + size, size * 3.0)),
                    ..PropsUpdate::default()
                },
            });
        }
        self.apply(commands)
    }

    /// Writes the text of the panels of a scene anew: the effect starts at
    /// 0 and the core animates it to 1. Nothing when the setting is off.
    #[allow(clippy::cast_precision_loss)] // at most 4 KiB of lines
    fn write(&self, name: &str, panels: &[&Panel]) -> Vec<Command> {
        let config = &self.settings.value;
        let Some(scene) = self.scenes.get(name).filter(|_| config.write.enabled) else {
            return Vec::new();
        };
        let size = config.font.size;
        let effect = |rows: [f64; 3], progress: f64| {
            Some(vec![ShaderEffect {
                shader: self.shader.to_string_lossy().into_owned(),
                uniforms: [
                    ("progress".into(), Uniform::Float(progress)),
                    ("rows".into(), Uniform::Vector(rows.to_vec())),
                ]
                .into(),
            }])
        };
        let mut commands = Vec::new();
        for panel in panels {
            let Some([_, _, _, h]) = self.place(scene, panel) else {
                continue;
            };
            // ponytail: a line that wraps writes with the last line; the
            // plugin does not know the layout of the core.
            let rows = [
                3.0 * size / h,
                LINE_HEIGHT * size / h,
                1.0 + panel.lines.len() as f64,
            ];
            let id = panel_id(name, &panel.id);
            commands.push(Command::Update {
                id: id.clone(),
                props: PropsUpdate {
                    effects: effect(rows, 0.0),
                    ..PropsUpdate::default()
                },
            });
            commands.push(Command::Animate {
                id,
                props: PropsUpdate {
                    effects: effect(rows, 1.0),
                    ..PropsUpdate::default()
                },
                duration: config.write.duration_ms,
                easing: Easing::Linear,
            });
        }
        commands
    }

    /// Puts a window into its scene at the size of the screen, without a
    /// frame.
    fn place_window(&self, name: &str, surface: &str, attach: bool) -> Vec<Command> {
        let Some(output) = &self.output else {
            return Vec::new();
        };
        let parent = scene_id(name);
        let put = if attach {
            Command::Attach {
                surface: surface.into(),
                parent,
            }
        } else {
            Command::Move {
                id: surface.into(),
                parent: Some(parent),
                index: None,
            }
        };
        vec![
            put,
            Command::Configure {
                surface: surface.into(),
                size: Some(output.size),
                states: vec![ToplevelState::Fullscreen],
            },
        ]
    }

    fn hud_text(&self) -> String {
        let SetHud { state, text } = &self.hud;
        match (state.is_empty(), text.is_empty()) {
            (false, false) => format!("{state} · {text}"),
            (false, true) => state.clone(),
            _ => text.clone(),
        }
    }

    fn hud_props(&self) -> PropsUpdate {
        let config = &self.settings.value;
        let text = self.hud_text();
        let size = config.font.size;
        let height = self.output.as_ref().map_or(0.0, |o| f64::from(o.size[1]));
        PropsUpdate {
            visible: Some(!text.is_empty()),
            // A text node needs a text.
            text: Some(if text.is_empty() { " ".into() } else { text }),
            family: Some(config.font.family.clone()),
            font_size: Some(size * 0.8),
            color: Some(config.colors().accent),
            transform: Some(translate(size * 3.0, height - size * 2.0)),
            ..PropsUpdate::default()
        }
    }

    fn hud_update(&self) -> Command {
        Command::Update {
            id: HUD.into(),
            props: self.hud_props(),
        }
    }

    fn show(&mut self, name: &str) -> Result<(), RpcError> {
        let Some(scene) = self.scenes.get(name) else {
            return Err(invalid(format!("no scene {name}")));
        };
        let app = scene.app.clone();
        let mut commands = Vec::new();
        if self.current.as_deref() != Some(name) {
            if let Some(old) = &self.current {
                commands.push(Command::Animate {
                    id: scene_id(old),
                    props: PropsUpdate {
                        opacity: Some(0.0),
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                    duration: FADE,
                    easing: Easing::EaseInOut,
                });
            }
            commands.push(Command::Update {
                id: scene_id(name),
                props: PropsUpdate {
                    visible: Some(true),
                    ..PropsUpdate::default()
                },
            });
            commands.push(Command::Animate {
                id: scene_id(name),
                props: PropsUpdate {
                    opacity: Some(1.0),
                    ..PropsUpdate::default()
                },
                duration: FADE,
                easing: Easing::EaseInOut,
            });
            let panels: Vec<&Panel> = scene.panels.iter().collect();
            commands.extend(self.write(name, &panels));
        }
        let window = self.windows.get(name).cloned();
        commands.push(Command::Focus {
            surface: window.clone(),
        });
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        self.current = Some(name.into());
        let waiting = self.starting.values().any(|v| v == name);
        if let (Some(command), None, false) = (app, window, waiting) {
            self.launch(name, &command);
        }
        Ok(())
    }

    /// Starts the app of a scene with `choractl run`, whose activation
    /// token finds its window again.
    fn launch(&mut self, name: &str, command: &str) {
        let run = std::process::Command::new("choractl")
            .args(["run", "--", "sh", "-c", command])
            .stdin(std::process::Stdio::null())
            .output();
        let line = match run {
            Ok(out) if out.status.success() => out.stdout,
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                eprintln!("chora-mc: {command}: {}", err.trim());
                return;
            }
            Err(err) => {
                eprintln!("chora-mc: choractl run: {err}");
                return;
            }
        };
        let answer: Value = serde_json::from_slice(&line).unwrap_or_default();
        if let Some(token) = answer["token"].as_str() {
            self.starting.insert(token.into(), name.into());
        }
    }

    /// The scene of `app.open`, named after the program.
    fn app_scene(&mut self, command: &str) -> Result<String, RpcError> {
        let word = command
            .split_whitespace()
            .next()
            .ok_or_else(|| invalid("an empty command".into()))?;
        // The scene of the same command again, or the first free name of
        // the program: foot, foot-2, foot-3 ...
        let base = name_of(word);
        let mut name = base.clone();
        for n in 2.. {
            match self.scenes.get(&name) {
                Some(scene) if scene.app.as_deref() == Some(command) => return Ok(name),
                Some(_) => name = format!("{base}-{n}"),
                None => break,
            }
        }
        self.scenes.insert(name.clone(), Scene::app(command));
        let group = create(
            &scene_id(&name),
            NodeKind::Group,
            NAME,
            PropsUpdate {
                visible: Some(false),
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        );
        self.apply(vec![group])
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(name)
    }

    /// Takes a new window into the scene that started it, or into a new
    /// scene of its own, and shows it.
    fn adopt(
        &mut self,
        surface: &str,
        app_id: Option<&str>,
        token: Option<&str>,
    ) -> Result<(), ClientError> {
        if !self.built {
            return Ok(());
        }
        let started = token.and_then(|t| self.starting.remove(t));
        let mut commands = Vec::new();
        let name = match started {
            Some(name) if !self.windows.contains_key(&name) => name,
            _ => {
                let mut name = name_of(app_id.unwrap_or("app"));
                if self.scenes.contains_key(&name) {
                    name = format!("{name}-{surface}");
                }
                self.scenes.insert(name.clone(), Scene::default());
                commands.push(create(
                    &scene_id(&name),
                    NodeKind::Group,
                    NAME,
                    PropsUpdate {
                        visible: Some(false),
                        opacity: Some(0.0),
                        ..PropsUpdate::default()
                    },
                ));
                name
            }
        };
        commands.extend(self.place_window(&name, surface, true));
        self.apply(commands)?;
        self.windows.insert(name.clone(), surface.into());
        if let Err(err) = self.show(&name) {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        Ok(())
    }

    /// A window closed: a scene that only held it goes, and the default
    /// scene shows instead of a scene without its window.
    fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some(name) = self
            .windows
            .iter()
            .find(|(_, s)| *s == surface)
            .map(|(n, _)| n.clone())
        else {
            return Ok(());
        };
        self.windows.remove(&name);
        if self.current.as_deref() == Some(&name) {
            let default = self.settings.value.default_scene.clone();
            if let Err(err) = self.show(&default) {
                eprintln!("chora-mc: {default}: {}", err.message);
            }
        }
        if self.scenes[&name].app.is_none() && self.scenes[&name].panels.is_empty() {
            self.scenes.remove(&name);
            self.apply(vec![Command::Remove {
                id: scene_id(&name),
                to: None,
            }])?;
        }
        Ok(())
    }

    fn set_panel(&mut self, set: SetPanel) -> Result<(), RpcError> {
        let SetPanel {
            scene,
            panel,
            title,
            lines,
        } = set;
        let found = self
            .scenes
            .get_mut(&scene)
            .and_then(|v| v.panels.iter_mut().find(|p| p.id == panel))
            .ok_or_else(|| invalid(format!("no panel {panel} in scene {scene}")))?;
        let new = Panel {
            title: title.unwrap_or_else(|| found.title.clone()),
            lines,
            ..found.clone()
        };
        new.check_text().map_err(invalid)?;
        // The same text again neither writes nor builds anything.
        if *found == new {
            return Ok(());
        }
        *found = new.clone();
        let mut commands = vec![Command::Remove {
            id: panel_id(&scene, &panel),
            to: None,
        }];
        commands.extend(self.panel(&scene, &new));
        if self.current.as_deref() == Some(&scene) {
            commands.extend(self.write(&scene, &[&new]));
        }
        self.apply(commands)
            .and_then(|()| self.align(&scene))
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }

    /// Shows the rest of `hud.say` at once: the next build has a new tick
    /// node without an animation.
    fn finish_typing(&mut self) {
        if let Some(typing) = self.typing.take() {
            for (word, _) in &typing.words[typing.shown..] {
                push_word(&mut self.hud.text, word);
            }
        }
    }

    /// Starts `hud.say`: the first word shows now, each next one after
    /// the time of the word before, by its share of the letters.
    #[allow(clippy::cast_precision_loss)] // at most 4 KiB of text
    fn say(&mut self, say: Say) -> Result<(), ClientError> {
        let words: Vec<&str> = say.text.split_whitespace().collect();
        let letters: usize = words.iter().map(|w| w.chars().count() + 1).sum();
        let ms = say.duration_ms.clamp(0.0, MAX_DURATION) / letters.max(1) as f64;
        let words = words
            .iter()
            .map(|w| ((*w).to_owned(), ms * (w.chars().count() + 1) as f64))
            .collect();
        self.hud = SetHud {
            state: say.state,
            text: String::new(),
        };
        self.typing = Some(Typing { words, shown: 0 });
        self.type_next()
    }

    /// Shows the next word of `hud.say` and waits for the one after it.
    fn type_next(&mut self) -> Result<(), ClientError> {
        let Some(typing) = &mut self.typing else {
            return Ok(());
        };
        let Some((word, ms)) = typing.words.get(typing.shown).cloned() else {
            self.typing = None;
            return Ok(());
        };
        typing.shown += 1;
        push_word(&mut self.hud.text, &word);
        let tick = Command::Animate {
            id: TICK.into(),
            props: PropsUpdate::default(),
            duration: ms,
            easing: Easing::Linear,
        };
        let update = self.hud_update();
        self.apply(vec![update, tick])
    }

    fn act(&mut self, binding: &str) {
        let Some(action) = self.rt.action(binding).cloned() else {
            return;
        };
        let names: Vec<String> = self.scenes.keys().cloned().collect();
        let at = self
            .current
            .as_ref()
            .and_then(|c| names.iter().position(|n| n == c))
            .unwrap_or(0);
        let next = match action {
            Action::NextScene => names[(at + 1) % names.len()].clone(),
            Action::PreviousScene => names[(at + names.len() - 1) % names.len()].clone(),
            Action::Scene(name) => name,
            Action::Run(command) => {
                chora_kit::run(&command);
                return;
            }
        };
        if let Err(err) = self.show(&next) {
            eprintln!("chora-mc: {next}: {}", err.message);
        }
    }
}
