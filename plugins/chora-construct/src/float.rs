//! Floating windows on the cube (`chora_kit::float`): while the cube
//! stands back or turns, each window of the window manager on a face
//! that shows floats in front of the face on a plane of its own, the
//! higher ones of the stack farther, as on the desktop cube of xp. The
//! window manager hides the windows in the output for that time
//! (`windows.float`), and each plane reads the node of a window as a
//! node texture. At rest in use mode the planes lie in the face, the
//! windows show in the output again, and then the planes go: the face
//! is flat, with exact pixels and input.
//!
//! A window must never show nowhere. The planes of a face go only after
//! the window manager answered that its windows show in the output
//! again; while it does not answer, the planes stay and the construct
//! asks again. A face whose planes cannot be made, or get no texture,
//! lands at once and stays flat until the cube rests.

use std::time::{Duration, Instant};

use chora_kit::float::{self, Float, Floated, Floating};
use chora_scene::NodeTexture;
use serde_json::json;

use super::{
    BTreeMap, ClientError, Command, Construct, NodeKind, PropsUpdate, Uniform, create, cube,
};

/// How long the planes stay after the windows show in the output again:
/// the output draws them first, so that no frame lacks a window.
const LAND: Duration = Duration::from_millis(120);
/// How long the window manager has for an answer, and when the
/// construct asks again after none.
pub(crate) const ANSWER: Duration = Duration::from_millis(400);
const AGAIN: Duration = Duration::from_millis(500);

/// The id of the plane of a window on face `face`.
pub(crate) fn plane_id(face: usize, surface: &str) -> String {
    format!("{}float-{face}-{surface}", cube::CUBE_PREFIX)
}

/// The window of the plane of a window on face `face`.
pub(crate) fn plane_window(face: usize, node: &str) -> Option<&str> {
    let prefix = format!("{}float-{face}-", cube::CUBE_PREFIX);
    node.strip_prefix(&prefix)
}

/// The face of the plane of a window.
pub(crate) fn plane_face(node: &str) -> Option<usize> {
    let rest = node
        .strip_prefix(cube::CUBE_PREFIX)?
        .strip_prefix("float-")?;
    rest.split('-').next()?.parse().ok()
}

/// The windows of an answer or an event that the construct shows: each
/// names the group of its surface, as the window manager makes it, and
/// lies with a size on or near its face of `size`; the top `MAX`.
fn valid(windows: Vec<Floating>, size: [f64; 2]) -> Vec<Floating> {
    let near = |v: f64, side: f64| v.is_finite() && (-side..=2.0 * side).contains(&v);
    let fits = |v: f64, side: f64| v.is_finite() && v >= 1.0 && v <= 2.0 * side;
    let sound = |w: &Floating| {
        let [x, y, width, height] = w.rect;
        w.node == format!("{}-win-{}", cube::MANAGER, w.surface)
            && near(x, size[0])
            && near(y, size[1])
            && fits(width, size[0])
            && fits(height, size[1])
    };
    float::top(windows.into_iter().filter(sound).collect())
}

impl Construct {
    /// Asks the window manager that the windows of an output float, or
    /// land, and waits a short time for the answer.
    fn ask_float(&mut self, output: &str, on: bool) -> Result<Vec<Floating>, ClientError> {
        let float = Float {
            output: output.to_owned(),
            on,
        };
        let call = json!({"client": cube::MANAGER, "method": float::METHOD, "params": float});
        let floated: Floated = (self.rt.conn).call_within("methods.call", call, ANSWER)?;
        Ok(floated.windows)
    }

    /// Asks as [`Self::ask_float`] for a face. Without an answer the face
    /// owes one: the construct asks again. Answers the windows.
    fn ask_face(&mut self, face: usize, on: bool) -> Option<Vec<Floating>> {
        let output = self.cube.as_ref()?.outputs().get(face)?.clone();
        let size = [self.lens.w, self.lens.h];
        let answer = self.ask_float(&output, on);
        let host = self.cube.as_mut()?;
        match answer {
            Ok(windows) => {
                host.owes(face, false);
                Some(valid(windows, size))
            }
            Err(err) => {
                if host.owes(face, true) {
                    eprintln!("chora-construct: {} {output}: {err}", float::METHOD);
                }
                None
            }
        }
    }

    /// Lifts the windows off the faces that show when the cube leaves its
    /// rest in use mode, and lets them land when it rests there again.
    /// Called after each frame of a cube that moves, and again a while
    /// after the window manager did not answer. The first face that gets
    /// no answer ends the round, and the next round starts a while after
    /// that wait: a deaf window manager blocks the construct for one wait
    /// in a round, and the construct handles its events between two.
    pub(crate) fn sync_float(&mut self) -> Result<(), ClientError> {
        let now = Instant::now();
        let Some(host) = &self.cube else {
            return Ok(());
        };
        if host.asks_again().is_some_and(|at| at > now) {
            return Ok(());
        }
        let (faces, wanted) = (host.outputs().len(), host.wants_float());
        let mut answered = true;
        for face in 0..faces {
            answered &= self.sync_face(face, wanted)?;
            if !answered {
                break;
            }
        }
        if answered && !wanted {
            answered &= self.land(now)?;
        }
        if let Some(host) = &mut self.cube {
            host.ask_again((!answered).then(|| Instant::now() + AGAIN));
        }
        Ok(())
    }

    /// One face: its windows float when it shows while the cube floats,
    /// and land at once when it looks away. Answers whether the window
    /// manager answered what was asked.
    fn sync_face(&mut self, face: usize, wanted: bool) -> Result<bool, ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(true);
        };
        let (wants, floats) = (host.face_floats(face), host.floats_of(face).is_some());
        if wants {
            host.land_at(None);
            return self.lift_face(face);
        }
        if floats && wanted {
            return self.drop_face(face);
        }
        if !floats && host.owed(face) {
            return Ok(self.ask_face(face, false).is_some());
        }
        Ok(true)
    }

    /// The windows of a face float, each on its plane. A face whose
    /// planes the core refuses lands and stays flat.
    fn lift_face(&mut self, face: usize) -> Result<bool, ClientError> {
        let Some(windows) = self.ask_face(face, true) else {
            return Ok(false);
        };
        match self.set_planes(face, windows)? {
            None => Ok(true),
            Some(err) => {
                eprintln!("chora-construct: the planes of face {face}: {err}");
                self.flatten(face)
            }
        }
    }

    /// The windows of a face show in its output again and its planes go
    /// at once: the face looks away.
    fn drop_face(&mut self, face: usize) -> Result<bool, ClientError> {
        let answered = self.ask_face(face, false).is_some();
        let planes = self.cube.as_mut().map(|host| host.set_floats(face, None));
        self.remove_planes(planes.unwrap_or_default())?;
        Ok(answered)
    }

    /// A face lands at once and floats no more until the cube rests.
    fn flatten(&mut self, face: usize) -> Result<bool, ClientError> {
        if let Some(host) = &mut self.cube {
            host.deny(face);
        }
        self.drop_face(face)
    }

    /// The cube rests in use mode: the windows show in their outputs
    /// again, and a while after every answer came the planes go.
    fn land(&mut self, now: Instant) -> Result<bool, ClientError> {
        let Some(host) = &self.cube else {
            return Ok(true);
        };
        let floating = host.floating();
        match host.landing() {
            Some(at) if at <= now => {
                let planes = self.cube.as_mut().map(cube::Host::landed);
                self.remove_planes(planes.unwrap_or_default())?;
                Ok(true)
            }
            Some(_) => Ok(true),
            None if floating.is_empty() => Ok(true),
            None => {
                let mut faces = floating.into_iter();
                let answered = faces.all(|face| self.ask_face(face, false).is_some());
                if let Some(host) = self.cube.as_mut().filter(|_| answered) {
                    host.land_at(Some(now + LAND));
                }
                Ok(answered)
            }
        }
    }

    /// The floating windows of an output changed, by the event of the
    /// window manager: its planes follow.
    pub(crate) fn floated(&mut self, floated: Floated) -> Result<(), ClientError> {
        let size = [self.lens.w, self.lens.h];
        let face = (self.cube.as_ref())
            .filter(|host| host.landing().is_none())
            .and_then(|host| host.outputs().iter().position(|o| *o == floated.output))
            .filter(|face| (self.cube.as_ref()).is_some_and(|h| h.floats_of(*face).is_some()));
        let Some(face) = face else {
            return Ok(());
        };
        if let Some(err) = self.set_planes(face, valid(floated.windows, size))? {
            eprintln!("chora-construct: the planes of face {face}: {err}");
            self.flatten(face)?;
        }
        Ok(())
    }

    /// Planes that got no texture, as the layers that the pixel budget of
    /// a frame leaves out: their faces land and stay flat.
    pub(crate) fn float_dropped(&mut self, nodes: &[String]) -> Result<(), ClientError> {
        let mut faces: Vec<usize> = nodes.iter().filter_map(|n| plane_face(n)).collect();
        faces.sort_unstable();
        faces.dedup();
        for face in faces {
            self.flatten(face)?;
        }
        Ok(())
    }

    /// Makes the planes of a face for its floating windows in one
    /// transaction: the old ones go, the new ones come at their places.
    /// Nothing happens when the windows are the same. Answers the error
    /// of the core.
    fn set_planes(
        &mut self,
        face: usize,
        windows: Vec<Floating>,
    ) -> Result<Option<chora_scene::ipc::RpcError>, ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(None);
        };
        if host.floats_of(face) == Some(windows.as_slice()) {
            return Ok(None);
        }
        let planes = windows.iter().map(|window| {
            let win = Uniform::Texture(NodeTexture {
                node: window.node.clone(),
            });
            create(
                &plane_id(face, &window.surface),
                NodeKind::Shader,
                cube::CUBE,
                PropsUpdate {
                    size: Some([window.rect[2], window.rect[3]]),
                    shader: Some(self.source_shader.clone()),
                    uniforms: Some(BTreeMap::from([("win".into(), win)])),
                    visible: Some(false),
                    ..PropsUpdate::default()
                },
            )
        });
        let planes: Vec<Command> = planes.collect();
        let old = host.set_floats(face, Some(windows));
        let mut commands: Vec<Command> = (old.into_iter())
            .map(|id| Command::Remove { id, to: None })
            .collect();
        commands.extend(planes);
        commands.extend(host.plane_updates(face));
        self.rt.conn.try_apply(commands)
    }

    fn remove_planes(&mut self, planes: Vec<String>) -> Result<(), ClientError> {
        let commands: Vec<Command> = (planes.into_iter())
            .map(|id| Command::Remove { id, to: None })
            .collect();
        if commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.try_apply(commands).map(drop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(surface: &str, rect: [f64; 4]) -> Floating {
        Floating {
            node: format!("xp-win-{surface}"),
            surface: surface.into(),
            rect,
        }
    }

    #[test]
    fn only_sound_windows_get_a_plane() {
        let size = [1280.0, 800.0];
        let good = window("s1", [10.0, 20.0, 300.0, 200.0]);
        let mut foreign = good.clone();
        foreign.node = "construct-look".into();
        let far = window("s2", [9000.0, 0.0, 300.0, 200.0]);
        let huge = window("s3", [0.0, 0.0, 30000.0, 200.0]);
        let empty = window("s4", [0.0, 0.0, 0.0, 200.0]);
        let broken = window("s5", [f64::NAN, 0.0, 10.0, 10.0]);
        let all = vec![good.clone(), foreign, far, huge, empty, broken];
        assert_eq!(valid(all, size), vec![good]);
    }

    #[test]
    fn at_most_the_top_windows_get_a_plane() {
        let many: Vec<Floating> = (0..float::MAX + 4)
            .map(|i| window(&format!("s{i}"), [0.0, 0.0, 10.0, 10.0]))
            .collect();
        let shown = valid(many, [1280.0, 800.0]);
        assert_eq!(shown.len(), float::MAX);
        assert_eq!(shown[0].surface, "s4");
    }

    #[test]
    fn a_plane_names_its_face() {
        assert_eq!(plane_face(&plane_id(2, "s7")), Some(2));
        assert_eq!(plane_window(2, &plane_id(2, "s7")), Some("s7"));
        assert_eq!(plane_face("construct-cube-face-1"), None);
    }
}
