//! Outputs coming, changing and going, and the windows that follow them.

use super::{Bar, Command, OutputInfo, Rect, Wm, moved, wallpaper, window};

impl Wm {
    pub(super) fn output_changed(&mut self, output: &OutputInfo) -> Vec<Command> {
        let Some(i) = self.bars.iter().position(|b| b.output == output.name) else {
            let mut commands = self.add_output(output);
            commands.extend(self.sync_tasks());
            commands.extend(self.claim());
            self.sync_cube_keys();
            self.reserve_bars();
            return commands;
        };
        let mine: Vec<usize> = (0..self.windows.len())
            .filter(|&w| self.bars[i].contains(self.windows[w].pos))
            .collect();
        let mut commands = vec![wallpaper::update(
            &mut self.theme,
            &self.settings.value.wallpaper,
            output,
        )];
        let tray = (self.clock.as_str(), self.icons());
        commands.extend(self.bars[i].resize(&mut self.theme, output, tray));
        commands.extend(self.pagers());
        let area = self.bars[i].usable();
        commands.extend(self.keep_inside(&mine, area));
        commands.extend(self.sync_tasks());
        commands.extend(self.refresh_icons());
        commands.extend(self.claim());
        commands
    }

    /// Moves windows into the new area of their output: a maximized one
    /// fills it, another keeps its size where it fits, else it shrinks
    /// to the area.
    pub(super) fn keep_inside(&mut self, windows: &[usize], area: Rect) -> Vec<Command> {
        let mut commands = Vec::new();
        for &i in windows {
            let active = self.focused.as_deref() == Some(self.windows[i].surface.as_str());
            let w = &mut self.windows[i];
            if w.maximized() {
                w.maximize(area);
                commands.push(w.configure(active));
                commands.extend(w.style(&mut self.theme, active));
                continue;
            }
            if !w.shown {
                continue;
            }
            let [x, y, fw, fh] = window::inside(w.rect(), area);
            if [fw, fh] != w.outer() {
                commands.push(w.fit([x, y, fw, fh], active));
                commands.extend(w.style(&mut self.theme, active));
            } else if [x, y] != w.pos {
                w.pos = [x, y];
            } else {
                continue;
            }
            commands.push(w.place());
        }
        commands
    }

    pub(super) fn output_removed(&mut self, name: &str) -> Vec<Command> {
        let mut commands = Vec::new();
        if self.menu.output() == Some(name) {
            self.open.shut(super::surfaces::MENU);
            commands.extend(self.menu.close(&mut self.theme));
        }
        if let Some(i) = self.bars.iter().position(|b| b.output == name) {
            let bar = self.bars.remove(i);
            commands.push(bar.remove());
            commands.push(Command::Remove {
                id: wallpaper::id(name),
                to: None,
            });
            if let Some(target) = self.bars.first().map(Bar::usable) {
                commands.extend(self.move_windows(&bar, target));
            }
            commands.extend(self.refresh_icons());
            commands.extend(self.claim());
            self.sync_cube_keys();
        }
        commands
    }

    /// Moves the windows of a removed output to the area of another one, to
    /// the same place, as far as it fits.
    pub(super) fn move_windows(&mut self, bar: &Bar, target: Rect) -> Vec<Command> {
        let from = bar.usable();
        let mut commands = Vec::new();
        for i in 0..self.windows.len() {
            if bar.contains(self.windows[i].pos) {
                commands.extend(self.move_window(i, from, target));
            }
        }
        commands
    }

    /// Moves one window from the area of an output to that of another.
    pub(super) fn move_window(&mut self, i: usize, from: Rect, target: Rect) -> Vec<Command> {
        let mut commands = Vec::new();
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
        commands
    }
}
