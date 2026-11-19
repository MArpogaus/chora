//! The limits and checks of properties, meta data and effects.

use super::{
    BLURS, BTreeMap, FONT_SIZES, LETTER_SPACINGS, LINE_HEIGHTS, MAX_EFFECTS, MAX_FEATURES,
    MAX_LENGTH, MAX_MESH_CELLS, MAX_NODE_TEXTURES, MAX_UNIFORMS, Node, NodeKind, Order,
    PERSPECTIVES, Props, PropsUpdate, SATURATIONS, SceneError, ShaderEffect, Uniform, WEIGHTS,
    apply_meta,
};

pub(super) const MAX_PATH: usize = 4096;

/// The most bytes of the text and the family of a text node.
pub(super) const MAX_TEXT: usize = 4096;
/// The most keys in the meta data of a node.
pub(super) const MAX_META_KEYS: usize = 32;
/// The longest key or value of the meta data, in bytes.
pub(super) const MAX_META_VALUE: usize = 1024;
/// The most bytes of all keys and values of the meta data of a node.
pub(super) const MAX_META_BYTES: usize = 4096;

/// The meta data of a node after an update stays within its limits, so
/// that no client fills the memory of the core or the answers of queries.
pub(super) fn check_meta(
    id: &str,
    meta: &BTreeMap<String, String>,
    props: &PropsUpdate,
) -> Result<(), SceneError> {
    let Some(update) = &props.meta else {
        return Ok(());
    };
    let mut after = meta.clone();
    apply_meta(&mut after, update);
    let long = after
        .iter()
        .any(|(k, v)| k.len() > MAX_META_VALUE || v.len() > MAX_META_VALUE);
    let bytes: usize = after.iter().map(|(k, v)| k.len() + v.len()).sum();
    if after.len() > MAX_META_KEYS || long || bytes > MAX_META_BYTES {
        return Err(SceneError::Invalid(format!(
            "the meta data of {id:?} takes at most {MAX_META_KEYS} keys, \
             {MAX_META_VALUE} bytes per key or value and {MAX_META_BYTES} bytes in all"
        )));
    }
    Ok(())
}

/// Only rects and images have a size, only rects and text a color, only
/// images a file and a slice, only text its text properties and no size.
/// Image paths are absolute, sizes and slices are not negative, text has
/// at most `MAX_TEXT` bytes, and all numbers are finite.
pub(super) fn check_props(id: &str, kind: NodeKind, props: &PropsUpdate) -> Result<(), SceneError> {
    check_kinds(id, kind, props)?;
    check_numbers(id, props)?;
    if props.order == Some(Order::Lock) {
        return Err(SceneError::Invalid(format!(
            "{id:?}: the lock order belongs to the core, so that no plugin draws above the lock"
        )));
    }
    if props
        .claim
        .flatten()
        .is_some_and(|c| !c.iter().all(|v| v.is_finite()) || c[2] < 0.0 || c[3] < 0.0)
    {
        return Err(SceneError::Invalid(format!(
            "the claim of {id:?} needs finite numbers and no negative size"
        )));
    }
    check_shapes(id, props)?;
    if props
        .mesh
        .as_ref()
        .is_some_and(|m| !m.points.is_empty() && !m.is_valid())
    {
        return Err(SceneError::Invalid(format!(
            "the mesh of {id:?} needs 1 to {MAX_MESH_CELLS} cells per side, a point per grid point, and no depth or a depth per point"
        )));
    }
    if let Some(effects) = &props.effects {
        check_effects(id, effects)?;
    }
    let paths = props.image.iter().chain(props.shader.iter());
    if paths
        .into_iter()
        .any(|p| !p.starts_with('/') || p.len() > MAX_PATH)
    {
        return Err(SceneError::Invalid(format!(
            "the file of {id:?} needs an absolute path"
        )));
    }
    if props
        .text
        .iter()
        .chain(props.family.iter())
        .any(|t| t.len() > MAX_TEXT)
    {
        return Err(SceneError::Invalid(format!(
            "the text and the family of {id:?} have at most {MAX_TEXT} bytes"
        )));
    }
    Ok(())
}

/// The properties of each kind of node.
/// A text node reports the size of its layout; a given size has no use.
pub(super) fn check_kinds(id: &str, kind: NodeKind, props: &PropsUpdate) -> Result<(), SceneError> {
    let wrong = |kinds| Err(SceneError::WrongKind(id.into(), kinds));
    if kind == NodeKind::Surface && (props.size.is_some() || props.clip.is_some()) {
        return wrong("group, rect, image or shader");
    }
    if kind != NodeKind::Rect && kind != NodeKind::Text && props.color.is_some() {
        return wrong("rect or text");
    }
    if kind != NodeKind::Rect
        && (props.radius.is_some() || props.border.is_some() || props.backdrop.is_some())
    {
        return wrong("rect");
    }
    if kind != NodeKind::Image && (props.image.is_some() || props.slice.is_some()) {
        return wrong("image");
    }
    let shader_only = props.shader.is_some() || props.uniforms.is_some() || props.forward.is_some();
    if kind != NodeKind::Shader && shader_only {
        return wrong("shader");
    }
    if kind != NodeKind::Group && (props.perspective.is_some() || props.order.is_some()) {
        return wrong("group");
    }
    if kind != NodeKind::Text
        && (props.text.is_some()
            || props.family.is_some()
            || props.font_size.is_some()
            || props.max_width.is_some()
            || props.line_height.is_some()
            || props.weight.is_some()
            || props.style.is_some()
            || props.letter_spacing.is_some()
            || props.features.is_some())
    {
        return wrong("text");
    }
    if kind == NodeKind::Text && props.size.is_some() {
        return wrong("group, rect, image or shader");
    }
    Ok(())
}

/// The radius, the border, the clip, the perspective and the uniforms of a
/// shader node.
pub(super) fn check_shapes(id: &str, props: &PropsUpdate) -> Result<(), SceneError> {
    if props
        .perspective
        .flatten()
        .is_some_and(|d| !PERSPECTIVES.contains(&d))
    {
        return Err(SceneError::Invalid(format!(
            "the perspective of {id:?} is not from 1 to 1000000"
        )));
    }
    let lengths = props
        .radius
        .iter()
        .chain(props.border.iter().flatten().map(|b| &b.width))
        .chain(props.clip.iter().flatten().map(|c| &c.radius));
    if lengths.copied().any(|n| !(0.0..=MAX_LENGTH).contains(&n)) {
        return Err(SceneError::Invalid(format!(
            "a radius or a border width of {id:?} is not from 0 to {MAX_LENGTH}"
        )));
    }
    if let Some(backdrop) = props.backdrop.flatten()
        && (!BLURS.contains(&backdrop.blur) || !SATURATIONS.contains(&backdrop.saturation))
    {
        return Err(SceneError::Invalid(format!(
            "the backdrop of {id:?} needs a blur from 2 to 128 and a saturation from 0 to 4"
        )));
    }
    if let Some(uniforms) = &props.uniforms {
        check_uniforms(id, uniforms, true)?;
        let samplers = uniforms.values().filter(|u| u.values().is_empty());
        if samplers.count() > MAX_NODE_TEXTURES {
            return Err(SceneError::Invalid(format!(
                "{id:?}: at most {MAX_NODE_TEXTURES} node and output textures"
            )));
        }
        let reads_itself = |u: &Uniform| matches!(u, Uniform::Texture(t) if t.node == id);
        if uniforms.values().any(reads_itself) {
            return Err(SceneError::Invalid(format!(
                "{id:?} cannot read itself as a texture"
            )));
        }
    }
    Ok(())
}

/// A node with `forward` reads exactly one virtual output, which the
/// pointer then enters.
pub(super) fn check_forward(
    id: &str,
    forward: bool,
    uniforms: &BTreeMap<String, Uniform>,
) -> Result<(), SceneError> {
    let outputs = uniforms
        .values()
        .filter(|u| matches!(u, Uniform::Output(_)));
    if forward && outputs.count() != 1 {
        return Err(SceneError::Invalid(format!(
            "{id:?}: forward needs exactly one uniform {{\"output\": NAME}}"
        )));
    }
    Ok(())
}

/// Every number is finite, and sizes are not negative.
pub(super) fn check_numbers(id: &str, props: &PropsUpdate) -> Result<(), SceneError> {
    let numbers = props
        .transform
        .iter()
        .flat_map(|t| t.matrix().iter())
        .chain(props.size.iter().flatten())
        .chain(props.color.iter().flatten())
        .chain(props.slice.iter().flatten())
        .chain(props.mesh.iter().flat_map(|m| m.points.iter().flatten()))
        .chain(props.mesh.iter().flat_map(|m| m.depth.iter()))
        .chain(
            props
                .effects
                .iter()
                .flatten()
                .flat_map(|e| e.uniforms.values().flat_map(Uniform::values)),
        )
        .chain(props.opacity.iter())
        .chain(props.radius.iter())
        .chain(props.font_size.iter())
        .chain(props.max_width.iter())
        .chain(props.line_height.iter())
        .chain(props.letter_spacing.iter())
        .chain(
            props
                .border
                .iter()
                .flatten()
                .flat_map(|b| std::iter::once(&b.width).chain(&b.color)),
        )
        .chain(props.clip.iter().flatten().map(|c| &c.radius))
        .chain(
            props
                .uniforms
                .iter()
                .flat_map(|u| u.values().flat_map(Uniform::values)),
        );
    if numbers.copied().any(|n| !n.is_finite()) {
        return Err(SceneError::Invalid(format!(
            "a number of {id:?} is not finite"
        )));
    }
    let negative = props
        .size
        .iter()
        .flatten()
        .chain(props.slice.iter().flatten())
        .chain(props.font_size.iter())
        .chain(props.max_width.iter());
    if negative.copied().any(|n| n < 0.0) {
        return Err(SceneError::Invalid(format!("negative size for {id:?}")));
    }
    if props.font_size.is_some_and(|f| !FONT_SIZES.contains(&f)) {
        return Err(SceneError::Invalid(format!(
            "the font_size of {id:?} is not between 1 and 1024"
        )));
    }
    check_type(id, props)
}

/// The line height, the weight, the letter spacing and the features of a
/// text node: each within its range; a feature tag has four ASCII
/// characters.
pub(super) fn check_type(id: &str, props: &PropsUpdate) -> Result<(), SceneError> {
    let invalid = |what: &str| Err(SceneError::Invalid(format!("{id:?}: {what}")));
    if props
        .line_height
        .is_some_and(|l| !LINE_HEIGHTS.contains(&l))
    {
        return invalid("a line_height from 0.5 to 10");
    }
    if props.weight.is_some_and(|w| !WEIGHTS.contains(&w)) {
        return invalid("a weight from 1 to 1000");
    }
    if (props.letter_spacing).is_some_and(|l| !LETTER_SPACINGS.contains(&l)) {
        return invalid("a letter_spacing from -1 to 1");
    }
    let features = props.features.iter().flatten();
    if props
        .features
        .as_ref()
        .is_some_and(|f| f.len() > MAX_FEATURES)
    {
        return invalid(&format!("at most {MAX_FEATURES} features"));
    }
    let tag = |t: &str| t.len() == 4 && t.bytes().all(|b| (b' '..=b'~').contains(&b));
    if features.into_iter().any(|(t, _)| !tag(t)) {
        return invalid("a feature tag has four ASCII characters");
    }
    Ok(())
}

/// Removes the mesh, the effects and the shader of properties. Returns
/// whether there were some. A perspective stays: it costs no pass, and
/// without it a context would show and hit its planes flat.
/// The cost of the backdrop of a node in effects: one for each started
/// 256x256 logical pixels of its size, so that a large glass weighs more.
pub(super) fn backdrop_units(node: &Node) -> u64 {
    const UNIT: f64 = 256.0 * 256.0;
    let Some([w, h]) = node.props.size.filter(|_| node.props.backdrop.is_some()) else {
        return 0;
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let units = (w * h / UNIT).ceil().max(1.0) as u64;
    units
}

pub(super) fn strip(props: &mut Props) -> bool {
    let found = props.mesh.is_some()
        || !props.effects.is_empty()
        || props.shader.is_some()
        || props.backdrop.is_some();
    props.mesh = None;
    props.backdrop = None;
    props.effects.clear();
    props.shader = None;
    found
}

/// At most 8 effects with absolute paths, at most 16 uniforms each, with
/// names of GLSL identifiers and one to four numbers.
pub(super) fn check_effects(id: &str, effects: &[ShaderEffect]) -> Result<(), SceneError> {
    let invalid = |what: &str| Err(SceneError::Invalid(format!("{id:?}: {what}")));
    if effects.len() > MAX_EFFECTS {
        return invalid(&format!("at most {MAX_EFFECTS} effects"));
    }
    for effect in effects {
        if !effect.shader.starts_with('/') || effect.shader.len() > MAX_PATH {
            return invalid("a shader needs an absolute path");
        }
        check_uniforms(id, &effect.uniforms, false)?;
    }
    Ok(())
}

/// At most 16 uniforms with names of GLSL identifiers, each one to four
/// numbers, or a node texture where `textures` allows it.
pub(super) fn check_uniforms(
    id: &str,
    uniforms: &BTreeMap<String, Uniform>,
    textures: bool,
) -> Result<(), SceneError> {
    let invalid = |what: &str| Err(SceneError::Invalid(format!("{id:?}: {what}")));
    if uniforms.len() > MAX_UNIFORMS {
        return invalid(&format!("at most {MAX_UNIFORMS} uniforms per shader"));
    }
    for (name, value) in uniforms {
        let identifier = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && name.len() <= 64;
        let fits = match value {
            Uniform::Texture(t) => textures && !t.node.is_empty(),
            Uniform::Output(o) => textures && !o.output.is_empty(),
            _ => (1..=4).contains(&value.values().len()),
        };
        if !identifier || !fits {
            return invalid(&format!(
                "the uniform {name:?} needs a name and 1 to 4 numbers{}",
                if textures {
                    ", a node or an output"
                } else {
                    ""
                }
            ));
        }
    }
    Ok(())
}
