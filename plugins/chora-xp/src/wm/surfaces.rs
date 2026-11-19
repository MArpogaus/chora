//! The open shell surfaces of xp on `kit::wm::open` (WM3): the Start menu,
//! a context menu, the Turn Off dialog, the settings dialogs and the
//! switcher are registered where they open and shut where they close, so
//! that `state.get`, Escape and the drag or the give of a window read one
//! list.

use chora_kit::wm::open::{Kind, Surface, Why};

use super::{Command, Panel, Wm};

/// The node of the Start menu.
pub(super) const MENU: &str = "xp-menu";
/// The node of a context menu and of the system menu of a window.
pub(super) const CONTEXT: &str = "xp-ctx";
/// The node of the Log Off and Turn Off dialog.
pub(super) const SESSION: &str = "xp-session";
/// The node of the Alt+Tab switcher.
pub(super) const SWITCHER: &str = "xp-switcher";

impl Wm {
    /// The Start menu opened.
    pub(super) fn menu_opened(&mut self) {
        self.open.open(Surface::new(MENU, Kind::Menu));
    }

    /// A context menu or the system menu of a window opened.
    pub(super) fn context_opened(&mut self, window: Option<&str>) {
        let menu = Surface::new(CONTEXT, Kind::Menu);
        self.open.open(match window {
            Some(surface) => menu.by(surface),
            None => menu,
        });
    }

    /// The Turn Off dialog opened: it holds the keyboard and the pointer.
    pub(super) fn session_opened(&mut self) {
        self.open.open(Surface::new(SESSION, Kind::Dialog).modal());
    }

    /// The switcher opened. It follows the keys and stays through a drag.
    pub(super) fn switcher_opened(&mut self) {
        self.open
            .open(Surface::new(SWITCHER, Kind::Switcher).staying());
    }

    /// Puts a dialog of the settings on top of the other dialogs. It stays
    /// through a drag and a give, as its title bar drags it.
    pub(super) fn push_panel(&mut self, panel: Panel) {
        let id = panel.window_ref().group();
        self.open.open(Surface::new(id, Kind::Dialog).staying());
        self.panels.push(panel);
    }

    /// Takes a dialog of the settings off the list.
    pub(super) fn take_panel(&mut self, i: usize) -> Panel {
        let panel = self.panels.remove(i);
        self.open.shut(&panel.window_ref().group());
        panel
    }

    /// A drag of a window began, or a window went to another window
    /// manager: the menus shut, the surfaces that declared to stay stay.
    pub(super) fn shut_surfaces(&mut self, why: Why) -> Vec<Command> {
        let mut commands = Vec::new();
        for surface in self.open.shut_at(why, &[]) {
            commands.extend(match surface.id.as_str() {
                MENU => self.close_menu(),
                CONTEXT => self.close_context(),
                _ => Vec::new(),
            });
        }
        commands
    }
}
