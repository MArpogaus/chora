//! A window that goes to another window manager (D55).
#![allow(
    missing_docs,
    dead_code,
    clippy::missing_docs_in_private_items,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    reason = "red: the cells come before the book"
)]

use chora_scene::Command;

use super::book::Book;

impl Book {
    /// Proposes to give an own window to `to`.
    pub fn give(&mut self, _id: &str, _to: &str) -> bool {
        false
    }
}

/// Passes a surface on to any window manager.
#[must_use]
pub fn pass_on(_surface: &str, _parent: &str) -> Vec<Command> {
    Vec::new()
}
