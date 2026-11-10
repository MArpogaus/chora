//! 4x4 transforms of scene nodes.

use glam::{DMat3, DMat4, DQuat, DVec3};
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use serde_json::Value;

/// A transform decomposed into a translation, a rotation and a scale, which
/// apply in the order scale, rotation, translation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decomposed {
    /// The translation along x, y and z.
    pub translation: [f64; 3],
    /// The rotation as a unit quaternion `[x, y, z, w]`.
    pub rotation: [f64; 4],
    /// The scale along x, y and z; one of them is negative for a mirror.
    pub scale: [f64; 3],
}

/// The largest entry change that still counts as no shear in `decompose`.
const DECOMPOSE_TOLERANCE: f64 = 1e-9;

/// The determinant relative to the n-th power of the longest column of the
/// linear part, under which a matrix counts as singular.
const SINGULAR_RATIO: f64 = 1e-12;

fn invertible(determinant: f64, longest: f64, size: i32) -> bool {
    let scale = longest.powi(size);
    determinant.is_finite() && scale.is_finite() && determinant.abs() > scale * SINGULAR_RATIO
}

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

    /// The projection of a 3D context with the eye at distance `d` in front
    /// of `centre`, looking along +z: a point shows with w = 1 + z / d
    /// around the centre, and keeps z for the depth order.
    #[must_use]
    pub fn perspective([cx, cy]: [f64; 2], d: f64) -> Self {
        let mut t = Self::IDENTITY;
        t.m[8] = cx / d;
        t.m[9] = cy / d;
        t.m[11] = 1.0 / d;
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

    /// Whether the transform is a 2D translation with a positive scale, so
    /// that a node under it draws and hits without a texture of its own.
    #[must_use]
    pub fn is_plain(&self) -> bool {
        self.scale_2d().is_some_and(|(sx, sy)| sx > 0.0 && sy > 0.0)
    }

    /// The inverse, or `None` if the matrix is singular.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let m = DMat4::from_cols_array(&self.m);
        let longest = (0..3).map(|i| m.col(i).length()).fold(0.0, f64::max);
        if !invertible(m.determinant(), longest, 4) {
            return None;
        }
        let inverse = m.inverse().to_cols_array();
        inverse
            .iter()
            .all(|v| v.is_finite())
            .then_some(Self { m: inverse })
    }

    /// The inverse of the homography that maps a point `(u, v)` of the
    /// plane z = 0 to the screen point `(x / w, y / w)`, as a column-major
    /// 3x3 matrix. It takes `(x, y, w)` to `(u, v, s)` with the plane point
    /// `(u / s, v / s)`. It is `None` if the plane is edge on or the matrix
    /// is singular.
    #[must_use]
    pub fn plane_inverse(&self) -> Option<[f64; 9]> {
        let m = &self.m;
        let h = DMat3::from_cols(
            DVec3::new(m[0], m[1], m[3]),
            DVec3::new(m[4], m[5], m[7]),
            DVec3::new(m[12], m[13], m[15]),
        );
        let longest = (0..2).map(|i| h.col(i).length()).fold(0.0, f64::max);
        if !invertible(h.determinant(), longest, 3) {
            return None;
        }
        let inverse = h.inverse().to_cols_array();
        inverse.iter().all(|v| v.is_finite()).then_some(inverse)
    }

    /// The translation, rotation and scale of an affine matrix without
    /// shear, or `None` for a matrix that has a perspective part, a shear or
    /// a zero scale.
    #[must_use]
    pub fn decompose(&self) -> Option<Decomposed> {
        let matrix = DMat4::from_cols_array(&self.m);
        if matrix.row(3) != glam::DVec4::W || !matrix.is_finite() {
            return None;
        }
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        if scale.to_array().iter().any(|s| *s == 0.0 || !s.is_finite()) {
            return None;
        }
        let back = DMat4::from_scale_rotation_translation(scale, rotation, translation);
        let exact = back
            .to_cols_array()
            .iter()
            .zip(&self.m)
            .all(|(a, b)| (a - b).abs() <= DECOMPOSE_TOLERANCE * (1.0 + b.abs()));
        exact.then(|| Decomposed {
            translation: translation.to_array(),
            rotation: rotation.to_array(),
            scale: scale.to_array(),
        })
    }

    /// The column-major matrix.
    #[must_use]
    pub fn matrix(&self) -> &[f64; 16] {
        &self.m
    }

    /// The transform of a translation, a rotation and a scale.
    #[must_use]
    pub(crate) fn compose(parts: &Decomposed) -> Self {
        let rotation = DQuat::from_array(parts.rotation).normalize();
        let m = DMat4::from_scale_rotation_translation(
            DVec3::from_array(parts.scale),
            rotation,
            DVec3::from_array(parts.translation),
        );
        Self {
            m: m.to_cols_array(),
        }
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
    fn only_a_translation_with_a_positive_scale_is_plain() {
        for plain in [
            Transform::IDENTITY,
            Transform::translate(5.0, -3.0, 2.0),
            Transform::scale(2.0, 0.5, 1.0),
        ] {
            assert!(plain.is_plain(), "{plain:?}");
        }
        for not_plain in [
            Transform::rotate_z(30.0),
            Transform::scale(-1.0, 1.0, 1.0),
            Transform::scale(1.0, 0.0, 1.0),
            Transform::from_matrix([
                1.0, 0.0, 0.0, 0.01, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]),
        ] {
            assert!(!not_plain.is_plain(), "{not_plain:?}");
        }
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

    fn close(a: &[f64], b: &[f64]) {
        for (x, y) in a.iter().zip(b) {
            assert!((x - y).abs() < 1e-9, "{a:?} != {b:?}");
        }
    }

    #[test]
    fn inverse_undoes_translation_scale_and_rotation() {
        let t = Transform::translate(10.0, 20.0, 5.0)
            .then(&Transform::rotate_z(90.0).then(&Transform::scale(2.0, 4.0, 1.0)));
        let inverse = t.inverse().unwrap();
        close(inverse.then(&t).matrix(), Transform::IDENTITY.matrix());
        close(t.then(&inverse).matrix(), Transform::IDENTITY.matrix());
        let known = Transform::translate(10.0, 20.0, 0.0)
            .then(&Transform::scale(2.0, 4.0, 1.0))
            .inverse()
            .unwrap();
        assert_eq!(known.apply(12.0, 28.0), (1.0, 2.0));
    }

    #[test]
    fn singular_matrices_have_no_inverse() {
        assert_eq!(Transform::scale(0.0, 1.0, 1.0).inverse(), None);
        assert_eq!(Transform::from_matrix([0.0; 16]).plane_inverse(), None);
        assert_eq!(Transform::from_matrix([f64::NAN; 16]).inverse(), None);
    }

    #[test]
    fn plane_inverse_takes_a_projected_point_back_to_the_plane() {
        let mut matrix = *Transform::translate(100.0, 50.0, 0.0)
            .then(&Transform::rotate_z(30.0))
            .matrix();
        matrix[3] = 0.001;
        matrix[7] = 0.002;
        let inverse = Transform::from_matrix(matrix).plane_inverse().unwrap();
        let plane = [40.0, 25.0];
        let screen: Vec<f64> = [0, 1, 3]
            .iter()
            .map(|row| matrix[*row] * plane[0] + matrix[row + 4] * plane[1] + matrix[row + 12])
            .collect();
        let back: Vec<f64> = (0..3)
            .map(|row| (0..3).map(|k| inverse[k * 3 + row] * screen[k]).sum())
            .collect();
        close(&[back[0] / back[2], back[1] / back[2]], &plane);
    }

    #[test]
    fn plane_inverse_of_a_flat_plane_is_the_2d_inverse() {
        let t = Transform::translate(10.0, 20.0, 0.0).then(&Transform::scale(2.0, 4.0, 1.0));
        let inverse = t.plane_inverse().unwrap();
        close(&inverse, &[0.5, 0.0, 0.0, 0.0, 0.25, 0.0, -5.0, -5.0, 1.0]);
    }

    #[test]
    fn a_nearly_edge_on_plane_has_no_plane_inverse() {
        let (sin, cos) = 90f64.to_radians().sin_cos();
        let mut m = *Transform::IDENTITY.matrix();
        m[5] = cos;
        m[6] = sin;
        m[9] = -sin;
        m[10] = cos;
        assert_eq!(Transform::from_matrix(m).plane_inverse(), None);
        assert_eq!(Transform::scale(1.0, 1e-15, 1.0).inverse(), None);
        assert!(Transform::from_matrix(m).inverse().is_some());
    }

    #[test]
    fn an_edge_on_plane_has_no_plane_inverse() {
        let t = Transform::scale(1.0, 0.0, 1.0);
        assert_eq!(t.plane_inverse(), None);
    }

    #[test]
    fn decompose_reads_translation_rotation_and_scale() {
        let t = Transform::translate(1.0, 2.0, 3.0)
            .then(&Transform::rotate_z(90.0).then(&Transform::scale(2.0, 3.0, 4.0)));
        let parts = t.decompose().unwrap();
        close(&parts.translation, &[1.0, 2.0, 3.0]);
        close(&parts.scale, &[2.0, 3.0, 4.0]);
        let half = std::f64::consts::FRAC_1_SQRT_2;
        close(&parts.rotation, &[0.0, 0.0, half, half]);
        close(Transform::compose(&parts).matrix(), t.matrix());
    }

    #[test]
    fn decompose_keeps_a_mirror_as_a_negative_scale() {
        let parts = Transform::scale(-2.0, 3.0, 1.0).decompose().unwrap();
        assert_eq!(parts.scale.iter().filter(|s| **s < 0.0).count(), 1);
        let back = Transform::compose(&parts);
        close(back.matrix(), Transform::scale(-2.0, 3.0, 1.0).matrix());
    }

    #[test]
    fn decompose_refuses_shear_perspective_and_zero_scale() {
        let mut shear = *Transform::IDENTITY.matrix();
        shear[4] = 1.0;
        let mut perspective = *Transform::IDENTITY.matrix();
        perspective[3] = 0.001;
        for m in [shear, perspective] {
            assert_eq!(Transform::from_matrix(m).decompose(), None);
        }
        assert_eq!(Transform::scale(0.0, 1.0, 1.0).decompose(), None);
    }
}
