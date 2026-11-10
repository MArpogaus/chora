//! The cube (D69 step 9, D76): the construct adds virtual outputs, starts
//! one window manager on them and shows each output on a face of a cube
//! in its 3D context. A face forwards the pointer into its output, so
//! the windows on it take clicks; it forwards nothing while the cube
//! turns, and hides while it looks away. When the window manager ends,
//! the outputs and the cube go.

use std::time::{Duration, Instant};

use chora_kit::geometry::{Cube, Matrix, Turn};
use chora_scene::{NodeKind, OutputInfo, OutputTexture};
use serde_json::{Value, json};

use super::{
    BTreeMap, ClientError, Command, Construct, PANES, PropsUpdate, RpcError, Transform, Uniform,
    create, update,
};

/// The group of the cube in the camera group, and the prefix of a face.
const CUBE: &str = "construct-cube";
const FACE: &str = "construct-cube-face-";
/// The client name of the window manager of the cube.
pub(crate) const MANAGER: &str = "xp";
/// How long a quarter turn takes.
const TURN: Duration = Duration::from_millis(500);
/// The most faces: the side faces of a cube.
const FACES: usize = 4;

/// The cube that the construct hosts: its outputs in face order, the
/// process of the window manager, where it stands and how it is turned.
pub(crate) struct Host {
    outputs: Vec<String>,
    cube: Cube,
    /// The centre on the ground, (x, z), and the yaw of the cube: face 0
    /// looks back along it.
    at: [f64; 2],
    yaw: f64,
    /// The face at the front, the angle and the running turn.
    front: usize,
    angle: f64,
    turn: Option<Turn>,
    /// Whether each face shows, as last sent.
    shown: Vec<bool>,
}

fn face_id(i: usize) -> String {
    format!("{FACE}{i}")
}

/// The transform of the cube group in the camera group: the space of the
/// cube, with y down, at `at` on the ground, turned by `yaw`.
fn placement(at: [f64; 2], yaw: f64) -> Matrix {
    let (s, c) = yaw.sin_cos();
    [
        c, 0.0, -s, 0.0, 0.0, -1.0, 0.0, 0.0, s, 0.0, c, 0.0, at[0], 0.0, at[1], 1.0,
    ]
}

impl Host {
    /// Where the eye at `eye` on the ground is in the space of the cube.
    fn eye(&self, eye: [f64; 2]) -> [f64; 3] {
        let rel = [eye[0] - self.at[0], eye[1] - self.at[1]];
        let (s, c) = self.yaw.sin_cos();
        [rel[0] * c - rel[1] * s, 0.0, rel[0] * s + rel[1] * c]
    }

    /// The updates of the faces at the angle now: the transform of each,
    /// whether it shows, sent on a change, and whether it forwards, which
    /// it does only while the cube rests.
    pub(crate) fn faces(&mut self, eye: [f64; 2]) -> Vec<Command> {
        let eye = self.eye(eye);
        let mut commands = Vec::new();
        for i in 0..self.outputs.len() {
            let shows = self.cube.faces_eye(i, self.angle, eye);
            let changed = self.shown.get(i) != Some(&shows);
            if let Some(slot) = self.shown.get_mut(i) {
                *slot = shows;
            }
            commands.push(update(
                &face_id(i),
                PropsUpdate {
                    transform: Some(Transform::from_matrix(self.cube.face(i, self.angle))),
                    visible: changed.then_some(shows),
                    forward: Some(self.turn.is_none()),
                    ..PropsUpdate::default()
                },
            ));
        }
        commands
    }
}

impl Construct {
    /// Opens the cube: `faces` virtual outputs of the size of the view, one
    /// window manager on all of them, and a face for each, ahead of the
    /// camera so that the front face shows its output at 1:1.
    pub(crate) fn open_cube(&mut self, faces: usize) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        if self.cube.is_some() {
            return refuse("a cube is open; it goes when its window manager ends".into());
        }
        if !(1..=FACES).contains(&faces) {
            return refuse(format!("faces: {faces} is not 1 to {FACES}"));
        }
        let size = [self.lens.w, self.lens.h];
        let outputs = self.add_outputs(faces, size)?;
        let flags: Vec<String> = outputs.iter().map(|o| format!("--output {o}")).collect();
        let command = format!("{} {}", self.settings.value.cube_manager, flags.join(" "));
        let mint = self.rt.conn.activation_token();
        if chora_kit::launch(&command, mint.as_deref()).is_none() {
            self.remove_outputs(&outputs);
            return refuse(format!("cannot start {command:?}"));
        }
        let cube = Cube { size, caps: false };
        let [fx, fz] = self.pose.forward();
        let reach = self.lens.focal + size[0] / 2.0;
        let mut host = Host {
            outputs: outputs.clone(),
            cube,
            at: [self.pose.x + fx * reach, self.pose.z + fz * reach],
            yaw: self.pose.yaw,
            front: 0,
            angle: 0.0,
            turn: None,
            shown: vec![false; faces],
        };
        let mut commands = vec![create(
            CUBE,
            NodeKind::Group,
            PANES,
            PropsUpdate {
                transform: Some(Transform::from_matrix(placement(host.at, host.yaw))),
                ..PropsUpdate::default()
            },
        )];
        commands.extend(self.face_nodes(&outputs, &cube));
        commands.extend(host.faces([self.pose.x, self.pose.z]));
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            self.remove_outputs(&outputs);
            return Err(ClientError::Rpc(err));
        }
        self.cube = Some(host);
        Ok(json!({"outputs": outputs}))
    }

    /// Turns the cube by `by` quarter turns.
    pub(crate) fn turn_cube(&mut self, by: i32) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Err(ClientError::Rpc(RpcError::new(
                RpcError::REJECTED,
                "no cube is open; cube.open first",
            )));
        };
        let n = i32::try_from(host.outputs.len()).unwrap_or(1).max(1);
        #[allow(clippy::cast_sign_loss, reason = "rem_euclid of a positive n")]
        let front = (i32::try_from(host.front).unwrap_or(0) + by).rem_euclid(n) as usize;
        host.front = front;
        host.turn = Some(Turn::new(
            host.angle,
            Cube::turn_of(front),
            Instant::now(),
            TURN,
        ));
        Ok(())
    }

    /// One step of the turn of the cube, then the frame.
    pub(crate) fn step_cube(&mut self) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(());
        };
        let now = Instant::now();
        if let Some(turn) = host.turn {
            host.angle = turn.at(now);
            if turn.done(now) {
                host.turn = None;
            }
        }
        self.frame()
    }

    /// The updates of the faces of the cube for the pose now.
    pub(crate) fn cube_faces(&mut self) -> Vec<Command> {
        let eye = [self.pose.x, self.pose.z];
        self.cube.as_mut().map(|h| h.faces(eye)).unwrap_or_default()
    }

    /// Whether the cube turns and needs frames.
    pub(crate) fn cube_turns(&self) -> bool {
        self.cube.as_ref().is_some_and(|c| c.turn.is_some())
    }

    /// The window manager of the cube ended: its outputs and the cube go.
    pub(crate) fn close_cube(&mut self) -> Result<(), ClientError> {
        let Some(host) = self.cube.take() else {
            return Ok(());
        };
        self.remove_outputs(&host.outputs);
        self.rt.conn.try_apply(vec![Command::Remove {
            id: CUBE.into(),
            to: None,
        }])?;
        Ok(())
    }

    /// The shader nodes of the faces, hidden until the first frame: each
    /// reads its output.
    fn face_nodes(&self, outputs: &[String], cube: &Cube) -> Vec<Command> {
        (outputs.iter().enumerate())
            .map(|(i, output)| {
                let win = Uniform::Output(OutputTexture {
                    output: output.clone(),
                });
                create(
                    &face_id(i),
                    NodeKind::Shader,
                    CUBE,
                    PropsUpdate {
                        size: Some(cube.size_of(i)),
                        shader: Some(self.source_shader.clone()),
                        uniforms: Some(BTreeMap::from([("win".into(), win)])),
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                )
            })
            .collect()
    }

    /// Adds `faces` virtual outputs of `size`; none when one fails.
    fn add_outputs(&mut self, faces: usize, size: [f64; 2]) -> Result<Vec<String>, ClientError> {
        let spec = format!("{}x{}", size[0].round(), size[1].round());
        let mut outputs = Vec::new();
        for _ in 0..faces {
            let added: Result<OutputInfo, _> =
                (self.rt.conn).call("output.add", json!({"spec": spec, "virtual": true}));
            match added {
                Ok(output) => outputs.push(output.name),
                Err(err) => {
                    self.remove_outputs(&outputs);
                    return Err(err);
                }
            }
        }
        Ok(outputs)
    }

    fn remove_outputs(&mut self, outputs: &[String]) {
        for output in outputs {
            let removed: Result<Value, _> =
                (self.rt.conn).call("output.remove", json!({"output": output}));
            if let Err(err) = removed {
                eprintln!("chora-construct: output.remove {output}: {err}");
            }
        }
    }
}
