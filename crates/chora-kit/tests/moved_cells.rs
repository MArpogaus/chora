//! Cells of the kit parts that were added since the RC and had no test of
//! their own: the turns of the cube, the give of a window out of a look, the
//! open surfaces, and what a restart keeps. Each cell asserts the value the
//! name claims.

#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use chora_kit::{
    cube::{Turn, quarters_to},
    wm::{
        Book,
        give::pass_on,
        kept::{self, Kept},
        open::{Kind, Open, Surface},
    },
};
use chora_scene::Command;
use serde_json::json;

/// The short way round the cube: the quarter turns to a face, to the right
/// at a tie.
#[test]
fn the_cube_turns_the_short_way_and_right_at_a_tie() {
    assert_eq!(quarters_to(0, 1, 4), 1);
    assert_eq!(quarters_to(0, 3, 4), -1);
    assert_eq!(quarters_to(0, 2, 4), 2, "a tie turns to the right");
    assert_eq!(quarters_to(3, 0, 4), 1, "the turn wraps round the cube");
}

/// A turn gives either quarter turns or a face, and a face must exist.
#[test]
fn a_turn_takes_either_quarters_or_a_face_and_no_other_face() {
    let by = Turn {
        by: Some(2),
        ..Turn::default()
    };
    assert_eq!(by.quarters(0, 4), Ok(2));
    let to = Turn {
        to: Some(1),
        ..Turn::default()
    };
    assert_eq!(to.quarters(0, 4), Ok(1));
    let past = Turn {
        to: Some(9),
        ..Turn::default()
    };
    assert!(past.quarters(0, 4).is_err(), "face 9 is not on a cube of 4");
    assert!(Turn::default().quarters(0, 4).is_err(), "neither is given");
    let both = Turn {
        by: Some(1),
        to: Some(1),
        ..Turn::default()
    };
    assert!(both.quarters(0, 4).is_err(), "both are given");
}

/// A window the look proposes to give to another window manager stays
/// with the look until the give is confirmed; a refused give keeps it.
#[test]
fn a_give_out_of_the_look_keeps_the_window_until_it_is_confirmed() {
    let mut book = Book::new("mc", false);
    book.keep("s1");
    assert!(!book.give("zz", "xp"), "a window the look does not hold");
    assert!(!book.give("s1", "mc"), "the look does not give to itself");
    assert!(book.give("s1", "xp"));
    book.refused("s1");
    assert!(book.has("s1"), "a refused give keeps the window");
    assert!(book.give("s1", "xp"));
    book.confirm("s1");
    assert!(!book.has("s1"), "a confirmed give forgets the window");
}

/// A surface that the core reserved for a look without outputs is attached
/// to its look and removed at once, without a target.
#[test]
fn a_passed_on_surface_is_attached_to_its_look_and_let_go() {
    let commands = pass_on("s1", "look");
    assert_eq!(commands.len(), 2);
    match &commands[0] {
        Command::Attach { surface, parent } => {
            assert_eq!((surface.as_str(), parent.as_str()), ("s1", "look"));
        }
        other => panic!("the first command attaches: {other:?}"),
    }
    match &commands[1] {
        Command::Remove { id, to } => {
            assert_eq!(id, "s1");
            assert!(to.is_none(), "it goes to no window manager in particular");
        }
        other => panic!("the second command removes: {other:?}"),
    }
}

/// An open surface replaces one with the same id and then counts as the
/// newest; a shut surface is no longer open.
#[test]
fn an_open_surface_replaces_its_twin_and_a_shut_one_is_gone() {
    let mut open = Open::default();
    assert!(open.is_empty());
    assert!(open.open(Surface::new("menu", Kind::Menu)));
    assert!(open.open(Surface::new("list", Kind::Popup)));
    assert_eq!(open.ids(), ["menu", "list"]);
    assert_eq!(open.newest().map(|s| s.id.as_str()), Some("list"));
    assert!(
        !open.open(Surface::new("menu", Kind::Menu)),
        "a twin is replaced"
    );
    assert_eq!(open.ids(), ["list", "menu"]);
    assert_eq!(open.newest().map(|s| s.id.as_str()), Some("menu"));
    assert_eq!(open.len(), 2);
    shut_and_check(&mut open);
}

/// Shuts the newest surface, which is then no longer open.
fn shut_and_check(open: &mut Open) {
    assert!(open.shut("menu").is_some());
    assert!(!open.contains("menu"), "a shut surface is no longer open");
    assert_eq!(open.len(), 1);
}

/// A restart keeps the meta of a look and of its windows under the versioned
/// key; a node without the key keeps the legacy keys that the look names.
#[test]
fn a_restart_keeps_the_versioned_meta_or_the_legacy_keys() {
    let nodes = vec![
        json!({"id": "root", "kind": "group",
               "meta": {"kept.1.layout": "grid", "kept.1.other": "1"}}),
        json!({"id": "s1", "kind": "surface",
               "meta": {"kept.1.size": "3", "legacy": "2", "other": "3"}}),
        json!({"id": "s2", "kind": "surface",
               "meta": {"legacy": "5", "other": "6"}}),
    ];
    let legacy = |key: &str| key == "legacy";
    let kept: Kept = kept::read(&nodes, "root", legacy);
    assert_eq!(kept.root.get("layout").map(String::as_str), Some("grid"));
    assert_eq!(kept.root.get("other").map(String::as_str), Some("1"));
    let s1 = &kept.windows[0];
    assert_eq!(s1.0, "s1");
    assert_eq!(s1.1.get("size").map(String::as_str), Some("3"));
    assert!(
        !s1.1.contains_key("legacy"),
        "a versioned node reads its versioned keys only"
    );
    let s2 = &kept.windows[1];
    assert_eq!(s2.0, "s2");
    assert_eq!(
        s2.1.get("legacy").map(String::as_str),
        Some("5"),
        "a legacy key is read as it is"
    );
    assert!(
        !s2.1.contains_key("other"),
        "a key the look does not name is not read"
    );
}
