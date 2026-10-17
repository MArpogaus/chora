//! 4x4 transforms of scene nodes.

use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

/// A 4x4 transform matrix in column-major order.
///
/// The JSON form is either `{"matrix": [16 numbers]}` or the short form
/// `{"translate": [x, y, z], "scale": [x, y, z], "rotate": degrees}`. The
/// short form applies scale first, then the rotation around the z axis, then
/// the translation. Every field of the short form is optional.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(try_from = "TransformSpec", into = "MatrixSpec")]
pub struct Transform {
    m: [f64; 16],
}

/// The two JSON forms. The parse picks the form by the key `matrix`, so
/// that an error names the wrong key, not just that no form matched.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(untagged))]
enum TransformSpec {
    /// The full matrix.
    Matrix(MatrixSpec),
    /// The short form.
    Short(ShortSpec),
}

impl<'de> Deserialize<'de> for TransformSpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::Object(serde_json::Map::deserialize(deserializer)?);
        let spec = if value.get("matrix").is_some() {
            serde_json::from_value(value).map(Self::Matrix)
        } else {
            serde_json::from_value(value).map(Self::Short)
        };
        spec.map_err(|e| {
            D::Error::custom(format!(
                "transform: {e}; it takes `matrix` alone, or `translate`, `scale` and `rotate`"
            ))
        })
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
struct MatrixSpec {
    /// 16 numbers in column-major order.
    #[cfg_attr(feature = "schema", schemars(length(equal = 16)))]
    matrix: Vec<f64>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
struct ShortSpec {
    /// `[x, y]` or `[x, y, z]`, in the coordinates of the parent.
    #[cfg_attr(feature = "schema", schemars(length(max = 3)))]
    translate: Option<Vec<f64>>,
    /// A number for all axes, or `[x, y]` or `[x, y, z]`.
    scale: Option<Scale>,
    /// Degrees around the z axis, clockwise on the screen.
    rotate: Option<f64>,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(
    untagged,
    expecting = "a scale of a number or a list of up to 3 numbers"
)]
enum Scale {
    /// The same for all axes.
    Uniform(f64),
    /// One per axis.
    Axes(#[cfg_attr(feature = "schema", schemars(length(max = 3)))] Vec<f64>),
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
    #[allow(clippy::float_cmp)] // translate and scale leave these entries exactly 0 or 1
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

    /// The transform of a column-major matrix.
    #[must_use]
    pub fn from_matrix(m: [f64; 16]) -> Self {
        Self { m }
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
    fn a_wrong_key_or_shape_names_the_problem() {
        for (json, message) in [
            (
                r#"{"rotate_z": 45}"#,
                "transform: unknown field `rotate_z`, expected one of `translate`, `scale`, \
                 `rotate`; it takes `matrix` alone, or `translate`, `scale` and `rotate`",
            ),
            (
                r#"{"matrix": [], "scale": 2}"#,
                "transform: unknown field `scale`, expected `matrix`; it takes `matrix` \
                 alone, or `translate`, `scale` and `rotate`",
            ),
            (
                r#"{"scale": "big"}"#,
                "transform: a scale of a number or a list of up to 3 numbers; it takes \
                 `matrix` alone, or `translate`, `scale` and `rotate`",
            ),
        ] {
            let err = serde_json::from_str::<Transform>(json).unwrap_err();
            assert_eq!(err.to_string(), message, "{json}");
        }
    }

    #[test]
    fn parent_then_child_composes() {
        let parent = Transform::translate(100.0, 0.0, 0.0);
        let child = Transform::scale(2.0, 2.0, 1.0);
        assert_eq!(parent.then(&child).apply(1.0, 1.0), (102.0, 2.0));
    }
}
