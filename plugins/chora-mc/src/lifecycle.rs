use crate::windows::Named;

use super::{
    Action, CLOCK, ClientError, Command, EVENTS, ExportMethod, HUD, Mc, NAME, PARKED, PropsUpdate,
    Query, Shaders, TICK, VOLUME, VOLUME_AGAIN, Value, WAIT, needs_claim, panel_id, volume_wait,
};

impl Mc {
    /// The end of an empty animation of a hidden node that times
    /// something.
    pub(crate) fn timer(&mut self, node: &str) -> Result<(), ClientError> {
        match node {
            TICK => self.type_next(),
            WAIT => self.look_at_launched(),
            CLOCK => {
                let tick = self.tick_clocks();
                self.apply(tick)?;
                self.render()
            }
            VOLUME => self.volume_tick(),
            PARKED => {
                let late: Vec<String> = self.fitting.keys().cloned().collect();
                late.iter().try_for_each(|surface| self.fit(surface))
            }
            _ => self.written(node),
        }
    }

    /// Takes the reading of the volume that the worker has, shows it when
    /// it changed, and asks for a new one now and then.
    pub(crate) fn volume_tick(&mut self) -> Result<(), ClientError> {
        self.volume_ticks += 1;
        if self.volume_ticks % VOLUME_AGAIN == 1 {
            self.volume.refresh();
        }
        self.apply(vec![volume_wait()])?;
        let reading = self.volume.take();
        if reading.is_some_and(|r| Some(r) != self.volume_reading) {
            self.volume_reading = reading;
            self.render()?;
        }
        Ok(())
    }

    /// The watchdog of the core removed the effects of the plugin. An
    /// update that was on its way then may have put a blur or a write
    /// effect back, so the effects of the panels go once more.
    pub(crate) fn effects_dropped(&mut self) -> Result<(), ClientError> {
        if self.shaders == Shaders::On {
            eprintln!(
                "chora-mc: frames were slow, the core dropped the effects; the panels \
                 stay plain until the settings change"
            );
        }
        self.shaders = Shaders::Dropped;
        let mut commands = Vec::new();
        for (name, scene) in &self.scenes {
            for panel in &scene.panels {
                let id = panel_id(name, &panel.id);
                let shaded = panel.blur.is_some() || self.writing.contains_key(&id);
                if shaded && self.place(name, panel).is_some() {
                    commands.push(Command::Update {
                        id,
                        props: PropsUpdate {
                            effects: Some(Vec::new()),
                            ..PropsUpdate::default()
                        },
                    });
                }
            }
        }
        self.writing.clear();
        self.apply(commands)
    }

    /// The write animation of a panel ended, and with it those of the
    /// same transaction: their effects go in one transaction, so that a
    /// page that rests costs no shader pass per panel.
    pub(crate) fn written(&mut self, node: &str) -> Result<(), ClientError> {
        let Some(batch) = self.writing.get(node).copied() else {
            return Ok(());
        };
        let mut commands = Vec::new();
        for (name, scene) in &self.scenes {
            for panel in &scene.panels {
                let id = panel_id(name, &panel.id);
                if self.writing.get(&id) == Some(&batch) {
                    commands.push(Command::Update {
                        id,
                        props: PropsUpdate {
                            effects: Some(self.blur_effect(panel).into_iter().collect()),
                            ..PropsUpdate::default()
                        },
                    });
                }
            }
        }
        self.writing.retain(|_, b| *b != batch);
        self.apply(commands)
    }

    /// Remembers the panels that the commands start to write.
    pub(crate) fn note_writes(&mut self, commands: &[Command]) {
        let batch = self.writing.values().max().map_or(0, |b| b + 1);
        for command in commands {
            if let Command::Animate { id, props, .. } = command
                && props.effects.is_some()
            {
                self.writing.insert(id.clone(), batch);
            }
        }
    }

    /// The nodes that the core kept over a restart: the windows in them
    /// become free and come back as scenes of their own.
    /// After the removal: it detaches the kept windows, and those
    /// events would adopt them a second time.
    pub(crate) fn start(&mut self, methods: Vec<ExportMethod>) -> Result<(), ClientError> {
        self.voiced_word.ask(&mut self.rt.conn);
        self.output = self.rt.outputs()?.into_iter().next();
        let kept: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new(format!("owner:{NAME}")))?;
        let old = kept
            .iter()
            .filter_map(|n| n["id"].as_str())
            .filter(|id| *id == NAME || *id == HUD)
            .map(|id| Command::Remove {
                id: id.into(),
                to: None,
            })
            .collect();
        self.apply(old)?;
        self.rt.announce(&EVENTS, methods)?;
        self.name_theme()?;
        self.grab_all()?;
        self.build()?;
        self.grab_claimed()?;
        let first = self.order().into_iter().next();
        let default = Some(self.settings.value.default_scene.clone())
            .filter(|v| self.scenes.contains_key(v))
            .or(first);
        if let Some(name) = default
            && let Err(err) = self.show(&name, false)
        {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        self.autostart();
        self.take_free_keyboard()?;
        let free: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new("kind:surface"))?;
        self.adopt_free(&free)
    }

    /// Adopts the free surfaces among `nodes`, and those that the core
    /// reserved for mc, as after a restart.
    pub(crate) fn adopt_free(&mut self, nodes: &[Value]) -> Result<(), ClientError> {
        for node in nodes {
            let Some(id) = node["id"].as_str().filter(|_| node["owner"].is_null()) else {
                continue;
            };
            let named = Named::Is(node["surface"]["app_id"].as_str().map(str::to_owned));
            let changes = self.book.claim(id, node["to"].as_str(), None);
            self.settle(changes, &named)?;
        }
        Ok(())
    }

    /// Grabs the bindings of the settings; one that another client holds
    /// only logs.
    #[allow(clippy::expect_used, reason = "the settings check their bindings")]
    pub(crate) fn grab_all(&mut self) -> Result<(), ClientError> {
        self.grab_some(|action| !needs_claim(action))?;
        if self.built {
            self.grab_claimed()?;
        }
        self.rt.give_keys(Some(|to| {
            Action::System(chora_kit::actions::System::Give(to.into()))
        }));
        self.rt
            .accept_injected(self.settings.value.common.test_accept_injected_keys);
        self.rt.grab_give_keys()
    }

    /// Grabs the bindings whose action `wanted` accepts.
    #[allow(clippy::expect_used, reason = "the settings check their bindings")]
    pub(crate) fn grab_some(
        &mut self,
        wanted: impl Fn(&Action) -> bool,
    ) -> Result<(), ClientError> {
        let bindings = self.settings.value.common.bindings.clone();
        for (text, action) in bindings.into_iter().filter(|(_, a)| wanted(a)) {
            let binding = text.parse().expect("the settings check their bindings");
            self.rt.grab(binding, action)?;
        }
        Ok(())
    }

    /// Grabs the `drag` and `volume` bindings, after the build that sets
    /// the claim: only window managers with a claim share a pointer button
    /// (D55) and grab a media key without a grant, so mc grabs none unless
    /// it manages windows.
    pub(crate) fn grab_claimed(&mut self) -> Result<(), ClientError> {
        if !self.manage_windows {
            return Ok(());
        }
        self.grab_some(needs_claim)
    }
}
