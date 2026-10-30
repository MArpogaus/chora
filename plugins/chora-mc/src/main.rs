//! chora-mc, experimental: a voice-first desktop without window
//! management. The whole screen shows one scene, a grid of text panels or
//! one app, and quiet regions around it show the context: a header with
//! the running head, the state of the agent and the clock, a footer with
//! the words of the agent, notes in the margin and a drawer with a docked
//! terminal. The agent and small collector scripts fill the panels and
//! the regions through the exported methods; the plugin reads no sources
//! itself.

mod agent;
mod config;
mod region;
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
    config::{Action, Config, Layout, Page, Style, Type},
    region::{Kind, MAX_LINES, REGIONS, Region, Rule},
    scene::{Panel, Scene},
};

/// The client name, and the id of the group that holds the scenes.
const NAME: &str = "mc";
/// The id of the line of the HUD before the regions; a start removes it.
const HUD: &str = "mc-hud";
/// The id of a hidden node whose empty animations time the typewriter.
const TICK: &str = "mc-tick";
/// The id of a hidden node whose empty animations time the clocks.
const CLOCK: &str = "mc-clock";
/// The id of a hidden node whose empty animations time the look at the
/// apps that started without a window yet.
const WAIT: &str = "mc-wait";
/// How often the plugin looks at those apps, in milliseconds.
const WAIT_MS: f64 = 500.0;
/// The cross-fade between two scenes, in milliseconds.
const FADE: f64 = 250.0;
/// The effect that writes new text.
const WRITE_SHADER: &str = include_str!("../write.frag");
/// The effect that blurs a panel.
const BLUR_SHADER: &str = include_str!("../blur.frag");
/// The lines of the baseline grid above the head of a scene.
const HEAD: f64 = 3.0;

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

impl Geometry {
    /// The space between two columns of a grid: room for the margin of
    /// the column on the right, with a gap on both sides.
    fn columns(&self) -> f64 {
        self.note + 2.0 * self.gap
    }
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

/// Parameters of `region.toggle`; without `shown` it flips.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Toggle {
    region: Region,
    shown: Option<bool>,
}

/// Parameters of `region.put`; without a region the content follows the
/// rules of the settings again.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Put {
    content: String,
    region: Option<Region>,
    #[serde(default)]
    slot: i32,
}

/// Parameters of `content.set`; a missing title keeps the title.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetContent {
    content: String,
    title: Option<String>,
    lines: Vec<String>,
}

/// Parameters of `content.add`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AddLine {
    content: String,
    line: String,
}

/// Parameters of `region.solo`; without `shown` it flips.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SoloParams {
    content: String,
    region: Region,
    shown: Option<bool>,
}

/// A content alone in a region, and whether the region showed by a
/// toggle before it.
pub struct Solo {
    content: String,
    region: Region,
    before: Option<bool>,
}

/// What an app that starts goes to.
#[derive(Clone, Debug, PartialEq)]
enum Target {
    /// A scene of its own, by name.
    Scene(String),
    /// A region, as the content of this name.
    Dock(String),
}

struct Mc {
    rt: Runtime<Action>,
    settings: settings::File<Config>,
    /// The directory of the scene files.
    dir: PathBuf,
    scenes: BTreeMap<String, Scene>,
    /// The window of an app scene, by scene.
    windows: HashMap<String, String>,
    /// Where an app that starts goes, by its activation token.
    starting: HashMap<String, Target>,
    /// The app scenes that wait for their window, with the process of
    /// the app; `None` for one that ended before its pid was known.
    launched: Vec<(String, Option<u32>)>,
    current: Option<String>,
    /// The scene before the current one.
    previous: Option<String>,
    /// The one output that the plugin fills.
    output: Option<OutputInfo>,
    /// Whether the nodes exist.
    built: bool,
    hud: SetHud,
    typing: Option<Typing>,
    /// The files of the effects that write new text and that blur.
    shader: PathBuf,
    blur: PathBuf,
    /// The rules of `region.put`, before those of the settings.
    put: Vec<Rule>,
    /// The regions that a key or `region.toggle` showed or hid.
    toggled: BTreeMap<Region, bool>,
    /// The window of a docked app, by content.
    docks: HashMap<String, String>,
    /// The regions that showed at the last render.
    drawn: BTreeMap<Region, bool>,
    /// The scene, the regions and the items of the last render; `None`
    /// before the first or after a build.
    rendered: Option<Vec<String>>,
    /// The surface that the plugin gave the keyboard last.
    focused: Option<String>,
    /// The text of each clock.
    clocks: BTreeMap<String, String>,
    /// The title and the lines of each text content.
    texts: BTreeMap<String, (String, Vec<String>)>,
    /// The agents of `agent.set`, in the order of their first report.
    agents: Vec<agent::Agent>,
    /// The content of `region.solo` that shows alone in its region.
    solo: Option<Solo>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    runtime::run(NAME, || play(&args))
}

#[allow(clippy::too_many_lines)] // one line per field of the plugin
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
    let [shader, blur] = [("write", WRITE_SHADER), ("blur", BLUR_SHADER)].map(|(name, text)| {
        let path = runtime.join(format!("chora-mc/{name}.frag"));
        let write = || -> std::io::Result<()> {
            if chora_kit::read_text(&path, chora_kit::MAX_TEXT)
                .ok()
                .as_deref()
                != Some(text)
            {
                std::fs::create_dir_all(path.parent().expect("a file in a directory"))?;
                std::fs::write(&path, text)?;
            }
            Ok(())
        };
        write().map(|()| path)
    });
    let (shader, blur) = (shader?, blur?);
    let rt = Runtime::connect(NAME, Vec::new())?;
    let mut mc = Mc {
        rt,
        settings,
        scenes: scene::load(&dir),
        dir,
        windows: HashMap::new(),
        starting: HashMap::new(),
        launched: Vec::new(),
        current: None,
        previous: None,
        output: None,
        built: false,
        hud: SetHud {
            state: String::new(),
            text: String::new(),
        },
        typing: None,
        shader,
        blur,
        put: Vec::new(),
        toggled: BTreeMap::new(),
        docks: HashMap::new(),
        drawn: BTreeMap::new(),
        rendered: None,
        focused: None,
        clocks: BTreeMap::new(),
        texts: BTreeMap::new(),
        agents: Vec::new(),
        solo: None,
    };
    mc.keep_texts();
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

/// The methods; those that start programs or write files are privileged
/// (D62), because a caller with `methods.call` alone must not run commands.
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
        runtime::privileged(runtime::method(
            "scene.save",
            "Saves a scene in its file and shows the change: a grid of panels, or an app.",
            json!({
                "name": {"type": "string"},
                "grid": {"type": "array", "items": {"type": "integer"}, "description": "columns and rows"},
                "panels": panels,
                "app": {"type": "string", "description": "a shell command whose window fills the screen"},
            }),
            &["name"],
        )),
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
        runtime::privileged(runtime::method(
            "app.open",
            "Starts a shell command as an app in a full-screen scene of its own.",
            json!({"command": {"type": "string"}}),
            &["command"],
        )),
        runtime::method(
            "region.list",
            "Answers the regions around the page with what each shows, and the contents.",
            json!({}),
            &[],
        ),
        runtime::privileged(runtime::method(
            "region.toggle",
            "Shows or hides a region: header, footer, left, right or drawer; without shown \
             it flips. A docked app in it starts and takes the keyboard.",
            json!({"region": {"type": "string"}, "shown": {"type": "boolean"}}),
            &["region"],
        )),
        runtime::privileged(runtime::method(
            "region.put",
            "Puts a content into a region and slot over the rules, until the next start; \
             without a region the content follows the rules again.",
            json!({
                "content": {"type": "string"},
                "region": {"type": "string"},
                "slot": {"type": "integer"},
            }),
            &["content"],
        )),
        runtime::method(
            "content.set",
            "Sets the lines and the title of a text content, such as the notes.",
            json!({"content": {"type": "string"}, "title": {"type": "string"}, "lines": lines}),
            &["content", "lines"],
        ),
        runtime::method(
            "content.add",
            "Adds a line to a text content, such as a note; the oldest go after 12 lines.",
            json!({"content": {"type": "string"}, "line": {"type": "string"}}),
            &["content", "line"],
        ),
        runtime::privileged(runtime::method(
            "region.solo",
            "Shows a content alone in a region over any scene, such as the test overlay in \
             the drawer; without shown it flips. Answers shown.",
            json!({
                "content": {"type": "string"},
                "region": {"type": "string"},
                "shown": {"type": "boolean"},
            }),
            &["content", "region"],
        )),
        runtime::method(
            "agent.set",
            "Adds or changes an agent: its state (idle, listening, thinking, working, \
             speaking, question, blocked, done or asleep), its task, its last words and \
             its voice; a missing field stays.",
            json!({
                "name": {"type": "string"},
                "state": {"type": "string"},
                "task": {"type": "string"},
                "said": {"type": "string"},
                "voice": {"type": "string"},
            }),
            &["name"],
        ),
        runtime::method(
            "agent.remove",
            "Removes an agent.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::method("agent.list", "Answers the agents.", json!({}), &[]),
        runtime::method(
            "notify.push",
            "Shows a notification on top of the stack of notify; a critical one also in \
             red in urgent.",
            json!({
                "title": {"type": "string"},
                "body": {"type": "string"},
                "urgency": {"type": "string", "enum": ["low", "normal", "critical"]},
            }),
            &["title"],
        ),
        runtime::method("notify.clear", "Empties the notifications.", json!({}), &[]),
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
            Event::SurfaceDetached { surface } => self.reattach(&surface),
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
                // The next output that stays takes the scenes (W11).
                self.output = (self.rt.outputs()?.into_iter()).find(|o| o.name != name);
                if self.output.is_some() {
                    self.build()?;
                }
                Ok(())
            }
            Event::AnimationDone {
                node: Some(node), ..
            } if node == TICK => self.type_next(),
            Event::AnimationDone {
                node: Some(node), ..
            } if node == WAIT => self.look_at_launched(),
            Event::AnimationDone {
                node: Some(node), ..
            } if node == CLOCK => {
                let tick = self.tick_clocks();
                self.apply(tick)?;
                self.render()
            }
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
                let SetHud { state, text } = runtime::params(params)?;
                let text = hud_tail(&text).into();
                self.hud = SetHud { state, text };
                self.typing = None;
                self.render().map_err(rejected)?;
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
            _ => self.region_method(method, params),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.keep_texts();
        let contents = &self.settings.value.contents;
        (self.put).retain(|r| (r.content.as_ref()).is_some_and(|c| contents.contains_key(c)));
        if (self.solo.as_ref()).is_some_and(|s| !contents.contains_key(&s.content)) {
            self.solo = None;
        }
        let states = &self.settings.value.states;
        for agent in &mut self.agents {
            if !states.contains_key(&agent.state) {
                "idle".clone_into(&mut agent.state);
            }
        }
        self.rt
            .ungrab_all()
            .and_then(|()| self.grab_all())
            .and_then(|()| self.build())
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

impl Mc {
    /// The methods of the regions and their contents.
    fn region_method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        match method {
            "region.list" => Ok(self.list()),
            "region.toggle" => {
                let Toggle { region, shown } = runtime::params(params)?;
                let shown = self.toggle(region, shown).map_err(rejected)?;
                Ok(json!({"shown": shown}))
            }
            "region.put" => {
                let put: Put = runtime::params(params)?;
                self.put_content(put)?;
                self.render().map_err(rejected)?;
                Ok(Value::Null)
            }
            "content.set" => {
                let SetContent {
                    content,
                    title,
                    lines,
                } = runtime::params(params)?;
                self.set_text(&content, title, |_| lines)?;
                Ok(Value::Null)
            }
            "content.add" => {
                let AddLine { content, line } = runtime::params(params)?;
                self.set_text(&content, None, |mut lines| {
                    lines.push(line);
                    let extra = lines.len().saturating_sub(MAX_LINES);
                    lines.split_off(extra)
                })?;
                Ok(Value::Null)
            }
            "region.solo" => {
                let SoloParams {
                    content,
                    region,
                    shown,
                } = runtime::params(params)?;
                let found = (self.settings.value.contents.get(&content))
                    .ok_or_else(|| invalid(format!("no content {content}")))?;
                region::put_check(found, region).map_err(invalid)?;
                let shown = self.solo(&content, region, shown).map_err(rejected)?;
                Ok(json!({"shown": shown}))
            }
            _ => self.agent_method(method, params),
        }
    }

    /// The methods of the agents and the notifications.
    fn agent_method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        match method {
            "agent.set" => self.set_agent(runtime::params(params)?),
            "agent.remove" => self.remove_agent(&runtime::params(params)?),
            "agent.list" => return Ok(self.list_agents()),
            "notify.push" => self.push(runtime::params(params)?),
            "notify.clear" => self.clear_notifications(),
            _ => Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
        .map(|()| Value::Null)
    }
}

fn invalid(message: String) -> RpcError {
    RpcError::new(RpcError::INVALID_PARAMS, message)
}

#[allow(clippy::needless_pass_by_value)] // for map_err
fn rejected(err: ClientError) -> RpcError {
    RpcError::new(RpcError::REJECTED, err.to_string())
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
    *text = hud_tail(text).into();
}

/// The most characters of the HUD line.
const HUD_CHARS: usize = 160;

/// The end of a HUD text that fits one line, from a word on: the HUD
/// moves on as a ticker. 1200 words made a texture of 29953 pixels
/// that the GPU refused (W11).
fn hud_tail(text: &str) -> &str {
    let count = text.chars().count();
    if count <= HUD_CHARS {
        return text;
    }
    let (cut, _) = text
        .char_indices()
        .nth(count - HUD_CHARS)
        .unwrap_or_default();
    let rest = &text[cut..];
    rest.split_once(' ').map_or(rest, |(_, words)| words)
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
        self.autostart();
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

    /// Creates the group of a scene with its panels and its window.
    fn scene_commands(&self, name: &str) -> Vec<Command> {
        let shown = self.current.as_deref() == Some(name);
        let mut commands = vec![create(
            &scene_id(name),
            NodeKind::Group,
            NAME,
            PropsUpdate {
                visible: Some(shown),
                opacity: Some(if shown { 1.0 } else { 0.0 }),
                ..PropsUpdate::default()
            },
        )];
        commands.extend(self.panels(name));
        if let Some(surface) = self.windows.get(name) {
            commands.extend(self.place_window(name, surface, false));
        }
        commands
    }

    /// Applies the base of the plugin and its scenes in one transaction.
    /// One scene that the core refuses must not blank the others: then
    /// each goes on its own, and a refused one is left out with a log line
    /// until its file changes.
    fn apply_scenes(
        &mut self,
        base: Vec<Command>,
        scenes: Vec<(String, Vec<Command>)>,
    ) -> Result<(), ClientError> {
        let all = (base.iter())
            .chain(scenes.iter().flat_map(|(_, c)| c))
            .cloned()
            .collect();
        let Some(err) = self.rt.conn.try_apply(all)? else {
            return Ok(());
        };
        eprintln!("chora-mc: {err}; the scenes go one by one");
        self.apply(base)?;
        for (name, commands) in scenes {
            if let Some(err) = self.rt.conn.try_apply(commands)? {
                eprintln!("chora-mc: scene {name}: {err}; left out");
                self.scenes.remove(&name);
                self.windows.remove(&name);
            }
        }
        Ok(())
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
        commands.push(create(CLOCK, NodeKind::Group, NAME, PropsUpdate::default()));
        commands.push(create(WAIT, NodeKind::Group, NAME, PropsUpdate::default()));
        commands.extend(self.tick_clocks());
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
        let scenes = (self.scenes.keys())
            .map(|name| (name.clone(), self.scene_commands(name)))
            .collect();
        self.apply_scenes(commands, scenes)?;
        self.built = true;
        self.rendered = None;
        let names: Vec<String> = self.scenes.keys().cloned().collect();
        names.iter().try_for_each(|name| self.align(name))?;
        self.autostart();
        self.render()
    }

    /// Removes all nodes; the windows wait under the root.
    fn unbuild(&self) -> Vec<Command> {
        let surfaces = self.windows.values().chain(self.docks.values());
        let moves = surfaces.map(|surface| Command::Move {
            id: surface.clone(),
            parent: Some(ROOT.into()),
            index: None,
        });
        let removes = [NAME].map(|id| Command::Remove {
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
            commands.extend(self.rule(name, panel));
        }
        commands
    }

    /// The hairline of a panel, in the middle of the space before its
    /// margin; outside the panel, which `panel.set` replaces.
    fn rule(&self, name: &str, panel: &Panel) -> Option<Command> {
        let color = self.settings.value.colors().get(panel.rule.as_ref()?)?;
        let ([x, y, _, h], geo) = (self.place(name, panel)?, self.geometry(name)?);
        let line = [(x - geo.note - 1.5 * geo.gap).round(), y, 1.0, h];
        let id = format!("{}-rule", panel_id(name, &panel.id));
        Some(region::rect(&id, &panels_id(name), line, *color))
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

    /// The layout of the head of a scene: its own, else the settings.
    fn layout(&self, scene: &Scene) -> Layout {
        (scene.chapter.layout).unwrap_or(self.settings.value.chapter.layout)
    }

    /// The text block of a scene: its own, else the settings.
    pub fn page<'a>(&'a self, scene: &'a Scene) -> &'a Page {
        scene.page.as_ref().unwrap_or(&self.settings.value.page)
    }

    /// The height of the head of a chapter, with the line under it.
    fn chapter_height(&self, scene: &Scene) -> f64 {
        let b = self.settings.value.scale.baseline;
        match self.layout(scene) {
            Layout::Number => {
                self.ty("chapter_number", None).line + self.ty("chapter", None).line + b
            }
            Layout::Caps => self.ty("chapter", None).line + b,
            Layout::Large => self.ty("chapter_large", None).line + b,
            Layout::None => 0.0,
        }
    }

    /// Where the parts of the page of a scene go on the output.
    fn geometry(&self, name: &str) -> Option<Geometry> {
        self.geometry_of(name, self.scenes.get(name)?)
    }

    /// Where the parts of a page go on the output, with the sides of the
    /// scene `name` beside it.
    fn geometry_of(&self, name: &str, scene: &Scene) -> Option<Geometry> {
        let output = self.output.as_ref()?;
        let [w, h] = output.size.map(f64::from);
        let config = &self.settings.value;
        let (em, b) = (config.scale.base, config.scale.baseline);
        let page = self.page(scene);
        let (note, gap) = (page.margin * em, page.gap * em);
        let cols = f64::from(scene.grid[0]);
        let want = cols * page.measure * em + (cols - 1.0) * (note + 2.0 * gap);
        // A margin column on both sides, so that the text stays centered,
        // and room for the sides, so that the text never moves when one
        // shows: centered where it fits, else beside the sides.
        let [left, right] = self.side_widths(name, scene);
        let need = |side: f64| if side > 0.0 { side + gap } else { 0.0 };
        let (lneed, rneed) = (note + gap + need(left), (note + gap).max(need(right)));
        let width = want.min(w - lneed - rneed - 4.0 * b).max(b);
        let centered = ((w - width) / 2.0).round();
        let x = centered
            .min(w - 2.0 * b - rneed - width)
            .max(2.0 * b + lneed)
            .round();
        let top = if Self::has_chapter(scene) {
            HEAD * b + self.chapter_height(scene)
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
        let Some(geo) = self.geometry(name).filter(|_| Self::has_chapter(scene)) else {
            return Vec::new();
        };
        let [x, _, _, _] = geo.text;
        let b = geo.baseline;
        let chapter = &scene.chapter;
        let number = self.number(name, scene);
        let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
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
        let mut commands = Vec::new();
        match self.layout(scene) {
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
            Layout::None => {}
        }
        commands
    }

    /// The number of a scene: its own, or its place among the scenes of
    /// panels.
    fn number(&self, name: &str, scene: &Scene) -> String {
        scene.chapter.number.clone().unwrap_or_else(|| {
            let numbered = self.scenes.iter().filter(|(_, s)| Self::has_chapter(s));
            let at = numbered.take_while(|(n, _)| *n != name).count();
            (at + 1).to_string()
        })
    }

    /// The box of a panel on the screen, without its margin.
    fn place(&self, name: &str, panel: &Panel) -> Option<[f64; 4]> {
        let scene = self.scenes.get(name)?;
        let geo = self.geometry(name)?;
        Some(scene::place(
            panel.cell,
            scene.grid,
            geo.text,
            [geo.columns(), geo.baseline],
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
        let (Some([x, y, w, h]), Some(geo)) = (self.place(name, panel), self.geometry(name)) else {
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
                opacity: panel.opacity,
                effects: self.blur_effect(panel).map(|e| vec![e]),
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
    /// of a scene flush right to the margin.
    fn align(&mut self, name: &str) -> Result<(), ClientError> {
        let Some(scene) = self.scenes.get(name).cloned() else {
            return Ok(());
        };
        let Some(geo) = self.geometry(name) else {
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

    /// The effect of the blur of a panel, if it has one.
    fn blur_effect(&self, panel: &Panel) -> Option<ShaderEffect> {
        let radius = panel.blur.filter(|r| *r > 0.0)?;
        Some(ShaderEffect {
            shader: self.blur.to_string_lossy().into_owned(),
            uniforms: [("radius".into(), Uniform::Float(radius))].into(),
        })
    }

    /// Writes the text of the panels of a scene anew: the effect starts at
    /// 0 and the core animates it to 1. Nothing when the setting is off.
    #[allow(clippy::cast_precision_loss)] // at most 4 KiB of lines
    fn write(&self, name: &str, panels: &[&Panel]) -> Vec<Command> {
        let config = &self.settings.value;
        if !config.write.enabled {
            return Vec::new();
        }
        let mut commands = Vec::new();
        for panel in panels {
            let effect = |rows: [f64; 3], progress: f64| {
                let write = ShaderEffect {
                    shader: self.shader.to_string_lossy().into_owned(),
                    uniforms: [
                        ("progress".into(), Uniform::Float(progress)),
                        ("rows".into(), Uniform::Vector(rows.to_vec())),
                    ]
                    .into(),
                };
                Some(
                    std::iter::once(write)
                        .chain(self.blur_effect(panel))
                        .collect(),
                )
            };
            let Some([_, _, _, h]) = self.place(name, panel) else {
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
        self.focused.clone_from(&window);
        commands.push(Command::Focus {
            surface: window.clone(),
        });
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        if self.current.as_deref() != Some(name) {
            self.previous = self.current.replace(name.into());
        }
        // A hidden scene has no layout of the core: its notes align now.
        self.align(name)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        // The regions follow the scene.
        self.render()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        let waiting = (self.starting.values()).any(|t| *t == Target::Scene(name.into()));
        if let (Some(command), None, false) = (app, window, waiting) {
            self.launch_scene(name, &command)
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        Ok(())
    }

    /// Starts the app of a scene, whose window then fills it.
    fn launch_scene(&mut self, name: &str, command: &str) -> Result<(), ClientError> {
        let Some((token, pid)) = launch(command) else {
            return Ok(());
        };
        self.starting.insert(token, Target::Scene(name.into()));
        if self.launched.is_empty() {
            self.apply(vec![wait()])?;
        }
        self.launched.push((name.into(), pid));
        Ok(())
    }

    /// An app of a scene that ended without a window leaves its scene
    /// empty: the scene before it shows again (W11).
    fn look_at_launched(&mut self) -> Result<(), ClientError> {
        let windows = &self.windows;
        self.launched
            .retain(|(name, _)| !windows.contains_key(name));
        let (ended, running) = std::mem::take(&mut self.launched)
            .into_iter()
            .partition(|(_, pid)| !pid.is_some_and(alive));
        self.launched = running;
        let ended: Vec<(String, Option<u32>)> = ended;
        for (name, _) in ended {
            // The next show of the scene starts the app again.
            let target = Target::Scene(name.clone());
            self.starting.retain(|_, t| *t != target);
            if self.current.as_deref() != Some(&name) || self.windows.contains_key(&name) {
                continue;
            }
            let back = (self.previous.clone())
                .filter(|p| *p != name && self.scenes.contains_key(p))
                .unwrap_or_else(|| self.settings.value.default_scene.clone());
            if let Err(err) = self.show(&back) {
                eprintln!("chora-mc: {back}: {}", err.message);
            }
        }
        if self.launched.is_empty() {
            return Ok(());
        }
        self.apply(vec![wait()])
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
        self.apply(vec![group, regions_on_top()])
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
        let mut known = self.windows.values().chain(self.docks.values());
        if !self.built || known.any(|s| s == surface) {
            return Ok(());
        }
        let started = token.and_then(|t| self.starting.remove(t));
        if let Some(Target::Dock(content)) = &started
            && !self.docks.contains_key(content)
        {
            // Into its region at the render, with the keyboard.
            self.apply(vec![Command::Attach {
                surface: surface.into(),
                parent: REGIONS.into(),
            }])?;
            // A new dock changes the items, so the render draws anew.
            self.docks.insert(content.clone(), surface.into());
            self.render()?;
            return self.focus(Some(surface.into()));
        }
        let mut commands = Vec::new();
        let name = match started {
            Some(Target::Scene(name)) if !self.windows.contains_key(&name) => name,
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
                commands.push(regions_on_top());
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

    /// A surface lost its parent and its owner. One of the plugin, for
    /// example after a hotplug, goes back to its scene or region: the
    /// plugin still counts it as its own, and a later move of it failed
    /// with "belongs to nobody" (W11).
    fn reattach(&mut self, surface: &str) -> Result<(), ClientError> {
        let scene = (self.windows.iter()).find(|(_, s)| *s == surface);
        if let Some(name) = scene.map(|(n, _)| n.clone()) {
            let commands = self.place_window(&name, surface, true);
            return self.apply(commands);
        }
        if self.docks.values().any(|s| s == surface) {
            return self.apply(vec![Command::Attach {
                surface: surface.into(),
                parent: REGIONS.into(),
            }]);
        }
        self.adopt(surface, None, None)
    }

    /// A window closed: a scene that only held it goes, and the default
    /// scene shows instead of a scene without its window.
    fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        let dock = (self.docks.iter()).find(|(_, s)| *s == surface);
        if let Some(content) = dock.map(|(c, _)| c.clone()) {
            // A docked app ended: its region hides, the keyboard goes back.
            self.docks.remove(&content);
            let name = self.current.clone().unwrap_or_default();
            if let Some(region) = (self.placed(&name).into_iter())
                .find(|(_, slots)| slots.iter().any(|(_, c)| *c == content))
                .map(|(r, _)| r)
            {
                self.toggled.insert(region, false);
            }
            if self.focused.as_deref() == Some(surface) {
                let window = self.windows.get(&name).cloned();
                self.focus(window)?;
            }
            return self.render();
        }
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

    /// Keeps the lines of the text contents that stay, and takes those of
    /// the settings for new ones.
    fn keep_texts(&mut self) {
        let contents = &self.settings.value.contents;
        let old = std::mem::take(&mut self.texts);
        self.texts = (contents.iter())
            .filter(|(_, c)| c.kind == Kind::Text)
            .map(|(name, c)| {
                let text = old
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| (c.title.clone(), c.lines.clone()));
                (name.clone(), text)
            })
            .collect();
    }

    /// Changes the lines of a text content, and its title.
    fn set_text(
        &mut self,
        content: &str,
        title: Option<String>,
        change: impl FnOnce(Vec<String>) -> Vec<String>,
    ) -> Result<(), RpcError> {
        let (old_title, lines) = (self.texts.get(content).cloned())
            .ok_or_else(|| invalid(format!("no text content {content}")))?;
        let new = (title.unwrap_or(old_title), change(lines));
        if new.0.len() + new.1.join("\n").len() > scene::MAX_TEXT {
            return Err(invalid(format!("{content}: more than 4 KiB of text")));
        }
        self.texts.insert(content.into(), new);
        self.render()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }

    /// Puts a content into a region over the rules, or back to them.
    fn put_content(&mut self, put: Put) -> Result<(), RpcError> {
        let Put {
            content,
            region,
            slot,
        } = put;
        let found = (self.settings.value.contents.get(&content))
            .ok_or_else(|| invalid(format!("no content {content}")))?;
        if let Some(region) = region {
            region::put_check(found, region).map_err(invalid)?;
        }
        self.put.retain(|r| r.content.as_ref() != Some(&content));
        if let Some(region) = region {
            let rule = Rule {
                content: Some(content),
                kind: None,
                scenes: Vec::new(),
                region,
                slot,
            };
            self.put.insert(0, rule);
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
        let found = (self.scenes.get(&scene))
            .and_then(|v| v.panels.iter().find(|p| p.id == panel))
            .ok_or_else(|| invalid(format!("no panel {panel} in scene {scene}")))?;
        let new = Panel {
            title: title.unwrap_or_else(|| found.title.clone()),
            lines,
            ..found.clone()
        };
        self.replace_panel(&scene, &new)
    }

    /// Puts a changed panel into its scene, by its id; it writes itself
    /// when the scene shows.
    fn replace_panel(&mut self, scene: &str, new: &Panel) -> Result<(), RpcError> {
        new.check_text().map_err(invalid)?;
        let found = (self.scenes.get_mut(scene))
            .and_then(|v| v.panels.iter_mut().find(|p| p.id == new.id))
            .ok_or_else(|| invalid(format!("no panel {} in scene {scene}", new.id)))?;
        // The same text again neither writes nor builds anything.
        if found == new {
            return Ok(());
        }
        *found = new.clone();
        let scene = scene.to_owned();
        let mut commands = vec![Command::Remove {
            id: panel_id(&scene, &new.id),
            to: None,
        }];
        commands.extend(self.panel(&scene, new));
        if self.current.as_deref() == Some(&scene) {
            commands.extend(self.write(&scene, &[new]));
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
        self.apply(vec![tick])?;
        self.render()
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
            Action::Toggle(region) => {
                if let Err(err) = self.toggle(region, None) {
                    eprintln!("chora-mc: toggle {}: {err}", region.name());
                }
                return;
            }
            Action::Solo { content, region } => {
                if let Err(err) = self.solo(&content, region, None) {
                    eprintln!("chora-mc: solo {content}: {err}");
                }
                return;
            }
        };
        if let Err(err) = self.show(&next) {
            eprintln!("chora-mc: {next}: {}", err.message);
        }
    }
}

/// Keeps the regions over a scene group that was just created.
fn regions_on_top() -> Command {
    Command::Move {
        id: REGIONS.into(),
        parent: None,
        index: None,
    }
}

/// The empty animation after which the plugin looks at the apps that wait
/// for their window.
fn wait() -> Command {
    Command::Animate {
        id: WAIT.into(),
        props: PropsUpdate::default(),
        duration: WAIT_MS,
        easing: Easing::Linear,
    }
}

/// Whether a process runs, not ended and not a zombie.
fn alive(pid: u32) -> bool {
    let stat = chora_kit::read_text(std::path::Path::new(&format!("/proc/{pid}/stat")), 4096);
    // The state follows the name in parentheses, which may hold any byte.
    stat.ok()
        .and_then(|s| Some(s[s.rfind(')')? + 1..].trim_start().starts_with('Z')))
        .is_some_and(|zombie| !zombie)
}

/// Starts a shell command with `choractl run`, and answers the activation
/// token that finds its window again, and the process, if it is known.
fn launch(command: &str) -> Option<(String, Option<u32>)> {
    let run = std::process::Command::new("choractl")
        .args(["run", "--", "sh", "-c", command])
        .stdin(std::process::Stdio::null())
        .output();
    let line = match run {
        Ok(out) if out.status.success() => out.stdout,
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            eprintln!("chora-mc: {command}: {}", err.trim());
            return None;
        }
        Err(err) => {
            eprintln!("chora-mc: choractl run: {err}");
            return None;
        }
    };
    let answer: Value = serde_json::from_slice(&line).unwrap_or_default();
    let pid = answer["pid"].as_u64().and_then(|p| u32::try_from(p).ok());
    Some((answer["token"].as_str()?.into(), pid))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_hud_keeps_the_end_of_a_long_text() {
        let mut text = String::new();
        for i in 0..1200 {
            super::push_word(&mut text, &format!("wordy{i}"));
        }
        assert!(text.chars().count() <= super::HUD_CHARS, "{}", text.len());
        assert!(text.ends_with("wordy1199") && text.starts_with("wordy"));
        assert_eq!(super::hud_tail("short"), "short");
    }

    #[test]
    fn a_running_process_is_alive_and_a_missing_one_not() {
        assert!(super::alive(std::process::id()));
        assert!(!super::alive(u32::MAX));
    }

    /// A client with `methods.call` alone ran commands through app.open and
    /// scene.save (W12).
    #[test]
    fn the_methods_that_start_programs_are_privileged() {
        let privileged: Vec<String> = super::exported_methods()
            .iter()
            .filter(|m| m.privileged())
            .map(|m| m.name().to_owned())
            .collect();
        assert_eq!(
            privileged,
            [
                "scene.save",
                "app.open",
                "region.toggle",
                "region.put",
                "region.solo"
            ]
        );
    }
}
