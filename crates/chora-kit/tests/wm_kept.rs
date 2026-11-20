//! What a restart keeps of a look (`kit::wm::kept`): the versioned meta
//! that the look writes to its nodes, read back through the fake core,
//! and the keys of before that a look still reads.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use std::collections::BTreeMap;

use chora_kit::{
    testing::{FakeCore, surface},
    wm::kept::{KEY, Meta, read, reserved, write},
};
use chora_scene::{Command, NodeKind, PropsUpdate, ROOT};
use serde_json::{Value, json};

const NAME: &str = "look";

fn no_legacy(_: &str) -> bool {
    false
}

fn meta(pairs: &[(&str, &str)]) -> Meta {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect::<BTreeMap<_, _>>()
}

/// The nodes that the core answers for the look and its surfaces.
fn nodes(conn: &mut chora_kit::conn::Conn, selectors: &[&str]) -> Vec<Value> {
    selectors
        .iter()
        .flat_map(|s| {
            conn.call::<_, Vec<Value>>("scene.query", json!({"selector": s}))
                .unwrap()
        })
        .collect()
}

/// What the look writes to its root group is what a restart reads back,
/// through the core: the meta of the group under the versioned key.
#[test]
fn what_the_look_writes_is_read_back_from_the_core() {
    let core = FakeCore::start();
    let mut conn = core.connect(NAME);
    let sub = meta(&[("current", "2"), ("tags:DP-1", "1")]);
    conn.apply(vec![Command::Create {
        id: NAME.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate {
            meta: Some(write(&sub, &[])),
            ..PropsUpdate::default()
        },
    }])
    .unwrap();
    let stored = nodes(&mut conn, &[&format!("id:{NAME}")]);
    let kept = read(&stored, NAME, no_legacy);
    assert_eq!(kept.root, sub);
    assert_eq!(
        kept.windows,
        [] as [(
            std::string::String,
            std::collections::BTreeMap<std::string::String, std::string::String>
        ); 0]
    );
}

/// A kept window reads back with its own meta, and a node that is not a
/// surface is not a window of the look.
#[test]
fn a_kept_window_reads_back_and_a_group_is_no_window() {
    let core = FakeCore::start();
    let mut conn = core.connect(NAME);
    surface(&core.model(), "s1");
    conn.apply(vec![Command::Create {
        id: "g1".into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate {
            meta: Some(write(&meta(&[("desk", "3")]), &[])),
            ..PropsUpdate::default()
        },
    }])
    .unwrap();
    let all = nodes(&mut conn, &["id:s1", "id:g1"]);
    let kept = read(&all, NAME, no_legacy);
    assert_eq!(
        kept.windows,
        [("s1".to_owned(), Meta::new())],
        "the surface is a window, the group is not"
    );
    assert_eq!(kept.root, Meta::new(), "the root is not in the nodes");
}

/// Keys of before are read as they are when the node has no versioned
/// key; the keys that the legacy filter does not name are left out.
#[test]
fn keys_of_before_are_read_when_nothing_is_versioned() {
    let node = json!({"id": NAME, "kind": "group",
        "meta": {"window": "s1", "current": "1", "unrelated": "x"}});
    let kept = read(&[node], NAME, |k| k == "window" || k == "current");
    assert_eq!(kept.root, meta(&[("window", "s1"), ("current", "1")]));
}

/// A versioned key wins: when a node has both, the keys of before are
/// not read at all.
#[test]
fn a_versioned_key_wins_over_a_key_of_before() {
    let node = json!({"id": NAME, "kind": "group",
        "meta": {"kept.1.current": "2", "window": "s1"}});
    let kept = read(&[node], NAME, |k| k == "window");
    assert_eq!(kept.root, meta(&[("current", "2")]));
}

/// A write retires the keys of before by emptying them, and writes the
/// versioned keys beside them.
#[test]
fn a_write_retires_the_keys_of_before() {
    let written = write(&meta(&[("current", "2")]), &["window".to_owned()]);
    assert_eq!(written.get("window").map(String::as_str), Some(""));
    assert_eq!(
        written.get(&format!("{KEY}.current")).map(String::as_str),
        Some("2")
    );
    assert_eq!(written.len(), 2);
}

/// The core keeps no empty meta value: once the look writes an empty
/// key of before, the node has no such key left.
#[test]
fn a_retired_key_is_gone_from_the_core() {
    let core = FakeCore::start();
    let mut conn = core.connect(NAME);
    let props = |meta| PropsUpdate {
        meta: Some(meta),
        ..PropsUpdate::default()
    };
    conn.apply(vec![Command::Create {
        id: NAME.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: props(meta(&[("window", "s1")])),
    }])
    .unwrap();
    conn.apply(vec![Command::Update {
        id: NAME.into(),
        props: props(write(&Meta::new(), &["window".to_owned()])),
    }])
    .unwrap();
    let stored = nodes(&mut conn, &[&format!("id:{NAME}")]);
    let meta = stored[0]["meta"].as_object().unwrap();
    assert!(!meta.contains_key("window"), "{meta:?}");
}

/// Only the keys of the look are reserved: the key of the version and
/// its sub keys, not a key that only looks alike.
#[test]
fn only_the_versioned_keys_are_reserved() {
    assert!(reserved(KEY));
    assert!(reserved(&format!("{KEY}.current")));
    assert!(!reserved("kept.10"), "a longer version is no sub key");
    assert!(!reserved("window"));
}
