//! The shared window-management core (arch plan 1.1, 2.2), a model
//! without IO: events in, changes out; the look turns the changes into
//! scene commands and settles them with [`book::Book::confirm`] or
//! [`book::Book::refused`].
//!
//! This first slice (WM1) has the lifecycle: [`book`] (which windows are
//! the look's), [`give`] (a window that goes to another window manager),
//! [`kept`] (what a restart keeps in node meta) and [`state`] (the shape of
//! `state.get`). WM2 adds [`focus`] (who holds the keyboard, and the
//! [`focus::Policy`] of the look for the focus after a window goes). Open
//! shell surfaces, desks and the stacking order follow in later slices.

pub mod book;
pub mod focus;
pub mod give;
pub mod kept;
pub mod state;

pub use book::{Book, Change, Origin, Phase, Why};
