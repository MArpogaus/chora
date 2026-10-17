//! The Explorer windows in the window manager: windows of the plugin
//! itself, in the stack and the taskbar like those of clients.

use chora_scene::Command;

use super::{MENUS, Wm, desktops::desk_id};
use crate::{
    context,
    decoration::{self, BORDER, TITLE},
    explorer::{self, Cmd, Explorer, Outcome},
    folder::Place,
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
    pub(super) fn press_explorer(&mut self, i: usize, node: &str) -> Vec<Command> {
        let id = self.explorers[i].id.clone();
        let mut commands = self.activate(&id);
        if let Some(i) = self.explorers.iter().position(|e| e.id == id) {
            commands.extend(self.explorers[i].press(&mut self.theme, node));
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

    /// Does what a click on an Explorer window says.
    fn explorer_outcome(&mut self, id: &str, outcome: Outcome) -> Vec<Command> {
        match outcome {
            Outcome::None => Vec::new(),
            Outcome::Redraw => self.redraw_explorer(id),
            Outcome::Close => self.close_explorer(id),
            Outcome::ControlPanel => self.open_control_panel(),
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
                let commands = menu.create(&mut self.theme, MENUS);
                self.context = Some((menu, area));
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
    fn redraw_explorer(&mut self, id: &str) -> Vec<Command> {
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
