//! The context menus of the desktop icons and of the entries of the start
//! menu, and what their rows do: open and leave the start menu.

use chora_scene::Command;

use super::{Target, Wm};
use crate::{
    context::{self, Item},
    desktop_icons::{DesktopIcons, Open},
    startmenu::Menu,
    taskbar::Bar,
    theme::Rect,
};

impl Wm {
    /// The menu of the icon or the start menu entry under the pointer;
    /// `None` when the node is neither.
    pub(super) fn open_item_menu(&mut self, node: &str, point: [i32; 2]) -> Option<Vec<Command>> {
        let (target, kind, mut commands) = if Menu::owns(node) {
            let item = self.menu.item(node, &mut self.icons)?;
            let kind = context::Target::Program(item.program.is_some());
            let held = self.menu.hold(&mut self.theme, Some(node));
            (Target::Start(item), kind, held)
        } else if DesktopIcons::owns(node) {
            let item = self.desk.item(node)?;
            let kind = match &item.open {
                Open::Folder(_) => context::Target::Folder,
                Open::Computer => context::Target::Computer,
                Open::Trash => context::Target::Trash,
                Open::Run(_) | Open::File(_) => context::Target::File,
            };
            let mut commands = self.focus(None);
            commands.extend(self.desk.select_for_menu(
                &mut self.theme,
                &mut self.icons,
                item.key.clone(),
            ));
            (Target::Icon(item), kind, commands)
        } else {
            return None;
        };
        let area = self
            .bars
            .iter()
            .find(|b| b.contains(point))
            .or(self.bars.first())
            .map_or([0, 0, 1280, 800], Bar::area);
        let menu = context::Menu::new(&self.theme, context::item(kind), point, area, 0);
        commands.extend(self.show_context(menu, area));
        self.item = Some(target);
        Some(commands)
    }

    /// Does what a row of the menu of an icon or an entry says.
    pub(super) fn act_item(&mut self, act: Item) -> Vec<Command> {
        match self.item.take() {
            Some(Target::Start(item)) => {
                let output = self.menu.output().map(String::from);
                let mut commands = self.close_menu();
                match act {
                    Item::Open => {
                        commands.extend(self.run(&item.action, output.as_deref()));
                    }
                    Item::Remove => {
                        if let Some(i) = item.program {
                            commands.extend(self.menu.remove_program(
                                i,
                                &mut self.theme,
                                &mut self.icons,
                            ));
                        }
                    }
                }
                commands
            }
            Some(Target::Icon(item)) => match act {
                Item::Open => self.open_icon(item.open),
                Item::Remove => Vec::new(),
            },
            None => Vec::new(),
        }
    }

    pub(super) fn work_area(&self) -> Rect {
        self.bars.first().map_or([0, 0, 1280, 770], Bar::usable)
    }
}
