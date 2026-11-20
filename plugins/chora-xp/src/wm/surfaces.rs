//! The open shell surfaces of xp on `kit::wm::open` (WM3): the Start menu,
//! a context menu, the Turn Off dialog, the settings dialogs and the
//! switcher are registered where they open and shut where they close, so
//! that `state.get`, Escape and the drag or the give of a window read one
//! list.

use chora_kit::wm::open::{Kind, Surface};

use super::{Command, Panel, Wm};

/// What one Escape shuts: the newest of the open surfaces.
pub(super) enum Escape {
    /// The Turn Off or Log Off dialog.
    Dialog,
    /// The Help panel at this place of the panel list.
    Help(usize),
    /// A context menu or the Start menu, which close their cascades first.
    Menu,
}

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
        self.clear_starting();
        self.open.open(Surface::new(MENU, Kind::Menu));
    }

    /// A context menu or the system menu of a window opened.
    pub(super) fn context_opened(&mut self, window: Option<&str>) {
        self.clear_starting();
        let menu = Surface::new(CONTEXT, Kind::Menu);
        self.open.open(match window {
            Some(surface) => menu.by(surface),
            None => menu,
        });
    }

    /// The Turn Off dialog opened: it holds the keyboard and the pointer.
    pub(super) fn session_opened(&mut self) {
        self.clear_starting();
        self.open.open(Surface::new(SESSION, Kind::Dialog).modal());
    }

    /// The switcher opened. It follows the keys and stays through a drag.
    pub(super) fn switcher_opened(&mut self) {
        self.clear_starting();
        self.open
            .open(Surface::new(SWITCHER, Kind::Switcher).staying());
    }

    /// Puts a dialog of the settings on top of the other dialogs. It stays
    /// through a drag and a give, as its title bar drags it.
    pub(super) fn push_panel(&mut self, panel: Panel) {
        let id = panel.window_ref().group();
        self.clear_starting();
        self.open.open(Surface::new(id, Kind::Dialog).staying());
        self.panels.push(panel);
    }

    /// A menu, a popup or a dialog opens: the messages that a program
    /// starts go, so that no message covers it (I5). The start goes on.
    fn clear_starting(&mut self) {
        for note in self.launches.take_notes() {
            self.dismiss_notice(note);
        }
    }

    /// Takes a dialog of the settings off the list.
    pub(super) fn take_panel(&mut self, i: usize) -> Panel {
        let panel = self.panels.remove(i);
        self.open.shut(&panel.window_ref().group());
        panel
    }

    /// The surface that an Escape shuts, by the order of `state.open`: the
    /// newest first. Other kinds of surface do not answer Escape here.
    pub(super) fn escape_target(&self) -> Option<Escape> {
        self.open.ids().into_iter().rev().find_map(|id| match id {
            SESSION if self.dialog.is_some() => Some(Escape::Dialog),
            CONTEXT | MENU => Some(Escape::Menu),
            group => {
                let help =
                    |p: &&Panel| matches!(p, Panel::Help(_)) && p.window_ref().group() == group;
                self.panels.iter().position(|p| help(&p)).map(Escape::Help)
            }
        })
    }

    /// A drag of a window began, or a window went to another window
    /// manager: the menus shut, the surfaces that declared to stay stay.
    pub(super) fn shut_surfaces(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        for surface in self.open.shut_at(&[]) {
            commands.extend(match surface.id.as_str() {
                MENU => self.close_menu(),
                CONTEXT => self.close_context(),
                _ => Vec::new(),
            });
        }
        commands
    }
}
