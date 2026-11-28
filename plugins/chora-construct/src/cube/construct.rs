//! The construct's side of the cube: open it, look after it each frame,
//! build and update its nodes, and close it.

use std::{
    collections::BTreeSet,
    process::Child,
    time::{Duration, Instant},
};

use chora_kit::{geometry::Cube, modes::View, words::Language};
use chora_scene::{Border, Event, InputMode, NodeKind, OutputInfo, OutputTexture};
use serde_json::{Value, json};

use super::{
    CUBE, EDGE, EDGE_GLOW, EDGE_LIFT, EDGE_OUT, EDGE_WIDTH, Edge, FACES, Host, IDENTITY, LIT_GLOW,
    STAND, TURN, Tail, face_id, fill_id, launch, mirror_id, mul4,
};
use crate::{
    BTreeMap, ClientError, Command, Construct, PANES, PropsUpdate, RpcError, Transform, Uniform,
    create, faces::Health, update,
};

impl Construct {
    /// Opens the cube: `faces` virtual outputs of the size of the view at
    /// its scale, one window manager on all of them, and a face for each,
    /// ahead of the camera, which levels, so that the front face shows its
    /// output at 1:1. From inside a pane, or on the way into one, the
    /// camera first leaves it, to where it stood before; a running glide
    /// ends at its goal. Refused while another client `xp` runs.
    pub(crate) fn open_cube(&mut self, faces: usize) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        if self.cube.is_some() {
            return refuse("a cube is open; cube.close first".into());
        }
        if !(1..=FACES).contains(&faces) {
            return refuse(format!("faces: {faces} is not 1 to {FACES}"));
        }
        let client = self.face_manager();
        if self.has_manager(&client) || self.manager_runs()? {
            return refuse(format!("a client {client} runs already"));
        }
        if self.inside().is_some() {
            self.leave_window()?;
        }
        if let Some(glide) = self.glide.take() {
            self.pose = glide.to;
        }
        let size = [self.lens.w, self.lens.h];
        let outputs = self.add_outputs(faces, size)?;
        let flags: Vec<String> = outputs.iter().map(|o| format!("--output {o}")).collect();
        let manager = self.settings.value.cube_manager.clone();
        let command = format!("exec {manager} {}", flags.join(" "));
        let spawned = launch(&command);
        let (child, tail) = match spawned {
            Ok(started) => started,
            Err(err) => {
                self.remove_outputs(&outputs);
                return refuse(format!("cannot start {command:?}: {err}"));
            }
        };
        self.glide = None;
        self.pose.pitch = 0.0;
        let cube = Cube { size, caps: false };
        let mut host = self.host(
            outputs.clone(),
            Some((child, tail)),
            (&manager, &command),
            cube,
        );
        let mut commands = vec![create(CUBE, NodeKind::Group, PANES, PropsUpdate::default())];
        commands.extend(self.face_nodes(&outputs, &cube));
        commands.extend(self.edge_nodes());
        commands.extend(host.faces([self.pose.x, self.pose.z], self.lens.ground));
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            host.end_manager();
            self.remove_outputs(&outputs);
            return Err(ClientError::Rpc(err));
        }
        self.cube = Some(host);
        self.covered_focus_to_nobody()?;
        self.open_modes()?;
        self.frame()?;
        Ok(json!({"outputs": outputs}))
    }

    /// The host of a new cube ahead of the camera, in use mode.
    pub(super) fn host(
        &self,
        outputs: Vec<String>,
        started: Option<(Child, Tail)>,
        (manager, command): (&str, &str),
        cube: Cube,
    ) -> Host {
        let [fx, fz] = self.pose.forward();
        let reach = self.lens.focal + cube.size[0] / 2.0;
        let faces = outputs.len();
        let (child, tail) = started.unzip();
        let mut host = Host {
            shown: vec![false; faces],
            seen_from: [0.0; 3],
            floats: vec![None; faces],
            outputs,
            child,
            command: command.to_owned(),
            program: crate::names::program_of(manager).to_owned(),
            name: crate::names::cube_manager_name(manager),
            since: Instant::now(),
            tail: tail.unwrap_or_default(),
            health: vec![Health::Starting; faces],
            cube,
            at: [self.pose.x + fx * reach, self.pose.z + fz * reach],
            yaw: self.pose.yaw,
            front: 0,
            angle: 0.0,
            turn: None,
            view: View::default(),
            far: STAND * self.lens.focal,
            stand: 0.0,
            shift: 0.0,
            lean: 0.0,
            step: None,
            home: self.pose,
            focus: None,
            queued: false,
            switching: false,
            landing: None,
            hover: None,
            lit: None,
            press: None,
            live: None,
            step_back_key: None,
            owing: BTreeSet::new(),
            again: None,
            denied: BTreeSet::new(),
            told: None,
            put_later: None,
        };
        host.seen_from = host.eye([self.pose.x, self.pose.z]);
        host
    }

    /// The window manager of the cube connected, and no longer has to;
    /// or it went, and the cube goes with it.
    pub(crate) fn cube_client(&mut self, event: &Event) -> Result<(), ClientError> {
        if matches!(event, Event::ClientDisconnected { .. }) {
            return self.close_cube();
        }
        if let Some(host) = &mut self.cube {
            host.connected();
        }
        self.frame()
    }

    /// The time by which the construct looks again at a window manager.
    pub(crate) fn cube_deadline(&self) -> Option<Instant> {
        let cube = self
            .cube
            .as_ref()
            .and_then(|c| c.start_deadline(Instant::now()));
        let panes = self.manager_deadline();
        let put = (self.cube.as_ref()).and_then(|c| c.put_later.as_ref().map(|put| put.at));
        cube.into_iter().chain(panes).chain(put).min()
    }

    /// The faces fail when their window manager exited or did not connect
    /// in time; the cube stays, and the card says why.
    pub(crate) fn check_cube(&mut self) -> Result<(), ClientError> {
        self.check_managers()?;
        self.put_when_ready()?;
        let language = Language::of_session();
        let failed = (self.cube.as_mut()).and_then(|c| c.check_start(language, Instant::now()));
        if let Some(reason) = failed {
            crate::log::log_error!(
                "chora-construct: the window manager of the cube failed: {reason}"
            );
            self.frame()?;
        }
        Ok(())
    }

    /// Tries again: the window manager of the faces that failed starts
    /// anew.
    pub(crate) fn retry_cube(&mut self) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(());
        };
        if let Err(reason) = host.restart(Instant::now()) {
            crate::log::log_warn!("chora-construct: the window manager of the cube: {reason}");
            host.fail(&reason);
        }
        self.frame()
    }

    /// Turns the cube by `by` quarter turns in `ms`, a quarter turn in
    /// 360 ms by default. A turn while one runs is ignored; with `queue`,
    /// for the keys, the chips and the drag of a person, one such turn
    /// goes on from the running one. Answers whether the cube took the
    /// turn.
    pub(crate) fn turn_cube(
        &mut self,
        by: i32,
        ms: Option<u64>,
        queue: bool,
    ) -> Result<bool, ClientError> {
        let Some(host) = &mut self.cube else {
            return Err(crate::modes::no_cube());
        };
        let duration = ms.map_or(TURN, Duration::from_millis);
        let taken = host.start_turn(by, duration, queue, chora_kit::clock::now());
        if taken {
            self.draw_modes()?;
        }
        Ok(taken)
    }

    /// One step of the turn of the cube, then the frame.
    pub(crate) fn step_cube(&mut self) -> Result<(), ClientError> {
        let Some(host) = &mut self.cube else {
            return Ok(());
        };
        let now = chora_kit::clock::now();
        if let Some(turn) = host.turn {
            host.angle = turn.at(now);
            if turn.done(now) {
                host.turn = None;
                host.queued = false;
                host.view.turn_ended();
            }
        }
        if let Some(step) = host.step {
            host.stand = step.at(now);
            if step.done(now) {
                host.stand = host.stand.round();
                host.step = None;
            }
        }
        self.move_asides()?;
        self.frame()?;
        self.sync_float()
    }

    /// The updates of the faces of the cube for the pose now, and of the
    /// lit edge.
    pub(crate) fn cube_faces(&mut self) -> Vec<Command> {
        let eye = [self.pose.x, self.pose.z];
        let ground = self.lens.ground;
        let (lens, pose) = (self.lens, self.pose);
        let Some(host) = self.cube.as_mut() else {
            return Vec::new();
        };
        host.aim(&lens, &pose);
        let mut commands = host.faces(eye, ground);
        let plan = host.edge_plan();
        commands.extend(self.edge_commands(plan));
        commands
    }

    /// The effects of the mirror images of the faces for `look`.
    pub(crate) fn cube_mirror_looks(&self, look: &crate::panes::PaneLook) -> Vec<Command> {
        let Some(host) = &self.cube else {
            return Vec::new();
        };
        (host.outputs.iter().enumerate())
            .map(|(i, output)| {
                let win = Uniform::Output(OutputTexture {
                    output: output.clone(),
                });
                update(
                    &mirror_id(i),
                    PropsUpdate {
                        uniforms: Some(crate::panes::mirror_uniforms(
                            win,
                            look,
                            (crate::panes::CUBE_FEATHER, crate::panes::CUBE_FALL),
                        )),
                        ..PropsUpdate::default()
                    },
                )
            })
            .collect()
    }

    /// Whether the cube turns or steps and needs frames.
    pub(crate) fn cube_turns(&self) -> bool {
        (self.cube.as_ref()).is_some_and(|c| c.turn.is_some() || c.step.is_some() || c.waits())
    }

    /// Closes the cube: its window manager ends, and its outputs and the
    /// cube go.
    pub(crate) fn close_cube(&mut self) -> Result<(), ClientError> {
        let Some(mut host) = self.cube.take() else {
            return Ok(());
        };
        if let Some(crate::put::Put { surface, .. }) = host.put_later.take() {
            let language = Language::of_session();
            let why = crate::names::Unput::Closed.words(language);
            let told = crate::names::not_put(language, host.client(), Some(why));
            crate::log::log_info!("chora-construct: {surface} stays a pane: {told}");
            self.tell(&told);
        }
        self.clients.remove(host.client());
        host.end_manager();
        self.tell_end(&host);
        self.remove_outputs(&host.outputs);
        let removed = (0..host.outputs.len())
            .map(mirror_id)
            .chain([CUBE.to_owned()])
            .map(|id| Command::Remove { id, to: None })
            .collect();
        self.rt.conn.try_apply(removed)?;
        self.move_asides()?;
        self.close_modes()
    }

    /// The shader nodes of the faces, hidden until the first frame: each
    /// reads its output.
    pub(super) fn face_nodes(&self, outputs: &[String], cube: &Cube) -> Vec<Command> {
        let [r, g, b, _] = self.styled().glass.strong;
        let look = self.current_look();
        (outputs.iter().enumerate())
            .flat_map(|(i, output)| {
                let win = Uniform::Output(OutputTexture {
                    output: output.clone(),
                });
                [
                    create(
                        &mirror_id(i),
                        NodeKind::Shader,
                        crate::MIRRORS,
                        PropsUpdate {
                            size: Some(crate::panes::mirror_size(cube.size_of(i))),
                            shader: Some(self.mirror_shader.clone()),
                            uniforms: Some(crate::panes::mirror_uniforms(
                                win.clone(),
                                &look,
                                (crate::panes::CUBE_FEATHER, crate::panes::CUBE_FALL),
                            )),
                            visible: Some(false),
                            ..PropsUpdate::default()
                        },
                    ),
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
                    ),
                    create(
                        &fill_id(i),
                        NodeKind::Rect,
                        CUBE,
                        PropsUpdate {
                            size: Some(cube.size_of(i)),
                            color: Some([r, g, b, 1.0]),
                            visible: Some(false),
                            input: Some(InputMode::Pass),
                            ..PropsUpdate::default()
                        },
                    ),
                ]
            })
            .collect()
    }

    /// The glow and the lit edge of a face, hidden: the glow in the accent
    /// colour, and a border in it; both pass the pointer to the face
    /// behind them.
    pub(super) fn edge_nodes(&self) -> [Command; 2] {
        let [r, g, b] = self.settings.value.style.accent;
        let hidden = PropsUpdate {
            visible: Some(false),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        };
        [
            create(
                EDGE_GLOW,
                NodeKind::Shader,
                CUBE,
                PropsUpdate {
                    size: Some([1.0, 1.0]),
                    shader: Some(self.glow_shader.clone()),
                    ..hidden.clone()
                },
            ),
            create(
                EDGE,
                NodeKind::Rect,
                CUBE,
                PropsUpdate {
                    size: Some([1.0, 1.0]),
                    color: Some([0.0; 4]),
                    border: Some(Some(Border {
                        width: EDGE_WIDTH,
                        color: [r, g, b, 1.0],
                    })),
                    ..hidden
                },
            ),
        ]
    }

    /// The edge and the glow lit around face `face`, `scale` units of it
    /// to a pixel of the screen.
    fn lit_edge(&self, host: &Host, face: usize, scale: f64) -> Vec<Command> {
        let style = &self.settings.value.style;
        let [r, g, b] = style.accent;
        let size = host.cube.size_of(face);
        let angle = host.drawn_angle();
        let lit = PropsUpdate {
            visible: Some(true),
            opacity: Some(1.0),
            ..PropsUpdate::default()
        };
        let edge = PropsUpdate {
            transform: Some(Transform::from_matrix(host.lifted_at(
                face,
                angle,
                [0.0, 0.0],
                EDGE_LIFT,
            ))),
            size: Some(size),
            border: Some(Some(Border {
                width: EDGE_WIDTH * scale,
                color: [r, g, b, 1.0],
            })),
            ..lit.clone()
        };
        let reach = style.glow * scale;
        let k = crate::FOCUS_SCALE;
        let inner = [size[0] / k, size[1] / k];
        let mut uniforms = self
            .styled()
            .glow_uniforms(inner, 0.0, reach / k, [0.0; 4], 0.0);
        uniforms.insert("accent".into(), Uniform::Vector(vec![r, g, b, LIT_GLOW]));
        let mut grown = IDENTITY;
        (grown[0], grown[5], grown[12], grown[13]) = (k, k, -reach, -reach);
        let place = mul4(
            &host.lifted_at(face, angle, [0.0, 0.0], EDGE_LIFT / 2.0),
            &grown,
        );
        let glow = PropsUpdate {
            transform: Some(Transform::from_matrix(place)),
            size: Some([inner[0] + 2.0 * reach / k, inner[1] + 2.0 * reach / k]),
            uniforms: Some(uniforms),
            ..lit
        };
        vec![update(EDGE_GLOW, glow), update(EDGE, edge)]
    }

    /// What the lit edge and its glow do in this frame (`Host::edge_plan`).
    /// Lit, the edge is 2 px wide on the screen and the glow reaches 16 px
    /// out at 0.35, however far the cube stands.
    fn edge_commands(&self, plan: Edge) -> Vec<Command> {
        let Some(host) = &self.cube else {
            return Vec::new();
        };
        let shown = |id: &str, visible: bool| {
            update(
                id,
                PropsUpdate {
                    visible: Some(visible),
                    ..PropsUpdate::default()
                },
            )
        };
        match plan {
            Edge::Off => Vec::new(),
            Edge::Hide => vec![shown(EDGE_GLOW, false), shown(EDGE, false)],
            Edge::Fade => vec![
                crate::hud::fade(EDGE_GLOW, 0.0, EDGE_OUT),
                crate::hud::fade(EDGE, 0.0, EDGE_OUT),
            ],
            Edge::Lit { face, scale } => self.lit_edge(host, face, scale),
        }
    }

    /// Adds `faces` virtual outputs of `size` at the scale of the view, in
    /// one round trip ([`chora_kit::conn::Conn::call_all`]): under load each
    /// round trip waits for a frame of the core, and four in a row put
    /// `cube.open` past its 300 ms (docs/cube-modes.md). None when one
    /// fails: the others go again.
    pub(crate) fn add_outputs(
        &mut self,
        faces: usize,
        size: [f64; 2],
    ) -> Result<Vec<String>, ClientError> {
        let scale = self.output.as_ref().map_or(1.0, |o| o.scale);
        let spec = format!(
            "{}x{}@{scale}",
            (size[0] * scale).round(),
            (size[1] * scale).round()
        );
        let add = ("output.add", json!({"spec": spec, "virtual": true}));
        let answers = self.rt.conn.call_all(&vec![add; faces])?;
        let mut outputs = Vec::new();
        let mut failed = None;
        for answer in answers {
            let added = answer.map_err(ClientError::Rpc).and_then(|value| {
                serde_json::from_value::<OutputInfo>(value).map_err(ClientError::from)
            });
            match added {
                Ok(output) => outputs.push(output.name),
                Err(err) => failed = failed.or(Some(err)),
            }
        }
        if let Some(err) = failed {
            self.remove_outputs(&outputs);
            return Err(err);
        }
        Ok(outputs)
    }

    pub(crate) fn remove_outputs(&mut self, outputs: &[String]) {
        for output in outputs {
            let removed: Result<Value, _> =
                (self.rt.conn).call("output.remove", json!({"output": output}));
            if let Err(err) = removed {
                crate::log::log_warn!("chora-construct: output.remove {output}: {err}");
            }
        }
    }
}
