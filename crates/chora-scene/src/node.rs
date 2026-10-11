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
    /// A PNG image, for example a part of a window border.
    Image,
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

/// The most cells of a mesh along one side.
pub const MAX_MESH_CELLS: u32 = 64;

/// A grid that deforms how a node shows, for example for wobbly windows:
/// `cols` by `rows` cells over the size of the node. `points` holds the
/// place of each grid point in node coordinates, row by row from the top
/// left, `(cols + 1) * (rows + 1)` points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh {
    /// Cells along x, from 1 to 64.
    pub cols: u32,
    /// Cells along y, from 1 to 64.
    pub rows: u32,
    /// The grid points.
    pub points: Vec<[f64; 2]>,
}

impl Mesh {
    /// The undeformed grid over a size.
    #[must_use]
    pub fn flat(cols: u32, rows: u32, [w, h]: [f64; 2]) -> Self {
        let points = (0..=rows)
            .flat_map(|r| {
                (0..=cols).map(move |c| {
                    [
                        w * f64::from(c) / f64::from(cols),
                        h * f64::from(r) / f64::from(rows),
                    ]
                })
            })
            .collect();
        Self { cols, rows, points }
    }

    /// Whether the grid has 1 to 64 cells per side and a point for each
    /// grid point.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let cells = 1..=MAX_MESH_CELLS;
        cells.contains(&self.cols)
            && cells.contains(&self.rows)
            && self.points.len() == ((self.cols + 1) * (self.rows + 1)) as usize
    }
}

/// The most effects of one node.
pub const MAX_EFFECTS: usize = 8;
/// The most uniforms of one effect.
pub const MAX_UNIFORMS: usize = 16;

/// A fragment shader that changes how a layer looks, with its inputs. The
/// core applies the effects of a node one after another to its texture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShaderEffect {
    /// The absolute path of a GLSL ES fragment shader.
    pub shader: String,
    /// Values of uniforms of the shader by name: a number for a `float`,
    /// two to four for a `vec2` to `vec4`.
    #[serde(default)]
    pub uniforms: BTreeMap<String, Uniform>,
}

/// The value of a uniform of a shader.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Uniform {
    /// A `float`.
    Float(f64),
    /// A `vec2`, `vec3` or `vec4`.
    Vector(Vec<f64>),
}

impl Uniform {
    /// The numbers of the value.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        match self {
            Self::Float(v) => std::slice::from_ref(v),
            Self::Vector(v) => v,
        }
    }
}

/// Who draws the frame of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decoration {
    /// A plugin: the client asked for it through `xdg-decoration`.
    Server,
    /// The client itself, for example a GTK program.
    Client,
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
    /// Who draws the frame of the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decoration: Option<Decoration>,
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

impl Node {
    /// Whether the node renders into a texture of its size first: a node
    /// with a size and a mesh, or with a transform that is not a plain 2D
    /// translation and scale, for example a rotation or a perspective.
    #[must_use]
    pub fn is_layer(&self) -> bool {
        self.id != ROOT
            && self.props.size.is_some()
            && (self.props.mesh.is_some()
                || !self.props.effects.is_empty()
                || self.props.transform.scale_2d().is_none())
    }
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
    /// Width and height of a rect or an image. An image without a size
    /// has the size of its file. For a group, the area that a layer
    /// shows.
    pub size: Option<[f64; 2]>,
    /// RGBA color of a rect, each from 0 to 1.
    pub color: Option<[f64; 4]>,
    /// The absolute path of the PNG file of an image.
    pub image: Option<String>,
    /// The insets of an image that keep their size when the image
    /// stretches, in pixels of the file: left, top, right, bottom. The
    /// edges between them stretch in one direction, the center in both.
    pub slice: Option<[f64; 4]>,
    /// A grid that deforms the node and its descendants.
    pub mesh: Option<Mesh>,
    /// Shaders that change how the node and its descendants look.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<ShaderEffect>,
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
            image: None,
            slice: None,
            mesh: None,
            effects: Vec::new(),
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
    /// See [`Props::image`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// See [`Props::slice`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slice: Option<[f64; 4]>,
    /// See [`Props::mesh`]; a mesh without points removes the mesh.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<Mesh>,
    /// See [`Props::effects`]; an empty list removes the effects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<ShaderEffect>>,
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
        self.apply_drawing(update);
    }

    /// Applies the properties of rects and images.
    fn apply_drawing(&mut self, update: &PropsUpdate) {
        if update.size.is_some() {
            self.size = update.size;
        }
        if update.color.is_some() {
            self.color = update.color;
        }
        if update.image.is_some() {
            self.image.clone_from(&update.image);
        }
        if update.slice.is_some() {
            self.slice = update.slice;
        }
        if let Some(mesh) = &update.mesh {
            self.mesh = (!mesh.points.is_empty()).then(|| mesh.clone());
        }
        if let Some(effects) = &update.effects {
            self.effects.clone_from(effects);
        }
    }
}
