//! HUD glue: drawing the list and the help, styling, text measure and
//! restyle.

use super::{
    BTreeMap, ClientError, Command, Construct, Duration, Instant, LIST, LIST_ROWS, Look, NAME,
    NodeKind, PropsUpdate, Query, VOICE, Value, create, hud, json,
};

impl Construct {
    /// The window list over the scrim (construct HUD spec, section 5):
    /// the search line with the count, a row per window that its search
    /// leaves, with its thumbnail, its name and where it is, and the keys
    /// at the bottom. `open` fades it in; any change makes the body anew.
    #[allow(
        clippy::too_many_lines,
        reason = "the parts of the list, one after the other"
    )]
    pub(crate) fn draw_list(&mut self, open: bool) -> Result<(), ClientError> {
        use chora_kit::words::{Language, Word};
        let Some((sel, query, hovered, giving)) = self
            .list
            .as_ref()
            .map(|l| (l.sel, l.query.clone(), l.hovered, l.give.is_some()))
        else {
            return Ok(());
        };
        let rows = self.window_rows()?;
        let sel = sel.min(rows.len().saturating_sub(1));
        if let Some(list) = self.list.as_mut() {
            list.sel = sel;
        }
        let first = (sel + 1).saturating_sub(LIST_ROWS);
        let shown: Vec<crate::command::WindowRow> =
            rows.into_iter().skip(first).take(LIST_ROWS).collect();
        let language = Language::of_session();
        let count = rows_count(self.windows()?.len(), language, giving);
        let out = [self.lens.w, self.lens.h];
        let thumb = self.thumb_shader.clone();
        let mut commands = Vec::new();
        if !self
            .rt
            .conn
            .query(&format!("id:{}", crate::command::LIST_BODY))?
            .is_empty()
        {
            commands.push(Command::Remove {
                id: crate::command::LIST_BODY.into(),
                to: None,
            });
            commands.push(Command::Remove {
                id: crate::command::SCRIM.into(),
                to: None,
            });
        }
        commands.push(crate::panes::update(
            LIST,
            PropsUpdate {
                transform: Some(chora_scene::Transform::translate(0.0, 0.0, 0.0)),
                visible: Some(true),
                meta: Some(hud::list_meta("open")),
                ..PropsUpdate::default()
            },
        ));
        let footer = [
            (
                vec!["↑".to_owned(), "↓".to_owned()],
                Word::Choose.text(language).to_owned(),
            ),
            (
                vec!["Return".to_owned()],
                Word::FlyTo.text(language).to_owned(),
            ),
            (vec!["Esc".to_owned()], close_word(&query, language)),
        ];
        let placeholder = Word::FindWindow.text(language).to_owned();
        commands.extend(self.fitted(|s| {
            let model = crate::command::ListModel {
                rows: &shown,
                sel: sel - first,
                hovered: hovered.and_then(|h| h.checked_sub(first)),
                query: &query,
                count: count.clone(),
                placeholder: placeholder.clone(),
                footer: footer.clone(),
                thumb_shader: Some(&thumb),
            };
            crate::command::list(&s.styled(), &model, LIST, out)
        })?);
        if open {
            commands.push(hud::fade(LIST, 1.0, hud::FADE_IN));
        }
        commands.extend(self.label_commands());
        self.rt.conn.apply(commands)?;
        self.draw_modes()
    }

    /// The windows of the list that its search leaves, with their notes
    /// and thumbnails; the window managers while it gives a window.
    fn window_rows(&mut self) -> Result<Vec<crate::command::WindowRow>, ClientError> {
        let rows = self.rows()?;
        let entries = self.windows()?;
        let titles = self.titles()?;
        Ok(rows
            .into_iter()
            .map(|row| {
                let entry = entries.iter().find(|e| e.surface == row.target);
                let title = titles
                    .iter()
                    .find(|(id, _)| *id == row.target)
                    .map(|(_, t)| t.clone());
                let shown = entry.is_some_and(|e| e.pane == "shown");
                let thumb = (shown && !self.glass.plain)
                    .then(|| {
                        self.pane_size(&row.target)
                            .map(|size| (row.target.clone(), size))
                    })
                    .flatten();
                crate::command::WindowRow {
                    name: title.filter(|t| !t.trim().is_empty()).unwrap_or(row.label),
                    note: entry.map_or_else(|| row.note.clone(), |e| self.bearing(e)),
                    thumb,
                }
            })
            .collect())
    }

    /// The titles of the windows by surface.
    fn titles(&mut self) -> Result<Vec<(String, String)>, ClientError> {
        let nodes = self.rt.conn.query("kind:surface")?;
        Ok(nodes
            .into_iter()
            .filter_map(|n| Some((n.id, n.surface?.title?)))
            .collect())
    }

    /// The size of the pane of a window.
    fn pane_size(&self, surface: &str) -> Option<[f64; 2]> {
        (self.panes.iter())
            .find(|p| p.window() == Some(surface))
            .map(|p| p.size)
    }

    /// Where a window is from the camera: ahead, left, right, behind,
    /// or not in view.
    fn bearing(&self, entry: &crate::Entry) -> String {
        if entry.pane != "shown" {
            return if entry.pane == "none" || entry.pane == "covered" {
                "not in view".into()
            } else {
                "behind you".into()
            };
        }
        let Some(pane) = self
            .panes
            .iter()
            .find(|p| p.window() == Some(entry.surface.as_str()))
        else {
            return String::new();
        };
        side(
            self.pose.forward(),
            [pane.at[0] - self.pose.x, pane.at[1] - self.pose.z],
        )
        .into()
    }

    /// What the elements over the space draw with.
    pub(crate) fn styled(&self) -> Look<'_> {
        let style = &self.settings.value.style;
        Look {
            style,
            colors: style.colors(self.settings.value.theme),
            plain: self.glass.plain,
            shader: &self.glass_shader,
            shadow: &self.shadow_shader,
            glass: chora_kit::glass::Material::of(
                self.settings.value.theme == crate::config::Theme::Dark,
            ),
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
        self.label_drawn = false;
        self.start_card.up = false;
        self.face_card = None;
        self.face_place = None;
        self.draw_hud()?;
        self.draw_ask()?;
        if self.list.is_some() {
            self.draw_list(true)?;
        }
        self.draw_help(false)
    }

    /// Shows or hides the help; a new help starts without a search.
    pub(crate) fn toggle_help(&mut self) -> Result<(), ClientError> {
        self.help = !self.help;
        self.help_query.clear();
        self.help_sel = 0;
        if self.help {
            self.draw_help(true)?;
        } else {
            self.rt
                .conn
                .try_apply(vec![hud::fade(hud::HELP, 0.0, hud::FADE_OUT)])?;
        }
        let label = self.label_commands();
        self.rt.conn.apply(label)?;
        self.draw_modes()
    }

    /// The help anew, with the keys that work now, while it shows.
    #[allow(
        clippy::too_many_lines,
        reason = "the parts of the help, one after the other"
    )]
    pub(crate) fn draw_help(&mut self, fresh: bool) -> Result<(), ClientError> {
        use chora_kit::words::{Language, Word, say};
        if !self.help || !self.active {
            return Ok(());
        }
        let language = Language::of_session();
        let groups = self.settings.value.help_groups(self.cube.is_some());
        let query = self.help_query.clone();
        let found: usize = crate::command::found(&groups, &query)
            .iter()
            .map(|(_, r)| r.len())
            .sum();
        self.help_sel = self.help_sel.min(found.saturating_sub(1));
        let footer = if query.is_empty() {
            vec![(
                vec!["Esc".to_owned()],
                Word::Close.text(language).to_owned(),
            )]
        } else {
            let count = found.to_string();
            vec![
                (
                    vec!["↑".to_owned(), "↓".to_owned()],
                    Word::MoveSelection.text(language).to_owned(),
                ),
                (
                    Vec::new(),
                    say(language, Word::KeysMatch, &[("count", &count)]),
                ),
                (
                    vec!["Esc".to_owned()],
                    Word::Clear.text(language).to_owned(),
                ),
            ]
        };
        let scope = if self.flown.is_some() {
            "Construct · in a pane"
        } else if self.cube.is_some() {
            "Construct · cube"
        } else {
            "Construct · walking"
        };
        let model = crate::command::HelpModel {
            groups: &groups,
            query: &query,
            sel: self.help_sel,
            scope: scope.into(),
            placeholder: Word::SearchKeys.text(language).to_owned(),
            every_look: Word::EveryLook.text(language).to_owned(),
            or: "or".into(),
            note: Word::OnlyNow.text(language).to_owned(),
            footer,
        };
        let out = [self.lens.w, self.lens.h];
        let mut commands = Vec::new();
        if !self.rt.conn.query(&format!("id:{}", hud::HELP))?.is_empty() {
            commands.push(Command::Remove {
                id: hud::HELP.into(),
                to: None,
            });
        }
        commands.extend(
            self.fitted(|s| crate::command::help(&s.styled(), &model, (hud::HELP, NAME), out))?,
        );
        if fresh {
            commands.push(crate::panes::update(
                hud::HELP,
                PropsUpdate {
                    opacity: Some(0.0),
                    ..PropsUpdate::default()
                },
            ));
            commands.push(hud::fade(hud::HELP, 1.0, hud::FADE_IN));
        }
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

/// The notice of the construct under the island of the host.
const NOTICE: &str = "construct-notice";
/// The widest notice, and its top: under the island, 8 below it.
const NOTICE_WIDTH: f64 = 520.0;
const NOTICE_TOP: f64 = 80.0;

/// Where a point `delta` from the camera is for the person who looks along
/// `forward` on the ground: ahead, left, right or behind.
fn side([fx, fz]: [f64; 2], [dx, dz]: [f64; 2]) -> &'static str {
    let ahead = dx * fx + dz * fz;
    let right = dx * fz - dz * fx;
    if ahead <= 0.0 {
        "behind you"
    } else if right.abs() < ahead * 0.35 {
        "ahead"
    } else if right < 0.0 {
        "left"
    } else {
        "right"
    }
}

/// The count of the list: `5 windows`, or the words of the give menu.
fn rows_count(windows: usize, language: chora_kit::words::Language, giving: bool) -> String {
    use chora_kit::words::{Word, say};
    if giving {
        return "Give to".into();
    }
    if windows == 1 {
        return Word::WindowOne.text(language).to_owned();
    }
    let count = windows.to_string();
    say(language, Word::WindowCount, &[("count", &count)])
}

/// The word of Esc in a list: it clears a search, else it closes.
fn close_word(query: &str, language: chora_kit::words::Language) -> String {
    use chora_kit::words::Word;
    if query.is_empty() {
        Word::Close.text(language).to_owned()
    } else {
        Word::Clear.text(language).to_owned()
    }
}

impl Construct {
    /// The notice of the construct, with the HUD of the host: one line of
    /// `text` on a glass capsule under the island at the top right, at
    /// most 520 wide, for its 4 seconds (construct HUD spec, section 5).
    pub(crate) fn draw_notice(&mut self) -> Result<(), ClientError> {
        use chora_kit::glass::{CAPSULE, Type, margin};
        let mut commands = Vec::new();
        if std::mem::take(&mut self.notice_drawn) {
            commands.push(Command::Remove {
                id: NOTICE.into(),
                to: None,
            });
        }
        if let Some(said) = self.voice.say.clone() {
            let out = [self.lens.w, self.lens.h];
            commands.extend(self.fitted(|s| {
                let look = s.styled();
                let role = crate::glass::face(Type::Text, 400);
                let line = look.one_line(role, &said, NOTICE_WIDTH - 32.0);
                let w = (look.size(role, &line, None)[0] + 32.0).ceil();
                let x = out[0] - margin(out[0]) - w;
                let mut commands = crate::glass::surface(
                    &look,
                    &look.glass,
                    &crate::glass::Surface {
                        id: NOTICE,
                        parent: NAME,
                        rect: [x, NOTICE_TOP, w, CAPSULE],
                        radius: CAPSULE / 2.0,
                        strong: false,
                        name: "notice",
                        input: chora_scene::InputMode::Pass,
                    },
                );
                commands.push(crate::glass::text(
                    &look,
                    &format!("{NOTICE}-text"),
                    NOTICE,
                    (role, &line),
                    [16.0, 8.0],
                    look.glass.text,
                ));
                commands
            })?);
            self.notice_drawn = true;
        }
        self.rt.conn.apply(commands)
    }

    /// The set of hints that applies now, and the hint of the pointer,
    /// none while a command surface, a question or a moving pane hides
    /// them. A change of the camera into or out of a pane starts the time
    /// of its hints.
    fn hint_set(&mut self) -> Option<crate::hints::Set> {
        use crate::hints::{FLOWN, Set, WALK};
        use chora_kit::modes::Mode;
        let now = Instant::now();
        let flown = self.in_pane();
        let in_face = (self.cube.as_ref()).is_some_and(|c| c.view().mode() == Mode::Use);
        let engaged = flown || in_face;
        if !self.hints.started || flown != self.hints.flown {
            self.hints.announce = true;
        }
        if !self.hints.started || engaged != self.hints.engaged {
            self.hints.started = true;
            self.hints.until = Some(now + if engaged { FLOWN } else { WALK });
        }
        self.hints.flown = flown;
        self.hints.engaged = engaged;
        if self.hints.until.is_some_and(|t| t <= now) {
            self.hints.until = None;
        }
        let stood_back = self.cube.as_ref().is_some_and(|c| !c.view().forwards());
        let busy = self.shows_start()
            || self.list.is_some()
            || self.help
            || self.asked
            || self.moving.is_some()
            || (self.glide.is_some() && !self.flying_into())
            || self
                .cube
                .as_ref()
                .is_some_and(crate::cube::Host::hides_hints)
            || self.effects.animating();
        if busy || !self.active {
            return None;
        }
        if stood_back {
            return Some(Set::Cube);
        }
        crate::hints::resting_set(crate::hints::Resting {
            engaged,
            has_cube: self.cube.is_some(),
            timed: self.hints.until.is_some(),
            low: self.hints.low,
        })
    }

    /// The caps of the key of an action: a one-letter key if it has one.
    pub(crate) fn keys_of(&self, action: crate::config::Action) -> Vec<String> {
        let bound = || (self.settings.value.keys.iter()).filter(|(_, a)| **a == action);
        (bound().find(|(k, _)| k.chars().count() == 1))
            .or_else(|| bound().next())
            .map_or_else(Vec::new, |(k, _)| crate::glass::chord(k))
    }

    /// The hints of a set, with the keys of the settings.
    fn hint_items(&self, set: crate::hints::Set) -> Vec<crate::hints::Item> {
        use crate::{
            config::Action,
            glass::chord,
            hints::{HINT, Item, Set, Walking},
        };
        use chora_kit::words::{Language, Word};
        let language = Language::of_session();
        let key = |action: Action| self.keys_of(action);
        let item = |keys: Vec<String>, word: Word, hit: &str| Item {
            keys,
            word: word.text(language).to_owned(),
            keys_last: false,
            hit: format!("{HINT}{hit}"),
            chip: false,
        };
        match set {
            Set::Walk => {
                let walk: Vec<String> =
                    [Action::Forward, Action::Left, Action::Back, Action::Right]
                        .into_iter()
                        .flat_map(key)
                        .collect();
                let ahead = self.pane_ahead().is_some();
                crate::hints::walking(self.panes.len(), ahead)
                    .into_iter()
                    .map(|hint| match hint {
                        Walking::Walk => item(walk.clone(), Word::Walk, "walk"),
                        Walking::FlyIn => item(key(Action::FlyIn), Word::FlyIn, "fly-in"),
                        Walking::Windows => {
                            item(key(Action::Windows), Word::WindowsHint, "windows")
                        }
                        Walking::Terminal => Item {
                            word: crate::hints::terminal_word(language).to_owned(),
                            ..item(key(Action::Terminal), Word::Walk, "terminal")
                        },
                        Walking::Keys => item(key(Action::Help), Word::AllKeys, "keys"),
                    })
                    .collect()
            }
            Set::Flown => vec![item(
                chord(&self.settings.value.fly_out),
                Word::StepBack,
                "step-back",
            )],
            Set::Cube => self.cube_hints(language),
        }
    }

    /// The hints of the cube stood back: the turn keys, and Return, which
    /// steps in or, while the face ahead failed, tries again.
    fn cube_hints(&self, language: chora_kit::words::Language) -> Vec<crate::hints::Item> {
        use crate::{glass::chord, hints::Item};
        use chora_kit::words::Word;
        let chip = |keys: Vec<String>, word: Word, last: bool| Item {
            keys,
            word: word.text(language).to_owned(),
            keys_last: last,
            hit: crate::modes::chip_node(word),
            chip: true,
        };
        let failed = self
            .cube
            .as_ref()
            .is_some_and(crate::cube::Host::front_failed);
        let middle = Item {
            word: crate::hints::middle_word(language, failed),
            ..chip(chord("Return"), Word::StepIn, false)
        };
        vec![
            chip(chord("Ctrl+Alt+Left"), Word::TurnLeft, false),
            middle,
            chip(chord("Ctrl+Alt+Right"), Word::TurnRight, true),
        ]
    }

    /// The hints anew when their set or the hint of the pointer changed.
    pub(crate) fn draw_hints(&mut self) -> Result<(), ClientError> {
        if self.output.is_none() {
            return Ok(());
        }
        self.draw_start()?;
        self.draw_face_card()?;
        let set = self.hint_set();
        self.hint_hover = self.hint_hover.take().filter(|_| set.is_some());
        if std::mem::take(&mut self.hints.announce) {
            let view = chora_kit::hud::view_event(NAME, self.hints.flown);
            if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", view) {
                eprintln!("chora-construct: emit the view: {err}");
            }
        }
        let stamp = match set {
            Some(crate::hints::Set::Walk) => {
                self.panes.len().min(2) * 2 + usize::from(self.pane_ahead().is_some())
            }
            _ => self.cube_stamp(),
        };
        let wanted = set.map(|s| (s, self.hovered_hint(), stamp));
        if wanted == self.hints.shown && !std::mem::take(&mut self.hints.stale) {
            self.follow_label()?;
            return self.turn_dots();
        }
        let had = self.hints.shown.take();
        let mut commands = if had.is_some() {
            crate::hints::remove()
        } else {
            Vec::new()
        };
        if let Some(set) = set {
            let items = self.hint_items(set);
            let out = [self.lens.w, self.lens.h];
            let hover = self.hovered_hint();
            commands.extend(self.fitted(|s| {
                let look = s.styled();
                let mut commands = crate::hints::draw(&look, &items, out, hover.as_deref());
                let places = crate::hints::places(&look, &items, out);
                for (item, at) in items.iter().zip(places).filter(|(i, _)| i.chip) {
                    commands.push(crate::hints::chip_word(&look, item, at));
                }
                commands
            })?);
            if set == crate::hints::Set::Cube {
                let corner = crate::hints::origin(&self.styled(), &items, out);
                commands.extend(self.face_label(corner)?);
                if self.cube.as_ref().is_some_and(crate::cube::Host::turning) {
                    commands.extend(crate::hints::dim(&items[1]));
                }
                commands.push(Command::Move {
                    id: crate::modes::MODES.into(),
                    parent: None,
                    index: None,
                });
            }
        }
        self.hints.shown = wanted;
        self.hints.weights.clear();
        self.rt.conn.apply(commands)?;
        self.turn_dots()
    }

    /// The hint with the hover fill: the one under the pointer, or Step
    /// in while the pointer lights the edge of a face.
    fn hovered_hint(&self) -> Option<String> {
        let lit = self
            .cube
            .as_ref()
            .is_some_and(crate::cube::Host::lights_edge);
        let step_in = crate::modes::chip_node(chora_kit::words::Word::StepIn);
        crate::hints::hovered(self.hint_hover.as_deref(), lit, &step_in)
    }

    /// What the hints of the cube show of its face: the face that the
    /// label names, whether it failed, and whether the cube turns.
    fn cube_stamp(&self) -> usize {
        let Some(host) = self.cube.as_ref() else {
            return 0;
        };
        let face = host.label_face();
        let failed = matches!(host.health_of(face), crate::faces::Health::Failed(_));
        (face * 2 + usize::from(failed)) * 2 + usize::from(host.turning())
    }

    /// While the cube turns, the dots of the label fill with the turn:
    /// the dot of the face before the angle loses what the next one gains.
    fn turn_dots(&mut self) -> Result<(), ClientError> {
        let shown = matches!(self.hints.shown, Some((crate::hints::Set::Cube, ..)));
        let weights = (self.cube.as_ref())
            .filter(|c| shown && c.turning() && c.works())
            .map(crate::cube::Host::dot_weights);
        let Some(weights) = weights else {
            self.hints.weights.clear();
            return Ok(());
        };
        if weights == self.hints.weights {
            return Ok(());
        }
        let commands = crate::hints::dot_updates(&self.styled().glass, &weights);
        self.hints.weights = weights;
        self.rt.conn.apply(commands)
    }

    /// Whether the start card shows: the floor is empty and nothing else
    /// takes the screen.
    pub(crate) fn shows_start(&self) -> bool {
        self.active
            && !self.overlay
            && self.output.is_some()
            && self.panes.is_empty()
            && self.cube.is_none()
            && self.list.is_none()
            && !self.help
            && !self.asked
            && self.launcher_window.is_none()
    }

    /// The start card made anew when it appears or its output changes,
    /// faded out when it stops showing.
    pub(crate) fn draw_start(&mut self) -> Result<(), ClientError> {
        use crate::{config::Action, start};
        use chora_kit::words::Language;
        let wanted = self.shows_start();
        let out = [self.lens.w, self.lens.h];
        if !wanted {
            self.start_card.hover = None;
            if std::mem::take(&mut self.start_card.up) {
                self.rt.conn.try_apply(start::hide())?;
            }
            return Ok(());
        }
        let same = (self.start_card.out.iter().zip(out)).all(|(a, b)| (a - b).abs() < 0.5);
        if self.start_card.up && same {
            return Ok(());
        }
        let words = start::words(Language::of_session());
        let keys = [Action::Terminal, Action::Launcher, Action::Cube].map(|a| self.keys_of(a));
        let help = self.keys_of(Action::Help);
        let hovered = self.start_card.hover;
        let mut commands = Vec::new();
        if !self
            .rt
            .conn
            .query(&format!("id:{}", start::START))?
            .is_empty()
        {
            commands.push(Command::Remove {
                id: start::START.into(),
                to: None,
            });
        }
        commands.extend(self.fitted(|s| {
            let model = start::Model {
                words: &words,
                keys: keys.clone(),
                help: help.clone(),
                hovered,
            };
            start::draw(&s.styled(), &model, out)
        })?);
        commands.push(crate::panes::update(
            start::START,
            PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        ));
        commands.push(hud::fade(start::START, 1.0, hud::FADE_IN));
        self.rt.conn.apply(commands)?;
        self.start_card.up = true;
        self.start_card.out = out;
        Ok(())
    }

    /// The box of face `face` on the screen, if the core projects it.
    fn face_rect(&mut self, face: usize) -> Result<Option<[f64; 4]>, ClientError> {
        let node = crate::cube::face_id(face);
        let nodes: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new(format!("id:{node}")))?;
        let rect = nodes
            .first()
            .and_then(|n| n["rects"].as_object()?.values().next()?.as_array().cloned());
        let sides: Vec<f64> = rect.iter().flatten().filter_map(Value::as_f64).collect();
        Ok(<[f64; 4]>::try_from(sides).ok())
    }

    /// The face label under the front face of the cube stood back. It
    /// names the face nearest to the front while the cube turns, and it
    /// hangs under the middle of the projected bottom edge of that face.
    fn face_label(&mut self, [hx, hy]: [f64; 2]) -> Result<Vec<Command>, ClientError> {
        use chora_kit::words::{Language, Word, say};
        let Some(host) = self.cube.as_ref() else {
            return Ok(Vec::new());
        };
        let Some(anchor) = host.label_anchor(&self.lens, &self.pose) else {
            return Ok(Vec::new());
        };
        let faces = host.front_face().1;
        let (named, language) = (host.label_face(), Language::of_session());
        let manager = host.manager_name().to_owned();
        let failed = |i: usize| matches!(host.health_of(i), crate::faces::Health::Failed(_));
        let hollow: Vec<bool> = (0..faces).map(failed).collect();
        let (face, count) = ((named + 1).to_string(), faces.to_string());
        let words = if hollow.get(named) == Some(&true) {
            crate::faces::label_words(language).to_owned()
        } else {
            say(language, Word::Face, &[("face", &face), ("faces", &count)])
        };
        let at = [anchor[0] - hx, anchor[1] - hy];
        let (commands, width) = self.fitted(|s| {
            let look = s.styled();
            let title = crate::names::face_title(&manager, named, faces);
            let said = (title.as_str(), format!("\u{b7} {words}"));
            let width = crate::hints::face_label_width(&look, (said.0, &said.1), faces);
            let drawn =
                crate::hints::face_label(&look, (said.0, &said.1), (named, faces), at, &hollow);
            (drawn, width)
        })?;
        self.hints.label = Some(crate::hints::Label {
            width,
            corner: [hx, hy],
            at: crate::hints::face_label_place(width, at),
        });
        Ok(commands)
    }

    /// The face label follows the front face of the cube: where the face
    /// has gone since the label was drawn, it moves.
    fn follow_label(&mut self) -> Result<(), ClientError> {
        let (Some(label), Some(host)) = (self.hints.label, self.cube.as_ref()) else {
            return Ok(());
        };
        let Some(anchor) = host.label_anchor(&self.lens, &self.pose) else {
            return Ok(());
        };
        let at = crate::hints::face_label_place(
            label.width,
            [anchor[0] - label.corner[0], anchor[1] - label.corner[1]],
        );
        if at.iter().zip(label.at).all(|(a, b)| (a - b).abs() < 0.5) {
            return Ok(());
        }
        self.hints.label = Some(crate::hints::Label { at, ..label });
        self.rt.conn.apply(vec![crate::hints::face_label_move(at)])
    }

    /// The face whose card shows now: the face ahead while the cube
    /// rests and its window manager is not up, with its health, the name
    /// of its manager and whether another face works.
    fn card_wanted(&self) -> Option<(usize, crate::faces::Health, String)> {
        let hidden = !self.active
            || self.flown.is_some()
            || self.help
            || self.list.is_some()
            || self.asked
            || self.output.is_none();
        let host = (self.cube.as_ref()).filter(|c| !hidden && c.rests())?;
        let front = host.front_face().0;
        let health = host.health_of(front);
        (health != crate::faces::Health::Up)
            .then(|| (front, health, host.manager_name().to_owned()))
    }

    /// The card of the face ahead while its window manager starts or
    /// failed: flat, centred on the face, drawn anew when its words or
    /// its place change, faded out when it is not needed.
    fn draw_face_card(&mut self) -> Result<(), ClientError> {
        use crate::faces::{self, Health};
        use chora_kit::words::Language;
        let ahead = self.card_wanted();
        let out = [self.lens.w, self.lens.h];
        let stood_back = self.cube.as_ref().is_some_and(|c| !c.view().forwards());
        let placed = match ahead {
            Some((front, ref health, _)) => {
                let inputs = format!("{front}{health:?}{out:?}{stood_back}");
                let mut cache = self.face_place.take();
                let placed = faces::cached(&mut cache, &inputs, || self.face_rect(front));
                self.face_place = cache;
                placed?
            }
            None => None,
        };
        let (Some((front, health, name)), Some(rect)) = (ahead, placed) else {
            if self.face_card.take().is_some() {
                self.rt.conn.try_apply(faces::hide())?;
            }
            return Ok(());
        };
        let key = format!("{front}{health:?}{rect:?}{out:?}{stood_back}");
        if self.face_card.as_deref() == Some(&key) {
            return Ok(());
        }
        let words = faces::words(Language::of_session(), &name, &health);
        let centre = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
        let mut commands = Vec::new();
        if !self
            .rt
            .conn
            .query(&format!("id:{}", faces::CARD))?
            .is_empty()
        {
            commands.push(Command::Remove {
                id: faces::CARD.into(),
                to: None,
            });
        }
        commands.extend(self.fitted(|s| {
            faces::draw(&s.styled(), &words, health == Health::Starting, centre, out)
        })?);
        commands.push(crate::panes::update(
            faces::CARD,
            PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        ));
        commands.push(hud::fade(faces::CARD, 1.0, hud::FADE_IN));
        self.rt.conn.apply(commands)?;
        self.face_card = Some(key);
        Ok(())
    }

    /// The pointer rests low or not: the hints of the walk follow it.
    pub(crate) fn hint_pointer(&mut self, y: f64) -> Result<(), ClientError> {
        let low = y >= self.lens.h - crate::hints::BOTTOM;
        if low == self.hints.low {
            return Ok(());
        }
        self.hints.low = low;
        if !low && self.hints.until.is_none() {
            self.hints.until = Some(Instant::now() + Duration::from_millis(300));
        }
        self.draw_hints()
    }

    /// A click on a hint does its act.
    pub(crate) fn hint_press(&mut self, node: &str) -> Result<bool, ClientError> {
        let Some(name) = node.strip_prefix(crate::hints::HINT) else {
            return Ok(false);
        };
        match name {
            "fly-in" => self.act(crate::config::Action::FlyIn),
            "terminal" => self.start_terminal(),
            "windows" => self.open_list()?,
            "keys" => self.toggle_help()?,
            "step-back" if self.flown.is_none() && self.cube.is_some() => {
                self.mode_act(chora_kit::modes::Act::StepBack, crate::modes::By::Person)?;
            }
            "step-back" => self.fly_out()?,
            "cube" => self.cube_key()?,
            _ => {}
        }
        Ok(true)
    }

    /// The pointer came over a hint or left it.
    pub(crate) fn hint_hovered(&mut self, node: &str, over: bool) -> Result<bool, ClientError> {
        let hint = node.starts_with(crate::hints::HINT)
            || (self
                .hints
                .shown
                .as_ref()
                .is_some_and(|(s, _, _)| *s == crate::hints::Set::Cube)
                && crate::modes::is_chip(node));
        if !hint {
            return Ok(false);
        }
        let now = over.then(|| node.to_owned());
        if now == self.hint_hover || (!over && self.hint_hover.as_deref() != Some(node)) {
            return Ok(true);
        }
        self.hint_hover = now;
        self.draw_hints()?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_count_of_one_window_is_singular() {
        use chora_kit::words::Language;
        assert_eq!(super::rows_count(1, Language::En, false), "1 window");
        assert_eq!(super::rows_count(5, Language::En, false), "5 windows");
        assert_eq!(super::rows_count(1, Language::De, false), "1 Fenster");
    }

    #[test]
    fn a_pane_at_the_left_of_the_view_is_left() {
        let forward = crate::space::Pose {
            x: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        }
        .forward();
        assert_eq!(super::side(forward, [0.0, 1500.0]), "ahead");
        assert_eq!(super::side(forward, [-770.0, 1440.0]), "left");
        assert_eq!(super::side(forward, [770.0, 1440.0]), "right");
        assert_eq!(super::side(forward, [0.0, -500.0]), "behind you");
    }
}
