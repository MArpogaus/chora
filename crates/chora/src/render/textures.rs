//! The textures of layer nodes and node textures of a render.

use super::{
    GlesRenderer, GlesTexture, Layers, Node, NodeKind, Parts, Transform, View, flat_elements, mesh,
    project_point, settle_backdrops,
};

/// The scale of the texture of a layer node, in texture pixels per logical
/// pixel: the output scale, and for a plane without a mesh a power of two
/// step below it that follows the size on screen, so that a far plane
/// renders fewer pixels and a walk crosses a step rarely.
pub(super) fn texture_scale(
    layers: &mut Layers,
    node: &Node,
    transform: &Transform,
    view: &View,
) -> f64 {
    let scale = view.scale;
    if !view.stepped {
        return scale;
    }
    let ratio = (!transform.is_plain() && node.props.mesh.is_none())
        .then_some(node.props.size)
        .flatten()
        .and_then(|size| projected_ratio(transform, size));
    scale * layers.step(&node.id, ratio)
}

/// The most that a plane on screen is larger than its size: the longest
/// edge of its projection over the edge of its size. `None` when a corner
/// lies behind the eye.
pub(super) fn projected_ratio(transform: &Transform, [width, height]: [f64; 2]) -> Option<f64> {
    let at = |across, down| project_point(transform, across, down);
    let corners = [
        at(0.0, 0.0)?,
        at(width, 0.0)?,
        at(width, height)?,
        at(0.0, height)?,
    ];
    let edge = |from: usize, to: usize| {
        let (p, q) = (corners[from], corners[to]);
        (p.0 - q.0).hypot(p.1 - q.1)
    };
    let ratio = (edge(0, 1).max(edge(3, 2)) / width).max(edge(0, 3).max(edge(1, 2)) / height);
    ratio.is_finite().then_some(ratio)
}

/// The texture of a layer node with its passes and its version, if the
/// budget of the frame has room for it.
pub(super) fn node_texture(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &Node,
    scale: f64,
) -> Option<(GlesTexture, u64)> {
    let max = parts.assets.layers.max_side(gles);
    let scale = mesh::fit_scale(node.props.size?, scale, max);
    let size = mesh::texture_size(node, scale, max)?;
    let textures = 1 + mesh::passes(node, scale).len().min(2);
    if !parts.assets.layers.spend(&node.id, size, textures) {
        return None;
    }
    layer_texture(parts, gles, node, size, scale)
}

/// Renders a layer node with its descendants into its texture, and applies
/// its effects. The texture keeps its content, so that only what changed
/// since the last frame draws; the version counts the changes.
pub(super) fn layer_texture(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &chora_scene::Node,
    size: smithay::utils::Size<i32, smithay::utils::Buffer>,
    scale: f64,
) -> Option<(GlesTexture, u64)> {
    let inner = View {
        camera: Transform::IDENTITY,
        scale,
        size: (size.w, size.h).into(),
        near: mesh::NEAR,
        stepped: true,
    };
    let mut content = flat_elements(parts, gles, &node.id, &inner)?;
    settle_backdrops(parts.assets, &mut content, scale);
    let assets = &mut *parts.assets;
    let drawn = assets.layers.draw(
        (gles, &mut assets.stats),
        (&node.id, 0),
        (size, scale),
        &content,
    );
    let drawn = drawn
        .inspect_err(|err| tracing::warn!(%err, "a layer failed"))
        .ok()?;
    let pixels = node
        .props
        .size
        .map_or(scale, |[w, _]| f64::from(size.w) / w.max(f64::MIN_POSITIVE));
    let sources = node_textures(parts, gles, node, pixels);
    let assets = &mut *parts.assets;
    Some(mesh::apply_passes(
        gles,
        (
            &mut assets.layers,
            &mut assets.effect_errors,
            &mut assets.stats,
        ),
        node,
        drawn,
        (pixels, &sources),
    ))
}

/// The node textures of a shader node by the names of their uniforms: each
/// node that a uniform names, rendered on its own at its size, once for
/// all the nodes that read it. Inside a node texture, shader nodes get
/// none, so that shader nodes that read each other do not recurse.
pub(super) fn node_textures(
    parts: &mut Parts<'_>,
    gles: &mut GlesRenderer,
    node: &chora_scene::Node,
    scale: f64,
) -> Vec<mesh::Source> {
    let mut out = Vec::new();
    if node.kind != NodeKind::Shader || parts.depth > 0 {
        return out;
    }
    for (name, uniform) in &node.props.uniforms {
        let source = match uniform {
            chora_scene::Uniform::Texture(source) => source,
            chora_scene::Uniform::Output(read) => {
                let reader = node.owner.as_deref();
                let faces = parts.assets.virtuals.iter();
                let frame = faces
                    .filter(|v| v.output().name() == read.output)
                    .find_map(|v| v.frame(reader))
                    .or_else(|| Some((parts.assets.blank(gles)?, 0)));
                out.extend(frame.map(|(texture, version)| (name.clone(), texture, version)));
                continue;
            }
            _ => continue,
        };
        let Some(logical) = source_size(parts, &source.node) else {
            continue;
        };
        let max = parts.assets.layers.max_side(gles);
        let scale = mesh::fit_scale(logical, scale, max);
        let Some(size) = mesh::pixel_size(logical, scale, max) else {
            continue;
        };
        parts.assets.layers.reader = Some(node.id.clone());
        let fits = parts.assets.layers.spend(&source.node, size, 1);
        if !fits {
            parts.assets.layers.reader = None;
            continue;
        }
        let view = View {
            camera: Transform::IDENTITY,
            scale,
            size: (size.w, size.h).into(),
            near: mesh::NEAR,
            stepped: true,
        };
        parts.depth += 1;
        let content = flat_elements(parts, gles, &source.node, &view);
        parts.depth -= 1;
        parts.assets.layers.reader = None;
        let Some(content) = content else { continue };
        let assets = &mut *parts.assets;
        let drawn = assets.layers.draw(
            (gles, &mut assets.stats),
            (&source.node, mesh::SOURCE_SLOT),
            (size, scale),
            &content,
        );
        if let Ok((texture, version)) = drawn {
            out.push((name.clone(), texture, version));
        }
    }
    out
}

/// The logical size of a node as a node texture: its size, or the window
/// geometry of a surface.
pub(super) fn source_size(parts: &Parts<'_>, id: &str) -> Option<[f64; 2]> {
    let node = parts.scene.node(id)?;
    if node.kind == NodeKind::Surface {
        let size = parts.windows.get(id)?.geometry().size;
        return Some([f64::from(size.w), f64::from(size.h)]);
    }
    node.props.size
}
