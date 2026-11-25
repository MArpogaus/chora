//! The windows of the look: adopting, focusing, minimizing, forgetting, requests, tasks and hover.

use super::{
    Bar, ClientError, Command, DATETIME_TASK, DESKTOPS, DISPLAY_TASK, Decoration, Dialog,
    HELP_TASK, Instant, MENUS, Menu, NAME, Panel, Part, PropsUpdate, Rect, RpcError,
    SurfaceRequest, Task, Value, Window, Wm, change_keys, close, context, decoration, desk_id,
    dropped_at, help, home_bar, restart, taskbar, window, window_icon,
};
use chora_kit::wm::focus;

impl Wm {
    pub(super) fn index(&self, surface: &str) -> Option<usize> {
        self.windows.iter().position(|w| w.surface == surface)
    }

    pub(super) fn is_active(&self, surface: &str) -> bool {
        self.focused.as_deref() == Some(surface)
    }

    /// Takes a surface that the book proposes into a hidden group with
    /// its decoration and focuses it; the menus shut, so that the Start
    /// menu does not cover a window that came (sd-h2). It shows once it
    /// has a size. True when the window is xp's now; false when the core
    /// refused it.
    pub(super) fn adopt(
        &mut self,
        surface: &str,
        app_id: Option<String>,
        title: Option<&str>,
    ) -> Result<bool, ClientError> {
        let previous = self.focused.clone();
        let mut window = Window::new(surface, app_id, title);
        window.icon = window_icon(&mut self.icons, window.app_id.as_deref());
        window.desktop = self.kept.get(surface).map_or(self.desktop, |k| k.desktop);
        if !self.kept.contains_key(surface) {
            window.turn = self.places.turn();
        }
        let desk = desk_id(window.desktop);
        self.windows.push(window);
        self.opened.push(surface.into());
        let mut commands = decoration::create(surface, &desk, false);
        if let Some(look) = self.kept.get(surface).map(|k| k.look.clone()) {
            if let Some(opacity) = look.opacity {
                self.floats.seed(surface, opacity);
            }
            commands.push(Command::Update {
                id: decoration::group_id(surface),
                props: look,
            });
        }
        commands.extend(self.focus(Some(surface)));
        if let Some(err) = self.try_apply(commands)? {
            error_adopt(surface, &err);
            self.windows.retain(|w| w.surface != surface);
            self.opened.retain(|s| s != surface);
            self.focused = previous;
            return Ok(false);
        }
        let shut = self.shut_surfaces();
        if !shut.is_empty() {
            self.apply(shut)?;
        }
        let Some(node) = self.rt.conn.query(&format!("id:{surface}"))?.pop() else {
            return Ok(true);
        };
        let info = node.surface.unwrap_or_default();
        let mut commands = self.redecorate(surface, info.decoration);
        commands.extend(self.retitle(surface, info.app_id, info.title.as_deref()));
        if let Some(size) = info.size.filter(|[w, h]| *w > 0 && *h > 0) {
            commands.extend(self.resized(surface, size));
        }
        self.apply(commands)?;
        Ok(true)
    }

    /// The commands for a new client size: a new window shows, and the
    /// frame follows a client that resized itself.
    pub(super) fn resized(&mut self, surface: &str, size: [i32; 2]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        if self.windows[i].size == Some(size) && self.windows[i].shown {
            return Vec::new();
        }
        let first = !self.windows[i].shown;
        self.windows[i].size = Some(size);
        self.anchor(i);
        let area = self.home();
        let kept = first.then(|| self.kept.remove(surface)).flatten();
        let active = self.is_active(surface);
        let mut commands = Vec::new();
        if let Some(k) = kept {
            let w = &mut self.windows[i];
            w.pos = [k.frame[0], k.frame[1]];
            if let Some(restore) = k.restore {
                w.restore = Some(restore);
                w.maximize(area);
                commands.push(w.configure(active));
            }
            w.minimized = k.minimized;
        } else if first && !self.windows[i].maximized() {
            let (turn, outer) = (self.windows[i].turn, self.windows[i].outer());
            self.windows[i].pos = self.places.place(turn, outer, area, self.desk.strip(area));
        }
        self.windows[i].shown = true;
        if self.windows[i].minimized {
            commands.push(self.windows[i].place());
        }
        commands.extend(self.windows[i].style(&mut self.theme, active));
        if first {
            commands.extend(self.sync_tasks());
        }
        commands
    }

    /// Moves the focus: the old and the new window change their look.
    pub(super) fn focus(&mut self, target: Option<&str>) -> Vec<Command> {
        let previous = std::mem::replace(&mut self.focused, target.map(String::from));
        self.arrival = super::Arrival::Settled;
        self.grab_system_key(target.is_some());
        let mut commands = self.close_context();
        commands.extend(
            self.desk
                .focus(&mut self.theme, &mut self.icons, target.is_none()),
        );
        if previous.as_deref() != target {
            for surface in [previous.as_deref(), target].into_iter().flatten() {
                commands.extend(self.restyle(surface));
            }
        }
        commands.push(target.map_or_else(focus::release, focus::give));
        commands.push(self.desktop_keys());
        commands
    }

    /// The configure and the decoration of a window in its current state.
    pub(super) fn restyle(&mut self, surface: &str) -> Vec<Command> {
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
    pub(super) fn activate(&mut self, surface: &str) -> Vec<Command> {
        if surface == HELP_TASK {
            return self.focus_help();
        }
        if surface == DISPLAY_TASK {
            return self.focus_display();
        }
        if surface == DATETIME_TASK {
            return self.focus_clock();
        }
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

    /// The topmost window of the desktop that shows, other than one; on
    /// the faces of a host, of the face at the front.
    pub(super) fn topmost_except(&self, surface: &str) -> Option<String> {
        self.windows
            .iter()
            .rev()
            .find(|w| {
                w.surface != surface
                    && w.shown
                    && !w.minimized
                    && w.desktop == self.desktop
                    && self.on_front(&w.surface)
            })
            .map(|w| w.surface.clone())
    }

    /// The task button of a window in scene coordinates.
    pub(super) fn task_button(&self, surface: &str) -> [f64; 4] {
        self.index(surface)
            .and_then(|i| self.bar_at(self.windows[i].rect()))
            .and_then(|b| b.task_target(surface))
            .unwrap_or([0.0, 0.0, 160.0, 25.0])
    }

    pub(super) fn minimize(&mut self, surface: &str) -> Vec<Command> {
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

    pub(super) fn toggle_maximize(&mut self, surface: &str) -> Vec<Command> {
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
    pub(super) fn forget(&mut self, surface: &str) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let mut commands = self.drop_from_switcher(surface);
        self.floats.forget(surface);
        self.windows.remove(i);
        self.opened.retain(|s| s != surface);
        if self.drag.as_ref().is_some_and(|d| d.surface == surface) {
            self.drag = None;
        }
        if self.resize.as_ref().is_some_and(|r| r.surface == surface) {
            self.resize = None;
        }
        self.effects.driver.cancel(surface);
        commands.push(Command::Remove {
            id: decoration::group_id(surface),
            to: None,
        });
        if self.is_active(surface) || self.dropped_by_its_close(surface) {
            let next = self.topmost_except(surface);
            commands.extend(self.focus(next.as_deref()));
        }
        commands.extend(self.sync_tasks());
        commands
    }

    /// Whether the core stopped focusing the window `surface` as it closed:
    /// it focuses nothing since and no node of another client took the
    /// keys, so the keys would go nowhere (co-51).
    fn dropped_by_its_close(&mut self, surface: &str) -> bool {
        self.dropped.take_if(|d| d == surface).is_some()
            && self.focused.is_none()
            && self.core_focus.is_none()
            && !self.node_away()
    }

    pub(super) fn retitle(
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
        self.windows[i].icon = window_icon(&mut self.icons, app_id.as_deref());
        self.windows[i].app_id = app_id;
        let active = self.is_active(surface);
        let mut commands = self.windows[i].style(&mut self.theme, active);
        commands.extend(self.sync_tasks());
        commands
    }

    /// Who draws the frame: the XP frame shows only for clients that ask
    /// for one; a client that draws its own, as GTK programs do, gets none.
    pub(super) fn redecorate(
        &mut self,
        surface: &str,
        decoration: Option<Decoration>,
    ) -> Vec<Command> {
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

    pub(super) fn request(&mut self, surface: &str, request: SurfaceRequest) -> Vec<Command> {
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

    /// Applies the look that another client asks for a window or its
    /// group (D58): opacity and effects on the group, so that the frame
    /// looks the same, and meta on the asked node without the key of
    /// chora-xp. The plugin sends no opacity or effects to the groups
    /// otherwise, so the look stays; a restart keeps it too.
    pub(super) fn requested_look(&mut self, node: &str, args: &Value) -> Vec<Command> {
        let surface = decoration::surface_of(node).unwrap_or(node);
        let Ok(asked) = serde_json::from_value::<PropsUpdate>(args.clone()) else {
            return Vec::new();
        };
        if self.index(surface).is_none() {
            return Vec::new();
        }
        let mut commands = Vec::new();
        let opacity = asked.opacity.filter(|o| self.floats.ask(surface, *o));
        if opacity.is_some() || asked.effects.is_some() {
            commands.push(Command::Update {
                id: decoration::group_id(surface),
                props: PropsUpdate {
                    opacity,
                    effects: asked.effects,
                    ..PropsUpdate::default()
                },
            });
        }
        let meta = asked.meta.map(|mut meta| {
            meta.remove(restart::KEY);
            meta
        });
        if let Some(meta) = meta.filter(|m| !m.is_empty()) {
            commands.push(Command::Update {
                id: node.into(),
                props: PropsUpdate {
                    meta: Some(meta),
                    ..PropsUpdate::default()
                },
            });
        }
        commands
    }

    /// Whether the plugin has no desktop cube: all its outputs are virtual
    /// ones, which a construct shows on a cube of its own.
    pub(crate) fn cube_off(&self) -> bool {
        !self.bars.is_empty() && self.bars.iter().all(Bar::is_virtual)
    }

    /// Grabs or lets go the bindings of the cube after the outputs
    /// changed: all with a screen, none on virtual outputs alone or on the
    /// faces of a hosted cube.
    pub(super) fn sync_cube_keys(&mut self) {
        self.find_host();
        let cube = self.wanted_keys();
        if self.cube_keys == cube {
            return;
        }
        self.cube_keys = cube;
        let grabbed = change_keys(&mut self.rt, &self.settings.value.common.bindings, cube)
            .and_then(|()| self.rt.grab_give_keys());
        if let Err(err) = grabbed {
            chora_kit::runtime::report(crate::wm::NAME, &err);
        }
    }

    /// The output of a node of an output, as its wallpaper or its taskbar,
    /// or of a window.
    pub(super) fn output_of(&self, node: &str) -> Option<String> {
        let named = |b: &&Bar| node.ends_with(&format!("-{}", b.output));
        let window = || {
            self.index(node)
                .and_then(|i| self.bar_at(self.windows[i].rect()))
        };
        let bar = self.bars.iter().find(named).or_else(window)?;
        Some(bar.output.clone())
    }

    /// The work area for a new window: on the output that the pointer
    /// entered last, else on the first one.
    pub(super) fn home(&self) -> Rect {
        let bar = home_bar(&self.bars, self.pointer_output.as_deref());
        bar.map_or([0, 0, 1280, 770], Bar::usable)
    }

    /// Moves a window to another output of the plugin, to the same place
    /// there, as far as it fits, and raises it. With `at`, a point of
    /// that output, the middle of its title bar goes there instead, as
    /// after a drop.
    pub(super) fn move_to_output(
        &mut self,
        surface: &str,
        output: &str,
        at: Option<[i32; 2]>,
    ) -> Result<Vec<Command>, RpcError> {
        let unknown = |what: &str| RpcError::new(RpcError::REJECTED, format!("xp has no {what}"));
        let i = self.index(surface).ok_or_else(|| unknown(surface))?;
        let target = self.bars.iter().find(|b| b.output == output);
        let target = target.map(Bar::usable).ok_or_else(|| unknown(output))?;
        let from = self
            .bar_at(self.windows[i].rect())
            .map_or(target, Bar::usable);
        let mut commands = self.move_window(i, from, target);
        if let Some(at) = at.filter(|_| !self.windows[i].maximized()) {
            let [.., w, _] = self.windows[i].rect();
            self.windows[i].pos = dropped_at(target, w, at);
            commands.push(self.windows[i].place());
        }
        commands.extend(self.activate(surface));
        Ok(commands)
    }

    /// The taskbar of the output that shows most of a rectangle: the one
    /// with its center, else the first.
    pub(super) fn bar_at(&self, [x, y, w, h]: Rect) -> Option<&Bar> {
        let center = [x + w / 2, y + h / 2];
        self.bars
            .iter()
            .find(|b| b.contains(center))
            .or(self.bars.first())
    }

    /// The task buttons of every taskbar. The taskbar of a virtual output
    /// lists the windows of that output, as each face of a cube is a
    /// desktop of its own; the taskbar of a screen lists all windows, or
    /// with the setting `tasks_per_output` those of its screen.
    pub(super) fn sync_tasks(&mut self) -> Vec<Command> {
        let hovered = match self.hover.as_deref().and_then(taskbar::Part::of) {
            Some(taskbar::Part::Task(surface)) => Some(surface),
            _ => None,
        };
        let panels = self.panel_tasks(hovered.as_deref());
        let tasks: Vec<(Task<'_>, usize)> = self
            .opened
            .iter()
            .filter_map(|surface| {
                let w = &self.windows[self.windows.iter().position(|w| &w.surface == surface)?];
                let task = Task {
                    surface,
                    title: &w.title,
                    icon: w.icon.clone(),
                    active: self.focused.as_deref() == Some(surface),
                    minimized: w.minimized,
                    hovered: hovered.as_deref() == Some(surface.as_str()),
                };
                let [x, y, width, height] = w.rect();
                let centre = [x + width / 2, y + height / 2];
                let bar = self.bars.iter().position(|b| b.contains(centre));
                (w.shown && w.desktop == self.desktop).then_some((task, bar.unwrap_or(0)))
            })
            .collect();
        let per_output = self.settings.value.tasks_per_output;
        let lists: Vec<Vec<Task<'_>>> = (self.bars.iter().enumerate())
            .map(|(i, bar)| {
                let own = per_output || bar.is_virtual();
                let listed = tasks.iter().filter(|(_, at)| !own || *at == i);
                listed.map(|(task, _)| task.clone()).collect()
            })
            .collect();
        drop(tasks);
        let mut lists = lists;
        if let Some(list) = lists.first_mut() {
            list.extend(panels);
        }
        let mut commands = Vec::new();
        for (bar, list) in self.bars.iter_mut().zip(&lists) {
            commands.extend(bar.tasks(&mut self.theme, list));
        }
        commands
    }

    /// The task buttons of the open dialogs that have one, in the order of
    /// Display Properties, Date and Time and Help.
    fn panel_tasks(&mut self, hovered: Option<&str>) -> Vec<Task<'static>> {
        let mut tasks = Vec::new();
        if self.has_display() {
            tasks.push(Task {
                surface: DISPLAY_TASK,
                title: crate::display::WINDOW_TITLE,
                icon: self.icons.find("preferences-desktop-display"),
                active: self.display_active(),
                minimized: false,
                hovered: hovered == Some(DISPLAY_TASK),
            });
        }
        if self.has_clock() {
            tasks.push(Task {
                surface: DATETIME_TASK,
                title: crate::datetime::WINDOW_TITLE,
                icon: self.icons.find("x-office-calendar"),
                active: self.clock_active(),
                minimized: false,
                hovered: hovered == Some(DATETIME_TASK),
            });
        }
        if self.has_help() {
            tasks.push(Task {
                surface: HELP_TASK,
                title: help::TITLE,
                icon: self.icons.find("help-browser"),
                active: self.help_active(),
                minimized: false,
                hovered: hovered == Some(HELP_TASK),
            });
        }
        tasks
    }

    /// Whether Display Properties is open.
    pub(super) fn has_display(&self) -> bool {
        self.panels.iter().any(|p| matches!(p, Panel::Display(_)))
    }

    /// Whether Display Properties has the focus: it, or its Effects, is the
    /// topmost panel and no window has the focus.
    pub(super) fn display_active(&self) -> bool {
        self.focused.is_none()
            && matches!(
                self.panels.last(),
                Some(Panel::Display(_) | Panel::Effects(_))
            )
    }

    /// Takes Display Properties to the top, its Effects above it.
    pub(super) fn focus_display(&mut self) -> Vec<Command> {
        let mut commands = self
            .raise_panel(|p| matches!(p, Panel::Display(_)))
            .unwrap_or_default();
        commands.extend(
            self.raise_panel(|p| matches!(p, Panel::Effects(_)))
                .unwrap_or_default(),
        );
        commands.extend(self.focus(None));
        if let Some(panel) = self.panels.last() {
            commands.push(Command::FocusNode {
                node: Some(panel.window_ref().group()),
            });
        }
        commands.extend(self.sync_tasks());
        commands
    }

    /// Whether Date and Time Properties is open.
    pub(super) fn has_clock(&self) -> bool {
        self.panels.iter().any(|p| matches!(p, Panel::Clock(_)))
    }

    /// Whether Date and Time Properties has the focus: it is the topmost
    /// panel and no window has the focus.
    pub(super) fn clock_active(&self) -> bool {
        self.focused.is_none() && matches!(self.panels.last(), Some(Panel::Clock(_)))
    }

    /// Takes Date and Time Properties to the top.
    pub(super) fn focus_clock(&mut self) -> Vec<Command> {
        let mut commands = self
            .raise_panel(|p| matches!(p, Panel::Clock(_)))
            .unwrap_or_default();
        commands.extend(self.focus(None));
        if let Some(panel) = self.panels.last() {
            commands.push(Command::FocusNode {
                node: Some(panel.window_ref().group()),
            });
        }
        commands.extend(self.sync_tasks());
        commands
    }

    /// Whether Help and Support is open.
    pub(super) fn has_help(&self) -> bool {
        self.panels.iter().any(|p| matches!(p, Panel::Help(_)))
    }

    /// Whether Help and Support has the focus: it is the topmost panel and
    /// no window has the focus.
    pub(super) fn help_active(&self) -> bool {
        self.focused.is_none() && matches!(self.panels.last(), Some(Panel::Help(_)))
    }

    /// Takes Help and Support to the top with the keyboard.
    pub(super) fn focus_help(&mut self) -> Vec<Command> {
        let mut commands = self
            .raise_panel(|p| matches!(p, Panel::Help(_)))
            .unwrap_or_default();
        commands.extend(self.focus(None));
        if let Some(Panel::Help(panel)) = self.panels.last() {
            commands.push(Command::FocusNode {
                node: Some(panel.dialog.group()),
            });
        }
        commands.extend(self.sync_tasks());
        commands
    }

    /// The desktop buttons of every taskbar: on a screen, and on the
    /// faces of a hosted cube.
    pub(super) fn pagers(&mut self) -> Vec<Command> {
        let hovered = match self.hover.as_deref().and_then(taskbar::Part::of) {
            Some(taskbar::Part::Pager(desktop)) => Some(desktop),
            _ => None,
        };
        let hosted = self.host.is_some();
        let desks: Vec<usize> = self.bars.iter().map(|b| self.desk_of(b)).collect();
        let mut commands = Vec::new();
        let mut changed = false;
        for (bar, desk) in self.bars.iter_mut().zip(desks) {
            changed |= bar.show_pager(hosted);
            commands.extend(bar.pager(&mut self.theme, desk, hovered));
        }
        if changed {
            commands.extend(self.sync_tasks());
        }
        commands
    }

    /// The start button of an output: down while its menu is open, lit
    /// under the pointer.
    pub(super) fn start_look(&mut self, output: &str) -> Vec<Command> {
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
    pub(super) fn hovered(&mut self, node: &str, on: bool) -> Vec<Command> {
        if on {
            self.hover = Some(node.into());
            self.pointer_output = self.output_of(node).or(self.pointer_output.take());
        } else if self.hover.as_deref() == Some(node) {
            self.hover = None;
        }
        let mut commands = self.auto_hide();
        commands.extend(self.rest_tip(node, on));
        if let Some((menu, _)) = self.context.as_mut().filter(|_| context::Menu::owns(node)) {
            commands.extend(menu.hover(&mut self.theme, MENUS, node, on));
            return commands;
        }
        if let Some(panel) = self.panels.iter_mut().find(|p| p.window_ref().owns(node)) {
            commands.extend(panel.window().hover(&mut self.theme, node, on));
            return commands;
        }
        commands.extend(self.hovered_node(node, on));
        commands
    }

    /// The pointer came to a part of the taskbar that has a tooltip, or left
    /// it: the tooltip waits, or goes.
    pub(super) fn rest_tip(&mut self, node: &str, on: bool) -> Vec<Command> {
        if !on {
            return self.tip.cancel(Some(node));
        }
        if Menu::owns(node) {
            if let Some((full, row)) = self.menu.cut_title(&self.theme, node) {
                self.tip.rest(node, full, row, Instant::now());
            }
            return Vec::new();
        }
        let Some(output) = self.output_of(node) else {
            return Vec::new();
        };
        let Some(bar) = self.bars.iter().find(|b| b.output == output) else {
            return Vec::new();
        };
        if node.starts_with("xp-tray-") {
            if let Some(clock) = bar.clock_rect() {
                let date = chrono::Local::now().format("%A, %B %-d, %Y").to_string();
                self.tip.rest(node, date, clock, Instant::now());
            }
            return Vec::new();
        }
        let (text, target) = match taskbar::Part::of(node) {
            Some(taskbar::Part::Pager(desktop)) => (
                format!("Desktop {} of {DESKTOPS}", desktop + 1),
                bar.pager_rect(desktop),
            ),
            Some(taskbar::Part::Speaker(_)) => (self.speaker_tip(), bar.speaker_rect()),
            Some(taskbar::Part::Helper(_)) => (self.helper_tip(), bar.helper_rect()),
            _ => return Vec::new(),
        };
        self.tip.rest(node, text, target, Instant::now());
        Vec::new()
    }

    pub(super) fn hovered_node(&mut self, node: &str, on: bool) -> Vec<Command> {
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
            Some(taskbar::Part::Speaker(_) | taskbar::Part::Helper(_)) | None => Vec::new(),
        }
    }

    /// The claim of the plugin on its root group: the box around the work
    /// areas of its outputs, where new windows come to it (D55).
    pub(super) fn claim(&self) -> Option<Command> {
        let areas = self.bars.iter().map(Bar::area);
        let x0 = areas.clone().map(|a| a[0]).min()?;
        let y0 = areas.clone().map(|a| a[1]).min()?;
        let x1 = areas.clone().map(|a| a[0] + a[2]).max()?;
        let y1 = areas.map(|a| a[1] + a[3]).max()?;
        Some(Command::Update {
            id: NAME.into(),
            props: PropsUpdate {
                claim: Some(Some([x0, y0, x1 - x0, y1 - y0].map(f64::from))),
                ..PropsUpdate::default()
            },
        })
    }
}

fn error_adopt(surface: &str, err: &dyn std::fmt::Display) {
    tracing::error!("{surface}: {err}");
}
