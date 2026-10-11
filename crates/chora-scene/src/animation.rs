//! Animations of node properties (D28): the core moves properties toward
//! targets over a duration, frame by frame, so that a plugin sends one
//! command and not one transaction per frame.
//!
//! The transform, the opacity, the size and the color move along an easing
//! curve. The other properties, for example `visible`, change at the end,
//! so that a node can fade out and then hide.

use serde::{Deserialize, Serialize};

use crate::{Props, PropsUpdate, Transform};

/// The longest animation, in milliseconds.
pub const MAX_DURATION: f64 = 60_000.0;

/// How an animation moves from the start to the target.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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
        props
    }
}

impl Transform {
    /// The transform between `self` at 0 and `other` at 1, entry by entry.
    /// Translations and scales move straight; a rotation shrinks on the
    /// way, as in other compositors that blend matrices.
    #[must_use]
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let (a, b) = (self.matrix(), other.matrix());
        Self::from_matrix(std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t))
    }
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
}
