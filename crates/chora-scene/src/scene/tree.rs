//! Creating, attaching and moving nodes, and clearing the scene of an owner.

use super::{
    Effect, INJECTION, MAX_NODES, Node, NodeKind, OVERLAY, OrdMap, Props, ROOT, Scene, SceneError,
    Transform, configure, grid, is_overlay, is_surface_id,
};

impl Scene {
    /// The root and the surfaces do not count.
    pub(super) fn create(&mut self, node: Node, parent: &str) -> Result<(), SceneError> {
        let id = node.id.clone();
        if is_surface_id(&id) {
            return Err(SceneError::Invalid(format!(
                "{id:?}: ids of the form s1, s2 belong to surfaces"
            )));
        }
        if is_overlay(&id) {
            return Err(SceneError::Invalid(format!(
                "{id:?}: ids that start with {OVERLAY:?} belong to the outputs"
            )));
        }
        if id == INJECTION {
            return Err(SceneError::Invalid(format!(
                "{id:?}: this id belongs to the injection mark of the core"
            )));
        }
        if self.nodes.contains_key(&id) {
            return Err(SceneError::Exists(id));
        }
        if self.nodes.len() - self.surfaces > MAX_NODES {
            return Err(SceneError::Limit(format!(
                "{id:?}: the scene holds {MAX_NODES} nodes, its limit"
            )));
        }
        if node.kind == NodeKind::Surface {
            return Err(SceneError::WrongKind(id, "group, rect or image"));
        }
        self.group(parent)?;
        self.check_depth(&id, parent, 1)?;
        self.nodes.insert(id.clone(), node);
        self.link(&id, parent, None);
        Ok(())
    }

    pub(super) fn attach(
        &mut self,
        owner: &str,
        surface: &str,
        parent: &str,
    ) -> Result<(), SceneError> {
        let node = self.surface(surface)?;
        if let Some(to) = node.to.as_ref().filter(|to| *to != owner) {
            return Err(SceneError::Invalid(format!(
                "surface {surface:?} is reserved for {to:?}"
            )));
        }
        match node.owner.as_deref() {
            None => {}
            Some(o) if o == owner => return Err(SceneError::Attached(surface.into())),
            Some(other) => return Err(SceneError::NotOwner(surface.into(), other.into())),
        }
        self.group(parent)?;
        self.check_depth(surface, parent, 1)?;
        self.unlink(surface);
        self.link(surface, parent, None);
        let node = self.node_mut(surface)?;
        node.owner = Some(owner.into());
        node.to = None;
        Ok(())
    }

    pub(super) fn move_node(
        &mut self,
        owner: &str,
        id: &str,
        parent: Option<&str>,
        index: Option<usize>,
    ) -> Result<(), SceneError> {
        let node = self.owned_mut(owner, id)?;
        #[allow(
            clippy::expect_used,
            reason = "only the root has no parent, and no client owns it"
        )]
        let parent = parent
            .map(str::to_owned)
            .or_else(|| node.parent.clone())
            .expect("every owned node has a parent");
        self.group(&parent)?;
        if self.ancestors(&parent).any(|a| a == id) {
            return Err(SceneError::Cycle(id.into()));
        }
        self.check_depth(id, &parent, self.height(id))?;
        self.unlink(id);
        self.link(id, &parent, index);
        Ok(())
    }

    /// Removes the nodes and the cameras of a client name and detaches its
    /// surfaces. Returns a `Detached` effect for each detached surface and a
    /// `Camera` effect for each output that gets its default camera back.
    pub fn disconnect(&mut self, owner: &str) -> Vec<Effect> {
        let ids: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.owner.as_deref() == Some(owner))
            .map(|n| n.id.clone())
            .collect();
        let mut detached = Vec::new();
        for id in ids {
            let still_owned = self
                .nodes
                .get(&id)
                .is_some_and(|n| n.owner.as_deref() == Some(owner));
            if still_owned {
                detached.extend(self.remove_subtree(&id));
            }
        }
        let cameras: Vec<String> = self
            .cameras
            .iter()
            .filter(|(_, (o, _))| o == owner)
            .map(|(output, _)| output.clone())
            .collect();
        for output in &cameras {
            self.cameras.remove(output);
            self.camera_moves.remove(output);
        }
        self.focus_returns.remove(owner);
        let detached = detached.into_iter().map(Effect::Detached);
        detached
            .chain(self.unreserve(owner))
            .chain(cameras.into_iter().map(Effect::Camera))
            .collect()
    }

    /// Ends the reservations for a client name: the surfaces are free to
    /// take again. Returns a `Detached` effect for each.
    pub fn unreserve(&mut self, name: &str) -> Vec<Effect> {
        let reserved: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.to.as_deref() == Some(name))
            .map(|n| n.id.clone())
            .collect();
        for id in &reserved {
            self.reserve(id, None);
        }
        reserved.into_iter().map(Effect::Detached).collect()
    }

    /// The emergency reset: removes every node except the root and the
    /// surfaces, removes the cameras, and puts all surfaces in a grid that
    /// fills `area`, given as x, y, width and height. Returns the configure
    /// effects for the new cell size.
    ///
    /// # Panics
    ///
    /// Never: the root always exists.
    #[allow(clippy::cast_possible_truncation)]
    pub fn reset(&mut self, area: [f64; 4]) -> Vec<Effect> {
        let surfaces: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Surface)
            .map(|n| n.id.clone())
            .collect();
        self.nodes = std::mem::take(&mut self.nodes)
            .into_iter()
            .filter(|(id, n)| id == ROOT || is_overlay(id) || n.kind == NodeKind::Surface)
            .collect();
        let overlays: Vec<String> = self
            .nodes
            .keys()
            .filter(|id| is_overlay(id))
            .cloned()
            .collect();
        for id in overlays {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.children.clear();
            }
        }
        #[allow(clippy::expect_used, reason = "the root is never removed")]
        let root = self.nodes.get_mut(ROOT).expect("the root always exists");
        root.children.clone_from(&surfaces);
        root.props = Props::default();
        self.cameras = OrdMap::new();
        self.animations = OrdMap::new();
        self.camera_moves = OrdMap::new();
        self.focus_returns = OrdMap::new();
        self.focused_node = None;
        let cells = grid(surfaces.len(), area);
        surfaces
            .iter()
            .zip(cells)
            .map(|(id, [x, y, w, h])| {
                #[allow(clippy::expect_used, reason = "collected above")]
                let node = self.nodes.get_mut(id).expect("collected above");
                node.owner = None;
                node.to = None;
                node.parent = Some(ROOT.into());
                node.props = Props::default();
                node.props.transform = Transform::translate(x, y, 0.0);
                configure(id, Some([w as i32, h as i32]), Vec::new())
            })
            .collect()
    }
}
