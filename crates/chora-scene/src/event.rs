//! Events that the core sends to subscribed clients.

use serde::{Deserialize, Serialize};

/// One event. The JSON form carries the name in the field `event`, for
/// example `{"event": "surface.created", "surface": "s1", ...}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    /// A toplevel committed its first buffer state and waits for a plugin.
    #[serde(rename = "surface.created")]
    SurfaceCreated {
        /// The surface node.
        surface: String,
        /// The application id.
        app_id: Option<String>,
        /// The window title.
        title: Option<String>,
    },
    /// The title or the application id of a surface changed.
    #[serde(rename = "surface.changed")]
    SurfaceChanged {
        /// The surface node.
        surface: String,
        /// The application id.
        app_id: Option<String>,
        /// The window title.
        title: Option<String>,
    },
    /// The client destroyed a surface.
    #[serde(rename = "surface.destroyed")]
    SurfaceDestroyed {
        /// The surface node.
        surface: String,
    },
    /// A surface lost its parent and its owner.
    #[serde(rename = "surface.detached")]
    SurfaceDetached {
        /// The surface node.
        surface: String,
    },
    /// A grabbed key binding was pressed.
    #[serde(rename = "input.key")]
    InputKey {
        /// The binding as the client grabbed it.
        binding: String,
    },
    /// An output appeared or changed its mode, scale or position.
    #[serde(rename = "output.changed")]
    OutputChanged {
        /// The output.
        output: OutputInfo,
    },
    /// An output disappeared.
    #[serde(rename = "output.removed")]
    OutputRemoved {
        /// The output name.
        name: String,
    },
    /// A client named itself.
    #[serde(rename = "client.connected")]
    ClientConnected {
        /// The client name.
        name: String,
    },
    /// The last connection of a client name closed.
    #[serde(rename = "client.disconnected")]
    ClientDisconnected {
        /// The client name.
        name: String,
    },
}

impl Event {
    /// The name of the event, for example `surface.created`: the value of
    /// the field `event` in the JSON form.
    ///
    /// # Panics
    ///
    /// Never: every event serializes with its name.
    #[must_use]
    pub fn name(&self) -> String {
        let json = serde_json::to_value(self).expect("events always serialize");
        json["event"]
            .as_str()
            .expect("every event has a name")
            .to_owned()
    }
}

/// An output and the part of the scene that it shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputInfo {
    /// The connector name, for example `DP-1`.
    pub name: String,
    /// The position in the output layout, in logical pixels.
    pub position: [i32; 2],
    /// The size in logical pixels.
    pub size: [i32; 2],
    /// The scale factor.
    pub scale: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_the_json_tag() {
        let e = Event::SurfaceDetached {
            surface: "s1".into(),
        };
        assert_eq!(e.name(), "surface.detached");
    }
}
