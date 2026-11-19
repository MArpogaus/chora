//! The commands of a batch and the camera rules of the scene.

use super::{
    BTreeMap, CameraMove, Command, Easing, Effect, MAX_CAMERAS, MAX_DURATION, MAX_PAN, MAX_ZOOM,
    NodeKind, Scene, SceneError, Transform, check_meta, check_props, configure, flat, owned_node,
    request, valid_camera,
};

impl Scene {
    /// Only window managers receive windows (D55).
    pub(super) fn apply_one(
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
                self.check_reader(owner, id, props)?;
                check_meta(id, &BTreeMap::new(), props)?;
                if *kind == NodeKind::Shader && props.shader.is_none() {
                    return Err(SceneError::Invalid(format!(
                        "the shader node {id:?} needs a shader"
                    )));
                }
                if *kind == NodeKind::Text && props.text.is_none() {
                    return Err(SceneError::Invalid(format!(
                        "the text node {id:?} needs a text"
                    )));
                }
                self.create(owned_node(owner, id, *kind, props), parent)
            }
            Command::Attach { surface, parent } => self.attach(owner, surface, parent),
            Command::Update { .. } | Command::Animate { .. } => {
                self.change_props(owner, command, effects)
            }
            Command::Move { .. } => self.move_command(owner, command, effects),
            Command::Remove { id, to: None } => {
                self.owned_mut(owner, id)?;
                effects.extend(self.remove_subtree(id).into_iter().map(Effect::Detached));
                Ok(())
            }
            Command::Remove { id, to: Some(to) } => {
                self.owned_surface(owner, id)?;
                if !self.holds_claim(to) {
                    return Err(SceneError::Invalid(format!(
                        "{to:?} holds no claim; only a window manager receives a window"
                    )));
                }
                self.remove_subtree(id);
                self.reserve(id, Some(to));
                effects.push(Effect::Reserved {
                    surface: id.clone(),
                    to: to.clone(),
                    from: owner.into(),
                });
                Ok(())
            }
            Command::Configure { .. } => self.configure_command(owner, command, effects),
            Command::Focus { surface } => self.focus_command(owner, surface.as_deref(), effects),
            Command::FocusNode { node } => self.focus_node_command(owner, node.as_deref()),
            Command::Close { .. } => self.close_command(owner, command, effects),
            Command::GiveGrab { .. } => Self::give_grab_command(owner, command, effects),
            camera @ Command::Camera { .. } => self.camera_command(owner, camera, effects),
        }
    }

    /// `give_grab`: the drag goes with a surface that the owner reserved
    /// for `to` earlier in this transaction, and with no other.
    pub(super) fn give_grab_command(
        owner: &str,
        command: &Command,
        effects: &mut Vec<Effect>,
    ) -> Result<(), SceneError> {
        let Command::GiveGrab {
            surface,
            to,
            point,
            velocity,
            wobble,
        } = command
        else {
            unreachable!("only give_grab");
        };
        let reserved = effects.iter().any(|e| {
            matches!(e, Effect::Reserved { surface: s, to: t, from }
                if s == surface && t == to && from == owner)
        });
        if !reserved {
            return Err(SceneError::Invalid(format!(
                "the drag of {surface:?} goes only with a remove to {to:?} in the same transaction"
            )));
        }
        effects.push(Effect::GrabGiven {
            surface: surface.clone(),
            to: to.clone(),
            from: owner.into(),
            grab: crate::GivenGrab {
                point: *point,
                velocity: *velocity,
                wobble: wobble.clone(),
            },
        });
        Ok(())
    }

    /// `move`: a node of another client answers with a request (D31).
    pub(super) fn move_command(
        &mut self,
        owner: &str,
        command: &Command,
        effects: &mut Vec<Effect>,
    ) -> Result<(), SceneError> {
        let Command::Move { id, parent, index } = command else {
            unreachable!("only move commands come here");
        };
        if let Some(other) = self.other_owner(owner, id) {
            let args = serde_json::json!({"parent": parent, "index": index});
            effects.push(request(owner, id, other, "move", args));
            return Ok(());
        }
        self.move_node(owner, id, parent.as_deref(), *index)
    }

    /// `configure`: a surface of another client answers with a request
    /// for a resize (D31).
    pub(super) fn configure_command(
        &mut self,
        owner: &str,
        command: &Command,
        effects: &mut Vec<Effect>,
    ) -> Result<(), SceneError> {
        let Command::Configure {
            surface,
            size,
            states,
        } = command
        else {
            unreachable!("only configure commands come here");
        };
        self.surface(surface)?;
        if size.is_some_and(|[w, h]| w < 0 || h < 0) {
            return Err(SceneError::Invalid(format!(
                "negative size for {surface:?}"
            )));
        }
        if let Some(other) = self.other_owner(owner, surface) {
            let args = serde_json::json!({"size": size, "states": states});
            effects.push(request(owner, surface, other, "resize", args));
            return Ok(());
        }
        self.owned_surface(owner, surface)?;
        effects.push(configure(surface, *size, states.clone()));
        Ok(())
    }

    /// `close`: a surface of another client answers with a request (D31).
    pub(super) fn close_command(
        &mut self,
        owner: &str,
        command: &Command,
        effects: &mut Vec<Effect>,
    ) -> Result<(), SceneError> {
        let Command::Close { surface } = command else {
            unreachable!("only close commands come here");
        };
        self.surface(surface)?;
        if let Some(other) = self.other_owner(owner, surface) {
            let empty = serde_json::json!({});
            effects.push(request(owner, surface, other, "close", empty));
            return Ok(());
        }
        self.owned_surface(owner, surface)?;
        effects.push(Effect::Close(surface.clone()));
        Ok(())
    }

    pub(super) fn camera_command(
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

    /// Only outputs that the core knows, so that no client claims an
    /// output before it plugs in or uses up the cameras with names.
    /// A move starts where the camera is now, also in the middle of
    /// another move.
    pub(super) fn set_camera(
        &mut self,
        owner: &str,
        output: &str,
        transform: Option<Transform>,
        timing: Option<(f64, Easing)>,
    ) -> Result<(), SceneError> {
        if let Some((other, _)) = self.cameras.get(output).filter(|(o, _)| o != owner) {
            return Err(SceneError::CameraOwner(output.into(), other.clone()));
        }
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
            return Err(SceneError::Limit(format!(
                "{output:?}: the scene holds {MAX_CAMERAS} cameras, its limit"
            )));
        }
        self.cameras.insert(output.into(), (owner.into(), t));
        Ok(())
    }

    /// Takes the camera of an output away from its owner, so that the
    /// view goes back to its default. For the operator of the admin
    /// socket, when an owner is gone and does not come back.
    #[must_use]
    pub fn release_camera(&mut self, output: &str) -> Option<String> {
        let (owner, _) = self.cameras.remove(output)?;
        self.camera_moves.remove(output);
        Some(owner)
    }
}
