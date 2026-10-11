//! Commands that change the scene, and the effects they ask of the core.

use serde::{Deserialize, Serialize};

use crate::{NodeKind, PropsUpdate, Transform};

/// One command of a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    /// Creates a group or rect node under a parent.
    Create {
        /// The id of the new node.
        id: String,
        /// Group or rect; the core creates surface nodes itself.
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
    },
}

/// A state of an `xdg_toplevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// The camera of an output changed; tell the clients.
    Camera(String),
}
