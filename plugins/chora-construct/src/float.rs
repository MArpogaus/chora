//! Floating windows on the cube (`chora_kit::float`): while the cube
//! stands back or turns, each window of the window manager on a face
//! floats in front of the face on a plane of its own, the higher ones of
//! the stack farther, as on the desktop cube of xp. The window manager
//! hides the windows in the output for that time (`windows.float`), and
//! each plane reads the node of a window as a node texture. At rest in
//! use mode the planes lie in the face, the windows show in the output
//! again, and then the planes go: the face is flat, with exact pixels
//! and input.

use std::time::{Duration, Instant};

use chora_kit::float::{self, Float, Floated, Floating};
use chora_scene::{InputMode, NodeTexture};
use serde_json::json;

use super::{
    BTreeMap, ClientError, Command, Construct, NodeKind, PropsUpdate, Uniform, create, cube,
};

/// How long the planes stay after the windows show in the output again:
/// the output draws them first, so that no frame lacks a window.
const LAND: Duration = Duration::from_millis(120);

/// The id of the plane of a window on face `face`.
pub(crate) fn plane_id(face: usize, surface: &str) -> String {
    format!("{}float-{face}-{surface}", cube::CUBE_PREFIX)
}

impl Construct {
    /// Asks the window manager that the windows of an output float, or
    /// land. Answers them; none when the window manager has no such
    /// method, which is logged.
    fn ask_float(&mut self, output: &str, on: bool) -> Vec<Floating> {
        let float = Float {
            output: output.to_owned(),
            on,
        };
        let call = json!({"client": cube::MANAGER, "method": float::METHOD, "params": float});
        match self.rt.conn.call::<_, Floated>("methods.call", call) {
            Ok(floated) => floated.windows,
            Err(err) => {
                eprintln!("chora-construct: {} {output}: {err}", float::METHOD);
                Vec::new()
            }
        }
    }

    /// Lifts the windows off the faces when the cube leaves its rest in
    /// use mode, and lets them land when it rests there again. Called
    /// with each step of the cube.
    pub(crate) fn sync_float(&mut self) -> Result<(), ClientError> {
        let Some(host) = &self.cube else {
            return Ok(());
        };
        let (wanted, floating, landing) = (host.wants_float(), host.floating(), host.landing());
        let now = Instant::now();
        if wanted && (!floating || landing.is_some()) {
            return self.lift_windows();
        }
        if !wanted && floating && landing.is_none() {
            for output in host.outputs().to_vec() {
                self.ask_float(&output, false);
            }
            if let Some(host) = &mut self.cube {
                host.land(Some(now + LAND));
            }
        } else if landing.is_some_and(|at| at <= now) {
            let planes = self.cube.as_mut().map(cube::Host::landed);
            self.remove_planes(planes.unwrap_or_default())?;
        }
        Ok(())
    }

    /// The windows of every face float from now on, each on its plane.
    fn lift_windows(&mut self) -> Result<(), ClientError> {
        let outputs = self.cube.as_ref().map(|c| c.outputs().to_vec());
        for (face, output) in outputs.unwrap_or_default().iter().enumerate() {
            let windows = self.ask_float(output, true);
            self.set_planes(face, windows)?;
        }
        if let Some(host) = &mut self.cube {
            host.land(None);
        }
        Ok(())
    }

    /// The floating windows of an output changed, by the event of the
    /// window manager: its planes follow.
    pub(crate) fn floated(&mut self, floated: Floated) -> Result<(), ClientError> {
        let face = (self.cube.as_ref())
            .filter(|host| host.floating() && host.landing().is_none())
            .and_then(|host| host.outputs().iter().position(|o| *o == floated.output));
        match face {
            Some(face) => {
                self.set_planes(face, floated.windows)?;
                self.frame()
            }
            None => Ok(()),
        }
    }

    /// Makes the planes of a face anew for its floating windows.
    fn set_planes(&mut self, face: usize, windows: Vec<Floating>) -> Result<(), ClientError> {
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
                    input: Some(InputMode::Pass),
                    ..PropsUpdate::default()
                },
            )
        });
        let commands: Vec<Command> = planes.collect();
        let Some(host) = &mut self.cube else {
            return Ok(());
        };
        let old = host.set_floats(face, windows);
        self.remove_planes(old)?;
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            eprintln!("chora-construct: the planes of face {face}: {err}");
        }
        Ok(())
    }

    fn remove_planes(&mut self, planes: Vec<String>) -> Result<(), ClientError> {
        for id in planes {
            self.rt
                .conn
                .try_apply(vec![Command::Remove { id, to: None }])?;
        }
        Ok(())
    }
}
