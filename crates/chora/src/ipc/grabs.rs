//! The `grab` and `ungrab` calls of a connection.

use chora_scene::Event;

use super::{
    Binding, Capability, MAX_GRABS, RpcError, State, Trigger, Value, is_safety_key, media_key,
    rejected, trigger,
};

impl State {
    /// Grabs a binding of a key or a pointer button for a named connection.
    /// A binding needs Ctrl, Alt or Super, so that typing and clicking still
    /// work; `input.grab-any` lifts this (D45), and a media key of
    /// [`MEDIA_KEYS`] needs none for an unconfined client with a claim. Ctrl+Alt with the keys of
    /// the safety layer belongs to the safety layer either way. The result
    /// is the binding in the normal form of `input.key`.
    /// A button binding may have several grabs while each client holds a
    /// claim; a press then goes by the claim under the pointer (D55).
    /// Only on the main socket: a confined client could claim a little
    /// less than an output and so take every press of the binding.
    /// Each window manager gets the moves of its own windows.
    pub(super) fn ipc_grab(&mut self, conn: u64, binding: Binding) -> Result<Value, RpcError> {
        let Some(name) = self.ipc.name(conn).map(str::to_owned) else {
            return Err(rejected("send hello first"));
        };
        let m = binding.mods;
        let trigger = trigger(&binding)?;
        let window_manager = !self.ipc.confined(conn) && self.scene.holds_claim(&name);
        let free = trigger == Trigger::Move
            || (media_key(trigger) && window_manager)
            || self.grants.allows(&name, Capability::GrabAny);
        if !(m.ctrl || m.alt || m.logo || free) {
            return Err(RpcError::new(
                RpcError::MISSING_CAPABILITY,
                format!("{binding} needs Ctrl, Alt or Super or the capability input.grab-any"),
            ));
        }
        if m.ctrl && m.alt && matches!(trigger, Trigger::Key(sym) if is_safety_key(sym)) {
            return Err(rejected(format!("{binding} belongs to the safety layer")));
        }
        let claims = |c: u64| {
            !self.ipc.confined(c) && (self.ipc.name(c)).is_some_and(|n| self.scene.holds_claim(n))
        };
        let own = trigger == Trigger::Move;
        let mut holders = (self.ipc.grabs.iter())
            .filter(|(b, t, c)| b.mods == binding.mods && *t == trigger && (!own || *c == conn))
            .map(|(_, _, c)| *c)
            .peekable();
        let shared = matches!(trigger, Trigger::Button(_)) && claims(conn);
        if holders.peek().is_some() && !(shared && holders.all(|c| c != conn && claims(c))) {
            return Err(rejected(format!("{binding} is grabbed already")));
        }
        if self.ipc.grabs.iter().filter(|(_, _, c)| *c == conn).count() >= MAX_GRABS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a connection holds at most {MAX_GRABS} grabs"),
            ));
        }
        let normal = binding.to_string();
        self.ipc.grabs.push((binding, trigger, conn));
        Ok(Value::String(normal))
    }

    /// Ends a grab of a connection (D45) and tells the subscribers of
    /// `input.ungrabbed` (D117).
    pub(super) fn ipc_ungrab(&mut self, conn: u64, binding: &Binding) -> Result<Value, RpcError> {
        let trigger = trigger(binding)?;
        let before = self.ipc.grabs.len();
        self.ipc
            .grabs
            .retain(|(b, t, c)| !(*c == conn && b.mods == binding.mods && *t == trigger));
        if self.ipc.grabs.len() == before {
            return Err(rejected(format!("{binding} is no grab of this connection")));
        }
        self.tell_ungrabbed([binding.clone()]);
        Ok(Value::Null)
    }

    /// Tells the subscribers of `input.ungrabbed` that grabs ended: by
    /// `input.ungrab`, with their connection or with the revoke of
    /// `input.grab-any`, so that a client that waits for the binding asks
    /// again (D117).
    pub(crate) fn tell_ungrabbed(&mut self, bindings: impl IntoIterator<Item = Binding>) {
        for binding in bindings {
            self.emit(&Event::InputUngrabbed {
                binding: binding.to_string(),
            });
        }
    }
}
