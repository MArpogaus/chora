//! Commands that change the scene, and the effects they ask of the core.

use serde::{Deserialize, Serialize};

use crate::{Easing, NodeKind, PropsUpdate, Transform};

/// One command of a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    /// Creates a group, rect, image, shader or text node under a parent.
    Create {
        /// The id of the new node.
        id: String,
        /// Group, rect, image, shader or text; the core creates surface
        /// nodes itself.
        kind: NodeKind,
        /// The parent group.
        parent: String,
        /// The initial properties.
        #[serde(default)]
        props: PropsUpdate,
    },
    /// Puts a surface node without an owner under a parent. The caller
    /// becomes its owner.
    Attach {
        /// The surface node.
        surface: String,
        /// The parent group.
        parent: String,
    },
    /// Changes properties of a node.
    Update {
        /// The node.
        id: String,
        /// The new values.
        props: PropsUpdate,
    },
    /// Moves properties of a node to new values over a duration. A later
    /// `animate` or `update` of the node ends the animation where it is.
    /// The core sends `animation.done` at the end.
    Animate {
        /// The node.
        id: String,
        /// The targets.
        props: PropsUpdate,
        /// In milliseconds, at most one minute.
        duration: f64,
        /// The curve; linear if missing.
        #[serde(default)]
        easing: Easing,
    },
    /// Changes the parent or the position among the siblings.
    Move {
        /// The node.
        id: String,
        /// The new parent; the old parent if missing.
        parent: Option<String>,
        /// The new position among the siblings, 0 at the bottom; the top if
        /// missing.
        index: Option<usize>,
    },
    /// Removes a node and its descendants. Surfaces become detached.
    Remove {
        /// The node.
        id: String,
        /// For a surface: the client name that gets it (D55). Only this
        /// client can attach it, and only it gets `surface.detached`. It
        /// needs a connection.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    /// Asks a client for a new size or new toplevel states.
    Configure {
        /// The surface node.
        surface: String,
        /// Width and height in logical pixels; the client decides if missing.
        size: Option<[i32; 2]>,
        /// The toplevel states.
        #[serde(default)]
        states: Vec<ToplevelState>,
    },
    /// Sets the keyboard focus.
    Focus {
        /// The surface node; no focus if missing.
        surface: Option<String>,
    },
    /// Gives the keyboard to a group, rect, image or shader node of the
    /// caller: the core sends its keys as `node.key` events to the client,
    /// and no surface gets them. Without a node, the caller gives the
    /// keyboard back to the focused surface. The focus of a surface, the
    /// removal of the node, the disconnect of the client and the lock of the
    /// session end it too.
    FocusNode {
        /// The node; the end of the focus of a node of the caller if missing.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node: Option<String>,
    },
    /// Asks a client to close a surface.
    Close {
        /// The surface node.
        surface: String,
    },
    /// Sets the camera of an output: the transform from scene coordinates
    /// to the logical coordinates of the output. The first client that sets
    /// the camera owns it.
    Camera {
        /// The output name, for example `DP-1`.
        output: String,
        /// A 2D translation and positive scale; the default camera if
        /// missing.
        transform: Option<Transform>,
        /// In milliseconds, at most one minute: the camera moves there from
        /// where it is, and the core sends `animation.done` with the output
        /// at the end. At once if missing.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<f64>,
        /// The curve; linear if missing.
        #[serde(default)]
        easing: Easing,
    },
}

/// A state of an `xdg_toplevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ToplevelState {
    /// The window has the focus.
    Activated,
    /// The window fills its area; the client draws no shadows.
    Tiled,
    /// The window is maximized.
    Maximized,
    /// The window is fullscreen.
    Fullscreen,
}

/// An action of a transaction outside the scene. The core executes it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Send a configure event.
    Configure {
        /// The surface node.
        surface: String,
        /// See [`Command::Configure`].
        size: Option<[i32; 2]>,
        /// See [`Command::Configure`].
        states: Vec<ToplevelState>,
    },
    /// Set the keyboard focus.
    Focus(Option<String>),
    /// Ask a client to close a surface.
    Close(String),
    /// A surface lost its parent and its owner; tell the clients.
    Detached(String),
    /// A surface lost its parent and its owner, reserved for a client
    /// name; tell that client.
    Reserved(String, String),
    /// The camera of an output changed; tell the clients.
    Camera(String),
    /// A client asked to change a node of another client (D31); tell the
    /// owner, who decides by doing it or not.
    Requested {
        /// The node.
        node: String,
        /// The owner of the node, who decides.
        owner: String,
        /// The client that asked.
        by: String,
        /// `focus`, `move`, `resize`, `close` or `update`.
        action: &'static str,
        /// The arguments of the command, for example `{"parent": "g"}`.
        args: serde_json::Value,
    },
}
