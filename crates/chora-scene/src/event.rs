//! Events that the core sends to subscribed clients.

use serde::{Deserialize, Serialize};

use crate::{Decoration, Transform};

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
        /// Who draws the frame of the window.
        #[serde(default)]
        decoration: Option<Decoration>,
    },
    /// The title, the application id, the size or the decoration of a
    /// surface changed.
    #[serde(rename = "surface.changed")]
    SurfaceChanged {
        /// The surface node.
        surface: String,
        /// The application id.
        app_id: Option<String>,
        /// The window title.
        title: Option<String>,
        /// The size of the window geometry, in logical pixels.
        #[serde(default)]
        size: Option<[i32; 2]>,
        /// Who draws the frame of the window.
        #[serde(default)]
        decoration: Option<Decoration>,
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
    /// The key of a grabbed binding went down, or up again.
    #[serde(rename = "input.key")]
    InputKey {
        /// The binding as the client grabbed it.
        binding: String,
        /// Pressed or released.
        #[serde(default)]
        state: crate::ipc::PressState,
    },
    /// A button went down over a node that takes pointer input: a surface,
    /// a rect or an image. Over a rect or an image, `pointer.moved` and
    /// `pointer.released` follow until the button goes up, for example for
    /// the drag of a title bar; the clients see nothing of it.
    #[serde(rename = "pointer.pressed")]
    PointerPressed {
        /// The node.
        node: String,
        /// The Linux button code, for example 272 for the left button.
        button: u32,
        /// The pointer in scene coordinates, or in output coordinates for a
        /// node under `screen:NAME`.
        position: [f64; 2],
    },
    /// The pointer came over a node that takes pointer input: the top one
    /// under it, a surface, a rect or an image. Not during a drag of a
    /// node, nor while the session is locked.
    #[serde(rename = "pointer.entered")]
    PointerEntered {
        /// The node.
        node: String,
        /// Whether the pointer moved; false when the scene moved under a
        /// pointer that stood still.
        #[serde(default)]
        moved: bool,
    },
    /// The pointer left the node of the last `pointer.entered`.
    #[serde(rename = "pointer.left")]
    PointerLeft {
        /// The node.
        node: String,
    },
    /// The pointer moved while a button that went down over a rect or an
    /// image is still down.
    #[serde(rename = "pointer.moved")]
    PointerMoved {
        /// The node of the press.
        node: String,
        /// The pointer in scene coordinates, or in output coordinates for a
        /// node under `screen:NAME`.
        position: [f64; 2],
    },
    /// The button of a press over a rect or an image went up.
    #[serde(rename = "pointer.released")]
    PointerReleased {
        /// The node of the press.
        node: String,
        /// The Linux button code.
        button: u32,
        /// The pointer in scene coordinates, or in output coordinates for a
        /// node under `screen:NAME`.
        position: [f64; 2],
    },
    /// Frames took too long again and again, so the watchdog of the core
    /// removed the effects of the nodes of the client whose nodes cost the
    /// most. If frames stay slow, the core disconnects that client.
    #[serde(rename = "effects.dropped")]
    EffectsDropped {
        /// The name of the client.
        client: String,
    },
    /// The layers of nodes did not fit into the pixel budget of a frame and
    /// stay out of the output, or are back. Sent when the list of an
    /// output changes; an empty list means that all layers show again.
    #[serde(rename = "layers.dropped")]
    LayersDropped {
        /// The output.
        output: String,
        /// The nodes whose layers stay out.
        nodes: Vec<String>,
    },
    /// The shader of an effect failed after a change of its file. The
    /// effect keeps its last working version, or shows nothing.
    #[serde(rename = "effect.failed")]
    EffectFailed {
        /// The path and the log of the compiler.
        error: String,
    },
    /// An animation reached its targets: of a node or of the camera of an
    /// output.
    #[serde(rename = "animation.done")]
    AnimationDone {
        /// The node.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node: Option<String>,
        /// The output of the camera.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
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
    /// Another client calls a method that this connection exported. Only
    /// the exporting connection gets this event, subscribed or not. It
    /// answers with `methods.return` and the same `call`.
    #[serde(rename = "method.called")]
    MethodCalled {
        /// The id of the call.
        call: u64,
        /// The client name of the caller.
        caller: String,
        /// The method name, without the client name.
        method: String,
        /// The parameters of the caller.
        #[serde(default)]
        params: serde_json::Value,
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
    /// panels and `output.reserve` reserve: x, y, width and height in scene
    /// coordinates.
    pub work_area: [i32; 4],
    /// The work area as the default camera shows it: the place of the
    /// windows of a plugin, which a camera move or a zoom does not change.
    #[serde(default)]
    pub default_work_area: [i32; 4],
}

impl OutputInfo {
    /// The area for the windows of a plugin: `default_work_area`, or the
    /// `work_area` of a core that does not send it yet.
    #[must_use]
    pub fn layout_area(&self) -> [i32; 4] {
        if self.default_work_area == [0; 4] {
            self.work_area
        } else {
            self.default_work_area
        }
    }
}
