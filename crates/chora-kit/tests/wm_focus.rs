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

/// The policy of the look: the last window of its book that is not
/// `gone`.
impl Policy for Look {
    fn focus_after(&self, gone: &str, _why: &Why) -> Option<String> {
        (self.book.windows())
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
    assert_eq!(
        xp.focus.holder(xp.name, |s| xp.book.has(s)),
        Holder::Foreign
    );
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
    assert_eq!(
        mc.focus.holder(mc.name, |_| false),
        Holder::Node("page".into())
    );
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
    let after = |look: &Look, gone: &str, why: &Why| look.focus.after(gone, why, look);
    assert_eq!(after(&look, "t3", &Why::Destroyed), Some(Some("t2".into())));
    assert_eq!(after(&look, "t1", &Why::Destroyed), None, "t1 had no focus");
    assert_eq!(after(&look, "t3", &Why::Given("xp".into())), None);
    look.focus.gave("t3");
    assert_eq!(
        look.focus.holder(look.name, |s| look.book.has(s)),
        Holder::Foreign
    );
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
    assert_eq!(
        look.focus.holder(look.name, |s| look.book.has(s)),
        Holder::Nobody
    );
    look.focus.sent(Some("t1".into()));
    assert_eq!(
        look.focus.holder(look.name, |s| look.book.has(s)),
        Holder::Window("t1".into())
    );
    look.focus.sent(Some("gone".into()));
    assert_eq!(
        look.focus.holder(look.name, |s| look.book.has(s)),
        Holder::Nobody
    );
    look.focus.hear(&changed(Some("t1")));
    assert_eq!(
        look.focus.holder(look.name, |s| look.book.has(s)),
        Holder::Window("t1".into()),
        "a window of the look that the core focused holds it"
    );
    let command = look.focus.to(Some("t1".into()));
    assert!(matches!(command, Command::Focus { .. }));
    assert!(look.focus.sent && !look.focus.sends(Some("t1")));
}

/// The `focus.node` of the core, read from its JSON as a client gets it.
fn node_event(node: Option<&str>, owner: Option<&str>) -> Event {
    serde_json::from_value(serde_json::json!({"event": "focus.node", "node": node, "owner": owner}))
        .unwrap()
}

/// I3, p2r2 ricer: tiling and xp on one output. Tiling focused its window
/// t1, then the Start menu of xp took the keyboard with a node, which
/// leaves the focused surface as it is: tiling holds no focus while the
/// node of xp has the keyboard, and holds it again when the node lets go.
#[test]
fn a_node_of_another_look_takes_the_focus_from_the_window() {
    let mut tiling = Look::new("tiling", &["t1"]);
    tiling.focus.sent(Some("t1".into()));
    tiling.focus.hear(&changed(Some("t1")));
    assert_eq!(tiling.state(), holds("tiling", Some("t1")));
    tiling.focus.hear(&node_event(Some("xp-menu"), Some("xp")));
    assert_eq!(
        tiling.state(),
        state::Focus::default(),
        "the menu of xp has the keyboard"
    );
    assert_eq!(
        tiling.focus.holder(tiling.name, |s| tiling.book.has(s)),
        Holder::Foreign
    );
    tiling.focus.hear(&node_event(None, None));
    assert_eq!(
        tiling.state(),
        holds("tiling", Some("t1")),
        "the keyboard is back on t1"
    );
    tiling
        .focus
        .hear(&node_event(Some("tiling-keys"), Some("tiling")));
    assert_eq!(
        tiling.state(),
        holds("tiling", Some("t1")),
        "an own node is no loss"
    );
}

/// rw-5: an own node that the look does not track (the key list of
/// tiling, no `to_node`) is no loss for `holder` either: it answers the
/// window of the look, as `state` does.
#[test]
fn holder_and_state_agree_for_an_own_node_that_is_not_tracked() {
    let mut tiling = Look::new("tiling", &["t1"]);
    tiling.focus.sent(Some("t1".into()));
    tiling.focus.hear(&changed(Some("t1")));
    tiling
        .focus
        .hear(&node_event(Some("tiling-keys"), Some("tiling")));
    assert_eq!(tiling.state(), holds("tiling", Some("t1")));
    assert_eq!(
        tiling.focus.holder(tiling.name, |s| tiling.book.has(s)),
        Holder::Window("t1".into())
    );
}

/// co-i3 r3, rv-2: when a window takes the keys from a node of another
/// client, the core tells `focus.node` null (`tell_key_node`, chora
/// `state.rs`) and `focus.changed`; `moved` needs no clearing of its own,
/// the look holds the window again.
#[test]
fn a_window_taking_the_keys_from_a_foreign_node_ends_the_loss() {
    let mut tiling = Look::new("tiling", &["t1", "t2"]);
    tiling.focus.sent(Some("t1".into()));
    tiling.focus.hear(&changed(Some("t1")));
    tiling.focus.hear(&node_event(Some("xp-menu"), Some("xp")));
    assert_eq!(tiling.state(), state::Focus::default());
    tiling.focus.hear(&node_event(None, None));
    tiling.focus.hear(&changed(Some("t2")));
    assert_eq!(tiling.state(), holds("tiling", Some("t2")));
}

/// sh-42, D119: a look that joins beside a focused window of another look
/// sends no empty focus with its first transaction, which would take the
/// keys from that window; a look that sent a window still drops it.
#[test]
fn a_joining_look_sends_no_empty_focus() {
    let tiling = Look::new("tiling", &[]);
    assert!(!tiling.focus.sends(None), "the join takes the keyboard");
    let mut look = Look::new("tiling", &["t1"]);
    look.focus.sent(Some("t1".into()));
    assert!(look.focus.sends(None), "the look drops its own focus");
}

/// D119: the look keeps the keyboard when its focused window goes to a
/// look on another output: its next window takes it, or without one its
/// desktop node; either way the look says it holds the keyboard.
#[test]
fn a_kept_keyboard_goes_to_the_next_window_or_the_desktop() {
    let mut look = Look::new("tiling", &["s1", "s2"]);
    look.focus.sent(Some("s2".into()));
    let keep = look.focus.keep(Some("s1".into()), "tiling");
    assert_eq!(
        keep,
        Command::Focus {
            surface: Some("s1".into())
        }
    );
    look.focus.moved(Some("s1"));
    assert_eq!(look.state().surface.as_deref(), Some("s1"));

    let mut alone = Look::new("tiling", &["s1"]);
    alone.focus.sent(Some("s1".into()));
    let keep = alone.focus.keep(None, "tiling");
    assert_eq!(
        keep,
        Command::FocusNode {
            node: Some("tiling".into())
        }
    );
    assert_eq!(
        alone.focus.holder("tiling", |_| false),
        Holder::Node("tiling".into())
    );
    let state = alone.state();
    assert_eq!(
        (state.holder.as_deref(), state.surface),
        (Some("tiling"), None)
    );
    assert!(
        !alone.focus.sends(None),
        "a relayout without a window sends no focus"
    );
}

/// D119: a window given in takes the keyboard when nobody holds it, the
/// look itself does or a passing node of another look does, and not while
/// a window or the desktop of another look holds it.
#[test]
fn a_window_given_in_takes_the_keyboard_only_when_no_other_client_holds_it() {
    let mut xp = Look::new("xp", &["s5"]);
    let own = |s: &str| s == "s5";
    assert!(xp.focus.takes_given("xp", own), "nobody holds the keyboard");
    xp.focus.moved(Some("s5"));
    assert!(xp.focus.takes_given("xp", own), "the look holds it");
    xp.focus.moved(Some("s1"));
    assert!(
        !xp.focus.takes_given("xp", own),
        "a window of tiling holds it"
    );
    xp.focus.moved(None);
    xp.focus.hear(&Event::FocusNode {
        node: Some("tiling".into()),
        owner: Some("tiling".into()),
    });
    assert!(
        !xp.focus.takes_given("xp", own),
        "the desktop of tiling holds it"
    );
    xp.focus.hear(&Event::FocusNode {
        node: Some("construct-walk".into()),
        owner: Some("construct".into()),
    });
    assert!(
        xp.focus.takes_given("xp", own),
        "the walk of the construct holds it only for a moment"
    );
}
