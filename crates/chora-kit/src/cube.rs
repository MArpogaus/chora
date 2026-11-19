//! The cube role (D71, arch plan 1.2): the client that hosts a cube of
//! faces exports `cube.list`, `cube.turn` and `cube.pull`, and emits the
//! event `NAME.cube` whenever its front face or its turning changes,
//! where NAME is its client name. A look reaches the cube through the
//! role and never names the plugin: [`exporter`] finds the client with
//! `methods.list`. Without an exporter a look switches its desks flat.

use chora_scene::ipc::{ClientError, ExportMethod};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{conn::Conn, runtime};

/// The method that lists the faces.
pub const LIST: &str = "cube.list";
/// The method that turns the cube.
pub const TURN: &str = "cube.turn";
/// The method that lifts a window off its face.
pub const PULL: &str = "cube.pull";

/// A face of the cube: the virtual output it shows, the client name of
/// its window manager, and whether that manager is `starting`, `up` or
/// `failed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Face {
    /// The virtual output of the face.
    pub output: String,
    /// The client name of the window manager on the output.
    pub manager: String,
    /// `starting`, `up` or `failed`.
    pub health: String,
}

/// The answer of `cube.list`: the faces in turn order, the face at the
/// front, whether the cube turns, and the mode of the view: `use` when
/// the front face shows at 1:1 and takes the input, else `navigate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listing {
    /// The faces, face 0 first.
    pub faces: Vec<Face>,
    /// The face at the front, or the one a running turn ends on.
    pub front: usize,
    /// Whether the cube turns, by a key, a method or the hand.
    pub turning: bool,
    /// `use` or `navigate`.
    pub mode: String,
}

/// The data of the event `NAME.cube`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Turned {
    /// The face at the front, or the one a running turn ends on.
    pub front: usize,
    /// Whether the cube turns.
    pub turning: bool,
}

/// The parameters of `cube.turn`: either `by` quarter turns, positive to
/// the right, or `to` a face the short way; in `ms` milliseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Turn {
    /// Quarter turns, positive to the right.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<i32>,
    /// The face to turn to, the short way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<usize>,
    /// How long a quarter turn takes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ms: Option<u64>,
}

impl Turn {
    /// The quarter turns from the face `front` of `faces`.
    ///
    /// # Errors
    ///
    /// Both or neither of `by` and `to` are given, or `to` is no face.
    pub fn quarters(&self, front: usize, faces: usize) -> Result<i32, String> {
        match (self.by, self.to) {
            (Some(by), None) => Ok(by),
            (None, Some(to)) if to < faces => Ok(quarters_to(front, to, faces)),
            (None, Some(to)) => Err(format!(
                "to: {to} is not a face of 0 to {}",
                faces.saturating_sub(1)
            )),
            _ => Err("give either by or to".into()),
        }
    }
}

/// The parameters of `cube.pull`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pull {
    /// The surface id of a window on a face.
    pub surface: String,
}

/// The quarter turns from the face `front` to the face `to` of `faces`,
/// the short way; to the right at a tie.
#[must_use]
pub fn quarters_to(front: usize, to: usize, faces: usize) -> i32 {
    let n = i32::try_from(faces).unwrap_or(1).max(1);
    let to = i32::try_from(to).unwrap_or(0);
    let by = (to - i32::try_from(front).unwrap_or(0)).rem_euclid(n);
    if by * 2 > n { by - n } else { by }
}

/// The exports of the role, with their descriptions and schemas.
#[must_use]
pub fn methods() -> Vec<ExportMethod> {
    vec![
        runtime::method(
            LIST,
            "Answers the faces of the cube in turn order, each with its virtual output, \
             the client name of its window manager and its health (starting, up, failed); \
             the front face, whether the cube turns, and the mode, use or navigate.",
            json!({}),
            &[],
        ),
        runtime::method(
            TURN,
            "Turns the cube BY quarter turns, positive to the right, or TO a face the \
             short way, in MS milliseconds a quarter; ignored while a turn runs. Answers \
             whether the cube took the turn. The event NAME.cube follows.",
            json!({
                "by": {"type": "integer"},
                "to": {"type": "integer", "minimum": 0},
                "ms": {"type": "integer", "minimum": 0},
            }),
            &[],
        ),
        runtime::method(
            PULL,
            "Lifts the window SURFACE off its face into the space as a pane, where it \
             stays until a drag drops it on a face again or the pane goes.",
            json!({"surface": {"type": "string"}}),
            &["surface"],
        ),
    ]
}

/// The name of the event of the cube host with the client name `client`.
#[must_use]
pub fn event(client: &str) -> String {
    format!("{client}.cube")
}

/// The client that exports the role, if one does.
///
/// # Errors
///
/// The core does not answer `methods.list`.
pub fn exporter(conn: &mut Conn) -> Result<Option<String>, ClientError> {
    let listed: Value = conn.call("methods.list", Value::Null)?;
    Ok(exporter_in(&listed))
}

/// The client of the answer of `methods.list` that exports `cube.list`.
fn exporter_in(listed: &Value) -> Option<String> {
    (listed.as_array()?.iter())
        .find(|m| m["name"].as_str() == Some(LIST))
        .and_then(|m| m["client"].as_str())
        .map(str::to_owned)
}

/// Asks the cube host `client` for its faces.
///
/// # Errors
///
/// The call fails or the answer is no listing.
pub fn list(conn: &mut Conn, client: &str) -> Result<Listing, ClientError> {
    let call = json!({"client": client, "method": LIST, "params": {}});
    let answer: Value = conn.call("methods.call", call)?;
    serde_json::from_value(answer).map_err(ClientError::Json)
}

/// Asks the cube host `client` to turn.
///
/// # Errors
///
/// The call fails or the host refuses it.
pub fn turn(conn: &mut Conn, client: &str, turn: Turn) -> Result<bool, ClientError> {
    let call = json!({"client": client, "method": TURN, "params": turn});
    let answer: Value = conn.call("methods.call", call)?;
    Ok(answer.as_bool().unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turn_to_a_face_goes_the_short_way() {
        assert_eq!(quarters_to(0, 1, 4), 1);
        assert_eq!(quarters_to(0, 3, 4), -1);
        assert_eq!(quarters_to(0, 2, 4), 2, "to the right at a tie");
        assert_eq!(quarters_to(3, 0, 4), 1);
        assert_eq!(quarters_to(2, 2, 4), 0);
        assert_eq!(quarters_to(0, 1, 2), 1);
    }

    #[test]
    fn a_turn_takes_either_by_or_to() {
        let by = Turn {
            by: Some(-1),
            ..Turn::default()
        };
        assert_eq!(by.quarters(0, 4), Ok(-1));
        let to = Turn {
            to: Some(3),
            ..Turn::default()
        };
        assert_eq!(to.quarters(1, 4), Ok(2));
        let far = Turn {
            to: Some(4),
            ..Turn::default()
        };
        assert!(far.quarters(0, 4).unwrap_err().contains("not a face"));
        assert!(Turn::default().quarters(0, 4).is_err());
        let both = Turn {
            by: Some(1),
            to: Some(1),
            ms: None,
        };
        assert!(both.quarters(0, 4).is_err());
    }

    #[test]
    fn unknown_parameters_are_refused() {
        assert!(serde_json::from_value::<Turn>(json!({"to": 1, "face": 2})).is_err());
        assert!(serde_json::from_value::<Pull>(json!({"surface": "s", "x": 1})).is_err());
    }

    #[test]
    fn the_exporter_is_the_client_of_cube_list() {
        let listed = json!([
            {"client": "xp", "name": "windows.float"},
            {"client": "construct", "name": "cube.turn"},
            {"client": "construct", "name": "cube.list"},
        ]);
        assert_eq!(exporter_in(&listed).as_deref(), Some("construct"));
        assert_eq!(exporter_in(&json!([])), None);
    }

    #[test]
    fn the_role_exports_three_methods_and_names_its_event_by_the_client() {
        let names: Vec<String> = (methods().into_iter())
            .map(|m| match m {
                ExportMethod::Described(d) => d.name,
                ExportMethod::Plain(n) => n,
            })
            .collect();
        assert_eq!(names, [LIST, TURN, PULL]);
        assert_eq!(event("construct"), "construct.cube");
        let data = serde_json::to_value(Turned {
            front: 2,
            turning: false,
        })
        .unwrap();
        assert_eq!(data, json!({"front": 2, "turning": false}));
    }
}
