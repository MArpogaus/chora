//! The pull-out (E3): a window that another window manager gives with
//! its drag, as `chora-xp` does when a title bar leaves a face of the
//! cube, becomes a pane under the pointer and goes on with the drag and
//! with the wobble, now with a depth.

use chora_kit::effects::Target;
use chora_scene::GivenGrab;

use super::{ClientError, Construct, Event, Moving, PressState, give, space};

/// Where a point of a window shows on the output, for a pane with the
/// screen box `shown` (left, top, right, bottom) and the size of the
/// window. The point comes from another client: it is kept inside the
/// window, and a window without a size is held at its middle.
pub(crate) fn held_at(shown: [f64; 4], size: [f64; 2], point: [f64; 2]) -> [f64; 2] {
    let part = |axis: usize| {
        if size[axis] > 0.0 && point[axis].is_finite() {
            (point[axis] / size[axis]).clamp(0.0, 1.0)
        } else {
            0.5
        }
    };
    [
        shown[0] + (shown[2] - shown[0]) * part(0),
        shown[1] + (shown[3] - shown[1]) * part(1),
    ]
}

impl Construct {
    /// Takes the press of `move` that comes with a window and its drag;
    /// any other event goes back to the caller.
    pub(crate) fn pulled(&mut self, event: Event) -> Result<Option<Event>, ClientError> {
        match event {
            Event::InputButton {
                binding,
                state: PressState::Pressed,
                position,
                surface: Some(surface),
                grab: Some(grab),
                ..
            } if binding == give::MOVE => {
                let at = self.local(position);
                self.take_pulled(&surface, at, &grab).map(|()| None)
            }
            event => Ok(Some(event)),
        }
    }

    /// The press of `move` that comes with a window and its drag, at a
    /// point of the output. The window has its pane from the handover, or
    /// gets it now; a window without room for a pane stays hidden and is
    /// not lost. The pane moves so that the point of the window that the
    /// giver names is under the pointer, and the moves and the release of
    /// `move` then drag and drop it as a drag with the modifier does. A
    /// pane of an own window wobbles on in 3D, from the springs of the
    /// giver when it sent some.
    pub(crate) fn take_pulled(
        &mut self,
        surface: &str,
        at: [f64; 2],
        grab: &GivenGrab,
    ) -> Result<(), ClientError> {
        let of_window =
            |construct: &Self| (construct.panes.iter()).position(|p| p.window() == Some(surface));
        if of_window(self).is_none() {
            self.detached(surface)?;
        }
        let Some(pane) = of_window(self).map(|i| &self.panes[i]) else {
            return Ok(());
        };
        let shown = space::screen_box(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
        let Some(shown) = shown else {
            return Ok(());
        };
        let from = held_at(shown, pane.size, grab.point);
        self.moving = Some(Moving {
            name: pane.name.clone(),
            from,
            at: pane.at,
            yaw: pane.yaw,
        });
        if pane.attached {
            let target = Target {
                node: pane.node(),
                origin: [0.0; 2],
                size: pane.size,
                set_size: true,
            };
            let held = [0, 1].map(|axis| grab.point[axis].clamp(0.0, pane.size[axis].max(0.0)));
            let name = pane.name.clone();
            (self.effects).grab_3d(&name, &target, [0.0; 3], [held[0], held[1], 0.0], [0.0; 2]);
            if let Some(springs) = &grab.wobble {
                self.effects.take_springs(&name, [0.0; 2], springs);
            }
        }
        self.moved(at)
    }
}

#[cfg(test)]
mod tests {
    use super::held_at;

    /// The pointer holds the pulled window where the giver held it: the
    /// point of the window, scaled to the box that the pane shows in, and
    /// never outside the window.
    #[test]
    fn a_pulled_window_is_held_at_the_point_of_the_giver() {
        let shown = [100.0, 50.0, 500.0, 250.0];
        let size = [800.0, 400.0];
        assert_eq!(held_at(shown, size, [0.0, 0.0]), [100.0, 50.0]);
        assert_eq!(held_at(shown, size, [400.0, 100.0]), [300.0, 100.0]);
        assert_eq!(held_at(shown, size, [9000.0, -5.0]), [500.0, 50.0]);
        assert_eq!(held_at(shown, [0.0, 0.0], [3.0, 3.0]), [300.0, 150.0]);
        assert_eq!(held_at(shown, size, [f64::NAN, 100.0]), [300.0, 100.0]);
    }
}
