//! The Explorer windows in the window manager: windows of the plugin
//! itself, in the stack and the taskbar like those of clients.

use chora_scene::Command;

use super::{DIALOGS, Panel, Wm, desktops::desk_id, spawn};
use crate::{
    confirm::ConfirmDialog,
    context,
    decoration::{self, BORDER, TITLE},
    desktop_icons,
    explorer::{self, Cmd, Explorer, Outcome},
    folder::{self, Place},
    taskbar::{self, Bar},
    window::Window,
};

impl Wm {
    /// Opens an Explorer window at a place, cascaded on the first output,
    /// raised and focused.
    pub(super) fn open_explorer(&mut self, place: Place) -> Vec<Command> {
        self.explorer_count += 1;
        let id = format!("xpx{}", self.explorer_count);
        let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
        let [w, h] = explorer::SIZE;
        let size = [
            w.min(area[2] - 2 * BORDER - 48).max(explorer::MIN_SIZE[0]),
            h.min(area[3] - TITLE - BORDER - 48)
                .max(explorer::MIN_SIZE[1]),
        ];
        let mut e = Explorer::new(&id, place, size);
        let mut window = Window::new(&id, None, Some(&e.title()));
        window.own = true;
        window.icon = e.icon(&mut self.icons);
        window.desktop = self.desktop;
        self.windows.push(window);
        self.opened.push(id.clone());
        let mut commands = decoration::create(&id, &desk_id(self.desktop), true);
        commands.extend(e.create(&mut self.theme, &mut self.icons));
        self.explorers.push(e);
        commands.extend(self.focus(Some(&id)));
        commands.extend(self.resized(&id, size));
        commands
    }

    /// The Explorer window that owns a node of its page.
    pub(super) fn explorer_of(&self, node: &str) -> Option<usize> {
        self.explorers.iter().position(|e| e.owns(node))
    }

    /// Whether a surface is an Explorer window.
    pub(super) fn is_explorer(&self, surface: &str) -> bool {
        self.explorers.iter().any(|e| e.id == surface)
    }

    /// Closes an Explorer window.
    pub(super) fn close_explorer(&mut self, surface: &str) -> Vec<Command> {
        self.explorers.retain(|e| e.id != surface);
        self.forget(surface)
    }

    /// A press on the page of an Explorer window raises it, and the
    /// control under the pointer goes down.
    pub(super) fn press_explorer(
        &mut self,
        i: usize,
        node: &str,
        position: [f64; 2],
    ) -> Vec<Command> {
        let id = self.explorers[i].id.clone();
        let mut commands = self.activate(&id);
        if let Some(i) = self.explorers.iter().position(|e| e.id == id) {
            commands.extend(self.explorers[i].press(&mut self.theme, node, position));
        }
        commands
    }

    /// The top-left corner of the client area of a window in the scene.
    fn client_origin(&self, surface: &str) -> [i32; 2] {
        self.index(surface).map_or([0, 0], |i| {
            let w = &self.windows[i];
            let border = if w.maximized() { 0 } else { BORDER };
            [w.pos[0] + border, w.pos[1] + TITLE]
        })
    }

    /// The release of the button after a press on an Explorer window.
    pub(super) fn release_explorer(&mut self, i: usize, position: [f64; 2]) -> Vec<Command> {
        let id = self.explorers[i].id.clone();
        let [ox, oy] = self.client_origin(&id);
        #[allow(clippy::cast_possible_truncation)]
        let point = [
            position[0].floor() as i32 - ox,
            position[1].floor() as i32 - oy,
        ];
        let (mut commands, outcome) = self.explorers[i].release(&mut self.theme, point);
        commands.extend(self.explorer_outcome(&id, outcome));
        commands
    }

    /// Acts on the outcome of an Explorer window.
    pub(super) fn explorer_outcome(&mut self, id: &str, outcome: Outcome) -> Vec<Command> {
        match outcome {
            Outcome::None => Vec::new(),
            Outcome::Redraw => self.redraw_explorer(id),
            Outcome::Close => self.close_explorer(id),
            Outcome::ControlPanel => self.open_control_panel(),
            Outcome::EmptyTrash(names) => self.confirm_empty(&names),
            Outcome::Open(path) => {
                spawn(&desktop_icons::xdg_open(&path));
                Vec::new()
            }
            Outcome::Menu(entries, [x, y]) => {
                let [ox, oy] = self.client_origin(id);
                let point = [ox + x, oy + y];
                let area = self
                    .bars
                    .iter()
                    .find(|b| b.contains(point))
                    .or(self.bars.first())
                    .map_or([0, 0, 1280, 800], Bar::area);
                let menu = context::Menu::new(&self.theme, entries, point, area, 0);
                let mut commands = self.show_context(menu, area);
                if self.explorers.iter().any(|e| e.id == id && e.bar_open()) {
                    self.bar_menu = Some(id.into());
                    commands.extend(self.redraw_explorer(id));
                }
                commands
            }
        }
    }

    /// A row of an Explorer menu acts on the focused Explorer window.
    pub(super) fn explore(&mut self, cmd: Cmd) -> Vec<Command> {
        let Some(id) = self.focused.clone().filter(|f| self.is_explorer(f)) else {
            return Vec::new();
        };
        let Some(i) = self.explorers.iter().position(|e| e.id == id) else {
            return Vec::new();
        };
        let outcome = self.explorers[i].command(cmd);
        self.explorer_outcome(&id, outcome)
    }

    /// Shows the page of an Explorer window again, with its title and
    /// icon in the title bar and on the task button.
    pub(super) fn redraw_explorer(&mut self, id: &str) -> Vec<Command> {
        let Some(e) = self.explorers.iter_mut().find(|e| e.id == id) else {
            return Vec::new();
        };
        let mut commands = e.redraw(&mut self.theme, &mut self.icons);
        let (title, icon) = (e.title(), e.icon(&mut self.icons));
        let Some(i) = self.index(id) else {
            return commands;
        };
        if self.windows[i].title != title || self.windows[i].icon != icon {
            self.windows[i].title = title;
            self.windows[i].icon = icon;
            let active = self.is_active(id);
            commands.extend(self.windows[i].style(&mut self.theme, active));
            commands.extend(self.sync_tasks());
        }
        commands
    }

    /// The wheel or the touchpad scrolled over an Explorer window.
    pub(super) fn scroll_explorer(&mut self, node: &str, dy: f64, notches: f64) -> Vec<Command> {
        let Some(i) = self.explorer_of(node) else {
            return Vec::new();
        };
        let outcome = self.explorers[i].wheel(dy, notches);
        let id = self.explorers[i].id.clone();
        self.explorer_outcome(&id, outcome)
    }

    /// The pointer moved while the thumb of a scroll bar is down; `None`
    /// when no thumb is down.
    pub(super) fn drag_explorer(&mut self, position: [f64; 2]) -> Option<Vec<Command>> {
        let e = self.explorers.iter_mut().find(|e| e.dragging())?;
        let outcome = e.drag(position);
        let id = e.id.clone();
        Some(self.explorer_outcome(&id, outcome))
    }

    /// Lays out the Explorer windows whose size changed: maximized,
    /// restored or tiled.
    pub(super) fn relayout_explorers(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        for e in &mut self.explorers {
            let Some(size) = self
                .windows
                .iter()
                .find(|w| w.surface == e.id)
                .and_then(|w| w.size)
            else {
                continue;
            };
            if size != e.size() {
                e.resize(size);
                commands.extend(e.redraw(&mut self.theme, &mut self.icons));
            }
        }
        commands
    }

    /// Asks whether to empty the Recycle Bin, unless the question is open
    /// already.
    pub(super) fn confirm_empty(&mut self, names: &[String]) -> Vec<Command> {
        if let Some(commands) = self.raise_panel(|p| matches!(p, Panel::Confirm(_))) {
            return commands;
        }
        let area = self.bars.first().map_or([0, 0, 1280, 770], Bar::usable);
        let icon = self.icons.find_sized("user-trash-full", 32);
        let confirm = ConfirmDialog::new(&self.theme, names, icon, area);
        let page = confirm.page();
        let mut panel = Panel::Confirm(confirm);
        let commands = panel.window().create(&mut self.theme, DIALOGS, page);
        self.panels.push(panel);
        commands
    }

    /// Empties the Recycle Bin for good, and shows the Explorer windows
    /// of it again.
    pub(super) fn empty_trash(&mut self) -> Vec<Command> {
        if let Err(err) = folder::empty_trash(&folder::trash_root()) {
            eprintln!("chora-xp: empty the Recycle Bin: {err}");
        }
        self.refresh_trash()
    }

    /// Shows the desktop icons and the Explorer windows again after the
    /// trash changed: those of the Recycle Bin and of the folders that a
    /// file left.
    pub(super) fn refresh_trash(&mut self) -> Vec<Command> {
        let ids: Vec<String> = self
            .explorers
            .iter_mut()
            .map(|e| {
                e.command(Cmd::Refresh);
                e.id.clone()
            })
            .collect();
        let mut commands: Vec<Command> =
            ids.iter().flat_map(|id| self.redraw_explorer(id)).collect();
        commands.extend(self.refresh_icons());
        commands
    }

    /// The Control Panel of XP's links: the start menu of the first
    /// output with its Control Panel open.
    fn open_control_panel(&mut self) -> Vec<Command> {
        let Some(output) = self.bars.first().map(|b| b.output.clone()) else {
            return Vec::new();
        };
        let mut commands = self.close_menu();
        commands.extend(self.press_taskbar(taskbar::Part::Start(output)));
        commands.extend(
            self.menu
                .open_control(&mut self.theme, &mut self.icons, usize::MAX),
        );
        commands
    }
}
