//! The window manager: a stack of windows with one focus, the taskbars,
//! the start menu and the wallpapers, driven by the events of the core.
//! The effects live in [`effects`], the desktops of the cube in
//! [`desktops`].

mod desktops;
mod effects;
mod explorers;
mod items;
mod resize;

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Command, Decoration, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, ShaderEffect,
    SurfaceRequest, Uniform,
    ipc::{ClientError, Grab, OutputConfigure, PressState, Subscribe},
};
use serde_json::Value;

use self::{desktops::desk_id, effects::Effects};
use crate::{
    apps,
    balloon::{self, Balloon, Queue},
    confirm::ConfirmDialog,
    conn::Conn,
    context::{self, Act},
    control::{self, Setting, Toggles},
    cube::DESKTOPS,
    datetime::{Click, DateDialog},
    decoration::{self, Button, Part},
    desktop_icons::{self, DesktopIcons},
    dialog::{self, Hit},
    display::{self, DisplayDialog, EffectsDialog},
    explorer::Explorer,
    folder::Place,
    icons::Icons,
    notify::{Bus, Note, Reason, Request, Signal},
    properties::PropertiesDialog,
    session::{self, Choice, Dialog, Mode},
    sound::{self, Sound},
    startmenu::{self, Action, Menu},
    taskbar::{self, Bar, Task},
    taskbar_dialog::{Outcome, TaskbarDialog},
    theme::{Rect, Theme},
    wallpaper,
    window::{self, Window},
};

/// The client name, and the group of all nodes of the plugin.
pub const NAME: &str = "xp";
const WALLPAPERS: &str = "xp-wallpapers";
const WINDOWS: &str = "xp-windows";
const BARS: &str = "xp-bars";
/// The settings dialogs, above the taskbar and below the menus.
const DIALOGS: &str = "xp-dialogs";
const MENUS: &str = "xp-menus";
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

/// The settings of the window manager.
pub struct Config {
    /// The shell command of the terminal.
    pub terminal: String,
    /// The strftime format of the clock.
    pub clock: String,
    /// The background of the desktop.
    pub wallpaper: wallpaper::Style,
    /// Stretches the time of the magic lamp and of the cube, for tests
    /// and demos; 1 is the normal speed.
    pub slow: f64,
    /// The effects and sounds that are on at the start.
    pub toggles: Toggles,
}

/// The events of the core that the plugin handles.
const EVENTS: [&str; 14] = [
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
    "effects.dropped",
];

/// What a key binding does.
#[derive(Clone, Copy)]
enum Key {
    Close,
    Terminal,
    /// Turns the cube by quarter turns.
    Turn(i32),
    /// Moves the focused window along with a turn.
    Carry(i32),
}

/// The bindings: Ctrl+Alt+arrows turn the cube, as in Compiz. Super+Ctrl
/// stands in when another client holds them.
const KEYS: [(&str, &str, Key); 6] = [
    ("Alt+F4", "", Key::Close),
    ("Super+e", "", Key::Terminal),
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

/// The state of the window manager.
pub struct Wm {
    conn: Conn,
    theme: Theme,
    icons: Icons,
    config: Config,
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
    keys: Vec<(String, Key)>,
    cascade: i32,
    clock: String,
    /// The desktop that shows.
    desktop: usize,
    /// The node under the pointer.
    hover: Option<String>,
    /// Escape, while the start menu holds it.
    escape: Option<String>,
    effects: Effects,
    /// The Log Off or Turn Off dialog.
    dialog: Option<Dialog>,
    /// When the dialog opened, while the desktop fades to gray.
    fade: Option<Instant>,
    /// The shader of the fade, in the cache.
    fade_shader: Option<String>,
    /// The notification server, if the session bus had the name free.
    bus: Option<Bus>,
    /// The notifications that show and wait.
    notes: Queue,
    /// The balloon that shows, with its notification.
    balloon: Option<(Note, Balloon)>,
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
    /// The settings dialogs, bottom first.
    panels: Vec<Panel>,
    /// When a taskbar that hides itself slides away.
    hide_at: Option<Instant>,
    /// The last press on a clock, for the double click.
    last_clock: Option<Instant>,
    /// The Explorer windows, drawn by the plugin.
    explorers: Vec<Explorer>,
    /// The number of Explorer windows so far, for their ids.
    explorer_count: u32,
    /// The tab of Display Properties when it last closed, as XP keeps it.
    display_tab: usize,
    /// The icon or the start menu entry of the open context menu.
    item: Option<Target>,
    /// Turn Off ended the plugin.
    pub done: bool,
    /// Restart ended the plugin, to start it again.
    pub restart: bool,
}

impl Wm {
    /// Creates the scene of the plugin and adopts the windows that have
    /// no owner.
    pub fn start(
        conn: Conn,
        theme: Theme,
        menu: Menu,
        icons: Icons,
        config: Config,
    ) -> Result<Self, ClientError> {
        let mut wm = Self {
            conn,
            theme,
            icons,
            clock: String::new(),
            desk: DesktopIcons::new(&config.terminal),
            effects: Effects::new(config.toggles),
            config,
            windows: Vec::new(),
            opened: Vec::new(),
            focused: None,
            bars: Vec::new(),
            menu,
            drag: None,
            resize: None,
            last_title: None,
            keys: Vec::new(),
            cascade: 0,
            desktop: 0,
            hover: None,
            escape: None,
            dialog: None,
            fade: None,
            fade_shader: None,
            bus: Bus::start_or_log(),
            notes: Queue::default(),
            balloon: None,
            context: None,
            sys_window: None,
            last_icon: None,
            taskbar: taskbar::Settings::default(),
            panels: Vec::new(),
            hide_at: None,
            last_clock: None,
            explorers: Vec::new(),
            explorer_count: 0,
            display_tab: 0,
            item: None,
            done: false,
            restart: false,
        };
        wm.clock = wm.now();
        wm.create_scene()?;
        wm.grab()?;
        wm.conn.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: EVENTS.map(String::from).to_vec(),
            },
        )?;
        for node in wm.conn.query("kind:surface")? {
            if node.owner.is_none() {
                let info = node.surface.unwrap_or_default();
                wm.adopt(&node.id, info.app_id, info.title.as_deref())?;
            }
        }
        Ok(wm)
    }

    /// The groups, the wallpapers, the taskbars and the hidden menu, in
    /// one transaction, and XP's logon sound. It fails if another instance
    /// runs.
    fn create_scene(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.conn.call("output.list", Value::Null)?;
        let group = |id: &str, parent: &str| Command::Create {
            id: id.into(),
            kind: NodeKind::Group,
            parent: parent.into(),
            props: PropsUpdate::default(),
        };
        let mut commands = vec![group(NAME, ROOT)];
        for id in [
            WALLPAPERS,
            desktop_icons::GROUP,
            WINDOWS,
            BARS,
            DIALOGS,
            MENUS,
        ] {
            commands.push(group(id, NAME));
        }
        for desktop in 0..DESKTOPS {
            commands.push(group(&desk_id(desktop), WINDOWS));
            if desktop != self.desktop {
                commands.push(Command::Update {
                    id: desk_id(desktop),
                    props: PropsUpdate {
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                });
            }
        }
        for output in &outputs {
            commands.extend(self.add_output(output));
        }
        commands.extend(self.menu.create(&mut self.theme, MENUS));
        commands.extend(self.refresh_icons());
        match self.conn.try_apply(commands)? {
            None => {
                self.play(Sound::Logon);
                Ok(())
            }
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    fn add_output(&mut self, output: &OutputInfo) -> Vec<Command> {
        let mut commands =
            wallpaper::create(&mut self.theme, &self.config.wallpaper, output, WALLPAPERS);
        let mut bar = Bar::new(output);
        bar.configure(self.taskbar);
        commands.extend(bar.create(&mut self.theme, BARS, &self.clock, self.desktop));
        self.bars.push(bar);
        commands
    }

    fn grab(&mut self) -> Result<(), ClientError> {
        for (binding, fallback, key) in KEYS {
            for binding in [binding, fallback].into_iter().filter(|b| !b.is_empty()) {
                let binding: Binding = binding.parse().expect("valid bindings");
                match self.conn.call::<_, String>("input.grab", Grab { binding }) {
                    Ok(normal) => {
                        self.keys.push((normal, key));
                        break;
                    }
                    // Another client may own the binding.
                    Err(ClientError::Rpc(err)) => eprintln!("chora-xp: {err}"),
                    Err(err) => return Err(err),
                }
            }
        }
        Ok(())
    }

    /// The next event, or `None` after a timeout.
    pub fn conn_event(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        self.conn.next_event(timeout)
    }

    /// Handles one event of the core.
    pub fn handle(&mut self, event: Event) -> Result<(), ClientError> {
        let commands = match event {
            Event::SurfaceCreated {
                surface,
                app_id,
                title,
                ..
            } => return self.adopt(&surface, app_id, title.as_deref()),
            Event::SurfaceDetached { surface } => {
                let commands = self.forget(&surface);
                self.conn.apply(commands)?;
                // Free again, so it is free to take, as at the start.
                return self.adopt(&surface, None, None);
            }
            Event::SurfaceDestroyed { surface } => self.forget(&surface),
            Event::SurfaceChanged {
                surface,
                app_id,
                title,
                size,
                decoration,
            } => {
                let mut commands = self.redecorate(&surface, decoration);
                commands.extend(self.retitle(&surface, app_id, title.as_deref()));
                if let Some(size) = size.filter(|[w, h]| *w > 0 && *h > 0) {
                    commands.extend(self.resized(&surface, size));
                }
                commands
            }
            Event::SurfaceRequested { surface, request } => self.request(&surface, request),
            Event::PointerPressed {
                node,
                button,
                position,
                ..
            } => self.pressed(&node, button, position),
            Event::PointerMoved { position, .. } => self.dragged(position),
            Event::PointerReleased { node, position, .. } => self.released(&node, position),
            Event::OutputChanged { output } => self.output_changed(&output),
            Event::OutputRemoved { name } => self.output_removed(&name),
            // A binding acts on its press, not on its release.
            Event::InputKey {
                binding,
                state: PressState::Pressed,
            } => self.key(&binding),
            Event::PointerEntered { node, .. } => self.hovered(&node, true),
            Event::PointerLeft { node } => self.hovered(&node, false),
            Event::EffectsDropped { client } if is_own(&client) => self.effects_dropped(),
            _ => Vec::new(),
        };
        let mut commands = commands;
        commands.extend(self.relayout_explorers());
        self.conn.apply(commands)
    }

    /// When the pointer has rested on the start menu long enough to open
    /// or close a cascade.
    pub fn menu_due(&self) -> Option<Instant> {
        self.menu.due()
    }

    /// Opens or closes the cascade where the pointer rests.
    pub fn menu_rest(&mut self, now: Instant) -> Result<(), ClientError> {
        let commands = self.menu.rest(&mut self.theme, &mut self.icons, now);
        if commands.is_empty() {
            return Ok(());
        }
        self.conn.apply(commands)
    }

    /// Updates the clock.
    pub fn tick(&mut self) -> Result<(), ClientError> {
        let mut commands = self.refresh_icons();
        // The hands of Date and Time move.
        for i in 0..self.panels.len() {
            if matches!(self.panels[i], Panel::Clock(_)) {
                commands.extend(self.redraw_panel(i));
            }
        }
        if !commands.is_empty() {
            self.conn.apply(commands)?;
        }
        let now = self.now();
        if now != self.clock {
            self.clock = now;
            let commands = self
                .bars
                .iter()
                .map(|bar| bar.tray(&mut self.theme, &self.clock))
                .collect();
            self.conn.apply(commands)?;
        }
        Ok(())
    }

    fn now(&self) -> String {
        chrono::Local::now().format(&self.config.clock).to_string()
    }

    // Windows

    fn index(&self, surface: &str) -> Option<usize> {
        self.windows.iter().position(|w| w.surface == surface)
    }

    fn is_active(&self, surface: &str) -> bool {
        self.focused.as_deref() == Some(surface)
    }

    /// Takes a surface without an owner into a hidden group with its
    /// decoration and focuses it. It shows once it has a size.
    fn adopt(
        &mut self,
        surface: &str,
        app_id: Option<String>,
        title: Option<&str>,
    ) -> Result<(), ClientError> {
        if self.index(surface).is_some() {
            return Ok(());
        }
        let previous = self.focused.clone();
        let mut window = Window::new(surface, app_id, title);
        window.icon = window.app_id.as_deref().and_then(|a| self.icons.find(a));
        window.desktop = self.desktop;
        self.windows.push(window);
        self.opened.push(surface.into());
        let mut commands = decoration::create(surface, &desk_id(self.desktop), false);
        commands.extend(self.focus(Some(surface)));
        if let Some(err) = self.conn.try_apply(commands)? {
            // Another client took it first.
            eprintln!("chora-xp: {surface}: {err}");
            self.windows.retain(|w| w.surface != surface);
            self.opened.retain(|s| s != surface);
            self.focused = previous;
            return Ok(());
        }
        // The transaction waited for the client to commit its size, or the
        // size follows with surface.changed. The title of a surface that
        // another client gave up comes with the query too.
        let Some(node) = self.conn.query(&format!("id:{surface}"))?.pop() else {
            return Ok(());
        };
        let info = node.surface.unwrap_or_default();
        let mut commands = self.redecorate(surface, info.decoration);
        commands.extend(self.retitle(surface, info.app_id, info.title.as_deref()));
        if let Some(size) = info.size.filter(|[w, h]| *w > 0 && *h > 0) {
            commands.extend(self.resized(surface, size));
        }
        self.conn.apply(commands)
    }

    /// The commands for a new client size: a new window shows, and the
    /// frame follows a client that resized itself.
    fn resized(&mut self, surface: &str, size: [i32; 2]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        if self.windows[i].size == Some(size) && self.windows[i].shown {
            return Vec::new();
        }
        let first = !self.windows[i].shown;
        self.windows[i].size = Some(size);
        self.anchor(i);
        // A window maximized before its first commit keeps its place.
        if first && !self.windows[i].maximized() {
            let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
            let outer = self.windows[i].outer();
            self.windows[i].pos = window::cascade(&mut self.cascade, outer, area);
        }
        self.windows[i].shown = true;
        let active = self.is_active(surface);
        let mut commands = self.windows[i].style(&mut self.theme, active);
        if first {
            commands.extend(self.sync_tasks());
        }
        commands
    }

    /// Moves the focus: the old and the new window change their look.
    fn focus(&mut self, target: Option<&str>) -> Vec<Command> {
        let previous = std::mem::replace(&mut self.focused, target.map(String::from));
        let mut commands = self
            .desk
            .focus(&mut self.theme, &mut self.icons, target.is_none());
        if previous.as_deref() != target {
            for surface in [previous.as_deref(), target].into_iter().flatten() {
                commands.extend(self.restyle(surface));
            }
        }
        // A window of the plugin takes the focus from the clients.
        let client = target.filter(|t| !self.is_explorer(t));
        commands.push(Command::Focus {
            surface: client.map(String::from),
        });
        commands
    }

    /// The configure and the decoration of a window in its current state.
    fn restyle(&mut self, surface: &str) -> Vec<Command> {
        let active = self.is_active(surface);
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let mut commands: Vec<Command> = self.windows[i].configure(active).into_iter().collect();
        commands.extend(self.windows[i].style(&mut self.theme, active));
        commands
    }

    /// Raises, restores and focuses a window; a window of another desktop
    /// turns the cube to it.
    fn activate(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let window = self.windows.remove(i);
        let (desktop, minimized) = (window.desktop, window.minimized);
        self.windows.push(window);
        let mut commands = if desktop == self.desktop {
            Vec::new()
        } else {
            self.turn_to(desktop)
        };
        if minimized {
            let button = self.task_button(surface);
            commands.extend(self.lamp_in(surface, button));
        }
        commands.push(Command::Move {
            id: decoration::group_id(surface),
            parent: None,
            index: None,
        });
        commands.extend(self.focus(Some(surface)));
        commands.extend(self.sync_tasks());
        commands
    }

    /// The topmost window of the desktop that shows, other than one.
    fn topmost_except(&self, surface: &str) -> Option<String> {
        self.windows
            .iter()
            .rev()
            .find(|w| w.surface != surface && w.shown && !w.minimized && w.desktop == self.desktop)
            .map(|w| w.surface.clone())
    }

    /// The task button of a window in scene coordinates.
    fn task_button(&self, surface: &str) -> [f64; 4] {
        self.index(surface)
            .and_then(|i| self.bar_at(self.windows[i].rect()))
            .and_then(|b| b.task_target(surface))
            .unwrap_or([0.0, 0.0, 160.0, 25.0])
    }

    fn minimize(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        if self.windows[i].minimized || !self.windows[i].shown {
            return Vec::new();
        }
        let button = self.task_button(surface);
        let mut commands = self.lamp_out(surface, button);
        if self.is_active(surface) {
            let next = self.topmost_except(surface);
            commands.extend(self.focus(next.as_deref()));
        }
        commands.extend(self.sync_tasks());
        commands
    }

    fn toggle_maximize(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let active = self.is_active(surface);
        let mut commands = Vec::new();
        if self.windows[i].maximized() {
            commands.extend(self.windows[i].unmaximize(active));
        } else {
            let area = self
                .bar_at(self.windows[i].rect())
                .map_or([0, 0, 1280, 770], Bar::usable);
            self.windows[i].maximize(area);
            commands.extend(self.windows[i].configure(active));
        }
        commands.extend(self.windows[i].style(&mut self.theme, active));
        commands
    }

    /// Forgets a window that closed or that another client took.
    fn forget(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        self.windows.remove(i);
        self.opened.retain(|s| s != surface);
        if self.drag.as_ref().is_some_and(|d| d.surface == surface) {
            self.drag = None;
        }
        if self.resize.as_ref().is_some_and(|r| r.surface == surface) {
            self.resize = None;
        }
        self.effects.wobbles.remove(surface);
        self.effects.lamps.remove(surface);
        let mut commands = vec![Command::Remove {
            id: decoration::group_id(surface),
        }];
        if self.is_active(surface) {
            let next = self.topmost_except(surface);
            commands.extend(self.focus(next.as_deref()));
        }
        commands.extend(self.sync_tasks());
        commands
    }

    fn retitle(
        &mut self,
        surface: &str,
        app_id: Option<String>,
        title: Option<&str>,
    ) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let title = window::title_of(title, app_id.as_deref());
        if self.windows[i].title == title && self.windows[i].app_id == app_id {
            return Vec::new();
        }
        self.windows[i].title = title;
        self.windows[i].icon = app_id.as_deref().and_then(|a| self.icons.find(a));
        self.windows[i].app_id = app_id;
        let active = self.is_active(surface);
        let mut commands = self.windows[i].style(&mut self.theme, active);
        commands.extend(self.sync_tasks());
        commands
    }

    /// Who draws the frame: the XP frame shows only for clients that ask
    /// for one; a client that draws its own, as GTK programs do, gets none.
    fn redecorate(&mut self, surface: &str, decoration: Option<Decoration>) -> Vec<Command> {
        let (Some(i), Some(decoration)) = (self.index(surface), decoration) else {
            return Vec::new();
        };
        let framed = decoration != Decoration::Client;
        if self.windows[i].framed == framed {
            return Vec::new();
        }
        self.windows[i].framed = framed;
        let active = self.is_active(surface);
        self.windows[i].style(&mut self.theme, active)
    }

    fn request(&mut self, surface: &str, request: SurfaceRequest) -> Vec<Command> {
        let maximized = self
            .index(surface)
            .is_some_and(|i| self.windows[i].maximized());
        match request {
            SurfaceRequest::Activate => self.activate(surface),
            SurfaceRequest::Close => close(surface),
            SurfaceRequest::Minimize => self.minimize(surface),
            SurfaceRequest::Maximize if !maximized => self.toggle_maximize(surface),
            SurfaceRequest::Unmaximize if maximized => self.toggle_maximize(surface),
            _ => Vec::new(),
        }
    }

    // Taskbars

    /// The taskbar of the output that shows most of a rectangle: the one
    /// with its center, else the first.
    fn bar_at(&self, [x, y, w, h]: Rect) -> Option<&Bar> {
        let center = [x + w / 2, y + h / 2];
        self.bars
            .iter()
            .find(|b| b.contains(center))
            .or(self.bars.first())
    }

    /// The task buttons of every taskbar.
    fn sync_tasks(&mut self) -> Vec<Command> {
        let hovered = match self.hover.as_deref().and_then(taskbar::Part::of) {
            Some(taskbar::Part::Task(surface)) => Some(surface),
            _ => None,
        };
        let tasks: Vec<Task<'_>> = self
            .opened
            .iter()
            .filter_map(|surface| {
                let w = &self.windows[self.windows.iter().position(|w| &w.surface == surface)?];
                (w.shown && w.desktop == self.desktop).then(|| Task {
                    surface,
                    title: &w.title,
                    icon: w.icon.clone(),
                    active: self.focused.as_deref() == Some(surface),
                    minimized: w.minimized,
                    hovered: hovered.as_deref() == Some(surface.as_str()),
                })
            })
            .collect();
        let mut commands = Vec::new();
        for bar in &mut self.bars {
            commands.extend(bar.tasks(&mut self.theme, &tasks));
        }
        commands
    }

    /// The desktop buttons of every taskbar.
    fn pagers(&mut self) -> Vec<Command> {
        let hovered = match self.hover.as_deref().and_then(taskbar::Part::of) {
            Some(taskbar::Part::Pager(desktop)) => Some(desktop),
            _ => None,
        };
        let mut commands = Vec::new();
        for bar in &self.bars {
            commands.extend(bar.pager(&mut self.theme, self.desktop, hovered));
        }
        commands
    }

    /// The start button of an output: down while its menu is open, lit
    /// under the pointer.
    fn start_look(&mut self, output: &str) -> Vec<Command> {
        let look = if self.menu.output() == Some(output) {
            "pressed"
        } else if self.hover.as_deref() == Some(format!("xp-start-{output}").as_str()) {
            "hover"
        } else {
            "normal"
        };
        let mut commands = Vec::new();
        for bar in self.bars.iter().filter(|b| b.output == output) {
            commands.push(bar.start(&mut self.theme, look));
        }
        commands
    }

    /// The pointer entered or left a node: buttons light up under it, and
    /// so do the entries of the start menu.
    fn hovered(&mut self, node: &str, on: bool) -> Vec<Command> {
        if on {
            self.hover = Some(node.into());
        } else if self.hover.as_deref() == Some(node) {
            self.hover = None;
        }
        let mut commands = self.auto_hide();
        if let Some((menu, _)) = self.context.as_mut().filter(|_| context::Menu::owns(node)) {
            commands.extend(menu.hover(&mut self.theme, MENUS, node, on));
            return commands;
        }
        if let Some(panel) = self.panels.iter_mut().find(|p| p.window_ref().owns(node)) {
            commands.extend(panel.window().hover(&mut self.theme, node, on));
            return commands;
        }
        if let Some(i) = self.explorer_of(node) {
            commands.extend(self.explorers[i].hover(&mut self.theme, node, on));
            return commands;
        }
        commands.extend(self.hovered_node(node, on));
        commands
    }

    fn hovered_node(&mut self, node: &str, on: bool) -> Vec<Command> {
        if let Some(dialog) = self.dialog.as_mut().filter(|_| Dialog::owns(node)) {
            return dialog.hover(&mut self.theme, node, on);
        }
        if let Some((_, balloon)) = self.balloon.as_mut().filter(|_| Balloon::owns(node)) {
            return balloon.hover(&mut self.theme, node, on);
        }
        if let Some(Part::Button(surface, button)) = Part::of(node) {
            let Some(i) = self.index(&surface) else {
                return Vec::new();
            };
            self.windows[i].hovered = on.then_some(button);
            let active = self.is_active(&surface);
            return self.windows[i].style(&mut self.theme, active);
        }
        match taskbar::Part::of(node) {
            Some(taskbar::Part::Start(output)) => self.start_look(&output),
            Some(taskbar::Part::Task(_)) => self.sync_tasks(),
            Some(taskbar::Part::Pager(_)) => self.pagers(),
            None if Menu::owns(node) => self.menu.hover(&mut self.theme, node, on),
            None => Vec::new(),
        }
    }

    fn output_changed(&mut self, output: &OutputInfo) -> Vec<Command> {
        let Some(i) = self.bars.iter().position(|b| b.output == output.name) else {
            let mut commands = self.add_output(output);
            commands.extend(self.sync_tasks());
            return commands;
        };
        let old = self.bars[i].usable();
        let mut commands = vec![wallpaper::update(
            &mut self.theme,
            &self.config.wallpaper,
            output,
        )];
        commands.extend(self.bars[i].resize(&mut self.theme, output, &self.clock));
        commands.extend(self.pagers());
        let area = self.bars[i].usable();
        // Maximized windows of the output fill its new area.
        for w in self
            .windows
            .iter_mut()
            .filter(|w| w.maximized() && w.pos == [old[0], old[1]])
        {
            w.maximize(area);
            let active = self.focused.as_deref() == Some(w.surface.as_str());
            commands.extend(w.configure(active));
            commands.extend(w.style(&mut self.theme, active));
        }
        commands.extend(self.sync_tasks());
        commands.extend(self.refresh_icons());
        commands
    }

    fn output_removed(&mut self, name: &str) -> Vec<Command> {
        let mut commands = Vec::new();
        if self.menu.output() == Some(name) {
            commands.extend(self.menu.close(&mut self.theme));
        }
        if let Some(i) = self.bars.iter().position(|b| b.output == name) {
            let bar = self.bars.remove(i);
            commands.push(bar.remove());
            commands.push(Command::Remove {
                id: wallpaper::id(name),
            });
            if let Some(target) = self.bars.first().map(Bar::usable) {
                commands.extend(self.move_windows(&bar, target));
            }
            commands.extend(self.refresh_icons());
        }
        commands
    }

    /// Moves the windows of a removed output to the area of another one, to
    /// the same place, as far as it fits.
    fn move_windows(&mut self, bar: &Bar, target: Rect) -> Vec<Command> {
        let from = bar.usable();
        let mut commands = Vec::new();
        for i in 0..self.windows.len() {
            if !bar.contains(self.windows[i].pos) {
                continue;
            }
            let active = self.focused.as_deref() == Some(self.windows[i].surface.as_str());
            let w = &mut self.windows[i];
            w.restore = w.restore.map(|[x, y, rw, rh]| {
                let [x, y] = moved([x, y], from, target);
                [x, y, rw, rh]
            });
            if w.maximized() {
                w.maximize(target);
                commands.extend(w.configure(active));
                commands.extend(w.style(&mut self.theme, active));
            } else {
                w.pos = moved(w.pos, from, target);
            }
            commands.push(w.place());
        }
        commands
    }

    // Input

    fn key(&mut self, binding: &str) -> Vec<Command> {
        if self.escape.as_deref() == Some(binding) {
            return match self.menu.escape(&mut self.theme) {
                Some(commands) => commands,
                None => self.close_menu(),
            };
        }
        match self
            .keys
            .iter()
            .find(|(b, _)| b == binding)
            .map(|(_, k)| *k)
        {
            Some(Key::Close) => match self.focused.clone() {
                Some(surface) => self.close_window(&surface),
                None => Vec::new(),
            },
            Some(Key::Terminal) => {
                spawn(&self.config.terminal);
                Vec::new()
            }
            Some(Key::Turn(steps)) => self.turn(steps),
            Some(Key::Carry(steps)) => self.carry(steps),
            None => Vec::new(),
        }
    }

    /// A press on a context menu goes to it; one elsewhere closes it and
    /// goes on.
    fn pressed(&mut self, node: &str, button: u32, position: [f64; 2]) -> Vec<Command> {
        if let Some((menu, _)) = self.context.as_mut().filter(|_| context::Menu::owns(node)) {
            menu.press(node);
            return Vec::new();
        }
        self.resize = None;
        let mut commands = self.close_context();
        commands.extend(self.press_node(node, button, position));
        commands
    }

    fn press_node(&mut self, node: &str, button: u32, position: [f64; 2]) -> Vec<Command> {
        if Balloon::owns(node) {
            return self.press_balloon(node);
        }
        if self.dialog.is_some() {
            return self.press_dialog(node);
        }
        if let Some(i) = self.panels.iter().position(|p| p.window_ref().owns(node)) {
            return self.press_panel(i, node, position);
        }
        let start = matches!(taskbar::Part::of(node), Some(taskbar::Part::Start(_)));
        let mut commands = Vec::new();
        if self.menu.is_open() && !Menu::owns(node) && !start {
            commands.extend(self.close_menu());
        }
        if let Some(i) = self.explorer_of(node) {
            commands.extend(self.press_explorer(i, node, position));
        } else if self.index(node).is_some() {
            commands.extend(self.activate(node));
        } else if let Some(part) = Part::of(node) {
            commands.extend(self.press_decoration(&part, button, position));
        } else if button == LEFT_BUTTON {
            commands.extend(self.press_left(node, position));
        } else if button == RIGHT_BUTTON {
            commands.extend(self.press_right(node, position));
        }
        commands
    }

    /// A press on a dialog raises it; its title bar starts a drag, its
    /// controls go down.
    fn press_panel(&mut self, i: usize, node: &str, position: [f64; 2]) -> Vec<Command> {
        let mut panel = self.panels.remove(i);
        let mut commands = vec![Command::Move {
            id: panel.window().group(),
            parent: None,
            index: None,
        }];
        commands.extend(panel.window().press(&mut self.theme, node, position));
        let dragging = panel
            .window_ref()
            .dragging()
            .then(|| panel.window_ref().group());
        self.panels.push(panel);
        // XP's settings dialogs move freely, so they wobble too.
        if let Some(group) = dragging {
            self.start_wobble(&group, position);
        }
        commands
    }

    /// The right button opens the menu of the clock, of the taskbar or of
    /// the desktop.
    fn press_right(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        #[allow(clippy::cast_possible_truncation)]
        let point = position.map(|p| p.floor() as i32);
        if let Some(taskbar::Part::Task(surface)) = taskbar::Part::of(node) {
            return self.open_system(&surface, point);
        }
        if let Some(commands) = self.open_item_menu(node, point) {
            return commands;
        }
        let entries = if node.starts_with("xp-tray-") {
            context::clock(self.taskbar.locked)
        } else if node.starts_with("xp-panel-") || node.starts_with("xp-pager-") {
            context::taskbar(self.taskbar.locked)
        } else if wallpaper::owns(node) || DesktopIcons::owns(node) {
            context::desktop()
        } else {
            return Vec::new();
        };
        let Some(bar) = self
            .bars
            .iter()
            .find(|b| b.contains(point))
            .or(self.bars.first())
        else {
            return Vec::new();
        };
        let (area, usable) = (bar.area(), bar.usable());
        let menu = context::Menu::new(&self.theme, entries, point, area, 0);
        let commands = menu.create(&mut self.theme, MENUS);
        self.context = Some((menu, usable));
        commands
    }

    /// XP's system menu of a window at a point.
    fn open_system(&mut self, surface: &str, point: [i32; 2]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let w = &self.windows[i];
        let entries = context::system(w.maximized(), w.minimized);
        let Some(bar) = self
            .bars
            .iter()
            .find(|b| b.contains(point))
            .or(self.bars.first())
        else {
            return Vec::new();
        };
        let (area, usable) = (bar.area(), bar.usable());
        let menu = context::Menu::new(&self.theme, entries, point, area, 0);
        let commands = menu.create(&mut self.theme, MENUS);
        self.context = Some((menu, usable));
        self.sys_window = Some(surface.into());
        commands
    }

    /// Does what a row of the system menu says to its window.
    fn act_window(&mut self, sys: context::Sys) -> Vec<Command> {
        let Some(surface) = self.sys_window.take() else {
            return Vec::new();
        };
        let Some(i) = self.index(&surface) else {
            return Vec::new();
        };
        let (maximized, minimized) = (self.windows[i].maximized(), self.windows[i].minimized);
        match sys {
            context::Sys::Restore if minimized => self.activate(&surface),
            context::Sys::Restore | context::Sys::Maximize => {
                let mut commands = self.activate(&surface);
                if maximized == (sys == context::Sys::Restore) {
                    commands.extend(self.toggle_maximize(&surface));
                }
                commands
            }
            context::Sys::Minimize => self.minimize(&surface),
            context::Sys::Close => self.close_window(&surface),
        }
    }

    fn close_context(&mut self) -> Vec<Command> {
        self.context
            .take()
            .map(|(menu, _)| menu.remove())
            .unwrap_or_default()
    }

    /// Does what a row of a context menu says, on the output of the menu.
    fn act(&mut self, act: Act, area: Rect) -> Vec<Command> {
        match act {
            Act::Cascade => self.arrange_windows(area, None),
            Act::TileRows => self.arrange_windows(area, Some(false)),
            Act::TileColumns => self.arrange_windows(area, Some(true)),
            Act::ShowDesktop => {
                let shown: Vec<String> = self
                    .windows
                    .iter()
                    .filter(|w| w.shown && !w.minimized && w.desktop == self.desktop)
                    .map(|w| w.surface.clone())
                    .collect();
                shown.iter().flat_map(|s| self.minimize(s)).collect()
            }
            Act::Lock => self.set_taskbar(taskbar::Settings {
                locked: !self.taskbar.locked,
                ..self.taskbar
            }),
            Act::ArrangeByName | Act::ArrangeByType => {
                self.desk
                    .arrange(&mut self.theme, &mut self.icons, act == Act::ArrangeByType)
            }
            Act::Refresh => self.desk.reload(&mut self.theme, &mut self.icons),
            Act::DesktopProperties => self.open_display_dialog(area),
            Act::TaskbarProperties => self.open_taskbar_dialog(area),
            Act::DateTime => self.open_date_dialog(area),
            Act::Explore(cmd) => self.explore(cmd),
            Act::Window(sys) => self.act_window(sys),
            Act::Item(item) => self.act_item(item),
            Act::Position(index) => self.with_display(|d| d.set_position(index)),
            Act::Color(color) => self.with_display(|d| d.set_color(color)),
            Act::Month(month) => {
                let Some(i) = self
                    .panels
                    .iter()
                    .position(|p| matches!(p, Panel::Clock(_)))
                else {
                    return Vec::new();
                };
                if let Panel::Clock(panel) = &mut self.panels[i] {
                    panel.set_month(month);
                }
                self.redraw_panel(i)
            }
        }
    }

    /// Cascades the windows of the current desktop on an output, from the
    /// bottom one, or tiles them in rows or columns.
    fn arrange_windows(&mut self, area: Rect, columns: Option<bool>) -> Vec<Command> {
        let [ax, ay, aw, ah] = area;
        let inside = |[x, y, w, h]: Rect| {
            let [cx, cy] = [x + w / 2, y + h / 2];
            (ax..ax + aw).contains(&cx) && (ay..ay + ah).contains(&cy)
        };
        let chosen: Vec<usize> = (0..self.windows.len())
            .filter(|&i| {
                let w = &self.windows[i];
                w.shown && !w.minimized && w.desktop == self.desktop && inside(w.rect())
            })
            .collect();
        let tiles = columns.map(|c| window::tile(chosen.len(), area, c));
        let mut step = 0;
        let mut commands = Vec::new();
        for (n, i) in chosen.into_iter().enumerate() {
            let active = self.is_active(&self.windows[i].surface);
            let w = &mut self.windows[i];
            if let Some(tiles) = &tiles {
                commands.extend(w.fit(tiles[n], active));
            } else {
                commands.extend(w.unmaximize(active));
                w.pos = window::cascade(&mut step, w.outer(), area);
            }
            commands.extend(w.style(&mut self.theme, active));
        }
        commands
    }

    /// Only the left button works the taskbar, the menu and the desktop.
    fn press_left(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        if node.starts_with("xp-tray-") {
            self.press_clock(position)
        } else if let Some(part) = taskbar::Part::of(node) {
            self.press_taskbar(part)
        } else if Menu::owns(node) {
            self.menu.press(&mut self.theme, node)
        } else if DesktopIcons::owns(node) {
            self.press_icon(node, position)
        } else if wallpaper::owns(node) {
            self.press_wallpaper(position)
        } else {
            Vec::new()
        }
    }

    fn press_decoration(&mut self, part: &Part, button: u32, position: [f64; 2]) -> Vec<Command> {
        let (Part::Frame(surface)
        | Part::Title(surface)
        | Part::Button(surface, _)
        | Part::Grip(surface)) = part;
        let mut commands = self.activate(surface);
        let Some(i) = self.index(surface) else {
            return commands;
        };
        if button == RIGHT_BUTTON && matches!(part, Part::Title(_)) {
            #[allow(clippy::cast_possible_truncation)]
            let point = position.map(|p| p.floor() as i32);
            commands.extend(self.open_system(surface, point));
            return commands;
        }
        if button != LEFT_BUTTON {
            return commands;
        }
        if matches!(part, Part::Title(_)) && self.windows[i].on_icon(position) {
            commands.extend(self.press_icon_of(surface));
            return commands;
        }
        let edges = self.index(surface).and_then(|i| {
            let w = &self.windows[i];
            w.edges(position).filter(|_| !w.held).map(|e| (e, w.rect()))
        });
        if let (Some((edges, from)), false) = (edges, matches!(part, Part::Button(..))) {
            commands.extend(self.start_resize(surface, edges, from, position));
            return commands;
        }
        match part {
            Part::Title(_) => commands.extend(self.press_title(surface, position)),
            Part::Button(_, b) => {
                if let Some(i) = self.index(surface) {
                    self.windows[i].pressed = Some(*b);
                    commands.extend(self.windows[i].style(&mut self.theme, true));
                }
            }
            Part::Frame(_) | Part::Grip(_) => {}
        }
        commands
    }

    /// A click on the icon of a title bar opens the system menu below
    /// it; a double click closes the window, as on XP.
    fn press_icon_of(&mut self, surface: &str) -> Vec<Command> {
        let now = Instant::now();
        let double = self
            .last_icon
            .take()
            .is_some_and(|(s, at)| s == surface && now.duration_since(at) < DOUBLE_CLICK);
        if double {
            return self.close_window(surface);
        }
        self.last_icon = Some((surface.into(), now));
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let w = &self.windows[i];
        let border = if w.maximized() { 0 } else { decoration::BORDER };
        let point = [w.pos[0] + border, w.pos[1] + decoration::TITLE];
        self.open_system(surface, point)
    }

    /// A press on a title bar starts a drag; the second press of a double
    /// click maximizes or restores the window.
    fn press_title(&mut self, surface: &str, position: [f64; 2]) -> Vec<Command> {
        let now = Instant::now();
        let double = self
            .last_title
            .take()
            .is_some_and(|(s, at)| s == surface && now.duration_since(at) < DOUBLE_CLICK);
        if double {
            return self.toggle_maximize(surface);
        }
        self.last_title = Some((surface.into(), now));
        if let Some(i) = self
            .index(surface)
            .filter(|i| !self.windows[*i].maximized() && !self.windows[*i].held)
        {
            self.drag = Some(Drag {
                surface: surface.into(),
                start: position,
                origin: self.windows[i].pos,
                last: position,
            });
            self.start_wobble(surface, position);
        }
        Vec::new()
    }

    fn press_taskbar(&mut self, part: taskbar::Part) -> Vec<Command> {
        match part {
            taskbar::Part::Start(output) => {
                if self.menu.is_open() {
                    return self.close_menu();
                }
                let Some(bar) = self.bars.iter().find(|b| b.output == output) else {
                    return Vec::new();
                };
                let (origin, area) = (bar.origin(), bar.usable());
                let control = self.control_panel();
                let mut commands = self.menu.open(&output, origin, area, control);
                commands.extend(self.start_look(&output));
                self.grab_escape();
                commands
            }
            taskbar::Part::Pager(desktop) => self.turn_to(desktop),
            taskbar::Part::Task(surface) => {
                let minimized = self
                    .index(&surface)
                    .is_some_and(|i| self.windows[i].minimized);
                if self.is_active(&surface) && !minimized {
                    self.minimize(&surface)
                } else {
                    self.activate(&surface)
                }
            }
        }
    }

    fn dragged(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
        let top = self.bars.iter().map(|b| b.usable()[1]).min().unwrap_or(0);
        if let Some(panel) = self.panels.iter_mut().find(|p| p.window_ref().dragging()) {
            let group = panel.window_ref().group();
            let mut commands = panel.window().drag([px, py], top);
            commands.extend(self.wobble_mesh(&group));
            return commands;
        }
        if self.desk.pressed() {
            return self.desk.drag(&mut self.theme, &mut self.icons, [px, py]);
        }
        if let Some(commands) = self.drag_explorer([px, py]) {
            return commands;
        }
        if let Some(r) = self.resize.as_ref().filter(|r| !r.released) {
            let target = self.resize_target(r, [px, py]);
            if let Some(r) = self.resize.as_mut().filter(|r| r.target != target) {
                r.target = target;
                r.pending = Some(target);
            }
            return Vec::new();
        }
        let Some(drag) = &mut self.drag else {
            return Vec::new();
        };
        // The same position again, exactly as the core sent it before.
        #[allow(clippy::float_cmp)]
        if drag.last == [px, py] {
            return Vec::new();
        }
        drag.last = [px, py];
        let (surface, start, origin) = (drag.surface.clone(), drag.start, drag.origin);
        let Some(i) = self.index(&surface) else {
            return Vec::new();
        };
        #[allow(clippy::cast_possible_truncation)]
        let pos = [
            origin[0] + (px - start[0]).round() as i32,
            origin[1] + (py - start[1]).round() as i32,
        ];
        // The title bar stays below the top edge of the outputs.
        let top = self.bars.iter().map(|b| b.usable()[1]).min().unwrap_or(0);
        self.windows[i].pos = [pos[0], pos[1].max(top)];
        // The mesh moves with the group, so that the frame does not jump.
        let mut commands = vec![self.windows[i].place()];
        commands.extend(self.wobble_mesh(&surface));
        commands
    }

    fn released(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        if let Some((menu, area)) = self.context.as_mut().filter(|_| context::Menu::owns(node)) {
            let area = *area;
            let Some(act) = menu.release(position) else {
                return Vec::new();
            };
            let mut commands = self.close_context();
            commands.extend(self.act(act, area));
            return commands;
        }
        if let Some(drag) = self.drag.take() {
            self.release_wobble(&drag.surface);
        }
        if let Some(group) = self
            .panels
            .iter()
            .map(Panel::window_ref)
            .find(|d| d.dragging())
            .map(dialog::Dialog::group)
        {
            self.release_wobble(&group);
        }
        if let Some(commands) = self.release_resize(position) {
            return commands;
        }
        if dialog::Dialog::any_owns(node) {
            return self.release_panel(position);
        }
        if let Some(i) = self.explorer_of(node) {
            return self.release_explorer(i, position);
        }
        if self.desk.pressed() {
            return self
                .desk
                .release(&mut self.theme, &mut self.icons, position);
        }
        if let Some(dialog) = &mut self.dialog {
            let (mut commands, choice) = dialog.release(&mut self.theme, position);
            if let Some(choice) = choice {
                commands.extend(self.choose(choice));
            }
            return commands;
        }
        if let Some(Part::Button(surface, button)) = Part::of(node) {
            return self.release_button(&surface, button, position);
        }
        if !Menu::owns(node) {
            return Vec::new();
        }
        let (mut commands, action) = self
            .menu
            .release(&mut self.theme, &mut self.icons, position);
        if let Some(action) = action {
            let output = self.menu.output().map(String::from);
            commands.extend(self.close_menu());
            commands.extend(self.run(&action, output.as_deref()));
        }
        commands
    }

    /// A button acts when the pointer goes up on it.
    fn release_button(
        &mut self,
        surface: &str,
        button: Button,
        position: [f64; 2],
    ) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let window = &mut self.windows[i];
        if window.pressed.take() != Some(button) {
            return Vec::new();
        }
        let inside = window.on_button(button, position);
        let active = self.focused.as_deref() == Some(surface);
        let mut commands = window.style(&mut self.theme, active);
        if inside {
            commands.extend(match button {
                Button::Close => self.close_window(surface),
                Button::Maximize => self.toggle_maximize(surface),
                Button::Minimize => self.minimize(surface),
            });
        }
        commands
    }

    /// Escape closes the cascades and the menu while it is open, as on XP.
    /// The core takes no grab without Ctrl, Alt or Super from a client
    /// without the capability `input.grab-any` (D45), so until it has it,
    /// only a click closes them.
    fn grab_escape(&mut self) {
        let binding = "Escape".parse().expect("a valid binding");
        if let Ok(normal) = self.conn.call::<_, String>("input.grab", Grab { binding }) {
            self.escape = Some(normal);
        }
    }

    fn close_menu(&mut self) -> Vec<Command> {
        if let Some(binding) = self.escape.take() {
            let binding = binding.parse().expect("the binding of the grab");
            if let Err(err) = self.conn.call::<_, Value>("input.ungrab", Grab { binding }) {
                eprintln!("chora-xp: {err}");
            }
        }
        let output = self.menu.output().map(String::from);
        let mut commands = self.menu.close(&mut self.theme);
        if let Some(output) = output {
            commands.extend(self.start_look(&output));
        }
        commands
    }

    fn run(&mut self, action: &Action, output: Option<&str>) -> Vec<Command> {
        match action {
            Action::Run(command) => {
                spawn(command);
                Vec::new()
            }
            Action::LogOff => self.open_dialog(Mode::LogOff, output),
            Action::TurnOff => self.open_dialog(Mode::TurnOff, output),
            Action::Set(setting) => self.set(setting, output),
            Action::Explore(place) => self.open_explorer(place.clone()),
        }
    }

    // The settings dialogs

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
        let style = self.config.wallpaper.clone();
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
                    display::Effects::Take(toggles) => self.effects.toggles = toggles,
                    display::Effects::Close => {}
                }
                return vec![self.panels.remove(i).window_ref().remove()];
            }
            Panel::Display(panel) => panel.click(hit),
            _ => return Vec::new(),
        };
        match click {
            display::Click::Redraw => self.redraw_panel(i),
            display::Click::Close => self.close_display(i),
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
                    self.close_display(i)
                } else {
                    self.redraw_panel(i)
                };
                if style != self.config.wallpaper {
                    commands.extend(self.set_wallpaper(style));
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
        let menu = context::Menu::new(&self.theme, entries, point, area, width);
        let commands = menu.create(&mut self.theme, MENUS);
        self.context = Some((menu, area));
        commands
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

    /// Closes Display Properties, keeping its tab for the next time.
    fn close_display(&mut self, i: usize) -> Vec<Command> {
        let panel = self.panels.remove(i);
        if let Panel::Display(display) = &panel {
            self.display_tab = display.tab;
        }
        vec![panel.window_ref().remove()]
    }

    /// Shows a background on every output.
    fn set_wallpaper(&mut self, style: wallpaper::Style) -> Vec<Command> {
        self.config.wallpaper = style;
        self.outputs()
            .iter()
            .map(|o| wallpaper::update(&mut self.theme, &self.config.wallpaper, o))
            .collect()
    }

    /// A double click on the clock of a taskbar opens Date and Time.
    fn press_clock(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
        #[allow(clippy::cast_possible_truncation)]
        let [x, y] = [px.floor() as i32, py.floor() as i32];
        let on = |[cx, cy, cw, ch]: Rect| (cx..cx + cw).contains(&x) && (cy..cy + ch).contains(&y);
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
        if let Some(more) = self.answer(i, &hit) {
            commands.extend(more);
            return commands;
        }
        if matches!(self.panels[i], Panel::Display(_) | Panel::Effects(_)) {
            commands.extend(self.click_display(i, &hit));
            return commands;
        }
        let outcome = match (&mut self.panels[i], &hit) {
            (_, Hit::Close)
            | (
                Panel::Confirm(_) | Panel::Display(_) | Panel::Effects(_) | Panel::Properties(_),
                _,
            ) => Outcome::Close,
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
                    let menu = context::Menu::new(&self.theme, entries, point, area, width);
                    commands.extend(menu.create(&mut self.theme, MENUS));
                    self.context = Some((menu, area));
                    return commands;
                }
            },
        };
        match outcome {
            Outcome::Redraw => commands.extend(self.redraw_panel(i)),
            Outcome::Close => commands.push(self.panels.remove(i).window_ref().remove()),
            Outcome::Apply(settings, close) => {
                if close {
                    commands.push(self.panels.remove(i).window_ref().remove());
                }
                commands.extend(self.set_taskbar(settings));
            }
        }
        commands
    }

    /// A click on a question or on Properties: Yes deletes or empties,
    /// the buttons close. `None` for the other dialogs.
    fn answer(&mut self, i: usize, hit: &Hit) -> Option<Vec<Command>> {
        let mut commands = Vec::new();
        match &self.panels[i] {
            Panel::Confirm(confirm) => {
                let trash = confirm.trash.clone();
                commands.push(self.panels.remove(i).window_ref().remove());
                if ConfirmDialog::yes(hit) {
                    commands.extend(match trash {
                        Some(path) => self.trash(&path),
                        None => self.empty_trash(),
                    });
                }
            }
            Panel::Properties(_) => {
                let button = matches!(hit, Hit::Control(n, _) if n == "ok" || n == "cancel");
                if button || *hit == Hit::Close {
                    commands.push(self.panels.remove(i).window_ref().remove());
                }
            }
            _ => return None,
        }
        Some(commands)
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
            Panel::Confirm(_) | Panel::Effects(_) | Panel::Properties(_) => Vec::new(),
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
            commands.push(self.bars[i].tray(&mut self.theme, &self.clock));
            commands.extend(self.refit(before, self.bars[i].usable()));
        }
        if old.on_top != settings.on_top {
            // Above the windows, or between the icons and the windows.
            let index = if settings.on_top { 3 } else { 2 };
            commands.push(Command::Move {
                id: BARS.into(),
                parent: Some(NAME.into()),
                index: Some(index),
            });
        }
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
            commands.extend(w.configure(active));
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

    // The icons on the desktop

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

    /// Opens what an icon stands for: a place in Explorer, a program or
    /// a file with the program of its type.
    fn open_icon(&mut self, open: desktop_icons::Open) -> Vec<Command> {
        let place = match open {
            desktop_icons::Open::Folder(path) => Place::Folder(path),
            desktop_icons::Open::Computer => Place::Computer,
            desktop_icons::Open::Trash => Place::Trash,
            desktop_icons::Open::Run(command) => {
                spawn(&command);
                return Vec::new();
            }
            desktop_icons::Open::File(path) => {
                spawn(&desktop_icons::xdg_open(&path));
                return Vec::new();
            }
        };
        self.open_explorer(place)
    }

    // The Control Panel of the start menu

    /// The outputs as the core has them.
    fn outputs(&mut self) -> Vec<OutputInfo> {
        self.conn
            .call("output.list", Value::Null)
            .unwrap_or_else(|err| {
                eprintln!("chora-xp: {err}");
                Vec::new()
            })
    }

    /// The Control Panel for an output, with the current choices.
    fn control_panel(&mut self) -> Vec<apps::Node> {
        let walls = control::wallpapers(&wallpaper::dirs(&self.theme));
        let current = self.config.wallpaper.file.as_deref();
        control::menu(current, &walls, self.effects.toggles)
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
                // The core answers with output.changed, which fits the
                // taskbar and the wallpaper.
                if let Err(err) = self.conn.call::<_, Value>("output.configure", params) {
                    eprintln!("chora-xp: {err}");
                }
                Vec::new()
            }
            Setting::Wallpaper(file) => {
                let style = wallpaper::Style {
                    file: Some(file.clone()),
                    ..self.config.wallpaper.clone()
                };
                self.set_wallpaper(style)
            }
            Setting::Effect(effect) => {
                self.effects.toggles.flip(*effect);
                Vec::new()
            }
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

    // The Log Off and Turn Off dialogs

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
        let suspend = session::on_path("systemctl");
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
    fn fade_step(&self, amount: f64) -> Option<Command> {
        let shader = self.fade_shader.clone()?;
        // ponytail: from the origin of the scene; outputs left of or above
        // it stay in color. Take the bounds of all outputs if that matters.
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

    /// Does what a button of a dialog says.
    fn choose(&mut self, choice: Choice) -> Vec<Command> {
        let mut commands = self.close_dialog();
        match choice {
            Choice::LogOff => {
                self.play(Sound::LogOff);
                let surfaces: Vec<String> =
                    self.windows.iter().map(|w| w.surface.clone()).collect();
                for surface in surfaces {
                    commands.extend(self.close_window(&surface));
                }
            }
            Choice::StandBy => spawn("systemctl suspend"),
            Choice::TurnOff => {
                self.play(Sound::ShutDown);
                self.done = true;
            }
            Choice::Restart => {
                self.done = true;
                self.restart = true;
            }
            Choice::SwitchUser | Choice::Cancel => {}
        }
        commands
    }
}

impl Wm {
    // The balloons of the notifications

    /// When the main loop looks at the notifications again: soon, while
    /// the server runs.
    pub fn notes_due(&self, now: Instant) -> Option<Instant> {
        self.bus.as_ref()?;
        let poll = now + balloon::POLL;
        Some(self.notes.due().map_or(poll, |due| due.min(poll)))
    }

    /// Takes the calls of the clients and closes an expired balloon.
    pub fn notifications(&mut self, now: Instant) -> Result<(), ClientError> {
        let Some(bus) = &self.bus else {
            return Ok(());
        };
        let requests: Vec<Request> = bus.requests.try_iter().collect();
        let mut signals = Vec::new();
        for request in requests {
            match request {
                Request::Notify(note) => self.notes.push(note, now),
                Request::Close(id) => signals.extend(self.notes.close(id, Reason::Closed, now)),
            }
        }
        signals.extend(self.notes.expire(now));
        let commands = self.after_notes(&signals);
        if commands.is_empty() {
            return Ok(());
        }
        self.conn.apply(commands)
    }

    /// A click on the close button dismisses the balloon; one on the
    /// balloon invokes its default action, too.
    fn press_balloon(&mut self, node: &str) -> Vec<Command> {
        let now = Instant::now();
        let signals = match self.notes.current() {
            Some(shown) if Balloon::is_close(node) => {
                self.notes.close(shown.id, Reason::Dismissed, now)
            }
            _ => self.notes.click(now),
        };
        self.after_notes(&signals)
    }

    /// Sends the signals and shows the balloon of the current
    /// notification, if it changed.
    fn after_notes(&mut self, signals: &[Signal]) -> Vec<Command> {
        if let Some(bus) = &self.bus {
            for signal in signals {
                bus.emit(signal);
            }
        }
        let current = self.notes.current();
        if self.balloon.as_ref().map(|(note, _)| note) == current {
            return Vec::new();
        }
        let current = current.cloned();
        let mut commands = Vec::new();
        if self.balloon.take().is_some() {
            commands.push(Balloon::remove());
        }
        let icon = current
            .as_ref()
            .map(|n| n.icon.as_str())
            .filter(|i| !i.is_empty())
            .and_then(|i| self.icons.find(i));
        // The notification area shows the icon of the balloon, or XP's
        // information icon, while the balloon shows.
        let tray = current.as_ref().map(|_| {
            icon.clone()
                .unwrap_or_else(|| self.theme.path("winxp/balloon-info.svg"))
        });
        commands.extend(self.tray_icon(tray));
        let (Some(note), Some(bar)) = (current, self.bars.first()) else {
            return commands;
        };
        let balloon = Balloon::new(&mut self.theme, &note, icon.as_deref(), bar);
        commands.extend(balloon.create(&mut self.theme));
        self.balloon = Some((note, balloon));
        self.play(Sound::Balloon);
        commands
    }

    /// Puts an icon into the notification area of the first output, or
    /// takes it out; the task and desktop buttons make room.
    fn tray_icon(&mut self, icon: Option<PathBuf>) -> Vec<Command> {
        let Some(bar) = self.bars.first_mut() else {
            return Vec::new();
        };
        if bar.icon == icon {
            return Vec::new();
        }
        bar.icon = icon;
        let mut commands = vec![bar.tray(&mut self.theme, &self.clock)];
        commands.extend(self.pagers());
        commands.extend(self.sync_tasks());
        commands
    }
}

/// A settings dialog, or a question.
enum Panel {
    /// Taskbar and Start Menu Properties.
    Taskbar(TaskbarDialog),
    /// Date and Time Properties.
    Clock(DateDialog),
    /// Confirm File Delete before Empty the Recycle Bin.
    Confirm(ConfirmDialog),
    /// Display Properties.
    Display(DisplayDialog),
    /// Effects of Display Properties.
    Effects(EffectsDialog),
    /// Properties of an icon or an entry of the start menu.
    Properties(PropertiesDialog),
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
            Self::Confirm(panel) => &mut panel.dialog,
            Self::Display(panel) => &mut panel.dialog,
            Self::Effects(panel) => &mut panel.dialog,
            Self::Properties(panel) => &mut panel.dialog,
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
            Self::Confirm(panel) => &panel.dialog,
            Self::Display(panel) => &panel.dialog,
            Self::Effects(panel) => &panel.dialog,
            Self::Properties(panel) => &panel.dialog,
        }
    }
}

impl Wm {
    /// Asks a client to close its window, or closes an Explorer window.
    fn close_window(&mut self, surface: &str) -> Vec<Command> {
        if self.is_explorer(surface) {
            self.close_explorer(surface)
        } else {
            close(surface)
        }
    }
}

/// Whether the watchdog of the core named chora-xp, and not another
/// client, when it dropped the effects.
fn is_own(client: &str) -> bool {
    client == NAME
}

fn close(surface: &str) -> Vec<Command> {
    vec![Command::Close {
        surface: surface.into(),
    }]
}

/// Runs a shell command. `&` lets the shell exit at once, so the command
/// never becomes a zombie of this plugin.
pub fn spawn(command: &str) {
    let spawned = std::process::Command::new("sh")
        .args(["-c", &format!("{command} &")])
        .status();
    if let Err(err) = spawned {
        eprintln!("chora-xp: cannot run {command:?}: {err}");
    }
}

/// A position in one area at the same place in another, so that at least
/// the title bar stays inside it.
fn moved([px, py]: [i32; 2], [fx, fy, ..]: Rect, [tx, ty, tw, th]: Rect) -> [i32; 2] {
    let keep = 64;
    [
        (tx + px - fx).clamp(tx, tx + (tw - keep).max(0)),
        (ty + py - fy).clamp(ty, ty + (th - keep).max(0)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
