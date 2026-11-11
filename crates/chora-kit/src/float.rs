//! Floating windows: a window manager on a virtual output lets the
//! client that shows the output on a shape, such as a face of a cube,
//! lift its windows off the face. The window manager exports the method
//! `windows.float`: while an output floats, it hides the groups of its
//! windows there with opacity 0, so that the output shows the desktop
//! without them, and names the groups. A node with opacity 0 still shows
//! in a node texture, so the caller shows each group on a plane of its
//! own, in front of the face. The window manager emits the event
//! `NAME.floating` when the windows of a floating output change.

use chora_scene::ipc::ExportMethod;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::runtime;

/// The name of the method.
pub const METHOD: &str = "windows.float";
/// The most windows of an output that float, the top ones of the stack;
/// the others stay in the output.
pub const MAX: usize = 12;
/// How far the top window lifts off its face, in logical pixels.
pub const LIFT: f64 = 160.0;

/// The parameters of `windows.float`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Float {
    /// The name of a virtual output of the window manager.
    pub output: String,
    /// Whether the windows of the output float from now on.
    pub on: bool,
}

/// A window that floats: the node that shows it, and where it lies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floating {
    /// The node of the window with its frame; it has a size.
    pub node: String,
    /// The surface id of the window.
    pub surface: String,
    /// The node on its output: x, y, width and height in logical pixels.
    pub rect: [f64; 4],
}

/// The answer of `windows.float` and the data of the event: the windows
/// of an output that float, the lowest of the stack first.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floated {
    /// The output.
    pub output: String,
    /// Its floating windows; none while the output does not float.
    pub windows: Vec<Floating>,
}

/// The export of `windows.float`, with its description and schema. It is
/// privileged: it hides windows in the output that a person may look at.
#[must_use]
pub fn method() -> ExportMethod {
    runtime::privileged(runtime::method(
        METHOD,
        "Lets the windows of a virtual output float: while ON, their groups have opacity 0 \
         in the output and the answer names them, the lowest first, for node textures.",
        json!({
            "output": {"type": "string", "description": "The name of the virtual output."},
            "on": {"type": "boolean", "description": "Whether its windows float."},
        }),
        &["output", "on"],
    ))
}

/// The name of the event of a window manager with the client name
/// `client`.
#[must_use]
pub fn event(client: &str) -> String {
    format!("{client}.floating")
}

/// The windows that float of those of an output, the lowest first: the
/// top `MAX`.
#[must_use]
pub fn top(mut windows: Vec<Floating>) -> Vec<Floating> {
    let below = windows.len().saturating_sub(MAX);
    windows.split_off(below)
}

/// How far window `index` of `count`, the lowest first, is off its face
/// at a `phase` from 0, flat, to 1: higher windows of the stack float
/// farther.
#[must_use]
pub fn lift(phase: f64, index: usize, count: usize) -> f64 {
    #[allow(clippy::cast_precision_loss, reason = "a dozen windows")]
    let share = index as f64 / count.saturating_sub(1).max(1) as f64;
    LIFT * (0.6 + 0.4 * share) * phase.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(i: usize) -> Floating {
        Floating {
            node: format!("win-{i}"),
            surface: format!("s{i}"),
            rect: [0.0, 0.0, 10.0, 10.0],
        }
    }

    #[test]
    fn higher_windows_float_farther_and_all_land() {
        assert!(lift(1.0, 0, 3) < lift(1.0, 1, 3));
        assert!(lift(1.0, 1, 3) < lift(1.0, 2, 3));
        assert!((lift(1.0, 2, 3) - LIFT).abs() < 1e-9);
        assert!(lift(1.0, 0, 1) > 0.0, "one window floats too");
        assert!(lift(0.0, 2, 3).abs() < 1e-9, "flat at rest");
        assert!(lift(0.5, 2, 3) < lift(1.0, 2, 3));
    }

    #[test]
    fn the_top_windows_float() {
        let all: Vec<Floating> = (0..MAX + 3).map(window).collect();
        let floating = top(all);
        assert_eq!(floating.len(), MAX);
        assert_eq!(floating[0].surface, "s3", "the lowest three stay");
        assert_eq!(top(vec![window(0)]).len(), 1);
    }

    #[test]
    fn the_method_is_privileged_and_the_event_has_the_client_name() {
        let ExportMethod::Described(method) = method() else {
            panic!("a described method");
        };
        assert!(method.privileged);
        assert_eq!(method.name, METHOD);
        assert_eq!(event("xp"), "xp.floating");
    }

    #[test]
    fn unknown_parameters_are_refused() {
        let asked = json!({"output": "VIRTUAL-1", "on": true, "other": 1});
        assert!(serde_json::from_value::<Float>(asked).is_err());
    }
}
