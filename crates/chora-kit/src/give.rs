//! Windows that move between window managers (D55): the claim under a
//! point, and the method `window.give` that every window manager exports.

use chora_scene::{
    Command, claimant_at,
    ipc::{ExportMethod, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{conn::Conn, runtime};

/// The name of the method that hands a window to another window manager.
pub const METHOD: &str = "window.give";

pub use chora_scene::MOVE;

/// The parameters of `window.give`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Give {
    /// The surface id of the window.
    pub surface: String,
    /// The client name of the window manager that gets it.
    pub to: String,
}

/// The export of `window.give`, with its description and schema. It is
/// privileged: a give hides a window from view and hands it to another
/// client, which may then focus, close or configure it.
#[must_use]
pub fn method() -> ExportMethod {
    runtime::privileged(runtime::method(
        METHOD,
        "Hands a window to another window manager (D55), and answers with its surface id.",
        json!({
            "surface": {"type": "string", "description": "The surface id of the window."},
            "to": {"type": "string", "description": "The client name of the window manager."},
        }),
        &["surface", "to"],
    ))
}

impl Conn {
    /// The window manager under a point in scene coordinates: the owner of
    /// the smallest claim of another client that contains it, as the core
    /// picks one for a new toplevel under the pointer; `None` outside the
    /// claims of others.
    ///
    /// # Errors
    ///
    /// The query fails.
    pub fn claimant_at(&mut self, point: [f64; 2]) -> Result<Option<String>, RpcError> {
        let nodes = self.query("*").map_err(rejected)?;
        let me = self.name();
        Ok(claimant_at(&nodes, point, |o| o != me).map(String::from))
    }

    /// Hands a surface of the plugin to the window manager `to` with a
    /// `remove` with `to`, and returns the answer of `window.give`: the
    /// surface id. The core refuses a `to` that is not connected or holds
    /// no claim, and the surface then stays with the plugin.
    ///
    /// # Errors
    ///
    /// `to` is the plugin itself, the core refuses, or the socket fails.
    pub fn give(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        if to == self.name() {
            return Err(RpcError::new(
                RpcError::INVALID_PARAMS,
                format!("{surface} is with {to} already"),
            ));
        }
        let remove = Command::Remove {
            id: surface.into(),
            to: Some(to.into()),
        };
        match self.try_apply(vec![remove]).map_err(rejected)? {
            Some(err) => Err(err),
            None => Ok(Value::String(surface.into())),
        }
    }
}

fn rejected(err: impl std::fmt::Display) -> RpcError {
    RpcError::new(RpcError::REJECTED, err.to_string())
}

#[cfg(test)]
mod tests {
    use chora_scene::ipc::ExportMethod;

    #[test]
    fn window_give_is_privileged() {
        assert!(matches!(super::method(), ExportMethod::Described(d) if d.privileged));
    }

    /// `choractl give` always names the surface, so the schema needs it.
    #[test]
    fn window_give_needs_a_surface() {
        let ExportMethod::Described(d) = super::method() else {
            panic!("a described method");
        };
        let required = &d.input_schema.unwrap()["required"];
        assert_eq!(required, &serde_json::json!(["surface", "to"]));
        let params = serde_json::json!({"to": "xp"});
        assert!(serde_json::from_value::<super::Give>(params).is_err());
    }
}
