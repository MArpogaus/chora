//! Tests of the hits and close marks of a region on the fake core: the two
//! bars of a close mark, the focus mark under the pointer, and the clicks
//! that a hit answers.

use chora_kit::testing::{FakeCore, Model};
use chora_scene::{InputMode, NodeKind};

use super::super::{Command, Item, Put};
use super::{CLOSE, CLOSE_GAP, close_id, close_nodes, close_place, lead, left_of, mark_id};
use crate::manage::OVERVIEW;
use crate::testing;

/// An item of a side with the close mark `mark` and the colour `color`.
fn item(mark: bool, put: Put, x: f64) -> Item {
    Item {
        id: "line".into(),
        slot: 0,
        props: chora_scene::PropsUpdate {
            color: Some([0.5, 0.5, 0.5, 1.0]),
            ..chora_scene::PropsUpdate::default()
        },
        first: false,
        x,
        put,
        before: 0.0,
        hit: None,
        mark,
        shed: false,
    }
}

/// The opacity of a node of the model.
fn opacity(model: &Model, id: &str) -> Option<f64> {
    Some(model.lock().unwrap().scene.node(id)?.props.opacity)
}

/// The two bars of a close mark, created in the colour of the text and
/// passed by the pointer.
#[test]
fn a_close_mark_is_two_passing_bars_in_the_colour_of_the_text() {
    let item = item(true, Put::Stack, 0.0);
    let bars = close_nodes(&item, "parent", [1.0; 4]);
    assert_eq!(bars.len(), 2, "two bars");
    for (n, bar) in bars.iter().enumerate() {
        assert_bar(bar, n);
    }
}

/// Creates a bar of the close mark `n` of the item of the tests.
fn assert_bar(bar: &Command, n: usize) {
    let Command::Create {
        id,
        kind,
        parent,
        props,
    } = bar
    else {
        panic!("a bar is created: {bar:?}");
    };
    assert_eq!(id, &close_id("line", n), "the id names its bar");
    assert!(matches!(kind, NodeKind::Rect), "a bar is a rect");
    assert_eq!(parent, "parent", "the bar is in the parent");
    assert_eq!(
        props.input,
        Some(InputMode::Pass),
        "the bar passes the pointer"
    );
    assert_eq!(
        props.color,
        Some([0.5, 0.5, 0.5, 1.0]),
        "the colour of the text"
    );
}

/// The cross of a close mark turns its two bars at 45 degrees, and the
/// room of the mark is the size of the cross and its gap.
#[test]
fn the_cross_turns_both_bars_and_the_mark_leaves_room_for_them() {
    let item = item(true, Put::Stack, 0.0);
    let placed = close_place(&item, [100.0, 50.0]);
    assert_eq!(placed.len(), 2, "both bars are placed");
    for (n, command) in placed.iter().enumerate() {
        let Command::Update { id, props } = command else {
            panic!("a bar is moved: {command:?}");
        };
        assert_eq!(id, &close_id("line", n), "the move names its bar");
        assert!(props.transform.is_some(), "the bar gets its cross");
    }
    assert_eq!(
        lead(&item),
        CLOSE + CLOSE_GAP,
        "the mark leaves room for its bars"
    );
}

/// An item without a close mark has no bars and no room for one.
#[test]
fn an_item_without_a_close_mark_has_no_bars_and_no_room() {
    let item = item(false, Put::Stack, 0.0);
    assert!(close_nodes(&item, "parent", [1.0; 4]).is_empty(), "no bars");
    assert!(
        close_place(&item, [0.0, 0.0]).is_empty(),
        "nothing to place"
    );
    assert_eq!(lead(&item), 0.0, "no room before the words");
}

/// The left edge of an item: flush right at the edge of its side, right
/// after the item before it, else at its own x.
#[test]
fn an_item_stands_flush_after_or_at_its_own_x() {
    let flush_right = item(false, Put::End, 0.0);
    assert_eq!(
        left_of(&flush_right, 300.0, None),
        300.0,
        "flush with the side"
    );
    assert_eq!(
        left_of(&flush_right, -5.0, None),
        0.0,
        "never left of the side"
    );
    let after = item(false, Put::After, 10.0);
    assert_eq!(
        left_of(&after, 300.0, Some(40.0)),
        40.0,
        "right after the item before"
    );
    assert_eq!(left_of(&after, 300.0, None), 10.0, "without one, its own x");
    let stacked = item(false, Put::Stack, 25.0);
    assert_eq!(
        left_of(&stacked, 300.0, Some(40.0)),
        25.0,
        "a stack keeps its x"
    );
}

/// The focus mark of a hit of mc shows while the pointer is over it and
/// hides when it leaves; a node that is not a hit of mc changes nothing.
#[test]
fn the_focus_mark_follows_the_pointer_over_a_hit_of_mc() {
    let (mut mc, model) = testing::mc("hits-mark", [1280, 800]);
    mc.show("agenda", false).unwrap();
    let hit = "mc-s-agenda-chapter-head-hit";
    let mark = mark_id("mc-s-agenda-chapter-head");
    mc.mark(hit, true).unwrap();
    assert_eq!(
        opacity(&model, &mark),
        Some(1.0),
        "the mark shows under the pointer"
    );
    mc.mark(hit, false).unwrap();
    assert_eq!(
        opacity(&model, &mark),
        Some(0.0),
        "the mark hides when it leaves"
    );
    mc.mark("other-hit", true).unwrap();
    mc.mark("mc-s-agenda-chapter-head", true).unwrap();
    assert_eq!(opacity(&model, &mark), Some(0.0), "no other node is marked");
}

/// A node without the `-hit` suffix is no hit: a click on it is not
/// answered, and the page is not changed.
#[test]
fn a_node_without_the_hit_suffix_is_not_a_hit() {
    let (mut mc, _model) = testing::mc("hits-not", [1280, 800]);
    assert!(
        !mc.click_hit("mc-s-agenda-chapter-head"),
        "no suffix, no hit"
    );
}

/// A click on a row of a page is answered as a hit, and a row that does
/// not name its page, panel or index is not a row.
#[test]
fn a_click_on_a_row_of_a_page_is_a_hit_and_a_malformed_row_is_not() {
    let (mut mc, _model) = testing::mc("hits-row", [1280, 800]);
    assert!(
        mc.click_hit("mc-s-agenda-panels-list-row-1-hit"),
        "a row of a page"
    );
    assert!(!mc.click_hit("mc-x-hit"), "not a scene of mc");
    assert!(
        !mc.click_hit("mc-s-agenda-panels-nodash-hit"),
        "no row in the name"
    );
    assert!(
        !mc.click_hit("mc-s-agenda-panels-list-row-x-hit"),
        "no index in the name"
    );
}

/// A click on a row of the page list shows the scene of that row.
#[test]
fn a_click_on_a_row_of_the_page_list_shows_that_scene() {
    let (mut mc, _model) = testing::mc("hits-overview", [1280, 800]);
    let first = mc
        .order()
        .into_iter()
        .next()
        .expect("a scene is in the order");
    let row = format!("mc-s-{OVERVIEW}-panels-pages-row-0-hit");
    assert!(mc.click_hit(&row), "the row is a hit");
    assert_eq!(
        mc.current.as_deref(),
        Some(first.as_str()),
        "the first page is shown"
    );
}

/// A row whose choice cannot be sent is logged, and the click is still a
/// hit of mc.
#[test]
fn a_refused_row_choice_is_only_logged() {
    let core = FakeCore::start();
    let mut mc = testing::mc_on(&core, "hits-refused", [1280, 800]);
    core.refuse_next("events.emit");
    assert!(
        mc.click_hit("mc-s-agenda-panels-list-row-2-hit"),
        "still a hit"
    );
}
