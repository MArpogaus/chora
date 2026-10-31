//! chora-mc, experimental: a voice-first desktop without window
//! management. The whole screen shows one scene, a grid of text panels or
//! one app, and quiet regions around it show the context: a header with
//! the running head, the state of the agent and the clock, a footer with
//! the words of the agent, notes in the margin and a drawer with a docked
//! terminal. The agent and small collector scripts fill the panels and
//! the regions through the exported methods; the plugin reads no sources
//! itself.

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod agent;
mod config;
mod manage;
mod region;
mod scene;
#[cfg(test)]
mod testing;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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
    region::{FIT_MS, Kind, MAX_LINES, PARKED, REGIONS, Region, Rule},
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
/// The cross-fade of the colors when the theme changes, in milliseconds.
const THEME_FADE: f64 = 600.0;
/// The effect that writes new text.
const WRITE_SHADER: &str = include_str!("../write.frag");
/// The effect that blurs a panel.
const BLUR_SHADER: &str = include_str!("../blur.frag");
/// The lines of the baseline grid above the head of a scene.
const HEAD: f64 = 3.0;
/// The space on each side of the bar of a head, in em of its title.
const BAR_SPACE: f64 = 0.4;

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
    /// The names of the scenes in the order they appeared; `order` of
    /// the settings goes before it.
    appeared: Vec<String>,
    /// The window of an app scene, by scene.
    windows: HashMap<String, String>,
    /// The scenes that go when their app ends: those of `app.open` and of
    /// a window that no scene started.
    transient: BTreeSet<String>,
    /// The process of the app of a scene, if it is known.
    pids: HashMap<String, Option<u32>>,
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
    /// A docked app that waits parked for its first size that fits: its
    /// content and that size, by surface.
    fitting: HashMap<String, (String, [i32; 2])>,
    /// The scene, the regions and the items of the last render; `None`
    /// before the first or after a build. An empty key draws anew.
    rendered: Option<Vec<String>>,
    /// The set of the regions of the last render, 0 or 1.
    set: u8,
    /// How long the regions cross-fade, in milliseconds.
    fade: f64,
    /// The docked apps that faded out with the set before; `None` while
    /// that set does not exist.
    left: Option<Vec<String>>,
    /// What each region of the last render shows.
    sigs: BTreeMap<Region, String>,
    /// The settings of the last build without the theme: a change of the
    /// theme alone fades the colors.
    shape: Option<Value>,
    /// The surface that the plugin gave the keyboard last.
    focused: Option<String>,
    /// The activation tokens of the apps of `app.open`: voice and agents
    /// start them, and their first window does not take the keyboard.
    quiet: BTreeSet<String>,
    /// The docked app that the user started with a toggle: it takes the
    /// keyboard when its window fits.
    dock_focus: Option<String>,
    /// The text of each clock.
    clocks: BTreeMap<String, String>,
    /// The title and the lines of each text content.
    texts: BTreeMap<String, (String, Vec<String>)>,
    /// The agents of `agent.set`, in the order of their first report.
    agents: Vec<agent::Agent>,
    /// The agent that the agents scene shows alone, with its transcript.
    agent_focus: Option<String>,
    /// The content of `region.solo` that shows alone in its region.
    solo: Option<Solo>,
    /// The id of the spoken confirmation that shows, until its answer.
    confirming: Option<String>,
    /// Whether mc claims the state indicator of the voice agent at its
    /// next event: after the start and after the agent connected.
    claim_due: bool,
    /// How far the page of a scene scrolled down, by scene.
    scroll: BTreeMap<String, f64>,
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
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                std::fs::write(&path, text)?;
            }
            Ok(())
        };
        write().map(|()| path)
    });
    let (shader, blur) = (shader?, blur?);
    let rt = Runtime::connect(NAME, Vec::new())?;
    let mut mc = Mc::new(rt, settings, dir, [shader, blur]);
    mc.start(methods)?;
    // The kit logged it; the defaults that apply show notify.
    if let Some(err) = mc.settings.broken() {
        let push = json!({"title": "Settings", "body": err, "urgency": "critical"});
        let shown = serde_json::from_value(push)
            .map_err(|e| e.to_string())
            .and_then(|push| mc.push(push).map_err(|e| e.message));
        if let Err(err) = shown {
            eprintln!("chora-mc: notify: {err}");
        }
    }
    Ok(runtime::serve(&mut mc)?)
}

impl Mc {
    /// The plugin before its start: no nodes, no scene shown. `effects`
    /// are the files of the shaders that write and that blur.
    #[allow(clippy::too_many_lines)] // one line per field of the plugin
    fn new(
        rt: Runtime<Action>,
        settings: settings::File<Config>,
        dir: PathBuf,
        [shader, blur]: [PathBuf; 2],
    ) -> Self {
        let scenes = scene::load(&dir);
        let mut mc = Self {
            rt,
            appeared: scenes.keys().cloned().collect(),
            scenes,
            settings,
            dir,
            windows: HashMap::new(),
            transient: BTreeSet::new(),
            pids: HashMap::new(),
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
            set: 0,
            fade: FADE,
            left: None,
            sigs: BTreeMap::new(),
            fitting: HashMap::new(),
            shape: None,
            rendered: None,
            focused: None,
            quiet: BTreeSet::new(),
            dock_focus: None,
            clocks: BTreeMap::new(),
            texts: BTreeMap::new(),
            agents: Vec::new(),
            agent_focus: None,
            solo: None,
            confirming: None,
            claim_due: true,
            scroll: BTreeMap::new(),
        };
        mc.keep_texts();
        mc
    }
}

/// The client name of the voice agent.
const VOICE: &str = "voice";

/// The events; `voice.*` are those of the voice agent (D65).
const EVENTS: [&str; 12] = [
    "voice.*",
    "client.connected",
    "client.disconnected",
    "pointer.scroll",
    "pointer.pressed",
    "animation.done",
    "surface.created",
    "surface.destroyed",
    "surface.changed",
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
                "grid": {"type": "array", "items": {"type": "integer"}, "description": "columns and rows, by default [4, 3]"},
                "panels": panels,
                "app": {"type": "string", "description": "a shell command whose window fills the text block, or the name of a content of kind app, such as shell"},
            }),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "scene.rename",
            "Renames a scene: its file, its entry in the order and the scene.",
            json!({"name": {"type": "string"}, "to": {"type": "string"}}),
            &["name", "to"],
        )),
        runtime::privileged(runtime::method(
            "scene.move",
            "Moves a scene to the number to, from 1, and saves the order; the scenes \
             between move by one.",
            json!({"name": {"type": "string"}, "to": {"type": "integer", "minimum": 1}}),
            &["name", "to"],
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
             speaking, question, blocked, done or asleep), its task, its last words, its \
             voice, its plan, its open question and the last lines of its transcript; a \
             missing field stays.",
            json!({
                "name": {"type": "string"},
                "state": {"type": "string"},
                "task": {"type": "string"},
                "said": {"type": "string"},
                "voice": {"type": "string"},
                "plan": lines.clone(),
                "question": {"type": "string"},
                "log": lines.clone(),
            }),
            &["name"],
        ),
        runtime::method(
            "agent.focus",
            "Shows the agents scene with one agent alone and its transcript; an empty \
             name shows all agents again.",
            json!({"name": {"type": "string"}}),
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
            Event::SurfaceChanged {
                surface,
                size: Some(size),
                ..
            } if self.fitting.get(&surface).is_some_and(|(_, s)| *s == size) => self.fit(&surface),
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
            } => self.timer(&node),
            Event::ClientConnected { name } | Event::ClientDisconnected { name }
                if name == VOICE =>
            {
                self.claim_due = true;
                self.claim_indicator(false);
                Ok(())
            }
            Event::InputKey {
                binding,
                state: PressState::Pressed,
            } => {
                self.act(&binding);
                Ok(())
            }
            Event::Emitted {
                event,
                client,
                data,
            } => self.follow(&event, &client, data),
            Event::PointerScroll { node, dy, .. } => self.scroll_page(&node, dy),
            Event::PointerPressed { node, position, .. } => self.click(&node, position),
            _ => Ok(()),
        }
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        match method {
            "scene.show" => {
                let Show { name } = runtime::params(params)?;
                self.show(&name, false)?;
                Ok(Value::Null)
            }
            "scene.list" => Ok(json!({
                "scenes": self.order(),
                "current": self.current,
            })),
            "scene.save" => {
                let Save { name, scene } = runtime::params(params)?;
                scene::save(&self.dir, &name, &scene).map_err(invalid)?;
                self.add_scene(&name, scene);
                self.build().map_err(rejected)?;
                Ok(Value::Null)
            }
            "scene.rename" => {
                self.rename_scene(runtime::params(params)?)?;
                Ok(Value::Null)
            }
            "scene.move" => {
                self.move_scene(runtime::params(params)?)?;
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
                self.show(&name, false)?;
                let target = Target::Scene(name.clone());
                let tokens = (self.starting.iter()).filter(|(_, t)| **t == target);
                self.quiet.extend(tokens.map(|(token, _)| token.clone()));
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
        let theme_only = self.built && self.shape.as_ref() == Some(&self.shape_of());
        self.rt
            .ungrab_all()
            .and_then(|()| self.grab_all())
            .and_then(|()| {
                if theme_only && self.recolor()? {
                    return Ok(());
                }
                self.build()
            })
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
                let shown = self.toggle(region, shown, false).map_err(rejected)?;
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
                let shown = (self.solo(&content, region, shown, false)).map_err(rejected)?;
                Ok(json!({"shown": shown}))
            }
            _ => self.agent_method(method, params),
        }
    }

    /// An event of a client (D65): `NAME.state` sets the agent `NAME` as
    /// `agent.set` does, or the sub-agent that `name` in the data names;
    /// `NAME.remove` removes such a sub-agent; `NAME.say` shows a sentence
    /// as `hud.say` does. Wrong data only logs, since the emitter gets no
    /// answer; other events do nothing.
    fn follow(&mut self, event: &str, client: &str, mut data: Value) -> Result<(), ClientError> {
        if client == VOICE {
            self.claim_indicator(true);
        }
        let kind = event.strip_prefix(client).and_then(|e| e.strip_prefix('.'));
        let done = match kind {
            Some("state") => {
                if let Some(object) = data.as_object_mut() {
                    object.entry("name").or_insert_with(|| client.into());
                }
                runtime::params(data).and_then(|set| self.set_agent(set))
            }
            Some("remove") => runtime::params(data).and_then(|r| self.remove_agent(&r)),
            Some("confirm") => runtime::params(data).and_then(|c| self.confirm(c)),
            Some("say") => match runtime::params(data) {
                Ok(say) => return self.say(say),
                Err(e) => Err(e),
            },
            _ => Ok(()),
        };
        if let Err(e) = done {
            eprintln!("chora-mc: {event}: {e}");
        }
        Ok(())
    }

    /// Asks the voice agent to hide its own state indicator, since mc
    /// shows the state, while one is due. An error, such as no agent or
    /// an agent without the method, only leaves it due; after an event
    /// of the agent (`last`) it is no longer due.
    fn claim_indicator(&mut self, last: bool) {
        if !self.claim_due {
            return;
        }
        let call = json!({"client": VOICE, "method": "indicator.claim", "params": {}});
        let done = self.rt.conn.call::<_, Value>("methods.call", call).is_ok();
        self.claim_due = !done && !last;
    }

    /// The methods of the agents and the notifications.
    fn agent_method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        match method {
            "agent.set" => self.set_agent(runtime::params(params)?),
            "agent.remove" => self.remove_agent(&runtime::params(params)?),
            "agent.list" => return Ok(self.list_agents()),
            "agent.focus" => self.focus_agent(&runtime::params(params)?),
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

/// The id of the page of a scene: its head and its panels, clipped.
fn page_id(name: &str) -> String {
    format!("{}-page", scene_id(name))
}

/// The id of the rect under the page of a scene that takes the wheel.
fn wheel_id(name: &str) -> String {
    format!("mc-s-{name}-wheel")
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

/// The creates of the same nodes as updates, so that they change in
/// place; `None` when a node comes or goes, or a prop would stay that the
/// new props leave out.
fn in_place(old: &[Command], new: &[Command]) -> Option<Vec<Command>> {
    let keys = |props: &PropsUpdate| {
        let value = serde_json::to_value(props).unwrap_or_default();
        (value
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys().cloned()))
        .collect::<Vec<_>>()
    };
    if old.len() != new.len() {
        return None;
    }
    let pairs = old.iter().zip(new);
    pairs
        .map(|pair| match pair {
            (
                Command::Create {
                    id: a,
                    kind: ka,
                    props: pa,
                    ..
                },
                Command::Create {
                    id, kind, props, ..
                },
            ) if a == id && ka == kind && keys(pa).iter().all(|k| keys(props).contains(k)) => {
                Some(Command::Update {
                    id: id.clone(),
                    props: props.clone(),
                })
            }
            _ => None,
        })
        .collect()
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
    /// The end of an empty animation of a hidden node that times
    /// something.
    fn timer(&mut self, node: &str) -> Result<(), ClientError> {
        match node {
            TICK => self.type_next(),
            WAIT => self.look_at_launched(),
            CLOCK => {
                let tick = self.tick_clocks();
                self.apply(tick)?;
                self.render()
            }
            PARKED => {
                let late: Vec<String> = self.fitting.keys().cloned().collect();
                late.iter().try_for_each(|surface| self.fit(surface))
            }
            _ => Ok(()),
        }
    }

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
        let first = self.order().into_iter().next();
        let default = Some(self.settings.value.default_scene.clone())
            .filter(|v| self.scenes.contains_key(v))
            .or(first);
        if let Some(name) = default
            && let Err(err) = self.show(&name, false)
        {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        self.autostart();
        self.claim_indicator(false);
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
    #[allow(clippy::expect_used, reason = "the settings check their bindings")]
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
        // A build ends the wait of the old node.
        commands.extend((!self.launched.is_empty()).then(wait));
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
        commands.extend(self.parked());
        if let Some(panel) = self.overview_panel()
            && let Some(scene) = self.scenes.get_mut(manage::OVERVIEW)
        {
            scene.panels.retain(|p| p.id != panel.id);
            scene.panels.push(panel);
        }
        let scenes = (self.scenes.keys())
            .map(|name| (name.clone(), self.scene_commands(name)))
            .collect();
        self.apply_scenes(commands, scenes)?;
        self.apply(vec![regions_on_top()])?;
        self.built = true;
        self.rendered = None;
        self.left = None;
        self.sigs.clear();
        self.shape = Some(self.shape_of());
        let names: Vec<String> = self.scenes.keys().cloned().collect();
        names.iter().try_for_each(|name| self.align(name))?;
        self.autostart();
        self.render()
    }

    /// The group of the regions, and the docked apps hidden in it, not on
    /// the root, until the render.
    fn parked(&self) -> Vec<Command> {
        let hidden = PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        };
        let mut commands = vec![
            create(REGIONS, NodeKind::Group, NAME, PropsUpdate::default()),
            create(PARKED, NodeKind::Group, REGIONS, hidden),
        ];
        let docks = self.docks.values().chain(self.fitting.keys());
        commands.extend(docks.map(|surface| Command::Move {
            id: surface.clone(),
            parent: Some(PARKED.into()),
            index: None,
        }));
        commands
    }

    /// The settings without the theme.
    fn shape_of(&self) -> Value {
        let mut shape = serde_json::to_value(&self.settings.value).unwrap_or_default();
        if let Some(map) = shape.as_object_mut() {
            map.remove("theme");
        }
        shape
    }

    /// After a change of the theme alone: the nodes fade to their new
    /// colors and the regions cross-fade, instead of a build. False when
    /// the core refuses that, for example when a color of a rule is
    /// missing in one theme: then a build follows.
    fn recolor(&mut self) -> Result<bool, ClientError> {
        let background = PropsUpdate {
            color: Some(self.settings.value.background()),
            ..PropsUpdate::default()
        };
        let mut created = vec![create("mc-bg", NodeKind::Rect, NAME, background)];
        created.extend(
            self.scenes
                .keys()
                .flat_map(|name| self.scene_commands(name)),
        );
        let mut fades: Vec<Command> = (created.into_iter())
            .filter_map(|command| match command {
                Command::Create {
                    id,
                    props:
                        PropsUpdate {
                            color: Some(color), ..
                        },
                    ..
                } => Some(Command::Animate {
                    id,
                    props: PropsUpdate {
                        color: Some(color),
                        ..PropsUpdate::default()
                    },
                    duration: THEME_FADE,
                    easing: Easing::EaseInOut,
                }),
                _ => None,
            })
            .collect();
        // The regions cross-fade in the same transaction, as slowly.
        self.rendered = Some(Vec::new());
        self.fade = THEME_FADE;
        let regions = self.regions();
        self.fade = FADE;
        fades.extend(regions?);
        if let Some(err) = self.rt.conn.try_apply(fades)? {
            eprintln!("chora-mc: {err}; the theme builds anew");
            return Ok(false);
        }
        Ok(true)
    }

    /// Removes all nodes; the windows wait under the root.
    fn unbuild(&self) -> Vec<Command> {
        let surfaces = (self.windows.values())
            .chain(self.docks.values())
            .chain(self.fitting.keys());
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
        // A page between the header and the footer that clips the panels
        // and takes the wheel; the panels move up in it as it scrolls.
        let [w, h] = (self.output.as_ref()).map_or([0.0; 2], |o| o.size.map(f64::from));
        let b = self.settings.value.scale.baseline;
        let page = page_id(name);
        let scroll = self.scroll.get(name).copied().unwrap_or_default();
        let mut commands = vec![
            create(
                &page,
                NodeKind::Group,
                &scene_id(name),
                PropsUpdate {
                    transform: Some(translate(0.0, 2.0 * b)),
                    size: Some([w, (h - 4.0 * b).max(b)]),
                    clip: Some(Some(chora_scene::Clip { radius: 0.0 })),
                    // Other plugins find the page by this tag and show it
                    // through its node texture, as chora-construct does.
                    meta: Some(BTreeMap::from([("pane".into(), name.into())])),
                    ..PropsUpdate::default()
                },
            ),
            region::rect(&wheel_id(name), &page, [0.0, 0.0, w, h], [0.0; 4]),
            create(
                &panels_id(name),
                NodeKind::Group,
                &page,
                PropsUpdate {
                    transform: Some(translate(0.0, -2.0 * b - scroll)),
                    ..PropsUpdate::default()
                },
            ),
        ];
        commands.extend(self.chapter(name, scene));
        for panel in &scene.panels {
            commands.extend(self.panel(name, panel));
            commands.extend(self.rule(name, panel));
        }
        commands
    }

    /// The hairline of a panel, in the middle of the space before its
    /// margin; outside the panel, which `panel.set` replaces.
    /// A rule shows beside a panel with text, not at the left edge of the
    /// text block; else it is clear, so that `replace_panel` always
    /// finds one to replace.
    fn rule(&self, name: &str, panel: &Panel) -> Option<Command> {
        let ([x, y, _, h], geo) = (self.place(name, panel)?, self.geometry(name)?);
        let shown = x > geo.text[0] && !panel.is_empty();
        let color = (panel.rule.as_ref().filter(|_| shown))
            .and_then(|c| self.settings.value.colors().get(c))
            .copied()
            .unwrap_or_default();
        let line = [(x - geo.note - 1.5 * geo.gap).round(), y, 1.0, h];
        let id = format!("{}-rule", panel_id(name, &panel.id));
        Some(region::rect(&id, &panels_id(name), line, color))
    }

    /// A style of the settings, changed by a scene; a change that fails
    /// logs and leaves the style as it is.
    #[allow(clippy::expect_used, reason = "the settings check their styles")]
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

    /// Whether a scene has a head: a scene of panels or of an app.
    fn has_chapter(scene: &Scene) -> bool {
        scene.app.is_some() || !scene.panels.is_empty()
    }

    /// The layout of the head of a scene: its own, else the settings.
    fn layout(&self, scene: &Scene) -> Layout {
        (scene.chapter.layout).unwrap_or(self.settings.value.chapter.layout)
    }

    /// The height of the head of a chapter, with the line under it.
    fn chapter_height(&self, scene: &Scene) -> f64 {
        let b = self.settings.value.scale.baseline;
        match self.layout(scene) {
            Layout::Number => {
                let line = |name| self.ty(name, None).line;
                line("chapter").max(line("chapter_number")) + b
            }
            Layout::None => 0.0,
        }
    }

    /// The bar between the number and the title of a head at `x`: a
    /// hairline from the baseline of the title up to its capitals.
    // ponytail: tuned by eye for TeX Gyre Pagella, as `baseline_shift`;
    // read the metrics of the font when another font is off.
    fn bar(&self, x: f64) -> [f64; 4] {
        let title = self.ty("chapter", None);
        let baseline = HEAD * self.settings.value.scale.baseline + title.line / 2.0;
        let baseline = baseline + 0.21 * title.size;
        let cap = 0.69 * title.size;
        [x, (baseline - cap).round(), 1.0, cap.round()]
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
        let page = &config.page;
        let (note, gap) = (page.margin * em, page.gap * em);
        let cols = f64::from(scene.grid()[0]);
        let want = cols * page.measure * em + (cols - 1.0) * (note + 2.0 * gap);
        // A margin column on both sides, so that the text stays centered,
        // and room for the sides, so that the text never moves when one
        // shows: centered where it fits, else beside the sides.
        let [left, right] = self.side_widths(name);
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
        let number = self.number(name);
        let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
        let mut commands = Vec::new();
        match self.layout(scene) {
            Layout::Number => {
                // The bar and the title follow the number; `align` moves
                // them there once the core knows its width.
                let (number_ty, title_ty) =
                    (self.ty("chapter_number", None), self.ty("chapter", None));
                let chars: f64 = number.chars().map(|_| 1.0).sum();
                let guess = x + 0.6 * number_ty.size * chars;
                commands.push(region::rect(
                    &chapter_id(name, "bar"),
                    &panels_id(name),
                    self.bar(guess),
                    number_ty.color,
                ));
                let parts = [
                    ("number", number_ty, number, x),
                    ("title", title_ty, title, guess),
                ];
                for (part, ty, text, x) in parts {
                    let props = PropsUpdate {
                        transform: Some(translate(x, HEAD * b)),
                        ..ty.props(&text)
                    };
                    commands.push(create(
                        &chapter_id(name, part),
                        NodeKind::Text,
                        &panels_id(name),
                        props,
                    ));
                }
            }
            Layout::None => {}
        }
        commands
    }

    /// The box of a panel on the screen, without its margin.
    /// As many columns as fit at the least width of a panel; the panels
    /// wrap into new rows, which keep at least the least height.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few columns"
    )]
    fn place(&self, name: &str, panel: &Panel) -> Option<[f64; 4]> {
        let scene = self.scenes.get(name)?;
        let geo = self.geometry(name)?;
        let config = &self.settings.value;
        let [min_w, min_h] = config.min_panel.map(|m| m * config.scale.base);
        let gap = [geo.columns(), geo.baseline];
        let [x, y, w, h] = geo.text;
        let [cols, rows] = scene.grid();
        let fit = ((w + gap[0]) / (min_w + gap[0])).floor().max(1.0) as u32;
        let (cell, grid) = if fit >= cols {
            (panel.cell, [cols, rows])
        } else {
            // An empty panel takes no room.
            let cells: Vec<_> = (scene.panels.iter())
                .map(|p| {
                    let [x, y, w, h] = p.cell;
                    [x, y, w, if p.is_empty() { 0 } else { h }]
                })
                .collect();
            let (flowed, flowed_rows) = scene::flow(&cells, fit);
            let at = scene.panels.iter().position(|p| p.id == panel.id)?;
            // It still gets its own height: a node of no size stalls the
            // shader of the write.
            let [x, y, w, _] = flowed[at];
            ([x, y, w, panel.cell[3]], [fit, flowed_rows.max(1)])
        };
        let row = ((h - gap[1] * f64::from(rows - 1)) / f64::from(rows)).max(min_h);
        let shown = f64::from(grid[1]);
        let height = row * shown + gap[1] * (shown - 1.0);
        Some(scene::place(cell, grid, [x, y, w, height], gap))
    }

    /// The top of the lines of a panel, below its title.
    fn lines_top(&self, panel: &Panel) -> f64 {
        if panel.title.is_empty() {
            return 0.0;
        }
        let title = self.ty("heading", panel.title_style.as_ref()).line;
        title + self.settings.value.scale.baseline
    }

    /// The height of a panel cut to whole lines of its text, so that its
    /// clip never cuts through a line.
    fn whole_lines(&self, panel: &Panel, h: f64) -> f64 {
        let (top, line) = (
            self.lines_top(panel),
            self.ty("body", panel.style.as_ref()).line,
        );
        (top + ((h - top) / line).floor() * line).clamp(0.0, h)
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
                size: Some([w + indent, self.whole_lines(panel, h)]),
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

    /// Moves what the core lays out to its place: the title of the head
    /// after the number, the notes of the panels of a scene flush right to
    /// the margin.
    fn align(&mut self, name: &str) -> Result<(), ClientError> {
        let Some(scene) = self.scenes.get(name).cloned() else {
            return Ok(());
        };
        let Some(geo) = self.geometry(name) else {
            return Ok(());
        };
        let indent = geo.note + geo.gap;
        let mut commands = Vec::new();
        if Self::has_chapter(&scene)
            && self.layout(&scene) == Layout::Number
            && let Some([_, _, width, _]) = self.rect(&chapter_id(name, "number"))?
        {
            let space = BAR_SPACE * self.ty("chapter", None).size;
            let bar = self.bar((geo.text[0] + width + space).round());
            commands.push(move_to(chapter_id(name, "bar"), bar[0], bar[1]));
            let x = bar[0] + bar[2] + space.round();
            commands.push(move_to(chapter_id(name, "title"), x, HEAD * geo.baseline));
        }
        for panel in &scene.panels {
            let id = panel_id(name, &panel.id);
            // A title that wraps pushes the lines down by its extra lines.
            let mut y = self.lines_top(panel);
            if !panel.title.is_empty()
                && let Some([_, _, _, height]) = self.rect(&format!("{id}-title"))?
            {
                let line = self.ty("heading", panel.title_style.as_ref()).line;
                y += (height - line).max(0.0);
            }
            let Some(rows) = panel.columns() else {
                if !panel.lines.is_empty() && y > self.lines_top(panel) {
                    commands.push(move_to(format!("{id}-lines"), indent, y));
                }
                continue;
            };
            let body = self.ty("body", panel.style.as_ref());
            let note = self.ty("margin", panel.note_style.as_ref());
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

    /// Scrolls the page of the current scene by `dy`, from its top to its
    /// last panel at the bottom of the text block.
    fn scroll_page(&mut self, node: &str, dy: f64) -> Result<(), ClientError> {
        let Some(name) = self.current.clone().filter(|n| wheel_id(n) == node) else {
            return Ok(());
        };
        let (Some(scene), Some(geo)) = (self.scenes.get(&name), self.geometry(&name)) else {
            return Ok(());
        };
        let bottom = (scene.panels.iter())
            .filter_map(|p| self.place(&name, p).map(|[_, y, _, h]| y + h))
            .fold(0.0, f64::max);
        let most = (bottom - geo.text[1] - geo.text[3]).max(0.0);
        let old = self.scroll.get(&name).copied().unwrap_or_default();
        let new = (old + dy).clamp(0.0, most);
        if (new - old).abs() < f64::EPSILON {
            return Ok(());
        }
        self.scroll.insert(name.clone(), new);
        let b = self.settings.value.scale.baseline;
        self.apply(vec![move_to(panels_id(&name), 0.0, -2.0 * b - new)])
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

    /// The rect of an item, or of its hidden copy when `probed`.
    fn measure(&mut self, id: &str, probed: bool) -> Result<[f64; 4], ClientError> {
        let rect = if probed {
            self.rect(&region::probe(id))
        } else {
            self.rect(id)
        };
        Ok(rect?.unwrap_or_default())
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
            let Some(h) = self
                .place(name, panel)
                .map(|r| self.whole_lines(panel, r[3]))
            else {
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

    /// Puts a window into the text block of its scene, below the head,
    /// without a frame, at the box that the render gives it too.
    fn place_window(&self, name: &str, surface: &str, attach: bool) -> Vec<Command> {
        let (Some(output), Some(window)) = (&self.output, self.window_box(name)) else {
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
        let mut commands = vec![put];
        commands.extend(region::place(surface, window, output.size));
        commands
    }

    /// Shows a scene. A switch of the user, a new window or the scene
    /// after a closed window gives the keyboard to the window of the
    /// scene, or to none: a method never takes it from a window.
    fn show(&mut self, name: &str, user: bool) -> Result<(), RpcError> {
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
        if user {
            self.focused.clone_from(&window);
            commands.push(Command::Focus {
                surface: window.clone(),
            });
        }
        if self.current.as_deref() != Some(name) {
            self.previous = self.current.replace(name.into());
        }
        // The regions switch in the same transaction as the scene.
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        commands.extend(self.regions().map_err(rejected)?);
        self.apply(commands).map_err(rejected)?;
        // A hidden scene has no layout of the core: its notes align now.
        self.align(name).map_err(rejected)?;
        let waiting = (self.starting.values()).any(|t| *t == Target::Scene(name.into()));
        if let (Some(command), None, false) = (app, window, waiting) {
            self.launch_scene(name, &command)
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        Ok(())
    }

    /// Starts the app of a scene, whose window then fills its text block.
    /// A command that names a content of kind app runs that content.
    fn launch_scene(&mut self, name: &str, command: &str) -> Result<(), ClientError> {
        let config = &self.settings.value;
        let command = match config.contents.get(command) {
            Some(app) if app.kind == Kind::App => region::command(&app.command, config.colors()),
            _ => command.into(),
        };
        let Some((token, pid)) = launch(&command) else {
            return Ok(());
        };
        self.starting.insert(token, Target::Scene(name.into()));
        self.pids.insert(name.into(), pid);
        self.launched.push((name.into(), pid));
        self.apply(vec![wait()])
    }

    /// An app of a scene that ended without a window leaves its scene
    /// empty: the scene before it shows again (W11), and the scene of
    /// `app.open` goes.
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
            let starting = &self.starting;
            self.quiet.retain(|t| starting.contains_key(t));
            if self.windows.contains_key(&name) {
                continue;
            }
            // The scene of app.open goes with its app, also of one such as
            // ls that never had a window.
            if self.transient.contains(&name) {
                self.remove_scene(&name)?;
            } else {
                self.leave(&name);
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
        self.add_scene(&name, Scene::app(command));
        self.transient.insert(name.clone());
        // The group with the head, which the build of a scene has too.
        let mut commands = self.scene_commands(&name);
        commands.push(regions_on_top());
        self.apply(commands).map_err(rejected)?;
        self.renumber().map_err(rejected)?;
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
        let mut known = (self.windows.values())
            .chain(self.docks.values())
            .chain(self.fitting.keys());
        if !self.built || known.any(|s| s == surface) {
            return Ok(());
        }
        let started = token.and_then(|t| self.starting.remove(t));
        // A new window takes the keyboard, except one of app.open.
        let user = !token.is_some_and(|t| self.quiet.remove(t));
        if let Some(Target::Dock(content)) = &started
            && !self.docks.contains_key(content)
        {
            return self.adopt_dock(surface, content);
        }
        let mut commands = Vec::new();
        let name = match started {
            Some(Target::Scene(name)) if !self.windows.contains_key(&name) => name,
            _ => {
                let mut name = name_of(app_id.unwrap_or("app"));
                if self.scenes.contains_key(&name) {
                    name = format!("{name}-{surface}");
                }
                self.add_scene(&name, Scene::default());
                self.transient.insert(name.clone());
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
        let new = !commands.is_empty();
        commands.extend(self.place_window(&name, surface, true));
        self.apply(commands)?;
        self.windows.insert(name.clone(), surface.into());
        if let Err(err) = self.show(&name, user) {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        if new {
            // The overview lists the new scene.
            self.renumber()?;
        }
        Ok(())
    }

    /// Takes the window of a docked app that started. It waits hidden
    /// until it has the size of its region, so that it never shows at the
    /// size that it chose itself.
    fn adopt_dock(&mut self, surface: &str, content: &str) -> Result<(), ClientError> {
        let mut commands = vec![Command::Attach {
            surface: surface.into(),
            parent: PARKED.into(),
        }];
        let Some(size) = self.dock_size(content) else {
            self.apply(commands)?;
            self.fitting
                .insert(surface.into(), (content.into(), [0, 0]));
            return self.fit(surface);
        };
        commands.push(Command::Configure {
            surface: surface.into(),
            size: Some(size),
            states: vec![ToplevelState::Tiled],
        });
        // A client that takes another size shows after this time.
        commands.push(Command::Animate {
            id: PARKED.into(),
            props: PropsUpdate::default(),
            duration: FIT_MS,
            easing: Easing::Linear,
        });
        self.apply(commands)?;
        self.fitting.insert(surface.into(), (content.into(), size));
        Ok(())
    }

    /// A docked app that waited parked goes into its region, with the
    /// keyboard when the user started it. A new dock changes the items,
    /// so the render draws anew.
    fn fit(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some((content, _)) = self.fitting.remove(surface) else {
            return Ok(());
        };
        let user = self.dock_focus.as_ref() == Some(&content);
        if user {
            self.dock_focus = None;
        }
        self.docks.insert(content, surface.into());
        self.render()?;
        if user {
            self.focus(Some(surface.into()))?;
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
        let docked = self.docks.values().any(|s| s == surface);
        if docked || self.fitting.contains_key(surface) {
            self.apply(vec![Command::Attach {
                surface: surface.into(),
                parent: PARKED.into(),
            }])?;
            // Back into its region.
            if docked && self.rendered.is_some() {
                self.rendered = Some(Vec::new());
            }
            return self.render();
        }
        self.adopt(surface, None, None)
    }

    /// A window closed: the scene before shows instead of a scene without
    /// its window, and the scene of `app.open` or of a window that no
    /// scene started goes once its app ended.
    fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        if self.fitting.remove(surface).is_some() {
            return Ok(());
        }
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
        self.leave(&name);
        if !self.transient.contains(&name) {
            return Ok(());
        }
        // The scene goes when the app ended too; until then it waits.
        let pid = self.pids.get(&name).copied().flatten();
        if pid.is_some_and(alive) {
            self.launched.push((name, pid));
            return self.apply(vec![wait()]);
        }
        self.remove_scene(&name)
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
        self.replace_panel(&scene, &new, true)
    }

    /// Puts a changed panel into its scene, by its id; it writes itself
    /// when the scene shows.
    /// Without `write` the panel changes in place when it keeps its nodes,
    /// so that an agent state does not blank and write its column anew.
    pub fn replace_panel(&mut self, scene: &str, new: &Panel, write: bool) -> Result<(), RpcError> {
        new.check_text().map_err(invalid)?;
        let found = (self.scenes.get_mut(scene))
            .and_then(|v| v.panels.iter_mut().find(|p| p.id == new.id))
            .ok_or_else(|| invalid(format!("no panel {} in scene {scene}", new.id)))?;
        // The same text again neither writes nor builds anything.
        if found == new {
            return Ok(());
        }
        // A panel that moves, fills or empties moves the panels after it
        // on a narrow page: the page builds anew.
        let moved = found.cell != new.cell || found.is_empty() != new.is_empty();
        let old = std::mem::replace(found, new.clone());
        let scene = scene.to_owned();
        let shown = self.current.as_deref() == Some(&scene);
        let commands = if moved {
            let mut commands = vec![Command::Remove {
                id: page_id(&scene),
                to: None,
            }];
            commands.extend(self.panels(&scene));
            if write && shown {
                commands.extend(self.write(&scene, &[new]));
            }
            commands
        } else {
            let created = self.panel(&scene, new);
            let updates = (!write)
                .then(|| in_place(&self.panel(&scene, &old), &created))
                .flatten();
            let mut commands = updates.unwrap_or_else(|| {
                let mut commands = vec![Command::Remove {
                    id: panel_id(&scene, &new.id),
                    to: None,
                }];
                commands.extend(created);
                if write && shown {
                    commands.extend(self.write(&scene, &[new]));
                }
                commands
            });
            commands.push(Command::Remove {
                id: format!("{}-rule", panel_id(&scene, &new.id)),
                to: None,
            });
            commands.extend(self.rule(&scene, new));
            commands
        };
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
        let names = self.order();
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
                // The new window takes the keyboard.
                let _ = launch(&command);
                return;
            }
            Action::Toggle(region) => {
                if let Err(err) = self.toggle(region, None, true) {
                    eprintln!("chora-mc: toggle {}: {err}", region.name());
                }
                return;
            }
            Action::Solo { content, region } => {
                if let Err(err) = self.solo(&content, region, None, true) {
                    eprintln!("chora-mc: solo {content}: {err}");
                }
                return;
            }
        };
        if let Err(err) = self.show(&next, true) {
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
    use chora_scene::{Command, NodeKind, PropsUpdate};

    fn text(id: &str, text: &str, color: Option<[f64; 4]>) -> Command {
        super::create(
            id,
            NodeKind::Text,
            "panel",
            PropsUpdate {
                text: Some(text.into()),
                color,
                ..PropsUpdate::default()
            },
        )
    }

    /// A new state of an agent changes the text of its column in place:
    /// updates only, no remove and no new node that blanks it.
    #[test]
    fn the_same_nodes_change_in_place() {
        let old = [
            text("a-title", "Voice", None),
            text("a-lines", "listening", None),
        ];
        let new = [
            text("a-title", "Voice", Some([1.0; 4])),
            text("a-lines", "thinking", None),
        ];
        let updates = super::in_place(&old, &new).expect("the same nodes");
        assert_eq!(updates.len(), 2);
        assert!(matches!(&updates[1], Command::Update { id, props }
            if id == "a-lines" && props.text.as_deref() == Some("thinking")));
    }

    /// A node that comes or goes, or a prop that an update cannot clear,
    /// needs the panel anew.
    #[test]
    fn other_nodes_or_a_cleared_prop_build_anew() {
        let old = [text("a-lines", "x", Some([1.0; 4]))];
        assert!(super::in_place(&old, &[text("a-lines", "x", None)]).is_none());
        assert!(super::in_place(&old, &[text("a-title", "x", Some([1.0; 4]))]).is_none());
        assert!(super::in_place(&old, &[]).is_none());
    }

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

    /// The claim of the indicator of the voice agent goes out at the
    /// start, again after the agent connected, and with its first event
    /// after a failed claim; then no more.
    #[test]
    fn mc_claims_the_indicator_of_voice_once_per_connection() {
        let (mut mc, model) = crate::testing::mc("claim", [1920, 1080]);
        let claims = || model.lock().unwrap().calls.len();
        mc.claim_indicator(false);
        assert_eq!(claims(), 1);
        mc.follow("voice.state", "voice", serde_json::json!({}))
            .unwrap();
        assert_eq!(claims(), 1, "claimed already");
        let connected = chora_scene::Event::ClientConnected {
            name: "voice".into(),
        };
        chora_kit::runtime::Plugin::event(&mut mc, connected).unwrap();
        assert_eq!(claims(), 2);
        assert!(!mc.claim_due);
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
                "scene.rename",
                "scene.move",
                "app.open",
                "region.toggle",
                "region.put",
                "region.solo"
            ]
        );
    }

    /// Live: the cursor jumped out of the terminal while the voice agent
    /// changed scenes, regions and contents. Methods never move the
    /// keyboard; a switch of the user does.
    #[test]
    fn only_the_user_moves_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        use serde_json::json;
        let (mut mc, model) = crate::testing::mc("focus", [1920, 1080]);
        let focuses = || model.lock().unwrap().focuses.len();
        mc.show("agenda", true).unwrap();
        let before = focuses();
        assert_eq!(before, 1);
        for (method, params) in [
            ("scene.show", json!({"name": "morning"})),
            ("content.set", json!({"content": "notes", "lines": ["x"]})),
            ("hud.say", json!({"text": "one two", "duration_ms": 100})),
            ("agent.set", json!({"name": "ada", "state": "working"})),
            ("agent.focus", json!({"name": "ada"})),
            ("region.put", json!({"content": "notes", "region": "left"})),
            ("region.toggle", json!({"region": "right"})),
            (
                "region.solo",
                json!({"content": "notes", "region": "drawer"}),
            ),
            ("scene.move", json!({"name": "agenda", "to": 3})),
        ] {
            mc.method(method, params).unwrap();
        }
        assert_eq!(focuses(), before);
        mc.show("agenda", true).unwrap();
        assert_eq!(focuses(), before + 1);
    }
}
