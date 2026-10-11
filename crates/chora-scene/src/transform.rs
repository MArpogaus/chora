//! 4x4 transforms of scene nodes.

use serde::{Deserialize, Serialize};

/// A 4x4 transform matrix in column-major order.
///
/// The JSON form is either `{"matrix": [16 numbers]}` or the short form
/// `{"translate": [x, y, z], "scale": [x, y, z], "rotate": degrees}`. The
/// short form applies scale first, then the rotation around the z axis, then
/// the translation. Every field of the short form is optional.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "TransformSpec", into = "MatrixSpec")]
pub struct Transform {
    m: [f64; 16],
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TransformSpec {
    Matrix(MatrixSpec),
    Short(ShortSpec),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MatrixSpec {
    matrix: Vec<f64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShortSpec {
    #[serde(default)]
    translate: Option<Vec<f64>>,
    #[serde(default)]
    scale: Option<Scale>,
    #[serde(default)]
    rotate: Option<f64>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Scale {
    Uniform(f64),
    Axes(Vec<f64>),
}

impl TryFrom<TransformSpec> for Transform {
    type Error = String;

    fn try_from(spec: TransformSpec) -> Result<Self, Self::Error> {
        match spec {
            TransformSpec::Matrix(MatrixSpec { matrix }) => {
                let m: [f64; 16] = matrix
                    .try_into()
                    .map_err(|v: Vec<f64>| format!("a matrix needs 16 numbers, not {}", v.len()))?;
                Ok(Self { m })
            }
            TransformSpec::Short(short) => short.try_into(),
        }
    }
}

impl TryFrom<ShortSpec> for Transform {
    type Error = String;

    fn try_from(short: ShortSpec) -> Result<Self, Self::Error> {
        let [tx, ty, tz] = vec3(short.translate.as_deref().unwrap_or(&[]), 0.0, "translate")?;
        let [sx, sy, sz] = match short.scale {
            None => [1.0; 3],
            Some(Scale::Uniform(s)) => [s; 3],
            Some(Scale::Axes(v)) => vec3(&v, 1.0, "scale")?,
        };
        let scale = Self::scale(sx, sy, sz);
        let rotate = Self::rotate_z(short.rotate.unwrap_or(0.0));
        Ok(Self::translate(tx, ty, tz).then(&rotate.then(&scale)))
    }
}

/// Reads up to three numbers and fills the missing ones with `fill`.
fn vec3(v: &[f64], fill: f64, name: &str) -> Result<[f64; 3], String> {
    if v.len() > 3 {
        return Err(format!("{name} takes at most 3 numbers, not {}", v.len()));
    }
    let mut out = [fill; 3];
    out[..v.len()].copy_from_slice(v);
    Ok(out)
}

impl From<Transform> for MatrixSpec {
    fn from(t: Transform) -> Self {
        Self {
            matrix: t.m.to_vec(),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// The transform that changes nothing.
    pub const IDENTITY: Self = Self {
        m: [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    /// A translation.
    #[must_use]
    pub fn translate(x: f64, y: f64, z: f64) -> Self {
        let mut t = Self::IDENTITY;
        t.m[12] = x;
        t.m[13] = y;
        t.m[14] = z;
        t
    }

    /// A scale along the three axes.
    #[must_use]
    pub fn scale(x: f64, y: f64, z: f64) -> Self {
        let mut t = Self::IDENTITY;
        t.m[0] = x;
        t.m[5] = y;
        t.m[10] = z;
        t
    }

    /// A rotation around the z axis, in degrees.
    #[must_use]
    pub fn rotate_z(degrees: f64) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let mut t = Self::IDENTITY;
        t.m[0] = cos;
        t.m[1] = sin;
        t.m[4] = -sin;
        t.m[5] = cos;
        t
    }

    /// The transform that applies `inner` first and then `self`.
    #[must_use]
    pub fn then(&self, inner: &Self) -> Self {
        let (a, b) = (&self.m, &inner.m);
        let mut m = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                m[col * 4 + row] = (0..4).map(|k| a[k * 4 + row] * b[col * 4 + k]).sum();
            }
        }
        Self { m }
    }

    /// Maps a point of the plane z = 0.
    #[must_use]
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let m = &self.m;
        (m[0] * x + m[4] * y + m[12], m[1] * x + m[5] * y + m[13])
    }

    /// The translation in the plane z = 0.
    #[must_use]
    pub fn translation_2d(&self) -> (f64, f64) {
        (self.m[12], self.m[13])
    }

    /// The scale along x and y, if the transform is a 2D translation and
    /// scale only.
    #[must_use]
    #[allow(clippy::float_cmp)] // exact zeros: translate and scale write no others
    pub fn scale_2d(&self) -> Option<(f64, f64)> {
        let m = &self.m;
        let plain = m[1] == 0.0 && m[4] == 0.0 && m[3] == 0.0 && m[7] == 0.0 && m[15] == 1.0;
        plain.then_some((m[0], m[5]))
    }

    /// The column-major matrix.
    #[must_use]
    pub fn matrix(&self) -> &[f64; 16] {
        &self.m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_form_applies_scale_then_translation() {
        let t: Transform = serde_json::from_str(r#"{"translate": [10, 20], "scale": 2}"#).unwrap();
        assert_eq!(t.apply(1.0, 1.0), (12.0, 22.0));
        assert_eq!(t.scale_2d(), Some((2.0, 2.0)));
    }

    #[test]
    fn rotation_is_not_a_plain_2d_scale() {
        let t: Transform = serde_json::from_str(r#"{"rotate": 90}"#).unwrap();
        let (x, y) = t.apply(1.0, 0.0);
        assert!(x.abs() < 1e-9 && (y - 1.0).abs() < 1e-9);
        assert_eq!(t.scale_2d(), None);
    }

    #[test]
    fn matrix_form_round_trips() {
        let t = Transform::translate(3.0, 4.0, 0.0);
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<Transform>(&json).unwrap(), t);
    }

    #[test]
    fn wrong_matrix_length_is_an_error() {
        let err = serde_json::from_str::<Transform>(r#"{"matrix": [1, 2]}"#).unwrap_err();
        assert!(err.to_string().contains("16 numbers"), "{err}");
    }

    #[test]
    fn parent_then_child_composes() {
        let parent = Transform::translate(100.0, 0.0, 0.0);
        let child = Transform::scale(2.0, 2.0, 1.0);
        assert_eq!(parent.then(&child).apply(1.0, 1.0), (102.0, 2.0));
    }
}
