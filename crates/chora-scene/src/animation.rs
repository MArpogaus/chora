//! Animations of node properties (D28): the core moves properties toward
//! targets over a duration, frame by frame, so that a plugin sends one
//! command and not one transaction per frame.
//!
//! The transform, the opacity, the size, the color, a mesh of the same
//! grid and the numeric uniforms of effects move along an easing curve.
//! The other properties, for example `visible`, change at the end,
//! so that a node can fade out and then hide.

use glam::DQuat;
use serde::{Deserialize, Serialize};

use crate::{Decomposed, Mesh, Props, PropsUpdate, ShaderEffect, Transform, Uniform};

/// The longest animation, in milliseconds.
pub const MAX_DURATION: f64 = 60_000.0;

/// How an animation moves from the start to the target.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Easing {
    /// At a constant speed.
    #[default]
    Linear,
    /// Slow at the start.
    EaseIn,
    /// Slow at the end.
    EaseOut,
    /// Slow at the start and at the end.
    EaseInOut,
}

impl Easing {
    /// The progress of the curve at the time `t` from 0 to 1, cubic.
    #[must_use]
    pub fn at(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut if t < 0.5 => 4.0 * t * t * t,
            Self::EaseInOut => 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0,
        }
    }
}

/// A running animation of one node.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Animation {
    /// The properties at the start.
    pub from: Props,
    /// The targets.
    pub to: PropsUpdate,
    /// In milliseconds.
    pub duration: f64,
    /// The time since the start, in milliseconds.
    pub elapsed: f64,
    pub easing: Easing,
}

impl Animation {
    /// The time from 0 to 1.
    pub fn progress(&self) -> f64 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).min(1.0)
        } else {
            1.0
        }
    }

    /// The properties at the current time; at the end, all targets.
    pub fn props(&self) -> Props {
        let t = self.progress();
        let mut props = self.from.clone();
        if t >= 1.0 {
            props.apply(&self.to);
            return props;
        }
        let e = self.easing.at(t);
        let lerp = |a: f64, b: f64| a + (b - a) * e;
        if let Some(to) = &self.to.transform {
            props.transform = self.from.transform.lerp(to, e);
        }
        if let Some(to) = self.to.opacity {
            props.opacity = lerp(self.from.opacity, to.clamp(0.0, 1.0));
        }
        if let (Some(from), Some(to)) = (self.from.size, self.to.size) {
            props.size = Some([lerp(from[0], to[0]), lerp(from[1], to[1])]);
        }
        if let (Some(from), Some(to)) = (self.from.color, self.to.color) {
            props.color = Some(std::array::from_fn(|i| lerp(from[i], to[i])));
        }
        if let Some(to) = self.to.radius {
            props.radius = lerp(self.from.radius, to);
        }
        if let Some(to) = &self.to.effects {
            lerp_uniforms(&mut props.effects, to, e);
        }
        if let (Some(from), Some(to)) = (&self.from.mesh, &self.to.mesh)
            && (from.cols, from.rows) == (to.cols, to.rows)
            && from.points.len() == to.points.len()
        {
            let points = from.points.iter().zip(&to.points);
            let points = points.map(|(a, b)| [lerp(a[0], b[0]), lerp(a[1], b[1])]);
            let depth = if from.depth.is_empty() && to.depth.is_empty() {
                Vec::new()
            } else {
                (0..to.points.len())
                    .map(|i| lerp(from.z(i), to.z(i)))
                    .collect()
            };
            props.mesh = Some(Mesh {
                points: points.collect(),
                depth,
                ..to.clone()
            });
        }
        props
    }
}

/// Moves the numeric uniforms of each effect toward the target of the
/// same index and shader; the list itself and the other uniforms change at
/// the end.
fn lerp_uniforms(effects: &mut [ShaderEffect], to: &[ShaderEffect], t: f64) {
    let lerp = |a: &mut f64, b: f64| *a += (b - *a) * t;
    for (effect, to) in effects.iter_mut().zip(to) {
        if effect.shader != to.shader {
            continue;
        }
        for (name, value) in &mut effect.uniforms {
            match (value, to.uniforms.get(name)) {
                (Uniform::Float(a), Some(Uniform::Float(b))) => lerp(a, *b),
                (Uniform::Vector(a), Some(Uniform::Vector(b))) if a.len() == b.len() => {
                    a.iter_mut().zip(b).for_each(|(a, b)| lerp(a, *b));
                }
                _ => {}
            }
        }
    }
}

/// A camera that moves from where it was to its target.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CameraMove {
    /// The camera at the start.
    pub from: Transform,
    /// In milliseconds.
    pub duration: f64,
    /// Since the start, in milliseconds.
    pub elapsed: f64,
    /// The curve.
    pub easing: Easing,
}

impl CameraMove {
    /// The camera at the current time on the way to `to`.
    pub fn at(&self, to: &Transform) -> Transform {
        if self.ended() {
            return *to;
        }
        let t = (self.elapsed / self.duration).min(1.0);
        self.from.lerp(to, self.easing.at(t))
    }

    /// Whether the camera reached its target.
    pub fn ended(&self) -> bool {
        self.elapsed >= self.duration
    }
}

impl Transform {
    /// The transform between `self` at 0 and `other` at 1. If both
    /// decompose and their rotations differ, the translation and the scale
    /// move straight and the rotation turns along the shortest arc, so the
    /// size stays on the way. Otherwise the entries blend one by one.
    #[must_use]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        if let (Some(a), Some(b)) = (self.decompose(), other.decompose())
            && DQuat::from_array(a.rotation) != DQuat::from_array(b.rotation)
        {
            let rotation = DQuat::from_array(a.rotation).slerp(DQuat::from_array(b.rotation), t);
            return Self::compose(&Decomposed {
                translation: mix(a.translation, b.translation, t),
                rotation: rotation.to_array(),
                scale: mix(a.scale, b.scale, t),
            });
        }
        let (a, b) = (self.matrix(), other.matrix());
        Self::from_matrix(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t))
    }
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curves_start_at_0_and_end_at_1() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
        ] {
            assert!(easing.at(0.0).abs() < 1e-12);
            assert!((easing.at(1.0) - 1.0).abs() < 1e-12);
        }
        assert!(Easing::EaseOut.at(0.5) > 0.5);
        assert!(Easing::EaseIn.at(0.5) < 0.5);
    }

    #[test]
    fn an_animation_moves_and_hides_at_the_end() {
        let mut animation = Animation {
            from: Props::default(),
            to: PropsUpdate {
                transform: Some(Transform::translate(100.0, 0.0, 0.0)),
                opacity: Some(0.0),
                visible: Some(false),
                ..PropsUpdate::default()
            },
            duration: 200.0,
            elapsed: 100.0,
            easing: Easing::Linear,
        };
        let half = animation.props();
        assert_eq!(half.transform.translation_2d(), (50.0, 0.0));
        assert!((half.opacity - 0.5).abs() < 1e-12);
        assert!(half.visible);
        animation.elapsed = 250.0;
        let end = animation.props();
        assert_eq!(end.transform.translation_2d(), (100.0, 0.0));
        assert!(!end.visible);
    }

    #[test]
    fn numeric_uniforms_of_effects_move_and_the_rest_changes_at_the_end() {
        let effect = |shader: &str, p: f64, tint: f64| ShaderEffect {
            shader: shader.into(),
            uniforms: [
                ("progress".into(), Uniform::Float(p)),
                ("tint".into(), Uniform::Vector(vec![tint, tint])),
            ]
            .into(),
        };
        let mut animation = Animation {
            from: Props {
                effects: vec![effect("/a.frag", 0.0, 0.0), effect("/b.frag", 0.0, 0.0)],
                ..Props::default()
            },
            to: PropsUpdate {
                effects: Some(vec![
                    effect("/a.frag", 1.0, 2.0),
                    effect("/c.frag", 1.0, 2.0),
                    effect("/d.frag", 1.0, 2.0),
                ]),
                ..PropsUpdate::default()
            },
            duration: 200.0,
            elapsed: 100.0,
            easing: Easing::Linear,
        };
        let half = animation.props().effects;
        assert_eq!(
            half[0],
            effect("/a.frag", 0.5, 1.0),
            "same index and shader"
        );
        assert_eq!(half[1], effect("/b.frag", 0.0, 0.0), "another shader waits");
        assert_eq!(half.len(), 2, "the list changes at the end");
        animation.elapsed = 200.0;
        assert_eq!(animation.props().effects, animation.to.effects.unwrap());
    }

    fn column_length(t: &Transform) -> f64 {
        let m = t.matrix();
        m[0].hypot(m[1])
    }

    #[test]
    fn a_rotation_keeps_the_size_of_the_node_on_the_way() {
        let from = Transform::scale(100.0, 100.0, 1.0);
        let to = Transform::rotate_z(90.0).then(&from);
        let half = from.lerp(&to, 0.5);
        assert!((column_length(&half) - 100.0).abs() < 1e-9);
        let (x, y) = half.apply(1.0, 0.0);
        let angle = y.atan2(x).to_degrees();
        assert!((angle - 45.0).abs() < 1e-9, "{angle}");
        let end = from.lerp(&to, 1.0);
        assert!((end.apply(1.0, 0.0).1 - 100.0).abs() < 1e-9);
    }

    #[test]
    fn a_node_turns_by_half_a_turn_without_shrinking() {
        let from = Transform::translate(50.0, 0.0, 0.0);
        let to = Transform::translate(50.0, 0.0, 0.0).then(&Transform::rotate_z(180.0));
        let mut animation = Animation {
            from: Props {
                transform: from,
                ..Props::default()
            },
            to: PropsUpdate {
                transform: Some(to),
                ..PropsUpdate::default()
            },
            duration: 200.0,
            elapsed: 100.0,
            easing: Easing::Linear,
        };
        for elapsed in [0.0, 25.0, 100.0, 175.0] {
            animation.elapsed = elapsed;
            let t = animation.props().transform;
            assert!((column_length(&t) - 1.0).abs() < 1e-9, "{elapsed}");
        }
        animation.elapsed = 200.0;
        let end = animation.props().transform;
        assert!((end.matrix()[0] + 1.0).abs() < 1e-9);
        assert!(end.matrix()[1].abs() < 1e-9);
    }

    #[test]
    fn a_camera_ends_exactly_on_its_target_after_a_rotation() {
        let to = Transform::translate(-30.3, 7.1, 0.0)
            .then(&Transform::rotate_z(123.4).then(&Transform::scale(0.7, 1.9, 1.0)));
        let mut camera = CameraMove {
            from: Transform::IDENTITY,
            duration: 100.0,
            elapsed: 100.0,
            easing: Easing::Linear,
        };
        assert_eq!(camera.at(&to), to);
        camera.elapsed = 500.0;
        assert_eq!(camera.at(&to), to);
    }

    #[test]
    fn translation_and_scale_blend_entry_by_entry() {
        let from = Transform::translate(10.0, 20.0, 0.0).then(&Transform::scale(2.0, 4.0, 1.0));
        let to = Transform::translate(-30.0, 5.0, 8.0).then(&Transform::scale(5.0, 1.0, 3.0));
        for t in [0.0, 0.1, 0.5, 0.9, 1.0] {
            let blend: [f64; 16] =
                std::array::from_fn(|i| from.matrix()[i] + (to.matrix()[i] - from.matrix()[i]) * t);
            assert_eq!(from.lerp(&to, t).matrix(), &blend);
        }
    }

    #[test]
    fn a_matrix_that_does_not_decompose_blends_entry_by_entry() {
        let mut skew = *Transform::IDENTITY.matrix();
        skew[4] = 1.0;
        let from = Transform::from_matrix(skew);
        let to = Transform::rotate_z(90.0);
        let half = from.lerp(&to, 0.5);
        let blend: [f64; 16] = std::array::from_fn(|i| skew[i] + (to.matrix()[i] - skew[i]) * 0.5);
        assert_eq!(half.matrix(), &blend);
    }

    #[test]
    fn a_rotation_takes_the_short_way_round() {
        let from = Transform::rotate_z(10.0);
        let to = Transform::rotate_z(350.0);
        let (x, y) = from.lerp(&to, 0.5).apply(1.0, 0.0);
        assert!(y.atan2(x).abs() < 1e-9);
    }
}
