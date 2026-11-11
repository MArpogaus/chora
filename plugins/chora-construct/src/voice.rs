//! Voice glue: following the voice agent, answers, the tick, and drawing
//! the voice HUD.

use super::{
    ClientError, Command, Construct, Duration, Instant, Look, PropsUpdate, Transform, VOICE, Value,
    hud, json, space, update,
};

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
        let hidden = self.flown.is_some()
            || self.moving.is_some()
            || self.effects.animating()
            || self.list.is_some()
            || self.help
            || self.glide.as_ref().is_some_and(|g| g.into.is_some());
        let pick = if hidden { None } else { self.focus() };
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
