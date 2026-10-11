//! The window manager: a stack of windows with one focus, the taskbars,
//! the start menu and the wallpapers, driven by the events of the core.

use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use chora_scene::{
    Binding, Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceRequest,
    ipc::{ClientError, Grab, Subscribe},
};
use serde_json::Value;

use crate::{
    conn::Conn,
    decoration::{self, Button, Part},
    icons::Icons,
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

/// The settings of the window manager.
pub struct Config {
    /// The shell command of the terminal.
    pub terminal: String,
    /// The strftime format of the clock.
    pub clock: String,
    /// The wallpaper file.
    pub wallpaper: PathBuf,
}

/// What a key binding does.
#[derive(Clone, Copy)]
enum Key {
    Close,
    Terminal,
}

/// A title bar that the pointer drags.
struct Drag {
    surface: String,
    /// The pointer at the press.
    start: [f64; 2],
    /// The window at the press.
    origin: [i32; 2],
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
    /// Turn Off ended the plugin.
    pub done: bool,
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
            done: false,
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
            "output.changed",
            "output.removed",
            "input.key",
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
        commands.extend(bar.create(&mut self.theme, BARS, &self.clock));
        self.bars.push(bar);
        commands
    }

    fn grab(&mut self) -> Result<(), ClientError> {
        for (binding, key) in [("Alt+F4", Key::Close), ("Super+e", Key::Terminal)] {
            let binding: Binding = binding.parse().expect("valid bindings");
            match self.conn.call::<_, String>("input.grab", Grab { binding }) {
                Ok(normal) => self.keys.push((normal, key)),
                // Another client may own the binding.
                Err(ClientError::Rpc(err)) => eprintln!("chora-xp: {err}"),
                Err(err) => return Err(err),
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
                ..
            } => self.retitle(&surface, app_id, title.as_deref()),
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
            Event::InputKey { binding } => self.key(&binding),
            _ => Vec::new(),
        };
        self.conn.apply(commands)
    }

    /// Updates the clock and the frames of windows that changed their
    /// size.
    pub fn tick(&mut self) -> Result<(), ClientError> {
        self.sync_sizes()?;
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
        self.windows.push(window);
        self.opened.push(surface.into());
        let mut commands = decoration::create(surface, WINDOWS);
        commands.extend(self.focus(Some(surface)));
        if let Some(err) = self.conn.try_apply(commands)? {
            // Another client took it first.
            eprintln!("chora-xp: {surface}: {err}");
            self.windows.retain(|w| w.surface != surface);
            self.opened.retain(|s| s != surface);
            self.focused = previous;
            return Ok(());
        }
        // The transaction waited for the client to commit its size.
        self.sync_sizes()
    }

    /// Learns the sizes of the clients: new windows show, and frames follow
    /// clients that resized themselves. Titles that no event brought, for
    /// example of a surface that another client gave up, follow too.
    fn sync_sizes(&mut self) -> Result<(), ClientError> {
        let nodes = self.conn.query(&format!("kind:surface owner:{NAME}"))?;
        let mut commands = Vec::new();
        let mut shown = false;
        for node in nodes {
            let info = node.surface.unwrap_or_default();
            commands.extend(self.retitle(&node.id, info.app_id, info.title.as_deref()));
            if let Some(size) = info.size.filter(|[w, h]| *w > 0 && *h > 0) {
                let (resize, first) = self.resized(&node.id, size);
                commands.extend(resize);
                shown |= first;
            }
        }
        if shown {
            commands.extend(self.sync_tasks());
        }
        self.conn.apply(commands)
    }

    /// The commands for a new client size; and whether the window shows for
    /// the first time.
    fn resized(&mut self, surface: &str, size: [i32; 2]) -> (Vec<Command>, bool) {
        let Some(i) = self.index(surface) else {
            return (Vec::new(), false);
        };
        if self.windows[i].size == Some(size) && self.windows[i].shown {
            return (Vec::new(), false);
        }
        let first = !self.windows[i].shown;
        self.windows[i].size = Some(size);
        if first {
            let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
            let outer = self.windows[i].outer();
            self.windows[i].pos = window::cascade(&mut self.cascade, outer, area);
            self.windows[i].shown = true;
        }
        let active = self.is_active(surface);
        (self.windows[i].style(&mut self.theme, active), first)
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

    /// Raises, restores and focuses a window.
    fn activate(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let mut window = self.windows.remove(i);
        let mut commands = if window.minimized {
            window.unminimize()
        } else {
            Vec::new()
        };
        self.windows.push(window);
        commands.push(Command::Move {
            id: decoration::group_id(surface),
            parent: None,
            index: None,
        });
        commands.extend(self.focus(Some(surface)));
        commands.extend(self.sync_tasks());
        commands
    }

    /// The topmost window that shows, other than one.
    fn topmost_except(&self, surface: &str) -> Option<String> {
        self.windows
            .iter()
            .rev()
            .find(|w| w.surface != surface && w.shown && !w.minimized)
            .map(|w| w.surface.clone())
    }

    fn minimize(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        if self.windows[i].minimized || !self.windows[i].shown {
            return Vec::new();
        }
        let target = self
            .bar_at(self.windows[i].rect())
            .and_then(|b| b.task_target(surface))
            .unwrap_or([0.0, 0.0, 160.0, 25.0]);
        let mut commands = vec![self.windows[i].minimize(target)];
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
        let tasks: Vec<Task<'_>> = self
            .opened
            .iter()
            .filter_map(|surface| {
                let w = &self.windows[self.windows.iter().position(|w| &w.surface == surface)?];
                w.shown.then(|| Task {
                    surface,
                    title: &w.title,
                    icon: w.icon.clone(),
                    active: self.focused.as_deref() == Some(surface),
                    minimized: w.minimized,
                })
            })
            .collect();
        let mut commands = Vec::new();
        for bar in &mut self.bars {
            commands.extend(bar.tasks(&mut self.theme, &tasks));
        }
        commands
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
        commands.extend(self.bars[i].resize(&mut self.theme, output.work_area, &self.clock));
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
            commands.push(self.bars.remove(i).remove());
            commands.push(Command::Remove {
                id: wallpaper::id(name),
            });
        }
        commands
    }

    // Input

    fn key(&mut self, binding: &str) -> Vec<Command> {
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
            None => Vec::new(),
        }
    }

    fn pressed(&mut self, node: &str, button: u32, position: [f64; 2]) -> Vec<Command> {
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
            .filter(|i| !self.windows[*i].maximized())
        {
            self.drag = Some(Drag {
                surface: surface.into(),
                start: position,
                origin: self.windows[i].pos,
            });
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
                let mut commands = vec![bar.start(&mut self.theme, true)];
                commands.extend(self.menu.open(&output, bar.origin()));
                commands
            }
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
        let Some(drag) = &self.drag else {
            return Vec::new();
        };
        let Some(i) = self.index(&drag.surface) else {
            return Vec::new();
        };
        #[allow(clippy::cast_possible_truncation)]
        let pos = [
            drag.origin[0] + (px - drag.start[0]).round() as i32,
            drag.origin[1] + (py - drag.start[1]).round() as i32,
        ];
        // The title bar stays below the top edge of the outputs.
        let top = self.bars.iter().map(|b| b.usable()[1]).min().unwrap_or(0);
        self.windows[i].pos = [pos[0], pos[1].max(top)];
        vec![self.windows[i].place()]
    }

    fn released(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        self.drag = None;
        if let Some(Part::Button(surface, button)) = Part::of(node) {
            return self.release_button(&surface, button, position);
        }
        if !Menu::owns(node) {
            return Vec::new();
        }
        let (mut commands, action) = self.menu.release(&mut self.theme, position);
        if let Some(action) = action {
            commands.extend(self.close_menu());
            commands.extend(self.run(&action));
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

    fn close_menu(&mut self) -> Vec<Command> {
        let output = self.menu.output().map(String::from);
        let mut commands = self.menu.close(&mut self.theme);
        if let Some(bar) = output.and_then(|o| self.bars.iter().find(|b| b.output == o)) {
            commands.push(bar.start(&mut self.theme, false));
        }
        commands
    }

    fn run(&mut self, action: &Action) -> Vec<Command> {
        match action {
            Action::Run(command) => {
                spawn(command);
                Vec::new()
            }
            Action::LogOff => self
                .windows
                .iter()
                .flat_map(|w| close(&w.surface))
                .collect(),
            Action::TurnOff => {
                self.done = true;
                Vec::new()
            }
        }
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
