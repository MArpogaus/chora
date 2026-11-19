//! The actions that every look shares (KIT2, C10): one enum, one spelling
//! in a YAML binding. A look adds its own actions as a second enum and
//! binds both in one map with [`Action`].

use serde::{Deserialize, Serialize};

pub mod perform;

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
    /// Shows all keys in the key list. `help` is the old spelling of the
    /// construct and `show_keys` that of tiling, read for one release.
    #[serde(alias = "help", alias = "show_keys")]
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

impl System {
    /// Whether the action is one of the keys that every look has: it works
    /// in the walk and as a grab in a pane, the launcher, the key list, the
    /// chat, lock, power, the screenshot and the volume.
    #[must_use]
    pub const fn in_every_look(&self) -> bool {
        matches!(
            self,
            Self::Launcher
                | Self::Keys
                | Self::Chat
                | Self::Lock
                | Self::Power
                | Self::Screenshot
                | Self::Volume(_)
        )
    }

    /// The shared script that runs the action and its arguments, for
    /// [`crate::shared::command`]: the locker, the power menu and the
    /// screenshot.
    #[must_use]
    pub fn script(&self) -> Option<(&'static str, &'static [&'static str])> {
        match self {
            Self::Lock => Some(("power", &["lock"])),
            Self::Power => Some(("power", &[])),
            Self::Screenshot => Some(("screenshot", &[])),
            _ => None,
        }
    }
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

/// The actions of the system that a binding spells as a plain name; `help`
/// and `show_keys` are old spellings of `keys`.
const NAMES: [&str; 11] = [
    "terminal",
    "launcher",
    "keys",
    "help",
    "show_keys",
    "chat",
    "lock",
    "power",
    "screenshot",
    "close",
    "drag",
];

/// The actions of the system that a binding spells as a map with one key.
const WORDS: [&str; 5] = ["run", "volume", "give", "desk", "cube"];

/// The old spellings of the three volume keys of the construct, read for
/// one release: the plain name and the change that it stands for.
const OLD: [(&str, &str); 3] = [
    ("volume_up", "up"),
    ("volume_down", "down"),
    ("mute", "mute"),
];

/// Rewrites an old spelling to the spelling of today.
fn current(value: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::String(name) = &value
        && let Some((_, change)) = OLD.iter().find(|(old, _)| old == name)
    {
        return serde_json::json!({ "volume": change });
    }
    value
}

/// Whether a value names an action of the system: a plain name of
/// [`NAMES`], or a map with one key of [`WORDS`]. A bare `cube` is the
/// cube action of a look, `{cube: left}` the turn of the system.
fn names_system(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::String(name) => NAMES.contains(&name.as_str()),
        serde_json::Value::Object(map) => {
            map.len() == 1 && map.keys().all(|k| WORDS.contains(&k.as_str()))
        }
        _ => false,
    }
}

impl<L> From<System> for Action<L> {
    fn from(system: System) -> Self {
        Self::System(system)
    }
}

impl<'de, L: serde::de::DeserializeOwned> Deserialize<'de> for Action<L> {
    /// A system action first, else an action of the look. A mistake is
    /// reported by the side that the value names: `{volume: sideways}`
    /// is a bad volume, not an unknown action of the look.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = current(serde_json::Value::deserialize(deserializer)?);
        match System::deserialize(&value) {
            Ok(system) => Ok(Self::System(system)),
            Err(err) if names_system(&value) => Err(D::Error::custom(err)),
            Err(_) => L::deserialize(value)
                .map(Self::Look)
                .map_err(D::Error::custom),
        }
    }
}
