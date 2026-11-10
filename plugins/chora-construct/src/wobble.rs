//! What follows the wobble of a pane, so that the pane bends as one
//! thing: its mirror image in the floor, the glow behind the focused
//! pane, and its shadow on the floor. The driver of `chora-kit` bends the
//! group of the pane; the hairline of the focused pane is part of that
//! group.
#![cfg_attr(
    not(test),
    expect(dead_code, reason = "the frames of the wobble call them")
)]

use chora_scene::Mesh;

/// The mesh of the mirror image of a pane with the mesh `pane`: the same
/// bend on a node of the pane's size divided by `k`.
pub(crate) fn mirror_mesh(pane: &Mesh, k: f64) -> Mesh {
    let _ = k;
    pane.clone()
}

/// The mesh of the glow node behind a pane, from the mesh of the pane
/// that reaches `reach` past its edges: the node is the pane plus the
/// reach on each side, divided by `k`.
pub(crate) fn glow_mesh(around: &Mesh, reach: f64, k: f64) -> Mesh {
    let _ = (reach, k);
    around.clone()
}

/// How far a bent pane of a size is from its place on average: along its
/// face and along its normal.
pub(crate) fn drift(mesh: &Mesh, size: [f64; 2]) -> [f64; 2] {
    let _ = (mesh, size);
    [0.0; 2]
}

/// The place on the floor of a pane at `at` with the turn `yaw` that
/// drifted from its place, for its shadow.
pub(crate) fn drifted(at: [f64; 2], yaw: f64, drift: [f64; 2]) -> [f64; 2] {
    let _ = (yaw, drift);
    at
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bent() -> Mesh {
        let mut mesh = Mesh::flat(2, 1, [400.0, 300.0]);
        for point in &mut mesh.points {
            point[0] -= 30.0;
        }
        mesh.depth = vec![12.0; 6];
        mesh
    }

    #[test]
    fn the_mirror_image_bends_with_its_pane() {
        let mirror = mirror_mesh(&bent(), 2.0);
        assert!(mirror.is_valid());
        assert_eq!(mirror.points[0], [-15.0, 0.0]);
        assert_eq!(mirror.points[5], [185.0, 150.0]);
        assert_eq!(mirror.depth, vec![12.0; 6]);
    }

    #[test]
    fn the_glow_bends_with_its_pane() {
        let mut around = Mesh::flat(2, 1, [432.0, 332.0]);
        for point in &mut around.points {
            *point = [point[0] - 16.0 - 30.0, point[1] - 16.0];
        }
        around.depth = vec![12.0; 6];
        let glow = glow_mesh(&around, 16.0, 4.0);
        assert_eq!(glow.points[0], [-7.5, 0.0]);
        assert_eq!(glow.points[5], [100.5, 83.0]);
        assert_eq!(glow.depth, vec![12.0; 6]);
    }

    #[test]
    fn the_shadow_goes_where_the_bent_pane_is() {
        let by = drift(&bent(), [400.0, 300.0]);
        assert_eq!(by, [-30.0, 12.0]);
        let near = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-9;
        assert!(near(drifted([100.0, 2000.0], 0.0, by), [70.0, 2012.0]));
        let turned = drifted([100.0, 2000.0], std::f64::consts::FRAC_PI_2, by);
        assert!(near(turned, [112.0, 2030.0]), "{turned:?}");
    }
}
