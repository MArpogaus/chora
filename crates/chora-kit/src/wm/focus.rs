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

use super::{Book, Why, state};

/// The rules of focus that a look decides; it sees its own model.
pub trait Policy {
    /// The window of the look that gets the keyboard after its focused
    /// window `gone` went for `why`; none leaves the keyboard to the
    /// look's own nodes, or to nobody.
    fn focus_after(&self, gone: &str, why: &Why, book: &Book) -> Option<String>;
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
}

impl Focus {
    /// Takes in a `focus.changed` of the core; other events change
    /// nothing.
    pub fn hear(&mut self, event: &Event) {
        let _ = event;
    }

    /// The core moved the focus to `surface`.
    pub fn moved(&mut self, surface: Option<&str>) {
        let _ = surface;
    }

    /// The look gives the window with its focus to another window manager,
    /// which focuses it.
    pub fn gave(&mut self, surface: &str) {
        let _ = surface;
    }

    /// Another client took the keyboard without a window, as a chat that
    /// opens over the look.
    pub fn lose(&mut self) {}

    /// A window of the look takes the focus back: the next transaction
    /// sends it, even when it is the one sent before.
    pub fn take(&mut self) {}

    /// Whether a transaction sends the focus `focus`.
    #[must_use]
    pub fn sends(&self, focus: Option<&str>) -> bool {
        let _ = focus;
        true
    }

    /// A transaction sent the focus `focus`.
    pub fn sent(&mut self, focus: Option<String>) {
        self.surface = focus;
    }

    /// The command that gives the keyboard to `surface`, or to none.
    pub fn to(&mut self, surface: Option<String>) -> Command {
        Command::Focus { surface }
    }

    /// The command that gives the keyboard to an own node, or ends the
    /// focus of the own node.
    pub fn to_node(&mut self, node: Option<String>) -> Command {
        Command::FocusNode { node }
    }

    /// Who holds the keyboard; `own` tells the windows of the look.
    #[must_use]
    pub fn holder(&self, own: impl Fn(&str) -> bool) -> Holder {
        let _ = (own, &self.node);
        Holder::Nobody
    }

    /// The focus of `state.get` of the look `name`.
    #[must_use]
    pub fn state(&self, name: &str, own: impl Fn(&str) -> bool) -> state::Focus {
        let _ = (name, own);
        state::Focus::default()
    }

    /// The focus after the window `gone` went for `why`: `None` when the
    /// look did not hold the keyboard on it or gave it away (the receiver
    /// focuses it), else what the policy says.
    #[must_use]
    pub fn after(
        &self,
        gone: &str,
        why: &Why,
        policy: &dyn Policy,
        book: &Book,
    ) -> Option<Option<String>> {
        let _ = (gone, why, policy, book);
        None
    }
}
