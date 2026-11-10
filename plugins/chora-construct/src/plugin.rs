//! The `Plugin` implementation: events, methods and settings of the
//! construct.

use super::{
    ClientError, Command, Construct, Entry, Event, Give, Grab, LOOK, MovePane, Named, Plugin,
    ResizePane, RpcError, Runtime, Store, Value, file, give, json, moved, runtime,
};

impl Plugin for Construct {
    type Action = Grab;

    fn runtime(&mut self) -> &mut Runtime<Grab> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::SurfaceCreated {
                surface,
                app_id,
                title,
                to,
                pid,
                token,
                ..
            } => {
                let label = app_id.or(title).unwrap_or_default();
                self.created(&surface, &label, to.as_deref(), pid, token.as_deref())
            }
            Event::SurfaceDetached { surface } => self.detached(&surface),
            Event::SurfaceDestroyed { surface }
                if self.launcher_window.as_deref() == Some(&surface) =>
            {
                self.launcher_window = None;
                self.rt.conn.apply(vec![Command::FocusNode {
                    node: Some(LOOK.into()),
                }])
            }
            Event::SurfaceDestroyed { surface } => {
                self.hidden.retain(|s| *s != surface);
                self.forget(&surface)
            }
            Event::SurfaceChanged {
                surface,
                size: Some([w, h]),
                ..
            } => self.resize(&surface, [f64::from(w), f64::from(h)]),
            Event::OutputChanged { output }
                if self.output.as_ref().is_none_or(|o| o.name == output.name) =>
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
                self.output = self.rt.outputs()?.into_iter().find(|o| o.name != name);
                if self.output.is_some() {
                    self.build()?;
                }
                Ok(())
            }
            event => self.other(event),
        }
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
                let Give { surface, to } = runtime::params(params)?;
                self.give(&surface, &to).map_err(rejected)
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
            "pane.fly" => {
                let Named { name } = runtime::params(params)?;
                let i = self.pane_named(&name)?;
                if self.flown.is_some() {
                    self.fly_out().map_err(rejected)?;
                }
                self.fly_in(i);
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
        self.effects.set(&self.settings.value.effects);
        self.regrab().map_err(rejected)?;
        self.restyle().map_err(rejected)?;
        self.frame().map_err(rejected)
    }
}
