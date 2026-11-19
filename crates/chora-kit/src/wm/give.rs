//! A window that goes to another window manager (D55): the give out as a
//! proposal of the book, and the pass-on of a surface that the core
//! reserved for a look without outputs. A window given in is
//! [`super::Origin::Given`].

use chora_scene::Command;

use super::book::{Book, Pending};

impl Book {
    /// Proposes to give an own window to the window manager `to`; false
    /// when the window is not the look's. The look sends the give, then
    /// answers [`Book::confirm`], which forgets the window, or
    /// [`Book::refused`], which keeps it.
    pub fn give(&mut self, id: &str, to: &str) -> bool {
        if !self.has(id) || to == self.name {
            return false;
        }
        self.pending.insert(id.into(), Pending::Give(to.into()));
        true
    }
}

/// Passes a surface that the core reserved for the look `parent` on to
/// any window manager: the look takes it and lets it go at once, without
/// `to`.
#[must_use]
pub fn pass_on(surface: &str, parent: &str) -> Vec<Command> {
    vec![
        Command::Attach {
            surface: surface.into(),
            parent: parent.into(),
        },
        Command::Remove {
            id: surface.into(),
            to: None,
        },
    ]
}
