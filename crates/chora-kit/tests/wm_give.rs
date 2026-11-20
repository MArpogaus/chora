//! The give of a window to another window manager (`kit::wm::give`): the
//! proposal of the book, the answer of the core, and the pass-on of a
//! surface that the core reserved for a look without outputs.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use chora_kit::{
    testing::{FakeCore, created, has, owner, surface},
    wm::{Book, Change, Origin, Phase, Why, give::pass_on},
};
use chora_scene::{Command, ROOT};

const NAME: &str = "look";

/// A book that owns `id`, as the look settles a new window of its own.
fn owning(id: &str) -> Book {
    let mut book = Book::new(NAME, false);
    let changes = book.claim(id, None, Some("t1".into()));
    assert!(
        matches!(&changes[..], [Change::Adopt { .. }]),
        "{changes:?}"
    );
    book.confirm(id);
    assert!(book.has(id));
    book
}

/// The look gives an own window: the give waits for the core, and the
/// confirm forgets the window and says that it went to `to`.
#[test]
fn a_given_window_is_forgotten_once_the_core_took_it() {
    let mut book = owning("s1");
    assert!(book.give("s1", "other"), "an own window can go");
    assert!(book.has("s1"), "the book keeps it until the core confirms");
    let changes = book.confirm("s1");
    assert_eq!(
        changes,
        [
            Change::Phase {
                id: "s1".into(),
                phase: Phase::Given,
            },
            Change::Forget {
                id: "s1".into(),
                why: Why::Given("other".into()),
            },
        ]
    );
    assert!(!book.has("s1"));
    assert_eq!(book.origin("s1"), None);
}

/// A give that the core refuses keeps the window, and no change follows.
#[test]
fn a_refused_give_keeps_the_window() {
    let mut book = owning("s1");
    assert!(book.give("s1", "other"));
    assert!(book.refused("s1").is_empty(), "nothing changes");
    assert_eq!(
        book.origin("s1"),
        Some(&Origin::New {
            token: Some("t1".into())
        })
    );
    assert!(book.confirm("s1").is_empty(), "the proposal is gone");
    assert!(book.has("s1"));
}

/// A window that is not the look's cannot be given, and the look cannot
/// give a window to itself.
#[test]
fn a_give_of_a_foreign_window_or_to_oneself_is_refused() {
    let mut book = owning("s1");
    assert!(!book.give("s9", "other"), "not the look's");
    assert!(!book.give("s1", NAME), "to itself");
    assert!(book.confirm("s1").is_empty(), "no give was proposed");
    assert!(book.has("s1"));
}

/// A window given in is the look's no longer, and a second give of it
/// is refused like any foreign window.
#[test]
fn a_window_that_was_given_cannot_be_given_again() {
    let mut book = owning("s1");
    assert!(book.give("s1", "other"));
    book.confirm("s1");
    assert!(!book.give("s1", "other"));
}

/// The pass-on attaches the reserved surface to the look and removes it
/// at once: a removed surface is detached, so it stays on the core, and no
/// look owns it any more, which leaves it free for any window manager.
#[test]
fn a_pass_on_leaves_the_surface_free_for_any_manager() {
    let core = FakeCore::start();
    let mut conn = core.connect(NAME);
    conn.apply(vec![Command::Create {
        id: NAME.into(),
        kind: chora_scene::NodeKind::Group,
        parent: ROOT.into(),
        props: chora_scene::PropsUpdate::default(),
    }])
    .unwrap();
    surface(&core.model(), "s1");
    assert_eq!(owner(&core.model(), "s1"), None, "free before the pass-on");

    let commands = pass_on("s1", NAME);
    assert!(
        matches!(
            &commands[..],
            [
                Command::Attach { surface, parent },
                Command::Remove { id, to: None },
            ] if surface == "s1" && parent == NAME && id == "s1"
        ),
        "{commands:?}"
    );
    conn.apply(commands).unwrap();
    assert!(has(&core.model(), "s1"), "the surface stays on the core");
    assert_eq!(owner(&core.model(), "s1"), None, "no look owns it any more");
}

/// A window that the look gives away is one the core announced to it:
/// the give leaves no phase for a window the look never opened.
#[test]
fn a_created_window_given_away_opens_then_closes_in_order() {
    let core = FakeCore::start();
    let mut book = Book::new(NAME, false);
    surface(&core.model(), "s1");
    let adopt = book.apply(&created("s1", None));
    assert!(matches!(&adopt[..], [Change::Adopt { .. }]), "{adopt:?}");
    assert_eq!(book.confirm("s1").len(), 1, "the window opened");
    assert!(book.give("s1", "other"));
    let gone = book.confirm("s1");
    assert!(
        matches!(
            gone.first(),
            Some(Change::Phase {
                phase: Phase::Given,
                ..
            })
        ),
        "{gone:?}"
    );
}
