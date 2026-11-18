//! What a restart keeps of a look (arch plan 1.1): its data in node meta
//! under one versioned key, [`KEY`], with sub keys that the look names,
//! and a reader for the keys that looks wrote before (tiling: `window`,
//! `current`, `tags:<output>`). A write retires those keys, so that the
//! meta of a node stays within its limits.

use std::collections::BTreeMap;

use serde_json::Value;

/// Meta by key, or by sub key of [`KEY`].
pub type Meta = BTreeMap<String, String>;

/// The versioned key: a sub key `s` is the meta key `kept.1.s`.
pub const KEY: &str = "kept.1";

/// What a restart keeps of a look, by sub key: the meta of its root group
/// and of each kept window, in the order of the nodes.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Kept {
    /// The meta of the root group of the look.
    pub root: Meta,
    /// The meta of each surface that the look kept, by surface id.
    pub windows: Vec<(String, Meta)>,
}

/// Reads what a restart keeps from the nodes that `scene.query` answers:
/// the group `root` and the surfaces. A node with a key of [`KEY`] reads
/// those; a node without reads the keys that `legacy` names as they are.
#[must_use]
pub fn read(nodes: &[Value], root: &str, legacy: fn(&str) -> bool) -> Kept {
    let mut kept = Kept::default();
    for node in nodes {
        let Some(id) = node["id"].as_str() else {
            continue;
        };
        let meta = serde_json::from_value::<Meta>(node["meta"].clone()).unwrap_or_default();
        if id == root {
            kept.root = sub_keys(meta, legacy);
        } else if node["kind"] == "surface" {
            kept.windows.push((id.into(), sub_keys(meta, legacy)));
        }
    }
    kept
}

/// The meta of a node by sub key.
fn sub_keys(meta: Meta, legacy: fn(&str) -> bool) -> Meta {
    let versioned: Meta = (meta.iter())
        .filter_map(|(key, value)| Some((sub_key(key)?.to_owned(), value.clone())))
        .collect();
    if !versioned.is_empty() {
        return versioned;
    }
    meta.into_iter().filter(|(key, _)| legacy(key)).collect()
}

/// The sub key of a key of [`KEY`].
fn sub_key(key: &str) -> Option<&str> {
    key.strip_prefix(KEY)?.strip_prefix('.')
}

/// The meta to write for data by sub key: each under [`KEY`], and the keys
/// of before in `retired` emptied, which removes them.
#[must_use]
pub fn write(sub: &Meta, retired: &[String]) -> Meta {
    let keys = sub.iter().map(|(k, v)| (format!("{KEY}.{k}"), v.clone()));
    let gone = retired.iter().map(|k| (k.clone(), String::new()));
    gone.chain(keys).collect()
}

/// Whether a meta key is one of [`KEY`], which only the look writes.
#[must_use]
pub fn reserved(key: &str) -> bool {
    key == KEY || sub_key(key).is_some()
}
