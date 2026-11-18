//! The actions that every look shares (KIT2, C10): one enum, one spelling
//! in a YAML binding. A look adds its own actions as a second enum and
//! binds both in one map with [`Action`].

use serde::{Deserialize, Serialize};

use crate::volume::Change;

/// A step through the desks of a look. Only a value for now: no look acts
/// on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    /// The next desk.
    Next,
    /// The previous desk.
    Previous,
}

/// A turn of the cube. Only a value for now: no look acts on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Turn {
    /// A quarter turn to the left.
    Left,
    /// A quarter turn to the right.
    Right,
}

/// What a binding does in every look.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum System {
    /// Runs a shell command as an app.
    Run(String),
    /// Starts the terminal of the look.
    Terminal,
    /// Starts the launcher of the look.
    Launcher,
    /// Shows all keys in the key list.
    Keys,
    /// Asks the HUD host to open the chat with a helper.
    Chat,
    /// Locks the screen.
    Lock,
    /// Opens the power menu: Lock, Log out, Restart, Shut down.
    Power,
    /// Takes a screenshot of the output.
    Screenshot,
    /// Changes the volume of the default sound output.
    Volume(Change),
    /// Asks the focused window to close.
    Close,
    /// Gives the focused window to the window manager of this name (D55).
    Give(String),
    /// On a pointer button: a window that is pressed and let go on the
    /// claim of another window manager goes to that one (D55).
    Drag,
    /// Steps through the desks.
    Desk(Step),
    /// Turns the cube.
    Cube(Turn),
}

/// The action of a binding: one of the system, or one of the look.
#[derive(Clone, Debug, PartialEq, Serialize, schemars::JsonSchema)]
#[serde(untagged)]
pub enum Action<L> {
    /// An action that every look has.
    System(System),
    /// An action of this look only.
    Look(L),
}

impl<'de, L> Deserialize<'de> for Action<L> {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("not yet"))
    }
}
