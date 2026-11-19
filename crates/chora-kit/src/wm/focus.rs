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
}

impl Focus {
    /// Takes in a `focus.changed` of the core; other events change
    /// nothing.
    pub fn hear(&mut self, event: &Event) {
        if let Event::FocusChanged { surface } = event {
            self.moved(surface.as_deref());
        }
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
        }
    }

    /// Another client took the keyboard without a window, as a chat that
    /// opens over the look.
    pub fn lose(&mut self) {
        self.lost = true;
        self.node = None;
        self.core = None;
    }

    /// A window of the look takes the focus back: the next transaction
    /// sends it, even when it is the one sent before.
    pub fn take(&mut self) {
        if self.lost {
            self.lost = false;
            self.sent = false;
        }
    }

    /// Whether a transaction sends the focus `focus`.
    #[must_use]
    pub fn sends(&self, focus: Option<&str>) -> bool {
        !self.lost && (!self.sent || self.surface.as_deref() != focus)
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

    /// Who holds the keyboard; `own` tells the windows of the look. A
    /// window of the look that the core focuses holds it, also one that
    /// the look did not send.
    #[must_use]
    pub fn holder(&self, own: impl Fn(&str) -> bool) -> Holder {
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
        let (holder, surface) = match self.holder(own) {
            Holder::Window(surface) => (Some(name.into()), Some(surface)),
            Holder::Node(_) => (Some(name.into()), None),
            Holder::Foreign | Holder::Nobody => (None, None),
        };
        state::Focus { holder, surface }
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
