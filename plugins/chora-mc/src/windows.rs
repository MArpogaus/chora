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
        if pid.is_some_and(chora_kit::alive) {
            self.launched.push((name, pid));
            return self.apply(vec![wait()]);
        }
        self.remove_scene(&name)
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::{
        give::Give,
        runtime::Plugin,
        testing::{FakeCore, Script, Subject, created, owner, surface},
    };
    use chora_scene::Event;
    use serde_json::{Value, json};

    use crate::{Mc, NAME, testing};

    /// mc under the script of the life of a window.
    struct Driven<'a>(&'a mut Mc);

    impl Subject for Driven<'_> {
        fn hear(&mut self, event: Event) {
            self.0.event(event).unwrap();
        }

        fn has(&self, surface: &str) -> bool {
            chora_kit::give::Windows::has(&*self.0, surface)
        }

        fn give(&mut self, surface: &str, to: &str) -> bool {
            let give = Give {
                surface: surface.into(),
                to: to.into(),
                grab: None,
            };
            self.0.give(give).is_ok()
        }
    }

    /// The script that every look runs: create, refused attach, reserved
    /// for another window manager, detach (mc puts the window back),
    /// destroy, output removed (the windows stay), give in, refused give
    /// out, give out.
    #[test]
    fn mc_runs_the_script_of_the_life_of_a_window() {
        let core = FakeCore::start();
        let mut mc = testing::mc_on(&core, "script", [800, 600]);
        Script::new(&core, NAME).run(&mut Driven(&mut mc), "test");
    }

    /// The state tells where a window came from, and no token.
    #[test]
    fn the_state_tells_the_origin_of_a_window() {
        let core = FakeCore::start();
        let mut mc = testing::mc_on(&core, "origin", [800, 600]);
        surface(&core.model(), "s1");
        mc.event(created("s1", None)).unwrap();
        surface(&core.model(), "s2");
        core.model().lock().unwrap().scene.reserve("s2", Some(NAME));
        mc.event(created("s2", Some(NAME))).unwrap();
        let state = mc.state().unwrap();
        let origin = |id: &str| {
            let windows = state["windows"].as_array().unwrap();
            let window = windows.iter().find(|w| w["surface"] == id).unwrap();
            window["origin"].clone()
        };
        assert_eq!(origin("s1"), "new");
        assert_eq!(origin("s2"), "given");
        assert!(!state.to_string().contains("t1"), "no token: {state}");
        assert!(
            state.get("open").is_none(),
            "open stays out until mc fills it"
        );
    }

    /// After a restart the core holds the windows free: mc adopts the
    /// free ones and those reserved for it, each into the scene that its
    /// app id names, and leaves those of another manager.
    #[test]
    fn a_restart_adopts_the_free_windows_of_mc() {
        let core = FakeCore::start();
        let mut mc = testing::mc_on(&core, "restart", [800, 600]);
        for id in ["s1", "s2", "s3", "s4"] {
            surface(&core.model(), id);
        }
        let node = |id: &str, owner: Value, to: Value| {
            json!({"id": id, "kind": "surface", "owner": owner, "to": to,
                "surface": {"app_id": "probe"}})
        };
        let nodes = [
            node("s1", Value::Null, Value::Null),
            node("s2", Value::Null, json!(NAME)),
            node("s3", Value::Null, json!("xp")),
            node("s4", json!("xp"), Value::Null),
        ];
        mc.adopt_free(&nodes).unwrap();
        assert!(chora_kit::give::Windows::has(&mc, "s1"));
        assert!(chora_kit::give::Windows::has(&mc, "s2"));
        assert!(!chora_kit::give::Windows::has(&mc, "s3"));
        assert!(!chora_kit::give::Windows::has(&mc, "s4"));
        assert_eq!(owner(&core.model(), "s1").as_deref(), Some(NAME));
        assert_eq!(owner(&core.model(), "s3"), None);
        let scenes: Vec<_> = (mc.windows.iter())
            .map(|(scene, surface)| (scene.clone(), surface.clone()))
            .collect();
        let mut scenes = scenes;
        scenes.sort();
        assert_eq!(
            scenes,
            [
                ("probe".into(), "s1".to_owned()),
                ("probe-s2".into(), "s2".to_owned())
            ]
        );
    }

    /// The scenes of windows come at the end of the order, in the order
    /// of their windows, and go with them; the number of a scene that
    /// went is free again.
    #[test]
    fn the_order_of_the_scenes_follows_the_windows() {
        let core = FakeCore::start();
        let mut mc = testing::mc_on(&core, "order", [800, 600]);
        let first = mc.order();
        let mut app = |id: &str, app_id: &str| {
            surface(&core.model(), id);
            let event = Event::SurfaceCreated {
                surface: id.into(),
                app_id: Some(app_id.into()),
                title: None,
                decoration: None,
                to: None,
                pid: None,
                token: None,
            };
            mc.event(event).unwrap();
        };
        app("s1", "foot");
        app("s2", "firefox");
        app("s3", "foot");
        let opened = mc.order();
        assert_eq!(opened[..first.len()], first[..]);
        assert_eq!(opened[first.len()..], ["foot", "firefox", "foot-s3"]);
        let closed = Event::SurfaceDestroyed {
            surface: "s2".into(),
        };
        mc.event(closed).unwrap();
        let after = mc.order();
        assert_eq!(after[first.len()..], ["foot", "foot-s3"]);
        let number = mc.number("foot-s3");
        assert_eq!(
            number,
            (first.len() + 2).to_string(),
            "the place of firefox is free"
        );
    }
}
