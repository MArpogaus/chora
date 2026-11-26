use chora_kit::wm::{Book, Change, Created, Origin, Phase, Why, focus::Policy};
use chora_scene::Event;

use super::{
    ClientError, Command, Easing, FIT_MS, Mc, NAME, NodeKind, PARKED, PropsUpdate, Query, Scene,
    Target, ToplevelState, Value, create, name_of, regions_on_top, scene_id, wait,
};

/// The focus after a window of mc goes: the window of the scene that
/// shows, unless it went; else the page takes the keyboard.
impl Policy for Mc {
    fn focus_after(&self, gone: &str, _why: &Why) -> Option<String> {
        let name = self.current.as_ref()?;
        self.windows.get(name).filter(|w| *w != gone).cloned()
    }
}

/// What mc knows of the app of a surface that the book proposes.
pub(crate) enum Named {
    /// The event of the core said it: the app id, if it had one, and the
    /// activation token that mapped the window, if one did.
    Is {
        app_id: Option<String>,
        token: Option<String>,
    },
    /// The core says it: the surface lost its parent.
    Ask,
}

impl Mc {
    /// An event of the life of a window: the book decides which windows
    /// are mc's, and mc settles what it proposes.
    pub(crate) fn lifecycle(&mut self, event: &Event) -> Result<(), ClientError> {
        let token = Created::of(event).and_then(|created| created.token);
        let named = Book::announced_app(event).map_or(Named::Ask, |app_id| Named::Is {
            app_id: app_id.map(str::to_owned),
            token,
        });
        let changes = self.book.apply(event);
        self.settle(changes, &named)
    }

    /// Does what the book says, in order. A detached window of mc leaves
    /// no data: the proposal that follows puts it back. A window given in
    /// while a window or the desktop of another look holds the keyboard
    /// comes without it ([`Focus::takes_given`], D119): its scene shows,
    /// and mc sends no focus.
    ///
    /// [`Focus::takes_given`]: chora_kit::wm::focus::Focus::takes_given
    /// A new window that the core reserved for mc (mc holds the claim of
    /// its output, `to` names it) is `new` in the book and keeps the token
    /// of its `surface.created`, which finds the scene that started it
    /// (mc-42).
    pub(crate) fn settle(
        &mut self,
        changes: Vec<Change>,
        named: &Named,
    ) -> Result<(), ClientError> {
        for change in changes {
            match change {
                Change::Adopt { id, origin } => {
                    let token = match (&origin, named) {
                        (Origin::New { token: Some(token) }, _) => Some(token.as_str()),
                        (Origin::New { .. } | Origin::Given, Named::Is { token, .. }) => {
                            token.as_deref()
                        }
                        (Origin::New { token: None } | Origin::Given, Named::Ask)
                        | (Origin::Kept, _) => None,
                    };
                    let keys =
                        origin != Origin::Given || self.focus.takes_given(NAME, |s| self.holds(s));
                    let next = match self.take(&id, named, token, keys) {
                        Ok(true) => self.book.confirm_keys(&id, keys),
                        Ok(false) => self.book.refused(&id),
                        Err(err) => {
                            self.book.refused(&id);
                            return Err(err);
                        }
                    };
                    self.settle(next, named)?;
                }
                Change::Forget {
                    why: Why::Detached | Why::Given(_),
                    ..
                }
                | Change::Outputs => {}
                Change::Forget { id, .. } => self.forget(&id)?,
                Change::Phase { id, phase } => self.tell_phase(&id, phase),
                Change::Apply(commands) => self.apply(commands)?,
            }
        }
        Ok(())
    }

    /// Emits `mc.window` for a phase. A refused emit only logs.
    pub(crate) fn tell_phase(&mut self, surface: &str, phase: Phase) {
        let emit = self.book.phase_event(surface, phase, None);
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", emit) {
            tracing::warn!("mc.window: {err}");
        }
    }

    /// Whether mc holds the surface: in a scene, in a region or on its
    /// way into one.
    fn holds(&self, surface: &str) -> bool {
        (self.windows.values())
            .chain(self.docks.values())
            .chain(self.fitting.keys())
            .any(|s| s == surface)
    }

    /// Takes a surface that the book proposes: one that mc holds goes
    /// back where it was (the detach policy of mc), another is adopted,
    /// with the keyboard only with `keys`.
    /// False when the core refused or mc is not built yet.
    fn take(
        &mut self,
        surface: &str,
        named: &Named,
        token: Option<&str>,
        keys: bool,
    ) -> Result<bool, ClientError> {
        if self.holds(surface) {
            self.put_back(surface)?;
            return Ok(true);
        }
        let app_id = match named {
            Named::Is { app_id, .. } => app_id.clone(),
            Named::Ask => {
                let nodes: Vec<Value> =
                    (self.rt.conn).call("scene.query", Query::new(format!("id:{surface}")))?;
                let found = nodes.first().and_then(|n| n["surface"]["app_id"].as_str());
                found.map(str::to_owned)
            }
        };
        self.adopt(surface, app_id.as_deref(), token, keys)
    }

    /// Takes a new window into the scene that started it, or into a new
    /// scene of its own, and shows it. True when it is mc's now.
    /// The start adopts the free surfaces that events then announce too.
    /// A new window takes the keyboard, except one of app.open, or one
    /// without `keys`.
    /// Another window manager took it first; the new scene goes.
    /// The overview lists the new scene.
    pub(crate) fn adopt(
        &mut self,
        surface: &str,
        app_id: Option<&str>,
        token: Option<&str>,
        keys: bool,
    ) -> Result<bool, ClientError> {
        if !self.built {
            return Ok(false);
        }
        let started = token.and_then(|t| self.starting.remove(t));
        let user = !token.is_some_and(|t| self.quiet.remove(t)) && keys;
        if let Some(Target::Dock(content)) = &started
            && !self.docks.contains_key(content)
        {
            self.adopt_dock(surface, content)?;
            return Ok(true);
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
        if let Some(err) = self.try_apply(&commands)? {
            tracing::warn!("{surface}: {err}");
            if new {
                self.scenes.remove(&name);
                self.transient.remove(&name);
                self.appeared.retain(|n| *n != name);
            }
            return Ok(false);
        }
        self.windows.insert(name.clone(), surface.into());
        self.plain_turn = self.take_plain_start(&name);
        if let Err(err) = self.show(&name, user) {
            chora_kit::runtime::report(crate::NAME, &format_args!("{}: {}", name, err.message));
        }
        self.plain_turn = false;
        if new {
            self.renumber()?;
        }
        Ok(true)
    }

    /// Whether a plain turn started the program of scene `name` and that
    /// scene shows: the start counts once (mc-42). A window of another
    /// scene does not leave the start: `show` of that scene clears it.
    /// The name match stays as a guard of its own, because the kept name
    /// is what keeps `plain_start` an `Option` and the struct at three
    /// bools, the limit of clippy.
    fn take_plain_start(&mut self, name: &str) -> bool {
        let own = self.plain_start.as_deref() == Some(name);
        if own {
            self.plain_start = None;
        }
        own && self.current.as_deref() == Some(name)
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

    /// A surface of mc lost its parent and its owner, for example after a
    /// hotplug: it goes back to its scene or region. The plugin still
    /// counts it as its own, and a later move of it failed with "belongs
    /// to nobody" (W11).
    fn put_back(&mut self, surface: &str) -> Result<(), ClientError> {
        let scene = (self.windows.iter()).find(|(_, s)| *s == surface);
        if let Some(name) = scene.map(|(n, _)| n.clone()) {
            let commands = self.place_window(&name, surface, true);
            return self.apply(commands);
        }
        self.apply(vec![Command::Attach {
            surface: surface.into(),
            parent: PARKED.into(),
        }])?;
        if self.docks.values().any(|s| s == surface) && self.rendered.is_some() {
            self.rendered = Some(Vec::new());
        }
        self.render()
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
            if let Some(next) = self.focus.after(surface, &Why::Destroyed, self) {
                self.focus(next)?;
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

#[cfg(test)]
mod tests {
    use chora_kit::{
        give::{self, Give, Windows},
        runtime::Plugin,
        testing::{FakeCore, Script, Subject, created, created_by, owner, surface},
    };
    use chora_scene::{Command, Event, NodeKind, PropsUpdate, ROOT};
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
                stay: false,
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
        assert_eq!(origin("s2"), "new");
        assert!(!state.to_string().contains("t1"), "no token: {state}");
        assert_eq!(state["open"], serde_json::json!([]), "no key list open");
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

    /// mc on 0,0 800x600 beside the look `other` on the output right of
    /// it, with the windows `apps` of the apps of these ids, in order (the
    /// last shows and holds the keyboard); and the log of the commands.
    fn beside(
        tag: &str,
        other: &str,
        apps: &[(&str, &str)],
    ) -> (FakeCore, Mc, chora_kit::testing::Log) {
        let core = FakeCore::start().selecting();
        let mut mc = testing::mc_on(&core, tag, [800, 600]);
        let mut receiver = core.connect(other);
        receiver
            .apply(vec![Command::Create {
                id: other.into(),
                kind: NodeKind::Group,
                parent: ROOT.into(),
                props: PropsUpdate {
                    claim: Some(Some([800.0, 0.0, 800.0, 600.0])),
                    ..PropsUpdate::default()
                },
            }])
            .unwrap();
        for (id, app_id) in apps {
            surface(&core.model(), id);
            mc.event(created_by(id, app_id, None, None)).unwrap();
        }
        let methods = core.record(|method, params| {
            (method == "scene.apply").then(|| params["commands"].to_string())
        });
        (core, mc, methods)
    }

    /// The commands that mc sent from the call `from` on.
    fn sent_from(methods: &chora_kit::testing::Log, from: usize) -> Vec<Value> {
        let calls = methods.lock().unwrap().clone();
        (calls[from..].iter())
            .flat_map(|c| serde_json::from_str::<Vec<Value>>(c).unwrap())
            .collect()
    }

    /// The holder that `state.get` of mc tells.
    fn holder(mc: &Mc) -> (Option<String>, Option<String>) {
        let held = mc.focus.state(NAME, |s| Windows::has(mc, s));
        (held.holder, held.surface)
    }

    /// rtf2-1, D119: the give key Super+Alt+1 in mc sends the window to
    /// tiling on another output; the keyboard stays with mc where the
    /// person looks, before the window goes: on the window of the scene
    /// that shows next, or without one on the root node of mc.
    #[test]
    fn the_give_key_keeps_the_keyboard_with_mc() {
        type Apps<'a> = &'a [(&'a str, &'a str)];
        let cases: [(Apps, &str, Value); 2] = [
            (
                &[("s1", "foot"), ("s2", "firefox")],
                "s2",
                json!({"focus": {"surface": "s1"}}),
            ),
            (
                &[("s2", "firefox")],
                "s2",
                json!({"focus_node": {"node": NAME}}),
            ),
        ];
        for (n, (apps, given, keep)) in cases.into_iter().enumerate() {
            let (_core, mut mc, methods) = beside(&format!("key-keep-{n}"), "tiling", apps);
            assert_eq!(mc.focus.surface.as_deref(), Some(given), "case {n}");
            let from = methods.lock().unwrap().len();
            give::focused(&mut mc, "tiling").unwrap();
            let sent: Vec<Value> = (sent_from(&methods, from).into_iter())
                .filter(|c| {
                    ["focus", "focus_node", "remove"]
                        .iter()
                        .any(|k| c.get(k).is_some())
                })
                .collect();
            let at = |wanted: &dyn Fn(&Value) -> bool| sent.iter().position(wanted);
            let gone = at(&|c| c["remove"]["id"] == given && c["remove"]["to"] == "tiling");
            let gone = gone.expect("the give");
            let kept = at(&|c| *c == keep);
            assert!(
                kept.is_some_and(|k| k < gone),
                "case {n}: {keep} before {given} goes: {sent:?}"
            );
            assert!(!Windows::has(&mc, given), "case {n}: the window went");
            let held = holder(&mc);
            assert_eq!(held.0.as_deref(), Some(NAME), "case {n}: holder {held:?}");
        }
    }

    /// rtf2-1: a give of the person that the core refuses leaves the
    /// window with mc, and it takes the keyboard back.
    #[test]
    fn a_refused_give_key_gives_the_keyboard_back() {
        let (_core, mut mc, methods) = beside("key-back", "tiling", &[("s1", "foot")]);
        let from = methods.lock().unwrap().len();
        assert!(
            give::focused(&mut mc, "gone").is_err(),
            "gone holds no claim"
        );
        assert!(Windows::has(&mc, "s1"), "the window stays");
        let sent: Vec<Value> = (sent_from(&methods, from).into_iter())
            .filter(|c| c.get("focus").is_some())
            .collect();
        assert_eq!(
            sent.last(),
            Some(&json!({"focus": {"surface": "s1"}})),
            "s1 takes the keyboard back: {sent:?}"
        );
        assert_eq!(holder(&mc), (Some(NAME.into()), Some("s1".into())));
    }

    /// rtf2-1: a window that tiling gives to mc while a window of tiling
    /// or the desktop of tiling holds the keyboard (the person gave it
    /// with Super+Alt+3 on another output) comes without it: mc shows it
    /// and sends no focus for it.
    #[test]
    fn a_window_given_in_while_tiling_holds_the_keyboard_comes_without_it() {
        let focus = chora_kit::testing::focus_changed;
        let desk = Event::FocusNode {
            node: Some("tiling".into()),
            owner: Some("tiling".into()),
        };
        let held = [
            vec![focus(Some("t1"))],
            vec![focus(Some("t1")), focus(None), desk],
        ];
        for (n, held) in held.into_iter().enumerate() {
            let (core, mut mc, methods) = beside(&format!("given-in-{n}"), "tiling", &[]);
            for event in held {
                mc.event(event).unwrap();
            }
            surface(&core.model(), "s5");
            core.model().lock().unwrap().scene.reserve("s5", Some(NAME));
            let from = methods.lock().unwrap().len();
            mc.event(Event::SurfaceDetached {
                surface: "s5".into(),
            })
            .unwrap();
            assert!(Windows::has(&mc, "s5"), "case {n}: mc took the window");
            let keys: Vec<Value> = (sent_from(&methods, from).into_iter())
                .filter(|c| c.get("focus").is_some() || c.get("focus_node").is_some())
                .collect();
            assert!(
                keys.is_empty(),
                "case {n}: mc took the keyboard for a window given in: {keys:?}"
            );
            assert_eq!(holder(&mc).0, None, "case {n}: mc holds the keyboard");
        }
    }

    /// A window given to mc while nobody holds the keyboard takes it, as
    /// before (D119).
    #[test]
    fn a_window_given_in_while_nobody_holds_the_keyboard_takes_it() {
        let (core, mut mc, methods) = beside("given-free", "tiling", &[]);
        mc.event(chora_kit::testing::focus_changed(None)).unwrap();
        surface(&core.model(), "s5");
        core.model().lock().unwrap().scene.reserve("s5", Some(NAME));
        let from = methods.lock().unwrap().len();
        mc.event(Event::SurfaceDetached {
            surface: "s5".into(),
        })
        .unwrap();
        let sent = sent_from(&methods, from);
        assert!(
            sent.iter()
                .any(|c| *c == json!({"focus": {"surface": "s5"}})),
            "s5 takes the keyboard"
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
            let event = created_by(id, app_id, None, None);
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
