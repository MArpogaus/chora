//! Events that the core sends to subscribed clients.

use serde::{Deserialize, Serialize};

use crate::Transform;

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
    /// A client, or a taskbar for it, asks for a change that a plugin
    /// decides, for example fullscreen.
    #[serde(rename = "surface.requested")]
    SurfaceRequested {
        /// The surface node.
        surface: String,
        /// The change.
        request: SurfaceRequest,
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

/// A change that a client asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceRequest {
    /// Focus and raise the surface, for example after a click on a
    /// notification.
    Activate,
    /// Close the surface, for example from a taskbar.
    Close,
    /// Show the surface fullscreen.
    Fullscreen,
    /// End fullscreen.
    Unfullscreen,
    /// Maximize the surface.
    Maximize,
    /// End the maximized state.
    Unmaximize,
    /// Minimize the surface; Wayland has no state for it.
    Minimize,
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
    /// The camera: the transform from scene coordinates to the logical
    /// coordinates of the output.
    pub camera: Transform,
    /// The part of the scene that the output shows, without the areas that
    /// panels reserve: x, y, width and height in scene coordinates.
    pub work_area: [i32; 4],
}
