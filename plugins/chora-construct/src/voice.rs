//! Voice glue: following the voice agent, answers, the tick, and drawing
//! the voice HUD.

use super::{
    ClientError, Command, Construct, Duration, Instant, Look, PropsUpdate, Transform, VOICE, Value,
    hud, json, space, update,
};

/// Whether the pane label hides: it hides while any of `reasons` holds;
/// the last is an open cube, stood back or stepped into a face, where a
/// label would float over a face or the cube.
pub(crate) fn label_hidden(reasons: &[bool]) -> bool {
    reasons.iter().any(|r| *r)
}

/// Whether a pane whose box on the output is `[left, top, right, bottom]`
/// gets a label of its own in the walk: at least 40 % of its width is on
/// an output `width` wide (every pane has a label, the brief).
pub(crate) fn neighbour_shows([left, _, right, _]: [f64; 4], width: f64) -> bool {
    let all = right - left;
    let seen = right.min(width) - left.max(0.0);
    all > 0.0 && seen >= 0.4 * all
}

impl Construct {
    /// An event `voice.KIND` of the voice agent, while the construct
    /// draws the voice HUD. Wrong data only log.
    pub(crate) fn follow(&mut self, event: &str, data: Value) -> Result<(), ClientError> {
        if event == "voice.confirm" {
            self.asked = data["done"] != true && data.get("prompt").is_some();
            self.draw_hints()?;
        }
        if !self.settings.value.draws_voice() {
            return Ok(());
        }
        self.claim_indicator(true);
        let kind = event.strip_prefix("voice.").unwrap_or_default();
        match self.voice.follow(kind, data, Instant::now()) {
            Ok(changed) => {
                if changed.hud {
                    self.draw_hud()?;
                }
                if changed.ask {
                    self.draw_ask()?;
                }
                Ok(())
            }
            Err(err) => {
                eprintln!("chora-construct: {event}: {err}");
                Ok(())
            }
        }
    }

    /// Answers the open question of the voice agent by the button `node`
    /// with `confirm.answer`; a refusal, as for an old id, only logs.
    pub(crate) fn answer(&mut self, node: &str, injected: bool) {
        let Some(params) = self.voice.answer(node, injected) else {
            return;
        };
        let call = json!({"client": VOICE, "method": "confirm.answer", "params": params});
        if let Err(err) = self.rt.conn.call::<_, Value>("methods.call", call) {
            eprintln!("chora-construct: confirm.answer: {err}");
        }
    }

    /// The deadlines of the sentence: the waveform stops at its end, the
    /// sentence fades out 2 s later, and then the panel fits again.
    pub(crate) fn tick(&mut self) -> Result<(), ClientError> {
        let now = Instant::now();
        let due = |t: Option<Instant>| t.is_some_and(|t| t <= now);
        if due(self.voice.say_until) {
            self.voice.say_until = None;
            self.draw_hud()?;
        }
        if due(self.voice.fade_at) {
            self.voice.fade_at = None;
            self.voice.drop_at = Some(now + Duration::from_millis(120));
            if self.shows_hud() {
                let fade = hud::fade(hud::HUD_SAY, 0.0, hud::FADE_OUT);
                self.rt.conn.try_apply(vec![fade])?;
            }
        }
        if due(self.voice.drop_at) {
            self.voice.drop_at = None;
            self.voice.say = None;
            self.draw_hud()?;
        }
        self.draw_hints()
    }

    /// Whether the HUD line shows: the voice HUD with `voice_hud: own`,
    /// and the notices of the construct always.
    pub(crate) fn shows_hud(&self) -> bool {
        self.active && self.output.is_some()
    }

    /// The voice HUD and the notices: at the bottom centre, or compact at
    /// the bottom right while flown into a pane.
    pub(crate) fn draw_hud(&mut self) -> Result<(), ClientError> {
        if !self.shows_hud() {
            return Ok(());
        }
        if !self.settings.value.draws_voice() {
            return self.draw_notice();
        }
        let out = [self.lens.w, self.lens.h];
        let level = self.voice.level(Instant::now());
        let (compact, glow) = (self.flown.is_some(), !self.glass.hidden);
        let commands = self.fitted(|s| s.voice.hud(&s.styled(), out, compact, level, glow))?;
        if commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.apply(commands)
    }

    /// The question of the voice agent, on top, made anew.
    pub(crate) fn draw_ask(&mut self) -> Result<(), ClientError> {
        if !(self.settings.value.draws_voice() && self.shows_hud()) {
            return Ok(());
        }
        let out = [self.lens.w, self.lens.h];
        let glow = !self.glass.hidden;
        let mut commands = self.fitted(|s| {
            let look = Look {
                style: &s.settings.value.style,
                colors: s.settings.value.style.colors(s.settings.value.theme),
                plain: s.glass.plain,
                shader: &s.glass_shader,
                shadow: &s.shadow_shader,
                glass: chora_kit::glass::Material::of(
                    s.settings.value.theme == crate::config::Theme::Dark,
                ),
                metrics: &s.metrics,
            };
            s.voice.ask(&look, out, glow)
        })?;
        if self.rt.conn.query(&format!("id:{}", hud::ASK))?.is_empty() {
            commands.remove(0);
        }
        self.rt.conn.apply(commands)
    }

    /// The pane label: the name of the pane under the pointer, else of the
    /// pane ahead, 8 below its bottom edge; hidden while flown in, while
    /// the list is open, and when the pane does not show. The text and the
    /// pill change only with the pane.
    pub(crate) fn label_commands(&mut self) -> Vec<Command> {
        let hidden = label_hidden(&[
            self.flown.is_some(),
            self.moving.is_some(),
            self.effects.animating(),
            self.list.is_some(),
            self.help,
            self.glide.as_ref().is_some_and(|g| g.into.is_some()),
            self.cube.is_some(),
        ]);
        let pick = if hidden { None } else { self.focus() };
        let mut commands = self.focus_label(pick);
        commands.extend(self.neighbour_labels(pick, hidden));
        commands
    }

    /// The labels of the other panes that show at least 40 % (every pane
    /// has a label, the brief), named at 60 % and 8 below their bottom
    /// edge. None while the label hides.
    fn neighbour_labels(&mut self, focus: Option<usize>, hidden: bool) -> Vec<Command> {
        let want: Vec<(usize, [f64; 4])> = if hidden || focus.is_none() {
            Vec::new()
        } else {
            (0..self.panes.len())
                .filter(|k| Some(*k) != focus)
                .filter_map(|k| {
                    let p = &self.panes[k];
                    let b = space::screen_box(&self.lens, &self.pose, p.at, p.size, p.yaw)?;
                    neighbour_shows(b, self.lens.w).then_some((k, b))
                })
                .collect()
        };
        let mut commands = Vec::new();
        let gone: Vec<usize> = self
            .neighbours
            .iter()
            .filter(|(k, (name, ..))| {
                (want.iter()).all(|(w, _)| w != *k || self.panes[*w].name != *name)
            })
            .map(|(k, _)| *k)
            .collect();
        for k in gone {
            self.neighbours.remove(&k);
            commands.push(Command::Remove {
                id: hud::neighbour_id(k),
                to: None,
            });
        }
        for (k, _) in want {
            let name = self.pane_title(k);
            let known = self.neighbours.get(&k);
            let width = match known {
                Some((_, text, w)) if *text == name => *w,
                _ => {
                    if known.is_some() {
                        commands.push(Command::Remove {
                            id: hud::neighbour_id(k),
                            to: None,
                        });
                    }
                    let fitted = self.fitted(|s| s.styled().neighbour_label(k, &name));
                    let (drawn, w) = fitted.unwrap_or_else(|err| {
                        eprintln!("chora-construct: measure the label: {err}");
                        self.styled().neighbour_label(k, &name)
                    });
                    commands.extend(drawn);
                    self.neighbours
                        .insert(k, (self.panes[k].name.clone(), name, w));
                    w
                }
            };
            commands.push(self.place_neighbour(k, width));
        }
        commands
    }

    /// Moves the label of pane `k` of `width` under the pane, inside the
    /// output, so that a pane cut by its edge keeps its name.
    fn place_neighbour(&self, k: usize, width: f64) -> Command {
        let pane = &self.panes[k];
        let matrix = space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
        let [x, y] = space::bottom_middle(&matrix, pane.size);
        let unit = self.settings.value.style.unit;
        let x = (x - width / 2.0)
            .round()
            .clamp(8.0, (self.lens.w - width - 8.0).max(8.0));
        update(
            &hud::neighbour_id(k),
            PropsUpdate {
                transform: Some(Transform::translate(x, (y + unit).round(), 0.0)),
                ..PropsUpdate::default()
            },
        )
    }

    fn focus_label(&mut self, pick: Option<usize>) -> Vec<Command> {
        let Some(k) = pick else {
            return match self.labelled.take() {
                Some(_) => Look::label_hide(),
                None => Vec::new(),
            };
        };
        let fresh = self.labelled.is_none();
        let drawn = self.label_drawn;
        let name = self.pane_title(k);
        let fitted = self.fitted(|s| s.styled().label(&name, fresh, drawn));
        let (label, pill) = fitted.unwrap_or_else(|err| {
            eprintln!("chora-construct: measure the label: {err}");
            self.styled().label(&name, fresh, drawn)
        });
        let pane = &self.panes[k];
        let matrix = space::pane_matrix(&self.lens, &self.pose, pane.at, pane.size, pane.yaw);
        let [x, y] = space::bottom_middle(&matrix, pane.size);
        let unit = self.settings.value.style.unit;
        let place = Transform::translate((x - pill / 2.0).round(), (y + unit).round(), 0.0);
        let mut commands = vec![update(
            hud::LABEL,
            PropsUpdate {
                transform: Some(place),
                ..PropsUpdate::default()
            },
        )];
        if self.labelled.as_deref() != Some(pane.name.as_str()) || self.label_text != name {
            self.labelled = Some(pane.name.clone());
            self.label_text = name;
            self.label_drawn = true;
            commands.extend(label);
        }
        commands
    }

    /// The label text of a pane: the title of its window, or what
    /// `names::pane_label` makes of a window without one; the name of a
    /// pane without a window.
    fn pane_title(&mut self, k: usize) -> String {
        let pane = &self.panes[k];
        let fallback = pane.name.clone();
        let Some(surface) = pane.window().map(str::to_owned) else {
            return fallback;
        };
        let command = match &pane.content {
            crate::Content::Window { command } => command.clone(),
            _ => None,
        };
        let label = self.window_label(&surface, command.as_deref());
        if label.is_empty() { fallback } else { label }
    }

    /// The label of the window `surface` that was started with `command`,
    /// from what the core knows of it now.
    pub(crate) fn window_label(&mut self, surface: &str, command: Option<&str>) -> String {
        let info = (self.rt.conn.query(&format!("id:{surface}")).ok())
            .and_then(|nodes| nodes.into_iter().next()?.surface)
            .unwrap_or_default();
        crate::names::pane_label(
            (info.title.as_deref(), info.app_id.as_deref()),
            (command, &self.settings.value.terminal),
            chora_kit::words::Language::of_session(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::label_hidden;

    #[test]
    fn a_pane_label_hides_while_a_cube_is_open_stood_back_or_stepped_in() {
        assert!(!label_hidden(&[false; 7]), "walking shows the label");
        let mut with_cube = [false; 7];
        with_cube[6] = true;
        assert!(label_hidden(&with_cube), "no label over a face or the cube");
    }

    #[test]
    fn a_neighbour_pane_has_a_label_when_40_percent_of_it_is_on_screen() {
        use super::neighbour_shows;
        assert!(neighbour_shows([0.0, 0.0, 329.0, 340.0], 1280.0), "whole");
        assert!(neighbour_shows([-190.0, 0.0, 330.0, 340.0], 1280.0), "63 %");
        assert!(
            !neighbour_shows([-310.0, 0.0, 200.0, 340.0], 1280.0),
            "under 40 %"
        );
        assert!(
            neighbour_shows([1100.0, 0.0, 1400.0, 340.0], 1280.0),
            "60 %"
        );
        assert!(
            !neighbour_shows([1200.0, 0.0, 1600.0, 340.0], 1280.0),
            "20 %"
        );
        assert!(!neighbour_shows([5.0, 0.0, 5.0, 340.0], 1280.0), "no width");
    }
}
