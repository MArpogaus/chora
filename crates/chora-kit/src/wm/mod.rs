//! The shared window-management core (arch plan 1.1, 2.2), a model
//! without IO: events in, changes out; the look turns the changes into
//! scene commands and settles them with [`book::Book::confirm`] or
//! [`book::Book::refused`].
//!
//! The parts: [`book`] (which windows are the look's), [`give`] (a window
//! that goes to another window manager), [`kept`] (what a restart keeps in
//! node meta), [`state`] (the shape of `state.get`), [`focus`] (who holds the
//! keyboard, and the [`focus::Policy`] of the look for the focus after a
//! window goes), [`open`] (the open menus, popups, dialogs, sheets and
//! launchers) and [`stack`] (the two stacking rules that every look shares).
//! Desks follow in a later slice.

pub mod book;
pub mod focus;
pub mod give;
pub mod kept;
pub mod open;
pub mod stack;
pub mod state;

pub use book::{Book, Change, Created, Origin, Phase, Why};
