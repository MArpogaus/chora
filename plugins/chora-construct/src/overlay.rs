//! HUD glue: drawing the list and the help, styling, text measure and
//! restyle.

use super::{
    BTreeMap, ClientError, Command, Construct, LIST, LIST_LINE, LIST_ROW, LIST_ROWS, Look, NAME,
    NodeKind, PropsUpdate, Query, VOICE, Value, create, hud, json,
};

impl Construct {
    /// The window list as a glass panel in the middle of the output
    /// (docs/construct-style.md): the label `WINDOWS` or `GIVE TO`, then
    /// one row per window with its name and, as a note in data, its owner
    /// and the place of its pane. An accent mark shows the selected row.
    /// `open` fades the panel in; a new selection only makes the rows anew.
    pub(crate) fn draw_list(&mut self, open: bool) -> Result<(), ClientError> {
        let Some(sel) = self.list.as_ref().map(|l| l.sel) else {
            return Ok(());
        };
        let rows = self.rows()?;
        let sel = sel.min(rows.len().saturating_sub(1));
        let giving = self.list.as_mut().is_some_and(|l| {
            l.sel = sel;
            l.give.is_some()
        });
        let unit = self.settings.value.style.unit;
        let first = (sel + 1).saturating_sub(LIST_ROWS);
        let shown = rows.len().saturating_sub(first).clamp(1, LIST_ROWS);
        let size = hud::list_size(unit, shown);
        let place = [
            ((self.lens.w - size[0]) / 2.0).round(),
            ((self.lens.h - size[1]) / 2.0).round(),
        ];
        let label = if giving { "GIVE TO" } else { "WINDOWS" };
        let mut commands = vec![
            Command::Remove {
                id: hud::LIST_ROWS_GROUP.into(),
                to: None,
            },
            create(
                hud::LIST_ROWS_GROUP,
                NodeKind::Group,
                LIST,
                PropsUpdate::default(),
            ),
        ];
        let top = hud::list_top(unit);
        commands.extend(self.fitted(|s| {
            let look = s.styled();
            let mut commands = look.list(size, place, label, open);
            if rows.is_empty() {
                let none = if giving {
                    "no other window manager runs"
                } else {
                    "no windows"
                };
                commands.extend(look.row(&format!("{LIST_ROW}none"), top, none, "", false));
            }
            for (k, row) in rows.iter().enumerate().skip(first).take(LIST_ROWS) {
                #[allow(clippy::cast_precision_loss, reason = "a dozen rows")]
                let y = top + LIST_LINE * (k - first) as f64;
                let id = format!("{LIST_ROW}{k}");
                commands.extend(look.row(&id, y, &row.label, &row.note, k == sel));
            }
            commands
        })?);
        commands.extend(self.label_commands());
        if self
            .rt
            .conn
            .query(&format!("id:{}", hud::LIST_ROWS_GROUP))?
            .is_empty()
        {
            commands.remove(0);
        }
        self.rt.conn.apply(commands)
    }

    /// What the elements over the space draw with.
    pub(crate) fn styled(&self) -> Look<'_> {
        let style = &self.settings.value.style;
        Look {
            style,
            colors: style.colors(self.settings.value.theme),
            plain: self.glass.plain,
            shader: &self.glass_shader,
            metrics: &self.metrics,
        }
    }

    /// Builds the elements over the space with the sizes of their texts
    /// as the core lays them out: a build that asks for texts not yet
    /// measured runs again after the core measured them.
    pub(crate) fn fitted<T>(
        &mut self,
        mut build: impl FnMut(&mut Self) -> T,
    ) -> Result<T, ClientError> {
        let mut out = build(self);
        for _ in 0..3 {
            let misses = self.metrics.take_misses();
            if misses.is_empty() {
                break;
            }
            self.measure(misses)?;
            out = build(self);
        }
        Ok(out)
    }

    /// Measures texts with probe text nodes in a hidden group: their
    /// rects are the sizes that the core lays them out at.
    pub(crate) fn measure(&mut self, probes: Vec<hud::Probe>) -> Result<(), ClientError> {
        const PROBE: &str = "construct-probe";
        let meta = BTreeMap::from([("probe".to_owned(), "1".to_owned())]);
        let mut commands = vec![create(
            PROBE,
            NodeKind::Group,
            NAME,
            PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        )];
        let look = self.styled();
        commands.extend(probes.iter().enumerate().map(|(k, probe)| {
            create(
                &format!("{PROBE}-{k}"),
                NodeKind::Text,
                PROBE,
                PropsUpdate {
                    meta: Some(meta.clone()),
                    ..look.probe_props(probe)
                },
            )
        }));
        self.rt.conn.apply(commands)?;
        let nodes: Result<Vec<Value>, ClientError> = (self.rt.conn).call(
            "scene.query",
            Query::new(format!("owner:{NAME} meta:probe=1")),
        );
        self.rt.conn.apply(vec![Command::Remove {
            id: PROBE.into(),
            to: None,
        }])?;
        let sizes: BTreeMap<String, [f64; 2]> = (nodes?.iter())
            .filter_map(|n| {
                let r = n["rects"].as_object()?.values().next()?.as_array()?;
                let size = [r.get(2)?.as_f64()?, r.get(3)?.as_f64()?];
                Some((n["id"].as_str()?.to_owned(), size))
            })
            .collect();
        for (k, probe) in probes.into_iter().enumerate() {
            if let Some(size) = sizes.get(&format!("{PROBE}-{k}")) {
                self.metrics.insert(probe, *size);
            }
        }
        Ok(())
    }

    /// The elements anew in the style of the settings, which changed.
    pub(crate) fn restyle(&mut self) -> Result<(), ClientError> {
        if !self.active || self.output.is_none() {
            return Ok(());
        }
        let mut commands: Vec<Command> = [hud::LABEL, hud::HUD, LIST]
            .map(|id| Command::Remove {
                id: id.into(),
                to: None,
            })
            .into();
        commands.extend(self.styled().groups(NAME));
        self.rt.conn.try_apply(commands)?;
        self.labelled = None;
        self.draw_hud()?;
        self.draw_ask()?;
        if self.list.is_some() {
            self.draw_list(true)?;
        }
        self.draw_help(false)
    }

    /// Shows or hides the help panel.
    pub(crate) fn toggle_help(&mut self) -> Result<(), ClientError> {
        self.help = !self.help;
        if self.help {
            return self.draw_help(true);
        }
        self.rt
            .conn
            .try_apply(vec![hud::fade(hud::HELP, 0.0, hud::FADE_OUT)])?;
        Ok(())
    }

    /// The help panel anew, with the bindings of the settings, while it
    /// shows.
    pub(crate) fn draw_help(&mut self, fresh: bool) -> Result<(), ClientError> {
        if !self.help || !self.active {
            return Ok(());
        }
        let rows = self.settings.value.help(self.cube.is_some());
        let out = [self.lens.w, self.lens.h];
        let mut commands = vec![Command::Remove {
            id: hud::HELP.into(),
            to: None,
        }];
        if self.rt.conn.query(&format!("id:{}", hud::HELP))?.is_empty() {
            commands.clear();
        }
        commands.extend(self.fitted(|s| s.styled().help(&rows, out, fresh))?);
        self.rt.conn.apply(commands)
    }

    /// Asks the voice agent to hide its dot while the HUD shows its
    /// state, once per connection of the agent (docs/voice.md, 10).
    pub(crate) fn claim_indicator(&mut self, last: bool) {
        if !self.settings.value.draws_voice() || !self.active {
            return;
        }
        let conn = &mut self.rt.conn;
        self.claim.claim(last, || {
            let call = json!({"client": VOICE, "method": "indicator.claim", "params": {}});
            conn.call::<_, Value>("methods.call", call).is_ok()
        });
    }
}
