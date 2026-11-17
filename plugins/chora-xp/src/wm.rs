//! The window manager: a stack of windows with one focus, the taskbars,
//! the start menu and the wallpapers, driven by the events of the core.
//! The effects live in [`effects`], the desktops of the cube in
//! [`desktops`].

mod desktops;
mod dispatch;
mod effects;
mod float;
mod items;
mod keys;
mod output;
mod pointer;
mod press;
mod resize;
mod restart;
mod startup;
mod windows;

use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
    time::{Duration, Instant},
};

use chora_kit::{
    give,
    icons::Icons,
    notify::{Message, Notifier},
    runtime::{Plugin, Runtime},
    settings::{self, Store},
    shared,
    volume::{self, Mixer, Reading},
    words::{self, Language, Word},
};
use chora_scene::{
    Binding, Command, Decoration, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, ShaderEffect,
    SurfaceRequest, Transform, Uniform,
    ipc::{ClientError, ExportMethod, Grab, OutputConfigure, PressState, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use self::{desktops::desk_id, effects::Effects};
use crate::{
    apps,
    config::{Config, Picture, Wallpaper},
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
/// How often and how long apart the switcher asks for Alt+Escape: a
/// client that holds it gives it up on the event `xp.switcher`.
const ESCAPE_TRIES: u32 = 4;
const ESCAPE_WAIT: Duration = Duration::from_millis(15);
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
const EVENTS: [&str; 22] = [
    "client.disconnected",
    "surface.created",
    "surface.changed",
    "surface.destroyed",
    "surface.detached",
    "surface.requested",
    "pointer.pressed",
    "pointer.moved",
    "pointer.released",
    "pointer.entered",
    "pointer.left",
    "output.changed",
    "output.removed",
    "input.key",
    "focus.changed",
    "node.key",
    "input.button",
    "input.moved",
    "effects.dropped",
    "node.requested",
    "voice.state",
    "voice.remove",
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

/// What a key binding does.
#[derive(Clone, Copy)]
pub enum Key {
    Close,
    /// Super+e opens the file manager.
    Files,
    /// Super+Space opens the launcher, the Run box of the start menu.
    Launcher,
    /// Super and a key of the show-the-desktop kind: Super+d.
    Desktop,
    /// Super pressed and released alone opens or closes the start menu.
    Tap,
    /// Turns the cube by quarter turns.
    Turn(i32),
    /// Moves the focused window along with a turn.
    Carry(i32),
    /// XP's Alt+Tab, forward or back.
    Switch(i32),
    /// Super and the left button turn the cube with the pointer.
    Cube,
    /// Ctrl+Escape opens or closes the start menu, as on XP.
    Start,
    /// Gives the focused window to the window manager of this name (D55).
    Give(&'static str),
    /// A media key changes the volume.
    Volume(volume::Change),
    /// Opens the chat, as on tiling.
    Chat,
    /// Super+F1 opens Help and Support on the keyboard shortcuts.
    Help,
    /// Super+/ opens the same help, without a row of its own.
    HelpAlias,
    /// Super+Shift+Escape locks the screen.
    Lock,
    /// Super+Shift+e opens the Turn Off dialog.
    Power,
    /// Super+Print takes a screenshot.
    Screenshot,
}

/// The bindings: Ctrl+Alt+arrows turn the cube, as in Compiz. Super+Ctrl
/// stands in when another client holds them. The media keys and the chat
/// key are those of tiling.
pub const KEYS: [(&str, &str, Key); 23] = [
    ("Super+Super_L", "", Key::Tap),
    ("Super+Super_R", "", Key::Tap),
    ("Ctrl+Escape", "", Key::Start),
    ("Super+d", "", Key::Desktop),
    ("Super+space", "", Key::Launcher),
    ("Super+F1", "", Key::Help),
    ("Super+slash", "", Key::HelpAlias),
    ("Super+Shift+Escape", "", Key::Lock),
    ("Super+Shift+e", "", Key::Power),
    ("Super+Print", "", Key::Screenshot),
    ("Super+BTN_LEFT", "", Key::Cube),
    ("Alt+F4", "", Key::Close),
    ("Alt+Tab", "", Key::Switch(1)),
    ("Alt+Shift+Tab", "", Key::Switch(-1)),
    ("Super+e", "", Key::Files),
    (control::CHAT_KEY, "", Key::Chat),
    ("XF86AudioRaiseVolume", "", Key::Volume(volume::Change::Up)),
    (
        "XF86AudioLowerVolume",
        "",
        Key::Volume(volume::Change::Down),
    ),
    ("XF86AudioMute", "", Key::Volume(volume::Change::Mute)),
    ("Ctrl+Alt+Left", "Super+Ctrl+Left", Key::Turn(-1)),
    ("Ctrl+Alt+Right", "Super+Ctrl+Right", Key::Turn(1)),
    (
        "Ctrl+Alt+Shift+Left",
        "Super+Ctrl+Shift+Left",
        Key::Carry(-1),
    ),
    (
        "Ctrl+Alt+Shift+Right",
        "Super+Ctrl+Shift+Right",
        Key::Carry(1),
    ),
];

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

/// What voice announces about the helpers: the state and task of each
/// by name, and whether the microphone is off.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Helper {
    agents: BTreeMap<String, (String, String)>,
    mic_off: bool,
}

impl Helper {
    /// Takes in a `voice.state`: of a named helper, or of voice itself.
    fn follow(&mut self, data: &Value) {
        let text = |key: &str| data[key].as_str().unwrap_or_default().to_owned();
        match data["name"].as_str() {
            Some(name) => {
                self.agents
                    .insert(name.to_owned(), (text("state"), text("task")));
            }
            None => self.mic_off = data["muted"]["mic"].as_bool().unwrap_or(false),
        }
    }

    /// Takes in a `voice.remove`: the helper is gone.
    fn forget(&mut self, data: &Value) {
        if let Some(name) = data["name"].as_str() {
            self.agents.remove(name);
        }
    }

    /// The words of its tooltip: "Voice off" while the microphone is off,
    /// else the row of the first helper, else "No helper".
    fn tip(&self, language: Language) -> String {
        if self.mic_off {
            return Word::VoiceOff.text(language).to_owned();
        }
        match self.agents.iter().next() {
            Some((name, (state, task))) => words::row(language, name, state, task),
            None => Word::NoHelperDoor.text(language).to_owned(),
        }
    }
}

/// The state of the window manager.
pub struct Wm {
    rt: Runtime<Key>,
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
    /// Whether the bindings of the cube are grabbed.
    cube_keys: bool,
    /// The windows that float off their virtual output.
    floats: float::Floats,
    effects: Effects,
    /// The Log Off or Turn Off dialog.
    dialog: Option<Dialog>,
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
    /// The last press on a clock, for the double click.
    last_clock: Option<Instant>,
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
    /// The programs that started and show no window yet.
    launches: Launches,
    notepad: apps::Editors,
    notifier: Notifier,
    /// The reading that the speaker of the taskbars shows; none before
    /// the first.
    volume: Option<Reading>,
    helper: Helper,
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
    type Action = Key;

    fn runtime(&mut self) -> &mut Runtime<Key> {
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

    fn settings_changed(&mut self) -> Result<(), RpcError> {
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
        self.index(surface).is_some()
    }

    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        let (answer, commands) = self.give(surface, to)?;
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(answer)
    }
}

impl Wm {
    /// Tells the user that a program starts or did not open.
    fn tell(&mut self, change: &Change) {
        if let Change::Done { note: Some(note) } = change {
            self.notifier.close(*note);
        }
        if matches!(change, Change::Done { .. }) {
            return;
        }
        let (summary, body) = change.text();
        let (replaces, expire_ms) = match change {
            Change::Starting { .. } => (
                None,
                i32::try_from(launch::GIVE_UP.as_millis()).unwrap_or(-1),
            ),
            Change::Failed { note, .. } | Change::Fallback { note, .. } => (*note, -1),
            Change::Done { .. } => return,
        };
        let message = Message {
            icon: "",
            summary: &summary,
            body: &body,
            expire_ms,
        };
        let id = self.notifier.show("chora-xp", replaces, &message);
        if let (Change::Starting { token, .. }, Some(id)) = (change, id) {
            self.launches.noted(token, id);
        }
    }

    fn close_menu(&mut self) -> Vec<Command> {
        let output = self.menu.output().map(String::from);
        let mut commands = self.menu.close(&mut self.theme);
        if let Some(output) = output {
            commands.push(self.desktop_keys());
            commands.extend(self.start_look(&output));
        }
        commands
    }

    fn run(&mut self, action: &Action, output: Option<&str>) -> Vec<Command> {
        match action {
            Action::Run(command) => {
                self.start_program(command);
                Vec::new()
            }
            Action::LogOff => self.open_dialog(Mode::LogOff, output),
            Action::TurnOff => self.open_dialog(Mode::TurnOff, output),
            Action::Set(setting) => self.set(setting, output),
            Action::Help => self.open_help(0),
            Action::Lock => {
                self.run_shared("power", &["lock"]);
                Vec::new()
            }
            Action::Screenshot => {
                self.run_shared("screenshot", &[]);
                Vec::new()
            }
        }
    }

    /// Raises an open dialog of a kind; `None` if none is open.
    fn raise_panel(&mut self, kind: fn(&Panel) -> bool) -> Option<Vec<Command>> {
        let i = self.panels.iter().position(kind)?;
        let panel = self.panels.remove(i);
        let id = panel.window_ref().group();
        self.panels.push(panel);
        Some(vec![Command::Move {
            id,
            parent: None,
            index: None,
        }])
    }

    /// Taskbar and Start Menu Properties in the middle of a work area, or
    /// the open one on top.
    fn open_taskbar_dialog(&mut self, area: Rect) -> Vec<Command> {
        if let Some(commands) = self.raise_panel(|p| matches!(p, Panel::Taskbar(_))) {
            return commands;
        }
        let mut panel = TaskbarDialog::new(self.taskbar, area);
        let page = panel.page(&mut self.theme, &self.clock);
        let commands = panel.dialog.create(&mut self.theme, DIALOGS, page);
        self.panels.push(Panel::Taskbar(panel));
        commands
    }

    /// Help and Support on a topic, in the middle of the work area, or the
    /// open one on top, showing that topic. The window takes the keyboard,
    /// so that Escape closes it.
    fn open_help(&mut self, topic: usize) -> Vec<Command> {
        let mut commands = Vec::new();
        if let Some(i) = self.panels.iter().position(|p| matches!(p, Panel::Help(_))) {
            if let Panel::Help(panel) = &mut self.panels[i] {
                panel.show_topic(topic);
            }
            commands.extend(self.redraw_panel(i));
            commands.extend(
                self.raise_panel(|p| matches!(p, Panel::Help(_)))
                    .unwrap_or_default(),
            );
        } else {
            let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
            let rows = help::rows(|binding| self.rt.holds(binding));
            let mut panel = HelpDialog::new(topic, rows, area);
            let page = panel.page(&self.theme);
            commands.extend(panel.dialog.create(&mut self.theme, DIALOGS, page));
            self.panels.push(Panel::Help(panel));
        }
        commands.extend(self.focus(None));
        if let Some(Panel::Help(panel)) = self.panels.last() {
            commands.push(Command::FocusNode {
                node: Some(panel.dialog.group()),
            });
        }
        commands.extend(self.sync_tasks());
        commands
    }

    /// Date and Time Properties in the middle of a work area, or the open
    /// one on top.
    fn open_date_dialog(&mut self, area: Rect) -> Vec<Command> {
        if let Some(commands) = self.raise_panel(|p| matches!(p, Panel::Clock(_))) {
            return commands;
        }
        let now = chrono::Local::now().naive_local();
        let mut panel = DateDialog::new(now, area);
        let page = panel.page(&self.theme, now);
        let commands = panel.dialog.create(&mut self.theme, DIALOGS, page);
        self.panels.push(Panel::Clock(panel));
        commands
    }

    /// Display Properties for the output of a work area, or the open one
    /// on top.
    fn open_display_dialog(&mut self, area: Rect) -> Vec<Command> {
        if let Some(commands) = self.raise_panel(|p| matches!(p, Panel::Display(_))) {
            return commands;
        }
        let center = [area[0] + area[2] / 2, area[1] + area[3] / 2];
        let name = self
            .bars
            .iter()
            .find(|b| b.contains(center))
            .or(self.bars.first())
            .map(|b| b.output.clone())
            .unwrap_or_default();
        let screen = self
            .outputs()
            .into_iter()
            .find(|o| o.name == name)
            .map_or_else(
                || display::Screen {
                    output: name.clone(),
                    physical: [area[2], area[3]],
                    scale: 100,
                },
                |o| {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    display::Screen {
                        output: o.name.clone(),
                        physical: o.size.map(|n| (f64::from(n) * o.scale).round() as i32),
                        scale: (o.scale * 100.0).round() as u32,
                    }
                },
            );
        let walls = control::wallpapers(&wallpaper::dirs(&self.theme));
        let art = display::Art {
            picture: self.icons.find_sized("image-x-generic", 16),
            bin: self
                .icons
                .first_sized(&["user-trash-full", "user-trash"], 32),
            walls,
        };
        let style = self.settings.value.wallpaper.resolved(&self.theme);
        let mut panel = DisplayDialog::new(style, screen, self.display_tab, area, art);
        let page = panel.page(&mut self.theme);
        let commands = panel.dialog.create(&mut self.theme, DIALOGS, page);
        self.panels.push(Panel::Display(panel));
        commands
    }

    /// Changes the open Display Properties and shows it again.
    fn with_display(&mut self, change: impl FnOnce(&mut DisplayDialog)) -> Vec<Command> {
        let Some(i) = self
            .panels
            .iter()
            .position(|p| matches!(p, Panel::Display(_)))
        else {
            return Vec::new();
        };
        if let Panel::Display(panel) = &mut self.panels[i] {
            change(panel);
        }
        self.redraw_panel(i)
    }

    /// A click on Display Properties or its Effects.
    fn click_display(&mut self, i: usize, hit: &Hit) -> Vec<Command> {
        let click = match &mut self.panels[i] {
            Panel::Effects(panel) => {
                match panel.click(hit) {
                    display::Effects::Redraw => {
                        let page = panel.page();
                        return panel.dialog.show(&mut self.theme, page);
                    }
                    display::Effects::Take(toggles) => {
                        let mut commands = self.close_panel(i);
                        commands.extend(self.change(json!({ "effects": toggles })));
                        return commands;
                    }
                    display::Effects::Close => {}
                }
                return self.close_panel(i);
            }
            Panel::Display(panel) => panel.click(hit),
            _ => return Vec::new(),
        };
        match click {
            display::Click::Redraw => self.redraw_panel(i),
            display::Click::Close => self.close_panel(i),
            display::Click::Apply {
                style,
                scale,
                close,
            } => {
                let output = match &self.panels[i] {
                    Panel::Display(panel) => panel.output().to_string(),
                    _ => String::new(),
                };
                let mut commands = if close {
                    self.close_panel(i)
                } else {
                    self.redraw_panel(i)
                };
                if style != self.settings.value.wallpaper.resolved(&self.theme) {
                    commands.extend(self.change(json!({ "wallpaper": style })));
                }
                let current = self
                    .outputs()
                    .into_iter()
                    .find(|o| o.name == output)
                    .map(|o| o.scale);
                if current.is_some_and(|c| (c * 100.0 - f64::from(scale)).abs() >= 0.5) {
                    commands.extend(self.set(&Setting::Scale(scale), Some(&output)));
                }
                commands
            }
            display::Click::List(rect, entries) => self.open_display_list(i, rect, entries),
            display::Click::Effects => self.open_effects(),
        }
    }

    /// A drop-down list of Display Properties below a rectangle of it.
    fn open_display_list(
        &mut self,
        i: usize,
        rect: Rect,
        entries: Vec<context::Entry>,
    ) -> Vec<Command> {
        let Panel::Display(panel) = &self.panels[i] else {
            return Vec::new();
        };
        let (point, width) = panel.list_at(rect);
        let area = self
            .bars
            .iter()
            .find(|b| b.contains(point))
            .or(self.bars.first())
            .map_or([0, 0, 1280, 800], Bar::area);
        let menu = context::Menu::list(&self.theme, entries, point, area, width);
        self.show_context(menu, area)
    }

    /// Effects of Appearance, or the open one on top.
    fn open_effects(&mut self) -> Vec<Command> {
        if let Some(commands) = self.raise_panel(|p| matches!(p, Panel::Effects(_))) {
            return commands;
        }
        let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
        let panel = EffectsDialog::new(self.effects.toggles, area);
        let page = panel.page();
        let mut panel = Panel::Effects(panel);
        let commands = panel.window().create(&mut self.theme, DIALOGS, page);
        self.panels.push(panel);
        commands
    }

    /// Closes a dialog. Display Properties keeps its tab for the next
    /// time. If no window has the focus, the topmost window of the
    /// desktop gets it, as in XP.
    fn close_panel(&mut self, i: usize) -> Vec<Command> {
        let panel = self.panels.remove(i);
        if let Panel::Display(display) = &panel {
            self.display_tab = display.tab;
        }
        let mut commands = vec![panel.window_ref().remove()];
        if self.focused.is_none()
            && let Some(top) = self.topmost_except("")
        {
            commands.extend(self.focus(Some(&top)));
        }
        commands.extend(self.sync_tasks());
        commands
    }

    /// Changes settings as `settings.set` does: checks them, applies them
    /// and saves the file. A change that cannot be saved stays applied,
    /// with a warning.
    fn change(&mut self, changes: Value) -> Vec<Command> {
        if let Err(err) = self.settings.set(changes) {
            eprintln!("chora-xp: {err}");
            return Vec::new();
        }
        let commands = self.configure();
        if let Err(err) = self.settings.save() {
            eprintln!("chora-xp: the change applies but is not saved: {err}");
        }
        commands
    }

    /// Applies the settings: draws the wallpaper, the taskbar and the
    /// menu again.
    fn configure(&mut self) -> Vec<Command> {
        let config = self.settings.value.clone();
        let mut commands: Vec<Command> = self
            .outputs()
            .iter()
            .map(|o| wallpaper::update(&mut self.theme, &config.wallpaper, o))
            .collect();
        commands.extend(self.set_taskbar(config.taskbar));
        self.effects.set_toggles(config.effects);
        self.desk.terminal.clone_from(&config.terminal);
        self.notepad = apps::system_editors(config.editor.as_deref());
        commands.extend(self.close_menu());
        commands.extend(self.menu.rebuild(
            &config,
            apps::load(&config.terminal, config.editor.as_deref()),
            &mut self.theme,
            &mut self.icons,
        ));
        commands
    }

    /// A click on the speaker of a taskbar mutes or unmutes, and a double
    /// click on the clock opens Date and Time.
    fn press_tray(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
        #[allow(clippy::cast_possible_truncation)]
        let [x, y] = [px.floor() as i32, py.floor() as i32];
        let on = |[cx, cy, cw, ch]: Rect| (cx..cx + cw).contains(&x) && (cy..cy + ch).contains(&y);
        if self.bars.iter().any(|b| on(b.speaker_rect())) {
            return self.change_volume(volume::Change::Mute, false);
        }
        let Some(bar) = self.bars.iter().find(|b| b.clock_rect().is_some_and(on)) else {
            return Vec::new();
        };
        let area = bar.usable();
        let now = Instant::now();
        let double = self
            .last_clock
            .take()
            .is_some_and(|at| now.duration_since(at) < DOUBLE_CLICK);
        if !double {
            self.last_clock = Some(now);
            return Vec::new();
        }
        self.open_date_dialog(area)
    }

    /// The release of the button after a press on a dialog: a click on a
    /// control or on the close button.
    fn release_panel(&mut self, position: [f64; 2]) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut hit = None;
        for (i, panel) in self.panels.iter_mut().enumerate() {
            let (more, h) = panel.window().release(&mut self.theme, position);
            commands.extend(more);
            if let Some(h) = h {
                hit = Some((i, h));
            }
        }
        let Some((i, hit)) = hit else {
            return commands;
        };
        if matches!(self.panels[i], Panel::Display(_) | Panel::Effects(_)) {
            commands.extend(self.click_display(i, &hit));
            return commands;
        }
        if let Panel::Help(panel) = &mut self.panels[i] {
            match panel.click(&hit) {
                help::Click::Redraw => commands.extend(self.redraw_panel(i)),
                help::Click::Close => commands.extend(self.close_panel(i)),
                help::Click::Helper => {
                    commands.extend(self.close_panel(i));
                    self.start_program(control::CHAT);
                }
            }
            return commands;
        }
        let outcome = match (&mut self.panels[i], &hit) {
            (_, Hit::Close) | (Panel::Display(_) | Panel::Effects(_) | Panel::Help(_), _) => {
                Outcome::Close
            }
            (Panel::Taskbar(panel), hit) => panel.click(hit),
            (Panel::Clock(panel), hit) => match panel.click(hit) {
                Click::Redraw => Outcome::Redraw,
                Click::Close => Outcome::Close,
                Click::Months(rect, entries) => {
                    let (point, width) = panel.list_at(rect);
                    let area = self
                        .bars
                        .iter()
                        .find(|b| b.contains(point))
                        .or(self.bars.first());
                    let area = area.map_or([0, 0, 1280, 800], Bar::area);
                    let menu = context::Menu::list(&self.theme, entries, point, area, width);
                    commands.extend(self.show_context(menu, area));
                    return commands;
                }
            },
        };
        match outcome {
            Outcome::Redraw => commands.extend(self.redraw_panel(i)),
            Outcome::Close => commands.extend(self.close_panel(i)),
            Outcome::Apply(settings, close) => {
                if close {
                    commands.extend(self.close_panel(i));
                }
                commands.extend(self.change(json!({ "taskbar": settings })));
            }
        }
        commands
    }

    /// Shows the page of a dialog again.
    fn redraw_panel(&mut self, i: usize) -> Vec<Command> {
        match &mut self.panels[i] {
            Panel::Taskbar(panel) => {
                let page = panel.page(&mut self.theme, &self.clock);
                panel.dialog.show(&mut self.theme, page)
            }
            Panel::Clock(panel) => {
                let page = panel.page(&self.theme, chrono::Local::now().naive_local());
                panel.dialog.show(&mut self.theme, page)
            }
            Panel::Display(panel) => {
                let page = panel.page(&mut self.theme);
                panel.dialog.show(&mut self.theme, page)
            }
            Panel::Effects(_) => Vec::new(),
            Panel::Help(panel) => {
                let page = panel.page(&self.theme);
                panel.dialog.show(&mut self.theme, page)
            }
        }
    }

    /// Takes the settings of the taskbar: every taskbar shows or hides
    /// its clock and hides itself or not, the maximized windows fill the
    /// new work area, and the taskbars stay above the windows or not.
    fn set_taskbar(&mut self, settings: taskbar::Settings) -> Vec<Command> {
        let old = std::mem::replace(&mut self.taskbar, settings);
        let mut commands = Vec::new();
        for i in 0..self.bars.len() {
            let before = self.bars[i].usable();
            self.bars[i].configure(settings);
            let icons = self.icons();
            commands.extend(self.bars[i].tray(&mut self.theme, &self.clock, icons));
            commands.extend(self.refit(before, self.bars[i].usable()));
        }
        if old.on_top != settings.on_top {
            let index = if settings.on_top { 3 } else { 2 };
            commands.push(Command::Move {
                id: BARS.into(),
                parent: Some(NAME.into()),
                index: Some(index),
            });
        }
        self.reserve_bars();
        commands.extend(self.pagers());
        commands.extend(self.sync_tasks());
        commands.extend(self.refresh_icons());
        for i in 0..self.panels.len() {
            if let Some(panel) = self.panels[i].taskbar() {
                panel.changed(settings);
                commands.extend(self.redraw_panel(i));
            }
        }
        commands
    }

    /// The maximized windows of an old work area fill the new one.
    fn refit(&mut self, old: Rect, area: Rect) -> Vec<Command> {
        let mut commands = Vec::new();
        if old == area {
            return commands;
        }
        for w in self
            .windows
            .iter_mut()
            .filter(|w| w.maximized() && w.pos == [old[0], old[1]])
        {
            w.maximize(area);
            let active = self.focused.as_deref() == Some(w.surface.as_str());
            commands.push(w.configure(active));
            commands.extend(w.style(&mut self.theme, active));
        }
        commands
    }

    /// A taskbar that hides itself comes back while the pointer is on it,
    /// and slides away a moment after the pointer left it.
    fn auto_hide(&mut self) -> Vec<Command> {
        if !self.taskbar.auto_hide {
            return Vec::new();
        }
        let hover = self.hover.clone().unwrap_or_default();
        let on = self.bars.iter().position(|b| b.owns(&hover));
        if let Some(i) = on {
            self.bars[i].hide(false);
            self.hide_at = None;
        } else if self.hide_at.is_none() {
            self.hide_at = Some(Instant::now() + HIDE_DELAY);
        }
        Vec::new()
    }

    /// The slides of the taskbars, in a frame of the effects. A taskbar
    /// stays while its start menu or a context menu is open.
    pub(super) fn slide_frame(&mut self, now: Instant, dt: f64) -> Vec<Command> {
        if self.hide_at.is_some_and(|at| now >= at) {
            if self.menu.is_open() || self.context.is_some() {
                self.hide_at = Some(now + HIDE_DELAY);
            } else {
                self.hide_at = None;
                let hover = self.hover.clone().unwrap_or_default();
                for bar in self.bars.iter_mut().filter(|b| !b.owns(&hover)) {
                    bar.hide(true);
                }
            }
        }
        self.bars
            .iter_mut()
            .filter_map(|b| b.slide(dt.min(0.05)))
            .collect()
    }

    /// Whether a taskbar slides or waits to.
    pub(super) fn sliding(&self) -> bool {
        self.hide_at.is_some() || self.bars.iter().any(Bar::sliding)
    }

    /// Looks at the Desktop folder and the trash, and shows the icons on
    /// the work area of the first output.
    fn refresh_icons(&mut self) -> Vec<Command> {
        let Some(area) = self.bars.first().map(Bar::usable) else {
            return Vec::new();
        };
        self.desk.refresh(&mut self.theme, &mut self.icons, area)
    }

    /// A click on the desktop takes the focus from the windows and the
    /// selection from the icons; a drag draws the selection rectangle.
    fn press_wallpaper(&mut self, position: [f64; 2]) -> Vec<Command> {
        let mut commands = self.focus(None);
        commands.extend(self.sync_tasks());
        commands.extend(
            self.desk
                .press_empty(&mut self.theme, &mut self.icons, position),
        );
        commands
    }

    /// A click on an icon gives the desktop the focus and selects the
    /// icon; a double click opens it.
    fn press_icon(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        let mut commands = self.focus(None);
        commands.extend(self.sync_tasks());
        let (more, open) = self
            .desk
            .press(&mut self.theme, &mut self.icons, node, position);
        commands.extend(more);
        if let Some(open) = open {
            commands.extend(self.open_icon(open));
        }
        commands
    }

    /// Opens what an icon stands for: a folder in the file manager, a
    /// program or a file with the program of its type.
    fn open_icon(&mut self, open: desktop_icons::Open) -> Vec<Command> {
        let fm = &self.settings.value.file_manager;
        let mut name = None;
        let command = match open {
            desktop_icons::Open::Folder(path) => desktop_icons::open_with(fm, &path),
            desktop_icons::Open::Computer => desktop_icons::open_with(fm, Path::new("/")),
            desktop_icons::Open::Trash => {
                desktop_icons::open_with(fm, &desktop_icons::trash_to_open())
            }
            desktop_icons::Open::Run(command) => command,
            desktop_icons::Open::File(path) => {
                name = path.file_name().map(|n| n.to_string_lossy().into_owned());
                desktop_icons::xdg_open(&path)
            }
        };
        match name {
            Some(name) => self.start_named(&command, name),
            None => self.start_program(&command),
        }
        Vec::new()
    }

    /// The outputs of the plugin as the core has them, but with the work
    /// areas that the taskbars reserve: xp lays them out itself.
    fn outputs(&mut self) -> Vec<OutputInfo> {
        let outputs = self.rt.outputs().unwrap_or_else(|err| {
            eprintln!("chora-xp: {err}");
            Vec::new()
        });
        (outputs.into_iter())
            .map(|o| without_reserve(&self.reserved, o))
            .collect()
    }

    /// Reserves the height of each taskbar of a screen at the bottom of
    /// the work area of the core, so that the chat, the notifications and
    /// other clients keep clear of it.
    fn reserve_bars(&mut self) {
        let wanted: Vec<(String, i32)> = (self.bars.iter())
            .map(|bar| (bar.output.clone(), bar.reserve()))
            .filter(|(output, height)| self.reserved.get(output).copied().unwrap_or(0) != *height)
            .collect();
        for (output, height) in wanted {
            self.reserved.insert(output.clone(), height);
            let edges = json!({"output": output, "edges": {"bottom": height}});
            if let Err(err) = self.rt.conn.call::<_, Value>("output.reserve", edges) {
                eprintln!("chora-xp: output.reserve: {err}");
            }
        }
    }

    /// The Control Panel for an output, with the current choices.
    fn control_panel(&mut self) -> Vec<apps::Node> {
        let walls = control::wallpapers(&wallpaper::dirs(&self.theme));
        let current = self.settings.value.wallpaper.file(&self.theme);
        control::menu(
            current.as_deref(),
            &walls,
            self.effects.toggles,
            self.volume.unwrap_or(Reading::Failed),
            &self.held_shortcuts(),
        )
    }

    /// Changes a setting: the scale of the output of the menu through the
    /// core, the wallpaper of every output, or an effect.
    fn set(&mut self, setting: &Setting, output: Option<&str>) -> Vec<Command> {
        match setting {
            Setting::Scale(percent) => {
                let params = OutputConfigure {
                    output: output.map(String::from),
                    scale: Some(f64::from(*percent) / 100.0),
                    transform: None,
                };
                if let Err(err) = self.rt.conn.call::<_, Value>("output.configure", params) {
                    eprintln!("chora-xp: {err}");
                }
                Vec::new()
            }
            Setting::Wallpaper(file) => {
                let style = Wallpaper {
                    picture: Picture::File(file.clone()),
                    ..self.settings.value.wallpaper.clone()
                };
                self.change(json!({ "wallpaper": style }))
            }
            Setting::Effect(effect) => {
                let mut toggles = self.effects.toggles;
                toggles.flip(*effect);
                self.change(json!({ "effects": toggles }))
            }
            Setting::Volume(change) => self.change_volume(*change, false),
            Setting::Give(to) => {
                if let Err(err) = give::focused(self, to) {
                    let message = Message {
                        icon: "",
                        summary: "Keyboard Shortcuts",
                        body: &err.message,
                        expire_ms: -1,
                    };
                    self.notifier.show("chora-xp", None, &message);
                }
                Vec::new()
            }
            Setting::Note => Vec::new(),
            Setting::Taskbar | Setting::DateTime | Setting::Display => {
                let area = output
                    .and_then(|o| self.bars.iter().find(|b| b.output == o))
                    .or(self.bars.first())
                    .map_or([0, 0, 1280, 770], Bar::usable);
                match setting {
                    Setting::Taskbar => self.open_taskbar_dialog(area),
                    Setting::DateTime => self.open_date_dialog(area),
                    _ => self.open_display_dialog(area),
                }
            }
        }
    }

    /// Shows a dialog in the middle of the output of the start menu, and
    /// lets the desktop behind it fade to gray.
    fn open_dialog(&mut self, mode: Mode, output: Option<&str>) -> Vec<Command> {
        let bar = output
            .and_then(|o| self.bars.iter().find(|b| b.output == o))
            .or(self.bars.first());
        let Some(bar) = bar else {
            return Vec::new();
        };
        let [.., w, h] = bar.area();
        let suspend = (self.settings.value.session.stand_by.as_deref())
            .and_then(|c| c.split_whitespace().next())
            .is_some_and(session::on_path);
        let dialog = Dialog::new(mode, &bar.output, [w, h], suspend);
        let mut commands = dialog.create(&mut self.theme);
        self.dialog = Some(dialog);
        if self.fade_shader.is_none() {
            match self.theme.cache_file("fade.frag", session::FADE_SHADER) {
                Ok(path) => self.fade_shader = Some(path.to_string_lossy().into_owned()),
                Err(err) => eprintln!("chora-xp: the shader of the fade: {err}"),
            }
        }
        if self.fade_shader.is_some() && self.effects_on() {
            self.fade = Some(Instant::now());
            commands.extend(self.fade_step(0.0));
        }
        commands
    }

    /// The gray of the desktop: the whole plugin becomes a layer with the
    /// shader of the fade. The dialog sits on the overlay root, outside it.
    /// The gray starts at the origin of the scene; outputs left of or above
    /// it stay in color.
    fn fade_step(&self, amount: f64) -> Option<Command> {
        let shader = self.fade_shader.clone()?;
        let size = self.bars.iter().fold([1.0_f64, 1.0_f64], |[w, h], bar| {
            let [x, y, bw, bh] = bar.area();
            [w.max(f64::from(x + bw)), h.max(f64::from(y + bh))]
        });
        Some(Command::Update {
            id: NAME.into(),
            props: PropsUpdate {
                size: Some(size),
                effects: Some(vec![ShaderEffect {
                    shader,
                    uniforms: [("amount".into(), Uniform::Float(amount))].into(),
                }]),
                ..PropsUpdate::default()
            },
        })
    }

    /// Whether the desktop still fades.
    pub fn fading(&self) -> bool {
        self.fade.is_some_and(|start| start.elapsed() < FADE)
    }

    /// The next frame of the fade.
    pub(super) fn fade_frame(&self, now: Instant) -> Option<Command> {
        let start = self.fade?;
        let t = now.saturating_duration_since(start).as_secs_f64() / FADE.as_secs_f64();
        self.fade_step(t.min(1.0))
    }

    /// A press on a button of the dialog; a press outside it closes it,
    /// as Cancel does.
    fn press_dialog(&mut self, node: &str) -> Vec<Command> {
        let Some(dialog) = &mut self.dialog else {
            return Vec::new();
        };
        match dialog.press(&mut self.theme, node) {
            Some(commands) => commands,
            None => self.close_dialog(),
        }
    }

    fn close_dialog(&mut self) -> Vec<Command> {
        if self.dialog.take().is_none() {
            return Vec::new();
        }
        let mut commands = vec![Dialog::remove()];
        if self.fade.take().is_some() {
            commands.push(Command::Update {
                id: NAME.into(),
                props: PropsUpdate {
                    effects: Some(Vec::new()),
                    ..PropsUpdate::default()
                },
            });
        }
        commands
    }

    /// Plays one of XP's sounds, unless the Control Panel turned them off.
    fn play(&self, sound: Sound) {
        if self.effects.toggles.sounds {
            sound::play(sound);
        }
    }

    /// Does what a button of a dialog says: runs the command of the
    /// settings, or the shared power script without one.
    fn choose(&mut self, choice: Choice) -> Vec<Command> {
        let commands = self.close_dialog();
        let session = self.settings.value.session.clone();
        match choice {
            Choice::Lock => self.run_shared("power", &["lock"]),
            Choice::LogOff => {
                self.play(Sound::LogOff);
                self.run_session(session.log_off, "logout");
            }
            Choice::StandBy => session.stand_by.into_iter().for_each(session::run),
            Choice::TurnOff => {
                self.play(Sound::ShutDown);
                self.run_session(session.turn_off, "shutdown");
            }
            Choice::Restart => self.run_session(session.restart, "restart"),
            Choice::SwitchUser | Choice::Cancel => {}
        }
        commands
    }

    /// Runs the command that the person set, or else the action of the
    /// shared power script.
    fn run_session(&mut self, own: Option<String>, action: &str) {
        match own {
            Some(command) => session::run(command),
            None => self.run_shared("power", &[action]),
        }
    }

    /// Runs a shared script; a notice says so when it is not installed.
    fn run_shared(&mut self, script: &str, args: &[&str]) {
        if let Some(command) = shared::command(script, args) {
            session::run(command);
            return;
        }
        let body = format!("The file {script}.sh of the shared scripts is not installed.");
        let message = Message {
            icon: "",
            summary: "Cannot do that",
            body: &body,
            expire_ms: 10_000,
        };
        self.notifier.show("chora-xp", None, &message);
    }
}

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

/// Whether chora-xp takes a created surface: a free one, or one the core
/// reserved for it. A surface of another window manager does nothing.
fn takes(to: Option<&str>) -> bool {
    to.is_none_or(is_own)
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
    let under_the_pointer = |b: &&Bar| !b.has_pager() && Some(b.output.as_str()) == entered;
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

/// Grabs the keys of the plugin, with or without those of the cube; a
/// key that another client holds falls back to its second binding.
#[allow(clippy::expect_used, reason = "the bindings are constants")]
fn grab_keys(rt: &mut Runtime<Key>, cube: bool) -> Result<(), ClientError> {
    let of_the_cube = |key: &Key| matches!(key, Key::Cube | Key::Turn(_) | Key::Carry(_));
    for (binding, fallback, key) in KEYS.into_iter().filter(|k| cube || !of_the_cube(&k.2)) {
        for binding in [binding, fallback].into_iter().filter(|b| !b.is_empty()) {
            let binding: Binding = binding.parse().expect("valid bindings");
            if rt.grab(binding, key)? {
                break;
            }
        }
    }
    Ok(())
}

/// Changes the grabs to the keys with or without those of the cube. A
/// key of both sets stays with the core, so it never reaches a window.
fn change_keys(rt: &mut Runtime<Key>, cube: bool) -> Result<(), ClientError> {
    rt.start_grabs();
    grab_keys(rt, cube)?;
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
        let bound: Vec<&str> = KEYS.iter().map(|(b, ..)| *b).collect();
        for key in [
            "Super+Super_L",
            "Super+d",
            "Super+space",
            "Super+e",
            "Super+a",
        ] {
            assert!(bound.contains(&key), "{key}");
        }
        assert!(matches!(
            KEYS.iter().find(|(b, ..)| *b == "Super+e").map(|k| k.2),
            Some(Key::Files)
        ));
    }

    use std::{
        io::{BufRead as _, BufReader, Write as _},
        os::unix::net::UnixListener,
        sync::{Arc, Mutex},
    };

    use chora_kit::{conn::Conn, volume::Change};
    use serde_json::{Value, json};

    use super::*;

    /// The calls `input.grab` and `input.ungrab` that a fake core saw,
    /// as `METHOD BINDING`, and a runtime on that core.
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    fn on_fake_core(tag: &str) -> (Runtime<Key>, Arc<Mutex<Vec<String>>>) {
        let dir = std::env::temp_dir().join(format!("chora-xp-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("core.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&calls);
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            for line in BufReader::new(&stream).lines().map_while(Result::ok) {
                let request: Value = serde_json::from_str(&line).unwrap();
                let method = request["method"].as_str().unwrap_or_default();
                let binding = request["params"]["binding"].clone();
                let result = match method {
                    "hello" => json!({"version": chora_scene::ipc::VERSION}),
                    "input.grab" => binding.clone(),
                    _ => Value::Null,
                };
                if method.starts_with("input.") {
                    let binding = binding.as_str().unwrap_or_default();
                    seen.lock().unwrap().push(format!("{method} {binding}"));
                }
                let answer = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
                writeln!(writer, "{answer}").unwrap();
            }
        });
        let conn = Conn::connect_at(NAME, &path).unwrap();
        (Runtime::with(conn, NAME, Vec::new()), calls)
    }

    /// A window manager on a fake core that answers `scene.query` with no
    /// nodes and every other request with null, and the methods that it saw.
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    fn wm_on_fake_core(tag: &str) -> (Wm, Arc<Mutex<Vec<String>>>) {
        let dir = std::env::temp_dir().join(format!("chora-xp-wm-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("core.sock");
        let listener = UnixListener::bind(&path).unwrap();
        let methods = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&methods);
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
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            for line in BufReader::new(&stream).lines().map_while(Result::ok) {
                let request: Value = serde_json::from_str(&line).unwrap();
                let method = request["method"].as_str().unwrap_or_default().to_owned();
                let result = match method.as_str() {
                    "hello" => json!({"version": chora_scene::ipc::VERSION}),
                    "input.grab" => request["params"]["binding"].clone(),
                    "scene.query" => json!([]),
                    "output.list" => json!([output]),
                    _ => Value::Null,
                };
                seen.lock().unwrap().push(method);
                let answer = json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
                writeln!(writer, "{answer}").unwrap();
            }
        });
        let conn = Conn::connect_at(NAME, &path).unwrap();
        let mut rt = Runtime::with(conn, NAME, Vec::new());
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
        let wm = Wm::start_on(rt, theme, settings, 1.0).expect("xp starts on the fake core");
        (wm, methods)
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

    /// When the cube of xp goes, as on the faces of a cube of the
    /// construct, xp lets only the keys of the cube go: a key that it
    /// keeps is never free in between, where it would reach a window.
    #[test]
    fn a_change_of_the_cube_keys_keeps_the_other_keys() {
        let (mut rt, calls) = on_fake_core("cube-keys");
        grab_keys(&mut rt, true).unwrap();
        calls.lock().unwrap().clear();
        change_keys(&mut rt, false).unwrap();
        let mut gone = std::mem::take(&mut *calls.lock().unwrap());
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
        assert!(matches!(rt.action("Super+d"), Some(Key::Desktop)));
        assert!(rt.action("Ctrl+Alt+Left").is_none());
        change_keys(&mut rt, true).unwrap();
        assert!(matches!(rt.action("Ctrl+Alt+Left"), Some(Key::Turn(-1))));
        assert!(
            calls
                .lock()
                .unwrap()
                .iter()
                .all(|c| c.starts_with("input.grab ")),
            "{calls:?}"
        );
    }

    /// The volume keys and the chat key are bound as on tiling.
    #[test]
    fn the_media_keys_and_the_chat_key_are_bound() {
        let bound = |name: &str| KEYS.iter().find(|(b, _, _)| *b == name).map(|k| k.2);
        assert!(matches!(
            bound("XF86AudioRaiseVolume"),
            Some(Key::Volume(Change::Up))
        ));
        assert!(matches!(
            bound("XF86AudioLowerVolume"),
            Some(Key::Volume(Change::Down))
        ));
        assert!(matches!(
            bound("XF86AudioMute"),
            Some(Key::Volume(Change::Mute))
        ));
        assert!(matches!(bound("Super+a"), Some(Key::Chat)));
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
    fn a_surface_of_another_window_manager_is_nothing_to_do() {
        assert!(takes(None));
        assert!(takes(Some("xp")));
        assert!(!takes(Some("tiling")));
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
        let mut helper = Helper::default();
        let tip = |helper: &Helper| helper.tip(Language::En);
        assert_eq!(tip(&helper), "No helper");
        helper.follow(&json!({"state": "listening"}));
        assert_eq!(tip(&helper), "No helper");
        helper.follow(&json!({"name": "coder", "state": "working", "task": "Write"}));
        assert_eq!(tip(&helper), "coder · working · Write");
        helper.follow(&json!({"state": "listening", "muted": {"mic": true}}));
        assert_eq!(tip(&helper), "Voice off");
        helper.follow(&json!({"state": "listening", "muted": {"mic": false}}));
        helper.forget(&json!({"name": "coder"}));
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
