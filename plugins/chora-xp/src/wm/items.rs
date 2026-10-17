//! The context menus of the desktop icons and of the entries of the start
//! menu, and what their rows do: open, explore, delete to the Recycle
//! Bin, empty it, leave the start menu and show the Properties.

use std::path::{Path, PathBuf};

use chora_scene::Command;

use super::{DIALOGS, Panel, Target, Wm};
use crate::{
    confirm::ConfirmDialog,
    context::{self, Item},
    desktop_icons::{DesktopIcons, Open},
    folder::{self, Place},
    properties::{self, PropertiesDialog},
    startmenu::{Action, Menu},
    taskbar::Bar,
    theme::Rect,
};

impl Wm {
    /// The menu of the icon or the start menu entry under the pointer;
    /// `None` when the node is neither.
    pub(super) fn open_item_menu(&mut self, node: &str, point: [i32; 2]) -> Option<Vec<Command>> {
        let (target, kind, mut commands) = if Menu::owns(node) {
            let item = self.menu.item(node, &mut self.icons)?;
            let kind = match (&item.action, item.program) {
                (_, Some(_)) => context::Target::Program(true),
                (Action::Explore(Place::Folder(_)), _) => context::Target::Place(true),
                (Action::Run(_), _) => context::Target::Program(false),
                _ => context::Target::Place(false),
            };
            (Target::Start(item), kind, Vec::new())
        } else if DesktopIcons::owns(node) {
            let item = self.desk.item(node)?;
            let kind = match &item.open {
                Open::Folder(_) if item.title == "My Documents" => context::Target::Documents,
                Open::Folder(_) => context::Target::Folder,
                Open::Computer => context::Target::Computer,
                Open::Trash => {
                    context::Target::Trash(!folder::trash(&folder::trash_root()).is_empty())
                }
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
                    Item::Open | Item::Explore => {
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
                    Item::Properties => {
                        let lines = match &item.action {
                            Action::Explore(Place::Folder(path)) => properties::of_path(path),
                            Action::Run(command) => properties::of_shortcut(command),
                            Action::Explore(place) => properties::of_shortcut(&place.title()),
                            _ => Vec::new(),
                        };
                        commands.extend(self.open_properties(&item.title, item.icon, lines));
                    }
                    Item::Delete | Item::EmptyTrash => {}
                }
                commands
            }
            Some(Target::Icon(item)) => match act {
                Item::Open => self.open_icon(item.open),
                Item::Explore => match item.open {
                    Open::Folder(path) => self.open_explorer(Place::Folder(path)),
                    Open::Computer => self.open_explorer(Place::Computer),
                    Open::Trash => self.open_explorer(Place::Trash),
                    _ => Vec::new(),
                },
                Item::Delete => self.confirm_trash(PathBuf::from(&item.key)),
                Item::EmptyTrash => {
                    let names: Vec<String> = folder::trash(&folder::trash_root())
                        .into_iter()
                        .map(|t| t.entry.name)
                        .collect();
                    self.confirm_empty(&names)
                }
                Item::Properties => {
                    let lines = match &item.open {
                        Open::Trash => properties::of_trash(&folder::trash(&folder::trash_root())),
                        Open::Run(command) => properties::of_shortcut(command),
                        _ => properties::of_path(Path::new(&item.key)),
                    };
                    let icon = self.icons.find_sized(&item.icon, 32);
                    self.open_properties(&item.title, icon, lines)
                }
                Item::Remove => Vec::new(),
            },
            None => Vec::new(),
        }
    }

    pub(super) fn work_area(&self) -> Rect {
        self.bars.first().map_or([0, 0, 1280, 770], Bar::usable)
    }

    /// Properties of an icon or an entry; one at a time, as the open one
    /// goes.
    fn open_properties(
        &mut self,
        name: &str,
        icon: Option<PathBuf>,
        lines: Vec<properties::Line>,
    ) -> Vec<Command> {
        let mut commands = Vec::new();
        if let Some(i) = self
            .panels
            .iter()
            .position(|p| matches!(p, Panel::Properties(_)))
        {
            commands.push(self.panels.remove(i).window_ref().remove());
        }
        let panel = PropertiesDialog::new(name, icon, lines, self.work_area());
        let page = panel.page(&self.theme);
        let mut panel = Panel::Properties(panel);
        commands.extend(panel.window().create(&mut self.theme, DIALOGS, page));
        self.panels.push(panel);
        commands
    }

    /// Asks whether to send a file or folder to the Recycle Bin.
    fn confirm_trash(&mut self, path: PathBuf) -> Vec<Command> {
        let mut commands = Vec::new();
        if let Some(i) = self
            .panels
            .iter()
            .position(|p| matches!(p, Panel::Confirm(_)))
        {
            commands.push(self.panels.remove(i).window_ref().remove());
        }
        let icon = self.icons.find_sized("user-trash-full", 32);
        let confirm = ConfirmDialog::trash(&self.theme, path, icon, self.work_area());
        let page = confirm.page();
        let mut panel = Panel::Confirm(confirm);
        commands.extend(panel.window().create(&mut self.theme, DIALOGS, page));
        self.panels.push(panel);
        commands
    }

    /// Sends a file or folder to the Recycle Bin, and shows the icons and
    /// the Explorer windows of the bin again.
    pub(super) fn trash(&mut self, path: &Path) -> Vec<Command> {
        let date = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        if let Err(err) = folder::trash_file(&folder::trash_root(), path, &date) {
            eprintln!("chora-xp: delete {}: {err}", path.display());
        }
        self.refresh_trash()
    }
}
