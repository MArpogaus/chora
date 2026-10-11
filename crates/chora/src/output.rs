//! Outputs, their layout and their cameras (D20).
//!
//! Outputs sit side by side in a layout of logical coordinates, as in
//! `wlr-output-management`. Each output shows the scene through a camera:
//! a transform from scene coordinates to the logical coordinates of the
//! output. The default camera shows the scene area at the layout position
//! of the output, so the layout and the scene coordinates agree.

use chora_scene::{Event, OutputInfo, Transform};
use smithay::{
    desktop::{layer_map_for_output, space::SpaceElement},
    output::Output,
    utils::{Logical, Point, Rectangle},
};

use crate::state::State;

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
        self.outputs.push(output);
        self.output_changed();
    }

    /// Removes an output and announces it.
    pub fn remove_output(&mut self, output: &Output) {
        self.outputs.retain(|o| o != output);
        for window in self.windows.values() {
            window.output_leave(output);
        }
        self.emit(&Event::OutputRemoved {
            name: output.name(),
        });
    }

    /// Announces new modes, scales or positions of the outputs.
    pub fn output_changed(&mut self) {
        for output in self.output_info() {
            self.emit(&Event::OutputChanged { output });
        }
    }

    /// The outputs as the API describes them.
    pub fn output_info(&self) -> Vec<OutputInfo> {
        self.outputs
            .iter()
            .map(|output| {
                let area = output_area(output);
                let camera = self.camera(output);
                let zone = layer_map_for_output(output).non_exclusive_zone();
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

/// Maps a point of the output to scene coordinates. Cameras are 2D
/// translations with a positive scale, so the inverse is simple.
pub fn to_scene(camera: &Transform, local: Point<f64, Logical>) -> Point<f64, Logical> {
    let (sx, sy) = camera.scale_2d().unwrap_or((1.0, 1.0));
    let (tx, ty) = camera.translation_2d();
    ((local.x - tx) / sx, (local.y - ty) / sy).into()
}

/// The scene area that a camera shows in an area of the output, as x, y,
/// width and height.
#[allow(clippy::cast_possible_truncation)]
fn scene_area(camera: &Transform, local: Rectangle<i32, Logical>) -> [i32; 4] {
    let local = local.to_f64();
    let origin = to_scene(camera, local.loc);
    let end = to_scene(camera, local.loc + local.size.to_point());
    [origin.x, origin.y, end.x - origin.x, end.y - origin.y].map(|v| v.round() as i32)
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
