//! The methods tiling exports to other clients, as `methods.list` answers them.

use chora_kit::{give, runtime};
use chora_scene::ipc::ExportMethod;
use serde_json::{Value, json};

/// The layout values that `set_layout` accepts.
pub(super) fn layout_schema() -> Value {
    json!({
        "enum": [
            "tile", "tile_left", "tile_bottom", "tile_top", "fair",
            "fair_horizontal", "spiral", "dwindle", "max", "fullscreen",
            "magnifier", "floating",
        ],
        "description": "The layout of the current tag.",
    })
}

/// The methods that other clients call, with description and schema of
/// their parameters, as `methods.list` answers them.
pub(super) fn exported_methods() -> Vec<ExportMethod> {
    let mut methods = layout_methods();
    methods.extend(window_methods());
    methods
}

/// The methods about layouts, tags and the key list.
pub(super) fn layout_methods() -> Vec<ExportMethod> {
    let field = |ty: &str, description: &str| json!({"type": ty, "description": description});
    vec![
        runtime::method(
            "set_layout",
            "Sets the layout and the gap of the current tag.",
            json!({
                "layout": layout_schema(),
                "gap": field("integer", "The gap in logical pixels."),
            }),
            &[],
        ),
        runtime::method(
            "show_keys",
            "Shows the list of all keys, or hides it, as the binding does.",
            json!({}),
            &[],
        ),
        runtime::privileged(runtime::method(
            "show_helper",
            "Opens the chat with the helper, as MOD+a does: the click on the helper word of the bar calls it, so that the field gets the keyboard.",
            json!({}),
            &[],
        )),
        runtime::method(
            "hide_keys",
            "Hides the list of all keys when it shows; nothing otherwise. The launcher and the power menu call it before they open.",
            json!({}),
            &[],
        ),
        runtime::method(
            "next_layout",
            "Goes to the next layout of the current tag, as the binding does.",
            json!({}),
            &[],
        ),
        runtime::method(
            "get_layout",
            "Answers the layout of the current tag.",
            json!({}),
            &[],
        ),
        runtime::method(
            "get_tags",
            "Answers the tags of each output: shown, focused, occupied and urgent.",
            json!({}),
            &[],
        ),
        give::method(),
        runtime::state_method(),
    ]
}

/// The methods about tags and windows.
pub(super) fn window_methods() -> Vec<ExportMethod> {
    let field = |ty: &str, description: &str| json!({"type": ty, "description": description});
    let window = || field("string", "The surface id of the window.");
    let tag = || field("integer", "The tag number, from 1.");
    let on_surface = json!({"surface": window()});
    vec![
        runtime::method(
            "view_tag",
            "Shows a tag on the current output.",
            json!({"tag": tag()}),
            &["tag"],
        ),
        runtime::method(
            "move_to_tag",
            "Moves a window to a tag.",
            json!({"surface": window(), "tag": tag()}),
            &["surface", "tag"],
        ),
        runtime::method(
            "focus",
            "Focuses a window.",
            on_surface.clone(),
            &["surface"],
        ),
        runtime::method("focus_next", "Focuses the next window.", json!({}), &[]),
        runtime::method("focus_prev", "Focuses the previous window.", json!({}), &[]),
        runtime::method(
            "swap_master",
            "Swaps a window into the master slot.",
            on_surface,
            &["surface"],
        ),
    ]
}

#[cfg(test)]
mod tests_moved;
