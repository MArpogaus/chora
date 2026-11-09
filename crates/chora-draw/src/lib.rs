//! chora-draw: the drawing that the blocks of the XP split share (D73,
//! `docs/xp-split.md`). It is a library, not a block: it has no client,
//! settings or asset folder. A block names the folder and the sheets, and
//! the library rasterizes its pictures and text into image nodes.

#![allow(
    clippy::print_stderr,
    reason = "a picture that fails to draw logs to stderr, which the session keeps"
)]

pub mod assets;
pub mod icons;
pub mod text;
pub mod theme;
