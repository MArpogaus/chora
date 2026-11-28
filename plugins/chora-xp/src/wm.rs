//! The window manager: a stack of windows with one focus, the taskbars,
//! the start menu and the wallpapers, driven by the events of the core.
//! The effects live in [`effects`], the desktops of the cube in
//! [`desktops`].

mod choices;
mod desktops;
mod dispatch;
mod effects;
mod float;
mod hosted;
mod items;
mod keys;
mod lifecycle;
mod output;
mod panels;
mod pointer;
mod press;
mod resize;
mod restart;
mod startup;
mod state;
mod surfaces;
mod windows;

#[cfg(test)]
mod tests_moved;

use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    time::{Duration, Instant},
};

use chora_kit::{
    actions::{System, Turn},
    give,
    icons::Icons,
    runtime::{Plugin, Runtime},
    settings::{self, Store},
    shared,
    volume::{self, Mixer, Reading},
    words::{Language, Word},
};
use chora_scene::{
    Binding, Command, Decoration, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, ShaderEffect,
    SurfaceRequest, Transform, Uniform,
    ipc::{ClientError, ExportMethod, Grab, OutputConfigure, PressState, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use self::{
    desktops::desk_id,
    effects::Effects,
    hosted::{CubeKeys, Host},
};
use crate::{
    apps,
    config::{Config, KeyAction, Look, Picture, Wallpaper},
    context::{self, Act},
    control::{self, Setting},
    cube::DESKTOPS,
    datetime::{Click, DateDialog},
    decoration::{self, Button, Part},
    desktop_icons::{self, DesktopIcons},
    dialog::{self, Hit},
    display::{self, DisplayDialog, EffectsDialog},
    help::{self, HelpDialog},
    launch::{self, Change, Launches},
    session::{self, Choice, Dialog, Mode},
    sound::{self, Sound},
    startmenu::{self, Action, Menu},
    switcher::{self, Switcher},
    taskbar::{self, Bar, Task},
    taskbar_dialog::{Outcome, TaskbarDialog},
    theme::{Rect, Theme},
    tooltip::Tooltip,
    wallpaper,
    window::{self, Window},
};

/// How often the speakers read the volume again; a change reads it at
/// once.
const VOLUME_AGAIN: Duration = Duration::from_secs(5);
/// The client name, and the group of all nodes of the plugin.
pub const NAME: &str = "xp";
/// The root group of the wallpaper and the icons, in the background order
/// (D89): the windows of another window manager on the same output show
/// above the desktop of xp, as above the page of mc.
const DESKTOP: &str = "xp-desktop";
const WALLPAPERS: &str = "xp-wallpapers";
const WINDOWS: &str = "xp-windows";
const BARS: &str = "xp-bars";
/// The settings dialogs, above the taskbar and below the menus.
const DIALOGS: &str = "xp-dialogs";
const MENUS: &str = "xp-menus";

/// The key that Shift+F10 and the Menu key stand for.
const MENU_KEY: &str = "ContextMenu";

/// The node of XP's Alt+Tab.
const SWITCHER: &str = "xp-alttab";
/// The name that the open Help and Support window has for the task
/// buttons and Alt+Tab, in the place of a surface. It has no dash, which
/// the ids of task buttons split at.
const HELP_TASK: &str = "helppanel";
/// The name that the open Display Properties has for the task buttons, as
/// [`HELP_TASK`] has for Help.
const DISPLAY_TASK: &str = "displaypanel";
/// The name that the open Date and Time Properties has for the task
/// buttons, as [`HELP_TASK`] has for Help.
const DATETIME_TASK: &str = "datetimepanel";
/// How long a taskbar that hides itself waits after the pointer left it.
const HIDE_DELAY: Duration = Duration::from_millis(500);
/// The left mouse button.
const LEFT_BUTTON: u32 = 272;
/// The right mouse button, which opens the context menus.
const RIGHT_BUTTON: u32 = 273;
/// Two presses on a title bar within this time maximize the window.
const DOUBLE_CLICK: Duration = Duration::from_millis(500);
/// How long the desktop takes to lose its color behind a dialog, as in
/// the Session Fade effect of the reference.
const FADE: Duration = Duration::from_millis(1500);

/// The events of the core that the plugin handles.
const EVENTS: [&str; 27] = [
    "client.connected",
    "client.disconnected",
    "surface.created",
    "surface.changed",
    "surface.destroyed",
    "surface.detached",
    "surface.requested",
    "pointer.pressed",
    "pointer.moved",
    "pointer.released",
    "pointer.scroll",
    "pointer.entered",
    "pointer.left",
    "output.changed",
    "output.removed",
    "input.key",
    "focus.changed",
    "focus.node",
    "node.key",
    "input.button",
    "input.moved",
    "effects.dropped",
    "node.requested",
    "voice.state",
    "voice.say",
    "voice.remove",
    "input.ungrabbed",
];

/// The methods that other clients call with the client name, as
/// `xp.turn_to`.
const METHODS: [&str; 6] = [
    "start_menu",
    "minimize",
    "restore",
    "activate",
    "restart",
    "turn_to",
];

/// The export of `window.move`. It is privileged, as `window.give` is: it
/// takes a window out of the view of the person.
fn move_method() -> ExportMethod {
    chora_kit::runtime::privileged(chora_kit::runtime::method(
        "window.move",
        "Moves a window to another output that chora-xp manages.",
        serde_json::json!({
            "surface": {"type": "string", "description": "The surface id of the window."},
            "output": {"type": "string", "description": "The name of the output."},
            "at": {
                "type": "array",
                "description": "A point of the output for the middle of the title bar.",
            },
        }),
        &["surface", "output"],
    ))
}

/// The key that stands in when another client holds a binding of the
/// cube: Super+Ctrl and the same arrow. Only a default binding with its
/// default action has a stand-in; a binding that the settings move, or
/// give another action, has none.
pub const FALLBACKS: [(&str, &str); 4] = [
    ("Ctrl+Alt+Left", "Super+Ctrl+Left"),
    ("Ctrl+Alt+Right", "Super+Ctrl+Right"),
    ("Ctrl+Alt+Shift+Left", "Super+Ctrl+Shift+Left"),
    ("Ctrl+Alt+Shift+Right", "Super+Ctrl+Shift+Right"),
];

/// The default action of a key that has a stand-in.
fn default_of(binding: &str) -> Option<KeyAction> {
    Some(match binding {
        "Ctrl+Alt+Left" => KeyAction::System(System::Cube(Turn::Left)),
        "Ctrl+Alt+Right" => KeyAction::System(System::Cube(Turn::Right)),
        "Ctrl+Alt+Shift+Left" => KeyAction::Look(Look::Carry(Turn::Left)),
        "Ctrl+Alt+Shift+Right" => KeyAction::Look(Look::Carry(Turn::Right)),
        _ => return None,
    })
}

/// The stand-in of a binding for its action, when it has one.
pub fn fallback(binding: &str, action: &KeyAction) -> Option<&'static str> {
    if default_of(binding).as_ref() != Some(action) {
        return None;
    }
    FALLBACKS
        .iter()
        .find(|(first, _)| *first == binding)
        .map(|(_, stand_in)| *stand_in)
}

/// A title bar that the pointer drags.
struct Drag {
    surface: String,
    /// The pointer at the press.
    start: [f64; 2],
    /// The window at the press.
    origin: [i32; 2],
    /// The pointer of the last move. The core reports the pointer again
    /// after each transaction, also when it did not move.
    last: [f64; 2],
    /// The core refused to give this drag away: the window stays for the
    /// rest of it.
    kept: bool,
    /// The speed of the pointer in scene pixels per second, smoothed, and
    /// the time of its last move.
    speed: [f64; 2],
    moved_at: Instant,
}

/// A border of a frame that the pointer drags.
struct Resize {
    surface: String,
    edges: window::Edges,
    /// The pointer at the press.
    start: [f64; 2],
    /// The frame at the press.
    from: Rect,
    /// The frame to ask for in the next frame of the effects.
    pending: Option<Rect>,
    /// The last frame that the pointer wanted. The core reports the
    /// pointer again after each transaction, also when it did not move.
    target: Rect,
    /// The client size of the last configure.
    asked: Option<[i32; 2]>,
    /// The button went up; the last commit of the client may follow.
    released: bool,
}

/// Puts Help and Support into the entries of the switcher: first while it
/// has the focus, so that the first step leaves it, else after the
/// windows.
fn add_help(entries: &mut Vec<switcher::Entry>, help: switcher::Entry, active: bool) {
    let at = if active { 0 } else { entries.len() };
    entries.insert(at, help);
}

/// The words of the tooltip of the helper door of the tray: "Voice off"
/// while the microphone is off, else the door of the helper machine with
/// the task of the helper.
fn helper_tip(helper: &chora_kit::helper::Helper, language: Language) -> String {
    if helper.mic_off() {
        return Word::VoiceOff.text(language).to_owned();
    }
    helper.door(language, true)
}

/// How xp came into the session (ti-38). `Joined`: another look held a
/// claim and xp had no window, so the keyboard stayed with that look; the
/// desktop takes it only when the person comes, by a click, a window of
/// xp that gets the focus or a node of xp that takes the keys. `Settled`
/// otherwise, and from then on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arrival {
    Settled,
    Joined,
}

/// The state of the window manager.
pub struct Wm {
    rt: Runtime<KeyAction>,
    theme: Theme,
    icons: Icons,
    /// The settings as the plugin applies them.
    settings: settings::File<Config>,
    /// Stretches the time of the magic lamp and of the cube, for tests
    /// and demos; 1 is the normal speed.
    slow: f64,
    /// The windows, bottom first.
    windows: Vec<Window>,
    /// The surfaces in the order they opened, for the taskbar.
    opened: Vec<String>,
    focused: Option<String>,
    /// The surface of the last `focus.changed`; read only by `state.get`.
    core_focus: Option<String>,
    /// The owner of the node with the keyboard, as `focus.node` told.
    key_node: Option<String>,
    /// Who holds the keyboard as the core tells it, through the kit: a
    /// window given in yields to a window or the desktop of another look
    /// ([`chora_kit::wm::focus::Focus::takes_given`], D119).
    heard: chora_kit::wm::focus::Focus,
    /// The focused window that the core stopped focusing without another
    /// window taking the keys, as before the window closes: when it closes,
    /// the next window gets the keys as if xp had still held them (co-51).
    dropped: Option<String>,
    /// The face at the front of the cube of the host, as its event told.
    front: usize,
    /// Whether xp joined beside another look and has had no keys since.
    arrival: Arrival,
    bars: Vec<Bar>,
    menu: Menu,
    drag: Option<Drag>,
    resize: Option<Resize>,
    last_title: Option<(String, Instant)>,
    places: window::Places,
    clock: String,
    /// The desktop that shows.
    desktop: usize,
    /// The node under the pointer.
    hover: Option<String>,
    /// The output that the pointer entered last, where a new window opens.
    pointer_output: Option<String>,
    /// The bindings of the cube that are grabbed.
    cube_keys: CubeKeys,
    /// The client that hosts the faces of a cube on the outputs, and the
    /// one that went, until it lists the same faces again.
    host: Option<Host>,
    left: Option<Host>,
    /// The windows that float off their virtual output.
    floats: float::Floats,
    effects: Effects,
    /// The Log Off or Turn Off dialog.
    dialog: Option<Dialog>,
    /// The open shell surfaces.
    open: chora_kit::wm::open::Open,
    /// When the dialog opened, while the desktop fades to gray.
    fade: Option<Instant>,
    /// The shader of the fade, in the cache.
    fade_shader: Option<String>,
    /// The icons on the desktop of the first output.
    desk: DesktopIcons,
    /// The open context menu, and the work area of its output.
    context: Option<(context::Menu, Rect)>,
    /// The window of the open system menu.
    sys_window: Option<String>,
    /// The client names of the looks that take a window, as of the last
    /// client event: the Send To rows read it, the press asks no one.
    looks: Vec<String>,
    /// The last `clients.list` failed (one warn line per streak).
    looks_failed: bool,
    /// The last press on the icon of a title bar, for the double click.
    last_icon: Option<(String, Instant)>,
    /// The settings of Taskbar and Start Menu Properties.
    taskbar: taskbar::Settings,
    /// The height that each taskbar reserves at the bottom of its screen,
    /// by output, so that other clients keep clear of it.
    reserved: HashMap<String, i32>,
    /// The settings dialogs, bottom first.
    panels: Vec<Panel>,
    /// When a taskbar that hides itself slides away.
    hide_at: Option<Instant>,
    /// The tab of Display Properties when it last closed, as XP keeps it.
    display_tab: usize,
    /// The icon or the start menu entry of the open context menu.
    item: Option<Target>,
    /// Turn Off ended the plugin.
    pub done: bool,
    /// Restart ended the plugin, to start it again.
    pub restart: bool,
    /// XP's Alt+Tab while Alt is down.
    switcher: Option<Switcher>,
    /// Alt+Escape, while the switcher holds it.
    switch_escape: Option<String>,
    /// Alt+Space, while a window of `chora-xp` has the focus.
    system_key: Option<String>,
    /// The tooltip of the part of the taskbar under the pointer.
    tip: Tooltip,
    /// Super is down alone: whether another key or button went down with it.
    tap: Option<bool>,
    /// The windows that Super+d put away, which it brings back.
    hidden: Vec<String>,
    /// Where the pointer took the cube, while Super and the left button
    /// are down.
    cube_grab: Option<f64>,
    /// The windows of clients before a restart, by surface, until they
    /// show again.
    kept: HashMap<String, restart::KeptWindow>,
    /// Which surfaces are xp's windows, and where they came from
    /// (`kit::wm::book`).
    book: chora_kit::wm::Book,
    /// The programs that started and show no window yet.
    launches: Launches,
    notepad: apps::Editors,
    /// The reading that the speaker of the taskbars shows; none before
    /// the first.
    volume: Option<Reading>,
    /// The word of `state.get`; only reads.
    voiced: chora_kit::helper::Helper,
    /// Runs wpctl off the main loop.
    mixer: Mixer,
    /// When the volume was last read.
    volume_read: Instant,
}

/// Whether the window manager acts on an event: not for a key, a press, a
/// drag or a release of `input.inject` unless it takes injected input as
/// the person's.
fn hears(event: &Event, accepts_injected: bool) -> bool {
    chora_kit::runtime::from_person(event, accepts_injected)
}

impl Plugin for Wm {
    type Action = KeyAction;

    fn runtime(&mut self) -> &mut Runtime<KeyAction> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if !hears(&event, self.rt.accepts_injected()) {
            return Ok(());
        }
        self.handle(event)
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        parse_call(method, params).and_then(|call| self.call(call))
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn state(&self) -> Option<Value> {
        Some(self.state_value())
    }

    /// New bindings change the grabs; a binding that stays is never free in
    /// between.
    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.rt.start_grabs();
        self.grab()
            .and_then(|()| self.rt.end_grabs())
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        let commands = self.configure();
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

impl give::Windows for Wm {
    fn conn(&mut self) -> &mut chora_kit::conn::Conn {
        &mut self.rt.conn
    }

    fn has(&self, surface: &str) -> bool {
        self.book.has(surface)
    }

    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        let (answer, commands) = self.give(surface, to)?;
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(answer)
    }

    fn hand_kept(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        let (answer, commands) = self.give_kept(surface, to)?;
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(answer)
    }
}

impl Wm {}

/// A settings dialog.
enum Panel {
    /// Taskbar and Start Menu Properties.
    Taskbar(TaskbarDialog),
    /// Date and Time Properties.
    Clock(DateDialog),
    /// Display Properties.
    Display(DisplayDialog),
    /// Effects of Display Properties.
    Effects(EffectsDialog),
    /// Help and Support.
    Help(HelpDialog),
}

/// What a context menu of an icon or of the start menu acts on.
enum Target {
    /// An entry of the start menu.
    Start(startmenu::Item),
    /// An icon of the desktop.
    Icon(desktop_icons::Item),
}

impl Panel {
    fn window(&mut self) -> &mut dialog::Dialog {
        match self {
            Self::Taskbar(panel) => &mut panel.dialog,
            Self::Clock(panel) => &mut panel.dialog,
            Self::Display(panel) => &mut panel.dialog,
            Self::Effects(panel) => &mut panel.dialog,
            Self::Help(panel) => &mut panel.dialog,
        }
    }

    fn taskbar(&mut self) -> Option<&mut TaskbarDialog> {
        match self {
            Self::Taskbar(panel) => Some(panel),
            _ => None,
        }
    }

    fn window_ref(&self) -> &dialog::Dialog {
        match self {
            Self::Taskbar(panel) => &panel.dialog,
            Self::Clock(panel) => &panel.dialog,
            Self::Display(panel) => &panel.dialog,
            Self::Effects(panel) => &panel.dialog,
            Self::Help(panel) => &panel.dialog,
        }
    }
}

/// What Super+d does.
#[derive(Debug, PartialEq, Eq)]
enum Plan {
    /// Puts these windows away.
    Away(Vec<String>),
    /// Brings these windows back.
    Back(Vec<String>),
}

/// Show the desktop: the windows that it put away come back while at
/// least one of them is still away; else the windows that show go away.
/// `away` lists each window and whether it is minimized.
fn desktop_plan(hidden: &[String], shown: &[String], away: &[(&str, bool)]) -> Plan {
    let still: Vec<String> = hidden
        .iter()
        .filter(|h| away.iter().any(|(s, minimized)| s == h && *minimized))
        .cloned()
        .collect();
    if still.is_empty() {
        Plan::Away(shown.to_vec())
    } else {
        Plan::Back(still)
    }
}

/// Whether the watchdog of the core named chora-xp, and not another
/// client, when it dropped the effects.
fn is_own(client: &str) -> bool {
    client == NAME
}

/// An output as xp lays it out: the height that its taskbar reserves
/// stays in the work area.
fn without_reserve(reserved: &HashMap<String, i32>, mut output: OutputInfo) -> OutputInfo {
    let height = reserved.get(&output.name).copied().unwrap_or(0);
    for area in [&mut output.work_area, &mut output.default_work_area] {
        if *area != [0; 4] {
            area[3] += height;
        }
    }
    output
}

/// A call of a method that `chora-xp` exports.
enum Call {
    /// Opens or closes the start menu, like its button.
    StartMenu {
        /// Whether the menu should be open afterwards.
        open: bool,
    },
    /// Pours a window into its task button, like the click on it.
    Minimize {
        /// The surface.
        surface: String,
    },
    /// Pours a window back and focuses it, like the click on its task
    /// button.
    Restore {
        /// The surface.
        surface: String,
    },
    /// Raises a window and focuses it, like a click on its task button.
    Activate {
        /// The surface.
        surface: String,
    },
    /// Restarts the plugin, like Restart in the Turn Off dialog.
    Restart,
    /// Turns the cube to a desktop, like its button in the pager.
    TurnTo {
        /// The desktop, from 0 to 3.
        desktop: usize,
    },
    /// Lets the windows of a virtual output float, or land.
    Float(chora_kit::float::Float),
    /// Moves a window to another output of the plugin, like a drag of
    /// its title bar onto it.
    Move {
        /// The surface.
        surface: String,
        /// The name of the output.
        output: String,
        /// The point of the output where the title bar goes.
        at: Option<[i32; 2]>,
    },
    /// Hands a window to another window manager, like a drag out of the
    /// work areas.
    Give(give::Give),
}

/// Parameters of `xp.window.move`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveTo {
    surface: String,
    output: String,
    /// The point of the output where the title bar goes, as a drop of a
    /// drag there.
    #[serde(default)]
    at: Option<[i32; 2]>,
}

/// Parameters of `xp.start_menu`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartMenu {
    open: bool,
}

/// Parameters of `xp.minimize`, `xp.restore` and `xp.activate`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForSurface {
    surface: String,
}

/// Parameters of `xp.turn_to`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TurnTo {
    desktop: usize,
}

/// Parses the method and the parameters of a `method.called` event.
fn parse_call(method: &str, params: Value) -> Result<Call, RpcError> {
    Ok(match method {
        "start_menu" => Call::StartMenu {
            open: parse::<StartMenu>(params)?.open,
        },
        "minimize" => Call::Minimize {
            surface: parse::<ForSurface>(params)?.surface,
        },
        "restore" => Call::Restore {
            surface: parse::<ForSurface>(params)?.surface,
        },
        "activate" => Call::Activate {
            surface: parse::<ForSurface>(params)?.surface,
        },
        "restart" => Call::Restart,
        "window.move" => {
            let MoveTo {
                surface,
                output,
                at,
            } = parse(params)?;
            Call::Move {
                surface,
                output,
                at,
            }
        }
        give::METHOD => Call::Give(parse(params)?),
        chora_kit::float::METHOD => Call::Float(parse(params)?),
        "turn_to" => {
            let TurnTo { desktop } = parse(params)?;
            if desktop >= DESKTOPS {
                return Err(RpcError::new(
                    RpcError::INVALID_PARAMS,
                    format!("no desktop {desktop}"),
                ));
            }
            Call::TurnTo { desktop }
        }
        other => {
            return Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("xp has no method {other}"),
            ));
        }
    })
}

/// Parses the parameters of an exported method.
fn parse<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, RpcError> {
    serde_json::from_value(params)
        .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))
}

/// Fails the method unless `chora-xp` manages the window.
fn known(found: bool, surface: &str) -> Result<(), RpcError> {
    found
        .then_some(())
        .ok_or_else(|| RpcError::new(RpcError::REJECTED, format!("xp has no window {surface}")))
}

fn close(surface: &str) -> Vec<Command> {
    vec![Command::Close {
        surface: surface.into(),
    }]
}

/// The largest scale of some outputs, at least 1.
fn max_scale(outputs: &[OutputInfo]) -> f64 {
    outputs.iter().map(|o| o.scale).fold(1.0, f64::max)
}

/// A position in one area at the same place in another, so that at least
/// the title bar stays inside it.
/// The taskbar of the output for a new window: the virtual output that
/// the pointer entered last, else the first output. Screens keep the
/// first one, as before virtual outputs.
fn home_bar<'a>(bars: &'a [Bar], entered: Option<&str>) -> Option<&'a Bar> {
    let under_the_pointer = |b: &&Bar| b.is_virtual() && Some(b.output.as_str()) == entered;
    bars.iter().find(under_the_pointer).or(bars.first())
}

/// The place of a frame whose title bar was dropped at a point of an
/// output that shows no part of it, as a drag does that ends on another
/// virtual output: under the pointer, which holds it at `grab` from its
/// corner. `None` when the output shows a part of the frame.
fn dropped_on(frame: Rect, area: Rect, point: [i32; 2], grab: [i32; 2]) -> Option<[i32; 2]> {
    let apart = |axis: usize| {
        frame[axis] + frame[axis + 2] <= area[axis] || area[axis] + area[axis + 2] <= frame[axis]
    };
    (apart(0) || apart(1)).then(|| [point[0] - grab[0], point[1] - grab[1]])
}

/// The place of a frame of a width whose title bar was dropped at a point
/// of an output with the area `area`: the middle of the title bar under
/// the point, and at least a part of the title bar inside the area.
fn dropped_at([ax, ay, aw, ah]: Rect, width: i32, [px, py]: [i32; 2]) -> [i32; 2] {
    let keep = 64;
    let x = ax + px - width / 2;
    let y = ay + py - decoration::TITLE / 2;
    [
        x.clamp(ax + keep - width, ax + (aw - keep).max(0)),
        y.clamp(ay, ay + (ah - decoration::TITLE).max(0)),
    ]
}

/// The most speed of a drag on an axis, in pixels per second.
const MAX_SPEED: f64 = 20_000.0;

/// The speed of a drag after a move of the pointer in a time: half the
/// speed before and half that of the move, so that one late event does
/// not decide it. A move within a millisecond keeps the speed before.
fn speed(before: [f64; 2], from: [f64; 2], to: [f64; 2], took: Duration) -> [f64; 2] {
    let seconds = took.as_secs_f64();
    if seconds < 0.001 {
        return before;
    }
    [0, 1].map(|axis| {
        let now = ((to[axis] - from[axis]) / seconds).clamp(-MAX_SPEED, MAX_SPEED);
        f64::midpoint(before[axis], now)
    })
}

/// How far past the edge of its face the pointer goes before a drag
/// gives its window away, in pixels: a drag along the edge stays.
const EXIT_MARGIN: i32 = 8;

/// The client that gets a window whose drag started on a face: the owner
/// of the virtual output, once the pointer is more than [`EXIT_MARGIN`]
/// outside it. A screen has no owner, and a drag there gives at the
/// release.
fn pulled_out(face: Rect, owner: Option<&str>, [px, py]: [i32; 2]) -> Option<&str> {
    let [left, top, w, h] = face;
    let across = left - EXIT_MARGIN..left + w + EXIT_MARGIN;
    let down = top - EXIT_MARGIN..top + h + EXIT_MARGIN;
    let near = across.contains(&px) && down.contains(&py);
    owner.filter(|_| !near)
}

fn moved([px, py]: [i32; 2], [fx, fy, ..]: Rect, [tx, ty, tw, th]: Rect) -> [i32; 2] {
    let keep = 64;
    [
        (tx + px - fx).clamp(tx, tx + (tw - keep).max(0)),
        (ty + py - fy).clamp(ty, ty + (th - keep).max(0)),
    ]
}

/// The application ids of terminals, which show XP's command prompt.
const TERMINALS: [&str; 9] = [
    "foot",
    "footclient",
    "xterm",
    "alacritty",
    "kitty",
    "konsole",
    "wezterm",
    "gnome-terminal",
    "org.wezfurlong.wezterm",
];

/// The icon of a window for its title bar and task button, 16 pixels: a
/// terminal gets the command prompt, a program its own icon of that size,
/// and a program without art the generic program icon of the theme.
fn window_icon(icons: &mut Icons, app_id: Option<&str>) -> Option<std::path::PathBuf> {
    let id = app_id.unwrap_or_default();
    let lower = id.to_ascii_lowercase();
    if let Some(prompt) = TERMINALS
        .contains(&lower.as_str())
        .then(|| icons.find_sized("utilities-terminal", 16))
        .flatten()
    {
        return Some(prompt);
    }
    icons
        .find_sized(id, 16)
        .filter(|file| !crate::startmenu::is_fragment(file))
        .or_else(|| icons.find_sized("applications-other", 16))
        .or_else(|| icons.program_sized(id, 16))
}

/// Whether an action is one of the cube: a turn, a carry or the pointer.
fn of_the_cube(action: &KeyAction) -> bool {
    matches!(
        action,
        KeyAction::System(System::Cube(_)) | KeyAction::Look(Look::Carry(_) | Look::Cube)
    )
}

/// Grabs the bindings of the settings with those of the cube that `cube`
/// names; a binding of the cube that another client holds falls back to
/// its stand-in. Xp is the window manager of its screen or face, so it
/// swallows the Super chords that it binds no action for: no window gets
/// them (xp-46).
fn grab_keys(
    rt: &mut Runtime<KeyAction>,
    bindings: &BTreeMap<String, KeyAction>,
    cube: CubeKeys,
) -> Result<(), ClientError> {
    grab_keys_of(rt, bindings, cube, |_| true)
}

/// Whether the core grabs a binding only for a client that holds a claim:
/// a pointer button, which window managers share by their claims (D55),
/// and a media key.
fn needs_claim(binding: &str) -> bool {
    binding.starts_with("XF86")
        || binding
            .parse::<Binding>()
            .is_ok_and(|b| b.key.to_ascii_uppercase().starts_with("BTN_"))
}

/// As [`grab_keys`], for the bindings that `wanted` accepts.
#[allow(clippy::expect_used, reason = "the settings check their bindings")]
fn grab_keys_of(
    rt: &mut Runtime<KeyAction>,
    bindings: &BTreeMap<String, KeyAction>,
    cube: CubeKeys,
    wanted: impl Fn(&str) -> bool,
) -> Result<(), ClientError> {
    let held = |action: &KeyAction| {
        !matches!(cube, CubeKeys::Flat | CubeKeys::Hosted) || !of_the_cube(action)
    };
    let chosen = bindings.iter().filter(|(t, a)| held(a) && wanted(t));
    for (text, action) in chosen {
        for text in std::iter::once(text.as_str()).chain(fallback(text, action)) {
            let binding: Binding = text.parse().expect("the settings check their bindings");
            if rt.grab(binding, action.clone())? {
                break;
            }
        }
    }
    rt.grab_swallowed("Super", &KeyAction::Look(Look::Swallow))
}

/// Changes the grabs to the keys with or without those of the cube. A
/// key of both sets stays with the core, so it never reaches a window.
fn change_keys(
    rt: &mut Runtime<KeyAction>,
    bindings: &BTreeMap<String, KeyAction>,
    cube: CubeKeys,
) -> Result<(), ClientError> {
    rt.start_grabs();
    grab_keys(rt, bindings, cube)?;
    rt.end_grabs()
}

#[cfg(test)]
mod tests {
    #[test]
    fn show_desktop_puts_windows_away_and_brings_the_same_ones_back() {
        let shown = vec!["s1".to_string(), "s2".to_string()];
        let both_up = [("s1", false), ("s2", false), ("s3", true)];
        assert_eq!(
            desktop_plan(&[], &shown, &both_up),
            Plan::Away(shown.clone())
        );
        let hidden = shown.clone();
        let away = [("s1", true), ("s2", true), ("s3", true)];
        assert_eq!(
            desktop_plan(&hidden, &[], &away),
            Plan::Back(hidden.clone())
        );
        let one_back = [("s1", false), ("s2", true), ("s3", true)];
        assert_eq!(
            desktop_plan(&hidden, &["s1".to_string()], &one_back),
            Plan::Back(vec!["s2".to_string()])
        );
        let all_back = [("s1", false), ("s2", false)];
        assert_eq!(desktop_plan(&hidden, &shown, &all_back), Plan::Away(shown));
    }

    /// A click on Start or on a menu entry started a program, and the
    /// Menu key opened the context menu, from `input.inject` and without a
    /// person (review security a r2, MINOR 5).
    type Make = fn(bool) -> Event;

    #[test]
    fn an_injected_press_or_key_does_nothing_in_xp() {
        let press = |injected| Event::PointerPressed {
            node: "xp-start".into(),
            button: 272,
            position: [0.0; 2],
            local: [0.0; 2],
            modifiers: Vec::new(),
            injected,
        };
        let release = |injected| Event::PointerReleased {
            node: "xp-menu-item".into(),
            button: 272,
            position: [0.0; 2],
            local: [0.0; 2],
            modifiers: Vec::new(),
            injected,
            time: 0,
        };
        let menu_key = |injected| Event::NodeKey {
            node: "xp-look".into(),
            key: "Menu".into(),
            code: 127,
            state: PressState::Pressed,
            modifiers: Vec::new(),
            text: None,
            injected,
        };
        let makers: [(&str, Make); 3] = [
            ("a press", press),
            ("a release", release),
            ("the Menu key", menu_key),
        ];
        for (name, make) in makers {
            assert!(hears(&make(false), false), "{name} of the person");
            assert!(!hears(&make(true), false), "{name} injected");
            assert!(hears(&make(true), true), "{name} injected in a test");
        }
    }

    #[test]
    fn the_keys_of_the_other_looks_are_bound_here_too() {
        let bound = crate::config::defaults().common.bindings;
        for key in [
            "Super+Super_L",
            "Super+d",
            "Super+space",
            "Super+e",
            "Super+a",
        ] {
            assert!(bound.contains_key(key), "{key}");
        }
        assert_eq!(bound["Super+e"], KeyAction::Look(Look::Files));
    }

    use chora_kit::{
        testing::{FakeCore, Log},
        volume::Change,
    };
    use serde_json::{Value, json};

    use super::*;

    /// The calls `input.grab` and `input.ungrab` that a fake core saw,
    /// as `METHOD BINDING`, and a runtime on that core.
    fn on_fake_core(tag: &str) -> (Runtime<KeyAction>, Log) {
        on_fake_core_refusing(tag, &[])
    }

    /// As [`on_fake_core`], with a core that refuses the grab of the
    /// `refused` bindings, as when another client holds them.
    fn on_fake_core_refusing(
        tag: &str,
        refused: &'static [&'static str],
    ) -> (Runtime<KeyAction>, Log) {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-xp-{tag}")).keep();
        std::fs::create_dir_all(&dir).unwrap();
        let core = FakeCore::start();
        for binding in refused {
            core.hold(binding);
        }
        let calls = core.record(|method, params| {
            let binding = params["binding"].as_str().unwrap_or_default();
            method
                .starts_with("input.")
                .then(|| format!("{method} {binding}"))
        });
        (core.runtime(NAME), calls)
    }

    /// A window manager on a fake core that answers `scene.query` with no
    /// nodes and every other request with null, and the methods that it saw.
    ///
    /// The windows of these cells are not in the scene: the fake core takes
    /// the commands as they come, and the test reads them.
    pub(super) fn wm_on_fake_core(tag: &str) -> (Wm, Log) {
        wm_on_fake_core_logging(tag, |method, _| Some(method.to_owned()))
    }

    /// As [`wm_on_fake_core`], the log holding what `line` makes of each request.
    #[allow(
        clippy::disallowed_methods,
        reason = "the test answers the requests of its fake core"
    )]
    pub(super) fn wm_on_fake_core_logging(
        tag: &str,
        line: impl Fn(&str, &Value) -> Option<String> + Send + Sync + 'static,
    ) -> (Wm, Log) {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-xp-wm-{tag}")).keep();
        std::fs::create_dir_all(&dir).unwrap();
        let output = chora_scene::OutputInfo {
            name: "X".into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: chora_scene::Transform::default(),
            work_area: [0, 0, 1280, 800],
            default_work_area: [0, 0, 1280, 800],
            is_virtual: None,
            owner: None,
        };
        let core = FakeCore::start();
        core.respond(move |method, _| match method {
            "output.list" => Some(json!([output])),
            "scene.apply" => Some(Value::Null),
            _ => None,
        });
        let methods = core.record(line);
        (wm_on(core.runtime(NAME), &dir), methods)
    }

    /// A window manager on the runtime `rt`, with the theme and the settings of a
    /// start in `dir`.
    fn wm_on(mut rt: Runtime<KeyAction>, dir: &Path) -> Wm {
        rt.pin_accepts_injected(false);
        let theme_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme");
        let theme = Theme::load(
            &theme_dir,
            &dir.join("cache"),
            &crate::sheets::ALL,
            crate::sheets::FAMILY,
        )
        .expect("the theme loads");
        let file = dir.join("chora-xp.yaml");
        let settings = settings::File::<Config>::load(file, crate::config::DEFAULT).unwrap();
        Wm::start_on(rt, theme, settings, 1.0).expect("xp starts on the fake core")
    }

    /// A window manager on the fake core of `chora_kit::testing`, which
    /// keeps the scene model that the script of the life of a window
    /// reads.
    pub(super) fn wm_on_kit_core(core: &chora_kit::testing::FakeCore, tag: &str) -> Wm {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-xp-kit-{tag}")).keep();
        std::fs::create_dir_all(&dir).unwrap();
        wm_on(core.runtime(NAME), &dir)
    }

    /// An event as the core sends it.
    fn wire(event: Value) -> Event {
        serde_json::from_value(event).expect("an event of the core")
    }

    #[test]
    fn an_injected_pointer_event_changes_nothing_in_xp() {
        let (mut wm, methods) = wm_on_fake_core("injected-pointer");
        let entered = |injected| {
            wire(
                json!({"event": "pointer.entered", "node": "xp-start", "moved": true,
                "injected": injected}),
            )
        };
        let seen = methods.lock().unwrap().len();
        wm.event(entered(true)).unwrap();
        assert_eq!(wm.hover, None, "an injected entry");
        assert_eq!(methods.lock().unwrap().len(), seen, "nothing sent");
        wm.event(entered(false)).unwrap();
        assert_eq!(
            wm.hover.as_deref(),
            Some("xp-start"),
            "the entry of the person"
        );
    }

    #[test]
    fn an_injected_button_grab_turns_no_cube_in_xp() {
        let (mut wm, _methods) = wm_on_fake_core("injected-cube");
        assert!(!wm.bars.is_empty(), "xp has an output");
        let press = |injected| {
            wire(json!({"event": "input.button", "binding": "Super+BTN_LEFT",
                "state": "pressed", "position": [100.0, 100.0], "output": "X",
                "local": [100.0, 100.0], "injected": injected}))
        };
        wm.event(press(true)).unwrap();
        assert_eq!(wm.cube_grab, None, "an injected press");
        wm.event(press(false)).unwrap();
        assert_eq!(wm.cube_grab, Some(100.0), "the press of the person");
    }

    /// On virtual outputs alone without a cube host, as in a pane, xp
    /// has no desktops to switch: it holds no turn key, so another
    /// client may, and no drag turns anything.
    #[test]
    fn without_a_cube_xp_holds_no_turn_key_and_no_drag() {
        let (mut rt, _calls) = on_fake_core("flat-keys");
        let bindings = crate::config::defaults().common.bindings;
        grab_keys(&mut rt, &bindings, CubeKeys::Flat).unwrap();
        assert!(rt.action("Ctrl+Alt+Left").is_none(), "no turn key");
        assert!(
            rt.action("Super+BTN_LEFT").is_none(),
            "no drag without a cube"
        );
    }

    /// On a face of a cube, xp is the window manager: a Super chord that
    /// it binds no action for is its own, so no window gets it. A chord
    /// it binds keeps its action; the same holds on a screen of its own.
    #[test]
    fn on_a_face_xp_swallows_the_super_chords_it_binds_nothing_for() {
        for cube in [CubeKeys::Flat, CubeKeys::Hosted] {
            let (mut rt, _calls) = on_fake_core("face-swallow");
            let bindings = crate::config::defaults().common.bindings;
            rt.start_grabs();
            grab_keys(&mut rt, &bindings, cube).unwrap();
            rt.end_grabs().unwrap();
            for chord in [
                "Super+q",
                "Super+Tab",
                "Super+1",
                "Shift+Super+x",
                "Ctrl+Super+z",
            ] {
                assert!(rt.action(chord).is_some(), "{cube:?}: {chord} is not held");
            }
            assert_eq!(
                rt.action("Super+d"),
                Some(&KeyAction::Look(Look::Desktop)),
                "{cube:?}: a bound chord keeps its action"
            );
            assert_eq!(
                rt.action("Super+e"),
                Some(&KeyAction::Look(Look::Files)),
                "{cube:?}"
            );
        }
    }

    /// xp-46 (rpx1-1): on a screen of its own xp swallows them too.
    #[test]
    fn on_a_screen_xp_swallows_the_super_chords_it_binds_nothing_for() {
        let (mut rt, _calls) = on_fake_core("own-swallow");
        let bindings = crate::config::defaults().common.bindings;
        grab_keys(&mut rt, &bindings, CubeKeys::Own).unwrap();
        for chord in ["Super+x", "Super+k", "Super+m", "Super+q"] {
            assert!(
                rt.action(chord).is_some(),
                "xp-46: {chord} is free on a screen of xp, so it types into a window"
            );
        }
        assert!(rt.action("Alt+q").is_none(), "another modifier is left");
    }

    /// A press of the binding `Super+BTN_RIGHT` that the user moved to
    /// `close`: the focused window closes, and no cube turns (rw-1).
    #[test]
    fn a_button_bound_to_close_closes_and_turns_no_cube() {
        let (mut wm, methods) = wm_on_fake_core("button-close");
        wm.windows
            .push(crate::window::Window::new("s1", None, Some("s1")));
        wm.focused = Some("s1".into());
        let binding: Binding = "Super+BTN_RIGHT".parse().unwrap();
        wm.rt
            .grab(binding, KeyAction::System(System::Close))
            .unwrap();
        let press = |state| {
            wire(
                json!({"event": "input.button", "binding": "Super+BTN_RIGHT",
                "state": state, "position": [100.0, 100.0], "output": "X",
                "local": [100.0, 100.0]}),
            )
        };
        let seen = methods.lock().unwrap().len();
        wm.event(press("pressed")).unwrap();
        assert_eq!(wm.cube_grab, None, "no turn of the cube");
        assert!(wm.effects.turn.is_none(), "no turn of the cube");
        let sent = methods.lock().unwrap()[seen..].to_vec();
        assert!(
            sent.iter().any(|m| m.starts_with("scene.")),
            "the close went out: {sent:?}"
        );
        wm.event(press("released")).unwrap();
        assert!(wm.effects.turn.is_none(), "the release turns nothing");
    }

    /// A key that `bindings` moves to a stand-in-less action gets no
    /// Super+Ctrl stand-in when the core refuses it (rw-2): the stand-in is
    /// for the cube alone.
    #[test]
    fn a_moved_cube_key_has_no_stand_in() {
        let (mut rt, calls) = on_fake_core_refusing("no-stand-in", &["Ctrl+Alt+Left"]);
        let mut bindings = crate::config::defaults().common.bindings;
        bindings.insert("Ctrl+Alt+Left".into(), KeyAction::System(System::Close));
        grab_keys(&mut rt, &bindings, CubeKeys::Own).unwrap();
        let grabs = (calls.lock().unwrap().iter())
            .filter(|c| *c == "input.grab Ctrl+Super+Left")
            .count();
        assert_eq!(
            grabs, 1,
            "only the swallow grabs Ctrl+Super+Left, no stand-in"
        );
        assert_eq!(
            rt.action("Ctrl+Super+Left"),
            Some(&KeyAction::Look(Look::Swallow)),
            "no turn of the cube, only the swallow of the free chord"
        );
        let (mut rt, _calls) = on_fake_core_refusing("stand-in", &["Ctrl+Alt+Left"]);
        let bindings = crate::config::defaults().common.bindings;
        grab_keys(&mut rt, &bindings, CubeKeys::Own).unwrap();
        assert_eq!(
            rt.action("Ctrl+Super+Left"),
            Some(&KeyAction::System(System::Cube(Turn::Left)))
        );
    }

    /// When the cube of xp goes, as on the faces of a cube of the
    /// construct, xp lets only the keys of the cube go: a key that it
    /// keeps is never free in between, where it would reach a window.
    #[test]
    fn a_change_of_the_cube_keys_keeps_the_other_keys() {
        let (mut rt, calls) = on_fake_core("cube-keys");
        let bindings = crate::config::defaults().common.bindings;
        grab_keys(&mut rt, &bindings, CubeKeys::Own).unwrap();
        calls.lock().unwrap().clear();
        change_keys(&mut rt, &bindings, CubeKeys::Hosted).unwrap();
        let sent = std::mem::take(&mut *calls.lock().unwrap());
        assert!(
            !sent.iter().any(|c| c.ends_with(" Super+q")),
            "the swallowed chords stay grabbed in between: {sent:?}"
        );
        let mut gone: Vec<String> = sent
            .into_iter()
            .filter(|c| c.starts_with("input.ungrab"))
            .collect();
        gone.sort();
        assert_eq!(
            gone,
            [
                "input.ungrab Ctrl+Alt+Left",
                "input.ungrab Ctrl+Alt+Right",
                "input.ungrab Ctrl+Alt+Shift+Left",
                "input.ungrab Ctrl+Alt+Shift+Right",
                "input.ungrab Super+BTN_LEFT",
            ]
        );
        assert_eq!(rt.action("Super+d"), Some(&KeyAction::Look(Look::Desktop)));
        assert!(rt.action("Ctrl+Alt+Left").is_none());
        change_keys(&mut rt, &bindings, CubeKeys::Own).unwrap();
        assert_eq!(
            rt.action("Ctrl+Alt+Left"),
            Some(&KeyAction::System(System::Cube(Turn::Left)))
        );
        assert_eq!(
            rt.action("Super+BTN_LEFT"),
            Some(&KeyAction::Look(Look::Cube))
        );
        let back = calls.lock().unwrap().clone();
        for kept in ["Super+d", "Super+e", "Super+F1"] {
            assert!(
                !back.iter().any(|c| c.ends_with(&format!(" {kept}"))),
                "{kept} is free in between: {back:?}"
            );
        }
    }

    /// A key does what its binding says, whatever the spelling: Super+F1
    /// and Super+/ open the key list, and the power key the Turn Off
    /// dialog of xp rather than the shared menu.
    #[test]
    fn a_key_does_the_action_of_its_binding() {
        let (mut wm, _methods) = wm_on_fake_core("keys-act");
        for key in ["Super+F1", "Super+slash"] {
            wm.panels.clear();
            let _ = wm.key(key);
            assert!(
                wm.panels.iter().any(|p| matches!(p, Panel::Help(_))),
                "{key}"
            );
        }
        assert!(wm.dialog.is_none());
        let power: Binding = "Super+Shift+e".parse().unwrap();
        let _ = wm.key(&power.to_string());
        assert!(wm.dialog.is_some(), "the Turn Off dialog");
    }

    /// The volume keys and the chat key are bound as on tiling.
    #[test]
    fn the_media_keys_and_the_chat_key_are_bound() {
        let bound = crate::config::defaults().common.bindings;
        let volume = |change| KeyAction::System(System::Volume(change));
        assert_eq!(bound["XF86AudioRaiseVolume"], volume(Change::Up));
        assert_eq!(bound["XF86AudioLowerVolume"], volume(Change::Down));
        assert_eq!(bound["XF86AudioMute"], volume(Change::Mute));
        assert_eq!(bound["Super+a"], KeyAction::System(System::Chat));
    }

    /// A window that comes back from the space lands with the middle of
    /// its title bar at the drop point, and never wholly off its output.
    #[test]
    fn a_dropped_window_has_its_title_bar_under_the_drop_point() {
        let area = [-262_144, -262_144, 1280, 770];
        let title = decoration::TITLE;
        let at = dropped_at(area, 228, [700, 400]);
        assert_eq!(at, [-262_144 + 586, -262_144 + 400 - title / 2]);
        let corner = dropped_at(area, 228, [-500, -500]);
        assert_eq!(corner, [-262_144 + 64 - 228, -262_144]);
        let far = dropped_at(area, 228, [5000, 5000]);
        assert_eq!(far, [-262_144 + 1280 - 64, -262_144 + 770 - title]);
    }

    /// The drag that goes with a window carries the speed of the pointer:
    /// smoothed over the moves, kept through a burst, and capped.
    #[test]
    fn a_drag_knows_the_speed_of_its_pointer() {
        let ms = Duration::from_millis;
        let first = speed([0.0; 2], [100.0, 100.0], [110.0, 100.0], ms(10));
        assert_eq!(first, [500.0, 0.0], "half of 1000 px/s after rest");
        let second = speed(first, [110.0, 100.0], [120.0, 95.0], ms(10));
        assert_eq!(second, [750.0, -250.0]);
        assert_eq!(speed(second, [0.0; 2], [50.0, 0.0], ms(0)), second);
        let jump = speed([0.0; 2], [0.0; 2], [1e9, 0.0], ms(10));
        assert_eq!(jump, [MAX_SPEED / 2.0, 0.0]);
    }

    /// E3: a drag that leaves its face gives the window to the client
    /// that shows the face; on a screen and inside the face nothing goes.
    #[test]
    fn a_drag_out_of_a_face_goes_to_the_owner_of_the_face() {
        let face = [-262_144, -262_144, 800, 600];
        let construct = Some("construct");
        assert_eq!(pulled_out(face, construct, [-262_000, -262_000]), None);
        assert_eq!(pulled_out(face, construct, [-261_344, -262_000]), None);
        assert_eq!(pulled_out(face, construct, [-262_000, -262_145]), None);
        let past = EXIT_MARGIN + 1;
        let right = [-261_344 + past, -262_000];
        assert_eq!(pulled_out(face, construct, right), construct, "the margin");
        let above = [-262_000, -262_144 - past];
        assert_eq!(pulled_out(face, construct, above), construct, "the margin");
        assert_eq!(pulled_out([0, 0, 800, 600], None, [900, 100]), None);
    }

    #[test]
    fn every_exported_method_parses_its_parameters() {
        assert!(matches!(
            parse_call("start_menu", serde_json::json!({"open": true})),
            Ok(Call::StartMenu { open: true })
        ));
        for method in ["minimize", "restore", "activate"] {
            let call = parse_call(method, serde_json::json!({"surface": "s1"})).expect(method);
            match call {
                Call::Minimize { surface }
                | Call::Restore { surface }
                | Call::Activate { surface } => assert_eq!(surface, "s1"),
                _ => panic!("{method} parsed as another call"),
            }
        }
        assert!(matches!(
            parse_call("restart", Value::Null),
            Ok(Call::Restart)
        ));
        assert!(matches!(
            parse_call("turn_to", serde_json::json!({"desktop": 2})),
            Ok(Call::TurnTo { desktop: 2 })
        ));
    }

    #[test]
    fn a_bad_call_fails_with_the_right_rpc_error() {
        // The cube has four faces.
        for (method, params, code) in [
            ("turn_to", Value::Null, RpcError::INVALID_PARAMS),
            ("start_menu", Value::Null, RpcError::INVALID_PARAMS),
            (
                "turn_to",
                serde_json::json!({"desktop": 4}),
                RpcError::INVALID_PARAMS,
            ),
            ("log_off", Value::Null, RpcError::METHOD_NOT_FOUND),
        ] {
            let err = parse_call(method, params)
                .err()
                .unwrap_or_else(|| panic!("{method} passed"));
            assert_eq!(err.code, code, "{method}: {err}");
        }
    }

    #[test]
    fn only_a_drop_of_its_own_effects_calms_the_plugin() {
        assert!(is_own("xp"));
        assert!(!is_own("tiling"));
        assert!(!is_own(""));
    }

    #[test]
    fn a_window_moves_to_the_same_place_on_another_output() {
        let (from, to) = ([1920, 0, 2560, 1400], [0, 0, 1920, 1040]);
        assert_eq!(moved([2020, 100], from, to), [100, 100]);
        // Beyond the new area, the title bar stays inside it.
        assert_eq!(moved([4300, 1300], from, to), [1856, 976]);
    }

    #[test]
    fn a_frame_dropped_on_a_far_output_lands_under_the_pointer() {
        let face = [-786_432, -262_144, 600, 350];
        let left_behind = [-262_500, -262_100, 300, 200];
        assert_eq!(
            dropped_on(left_behind, face, [-786_232, -262_044], [60, 10]),
            Some([-786_292, -262_054])
        );
        let straddling = [-786_500, -262_100, 300, 200];
        assert_eq!(
            dropped_on(straddling, face, [-786_400, -262_090], [60, 10]),
            None
        );
    }

    fn bar(name: &str, is_virtual: bool) -> Bar {
        Bar::new(&OutputInfo {
            name: name.into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: chora_scene::Transform::default(),
            work_area: [0, 0, 1280, 800],
            default_work_area: [0, 0, 1280, 800],
            is_virtual: is_virtual.then_some(true),
            owner: None,
        })
    }

    #[test]
    fn a_taskbar_of_a_screen_reserves_its_height_and_xp_lays_out_without_it() {
        let mut screen = bar("DP-1", false);
        assert_eq!(screen.reserve(), taskbar::HEIGHT);
        assert_eq!(bar("VIRTUAL-1", true).reserve(), 0, "a face of the cube");
        screen.configure(taskbar::Settings {
            locked: true,
            auto_hide: true,
            on_top: true,
            clock: true,
        });
        assert_eq!(screen.reserve(), taskbar::HIDDEN_STRIP);
        let reserved: HashMap<String, i32> = [("DP-1".to_owned(), 30)].into();
        let output = OutputInfo {
            name: "DP-1".into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: chora_scene::Transform::default(),
            work_area: [0, 0, 1280, 770],
            default_work_area: [0, 0, 1280, 770],
            is_virtual: None,
            owner: None,
        };
        let own = without_reserve(&reserved, output.clone());
        assert_eq!(
            (own.work_area, own.default_work_area),
            ([0, 0, 1280, 800], [0, 0, 1280, 800])
        );
        let other = OutputInfo {
            name: "DP-2".into(),
            ..output
        };
        assert_eq!(without_reserve(&reserved, other).work_area[3], 770);
    }

    #[test]
    fn the_helper_tooltip_names_the_helper_and_says_no_helper_without_one() {
        let mut helper = chora_kit::helper::Helper::default();
        let said = |data| Event::Emitted {
            event: "voice.state".into(),
            client: "voice".into(),
            data,
        };
        let tip = |helper: &chora_kit::helper::Helper| helper_tip(helper, Language::En);
        assert_eq!(tip(&helper), "No helper");
        helper.hear(&said(json!({"state": "listening"})));
        assert_eq!(tip(&helper), "No helper");
        helper.hear(&said(
            json!({"name": "coder", "state": "working", "task": "Write"}),
        ));
        assert_eq!(tip(&helper), "coder · working · Write");
        helper.hear(&said(json!({"state": "listening", "muted": {"mic": true}})));
        assert_eq!(tip(&helper), "Voice off");
        helper.hear(&said(
            json!({"state": "listening", "muted": {"mic": false}}),
        ));
        helper.hear(&Event::Emitted {
            event: "voice.remove".into(),
            client: "voice".into(),
            data: json!({"name": "coder"}),
        });
        assert_eq!(tip(&helper), "No helper");
    }

    #[test]
    fn help_and_support_is_a_task_button_and_an_entry_of_the_switcher() {
        assert_eq!(
            taskbar::Part::of(&format!("xp-task-{HELP_TASK}-DP-1")),
            Some(taskbar::Part::Task(HELP_TASK.into()))
        );
        let entry = |name: &str| switcher::Entry {
            surface: name.into(),
            title: name.into(),
            icon: None,
        };
        let surfaces = |entries: &[switcher::Entry]| {
            entries
                .iter()
                .map(|e| e.surface.clone())
                .collect::<Vec<_>>()
        };
        let mut away = vec![entry("s1"), entry("s2")];
        add_help(&mut away, entry(HELP_TASK), false);
        assert_eq!(surfaces(&away), ["s1", "s2", HELP_TASK]);
        let mut active = vec![entry("s1"), entry("s2")];
        add_help(&mut active, entry(HELP_TASK), true);
        assert_eq!(surfaces(&active), [HELP_TASK, "s1", "s2"]);
        assert_eq!(help::TITLE, "Help and Support");
    }

    #[test]
    fn a_new_window_opens_under_the_pointer_only_on_virtual_outputs() {
        let home = |bars: &[Bar], entered| home_bar(bars, Some(entered)).map(|b| b.output.clone());
        let faces = [bar("VIRTUAL-1", true), bar("VIRTUAL-2", true)];
        assert_eq!(home(&faces, "VIRTUAL-2").as_deref(), Some("VIRTUAL-2"));
        let screens = [bar("DP-1", false), bar("DP-2", false)];
        assert_eq!(home(&screens, "DP-2").as_deref(), Some("DP-1"), "as before");
        assert_eq!(home(&faces, "gone").as_deref(), Some("VIRTUAL-1"));
    }
}
