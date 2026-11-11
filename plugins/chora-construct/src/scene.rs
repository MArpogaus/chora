//! The construct as a whole: start, serve, load a construct file, leave
//! it, plugin panes, snapshot and list.

use super::{
    ClientError, Command, Config, Construct, Content, Duration, EVENTS, FRAME, Hold, Instant, Kind,
    NAME, Pane, PaneFile, RETRY, Value, Waiting, exported_methods, file, json, runtime, settings,
};

impl Construct {
    /// Names the theme of the space to the HUD host, which colours its
    /// look by it.
    pub(crate) fn name_theme(&mut self) -> Result<(), ClientError> {
        let dark = self.settings.value.theme == crate::config::Theme::Dark;
        let emit = chora_kit::hud::theme_event(NAME, dark);
        self.rt.conn.call::<_, Value>("events.emit", emit).map(drop)
    }

    /// Builds the space, then takes the events and the free windows.
    pub(crate) fn start(&mut self) -> Result<(), ClientError> {
        if !self.rt.conn.query(&format!("id:{NAME}"))?.is_empty() {
            self.rt.conn.apply(vec![Command::Remove {
                id: NAME.into(),
                to: None,
            }])?;
        }
        self.output = (self.rt.outputs()?.into_iter()).find(|o| o.is_virtual != Some(true));
        self.remove_left_outputs()?;
        let mut methods = exported_methods();
        methods.extend(settings::File::<Config>::methods());
        self.rt.announce(&EVENTS, methods)?;
        if self.overlay {
            return Ok(());
        }
        self.name_theme()?;
        self.active = true;
        self.build()?;
        self.regrab()?;
        self.claim_indicator(false);
        for node in self.rt.conn.query("kind:surface")? {
            let ours = node.to.as_deref().is_none_or(|t| t == NAME);
            if node.owner.is_none() && ours {
                let info = node.surface.unwrap_or_default();
                let label = info.app_id.or(info.title).unwrap_or_default();
                self.adopt(&node.id, &label, None, Hold::Own, false)?;
            }
        }
        let start = self.settings.value.start.clone();
        if !start.is_empty() {
            match file::dir().and_then(|d| file::load(&d, &start)) {
                Ok(file) => self.load(&start, file)?,
                Err(err) => eprintln!("chora-construct: start: {err}"),
            }
        }
        Ok(())
    }

    /// Handles events; during a glide, frames come every 16 ms.
    pub(crate) fn serve(&mut self) -> Result<(), ClientError> {
        loop {
            let plugins = self.waiting.iter().any(|w| w.pane.kind != Kind::Program);
            let wait = if self.glide.is_some() || self.effects.animating() || self.cube_turns() {
                FRAME
            } else if plugins {
                RETRY
            } else {
                Duration::from_secs(3600)
            };
            let wait = (self.voice.deadline().into_iter())
                .chain(self.cube_deadline())
                .chain(self.hints.until)
                .fold(wait, |wait, t| {
                    wait.min(t.saturating_duration_since(Instant::now()))
                });
            match self.rt.conn.next_event_timeout(wait)? {
                Some(event) => runtime::dispatch(self, event)?,
                None if plugins => self.retry_plugins()?,
                None => {}
            }
            self.tick()?;
            self.check_cube()?;
            self.animation_frame()?;
        }
    }

    /// One frame of the glide of the camera, of the wobbles and of the
    /// turn of the cube.
    pub(crate) fn animation_frame(&mut self) -> Result<(), ClientError> {
        if self.glide.is_some() {
            self.step_glide()?;
        }
        if self.effects.animating() {
            self.wobble_frame()?;
        }
        if self.cube_turns() {
            self.step_cube()?;
        }
        Ok(())
    }

    /// Loads a construct (F18). An overlay takes the output now; a
    /// flavor keeps the windows that it has as panes, and an overlay the
    /// windows that it owns, which were given to it: no window gets lost.
    /// The other panes of the construct before go.
    pub(crate) fn load(&mut self, name: &str, file: file::File) -> Result<(), ClientError> {
        if self.flown.is_some() {
            self.fly_out()?;
        }
        self.close_list()?;
        if self.overlay && !self.active {
            self.focus_before = self.focused_surface()?;
        }
        self.glide = None;
        let overlay = self.overlay;
        let (kept, gone): (Vec<Pane>, Vec<Pane>) = std::mem::take(&mut self.panes)
            .into_iter()
            .partition(|p| p.window().is_some() && (p.attached || !overlay));
        self.panes = kept;
        let commands: Vec<Command> = gone.iter().flat_map(Pane::remove).collect();
        self.rt.conn.apply(commands)?;
        self.end_managers();
        self.active = true;
        self.waiting.clear();
        self.build()?;
        self.radius = file.radius;
        self.pose = file.camera;
        self.current = Some(name.into());
        for pane in file.panes {
            match pane.kind {
                Kind::Program => {
                    let mint = self.rt.conn.activation_token();
                    let command = pane.command.as_deref().unwrap_or_default();
                    let token = chora_kit::launch(command, mint.as_deref()).map(|(t, _)| t);
                    self.waiting.push(Waiting::new(token, pane));
                }
                Kind::Plugin => {
                    if let Some(pane) = self.show_plugin(pane)? {
                        self.waiting.push(Waiting::new(None, pane));
                    }
                }
                Kind::Manager => {
                    if let Some(pane) = self.show_manager(pane)? {
                        self.waiting.push(Waiting::new(None, pane));
                    }
                }
            }
        }
        self.frame()
    }

    /// Gives the output back (F20, F31): the space goes, and the keyboard
    /// goes back to the window that had it, through its owner (D31). The
    /// windows stay with their owner, which the construct never changed.
    pub(crate) fn leave(&mut self) -> Result<(), ClientError> {
        if !self.active {
            return Ok(());
        }
        self.close_cube()?;
        self.end_managers();
        self.rt.conn.apply(vec![
            Command::FocusNode { node: None },
            Command::Remove {
                id: NAME.into(),
                to: None,
            },
        ])?;
        if let Some(surface) = self.focus_before.take() {
            self.rt.conn.try_apply(vec![Command::Focus {
                surface: Some(surface),
            }])?;
        }
        self.list = None;
        self.labelled = None;
        self.label_drawn = false;
        self.rt.ungrab_all()?;
        self.active = false;
        self.current = None;
        self.radius = None;
        self.panes.clear();
        self.waiting.clear();
        self.sent = None;
        self.glow = None;
        self.backs.clear();
        self.flown = None;
        self.glide = None;
        self.moving = None;
        self.drag = None;
        Ok(())
    }

    /// Shows a plugin pane through the node that its plugin tags
    /// `meta:pane=SCENE`; answers the pane back when the node is missing.
    pub(crate) fn show_plugin(&mut self, file: PaneFile) -> Result<Option<PaneFile>, ClientError> {
        let (Some(plugin), Some(scene)) = (file.plugin.clone(), file.scene.clone()) else {
            return Ok(None);
        };
        let nodes = (self.rt.conn).query(&format!("owner:{plugin} meta:pane={scene}"))?;
        let Some(node) = nodes.iter().find(|n| n.id == plugin).or(nodes.first()) else {
            return Ok(Some(file));
        };
        let size = (file.size)
            .or(node.props.size)
            .unwrap_or([self.lens.w, self.lens.h]);
        let at = self.place(file.at, size, true).unwrap_or_default();
        let pane = Pane {
            name: self.label(&scene),
            source: node.id.clone(),
            content: Content::Plugin { plugin, scene },
            at,
            yaw: self.yaw_at(Some(&file), at),
            size,
            taken: false,
            attached: false,
        };
        self.rt.conn.apply(self.pane_nodes(&pane))?;
        self.panes.push(pane);
        Ok(None)
    }

    /// Shows the manager pane of a file; answers the pane back while a
    /// client of that name still runs, as the manager of the construct
    /// before, which ends. A pane whose manager the setting `managers`
    /// lacks goes, with one message.
    pub(crate) fn show_manager(&mut self, file: PaneFile) -> Result<Option<PaneFile>, ClientError> {
        let manager = file.manager.clone().unwrap_or_default();
        if let Some(reason) = super::wm::unknown_manager(&self.settings.value.managers, &manager) {
            eprintln!("chora-construct: the pane of {manager} goes: {reason}");
            return Ok(None);
        }
        match self.open_manager(&manager, None, Some(&file)) {
            Ok(_) => Ok(None),
            Err(ClientError::Rpc(err)) if err.code == super::RpcError::REJECTED => {
                eprintln!(
                    "chora-construct: the pane of {manager} waits: {}",
                    err.message
                );
                Ok(Some(file))
            }
            Err(err) => Err(err),
        }
    }

    /// Looks again for the nodes of the plugin panes that wait, and for
    /// the window managers of the manager panes that wait.
    pub(crate) fn retry_plugins(&mut self) -> Result<(), ClientError> {
        let (plugins, programs): (Vec<Waiting>, Vec<Waiting>) = std::mem::take(&mut self.waiting)
            .into_iter()
            .partition(|w| w.pane.kind != Kind::Program);
        self.waiting = programs;
        let mut shown = false;
        for waiting in plugins {
            let Waiting { pane, .. } = waiting;
            let left = if pane.kind == Kind::Manager {
                self.show_manager(pane)?
            } else {
                self.show_plugin(pane)?
            };
            match left {
                Some(pane) => self.waiting.push(Waiting { pane, ..waiting }),
                None => shown = true,
            }
        }
        if shown { self.frame() } else { Ok(()) }
    }

    /// The construct as a file: the camera, the radius, the panes and the
    /// plugin panes that wait.
    pub(crate) fn snapshot(&self) -> file::File {
        let camera = self
            .flown
            .as_ref()
            .map_or_else(|| self.target(), |f| f.back);
        let panes = (self.panes.iter().filter_map(Pane::file))
            .chain(self.waiting.iter().filter_map(Waiting::file).cloned())
            .collect();
        file::File {
            camera,
            radius: self.radius,
            panes,
        }
    }

    /// The answer of `construct.list`: the files, the loaded construct
    /// and its panes with their labels.
    pub(crate) fn list(&self) -> Value {
        let panes: Vec<Value> = (self.panes.iter())
            .map(|p| {
                let mut v = json!({"name": p.name, "at": p.at, "yaw": p.yaw, "size": p.size});
                if let Some(surface) = p.window() {
                    v["surface"] = json!(surface);
                    v["taken"] = json!(p.taken);
                }
                match &p.content {
                    Content::Window { command } => {
                        v["kind"] = json!("program");
                        v["command"] = json!(command);
                    }
                    Content::Plugin { plugin, scene } => {
                        v["kind"] = json!("plugin");
                        v["plugin"] = json!(plugin);
                        v["scene"] = json!(scene);
                    }
                    Content::Manager { manager } => {
                        v["kind"] = json!("manager");
                        v["manager"] = json!(manager);
                        v["output"] = json!(self.manager_output(manager));
                    }
                }
                v
            })
            .collect();
        let waiting: Vec<&PaneFile> = self.waiting.iter().map(|w| &w.pane).collect();
        json!({
            "constructs": file::dir().map(|d| file::list(&d)).unwrap_or_default(),
            "current": self.current,
            "overlay": self.overlay,
            "shown": self.active,
            "panes": panes,
            "waiting": waiting,
        })
    }
}
