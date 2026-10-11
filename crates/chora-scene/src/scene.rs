//! The scene tree and the rules that change it.

use std::collections::BTreeMap;

use crate::{
    Command, Effect, InputMode, Node, NodeKind, Props, PropsUpdate, ROOT, Selector, SurfaceInfo,
    ToplevelState, Transform,
};

/// Why a command failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SceneError {
    /// The node does not exist.
    #[error("node {0:?} does not exist")]
    Missing(String),
    /// A node with this id exists already.
    #[error("node {0:?} exists already")]
    Exists(String),
    /// The node has another owner.
    #[error("node {0:?} belongs to {1:?}")]
    NotOwner(String, String),
    /// The node has the wrong kind for this command.
    #[error("node {0:?} is not a {1}")]
    WrongKind(String, &'static str),
    /// The surface has an owner; the owner moves it with `move`.
    #[error("surface {0:?} has an owner; the owner moves it with move")]
    Attached(String),
    /// The command would put a node into its own subtree.
    #[error("node {0:?} cannot move into its own subtree")]
    Cycle(String),
    /// A value is out of range, for example a negative size.
    #[error("{0}")]
    Invalid(String),
    /// The root cannot change.
    #[error("the root cannot change")]
    Root,
}

/// A failed transaction: the index of the failed command and the reason.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("command {index}: {error}")]
pub struct TransactionError {
    /// The position of the failed command in the transaction.
    pub index: usize,
    /// The reason.
    pub error: SceneError,
}

/// A visible node with its absolute transform, in drawing order.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed<'a> {
    /// The node.
    pub node: &'a Node,
    /// From node coordinates to scene coordinates.
    pub transform: Transform,
    /// The product of the opacities of the node and its ancestors.
    pub opacity: f64,
    /// The input mode, after the modes of the ancestors.
    pub input: InputMode,
}

/// The scene tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    nodes: BTreeMap<String, Node>,
}

impl Default for Scene {
    fn default() -> Self {
        let root = Node {
            id: ROOT.into(),
            kind: NodeKind::Group,
            owner: None,
            parent: None,
            children: Vec::new(),
            props: Props::default(),
            surface: None,
        };
        Self {
            nodes: BTreeMap::from([(ROOT.into(), root)]),
        }
    }
}

impl Scene {
    /// A node by id.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// All nodes that match a selector, in id order.
    #[must_use]
    pub fn query(&self, selector: &Selector) -> Vec<&Node> {
        self.nodes
            .values()
            .filter(|n| selector.matches(n))
            .collect()
    }

    /// Adds a detached surface node without an owner.
    pub fn add_surface(&mut self, id: &str, info: SurfaceInfo) {
        let node = Node {
            id: id.into(),
            kind: NodeKind::Surface,
            owner: None,
            parent: None,
            children: Vec::new(),
            props: Props::default(),
            surface: Some(info),
        };
        self.nodes.insert(id.into(), node);
    }

    /// Replaces the client data of a surface node.
    pub fn set_surface_info(&mut self, id: &str, info: SurfaceInfo) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.surface = Some(info);
        }
    }

    /// Removes a surface node after its client destroyed the surface.
    pub fn remove_surface(&mut self, id: &str) {
        self.unlink(id);
        self.nodes.remove(id);
    }

    /// Applies a transaction for `owner`. Either all commands apply, or
    /// none.
    ///
    /// # Errors
    ///
    /// The first command that breaks a rule.
    pub fn apply(
        &mut self,
        owner: &str,
        commands: &[Command],
    ) -> Result<Vec<Effect>, TransactionError> {
        let mut next = self.clone();
        let effects = next.run(owner, commands)?;
        *self = next;
        Ok(effects)
    }

    /// The effects that [`Scene::apply`] would return, without a change of
    /// the scene.
    ///
    /// # Errors
    ///
    /// The first command that breaks a rule.
    pub fn check(
        &self,
        owner: &str,
        commands: &[Command],
    ) -> Result<Vec<Effect>, TransactionError> {
        self.clone().run(owner, commands)
    }

    /// Runs the commands in place; a failure leaves a partial change.
    fn run(&mut self, owner: &str, commands: &[Command]) -> Result<Vec<Effect>, TransactionError> {
        let mut effects = Vec::new();
        for (index, command) in commands.iter().enumerate() {
            self.apply_one(owner, command, &mut effects)
                .map_err(|error| TransactionError { index, error })?;
        }
        Ok(effects)
    }

    fn apply_one(
        &mut self,
        owner: &str,
        command: &Command,
        effects: &mut Vec<Effect>,
    ) -> Result<(), SceneError> {
        match command {
            Command::Create {
                id,
                kind,
                parent,
                props,
            } => {
                check_rect_props(id, *kind, props)?;
                self.create(owned_node(owner, id, *kind, props), parent)
            }
            Command::Attach { surface, parent } => self.attach(owner, surface, parent),
            Command::Update { id, props } => {
                let node = self.owned_mut(owner, id)?;
                check_rect_props(id, node.kind, props)?;
                node.props.apply(props);
                Ok(())
            }
            Command::Move { id, parent, index } => {
                self.move_node(owner, id, parent.as_deref(), *index)
            }
            Command::Remove { id } => {
                self.owned_mut(owner, id)?;
                effects.extend(self.remove_subtree(id).into_iter().map(Effect::Detached));
                Ok(())
            }
            Command::Configure {
                surface,
                size,
                states,
            } => {
                self.owned_surface(owner, surface)?;
                if size.is_some_and(|[w, h]| w < 0 || h < 0) {
                    return Err(SceneError::Invalid(format!(
                        "negative size for {surface:?}"
                    )));
                }
                effects.push(configure(surface, *size, states.clone()));
                Ok(())
            }
            Command::Focus { surface } => {
                if let Some(s) = surface {
                    self.owned_surface(owner, s)?;
                }
                effects.push(Effect::Focus(surface.clone()));
                Ok(())
            }
            Command::Close { surface } => {
                self.owned_surface(owner, surface)?;
                effects.push(Effect::Close(surface.clone()));
                Ok(())
            }
        }
    }

    fn create(&mut self, node: Node, parent: &str) -> Result<(), SceneError> {
        let id = node.id.clone();
        if self.nodes.contains_key(&id) {
            return Err(SceneError::Exists(id));
        }
        if node.kind == NodeKind::Surface {
            return Err(SceneError::WrongKind(id, "group or rect"));
        }
        self.group(parent)?;
        self.nodes.insert(id.clone(), node);
        self.link(&id, parent, None);
        Ok(())
    }

    fn attach(&mut self, owner: &str, surface: &str, parent: &str) -> Result<(), SceneError> {
        match self.surface(surface)?.owner.as_deref() {
            None => {}
            Some(o) if o == owner => return Err(SceneError::Attached(surface.into())),
            Some(other) => return Err(SceneError::NotOwner(surface.into(), other.into())),
        }
        self.group(parent)?;
        self.unlink(surface);
        self.link(surface, parent, None);
        self.node_mut(surface)?.owner = Some(owner.into());
        Ok(())
    }

    fn move_node(
        &mut self,
        owner: &str,
        id: &str,
        parent: Option<&str>,
        index: Option<usize>,
    ) -> Result<(), SceneError> {
        let node = self.owned_mut(owner, id)?;
        let parent = parent
            .map(str::to_owned)
            .or_else(|| node.parent.clone())
            .ok_or_else(|| SceneError::WrongKind(id.into(), "node with a parent"))?;
        self.group(&parent)?;
        if self.ancestors(&parent).any(|a| a == id) {
            return Err(SceneError::Cycle(id.into()));
        }
        self.unlink(id);
        self.link(id, &parent, index);
        Ok(())
    }

    /// Removes the nodes of a client name and detaches its surfaces. Returns
    /// the detached surfaces.
    pub fn disconnect(&mut self, owner: &str) -> Vec<String> {
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
        detached
    }

    /// The emergency reset: removes every node except the root and the
    /// surfaces, and puts all surfaces in a grid that fills `area`. Returns
    /// the configure effects for the new cell size.
    ///
    /// # Panics
    ///
    /// Never: the root always exists.
    #[allow(clippy::cast_possible_truncation)]
    pub fn reset(&mut self, area: [f64; 2]) -> Vec<Effect> {
        let surfaces: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Surface)
            .map(|n| n.id.clone())
            .collect();
        self.nodes
            .retain(|id, n| id == ROOT || n.kind == NodeKind::Surface);
        let root = self.nodes.get_mut(ROOT).expect("the root always exists");
        root.children.clone_from(&surfaces);
        root.props = Props::default();
        let cells = grid(surfaces.len(), area);
        surfaces
            .iter()
            .zip(cells)
            .map(|(id, [x, y, w, h])| {
                let node = self.nodes.get_mut(id).expect("collected above");
                node.owner = None;
                node.parent = Some(ROOT.into());
                node.props = Props::default();
                node.props.transform = Transform::translate(x, y, 0.0);
                configure(id, Some([w as i32, h as i32]), Vec::new())
            })
            .collect()
    }

    /// The visible nodes below the root, bottom first, with their absolute
    /// transforms. The walk uses a stack, so deep trees cannot overflow the
    /// call stack.
    #[must_use]
    pub fn draw_list(&self) -> Vec<Placed<'_>> {
        let mut out = Vec::new();
        let mut stack = vec![(
            &self.nodes[ROOT],
            Transform::IDENTITY,
            1.0,
            InputMode::Accept,
        )];
        while let Some((node, parent, opacity, input)) = stack.pop() {
            if !node.props.visible {
                continue;
            }
            let placed = Placed {
                node,
                transform: parent.then(&node.props.transform),
                opacity: opacity * node.props.opacity,
                input: if input == InputMode::Accept {
                    node.props.input
                } else {
                    input
                },
            };
            for child in node.children.iter().rev() {
                stack.push((
                    &self.nodes[child],
                    placed.transform,
                    placed.opacity,
                    placed.input,
                ));
            }
            if node.id != ROOT {
                out.push(placed);
            }
        }
        out
    }

    /// Removes a node and its descendants. Surfaces become detached and
    /// lose their owner. Returns the detached surfaces.
    fn remove_subtree(&mut self, id: &str) -> Vec<String> {
        self.unlink(id);
        let mut detached = Vec::new();
        let mut stack = vec![id.to_owned()];
        while let Some(current) = stack.pop() {
            let Some(node) = self.nodes.get_mut(&current) else {
                continue;
            };
            stack.append(&mut node.children);
            if node.kind == NodeKind::Surface {
                node.parent = None;
                node.owner = None;
                node.props = Props::default();
                detached.push(current);
            } else {
                self.nodes.remove(&current);
            }
        }
        detached
    }

    fn link(&mut self, id: &str, parent: &str, index: Option<usize>) {
        let siblings = &mut self
            .nodes
            .get_mut(parent)
            .expect("checked by the caller")
            .children;
        let at = index.map_or(siblings.len(), |i| i.min(siblings.len()));
        siblings.insert(at, id.into());
        self.nodes
            .get_mut(id)
            .expect("checked by the caller")
            .parent = Some(parent.into());
    }

    fn unlink(&mut self, id: &str) {
        let Some(parent) = self.nodes.get_mut(id).and_then(|n| n.parent.take()) else {
            return;
        };
        if let Some(p) = self.nodes.get_mut(&parent) {
            p.children.retain(|c| c != id);
        }
    }

    fn ancestors<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a str> {
        std::iter::successors(Some(id), |current| {
            self.nodes.get(*current)?.parent.as_deref()
        })
    }

    fn node_mut(&mut self, id: &str) -> Result<&mut Node, SceneError> {
        self.nodes
            .get_mut(id)
            .ok_or_else(|| SceneError::Missing(id.into()))
    }

    fn owned_mut(&mut self, owner: &str, id: &str) -> Result<&mut Node, SceneError> {
        if id == ROOT {
            return Err(SceneError::Root);
        }
        let node = self.node_mut(id)?;
        match node.owner.as_deref() {
            Some(o) if o == owner => Ok(node),
            other => Err(SceneError::NotOwner(
                id.into(),
                other.unwrap_or("nobody").into(),
            )),
        }
    }

    fn owned_surface(&mut self, owner: &str, id: &str) -> Result<(), SceneError> {
        self.surface(id)?;
        self.owned_mut(owner, id).map(|_| ())
    }

    fn surface(&self, id: &str) -> Result<&Node, SceneError> {
        let node = self
            .nodes
            .get(id)
            .ok_or_else(|| SceneError::Missing(id.into()))?;
        if node.kind == NodeKind::Surface {
            Ok(node)
        } else {
            Err(SceneError::WrongKind(id.into(), "surface"))
        }
    }

    fn group(&self, id: &str) -> Result<(), SceneError> {
        match self.nodes.get(id) {
            None => Err(SceneError::Missing(id.into())),
            Some(n) if n.kind == NodeKind::Group => Ok(()),
            Some(_) => Err(SceneError::WrongKind(id.into(), "group")),
        }
    }
}

/// Only rects have a size and a color; sizes are not negative, and all
/// numbers are finite.
fn check_rect_props(id: &str, kind: NodeKind, props: &PropsUpdate) -> Result<(), SceneError> {
    if kind != NodeKind::Rect && (props.size.is_some() || props.color.is_some()) {
        return Err(SceneError::WrongKind(id.into(), "rect"));
    }
    let numbers = props
        .transform
        .iter()
        .flat_map(|t| t.matrix().iter())
        .chain(props.size.iter().flatten())
        .chain(props.color.iter().flatten())
        .chain(props.opacity.iter());
    if numbers.copied().any(|n| !n.is_finite()) {
        return Err(SceneError::Invalid(format!(
            "a number of {id:?} is not finite"
        )));
    }
    if props.size.is_some_and(|[w, h]| w < 0.0 || h < 0.0) {
        return Err(SceneError::Invalid(format!("negative size for {id:?}")));
    }
    Ok(())
}

fn owned_node(owner: &str, id: &str, kind: NodeKind, update: &PropsUpdate) -> Node {
    let mut props = Props::default();
    props.apply(update);
    Node {
        id: id.into(),
        kind,
        owner: Some(owner.into()),
        parent: None,
        children: Vec::new(),
        props,
        surface: None,
    }
}

fn configure(surface: &str, size: Option<[i32; 2]>, states: Vec<ToplevelState>) -> Effect {
    Effect::Configure {
        surface: surface.into(),
        size,
        states,
    }
}

/// Cells of a grid for `n` items that fills `area`: x, y, width, height.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn grid(n: usize, [w, h]: [f64; 2]) -> Vec<[f64; 4]> {
    if n == 0 {
        return Vec::new();
    }
    let cols = (n as f64).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    let (cw, ch) = (w / cols as f64, h / rows as f64);
    (0..n)
        .map(|i| [(i % cols) as f64 * cw, (i / cols) as f64 * ch, cw, ch])
        .collect()
}
