//! The `Plugin` implementation: events, methods and settings of the
//! construct.

use chora_kit::cube;

use super::{
    ClientError, Construct, Entry, Event, Give, GlideTo, Grab, ManagerPane, MovePane, Named,
    OpenCube, Plugin, Pose, ResizePane, RpcError, Runtime, Store, Value, file, give, json, moved,
    runtime,
};

impl Plugin for Construct {
    type Action = Grab;

    fn runtime(&mut self) -> &mut Runtime<Grab> {
        &mut self.rt
    }

    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        if !chora_kit::runtime::hears_pointer(&event, self.rt.accepts_injected()) {
            return Ok(());
        }
        self.voiced.hear_asking(&event, &mut self.rt.conn);
        let Some(event) = self.pulled(event)? else {
            return Ok(());
        };
        if let Some(created) = chora_kit::wm::Created::of(&event) {
            return self.window_created(&created);
        }
        if let Some(surface) = chora_kit::wm::focus::changed(&event) {
            self.core_focus = surface.map(String::from);
            return Ok(());
        }
        if matches!(
            event,
            Event::OutputChanged { .. } | Event::OutputRemoved { .. }
        ) {
            self.book.apply(&event);
        }
        match event {
            Event::SurfaceDetached { surface } => self.window_detached(&surface),
            Event::SurfaceDestroyed { ref surface } => {
                let surface = surface.clone();
                self.window_destroyed(&event, &surface)
            }
            Event::SurfaceChanged {
                surface,
                size: Some([w, h]),
                ..
            } => self.resize(&surface, [f64::from(w), f64::from(h)]),
            Event::OutputChanged { output }
                if output.is_virtual != Some(true)
                    && self.output.as_ref().is_none_or(|o| o.name == output.name) =>
            {
                let same = self
                    .output
                    .as_ref()
                    .is_some_and(|o| o.size == output.size && o.position == output.position);
                self.output = Some(output);
                if same { Ok(()) } else { self.build() }
            }
            Event::OutputRemoved { name }
                if self.output.as_ref().is_some_and(|o| o.name == name) =>
            {
                self.output = (self.rt.outputs()?.into_iter())
                    .find(|o| o.name != name && o.is_virtual != Some(true));
                if self.output.is_some() {
                    self.build()?;
                }
                Ok(())
            }
            event if host_ready(&event) => self.name_theme(),
            event if chat_closed(&event) => self.keyboard_back(),
            event => self.other(event),
        }
    }

    fn state(&self) -> Option<Value> {
        Some(self.state_value())
    }

    #[allow(clippy::too_many_lines, reason = "one arm per method")]
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.waiting.retain(|w| !w.stale());
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        let invalid = |e: String| RpcError::new(RpcError::INVALID_PARAMS, e);
        match method {
            "construct.load" => {
                let Named { name } = runtime::params(params)?;
                let file = file::dir()
                    .and_then(|d| file::load(&d, &name))
                    .map_err(invalid)?;
                self.load(&name, file).map_err(rejected)?;
                Ok(self.list())
            }
            "construct.save" => {
                let Named { name } = runtime::params(params)?;
                let dir = file::dir().map_err(invalid)?;
                file::save(&dir, &name, &self.snapshot()).map_err(invalid)?;
                self.current = Some(name);
                Ok(Value::Null)
            }
            "construct.list" => Ok(self.list()),
            chora_kit::clock::METHOD => {
                chora_kit::clock::step(params)?;
                self.animation_frame().map_err(rejected)?;
                Ok(Value::Null)
            }
            "cube.open" => {
                let OpenCube { faces } = runtime::params(params)?;
                self.open_cube(faces).map_err(rejected)
            }
            cube::LIST => Ok(json!(self.cube_listing())),
            cube::TURN => {
                let turn: cube::Turn = runtime::params(params)?;
                Ok(json!(self.turn_method(turn)?))
            }
            cube::PULL => {
                let cube::Pull { surface } = runtime::params(params)?;
                let pane = self.pull(&surface).map_err(rejected)?;
                Ok(json!({"pane": pane}))
            }
            "view.step_back" => self.mode_method(chora_kit::modes::Act::StepBack),
            "view.step_in" => self.mode_method(chora_kit::modes::Act::StepIn),
            "cube.close" => {
                self.close_cube().map_err(rejected)?;
                Ok(Value::Null)
            }
            "construct.leave" => {
                if !self.overlay {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        "the construct is the flavor; load another construct instead",
                    ));
                }
                self.leave().map_err(rejected)?;
                Ok(Value::Null)
            }
            "pane.move" => {
                let MovePane { name, right, ahead } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                let to = moved(self.panes[i].at, self.pose.forward(), right, ahead);
                self.move_pane(i, to).map_err(rejected)?;
                Ok(json!({"at": self.panes[i].at}))
            }
            "pane.resize" => {
                let ResizePane { name, scale } = runtime::params(params)?;
                if !(scale.is_finite() && scale > 0.0) {
                    return Err(invalid(format!("scale: {scale} is not above 0")));
                }
                let i = self.pane_named(&name)?;
                self.resize_pane(i, scale).map_err(rejected)?;
                Ok(Value::Null)
            }
            "pane.manager" => {
                let ManagerPane { manager, size } = runtime::params(params)?;
                self.open_manager(&manager, size, None).map_err(rejected)
            }
            "pane.remove" => {
                let Named { name } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                self.remove_pane(i).map_err(rejected)?;
                Ok(Value::Null)
            }
            "window.list" => {
                let windows = self.windows().map_err(rejected)?;
                Ok(json!({"windows": windows.iter().map(Entry::json).collect::<Vec<_>>()}))
            }
            "window.show" => {
                let Named { name } = runtime::params(params)?;
                if !self.active {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        "no construct shows; construct.load first",
                    ));
                }
                let windows = self.windows().map_err(rejected)?;
                let entry = (windows.into_iter())
                    .find(|e| e.name == name || e.surface == name)
                    .ok_or_else(|| {
                        invalid(format!("no window {name:?}; window.list names them"))
                    })?;
                let pane = self.show(&entry).map_err(rejected)?;
                Ok(json!({"pane": pane}))
            }
            give::METHOD => {
                let Give { surface, to, grab } = runtime::params(params)?;
                self.give_dragged(&surface, &to, grab).map_err(rejected)
            }
            "help.toggle" => {
                if !self.active {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        "no construct shows; construct.load first",
                    ));
                }
                self.toggle_help().map_err(rejected)?;
                Ok(json!({"shown": self.help}))
            }
            "camera.glide" => {
                let GlideTo {
                    x,
                    z,
                    yaw,
                    pitch,
                    ms,
                } = runtime::params(params)?;
                self.glide_smoothly(Pose { x, z, yaw, pitch }, ms)?;
                Ok(Value::Null)
            }
            "pane.fly" => {
                let Named { name } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                if self.flown.is_some() {
                    self.fly_out().map_err(rejected)?;
                }
                self.fly_in(i).map_err(rejected)?;
                Ok(Value::Null)
            }
            _ => Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        self.glass.plain = false;
        crate::config::log_warnings(&self.settings.value);
        self.effects.set(&self.settings.value.effects);
        self.regrab().map_err(rejected)?;
        self.restyle().map_err(rejected)?;
        self.name_theme().map_err(rejected)?;
        self.frame().map_err(rejected)
    }
}

/// Whether the event is `assistant.ready` of the HUD host, which then
/// listens for the theme.
fn host_ready(event: &Event) -> bool {
    matches!(event, Event::Emitted { event, client, .. }
        if client == "assistant" && event == "assistant.ready")
}

/// Whether the event is `assistant.chat` of the HUD host with the chat
/// closed.
fn chat_closed(event: &Event) -> bool {
    matches!(event, Event::Emitted { event, client, data }
        if client == "assistant" && event == "assistant.chat" && data["open"] == false)
}
