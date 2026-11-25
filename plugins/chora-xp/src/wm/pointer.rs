//! Pointer work: pressing on parts, dragging, releasing, handing windows over and launching.

use super::{
    Bar, Button, Command, DATETIME_TASK, DISPLAY_TASK, DOUBLE_CLICK, DesktopIcons, Drag, HELP_TASK,
    Instant, LEFT_BUTTON, Menu, NAME, Panel, Part, RIGHT_BUTTON, RpcError, Value, Wm, apps, close,
    context, control, decoration, dialog, dropped_on, launch, pulled_out, shared, speed, startmenu,
    taskbar, wallpaper,
};

impl Wm {
    /// Only the left button works the taskbar, the menu and the desktop.
    pub(super) fn press_left(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        if node.starts_with("xp-tray-") || node.starts_with("xp-speaker-") {
            self.press_tray(position)
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

    pub(super) fn press_decoration(
        &mut self,
        part: &Part,
        button: u32,
        position: [f64; 2],
    ) -> Vec<Command> {
        let (Part::Frame(surface)
        | Part::Title(surface)
        | Part::Button(surface, _)
        | Part::Grip(surface)) = part;
        let mut commands = self.activate(surface);
        let Some(i) = self.index(surface) else {
            return commands;
        };
        if button == RIGHT_BUTTON && matches!(part, Part::Title(_)) {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a pointer position in pixels"
            )]
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
    pub(super) fn press_icon_of(&mut self, surface: &str) -> Vec<Command> {
        let now = Instant::now();
        let double = self
            .last_icon
            .take()
            .is_some_and(|(s, at)| s == surface && now.duration_since(at) < DOUBLE_CLICK);
        if double {
            return close(surface);
        }
        self.last_icon = Some((surface.into(), now));
        self.system_below_icon(surface)
    }

    /// The system menu of a window below the icon of its title bar, as a
    /// click on the icon and Alt+Space open it.
    pub(super) fn system_below_icon(&mut self, surface: &str) -> Vec<Command> {
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
    pub(super) fn press_title(&mut self, surface: &str, position: [f64; 2]) -> Vec<Command> {
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
                kept: false,
                speed: [0.0; 2],
                moved_at: Instant::now(),
            });
            self.start_wobble(surface, position);
            return self.shut_surfaces();
        }
        Vec::new()
    }

    pub(super) fn press_taskbar(&mut self, part: taskbar::Part) -> Vec<Command> {
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
                self.menu_opened();
                commands.extend(self.start_look(&output));
                commands.push(Command::FocusNode {
                    node: Some(startmenu::FOCUS.into()),
                });
                commands
            }
            taskbar::Part::Pager(desktop) => self.go_to_desk(desktop),
            taskbar::Part::Speaker(_) => Vec::new(),
            taskbar::Part::Helper(_) => {
                self.start_program(control::CHAT);
                Vec::new()
            }
            taskbar::Part::Task(surface) if surface == HELP_TASK => self.focus_help(),
            taskbar::Part::Task(surface) if surface == DISPLAY_TASK => self.focus_display(),
            taskbar::Part::Task(surface) if surface == DATETIME_TASK => self.focus_clock(),
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

    pub(super) fn dragged(&mut self, [px, py]: [f64; 2]) -> Vec<Command> {
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
        #[expect(
            clippy::float_cmp,
            reason = "a stored position is compared with the same value, not a computed one"
        )]
        if drag.last == [px, py] {
            return Vec::new();
        }
        let now = Instant::now();
        drag.speed = speed(drag.speed, drag.last, [px, py], now - drag.moved_at);
        drag.moved_at = now;
        drag.last = [px, py];
        let (surface, start, origin) = (drag.surface.clone(), drag.start, drag.origin);
        if let Some(commands) = self.pull_out(&surface, start, origin, [px, py]) {
            return commands;
        }
        let Some(i) = self.index(&surface) else {
            return Vec::new();
        };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a rounded pointer delta in pixels"
        )]
        let pos = [
            origin[0] + (px - start[0]).round() as i32,
            origin[1] + (py - start[1]).round() as i32,
        ];
        let top = self.bars.iter().map(|b| b.usable()[1]).min().unwrap_or(0);
        self.windows[i].pos = [pos[0], pos[1].max(top)];
        let mut commands = vec![self.windows[i].place()];
        commands.extend(self.wobble_mesh(&surface));
        commands
    }

    /// The release of a button; after a drag of a window the task
    /// buttons follow the window to the output where it lies now.
    pub(super) fn released(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        let dragged = self.drag.is_some();
        let mut commands = self.release(node, position);
        if dragged {
            commands.extend(self.sync_tasks());
        }
        commands
    }

    /// What [`Self::released`] does, before the task buttons follow a
    /// window that a drag moved to another output.
    pub(super) fn release(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        if let Some((menu, area)) = self.context.as_mut().filter(|_| context::Menu::owns(node)) {
            let area = *area;
            let Some(act) = menu.release(position) else {
                return Vec::new();
            };
            return self.close_and_act(act, area);
        }
        if let Some(drag) = self.drag.take() {
            self.release_wobble(&drag.surface);
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a pointer position in pixels"
            )]
            let point = position.map(|p| p.floor() as i32);
            if !self.bars.iter().any(|b| b.contains(point)) {
                return self.hand_over(&drag.surface, position);
            }
            if let Some(commands) = self.drop_on_far_output(&drag, point) {
                return commands;
            }
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
        if self.desk.pressed() {
            return self.release_icons(position);
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
            commands.extend(self.close_and_run(&action));
        }
        commands
    }

    /// Places a window whose title bar was dropped on an output that
    /// shows no part of it, inside that output.
    pub(super) fn drop_on_far_output(
        &mut self,
        drag: &Drag,
        point: [i32; 2],
    ) -> Option<Vec<Command>> {
        let i = self.index(&drag.surface)?;
        let area = self.bars.iter().find(|b| b.contains(point))?.usable();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer offset in pixels, floored"
        )]
        let grab = [0, 1].map(|axis| drag.start[axis].floor() as i32 - drag.origin[axis]);
        self.windows[i].pos = dropped_on(self.windows[i].rect(), area, point, grab)?;
        let mut commands = vec![self.windows[i].place()];
        commands.extend(self.keep_inside(&[i], area));
        Some(commands)
    }

    /// The release after a press on the desktop: icons dropped on a window
    /// or on the Recycle Bin go back to their places, as XP shows "not
    /// allowed" there; else they go into their cells.
    pub(super) fn release_icons(&mut self, position: [f64; 2]) -> Vec<Command> {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer position in pixels"
        )]
        let [x, y] = position.map(|p| p.floor() as i32);
        let on_window = self
            .windows
            .iter()
            .filter(|w| w.desktop == self.desktop && w.shown && !w.minimized)
            .any(|w| {
                let [wx, wy, ww, wh] = w.rect();
                (wx..wx + ww).contains(&x) && (wy..wy + wh).contains(&y)
            });
        if (on_window || self.desk.on_trash([x, y]))
            && let Some(commands) = self.desk.drop_out()
        {
            return commands;
        }
        self.desk
            .release(&mut self.theme, &mut self.icons, position)
    }

    /// A drag of a title bar that leaves the face where it started gives
    /// the window with the drag to the client that shows the face, so
    /// that the window follows the pointer out of the cube (E3). The
    /// commands after the give; `None` on a screen, inside the face, and
    /// after the core refused, as it does for an injected drag: then the
    /// window stays with chora-xp.
    #[allow(clippy::cast_possible_truncation, reason = "scene coordinates")]
    pub(super) fn pull_out(
        &mut self,
        surface: &str,
        start: [f64; 2],
        origin: [i32; 2],
        pointer: [f64; 2],
    ) -> Option<Vec<Command>> {
        if self.drag.as_ref().is_none_or(|d| d.kept) {
            return None;
        }
        let pixel = |p: [f64; 2]| p.map(|v| v.floor() as i32);
        let face = self.bars.iter().find(|b| b.contains(pixel(start)))?;
        let to = pulled_out(face.area(), face.owner(), pixel(pointer))?.to_owned();
        let grab = chora_scene::GivenGrab {
            point: [
                start[0] - f64::from(origin[0]),
                start[1] - f64::from(origin[1]),
            ],
            velocity: self.drag.as_ref().map_or([0.0; 2], |d| d.speed),
            wobble: None,
        };
        match self.give_dragged(surface, &to, Some(grab)) {
            Ok((_, commands)) => Some(commands),
            Err(err) => {
                debug_stays_on_face(surface, &err.message);
                if let Some(drag) = &mut self.drag {
                    drag.kept = true;
                }
                None
            }
        }
    }

    /// A window dragged out of the work areas of `chora-xp` goes to the
    /// window manager whose claim holds the drop point, which places it by
    /// its own rules (D55). Without one there, or when the core refuses,
    /// the window stays.
    pub(super) fn hand_over(&mut self, surface: &str, point: [f64; 2]) -> Vec<Command> {
        let given = match self.rt.conn.claimant_at(point) {
            Ok(Some(to)) => self.give(surface, &to),
            Ok(None) => Err(RpcError::new(RpcError::REJECTED, "no claim there")),
            Err(err) => Err(err),
        };
        let err = match given {
            Ok((_, commands)) => return commands,
            Err(err) => err,
        };
        info_stays(surface, &err.message);
        let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
        match self.index(surface) {
            Some(i) => self.keep_inside(&[i], area),
            None => Vec::new(),
        }
    }

    /// Hands a window to the window manager `to` (D55), which gives it the
    /// focus: the answer of `window.give`, and the commands.
    pub(super) fn give(
        &mut self,
        surface: &str,
        to: &str,
    ) -> Result<(Value, Vec<Command>), RpcError> {
        self.give_dragged(surface, to, None)
    }

    /// As [`Self::give`]; with `grab` the drag of the title bar goes too,
    /// with the springs of the wobble unless the caller sent some, and the
    /// drag of chora-xp ends.
    pub(super) fn give_dragged(
        &mut self,
        surface: &str,
        to: &str,
        mut grab: Option<chora_scene::GivenGrab>,
    ) -> Result<(Value, Vec<Command>), RpcError> {
        if let Some(grab) = &mut grab {
            let springs = self.wobble_springs(surface);
            grab.wobble = grab.wobble.take().or(springs);
        }
        let dragged = grab.is_some();
        if to == NAME {
            return Err(RpcError::new(
                RpcError::INVALID_PARAMS,
                format!("{surface} is with {to} already"),
            ));
        }
        if !self.book.give(surface, to) {
            return Err(RpcError::new(
                RpcError::REJECTED,
                format!("xp has no window {surface}"),
            ));
        }
        let answer = match self.rt.conn.give_dragged(surface, to, grab) {
            Ok(answer) => answer,
            Err(err) => {
                self.book.refused(surface);
                return Err(err);
            }
        };
        for change in self.book.confirm(surface) {
            if let chora_kit::wm::Change::Phase { id, phase } = change {
                self.tell_phase(&id, phase);
            }
        }
        if dragged {
            self.drag.take_if(|d| d.surface == surface);
        }
        if self.is_active(surface) {
            self.focused = None;
            self.grab_system_key(false);
        }
        let mut commands = self.shut_surfaces();
        commands.extend(self.forget(surface));
        Ok((answer, commands))
    }

    /// A button acts when the pointer goes up on it.
    pub(super) fn release_button(
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

    /// Starts a program and follows it until its window shows.
    pub(super) fn start_program(&mut self, command: &str) {
        let exe = std::env::current_exe().unwrap_or_else(|_| "chora-xp".into());
        let themed = launch::themed(
            command,
            &self.theme.path("fuzzel.ini"),
            &exe.to_string_lossy(),
            &self.settings.value.terminal,
        );
        let themed = shared::launcher(&themed);
        if command == apps::NO_EDITOR
            || self
                .notepad
                .chain
                .first()
                .is_some_and(|c| c.command == command)
        {
            let notepad = (self.notepad.chain.clone(), self.notepad.skipped.clone());
            let token = self.rt.conn.activation_token();
            for change in self
                .launches
                .run_chain(apps::NOTEPAD, notepad, token, Instant::now())
            {
                self.tell(&change);
            }
            return;
        }
        self.start_named(&themed, launch::program_name(command));
    }

    /// Starts a program that the user knows by another name than its
    /// command, such as a file that `xdg-open` opens.
    pub(super) fn start_named(&mut self, command: &str, name: String) {
        let token = self.rt.conn.activation_token();
        if let Some(change) = self.launches.run(command, (name, token), Instant::now()) {
            self.tell(&change);
        }
    }

    /// Shows what the programs that wait for a window call for.
    pub(super) fn check_launches(&mut self) {
        let mut changes = self.launches.collect();
        changes.extend(self.launches.check(Instant::now()));
        for change in &changes {
            self.tell(change);
        }
    }
}

fn debug_stays_on_face(surface: &str, why: &str) {
    tracing::debug!("{surface} stays on its face: {why}");
}

fn info_stays(surface: &str, why: &str) {
    tracing::info!("{surface} stays: {why}");
}

#[cfg(test)]
mod tests_moved;
