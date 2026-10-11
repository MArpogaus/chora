//! Nodes of the scene and their properties.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Transform;

/// The id of the root group. Every other node is a descendant of it.
pub const ROOT: &str = "root";

/// The kind of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    /// Contains other nodes.
    Group,
    /// Shows a Wayland client.
    Surface,
    /// A colored rectangle.
    Rect,
}

/// How a node takes part in pointer input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    /// The node receives input.
    #[default]
    Accept,
    /// Input goes to the nodes below.
    Pass,
    /// The node takes the input and gives it to nobody.
    Block,
}

/// What the client tells about a surface.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SurfaceInfo {
    /// The application id, for example `foot`.
    pub app_id: Option<String>,
    /// The window title.
    pub title: Option<String>,
    /// The size of the window geometry, in logical pixels.
    pub size: Option<[i32; 2]>,
}

/// One element of the scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// The stable id of the node.
    pub id: String,
    /// The kind of the node.
    pub kind: NodeKind,
    /// The client name of the owner; `None` for the root and for surfaces
    /// without an owner.
    pub owner: Option<String>,
    /// The parent; `None` for the root and for detached surfaces.
    pub parent: Option<String>,
    /// The children, bottom first.
    pub children: Vec<String>,
    /// The properties that clients can change.
    #[serde(flatten)]
    pub props: Props,
    /// The client data of a surface node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface: Option<SurfaceInfo>,
}

/// The properties that clients can change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Props {
    /// Relative to the parent.
    pub transform: Transform,
    /// From 0 to 1, multiplied with the opacity of the parent.
    pub opacity: f64,
    /// An invisible node hides its descendants.
    pub visible: bool,
    /// Pointer input behavior.
    pub input: InputMode,
    /// Free key/value pairs to tag and find nodes.
    pub meta: BTreeMap<String, String>,
    /// Width and height of a rect.
    pub size: Option<[f64; 2]>,
    /// RGBA color of a rect, each from 0 to 1.
    pub color: Option<[f64; 4]>,
}

impl Default for Props {
    fn default() -> Self {
        Self {
            transform: Transform::IDENTITY,
            opacity: 1.0,
            visible: true,
            input: InputMode::Accept,
            meta: BTreeMap::new(),
            size: None,
            color: None,
        }
    }
}

/// A partial update of [`Props`]: every field that is set replaces the old
/// value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropsUpdate {
    /// See [`Props::transform`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,
    /// See [`Props::opacity`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// See [`Props::visible`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    /// See [`Props::input`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputMode>,
    /// Keys to set; an empty value removes the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<BTreeMap<String, String>>,
    /// See [`Props::size`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<[f64; 2]>,
    /// See [`Props::color`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<[f64; 4]>,
}

impl Props {
    /// Applies an update.
    pub fn apply(&mut self, update: &PropsUpdate) {
        if let Some(t) = update.transform {
            self.transform = t;
        }
        if let Some(o) = update.opacity {
            self.opacity = o.clamp(0.0, 1.0);
        }
        if let Some(v) = update.visible {
            self.visible = v;
        }
        if let Some(i) = update.input {
            self.input = i;
        }
        for (k, v) in update.meta.iter().flatten() {
            if v.is_empty() {
                self.meta.remove(k);
            } else {
                self.meta.insert(k.clone(), v.clone());
            }
        }
        if update.size.is_some() {
            self.size = update.size;
        }
        if update.color.is_some() {
            self.color = update.color;
        }
    }
}
