//! The keyboard focus of a look (arch plan 2.2): which window or own node
//! of the look holds it, as the look sent it and the core tells with
//! `focus.changed`, and the window that gets it when the focused one goes,
//! as the look's [`Policy`] says.
//!
//! Another window manager may take the focus at any time (a click into
//! its pane, a window it opens). The look then holds no focus and sends
//! none until one of its windows takes it back, so that no relayout of
//! the look steals the keyboard.

use chora_scene::{Command, Event};

use super::{Why, state};

/// The rules of focus that a look decides; it sees its own model.
pub trait Policy {
    /// The window of the look that gets the keyboard after its focused
    /// window `gone` went for `why`; none leaves the keyboard to the
    /// look's own nodes, or to nobody.
    fn focus_after(&self, gone: &str, why: &Why) -> Option<String>;
}

/// The command that takes the keyboard focus from every surface, for a
/// client that gives the keyboard to a node of its own next.
#[must_use]
pub fn release() -> Command {
    Command::Focus { surface: None }
}

/// The command that gives the keyboard to the window `surface`, for a
/// client that keeps no [`Focus`] (a look keeps one and sends
/// [`Focus::to`]).
#[must_use]
pub fn give(surface: impl Into<String>) -> Command {
    Command::Focus {
        surface: Some(surface.into()),
    }
}

/// The surface of a `focus.changed` of the core: `Some(None)` when the
/// core focuses no surface, `None` for every other event.
#[must_use]
pub fn changed(event: &Event) -> Option<Option<&str>> {
    match event {
        Event::FocusChanged { surface } => Some(surface.as_deref()),
        _ => None,
    }
}

/// Who holds the keyboard, seen from one look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Holder {
    /// A window of the look.
    Window(String),
    /// A node of the look, through `focus_node`.
    Node(String),
    /// Another client took it.
    Foreign,
    /// Nobody: the look sent no focus.
    Nobody,
}

/// The keyboard focus as the look shows it to the core.
#[derive(Debug, Default, Clone)]
pub struct Focus {
    /// The surface of the last focus that the look sent.
    pub surface: Option<String>,
    /// Whether the core has `surface`.
    pub sent: bool,
    /// Another client took the keyboard since. The look sends no focus
    /// until one of its windows takes it back.
    pub lost: bool,
    /// The own node that has the keyboard, through `focus_node`.
    node: Option<String>,
    /// The surface that the core focuses, as it told or the look sent.
    core: Option<String>,
    /// The owner of a node that took the keyboard, as `focus.node` told,
    /// unless it is the own node of `node`: the focused surface stays, but
    /// no window has the keys meanwhile.
    away: Option<String>,
    /// The node of `away` is the desktop of its look, its root node,
    /// which holds the keyboard in place of a window (D119).
    away_desk: bool,
}

impl Focus {
    /// Takes in a `focus.changed` of the core and returns its surface, as
    /// [`changed`]; other events change nothing.
    pub fn hear<'a>(&mut self, event: &'a Event) -> Option<Option<&'a str>> {
        if let Event::FocusNode { node, owner } = event {
            self.away = (node.as_ref())
                .filter(|n| Some(n.as_str()) != self.node.as_deref())
                .and(owner.clone());
            self.away_desk = self.away.is_some() && node == owner;
            return None;
        }
        let surface = changed(event)?;
        self.moved(surface);
        Some(surface)
    }

    /// The own node that has the keyboard, as the look sent it.
    #[must_use]
    pub fn node(&self) -> Option<&str> {
        self.node.as_deref()
    }

    /// The core does not focus the own node any more: another client took
    /// the keyboard with a node, which tells the look nothing.
    pub fn drop_node(&mut self) {
        self.node = None;
    }

    /// The surface that the core focuses, as it told or the look sent.
    #[must_use]
    pub fn core(&self) -> Option<&str> {
        self.core.as_deref()
    }

    /// The core moved the focus to `surface`. An empty focus is no loss:
    /// it happens when the focused window of the look itself exits, and
    /// the look must then still focus its next window. A window ends the
    /// focus of a node.
    pub fn moved(&mut self, surface: Option<&str>) {
        self.core = surface.map(String::from);
        let Some(surface) = surface else {
            return;
        };
        self.node = None;
        self.lost = Some(surface) != self.surface.as_deref();
    }

    /// The look gives the window with its focus to another window manager,
    /// which focuses it.
    pub fn gave(&mut self, surface: &str) {
        if self.surface.as_deref() == Some(surface) {
            self.lose();
            self.surface = None;
            self.core = None;
        }
    }

    /// Another client took the keyboard without a window, as a chat that
    /// opens over the look. Its field is a layer surface: the node of the
    /// look loses the keyboard, but the core still focuses the window
    /// under the field, which stays the holder (I3).
    pub fn lose(&mut self) {
        self.lost = true;
        self.node = None;
    }

    /// A window of the look takes the focus back: the next transaction
    /// sends it, even when it is the one sent before.
    pub fn take(&mut self) {
        if self.lost {
            self.lost = false;
            self.sent = false;
        }
    }

    /// A click or a pointer on a window of the look that the look had lost
    /// to another client or node ([`Focus::lost_to_other`]): the next
    /// transaction sends the focus, even when it is the one sent before.
    pub fn reclaim(&mut self) {
        self.lost = false;
        self.sent = false;
    }

    /// Whether a transaction sends the focus `focus`. An empty focus only
    /// drops a window that the look sent: a look that joins beside the
    /// focused window of another look leaves it the keys (D119, sh-42).
    #[must_use]
    pub fn sends(&self, focus: Option<&str>) -> bool {
        let drops_nothing = focus.is_none() && self.surface.is_none();
        !self.lost && !drops_nothing && (!self.sent || self.surface.as_deref() != focus)
    }

    /// A transaction sent the focus `focus`.
    pub fn sent(&mut self, focus: Option<String>) {
        if focus.is_some() {
            self.node = None;
        }
        self.core.clone_from(&focus);
        self.surface = focus;
        self.sent = true;
    }

    /// The command that gives the keyboard to `surface`, or to none.
    pub fn to(&mut self, surface: Option<String>) -> Command {
        self.lost = false;
        self.sent(surface.clone());
        Command::Focus { surface }
    }

    /// The command that gives the keyboard to an own node, or ends the
    /// focus of the own node.
    pub fn to_node(&mut self, node: Option<String>) -> Command {
        if node.is_some() {
            self.lost = false;
        }
        self.node.clone_from(&node);
        Command::FocusNode { node }
    }

    /// Who holds the keyboard, seen from the look `name`; `own` tells the
    /// windows of the look. A window of the look that the core focuses
    /// holds it, also one that the look did not send; a node of the look
    /// itself that `focus.node` tells is no loss.
    #[must_use]
    pub fn holder(&self, name: &str, own: impl Fn(&str) -> bool) -> Holder {
        if self.taken_by_other(name) {
            return Holder::Foreign;
        }
        self.held(own)
    }

    /// Whether a node of another client than `name` has the keyboard.
    #[must_use]
    pub fn taken_by_other(&self, name: &str) -> bool {
        self.away.as_deref().is_some_and(|owner| owner != name)
    }

    /// Whether the look `name` has lost the keyboard that it sent: another
    /// client took it ([`Focus::lost`]), a node of another client has it, or
    /// the core focuses another surface than the sent one, or none.
    #[must_use]
    pub fn lost_to_other(&self, name: &str) -> bool {
        self.lost
            || self.taken_by_other(name)
            || (self.sent && self.surface.is_some() && self.core != self.surface)
    }

    /// Who holds the keyboard, without the node of `focus.node`.
    fn held(&self, own: impl Fn(&str) -> bool) -> Holder {
        if let Some(node) = &self.node {
            return Holder::Node(node.clone());
        }
        match self.core.as_deref() {
            Some(surface) if own(surface) => Holder::Window(surface.into()),
            _ if self.lost => Holder::Foreign,
            _ => Holder::Nobody,
        }
    }

    /// The focus of `state.get` of the look `name`.
    #[must_use]
    pub fn state(&self, name: &str, own: impl Fn(&str) -> bool) -> state::Focus {
        let (holder, surface) = match self.holder(name, own) {
            Holder::Window(surface) => (Some(name.into()), Some(surface)),
            Holder::Node(_) => (Some(name.into()), None),
            Holder::Foreign | Holder::Nobody => (None, None),
        };
        state::Focus { holder, surface }
    }

    /// The command that keeps the keyboard with the look when its focused
    /// window goes to a look on another output (D119): the look's `next`
    /// window takes it, or without one its desktop `desk`, its root node
    /// named as the look, so that the look holds it with its desktop. The look sends it in a transaction
    /// before the give, so that the receiver sees the holder when the
    /// window comes and leaves the keyboard where it is.
    pub fn keep(&mut self, next: Option<String>, desk: &str) -> Command {
        if next.is_some() {
            return self.to(next);
        }
        self.lost = false;
        self.sent(None);
        self.to_node(Some(desk.into()))
    }

    /// Whether a window given in to the look `name` takes the keyboard
    /// (D119): not while another look holds it, with a window that the
    /// core focuses or with its desktop, as the giver that kept it on the
    /// output of the person ([`Focus::keep`]); `own` tells the windows of
    /// the look. Another node, such as a menu or the walk of a look, holds
    /// the keyboard only for a moment and does not keep it.
    #[must_use]
    pub fn takes_given(&self, name: &str, own: impl Fn(&str) -> bool) -> bool {
        let desk = self.away_desk && self.taken_by_other(name);
        !desk && self.core.as_deref().is_none_or(own)
    }

    /// The focus after the window `gone` went for `why`: `None` when the
    /// look did not hold the keyboard on it or gave it away (the receiver
    /// focuses it), else what the policy says.
    #[must_use]
    pub fn after(&self, gone: &str, why: &Why, policy: &dyn Policy) -> Option<Option<String>> {
        let held = !self.lost && self.surface.as_deref() == Some(gone);
        (held && !matches!(why, Why::Given(_))).then(|| policy.focus_after(gone, why))
    }
}
