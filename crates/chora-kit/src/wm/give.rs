//! A window that goes to another window manager (D55): the give out as a
//! proposal of the book, the pass-on of a surface that the core reserved
//! for a look without outputs, and whether the receiver shows where the
//! window was ([`claims_over`], D119). A window given in is
//! [`super::Origin::Given`].

use chora_scene::{Command, Node};

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

/// Whether a claim of the window manager `to` overlaps `area`, a box in
/// scene coordinates, such as the output of a window that goes to `to`
/// (D119): a receiver on another output gets the window without the
/// keyboard, which stays with the look where the person is. A box that
/// only touches the edge does not overlap.
#[must_use]
pub fn claims_over<'a>(
    nodes: impl IntoIterator<Item = &'a Node>,
    to: &str,
    area: [f64; 4],
) -> bool {
    let [ax, ay, aw, ah] = area;
    nodes
        .into_iter()
        .filter(|n| n.owner.as_deref() == Some(to))
        .filter_map(|n| n.props.claim)
        .any(|[x, y, w, h]| x < ax + aw && ax < x + w && y < ay + ah && ay < y + h)
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
