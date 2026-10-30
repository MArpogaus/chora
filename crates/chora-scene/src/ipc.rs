//! The JSON-RPC 2.0 protocol of the IPC socket, and a blocking client.
//!
//! Every message is one line of JSON. Events arrive as notifications with
//! the method `event`.

use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{Command, Event};

/// The environment variable with the path of the IPC socket.
pub const SOCKET_ENV: &str = "CHORA_SOCKET";

/// Variables of a host session that would let a program of chora reach
/// the host compositor instead. The startup command of the core drops
/// them, and the units of `choractl run` take them only from the caller.
pub const HOST_VARIABLES: [&str; 6] = [
    "WAYLAND_SOCKET",
    "DISPLAY",
    "SWAYSOCK",
    "I3SOCK",
    "NIRI_SOCKET",
    "HYPRLAND_INSTANCE_SIGNATURE",
];

/// The longest name: a method name or a client name.
pub const MAX_NAME: usize = 64;

/// Whether a method name or a client name is valid: lowercase letters,
/// digits, `_`, `-` and `.`, starting with a letter, at most 64 bytes.
/// The core refuses names like this in `hello`, `clients.create` and
/// method exports; asset names follow the rule of the core's uploads
/// module; clients check before connecting.
#[must_use]
pub fn valid_name(name: &str) -> bool {
    name.len() <= MAX_NAME
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_-.".contains(c))
}

/// The admin socket next to an IPC socket, for example
/// `wayland-1.admin.sock` next to `wayland-1.sock` (D40). Only `choractl`
/// uses it, for grants, client sockets and `camera.release`.
#[must_use]
pub fn admin_socket(socket: &Path) -> PathBuf {
    socket.with_extension("admin.sock")
}

/// The version of this build: the crate version and, when it was built in
/// a git checkout, the commit, for example `0.5.4+1ea0e89abc`. `hello`
/// answers with the version of the core, next to the protocol version of
/// [`PROTOCOL`], so that a client of another build can tell.
pub const VERSION: &str = env!("CHORA_VERSION");

/// The protocol version this build speaks (D29). It is the major version:
/// compatible additions arrive as features, a breaking change raises the
/// number. `hello` negotiates it; the core refuses a client with a higher
/// one and serves a lower one.
pub const PROTOCOL: u64 = 1;

/// The protocol features this build knows. Protocol 1 starts without any;
/// named additions appear here while the protocol stays 1.
pub const FEATURES: &[&str] = &[];

/// A request or a notification from the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Always `2.0`.
    pub jsonrpc: String,
    /// The request id; missing for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// The method name, for example `scene.apply`.
    pub method: String,
    /// The parameters.
    #[serde(default)]
    pub params: Value,
}

/// A message from the core: a response or an event notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Always `2.0`.
    pub jsonrpc: String,
    /// The id of the request; missing for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// The result of a successful request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The error of a failed request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    /// `event` for notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// The event of a notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{message} (code {code})")]
pub struct RpcError {
    /// The error code.
    pub code: i64,
    /// A readable description.
    pub message: String,
    /// What failed, machine readable: for a rejected transaction, the
    /// command as `{"command": INDEX}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    /// The method does not exist.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// The parameters are invalid.
    pub const INVALID_PARAMS: i64 = -32602;
    /// The request breaks a rule of the scene or of the core.
    pub const REJECTED: i64 = 1;
    /// A transaction names a surface node that is gone, for example of a
    /// window that closed in the meantime.
    pub const GONE_SURFACE: i64 = 2;
    /// The client speaks a newer protocol than the core (D29).
    pub const UNSUPPORTED_PROTOCOL: i64 = 3;
    /// The client lacks a capability that the method needs, for example
    /// `screen.capture` without a grant.
    pub const MISSING_CAPABILITY: i64 = 4;
    /// A command names a node that does not exist; a surface node that
    /// went away during a wait keeps code 2.
    pub const UNKNOWN_NODE: i64 = 5;
    /// A limit of the core is reached: too many connections, transactions
    /// or calls, or a line and an answer that are too long.
    pub const LIMIT_REACHED: i64 = 6;
    /// The client name belongs to a live connection or to a client socket.
    pub const NAME_TAKEN: i64 = 7;
    /// The method works only on the admin socket of `choractl`, not on a
    /// client socket.
    pub const ADMIN_SOCKET_ONLY: i64 = 8;
    /// A rule of client sockets refuses the request; the API reference
    /// lists the rules under Client sockets.
    pub const CLIENT_SOCKET_RULE: i64 = 9;

    /// An error with a code and a message and no `data`.
    #[must_use]
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            data: None,
            code,
            message: message.into(),
        }
    }

    /// Adds the machine readable part.
    #[must_use]
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Whether the core rejected a transaction because a surface node is
    /// gone. A missing node of the client itself is no such case.
    #[must_use]
    pub fn is_gone_surface(&self) -> bool {
        self.code == Self::GONE_SURFACE
    }

    /// The index of the command that failed a transaction, from the
    /// message `command N: ...`.
    #[must_use]
    pub fn command_index(&self) -> Option<usize> {
        let rest = self.message.strip_prefix("command ")?;
        rest[..rest.find(':')?].parse().ok()
    }
}

impl Message {
    /// A successful response.
    #[must_use]
    pub fn result(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            result: Some(result),
            error: None,
            method: None,
            params: None,
        }
    }

    /// A failed response.
    #[must_use]
    pub fn error(id: Value, error: RpcError) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: Some(id),
            result: None,
            error: Some(error),
            method: None,
            params: None,
        }
    }

    /// An event notification.
    ///
    /// # Panics
    ///
    /// Never: events always serialize.
    #[must_use]
    pub fn event(event: &Event) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: None,
            result: None,
            error: None,
            method: Some("event".into()),
            params: Some(serde_json::to_value(event).expect("events always serialize")),
        }
    }
}

/// Parameters of `hello`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    /// The client name, the owner of the nodes of this client.
    pub name: String,
    /// Keep the nodes after the last connection of the name closes.
    #[serde(default)]
    pub keep_nodes: bool,
    /// Keep the nodes for a grace time after the last connection of the
    /// name closes, and get them back with a `hello` within it. With
    /// `keep_nodes`, `resume` has no effect.
    #[serde(default)]
    pub resume: bool,
    /// The protocol version of the client (D29). Missing from a client
    /// that predates the versioning. [`Client`] fills in [`PROTOCOL`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<u64>,
    /// The features the client wants to use. The core answers with its
    /// own list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
}

/// The `hello` the client library sends: `protocol` and `features` filled
/// in from this build where the caller left them out.
fn hello_request(hello: &Hello) -> Hello {
    let mut hello = hello.clone();
    hello.protocol.get_or_insert(PROTOCOL);
    if hello.features.is_empty() {
        hello.features = FEATURES.iter().map(ToString::to_string).collect();
    }
    hello
}

/// The warning for a core that answers `hello` with a protocol other than
/// [`PROTOCOL`]; `None` when both speak the same one.
fn protocol_warning(core_protocol: u64) -> Option<String> {
    (core_protocol != PROTOCOL).then(|| {
        format!(
            "warning: the core speaks protocol {core_protocol}, this client speaks protocol \
             {PROTOCOL}"
        )
    })
}

/// Parameters of `scene.query`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    /// See [`crate::Selector`]; every node if missing.
    #[serde(default)]
    pub selector: String,
    /// A JSON filter the nodes must also match (D32).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
}

impl Query {
    /// A query for a selector, without a filter.
    #[must_use]
    pub fn new(selector: impl Into<String>) -> Self {
        Self {
            selector: selector.into(),
            filter: None,
        }
    }
}

/// A JSON filter for `scene.query`, next to the selector string (D32).
/// All fields must match; an empty filter matches everything. `focused`
/// and `visible` only filter when they are true.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Filter {
    /// Nodes of this client name.
    #[serde(default)]
    pub owner: Option<String>,
    /// Nodes of this kind.
    #[serde(default)]
    pub kind: Option<crate::NodeKind>,
    /// Surfaces with this application id.
    #[serde(default)]
    pub app_id: Option<String>,
    /// Only the focused surface or node.
    #[serde(default)]
    pub focused: bool,
    /// Only nodes that show on some output.
    #[serde(default)]
    pub visible: bool,
    /// Only nodes that show on this output.
    #[serde(default)]
    pub output: Option<String>,
}

impl Filter {
    /// Whether a node matches every field. `focused` tells whether the
    /// core marks the node as focused; `rects` maps the name of every
    /// output where the node shows to its rectangle there.
    #[must_use]
    pub fn matches(&self, node: &crate::Node, focused: bool, rects: &Value) -> bool {
        self.owner
            .as_deref()
            .is_none_or(|owner| node.owner.as_deref() == Some(owner))
            && self.kind.is_none_or(|kind| node.kind == kind)
            && self.app_id.as_deref().is_none_or(|app_id| {
                node.surface.as_ref().and_then(|s| s.app_id.as_deref()) == Some(app_id)
            })
            && (!self.focused || focused)
            && (!self.visible || rects.as_object().is_some_and(|r| !r.is_empty()))
            && self
                .output
                .as_deref()
                .is_none_or(|output| rects.get(output).is_some())
    }
}

/// Parameters of `scene.apply`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Apply {
    /// The commands of the transaction.
    pub commands: Vec<Command>,
}

/// Parameters of `events.emit` (D65).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Emit {
    /// The name: the client name, a dot and a name of the client, for
    /// example `voice.state`.
    pub event: String,
    /// The data that the subscribers get.
    #[serde(default)]
    pub data: Value,
}

/// Parameters of `events.subscribe`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscribe {
    /// Event names, for example `surface.created` or `voice.state`, or
    /// patterns such as `voice.*`; every core event if empty (D65).
    #[serde(default)]
    pub events: Vec<String>,
}

/// Parameters of `input.grab`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grab {
    /// The binding, for example `Super+Return`.
    pub binding: crate::Binding,
}

/// Parameters of `output.reserve`: space at the edges of an output that the
/// work area leaves out for the life of the connection, as the exclusive
/// zone of a panel does.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reserve {
    /// The output name; the first output if missing.
    #[serde(default)]
    pub output: Option<String>,
    /// The space at each edge; no space removes the reserve.
    #[serde(default)]
    pub edges: Edges,
}

/// Space at the edges of an output, in logical pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edges {
    /// At the top.
    #[serde(default)]
    pub top: u32,
    /// At the right.
    #[serde(default)]
    pub right: u32,
    /// At the bottom.
    #[serde(default)]
    pub bottom: u32,
    /// At the left.
    #[serde(default)]
    pub left: u32,
}

impl Edges {
    /// Whether no edge has space.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The space of both, edge by edge.
    #[must_use]
    pub fn plus(self, other: Self) -> Self {
        Self {
            top: self.top.saturating_add(other.top),
            right: self.right.saturating_add(other.right),
            bottom: self.bottom.saturating_add(other.bottom),
            left: self.left.saturating_add(other.left),
        }
    }
}

/// Parameters of `output.configure`: scale and transform of an output,
/// on the path of `wlr-output-management` (D44).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfigure {
    /// The output name; the first output if missing.
    #[serde(default)]
    pub output: Option<String>,
    /// The scale, from 0.1 to 10; unchanged if missing.
    #[serde(default)]
    pub scale: Option<f64>,
    /// The transform; unchanged if missing.
    #[serde(default)]
    pub transform: Option<OutputTransform>,
}

/// Parameters and answer of `input.configure`, and the `input` of a view
/// (D35): the settings of the keyboard, the touchpads and the mice. A
/// missing field keeps its value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputSettings {
    /// The key repeat.
    #[serde(default, skip_serializing_if = "is_default")]
    pub keyboard: KeyboardSettings,
    /// Devices that can tap, by libinput.
    #[serde(default, skip_serializing_if = "is_default")]
    pub touchpad: PointerSettings,
    /// The other pointer devices, such as mice and trackpoints.
    #[serde(default, skip_serializing_if = "is_default")]
    pub mouse: PointerSettings,
}

/// The key repeat, for `wl_keyboard.repeat_info` and for the keys of
/// nodes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyboardSettings {
    /// Repeats per second, 0 to 100; 0 turns the repeat off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_rate: Option<u32>,
    /// Milliseconds before a held key repeats, 100 to 5000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat_delay: Option<u32>,
}

/// The libinput settings of a class of pointer devices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointerSettings {
    /// Tap to click.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tap: Option<bool>,
    /// Content follows the fingers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natural_scroll: Option<bool>,
    /// Swaps the left and the right button.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_handed: Option<bool>,
    /// The acceleration profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accel_profile: Option<AccelProfile>,
    /// The pointer speed, -1 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accel_speed: Option<f64>,
}

/// The acceleration profile of libinput.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccelProfile {
    /// A constant factor.
    Flat,
    /// Faster with the speed of the motion.
    Adaptive,
}

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

impl InputSettings {
    /// Checks the ranges.
    ///
    /// # Errors
    ///
    /// Names the first value out of its range.
    pub fn check(&self) -> Result<(), String> {
        let k = &self.keyboard;
        if k.repeat_rate.is_some_and(|r| r > 100) {
            return Err("keyboard.repeat_rate must be 0 to 100".into());
        }
        if k.repeat_delay.is_some_and(|d| !(100..=5000).contains(&d)) {
            return Err("keyboard.repeat_delay must be 100 to 5000".into());
        }
        for (class, p) in [("touchpad", &self.touchpad), ("mouse", &self.mouse)] {
            if p.accel_speed.is_some_and(|s| !(-1.0..=1.0).contains(&s)) {
                return Err(format!("{class}.accel_speed must be -1 to 1"));
            }
        }
        Ok(())
    }

    /// Takes the fields that `new` sets.
    pub fn merge(&mut self, new: &Self) {
        let (k, n) = (&mut self.keyboard, &new.keyboard);
        k.repeat_rate = n.repeat_rate.or(k.repeat_rate);
        k.repeat_delay = n.repeat_delay.or(k.repeat_delay);
        for (p, n) in [
            (&mut self.touchpad, &new.touchpad),
            (&mut self.mouse, &new.mouse),
        ] {
            p.tap = n.tap.or(p.tap);
            p.natural_scroll = n.natural_scroll.or(p.natural_scroll);
            p.left_handed = n.left_handed.or(p.left_handed);
            p.accel_profile = n.accel_profile.or(p.accel_profile);
            p.accel_speed = n.accel_speed.or(p.accel_speed);
        }
    }
}

/// The transform of an output, with the names of `wlr-randr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputTransform {
    /// As the panel is.
    #[serde(rename = "normal")]
    Normal,
    /// Turned by 90 degrees.
    #[serde(rename = "90")]
    Turn90,
    /// Turned by 180 degrees.
    #[serde(rename = "180")]
    Turn180,
    /// Turned by 270 degrees.
    #[serde(rename = "270")]
    Turn270,
    /// Mirrored.
    #[serde(rename = "flipped")]
    Flipped,
    /// Mirrored and turned by 90 degrees.
    #[serde(rename = "flipped-90")]
    Flipped90,
    /// Mirrored and turned by 180 degrees.
    #[serde(rename = "flipped-180")]
    Flipped180,
    /// Mirrored and turned by 270 degrees.
    #[serde(rename = "flipped-270")]
    Flipped270,
}

/// Parameters of `assets.put`: a file for image nodes, shader nodes and
/// effects, from a client that shares no directory with the core.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetPut {
    /// The file name, for example `glow.frag`: letters, digits, `.`, `_`
    /// and `-`, not first a `.`, at most 64 characters. A second put of
    /// the name replaces the file.
    pub name: String,
    /// The content in base64, at most 512 KiB decoded.
    pub data: String,
}

/// Parameters of `assets.remove`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRemove {
    /// The file name of `assets.put`.
    pub name: String,
}

/// The answer of `assets.put`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// The absolute path of the file, for `image`, `shader` and effects.
    pub path: String,
}

/// A method that needs a grant of the user (D33).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// `input.inject`: send synthetic input.
    #[serde(rename = "input.inject")]
    InputInject,
    /// `screen.capture`: capture an output, a group or a surface.
    #[serde(rename = "screen.capture")]
    ScreenCapture,
    /// `input.grab-any`: grab keys and buttons without Ctrl, Alt or Super
    /// (D45).
    #[serde(rename = "input.grab-any")]
    GrabAny,
    /// `methods.call`: call the methods of plugins. Checked only for a
    /// client socket; the main socket may always call.
    #[serde(rename = "methods.call")]
    MethodsCall,
    /// `activation.token`: get activation tokens for `choractl run`.
    /// The core checks it only for a client socket; `choractl mcp` also
    /// needs it on the main socket for `run_program`.
    #[serde(rename = "activation.token")]
    ActivationToken,
    /// `session.quit`: end chora. Checked only for a client socket.
    #[serde(rename = "session.quit")]
    SessionQuit,
    /// `methods.privileged`: call a privileged method of a plugin, one
    /// that can run commands, such as `settings.set`, `settings.reload`
    /// and the methods that the export marks `privileged`, on top of
    /// `methods.call`; also the core method `input.configure`. Checked
    /// only for a client socket.
    #[serde(rename = "methods.privileged")]
    MethodsPrivileged,
}

impl Capability {
    /// Every capability.
    pub const ALL: [Self; 7] = [
        Self::InputInject,
        Self::ScreenCapture,
        Self::GrabAny,
        Self::MethodsCall,
        Self::ActivationToken,
        Self::SessionQuit,
        Self::MethodsPrivileged,
    ];

    /// The name, which is also the name of the method.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::InputInject => "input.inject",
            Self::ScreenCapture => "screen.capture",
            Self::GrabAny => "input.grab-any",
            Self::MethodsCall => "methods.call",
            Self::ActivationToken => "activation.token",
            Self::SessionQuit => "session.quit",
            Self::MethodsPrivileged => "methods.privileged",
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for Capability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|c| c.name() == s)
            .ok_or_else(|| {
                let known = Self::ALL.map(Self::name).join(", ");
                format!("unknown capability {s:?}; known: {known}")
            })
    }
}

/// Parameters of `grants.set` and `grants.revoke`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantChange {
    /// The client name.
    pub client: String,
    /// The capabilities; for `grants.revoke`, all if empty.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

/// Parameters of `methods.export`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// The methods, for example `set_layout`. Each entry is a plain name,
    /// or an object with `name`, `description` and `input_schema`. Other
    /// clients call them with the client name, as `tiling.set_layout`.
    pub names: Vec<ExportMethod>,
}

/// One method of a `methods.export`, plain or described.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExportMethod {
    /// The method name alone, without description and schema.
    Plain(String),
    /// The name with what a caller needs to know about it. Unknown keys in
    /// the object are refused, so a typo in `description` does not vanish.
    Described(DescribedMethod),
}

/// The described form of an [`ExportMethod`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DescribedMethod {
    /// The method name, for example `set_layout`.
    pub name: String,
    /// What the method does, in words for the caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A JSON schema of the parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
    /// A caller on a client socket needs the capability
    /// `methods.privileged`, for example for a method that runs a command.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub privileged: bool,
}

impl ExportMethod {
    /// The method name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Plain(name) | Self::Described(DescribedMethod { name, .. }) => name,
        }
    }

    /// The description, unless the exporter left it out.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        match self {
            Self::Plain(_) => None,
            Self::Described(d) => d.description.as_deref(),
        }
    }

    /// The JSON schema of the parameters, unless the exporter left it out.
    #[must_use]
    pub fn input_schema(&self) -> Option<&Value> {
        match self {
            Self::Plain(_) => None,
            Self::Described(d) => d.input_schema.as_ref(),
        }
    }

    /// Whether a caller on a client socket needs `methods.privileged`.
    /// `settings.set` and `settings.reload` always do.
    #[must_use]
    pub fn privileged(&self) -> bool {
        matches!(self.name(), "settings.set" | "settings.reload")
            || matches!(self, Self::Described(d) if d.privileged)
    }
}

impl From<&str> for ExportMethod {
    fn from(name: &str) -> Self {
        Self::Plain(name.to_owned())
    }
}

/// Parameters of `methods.call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodCall {
    /// The client that exported the method.
    pub client: String,
    /// The method name.
    pub method: String,
    /// The parameters, any JSON value.
    #[serde(default)]
    pub params: Value,
}

/// Parameters of `methods.return`: the answer to a `method.called` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodReturn {
    /// The `call` of the event.
    pub call: u64,
    /// The result, unless `error` is set.
    #[serde(default)]
    pub result: Value,
    /// The error, if the method failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// Parameters of `screen.capture`: an output, or a node with its
/// descendants. Without both, the first output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    /// The output name, for example `DP-1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// The node id, for example a group or a surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// Shrinks the picture until its longer side has at most this many
    /// pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_side: Option<u32>,
}

/// The result of `screen.capture`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// Pixels per logical pixel: a pixel at (px, py) shows the point
    /// `origin + (px, py) / scale`.
    pub scale: f64,
    /// The point of the top left pixel: in the logical coordinates of the
    /// output, so (0, 0), or in the coordinates of the node.
    pub origin: [f64; 2],
    /// A PNG file in base64.
    pub png: String,
    /// The nodes whose layers the last frame left out of the output, since
    /// they did not fit into the pixel budget. A capture shows them, the
    /// screen does not.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left_out: Vec<String>,
}

/// Parameters of `input.inject`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Inject {
    /// The events, in order. Keys and buttons that they leave down go up
    /// at the end, unless a button press holds.
    pub events: Vec<InjectEvent>,
    /// Check every event and answer ok or the error, without sending input.
    #[serde(default)]
    pub dry_run: bool,
}

/// One synthetic input event of `input.inject`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum InjectEvent {
    /// Moves the pointer, either to `position` in the logical coordinates
    /// of an output, the first one by default, or to `point` inside the
    /// node `node`.
    PointerMove {
        /// The point in the logical coordinates of an output.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        position: Option<[f64; 2]>,
        /// The output name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        /// The node, found by its id.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        node: Option<String>,
        /// The point inside the node, in the coordinates of the node.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        point: Option<[f64; 2]>,
    },
    /// Presses or releases a pointer button.
    Button {
        /// The Linux button code; 272, the left button, by default.
        #[serde(default = "left_button")]
        button: u32,
        /// Pressed or released.
        state: PressState,
        /// A press that stays down after the request ends; a later event
        /// releases it.
        #[serde(default)]
        hold: bool,
    },
    /// Presses and releases a pointer button.
    Click {
        /// The Linux button code; 272, the left button, by default.
        #[serde(default = "left_button")]
        button: u32,
    },
    /// Scrolls by logical pixels; a positive `dy` scrolls down.
    Scroll {
        /// Horizontal.
        #[serde(default)]
        dx: f64,
        /// Vertical.
        #[serde(default)]
        dy: f64,
    },
    /// Presses or releases a key.
    Key {
        /// The key.
        key: Key,
        /// Pressed or released.
        state: PressState,
    },
    /// Presses a key with modifiers, then releases them all, for example
    /// `Ctrl+Shift+t`.
    Shortcut {
        /// The modifiers and the key.
        keys: crate::Binding,
    },
    /// Types a text with the keys of the first layout of the keymap.
    Text {
        /// The text; a newline presses Return, a tab presses Tab.
        text: String,
    },
}

fn left_button() -> u32 {
    272
}

/// A key: a keysym name, for example `Return` or `a`, or a Linux key code,
/// for example 28 for Return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged, expecting = "a key of a keysym name or a Linux key code")]
pub enum Key {
    /// A Linux key code, as evdev numbers them.
    Code(u32),
    /// A keysym name.
    Name(String),
}

/// Whether a key or a button goes down or up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PressState {
    /// Down.
    #[default]
    Pressed,
    /// Up.
    Released,
}

/// Errors of the blocking [`Client`].
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The socket failed.
    #[error("socket: {0}")]
    Io(#[from] io::Error),
    /// A message was no valid JSON.
    #[error("invalid message: {0}")]
    Json(#[from] serde_json::Error),
    /// The core rejected the request.
    #[error(transparent)]
    Rpc(#[from] RpcError),
    /// The core closed the connection.
    #[error("the core closed the connection")]
    Closed,
    /// `CHORA_SOCKET` is not set.
    #[error("{SOCKET_ENV} is not set")]
    NoSocket,
}

/// A blocking client of the IPC socket.
pub struct Client {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    next_id: u64,
    events: VecDeque<Event>,
    /// The start of a line that a timeout interrupted, as bytes, since a
    /// timeout can split a character.
    partial: Vec<u8>,
    /// The version of the core from its answer to `hello`.
    core_version: Option<String>,
}

impl Client {
    /// Connects to the socket in `CHORA_SOCKET` and sends `hello`.
    ///
    /// # Errors
    ///
    /// The variable is missing, or the connection fails.
    pub fn from_env(hello: &Hello) -> Result<Self, ClientError> {
        let path = std::env::var_os(SOCKET_ENV).ok_or(ClientError::NoSocket)?;
        Self::connect(path, hello)
    }

    /// Connects to a socket and sends `hello`.
    ///
    /// # Errors
    ///
    /// The connection or the `hello` request fails.
    pub fn connect(path: impl AsRef<Path>, hello: &Hello) -> Result<Self, ClientError> {
        Self::from_stream(UnixStream::connect(path)?, hello)
    }

    /// Uses a connected stream and sends `hello`.
    ///
    /// # Errors
    ///
    /// The `hello` request fails.
    pub fn from_stream(writer: UnixStream, hello: &Hello) -> Result<Self, ClientError> {
        let reader = BufReader::new(writer.try_clone()?);
        let mut client = Self {
            reader,
            writer,
            next_id: 1,
            events: VecDeque::new(),
            partial: Vec::new(),
            core_version: None,
        };
        let answer: Value = client.call("hello", hello_request(hello))?;
        client.core_version = answer["version"].as_str().map(str::to_owned);
        if let Some(warning) = answer["protocol"].as_u64().and_then(protocol_warning) {
            static WARNED_PROTOCOL: std::sync::Once = std::sync::Once::new();
            WARNED_PROTOCOL.call_once(|| eprintln!("{warning}"));
        }
        if client.core_version.as_deref() != Some(VERSION) {
            static WARNED: std::sync::Once = std::sync::Once::new();
            WARNED.call_once(|| {
                let core = client
                    .core_version
                    .as_deref()
                    .unwrap_or("of an older build");
                eprintln!(
                    "warning: chora {core} and this client {VERSION} are different builds; \
                     requests may fail"
                );
            });
        }
        Ok(client)
    }

    /// The version of the core, from its answer to `hello`; `None` for a
    /// core that does not tell it.
    #[must_use]
    pub fn core_version(&self) -> Option<&str> {
        self.core_version.as_deref()
    }

    /// Applies a transaction, and returns the error of the core if it
    /// rejected it. A command on a surface that is gone, for example of a
    /// window that closed in the meantime, drops out, and the rest goes
    /// again: a plugin that cleans up after one closed window must not lose
    /// that work to a second one.
    ///
    /// # Errors
    ///
    /// The socket fails.
    pub fn apply(&mut self, mut commands: Vec<Command>) -> Result<Option<RpcError>, ClientError> {
        while !commands.is_empty() {
            let apply = Apply {
                commands: commands.clone(),
            };
            match self.call::<_, Value>("scene.apply", apply) {
                Ok(_) => return Ok(None),
                Err(ClientError::Rpc(err)) if err.is_gone_surface() => {
                    match err.command_index().filter(|i| *i < commands.len()) {
                        Some(i) => drop(commands.remove(i)),
                        None => return Ok(Some(err)),
                    }
                }
                Err(ClientError::Rpc(err)) => return Ok(Some(err)),
                Err(err) => return Err(err),
            }
        }
        Ok(None)
    }

    /// Sends a request and waits for its response. Events that arrive in
    /// between wait in a queue for [`Client::next_event`].
    ///
    /// # Errors
    ///
    /// The socket fails, or the core rejects the request.
    pub fn call<P: Serialize, R: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<R, ClientError> {
        let id = self.next_id;
        self.next_id += 1;
        let request = Request {
            jsonrpc: "2.0".into(),
            id: Some(id.into()),
            method: method.into(),
            params: serde_json::to_value(params)?,
        };
        let mut line = serde_json::to_string(&request)?;
        line.push('\n');
        if let Err(err) = self.writer.write_all(line.as_bytes()) {
            // A core that refuses a connection sends the reason first.
            return Err(match self.read() {
                Ok(Message {
                    error: Some(error), ..
                }) => error.into(),
                _ => err.into(),
            });
        }
        loop {
            let message = self.read()?;
            // An error without id answers a request that the core could not
            // parse, so it belongs to this call.
            let mine = match &message.id {
                Some(v) => v.as_u64() == Some(id),
                None => message.error.is_some(),
            };
            if !mine {
                self.queue(message);
                continue;
            }
            if let Some(error) = message.error {
                return Err(error.into());
            }
            return Ok(serde_json::from_value(
                message.result.unwrap_or(Value::Null),
            )?);
        }
    }

    /// Answers a `method.called` event.
    ///
    /// # Errors
    ///
    /// The socket fails, or the call is unknown, for example after its
    /// timeout.
    pub fn answer(
        &mut self,
        call: u64,
        result: Result<Value, RpcError>,
    ) -> Result<(), ClientError> {
        let (result, error) = match result {
            Ok(value) => (value, None),
            Err(error) => (Value::Null, Some(error)),
        };
        let answer = MethodReturn {
            call,
            result,
            error,
        };
        self.call::<_, Value>("methods.return", answer).map(drop)
    }

    /// The next event, from the queue or from the socket.
    ///
    /// # Errors
    ///
    /// The socket fails or closes.
    pub fn next_event(&mut self) -> Result<Event, ClientError> {
        loop {
            if let Some(event) = self.events.pop_front() {
                return Ok(event);
            }
            let message = self.read()?;
            self.queue(message);
        }
    }

    /// The next event, or `None` when none arrives within a time, for
    /// example for the frames of an effect.
    ///
    /// # Errors
    ///
    /// The socket fails or closes.
    pub fn next_event_timeout(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        if let Some(event) = self.events.pop_front() {
            return Ok(Some(event));
        }
        let deadline = Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return Ok(None);
            }
            self.reader.get_ref().set_read_timeout(Some(left))?;
            let message = self.read();
            self.reader.get_ref().set_read_timeout(None)?;
            match message {
                Ok(message) => self.queue(message),
                Err(ClientError::Io(e))
                    if matches!(
                        e.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
            if let Some(event) = self.events.pop_front() {
                return Ok(Some(event));
            }
        }
    }

    /// Keeps an event notification. Events of a newer core that this crate
    /// does not know are skipped.
    fn queue(&mut self, message: Message) {
        if message.method.as_deref() != Some("event") {
            return;
        }
        if let Some(event) = message.params.and_then(|p| serde_json::from_value(p).ok()) {
            self.events.push_back(event);
        }
    }

    fn read(&mut self) -> Result<Message, ClientError> {
        // A timeout keeps the bytes of the line so far for the next read.
        if self.reader.read_until(b'\n', &mut self.partial)? == 0 {
            return Err(ClientError::Closed);
        }
        if self.partial.last() != Some(&b'\n') {
            return Err(ClientError::Closed);
        }
        let line = std::mem::take(&mut self.partial);
        Ok(serde_json::from_slice(&line)?)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::BufRead, thread};

    use super::*;

    #[test]
    fn apply_drops_the_command_of_a_gone_surface() {
        let (client, server) = UnixStream::pair().unwrap();
        let core = thread::spawn(move || {
            let mut reader = BufReader::new(server.try_clone().unwrap());
            let mut writer = server;
            let mut requests = Vec::new();
            for answer in [
                r#"{"jsonrpc":"2.0","id":1,"result":null}"#,
                r#"{"jsonrpc":"2.0","id":2,"error":{"code":2,"message":"command 1: node \"s9\" does not exist"}}"#,
                r#"{"jsonrpc":"2.0","id":3,"result":null}"#,
            ] {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                requests.push(line);
                writeln!(writer, "{answer}").unwrap();
            }
            requests
        });
        let hello = Hello {
            name: "wm".into(),
            ..Hello::default()
        };
        let mut client = Client::from_stream(client, &hello).unwrap();
        let commands: Vec<Command> = serde_json::from_str(
            r#"[{"remove": {"id": "frame-s8"}}, {"focus": {"surface": "s9"}}]"#,
        )
        .unwrap();
        assert_eq!(client.apply(commands).unwrap(), None);
        let requests = core.join().unwrap();
        assert!(requests[1].contains("s9"));
        assert!(requests[2].contains("frame-s8") && !requests[2].contains("s9"));
    }

    /// A fake core that answers `hello`, then writes `lines` after the next
    /// request.
    fn fake_core(lines: &'static [&'static str]) -> UnixStream {
        let (client, server) = UnixStream::pair().unwrap();
        thread::spawn(move || {
            let mut reader = BufReader::new(server.try_clone().unwrap());
            let mut writer = server;
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(writer, r#"{{"jsonrpc":"2.0","id":1,"result":null}}"#).unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            for l in lines {
                writeln!(writer, "{l}").unwrap();
            }
        });
        client
    }

    fn hello() -> Hello {
        Hello {
            name: "test".into(),
            ..Hello::default()
        }
    }

    #[test]
    fn the_client_sends_its_protocol_and_keeps_the_answer() {
        let (client, server) = UnixStream::pair().unwrap();
        let core = thread::spawn(move || {
            let mut reader = BufReader::new(server);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(
                reader.get_mut(),
                r#"{{"jsonrpc":"2.0","id":1,"result":{{"version":"test","protocol":1,"features":[]}}}}"#
            )
            .unwrap();
            line
        });
        let client = Client::from_stream(client, &hello()).unwrap();
        assert_eq!(client.core_version(), Some("test"));
        let request: Value = serde_json::from_str(&core.join().unwrap()).unwrap();
        assert_eq!(request["params"]["protocol"], serde_json::json!(PROTOCOL));
    }

    #[test]
    fn the_hello_request_fills_protocol_and_features_of_this_build() {
        let filled = hello_request(&hello());
        assert_eq!(filled.protocol, Some(PROTOCOL));
        assert_eq!(
            filled.features,
            FEATURES
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        );
        let wanted = Hello {
            features: vec!["drag".into()],
            ..hello()
        };
        assert_eq!(hello_request(&wanted).features, vec!["drag".to_owned()]);
    }

    #[test]
    fn the_client_warns_when_the_core_protocol_differs() {
        assert!(protocol_warning(PROTOCOL).is_none());
        for other in [PROTOCOL + 1, PROTOCOL - 1] {
            let warning = protocol_warning(other).expect("a warning for a differing protocol");
            assert!(
                warning.contains(&format!("core speaks protocol {other}"))
                    && warning.contains(&format!("client speaks protocol {PROTOCOL}")),
                "{warning}"
            );
        }
    }

    #[test]
    fn unknown_events_do_not_break_a_call() {
        let stream = fake_core(&[
            r#"{"jsonrpc":"2.0","method":"event","params":{"event":"output.added"}}"#,
            r#"{"jsonrpc":"2.0","id":2,"result":[]}"#,
        ]);
        let mut client = Client::from_stream(stream, &hello()).unwrap();
        let nodes: Vec<Value> = client.call("scene.query", Value::Null).unwrap();
        assert_eq!(nodes, Vec::<Value>::new());
    }

    #[test]
    fn an_error_without_id_ends_the_call() {
        let stream = fake_core(&[
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"parse error"}}"#,
        ]);
        let mut client = Client::from_stream(stream, &hello()).unwrap();
        let err = client
            .call::<_, Value>("scene.query", Value::Null)
            .unwrap_err();
        assert!(
            matches!(err, ClientError::Rpc(RpcError { code: -32700, .. })),
            "{err}"
        );
    }

    #[test]
    fn the_client_keeps_the_version_of_the_core() {
        let (client, server) = UnixStream::pair().unwrap();
        writeln!(
            &server,
            r#"{{"jsonrpc":"2.0","id":1,"result":{{"version":"0.0.1+abc"}}}}"#
        )
        .unwrap();
        let client = Client::from_stream(client, &hello()).unwrap();
        assert_eq!(client.core_version(), Some("0.0.1+abc"));
        let old = Client::from_stream(fake_core(&[]), &hello()).unwrap();
        assert_eq!(old.core_version(), None);
        assert!(VERSION.starts_with(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn a_refused_connection_gives_its_reason() {
        let (client, server) = UnixStream::pair().unwrap();
        writeln!(
            &server,
            r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":1,"message":"too many connections"}}}}"#
        )
        .unwrap();
        drop(server);
        let Err(err) = Client::from_stream(client, &hello()) else {
            panic!("the hello fails");
        };
        assert_eq!(err.to_string(), "too many connections (code 1)");
    }

    #[test]
    fn a_timeout_keeps_the_start_of_a_line() {
        let (client, server) = UnixStream::pair().unwrap();
        let mut writer = server.try_clone().unwrap();
        let core = thread::spawn(move || {
            let mut reader = BufReader::new(server);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(writer, r#"{{"jsonrpc":"2.0","id":1,"result":null}}"#).unwrap();
            write!(writer, r#"{{"jsonrpc":"2.0","method":"event","#).unwrap();
            writer.flush().unwrap();
            writer
        });
        let mut client = Client::from_stream(client, &hello()).unwrap();
        let mut writer = core.join().unwrap();
        let short = Duration::from_millis(50);
        assert!(client.next_event_timeout(short).unwrap().is_none());
        writeln!(
            writer,
            r#""params":{{"event":"animation.done","node":"n"}}}}"#
        )
        .unwrap();
        let event = client.next_event_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            event,
            Some(Event::AnimationDone {
                node: Some("n".into()),
                output: None
            })
        );
    }

    #[test]
    fn a_timeout_inside_a_character_keeps_its_bytes() {
        let (client, server) = UnixStream::pair().unwrap();
        let mut writer = server.try_clone().unwrap();
        let core = thread::spawn(move || {
            let mut reader = BufReader::new(server);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(writer, r#"{{"jsonrpc":"2.0","id":1,"result":null}}"#).unwrap();
            let event = r#"{"jsonrpc":"2.0","method":"event","params":{"event":"animation.done","node":"é"}}"#;
            let bytes = event.as_bytes();
            let cut = event.find('é').unwrap() + 1;
            writer.write_all(&bytes[..cut]).unwrap();
            writer.flush().unwrap();
            (writer, bytes[cut..].to_vec())
        });
        let mut client = Client::from_stream(client, &hello()).unwrap();
        let (mut writer, rest) = core.join().unwrap();
        assert!(
            client
                .next_event_timeout(Duration::from_millis(50))
                .unwrap()
                .is_none()
        );
        writer.write_all(&rest).unwrap();
        writer.write_all(b"\n").unwrap();
        let event = client.next_event_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            event,
            Some(Event::AnimationDone {
                node: Some("é".into()),
                output: None
            })
        );
    }
}
