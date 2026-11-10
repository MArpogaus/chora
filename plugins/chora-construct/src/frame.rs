//! The per frame update of the space, the panes, their mirrors and the
//! focus glow.

use super::{
    BTreeMap, ClientError, Command, Construct, FOCUS, FOCUS_SCALE, FOG, FOG_SCALE, Pane,
    PropsUpdate, SHADOWS, SKY, Theme, Transform, Uniform, mirror_size, mirror_uniforms,
    pane_uniforms, shadow_w, space, update, vec3,
};

impl Construct {
    /// The sky, the ground and the fog for the pose, with the shadows of
    /// the nearest panes, `near` first.
    pub(crate) fn space_frame(&self, near: &[&Pane]) -> Vec<Command> {
        let palette = self.settings.value.theme.palette();
        let edge = (self.pose.x.hypot(self.pose.z) / self.radius()).min(1.0);
        let mut uniforms = BTreeMap::from([
            (
                "view".into(),
                Uniform::Vector(vec![self.lens.w, self.lens.h]),
            ),
            (
                "pose".into(),
                Uniform::Vector(vec![
                    self.pose.yaw,
                    self.pose.pitch,
                    self.pose.x,
                    self.pose.z,
                ]),
            ),
            (
                "lens".into(),
                Uniform::Vector(vec![self.lens.focal, self.lens.ground, FOG_SCALE, edge]),
            ),
            ("sky".into(), vec3(palette.sky)),
            ("fog".into(), vec3(palette.horizon)),
            ("ground".into(), vec3(palette.ground)),
            ("sheen".into(), vec3(palette.sheen)),
        ]);
        for k in 0..SHADOWS {
            let shadow = near.get(k).map_or(vec![0.0; 4], |p| {
                let lift = (self.lens.ground - p.size[1] / 2.0).max(0.0);
                vec![p.at[0], p.at[1], p.size[0] / 2.0, shadow_w(lift, p.yaw)]
            });
            uniforms.insert(format!("s{k}"), Uniform::Vector(shadow));
        }
        let color = Some([palette.sky[0], palette.sky[1], palette.sky[2], 1.0]);
        vec![
            update(
                SKY,
                PropsUpdate {
                    color,
                    ..PropsUpdate::default()
                },
            ),
            update(
                FOG,
                PropsUpdate {
                    uniforms: Some(uniforms),
                    ..PropsUpdate::default()
                },
            ),
        ]
    }

    /// Sends the pose: the matrix and the haze of each pane and its
    /// mirror image, back to front, and the camera of the fog.
    #[allow(clippy::too_many_lines, reason = "each pane, its mirror and the focus")]
    pub(crate) fn frame(&mut self) -> Result<(), ClientError> {
        if self.output.is_none() || !self.active {
            return Ok(());
        }
        let theme = self.settings.value.theme;
        let fog = theme.palette().horizon;
        let gloss = match theme {
            Theme::Light => 0.4,
            Theme::Dark => 0.55,
        };
        let radius = self.radius();
        let mut sorted: Vec<(&Pane, f64)> = (self.panes.iter())
            .map(|p| (p, space::depth(&self.pose, p.at)))
            .collect();
        sorted.sort_by(|a, b| b.1.total_cmp(&a.1));
        let near: Vec<&Pane> = (sorted.iter().rev())
            .filter(|(_, depth)| *depth > 0.0)
            .map(|(p, _)| *p)
            .collect();
        let mut commands = self.space_frame(&near);
        let focus = self.focus().map(|k| self.panes[k].name.clone());
        let accent = self.settings.value.style.accent;
        let mut glow = None;
        for (pane, depth) in &sorted {
            let focused = focus.as_deref() == Some(pane.name.as_str());
            let shown = *depth > self.lens.focal * 0.1
                && space::in_front(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
            let mut props = PropsUpdate {
                visible: Some(shown),
                ..PropsUpdate::default()
            };
            let mut mirror = props.clone();
            if shown {
                let matrix =
                    space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
                let far = (depth - self.lens.focal).max(0.0) / radius;
                let haze = if focused {
                    0.0
                } else {
                    (1.0 - (-far * 1.5).exp()) * 0.9
                };
                let px = depth / self.lens.focal;
                let edge = if focused {
                    [accent[0], accent[1], accent[2], (1.5 * px).max(1.0)]
                } else {
                    [0.0; 4]
                };
                props.transform = Some(Transform::from_matrix(matrix));
                props.uniforms = Some(pane_uniforms(&pane.source, haze, fog, edge));
                if focused {
                    glow = Some(self.focus_glow(pane, px));
                }
                let k = pane.size[0] / mirror_size(pane.size)[0];
                let matrix =
                    space::mirror_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw, k);
                mirror.transform = Some(Transform::from_matrix(matrix));
                let grazing = 1.0 - self.lens.ground / self.lens.ground.hypot(*depth);
                let fresnel = 0.05 + 0.95 * grazing.powi(5);
                let strength = gloss * (0.4 + 0.6 * fresnel) * (1.0 - haze);
                mirror.uniforms = Some(mirror_uniforms(&pane.source, strength));
            }
            commands.push(update(&pane.node(), props));
            commands.push(update(&pane.mirror(), mirror));
        }
        let shows_glow = glow.is_some() && self.flown.is_none();
        if !self.glass.plain {
            let props = glow.filter(|_| shows_glow).unwrap_or(PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            });
            commands.push(update(FOCUS, props));
        }
        let order: Vec<String> = (sorted.iter())
            .flat_map(|(p, _)| {
                let under = (shows_glow && focus.as_deref() == Some(p.name.as_str()))
                    .then(|| FOCUS.to_owned());
                under.into_iter().chain([p.node()])
            })
            .collect();
        if order != self.order {
            commands.extend(order.iter().map(|id| Command::Move {
                id: id.clone(),
                parent: None,
                index: None,
            }));
            self.order = order;
        }
        commands.extend(self.label_commands());
        self.rt.conn.apply(commands)
    }

    /// The pane in focus: the pane flown into, else the pane under the
    /// pointer or ahead, as the label picks it.
    pub(crate) fn focus(&self) -> Option<usize> {
        if let Some(flown) = &self.flown {
            return self
                .panes
                .iter()
                .position(|p| p.window() == Some(&flown.surface));
        }
        (self.pointer.and_then(|p| self.pane_at(p)))
            .or_else(|| self.pane_ahead())
            .filter(|&i| {
                let far = self.radius();
                space::sight(&self.lens, &self.pose, self.panes[i].spot(), far) == "shown"
            })
    }

    /// The soft accent glow behind the focused pane, as the glass of a
    /// panel has it: a node of a quarter of the size, scaled up, whose
    /// glow reaches 16 pixels of the output. `px` is the size of a pixel
    /// of the output in pixels of the pane.
    pub(crate) fn focus_glow(&self, pane: &Pane, px: f64) -> PropsUpdate {
        let reach = self.settings.value.style.glow * px;
        let [w, h] = pane.size;
        let size = [w / FOCUS_SCALE, h / FOCUS_SCALE];
        let r = reach / FOCUS_SCALE;
        let uniforms = self.styled().glow_uniforms(size, 0.0, r, [0.0; 4], 0.0);
        let matrix = space::glow_matrix(
            &self.lens,
            &self.pose,
            pane.at,
            pane.size,
            pane.yaw,
            reach,
            FOCUS_SCALE,
        );
        PropsUpdate {
            size: Some([size[0] + 2.0 * r, size[1] + 2.0 * r]),
            transform: Some(Transform::from_matrix(matrix)),
            uniforms: Some(uniforms),
            visible: Some(true),
            ..PropsUpdate::default()
        }
    }
}
