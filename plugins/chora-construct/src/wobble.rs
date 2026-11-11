//! What follows the wobble of a pane, so that the pane bends as one
//! thing: its mirror image in the floor, the glow behind the focused
//! pane, and its shadow on the floor. The driver of `chora-kit` bends the
//! group of the pane; the hairline of the focused pane is part of that
//! group.
use chora_scene::Mesh;

use super::{
    ClientError, Command, Construct, FOCUS, FOCUS_SCALE, Pane, PropsUpdate, mirror_size, space,
    update,
};

/// The mesh of the mirror image of a pane with the mesh `pane`: the same
/// bend on a node of the pane's size divided by `k` along each axis.
pub(crate) fn mirror_mesh(pane: &Mesh, k: [f64; 2]) -> Mesh {
    Mesh {
        points: (pane.points.iter())
            .map(|p| [p[0] / k[0], p[1] / k[1]])
            .collect(),
        ..pane.clone()
    }
}

/// How many pixels of a pane of a size one pixel of its mirror image
/// shows, along x and along y.
pub(crate) fn mirror_scale(size: [f64; 2]) -> [f64; 2] {
    let k = size[0] / mirror_size(size)[0];
    [k, k]
}

/// The pane whose mesh the glow node has.
#[allow(dead_code, reason = "the frames of the wobble use it")]
#[derive(Default)]
pub(crate) struct GlowBend {
    pane: Option<String>,
}

#[allow(
    dead_code,
    clippy::unused_self,
    reason = "the frames of the wobble use it"
)]
impl GlowBend {
    /// The glow takes the mesh of a pane.
    pub(crate) fn bends(&mut self, pane: &str) {
        self.pane = Some(pane.into());
    }

    /// A pane rests again: whether the glow must become flat.
    pub(crate) fn rests(&mut self, pane: &str) -> bool {
        let _ = pane;
        true
    }

    /// The focus is on a pane or on none: whether the glow must become
    /// flat, because it has the mesh of another pane.
    pub(crate) fn focus(&mut self, pane: Option<&str>) -> bool {
        let _ = pane;
        false
    }
}

/// The mesh of the glow node behind a pane, from the mesh of the pane
/// that reaches `reach` past its edges: the node is the pane plus the
/// reach on each side, divided by `k`.
pub(crate) fn glow_mesh(around: &Mesh, reach: f64, k: f64) -> Mesh {
    let points = around.points.iter();
    Mesh {
        points: points.map(|p| p.map(|v| (v + reach) / k)).collect(),
        ..around.clone()
    }
}

/// How far a bent pane of a size is from its place on average: along its
/// face and along its normal.
#[allow(clippy::cast_precision_loss, reason = "at most 65 points a side")]
pub(crate) fn drift(mesh: &Mesh, size: [f64; 2]) -> [f64; 2] {
    let count = mesh.points.len().max(1) as f64;
    let along: f64 = mesh.points.iter().map(|p| p[0]).sum::<f64>() / count - size[0] / 2.0;
    let deep: f64 = mesh.depth.iter().sum::<f64>() / count;
    [along, deep]
}

/// The place on the floor of a pane at `at` with the turn `yaw` that
/// drifted from its place, for its shadow.
pub(crate) fn drifted(at: [f64; 2], yaw: f64, [along, deep]: [f64; 2]) -> [f64; 2] {
    let (sin, cos) = yaw.sin_cos();
    [
        at[0] + along * cos + deep * sin,
        at[1] - along * sin + deep * cos,
    ]
}

fn bend(node: &str, mesh: Mesh) -> Command {
    update(
        node,
        PropsUpdate {
            mesh: Some(mesh),
            ..PropsUpdate::default()
        },
    )
}

fn flat() -> Mesh {
    Mesh {
        cols: 1,
        rows: 1,
        points: Vec::new(),
        depth: Vec::new(),
    }
}

impl Construct {
    /// How far the glow of a pane reaches past its edges, in the pixels
    /// of the pane.
    pub(crate) fn glow_reach(&self, pane: &Pane) -> f64 {
        let px = space::depth(&self.pose, pane.at).max(self.lens.focal * 0.1) / self.lens.focal;
        self.settings.value.style.glow * px
    }

    /// The place of the shadow of a pane: under the pane as it bends.
    pub(crate) fn shadow_at(&self, pane: &Pane) -> [f64; 2] {
        let by = self.drifts.get(&pane.name).copied().unwrap_or_default();
        drifted(pane.at, pane.yaw, by)
    }

    /// The mesh updates of the panes from the driver, with the updates of
    /// what follows each pane: its mirror image and, for the focused
    /// pane, the glow. The shadow takes the drift with the next frame.
    pub(crate) fn followed(&mut self, commands: Vec<Command>) -> Vec<Command> {
        let focus = self.focus().map(|k| self.panes[k].name.clone());
        let mut all = Vec::with_capacity(commands.len() * 3);
        for command in commands {
            let bent = match &command {
                Command::Update { id, props } => {
                    (props.mesh.clone()).zip(self.panes.iter().position(|p| p.node() == *id))
                }
                _ => None,
            };
            all.push(command);
            let Some((mesh, index)) = bent else {
                continue;
            };
            let pane = &self.panes[index];
            let (name, mirror) = (pane.name.clone(), pane.mirror());
            if mesh.points.is_empty() {
                self.drifts.remove(&name);
                all.push(bend(&mirror, flat()));
                if !self.glass.plain {
                    all.push(bend(FOCUS, flat()));
                }
                continue;
            }
            all.push(bend(&mirror, mirror_mesh(&mesh, mirror_scale(pane.size))));
            let reach = self.glow_reach(pane);
            let glows = !self.glass.plain && focus.as_deref() == Some(name.as_str());
            if let Some(around) = self.effects.mesh_around(&name, reach).filter(|_| glows) {
                all.push(bend(FOCUS, glow_mesh(&around, reach, FOCUS_SCALE)));
            }
            self.drifts.insert(name, drift(&mesh, pane.size));
        }
        self.drifts
            .retain(|name, _| self.panes.iter().any(|p| p.name == *name));
        all
    }

    /// One frame of the wobbles of the panes, with what follows them.
    pub(crate) fn wobble_frame(&mut self) -> Result<(), ClientError> {
        let panes = &self.panes;
        let frame = self.effects.tick(chora_kit::clock::now(), |name| {
            panes.iter().find(|p| p.name == name).map(wobble_target)
        });
        if frame.commands.is_empty() {
            return Ok(());
        }
        let commands = self.followed(frame.commands);
        self.rt.conn.try_apply(commands)?;
        self.frame()
    }
}

/// The node that the wobble of a pane bends: the group of the pane, in
/// its own coordinates.
pub(crate) fn wobble_target(pane: &Pane) -> chora_kit::effects::Target {
    chora_kit::effects::Target {
        node: pane.node(),
        origin: [0.0; 2],
        size: pane.size,
        set_size: true,
    }
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
        let mirror = mirror_mesh(&bent(), [2.0, 2.0]);
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

    #[test]
    fn the_mirror_image_of_an_odd_pane_bends_by_its_own_scale_per_axis() {
        let [kx, ky] = mirror_scale([400.0, 301.0]);
        assert!((kx - 2.0).abs() < 1e-12, "{kx}");
        assert!((ky - 301.0 / 151.0).abs() < 1e-12, "{ky}");
    }

    #[test]
    fn the_glow_becomes_flat_only_when_its_own_pane_rests_or_loses_the_focus() {
        let mut glow = GlowBend::default();
        assert!(!glow.rests("a"), "the glow has no mesh");
        glow.bends("a");
        assert!(!glow.rests("b"), "another pane rests");
        assert!(!glow.focus(Some("a")), "the focus stays");
        assert!(glow.focus(Some("b")), "the focus moves to another pane");
        assert!(!glow.focus(Some("b")), "flat already");
        glow.bends("a");
        assert!(glow.focus(None), "no pane has the focus");
        glow.bends("a");
        assert!(glow.rests("a"), "its own pane rests");
        assert!(!glow.rests("a"), "flat already");
    }
}
