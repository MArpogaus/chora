//! The exported `construct.*` methods and their parameters.

use super::{Deserialize, ExportMethod, give, json, runtime};

/// Parameters with a name.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Named {
    pub(crate) name: String,
}

/// Parameters of `pane.move`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MovePane {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) right: f64,
    #[serde(default)]
    pub(crate) ahead: f64,
}

/// Parameters of `pane.manager`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagerPane {
    pub(crate) manager: String,
    #[serde(default)]
    pub(crate) size: Option<[f64; 2]>,
}

/// Parameters of `cube.open`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenCube {
    #[serde(default = "four")]
    pub(crate) faces: usize,
}

fn four() -> usize {
    4
}

/// Parameters of `cube.turn`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TurnCube {
    pub(crate) by: i32,
    #[serde(default)]
    pub(crate) ms: Option<u64>,
}

/// Parameters of `pane.resize`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResizePane {
    pub(crate) name: String,
    pub(crate) scale: f64,
}

/// The methods; those that start programs or write files are privileged
/// (D62).
#[allow(clippy::too_many_lines, reason = "one entry per method")]
pub(crate) fn exported_methods() -> Vec<ExportMethod> {
    let name = || json!({"name": {"type": "string"}});
    vec![
        runtime::privileged(runtime::method(
            "construct.load",
            "Loads the construct NAME from ~/.config/chora/constructs/NAME.yaml and starts \
             its programs; beside another flavor, the construct takes the output.",
            name(),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "construct.save",
            "Saves the space as the construct NAME: the camera, the radius and the panes.",
            name(),
            &["name"],
        )),
        runtime::privileged(runtime::method(
            "cube.open",
            "Opens a cube of FACES virtual outputs, 4 by default, with one xp on them: each \
             output shows on a face, whose windows take the pointer. It goes when xp ends \
             or does not connect within 10 s.",
            json!({"faces": {"type": "integer", "minimum": 1, "maximum": 4}}),
            &[],
        )),
        runtime::method(
            "cube.turn",
            "Turns the cube BY quarter turns, positive to the right, in MS milliseconds, \
             500 a quarter by default; ignored while a turn runs.",
            json!({"by": {"type": "integer"}, "ms": {"type": "integer", "minimum": 0}}),
            &["by"],
        ),
        runtime::method(
            "cube.close",
            "Closes the cube: its xp ends, and its outputs and faces go.",
            json!({}),
            &[],
        ),
        runtime::method(
            "view.step_back",
            "Stands back from the front face of the cube, into navigate mode: the cube \
             shows whole, no face takes input, and the construct has the keyboard.",
            json!({}),
            &[],
        ),
        runtime::method(
            "view.step_in",
            "Steps into the front face of the cube, into use mode: the face shows at 1:1 \
             and takes the input.",
            json!({}),
            &[],
        ),
        runtime::method(
            "construct.list",
            "Answers the constructs, the loaded one, and its panes with their names.",
            json!({}),
            &[],
        ),
        runtime::method(
            "construct.leave",
            "Leaves a construct that was loaded beside another flavor: the output and \
             the keyboard go back.",
            json!({}),
            &[],
        ),
        runtime::method(
            "pane.move",
            "Moves the pane NAME by an offset from where it is: right and ahead, in \
             logical pixels, as the camera looks; negative values move it left or \
             nearer. Moves add up: right 1800 and then right -1800 bring it back. \
             The leash radius stops it. Answers its new place.",
            json!({
                "name": {"type": "string"},
                "right": {"type": "number"},
                "ahead": {"type": "number"},
            }),
            &["name"],
        ),
        runtime::method(
            "pane.resize",
            "Resizes a pane by its name: scale 1.25 makes it a quarter larger.",
            json!({"name": {"type": "string"}, "scale": {"type": "number"}}),
            &["name", "scale"],
        ),
        runtime::privileged(runtime::method(
            "pane.manager",
            "Adds a pane with a window manager: a virtual output of SIZE, 1024x768 by \
             default, with the MANAGER of the setting managers on it, such as xp or \
             tiling. Its windows take clicks through the pane. One pane per manager.",
            json!({
                "manager": {"type": "string"},
                "size": {"type": "array", "items": {"type": "number"},
                    "minItems": 2, "maxItems": 2},
            }),
            &["manager"],
        )),
        runtime::method(
            "pane.remove",
            "Removes a pane by its name; its window gets a close request.",
            name(),
            &["name"],
        ),
        runtime::method(
            "window.list",
            "Answers all windows: the label, the surface, the owner and the pane, which \
             is shown, behind, far, aside or none. Panes that do not show and windows \
             without a pane are the window list of the key Tab.",
            json!({}),
            &[],
        ),
        runtime::method(
            "window.show",
            "Flies to the window NAME (a label or a surface of window.list); a window of \
             another client becomes a pane first, and goes back to it when the pane or \
             the construct goes.",
            name(),
            &["name"],
        ),
        give::method(),
        runtime::method(
            "help.toggle",
            "Shows or hides the help panel with the key bindings of the construct; \
             answers whether it shows.",
            json!({}),
            &[],
        ),
        runtime::method(
            "pane.fly",
            "Flies into a pane by its name, until it covers the output at 1:1.",
            name(),
            &["name"],
        ),
    ]
}
