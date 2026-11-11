//! Panes and their scene nodes: building the space, the pane nodes,
//! uniforms, and moving, resizing and removing a pane.

use chora_scene::ShaderEffect;

use super::{
    BTreeMap, ClientError, Command, Construct, Content, FOCUS, FOG, FOG_SCALE, FRONT, InputMode,
    Kind, LOOK, Lens, MIRROR_SCALE, MIRRORS, NAME, NodeKind, PANE_MAX, PANE_MIN, PANES, POSE, Pane,
    PaneFile, PropsUpdate, RpcError, SKY, SPACE, ToplevelState, Transform, Uniform, WINDOWS,
    overlay_id, space, unique,
};

/// What the effects of the panes and their mirror images show: the
/// focused pane, the fog, the accent, the gloss of the floor, the focal
/// length over the leash radius for the haze, and the height of the eye
/// over the focal length for the fresnel of the floor.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PaneLook {
    pub focus: Option<String>,
    pub fog: [f64; 3],
    pub accent: [f64; 3],
    pub gloss: f64,
    pub reach: f64,
    pub ground: f64,
}

pub(crate) fn create(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

pub(crate) fn update(id: &str, props: PropsUpdate) -> Command {
    Command::Update {
        id: id.into(),
        props,
    }
}

pub(crate) fn vec3(c: [f64; 3]) -> Uniform {
    Uniform::Vector(c.to_vec())
}

/// The nodes of the space on the screen root of an output, bottom
/// first: the hidden windows, the sky, the fog, the look rect, which
/// takes the presses on the empty space and the keys of the walk, the 3D
/// context with the camera group, the mirror images and the panes, and
/// the group of the window that the camera is in.
pub(crate) fn space_nodes(fog_shader: &str, root: &str) -> Vec<Command> {
    let group = |input| PropsUpdate {
        input,
        ..PropsUpdate::default()
    };
    vec![
        create(NAME, NodeKind::Group, root, group(None)),
        create(WINDOWS, NodeKind::Group, NAME, group(None)),
        create(SKY, NodeKind::Rect, NAME, PropsUpdate::default()),
        create(
            FOG,
            NodeKind::Shader,
            NAME,
            PropsUpdate {
                shader: Some(fog_shader.into()),
                ..PropsUpdate::default()
            },
        ),
        create(
            LOOK,
            NodeKind::Rect,
            NAME,
            PropsUpdate {
                color: Some([0.0; 4]),
                ..PropsUpdate::default()
            },
        ),
        create(SPACE, NodeKind::Group, NAME, group(None)),
        create(POSE, NodeKind::Group, SPACE, group(None)),
        create(MIRRORS, NodeKind::Group, POSE, group(Some(InputMode::Pass))),
        create(PANES, NodeKind::Group, POSE, group(None)),
        create(FRONT, NodeKind::Group, NAME, group(None)),
        Command::FocusNode {
            node: Some(LOOK.into()),
        },
    ]
}

/// The last value of a shadow uniform of fog.frag: the lift in whole
/// pixels, and the yaw in the fraction.
/// ponytail: two values in one float, as the core passes at most 16
/// uniforms; about 4e-4 rad of yaw at a lift of 1000 px, with highp.
pub(crate) fn shadow_w(lift: f64, yaw: f64) -> f64 {
    let turn = (yaw / std::f64::consts::TAU + 0.5).rem_euclid(1.0);
    lift.floor() + turn * 0.99
}

/// The effect of a pane: pane.frag over its layer, with the fog, the
/// accent and whether the pane has the focus, and the haze distance.
pub(crate) fn pane_effect(shader: &str, look: &PaneLook, focused: bool) -> ShaderEffect {
    let [r, g, b] = look.accent;
    ShaderEffect {
        shader: shader.into(),
        uniforms: BTreeMap::from([
            ("fog".into(), vec3(look.fog)),
            (
                "edge".into(),
                Uniform::Vector(vec![r, g, b, if focused { 1.0 } else { 0.0 }]),
            ),
            ("reach".into(), Uniform::Float(look.reach)),
        ]),
    }
}

/// The pane of a window that another window manager hands over (D55):
/// at the size of the window, clamped to the view, else the view.
pub(crate) fn handed_over(size: Option<[f64; 2]>, lens: &space::Lens) -> PaneFile {
    let view = [lens.w, lens.h];
    let size = size.map_or(view, |s| {
        [0, 1].map(|i| s[i].clamp(PANE_MIN.min(view[i]), view[i]))
    });
    PaneFile {
        kind: Kind::Program,
        command: None,
        plugin: None,
        scene: None,
        manager: None,
        at: None,
        yaw: None,
        size: Some(size),
    }
}

/// The uniforms of a mirror image: the pane's node, the gloss of the
/// floor and the distances of the haze and the fresnel.
pub(crate) fn mirror_uniforms(win: Uniform, look: &PaneLook) -> BTreeMap<String, Uniform> {
    BTreeMap::from([
        ("win".into(), win),
        ("gloss".into(), Uniform::Float(look.gloss)),
        ("reach".into(), Uniform::Float(look.reach)),
        ("ground".into(), Uniform::Float(look.ground)),
    ])
}

pub(crate) fn mirror_size(size: [f64; 2]) -> [f64; 2] {
    size.map(|s| (s / MIRROR_SCALE).ceil())
}

pub(crate) fn configure(surface: &str, size: [f64; 2]) -> Command {
    #[allow(clippy::cast_possible_truncation, reason = "clamped to PANE_MAX")]
    let size = size.map(|s| s.round().clamp(1.0, PANE_MAX) as i32);
    Command::Configure {
        surface: surface.into(),
        size: Some(size),
        states: vec![ToplevelState::Maximized],
    }
}

impl Construct {
    /// The index of the pane with the label `name`.
    pub(crate) fn pane_named(&self, name: &str) -> Result<usize, RpcError> {
        (self.panes.iter().position(|p| p.name == name)).ok_or_else(|| {
            RpcError::new(
                RpcError::INVALID_PARAMS,
                format!("no pane {name:?}; construct.list names them"),
            )
        })
    }

    /// Moves a pane to `to`, inside the leash (C4).
    pub(crate) fn move_pane(&mut self, i: usize, to: [f64; 2]) -> Result<(), ClientError> {
        let len = to[0].hypot(to[1]);
        let r = self.radius();
        let to = if len > r {
            [to[0] * r / len, to[1] * r / len]
        } else {
            to
        };
        self.panes[i].put(to, [self.pose.x, self.pose.z]);
        self.rt.conn.apply(self.placed(&self.panes[i]))?;
        self.frame()
    }

    /// Scales a pane. A window of the flavor gets the new size, so that
    /// its content stays at 1:1; other panes stretch.
    pub(crate) fn resize_pane(&mut self, i: usize, scale: f64) -> Result<(), ClientError> {
        let pane = &mut self.panes[i];
        let size = pane
            .size
            .map(|s| (s * scale).round().clamp(PANE_MIN, PANE_MAX));
        pane.size = size;
        let pane = &self.panes[i];
        let mut commands = self.placed(pane);
        if let Some(surface) = pane.window().filter(|_| pane.attached) {
            commands.push(configure(surface, size));
        }
        self.rt.conn.apply(commands)?;
        self.frame()
    }

    /// Removes a pane; its window gets a close request, so a program with
    /// unsaved state can ask the user first (F13). The window of another
    /// flavor's owner gets it through `node.requested`. The pane goes
    /// when the window does: a window that refuses stays a pane.
    pub(crate) fn remove_pane(&mut self, i: usize) -> Result<(), ClientError> {
        if self.panes[i].taken {
            let surface = self.panes[i].source.clone();
            return self.forget(&surface);
        }
        let Some(surface) = self.panes[i].window().map(String::from) else {
            let pane = self.panes.remove(i);
            self.rt.conn.apply(pane.remove().into())?;
            if let Content::Manager { manager } = &pane.content {
                self.end_manager(manager);
            }
            if self
                .flown
                .as_ref()
                .is_some_and(|f| f.plugin.is_some() && f.surface == pane.source)
            {
                return self.fly_out();
            }
            return self.frame();
        };
        if let Some(err) = self.rt.conn.try_apply(vec![Command::Close { surface }])? {
            eprintln!("chora-construct: close: {err}");
        }
        Ok(())
    }

    /// The group of the space with the fog, the panes and the look rect,
    /// on the screen root of the output, at its size; the windows stay.
    /// As a flavor, it claims the output, so that new windows come to it
    /// also beside another window manager (D55).
    #[allow(clippy::too_many_lines, reason = "one node after the other")]
    pub(crate) fn build(&mut self) -> Result<(), ClientError> {
        let Some(output) = &self.output else {
            return Ok(());
        };
        if !self.active {
            return Ok(());
        }
        let [w, h] = output.size.map(f64::from);
        let [x, y] = output.position.map(f64::from);
        self.lens = Lens::new(w, h);
        let root = overlay_id(&output.name);
        let mut commands = Vec::new();
        match self.rt.conn.query(&format!("id:{NAME}"))?.first() {
            None => {
                commands = space_nodes(&self.fog_shader, &root);
                if !self.glass.plain {
                    commands.push(create(
                        FOCUS,
                        NodeKind::Shader,
                        PANES,
                        PropsUpdate {
                            shader: Some(self.glow_shader.clone()),
                            visible: Some(false),
                            input: Some(InputMode::Pass),
                            ..PropsUpdate::default()
                        },
                    ));
                }
                self.sent = None;
                self.glow = None;
                self.backs.clear();
                commands.extend(self.styled().groups(NAME));
                self.labelled = None;
            }
            Some(node) if node.parent.as_deref() != Some(root.as_str()) => {
                commands.push(Command::Move {
                    id: NAME.into(),
                    parent: Some(root),
                    index: None,
                });
            }
            Some(_) => {}
        }
        let claim = (!self.overlay).then_some([x, y, w, h]);
        commands.extend([
            update(
                NAME,
                PropsUpdate {
                    claim: Some(claim),
                    ..PropsUpdate::default()
                },
            ),
            update(
                SKY,
                PropsUpdate {
                    size: Some([w, h]),
                    ..PropsUpdate::default()
                },
            ),
            update(
                FOG,
                PropsUpdate {
                    size: Some([w / FOG_SCALE, h / FOG_SCALE]),
                    transform: Some(Transform::scale(FOG_SCALE, FOG_SCALE, 1.0)),
                    ..PropsUpdate::default()
                },
            ),
            update(
                LOOK,
                PropsUpdate {
                    size: Some([w, h]),
                    ..PropsUpdate::default()
                },
            ),
            update(
                SPACE,
                PropsUpdate {
                    size: Some([w, h]),
                    perspective: Some(Some(self.lens.focal)),
                    ..PropsUpdate::default()
                },
            ),
        ]);
        commands.extend(self.panes.iter().flat_map(|p| self.placed(p)));
        self.rt.conn.apply(commands)?;
        self.draw_hud()?;
        self.draw_ask()?;
        self.draw_modes()?;
        self.frame()
    }

    /// A unique label from `base`: the app id or title of a window, or
    /// the scene of a plugin pane, with `-2`, `-3` and on after it.
    pub(crate) fn label(&self, base: &str) -> String {
        unique(base, |n| self.panes.iter().any(|p| p.name == n))
    }

    /// The place of its file, or of a new pane of `size` in the view
    /// (C7); with `force`, ahead also when the view is full.
    pub(crate) fn place(
        &self,
        at: Option<[f64; 2]>,
        size: [f64; 2],
        force: bool,
    ) -> Option<[f64; 2]> {
        let distance = self.settings.value.pane_distance;
        let others: Vec<_> = self.panes.iter().map(|p| (p.at, p.size, p.yaw)).collect();
        at.or_else(|| space::place(&self.lens, &self.pose, distance, size, &others))
            .or_else(|| {
                let widths: Vec<_> = others.iter().map(|(at, size, _)| (*at, size[0])).collect();
                force.then(|| space::beside(&self.pose, distance, size[0], &widths))
            })
    }

    /// The yaw of a new pane at `at`: its file's, else facing the camera.
    pub(crate) fn yaw_at(&self, file: Option<&PaneFile>, at: [f64; 2]) -> f64 {
        (file.and_then(|f| f.yaw)).unwrap_or_else(|| space::facing([self.pose.x, self.pose.z], at))
    }

    /// The look that the panes have now, or the look of no focus before
    /// the first frame.
    pub(crate) fn current_look(&self) -> PaneLook {
        self.sent.clone().unwrap_or_else(|| self.pane_look(None))
    }

    /// The effect of a pane and the uniforms of its mirror image for
    /// `look`.
    pub(crate) fn pane_effects(&self, pane: &Pane, look: &PaneLook) -> [Command; 2] {
        let focused = look.focus.as_deref() == Some(pane.name.as_str());
        [
            update(
                &pane.node(),
                PropsUpdate {
                    effects: Some(vec![pane_effect(&self.pane_shader, look, focused)]),
                    ..PropsUpdate::default()
                },
            ),
            update(
                &pane.mirror(),
                PropsUpdate {
                    uniforms: Some(mirror_uniforms(pane.reads(), look)),
                    ..PropsUpdate::default()
                },
            ),
        ]
    }

    /// The size and the transform of a pane, of its mirror image and of
    /// the shader node of its source, after a move or a resize.
    pub(crate) fn placed(&self, pane: &Pane) -> Vec<Command> {
        let mirror = mirror_size(pane.size);
        let k = pane.size[0] / mirror[0];
        let at = |model| Some(Transform::from_matrix(model));
        let mut commands = vec![
            update(
                &pane.node(),
                PropsUpdate {
                    size: Some(pane.size),
                    transform: at(space::pane_model(pane.at, pane.size, pane.yaw)),
                    ..PropsUpdate::default()
                },
            ),
            update(
                &pane.mirror(),
                PropsUpdate {
                    size: Some(mirror),
                    transform: at(space::mirror_model(
                        &self.lens, pane.at, pane.size, pane.yaw, k,
                    )),
                    ..PropsUpdate::default()
                },
            ),
        ];
        if let Some(node) = pane.texture() {
            commands.push(update(
                &node,
                PropsUpdate {
                    size: Some(pane.size),
                    ..PropsUpdate::default()
                },
            ));
        }
        commands
    }

    /// The nodes of a new pane: its mirror image, a shader node that reads
    /// its source; and the pane, a group with the effect pane.frag in the
    /// camera group of the 3D context. The group holds the window of the
    /// construct itself, which takes the input through the ray of the
    /// core, or else a shader node that shows the source through a node
    /// texture.
    pub(crate) fn pane_nodes(&self, pane: &Pane) -> Vec<Command> {
        let look = self.current_look();
        let focused = look.focus.as_deref() == Some(pane.name.as_str());
        let mirror = mirror_size(pane.size);
        let k = pane.size[0] / mirror[0];
        let mut commands = vec![
            create(
                &pane.mirror(),
                NodeKind::Shader,
                MIRRORS,
                PropsUpdate {
                    size: Some(mirror),
                    shader: Some(self.mirror_shader.clone()),
                    uniforms: Some(mirror_uniforms(pane.reads(), &look)),
                    transform: Some(Transform::from_matrix(space::mirror_model(
                        &self.lens, pane.at, pane.size, pane.yaw, k,
                    ))),
                    ..PropsUpdate::default()
                },
            ),
            create(
                &pane.node(),
                NodeKind::Group,
                PANES,
                PropsUpdate {
                    size: Some(pane.size),
                    effects: Some(vec![pane_effect(&self.pane_shader, &look, focused)]),
                    transform: Some(Transform::from_matrix(space::pane_model(
                        pane.at, pane.size, pane.yaw,
                    ))),
                    ..PropsUpdate::default()
                },
            ),
        ];
        if let Some(node) = pane.texture() {
            commands.push(create(
                &node,
                NodeKind::Shader,
                &pane.node(),
                PropsUpdate {
                    size: Some(pane.size),
                    shader: Some(self.source_shader.clone()),
                    uniforms: Some(BTreeMap::from([("win".into(), pane.reads())])),
                    forward: pane.forwards(),
                    ..PropsUpdate::default()
                },
            ));
        }
        commands
    }
}
