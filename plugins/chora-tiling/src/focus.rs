//! The keyboard focus as tiling shows it to the core.

/// The keyboard focus as tiling shows it to the core.
#[derive(Debug, Default)]
pub(super) struct ShownFocus {
    /// The focus of the last transaction that sent one.
    pub(super) surface: Option<String>,
    /// Whether the core has `surface`.
    pub(super) sent: bool,
    /// Another window manager took the focus since. Tiling sends none
    /// until one of its windows takes the focus back, so that no relayout
    /// takes it.
    pub(super) lost: bool,
}

impl ShownFocus {
    /// The core moved the focus. An empty focus is no loss: it happens
    /// when the focused window of tiling itself exits, and tiling must
    /// then still focus its next window.
    pub(super) fn moved(&mut self, surface: Option<&str>) {
        match surface {
            None => {}
            Some(s) if Some(s) == self.surface.as_deref() => self.lost = false,
            Some(_) => self.lost = true,
        }
    }

    /// Tiling hands the window with its focus to another window manager,
    /// which focuses it. Tiling sends no focus until one of its windows
    /// takes it back, whether the receiver set it before or after.
    pub(super) fn gave(&mut self, surface: &str) {
        if self.surface.as_deref() == Some(surface) {
            self.lost = true;
        }
    }

    /// A window of tiling takes the focus back: by the pointer, a binding
    /// or as a new window. The next transaction sends it, even when it is
    /// the one sent before.
    pub(super) fn take(&mut self) {
        if self.lost {
            self.lost = false;
            self.sent = false;
        }
    }

    /// Whether a transaction sends a focus.
    pub(super) fn sends(&self, focus: Option<&str>) -> bool {
        !self.lost && (!self.sent || self.surface.as_deref() != focus)
    }

    /// A transaction sent a focus.
    pub(super) fn sent(&mut self, focus: Option<String>) {
        self.surface = focus;
        self.sent = true;
    }
}
