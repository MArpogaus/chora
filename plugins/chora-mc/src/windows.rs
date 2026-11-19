use super::{
    ClientError, Command, Easing, FIT_MS, Mc, NAME, NodeKind, PARKED, PropsUpdate, Query, Scene,
    Target, ToplevelState, Value, create, name_of, regions_on_top, scene_id, wait,
};

impl Mc {
    /// Takes a new window into the scene that started it, or into a new
    /// scene of its own, and shows it.
    /// The start adopts the free surfaces that events then announce too.
    /// A new window takes the keyboard, except one of app.open.
    /// Another window manager took it first; the new scene goes.
    /// The overview lists the new scene.
    pub(crate) fn adopt(
        &mut self,
        surface: &str,
        app_id: Option<&str>,
        token: Option<&str>,
    ) -> Result<(), ClientError> {
        let mut known = (self.windows.values())
            .chain(self.docks.values())
            .chain(self.fitting.keys());
        if !self.built || known.any(|s| s == surface) {
            return Ok(());
        }
        let started = token.and_then(|t| self.starting.remove(t));
        let user = !token.is_some_and(|t| self.quiet.remove(t));
        if let Some(Target::Dock(content)) = &started
            && !self.docks.contains_key(content)
        {
            return self.adopt_dock(surface, content);
        }
        let mut commands = Vec::new();
        let name = match started {
            Some(Target::Scene(name)) if !self.windows.contains_key(&name) => name,
            _ => {
                let mut name = name_of(app_id.unwrap_or("app"));
                if self.scenes.contains_key(&name) {
                    name = format!("{name}-{surface}");
                }
                self.add_scene(&name, Scene::default());
                self.transient.insert(name.clone());
                commands.push(create(
                    &scene_id(&name),
                    NodeKind::Group,
                    NAME,
                    PropsUpdate {
                        visible: Some(false),
                        opacity: Some(0.0),
                        ..PropsUpdate::default()
                    },
                ));
                commands.push(regions_on_top());
                name
            }
        };
        let new = !commands.is_empty();
        commands.extend(self.place_window(&name, surface, true));
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            eprintln!("chora-mc: {surface}: {err}");
            if new {
                self.scenes.remove(&name);
                self.transient.remove(&name);
                self.appeared.retain(|n| *n != name);
            }
            return Ok(());
        }
        self.windows.insert(name.clone(), surface.into());
        if let Err(err) = self.show(&name, user) {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        if new {
            self.renumber()?;
        }
        Ok(())
    }

    /// Takes the window of a docked app that started. It waits hidden
    /// until it has the size of its region, so that it never shows at the
    /// size that it chose itself.
    /// A client that takes another size shows after this time.
    pub(crate) fn adopt_dock(&mut self, surface: &str, content: &str) -> Result<(), ClientError> {
        let mut commands = vec![Command::Attach {
            surface: surface.into(),
            parent: PARKED.into(),
        }];
        let Some(size) = self.dock_size(content) else {
            self.apply(commands)?;
            self.fitting
                .insert(surface.into(), (content.into(), [0, 0]));
            return self.fit(surface);
        };
        commands.push(Command::Configure {
            surface: surface.into(),
            size: Some(size),
            states: vec![ToplevelState::Tiled],
        });
        commands.push(Command::Animate {
            id: PARKED.into(),
            props: PropsUpdate::default(),
            duration: FIT_MS,
            easing: Easing::Linear,
        });
        self.apply(commands)?;
        self.fitting.insert(surface.into(), (content.into(), size));
        Ok(())
    }

    /// A docked app that waited parked goes into its region, with the
    /// keyboard when the user started it. A new dock changes the items,
    /// so the render draws anew.
    pub(crate) fn fit(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some((content, _)) = self.fitting.remove(surface) else {
            return Ok(());
        };
        let user = self.dock_focus.as_ref() == Some(&content);
        if user {
            self.dock_focus = None;
        }
        self.docks.insert(content, surface.into());
        self.render()?;
        if user {
            self.focus(Some(surface.into()))?;
        }
        Ok(())
    }

    /// A surface lost its parent and its owner. One of the plugin, for
    /// example after a hotplug, goes back to its scene or region: the
    /// plugin still counts it as its own, and a later move of it failed
    /// with "belongs to nobody" (W11).
    /// Back into its region.
    /// Given to mc (D55) or free: the app id names its scene.
    pub(crate) fn reattach(&mut self, surface: &str) -> Result<(), ClientError> {
        let scene = (self.windows.iter()).find(|(_, s)| *s == surface);
        if let Some(name) = scene.map(|(n, _)| n.clone()) {
            let commands = self.place_window(&name, surface, true);
            return self.apply(commands);
        }
        let docked = self.docks.values().any(|s| s == surface);
        if docked || self.fitting.contains_key(surface) {
            self.apply(vec![Command::Attach {
                surface: surface.into(),
                parent: PARKED.into(),
            }])?;
            if docked && self.rendered.is_some() {
                self.rendered = Some(Vec::new());
            }
            return self.render();
        }
        let nodes: Vec<Value> =
            (self.rt.conn).call("scene.query", Query::new(format!("id:{surface}")))?;
        let app_id = nodes.first().and_then(|n| n["surface"]["app_id"].as_str());
        self.adopt(surface, app_id, None)
    }

    /// A window closed: the scene before shows instead of a scene without
    /// its window, and the scene of `app.open` or of a window that no
    /// scene started goes once its app ended.
    /// A docked app ended: its region hides, the keyboard goes back.
    /// The scene goes when the app ended too; until then it waits.
    pub(crate) fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        if self.fitting.remove(surface).is_some() {
            return Ok(());
        }
        let dock = (self.docks.iter()).find(|(_, s)| *s == surface);
        if let Some(content) = dock.map(|(c, _)| c.clone()) {
            self.docks.remove(&content);
            let name = self.current.clone().unwrap_or_default();
            if let Some(region) = (self.placed(&name).into_iter())
                .find(|(_, slots)| slots.iter().any(|(_, c)| *c == content))
                .map(|(r, _)| r)
            {
                self.toggled.insert(region, false);
            }
            if self.focused.as_deref() == Some(surface) {
                let window = self.windows.get(&name).cloned();
                self.focus(window)?;
            }
            return self.render();
        }
        let Some(name) = self
            .windows
            .iter()
            .find(|(_, s)| *s == surface)
            .map(|(n, _)| n.clone())
        else {
            return Ok(());
        };
        self.windows.remove(&name);
        self.leave(&name, true);
        if !self.transient.contains(&name) {
            return Ok(());
        }
        let pid = self.pids.get(&name).copied().flatten();
        if pid.is_some_and(chora_kit::launch::alive) {
            self.launched.push((name, pid));
            return self.apply(vec![wait()]);
        }
        self.remove_scene(&name)
    }
}
