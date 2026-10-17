//! The scene tree and the rules that change it.

use std::collections::BTreeMap;

use imbl::OrdMap;

use crate::{
    Command, Easing, Effect, InputMode, MAX_DURATION, MAX_EFFECTS, MAX_MESH_CELLS,
    MAX_NODE_TEXTURES, MAX_UNIFORMS, Node, NodeKind, Props, PropsUpdate, ROOT, Selector,
    ShaderEffect, SurfaceInfo, ToplevelState, Transform, Uniform,
    animation::{Animation, CameraMove},
    node::apply_meta,
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
    /// The camera of the output has another owner.
    #[error("the camera of {0:?} belongs to {1:?}")]
    CameraOwner(String, String),
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

impl TransactionError {
    /// Whether the transaction failed on a surface node that is gone.
    #[must_use]
    pub fn is_gone_surface(&self) -> bool {
        matches!(&self.error, SceneError::Missing(id) if is_surface_id(id))
    }
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
    /// The clips of the node and its ancestors: the transform of each node
    /// with a clip and its size, for hit tests.
    pub clips: Vec<(Transform, [f64; 2])>,
}

/// A node of a walk with the transform, the opacity, the input mode and the
/// clips of its parent.
type Step<'a> = (
    &'a Node,
    Transform,
    f64,
    InputMode,
    Vec<(Transform, [f64; 2])>,
);

/// The limit of the cameras that clients set.
pub const MAX_CAMERAS: usize = 64;
/// The largest translation of a camera, in logical pixels.
const MAX_PAN: f64 = 1e7;
/// The largest radius and border width, in logical pixels.
const MAX_LENGTH: f64 = 10_000.0;
/// The largest zoom of a camera, and the inverse of the smallest.
const MAX_ZOOM: f64 = 64.0;

/// A transform without its z axis: a camera works in 2D, so the z of a
/// uniform scale such as `{"scale": 1.5}`, or of a translation, does not
/// matter.
fn flat(t: Transform) -> Transform {
    let mut m = *t.matrix();
    m[10] = 1.0;
    m[14] = 0.0;
    Transform::from_matrix(m)
}

/// Whether a transform is a camera: exactly a translation in x and y after
/// a scale in x and y, within the limits.
#[allow(clippy::float_cmp)] // the other entries of such a matrix are exactly 0 or 1
fn valid_camera(t: &Transform) -> bool {
    let m = t.matrix();
    let zero = [1, 2, 3, 4, 6, 7, 8, 9, 11, 14];
    let plain = zero.iter().all(|&i| m[i] == 0.0) && m[10] == 1.0 && m[15] == 1.0;
    let zoom = |s: f64| (1.0 / MAX_ZOOM..=MAX_ZOOM).contains(&s);
    let pan = |p: f64| (-MAX_PAN..=MAX_PAN).contains(&p);
    plain && zoom(m[0]) && zoom(m[5]) && pan(m[12]) && pan(m[13])
}

/// The limit of the groups and rects that clients create. Surfaces do not
/// count, so that Wayland clients cannot use up the limit. The limit
/// keeps a buggy client from growing the scene, and with it each frame,
/// without bound.
pub const MAX_NODES: usize = 16_384;

/// The scene tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    nodes: OrdMap<String, Node>,
    /// The cameras that clients set: the owner and the transform by output.
    cameras: OrdMap<String, (String, Transform)>,
    /// The number of surface nodes, which the node limit leaves out.
    surfaces: usize,
    /// The running animations by node.
    animations: OrdMap<String, Animation>,
    /// The cameras that move to their target, by output.
    camera_moves: OrdMap<String, CameraMove>,
    /// The default camera of each output, which the core sets; the start
    /// and the end of moves from and to the default camera.
    default_cameras: OrdMap<String, Transform>,
    /// The surface with the keyboard focus, which the core sets.
    focused: Option<String>,
    /// The surface that had the focus when a client cleared it, by client.
    focus_returns: OrdMap<String, String>,
    /// The node of a client with the keyboard focus, above the focus of the
    /// surface.
    focused_node: Option<String>,
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
            to: None,
        };
        Self {
            nodes: OrdMap::unit(ROOT.into(), root),
            cameras: OrdMap::new(),
            surfaces: 0,
            animations: OrdMap::new(),
            camera_moves: OrdMap::new(),
            default_cameras: OrdMap::new(),
            focused: None,
            focus_returns: OrdMap::new(),
            focused_node: None,
        }
    }
}

impl Scene {
    /// The camera that a client set for an output; `None` for the default
    /// camera.
    #[must_use]
    pub fn camera(&self, output: &str) -> Option<&Transform> {
        self.cameras.get(output).map(|(_, t)| t)
    }

    /// The camera of an output now, on the way of a move; `None` for the
    /// default camera.
    #[must_use]
    pub fn current_camera(&self, output: &str) -> Option<Transform> {
        let target = self.camera(output).copied();
        let Some(moving) = self.camera_moves.get(output) else {
            return target;
        };
        let to = target.or_else(|| self.default_cameras.get(output).copied())?;
        Some(moving.at(&to))
    }

    /// Sets the default camera of an output, where moves to and from the
    /// default camera end and start.
    pub fn set_default_camera(&mut self, output: &str, camera: Transform) {
        self.default_cameras.insert(output.into(), camera);
    }

    /// A node by id.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// All nodes, in id order.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
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
        debug_assert!(!self.nodes.contains_key(id), "{id} exists");
        let node = Node {
            id: id.into(),
            kind: NodeKind::Surface,
            owner: None,
            parent: None,
            children: Vec::new(),
            props: Props::default(),
            surface: Some(info),
            to: None,
        };
        self.nodes.insert(id.into(), node);
        self.surfaces += 1;
    }

    /// Replaces the client data of a surface node.
    pub fn set_surface_info(&mut self, id: &str, info: SurfaceInfo) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.surface = Some(info);
        }
    }

    /// Removes a surface node after its client destroyed the surface.
    /// Returns whether the node existed.
    pub fn remove_surface(&mut self, id: &str) -> bool {
        self.unlink(id);
        let existed = self.nodes.remove(id).is_some();
        self.surfaces -= usize::from(existed);
        existed
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

    /// Reserves a detached surface for a client name: only that client can
    /// attach it.
    pub fn reserve(&mut self, surface: &str, to: Option<&str>) {
        if let Some(node) = self.nodes.get_mut(surface) {
            node.to = to.map(String::from);
        }
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
                check_props(id, *kind, props)?;
                check_meta(id, &BTreeMap::new(), props)?;
                if *kind == NodeKind::Shader && props.shader.is_none() {
                    return Err(SceneError::Invalid(format!(
                        "the shader node {id:?} needs a shader"
                    )));
                }
                self.create(owned_node(owner, id, *kind, props), parent)
            }
            Command::Attach { surface, parent } => self.attach(owner, surface, parent),
            Command::Update { .. } | Command::Animate { .. } => self.change_props(owner, command),
            Command::Move { id, parent, index } => {
                self.move_node(owner, id, parent.as_deref(), *index)
            }
            Command::Remove { id } => {
                self.owned_mut(owner, id)?;
                effects.extend(self.remove_subtree(id).into_iter().map(Effect::Detached));
                Ok(())
            }
            Command::Give { surface, to } => {
                self.owned_surface(owner, surface)?;
                self.remove_subtree(surface);
                self.reserve(surface, Some(to));
                effects.push(Effect::Given(surface.clone()));
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
                self.focus_command(owner, surface.as_deref())?;
                effects.push(Effect::Focus(surface.clone()));
                Ok(())
            }
            Command::FocusNode { node } => self.focus_node_command(owner, node.as_deref()),
            Command::Close { surface } => {
                self.owned_surface(owner, surface)?;
                effects.push(Effect::Close(surface.clone()));
                Ok(())
            }
            camera @ Command::Camera { .. } => self.camera_command(owner, camera, effects),
        }
    }

    fn camera_command(
        &mut self,
        owner: &str,
        command: &Command,
        effects: &mut Vec<Effect>,
    ) -> Result<(), SceneError> {
        let Command::Camera {
            output,
            transform,
            duration,
            easing,
        } = command
        else {
            unreachable!("only camera commands come here");
        };
        let timing = duration.map(|d| (d, *easing));
        self.set_camera(owner, output, *transform, timing)?;
        effects.push(Effect::Camera(output.clone()));
        Ok(())
    }

    fn set_camera(
        &mut self,
        owner: &str,
        output: &str,
        transform: Option<Transform>,
        timing: Option<(f64, Easing)>,
    ) -> Result<(), SceneError> {
        if let Some((other, _)) = self.cameras.get(output).filter(|(o, _)| o != owner) {
            return Err(SceneError::CameraOwner(output.into(), other.clone()));
        }
        // Only outputs that the core knows, so that no client claims an
        // output before it plugs in or uses up the cameras with names.
        if transform.is_some() && !self.default_cameras.contains_key(output) {
            return Err(SceneError::Invalid(format!(
                "there is no output {output:?}"
            )));
        }
        if timing.is_some_and(|(d, _)| !(0.0..=MAX_DURATION).contains(&d)) {
            return Err(SceneError::Invalid(format!(
                "the duration of the camera of {output:?} is not from 0 to {MAX_DURATION} ms"
            )));
        }
        // A move starts where the camera is now, also in the middle of
        // another move.
        let from = self
            .current_camera(output)
            .or_else(|| self.default_cameras.get(output).copied());
        self.camera_moves.remove(output);
        if let (Some(from), Some((duration, easing))) = (from, timing)
            && duration > 0.0
        {
            let moving = CameraMove {
                from,
                duration,
                elapsed: 0.0,
                easing,
            };
            self.camera_moves.insert(output.into(), moving);
        }
        let Some(t) = transform else {
            self.cameras.remove(output);
            return Ok(());
        };
        let t = flat(t);
        if !valid_camera(&t) {
            return Err(SceneError::Invalid(format!(
                "the camera of {output:?} takes only a translation in x and y within \
                 {MAX_PAN} and a scale in x and y from 1/{MAX_ZOOM} to {MAX_ZOOM}, \
                 without rotation"
            )));
        }
        if !self.cameras.contains_key(output) && self.cameras.len() >= MAX_CAMERAS {
            return Err(SceneError::Invalid(format!(
                "{output:?}: the scene holds {MAX_CAMERAS} cameras, its limit"
            )));
        }
        self.cameras.insert(output.into(), (owner.into(), t));
        Ok(())
    }

    fn create(&mut self, node: Node, parent: &str) -> Result<(), SceneError> {
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
        if self.nodes.contains_key(&id) {
            return Err(SceneError::Exists(id));
        }
        // The root and the surfaces do not count.
        if self.nodes.len() - self.surfaces > MAX_NODES {
            return Err(SceneError::Invalid(format!(
                "{id:?}: the scene holds {MAX_NODES} nodes, its limit"
            )));
        }
        if node.kind == NodeKind::Surface {
            return Err(SceneError::WrongKind(id, "group, rect or image"));
        }
        self.group(parent)?;
        self.nodes.insert(id.clone(), node);
        self.link(&id, parent, None);
        Ok(())
    }

    fn attach(&mut self, owner: &str, surface: &str, parent: &str) -> Result<(), SceneError> {
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
        self.unlink(surface);
        self.link(surface, parent, None);
        let node = self.node_mut(surface)?;
        node.owner = Some(owner.into());
        node.to = None;
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
            .expect("every owned node has a parent");
        self.group(&parent)?;
        if self.ancestors(&parent).any(|a| a == id) {
            return Err(SceneError::Cycle(id.into()));
        }
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
        // The surfaces reserved for the client are free to take again.
        let reserved: Vec<String> = self
            .nodes
            .values()
            .filter(|n| n.to.as_deref() == Some(owner))
            .map(|n| n.id.clone())
            .collect();
        for id in reserved {
            self.reserve(&id, None);
            detached.push(id);
        }
        self.focus_returns.remove(owner);
        let detached = detached.into_iter().map(Effect::Detached);
        detached
            .chain(cameras.into_iter().map(Effect::Camera))
            .collect()
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

    /// The visible nodes below the root, bottom first, with their absolute
    /// transforms. The walk uses a stack, so deep trees cannot overflow the
    /// call stack.
    #[must_use]
    pub fn draw_list(&self) -> Vec<Placed<'_>> {
        self.walk(ROOT, false)
    }

    /// The visible nodes under the overlay root of an output, bottom first,
    /// in the logical coordinates of the output (D43).
    #[must_use]
    pub fn overlay_draw_list(&self, output: &str) -> Vec<Placed<'_>> {
        self.walk(&overlay_id(output), false)
    }

    /// The nodes that the overlay root of an output shows as a layer list.
    #[must_use]
    pub fn overlay_layer_list(&self, output: &str) -> Vec<Placed<'_>> {
        self.walk(&overlay_id(output), true)
    }

    /// Adds the overlay root of an output: a group without owner, outside
    /// of the root, for nodes fixed to the screen (D43).
    pub fn add_overlay(&mut self, output: &str) {
        let id = overlay_id(output);
        if self.nodes.contains_key(&id) {
            return;
        }
        let node = Node {
            id: id.clone(),
            kind: NodeKind::Group,
            owner: None,
            parent: None,
            children: Vec::new(),
            props: Props::default(),
            surface: None,
            to: None,
        };
        self.nodes.insert(id, node);
    }

    /// The visible nodes that a layer shows, bottom first, in the
    /// coordinates of the layer: for the root, the nodes outside of
    /// layers; for a layer node, the node itself at the identity and its
    /// descendants. A nested layer is in the list, its descendants are
    /// not.
    #[must_use]
    pub fn layer_list(&self, id: &str) -> Vec<Placed<'_>> {
        self.walk(id, true)
    }

    fn walk(&self, start: &str, layers: bool) -> Vec<Placed<'_>> {
        let Some(first) = self.nodes.get(start) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        let mut stack = Vec::new();
        if start == ROOT || is_overlay(start) || !layers {
            stack.push((
                first,
                Transform::IDENTITY,
                1.0,
                InputMode::Accept,
                Vec::new(),
            ));
        } else if first.props.visible {
            // The start of a layer list shows without its own transform,
            // opacity and clip; the layer applies them.
            let placed = Placed {
                node: first,
                transform: Transform::IDENTITY,
                opacity: 1.0,
                input: first.props.input,
                clips: Vec::new(),
            };
            self.push_children(&mut stack, &placed);
            out.push(placed);
        }
        while let Some((node, parent, opacity, input, mut clips)) = stack.pop() {
            if !node.props.visible {
                continue;
            }
            let transform = parent.then(&node.props.transform);
            if let (Some(_), Some(size)) = (node.props.clip, node.props.size) {
                clips.push((transform, size));
            }
            let placed = Placed {
                node,
                transform,
                opacity: opacity * node.props.opacity,
                input: if input == InputMode::Accept {
                    node.props.input
                } else {
                    input
                },
                clips,
            };
            if !(layers && node.is_layer()) {
                self.push_children(&mut stack, &placed);
            }
            if node.id != ROOT {
                out.push(placed);
            }
        }
        out
    }

    /// Puts the children of a placed node on the stack of a walk, the
    /// bottom one last.
    fn push_children<'a>(&'a self, stack: &mut Vec<Step<'a>>, placed: &Placed<'a>) {
        for id in placed.node.children.iter().rev() {
            // A missing child is a bug of the tree links; a frame without
            // it is better than a crash of the session.
            debug_assert!(self.nodes.contains_key(id), "missing child {id:?}");
            if let Some(child) = self.nodes.get(id) {
                stack.push((
                    child,
                    placed.transform,
                    placed.opacity,
                    placed.input,
                    placed.clips.clone(),
                ));
            }
        }
    }

    /// Runs `update` and `animate`. Either ends a running animation of the
    /// node where it is.
    fn change_props(&mut self, owner: &str, command: &Command) -> Result<(), SceneError> {
        let (id, props, timing) = match command {
            Command::Update { id, props } => (id, props, None),
            Command::Animate {
                id,
                props,
                duration,
                easing,
            } => (id, props, Some((*duration, *easing))),
            _ => unreachable!("only update and animate"),
        };
        if timing.is_some_and(|(d, _)| !(0.0..=MAX_DURATION).contains(&d)) {
            return Err(SceneError::Invalid(format!(
                "the duration of {id:?} is not from 0 to {MAX_DURATION} ms"
            )));
        }
        let node = self.owned_mut(owner, id)?;
        check_props(id, node.kind, props)?;
        check_meta(id, &node.props.meta, props)?;
        self.animations.remove(id);
        let node = self.nodes.get_mut(id).expect("owned above");
        let Some((duration, easing)) = timing else {
            node.props.apply(props);
            return Ok(());
        };
        let animation = Animation {
            from: node.props.clone(),
            to: props.clone(),
            duration,
            elapsed: 0.0,
            easing,
        };
        self.animations.insert(id.clone(), animation);
        Ok(())
    }

    /// Moves the animations forward by a time in milliseconds. Returns the
    /// nodes whose animations ended.
    pub fn advance(&mut self, ms: f64) -> Vec<String> {
        let ids: Vec<String> = self.animations.keys().cloned().collect();
        let mut done = Vec::new();
        for id in ids {
            let Some(animation) = self.animations.get_mut(&id) else {
                continue;
            };
            animation.elapsed += ms;
            let (props, ended) = (animation.props(), animation.progress() >= 1.0);
            if let Some(node) = self.nodes.get_mut(&id) {
                node.props = props;
            }
            if ended {
                self.animations.remove(&id);
                done.push(id);
            }
        }
        done
    }

    /// Removes the effects of the nodes of an owner, for the watchdog: the
    /// meshes and the shaders, also at the start and the end of
    /// animations. Returns whether there were some.
    pub fn drop_effects(&mut self, owner: &str) -> bool {
        let of_owner =
            |node: Option<&Node>| node.is_some_and(|n| n.owner.as_deref() == Some(owner));
        let mut found = false;
        let ids: Vec<String> = self.nodes.keys().cloned().collect();
        for id in ids {
            if let Some(node) = self.nodes.get_mut(&id).filter(|n| of_owner(Some(n))) {
                found |= strip(&mut node.props);
            }
        }
        let ids: Vec<String> = self.animations.keys().cloned().collect();
        for id in ids {
            if !of_owner(self.nodes.get(&id)) {
                continue;
            }
            if let Some(animation) = self.animations.get_mut(&id) {
                found |= strip(&mut animation.from);
                found |= animation.to.mesh.take().is_some() | animation.to.effects.take().is_some();
            }
        }
        found
    }

    /// The load that the visible nodes of each owner put on a frame, for
    /// the watchdog: a node counts 1, a layer 25 more, and each effect 25
    /// more.
    #[must_use]
    pub fn load_by_owner(&self) -> BTreeMap<String, u64> {
        // ponytail: fixed weights from the thresholds of #59, where 200
        // rounded rects slow a frame as much as 5000 plain ones; measure
        // the passes of each owner if the blame misses.
        const LAYER: u64 = 25;
        let mut load = BTreeMap::new();
        let overlays = self.nodes.keys().filter(|id| is_overlay(id));
        for start in std::iter::once(ROOT).chain(overlays.map(String::as_str)) {
            for placed in self.walk(start, false) {
                let node = placed.node;
                let Some(owner) = &node.owner else {
                    continue;
                };
                let effects = node.props.effects.len() as u64;
                let cost = 1 + LAYER * (u64::from(node.is_layer()) + effects);
                *load.entry(owner.clone()).or_default() += cost;
            }
        }
        load
    }

    /// Moves the cameras forward by a time in milliseconds. Returns the
    /// outputs whose cameras reached their targets.
    pub fn advance_cameras(&mut self, ms: f64) -> Vec<String> {
        let outputs: Vec<String> = self.camera_moves.keys().cloned().collect();
        let mut done = Vec::new();
        for output in outputs {
            let Some(moving) = self.camera_moves.get_mut(&output) else {
                continue;
            };
            moving.elapsed += ms;
            if moving.ended() {
                self.camera_moves.remove(&output);
                done.push(output);
            }
        }
        done
    }

    /// Whether an animation runs, of a node or of a camera.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.animations.is_empty() || !self.camera_moves.is_empty()
    }

    /// Removes a node and its descendants. Surfaces become detached and
    /// lose their owner. Returns the detached surfaces.
    fn remove_subtree(&mut self, id: &str) -> Vec<String> {
        self.unlink(id);
        let mut detached = Vec::new();
        let mut stack = vec![id.to_owned()];
        while let Some(current) = stack.pop() {
            self.animations.remove(&current);
            if self.focused_node.as_ref() == Some(&current) {
                self.focused_node = None;
            }
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

    /// A client focuses its own surfaces. After it cleared the focus, it may
    /// give the focus back to the surface that had it, also of another
    /// client. A new focus ends every return, as in [`Scene::set_focused`].
    fn focus_command(&mut self, owner: &str, surface: Option<&str>) -> Result<(), SceneError> {
        // The focus follows the commands, so that a later command of the
        // same transaction sees it.
        let Some(id) = surface else {
            if let Some(had) = self.focused.take() {
                self.focus_returns.insert(owner.into(), had);
            }
            return Ok(());
        };
        let returns = self.focus_returns.get(owner).is_some_and(|had| had == id);
        if returns {
            self.surface(id)?;
            self.focus_returns.remove(owner);
        } else {
            self.owned_surface(owner, id)?;
        }
        self.set_focused(Some(id));
        self.focused_node = None;
        Ok(())
    }

    /// A client gives the keyboard to one of its nodes that is no surface,
    /// or ends the focus of its node.
    fn focus_node_command(&mut self, owner: &str, node: Option<&str>) -> Result<(), SceneError> {
        let Some(id) = node else {
            if self.focused_node_owner() == Some(owner) {
                self.focused_node = None;
            }
            return Ok(());
        };
        if self.owned_mut(owner, id)?.kind == NodeKind::Surface {
            return Err(SceneError::WrongKind(
                id.into(),
                "group, rect, image or shader; a surface takes the focus with focus",
            ));
        }
        self.focused_node = Some(id.into());
        Ok(())
    }

    /// The node with the keyboard focus of `focus_node`.
    #[must_use]
    pub fn focused_node(&self) -> Option<&str> {
        self.focused_node.as_deref()
    }

    /// The owner of the node with the keyboard focus.
    #[must_use]
    pub fn focused_node_owner(&self) -> Option<&str> {
        let node = self.nodes.get(self.focused_node.as_deref()?)?;
        node.owner.as_deref()
    }

    /// Ends the keyboard focus of a node, for example when the session
    /// locks or its client disconnects.
    pub fn unfocus_node(&mut self) {
        self.focused_node = None;
    }

    /// Whether a node is `ancestor` or below it.
    #[must_use]
    pub fn is_within(&self, id: &str, ancestor: &str) -> bool {
        let mut current = Some(id);
        while let Some(node) = current {
            if node == ancestor {
                return true;
            }
            current = self.nodes.get(node).and_then(|n| n.parent.as_deref());
        }
        false
    }

    /// Tells the scene the surface with the keyboard focus. A focus that
    /// the scene did not know yet, for example a click of the user, ends
    /// every return of an earlier focus, so that no client takes the focus
    /// from a window that the user chose since.
    pub fn set_focused(&mut self, surface: Option<&str>) {
        if surface.is_some() && self.focused.as_deref() != surface {
            self.focus_returns = OrdMap::new();
            self.focused_node = None;
        }
        self.focused = surface.map(Into::into);
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

/// Whether the id has the form of a surface id: `s` and digits. The core
/// gives these ids to surfaces before their nodes exist.
/// The start of the ids of the overlay roots of the outputs (D43).
pub const OVERLAY: &str = "screen:";

/// The id of the overlay root of an output.
#[must_use]
pub fn overlay_id(output: &str) -> String {
    format!("{OVERLAY}{output}")
}

fn is_overlay(id: &str) -> bool {
    id.starts_with(OVERLAY)
}

fn is_surface_id(id: &str) -> bool {
    id.strip_prefix('s')
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// The longest path of an image file.
const MAX_PATH: usize = 4096;
/// The most keys in the meta data of a node.
const MAX_META_KEYS: usize = 32;
/// The longest key or value of the meta data, in bytes.
const MAX_META_VALUE: usize = 1024;
/// The most bytes of all keys and values of the meta data of a node.
const MAX_META_BYTES: usize = 4096;

/// The meta data of a node after an update stays within its limits, so
/// that no client fills the memory of the core or the answers of queries.
fn check_meta(
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

/// Only rects and images have a size, only rects a color, only images a
/// file and a slice. Image paths are absolute, sizes and slices are not
/// negative, and all numbers are finite.
fn check_props(id: &str, kind: NodeKind, props: &PropsUpdate) -> Result<(), SceneError> {
    check_kinds(id, kind, props)?;
    check_numbers(id, props)?;
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
    Ok(())
}

/// The properties of each kind of node.
fn check_kinds(id: &str, kind: NodeKind, props: &PropsUpdate) -> Result<(), SceneError> {
    let wrong = |kinds| Err(SceneError::WrongKind(id.into(), kinds));
    if kind == NodeKind::Surface && (props.size.is_some() || props.clip.is_some()) {
        return wrong("group, rect, image or shader");
    }
    if kind != NodeKind::Rect
        && (props.color.is_some() || props.radius.is_some() || props.border.is_some())
    {
        return wrong("rect");
    }
    if kind != NodeKind::Image && (props.image.is_some() || props.slice.is_some()) {
        return wrong("image");
    }
    if kind != NodeKind::Shader && (props.shader.is_some() || props.uniforms.is_some()) {
        return wrong("shader");
    }
    Ok(())
}

/// The radius, the border, the clip and the uniforms of a shader node.
fn check_shapes(id: &str, props: &PropsUpdate) -> Result<(), SceneError> {
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
    if let Some(uniforms) = &props.uniforms {
        check_uniforms(id, uniforms, true)?;
        let textures = uniforms
            .values()
            .filter_map(|u| match u {
                Uniform::Texture(t) => Some(t),
                _ => None,
            })
            .collect::<Vec<_>>();
        if textures.len() > MAX_NODE_TEXTURES {
            return Err(SceneError::Invalid(format!(
                "{id:?}: at most {MAX_NODE_TEXTURES} node textures"
            )));
        }
        if textures.iter().any(|t| t.node == id) {
            return Err(SceneError::Invalid(format!(
                "{id:?} cannot read itself as a texture"
            )));
        }
    }
    Ok(())
}

/// Every number is finite, and sizes are not negative.
fn check_numbers(id: &str, props: &PropsUpdate) -> Result<(), SceneError> {
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
        .chain(props.slice.iter().flatten());
    if negative.copied().any(|n| n < 0.0) {
        return Err(SceneError::Invalid(format!("negative size for {id:?}")));
    }
    Ok(())
}

/// Removes the mesh, the effects and the shader of properties. Returns
/// whether there were some.
fn strip(props: &mut Props) -> bool {
    let found = props.mesh.is_some() || !props.effects.is_empty() || props.shader.is_some();
    props.mesh = None;
    props.effects.clear();
    props.shader = None;
    found
}

/// At most 8 effects with absolute paths, at most 16 uniforms each, with
/// names of GLSL identifiers and one to four numbers.
fn check_effects(id: &str, effects: &[ShaderEffect]) -> Result<(), SceneError> {
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
fn check_uniforms(
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
            _ => (1..=4).contains(&value.values().len()),
        };
        if !identifier || !fits {
            return invalid(&format!(
                "the uniform {name:?} needs a name and 1 to 4 numbers{}",
                if textures { ", or a node" } else { "" }
            ));
        }
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
        to: None,
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
fn grid(count: usize, [left, top, width, height]: [f64; 4]) -> Vec<[f64; 4]> {
    if count == 0 {
        return Vec::new();
    }
    let cols = (count as f64).sqrt().ceil() as usize;
    let rows = count.div_ceil(cols);
    let (cw, ch) = (width / cols as f64, height / rows as f64);
    (0..count)
        .map(|i| {
            let (col, row) = ((i % cols) as f64, (i / cols) as f64);
            [left + col * cw, top + row * ch, cw, ch]
        })
        .collect()
}
