//! Windows that move between window managers (D55): the claim under a
//! point, and the method `window.give` that every window manager exports.

use chora_scene::{
    Command, claimant_at,
    ipc::{ExportMethod, RpcError},
};
use serde::Deserialize;
use serde_json::json;

use crate::{conn::Conn, runtime};

/// The name of the method that hands a window to another window manager.
pub const METHOD: &str = "window.give";

pub use chora_scene::MOVE;

/// The parameters of `window.give`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Give {
    /// The window; the focused window of the window manager without it.
    pub surface: Option<String>,
    /// The client name of the window manager that gets it.
    pub to: String,
}

/// The export of `window.give`, with its description and schema.
#[must_use]
pub fn method() -> ExportMethod {
    runtime::method(
        METHOD,
        "Hands a window to another window manager (D55). Without surface: the focused window.",
        json!({
            "surface": {"type": "string", "description": "The surface id of the window."},
            "to": {"type": "string", "description": "The client name of the window manager."},
        }),
        &["to"],
    )
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
    /// `remove` with `to`. The core refuses a `to` that is not connected,
    /// and the surface then stays with the plugin.
    ///
    /// # Errors
    ///
    /// `to` is the plugin itself, the core refuses, or the socket fails.
    pub fn give(&mut self, surface: &str, to: &str) -> Result<(), RpcError> {
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
            None => Ok(()),
        }
    }
}

fn rejected(err: impl std::fmt::Display) -> RpcError {
    RpcError::new(RpcError::REJECTED, err.to_string())
}
