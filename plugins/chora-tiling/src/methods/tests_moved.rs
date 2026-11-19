//! Unit tests of the exported methods of tiling: the names, the required
//! parameters and the layout schema that `methods.list` answers.

use chora_scene::ipc::{DescribedMethod, ExportMethod};
use serde_json::{Value, json};

use super::{exported_methods, layout_schema};

/// The described form of the method of the name `name`.
fn find<'a>(methods: &'a [ExportMethod], name: &str) -> &'a DescribedMethod {
    methods
        .iter()
        .find_map(|m| match m {
            ExportMethod::Described(d) if d.name == name => Some(d),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no described method {name}"))
}

/// The properties of the input schema of a method.
fn properties(method: &DescribedMethod) -> Value {
    method.input_schema.as_ref().expect("a schema")["properties"].clone()
}

/// The names of the required parameters of a method.
fn required(method: &DescribedMethod) -> Vec<String> {
    let names = method.input_schema.as_ref().expect("a schema")["required"].clone();
    names
        .as_array()
        .expect("a list of names")
        .iter()
        .map(|n| n.as_str().expect("a name").to_owned())
        .collect()
}

/// The layout schema names the twelve layouts, in the order of the
/// binding that cycles them.
#[test]
fn the_layout_schema_names_the_twelve_layouts_in_order() {
    assert_eq!(
        layout_schema()["enum"],
        json!([
            "tile",
            "tile_left",
            "tile_bottom",
            "tile_top",
            "fair",
            "fair_horizontal",
            "spiral",
            "dwindle",
            "max",
            "fullscreen",
            "magnifier",
            "floating",
        ])
    );
    assert_eq!(
        layout_schema()["description"],
        json!("The layout of the current tag.")
    );
}

/// Each exported method has its own name, the layout methods come
/// first and the state comes last.
#[test]
fn the_exported_methods_are_named_once_each() {
    let names: Vec<String> = exported_methods()
        .iter()
        .map(|m| match m {
            ExportMethod::Described(d) => d.name.clone(),
            ExportMethod::Plain(name) => name.clone(),
        })
        .collect();
    assert_eq!(
        names,
        [
            "set_layout",
            "show_keys",
            "next_layout",
            "get_layout",
            "get_tags",
            "window.give",
            "state.get",
            "view_tag",
            "move_to_tag",
            "focus",
            "focus_next",
            "focus_prev",
            "swap_master",
        ]
    );
    let mut unique = names.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), names.len(), "a name is exported twice");
}

/// Every exported method describes itself: a description in words and
/// an object schema for its parameters.
#[test]
fn every_exported_method_describes_itself_with_an_object_schema() {
    for method in exported_methods() {
        let ExportMethod::Described(d) = method else {
            panic!("a method without its description");
        };
        assert!(
            d.description
                .as_deref()
                .is_some_and(|text| !text.is_empty()),
            "{} has no description",
            d.name
        );
        let schema = d.input_schema.as_ref().expect("a schema");
        assert_eq!(schema["type"], json!("object"), "{}", d.name);
    }
}

/// `set_layout` takes a layout from the schema and a gap as an integer,
/// and requires neither: a call without them keeps the layout and gap.
#[test]
fn set_layout_takes_a_layout_and_a_gap_and_requires_neither() {
    let methods = exported_methods();
    let set = find(&methods, "set_layout");
    assert_eq!(required(set), Vec::<String>::new());
    let props = properties(set);
    assert_eq!(props["layout"], layout_schema());
    assert_eq!(props["gap"]["type"], json!("integer"));
    assert_eq!(
        props["gap"]["description"],
        json!("The gap in logical pixels.")
    );
}

/// `view_tag` requires the tag, a number from 1.
#[test]
fn view_tag_requires_the_tag_as_an_integer() {
    let methods = exported_methods();
    let view = find(&methods, "view_tag");
    assert_eq!(required(view), ["tag"]);
    assert_eq!(properties(view)["tag"]["type"], json!("integer"));
    assert_eq!(
        properties(view)["tag"]["description"],
        json!("The tag number, from 1.")
    );
}

/// The methods about one window require its surface id as a string, and
/// `move_to_tag` requires the tag too.
#[test]
fn the_window_methods_require_their_surface_as_a_string() {
    let methods = exported_methods();
    for name in ["focus", "swap_master"] {
        let method = find(&methods, name);
        assert_eq!(required(method), ["surface"], "{name}");
        assert_eq!(
            properties(method)["surface"]["type"],
            json!("string"),
            "{name}"
        );
    }
    let moved = find(&methods, "move_to_tag");
    assert_eq!(required(moved), ["surface", "tag"]);
    assert_eq!(properties(moved)["tag"]["type"], json!("integer"));
}

/// The methods that take no parameters offer an empty object of
/// properties and require nothing.
#[test]
fn the_methods_without_parameters_take_nothing() {
    let methods = exported_methods();
    for name in [
        "show_keys",
        "next_layout",
        "get_layout",
        "get_tags",
        "focus_next",
        "focus_prev",
    ] {
        let method = find(&methods, name);
        assert_eq!(properties(method), json!({}), "{name}");
        assert_eq!(required(method), Vec::<String>::new(), "{name}");
    }
}
