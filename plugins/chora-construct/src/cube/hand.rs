//! The drag with the modifier on the front face at 1:1: the construct
//! holds the drag of the turn on the faces, so the hand turns the cube in
//! use mode as the drag of `chora-xp` turned its own cube, and the window manager
//! of the faces holds no such binding (`WM7b`).

use chora_kit::modes::Mode;

use super::{ClientError, Construct};

impl Construct {
    /// A press with the modifier at a point of the screen: on the front
    /// face of a cube at rest in use mode, the hand takes the cube.
    pub(crate) fn hand_press(&mut self, at: [f64; 2]) -> bool {
        let outside = self.inside().is_none();
        let Some(host) = self.cube.as_mut().filter(|c| c.view.mode() == Mode::Use) else {
            return false;
        };
        if !outside || !host.rests() {
            return false;
        }
        host.press(host.front, at);
        host.press.is_some()
    }

    /// The pointer moved while the hand holds the cube: it follows.
    pub(crate) fn hand_moved(&mut self, at: [f64; 2]) -> Result<bool, ClientError> {
        let width = self.lens.w;
        let Some(host) = self.cube.as_mut().filter(|c| c.press.is_some()) else {
            return Ok(false);
        };
        if let super::Dragged::Yaw(_) = host.drag(at, width, super::input_time(0)) {
            self.frame()?;
            self.sync_float()?;
        }
        Ok(true)
    }

    /// The hand lets go: the cube settles on the nearest face, or goes on
    /// after a flick.
    pub(crate) fn hand_released(&mut self) -> Result<bool, ClientError> {
        let Some(host) = self.cube.as_mut().filter(|c| c.press.is_some()) else {
            return Ok(false);
        };
        host.release(chora_kit::clock::now(), super::input_time(0));
        self.frame()?;
        Ok(true)
    }
}
