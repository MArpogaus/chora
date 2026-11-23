//! chora-mc, experimental: a voice-first desktop without window
//! management. The whole screen shows one scene, a grid of text panels or
//! one app, and quiet regions around it show the context: a header with
//! the running head, the state of the agent and the clock, a footer with
//! the words of the agent, notes in the margin and a drawer with a docked
//! terminal. The agent and small collector scripts fill the panels and
//! the regions through the exported methods; the plugin reads no sources
//! itself.

#![allow(
    clippy::cognitive_complexity,
    reason = "the tracing macros count as branches; the handlers are flat lists of arms"
)]
#![allow(
    clippy::print_stderr,
    reason = "the usage line of --launch is no log: the log starts in the runtime"
)]

mod acts;
mod agent;
#[cfg(test)]
mod audit;
mod build;
mod config;
mod content;
mod exports;
mod fonts;
mod keys;
mod layout;
mod lifecycle;
mod manage;
mod methods;
mod mic;
#[cfg(test)]
mod minor;
mod region;
mod scene;
mod show;
mod state;
mod windows;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::PathBuf,
    process::ExitCode,
    time::Instant,
};

use chora_kit::{
    launch,
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
    wm::focus::{Focus, Holder},
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
    config::{Action, Config, Layout, Look, Style, Type},
    exports::exported_methods,
    fonts::Ink,
    region::{FIT_MS, Kind, PARKED, REGIONS, Region, Rule},
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
    /// A turn of a plain key is going on: the page that it shows keeps the
    /// keyboard also when a window stands in it, so that the next plain key
    /// still turns the page (mc-42).
    plain_turn: bool,
    /// The scene that a plain turn showed and whose program it started:
    /// the window that comes keeps no keyboard from the page, as when the
    /// program already ran (mc-42).
    plain_start: Option<String>,
    /// The error of the last focus query of `refresh`, if it failed: a
    /// failure logs once and not on every `state.get`.
    focus_query_error: Option<String>,
    settings: settings::File<Config>,
    /// The directory of the scene files.
    dir: PathBuf,
    scenes: BTreeMap<String, Scene>,
    /// The names of the scenes in the order they appeared; `order` of
    /// the settings goes before it.
    appeared: Vec<String>,
    /// The window of an app scene, by scene.
    windows: HashMap<String, String>,
    /// Which surfaces are mc's, and where they came from (`kit::wm::book`).
    book: chora_kit::wm::Book,
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
    /// The nodes whose fade runs, each with the time that it is over at
    /// the latest: a fade whose end the core never tells does not keep mc
    /// busy for ever.
    fading: HashMap<String, Instant>,
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
    /// When the set before has faded out under the one of the last render;
    /// until then the look says it is busy.
    /// The docked apps that faded out with the set before; `None` while
    /// that set does not exist.
    left: Option<Vec<String>>,
    /// What each region of the last render shows.
    sigs: BTreeMap<Region, String>,
    /// The settings of the last build without the theme: a change of the
    /// theme alone fades the colors.
    shape: Option<Value>,
    /// The theme of the nodes that stand.
    themed: Option<Value>,
    /// The keyboard focus that mc gave and the core moved
    /// (`kit::wm::focus`).
    focus: Focus,
    /// The surface that has the keyboard focus of the core, from
    /// `focus.changed`.
    core_focus: Option<String>,
    /// The word of `state.get`; only reads.
    voiced_word: chora_kit::helper::Helper,
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
    /// The ids that the notification server gave the notices of mc, for `notify.clear`.
    notices: agent::Notices,
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
    /// Whether the key list drew the plain keys of a page the last time:
    /// it follows who holds the keyboard on the page of a window.
    keys_plain: std::cell::Cell<bool>,
    /// Whether the HUD host shows the chat in the margin column.
    chat: Chat,
    /// Where the box of the chat of the last layout is.
    slot: Option<[f64; 4]>,
    /// The output and the box that the host was told last.
    slot_sent: Option<(String, Option<[f64; 4]>)>,
    /// How far the page of a scene scrolled down, by scene.
    scroll: BTreeMap<String, f64>,
}

#[allow(clippy::print_stdout, reason = "--print-config prints the default")]
fn main() -> ExitCode {
    let words: Vec<String> = std::env::args().skip(1).collect();
    if words.first().is_some_and(|a| a == "--launch") {
        return launch(&words[1..]);
    }
    let args = Args::parse();
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    runtime::run(NAME, || play(&args))
}

/// `--launch fuzzel`: the Apps list, the installed programs and those of
/// the shared entries, as [`chora_kit::launcher`] shows them, so that text
/// that finds nothing shows the list again under the row "Nothing found"
/// and the launcher never closes without a word (mc-44). Another program
/// is no launch of mc.
fn launch(argv: &[String]) -> ExitCode {
    if argv != ["fuzzel"] {
        eprintln!("usage: chora-mc --launch fuzzel");
        return ExitCode::from(2);
    }
    chora_kit::launcher::run(&["fuzzel".to_owned()], "foot", None);
    ExitCode::SUCCESS
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
    let [shader, blur] = [("write", WRITE_SHADER), ("blur", BLUR_SHADER)]
        .map(|(name, text)| chora_kit::runtime_file("chora-mc", &format!("{name}.frag"), text));
    let (shader, blur) = (shader?, blur?);
    let rt = Runtime::connect(NAME, args.output.clone().into_iter().collect())?;
    let mut mc = Mc::new(rt, settings, dir, [shader, blur]);
    mc.manage_windows = args.manage_windows;
    if args.output.is_some() {
        mc.page_keys = keys::PageKeys::Bound;
    }
    mc.start(methods)?;
    if let Some(err) = mc.settings.broken().map(str::to_owned) {
        mc.notice("Settings", &err);
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
    #[allow(clippy::too_many_lines, reason = "one line per field of the plugin")]
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
            plain_turn: false,
            plain_start: None,
            focus_query_error: None,
            appeared: scenes.keys().cloned().collect(),
            scenes,
            settings,
            dir,
            windows: HashMap::new(),
            book: chora_kit::wm::Book::new(NAME, false),
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
            fading: HashMap::new(),
            put: Vec::new(),
            toggled: BTreeMap::new(),
            docks: HashMap::new(),
            set: 0,
            fade: FADE,
            left: None,
            sigs: BTreeMap::new(),
            fitting: HashMap::new(),
            shape: None,
            themed: None,
            rendered: None,
            focus: Focus::default(),
            core_focus: None,
            voiced_word: chora_kit::helper::Helper::default(),
            dragged: None,
            quiet: BTreeSet::new(),
            dock_focus: None,
            clocks: BTreeMap::new(),
            volume: chora_kit::volume::Mixer::new(NAME),
            volume_reading: None,
            volume_ticks: 0,
            texts: BTreeMap::new(),
            notices: agent::Notices::default(),
            agents: Vec::new(),
            voiced: Vec::new(),
            agent_focus: None,
            solo: None,
            keys: keys::List::default(),
            keys_plain: std::cell::Cell::new(false),
            chat: Chat::Closed,
            slot: None,
            slot_sent: None,
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
/// `assistant.chat` says where the HUD host shows its chat, and
/// `assistant.chat.keyboard` when its field takes the keyboard.
const EVENTS: [&str; 21] = [
    "voice.*",
    "focus.changed",
    "focus.node",
    "node.key",
    "assistant.chat",
    "assistant.chat.keyboard",
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

    /// Drops the node that mc holds the keyboard with when the core does
    /// not focus it: `focus.node` names the node that another client
    /// takes (heard, `Focus::away`), but not that the node of mc lost the
    /// keyboard to a surface, so `state.get` still asks the core.
    fn refresh(&mut self) {
        let Some(node) = self.focus.node().map(str::to_owned) else {
            return;
        };
        let query = Query {
            selector: format!("id:{node}"),
            filter: Some(chora_scene::ipc::Filter {
                focused: true,
                ..chora_scene::ipc::Filter::default()
            }),
        };
        match self.rt.conn.call::<_, Vec<Value>>("scene.query", query) {
            Ok(nodes) => {
                self.focus_query_error = None;
                if nodes.is_empty() {
                    self.focus.drop_node();
                }
            }
            Err(err) => {
                let text = err.to_string();
                if self.focus_query_error.as_deref() != Some(&text) {
                    chora_kit::runtime::report(NAME, &format_args!("focus query: {text}"));
                    self.focus_query_error = Some(text);
                }
            }
        }
    }

    fn state(&self) -> Option<Value> {
        Some(self.state_value())
    }

    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if !chora_kit::runtime::hears_pointer(&event, self.rt.accepts_injected()) {
            return Ok(());
        }
        self.voiced_word.hear_asking(&event, &mut self.rt.conn);
        if chora_kit::wm::Book::hears(&event) {
            self.lifecycle(&event)?;
        }
        if let Some(surface) = self.focus.hear(&event) {
            self.core_focus = surface.map(String::from);
            return self.sync_keys();
        }
        match event {
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
                .. // injected: the runtime drops an injected binding key unless it accepts them
            } => {
                self.act(&binding);
                Ok(())
            }
            Event::InputButton {
                binding,
                state,
                position,
                surface,
                .. // injected: Plugin::event drops an injected pointer event first
            } => {
                self.drag(&binding, state, position, surface);
                Ok(())
            }
            Event::Emitted {
                event,
                client,
                data,
            } => self.emitted(&event, &client, data),
            Event::NodeKey {
                key,
                state: PressState::Pressed,
                modifiers,
                node: _,
                code: _,
                text: _,
                injected,
            } => {
                self.page_key(&key, &modifiers, injected);
                Ok(())
            }
            Event::PointerScroll { node, dy, .. } => self.scroll_page(&node, dy), // injected: Plugin::event drops an injected pointer event first
            Event::PointerPressed {
                node,
                position,
                injected,
                ..
            } => self.press(&node, position, injected),
            Event::PointerEntered { node, .. } => self.mark(&node, true), // injected: Plugin::event drops an injected pointer event first
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
        let same_shape = self.built && !plain && self.shape.as_ref() == Some(&self.shape_of());
        let theme_only = same_shape && self.themed != Some(self.theme_of());
        let no_look = same_shape && !theme_only;
        self.rt.start_grabs();
        self.grab_all()
            .and_then(|()| self.rt.end_grabs())
            .and_then(|()| {
                if no_look {
                    return Ok(());
                }
                if theme_only && self.recolor()? {
                    self.themed = Some(self.theme_of());
                    return Ok(());
                }
                self.build()
            })
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

fn invalid(message: String) -> RpcError {
    RpcError::new(RpcError::INVALID_PARAMS, message)
}

#[allow(clippy::needless_pass_by_value, reason = "for map_err")]
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
            Some(surface) => self.focus.to(Some(surface.clone())),
            None => self.focus.to_node(Some(page_id(name))),
        };
        self.apply(vec![command])
    }
}

/// The keys of a remote without Super: the core lets a window manager with
/// a claim grab them without a modifier (mc-46).
const REMOTE_KEYS: [&str; 2] = ["XF86Back", "XF86HomePage"];

/// Whether a binding is grabbed only with the claim of a window manager.
fn needs_claim(key: &str, action: &Action) -> bool {
    matches!(
        action,
        Action::System(chora_kit::actions::System::Drag | chora_kit::actions::System::Volume(_))
    ) || REMOTE_KEYS.iter().any(|k| k.eq_ignore_ascii_case(key))
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
    use crate::{Action, Look};
    use chora_scene::{Command, NodeKind, PropsUpdate};

    /// The line that a page that cannot be shown leaves in the log (a
    /// guard of the move of `report` to the kit).
    #[test]
    fn report_line() {
        if chora_kit::testing::in_child() {
            chora_kit::runtime::report(
                crate::NAME,
                &format_args!("{}: {}", "Overview", "no such window"),
            );
            return;
        }
        assert_eq!(
            chora_kit::testing::stderr_of("tests::report_line"),
            "chora-mc: Overview: no such window\n"
        );
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

    #[test]
    fn the_root_of_mc_lies_in_the_background_order_below_every_other_root() {
        let core = chora_kit::testing::FakeCore::start();
        let mut xp = core.connect("xp");
        xp.apply(vec![super::create(
            "xp",
            NodeKind::Group,
            chora_scene::ROOT,
            PropsUpdate::default(),
        )])
        .unwrap();
        let mut mc = crate::testing::mc_on(&core, "background-order", [800, 600]);
        let model = core.model();
        let roots = || {
            model
                .lock()
                .unwrap()
                .scene
                .node(chora_scene::ROOT)
                .unwrap()
                .children
                .clone()
        };
        let order = model
            .lock()
            .unwrap()
            .scene
            .node(super::NAME)
            .unwrap()
            .props
            .order;
        assert_eq!(order, chora_scene::Order::Background);
        xp.apply(vec![Command::Move {
            id: "xp".into(),
            parent: None,
            index: Some(0),
        }])
        .unwrap();
        assert_eq!(
            roots().first().map(String::as_str),
            Some(super::NAME),
            "{:?}",
            roots()
        );
        mc.build().unwrap();
        assert_eq!(
            roots().first().map(String::as_str),
            Some(super::NAME),
            "{:?}",
            roots()
        );
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
        press_key_from(mc, key, modifiers, false);
    }

    fn press_key_from(mc: &mut super::Mc, key: &str, modifiers: &[&str], injected: bool) {
        use chora_kit::runtime::Plugin as _;
        mc.event(chora_scene::Event::NodeKey {
            node: super::page_id("agenda"),
            key: key.into(),
            code: 0,
            state: chora_scene::ipc::PressState::Pressed,
            modifiers: modifiers.iter().map(|m| (*m).to_owned()).collect(),
            text: None,
            injected,
        })
        .unwrap();
    }

    /// Home and `XF86HomePage` alone go to the first page; with a modifier
    /// they are no key of the page.
    #[test]
    fn plain_home_goes_to_the_first_page() {
        let (mut mc, _model) = crate::testing::mc("plain-home", [1280, 800]);
        mc.show("agenda", true).unwrap();
        press_key(&mut mc, "Right", &[]);
        press_key(&mut mc, "Home", &["Shift"]);
        assert_eq!(mc.current.as_deref(), Some("morning"));
        press_key(&mut mc, "Home", &[]);
        assert_eq!(mc.current.as_deref(), Some("agenda"));
        press_key(&mut mc, "Right", &[]);
        press_key(&mut mc, "XF86HomePage", &[]);
        assert_eq!(mc.current.as_deref(), Some("agenda"));
    }

    /// A remote without Super: `back` (`XF86Back`) takes the keyboard from a
    /// window to its page, which keeps the page; on a page it goes one step
    /// back; `XF86HomePage` goes to the first page. A plain key that the
    /// window got stays the window's (mc-46).
    #[test]
    fn back_leaves_a_window_for_its_page_and_home_goes_first() {
        let (mut mc, _model) = crate::testing::mc("remote-back", [1280, 800]);
        mc.windows.insert("terminal".into(), "s1".into());
        mc.show("agenda", true).unwrap();
        press_key(&mut mc, "Right", &[]);
        mc.show("terminal", true).unwrap();
        press_key(&mut mc, "Return", &[]);
        let window = |mc: &super::Mc| {
            matches!(
                mc.focus.holder(super::NAME, |s| s == "s1"),
                super::Holder::Window(_)
            )
        };
        assert!(window(&mc), "Return gave the window the keyboard");
        mc.turn(Look::Back);
        assert!(!window(&mc), "XF86Back took the keyboard from the window");
        assert_eq!(mc.current.as_deref(), Some("terminal"), "the page stays");
        mc.turn(Look::Back);
        assert_ne!(mc.current.as_deref(), Some("terminal"), "a page goes back");
        mc.show("terminal", true).unwrap();
        press_key(&mut mc, "Return", &[]);
        mc.turn(Look::Scene("agenda".into()));
        assert_eq!(mc.current.as_deref(), Some("agenda"));
        assert!(!window(&mc), "the first page holds the keyboard");
    }

    /// mc-42 (review r3, rv-1): a turn away from the page that a plain key
    /// started and back to it, before the window maps, keeps the plain
    /// start: the window that comes does not take the keyboard from the
    /// page.
    #[test]
    fn a_turn_away_and_back_while_the_program_starts_keeps_the_keyboard_on_the_page() {
        let (mut mc, model) = crate::testing::mc("turn-back-start", [1280, 800]);
        mc.show("agenda", true).unwrap();
        for _ in 0..4 {
            press_key(&mut mc, "Right", &[]);
        }
        assert_eq!(mc.current.as_deref(), Some("terminal"));
        mc.starting
            .insert("t1".into(), crate::Target::Scene("terminal".into()));
        press_key(&mut mc, "Left", &[]);
        press_key(&mut mc, "Right", &[]);
        assert_eq!(mc.current.as_deref(), Some("terminal"));
        model
            .lock()
            .unwrap()
            .scene
            .add_surface("s1", chora_scene::SurfaceInfo::default());
        mc.adopt("s1", Some("foot"), Some("t1")).unwrap();
        let focused = model
            .lock()
            .unwrap()
            .scene
            .focused_node()
            .map(str::to_owned);
        assert_eq!(
            focused,
            Some(super::page_id("terminal")),
            "the page keeps the keyboard"
        );
        assert!(
            matches!(
                mc.focus.holder(super::NAME, |s| s == "s1"),
                super::Holder::Node(_)
            ),
            "the window did not take the keyboard"
        );
    }

    /// The window of the program that the first plain Right from page 4
    /// started comes as `surface.created` that the core reserved for mc
    /// (mc holds the claim of its output, `to` names it): it lands in the
    /// terminal page, not in a page of its own, and the page keeps the
    /// keyboard (mc-42, persona round 4).
    #[test]
    fn a_window_that_the_core_reserved_for_mc_lands_in_the_page_that_started_it() {
        let (mut mc, model) = crate::testing::mc("reserved-start", [1280, 800]);
        mc.show("agenda", true).unwrap();
        for _ in 0..4 {
            press_key(&mut mc, "Right", &[]);
        }
        assert_eq!(mc.current.as_deref(), Some("terminal"));
        mc.starting
            .insert("t1".into(), crate::Target::Scene("terminal".into()));
        model
            .lock()
            .unwrap()
            .scene
            .add_surface("s1", chora_scene::SurfaceInfo::default());
        let created: chora_scene::Event = serde_json::from_value(serde_json::json!({
            "event": "surface.created", "surface": "s1", "app_id": "foot", "title": "foot",
            "to": super::NAME, "token": "t1",
        }))
        .unwrap();
        mc.lifecycle(&created).unwrap();
        assert_eq!(mc.current.as_deref(), Some("terminal"));
        assert_eq!(mc.windows.get("terminal").map(String::as_str), Some("s1"));
        assert!(
            !mc.scenes.contains_key("foot"),
            "the window has a page of its own"
        );
        let focused = model
            .lock()
            .unwrap()
            .scene
            .focused_node()
            .map(str::to_owned);
        assert_eq!(
            focused,
            Some(super::page_id("terminal")),
            "the page keeps the keyboard"
        );
    }

    /// The chat field of the host takes the keys over a window that holds
    /// the keyboard: the core still focuses that window under the field,
    /// so `state.get` says mc holds it (I3, persona round 4 sofa: a window
    /// of the launcher, then Super+a).
    #[test]
    fn the_chat_field_over_a_window_leaves_the_window_the_holder() {
        let (mut mc, model) = crate::testing::mc("chat-over-window", [1280, 800]);
        mc.show("agenda", true).unwrap();
        model
            .lock()
            .unwrap()
            .scene
            .add_surface("s1", chora_scene::SurfaceInfo::default());
        mc.adopt("s1", Some("foot"), None).unwrap();
        let focus = |mc: &super::Mc| mc.state_value()["focus"].clone();
        assert_eq!(
            focus(&mc),
            serde_json::json!({"holder": "mc", "surface": "s1"})
        );
        mc.emitted(
            "assistant.chat.keyboard",
            super::ASSISTANT,
            serde_json::json!({"on": true}),
        )
        .unwrap();
        assert_eq!(
            focus(&mc),
            serde_json::json!({"holder": "mc", "surface": "s1"}),
            "the core focuses s1 under the chat field"
        );
    }

    /// The default file binds the keys of a remote, and mc grabs them
    /// only with a claim, as the media keys.
    #[test]
    fn the_keys_of_a_remote_are_bound_and_need_a_claim() {
        let config =
            chora_kit::settings::parse::<crate::config::Config>(crate::config::DEFAULT).unwrap();
        let keys = &config.common.bindings;
        assert_eq!(keys["XF86Back"], Action::Look(Look::Back));
        assert_eq!(
            keys["XF86HomePage"],
            Action::Look(Look::Scene("agenda".into()))
        );
        for key in ["XF86Back", "XF86HomePage"] {
            assert!(super::needs_claim(key, &keys[key]), "{key}");
        }
        assert!(!super::needs_claim("Super+Home", &keys["Super+Home"]));
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

    /// A helper that fills a panel builds the page anew, and the page
    /// keeps the keyboard, so that the arrows of the remote still turn it.
    #[test]
    fn a_page_keeps_the_keyboard_when_a_helper_fills_a_panel() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("page-keyboard-fill", [1280, 800]);
        mc.show("agenda", true).unwrap();
        let focused = || {
            model
                .lock()
                .unwrap()
                .scene
                .focused_node()
                .map(str::to_owned)
        };
        assert_eq!(focused(), Some(super::page_id("agenda")), "before");
        let lines =
            serde_json::json!({"scene": "agenda", "panel": "today", "lines": ["09:00\tStand-up"]});
        mc.method("panel.set", lines).unwrap();
        assert_eq!(focused(), Some(super::page_id("agenda")), "after");
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
        assert!(keys.contains(&"Super+←\u{2009}→".to_owned()), "{keys:?}");
    }

    fn right_items(mc: &super::Mc) -> Vec<crate::region::Item> {
        let name = mc.current.clone().unwrap();
        let mut plan = mc.items(&name);
        plan.remove(&crate::region::Region::Right)
            .map(|(items, _)| items)
            .unwrap_or_default()
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
        assert_eq!(back.first(), Some(&chora_kit::wm::focus::give("s1")));
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

    /// While the chat is open its box lies over the doors, so the column
    /// has none; the doors come back when it shuts.
    #[test]
    fn the_doors_leave_while_the_chat_is_open() {
        let mut mc = crate::testing::mc("doors-chat", [1280, 800]).0;
        mc.show("agenda", false).unwrap();
        let doors = |mc: &super::Mc| {
            (right_items(mc).iter())
                .filter(|i| i.hit.as_deref().is_some_and(|h| h.starts_with("door:")))
                .count()
        };
        assert_eq!(doors(&mc), 5);
        mc.chat = crate::Chat::Open;
        assert_eq!(doors(&mc), 0);
    }

    /// A chat that cannot open says so in a critical notice of
    /// the notification server, not only in a log.
    #[test]
    fn a_chat_that_does_not_open_shows_a_notice() {
        use chora_kit::testing::FakeCore;
        use serde_json::json;
        let core = FakeCore::start();
        core.answer_with("notify.show", json!({"id": 3}));
        let mut mc = crate::testing::mc_on(&core, "chat-refused", [1280, 800]);
        mc.show("agenda", false).unwrap();
        core.refuse_next("methods.call");
        mc.chat_with(None);
        let called = core.model().lock().unwrap().called.clone();
        let shown: Vec<_> = (called.iter())
            .filter(|c| c["method"] == "notify.show")
            .collect();
        assert_eq!(shown.len(), 1, "{called:?}");
        let params = &shown[0]["params"];
        assert_eq!(params["urgency"], "critical");
        assert!(params["body"].as_str().unwrap().contains("does not open"));
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

    /// The HUD host draws the voice HUD and the question: mc shows
    /// neither the state nor the sentence of the voice agent nor its
    /// question, and still shows its helpers.
    #[test]
    fn the_host_draws_the_voice_status() {
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("host-status", [1920, 1080]);
        mc.follow("voice.state", "voice", json!({"state": "listening"}))
            .unwrap();
        mc.follow(
            "voice.say",
            "voice",
            json!({"text": "Hello there", "duration_ms": 0}),
        )
        .unwrap();
        let ask = json!({"id": 1, "prompt": "Atlas wants to write notes.txt.",
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
        assert!(mc.solo.is_none() && !mc.texts.contains_key("confirm"));
        assert!(mc.agents.iter().any(|a| a.name == "Atlas"));
        let head = mc.scenes["agents"]
            .chapter
            .running_head
            .clone()
            .unwrap_or_default();
        assert!(
            !head.to_lowercase().contains("agent"),
            "the head says {head:?}"
        );
    }

    /// mc never claims the indicator of voice: the HUD host does.
    #[test]
    fn mc_claims_no_indicator() {
        let (mut mc, model) = crate::testing::mc("host-claim", [1920, 1080]);
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

    fn door_items(mc: &super::Mc, scene: &str) -> usize {
        let items = mc.items(scene).into_values().flat_map(|(items, _)| items);
        items.filter(|i| i.id.contains("-doors-")).count()
    }

    /// The doors of the column stay while the chat is open: lock, power,
    /// programs and volume keep their mouse door, and the box of the chat
    /// ends above them, so that the chat covers none (MH-2).
    #[test]
    fn the_doors_of_the_column_stay_beside_the_open_chat() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, _) = crate::testing::mc("chat-doors", [1920, 1080]);
        let first = mc.settings.value.default_scene.clone();
        mc.show(&first, false).unwrap();
        let doors = door_items(&mc, &first);
        assert!(doors > 0, "the doors show");
        mc.event(chat_event("assistant", true, "test")).unwrap();
        assert_eq!(door_items(&mc, &first), doors, "the doors stay");
        let rect = mc.chat_slot().expect("the chat has a box");
        let column = mc.boxes_of(&first)[&crate::region::Region::Right];
        let baseline = mc.settings.value.scale.baseline;
        assert!(
            rect[1] + rect[3] + 2.0 * baseline <= column[1] + column[3],
            "the chat ends above the doors"
        );
        mc.event(chat_event("assistant", false, "test")).unwrap();
        assert_eq!(door_items(&mc, &first), doors, "the doors are back");
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

    /// Another client that takes the keyboard with a node tells mc
    /// nothing, so `state.get` asks the core before it says that mc holds
    /// the keyboard with a node (xp over mc, I3).
    #[test]
    fn state_says_nobody_when_a_node_of_another_client_has_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("node-taken", [1280, 800]);
        let command = mc.focus.to_node(Some(crate::page_id("agenda")));
        mc.apply(vec![command]).unwrap();
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": "mc"}));
        xp_takes_the_keyboard(&model);
        mc.refresh();
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// rd-5: mc hears `focus.node`, so a node of another client that takes
    /// the keyboard leaves mc no holder (one holder, I3).
    #[test]
    fn a_focus_node_of_another_client_leaves_mc_no_holder() {
        use chora_kit::runtime::Plugin as _;
        assert!(super::EVENTS.contains(&"focus.node"));
        let (mut mc, _model) = crate::testing::mc("node-heard", [1280, 800]);
        let command = mc.focus.to_node(Some(crate::page_id("agenda")));
        mc.apply(vec![command]).unwrap();
        mc.event(chora_scene::Event::FocusNode {
            node: Some("xp-icons".into()),
            owner: Some("xp".into()),
        })
        .unwrap();
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// The node of mc that still has the keyboard stays its holder.
    #[test]
    fn state_says_mc_when_its_node_still_has_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, _model) = crate::testing::mc("node-kept", [1280, 800]);
        let command = mc.focus.to_node(Some(crate::page_id("agenda")));
        mc.apply(vec![command]).unwrap();
        mc.refresh();
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": "mc"}));
    }

    /// The query of `refresh` names the node of mc, not every node of the
    /// scene, which the core would lay out on its main thread (rv-2).
    #[test]
    fn refresh_asks_the_core_for_its_own_node_only() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("node-asked", [1280, 800]);
        let node = crate::page_id("agenda");
        let command = mc.focus.to_node(Some(node.clone()));
        mc.apply(vec![command]).unwrap();
        model.lock().unwrap().selectors.clear();
        mc.refresh();
        assert_eq!(model.lock().unwrap().selectors, [format!("id:{node}")]);
    }

    fn xp_takes_the_keyboard(model: &chora_kit::testing::Model) {
        let mut core = model.lock().unwrap();
        core.scene
            .apply(
                "xp",
                &[
                    super::create(
                        "xp-icons",
                        NodeKind::Group,
                        chora_scene::ROOT,
                        PropsUpdate::default(),
                    ),
                    Command::FocusNode {
                        node: Some("xp-icons".into()),
                    },
                ],
            )
            .unwrap();
    }

    /// A window that the user clicked while the chat was open keeps the
    /// keyboard when the chat closes.
    #[test]
    fn a_window_clicked_under_the_chat_keeps_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-click", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.event(chat_event("assistant", true, "test")).unwrap();
        mc.event(chora_kit::testing::focus_changed(Some("s9")))
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
        for surface in [Some("s9"), None] {
            mc.event(chora_kit::testing::focus_changed(surface))
                .unwrap();
        }
        let before = model.lock().unwrap().focuses.len();
        mc.event(chat_event("assistant", false, "test")).unwrap();
        assert!(model.lock().unwrap().focuses.len() > before);
    }

    /// rw-1: a click on a window of mc after another look took the
    /// keyboard gives it back (D55, "until one of its windows takes it
    /// back"): mc sends the focus again, also for the window it sent last.
    #[test]
    fn a_click_on_its_window_after_another_look_took_the_keyboard_sends_the_focus() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("click-back", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.windows.insert("agenda".into(), "s1".into());
        mc.click("s1", [0.0, 0.0], false).unwrap();
        let sent = model.lock().unwrap().focuses.len();
        mc.event(chora_kit::testing::focus_changed(Some("s9")))
            .unwrap();
        mc.click("s1", [0.0, 0.0], false).unwrap();
        let focuses = model.lock().unwrap().focuses.clone();
        assert_eq!(focuses.len(), sent + 1, "{focuses:?}");
        assert_eq!(focuses.last(), Some(&Some("s1".to_owned())));
    }

    /// rw-2: `take_free_keyboard` gives the keyboard to the page node, and
    /// `state.get` tells that mc holds it.
    #[test]
    fn a_free_keyboard_taken_by_the_page_shows_in_the_state() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("free-keyboard", [1280, 800]);
        mc.show("agenda", false).unwrap();
        assert_eq!(model.lock().unwrap().scene.focused_node(), None);
        mc.take_free_keyboard().unwrap();
        assert!(model.lock().unwrap().scene.focused_node().is_some());
        assert_eq!(
            mc.state().unwrap()["focus"],
            serde_json::json!({"holder": "mc"})
        );
    }

    /// kit7-a1 (I3): a chat that opens without the keyboard leaves it with
    /// the page, so mc still says it holds the focus; once the field of
    /// the chat takes the keyboard, mc holds none.
    #[test]
    fn the_chat_takes_the_focus_of_mc_only_with_the_keyboard() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, _model) = crate::testing::mc("chat-claim", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.take_free_keyboard().unwrap();
        let holds = json!({"holder": "mc"});
        assert_eq!(mc.state().unwrap()["focus"], holds);
        mc.event(chat_event("assistant", true, "test")).unwrap();
        assert_eq!(mc.state().unwrap()["focus"], holds, "chat open");
        let keyboard = chora_scene::Event::Emitted {
            event: "assistant.chat.keyboard".into(),
            client: "assistant".into(),
            data: json!({"on": true}),
        };
        mc.event(keyboard).unwrap();
        assert_eq!(mc.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// A docked app that had the keyboard before the chat opened gets it
    /// back when the chat closes.
    #[test]
    fn a_docked_app_gets_the_keyboard_back_when_the_chat_closes() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("chat-dock", [1280, 800]);
        mc.show("agenda", false).unwrap();
        mc.docks.insert("shell".into(), "dock1".into());
        mc.focus.surface = Some("dock1".into());
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
    /// scene.save (W12), and wrote the words of the agent through hud.set
    /// and hud.say (review plugins r1, MC-03), and emptied the notifications
    /// through notify.clear (review security a r1).
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
                "hud.set",
                "hud.say",
                "app.open",
                "region.toggle",
                "region.put",
                "region.solo",
                "agent.set",
                "notify.push",
                "notify.clear",
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

    /// A panel below the text area of the page shows nothing while it
    /// writes, so it gets no write effect: the end of the write then
    /// changes the few panels that show, not 300 (FLK-MC-SCALE: 300 effect
    /// updates held the loop of mc for 0.4 s, 6 s under load, and the
    /// next `settings.set` waited behind them).
    #[test]
    fn a_panel_below_the_page_gets_no_write_effect() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("write-below", [640, 480]);
        mc.shader = "/write.frag".into();
        let panels: Vec<serde_json::Value> = (0..300)
            .map(|i| {
                json!({"id": format!("p{i}"), "cell": [i % 20, i / 20, 1, 1],
                    "title": format!("T{i}"), "lines": [format!("{i}\tline")]})
            })
            .collect();
        mc.method(
            "scene.save",
            json!({"name": "many", "grid": [20, 15], "panels": panels}),
        )
        .unwrap();
        mc.show("many", true).unwrap();
        let effects = |id: &str| {
            let core = model.lock().unwrap();
            core.scene
                .node(&super::panel_id("many", id))
                .unwrap()
                .props
                .effects
                .len()
        };
        let writing = (0..300).filter(|i| effects(&format!("p{i}")) > 0).count();
        assert_eq!(effects("p0"), 1, "the first panel writes");
        assert_eq!(effects("p299"), 0, "the last is far below the page");
        assert!(writing < 100, "{writing} of 300 panels write");
    }

    /// A build removes the nodes whose write animation runs and makes new
    /// ones without it, so no end of it comes: mc is quiet at once, not
    /// busy with a write that nobody finishes (mc-49).
    #[test]
    fn a_build_in_the_midst_of_a_write_leaves_mc_quiet() {
        let (mut mc, model) = crate::testing::mc("write-build", [1920, 1080]);
        mc.shader = "/write.frag".into();
        mc.show("agenda", true).unwrap();
        assert_eq!(panel_effects(&mc, &model, "agenda"), 1, "it writes");
        assert!(!mc.writing.is_empty());
        mc.build().unwrap();
        assert_eq!(
            panel_effects(&mc, &model, "agenda"),
            0,
            "the new node rests"
        );
        assert_eq!(
            busy_words(&mc, "write:"),
            Vec::<String>::new(),
            "mc waits for the end of a write that was removed"
        );
    }

    /// The words of `busy` that start with a prefix.
    fn busy_words(mc: &super::Mc, prefix: &str) -> Vec<String> {
        use chora_kit::runtime::Plugin as _;
        let state = mc.state().unwrap();
        let busy = state["busy"].as_array().unwrap();
        (busy.iter().filter_map(|b| b.as_str()))
            .filter(|b| b.starts_with(prefix))
            .map(str::to_owned)
            .collect()
    }

    /// Tells mc that every fade that it waits for ended.
    fn end_fades(mc: &mut super::Mc) {
        use chora_kit::runtime::Plugin as _;
        for word in busy_words(mc, "fade:") {
            mc.event(chora_scene::Event::AnimationDone {
                node: Some(word.trim_start_matches("fade:").to_owned()),
                output: None,
            })
            .unwrap();
        }
    }

    /// A page turn cross-fades its regions for a moment: both sets are
    /// shown then, so mc is busy until the end of each fade, and a check
    /// that waits for quiet never reads two sets one over the other
    /// (mc-48).
    #[test]
    fn a_cross_fade_keeps_mc_busy_until_its_end() {
        let (mut mc, _model) = crate::testing::mc("fade-busy", [1920, 1080]);
        mc.show("agenda", true).unwrap();
        end_fades(&mut mc);
        assert_eq!(busy_words(&mc, "fade:"), Vec::<String>::new());
        mc.show("morning", true).unwrap();
        assert!(
            !busy_words(&mc, "fade:").is_empty(),
            "the cross-fade of the page is not busy"
        );
        end_fades(&mut mc);
        assert_eq!(
            busy_words(&mc, "fade:"),
            Vec::<String>::new(),
            "the fades ended"
        );
    }

    /// A change of the theme fades the colors and cross-fades the regions
    /// through one transaction: mc is busy until their end, like for a
    /// page turn (pr2-1, rv-2).
    #[test]
    fn a_theme_change_keeps_mc_busy_until_its_fades_end() {
        let (mut mc, _model) = crate::testing::mc("theme-busy", [1920, 1080]);
        mc.show("agenda", true).unwrap();
        end_fades(&mut mc);
        assert_eq!(busy_words(&mc, "fade:"), Vec::<String>::new());
        assert!(mc.recolor().unwrap(), "the core takes the recolor");
        assert!(
            !busy_words(&mc, "fade:").is_empty(),
            "the fades of a theme change are not busy"
        );
        end_fades(&mut mc);
        assert_eq!(busy_words(&mc, "fade:"), Vec::<String>::new());
    }

    /// The removal of a scene forgets its own nodes, not those of a scene
    /// whose name starts with the same letters (rv-1 of mc-fix).
    #[test]
    fn removing_a_scene_keeps_the_fade_of_a_sibling_that_shares_its_prefix() {
        let (mut mc, _model) = crate::testing::mc("sibling", [1920, 1080]);
        let fade = |id: String| Command::Animate {
            id,
            props: PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
            duration: 250.0,
            easing: chora_scene::Easing::EaseInOut,
        };
        let sibling = super::panel_id("foot-server", "today");
        let today_x = super::panel_id("foot", "today-x");
        mc.note_writes(&[
            fade(super::panel_id("foot", "today")),
            fade(sibling.clone()),
            fade(today_x.clone()),
            fade(super::page_id("foot")),
        ]);
        mc.note_writes(&[Command::Remove {
            id: super::scene_id("foot"),
            to: None,
        }]);
        assert_eq!(
            busy_words(&mc, "fade:"),
            vec![format!("fade:{sibling}")],
            "only the nodes of scene foot are forgotten"
        );
        mc.note_writes(&[
            fade(today_x.clone()),
            fade(super::panel_id("foot", "today")),
        ]);
        mc.note_writes(&[Command::Remove {
            id: super::panel_id("foot", "today"),
            to: None,
        }]);
        assert!(busy_words(&mc, "fade:").contains(&format!("fade:{today_x}")));
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

    /// New settings change the grabs. A binding that stays is never
    /// free in between, where its key would reach a window: mc lets no
    /// binding go that it grabs again.
    #[test]
    fn new_settings_keep_the_bindings_that_stay() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("kept-grabs", [1920, 1080]);
        mc.grab_all().unwrap();
        assert!(mc.rt.action("Super+F1").is_some());
        mc.settings_changed().unwrap();
        assert_eq!(model.lock().unwrap().ungrabs, [""; 0]);
        assert!(mc.rt.action("Super+F1").is_some());
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

    /// A change of the write time or of the write switch changes no node
    /// that stands: new text reads them when it writes. The settings
    /// then rebuild nothing (the 300-panel scene took 3 s for it).
    #[test]
    fn a_write_setting_that_changes_no_look_rebuilds_nothing() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("write-only", [1920, 1080]);
        mc.settings_changed().unwrap();
        let before = model.lock().unwrap().queries;
        mc.settings.value.write.duration_ms = 300.0;
        mc.settings_changed().unwrap();
        mc.settings.value.write.enabled = false;
        mc.settings_changed().unwrap();
        assert_eq!(model.lock().unwrap().queries, before, "no build ran");
        assert_eq!(mc.shape, Some(mc.shape_of()));
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
        mc.focus.surface = Some("s1".into());
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
        assert_eq!(mc.focus.surface, None);
    }

    /// `panel.set` is unprivileged and rewrote the columns of the agents,
    /// the words that the page shows as the agent's, and the list of the
    /// overview (review security a r1, MC-03 alias): mc writes those alone.
    #[test]
    fn panel_set_refuses_the_scenes_that_mc_writes_itself() {
        use chora_kit::runtime::Plugin as _;
        use serde_json::json;
        let (mut mc, _) = crate::testing::mc("own-panels", [1920, 1080]);
        for (scene, panel) in [("agents", "agent-1"), ("overview", "scenes")] {
            let before = mc.scenes[scene].panels.clone();
            let set = json!({"scene": scene, "panel": panel, "title": "Run it", "lines": ["rm"]});
            assert!(mc.method("panel.set", set).is_err(), "panel.set {scene}");
            assert!(
                mc.scenes[scene].panels == before,
                "{scene} changed although panel.set was refused"
            );
        }
        let lines = json!({"scene": "agenda", "panel": "today", "lines": ["09:00\tStand-up"]});
        mc.method("panel.set", lines).unwrap();
    }

    /// The notifications are the notification server's: `notify.push` of mc shows
    /// through `notify.show` as `chora_kit::notice` does, a critical one
    /// until the person closes it, and `notify.clear` closes what mc
    /// showed, once per id (the fake server gives both the id 7).
    #[test]
    fn notify_push_shows_through_chora_notify_and_clear_closes_it() {
        use chora_kit::{runtime::Plugin as _, testing::FakeCore};
        use serde_json::json;
        let core = FakeCore::start();
        core.answer_with("notify.show", json!({"id": 7}));
        let mut mc = crate::testing::mc_on(&core, "forward", [1280, 800]);
        let push = json!({"title": "Disk", "body": "full", "urgency": "critical"});
        mc.method("notify.push", push).unwrap();
        mc.method("notify.push", json!({"title": "Update"}))
            .unwrap();
        mc.method("notify.clear", json!({})).unwrap();
        let sent: Vec<_> = (core.model().lock().unwrap().called.iter())
            .filter(|c| c["client"] == "notify")
            .map(|c| (c["method"].clone(), c["params"].clone()))
            .collect();
        let show = |summary, body, expire_ms, urgency| {
            json!({"app": "mc", "replaces": null, "icon": "", "summary": summary,
                   "body": body, "expire_ms": expire_ms, "urgency": urgency})
        };
        assert_eq!(
            sent,
            [
                (json!("notify.show"), show("Disk", "full", 0, "critical")),
                (json!("notify.show"), show("Update", "", -1, "normal")),
                (json!("notify.close"), json!({"id": 7})),
            ]
        );
    }

    /// A key of `input.inject` turned the page, or went back along the
    /// pages, as the key of a person (review security a r2, MINOR 4): the
    /// keys of the page are the person's, as the bindings are.
    #[test]
    fn an_injected_key_does_not_turn_the_page() {
        let (mut mc, _) = crate::testing::mc("injected-page-key", [1280, 800]);
        mc.show("agenda", true).unwrap();
        press_key_from(&mut mc, "Right", &[], true);
        assert_eq!(mc.current.as_deref(), Some("agenda"), "Right injected");
        press_key_from(&mut mc, "Right", &[], false);
        assert_eq!(mc.current.as_deref(), Some("morning"), "Right of a person");
        press_key_from(&mut mc, "Escape", &[], true);
        assert_eq!(mc.current.as_deref(), Some("morning"), "Escape injected");
        press_key_from(&mut mc, "Escape", &[], false);
        assert_eq!(mc.current.as_deref(), Some("agenda"), "Escape of a person");
    }

    fn mark_opacity(model: &crate::testing::Model, id: &str) -> Option<f64> {
        Some(model.lock().unwrap().scene.node(id)?.props.opacity)
    }

    /// The hover mark of a hit shows for a pointer of a person and stays
    /// out for a pointer that `input.inject` moves.
    #[test]
    fn an_injected_pointer_event_does_not_reach_the_hover_of_mc() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("injected-hover", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let hit = "mc-s-agenda-chapter-head-hit";
        let mark = "mc-s-agenda-chapter-head-mark";
        let entered = |injected| chora_scene::Event::PointerEntered {
            node: hit.to_owned(),
            moved: true,
            injected,
        };
        mc.event(entered(true)).unwrap();
        assert_ne!(mark_opacity(&model, mark), Some(1.0), "a program hovered");
        mc.event(entered(false)).unwrap();
        assert_eq!(mark_opacity(&model, mark), Some(1.0), "a person hovered");
    }
}

/// A plugin on the fake core of the kit, for the tests of this crate.
#[cfg(test)]
mod testing {
    use chora_kit::{settings, testing::FakeCore};
    use chora_scene::{OutputInfo, Transform};

    use crate::{Mc, NAME, config};

    pub use chora_kit::testing::{Model, has, text};

    /// A plugin on a fake core with the default settings, no scene files,
    /// one output of `size` and its nodes built; and the model.
    pub fn mc(dir: &str, size: [i32; 2]) -> (Mc, Model) {
        let core = FakeCore::start();
        (mc_on(&core, dir, size), core.model())
    }

    /// As [`mc`], on a fake core of the test.
    pub fn mc_on(core: &FakeCore, dir: &str, size: [i32; 2]) -> Mc {
        let mut rt = core.runtime(NAME);
        rt.pin_accepts_injected(false);
        let dir = chora_kit::testing::TempDir::unmade(&format!("chora-mc-{dir}")).keep();
        let file = dir.join("chora-mc.yaml");
        let settings = settings::File::load(file, config::DEFAULT).unwrap();
        let mut mc = Mc::new(
            rt,
            settings,
            dir.join("scenes"),
            [dir.join("write.frag"), dir.join("blur.frag")],
        );
        mc.output = Some(OutputInfo {
            name: "test".into(),
            position: [0, 0],
            size,
            scale: 1.0,
            camera: Transform::IDENTITY,
            work_area: [0, 0, size[0], size[1]],
            default_work_area: [0; 4],
            is_virtual: None,
            owner: None,
        });
        mc.build().unwrap();
        mc
    }
}
