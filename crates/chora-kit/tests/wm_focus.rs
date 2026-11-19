//! Cells of `kit::wm::focus` (WM2): I3 of the arch plan, once quiet the
//! core focus is a shown window of the one look that says it holds the
//! focus (sh-23: two looks in two panes), and the focus after a window
//! goes, as the policy of the look says.

#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use chora_kit::wm::{
    Book, Why,
    focus::{Focus, Holder, Policy},
    state,
};
use chora_scene::{Command, Event};

/// A look with a book of its windows and its focus.
struct Look {
    name: &'static str,
    book: Book,
    focus: Focus,
}

impl Look {
    fn new(name: &'static str, windows: &[&str]) -> Self {
        let mut book = Book::new(name, false);
        for id in windows {
            book.keep(id);
        }
        Self {
            name,
            book,
            focus: Focus::default(),
        }
    }

    fn state(&self) -> state::Focus {
        self.focus.state(self.name, |s| self.book.has(s))
    }
}

/// The focus goes to the last window of the book that is not `gone`.
struct LastWindow;

impl Policy for LastWindow {
    fn focus_after(&self, gone: &str, _why: &Why, book: &Book) -> Option<String> {
        book.windows()
            .filter(|w| *w != gone)
            .last()
            .map(String::from)
    }
}

fn changed(surface: Option<&str>) -> Event {
    Event::FocusChanged {
        surface: surface.map(String::from),
    }
}

/// The core moves the focus to `surface`, and both looks hear it.
fn both_hear(looks: [&mut Look; 2], surface: &str) {
    for look in looks {
        look.focus.hear(&changed(Some(surface)));
    }
}

fn holds(name: &str, surface: Option<&str>) -> state::Focus {
    state::Focus {
        holder: Some(name.into()),
        surface: surface.map(String::from),
    }
}

/// I3, sh-23: xp and tiling in two panes. xp focused its window, then
/// the person clicked into the pane of tiling and tiling focused its
/// window: only tiling says it holds the focus, and xp sends none until
/// one of its windows takes it back.
#[test]
fn two_looks_in_two_panes_do_not_both_hold_the_focus() {
    let mut xp = Look::new("xp", &["x1"]);
    let mut tiling = Look::new("tiling", &["t1"]);
    assert_eq!(
        xp.focus.to(Some("x1".into())),
        Command::Focus {
            surface: Some("x1".into())
        }
    );
    both_hear([&mut xp, &mut tiling], "x1");
    assert_eq!(xp.state(), holds("xp", Some("x1")));
    assert_eq!(tiling.state(), state::Focus::default());
    tiling.focus.sent(Some("t1".into()));
    both_hear([&mut xp, &mut tiling], "t1");
    assert_eq!(tiling.state(), holds("tiling", Some("t1")));
    assert_eq!(xp.state(), state::Focus::default(), "xp lost the focus");
    assert_eq!(xp.focus.holder(|s| xp.book.has(s)), Holder::Foreign);
    assert!(
        !xp.focus.sends(Some("x1")),
        "a relayout of xp takes no focus"
    );
    xp.focus.take();
    assert!(xp.focus.sends(Some("x1")), "a click on x1 takes it back");
}

/// mc gives the keyboard to the node of its page when its window goes
/// (known red net2-a2): it holds the focus, without a surface, until
/// another window takes it.
#[test]
fn a_look_holds_the_focus_of_its_own_node() {
    let mut mc = Look::new("mc", &[]);
    let command = mc.focus.to_node(Some("page".into()));
    assert_eq!(
        command,
        Command::FocusNode {
            node: Some("page".into())
        }
    );
    assert_eq!(mc.focus.holder(|_| false), Holder::Node("page".into()));
    assert_eq!(mc.state(), holds("mc", None));
    mc.focus.hear(&changed(Some("other")));
    assert_eq!(mc.state(), state::Focus::default());
}

/// An empty focus is no loss: the focused window of the look exited and
/// the look still focuses its next window.
#[test]
fn an_empty_focus_is_no_loss() {
    let mut look = Look::new("tiling", &["t1", "t2"]);
    look.focus.sent(Some("t2".into()));
    look.focus.hear(&changed(None));
    assert!(look.focus.sends(Some("t1")));
    assert!(!look.focus.sends(Some("t2")), "t2 is sent already");
}

/// The focus after a window goes: the policy names the next window when
/// the gone one held the keyboard; a given window leaves the focus to
/// its receiver, and a window without the focus changes nothing.
#[test]
fn the_policy_says_where_the_focus_goes_after_a_window() {
    let mut look = Look::new("tiling", &["t1", "t2", "t3"]);
    look.focus.sent(Some("t3".into()));
    let after =
        |look: &Look, gone: &str, why: &Why| look.focus.after(gone, why, &LastWindow, &look.book);
    assert_eq!(after(&look, "t3", &Why::Destroyed), Some(Some("t2".into())));
    assert_eq!(after(&look, "t1", &Why::Destroyed), None, "t1 had no focus");
    assert_eq!(after(&look, "t3", &Why::Given("xp".into())), None);
    look.focus.gave("t3");
    assert_eq!(look.focus.holder(|s| look.book.has(s)), Holder::Foreign);
    look.focus.lose();
    assert_eq!(
        after(&look, "t3", &Why::Destroyed),
        None,
        "the focus is away"
    );
}

/// A focus that the look sent and the core took is a window of the look;
/// a window the book does not hold is no holder.
#[test]
fn the_holder_is_a_window_of_the_book() {
    let mut look = Look::new("tiling", &["t1"]);
    assert_eq!(look.focus.holder(|s| look.book.has(s)), Holder::Nobody);
    look.focus.sent(Some("t1".into()));
    assert_eq!(
        look.focus.holder(|s| look.book.has(s)),
        Holder::Window("t1".into())
    );
    look.focus.sent(Some("gone".into()));
    assert_eq!(look.focus.holder(|s| look.book.has(s)), Holder::Nobody);
    let command = look.focus.to(Some("t1".into()));
    assert!(matches!(command, Command::Focus { .. }));
    assert!(look.focus.sent && !look.focus.sends(Some("t1")));
}
