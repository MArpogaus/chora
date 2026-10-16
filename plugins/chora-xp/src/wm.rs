//! The window manager: a stack of windows with one focus, the taskbars,
//! the start menu and the wallpapers, driven by the events of the core.
//! The effects live in [`effects`], the desktops of the cube in
//! [`desktops`].

mod desktops;
mod effects;

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
    conn::Conn,
    control::{self, Setting},
    cube::DESKTOPS,
    decoration::{self, Button, Part},
    icons::Icons,
    session::{self, Choice, Dialog, Mode},
    startmenu::{Action, Menu},
    taskbar::{self, Bar, Task},
    theme::{Rect, Theme},
    wallpaper,
    window::{self, Window},
};

/// The client name, and the group of all nodes of the plugin.
pub const NAME: &str = "xp";
const WALLPAPERS: &str = "xp-wallpapers";
const WINDOWS: &str = "xp-windows";
const BARS: &str = "xp-bars";
const MENUS: &str = "xp-menus";
/// The left mouse button.
const LEFT_BUTTON: u32 = 272;
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
    /// The wallpaper file.
    pub wallpaper: PathBuf,
    /// Stretches the time of the magic lamp and of the cube, for tests
    /// and demos; 1 is the normal speed.
    pub slow: f64,
}

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
            config,
            windows: Vec::new(),
            opened: Vec::new(),
            focused: None,
            bars: Vec::new(),
            menu,
            drag: None,
            last_title: None,
            keys: Vec::new(),
            cascade: 0,
            desktop: 0,
            hover: None,
            escape: None,
            effects: Effects::default(),
            dialog: None,
            fade: None,
            fade_shader: None,
            done: false,
            restart: false,
        };
        wm.clock = wm.now();
        wm.create_scene()?;
        wm.grab()?;
        let events = [
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
        wm.conn.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: events.map(String::from).to_vec(),
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
    /// one transaction. It fails if another instance runs.
    fn create_scene(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.conn.call("output.list", Value::Null)?;
        let group = |id: &str, parent: &str| Command::Create {
            id: id.into(),
            kind: NodeKind::Group,
            parent: parent.into(),
            props: PropsUpdate::default(),
        };
        let mut commands = vec![group(NAME, ROOT)];
        for id in [WALLPAPERS, WINDOWS, BARS, MENUS] {
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
        match self.conn.try_apply(commands)? {
            None => Ok(()),
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    fn add_output(&mut self, output: &OutputInfo) -> Vec<Command> {
        let mut commands =
            wallpaper::create(&mut self.theme, &self.config.wallpaper, output, WALLPAPERS);
        let bar = Bar::new(output);
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
            Event::EffectsDropped => self.effects_dropped(),
            _ => Vec::new(),
        };
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
        let mut commands = decoration::create(surface, &desk_id(self.desktop));
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
        let mut commands = Vec::new();
        if previous.as_deref() != target {
            for surface in [previous.as_deref(), target].into_iter().flatten() {
                commands.extend(self.restyle(surface));
            }
        }
        commands.push(Command::Focus {
            surface: target.map(String::from),
        });
        commands
    }

    /// The configure and the decoration of a window in its current state.
    fn restyle(&mut self, surface: &str) -> Vec<Command> {
        let active = self.is_active(surface);
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let mut commands = vec![self.windows[i].configure(active)];
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
            commands.push(self.windows[i].configure(active));
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
        if let Some(dialog) = self.dialog.as_mut().filter(|_| Dialog::owns(node)) {
            return dialog.hover(&mut self.theme, node, on);
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
        let area = output.layout_area();
        commands.extend(self.bars[i].resize(&mut self.theme, area, &self.clock));
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
            commands.push(w.configure(active));
            commands.extend(w.style(&mut self.theme, active));
        }
        commands.extend(self.sync_tasks());
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
                commands.push(w.configure(active));
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
            Some(Key::Close) => self.focused.as_deref().map(close).unwrap_or_default(),
            Some(Key::Terminal) => {
                spawn(&self.config.terminal);
                Vec::new()
            }
            Some(Key::Turn(steps)) => self.turn(steps),
            Some(Key::Carry(steps)) => self.carry(steps),
            None => Vec::new(),
        }
    }

    fn pressed(&mut self, node: &str, button: u32, position: [f64; 2]) -> Vec<Command> {
        if self.dialog.is_some() {
            return self.press_dialog(node);
        }
        let start = matches!(taskbar::Part::of(node), Some(taskbar::Part::Start(_)));
        let mut commands = Vec::new();
        if self.menu.is_open() && !Menu::owns(node) && !start {
            commands.extend(self.close_menu());
        }
        if self.index(node).is_some() {
            commands.extend(self.activate(node));
        } else if let Some(part) = Part::of(node) {
            commands.extend(self.press_decoration(&part, button, position));
        } else if button != LEFT_BUTTON {
            // Only the left button works the taskbar and the menu.
        } else if let Some(part) = taskbar::Part::of(node) {
            commands.extend(self.press_taskbar(part));
        } else if Menu::owns(node) {
            commands.extend(self.menu.press(&mut self.theme, node));
        } else if wallpaper::owns(node) {
            // A click on the desktop takes the focus from the windows.
            commands.extend(self.focus(None));
            commands.extend(self.sync_tasks());
        }
        commands
    }

    fn press_decoration(&mut self, part: &Part, button: u32, position: [f64; 2]) -> Vec<Command> {
        let (Part::Frame(surface) | Part::Title(surface) | Part::Button(surface, _)) = part;
        let mut commands = self.activate(surface);
        if button != LEFT_BUTTON {
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
            Part::Frame(_) => {}
        }
        commands
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
                let control = self.control_panel(&output);
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
        if let Some(drag) = self.drag.take() {
            self.release_wobble(&drag.surface);
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
                Button::Close => close(surface),
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
        }
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
    fn control_panel(&mut self, output: &str) -> Vec<apps::Node> {
        let scale = self
            .outputs()
            .into_iter()
            .find(|o| o.name == output)
            .map_or(1.0, |o| o.scale);
        let dirs = [
            self.theme.path("wallpaper"),
            PathBuf::from("/usr/share/backgrounds"),
        ];
        let walls = control::wallpapers(&dirs);
        control::menu(scale, &self.config.wallpaper, &walls, self.effects.toggles)
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
                self.config.wallpaper.clone_from(file);
                self.outputs()
                    .iter()
                    .map(|o| wallpaper::update(&mut self.theme, file, o))
                    .collect()
            }
            Setting::Effect(effect) => {
                self.effects.toggles.flip(*effect);
                Vec::new()
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

    /// Does what a button of a dialog says.
    fn choose(&mut self, choice: Choice) -> Vec<Command> {
        let mut commands = self.close_dialog();
        match choice {
            Choice::LogOff => commands.extend(self.windows.iter().flat_map(|w| close(&w.surface))),
            Choice::StandBy => spawn("systemctl suspend"),
            Choice::TurnOff => self.done = true,
            Choice::Restart => {
                self.done = true;
                self.restart = true;
            }
            Choice::SwitchUser | Choice::Cancel => {}
        }
        commands
    }
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
    fn a_window_moves_to_the_same_place_on_another_output() {
        let (from, to) = ([1920, 0, 2560, 1400], [0, 0, 1920, 1040]);
        assert_eq!(moved([2020, 100], from, to), [100, 100]);
        // Beyond the new area, the title bar stays inside it.
        assert_eq!(moved([4300, 1300], from, to), [1856, 976]);
    }
}
