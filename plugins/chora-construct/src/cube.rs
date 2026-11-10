//! The cube (D69 step 10, D76): the construct adds virtual outputs,
//! starts one window manager on them as its child and shows each output
//! on a face of a cube in its 3D context. A face forwards the pointer
//! into its output, so the windows on it take clicks; it forwards
//! nothing while the cube turns, and hides while it looks away. The
//! cube steps back during a turn, so that its edge stays in view. When
//! the window manager ends, or does not connect in time, the outputs and
//! the cube go.

use std::{
    f64::consts::{FRAC_PI_2, SQRT_2},
    process::{Child, Stdio},
    time::{Duration, Instant},
};

use chora_kit::geometry::{Cube, Matrix, Turn};
use chora_scene::{Event, NodeKind, OutputInfo, OutputTexture};
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
/// How long a quarter turn takes, by default.
const TURN: Duration = Duration::from_millis(500);
/// How long the window manager may take to connect.
const CONNECT: Duration = Duration::from_secs(10);
/// The most faces: the side faces of a cube.
const FACES: usize = 4;

/// The cube that the construct hosts: its outputs in face order, the
/// process of the window manager, where it stands and how it is turned.
pub(crate) struct Host {
    outputs: Vec<String>,
    /// The window manager, and until it connects, the time it has.
    child: Child,
    connect_by: Option<Instant>,
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
fn placement([x, z]: [f64; 2], yaw: f64) -> Matrix {
    let (s, c) = yaw.sin_cos();
    [
        c, 0.0, -s, 0.0, 0.0, -1.0, 0.0, 0.0, s, 0.0, c, 0.0, x, 0.0, z, 1.0,
    ]
}

/// The face that is at the front after `by` quarter turns from `front`,
/// of `n` faces.
pub(crate) fn next_front(front: usize, by: i32, n: usize) -> usize {
    let n = i32::try_from(n).unwrap_or(1).max(1);
    let front = i32::try_from(front).unwrap_or(0).rem_euclid(n);
    #[allow(clippy::cast_sign_loss, reason = "rem_euclid of a positive n")]
    let next = (front + by.rem_euclid(n)).rem_euclid(n) as usize;
    next
}

/// How far the cube steps back at `angle`: nothing at rest, and at 45
/// degrees past a face enough that the edge, sqrt(2) times as near as a
/// face, stays as far as a face at rest, with a margin.
fn step_back(angle: f64, depth: f64) -> f64 {
    let off = (angle / FRAC_PI_2 - (angle / FRAC_PI_2).round()).abs() * FRAC_PI_2;
    (depth / 2.0 * (SQRT_2 - 1.0) * 1.1) * (2.0 * off).sin()
}

impl Host {
    /// Where the eye at `eye` on the ground is in the space of the cube.
    fn eye(&self, eye: [f64; 2]) -> [f64; 3] {
        let at = self.centre();
        let rel = [eye[0] - at[0], eye[1] - at[1]];
        let (s, c) = self.yaw.sin_cos();
        [rel[0] * c - rel[1] * s, 0.0, rel[0] * s + rel[1] * c]
    }

    /// The centre now: the place, stepped back along the view of the open.
    fn centre(&self) -> [f64; 2] {
        let back = step_back(self.angle, self.cube.size[0]);
        let (s, c) = self.yaw.sin_cos();
        [self.at[0] + s * back, self.at[1] + c * back]
    }

    /// The updates of the cube at the angle now: its place, the transform
    /// of each face, whether it shows, sent on a change, and whether it
    /// forwards, which it does only while the cube rests.
    pub(crate) fn faces(&mut self, eye: [f64; 2]) -> Vec<Command> {
        let eye = self.eye(eye);
        let mut commands = vec![update(
            CUBE,
            PropsUpdate {
                transform: Some(Transform::from_matrix(placement(self.centre(), self.yaw))),
                ..PropsUpdate::default()
            },
        )];
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
    /// Opens the cube: `faces` virtual outputs of the size of the view at
    /// its scale, one window manager on all of them, and a face for each,
    /// ahead of the camera, which levels, so that the front face shows its
    /// output at 1:1. Refused while another client `xp` runs.
    pub(crate) fn open_cube(&mut self, faces: usize) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        if self.cube.is_some() {
            return refuse("a cube is open; cube.close first".into());
        }
        if !(1..=FACES).contains(&faces) {
            return refuse(format!("faces: {faces} is not 1 to {FACES}"));
        }
        if self.manager_runs()? {
            return refuse(format!("a client {MANAGER} runs already"));
        }
        let size = [self.lens.w, self.lens.h];
        let outputs = self.add_outputs(faces, size)?;
        let flags: Vec<String> = outputs.iter().map(|o| format!("--output {o}")).collect();
        let command = format!(
            "exec {} {}",
            self.settings.value.cube_manager,
            flags.join(" ")
        );
        let spawned = std::process::Command::new("sh")
            .args(["-c", &command])
            .stdin(Stdio::null())
            .spawn();
        let child = match spawned {
            Ok(child) => child,
            Err(err) => {
                self.remove_outputs(&outputs);
                return refuse(format!("cannot start {command:?}: {err}"));
            }
        };
        self.glide = None;
        self.pose.pitch = 0.0;
        let cube = Cube { size, caps: false };
        let [fx, fz] = self.pose.forward();
        let reach = self.lens.focal + size[0] / 2.0;
        let mut host = Host {
            outputs: outputs.clone(),
            child,
            connect_by: Some(Instant::now() + CONNECT),
            cube,
            at: [self.pose.x + fx * reach, self.pose.z + fz * reach],
            yaw: self.pose.yaw,
            front: 0,
            angle: 0.0,
            turn: None,
            shown: vec![false; faces],
        };
        let mut commands = vec![create(CUBE, NodeKind::Group, PANES, PropsUpdate::default())];
        commands.extend(self.face_nodes(&outputs, &cube));
        commands.extend(host.faces([self.pose.x, self.pose.z]));
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            end(&mut host.child);
            self.remove_outputs(&outputs);
            return Err(ClientError::Rpc(err));
        }
        self.cube = Some(host);
        self.frame()?;
        Ok(json!({"outputs": outputs}))
    }

    /// Whether a client of the name of the window manager is connected.
    fn manager_runs(&mut self) -> Result<bool, ClientError> {
        let clients: Value = self.rt.conn.call("clients.list", Value::Null)?;
        let list = clients.as_array().cloned().unwrap_or_default();
        Ok(list.iter().any(|c| c["name"].as_str() == Some(MANAGER)))
    }

    /// The window manager of the cube connected, and no longer has to;
    /// or it went, and the cube goes with it.
    pub(crate) fn cube_client(&mut self, event: &Event) -> Result<(), ClientError> {
        if matches!(event, Event::ClientDisconnected { .. }) {
            return self.close_cube();
        }
        if let Some(host) = &mut self.cube {
            host.connect_by = None;
        }
        Ok(())
    }

    /// The time by which the window manager must connect.
    pub(crate) fn cube_deadline(&self) -> Option<Instant> {
        self.cube.as_ref().and_then(|c| c.connect_by)
    }

    /// Closes the cube when its window manager did not connect in time.
    pub(crate) fn check_cube(&mut self) -> Result<(), ClientError> {
        if self.cube_deadline().is_some_and(|t| t <= Instant::now()) {
            eprintln!("chora-construct: the window manager of the cube did not connect");
            return self.close_cube();
        }
        Ok(())
    }

    /// Turns the cube by `by` quarter turns in `ms`, a quarter turn in
    /// 500 ms by default; a turn while one runs is ignored.
    pub(crate) fn turn_cube(&mut self, by: i32, ms: Option<u64>) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Err(ClientError::Rpc(RpcError::new(
                RpcError::REJECTED,
                "no cube is open; cube.open first",
            )));
        };
        if host.turn.is_some() {
            return Ok(());
        }
        let front = next_front(host.front, by, host.outputs.len());
        host.front = front;
        let duration = ms.map_or(TURN, Duration::from_millis);
        host.turn = Some(Turn::new(
            host.angle,
            Cube::turn_of(front),
            Instant::now(),
            duration,
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

    /// Closes the cube: its window manager ends, and its outputs and the
    /// cube go.
    pub(crate) fn close_cube(&mut self) -> Result<(), ClientError> {
        let Some(mut host) = self.cube.take() else {
            return Ok(());
        };
        end(&mut host.child);
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

    /// Removes the virtual outputs that the construct added before it
    /// started again.
    pub(crate) fn remove_left_outputs(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.rt.conn.call("output.list", Value::Null)?;
        let left: Vec<String> = (outputs.into_iter())
            .filter(|o| o.is_virtual == Some(true) && o.owner.as_deref() == Some(super::NAME))
            .map(|o| o.name)
            .collect();
        self.remove_outputs(&left);
        Ok(())
    }

    /// Adds `faces` virtual outputs of `size` at the scale of the view;
    /// none when one fails.
    fn add_outputs(&mut self, faces: usize, size: [f64; 2]) -> Result<Vec<String>, ClientError> {
        let scale = self.output.as_ref().map_or(1.0, |o| o.scale);
        let spec = format!(
            "{}x{}@{scale}",
            (size[0] * scale).round(),
            (size[1] * scale).round()
        );
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

/// Ends a window manager of the cube and reaps it.
fn end(child: &mut Child) {
    if let Err(err) = child.kill() {
        eprintln!("chora-construct: end the window manager of the cube: {err}");
    }
    if let Err(err) = child.wait() {
        eprintln!("chora-construct: wait for the window manager of the cube: {err}");
    }
}
