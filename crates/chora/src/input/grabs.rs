//! The pointer grabs that a client name holds, and the rules that let one go.

use chora_scene::{Command, ipc::RpcError};

use super::{
    ButtonGrab, GivenGrab, HeldPress, MAX_WOBBLE, MOVE, Press, PressState, State, send_button,
};

impl State {
    /// The press of the pointer grab that a client name holds: the grab
    /// of a button binding or of `move`, or the press on one of its own
    /// nodes.
    pub(super) fn held_press(&self, name: &str) -> Option<HeldPress> {
        let of_button = (self.button_grab.as_ref())
            .filter(|g| self.ipc.name(g.conn) == Some(name))
            .map(|g| HeldPress {
                button: g.button,
                real: g.real,
                window: g.window.clone(),
                node: None,
            });
        let owns = |node: &str| {
            let owner = self.scene.node(node).and_then(|n| n.owner.as_deref());
            owner == Some(name)
        };
        let of_node = (self.node_grab.as_ref())
            .filter(|g| owns(&g.node))
            .map(|g| HeldPress {
                button: g.button,
                real: g.real,
                window: None,
                node: Some(g.node.clone()),
            });
        of_button.or(of_node)
    }

    /// Whether a press drags a window: the window asked for the move or
    /// was under the pointer at the press of a binding, or the press hit
    /// a node of the group that holds the window, as a title bar.
    pub(super) fn drags(&self, held: &HeldPress, surface: &str) -> bool {
        if held.window.as_deref() == Some(surface) {
            return true;
        }
        let Some(group) = (held.node.as_deref())
            .and_then(|node| self.scene.node(node))
            .and_then(|node| node.parent.as_deref())
        else {
            return false;
        };
        let mut parent = self.scene.node(surface).and_then(|n| n.parent.as_deref());
        while let Some(id) = parent {
            if id == group {
                return true;
            }
            parent = self.scene.node(id).and_then(|n| n.parent.as_deref());
        }
        false
    }

    /// Why the window manager `name` with the connections `conns` must
    /// not give its drag of `surface` to the window manager `to`
    /// (`give_grab`), if so; else the connection of `to` that gets it and
    /// the press. The grab moves only between window managers: both hold
    /// a claim, neither is confined, and the giver answers no method call
    /// of a confined caller. No other client holds a button. The giver
    /// holds the press of a button that the user made and still holds,
    /// with no injected input since, and that press drags `surface`. The
    /// receiver grabbed `move`, which carries the rest of the drag.
    pub(super) fn grab_give_refusal(
        &self,
        conns: &[u64],
        name: &str,
        (surface, to): (&str, &str),
    ) -> Result<(u64, Press), String> {
        let confined = |c: &u64| self.ipc.confined(*c) || self.methods.serves_confined(*c);
        if conns.is_empty() || conns.iter().any(confined) {
            return Err("a confined client, or a call of one, gives no drag".into());
        }
        if !self.scene.holds_claim(name) || !self.scene.holds_claim(to) {
            return Err("a drag goes only between window managers with a claim".into());
        }
        let foreign = |g: &ButtonGrab| self.ipc.name(g.conn) != Some(name);
        if self.button_grab.as_ref().is_some_and(foreign) {
            return Err("another client holds a button of the pointer".into());
        }
        let Some(held) = self.held_press(name) else {
            return Err(format!("{name} holds no pointer grab"));
        };
        let press = (held.real).filter(|s| self.last_real_button == Some((*s, held.button)));
        let Some(press) = press.filter(|_| self.lock.is_none()) else {
            return Err("only the drag of a real press that the user still holds goes".into());
        };
        if !self.drags(&held, surface) {
            return Err(format!("the press is no drag of {surface}"));
        }
        if self.spot(self.pointer_layout).is_none() {
            return Err("the pointer is on no output".into());
        }
        let receiver = (self.ipc.move_grab_of(to)).filter(|c| !self.ipc.confined(*c));
        let receiver = receiver.ok_or_else(|| format!("{to} grabbed no `move`, or is confined"))?;
        Ok((receiver, (press, held.button)))
    }

    /// Refuses a transaction with a `give_grab` that must not move the
    /// pointer grab, before any command of it runs: by the rules of the
    /// grab, and a state of the drag that is not finite or larger than
    /// [`MAX_WOBBLE`]. Answers the press of the drag, which the
    /// transaction keeps until it applies.
    pub fn check_grab_gives(
        &self,
        conn: u64,
        commands: &[Command],
    ) -> Result<Option<Press>, RpcError> {
        let refuse = |why: String| RpcError::new(RpcError::REJECTED, why);
        let name = self.ipc.name(conn).unwrap_or_default();
        let mut press = None;
        for command in commands {
            let Command::GiveGrab {
                surface,
                to,
                point,
                velocity,
                wobble,
            } = command
            else {
                continue;
            };
            if !point.iter().chain(velocity).all(|v| v.is_finite()) {
                return Err(refuse(
                    "the point and the velocity of a drag are finite".into(),
                ));
            }
            let size = wobble.as_ref().map_or(0, |w| w.to_string().len());
            if size > MAX_WOBBLE {
                return Err(refuse(format!(
                    "the wobble of a drag has at most {MAX_WOBBLE} bytes"
                )));
            }
            let (_, seen) = self
                .grab_give_refusal(&[conn], name, (surface, to))
                .map_err(refuse)?;
            press = Some(seen);
        }
        Ok(press)
    }

    /// Refuses a transaction with a `give_grab` when it applies: the
    /// rules hold again, and the press is the one of the check, so a
    /// release and a new press while the transaction waited give nothing.
    /// The whole transaction fails and the window stays.
    pub fn check_grab_still_gives(
        &self,
        owner: &str,
        commands: &[Command],
        checked: Option<Press>,
    ) -> Result<(), RpcError> {
        let refuse = |why: String| RpcError::new(RpcError::REJECTED, why);
        let conns = self.ipc.conns_of(owner);
        for command in commands {
            if let Command::GiveGrab { surface, to, .. } = command {
                let (_, press) = self
                    .grab_give_refusal(&conns, owner, (surface, to))
                    .map_err(refuse)?;
                if checked != Some(press) {
                    return Err(refuse(
                        "the press of the drag ended while the transaction waited".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Moves the pointer grab of the window manager `from` to `to` with
    /// the window that `from` gave it in this transaction: `to` gets the
    /// press of `move` with the window and the state of the drag, and
    /// `from` hears no more of the button. The transaction passed
    /// [`Self::check_grab_still_gives`] just before it applied.
    pub fn move_grab(&mut self, surface: &str, to: &str, from: &str, grab: GivenGrab) {
        let receiver = (self.ipc.move_grab_of(to)).filter(|c| !self.ipc.confined(*c));
        let held = self.held_press(from);
        let press = held.and_then(|h| Some((h.real?, h.button)));
        let (Some(receiver), Some((real, button))) = (receiver, press) else {
            tracing::warn!(%surface, %from, %to, "the drag stays: its press or its receiver went");
            return;
        };
        let own = |g: &mut ButtonGrab| self.ipc.name(g.conn) == Some(from);
        let of_button = self.button_grab.take_if(own);
        let to_pointer = of_button.as_ref().is_none_or(|g| g.to_pointer);
        if of_button.is_none() {
            self.node_grab = None;
        }
        self.button_grab = Some(ButtonGrab {
            conn: receiver,
            binding: MOVE.into(),
            button,
            real: Some(real),
            to_pointer,
            given: Some(surface.into()),
            window: Some(surface.into()),
        });
        self.unpin_press(to);
        let window = Some((surface, Some(grab)));
        send_button(
            self,
            (receiver, MOVE.into()),
            PressState::Pressed,
            window,
            false,
        );
    }

    /// Ends a pointer grab that came with a window when its holder does
    /// not have the window, and no reservation keeps it for the holder:
    /// the grab never stays with a client that did not get the window.
    /// The holder gets the release of `move`, so that its drag ends too.
    /// Returns whether a grab ended.
    pub fn drop_stray_grab(&mut self) -> bool {
        let Some(grab) = &self.button_grab else {
            return false;
        };
        let Some(window) = &grab.given else {
            return false;
        };
        let holder = self.ipc.name(grab.conn);
        let kept = self.scene.node(window).is_some_and(|node| {
            let holds = |name: &Option<String>| name.as_deref() == holder;
            holder.is_some() && (holds(&node.owner) || node.owner.is_none() && holds(&node.to))
        });
        if kept {
            return false;
        }
        if let Some(stray) = self.button_grab.take() {
            let injected = stray.real.is_none();
            let to = (stray.conn, stray.binding);
            send_button(self, to, PressState::Released, None, injected);
        }
        true
    }
}
