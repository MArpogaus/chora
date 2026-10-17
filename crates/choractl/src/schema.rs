//! The JSON schemas of the tools of `choractl mcp`, from the types of
//! `chora-scene`. MCP clients hand them to the model, so that its
//! first call has the right shape more often.

use serde_json::Value;

/// The JSON schema of a type, with its definitions in `$defs`.
pub fn of<T: schemars::JsonSchema>() -> Value {
    let mut schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value();
    if let Some(root) = schema.as_object_mut() {
        root.remove("$schema");
        root.remove("title");
    }
    schema
}

/// Whether a value fits the schema `root`, for the keywords that schemars
/// writes. An unknown keyword fails the test, so that a new kind of schema
/// cannot pass unchecked.
#[cfg(test)]
pub fn fits(root: &Value, value: &Value) -> bool {
    fits_in(root, root, value)
}

#[cfg(test)]
fn fits_in(root: &Value, schema: &Value, value: &Value) -> bool {
    match schema {
        Value::Bool(b) => *b,
        Value::Object(rules) => rules
            .iter()
            .all(|(key, rule)| keyword(root, rules, key, rule, value)),
        other => panic!("no schema: {other}"),
    }
}

#[cfg(test)]
fn keyword(
    root: &Value,
    rules: &serde_json::Map<String, Value>,
    key: &str,
    rule: &Value,
    value: &Value,
) -> bool {
    match key {
        "$ref" => {
            let name = rule.as_str().and_then(|r| r.strip_prefix("#/$defs/"));
            let def = &root["$defs"][name.expect("a reference into $defs")];
            assert!(!def.is_null(), "no definition {rule}");
            fits_in(root, def, value)
        }
        "type" => match rule {
            Value::Array(types) => types.iter().any(|t| has_type(t, value)),
            single => has_type(single, value),
        },
        "const" => value == rule,
        "enum" => all(rule).contains(value),
        "anyOf" => all(rule).iter().any(|s| fits_in(root, s, value)),
        "oneOf" => all(rule).iter().filter(|s| fits_in(root, s, value)).count() == 1,
        "required" => value.as_object().is_none_or(|o| {
            all(rule)
                .iter()
                .all(|k| o.contains_key(k.as_str().unwrap_or_default()))
        }),
        "properties" => value.as_object().is_none_or(|o| {
            o.iter()
                .all(|(k, v)| rule.get(k).is_none_or(|s| fits_in(root, s, v)))
        }),
        "additionalProperties" => value.as_object().is_none_or(|o| {
            let known = |k: &String| rules.get("properties").is_some_and(|p| p.get(k).is_some());
            o.iter().all(|(k, v)| known(k) || fits_in(root, rule, v))
        }),
        "items" => value
            .as_array()
            .is_none_or(|a| a.iter().all(|v| fits_in(root, rule, v))),
        "minItems" | "maxItems" => value.as_array().is_none_or(|a| {
            let len = u64::try_from(a.len()).unwrap_or(u64::MAX);
            let bound = rule.as_u64().expect("a length");
            if key == "minItems" {
                len >= bound
            } else {
                len <= bound
            }
        }),
        "minimum" => value
            .as_f64()
            .is_none_or(|n| n >= rule.as_f64().expect("a number")),
        "description" | "default" | "format" | "$defs" => true,
        other => panic!("the test knows no keyword {other}"),
    }
}

#[cfg(test)]
fn all(rule: &Value) -> &Vec<Value> {
    rule.as_array().expect("a list of schemas")
}

#[cfg(test)]
fn has_type(name: &Value, value: &Value) -> bool {
    match name.as_str() {
        Some("object") => value.is_object(),
        Some("array") => value.is_array(),
        Some("string") => value.is_string(),
        Some("number") => value.is_number(),
        Some("integer") => value.is_i64() || value.is_u64(),
        Some("boolean") => value.is_boolean(),
        Some("null") => value.is_null(),
        _ => panic!("no type {name}"),
    }
}

#[cfg(test)]
mod tests {
    use chora_scene::{
        Command,
        ipc::{Apply, Inject, InjectEvent},
    };
    use serde_json::json;

    use super::*;

    /// The schema accepts a value exactly when serde does.
    fn agree<T: schemars::JsonSchema + serde::de::DeserializeOwned>(values: &[Value]) {
        let schema = of::<T>();
        for value in values {
            let serde = serde_json::from_value::<T>(value.clone()).is_ok();
            assert_eq!(
                fits(&schema, value),
                serde,
                "the schema and serde disagree on {value}"
            );
        }
    }

    fn matrix(n: usize) -> Value {
        json!({"matrix": vec![1.0; n]})
    }

    #[test]
    fn the_schema_of_a_command_agrees_with_its_type() {
        agree::<Command>(&[
            json!({"create": {"id": "a", "kind": "rect", "parent": "root",
                "props": {"size": [1, 2], "color": [1, 0, 0, 1], "radius": 3,
                    "border": {"width": 1, "color": [0, 0, 0, 1]}, "clip": null,
                    "meta": {"role": "button"}, "input": "pass",
                    "transform": {"translate": [1, 2], "scale": 2, "rotate": 45}}}}),
            json!({"create": {"id": "s", "kind": "shader", "parent": "root",
                "props": {"shader": "/s.frag",
                    "uniforms": {"t": 1.0, "v": [1, 2], "p": {"node": "a"}}}}}),
            json!({"attach": {"surface": "s", "parent": "root"}}),
            json!({"update": {"id": "a", "props": {"border": null, "clip": {}}}}),
            json!({"animate": {"id": "a", "props": {"opacity": 0.5}, "duration": 300,
                "easing": "ease_in_out"}}),
            json!({"move": {"id": "a", "index": 0}}),
            json!({"remove": {"id": "a"}}),
            json!({"configure": {"surface": "s", "size": [100, 200], "states": ["tiled"]}}),
            json!({"focus": {}}),
            json!({"focus_node": {"node": "a"}}),
            json!({"focus_node": {}}),
            json!({"close": {"surface": "s"}}),
            json!({"camera": {"output": "X", "transform": matrix(16)}}),
            json!({"camera": {"output": "X", "transform": {"scale": [2, 2]}}}),
            json!({"camera": {"output": "X"}}),
            json!({"camera": {"output": "X", "transform": {"scale": 2}, "duration": 300,
                "easing": "ease_out"}}),
            // Wrong shapes.
            json!({"create": {"id": "a", "kind": "rect"}}),
            json!({"create": {"id": "a", "kind": "circle", "parent": "root"}}),
            json!({"update": {"id": "a", "props": {"radious": 3}}}),
            json!({"update": {"id": "a", "props": {"size": [1]}}}),
            json!({"update": {"id": "a", "props": {"opacity": "half"}}}),
            json!({"update": {"id": "a", "props": {"uniforms": {"p": {"node": "a", "x": 1}}}}}),
            json!({"camera": {"output": "X", "transform": matrix(15)}}),
            json!({"camera": {"output": "X", "transform": {"translate": [1, 2, 3, 4]}}}),
            json!({"camera": {"output": "X", "transform": {"scale": [1, 2, 3, 4]}}}),
            json!({"camera": {"output": "X", "transform": {"skew": 1}}}),
            json!({"animate": {"id": "a", "props": {}, "duration": 300, "easing": "bounce"}}),
            json!({"configure": {"surface": "s", "states": ["minimized"]}}),
            json!({"move": {"id": "a", "index": -1}}),
            json!({"remove": {}}),
            json!({"remove": {"id": "a"}, "close": {"surface": "s"}}),
            json!({"rect": {"id": "a"}}),
        ]);
    }

    #[test]
    fn the_schema_of_an_input_event_agrees_with_its_type() {
        agree::<InjectEvent>(&[
            json!({"pointer_move": {"position": [1, 2], "output": "X"}}),
            json!({"button": {"state": "pressed"}}),
            json!({"click": {}}),
            json!({"scroll": {"dy": 3}}),
            json!({"key": {"key": 28, "state": "pressed"}}),
            json!({"key": {"key": "Return", "state": "released"}}),
            json!({"shortcut": {"keys": "Ctrl+Shift+t"}}),
            json!({"text": {"text": "hi\n"}}),
            // Wrong shapes.
            json!({"click": {"button": -1}}),
            json!({"key": {"key": "a"}}),
            json!({"pointer_move": {"position": [1, 2, 3]}}),
            json!({"scroll": {"dz": 1}}),
            json!({"shortcut": {"keys": 3}}),
            json!({"tap": {}}),
        ]);
    }

    #[test]
    fn the_tools_take_objects() {
        for schema in [of::<Apply>(), of::<Inject>()] {
            assert_eq!(schema["type"], "object");
            assert!(schema.get("$schema").is_none());
        }
    }
}
