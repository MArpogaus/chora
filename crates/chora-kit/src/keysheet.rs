//! The rows of a key list (KIT3): what each of the four looks draws, made
//! once from the bindings. Words come from [`System`] (one wording per
//! action and language), a chord is spelled one way, and the actions of
//! one look are named by the look. A look draws its own costume around
//! the rows: its groups, its caps, its order.

use crate::actions::{Action, System};
use crate::words::Language;

/// A row of a key list: what the bindings of one wording do.
#[derive(Clone, Debug, PartialEq)]
pub struct Row<L> {
    /// The action of the first binding of the row.
    pub action: Action<L>,
    /// What it does, in words.
    pub words: String,
    /// The chords that do it, as people read them, in the order of the
    /// bindings.
    pub chords: Vec<String>,
    /// The first binding, as the file spells it: the one that a click on
    /// the row acts like.
    pub binding: String,
}

/// The caps of a binding, one per key.
#[must_use]
pub fn caps(_binding: &str) -> Vec<String> {
    Vec::new()
}

/// A binding as people read it: its caps joined by a plus.
#[must_use]
pub fn chord(_binding: &str) -> String {
    String::new()
}

/// The words of an action of the system, none for an action that no
/// key list names.
#[must_use]
pub fn words(_system: &System, _language: Language) -> Option<String> {
    None
}

/// The rows of `bindings`, one per wording: the actions of the system
/// first, in the order of [`System`], then the actions of the look in the
/// order of the bindings, each named by `look` (none: no row).
pub fn rows<'a, L: Clone + 'a>(
    _bindings: impl IntoIterator<Item = (String, &'a Action<L>)>,
    _language: Language,
    _look: impl Fn(&str, &L) -> Option<String>,
) -> Vec<Row<L>> {
    Vec::new()
}

/// The keys that give a window to a look, as one row: the span of the
/// keys and the words.
#[must_use]
pub fn give_row(_language: Language) -> (String, String) {
    (String::new(), String::new())
}
