//! The open shell surfaces of a look (arch plan 2.1, row `open`): the menus,
//! popups, dialogs, sheets and launchers that it shows above its windows.
//! A model without IO. The look draws and removes the surfaces; this list
//! knows which are open, who opened each, which is modal, in what order
//! Escape shuts them, and which of them a drag or a give shuts.
//!
//! The list feeds the `open` of [`super::state::State`]: there is no second
//! path to the state.
//!
//! ```
//! use chora_kit::wm::open::{Kind, Open, Surface};
//!
//! let mut open = Open::default();
//! open.open(Surface::new("menu", Kind::Menu));
//! open.open(Surface::new("keys", Kind::Sheet).modal().staying());
//! // A drag shuts the menu; the sheet declared to stay.
//! let shut = open.shut_at(&[]);
//! assert_eq!(shut.len(), 1);
//! assert_eq!(open.ids(), ["keys"]);
//! // Escape shuts the newest.
//! assert_eq!(open.escape().map(|s| s.id), Some("keys".to_owned()));
//! assert!(open.is_empty());
//! ```

use super::state;

/// What an open surface is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A menu: the Start menu, a context menu, a system menu.
    Menu,
    /// A popup that does not take the keyboard.
    Popup,
    /// A dialog.
    Dialog,
    /// A sheet that a look shows over its page: the key list.
    Sheet,
    /// The launcher: a list of programs with a line to type in.
    Launcher,
    /// The switcher of windows that Alt+Tab shows.
    Switcher,
}

impl Kind {
    /// The word of the kind in `state.get`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Menu => "menu",
            Self::Popup => "popup",
            Self::Dialog => "dialog",
            Self::Sheet => "sheet",
            Self::Launcher => "launcher",
            Self::Switcher => "switcher",
        }
    }
}

/// An open surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surface {
    /// The id of its node (or of its group).
    pub id: String,
    /// What it is.
    pub kind: Kind,
    /// Whether it holds the keyboard and the pointer.
    pub modal: bool,
    /// What opened it: the surface of a window (the system menu of it), or
    /// nothing when the person opened it from the shell.
    pub by: Option<String>,
    /// Whether a drag or a give leaves it open.
    pub stays: bool,
}

impl Surface {
    /// A surface that the person opened from the shell, shut by a drag.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: Kind) -> Self {
        Self {
            id: id.into(),
            kind,
            modal: false,
            by: None,
            stays: false,
        }
    }

    /// The surface holds the keyboard and the pointer.
    #[must_use]
    pub const fn modal(mut self) -> Self {
        self.modal = true;
        self
    }

    /// The surface belongs to a window.
    #[must_use]
    pub fn by(mut self, surface: impl Into<String>) -> Self {
        self.by = Some(surface.into());
        self
    }

    /// The entry of the surface in `state.get`.
    #[must_use]
    pub fn entry(&self) -> state::Open {
        state::Open {
            id: self.id.clone(),
            kind: self.kind.word().to_owned(),
            modal: self.modal,
            by: self.by.clone(),
        }
    }

    /// A drag or a give leaves the surface open.
    #[must_use]
    pub const fn staying(mut self) -> Self {
        self.stays = true;
        self
    }
}

/// The open surfaces of a look, the oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Open {
    list: Vec<Surface>,
}

impl Open {
    /// Opens a surface above the others. An open surface with the same id
    /// is replaced and counts as the newest. True when it was not open.
    pub fn open(&mut self, surface: Surface) -> bool {
        let was = self.shut(&surface.id).is_some();
        self.list.push(surface);
        !was
    }

    /// Shuts a surface by id; none when it was not open.
    pub fn shut(&mut self, id: &str) -> Option<Surface> {
        let at = self.list.iter().position(|s| s.id == id)?;
        Some(self.list.remove(at))
    }

    /// Whether the surface is open.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.list.iter().any(|s| s.id == id)
    }

    /// Whether nothing is open.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// How many surfaces are open.
    #[must_use]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// The ids, the oldest first.
    #[must_use]
    pub fn ids(&self) -> Vec<&str> {
        self.list.iter().map(|s| s.id.as_str()).collect()
    }

    /// The surface that Escape shuts next: the newest.
    #[must_use]
    pub fn newest(&self) -> Option<&Surface> {
        self.list.last()
    }

    /// Shuts the newest surface: one Escape, one surface.
    pub fn escape(&mut self) -> Option<Surface> {
        self.list.pop()
    }

    /// The newest modal surface: while it is open, the keyboard and the
    /// pointer belong to it.
    #[must_use]
    pub fn modal(&self) -> Option<&Surface> {
        self.list.iter().rev().find(|s| s.modal)
    }

    /// A drag or a give shuts every open surface that is not modal, not
    /// declared to stay and not named in `keep`. Returns what shut, the
    /// newest first, so that the look removes the nodes in Escape order.
    pub fn shut_at(&mut self, keep: &[&str]) -> Vec<Surface> {
        let mut shut = Vec::new();
        let mut stay = Vec::new();
        for surface in self.list.drain(..).rev() {
            if surface.modal || surface.stays || keep.contains(&surface.id.as_str()) {
                stay.push(surface);
            } else {
                shut.push(surface);
            }
        }
        stay.reverse();
        self.list = stay;
        shut
    }

    /// The list as `state.get` tells it: Escape order, the newest first.
    #[must_use]
    pub fn state(&self) -> Vec<state::Open> {
        self.list.iter().rev().map(Surface::entry).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu(id: &str) -> Surface {
        Surface::new(id, Kind::Menu)
    }

    /// The entry of `state.get`: the id, the kind and whether it is modal;
    /// `by` only when a window opened it.
    #[test]
    fn an_entry_of_the_state_is_its_id_its_kind_and_its_modal() {
        let sheet = serde_json::to_value(Surface::new("keys", Kind::Sheet).modal().entry());
        assert_eq!(
            sheet.unwrap(),
            serde_json::json!({"id": "keys", "kind": "sheet", "modal": true})
        );
        let by = serde_json::to_value(menu("ctx").by("s1").entry()).unwrap();
        assert_eq!(
            by,
            serde_json::json!({"id": "ctx", "kind": "menu", "modal": false, "by": "s1"})
        );
    }

    #[test]
    fn surfaces_open_in_order_and_reopen_as_the_newest() {
        let mut open = Open::default();
        assert!(open.open(menu("a")));
        assert!(open.open(menu("b")));
        assert!(!open.open(menu("a")));
        assert_eq!(open.ids(), ["b", "a"]);
        assert_eq!(open.len(), 2);
        assert!(open.contains("b"));
        assert!(!open.contains("c"));
    }

    #[test]
    fn escape_shuts_the_newest_first_and_a_shut_by_name_leaves_the_rest() {
        let mut open = Open::default();
        for id in ["a", "b", "c"] {
            open.open(menu(id));
        }
        assert_eq!(open.newest().map(|s| s.id.as_str()), Some("c"));
        assert_eq!(open.shut("b").map(|s| s.id), Some("b".into()));
        assert!(open.shut("b").is_none());
        assert_eq!(open.escape().map(|s| s.id), Some("c".into()));
        assert_eq!(open.escape().map(|s| s.id), Some("a".into()));
        assert!(open.escape().is_none());
        assert!(open.is_empty());
    }

    #[test]
    fn a_drag_or_a_give_shuts_every_surface_not_declared_to_stay() {
        let mut open = Open::default();
        open.open(menu("start"));
        open.open(Surface::new("pop", Kind::Popup));
        open.open(Surface::new("panel", Kind::Dialog).staying());
        open.open(Surface::new("power", Kind::Dialog).modal());
        open.open(menu("own"));
        let shut: Vec<String> = open.shut_at(&["own"]).into_iter().map(|s| s.id).collect();
        assert_eq!(shut, ["pop", "start"], "newest first");
        assert_eq!(open.ids(), ["panel", "power", "own"]);
    }

    #[test]
    fn the_newest_modal_surface_is_the_modal() {
        let mut open = Open::default();
        assert!(open.modal().is_none());
        open.open(Surface::new("keys", Kind::Sheet).modal());
        open.open(menu("on top"));
        open.open(Surface::new("ask", Kind::Dialog).modal());
        assert_eq!(open.modal().map(|s| s.id.as_str()), Some("ask"));
        open.shut("ask");
        assert_eq!(open.modal().map(|s| s.id.as_str()), Some("keys"));
    }

    #[test]
    fn the_state_tells_kind_modal_and_opener_in_escape_order() {
        let mut open = Open::default();
        open.open(Surface::new("keys", Kind::Sheet).modal());
        open.open(Surface::new("ctx", Kind::Menu).by("s1"));
        let state = open.state();
        assert_eq!(state.len(), 2);
        assert_eq!(state[0].id, "ctx");
        assert_eq!(state[0].kind, "menu");
        assert_eq!(state[0].by.as_deref(), Some("s1"));
        assert_eq!(state[1].kind, "sheet");
        assert!(state[1].modal);
        let words = [Kind::Popup, Kind::Launcher, Kind::Switcher, Kind::Dialog].map(Kind::word);
        assert_eq!(words, ["popup", "launcher", "switcher", "dialog"]);
    }
}
