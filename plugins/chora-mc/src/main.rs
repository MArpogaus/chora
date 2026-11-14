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
#[cfg(test)]
mod audit;
mod config;
mod fonts;
mod keys;
mod manage;
mod mic;
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
    launch,
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Easing, Event, MAX_DURATION, NodeKind, OutputInfo, PropsUpdate, ROOT, ShaderEffect,
    ToplevelState, Transform, Uniform,
    ipc::{ClientError, ExportMethod, Filter, PressState, Query, RpcError},
};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::{Action, Config, Layout, Style, Type},
    fonts::Ink,
    region::{FIT_MS, Kind, MAX_LINES, PARKED, REGIONS, Region, Rule},
    scene::{Panel, Scene},
};

/// The client name, and the id of the group that holds the scenes.
const NAME: &str = "mc";
/// The most panels of a scene whose rows take the pointer.
const MAX_HIT_PANELS: usize = 24;
/// The id of the line of the HUD before the regions; a start removes it.
const HUD: &str = "mc-hud";
/// The id of a hidden node whose empty animations time the typewriter.
const TICK: &str = "mc-tick";
/// The id of a hidden node whose empty animations time the clocks.
const CLOCK: &str = "mc-clock";
/// The id of a hidden node whose empty animations time the reading of the
/// volume.
const VOLUME: &str = "mc-volume";
/// How often the plugin takes a reading of the volume, in milliseconds.
const VOLUME_MS: f64 = 500.0;
/// After how many timer runs the volume is read again by itself.
const VOLUME_AGAIN: u32 = 10;
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
    /// Fills only this output, by name, and leaves the others to other
    /// window managers; the first output without it.
    #[arg(long = "output", value_name = "NAME")]
    output: Option<String>,
    /// Manages the windows of its output: claims the work area, so that
    /// the core gives it new windows. Without it mc only shows its
    /// scenes, as the source of the panes of another plugin.
    #[arg(long)]
    manage_windows: bool,
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

/// Whether the HUD host shows the chat in the margin column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chat {
    /// The column shows the door.
    Closed,
    /// The column holds the box of the chat.
    Open,
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

/// Whether the panels may write and blur: not after the watchdog of the
/// core dropped the effects, until the settings change.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shaders {
    On,
    Dropped,
}

struct Mc {
    rt: Runtime<Action>,
    /// Whether mc claims its output (`--manage-windows`).
    manage_windows: bool,
    /// Whether a page takes the keyboard for its plain keys.
    page_keys: keys::PageKeys,
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
    /// The microphone of the voice agent, for the word of the state line.
    mic: mic::Mic,
    /// The scenes that showed before the current one, the last one last:
    /// Escape on a page goes back along them.
    trail: Vec<String>,
    /// The one output that the plugin fills.
    output: Option<OutputInfo>,
    /// Whether the nodes exist.
    built: bool,
    hud: SetHud,
    typing: Option<Typing>,
    /// The files of the effects that write new text and that blur.
    shader: PathBuf,
    blur: PathBuf,
    shaders: Shaders,
    /// The panels whose write animation runs, each with the number of
    /// the transaction that started it: those of one transaction end in
    /// the same frame.
    writing: HashMap<String, u64>,
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
    /// The surface that has the keyboard focus of the core, from
    /// `focus.changed`.
    core_focus: Option<String>,
    /// The window that a `drag` binding holds since its press.
    dragged: Option<String>,
    /// The activation tokens of the apps of `app.open`: voice and agents
    /// start them, and their first window does not take the keyboard.
    quiet: BTreeSet<String>,
    /// The docked app that the user started with a toggle: it takes the
    /// keyboard when its window fits.
    dock_focus: Option<String>,
    /// The text of each clock.
    clocks: BTreeMap<String, String>,
    /// The worker that reads and changes the volume.
    volume: chora_kit::volume::Mixer,
    /// The volume that the worker read last.
    volume_reading: Option<chora_kit::volume::Reading>,
    /// How often the volume timer ran.
    volume_ticks: u32,
    /// The title and the lines of each text content.
    texts: BTreeMap<String, (String, Vec<String>)>,
    /// The agents of `agent.set`, in the order of their first report.
    agents: Vec<agent::Agent>,
    /// The names of the helpers that voice reported.
    voiced: Vec<String>,
    /// The agent that the agents scene shows alone, with its transcript.
    agent_focus: Option<String>,
    /// The content of `region.solo` that shows alone in its region.
    solo: Option<Solo>,
    /// How much of the key list shows.
    keys: keys::List,
    /// Whether the HUD host shows the chat in the margin column.
    chat: Chat,
    /// Where the box of the chat of the last layout is.
    slot: Option<[f64; 4]>,
    /// The output and the box that the host was told last.
    slot_sent: Option<(String, Option<[f64; 4]>)>,
    /// The spoken confirmation that shows, until its answer.
    confirming: Option<agent::Confirming>,
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
    let path = settings::File::<Config>::path(args.config.clone(), "chora-mc")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let dir = args
        .scenes
        .clone()
        .or_else(|| chora_kit::xdg::config_home().map(|d| d.join("chora/mc/scenes")))
        .ok_or("no $XDG_CONFIG_HOME or $HOME")?;
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    // One file for all starts, written when it differs.
    let [shader, blur] = [("write", WRITE_SHADER), ("blur", BLUR_SHADER)].map(|(name, text)| {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR");
        shader_file(runtime, "chora-mc", &format!("{name}.frag"), text)
    });
    let (shader, blur) = (shader?, blur?);
    let rt = Runtime::connect(NAME, args.output.clone().into_iter().collect())?;
    let mut mc = Mc::new(rt, settings, dir, [shader, blur]);
    mc.manage_windows = args.manage_windows;
    if args.output.is_some() {
        mc.page_keys = keys::PageKeys::Bound;
    }
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
    loop {
        let event = mc.rt.conn.next_event()?;
        if let Some((call, err)) = spoofs_voice(&event) {
            mc.rt.conn.answer(call, Err(err))?;
            continue;
        }
        runtime::dispatch(&mut mc, event)?;
    }
}

/// A call of `agent.set` or `agent.remove` for the agent `voice` from
/// another client: its call and the refusal. Only the voice agent sets
/// the state that mc shows in place of its dot (review 4.9).
fn spoofs_voice(event: &Event) -> Option<(u64, RpcError)> {
    let Event::MethodCalled {
        call,
        caller,
        method,
        params,
        ..
    } = event
    else {
        return None;
    };
    let agent = matches!(method.as_str(), "agent.set" | "agent.remove");
    (agent && params["name"] == VOICE && caller != VOICE).then(|| {
        let err = format!("only the voice agent sets the agent {VOICE}");
        (*call, RpcError::new(RpcError::REJECTED, err))
    })
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
            manage_windows: false,
            page_keys: keys::PageKeys::Plain,
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
            mic: mic::Mic::default(),
            trail: Vec::new(),
            output: None,
            built: false,
            hud: SetHud {
                state: String::new(),
                text: String::new(),
            },
            typing: None,
            shader,
            blur,
            shaders: Shaders::On,
            writing: HashMap::new(),
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
            core_focus: None,
            dragged: None,
            quiet: BTreeSet::new(),
            dock_focus: None,
            clocks: BTreeMap::new(),
            volume: chora_kit::volume::Mixer::new(NAME),
            volume_reading: None,
            volume_ticks: 0,
            texts: BTreeMap::new(),
            agents: Vec::new(),
            voiced: Vec::new(),
            agent_focus: None,
            solo: None,
            keys: keys::List::default(),
            chat: Chat::Closed,
            slot: None,
            slot_sent: None,
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
/// The client name of the HUD host.
const ASSISTANT: &str = "assistant";

/// The events; `voice.*` are those of the voice agent (D65),
/// `assistant.chat` says where the HUD host shows its chat.
const EVENTS: [&str; 19] = [
    "voice.*",
    "focus.changed",
    "node.key",
    "assistant.chat",
    "assistant.ready",
    "effects.dropped",
    "client.connected",
    "client.disconnected",
    "pointer.scroll",
    "pointer.pressed",
    "pointer.entered",
    "pointer.left",
    "animation.done",
    "surface.created",
    "surface.destroyed",
    "surface.changed",
    "surface.detached",
    "output.changed",
    "output.removed",
];

/// The most scenes that Escape goes back along.
const TRAIL: usize = 32;

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
        runtime::method(
            "pane.focus",
            "Gives the keyboard to the app of the scene NAME, else to its page; a plugin that \
             shows the page as a pane calls it while the user flies into the pane.",
            json!({"name": {"type": "string"}}),
            &["name"],
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
        runtime::privileged(runtime::method(
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
        )),
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
        runtime::privileged(runtime::method(
            "notify.push",
            "Shows a notification on top of the stack of notify; a critical one also in \
             red in urgent.",
            json!({
                "title": {"type": "string"},
                "body": {"type": "string"},
                "urgency": {"type": "string", "enum": ["low", "normal", "critical"]},
            }),
            &["title"],
        )),
        runtime::method("notify.clear", "Empties the notifications.", json!({}), &[]),
        chora_kit::give::method(),
    ]
}

impl chora_kit::give::Windows for Mc {
    fn conn(&mut self) -> &mut chora_kit::conn::Conn {
        &mut self.rt.conn
    }

    fn has(&self, surface: &str) -> bool {
        self.windows.values().any(|s| s == surface)
    }

    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        self.give(chora_kit::give::Give {
            surface: surface.into(),
            to: to.into(),
            grab: None,
        })
    }
}

impl Plugin for Mc {
    type Action = Action;

    fn runtime(&mut self) -> &mut Runtime<Action> {
        &mut self.rt
    }

    #[allow(clippy::too_many_lines, reason = "one arm per event")]
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
            Event::EffectsDropped { client } if client == NAME => self.effects_dropped(),
            Event::ClientDisconnected { name } if name == VOICE => {
                self.voice_gone();
                self.voice_changed()
            }
            Event::ClientConnected { name } if name == VOICE => self.voice_changed(),
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                ..
            } => {
                self.act(&binding);
                Ok(())
            }
            Event::InputButton {
                binding,
                state,
                position,
                surface,
                ..
            } => {
                self.drag(&binding, state, position, surface);
                Ok(())
            }
            Event::Emitted {
                event,
                client,
                data,
            } => self.emitted(&event, &client, data),
            Event::FocusChanged { surface } => {
                self.core_focus = surface;
                Ok(())
            }
            Event::NodeKey {
                key,
                state: PressState::Pressed,
                modifiers,
                ..
            } => {
                self.page_key(&key, &modifiers);
                Ok(())
            }
            Event::PointerScroll { node, dy, .. } => self.scroll_page(&node, dy),
            Event::PointerPressed {
                node,
                position,
                injected,
                ..
            } => self.press(&node, position, injected),
            Event::PointerEntered { node, .. } => self.mark(&node, true),
            Event::PointerLeft { node } => self.mark(&node, false),
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
            chora_kit::give::METHOD => self.give(runtime::params(params)?),
            _ => self.region_method(method, params),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        let plain = std::mem::replace(&mut self.shaders, Shaders::On) == Shaders::Dropped;
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
        self.name_theme()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        let theme_only = self.built && !plain && self.shape.as_ref() == Some(&self.shape_of());
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
            "pane.focus" => {
                let Show { name } = runtime::params(params)?;
                // A scene that is not shown takes no keyboard (review
                // rc3 round 4, 2.7).
                if self.current.as_deref() != Some(name.as_str()) {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        format!("the scene {name} is not the current one"),
                    ));
                }
                self.focus_scene(&name)
                    .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
                Ok(Value::Null)
            }
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
        let own_state = client == VOICE && kind == Some("state") && data.get("name").is_none();
        if own_state && self.mic_follows(mic::Mic::of(&data)) {
            self.render()?;
        }
        let host = client == VOICE && self.settings.value.voice_hud == config::VoiceHud::Host;
        let hosts_it = match kind {
            Some("confirm" | "say") => true,
            Some("state") => data.get("name").is_none(),
            _ => false,
        };
        if host && hosts_it {
            return Ok(());
        }
        let done = match kind {
            Some("state") => {
                if let Some(object) = data.as_object_mut() {
                    object.entry("name").or_insert_with(|| client.into());
                    object.retain(|key, _| agent::STATE_FIELDS.contains(&key.as_str()));
                }
                if client == VOICE {
                    self.voiced(&data);
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
        if !self.claim_due || self.settings.value.voice_hud == config::VoiceHud::Host {
            return;
        }
        let call = json!({"client": VOICE, "method": "indicator.claim", "params": {}});
        let done = self.rt.conn.call::<_, Value>("methods.call", call).is_ok();
        self.claim_due = !done && !last;
    }

    /// An event of a client (D65): of the voice agent, or `assistant.chat`
    /// and `assistant.ready` of the HUD host. `voice.*` also matches a client named `voice.evil`:
    /// only the agent itself shows its state (review 4.9).
    fn emitted(&mut self, event: &str, client: &str, data: Value) -> Result<(), ClientError> {
        match client {
            VOICE => self.follow(event, client, data),
            ASSISTANT if event == "assistant.chat" => self.chat_drawer(&data),
            ASSISTANT if event == "assistant.ready" => self.name_theme(),
            _ => Ok(()),
        }
    }

    /// A press of the pointer: on a column of a helper it opens the chat,
    /// else it is a click.
    fn press(&mut self, node: &str, at: [f64; 2], injected: bool) -> Result<(), ClientError> {
        if self.click_agent(node, at) || self.click_mic(node, injected) || self.click_hit(node) {
            return Ok(());
        }
        self.click(node, at, injected)
    }

    /// `assistant.chat` of the HUD host: while it shows the chat on the
    /// output of mc, the margin column holds it: the notifications fold,
    /// the key list goes, and mc emits the box under the head `Chat` as
    /// `mc.chat.slot {output, rect}`; `rect` is `null` when it closes.
    fn chat_drawer(&mut self, data: &Value) -> Result<(), ClientError> {
        let Some(output) = self.output.as_ref().map(|o| o.name.clone()) else {
            return Ok(());
        };
        let here = data.get("output").and_then(Value::as_str) == Some(output.as_str());
        let open = here && data.get("open").and_then(Value::as_bool) == Some(true);
        if open == (self.chat == Chat::Open) {
            return Ok(());
        }
        self.chat = if open { Chat::Open } else { Chat::Closed };
        self.render()?;
        let still_open = data.get("open").and_then(Value::as_bool) == Some(true);
        let mut ours = self.windows.values().chain(self.docks.values());
        let taken = (self.core_focus.as_ref()).is_some_and(|s| !ours.any(|o| o == s));
        if open || still_open || taken {
            return Ok(());
        }
        let commands = self.keyboard_back();
        self.apply(commands)
    }

    /// Gives the keyboard to the page that shows, when no window and no
    /// node has it: at the start and after the chat of the host closed,
    /// so that the plain keys of a page work without a click.
    fn take_free_keyboard(&mut self) -> Result<(), ClientError> {
        let Some(name) = self.current.clone() else {
            return Ok(());
        };
        if self.page_keys == keys::PageKeys::Bound || self.windows.contains_key(&name) {
            return Ok(());
        }
        let query = Query {
            selector: "*".into(),
            filter: Some(chora_scene::ipc::Filter {
                focused: true,
                ..chora_scene::ipc::Filter::default()
            }),
        };
        let focused: Vec<Value> = self.rt.conn.call("scene.query", query)?;
        if !focused.is_empty() {
            return Ok(());
        }
        self.apply(vec![Command::FocusNode {
            node: Some(page_id(&name)),
        }])
    }

    /// The voice agent came or went: its indicator is due again, and the
    /// state of its microphone is unknown until its next state.
    fn voice_changed(&mut self) -> Result<(), ClientError> {
        self.claim_due = true;
        self.claim_indicator(false);
        if self.mic_follows(mic::Mic::Unknown) {
            self.render()?;
        }
        Ok(())
    }

    /// A key without a modifier while a page has the keyboard: Right and
    /// Left turn the page, Escape goes back one step.
    fn page_key(&mut self, key: &str, modifiers: &[String]) {
        let action = match key {
            _ if !modifiers.is_empty() => return,
            "Right" => Action::NextScene,
            "Left" => Action::PreviousScene,
            "Escape" => return self.back(),
            _ => return,
        };
        self.turn(action);
    }

    /// One step back: the full key list closes; else the page that showed
    /// before shows again, and at the end of that trail the first page.
    fn back(&mut self) {
        if self.keys == keys::List::All {
            self.keys = keys::List::Six;
            if let Err(err) = self.render() {
                eprintln!("chora-mc: keys: {err}");
            }
            return;
        }
        let current = self.current.clone();
        let known =
            |name: &String| self.scenes.contains_key(name) && Some(name) != current.as_ref();
        let mut trail = std::mem::take(&mut self.trail);
        let back = std::iter::from_fn(|| trail.pop()).find(known);
        let first = Some(self.settings.value.default_scene.clone()).filter(known);
        if let Some(name) = back.or(first)
            && let Err(err) = self.show(&name, true)
        {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        self.trail = trail;
    }

    /// Names the theme of mc to the HUD host, which colours its look by it.
    fn name_theme(&mut self) -> Result<(), ClientError> {
        let dark = self.settings.value.theme == config::Theme::Dark;
        let emit = chora_kit::hud::theme_event(NAME, dark);
        self.rt.conn.call::<_, Value>("events.emit", emit).map(drop)
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

/// The tag `pane` of the root group: the scene that shows.
fn pane_tag(scene: Option<&str>) -> BTreeMap<String, String> {
    BTreeMap::from([("pane".into(), scene.unwrap_or_default().into())])
}

fn pane_tag_update(scene: &str) -> Command {
    Command::Update {
        id: NAME.into(),
        props: PropsUpdate {
            meta: Some(pane_tag(Some(scene))),
            ..PropsUpdate::default()
        },
    }
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
            VOLUME => self.volume_tick(),
            PARKED => {
                let late: Vec<String> = self.fitting.keys().cloned().collect();
                late.iter().try_for_each(|surface| self.fit(surface))
            }
            _ => self.written(node),
        }
    }

    /// Takes the reading of the volume that the worker has, shows it when
    /// it changed, and asks for a new one now and then.
    fn volume_tick(&mut self) -> Result<(), ClientError> {
        self.volume_ticks += 1;
        if self.volume_ticks % VOLUME_AGAIN == 1 {
            self.volume.refresh();
        }
        self.apply(vec![volume_wait()])?;
        let reading = self.volume.take();
        if reading.is_some_and(|r| Some(r) != self.volume_reading) {
            self.volume_reading = reading;
            self.render()?;
        }
        Ok(())
    }

    /// The watchdog of the core removed the effects of the plugin. An
    /// update that was on its way then may have put a blur or a write
    /// effect back, so the effects of the panels go once more.
    fn effects_dropped(&mut self) -> Result<(), ClientError> {
        if self.shaders == Shaders::On {
            eprintln!(
                "chora-mc: frames were slow, the core dropped the effects; the panels \
                 stay plain until the settings change"
            );
        }
        self.shaders = Shaders::Dropped;
        let mut commands = Vec::new();
        for (name, scene) in &self.scenes {
            for panel in &scene.panels {
                let id = panel_id(name, &panel.id);
                let shaded = panel.blur.is_some() || self.writing.contains_key(&id);
                if shaded && self.place(name, panel).is_some() {
                    commands.push(Command::Update {
                        id,
                        props: PropsUpdate {
                            effects: Some(Vec::new()),
                            ..PropsUpdate::default()
                        },
                    });
                }
            }
        }
        self.writing.clear();
        self.apply(commands)
    }

    /// The write animation of a panel ended, and with it those of the
    /// same transaction: their effects go in one transaction, so that a
    /// page that rests costs no shader pass per panel.
    fn written(&mut self, node: &str) -> Result<(), ClientError> {
        let Some(batch) = self.writing.get(node).copied() else {
            return Ok(());
        };
        let mut commands = Vec::new();
        for (name, scene) in &self.scenes {
            for panel in &scene.panels {
                let id = panel_id(name, &panel.id);
                if self.writing.get(&id) == Some(&batch) {
                    commands.push(Command::Update {
                        id,
                        props: PropsUpdate {
                            effects: Some(self.blur_effect(panel).into_iter().collect()),
                            ..PropsUpdate::default()
                        },
                    });
                }
            }
        }
        self.writing.retain(|_, b| *b != batch);
        self.apply(commands)
    }

    /// Remembers the panels that the commands start to write.
    fn note_writes(&mut self, commands: &[Command]) {
        let batch = self.writing.values().max().map_or(0, |b| b + 1);
        for command in commands {
            if let Command::Animate { id, props, .. } = command
                && props.effects.is_some()
            {
                self.writing.insert(id.clone(), batch);
            }
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
        self.name_theme()?;
        self.grab_all()?;
        self.build()?;
        self.grab_claimed()?;
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
        self.take_free_keyboard()?;
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
        self.grab_some(|action| !needs_claim(action))?;
        if self.built {
            self.grab_claimed()?;
        }
        self.rt.give_keys(Some(|to| Action::Give(to.into())));
        self.rt
            .accept_injected(self.settings.value.test_accept_injected_keys);
        self.rt.grab_give_keys()
    }

    /// Grabs the bindings whose action `wanted` accepts.
    #[allow(clippy::expect_used, reason = "the settings check their bindings")]
    fn grab_some(&mut self, wanted: impl Fn(&Action) -> bool) -> Result<(), ClientError> {
        let bindings = self.settings.value.bindings.clone();
        for (text, action) in bindings.into_iter().filter(|(_, a)| wanted(a)) {
            let binding = text.parse().expect("the settings check their bindings");
            self.rt.grab(binding, action)?;
        }
        Ok(())
    }

    /// Grabs the `drag` and `volume` bindings, after the build that sets
    /// the claim: only window managers with a claim share a pointer button
    /// (D55) and grab a media key without a grant, so mc grabs none unless
    /// it manages windows.
    fn grab_claimed(&mut self) -> Result<(), ClientError> {
        if !self.manage_windows {
            return Ok(());
        }
        self.grab_some(needs_claim)
    }

    /// The claim: the work area of an output, below the header and above
    /// the footer. Another manager on the same output has the smaller
    /// claim only if mc's is not the whole output (review 2.10).
    fn work_area(&self, [x, y, w, h]: [f64; 4]) -> [f64; 4] {
        let config = &self.settings.value;
        let line = |r| config.regions.get(r).size * config.scale.baseline;
        let (head, foot) = (line(Region::Header), line(Region::Footer));
        [x, y + head, w, (h - head - foot).max(0.0)]
    }

    /// The keyboard goes to the window of a scene, else to its page.
    fn focus_scene(&mut self, name: &str) -> Result<(), ClientError> {
        let command = match self.windows.get(name) {
            Some(surface) => Command::Focus {
                surface: Some(surface.clone()),
            },
            None => Command::FocusNode {
                node: Some(page_id(name)),
            },
        };
        self.apply(vec![command])
    }

    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if commands.is_empty() {
            return Ok(());
        }
        self.note_writes(&commands);
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
        let work = self.work_area([x, y, w, h]);
        commands.push(create(
            NAME,
            NodeKind::Group,
            ROOT,
            PropsUpdate {
                transform: Some(translate(x, y)),
                // mc as a window manager claims its output: only a window
                // manager with a claim receives windows (D55).
                claim: self.manage_windows.then_some(Some(work)),
                // Other plugins find mc by this tag and show the whole
                // output, the regions with the page, through its node
                // texture, as chora-construct does; it names the scene
                // that shows. The page of every scene has its own tag.
                size: Some([w, h]),
                meta: Some(pane_tag(self.current.as_deref())),
                ..PropsUpdate::default()
            },
        ));
        for id in [TICK, CLOCK, WAIT, VOLUME] {
            commands.push(create(id, NodeKind::Group, NAME, PropsUpdate::default()));
        }
        // A build ends the wait of the old node.
        commands.extend((!self.launched.is_empty()).then(wait));
        commands.extend(self.tick_clocks().into_iter().chain([volume_wait()]));
        commands.push(create(
            "mc-bg",
            NodeKind::Rect,
            NAME,
            PropsUpdate {
                size: Some([w, h]),
                color: Some(self.settings.value.background()),
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

    /// The phase of the one baseline grid: where the baseline of the
    /// margin type stands in a line of the grid. Every baseline of the
    /// page, of the head, the panels, the column and the footer, falls on
    /// a line of the grid plus this.
    pub fn phase(&self) -> f64 {
        let ink = self.ty("margin", None).ink();
        ink.top + ink.height
    }

    /// How far a text whose baseline would stand at `baseline` on the
    /// output moves to stand on the nearest baseline of the grid, in whole
    /// pixels; half a line moves down.
    pub fn to_grid(&self, baseline: f64) -> f64 {
        let b = self.settings.value.scale.baseline;
        let up = (self.phase() - baseline).rem_euclid(b);
        (if up > b / 2.0 { up - b } else { up }).round()
    }

    /// The place `rel` below the top of a block that stands at `top` on the
    /// output, moved so that the baseline of a text of type `ty` that
    /// starts there stands on the grid.
    fn lift(&self, top: f64, rel: f64, ty: &Type) -> f64 {
        let ink = ty.ink();
        rel + self.to_grid(top + rel + ink.top + ink.height)
    }

    /// The top of the head of a scene: its line, moved so that the
    /// baseline of the title stands on the grid.
    fn head_y(&self) -> f64 {
        let top = HEAD * self.settings.value.scale.baseline;
        let ink = self.ty("chapter", None).ink();
        top + self.to_grid(top + ink.top + ink.height)
    }

    /// The bar between the number and the title of a head at `x`: a
    /// hairline from the capitals of the title down to its baseline.
    fn bar(&self, x: f64) -> [f64; 4] {
        let title = self.ty("chapter", None);
        let ink = title.ink();
        [x, self.head_y() + ink.top, 1.0, ink.height]
    }

    /// The ink of a head part: how far its first ink starts after its box
    /// and its last ink ends before the end of it, in pixels.
    fn bearings(&self, style: &str, text: &str) -> [f64; 2] {
        let ty = self.ty(style, None);
        let (family, weight) = (ty.family(), ty.weight());
        let text = ty.props(text).text.unwrap_or_default();
        let em = family.and_then(|f| fonts::bearings(f, weight, &text));
        em.unwrap_or([0.0; 2]).map(|e| e * ty.size)
    }

    /// Where the parts of the page of a scene go on the output.
    fn geometry(&self, name: &str) -> Option<Geometry> {
        self.geometry_of(name, self.scenes.get(name)?)
    }

    /// Where the parts of a page go on the output, with the sides of the
    /// scene `name` beside it. The text stays centered, with room for the
    /// sides, so that it never moves when one shows: centered where it
    /// fits, else beside the sides. The margin column of the right keeps
    /// one gutter at the edge and one beside the text, whatever its width.
    fn geometry_of(&self, name: &str, scene: &Scene) -> Option<Geometry> {
        let output = self.output.as_ref()?;
        let [w, h] = output.size.map(f64::from);
        let config = &self.settings.value;
        let (em, b) = (config.scale.base, config.scale.baseline);
        let page = &config.page;
        let (note, gap) = (page.margin * em, (page.gap * em).floor());
        let cols = f64::from(scene.grid()[0]);
        let want = cols * page.measure * em + (cols - 1.0) * (note + 2.0 * gap);
        let [left, right] = self.side_widths(name);
        let need = |side: f64| if side > 0.0 { side + gap } else { 0.0 };
        let lneed = note + gap + need(left);
        let (x, width) = if right > 0.0 {
            let x = (2.0 * b + lneed).round();
            (x, want.min(w - x - right - 4.0 * b).max(b))
        } else {
            let rneed = note + gap;
            let width = want.min(w - lneed - rneed - 4.0 * b).max(b);
            let centered = ((w - width) / 2.0).round();
            let x = centered
                .min(w - 2.0 * b - rneed - width)
                .max(2.0 * b + lneed)
                .round();
            (x, width)
        };
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
                        transform: Some(translate(x, self.head_y())),
                        ..ty.props(&text)
                    };
                    commands.push(create(
                        &chapter_id(name, part),
                        NodeKind::Text,
                        &panels_id(name),
                        props,
                    ));
                }
                commands.extend(self.head_hit(name));
            }
            Layout::None => {}
        }
        commands
    }

    /// The pointer hit of the head of a page, which opens the page list;
    /// the overview itself has none.
    fn head_hit(&self, name: &str) -> Vec<Command> {
        if name == manage::OVERVIEW {
            return Vec::new();
        }
        let ink = (self.settings.value.colors().get("text").copied()).unwrap_or([0.0; 4]);
        region::hit_nodes(&chapter_id(name, "head"), &panels_id(name), ink).to_vec()
    }

    /// Moves the bar and the title of the head after the number, which is
    /// `width` wide, and sizes the hit of the head to the three. The bar
    /// stands in the middle between the ink of the number and that of
    /// the title.
    fn align_head(
        &self,
        name: &str,
        scene: &Scene,
        geo: &Geometry,
        rect: &dyn Fn(&str) -> Option<[f64; 4]>,
    ) -> Vec<Command> {
        let Some([_, _, width, _]) = rect(&chapter_id(name, "number")) else {
            return Vec::new();
        };
        if !Self::has_chapter(scene) || self.layout(scene) != Layout::Number {
            return Vec::new();
        }
        let title = rect(&chapter_id(name, "title"));
        let space = BAR_SPACE * self.ty("chapter", None).size;
        let words = scene
            .chapter
            .title
            .clone()
            .unwrap_or_else(|| title_of(name));
        let after = self.bearings("chapter_number", &self.number(name))[1];
        let before = self.bearings("chapter", &words)[0];
        let bar = self.bar((geo.text[0] + width - after + space).round());
        let x = bar[0] + bar[2] + space.round() - before;
        let mut commands = vec![
            move_to(chapter_id(name, "bar"), bar[0], bar[1]),
            move_to(chapter_id(name, "title"), x, self.head_y()),
        ];
        commands.extend(self.head_hit_place(name, geo, x, title));
        commands
    }

    /// Sizes the hit of the head to the number, the bar and the title,
    /// whose title now stands at `x` and measures `title`.
    fn head_hit_place(
        &self,
        name: &str,
        geo: &Geometry,
        x: f64,
        title: Option<[f64; 4]>,
    ) -> Vec<Command> {
        let Some([_, _, width, height]) = title.filter(|_| name != manage::OVERVIEW) else {
            return Vec::new();
        };
        let left = geo.text[0];
        let at = [left, self.head_y(), x + width - left, height];
        region::hit_place(
            &chapter_id(name, "head"),
            at,
            left,
            self.ty("chapter", None).ink(),
        )
        .to_vec()
    }

    /// The box of a panel on the screen, without its margin.
    /// As many columns as fit at the least width of a panel; the panels
    /// wrap into new rows, which keep at least the least height. An empty
    /// panel takes no room, unless it says why it is empty: then it takes
    /// one short row for its words, so that every such message shows on a
    /// small output.
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
        if fit >= cols {
            let row = ((h - gap[1] * f64::from(rows - 1)) / f64::from(rows)).max(min_h);
            let shown = f64::from(rows);
            let height = row * shown + gap[1] * (shown - 1.0);
            return Some(scene::place(
                panel.cell,
                [cols, rows],
                [x, y, w, height],
                gap,
            ));
        }
        let cells: Vec<_> = (scene.panels.iter())
            .map(|p| {
                let [x, y, w, rows] = p.cell;
                let rows = match (p.is_empty(), p.empty.is_empty()) {
                    (false, _) => rows,
                    (true, true) => 0,
                    (true, false) => 1,
                };
                [x, y, w, rows]
            })
            .collect();
        let (flowed, flowed_rows) = scene::flow(&cells, fit);
        let at = scene.panels.iter().position(|p| p.id == panel.id)?;
        let [cx, cy, cw, ch] = flowed[at];
        let heights = self.row_heights(scene, &flowed, flowed_rows, (h, gap[1], min_h));
        let [bx, _, bw, _] = scene::place([cx, 0, cw, 1], [fit, 1], [x, y, w, 1.0], gap);
        if ch == 0 {
            let rows = f64::from(panel.cell[3].max(1));
            let by = y + heights.iter().sum::<f64>();
            return Some([bx, by, bw, min_h * rows + gap[1] * (rows - 1.0)]);
        }
        let above: f64 = heights.iter().take(cy as usize).sum();
        let spanned: f64 = heights.iter().skip(cy as usize).take(ch as usize).sum();
        let by = y + above + gap[1] * f64::from(cy);
        Some([bx, by, bw, spanned + gap[1] * f64::from(ch - 1)])
    }

    /// The heights of the rows of a page that wrapped: a row that holds
    /// only panels with their words of emptiness is as high as those
    /// words; the other rows share the rest of the height, and none is
    /// lower than the least height.
    fn row_heights(
        &self,
        scene: &Scene,
        flowed: &[[u32; 4]],
        rows: u32,
        (height, gap, min_h): (f64, f64, f64),
    ) -> Vec<f64> {
        let words = |p: &Panel| p.is_empty() && !p.empty.is_empty();
        let rows_of = |r: u32| {
            scene
                .panels
                .iter()
                .zip(flowed)
                .filter(move |(_, c)| c[3] > 0 && (c[1]..c[1] + c[3]).contains(&r))
                .map(|(p, _)| p)
        };
        let short: Vec<Option<f64>> = (0..rows)
            .map(|r| {
                let mut panels = rows_of(r).peekable();
                panels.peek()?;
                let mut most: f64 = 0.0;
                for p in panels {
                    if !words(p) {
                        return None;
                    }
                    let line = self.ty("body", p.style.as_ref()).line;
                    most = most.max(self.lines_top(p) + 2.0 * line);
                }
                Some(most)
            })
            .collect();
        let taken: f64 = short.iter().flatten().sum();
        let tall = u32::try_from(short.iter().filter(|s| s.is_none()).count()).unwrap_or(1);
        let gaps = gap * f64::from(rows.saturating_sub(1));
        let share = if tall == 0 {
            0.0
        } else {
            ((height - gaps - taken) / f64::from(tall)).max(min_h)
        };
        short.into_iter().map(|s| s.unwrap_or(share)).collect()
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
            let at = self.lift(y, 0.0, &title);
            commands.push(add("title", text(&title, &panel.title, indent, at, true)));
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
                #[allow(clippy::cast_precision_loss, reason = "rows of one screen")]
                let row = top + i as f64 * body.line;
                let at = self.lift(y, row, &body);
                let hit = ([indent, at, w, body.line], body.ink());
                commands.extend(self.row_hit(name, &id, i, hit));
                let words = text(&body, line, indent, at, true);
                commands.push(add(&format!("line-{i}"), words));
                if !time.is_empty() {
                    let at = self.lift(y, row, &note);
                    commands.push(add(&format!("note-{i}"), text(&note, time, 0.0, at, false)));
                }
            }
        } else if !panel.lines.is_empty() {
            if name == manage::OVERVIEW {
                commands.extend(self.overview_hits(name, &id, panel, [indent, top, w, body.line]));
            }
            let props = text(&body, &panel.text(), indent, self.lift(y, top, &body), true);
            commands.push(add("lines", props));
        } else if !panel.empty.is_empty() {
            let quiet = self.quiet(&body);
            let props = text(&quiet, &panel.empty, indent, self.lift(y, top, &body), true);
            commands.push(add("empty", props));
        }
        commands
    }

    /// The hit of each line of the overview.
    fn overview_hits(
        &self,
        name: &str,
        id: &str,
        panel: &Panel,
        [x, top, w, line]: [f64; 4],
    ) -> Vec<Command> {
        let ink = self.ty("body", panel.style.as_ref()).ink();
        let rows = (0..panel.lines.len()).map(|i| {
            #[allow(clippy::cast_precision_loss, reason = "rows of one screen")]
            let y = top + i as f64 * line;
            self.row_hit(name, id, i, ([x, y, w, line], ink))
        });
        rows.flatten().collect()
    }

    /// A type in `muted`: the words of an empty panel stay quiet.
    fn quiet(&self, ty: &Type) -> Type {
        let muted = self.settings.value.colors().get("muted").copied();
        let mut quiet = ty.clone();
        quiet.color = muted.unwrap_or(ty.color);
        quiet
    }

    /// Whether the rows of a scene take the pointer: not those of a scene
    /// with very many panels, which would build slowly.
    fn has_hits(&self, scene: &str) -> bool {
        (self.scenes.get(scene)).is_some_and(|s| s.panels.len() <= MAX_HIT_PANELS)
    }

    /// The hit and the focus mark of row `i` of a panel, at a box of the
    /// panel group: the hit spans the margin and the text.
    fn row_hit(
        &self,
        scene: &str,
        panel: &str,
        row: usize,
        (at, caps): ([f64; 4], Ink),
    ) -> Vec<Command> {
        if !self.has_hits(scene) {
            return Vec::new();
        }
        let [x, y, w, h] = at;
        let ink = (self.settings.value.colors().get("text").copied()).unwrap_or([0.0; 4]);
        let base = format!("{panel}-row-{row}");
        let mut commands = region::hit_nodes(&base, panel, ink).to_vec();
        commands.extend(region::hit_place(&base, [0.0, y, x + w, h], x, caps));
        commands
    }

    /// Moves what the core lays out to its place: the title of the head
    /// after the number, the notes of the panels of a scene flush right to
    /// the margin. One query reads the rects of all texts that show.
    fn align(&mut self, name: &str) -> Result<(), ClientError> {
        let Some(scene) = self.scenes.get(name).cloned() else {
            return Ok(());
        };
        let Some(geo) = self.geometry(name) else {
            return Ok(());
        };
        let rects = self.text_rects()?;
        let rect = |id: &str| rects.get(id).copied();
        let indent = geo.note + geo.gap;
        let mut commands = Vec::new();
        commands.extend(self.align_head(name, &scene, &geo, &|id| rect(id)));
        for panel in &scene.panels {
            let id = panel_id(name, &panel.id);
            // A title that wraps pushes the lines down by its extra lines.
            let mut y = self.lines_top(panel);
            if !panel.title.is_empty()
                && let Some([_, _, _, height]) = rect(&format!("{id}-title"))
            {
                let line = self.ty("heading", panel.title_style.as_ref()).line;
                y += (height - line).max(0.0);
            }
            let top_of_panel = self.place(name, panel).map_or(0.0, |p| p[1]);
            let Some(rows) = panel.columns() else {
                if !panel.lines.is_empty() && y > self.lines_top(panel) {
                    let body = self.ty("body", panel.style.as_ref());
                    let at = self.lift(top_of_panel, y, &body);
                    commands.push(move_to(format!("{id}-lines"), indent, at));
                }
                continue;
            };
            let body = self.ty("body", panel.style.as_ref());
            let note = self.ty("margin", panel.note_style.as_ref());
            let wide = self.place(name, panel).map_or(0.0, |p| p[2] + indent);
            for (i, (time, _)) in rows.iter().enumerate() {
                let Some([_, _, _, height]) = rect(&format!("{id}-line-{i}")) else {
                    break; // below the panel
                };
                let at = self.lift(top_of_panel, y, &body);
                commands.push(move_to(format!("{id}-line-{i}"), indent, at));
                let hit = region::hit_place(
                    &format!("{id}-row-{i}"),
                    [0.0, at, wide, height.max(body.line)],
                    indent,
                    body.ink(),
                );
                commands.extend(hit);
                let note_id = format!("{id}-note-{i}");
                if !time.is_empty()
                    && let Some([_, _, width, _]) = rect(&note_id)
                {
                    let x = (geo.note - width).max(0.0);
                    let at = self.lift(top_of_panel, y, &note);
                    commands.push(move_to(note_id, x, at));
                }
                y += height.max(body.line);
            }
        }
        self.apply(commands)
    }

    /// Scrolls the page of the current scene by `dy`, from its top to its
    /// last panel at the bottom of the text block.
    fn scroll_page(&mut self, node: &str, dy: f64) -> Result<(), ClientError> {
        let over =
            |n: &String| wheel_id(n) == node || node.starts_with(&format!("{}-", panels_id(n)));
        let Some(name) = self.current.clone().filter(over) else {
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

    /// The rects that the core lays out the visible texts of the plugin
    /// at, by node, with one query: each query costs the core a walk
    /// over the scene.
    fn text_rects(&mut self) -> Result<HashMap<String, [f64; 4]>, ClientError> {
        let query = Query {
            selector: format!("owner:{NAME} kind:text"),
            filter: Some(Filter {
                visible: true,
                ..Filter::default()
            }),
        };
        let nodes: Vec<Value> = self.rt.conn.call("scene.query", query)?;
        let rects = nodes.iter().filter_map(|n| {
            let r = n["rects"].as_object()?.values().next()?.as_array()?;
            let v: Vec<f64> = r.iter().filter_map(Value::as_f64).collect();
            Some((n["id"].as_str()?.to_owned(), <[f64; 4]>::try_from(v).ok()?))
        });
        Ok(rects.collect())
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

    /// The effect of the blur of a panel, if it has one and the watchdog
    /// of the core did not drop the effects.
    fn blur_effect(&self, panel: &Panel) -> Option<ShaderEffect> {
        let radius = (panel.blur).filter(|r| *r > 0.0 && self.shaders == Shaders::On)?;
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
        if !config.write.enabled || self.shaders == Shaders::Dropped {
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
            commands.extend(self.keyboard_to(name, window.clone()));
        }
        self.step_to(name);
        commands.push(pane_tag_update(name));
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

    /// The keyboard for a scene that the user shows: to its window, or to
    /// the node of a page, so that the plain keys of the page arrive. An
    /// open chat keeps the keyboard.
    fn keyboard_to(&mut self, name: &str, window: Option<String>) -> Vec<Command> {
        self.focused.clone_from(&window);
        if self.chat == Chat::Open {
            return Vec::new();
        }
        let plain = self.page_keys == keys::PageKeys::Plain;
        let page = (plain && window.is_none()).then(|| Command::FocusNode {
            node: Some(page_id(name)),
        });
        let focus = Command::Focus { surface: window };
        std::iter::once(focus).chain(page).collect()
    }

    /// The keyboard for the scene that shows, after the chat closed: the
    /// focus that `keyboard_to` held back while the chat had the keyboard,
    /// or the docked app that had it.
    fn keyboard_back(&mut self) -> Vec<Command> {
        let Some(name) = self.current.clone() else {
            return Vec::new();
        };
        let docked = (self.focused.clone()).filter(|f| self.docks.values().any(|d| d == f));
        let window = docked.or_else(|| self.windows.get(&name).cloned());
        self.keyboard_to(&name, window)
    }

    /// Makes `name` the current scene and puts the one before on the trail.
    fn step_to(&mut self, name: &str) {
        if self.current.as_deref() == Some(name) {
            return;
        }
        self.previous = self.current.replace(name.into());
        self.trail.extend(self.previous.clone());
        let extra = self.trail.len().saturating_sub(TRAIL);
        self.trail.drain(..extra);
    }

    /// Starts the app of a scene, whose window then fills its text block.
    /// A command that names a content of kind app runs that content.
    fn launch_scene(&mut self, name: &str, command: &str) -> Result<(), ClientError> {
        let config = &self.settings.value;
        let command = match config.contents.get(command) {
            Some(app) if app.kind == Kind::App => region::command(&app.command, config.colors()),
            _ => command.into(),
        };
        let mint = self.rt.conn.activation_token();
        let Some((token, pid)) = launch(&command, mint.as_deref()) else {
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
            .partition(|(_, pid)| !pid.is_some_and(chora_kit::alive));
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
                self.leave(&name, true);
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
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            // Another window manager took it first; the new scene goes.
            eprintln!("chora-mc: {surface}: {err}");
            if new {
                self.scenes.remove(&name);
                self.transient.remove(&name);
                self.appeared.retain(|n| *n != name);
            }
            return Ok(());
        }
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
        // Given to mc (D55) or free: the app id names its scene.
        let nodes: Vec<Value> =
            (self.rt.conn).call("scene.query", Query::new(format!("id:{surface}")))?;
        let app_id = nodes.first().and_then(|n| n["surface"]["app_id"].as_str());
        self.adopt(surface, app_id, None)
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
        self.leave(&name, true);
        if !self.transient.contains(&name) {
            return Ok(());
        }
        // The scene goes when the app ended too; until then it waits.
        let pid = self.pids.get(&name).copied().flatten();
        if pid.is_some_and(chora_kit::alive) {
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

    /// A drag (D55): the press takes the window of mc that asked for a
    /// move, else the window of the scene that shows, and a release on the claim of another window manager hands the
    /// window to that one. Elsewhere the window stays; mc moves none.
    fn drag(&mut self, binding: &str, state: PressState, point: [f64; 2], on: Option<String>) {
        if !matches!(self.rt.action(binding), Some(Action::Drag)) {
            return;
        }
        if state == PressState::Pressed {
            let shown = || self.windows.get(self.current.as_deref()?).cloned();
            self.dragged = on
                .filter(|s| self.windows.values().any(|w| w == s))
                .or_else(shown);
            return;
        }
        let Some(surface) = self.dragged.take() else {
            return;
        };
        let given = match self.rt.conn.claimant_at(point) {
            Ok(Some(to)) if to != NAME => self.give(chora_kit::give::Give {
                surface: surface.clone(),
                to,
                grab: None,
            }),
            Ok(_) => return,
            Err(err) => Err(err),
        };
        if let Err(err) = given {
            eprintln!("chora-mc: {surface} stays: {}", err.message);
        }
    }

    pub fn act(&mut self, binding: &str) {
        if let Some(action) = self.rt.action(binding).cloned() {
            self.perform(action);
        }
    }

    /// Runs a shared script as an app, or says that it is not installed.
    fn run_shared(&mut self, script: &str, args: &[&str]) {
        let Some(command) = chora_kit::shared::command(script, args) else {
            self.notice(script, "The shared script is not installed.");
            return;
        };
        let mint = self.rt.conn.activation_token();
        let _ = launch(&command, mint.as_deref());
    }

    /// Runs the actions of the system: the locker, the power menu, the
    /// screenshot and the volume.
    fn system(&mut self, action: &Action) {
        match action {
            Action::Lock => self.run_shared("power", &["lock"]),
            Action::Power => self.run_shared("power", &[]),
            Action::Screenshot => self.run_shared("screenshot", &[]),
            Action::Volume(change) => self.volume.change(*change, true),
            _ => {}
        }
    }

    pub fn perform(&mut self, action: Action) {
        self.system(&action);
        self.turn(action);
    }

    /// Does an action of a binding that is not one of the system.
    fn turn(&mut self, action: Action) {
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
                let mint = self.rt.conn.activation_token();
                let _ = launch(&chora_kit::shared::launcher(&command), mint.as_deref());
                return;
            }
            Action::Give(to) => {
                if let Err(err) = chora_kit::give::focused(self, &to) {
                    eprintln!("chora-mc: give to {to}: {}", err.message);
                }
                return;
            }
            Action::Drag
            | Action::Lock
            | Action::Power
            | Action::Screenshot
            | Action::Volume(_) => return,
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
            Action::Chat => {
                self.chat_with(None);
                return;
            }
            Action::Keys => {
                self.keys = self.keys.flip();
                if let Err(err) = self.render() {
                    eprintln!("chora-mc: keys: {err}");
                }
                return;
            }
        };
        if let Err(err) = self.show(&next, true) {
            eprintln!("chora-mc: {next}: {}", err.message);
        }
    }
}

/// Whether an action is grabbed only with the claim of a window manager.
fn needs_claim(action: &Action) -> bool {
    matches!(action, Action::Drag | Action::Volume(_))
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
fn volume_wait() -> Command {
    Command::Animate {
        id: VOLUME.into(),
        props: PropsUpdate::default(),
        duration: VOLUME_MS,
        easing: Easing::Linear,
    }
}

fn wait() -> Command {
    Command::Animate {
        id: WAIT.into(),
        props: PropsUpdate::default(),
        duration: WAIT_MS,
        easing: Easing::Linear,
    }
}

#[cfg(test)]
mod tests {
    use chora_scene::{Command, NodeKind, PropsUpdate};

    use super::shader_file;

    /// A shader goes to the runtime directory only, never to the shared
    /// temporary directory.
    #[test]
    fn a_shader_needs_the_runtime_dir() {
        assert!(shader_file(None, "d", "x.frag", "void main() {}").is_err());
        assert!(shader_file(Some("rel".into()), "d", "x.frag", "void main() {}").is_err());
        let dir = std::env::temp_dir().join(format!("chora-mc-shader-{}", std::process::id()));
        let path = shader_file(Some(dir.clone().into()), "d", "x.frag", "void main() {}").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "void main() {}");
        std::fs::remove_dir_all(dir).unwrap();
    }

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

    /// The head of a page opens the page list: it takes the pointer like
    /// every other item, and the overview itself has no such hit.
    #[test]
    fn a_click_on_the_head_of_a_page_shows_the_overview() {
        let (mut mc, model) = crate::testing::mc("head-hit", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let hit = "mc-s-agenda-chapter-head-hit";
        assert!(
            crate::testing::has(&model, hit),
            "the head takes the pointer"
        );
        assert!(mc.click_hit(hit));
        assert_eq!(mc.current.as_deref(), Some("overview"));
        assert!(!crate::testing::has(
            &model,
            "mc-s-overview-chapter-head-hit"
        ));
    }

    /// A page without data says what it is and why it is empty, in
    /// `muted`; with a line the words go.
    #[test]
    fn an_empty_panel_says_why_in_muted() {
        use chora_kit::runtime::Plugin as _;
        use serde_json::json;
        let (mut mc, model) = crate::testing::mc("empty-panel", [1280, 800]);
        mc.settings.value.write.enabled = false;
        mc.show("agenda", false).unwrap();
        let id = "mc-s-agenda-panels-today-empty";
        assert_eq!(
            crate::testing::text(&model, id).as_deref(),
            Some("Nothing planned today.\nSay “Atlas” or press Super+a.")
        );
        let muted = mc.settings.value.colors()["muted"];
        let color = model.lock().unwrap().scene.node(id).unwrap().props.color;
        assert_eq!(color, Some(muted));
        let lines = json!({"scene": "agenda", "panel": "today", "lines": ["09:00\tStand-up"]});
        mc.method("panel.set", lines).unwrap();
        assert!(!crate::testing::has(&model, id), "a line takes its place");
    }

    /// With no entry left today the footer points to the next page, and a
    /// click on it turns the page; an entry takes its place.
    #[test]
    fn the_footer_points_to_the_next_page_when_no_entry_is_next() {
        use chora_kit::runtime::Plugin as _;
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("next-page", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let next = |mc: &super::Mc| {
            let plan = mc.items("agenda");
            let (items, _) = &plan[&crate::region::Region::Footer];
            items
                .iter()
                .find(|i| i.id.ends_with("-next"))
                .map(|i| (i.props.text.clone().unwrap(), i.hit.clone()))
        };
        let (text, hit) = next(&mc).expect("a line in the footer");
        assert_eq!(text, "→ 2  The Morning");
        assert_eq!(hit.as_deref(), Some("key:Super+Right"));
        mc.method(
            "content.set",
            json!({"content": "next", "lines": ["11:30  Coffee"]}),
        )
        .unwrap();
        assert_eq!(next(&mc).unwrap().0, "11:30  Coffee");
    }

    /// The footer also points to the previous page, and a click on it
    /// turns the page back.
    #[test]
    fn the_footer_points_to_the_previous_page_and_a_click_goes_back() {
        let (mut mc, _) = crate::testing::mc("prev-page", [1280, 800]);
        mc.show("morning", false).unwrap();
        let plan = mc.items("morning");
        let (items, _) = &plan[&crate::region::Region::Footer];
        let prev = items
            .iter()
            .find(|i| i.id.ends_with("-prev"))
            .expect("a line to the previous page");
        assert_eq!(prev.props.text.as_deref(), Some("← 1  Agenda"));
        assert_eq!(prev.hit.as_deref(), Some("key:Super+Left"));
        assert!(prev.slot < 0, "the previous page stands at the left");
    }

    fn press_key(mc: &mut super::Mc, key: &str, modifiers: &[&str]) {
        use chora_kit::runtime::Plugin as _;
        mc.event(chora_scene::Event::NodeKey {
            node: super::page_id("agenda"),
            key: key.into(),
            code: 0,
            state: chora_scene::ipc::PressState::Pressed,
            modifiers: modifiers.iter().map(|m| (*m).to_owned()).collect(),
            text: None,
        })
        .unwrap();
    }

    /// On a page the arrows alone turn it, Escape goes back along the
    /// pages that showed and first closes the full key list; a key with a
    /// modifier is not a key of the page.
    #[test]
    fn plain_arrows_turn_the_page_and_escape_goes_back_one_step() {
        let (mut mc, _model) = crate::testing::mc("plain-keys", [1280, 800]);
        mc.show("agenda", true).unwrap();
        press_key(&mut mc, "Right", &[]);
        assert_eq!(mc.current.as_deref(), Some("morning"));
        press_key(&mut mc, "Right", &[]);
        assert_eq!(mc.current.as_deref(), Some("agents"));
        press_key(&mut mc, "Left", &[]);
        assert_eq!(mc.current.as_deref(), Some("morning"));
        press_key(&mut mc, "Right", &["Shift"]);
        assert_eq!(mc.current.as_deref(), Some("morning"));
        press_key(&mut mc, "Escape", &[]);
        assert_eq!(mc.current.as_deref(), Some("agents"));
        press_key(&mut mc, "Escape", &[]);
        assert_eq!(mc.current.as_deref(), Some("morning"));
        mc.keys = crate::keys::List::All;
        press_key(&mut mc, "Escape", &[]);
        assert_eq!(mc.keys, crate::keys::List::Six);
        assert_eq!(mc.current.as_deref(), Some("morning"));
        press_key(&mut mc, "Escape", &[]);
        press_key(&mut mc, "Escape", &[]);
        press_key(&mut mc, "Escape", &[]);
        assert_eq!(mc.current.as_deref(), Some("agenda"));
    }

    /// A page that the user shows takes the keyboard with its node, so
    /// that its plain keys arrive.
    #[test]
    fn a_page_that_the_user_shows_takes_the_keyboard() {
        let (mut mc, model) = crate::testing::mc("page-keyboard", [1280, 800]);
        mc.show("morning", true).unwrap();
        let focused = model
            .lock()
            .unwrap()
            .scene
            .focused_node()
            .map(str::to_owned);
        assert_eq!(focused, Some(super::page_id("morning")));
    }

    /// Beside other window managers a page takes no keyboard, since the
    /// node with the keyboard would draw their new windows to mc, and
    /// the key list names the bindings.
    #[test]
    fn a_page_beside_other_window_managers_takes_no_keyboard() {
        let (mut mc, model) = crate::testing::mc("page-no-keyboard", [1280, 800]);
        mc.page_keys = crate::keys::PageKeys::Bound;
        mc.show("morning", true).unwrap();
        mc.take_free_keyboard().unwrap();
        assert_eq!(model.lock().unwrap().scene.focused_node(), None);
        let keys: Vec<String> = right_items(&mc)
            .into_iter()
            .filter_map(|item| item.props.text)
            .collect();
        assert!(
            keys.contains(&"Super+\u{2009}←\u{2009}→".to_owned()),
            "{keys:?}"
        );
    }

    fn right_items(mc: &super::Mc) -> Vec<crate::region::Item> {
        let name = mc.current.clone().unwrap();
        let mut plan = mc.items(&name);
        plan.remove(&crate::region::Region::Right)
            .map(|(items, _)| items)
            .unwrap_or_default()
    }

    /// A note is one line: its time in the margin style, its words in the
    /// reading face at a hanging indent. No blank line between two notes.
    #[test]
    fn a_notification_is_one_line_with_its_time_in_the_margin_style() {
        use chora_kit::runtime::Plugin as _;
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("notes", [1280, 800]);
        mc.show("agenda", false).unwrap();
        for title in ["One", "Two", "Three", "Four"] {
            mc.method("notify.push", json!({"title": title})).unwrap();
        }
        let items = right_items(&mc);
        let words: Vec<_> = (items.iter())
            .filter(|i| i.id.contains("-notify-") && !i.id.ends_with("-title"))
            .collect();
        assert_eq!(words.len(), 6, "three notes of a time and words each");
        assert_eq!(words[0].props.family.as_deref(), Some("Inter"));
        assert_eq!(words[0].props.text.as_deref().map(str::len), Some(5));
        assert_eq!(words[1].props.text.as_deref(), Some("Four"));
        assert_eq!(words[1].put, crate::region::Put::Row);
        assert!(words[1].x > 40.0, "a hanging indent: {}", words[1].x);
        assert!(words[2].put == crate::region::Put::Stack, "the next note");
        let tall = |i: &&crate::region::Item| i.props.font_size.unwrap_or(0.0) < 15.0;
        assert!(words.iter().step_by(2).all(tall), "times at 14.25");
    }

    /// The empty words stay quiet and the folded head counts nothing.
    #[test]
    fn nothing_new_is_muted_and_the_folded_head_counts_zero() {
        let (mut mc, _) = crate::testing::mc("quiet", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let muted = mc.settings.value.colors()["muted"];
        let items = right_items(&mc);
        let empty = items.iter().find(|i| i.id.ends_with("-notify")).unwrap();
        assert_eq!(empty.props.text.as_deref(), Some("Nothing new."));
        assert_eq!(empty.props.color, Some(muted));
        mc.chat = crate::Chat::Open;
        let items = right_items(&mc);
        let head = items
            .iter()
            .find(|i| i.id.ends_with("-notify-title"))
            .unwrap();
        assert_eq!(head.props.text.as_deref(), Some("NOTIFICATIONS  0"));
    }

    /// An open chat keeps the keyboard; when it closes the page that
    /// shows gets it, to its window or its node.
    #[test]
    fn the_keyboard_waits_for_the_chat_and_goes_to_the_page_after() {
        let (mut mc, _) = crate::testing::mc("keyboard-chat", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.chat = crate::Chat::Open;
        assert_eq!(mc.keyboard_to("agenda", None).len(), 0);
        mc.chat = crate::Chat::Closed;
        assert_eq!(mc.keyboard_back().len(), 2);
        mc.windows.insert("agenda".into(), "s1".into());
        let back = mc.keyboard_back();
        assert!(matches!(
            back.first(),
            Some(Command::Focus { surface: Some(s) }) if s == "s1"
        ));
    }

    /// With the voice HUD to the host the first line of the column stays
    /// free for its state word, and the helpers are mc's own rows.
    #[test]
    fn the_column_keeps_the_state_line_free_and_draws_the_helpers() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("state-line", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let items = right_items(&mc);
        assert!(items[0].id.ends_with("-state"), "{}", items[0].id);
        mc.follow(
            "voice.state",
            "voice",
            json!({"name": "Atlas", "state": "working", "task": "Tidy notes"}),
        )
        .unwrap();
        let items = right_items(&mc);
        let name = items.iter().find(|i| i.id.ends_with("-0-name")).unwrap();
        assert_eq!(name.props.text.as_deref(), Some("Atlas"));
    }

    /// The open chat is the box of the HUD host, whose header says that no
    /// helper is set up: mc draws no line of its own for it.
    #[test]
    fn the_open_chat_leaves_the_words_of_no_helper_to_the_host() {
        let (mut mc, _) = crate::testing::mc("chat-none", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.chat = crate::Chat::Open;
        let items = right_items(&mc);
        assert!(items.iter().any(|i| i.id.ends_with("-chat-slot")));
        assert_eq!(text_of(&items, "-chat-none"), None);
    }

    fn text_of(items: &[crate::region::Item], suffix: &str) -> Option<String> {
        let found = items.iter().find(|i| i.id.ends_with(suffix))?;
        found.props.text.clone()
    }

    /// The column has a door for each action that has a key, and the
    /// volume door says what the mixer read.
    #[test]
    fn the_column_has_a_door_for_each_action() {
        use chora_kit::{
            volume::{Reading, Volume},
            words::{Language, Word},
        };
        let (mut mc, _) = crate::testing::mc("doors", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let items = right_items(&mc);
        let doors: Vec<&str> = (items.iter())
            .filter_map(|i| i.hit.as_deref())
            .filter(|hit| hit.starts_with("door:"))
            .collect();
        assert_eq!(
            doors,
            [
                "door:programs",
                "door:lock",
                "door:power",
                "door:screenshot",
                "door:volume"
            ]
        );
        mc.volume_reading = Some(Reading::Level(Volume {
            percent: 40,
            muted: false,
        }));
        let title = Word::Volume.text(Language::of_session());
        let volume = text_of(&right_items(&mc), "-doors-volume");
        assert_eq!(volume, Some(format!("{title} 40%")));
    }

    /// A chat that cannot open says so in a red notice, not only in a log.
    #[test]
    fn a_chat_that_does_not_open_shows_a_notice() {
        let (mut mc, model) = crate::testing::mc("chat-refused", [1280, 800]);
        mc.show("agenda", false).unwrap();
        model.lock().unwrap().refuse_calls = true;
        mc.chat_with(None);
        let urgent = mc.texts["urgent"].1.join(" ");
        assert!(urgent.contains("The chat does not open"), "{urgent}");
        let notes = &mc.texts["notify"].1;
        let said = notes
            .iter()
            .filter(|line| line.contains("The chat does not open"));
        assert_eq!(said.count(), 1, "{notes:?}");
    }

    /// The door of the chat has a state word: no helper, a helper, no
    /// helper again when voice ends.
    #[test]
    fn the_state_word_of_the_helper_door_follows_voice() {
        use chora_kit::words::{Language, Word};
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("helper-door", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let none = Word::NoHelperDoor.text(Language::of_session());
        let state = text_of(&right_items(&mc), "-chat-state");
        assert_eq!(state.as_deref(), Some(none));
        mc.follow(
            "voice.state",
            "voice",
            json!({"name": "Atlas", "state": "working"}),
        )
        .unwrap();
        let state = text_of(&right_items(&mc), "-chat-state");
        assert_eq!(state.as_deref(), Some("Working"));
        let gone = chora_scene::Event::ClientDisconnected {
            name: "voice".into(),
        };
        chora_kit::runtime::Plugin::event(&mut mc, gone).unwrap();
        let state = text_of(&right_items(&mc), "-chat-state");
        assert_eq!(state.as_deref(), Some(none));
    }

    /// The voice agent adds `seq`, `epoch`, `chat` and the steps to the
    /// state of a helper, for the host: mc still shows the helper.
    #[test]
    fn a_state_of_a_helper_with_the_fields_of_the_host_shows_the_helper() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("host-fields", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let state = json!({"name": "Atlas", "state": "working", "task": "Tidy",
            "seq": 4, "epoch": 2, "chat": true, "step": 1, "steps": 2});
        mc.follow("voice.state", "voice", state).unwrap();
        let items = right_items(&mc);
        assert!(
            items.iter().any(|i| i.id.ends_with("-0-name")),
            "a row for Atlas"
        );
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

    /// mc claims the work area of its output, so that other window managers can give it
    /// windows (D55), when it manages windows; else it claims nothing.
    #[test]
    fn mc_claims_its_output_only_as_a_window_manager() {
        let (mut mc, model) = crate::testing::mc("output-claim", [1920, 1080]);
        assert_eq!(
            model
                .lock()
                .unwrap()
                .scene
                .node(super::NAME)
                .unwrap()
                .props
                .claim,
            None
        );
        mc.manage_windows = true;
        mc.build().unwrap();
        let claim = model
            .lock()
            .unwrap()
            .scene
            .node(super::NAME)
            .unwrap()
            .props
            .claim;
        let [x, y, w, h] = claim.unwrap();
        assert_eq!([x, w], [0.0, 1920.0]);
        assert!(
            y > 0.0 && y + h < 1080.0,
            "below the header, above the footer"
        );
    }

    /// The claim of the indicator of the voice agent goes out at the
    /// start, again after the agent connected, and with its first event
    /// after a failed claim; then no more.
    #[test]
    fn mc_claims_the_indicator_of_voice_once_per_connection() {
        let (mut mc, model) = crate::testing::mc("claim", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Own;
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

    /// With `voice_hud: host` the host draws the voice HUD and the
    /// question: mc shows neither the state nor the sentence of the voice
    /// agent nor its question, and still shows its helpers.
    #[test]
    fn voice_hud_host_draws_no_voice_status() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("host-status", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Host;
        mc.follow("voice.state", "voice", json!({"state": "listening"}))
            .unwrap();
        mc.follow(
            "voice.say",
            "voice",
            json!({"text": "Hello there", "duration_ms": 0}),
        )
        .unwrap();
        let ask = json!({"id": "1", "prompt": "Atlas wants to write notes.txt.",
            "physical": true, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": true});
        mc.follow("voice.confirm", "voice", ask).unwrap();
        mc.follow(
            "voice.state",
            "voice",
            json!({"name": "Atlas", "state": "working"}),
        )
        .unwrap();
        assert!(mc.hud.state.is_empty() && mc.hud.text.is_empty());
        assert!(!mc.agents.iter().any(|a| a.name == "voice"));
        assert!(mc.confirming.is_none() && mc.solo.is_none());
        assert!(mc.agents.iter().any(|a| a.name == "Atlas"));
    }

    /// With `voice_hud: host` mc never claims the indicator of voice.
    #[test]
    fn voice_hud_host_claims_no_indicator() {
        let (mut mc, model) = crate::testing::mc("host-claim", [1920, 1080]);
        mc.settings.value.voice_hud = crate::config::VoiceHud::Host;
        mc.claim_indicator(false);
        mc.follow("voice.state", "voice", serde_json::json!({}))
            .unwrap();
        let connected = chora_scene::Event::ClientConnected {
            name: "voice".into(),
        };
        chora_kit::runtime::Plugin::event(&mut mc, connected).unwrap();
        assert!(
            !model
                .lock()
                .unwrap()
                .calls
                .iter()
                .any(|m| m == "indicator.claim")
        );
    }

    /// mc names its theme to the HUD host when the host says that it
    /// listens and when the settings change.
    #[test]
    fn mc_names_its_theme_to_the_hud_host() {
        let (mut mc, model) = crate::testing::mc("theme", [1920, 1080]);
        mc.settings.value.theme = crate::config::Theme::Light;
        let ready = |client: &str| chora_scene::Event::Emitted {
            event: "assistant.ready".into(),
            client: client.into(),
            data: serde_json::json!({}),
        };
        chora_kit::runtime::Plugin::event(&mut mc, ready("evil")).unwrap();
        assert_eq!(model.lock().unwrap().emits.last(), None, "only the host");
        chora_kit::runtime::Plugin::event(&mut mc, ready("assistant")).unwrap();
        let last = || model.lock().unwrap().emits.last().cloned();
        assert_eq!(last(), Some(chora_kit::hud::theme_event("mc", false)));
        mc.settings.value.theme = crate::config::Theme::Dark;
        chora_kit::runtime::Plugin::settings_changed(&mut mc).unwrap();
        assert_eq!(last(), Some(chora_kit::hud::theme_event("mc", true)));
    }

    use serde_json::{Value, json};

    fn chat_event(client: &str, open: bool, output: &str) -> chora_scene::Event {
        chora_scene::Event::Emitted {
            event: "assistant.chat".into(),
            client: client.into(),
            data: json!({"open": open, "name": "Atlas", "output": output}),
        }
    }

    fn key_items(mc: &super::Mc, scene: &str) -> usize {
        let items = mc.items(scene).into_values().flat_map(|(items, _)| items);
        items.filter(|i| i.id.contains("-keys-")).count()
    }

    /// Another output or another client than the host changes nothing.
    #[test]
    fn only_the_host_opens_the_chat_on_the_output_of_mc() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, _) = crate::testing::mc("chat-other", [1920, 1080]);
        let first = mc.settings.value.default_scene.clone();
        mc.show(&first, false).unwrap();
        mc.event(chat_event("evil", true, "test")).unwrap();
        mc.event(chat_event("assistant", true, "other")).unwrap();
        assert_eq!(mc.chat_slot(), None);
        assert!(key_items(&mc, &first) > 0, "the key list stays");
    }

    /// `assistant.chat` of the host opens the chat in the margin column of
    /// mc and mc emits its box as `mc.chat.slot`: the key list goes, and
    /// the page keeps its place; a close brings the key list back.
    #[test]
    fn an_open_chat_takes_the_margin_column_and_emits_its_box() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-column", [1920, 1080]);
        let first = mc.settings.value.default_scene.clone();
        mc.show(&first, false).unwrap();
        let text_block = mc.geometry(&first).unwrap().text;
        mc.event(chat_event("assistant", true, "test")).unwrap();
        let rect = mc.chat_slot().expect("the chat has a box");
        let column = mc.boxes_of(&first)[&crate::region::Region::Right];
        assert_eq!((rect[0], rect[2]), (column[0], column[2]), "the column");
        assert!(rect[1] > column[1] && rect[1] + rect[3] <= column[1] + column[3]);
        assert_eq!(key_items(&mc, &first), 0, "no key list beside the chat");
        assert_eq!(mc.geometry(&first).unwrap().text, text_block, "the page");
        let emit = |rect: Value| json!({"event": "mc.chat.slot", "data": {"output": "test", "rect": rect}});
        assert_eq!(model.lock().unwrap().emits.last(), Some(&emit(json!(rect))));
        mc.event(chat_event("assistant", false, "test")).unwrap();
        assert_eq!(mc.chat_slot(), None);
        assert!(key_items(&mc, &first) > 0, "the key list is back");
        assert_eq!(model.lock().unwrap().emits.last(), Some(&emit(Value::Null)));
    }

    /// A chat that moves to another output stays open and keeps the
    /// keyboard: mc hands nothing back on its own output, neither when
    /// the chat leaves nor when it closes there (review mc r4, C). The
    /// host moves an open chat only when its output goes.
    #[test]
    fn a_chat_that_moves_to_another_output_keeps_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-moves", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.event(chat_event("assistant", true, "test")).unwrap();
        let before = model.lock().unwrap().focuses.len();
        mc.event(chat_event("assistant", true, "other")).unwrap();
        mc.event(chat_event("assistant", false, "other")).unwrap();
        assert_eq!(model.lock().unwrap().focuses.len(), before);
    }

    /// A window that the user clicked while the chat was open keeps the
    /// keyboard when the chat closes.
    #[test]
    fn a_window_clicked_under_the_chat_keeps_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-click", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.event(chat_event("assistant", true, "test")).unwrap();
        mc.event(chora_scene::Event::FocusChanged {
            surface: Some("s9".into()),
        })
        .unwrap();
        let before = model.lock().unwrap().focuses.len();
        mc.event(chat_event("assistant", false, "test")).unwrap();
        assert_eq!(model.lock().unwrap().focuses.len(), before);
    }

    /// A window that the user clicked while the chat was open and that
    /// closed leaves the keyboard to nobody: the page gets it back when
    /// the chat closes.
    #[test]
    fn a_window_that_closes_under_the_chat_gives_the_keyboard_back() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-click-close", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.event(chat_event("assistant", true, "test")).unwrap();
        for surface in [Some("s9".to_owned()), None] {
            mc.event(chora_scene::Event::FocusChanged { surface })
                .unwrap();
        }
        let before = model.lock().unwrap().focuses.len();
        mc.event(chat_event("assistant", false, "test")).unwrap();
        assert!(model.lock().unwrap().focuses.len() > before);
    }

    /// A docked app that had the keyboard before the chat opened gets it
    /// back when the chat closes.
    #[test]
    fn a_docked_app_gets_the_keyboard_back_when_the_chat_closes() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-dock", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.docks.insert("shell".into(), "dock1".into());
        mc.focused = Some("dock1".into());
        mc.event(chat_event("assistant", true, "test")).unwrap();
        mc.event(chat_event("assistant", false, "test")).unwrap();
        assert_eq!(
            model.lock().unwrap().focuses.last(),
            Some(&Some("dock1".to_owned()))
        );
    }

    /// A page that is longer than the output scrolls (the mission scene
    /// with its data); every other panel of a built-in scene stands inside
    /// the text block at 1280 by 800, so that the words of an empty panel
    /// show.
    #[test]
    fn every_panel_of_a_scene_is_placed_at_1280_by_800() {
        let (mut mc, _model) = crate::testing::mc("panels-1280", [1280, 800]);
        for (name, _) in crate::scene::BUILT_IN {
            mc.show(name, false).unwrap();
            let scene = mc.scenes.get(name).unwrap().clone();
            let [tx, ty, tw, th] = mc.geometry(name).unwrap().text;
            for panel in &scene.panels {
                let [x, y, w, h] = mc.place(name, panel).expect("a box");
                assert!(w > 0.0 && h > 0.0, "{name}: panel {} has no size", panel.id);
                let inside = x >= tx && x + w <= tx + tw + 0.5 && y + h <= ty + th + 0.5;
                assert!(
                    inside || name == "mission",
                    "{name}: panel {} at {:?} is not inside the text block",
                    panel.id,
                    [x, y, w, h]
                );
            }
        }
    }

    /// The host is told again when the output of the box changes, and not
    /// again while nothing changed.
    #[test]
    fn the_slot_is_sent_again_for_another_output_only() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-slot-output", [1920, 1080]);
        let first = mc.settings.value.default_scene.clone();
        mc.show(&first, false).unwrap();
        mc.event(chat_event("assistant", true, "test")).unwrap();
        let sent = || model.lock().unwrap().emits.len();
        let once = sent();
        mc.send_slot().unwrap();
        assert_eq!(sent(), once, "nothing changed");
        mc.output.as_mut().unwrap().name = "other".into();
        mc.send_slot().unwrap();
        assert_eq!(sent(), once + 1, "the output changed");
        assert_eq!(
            model.lock().unwrap().emits.last().unwrap()["data"]["output"],
            "other"
        );
    }

    /// Only the voice agent sets the agent `voice`, by its events or by
    /// agent.set; a client named `voice.evil` or any other caller cannot.
    #[test]
    fn only_voice_sets_the_voice_agent() {
        let call = |caller: &str, method: &str, name: &str| chora_scene::Event::MethodCalled {
            call: 3,
            caller: caller.into(),
            method: method.into(),
            params: serde_json::json!({"name": name, "state": "listening"}),
            confined: false,
            connection: 1,
        };
        assert!(super::spoofs_voice(&call("evil", "agent.set", "voice")).is_some());
        assert!(super::spoofs_voice(&call("voice.evil", "agent.remove", "voice")).is_some());
        assert!(super::spoofs_voice(&call("voice", "agent.set", "voice")).is_none());
        assert!(super::spoofs_voice(&call("evil", "agent.set", "ada")).is_none());
        let (mut mc, _) = crate::testing::mc("spoof", [1920, 1080]);
        let emitted = |client: &str| chora_scene::Event::Emitted {
            event: format!("{client}.state"),
            client: client.into(),
            data: serde_json::json!({"name": "voice", "state": "listening"}),
        };
        chora_kit::runtime::Plugin::event(&mut mc, emitted("voice.evil")).unwrap();
        assert!(mc.agents.is_empty(), "a client voice.evil set the agent");
        chora_kit::runtime::Plugin::event(&mut mc, emitted("voice")).unwrap();
        assert_eq!(mc.agents.len(), 1);
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
                "region.solo",
                "agent.set",
                "notify.push",
                "window.give"
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

    /// The number of effects of the first panel of a scene in the model.
    fn panel_effects(mc: &super::Mc, model: &crate::testing::Model, scene: &str) -> usize {
        let panel = &mc.scenes[scene].panels[0].id;
        let id = super::panel_id(scene, panel);
        let core = model.lock().unwrap();
        core.scene.node(&id).unwrap().props.effects.len()
    }

    /// The write effect costs a shader pass per panel, so it goes when
    /// its animation ends: a page that rests has no effects.
    #[test]
    fn the_write_effect_goes_when_its_animation_ends() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("write-ends", [1920, 1080]);
        mc.shader = "/write.frag".into();
        mc.show("agenda", true).unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 1, "it writes");
        let panel = mc.scenes["agenda"].panels[0].id.clone();
        mc.event(chora_scene::Event::AnimationDone {
            node: Some(super::panel_id("agenda", &panel)),
            output: None,
        })
        .unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 0);
    }

    /// The panels that one transaction writes end in the same frame:
    /// the first end removes the effects of all of them at once, and a
    /// scene of 300 panels answers with one transaction, not with 300.
    #[test]
    fn the_panels_of_one_show_lose_the_write_effect_together() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("write-batch", [1920, 1080]);
        mc.shader = "/write.frag".into();
        mc.show("mission", true).unwrap();
        let ids: Vec<String> = (mc.scenes["mission"].panels.iter())
            .map(|p| super::panel_id("mission", &p.id))
            .collect();
        assert!(ids.len() > 1);
        let effects = |id: &String| {
            let core = model.lock().unwrap();
            core.scene.node(id).unwrap().props.effects.len()
        };
        assert_eq!(
            ids.iter().map(effects).sum::<usize>(),
            ids.len(),
            "all write"
        );
        mc.event(chora_scene::Event::AnimationDone {
            node: Some(ids[0].clone()),
            output: None,
        })
        .unwrap();
        assert_eq!(ids.iter().map(effects).sum::<usize>(), 0);
        assert!(mc.writing.is_empty());
    }

    /// After the watchdog of the core dropped the effects of mc, a scene
    /// shows without the write effect until the settings change.
    #[test]
    fn dropped_effects_stay_off_until_the_settings_change() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("dropped", [1920, 1080]);
        mc.shader = "/write.frag".into();
        assert!(super::EVENTS.contains(&"effects.dropped"));
        mc.event(chora_scene::Event::EffectsDropped {
            client: super::NAME.into(),
        })
        .unwrap();
        mc.show("agenda", true).unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 0, "no shader");
        mc.event(chora_scene::Event::EffectsDropped {
            client: "other".into(),
        })
        .unwrap();
        mc.settings_changed().unwrap();
        mc.show("morning", true).unwrap();
        assert_eq!(panel_effects(&mc, &model, "morning"), 1, "it writes again");
    }

    /// An end of a write animation that mc answers before it hears of
    /// the drop puts the blur back after the core removed it. The drop
    /// clears the effects of the panels once more, and the change of the
    /// settings that allows shaders again brings the blur back, also
    /// when only the theme changed.
    #[test]
    fn a_drop_clears_the_blur_and_the_settings_bring_it_back() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("blur", [1920, 1080]);
        mc.blur = "/blur.frag".into();
        mc.scenes.get_mut("agenda").unwrap().panels[0].blur = Some(4.0);
        mc.build().unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 1, "it blurs");
        mc.event(chora_scene::Event::EffectsDropped {
            client: super::NAME.into(),
        })
        .unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 0, "the drop");
        mc.settings_changed().unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 1, "it blurs again");
    }

    /// A query costs the core a walk over the scene, so a scene of 300
    /// panels must not ask one per title and row: the alignment of a
    /// scene reads all rects with one query.
    #[test]
    fn a_scene_aligns_with_one_query() {
        let (mut mc, model) = crate::testing::mc("one-query", [1920, 1080]);
        let rows = |i: u32| vec![format!("{i}\tone"), format!("{i}\ttwo")];
        let scene = mc.scenes.get_mut("mission").unwrap();
        for (i, panel) in (0..).zip(&mut scene.panels) {
            panel.title = format!("T{i}");
            panel.lines = rows(i);
        }
        mc.build().unwrap();
        let before = model.lock().unwrap().queries;
        mc.align("mission").unwrap();
        assert_eq!(model.lock().unwrap().queries - before, 1);
    }

    /// The window that mc hands away leaves the keyboard to its receiver:
    /// mc sends no focus while it leaves the scene of the window.
    #[test]
    fn a_handed_window_leaves_the_focus_to_its_receiver() {
        use chora_kit::give::Give;
        let (mut mc, model) = crate::testing::mc("handed", [1920, 1080]);
        mc.show("agenda", true).unwrap();
        let info = chora_scene::SurfaceInfo::default();
        model.lock().unwrap().scene.add_surface("s1", info);
        model.lock().unwrap().scene.reserve("s1", Some("mc"));
        mc.apply(vec![chora_scene::Command::Attach {
            surface: "s1".into(),
            parent: crate::scene_id("agenda"),
        }])
        .unwrap();
        let claim = chora_scene::Command::Create {
            id: "xp-claim".into(),
            kind: chora_scene::NodeKind::Group,
            parent: "root".into(),
            props: chora_scene::PropsUpdate {
                claim: Some(Some([0.0, 0.0, 10.0, 10.0])),
                ..chora_scene::PropsUpdate::default()
            },
        };
        model.lock().unwrap().scene.apply("xp", &[claim]).unwrap();
        mc.windows.insert("agenda".into(), "s1".into());
        mc.focused = Some("s1".into());
        let before = model.lock().unwrap().focuses.len();
        mc.give(Give {
            surface: "s1".into(),
            to: "xp".into(),
            grab: None,
        })
        .unwrap();
        let focuses = model.lock().unwrap().focuses.split_off(before);
        assert!(
            focuses.is_empty(),
            "mc sent a focus after the hand: {focuses:?}"
        );
        assert_eq!(mc.focused, None);
    }
}
