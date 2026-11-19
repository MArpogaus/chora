//! Menus, dialogs and panels: opening, drawing, clicking and closing them, and the taskbar and icons.

use super::{
    Action, BARS, Bar, Change, Click, Command, DIALOGS, DOUBLE_CLICK, DateDialog, DisplayDialog,
    EffectsDialog, HIDE_DELAY, HelpDialog, Hit, Instant, Message, Mode, NAME, Outcome, Panel, Path,
    Rect, Setting, Store, TaskbarDialog, Value, Wm, apps, context, control, desktop_icons, display,
    help, json, launch, taskbar, volume, wallpaper,
};

impl Wm {
    /// Tells the user that a program starts or did not open.
    pub(super) fn tell(&mut self, change: &Change) {
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

    pub(super) fn close_menu(&mut self) -> Vec<Command> {
        let output = self.menu.output().map(String::from);
        let mut commands = self.menu.close(&mut self.theme);
        if let Some(output) = output {
            commands.push(self.desktop_keys());
            commands.extend(self.start_look(&output));
        }
        commands
    }

    pub(super) fn run(&mut self, action: &Action, output: Option<&str>) -> Vec<Command> {
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
    pub(super) fn raise_panel(&mut self, kind: fn(&Panel) -> bool) -> Option<Vec<Command>> {
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
    pub(super) fn open_taskbar_dialog(&mut self, area: Rect) -> Vec<Command> {
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
    pub(super) fn open_help(&mut self, topic: usize) -> Vec<Command> {
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
    pub(super) fn open_date_dialog(&mut self, area: Rect) -> Vec<Command> {
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
    pub(super) fn open_display_dialog(&mut self, area: Rect) -> Vec<Command> {
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
    pub(super) fn with_display(&mut self, change: impl FnOnce(&mut DisplayDialog)) -> Vec<Command> {
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
    pub(super) fn click_display(&mut self, i: usize, hit: &Hit) -> Vec<Command> {
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
    pub(super) fn open_display_list(
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
    pub(super) fn open_effects(&mut self) -> Vec<Command> {
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
    pub(super) fn close_panel(&mut self, i: usize) -> Vec<Command> {
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
    pub(super) fn change(&mut self, changes: Value) -> Vec<Command> {
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
    pub(super) fn configure(&mut self) -> Vec<Command> {
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
    pub(super) fn press_tray(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
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
    pub(super) fn release_panel(&mut self, position: [f64; 2]) -> Vec<Command> {
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
    pub(super) fn redraw_panel(&mut self, i: usize) -> Vec<Command> {
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
    pub(super) fn set_taskbar(&mut self, settings: taskbar::Settings) -> Vec<Command> {
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
    pub(super) fn refit(&mut self, old: Rect, area: Rect) -> Vec<Command> {
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
    pub(super) fn auto_hide(&mut self) -> Vec<Command> {
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
    pub(crate) fn slide_frame(&mut self, now: Instant, dt: f64) -> Vec<Command> {
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
    pub(crate) fn sliding(&self) -> bool {
        self.hide_at.is_some() || self.bars.iter().any(Bar::sliding)
    }

    /// Looks at the Desktop folder and the trash, and shows the icons on
    /// the work area of the first output.
    pub(super) fn refresh_icons(&mut self) -> Vec<Command> {
        let Some(area) = self.bars.first().map(Bar::usable) else {
            return Vec::new();
        };
        self.desk.refresh(&mut self.theme, &mut self.icons, area)
    }

    /// A click on the desktop takes the focus from the windows and the
    /// selection from the icons; a drag draws the selection rectangle.
    pub(super) fn press_wallpaper(&mut self, position: [f64; 2]) -> Vec<Command> {
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
    pub(super) fn press_icon(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
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
    pub(super) fn open_icon(&mut self, open: desktop_icons::Open) -> Vec<Command> {
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
}
