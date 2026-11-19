//! The `query` call: the nodes of the scene and the screen boxes of their contents.

use super::{
    HashMap, HashSet, Logical, Node, NodeKind, Placed, Query, Rectangle, RpcError, Selector, State,
    Transform, Value, bounding_box_from, clients, intersect, is_texture, layer_box, output_area,
    window_rect,
};

impl State {
    /// The nodes of a selector and a JSON filter (D32); for a client
    /// socket only its own.
    /// (D33): only the nodes that survived the filter can show a focus.
    pub(super) fn query(&self, conn: u64, query: &Query) -> Result<Value, RpcError> {
        let selector: Selector = query
            .selector
            .parse()
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, format!("{e}")))?;
        let fixed = self.ipc.fixed(conn);
        let mut nodes = self.scene.query(&selector);
        nodes.retain(|n| fixed.is_none_or(|f| clients::own_node(n, f)));
        let focus = [self.scene.focused_surface(), self.scene.focused_node()];
        if nodes.is_empty() {
            return Ok(Value::Array(Vec::new()));
        }
        let matched: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
        let rects = self.screen_rects(&matched);
        if let Some(filter) = &query.filter {
            nodes.retain(|n| {
                filter.matches(
                    n,
                    focus.contains(&Some(n.id.as_str())),
                    rects.get(n.id.as_str()).unwrap_or(&Value::Null),
                )
            });
        }
        let nodes: Vec<Value> = nodes
            .into_iter()
            .map(|n| {
                let mut node = serde_json::json!(n);
                if focus.contains(&Some(n.id.as_str())) {
                    node["focused"] = Value::Bool(true);
                }
                node["rects"] = rects
                    .get(n.id.as_str())
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                node
            })
            .collect();
        Ok(Value::Array(nodes))
    }

    /// The screen rectangles by node: the output name maps to
    /// `[x, y, w, h]` in the logical pixels of that output, after the
    /// parent transforms and the camera of the output. Outputs where the
    /// node does not show are left out. Only the nodes `of` get a
    /// rectangle: a box costs a text layout or a projection, which a
    /// query for one node must not pay for the whole scene.
    /// The overlay sits in the coordinates of its output, without
    /// its camera (D43).
    /// The absolute transform by node, for the boxes of the
    /// layers in this list.
    pub(super) fn screen_rects(&self, of: &HashSet<&str>) -> HashMap<&str, Value> {
        let mut found: HashMap<&str, serde_json::Map<String, Value>> = HashMap::new();
        for output in &self.all_outputs() {
            let name = output.name();
            let screen = Rectangle::from_size(output_area(output).size).to_f64();
            let views = [
                (Transform::IDENTITY, self.scene.overlay_draw_list(&name)),
                (self.camera(output), self.scene.draw_list()),
            ];
            for (camera, list) in &views {
                let at: HashMap<&str, Transform> = list
                    .iter()
                    .map(|placed| (placed.node.id.as_str(), camera.then(&placed.transform)))
                    .collect();
                let wanted = list.iter().filter(|p| of.contains(p.node.id.as_str()));
                let shown = wanted.filter_map(|placed| {
                    Some((
                        placed.node.id.as_str(),
                        self.screen_rect(placed, &at, screen)?,
                    ))
                });
                for (id, rect) in shown {
                    let rect = [rect.loc.x, rect.loc.y, rect.size.w, rect.size.h];
                    found
                        .entry(id)
                        .or_default()
                        .insert(name.clone(), serde_json::json!(rect));
                }
            }
        }
        found
            .into_iter()
            .map(|(id, rects)| (id, Value::Object(rects)))
            .collect()
    }

    /// The rectangle of one placed node on an output, as the renderer
    /// draws it there. A layer, and with a size also a shader, shows
    /// through its mesh, so its box is the box of the projected shape.
    /// Any other node draws as a plane when its transform is not a plain
    /// translation with a positive scale, so its box is the box of its
    /// projected corners; every ancestor layer or plane cuts the box like a
    /// clip does. A quad that is edge on or collapsed shows nowhere, and in
    /// a 3D context a quad shows only in front of its near plane. The mesh
    /// of a layer does not move the boxes of descendants.
    /// Every ancestor layer cuts the box like a clip does.
    pub(super) fn screen_rect(
        &self,
        placed: &Placed<'_>,
        at: &HashMap<&str, Transform>,
        screen: Rectangle<f64, Logical>,
    ) -> Option<Rectangle<f64, Logical>> {
        let node = placed.node;
        let transform = at.get(node.id.as_str())?;
        let (layers, near) = self.lineage(node, at)?;
        if node.props.mesh.is_none() && !transform.is_plain() {
            transform.plane_inverse()?;
        }
        let mut rect = if is_texture(node, transform) {
            layer_box(node, transform, near)?
        } else {
            self.content_box(node, transform, near)?
        };
        for ancestor in layers {
            let ancestor_at = at.get(ancestor.id.as_str())?;
            rect = intersect(rect, layer_box(ancestor, ancestor_at, near)?)?;
        }
        rect.overlaps(screen).then_some(rect)
    }

    /// Every ancestor layer or plane, and the near w of the node: 1/64 in
    /// a 3D context. The node shows only inside every layer and plane: each
    /// composes into a texture of its size. A clip makes a layer and cuts
    /// through the same box.
    pub(super) fn lineage(
        &self,
        node: &Node,
        at: &HashMap<&str, Transform>,
    ) -> Option<(Vec<&Node>, f64)> {
        let mut layers = Vec::new();
        let mut near = crate::mesh::NEAR;
        let mut parent = node.parent.as_deref();
        while let Some(id) = parent {
            let ancestor = self.scene.node(id)?;
            if at.get(id).is_some_and(|t| is_texture(ancestor, t)) {
                layers.push(ancestor);
            }
            if ancestor.is_context() {
                near = crate::space::NEAR_W;
            }
            parent = ancestor.parent.as_deref();
        }
        Some((layers, near))
    }

    /// The box of a node that is not a layer or plane itself. A surface
    /// has the box of its buffer, subsurfaces and decorations, and under a
    /// plane the box of its geometry, which its texture holds.
    /// Without the file the renderer draws nothing.
    /// A text node reports its layout size, like an image
    /// without a size.
    /// A group without a layer of its own draws nothing, and
    /// so does a shader without a size.
    pub(super) fn content_box(
        &self,
        node: &Node,
        transform: &Transform,
        near: f64,
    ) -> Option<Rectangle<f64, Logical>> {
        let bounding_box = |transform, size| bounding_box_from(transform, size, near);
        match node.kind {
            NodeKind::Surface if !transform.is_plain() => {
                let size = self.windows.get(node.id.as_str())?.geometry().size;
                Some(bounding_box(
                    transform,
                    [f64::from(size.w), f64::from(size.h)],
                ))
            }
            NodeKind::Surface => Some(window_rect(self.windows.get(node.id.as_str())?, transform)),
            NodeKind::Rect => Some(bounding_box(transform, node.props.size?)),
            NodeKind::Image => {
                let image = self.assets.images.get(node.props.image.as_deref()?)?;
                let [w, h] = node
                    .props
                    .size
                    .unwrap_or([f64::from(image.size[0]), f64::from(image.size[1])]);
                Some(bounding_box(transform, [w, h]))
            }
            NodeKind::Text => {
                let size = self.assets.texts.size(node)?;
                Some(bounding_box(transform, size))
            }
            NodeKind::Group | NodeKind::Shader => None,
        }
    }
}
