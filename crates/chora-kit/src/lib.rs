//! Building blocks for chora plugins that are not part of the API of the
//! core: effects that a window manager computes itself, such as wobbly
//! windows and the magic lamp, and the freedesktop files that a shell
//! reads, such as icon themes and desktop entries.

pub mod conn;

pub mod desktop_entry;
pub mod icons;
pub mod lamp;
pub mod wobbly;
pub mod xdg;
