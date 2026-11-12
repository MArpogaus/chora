//! The camera and navigation: walk, look, glide, fly in, snap into a
//! window, fly out, and the pane under a point.

use super::{
    Action, ClientError, Command, Construct, Content, FLY_MS, FRONT, Flown, Glide, Grab, LOOK,
    LOOK_RATE, Pane, Pose, PropsUpdate, RpcError, STEP, STEP_MS, TURN, Transform, Value, json,
    space, stride, update,
};

/// The surface that the camera is in or on its way into.
pub(crate) fn window_flown_into<'a>(
    flown: Option<&'a Flown>,
    glide: Option<&'a Glide>,
) -> Option<&'a str> {
    let into = flown.or_else(|| glide.and_then(|g| g.into.as_ref()));
    into.map(|f| f.surface.as_str())
}

/// Whether a window handed over takes the camera: not while the user is
/// in a window.
pub(crate) fn takes_the_camera(inside: Option<&str>) -> bool {
    inside.is_none()
}

/// Turns the pose by a drag of `dx`, `dy`, and a running glide with it,
/// so that a walk during the drag goes on. Only the yaw and the pitch
/// change.
pub(crate) fn turn(pose: &mut Pose, glide: Option<&mut Glide>, dx: f64, dy: f64) {
    let by = |p: &mut Pose| {
        p.yaw += dx * LOOK_RATE;
        p.pitch = (p.pitch - dy * LOOK_RATE).clamp(-1.2, 1.2);
    };
    by(pose);
    if let Some(glide) = glide {
        by(&mut glide.from);
        by(&mut glide.to);
    }
}

/// A place moved by `right` and `ahead` as a camera that looks
/// `forward` sees them.
pub(crate) fn moved(at: [f64; 2], [fx, fz]: [f64; 2], right: f64, ahead: f64) -> [f64; 2] {
    [
        at[0] + fx * ahead + fz * right,
        at[1] + fz * ahead - fx * right,
    ]
}

/// The eased share of a glide after `k` of its time.
pub(crate) fn ease(k: f64) -> f64 {
    1.0 - (1.0 - k).powi(3)
}

/// The share of a glide of `camera.glide` after `k` of its time: slow at
/// both ends.
pub(crate) fn ease_in_out(k: f64) -> f64 {
    k * k * k * (k * (k * 6.0 - 15.0) + 10.0)
}

/// The pane nearest the middle of the view, in front of the camera and
/// on the output: a pane out of sight is no pane ahead, so a key never
/// acts on a pane that is not there to see.
pub(crate) fn ahead(panes: &[Pane], lens: &space::Lens, pose: &Pose, far: f64) -> Option<usize> {
    let [fx, fz] = pose.forward();
    (panes.iter().enumerate())
        .filter(|(_, p)| space::sight(lens, pose, p.spot(), far) == "shown")
        .filter_map(|(i, p)| {
            let (dx, dz) = (p.at[0] - pose.x, p.at[1] - pose.z);
            let cos = (dx * fx + dz * fz) / dx.hypot(dz).max(f64::EPSILON);
            (cos > 0.3).then_some((i, cos))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// The flight into a pane that `glide` is on, taken out of it; a glide
/// that goes nowhere into a pane stays.
#[allow(dead_code)]
pub(crate) fn take_flight(glide: &mut Option<Glide>) -> Option<Flown> {
    glide.take().and_then(|g| g.into)
}

impl Construct {
    /// The leash radius: the construct file's, or the settings'.
    pub(crate) fn radius(&self) -> f64 {
        self.radius.unwrap_or(self.settings.value.radius)
    }

    /// Whether the camera is in a pane or on its way into one.
    pub(crate) fn in_pane(&self) -> bool {
        self.flown.is_some() || self.flying_into()
    }

    /// Whether the camera glides into a pane.
    pub(crate) fn flying_into(&self) -> bool {
        self.glide.as_ref().is_some_and(|g| g.into.is_some())
    }

    /// The target of the walk: the end of the glide, or the pose.
    pub(crate) fn target(&self) -> Pose {
        self.glide.as_ref().map_or(self.pose, |g| g.to)
    }

    pub(crate) fn glide_to(&mut self, to: Pose, ms: f64, into: Option<Flown>) {
        if into.is_some() {
            self.stride = None;
        }
        self.glide = Some(Glide {
            from: self.pose,
            to,
            start: chora_kit::clock::now(),
            ms,
            into,
            smooth: false,
        });
    }

    /// `camera.glide`: flies the camera to a pose in `ms` milliseconds,
    /// slow at its start and at its end; the place stays on the leash.
    pub(crate) fn glide_smoothly(&mut self, to: Pose, ms: f64) -> Result<(), RpcError> {
        if self.flown.is_some() {
            return Err(RpcError::new(
                RpcError::REJECTED,
                "the camera is in a pane; fly out first",
            ));
        }
        self.stride = None;
        let finite = [to.x, to.z, to.yaw, to.pitch, ms]
            .iter()
            .all(|v| v.is_finite());
        if !finite || !(0.0..=60_000.0).contains(&ms) {
            return Err(RpcError::new(
                RpcError::INVALID_PARAMS,
                "x, z, yaw and pitch are numbers, and ms is 0 to 60000",
            ));
        }
        let [x, z] = space::leash([0.0; 2], [to.x, to.z], self.radius());
        let to = Pose {
            x,
            z,
            pitch: to.pitch.clamp(-1.2, 1.2),
            ..to
        };
        self.glide = Some(Glide {
            from: self.pose,
            to,
            start: chora_kit::clock::now(),
            ms: ms.max(1.0),
            into: None,
            smooth: true,
        });
        Ok(())
    }

    pub(crate) fn act(&mut self, action: Action) {
        let mut t = self.target();
        match action {
            Action::Forward => return self.walk(STEP, 0.0),
            Action::Back => return self.walk(-STEP, 0.0),
            Action::Left => return self.walk(0.0, -STEP),
            Action::Right => return self.walk(0.0, STEP),
            Action::TurnLeft => t.yaw -= TURN,
            Action::TurnRight => t.yaw += TURN,
            Action::LookUp => t.pitch = (t.pitch + TURN).min(1.2),
            Action::LookDown => t.pitch = (t.pitch - TURN).max(-1.2),
            Action::Home => {
                let yaw = self.panes.first().map_or(0.0, |p| p.at[0].atan2(p.at[1]));
                let home = Pose {
                    x: 0.0,
                    z: 0.0,
                    yaw,
                    pitch: 0.0,
                };
                self.stride = None;
                return self.glide_to(home, FLY_MS, None);
            }
            Action::Terminal
            | Action::Launcher
            | Action::Close
            | Action::Windows
            | Action::Help
            | Action::Cube => return,
            Action::FlyIn => {
                if let Some(i) = self.pane_ahead()
                    && let Err(err) = self.fly_in(i)
                {
                    eprintln!("chora-construct: the fly-in: {err}");
                }
                return;
            }
        }
        self.glide_to(t, STEP_MS, None);
    }

    /// A walk key went down: one pane step in a tap, more while it is
    /// held. A repeat of the key that is down changes nothing.
    pub(crate) fn stride_press(&mut self, key: &str, action: Action) {
        let (ahead, aside) = match action {
            Action::Forward => (1.0, 0.0),
            Action::Back => (-1.0, 0.0),
            Action::Left => (0.0, -1.0),
            Action::Right => (0.0, 1.0),
            _ => return,
        };
        if self.flown.is_some() || self.in_pane() {
            return;
        }
        if self
            .stride
            .as_ref()
            .is_some_and(|s| s.held() && s.key == key)
        {
            return;
        }
        let now = chora_kit::clock::now();
        let from = self
            .stride
            .as_ref()
            .map_or([self.pose.x, self.pose.z], |s| s.at(now).0);
        let [fx, fz] = self.pose.forward();
        let dir = [fx * ahead + fz * aside, fz * ahead - fx * aside];
        self.stride = Some(stride::Stride::new(key, dir, from, now));
    }

    /// A walk key went up.
    pub(crate) fn stride_release(&mut self, key: &str) {
        if let Some(stride) = self.stride.as_mut().filter(|s| s.key == key) {
            stride.release(chora_kit::clock::now());
        }
    }

    /// One frame of the walk: the place of the camera on the leash.
    pub(crate) fn step_stride(&mut self) -> Result<(), ClientError> {
        let Some((key, held)) = self.stride.as_ref().map(|s| (s.key.clone(), s.held())) else {
            return Ok(());
        };
        if held && (self.help || self.list.is_some() || self.in_pane()) {
            self.stride_release(&key);
        }
        let Some(stride) = &self.stride else {
            return Ok(());
        };
        let (at, done) = stride.at(chora_kit::clock::now());
        let radius = self.radius();
        let length = at[0].hypot(at[1]);
        let at = if length > radius {
            [at[0] * radius / length, at[1] * radius / length]
        } else {
            at
        };
        [self.pose.x, self.pose.z] = at;
        if let Some(glide) = self.glide.as_mut().filter(|g| g.into.is_none()) {
            for pose in [&mut glide.from, &mut glide.to] {
                [pose.x, pose.z] = at;
            }
        }
        if done {
            self.stride = None;
        }
        if self.glide.is_none() {
            return self.frame();
        }
        Ok(())
    }

    /// Walks `ahead` forward and `aside` to the right, on the leash.
    pub(crate) fn walk(&mut self, ahead: f64, aside: f64) {
        if self.flown.is_some() {
            return;
        }
        let mut t = self.target();
        let [fx, fz] = t.forward();
        let step = [fx * ahead + fz * aside, fz * ahead - fx * aside];
        [t.x, t.z] = space::leash([t.x, t.z], step, self.radius());
        self.glide_to(t, STEP_MS, None);
    }

    /// A drag looks around: sideways turns, up and down tilts. It turns
    /// by the move since the last event and keeps the place, so a walk
    /// during the drag stays.
    pub(crate) fn look(&mut self, p: [f64; 2]) -> Result<(), ClientError> {
        let Some(drag) = &mut self.drag else {
            return Ok(());
        };
        let (dx, dy) = (p[0] - drag.press[0], p[1] - drag.press[1]);
        drag.moved |= dx.abs() + dy.abs() >= 4.0;
        if !drag.moved {
            return Ok(());
        }
        let (dx, dy) = (p[0] - drag.last[0], p[1] - drag.last[1]);
        drag.last = p;
        self.glide.take_if(|g| g.into.is_some());
        turn(&mut self.pose, self.glide.as_mut(), dx, dy);
        self.frame()
    }

    /// The pane under a point of the output, the nearest one, for the
    /// presses that the construct grabs and the pane label.
    pub(crate) fn pane_at(&self, p: [f64; 2]) -> Option<usize> {
        self.pane_hit(p, |_| true)
    }

    /// The nearest pane under a point of the output of those that `keep`.
    pub(crate) fn pane_hit(&self, p: [f64; 2], keep: impl Fn(&Pane) -> bool) -> Option<usize> {
        (self.panes.iter().enumerate())
            .filter(|(_, pane)| keep(pane))
            .filter_map(|(i, pane)| {
                space::hit(&self.lens, &self.pose, p, pane.at, pane.size, pane.yaw)
                    .map(|(t, _)| (i, t))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    /// The pane nearest the middle of the view that shows.
    pub(crate) fn pane_ahead(&self) -> Option<usize> {
        ahead(&self.panes, &self.lens, &self.pose, self.radius())
    }

    /// Flies in front of the face of a pane, which keeps its yaw. The
    /// window of a window pane then takes over; a plugin pane stays a
    /// pane, at 1:1. A fly-in from inside a pane, or during a fly-in,
    /// keeps the pose of the first to go back to. The window of a window
    /// pane takes the keyboard at the start, so that no key typed during
    /// the flight is lost.
    pub(crate) fn fly_in(&mut self, i: usize) -> Result<(), ClientError> {
        let pane = &self.panes[i];
        let to = space::fly_in(&self.lens, pane.at, pane.yaw);
        let gliding_in = self.glide.as_ref().and_then(|g| g.into.as_ref());
        let back = (self.flown.as_ref().or(gliding_in)).map_or_else(|| self.target(), |f| f.back);
        let into = match &pane.content {
            Content::Window { .. } => Some(Flown {
                surface: pane.source.clone(),
                back,
                attached: pane.attached,
                plugin: None,
            }),
            Content::Plugin { plugin, scene } => Some(Flown {
                surface: pane.source.clone(),
                back,
                attached: false,
                plugin: Some((plugin.clone(), scene.clone())),
            }),
            Content::Manager { .. } => None,
        };
        let window = matches!(pane.content, Content::Window { .. }).then(|| pane.source.clone());
        self.glide_to(to, FLY_MS, into);
        if let Some(surface) = window {
            self.rt.conn.apply(vec![Command::Focus {
                surface: Some(surface),
            }])?;
        }
        self.regrab()
    }

    pub(crate) fn step_glide(&mut self) -> Result<(), ClientError> {
        let Some(glide) = &self.glide else {
            return Ok(());
        };
        let k = (chora_kit::clock::since(glide.start).as_secs_f64() * 1000.0 / glide.ms).min(1.0);
        let share = if glide.smooth {
            ease_in_out(k)
        } else {
            ease(k)
        };
        self.pose = glide.from.lerp(&glide.to, share);
        if k < 1.0 {
            return self.frame();
        }
        self.pose = glide.to;
        let into = self.glide.take().and_then(|g| g.into);
        self.frame()?;
        if let Some(flown) = into {
            self.snap(flown)?;
        }
        Ok(())
    }

    /// The transform of the window that the camera flew into: whole
    /// pixels in the middle of the output, so that it shows 1:1.
    pub(crate) fn snap_place(&self, size: [f64; 2], surface: &str) -> Command {
        let x = ((self.lens.w - size[0]) / 2.0).round();
        let y = ((self.lens.h - size[1]) / 2.0).round();
        update(
            surface,
            PropsUpdate {
                transform: Some(Transform::translate(x, y, 0.0)),
                ..PropsUpdate::default()
            },
        )
    }

    /// The end of a fly-in: the window leaves its pane and shows over the
    /// space at whole pixels, takes the pointer and the keyboard, and the
    /// fly-out key is grabbed.
    /// A window of another owner stays where its owner has it: the pane
    /// shows it at 1:1, and the owner gets the request for the keyboard
    /// (D31).
    pub(crate) fn snap(&mut self, flown: Flown) -> Result<(), ClientError> {
        if let Some((plugin, scene)) = &flown.plugin {
            let call = json!({"client": plugin, "method": "pane.focus", "params": {"name": scene}});
            if let Err(err) = self.rt.conn.call::<_, Value>("methods.call", call) {
                eprintln!("chora-construct: {plugin} pane.focus: {err}");
            }
            self.flown = Some(flown);
            self.draw_hud()?;
            self.frame()?;
            return self.regrab();
        }
        let surface = flown.surface.clone();
        let Some(pane) = self.panes.iter().find(|p| p.window() == Some(&surface)) else {
            return Ok(());
        };
        let commands = if pane.attached {
            vec![
                self.snap_place(pane.size, &surface),
                Command::Move {
                    id: surface.clone(),
                    parent: Some(FRONT.into()),
                    index: None,
                },
                update(
                    LOOK,
                    PropsUpdate {
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                ),
                Command::Focus {
                    surface: Some(surface.clone()),
                },
            ]
        } else {
            vec![Command::Focus {
                surface: Some(surface),
            }]
        };
        self.rt.conn.apply(commands)?;
        self.flown = Some(flown);
        self.draw_hud()?;
        self.frame()?;
        self.regrab()
    }

    /// Grabs the drag with the modifier as the flavor, and the fly-out
    /// binding while the camera is in a pane. Without the grab of the
    /// drag, which another client may hold or the core refuses for Shift
    /// (D45), a press with the modifier on the empty space or on a pane
    /// that shows a node texture still moves a pane, and one on a window
    /// of the construct reaches the window.
    pub(crate) fn regrab(&mut self) -> Result<(), ClientError> {
        let parse = |b: String| {
            b.parse().map_err(|e| {
                ClientError::Rpc(RpcError::new(RpcError::INVALID_PARAMS, format!("{b}: {e}")))
            })
        };
        let serves = self.active && !self.overlay;
        self.rt
            .accept_injected(self.settings.value.test_accept_injected_keys);
        self.rt.give_keys(serves.then_some(Grab::Give));
        self.rt.ungrab_all()?;
        if self.active && !self.overlay {
            let drag = parse(format!("{}+BTN_LEFT", self.settings.value.modifier))?;
            if let Err(err) = self.rt.grab(drag, Grab::Drag) {
                eprintln!("chora-construct: the drag: {err}");
            }
        }
        if self.active && !self.overlay && self.flown.is_none() {
            let moves = parse(chora_kit::give::MOVE.into())?;
            if let Err(err) = self.rt.grab(moves, Grab::Drag) {
                eprintln!("chora-construct: the drag of a given window: {err}");
            }
        }
        self.rt.grab_give_keys()?;
        if self.in_pane() {
            let fly_out = parse(self.settings.value.fly_out.clone())?;
            self.rt.grab(fly_out, Grab::FlyOut)?;
            for key in self.terminal_keys() {
                self.rt.grab(parse(key)?, Grab::Terminal)?;
            }
        }
        self.grab_cube_keys();
        Ok(())
    }

    /// Puts the window back into its pane and gives the keys back to the
    /// walk.
    pub(crate) fn unsnap(&mut self, surface: Option<&str>, walk: bool) -> Result<(), ClientError> {
        let mut commands = vec![update(
            LOOK,
            PropsUpdate {
                visible: Some(true),
                ..PropsUpdate::default()
            },
        )];
        if walk {
            commands.push(Command::FocusNode {
                node: Some(LOOK.into()),
            });
        }
        let pane = surface.and_then(|s| self.panes.iter().find(|p| p.window() == Some(s)));
        if let (Some(surface), Some(pane)) = (surface, pane) {
            commands.splice(
                0..0,
                [
                    Command::Move {
                        id: surface.into(),
                        parent: Some(pane.node()),
                        index: None,
                    },
                    update(
                        surface,
                        PropsUpdate {
                            transform: Some(Transform::translate(0.0, 0.0, 0.0)),
                            ..PropsUpdate::default()
                        },
                    ),
                ],
            );
        }
        self.rt.conn.try_apply(commands)?;
        self.regrab()
    }

    /// Steps back out of the pane that the camera is in, or on its way
    /// into: the window goes back into its pane and the camera glides to
    /// where the fly-in began.
    pub(crate) fn fly_out(&mut self) -> Result<(), ClientError> {
        let flown = self
            .flown
            .take()
            .or_else(|| self.glide.take().and_then(|g| g.into));
        let Some(flown) = flown else {
            return Ok(());
        };
        self.unsnap(flown.attached.then_some(&flown.surface), true)?;
        self.draw_hud()?;
        self.glide_to(flown.back, FLY_MS, None);
        self.frame()
    }

    /// The keys that start a terminal and carry a modifier: the keys that
    /// a program in the pane never gets, so that they work from use mode.
    fn terminal_keys(&self) -> Vec<String> {
        (self.settings.value.keys.iter())
            .filter(|(key, action)| **action == Action::Terminal && key.contains('+'))
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// The place right of the panes for a new pane, as seen from `from`, at
    /// the distance of the panes, and the yaw that faces `from`.
    pub(crate) fn beside_spot(&self, from: Pose) -> ([f64; 2], f64) {
        let size = [self.lens.w, self.lens.h];
        let others: Vec<_> = self.panes.iter().map(|p| (p.at, p.size[0])).collect();
        let at = space::beside(&from, self.settings.value.pane_distance, size[0], &others);
        (at, space::facing([from.x, from.z], at))
    }

    /// A terminal from use mode: steps back, then starts the terminal
    /// to open right of the pane that the camera stood in, at the same
    /// distance from where the fly-in began, and fly into it.
    pub(crate) fn terminal_beside(&mut self) -> Result<(), ClientError> {
        let back = (self.flown.as_ref())
            .or_else(|| self.glide.as_ref().and_then(|g| g.into.as_ref()))
            .map(|f| f.back);
        self.fly_out()?;
        let spot = back.map(|back| self.beside_spot(back));
        self.start_terminal_at(spot);
        Ok(())
    }
}
