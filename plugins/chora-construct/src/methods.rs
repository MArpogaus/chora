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
        chora_kit::clock::method(),
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
