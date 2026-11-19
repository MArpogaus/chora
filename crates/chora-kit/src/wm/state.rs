//! The shape of the answer of `state.get` (arch plan 2.2, 3.1), typed, so
//! that every look answers the same keys. Act contracts and invariants
//! read it (tests/acts); a port to `kit::wm` leaves it unchanged.

use serde::Serialize;
use serde_json::{Map, Value};

use super::book::Origin;

/// What a look shows and holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct State {
    /// The windows of the look, bottom first.
    pub windows: Vec<Window>,
    /// The desks that show, per output.
    pub desks: Vec<Desks>,
    /// Who holds the keyboard.
    pub focus: Focus,
    /// The open shell surfaces.
    pub open: Vec<Open>,
    /// What still moves.
    pub busy: Vec<String>,
    /// What only this look tells, such as the word of its helper.
    #[serde(flatten)]
    pub more: Map<String, Value>,
}

/// A window of the look.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the keys of state.get, as the acts read them"
)]
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Window {
    /// The surface id.
    pub surface: String,
    /// Where it came from, when the book knows it: `new`, `given` or
    /// `kept`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<&'static str>,
    /// Its title, when the look keeps it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Its application id, when the look keeps it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    /// The output that it is on.
    pub output: String,
    /// The desks that it is on, from 1.
    pub desks: Vec<usize>,
    /// Where it shows, `[x, y, w, h]`, while it shows.
    pub rect: Option<[f64; 4]>,
    /// Whether it shows.
    pub shown: bool,
    /// Whether it is minimized.
    pub minimized: bool,
    /// Whether it is maximized both ways.
    pub maximized: bool,
    /// Whether it is fullscreen.
    pub fullscreen: bool,
    /// Whether it floats.
    pub floating: bool,
    /// Its place in the stacking order, bottom first.
    pub z: Option<usize>,
}

/// The desks that show on an output.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Desks {
    /// The output.
    pub output: String,
    /// The desks that show, from 1.
    pub shown: Vec<usize>,
}

/// Who holds the keyboard.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Focus {
    /// The look that holds it; none when no look says so.
    pub holder: Option<String>,
    /// The window that has it, when a window has it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface: Option<String>,
}

/// An open shell surface.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Open {
    /// Its node id.
    pub id: String,
    /// What it is: `menu`, `popup`, `dialog`, `sheet` or `launcher`.
    pub kind: String,
    /// Whether it holds the keyboard and the pointer.
    pub modal: bool,
}

impl Origin {
    /// The word of the origin in [`Window::origin`]; the activation token
    /// stays out of the state.
    #[must_use]
    pub const fn word(&self) -> &'static str {
        match self {
            Self::New { .. } => "new",
            Self::Given => "given",
            Self::Kept => "kept",
        }
    }
}

impl State {
    /// The state as `state.get` answers it.
    #[must_use]
    pub fn value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn an_empty_state_has_every_key() {
        let state = State::default().value();
        let want = json!({"windows": [], "desks": [], "focus": {"holder": null},
            "open": [], "busy": []});
        assert_eq!(state, want);
    }

    #[test]
    fn a_window_tells_its_origin_but_not_its_token() {
        let origin = Origin::New {
            token: Some("secret".into()),
        };
        let window = Window {
            surface: "s1".into(),
            origin: Some(origin.word()),
            rect: Some([1.0, 2.0, 3.0, 4.0]),
            ..Window::default()
        };
        let mut more = Map::new();
        more.insert("helper".into(), json!("none"));
        let state = State {
            windows: vec![window],
            focus: Focus {
                holder: Some("look".into()),
                surface: Some("s1".into()),
            },
            more,
            ..State::default()
        }
        .value();
        assert_eq!(state["windows"][0]["origin"], "new");
        assert!(!state.to_string().contains("secret"));
        assert_eq!(state["focus"], json!({"holder": "look", "surface": "s1"}));
        assert_eq!(state["helper"], "none");
        assert_eq!(Origin::Given.word(), "given");
        assert_eq!(Origin::Kept.word(), "kept");
    }
}
