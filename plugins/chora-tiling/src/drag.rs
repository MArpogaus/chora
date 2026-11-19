//! Handing windows to other window managers and dragging windows with the pointer.

use chora_kit::{actions::System, effects::Target, give::Give, runtime};
use chora_scene::{
    GivenGrab,
    ipc::{ClientError, PressState, RpcError},
};
use serde_json::Value;

use crate::{Tiling, commands::taken, config::Action, only};

impl Tiling {
    /// `window.give`: hands a window to another window manager, and
    /// answers with its id.
    pub(super) fn give(&mut self, params: Value) -> Result<Value, RpcError> {
        let Give { surface, to, grab } = runtime::params(params)?;
        self.hand_dragged(&surface, &to, grab)
    }

    /// A press of `move` that came with a window and its drag from
    /// another window manager: the pointer holds the point of the window
    /// that the giver names, kept inside the window, and the window
    /// wobbles on from its springs.
    pub(super) fn take_given_drag(&mut self, grab: Option<GivenGrab>) {
        let (Some(grab), Some((id, _))) = (grab, self.dragged.clone()) else {
            return;
        };
        let size = self.target(&id, [0.0; 2]).map_or([0.0; 2], |t| t.size);
        let point = [0, 1].map(|axis| grab.point[axis].clamp(0.0, size[axis].max(0.0)));
        self.dragged = Some((id.clone(), point));
        self.springs = grab.wobble.map(|springs| (id, springs));
    }

    /// Hands a window to the window manager `to` with a remove with `to`
    /// (D55); the receiver gives it the focus. The answer of
    /// `window.give`.
    pub(super) fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        self.hand_dragged(surface, to, None)
    }

    /// As [`Self::hand`]; with `grab` the drag of the window goes too,
    /// with the springs of its wobble unless the caller sent some, and
    /// the drag of tiling ends.
    pub(super) fn hand_dragged(
        &mut self,
        surface: &str,
        to: &str,
        mut grab: Option<GivenGrab>,
    ) -> Result<Value, RpcError> {
        only(self.wm.has(surface), || {
            format!("tiling has no window {surface}")
        })?;
        let bent = self.bent.remove(surface);
        if let (Some(grab), Some(target)) = (&mut grab, &bent) {
            let springs = self.effects.springs(surface, target);
            grab.wobble = grab.wobble.take().or(springs);
        }
        if let Some(flat) = self.effects.unbend(surface, surface) {
            self.apply(vec![flat])
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        let dragged = grab.is_some();
        self.book.give(surface, to);
        let answer = match self.rt.conn.give_dragged(surface, to, grab) {
            Ok(answer) => answer,
            Err(err) => {
                self.book.refused(surface);
                return Err(err);
            }
        };
        if dragged {
            self.dragged.take_if(|(id, _)| id == surface);
        }
        self.focus.gave(surface);
        let given = self.book.confirm(surface);
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        self.settle(given).map_err(rejected)?;
        self.forget_window(surface);
        self.relayout().map_err(rejected)?;
        Ok(answer)
    }

    /// Whether a binding drags: the one of `drag`, or `move`, the
    /// interactive move of a client, for example from its title bar.
    pub(super) fn drags(&self, binding: &str) -> bool {
        matches!(self.rt.action(binding), Some(Action::System(System::Drag)))
    }

    /// A drag: the press takes the window that asked for a move, else the
    /// window under the pointer, and a release outside the claim of tiling
    /// hands it to the window manager whose claim holds the point (D55).
    /// Without one there, the window stays.
    ///
    /// Back from where the pointer left it.
    pub(super) fn drag(
        &mut self,
        binding: &str,
        state: PressState,
        point: [f64; 2],
        asked: Option<String>,
    ) {
        if !self.drags(binding) {
            return;
        }
        if state == PressState::Pressed {
            if let Some((old, _)) = self.dragged.take() {
                self.effects.release(&old);
            }
            let grabbed = taken(asked, |id| self.wm.has(id), || self.window_at(point));
            self.dragged = grabbed.map(|id| {
                let [x, y] = (self.sent.get(&id))
                    .and_then(|p| p.at)
                    .map_or(point, |at| at.map(f64::from));
                (id, [point[0] - x, point[1] - y])
            });
            return;
        }
        let Some((surface, _)) = self.dragged.take() else {
            return;
        };
        self.effects.release(&surface);
        if self.wm.output_at(point).is_some() {
            return;
        }
        let given = match self.rt.conn.claimant_at(point) {
            Ok(Some(to)) => self.hand(&surface, &to).map(drop),
            Ok(None) => Err(RpcError::new(RpcError::REJECTED, "no claim there")),
            Err(err) => Err(err),
        };
        if let Err(err) = given {
            eprintln!("chora-tiling: {surface} stays: {}", err.message);
            self.wm.unplace(&surface);
        }
    }

    /// A move of a drag: a window that does not tile follows the pointer.
    /// True when it moved.
    #[allow(clippy::cast_possible_truncation, reason = "scene coordinates")]
    pub(super) fn drag_to(&mut self, binding: &str, [px, py]: [f64; 2]) -> bool {
        let Some((id, [dx, dy])) = self.dragged.clone().filter(|_| self.drags(binding)) else {
            return false;
        };
        let at = [(px - dx).round() as i32, (py - dy).round() as i32];
        let placed = self.wm.place(&id, at, [px, py]);
        if placed {
            self.bend(&id, at.map(f64::from), [px, py]);
        }
        placed
    }

    /// The window as the effects bend it: at an origin, with its size.
    pub(super) fn target(&self, id: &str, origin: [f64; 2]) -> Option<Target> {
        let window = self.wm.windows.iter().find(|w| w.id == id)?;
        let size = self.sent.get(id).and_then(|p| p.size).or(window.size)?;
        Some(Target {
            node: id.into(),
            origin,
            size: size.map(f64::from),
            set_size: false,
        })
    }

    /// A dragged window wobbles: it starts to at the first move, and each
    /// move bends its mesh.
    pub(super) fn bend(&mut self, id: &str, origin: [f64; 2], pointer: [f64; 2]) {
        let Some(target) = self.target(id, origin) else {
            return;
        };
        if !self.effects.wobbles(id) {
            let springs = self.springs.take_if(|(of, _)| of == id).map(|(_, s)| s);
            (self.effects).grab_given(id, &target, pointer, springs.as_ref());
        }
        let mesh = self.effects.motion(id, &target);
        self.meshes.extend(mesh);
        if self.effects.wobbles(id) {
            self.bent.insert(id.into(), target);
        }
    }
}
