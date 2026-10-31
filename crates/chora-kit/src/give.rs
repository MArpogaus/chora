//! Windows that move between window managers (D55): the claim under a
//! point, and the method `window.give` that every window manager exports.

use chora_scene::{
    Command, Node,
    ipc::{ExportMethod, RpcError},
};
use serde::Deserialize;
use serde_json::json;

use crate::{conn::Conn, runtime};

/// The name of the method that hands a window to another window manager.
pub const METHOD: &str = "window.give";

/// The binding of the `input.button` and `input.moved` events of an
/// interactive move: a client asked to move its window, for example from
/// its title bar, and the window manager of the window gets the drag.
pub const MOVE: &str = "move";

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

/// The owner of the smallest claim that contains `point`, other than `me`:
/// the window manager under a drop point, as the core picks one for a
/// new toplevel under the pointer.
#[must_use]
pub fn claimant<'a>(nodes: &'a [Node], [px, py]: [f64; 2], me: &str) -> Option<&'a str> {
    nodes
        .iter()
        .filter_map(|n| Some((n.owner.as_deref()?, n.props.claim?)))
        .filter(|(owner, _)| *owner != me)
        .filter(|(_, [x, y, w, h])| (*x..x + w).contains(&px) && (*y..y + h).contains(&py))
        .min_by(|(_, a), (_, b)| (a[2] * a[3]).total_cmp(&(b[2] * b[3])))
        .map(|(owner, _)| owner)
}

impl Conn {
    /// The window manager under a point in scene coordinates, see
    /// [`claimant`]; `None` outside the claims of others.
    ///
    /// # Errors
    ///
    /// The query fails.
    pub fn claimant_at(&mut self, point: [f64; 2]) -> Result<Option<String>, RpcError> {
        let nodes = self.query("*").map_err(rejected)?;
        Ok(claimant(&nodes, point, self.name()).map(String::from))
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

#[cfg(test)]
mod tests {
    use chora_scene::{NodeKind, Props};

    use super::*;

    fn claim(owner: &str, claim: [f64; 4]) -> Node {
        Node {
            id: owner.into(),
            kind: NodeKind::Group,
            owner: Some(owner.into()),
            parent: Some("root".into()),
            children: Vec::new(),
            props: Props {
                claim: Some(claim),
                ..Props::default()
            },
            surface: None,
            to: None,
        }
    }

    #[test]
    fn the_smallest_claim_of_another_client_wins() {
        let nodes = [
            claim("xp", [0.0, 0.0, 1280.0, 800.0]),
            claim("tiling", [1280.0, 0.0, 1280.0, 800.0]),
            claim("mc", [1280.0, 0.0, 640.0, 400.0]),
        ];
        assert_eq!(claimant(&nodes, [1300.0, 10.0], "xp"), Some("mc"));
        assert_eq!(claimant(&nodes, [1300.0, 500.0], "xp"), Some("tiling"));
        assert_eq!(claimant(&nodes, [10.0, 10.0], "xp"), None);
        assert_eq!(claimant(&nodes, [10.0, 10.0], "tiling"), Some("xp"));
        assert_eq!(claimant(&nodes, [3000.0, 10.0], "xp"), None);
    }
}
