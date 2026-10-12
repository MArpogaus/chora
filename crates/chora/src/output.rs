//! Outputs, their layout and their cameras (D20).
//!
//! Outputs sit side by side in a layout of logical coordinates, as in
//! `wlr-output-management`. Each output shows the scene through a camera:
//! a transform from scene coordinates to the logical coordinates of the
//! output. The default camera shows the scene area at the layout position
//! of the output, so the layout and the scene coordinates agree.

use std::{collections::HashSet, ops::RangeInclusive, time::Duration};

use chora_scene::{
    Event, OutputInfo, Transform,
    ipc::{Edges, Reserve, RpcError},
};
use serde_json::Value;
use smithay::{
    desktop::{layer_map_for_output, space::SpaceElement},
    output::{Mode, Output, Scale},
    utils::{self, Logical, Physical, Point, Rectangle, Size},
};

use crate::{backend::Backend, ipc::rejected, state::State};

/// The scales that a client can give an output.
const SCALES: RangeInclusive<f64> = 0.1..=10.0;

/// The configuration of one output, as `wlr-output-management` asks for
/// it.
pub enum OutputConfig {
    /// Turn the output off.
    Disabled,
    /// Turn the output on with these changes.
    Enabled(OutputChanges),
}

/// The properties of an output that a client sets. A property without a
/// value stays as it is.
#[derive(Clone, Default)]
pub struct OutputChanges {
    pub mode: Option<ModeChoice>,
    pub position: Option<Point<i32, Logical>>,
    pub transform: Option<utils::Transform>,
    pub scale: Option<f64>,
}

/// A mode that a client asks for.
#[derive(Clone, Copy)]
pub enum ModeChoice {
    /// One of the modes of the output.
    Listed(Mode),
    /// A size, and a refresh rate in mHz or 0 for any rate.
    Custom {
        size: Size<i32, Physical>,
        refresh: i32,
    },
}

/// The state that a configuration gives an output.
struct Target {
    output: Output,
    /// The mode, or `None` for an output that turns off.
    mode: Option<Mode>,
    position: Point<i32, Logical>,
    transform: utils::Transform,
    scale: f64,
}

/// A change that the backend makes for a configuration.
enum Change {
    /// Turns an output off.
    Disable(Output),
    /// Gives an output that is on a new mode.
    Mode(Output, Mode),
    /// Turns an output on in a mode.
    Enable(Output, Mode),
}

impl State {
    /// Adds an output right of the others and announces it.
    pub fn add_output(&mut self, output: Output) {
        let right = self
            .outputs
            .iter()
            .map(|o| output_area(o).loc.x + output_area(o).size.w)
            .max()
            .unwrap_or(0);
        output.change_current_state(None, None, None, Some((right, 0).into()));
        self.output_management.add_head(&output, true);
        self.outputs.push(output);
        self.output_changed();
    }

    /// Removes an output that went away and announces it.
    pub fn remove_output(&mut self, output: &Output) {
        if self.outputs.contains(output) {
            self.leave_layout(output);
        }
        self.output_management.remove_head(output);
        self.output_management.update(&self.outputs);
        self.resize_lock_surfaces();
    }

    /// Takes an output out of the layout and announces it. A client can
    /// turn it on again through output configuration.
    fn leave_layout(&mut self, output: &Output) {
        self.outputs.retain(|o| o != output);
        // Layer surfaces of the output close; a client can open them again
        // on another output.
        let mut map = layer_map_for_output(output);
        for layer in map.layers().cloned().collect::<Vec<_>>() {
            layer.layer_surface().send_close();
            map.unmap_layer(&layer);
        }
        drop(map);
        for window in self.windows.values() {
            window.output_leave(output);
        }
        self.foreign_toplevel_manager_state
            .set_shown(output, &HashSet::new());
        self.emit(&Event::OutputRemoved {
            name: output.name(),
        });
    }

    /// Announces new modes, scales or positions of the outputs, to plugins
    /// and to the clients of output configuration.
    pub fn output_changed(&mut self) {
        for output in self.output_info() {
            self.emit(&Event::OutputChanged { output });
        }
        self.output_management.update(&self.outputs);
    }

    /// Tests a configuration of the outputs, and applies it if `apply`.
    /// The configuration applies as a whole: after an error, nothing
    /// changed. [`Self::outputs_configured`] announces the result.
    pub fn configure_outputs(
        &mut self,
        config: Vec<(Output, OutputConfig)>,
        apply: bool,
    ) -> Result<(), String> {
        let targets = config
            .into_iter()
            .map(|(output, config)| target(output, config))
            .collect::<Result<Vec<_>, _>>()?;
        let untouched = self
            .outputs
            .iter()
            .any(|o| targets.iter().all(|t| t.output != *o));
        if !untouched && targets.iter().all(|t| t.mode.is_none()) {
            return Err("one output must stay on".into());
        }
        let changes = changes(&self.outputs, &targets);
        let backend = self.backend.as_mut().ok_or("no backend")?;
        for (change, _) in &changes {
            change.test(backend)?;
        }
        if apply {
            run(backend, &changes)?;
            for target in &targets {
                self.place(target);
            }
        }
        Ok(())
    }

    /// Gives an output the mode, position, transform and scale of a
    /// target, and turns it on or off in the layout.
    fn place(&mut self, target: &Target) {
        let output = &target.output;
        let on = self.outputs.contains(output);
        let Some(mode) = target.mode else {
            if on {
                self.leave_layout(output);
            }
            return;
        };
        let scale = output.current_scale().fractional_scale();
        output.change_current_state(
            (output.current_mode() != Some(mode)).then_some(mode),
            (output.current_transform() != target.transform).then_some(target.transform),
            (scale.to_bits() != target.scale.to_bits()).then_some(Scale::Fractional(target.scale)),
            (output.current_location() != target.position).then_some(target.position),
        );
        if !on {
            self.outputs.push(output.clone());
        }
    }

    /// Places the layer surfaces again after a configuration and announces
    /// the outputs. The pointer moves into the nearest output.
    pub fn outputs_configured(&mut self) {
        for output in &self.outputs {
            layer_map_for_output(output).arrange();
        }
        self.update_preferred_scales();
        self.pointer_layout = self.clamp_to_outputs(self.pointer_layout);
        self.refresh_pointer();
        self.output_changed();
        self.update_focus();
    }

    /// The outputs as the API describes them.
    pub fn output_info(&self) -> Vec<OutputInfo> {
        self.outputs
            .iter()
            .map(|output| {
                let area = output_area(output);
                let camera = self.camera(output);
                let zone = layer_map_for_output(output).non_exclusive_zone();
                let zone = shrink(zone, self.ipc.reserved(&output.name()));
                OutputInfo {
                    name: output.name(),
                    position: [area.loc.x, area.loc.y],
                    size: [area.size.w, area.size.h],
                    scale: output.current_scale().fractional_scale(),
                    camera,
                    work_area: scene_area(&camera, zone),
                }
            })
            .collect()
    }

    /// `output.reserve`: the work area of an output leaves space at its
    /// edges until the connection closes. Answers with the output.
    pub fn ipc_reserve(&mut self, conn: u64, reserve: &Reserve) -> Result<Value, RpcError> {
        if self.ipc.name(conn).is_none() {
            return Err(rejected("send hello first"));
        }
        let output = match &reserve.output {
            Some(name) => self.outputs.iter().find(|o| o.name() == *name),
            None => self.outputs.first(),
        }
        .map(Output::name)
        .ok_or_else(|| rejected(format!("no output {:?}", reserve.output)))?;
        self.ipc.set_reserve(conn, output.clone(), reserve.edges);
        self.output_changed();
        let info = self.output_info().into_iter().find(|i| i.name == output);
        Ok(serde_json::json!(info))
    }

    /// The camera of an output: the one a client set, or the default.
    pub fn camera(&self, output: &Output) -> Transform {
        self.scene
            .camera(&output.name())
            .copied()
            .unwrap_or_else(|| {
                let loc = output_area(output).loc;
                Transform::translate(-f64::from(loc.x), -f64::from(loc.y), 0.0)
            })
    }

    /// The output at a point of the layout, and the point in output and in
    /// scene coordinates.
    pub fn spot(&self, layout: Point<f64, Logical>) -> Option<Spot> {
        let output = self
            .outputs
            .iter()
            .find(|o| output_area(o).to_f64().contains(layout))?;
        let local = layout - output_area(output).loc.to_f64();
        Some(Spot {
            output: output.clone(),
            local,
            scene: to_scene(&self.camera(output), local),
        })
    }

    /// Moves a layout point into the nearest output.
    pub fn clamp_to_outputs(&self, layout: Point<f64, Logical>) -> Point<f64, Logical> {
        let nearest = self
            .outputs
            .iter()
            .map(output_area)
            .min_by(|a, b| distance(layout, *a).total_cmp(&distance(layout, *b)));
        nearest.map_or(layout, |area| clamp(layout, area))
    }

    /// The area that the first output shows with its default camera: the
    /// area of the reset grid, as x, y, width and height.
    pub fn reset_area(&self) -> [f64; 4] {
        self.outputs.first().map_or([0.0; 4], |output| {
            let area = output_area(output);
            [area.loc.x, area.loc.y, area.size.w, area.size.h].map(f64::from)
        })
    }

    /// The scene area of the output that shows a scene point, or of the
    /// first output.
    pub fn scene_area_at(&self, point: Point<f64, Logical>) -> Rectangle<i32, Logical> {
        let areas: Vec<_> = self
            .outputs
            .iter()
            .map(|o| scene_area(&self.camera(o), Rectangle::from_size(output_area(o).size)))
            .map(|[x, y, w, h]| Rectangle::new((x, y).into(), (w, h).into()))
            .collect();
        areas
            .iter()
            .find(|a| a.to_f64().contains(point))
            .or(areas.first())
            .copied()
            .unwrap_or_default()
    }
}

/// A point of the layout on an output.
pub struct Spot {
    pub output: Output,
    /// The point in logical output coordinates.
    pub local: Point<f64, Logical>,
    /// The point in scene coordinates.
    pub scene: Point<f64, Logical>,
}

/// The time between two frames of the current mode.
pub fn frame_duration(output: &Output) -> Duration {
    let refresh = output.current_mode().map_or(60_000, |m| m.refresh);
    Duration::from_secs_f64(1000.0 / f64::from(refresh.max(1)))
}

/// The area of an output in the layout, in logical pixels.
pub fn output_area(output: &Output) -> Rectangle<i32, Logical> {
    let size = output.current_mode().map_or((0, 0).into(), |mode| {
        let size = mode
            .size
            .to_f64()
            .to_logical(output.current_scale().fractional_scale())
            .to_i32_round();
        output.current_transform().transform_size(size)
    });
    Rectangle::new(output.current_location(), size)
}

/// The state that a configuration gives an output. A property without a
/// value stays as it is.
fn target(output: Output, config: OutputConfig) -> Result<Target, String> {
    let OutputConfig::Enabled(changes) = config else {
        return Ok(Target {
            mode: None,
            position: output.current_location(),
            transform: output.current_transform(),
            scale: output.current_scale().fractional_scale(),
            output,
        });
    };
    let mode = match changes.mode {
        Some(choice) => pick_mode(&output, choice)?,
        None => output
            .current_mode()
            .or_else(|| output.preferred_mode())
            .ok_or_else(|| format!("{} has no mode", output.name()))?,
    };
    let scale = changes
        .scale
        .unwrap_or_else(|| output.current_scale().fractional_scale());
    if !SCALES.contains(&scale) {
        return Err(format!("the scale {scale} is outside of {SCALES:?}"));
    }
    let target = Target {
        mode: Some(mode),
        position: changes
            .position
            .unwrap_or_else(|| output.current_location()),
        transform: changes
            .transform
            .unwrap_or_else(|| output.current_transform()),
        scale,
        output,
    };
    target.check_area()?;
    Ok(target)
}

/// The mode of an output that a client asks for. A custom mode matches
/// the mode of the same size with the nearest refresh rate, at most 1 Hz
/// away. A custom refresh rate of 0 matches the highest rate.
fn pick_mode(output: &Output, choice: ModeChoice) -> Result<Mode, String> {
    let modes = output.modes();
    let (size, refresh) = match choice {
        ModeChoice::Listed(mode) => (mode.size, mode.refresh),
        ModeChoice::Custom { size, refresh } => (size, refresh),
    };
    let found = match choice {
        ModeChoice::Listed(mode) => modes.contains(&mode).then_some(mode),
        ModeChoice::Custom { refresh: 0, .. } => modes
            .iter()
            .filter(|m| m.size == size)
            .max_by_key(|m| m.refresh)
            .copied(),
        ModeChoice::Custom { .. } => modes
            .iter()
            .filter(|m| m.size == size && (m.refresh - refresh).abs() <= 1000)
            .min_by_key(|m| (m.refresh - refresh).abs())
            .copied(),
    };
    found.ok_or_else(|| {
        format!(
            "{} has no mode {}x{} at {refresh} mHz",
            output.name(),
            size.w,
            size.h
        )
    })
}

impl Change {
    /// Asks the backend whether it can make the change.
    fn test(&self, backend: &Backend) -> Result<(), String> {
        match self {
            Self::Disable(output) => backend.test_output(output, None),
            Self::Mode(output, mode) | Self::Enable(output, mode) => {
                backend.test_output(output, Some(*mode))
            }
        }
    }

    /// Makes the change in the backend.
    fn run(&self, backend: &mut Backend) -> Result<(), String> {
        match self {
            Self::Disable(output) => backend.disable_output(output),
            Self::Mode(output, mode) => backend.set_mode(output, *mode),
            Self::Enable(output, mode) => backend.enable_output(output, *mode),
        }
    }

    /// The order of the changes. Outputs turn off first, so that the others
    /// find free bandwidth.
    fn rank(&self) -> u8 {
        match self {
            Self::Disable(_) => 0,
            Self::Mode(..) => 1,
            Self::Enable(..) => 2,
        }
    }
}

/// The backend changes of a configuration, each with the change that
/// undoes it. `on` holds the outputs that are on.
fn changes(on: &[Output], targets: &[Target]) -> Vec<(Change, Option<Change>)> {
    let mut changes: Vec<_> = targets
        .iter()
        .filter_map(|t| change(on.contains(&t.output), t))
        .collect();
    changes.sort_by_key(|(change, _)| change.rank());
    changes
}

/// The backend change for one target and the change that undoes it, if
/// the target needs a change.
fn change(on: bool, target: &Target) -> Option<(Change, Option<Change>)> {
    let output = &target.output;
    let old = output.current_mode();
    let undo = |undo: fn(Output, Mode) -> Change| old.map(|m| undo(output.clone(), m));
    match (on, target.mode) {
        (true, None) => Some((Change::Disable(output.clone()), undo(Change::Enable))),
        (true, Some(mode)) if Some(mode) != old => {
            Some((Change::Mode(output.clone(), mode), undo(Change::Mode)))
        }
        (false, Some(mode)) => Some((
            Change::Enable(output.clone(), mode),
            Some(Change::Disable(output.clone())),
        )),
        _ => None,
    }
}

/// Makes the changes in the backend. After a failure, it undoes the
/// changes before it, the last one first.
fn run(backend: &mut Backend, changes: &[(Change, Option<Change>)]) -> Result<(), String> {
    for (i, (change, _)) in changes.iter().enumerate() {
        if let Err(err) = change.run(backend) {
            for undo in changes[..i].iter().rev().filter_map(|(_, u)| u.as_ref()) {
                undo_change(backend, undo);
            }
            return Err(err);
        }
    }
    Ok(())
}

/// Undoes a change. After a failure, the output keeps the new state.
fn undo_change(backend: &mut Backend, undo: &Change) {
    if let Err(err) = undo.run(backend) {
        tracing::warn!(%err, "undoing an output change failed");
    }
}

impl Target {
    /// Fails if the area of the output in the layout does not fit into
    /// 32-bit coordinates.
    fn check_area(&self) -> Result<(), String> {
        let size = self.mode.map_or_else(Size::default, |mode| {
            let size = mode.size.to_f64().to_logical(self.scale).to_i32_round();
            self.transform.transform_size(size)
        });
        let fits = self.position.x.checked_add(size.w).is_some()
            && self.position.y.checked_add(size.h).is_some();
        if fits {
            Ok(())
        } else {
            Err(format!("{} lies outside of the layout", self.output.name()))
        }
    }
}

/// Maps a point of the output to scene coordinates. Cameras are 2D
/// translations with a positive scale, so the inverse is simple.
pub fn to_scene(camera: &Transform, local: Point<f64, Logical>) -> Point<f64, Logical> {
    let (sx, sy) = camera.scale_2d().unwrap_or((1.0, 1.0));
    let (tx, ty) = camera.translation_2d();
    ((local.x - tx) / sx, (local.y - ty) / sy).into()
}

/// The scene rectangle that a camera shows in an area of the output.
pub fn scene_rect(camera: &Transform, local: Rectangle<i32, Logical>) -> Rectangle<f64, Logical> {
    let local = local.to_f64();
    let origin = to_scene(camera, local.loc);
    let end = to_scene(camera, local.loc + local.size.to_point());
    Rectangle::new(origin, (end - origin).to_size())
}

/// [`scene_rect`], rounded, as x, y, width and height.
#[allow(clippy::cast_possible_truncation)]
fn scene_area(camera: &Transform, local: Rectangle<i32, Logical>) -> [i32; 4] {
    let r = scene_rect(camera, local);
    [r.loc.x, r.loc.y, r.size.w, r.size.h].map(|v| v.round() as i32)
}

fn distance(point: Point<f64, Logical>, area: Rectangle<i32, Logical>) -> f64 {
    let nearest = clamp(point, area);
    (nearest.x - point.x).hypot(nearest.y - point.y)
}

/// The nearest point inside an area.
fn clamp(point: Point<f64, Logical>, area: Rectangle<i32, Logical>) -> Point<f64, Logical> {
    let area = area.to_f64();
    let max = area.loc + area.size.to_point() - Point::from((1.0, 1.0));
    (
        point.x.clamp(area.loc.x, max.x.max(area.loc.x)),
        point.y.clamp(area.loc.y, max.y.max(area.loc.y)),
    )
        .into()
}

/// A zone without space at its edges; at least one pixel stays.
fn shrink(zone: Rectangle<i32, Logical>, edges: Edges) -> Rectangle<i32, Logical> {
    let px = |v: u32| i32::try_from(v).unwrap_or(i32::MAX);
    // The space of the start edge, then of the end edge, within a side.
    let side = |side: i32, start: u32, end: u32| {
        let start = px(start).min(side - 1).max(0);
        let end = px(end).min(side - 1 - start).max(0);
        (start, side - start - end)
    };
    if zone.size.w < 1 || zone.size.h < 1 {
        return zone;
    }
    let (left, w) = side(zone.size.w, edges.left, edges.right);
    let (top, h) = side(zone.size.h, edges.top, edges.bottom);
    Rectangle::new((zone.loc.x + left, zone.loc.y + top).into(), (w, h).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserves_shrink_the_zone_and_keep_a_pixel() {
        let zone = Rectangle::new((0, 30).into(), (1280, 770).into());
        let edges = Edges {
            top: 40,
            left: 100,
            ..Edges::default()
        };
        assert_eq!(
            shrink(zone, edges),
            Rectangle::new((100, 70).into(), (1180, 730).into())
        );
        let all = Edges {
            top: u32::MAX,
            right: 5000,
            bottom: 1,
            left: 2000,
        };
        assert_eq!(
            shrink(zone, all),
            Rectangle::new((1279, 799).into(), (1, 1).into())
        );
    }
}
