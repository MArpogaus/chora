//! Presses on nodes: the context menus, window actions and arranging.

use super::surfaces::Escape;
use super::{
    Act, Action, Command, DesktopIcons, LEFT_BUTTON, MENU_KEY, MENUS, Menu, Panel, Part, Plan,
    RIGHT_BUTTON, Rect, Wm, close, context, desktop_icons, desktop_plan, json, taskbar, wallpaper,
    window,
};

impl Wm {
    /// A press on a context menu goes to it; one elsewhere closes it and
    /// goes on.
    pub(super) fn pressed(&mut self, node: &str, button: u32, position: [f64; 2]) -> Vec<Command> {
        if let Some((menu, _)) = self.context.as_mut().filter(|_| context::Menu::owns(node)) {
            menu.press(node);
            return Vec::new();
        }
        self.resize = None;
        if let Some(used) = &mut self.tap {
            *used = true;
        }
        let mut commands = self.tip.cancel(None);
        commands.extend(self.close_context());
        commands.extend(self.press_node(node, button, position));
        commands
    }

    pub(super) fn press_node(
        &mut self,
        node: &str,
        button: u32,
        position: [f64; 2],
    ) -> Vec<Command> {
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
        if self.index(node).is_some() {
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
    pub(super) fn press_panel(&mut self, i: usize, node: &str, position: [f64; 2]) -> Vec<Command> {
        let mut panel = self.take_panel(i);
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
        self.push_panel(panel);
        if let Some(group) = dragging {
            self.start_wobble(&group, position);
        }
        commands
    }

    /// The right button opens the menu of the clock, of the taskbar or of
    /// the desktop.
    pub(super) fn press_right(&mut self, node: &str, position: [f64; 2]) -> Vec<Command> {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a pointer position in pixels"
        )]
        let point = position.map(|p| p.floor() as i32);
        if let Some(taskbar::Part::Task(surface)) = taskbar::Part::of(node) {
            return self.open_system(&surface, point);
        }
        if let Some(commands) = self.open_item_menu(node, point) {
            return commands;
        }
        let entries = if node.starts_with("xp-tray-") || node.starts_with("xp-speaker-") {
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
        self.show_context(menu, usable)
    }

    /// XP's system menu of a window at a point.
    pub(super) fn open_system(&mut self, surface: &str, point: [i32; 2]) -> Vec<Command> {
        let Some(i) = self.index(surface) else {
            return Vec::new();
        };
        let w = &self.windows[i];
        let (maximized, minimized) = (w.maximized(), w.minimized);
        let entries = context::system(maximized, minimized, &self.looks_running());
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
        self.sys_window = Some(surface.into());
        let commands = self.show_context(menu, usable);
        self.context_opened(Some(surface));
        commands
    }

    /// The client names of the other looks that can take a window.
    fn looks_running(&mut self) -> Vec<String> {
        #[derive(serde::Deserialize)]
        struct Client {
            name: String,
            methods: Vec<String>,
        }
        let list: Vec<Client> = self
            .rt
            .conn
            .call("clients.list", serde_json::Value::Null)
            .unwrap_or_default();
        (list.into_iter())
            .filter(|c| c.methods.iter().any(|m| m == chora_kit::give::METHOD))
            .map(|c| c.name)
            .collect()
    }

    /// Gives the window of the Send To menu to a look.
    pub(super) fn act_give(&mut self, to: &str) -> Vec<Command> {
        let Some(surface) = self.sys_window.take() else {
            return Vec::new();
        };
        if let Err(err) = chora_kit::give::Windows::hand(self, &surface, to) {
            self.notice(None, "Send To", &err.message, -1);
        }
        Vec::new()
    }

    /// Does what a row of the system menu says to its window.
    pub(super) fn act_window(&mut self, sys: context::Sys) -> Vec<Command> {
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
            context::Sys::Close => close(&surface),
        }
    }

    /// Shows a context menu, which takes the keyboard.
    pub(super) fn show_context(&mut self, menu: context::Menu, area: Rect) -> Vec<Command> {
        let mut commands = menu.create(&mut self.theme, MENUS);
        commands.push(Command::FocusNode {
            node: Some(context::FOCUS.into()),
        });
        self.context = Some((menu, area));
        self.context_opened(None);
        commands
    }

    /// A key while a node of the plugin has the keyboard: a context menu
    /// or the start menu.
    pub(super) fn node_key(&mut self, key: &str) -> Vec<Command> {
        if key == "Escape" {
            match self.escape_target() {
                Some(Escape::Dialog) => return self.close_dialog(),
                Some(Escape::Help(i)) => return self.close_panel(i),
                Some(Escape::Menu) | None => {}
            }
        }
        if let Some((menu, area)) = &mut self.context {
            let area = *area;
            return match menu.key(&mut self.theme, MENUS, key) {
                context::Key::Commands(commands) => commands,
                context::Key::Act(act) => self.close_and_act(act, area),
                context::Key::Close => self.close_context(),
            };
        }
        if !self.menu.is_open() {
            return if key == MENU_KEY {
                self.desktop_menu_key()
            } else {
                Vec::new()
            };
        }
        match self.menu.key(&mut self.theme, &mut self.icons, key) {
            context::Key::Commands(commands) => commands,
            context::Key::Act(action) => self.close_and_run(&action),
            context::Key::Close => self.close_menu(),
        }
    }

    /// Shift+F10 or the Menu key while the desktop has the keyboard: the
    /// menu of the selected icon, or of the desktop, with its access keys.
    pub(super) fn desktop_menu_key(&mut self) -> Vec<Command> {
        let (node, point) = if let Some(selected) = self.desk.selected_node() {
            selected
        } else {
            let Some(bar) = self.bars.first() else {
                return Vec::new();
            };
            let [x, y, w, h] = bar.usable();
            (wallpaper::id(&bar.output), [x + w / 2, y + h / 2])
        };
        let mut commands = self.press_right(&node, point.map(f64::from));
        if let Some((menu, _)) = &mut self.context {
            commands.extend(menu.show_cues(&mut self.theme));
        }
        commands
    }

    /// The keyboard of the plugin's nodes: the desktop's while no window
    /// has the focus, for Shift+F10 and the Menu key; else none, so that
    /// the focused window gets it.
    pub(super) fn desktop_keys(&self) -> Command {
        Command::FocusNode {
            node: self.focused.is_none().then(|| desktop_icons::GROUP.into()),
        }
    }

    /// A row of a context menu acts: the menu closes first.
    pub(super) fn close_and_act(&mut self, act: Act, area: Rect) -> Vec<Command> {
        let mut commands = self.close_context();
        commands.extend(self.act(act, area));
        commands
    }

    /// An entry of the start menu acts on its output: the menu closes
    /// first.
    pub(super) fn close_and_run(&mut self, action: &Action) -> Vec<Command> {
        let output = self.menu.output().map(String::from);
        let mut commands = self.close_menu();
        commands.extend(self.run(action, output.as_deref()));
        commands
    }

    pub(super) fn close_context(&mut self) -> Vec<Command> {
        let Some((menu, _)) = self.context.take() else {
            return Vec::new();
        };
        self.open.shut(super::surfaces::CONTEXT);
        let mut commands = menu.remove();
        commands.extend(self.menu.hold(&mut self.theme, None));
        commands.push(self.desktop_keys());
        commands
    }

    /// Show the desktop: puts the windows of the desktop away, or brings
    /// back the same windows when it did that and they are still away.
    pub(super) fn toggle_desktop(&mut self) -> Vec<Command> {
        let shown: Vec<String> = self
            .windows
            .iter()
            .filter(|w| w.shown && !w.minimized && w.desktop == self.desktop)
            .map(|w| w.surface.clone())
            .collect();
        let away: Vec<(&str, bool)> = self
            .windows
            .iter()
            .map(|w| (w.surface.as_str(), w.minimized))
            .collect();
        match desktop_plan(&self.hidden, &shown, &away) {
            Plan::Away(surfaces) => {
                self.hidden.clone_from(&surfaces);
                surfaces.iter().flat_map(|s| self.minimize(s)).collect()
            }
            Plan::Back(surfaces) => {
                self.hidden.clear();
                surfaces.iter().flat_map(|s| self.activate(s)).collect()
            }
        }
    }

    /// Super+e: opens the home folder in the file manager, or tells the
    /// person that none is installed and which setting names one.
    pub(super) fn open_files(&mut self) -> Vec<Command> {
        let manager = self.settings.value.file_manager.clone();
        let installed = manager
            .split_whitespace()
            .next()
            .is_some_and(chora_kit::desktop_entry::on_path);
        if installed {
            let home = chora_scene::paths::home().unwrap_or_else(|| "/".into());
            self.start_program(&desktop_icons::open_with(&manager, &home));
        } else {
            self.notice(
                None,
                "No file manager",
                "No file manager is installed. Install one, or name it in the setting file_manager of chora-xp.yaml.",
                10_000,
            );
        }
        Vec::new()
    }

    /// Does what a row of a context menu says, on the output of the menu.
    pub(super) fn act(&mut self, act: Act, area: Rect) -> Vec<Command> {
        match act {
            Act::Cascade => self.arrange_windows(area, None),
            Act::TileRows => self.arrange_windows(area, Some(false)),
            Act::TileColumns => self.arrange_windows(area, Some(true)),
            Act::ShowDesktop => self.toggle_desktop(),
            Act::Lock => {
                let taskbar = taskbar::Settings {
                    locked: !self.taskbar.locked,
                    ..self.taskbar
                };
                self.change(json!({ "taskbar": taskbar }))
            }
            Act::ArrangeByName | Act::ArrangeByType => {
                self.desk
                    .arrange(&mut self.theme, &mut self.icons, act == Act::ArrangeByType)
            }
            Act::Refresh => self.desk.reload(&mut self.theme, &mut self.icons),
            Act::DesktopProperties => self.open_display_dialog(area),
            Act::TaskbarProperties => self.open_taskbar_dialog(area),
            Act::DateTime => self.open_date_dialog(area),
            Act::Window(sys) => self.act_window(sys),
            Act::Give(to) => self.act_give(to),
            Act::Item(item) => self.act_item(item),
            Act::Volume(change) => self.change_volume(change, false),
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
    pub(super) fn arrange_windows(&mut self, area: Rect, columns: Option<bool>) -> Vec<Command> {
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
        let mut places = window::Places::default();
        let mut commands = Vec::new();
        for (n, i) in chosen.into_iter().enumerate() {
            let active = self.is_active(&self.windows[i].surface);
            let w = &mut self.windows[i];
            if let Some(tiles) = &tiles {
                commands.push(w.fit(tiles[n], active));
            } else {
                commands.extend(w.unmaximize(active));
                let turn = places.turn();
                w.pos = places.place(turn, w.outer(), area);
            }
            commands.extend(w.style(&mut self.theme, active));
        }
        commands
    }
}
