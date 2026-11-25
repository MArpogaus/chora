use chora_scene::Order;

use super::{
    BTreeMap, CLOCK, ClientError, Command, Easing, FADE, HEAD, Layout, Mc, NAME, NodeKind, PARKED,
    Panel, PropsUpdate, REGIONS, ROOT, Scene, Style, THEME_FADE, TICK, Type, VOLUME, Value, WAIT,
    create, fonts, manage, page_id, pane_tag, panel_id, panels_id, region, regions_on_top,
    scene_id, translate, volume_wait, wait, wheel_id,
};

impl Mc {
    pub(crate) fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if commands.is_empty() {
            return Ok(());
        }
        self.note_writes(&commands);
        self.rt.conn.apply(commands)
    }

    /// Applies a transaction that the core may refuse. The writes and
    /// fades that it starts are noted once the core accepts it, so that
    /// one tracker sees every path.
    pub(crate) fn try_apply(
        &mut self,
        commands: &[Command],
    ) -> Result<Option<super::RpcError>, ClientError> {
        let refused = self.rt.conn.try_apply(commands.to_vec())?;
        if refused.is_none() {
            self.note_writes(commands);
        }
        Ok(refused)
    }

    /// Creates the group of a scene with its panels and its window.
    pub(crate) fn scene_commands(&self, name: &str) -> Vec<Command> {
        let shown = self.current.as_deref() == Some(name);
        let mut commands = vec![create(
            &scene_id(name),
            NodeKind::Group,
            NAME,
            PropsUpdate {
                visible: Some(shown),
                opacity: Some(if shown { 1.0 } else { 0.0 }),
                ..PropsUpdate::default()
            },
        )];
        commands.extend(self.panels(name));
        if let Some(surface) = self.windows.get(name) {
            commands.extend(self.place_window(name, surface, false));
        }
        commands
    }

    /// Applies the base of the plugin and its scenes in one transaction.
    /// One scene that the core refuses must not blank the others: then
    /// each goes on its own, and a refused one is left out with a log line
    /// until its file changes.
    pub(crate) fn apply_scenes(
        &mut self,
        base: Vec<Command>,
        scenes: Vec<(String, Vec<Command>)>,
    ) -> Result<(), ClientError> {
        let all: Vec<Command> = (base.iter())
            .chain(scenes.iter().flat_map(|(_, c)| c))
            .cloned()
            .collect();
        let Some(err) = self.try_apply(&all)? else {
            return Ok(());
        };
        tracing::warn!("{err}; the scenes go one by one");
        self.apply(base)?;
        for (name, commands) in scenes {
            if let Some(err) = self.try_apply(&commands)? {
                tracing::warn!("scene {name}: {err}; left out");
                self.scenes.remove(&name);
                self.windows.remove(&name);
            }
        }
        Ok(())
    }

    /// The root group over the output `[x, y, w, h]`, in the background
    /// order (D89): a window of another window manager that is dragged
    /// over the output shows above the opaque page, whatever the build
    /// order of the plugins.
    fn root(&self, [x, y, w, h]: [f64; 4], work: [f64; 4]) -> Command {
        create(
            NAME,
            NodeKind::Group,
            ROOT,
            PropsUpdate {
                transform: Some(translate(x, y)),
                claim: self.manage_windows.then_some(Some(work)),
                size: Some([w, h]),
                meta: Some(pane_tag(self.current.as_deref())),
                order: Some(Order::Background),
                ..PropsUpdate::default()
            },
        )
    }

    /// Builds all nodes again, for example after a change of the theme or
    /// of the output. The windows wait under the root meanwhile, all in one
    /// transaction, so nothing flickers.
    /// mc as a window manager claims its output: only a window
    /// manager with a claim receives windows (D55).
    /// Other plugins find mc by this tag and show the whole
    /// output, the regions with the page, through its node
    /// texture, as chora-construct does; it names the scene
    /// that shows. The page of every scene has its own tag.
    /// A build ends the wait of the old node.
    pub(crate) fn build(&mut self) -> Result<(), ClientError> {
        let Some(output) = self.output.clone() else {
            return Ok(());
        };
        let mut commands = if self.built {
            self.unbuild()
        } else {
            Vec::new()
        };
        self.note_writes(&commands);
        self.finish_typing();
        let [w, h] = output.size.map(f64::from);
        let [x, y] = output.position.map(f64::from);
        let work = self.work_area([x, y, w, h]);
        commands.push(self.root([x, y, w, h], work));
        for id in [TICK, CLOCK, WAIT, VOLUME] {
            commands.push(create(id, NodeKind::Group, NAME, PropsUpdate::default()));
        }
        commands.extend((!self.launched.is_empty()).then(wait));
        commands.extend(self.tick_clocks().into_iter().chain([volume_wait()]));
        commands.push(create(
            "mc-bg",
            NodeKind::Rect,
            NAME,
            PropsUpdate {
                size: Some([w, h]),
                color: Some(self.settings.value.background()),
                ..PropsUpdate::default()
            },
        ));
        commands.extend(self.parked());
        if let Some(panel) = self.overview_panel()
            && let Some(scene) = self.scenes.get_mut(manage::OVERVIEW)
        {
            scene.panels.retain(|p| p.id != panel.id);
            scene.panels.push(panel);
        }
        let scenes = (self.scenes.keys())
            .map(|name| (name.clone(), self.scene_commands(name)))
            .collect();
        self.apply_scenes(commands, scenes)?;
        self.apply(vec![regions_on_top()])?;
        self.built = true;
        self.rendered = None;
        self.left = None;
        self.sigs.clear();
        self.shape = Some(self.shape_of());
        self.themed = Some(self.theme_of());
        let names: Vec<String> = self.scenes.keys().cloned().collect();
        names.iter().try_for_each(|name| self.align(name))?;
        self.autostart();
        self.render()
    }

    /// The group of the regions, and the docked apps hidden in it, not on
    /// the root, until the render.
    pub(crate) fn parked(&self) -> Vec<Command> {
        let hidden = PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        };
        let mut commands = vec![
            create(REGIONS, NodeKind::Group, NAME, PropsUpdate::default()),
            create(PARKED, NodeKind::Group, REGIONS, hidden),
        ];
        let docks = self.docks.values().chain(self.fitting.keys());
        commands.extend(docks.map(|surface| Command::Move {
            id: surface.clone(),
            parent: Some(PARKED.into()),
            index: None,
        }));
        commands
    }

    /// The settings without the theme and the write effect, which new
    /// text reads when it writes: no node that stands depends on them.
    pub(crate) fn shape_of(&self) -> Value {
        let mut shape = serde_json::to_value(&self.settings.value).unwrap_or_default();
        if let Some(map) = shape.as_object_mut() {
            map.remove("theme");
            map.remove("write");
        }
        shape
    }

    /// The theme of the settings.
    pub(crate) fn theme_of(&self) -> Value {
        serde_json::to_value(&self.settings.value)
            .ok()
            .and_then(|mut v| v.get_mut("theme").map(Value::take))
            .unwrap_or_default()
    }

    /// After a change of the theme alone: the nodes fade to their new
    /// colors and the regions cross-fade, instead of a build. False when
    /// the core refuses that, for example when a color of a rule is
    /// missing in one theme: then a build follows.
    /// The regions cross-fade in the same transaction, as slowly.
    pub(crate) fn recolor(&mut self) -> Result<bool, ClientError> {
        let background = PropsUpdate {
            color: Some(self.settings.value.background()),
            ..PropsUpdate::default()
        };
        let mut created = vec![create("mc-bg", NodeKind::Rect, NAME, background)];
        created.extend(
            self.scenes
                .keys()
                .flat_map(|name| self.scene_commands(name)),
        );
        let mut fades: Vec<Command> = (created.into_iter())
            .filter_map(|command| match command {
                Command::Create {
                    id,
                    props:
                        PropsUpdate {
                            color: Some(color), ..
                        },
                    ..
                } => Some(Command::Animate {
                    id,
                    props: PropsUpdate {
                        color: Some(color),
                        ..PropsUpdate::default()
                    },
                    duration: THEME_FADE,
                    easing: Easing::EaseInOut,
                }),
                _ => None,
            })
            .collect();
        self.rendered = Some(Vec::new());
        self.fade = THEME_FADE;
        let regions = self.regions();
        self.fade = FADE;
        fades.extend(regions?);
        if let Some(err) = self.try_apply(&fades)? {
            tracing::warn!("{err}; the theme builds anew");
            return Ok(false);
        }
        Ok(true)
    }

    /// Removes all nodes; the windows wait under the root.
    pub(crate) fn unbuild(&self) -> Vec<Command> {
        let surfaces = (self.windows.values())
            .chain(self.docks.values())
            .chain(self.fitting.keys());
        let moves = surfaces.map(|surface| Command::Move {
            id: surface.clone(),
            parent: Some(ROOT.into()),
            index: None,
        });
        let removes = [NAME].map(|id| Command::Remove {
            id: id.into(),
            to: None,
        });
        moves.chain(removes).collect()
    }

    /// Creates the head and the panels of a scene, under a group of their
    /// own that the next call replaces.
    /// A page between the header and the footer that clips the panels
    /// and takes the wheel; the panels move up in it as it scrolls. It ends
    /// on a line of the grid over the card of chora-notify and the footer
    /// line (rpx1-11, rpx1-12).
    /// Other plugins find the page by this tag and show it
    /// through its node texture, as chora-construct does.
    pub(crate) fn panels(&self, name: &str) -> Vec<Command> {
        let Some(scene) = self.scenes.get(name) else {
            return Vec::new();
        };
        let [w, h] = (self.output.as_ref()).map_or([0.0; 2], |o| o.size.map(f64::from));
        let b = self.settings.value.scale.baseline;
        let page = page_id(name);
        let scroll = self.scroll.get(name).copied().unwrap_or_default();
        let bottom = self
            .geometry(name)
            .map_or(h - 5.0 * b, |g| g.text[1] + g.text[3]);
        let mut commands = vec![
            create(
                &page,
                NodeKind::Group,
                &scene_id(name),
                PropsUpdate {
                    transform: Some(translate(0.0, 2.0 * b)),
                    size: Some([w, (bottom - 2.0 * b).max(b)]),
                    clip: Some(Some(chora_scene::Clip { radius: 0.0 })),
                    meta: Some(BTreeMap::from([("pane".into(), name.into())])),
                    ..PropsUpdate::default()
                },
            ),
            region::rect(&wheel_id(name), &page, [0.0, 0.0, w, h], [0.0; 4]),
            create(
                &panels_id(name),
                NodeKind::Group,
                &page,
                PropsUpdate {
                    transform: Some(translate(0.0, -2.0 * b - scroll)),
                    ..PropsUpdate::default()
                },
            ),
        ];
        commands.extend(self.chapter(name, scene));
        for panel in &scene.panels {
            commands.extend(self.panel(name, panel));
            commands.extend(self.rule(name, panel));
        }
        commands
    }

    /// The hairline of a panel, in the middle of the space before its
    /// margin; outside the panel, which `panel.set` replaces.
    /// A rule shows beside a panel with text, not at the left edge of the
    /// text block; else it is clear, so that `replace_panel` always
    /// finds one to replace.
    pub(crate) fn rule(&self, name: &str, panel: &Panel) -> Option<Command> {
        let ([x, y, _, h], geo) = (self.place(name, panel)?, self.geometry(name)?);
        let shown = x > geo.text[0] && !panel.is_empty();
        let color = (panel.rule.as_ref().filter(|_| shown))
            .and_then(|c| self.settings.value.colors().get(c))
            .copied()
            .unwrap_or_default();
        let before = if panel.columns().is_some() {
            geo.note + 1.5 * geo.gap
        } else {
            f64::midpoint(geo.note, 2.0 * geo.gap)
        };
        let line = [(x - before).round(), y, 1.0, h];
        let id = format!("{}-rule", panel_id(name, &panel.id));
        Some(region::rect(&id, &panels_id(name), line, color))
    }

    /// A style of the settings, changed by a scene; a change that fails
    /// logs and leaves the style as it is.
    #[allow(clippy::expect_used, reason = "the settings check their styles")]
    pub(crate) fn ty(&self, name: &str, local: Option<&Style>) -> Type {
        let config = &self.settings.value;
        config
            .resolve(name, local)
            .or_else(|err| {
                tracing::warn!("{err}");
                config.resolve(name, None)
            })
            .expect("the settings check their styles")
    }

    /// Whether a scene has a head: a scene of panels or of an app.
    pub(crate) fn has_chapter(scene: &Scene) -> bool {
        scene.app.is_some() || !scene.panels.is_empty()
    }

    /// The layout of the head of a scene: its own, else the settings.
    pub(crate) fn layout(&self, scene: &Scene) -> Layout {
        (scene.chapter.layout).unwrap_or(self.settings.value.chapter.layout)
    }

    /// The height of the head of a chapter, with the line under it.
    pub(crate) fn chapter_height(&self, scene: &Scene) -> f64 {
        let b = self.settings.value.scale.baseline;
        match self.layout(scene) {
            Layout::Number => {
                let line = |name| self.ty(name, None).line;
                line("chapter").max(line("chapter_number")) + b
            }
            Layout::None => 0.0,
        }
    }

    /// The phase of the one baseline grid: where the baseline of the
    /// margin type stands in a line of the grid. Every baseline of the
    /// page, of the head, the panels, the column and the footer, falls on
    /// a line of the grid plus this.
    pub fn phase(&self) -> f64 {
        let ink = self.ty("margin", None).ink();
        ink.top + ink.height
    }

    /// How far a text whose baseline would stand at `baseline` on the
    /// output moves to stand on the nearest baseline of the grid, in whole
    /// pixels; half a line moves down.
    pub fn to_grid(&self, baseline: f64) -> f64 {
        let b = self.settings.value.scale.baseline;
        let up = (self.phase() - baseline).rem_euclid(b);
        (if up > b / 2.0 { up - b } else { up }).round()
    }

    /// The place `rel` below the top of a block that stands at `top` on the
    /// output, moved so that the baseline of a text of type `ty` that
    /// starts there stands on the grid.
    pub(crate) fn lift(&self, top: f64, rel: f64, ty: &Type) -> f64 {
        let ink = ty.ink();
        rel + self.to_grid(top + rel + ink.top + ink.height)
    }

    /// The top of the head of a scene: its line, moved so that the
    /// baseline of the title stands on the grid.
    pub(crate) fn head_y(&self) -> f64 {
        let top = HEAD * self.settings.value.scale.baseline;
        let ink = self.ty("chapter", None).ink();
        top + self.to_grid(top + ink.top + ink.height)
    }

    /// The bar between the number and the title of a head at `x`: a
    /// hairline from the capitals of the title down to its baseline.
    pub(crate) fn bar(&self, x: f64) -> [f64; 4] {
        let title = self.ty("chapter", None);
        let ink = title.ink();
        [x, self.head_y() + ink.top, 1.0, ink.height]
    }

    /// The ink of a head part: how far its first ink starts after its box
    /// and its last ink ends before the end of it, in pixels.
    pub(crate) fn bearings(&self, style: &str, text: &str) -> [f64; 2] {
        let ty = self.ty(style, None);
        let (family, weight) = (ty.family(), ty.weight());
        let text = ty.props(text).text.unwrap_or_default();
        let em = family.and_then(|f| fonts::bearings(f, weight, &text));
        em.unwrap_or([0.0; 2]).map(|e| e * ty.size)
    }
}
