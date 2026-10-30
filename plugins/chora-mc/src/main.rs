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
    config::{Action, Config, Layout, Style, Type},
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
/// The lines of the baseline grid above the head of a scene.
const HEAD: f64 = 3.0;
/// The line of the baseline grid of the running head.
const RUNNING: f64 = 1.0;

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

/// Where the parts of the page of a scene go, in logical pixels.
struct Geometry {
    /// The text block: x, y, width and height.
    text: [f64; 4],
    /// The width of the margin column left of the text.
    note: f64,
    /// The space between the margin and the text, and between columns.
    gap: f64,
    /// The distance of the lines of the grid.
    baseline: f64,
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
    // One file for all starts, written when it differs.
    let shader = runtime.join("chora-mc/write.frag");
    if std::fs::read_to_string(&shader).ok().as_deref() != Some(WRITE_SHADER) {
        std::fs::create_dir_all(shader.parent().expect("a file in a directory"))?;
        std::fs::write(&shader, WRITE_SHADER)?;
    }
    let rt = Runtime::connect(NAME, Vec::new())?;
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
    mc.start(methods)?;
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

/// The id of a part of the head of a scene.
fn chapter_id(scene: &str, part: &str) -> String {
    format!("{}-chapter-{part}", scene_id(scene))
}

/// A title from a scene name: its first letter in capitals.
fn title_of(name: &str) -> String {
    let mut chars = name.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// The date of today as `date` writes it in the locale, for example
/// "Friday, 30 October 2026"; empty when `date` fails.
// ponytail: the date of the last build; a scene shows a new day after
// the next build or a running head that a collector sets.
fn today() -> String {
    std::process::Command::new("date")
        .arg("+%A, %-d %B %Y")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// How far a note sits below the top of its line, so that its baseline
/// meets the baseline of the body: both have the same line, and the core
/// centers the glyphs in it.
// ponytail: tuned by eye for TeX Gyre Pagella in the body and Inter in
// the notes; read the ascent and descent of the fonts when this is off.
fn baseline_shift(body: &Type, note: &Type) -> f64 {
    (-0.2 * (body.size - note.size)).round()
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

/// Moves a node to `x`, `y` in its parent.
fn move_to(id: String, x: f64, y: f64) -> Command {
    Command::Update {
        id,
        props: PropsUpdate {
            transform: Some(translate(x, y)),
            ..PropsUpdate::default()
        },
    }
}

impl Mc {
    fn start(&mut self, methods: Vec<ExportMethod>) -> Result<(), ClientError> {
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
        // After the removal: it detaches the kept windows, and those
        // events would adopt them a second time.
        self.rt.announce(&EVENTS, methods)?;
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
        let background = self.settings.value.background();
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
                color: Some(background),
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

    /// Creates the head and the panels of a scene, under a group of their
    /// own that the next call replaces.
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
        commands.extend(self.chapter(name, scene));
        for panel in &scene.panels {
            commands.extend(self.panel(name, panel));
        }
        commands
    }

    /// A style of the settings, changed by a scene; a change that fails
    /// logs and leaves the style as it is.
    fn ty(&self, name: &str, local: Option<&Style>) -> Type {
        let config = &self.settings.value;
        config
            .resolve(name, local)
            .or_else(|err| {
                eprintln!("chora-mc: {err}");
                config.resolve(name, None)
            })
            .expect("the settings check their styles")
    }

    /// Whether a scene has a head: a scene of panels has one.
    fn has_chapter(scene: &Scene) -> bool {
        scene.app.is_none() && !scene.panels.is_empty()
    }

    /// The height of the head of a chapter.
    fn chapter_height(&self) -> f64 {
        match self.settings.value.chapter.layout {
            Layout::Number => self.ty("chapter_number", None).line + self.ty("chapter", None).line,
            Layout::Caps => self.ty("chapter", None).line,
            Layout::Large => self.ty("chapter_large", None).line,
        }
    }

    /// Where the parts of the page of a scene go on the output.
    fn geometry(&self, scene: &Scene) -> Option<Geometry> {
        let output = self.output.as_ref()?;
        let [w, h] = output.size.map(f64::from);
        let config = &self.settings.value;
        let (em, b) = (config.scale.base, config.scale.baseline);
        let (note, gap) = (config.page.margin * em, config.page.gap * em);
        let cols = f64::from(scene.grid[0]);
        let want = cols * config.page.measure * em + (cols - 1.0) * gap;
        // A margin column on both sides, so that the text stays centered.
        let width = want.min(w - 2.0 * (note + gap + 2.0 * b)).max(b);
        let x = ((w - width) / 2.0).round();
        let top = if Self::has_chapter(scene) {
            HEAD * b + self.chapter_height() + b
        } else {
            HEAD * b
        };
        let bottom = h - 4.0 * b;
        Some(Geometry {
            text: [x, top, width, (bottom - top).max(b)],
            note,
            gap,
            baseline: b,
        })
    }

    /// The head of a scene: its number, its title and the running head,
    /// as `chapter.layout` sets.
    fn chapter(&self, name: &str, scene: &Scene) -> Vec<Command> {
        let Some(geo) = self.geometry(scene).filter(|_| Self::has_chapter(scene)) else {
            return Vec::new();
        };
        let [x, _, _, _] = geo.text;
        let b = geo.baseline;
        let chapter = &scene.chapter;
        let number = chapter.number.clone().unwrap_or_else(|| {
            let numbered = self.scenes.iter().filter(|(_, s)| Self::has_chapter(s));
            let at = numbered.take_while(|(n, _)| *n != name).count();
            (at + 1).to_string()
        });
        let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
        let head = chapter.running_head.clone().unwrap_or_else(today);
        let node = |part: &str, ty: &Type, text: &str, y: f64| {
            let props = PropsUpdate {
                transform: Some(translate(x, y)),
                ..ty.props(text)
            };
            create(
                &chapter_id(name, part),
                NodeKind::Text,
                &panels_id(name),
                props,
            )
        };
        let top = HEAD * b;
        let running = self.ty("running_head", None);
        let mut commands = vec![node("running-head", &running, &head, RUNNING * b)];
        match self.settings.value.chapter.layout {
            Layout::Number => {
                let big = self.ty("chapter_number", None);
                commands.push(node("number", &big, &number, top));
                let caps = self.ty("chapter", None);
                commands.push(node("title", &caps, &title, top + big.line));
            }
            Layout::Caps => commands.push(node("title", &self.ty("chapter", None), &title, top)),
            Layout::Large => {
                let large = self.ty("chapter_large", None);
                commands.push(node("title", &large, &title, top));
            }
        }
        commands
    }

    /// The box of a panel on the screen, without its margin.
    fn place(&self, scene: &Scene, panel: &Panel) -> Option<[f64; 4]> {
        let geo = self.geometry(scene)?;
        Some(scene::place(
            panel.cell,
            scene.grid,
            geo.text,
            [geo.gap, geo.baseline],
        ))
    }

    /// The top of the lines of a panel, below its title.
    fn lines_top(&self, panel: &Panel) -> f64 {
        if panel.title.is_empty() {
            return 0.0;
        }
        let title = self.ty("heading", panel.title_style.as_ref()).line;
        title + self.settings.value.scale.baseline
    }

    /// Creates one panel: a group with a clip, its title and its lines,
    /// and the notes in the margin left of it.
    fn panel(&self, name: &str, panel: &Panel) -> Vec<Command> {
        let Some(scene) = self.scenes.get(name) else {
            return Vec::new();
        };
        let (Some([x, y, w, h]), Some(geo)) = (self.place(scene, panel), self.geometry(scene))
        else {
            return Vec::new();
        };
        let indent = geo.note + geo.gap;
        let id = panel_id(name, &panel.id);
        let mut commands = vec![create(
            &id,
            NodeKind::Group,
            &panels_id(name),
            PropsUpdate {
                transform: Some(translate(x - indent, y)),
                size: Some([w + indent, h]),
                clip: Some(Some(chora_scene::Clip { radius: 0.0 })),
                ..PropsUpdate::default()
            },
        )];
        let text = |ty: &Type, text: &str, x: f64, y: f64, wrap: bool| PropsUpdate {
            transform: Some(translate(x, y)),
            max_width: wrap.then_some(w),
            ..ty.props(text)
        };
        let add = |part: &str, props| create(&format!("{id}-{part}"), NodeKind::Text, &id, props);
        if !panel.title.is_empty() {
            let title = self.ty("heading", panel.title_style.as_ref());
            commands.push(add("title", text(&title, &panel.title, indent, 0.0, true)));
        }
        let top = self.lines_top(panel);
        let body = self.ty("body", panel.style.as_ref());
        if let Some(rows) = panel.columns() {
            // One node per row, so that a row wraps and its note stays on
            // its first line; `align` stacks the rows as the core lays
            // them out, and moves the notes flush right to the margin.
            let note = self.ty("margin", panel.note_style.as_ref());
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let shown = ((h - top) / body.line).ceil().max(0.0) as usize;
            for (i, (time, line)) in rows.iter().take(shown).enumerate() {
                #[allow(clippy::cast_precision_loss)] // rows of one screen
                let y = top + i as f64 * body.line;
                commands.push(add(
                    &format!("line-{i}"),
                    text(&body, line, indent, y, true),
                ));
                if !time.is_empty() {
                    let props = text(&note, time, 0.0, y + baseline_shift(&body, &note), false);
                    commands.push(add(&format!("note-{i}"), props));
                }
            }
        } else if !panel.lines.is_empty() {
            let props = text(&body, &panel.text(), indent, top, true);
            commands.push(add("lines", props));
        }
        commands
    }

    /// Moves what the core lays out to its place: the notes of the panels
    /// of a scene flush right to the margin, and the running head flush
    /// right to the text block.
    fn align(&mut self, name: &str) -> Result<(), ClientError> {
        let Some(scene) = self.scenes.get(name).cloned() else {
            return Ok(());
        };
        let Some(geo) = self.geometry(&scene) else {
            return Ok(());
        };
        let indent = geo.note + geo.gap;
        let mut commands = Vec::new();
        for panel in &scene.panels {
            let Some(rows) = panel.columns() else {
                continue;
            };
            let id = panel_id(name, &panel.id);
            let body = self.ty("body", panel.style.as_ref());
            let note = self.ty("margin", panel.note_style.as_ref());
            let mut y = self.lines_top(panel);
            for (i, (time, _)) in rows.iter().enumerate() {
                let Some([_, _, _, height]) = self.rect(&format!("{id}-line-{i}"))? else {
                    break; // below the panel
                };
                commands.push(move_to(format!("{id}-line-{i}"), indent, y));
                let note_id = format!("{id}-note-{i}");
                if !time.is_empty()
                    && let Some([_, _, width, _]) = self.rect(&note_id)?
                {
                    let x = (geo.note - width).max(0.0);
                    commands.push(move_to(note_id, x, y + baseline_shift(&body, &note)));
                }
                y += height.max(body.line);
            }
        }
        let id = chapter_id(name, "running-head");
        if Self::has_chapter(&scene)
            && let Some([_, _, width, _]) = self.rect(&id)?
        {
            let [x, _, w, _] = geo.text;
            let b = geo.baseline;
            commands.push(move_to(id, x + w - width, RUNNING * b));
        }
        self.apply(commands)
    }

    /// The rect that the core lays out a node at: x, y, width, height.
    fn rect(&mut self, id: &str) -> Result<Option<[f64; 4]>, ClientError> {
        let nodes: Vec<Value> =
            (self.rt.conn).call("scene.query", Query::new(format!("id:{id}")))?;
        let rect = nodes.first().and_then(|n| {
            let r = n["rects"].as_object()?.values().next()?.as_array()?;
            let v: Vec<f64> = r.iter().filter_map(Value::as_f64).collect();
            <[f64; 4]>::try_from(v).ok()
        });
        Ok(rect)
    }

    /// Writes the text of the panels of a scene anew: the effect starts at
    /// 0 and the core animates it to 1. Nothing when the setting is off.
    #[allow(clippy::cast_precision_loss)] // at most 4 KiB of lines
    fn write(&self, name: &str, panels: &[&Panel]) -> Vec<Command> {
        let config = &self.settings.value;
        let Some(scene) = self.scenes.get(name).filter(|_| config.write.enabled) else {
            return Vec::new();
        };
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
            let line = self.ty("body", panel.style.as_ref()).line;
            let rows = [
                self.lines_top(panel).max(line) / h,
                line / h,
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
        let text = self.hud_text();
        let b = self.settings.value.scale.baseline;
        let height = self.output.as_ref().map_or(0.0, |o| f64::from(o.size[1]));
        let scene = (self.current.as_ref()).and_then(|name| self.scenes.get(name));
        // Under the text block of the scene that shows.
        let x = (scene.and_then(|scene| self.geometry(scene))).map_or(2.0 * b, |geo| geo.text[0]);
        // Over an app it shows only while the agent is active.
        let page = scene.is_none_or(Self::has_chapter);
        PropsUpdate {
            visible: Some(!text.is_empty() && (page || !self.hud.state.is_empty())),
            transform: Some(translate(x, height - 3.0 * b)),
            ..self.ty("hud", None).props(&text)
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
        // The HUD follows the text block, and hides over an app.
        let hud = self.hud_update();
        self.apply(vec![hud])
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
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
        // The start adopts the free surfaces that events then announce too.
        if !self.built || self.windows.values().any(|s| s == surface) {
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
