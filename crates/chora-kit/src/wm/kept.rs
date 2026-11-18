//! What a restart keeps of a look, in node meta.
#![allow(
    missing_docs,
    dead_code,
    clippy::missing_docs_in_private_items,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    reason = "red: the cells come before the book"
)]

use std::collections::BTreeMap;

use serde_json::Value;

/// Meta by sub key.
pub type Meta = BTreeMap<String, String>;

/// The versioned key.
pub const KEY: &str = "kept.1";

/// What a restart keeps.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Kept {
    pub root: Meta,
    pub windows: Vec<(String, Meta)>,
}

#[must_use]
pub fn read(_nodes: &[Value], _root: &str, _legacy: fn(&str) -> bool) -> Kept {
    Kept::default()
}
#[must_use]
pub fn write(_sub: &Meta, _legacy: &[String]) -> Meta {
    Meta::new()
}
#[must_use]
pub fn reserved(_key: &str) -> bool {
    false
}
