//! The per frame update of the space: the camera, the fog and the focus
//! glow. The core projects, sorts and clips the panes in its 3D context
//! (D75); the panes and their mirrors read `depth` and `facing` for their
//! haze, edge and gloss, so a walk sends no update per pane. A pane gets
//! new effect uniforms only when the focus or the style changes.

use super::{
    BTreeMap, ClientError, Command, Construct, FOCUS, FOCUS_SCALE, FOG, FOG_SCALE, POSE, Pane,
    PropsUpdate, SHADOWS, SKY, Transform, Uniform, shadow_w, space, update, vec3,
};
use crate::panes::PaneLook;

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

    /// Sends the pose: the transform of the camera group and the camera
    /// of the fog, the focus glow, and the look of the panes when it
    /// changed.
    pub(crate) fn frame(&mut self) -> Result<(), ClientError> {
        if self.output.is_none() || !self.active {
            return Ok(());
        }
        let mut near: Vec<(&Pane, f64)> = (self.panes.iter())
            .map(|p| (p, space::depth(&self.pose, p.at)))
            .filter(|(_, depth)| *depth > 0.0)
            .collect();
        near.sort_by(|a, b| a.1.total_cmp(&b.1));
        let near: Vec<&Pane> = near.into_iter().map(|(p, _)| p).collect();
        let mut commands = self.space_frame(&near);
        commands.push(update(
            POSE,
            PropsUpdate {
                transform: Some(Transform::from_matrix(space::camera(
                    &self.lens, &self.pose,
                ))),
                ..PropsUpdate::default()
            },
        ));
        let focus = self.focus();
        let look = self.pane_look(focus.map(|k| self.panes[k].name.clone()));
        if self.sent.as_ref() != Some(&look) {
            commands.extend(self.panes.iter().flat_map(|p| self.pane_effects(p, &look)));
            self.sent = Some(look);
        }
        if !self.glass.plain {
            let glow = focus
                .filter(|_| self.flown.is_none())
                .map(|k| self.focus_glow(&self.panes[k]));
            let props = glow.unwrap_or(PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            });
            commands.push(update(FOCUS, props));
        }
        commands.extend(self.label_commands());
        self.rt.conn.apply(commands)
    }

    /// The look of the panes now: the focused pane, the fog, the accent,
    /// the gloss and the distances of the haze.
    pub(crate) fn pane_look(&self, focus: Option<String>) -> PaneLook {
        let theme = self.settings.value.theme;
        PaneLook {
            focus,
            fog: theme.palette().horizon,
            accent: self.settings.value.style.accent,
            gloss: theme.gloss(),
            reach: self.lens.focal / self.radius(),
            ground: self.lens.ground / self.lens.focal,
        }
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
    /// glow reaches 16 pixels of the output, behind the pane in the
    /// context.
    pub(crate) fn focus_glow(&self, pane: &Pane) -> PropsUpdate {
        let px = space::depth(&self.pose, pane.at).max(self.lens.focal * 0.1) / self.lens.focal;
        let reach = self.settings.value.style.glow * px;
        let [w, h] = pane.size;
        let size = [w / FOCUS_SCALE, h / FOCUS_SCALE];
        let r = reach / FOCUS_SCALE;
        let uniforms = self.styled().glow_uniforms(size, 0.0, r, [0.0; 4], 0.0);
        let model = space::glow_model(pane.at, pane.size, pane.yaw, reach, FOCUS_SCALE);
        PropsUpdate {
            size: Some([size[0] + 2.0 * r, size[1] + 2.0 * r]),
            transform: Some(Transform::from_matrix(model)),
            uniforms: Some(uniforms),
            visible: Some(true),
            ..PropsUpdate::default()
        }
    }
}
