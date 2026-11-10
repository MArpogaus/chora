//! The voice agent in the plugin: its events (D65) drive the figure of
//! [`crate::assistant`], the question card and the badge of a muted mic
//! of [`crate::card`]. A click on the figure mutes the mic; only a real
//! click on the badge turns it on again.

use std::time::{Duration, Instant};

use chora_kit::{
    ask::Question,
    hud::Anchor,
    words::{self, Word},
};
use chora_scene::{Command, NodeKind, OutputInfo, PropsUpdate, ipc::ClientError};
use serde_json::{Value, json};

use crate::{
    Agent,
    assistant::{self, Assistant},
    card::{self, Outcome, Shown},
    chat::{self, Where},
    context,
};

/// The client name of the voice agent before its first event.
const VOICE: &str = "voice";

/// The most helpers that the context counts.
const MAX_HELPERS: usize = 64;

impl Agent {
    /// Asks the voice agent for its state, so that the figure shows after
    /// a start or restart of chora-assistant while the agent is awake.
    pub fn ask_voice(&mut self) -> Result<(), ClientError> {
        self.claim_indicator(false);
        if let Ok(state) = self.voice_call("state", &json!({})) {
            let name = state["state"].as_str().map(assistant::context_state);
            self.voice = name.map(|n| (VOICE.to_owned(), n));
            self.muted = state["muted"]["mic"].as_bool().unwrap_or(false);
            let commands = self.sync_assistant(Instant::now())?;
            self.apply(commands)?;
        }
        Ok(())
    }

    /// An event of the voice agent: `NAME.state` sets the state and the
    /// mic, `NAME.say` shows a sentence, and `NAME.confirm` shows a question
    /// with its full detail until its end. The events of sub-agents, which
    /// carry a name of their own, leave the figure alone. `voice.*` also
    /// matches a client named `voice.evil`: only the agent itself drives the
    /// figure.
    pub fn voice_event(
        &mut self,
        event: &str,
        client: &str,
        data: &Value,
    ) -> Result<Vec<Command>, ClientError> {
        if client != VOICE {
            return Ok(Vec::new());
        }
        self.claim_indicator(true);
        let kind = event.strip_prefix(client).and_then(|e| e.strip_prefix('.'));
        let name = data.get("name").and_then(Value::as_str);
        let now = Instant::now();
        if let Some(helper) = name.filter(|n| *n != client) {
            if kind == Some("state") {
                self.helper_state(helper, data, now);
                self.chat_voice(event, data);
            }
            if kind == Some("remove") {
                self.helper_gone(helper, now);
                self.chat_voice(event, data);
            }
            return Ok(Vec::new());
        }
        match kind {
            Some("state") => {
                let state = data.get("state").and_then(Value::as_str).unwrap_or("");
                self.voice = Some((client.to_owned(), assistant::context_state(state)));
                if let Some(mic) = data["muted"]["mic"].as_bool() {
                    self.muted = mic;
                }
                let mut commands = self.sync_assistant(now)?;
                commands.extend(self.sync_badge()?);
                Ok(commands)
            }
            Some("say") => {
                self.voice = Some((client.to_owned(), "speaking"));
                let mut commands = self.sync_assistant(now)?;
                let text = data.get("text").and_then(Value::as_str).unwrap_or("");
                let time = Duration::from_millis(data["duration_ms"].as_u64().unwrap_or(0));
                if !text.is_empty() && self.card.is_none() {
                    commands.extend(self.say(text, time, now));
                }
                Ok(commands)
            }
            Some("confirm") => {
                let commands = self.confirm(data, now)?;
                self.chat_voice(event, data);
                Ok(commands)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Notes the state of a helper for the context; the helpers that the
    /// agent names are at most [`MAX_HELPERS`].
    fn helper_state(&mut self, name: &str, data: &Value, now: Instant) {
        let state = assistant::context_state(data["state"].as_str().unwrap_or(""));
        if self.helpers.len() < MAX_HELPERS || self.helpers.contains_key(name) {
            self.helpers.insert(name.to_owned(), state);
        }
        let agents = context::agents_of(self.helpers.values().copied());
        self.switcher.set_agents(agents, now);
    }

    /// A helper ended: it no longer counts for the context.
    fn helper_gone(&mut self, name: &str, now: Instant) {
        self.helpers.remove(name);
        let agents = context::agents_of(self.helpers.values().copied());
        self.switcher.set_agents(agents, now);
    }

    /// Shows a sentence in the balloon, through the template and the length
    /// of the folder.
    fn say(&mut self, text: &str, time: Duration, now: Instant) -> Vec<Command> {
        let Some(a) = self.assistant.as_mut() else {
            return Vec::new();
        };
        let line = a.line();
        let mut shown = chora_kit::hud::render(&line.text, &json!({ "text": text }));
        if let Some(most) = line.max_chars {
            shown = shown.chars().take(most).collect();
        }
        a.say(&mut self.theme, &shown, time, now)
    }

    /// The figure of the output, or where the neutral dot would stand.
    fn figure_rect(&self, output: &OutputInfo) -> [i32; 4] {
        self.assistant.as_ref().map_or_else(
            || {
                let size = [24, 24];
                let [x, y] = assistant::origin(
                    Anchor::BottomRight,
                    assistant::work_area(output),
                    size,
                    [16, 16],
                );
                [x, y, size[0], size[1]]
            },
            Assistant::rect,
        )
    }

    /// The question of `voice.confirm` as a card, once and in one place
    /// until it ends; its end takes the card away. A question whose whole
    /// text does not fit has no buttons and is refused at once.
    fn confirm(&mut self, data: &Value, now: Instant) -> Result<Vec<Command>, ClientError> {
        let Some(question) = Question::from_event(data, None) else {
            return Ok(self.end_card(data));
        };
        let id = question.answer_id.clone();
        if !self.settings.value.enabled || self.card.as_ref().is_some_and(|c| c.id == id) {
            return Ok(Vec::new());
        }
        let Some((output, scale)) = self.output()? else {
            return Ok(Vec::new());
        };
        self.card_theme.set_scale(scale);
        let figure = self.figure_rect(&output);
        let mut commands = self.ensure_top(&output.name);
        let mut shown = Shown {
            card: chora_kit::ask::Card::new(question, now),
            id,
            buttons: data["buttons"] == true,
            output: output.name.clone(),
            origin: [0, 0],
            notice: false,
            too_long: false,
            max_width: None,
        };
        self.place_card(&mut shown, figure, &output)?;
        let language = self.language;
        if self.card.take().is_some() {
            commands.extend(card::hide());
        }
        commands.extend(card::show(
            &mut self.card_theme,
            &shown,
            language,
            output.size,
            true,
        ));
        if self.badge {
            let rect = card::rect(&self.card_theme, &shown, language, output.size);
            commands.push(card::badge_move(
                &self.card_theme,
                language,
                (figure, assistant::work_area(&output)),
                rect,
            ));
        }
        if shown.too_long {
            let refusal = json!({"id": shown.id, "yes": false});
            if let Err(err) = self.voice_call("confirm.answer", &refusal) {
                eprintln!("chora-assistant: voice: confirm.answer: {err}");
            }
        }
        self.card = Some(shown);
        commands.extend(self.raise_badge());
        Ok(commands)
    }

    /// The end of a question, by `done` with its id.
    fn end_card(&mut self, data: &Value) -> Vec<Command> {
        let Some(id) = chora_kit::ask::event_id(data) else {
            return Vec::new();
        };
        if self.card.as_ref().is_some_and(|c| c.id == id) {
            self.card = None;
            self.chat.question_ended();
            return card::hide();
        }
        Vec::new()
    }

    /// The width, the size and the corner of a new card; a card that
    /// replaces another keeps the place of the old one.
    fn place_card(
        &mut self,
        shown: &mut Shown,
        figure: [i32; 4],
        output: &OutputInfo,
    ) -> Result<(), ClientError> {
        let kept = self.card.as_ref().map(|old| (old.origin, old.max_width));
        let view = match kept {
            Some(_) => None,
            None => self.card_view()?,
        };
        shown.max_width = kept.map_or(view.as_ref().map(|(_, width)| *width), |(_, w)| w);
        let (theme, language) = (&self.card_theme, self.language);
        shown.too_long = !card::fits(theme, shown, language, output.size);
        shown.buttons &= !shown.too_long;
        let size = card::size(theme, shown, language, output.size);
        shown.origin = match (kept, view) {
            (Some((old, _)), _) => old,
            (None, Some((frame, _))) => chat::card_origin(&frame, size),
            (None, None) => card::place(figure, size, assistant::work_area(output)),
        };
        Ok(())
    }

    /// Where a card that appears now goes, and where it stays: the frame
    /// and the width of the chat view if one is open, else the figure.
    fn card_view(&mut self) -> Result<Option<(chat::Frame, i32)>, ClientError> {
        if self.chat.question_asked() != Where::Chat {
            return Ok(None);
        }
        Ok(self.chat_frame()?.map(|frame| {
            let (anchor, width) = frame.look.place();
            let view = chat::view_rect(anchor, width, frame.area);
            (frame, view[2])
        }))
    }

    /// The group of the card and the badge, above the group of the folder,
    /// made once for an output.
    fn ensure_top(&mut self, output: &str) -> Vec<Command> {
        if self.top.as_deref() == Some(output) {
            return Vec::new();
        }
        let mut commands = Vec::new();
        if self.top.take().is_some() {
            commands.push(Command::Remove {
                id: card::TOP.into(),
                to: None,
            });
            self.badge = false;
        }
        self.top = Some(output.to_owned());
        commands.push(Command::Create {
            id: card::TOP.into(),
            kind: NodeKind::Group,
            parent: format!("screen:{output}"),
            props: PropsUpdate::default(),
        });
        commands
    }

    /// Puts the badge above the card.
    fn raise_badge(&self) -> Vec<Command> {
        if !self.badge {
            return Vec::new();
        }
        vec![Command::Move {
            id: card::BADGE.into(),
            parent: None,
            index: None,
        }]
    }

    /// Shows the badge while the mic is muted and the figure shows, and
    /// takes it away otherwise.
    pub fn sync_badge(&mut self) -> Result<Vec<Command>, ClientError> {
        let figure = self.assistant.as_ref().map(Assistant::rect);
        let want = self.muted && figure.is_some();
        if want == self.badge {
            return Ok(Vec::new());
        }
        let (Some(figure), true) = (figure, want) else {
            self.badge = false;
            return Ok(vec![Command::Remove {
                id: card::BADGE.into(),
                to: None,
            }]);
        };
        let Some((output, scale)) = self.output()? else {
            return Ok(Vec::new());
        };
        self.card_theme.set_scale(scale);
        let mut commands = self.ensure_top(&output.name);
        let (theme, language) = (&self.card_theme, self.language);
        let avoid = self
            .card
            .as_ref()
            .map(|shown| card::rect(theme, shown, language, output.size));
        commands.extend(card::badge(
            &mut self.card_theme,
            language,
            (figure, assistant::work_area(&output)),
            avoid,
        ));
        self.badge = true;
        Ok(commands)
    }

    /// The voice agent connected: it shows its own state indicator until
    /// a claim.
    pub fn voice_connected(&mut self, name: &str) {
        if name == VOICE {
            self.claim_due = true;
            self.claim_indicator(false);
        }
    }

    /// Asks the voice agent to hide its own state indicator, since the
    /// figure shows the state, while one is due and the assistant is
    /// enabled. An error, such as no agent or an agent without the
    /// method, only leaves it due; after an event of the agent (`last`)
    /// it is no longer due.
    pub fn claim_indicator(&mut self, last: bool) {
        if !self.claim_due || !self.settings.value.enabled {
            return;
        }
        let done = self.voice_call("indicator.claim", &json!({})).is_ok();
        self.claim_due = !done && !last;
    }

    /// The voice agent closed its connection.
    pub fn voice_gone(&mut self, name: &str) -> Result<Vec<Command>, ClientError> {
        if name == VOICE {
            self.claim_due = true;
        }
        if self.voice.as_ref().is_none_or(|(client, _)| client != name) {
            return Ok(Vec::new());
        }
        self.voice = None;
        self.sync_assistant(Instant::now())
    }

    /// The words of a state, in the language of the screen.
    fn state_label(&self, state: &str) -> String {
        words::state(state)
            .map(|word| word.text(self.language).to_owned())
            .unwrap_or_default()
    }

    /// Shows, moves into its state or removes the figure, as the agent,
    /// the settings and the outputs want it. Without the figure there is no
    /// card and no badge.
    pub fn sync_assistant(&mut self, now: Instant) -> Result<Vec<Command>, ClientError> {
        let enabled = self.settings.value.enabled;
        let want = self.voice.as_ref().map(|(_, s)| *s).filter(|_| enabled);
        self.switcher.set_state(want.unwrap_or("asleep"), now);
        let label = want
            .map(|state| self.state_label(state))
            .unwrap_or_default();
        match (want, self.assistant.as_mut()) {
            (None, None) => Ok(Vec::new()),
            (None, Some(_)) => {
                self.assistant = None;
                let mut commands = vec![Assistant::remove()];
                commands.extend(self.drop_top());
                Ok(commands)
            }
            (Some(state), Some(a)) => Ok(a.set_state(&mut self.theme, (state, &label), now)),
            (Some(state), None) => {
                let Some((output, scale)) = self.output()? else {
                    return Ok(Vec::new());
                };
                self.theme.set_scale(scale);
                let anchor = self
                    .settings
                    .value
                    .position
                    .map_or(self.values.anchor, anchor_of);
                let (a, commands) = Assistant::new(
                    &mut self.theme,
                    &self.values,
                    (&output, anchor),
                    (state, &label, now),
                );
                self.assistant = Some(a);
                Ok(commands)
            }
        }
    }

    /// Takes the card, the badge and their group away.
    fn drop_top(&mut self) -> Vec<Command> {
        self.card = None;
        self.badge = false;
        if self.top.take().is_none() {
            return Vec::new();
        }
        vec![Command::Remove {
            id: card::TOP.into(),
            to: None,
        }]
    }

    /// Builds the figure again, after a change of the outputs or of the
    /// settings. A card stays in its place on its output; on another output
    /// it appears anew, and its buttons take no click for a moment.
    pub fn replace_assistant(&mut self) -> Result<Vec<Command>, ClientError> {
        let now = Instant::now();
        let had_figure = self.assistant.take().is_some();
        let card = self.card.take();
        let mut commands = if had_figure {
            vec![Assistant::remove()]
        } else {
            Vec::new()
        };
        commands.extend(self.drop_top());
        commands.extend(self.sync_assistant(now)?);
        if let (Some(mut shown), Some((output, scale))) = (card, self.output()?) {
            self.card_theme.set_scale(scale);
            let figure = self.figure_rect(&output);
            commands.extend(self.ensure_top(&output.name));
            let theme = (&self.card_theme, self.language, figure);
            let area = assistant::work_area(&output);
            card::settle(&mut shown, (&output.name, output.size, area), theme, now);
            commands.extend(card::show(
                &mut self.card_theme,
                &shown,
                self.language,
                output.size,
                true,
            ));
            self.card = Some(shown);
        }
        commands.extend(self.sync_badge()?);
        Ok(commands)
    }

    /// When the figure moves, blinks or fades its balloon next.
    pub fn assistant_due(&self) -> Option<Instant> {
        self.assistant.as_ref().map(Assistant::due)
    }

    /// Moves the figure on.
    pub fn assistant_step(&mut self, now: Instant) -> Result<(), ClientError> {
        let Some(a) = self.assistant.as_mut().filter(|a| a.due() <= now) else {
            return Ok(());
        };
        let commands = a.step(&mut self.theme, now);
        self.apply(commands)
    }

    /// A click on the figure mutes the mic, if the folder says so; the
    /// badge that follows turns it on again. A click never unmutes here.
    pub fn press_assistant(&mut self, injected: bool) -> Vec<Command> {
        if !self.assistant.as_ref().is_some_and(Assistant::mutes) {
            return Vec::new();
        }
        if injected {
            let text = Word::Injected.text(self.language);
            return self.say(text, Duration::ZERO, Instant::now());
        }
        let muted = self.voice_call("mute", &json!({"mic": true})).map(|_| true);
        self.mic_says(muted.as_ref().ok().copied(), &muted)
    }

    /// A press on the badge turns the mic on, when a person pressed it; an
    /// injected press says that it does not count.
    pub fn press_badge(&mut self, injected: bool) -> Vec<Command> {
        if injected {
            let text = Word::Injected.text(self.language);
            return self.say(text, Duration::ZERO, Instant::now());
        }
        let result = self
            .voice_call("mute", &json!({"mic": false}))
            .map(|_| false);
        self.mic_says(result.as_ref().ok().copied(), &result)
    }

    /// The balloon and the badge after a try to switch the mic: `now` is
    /// the state of the mic after it, `None` after an error.
    fn mic_says(
        &mut self,
        now_muted: Option<bool>,
        result: &Result<bool, ClientError>,
    ) -> Vec<Command> {
        if let Err(err) = result {
            eprintln!("chora-assistant: voice: {err}");
        }
        let word = match now_muted {
            Some(true) => Word::Muted,
            Some(false) => Word::MicOn,
            None => Word::MicError,
        };
        if let Some(muted) = now_muted {
            self.muted = muted;
        }
        let text = word.text(self.language);
        let mut commands = self.say(text, Duration::ZERO, Instant::now());
        commands.extend(self.sync_badge().unwrap_or_default());
        commands
    }

    /// A press on a button of the card: a person's press answers with
    /// `confirm.answer`; an injected press puts the line on the card and
    /// leaves it; a press too soon after the card appeared does nothing.
    /// True when `node` was a button.
    pub fn press_card(
        &mut self,
        node: &str,
        injected: bool,
    ) -> Result<(bool, Vec<Command>), ClientError> {
        let Some(shown) = self.card.as_mut() else {
            return Ok((false, Vec::new()));
        };
        let Some(outcome) = card::press(shown, node, injected, Instant::now()) else {
            return Ok((false, Vec::new()));
        };
        match outcome {
            Outcome::Answer(params) => {
                if let Err(err) = self.voice_call("confirm.answer", &params) {
                    eprintln!("chora-assistant: voice: confirm.answer: {err}");
                }
                Ok((true, Vec::new()))
            }
            Outcome::Ignored => Ok((true, Vec::new())),
            Outcome::Notice => {
                shown.notice = true;
                let shown = shown.clone();
                let Some((output, _)) = self.output()? else {
                    return Ok((true, Vec::new()));
                };
                let commands = card::show(
                    &mut self.card_theme,
                    &shown,
                    self.language,
                    output.size,
                    false,
                );
                Ok((true, commands))
            }
        }
    }

    /// Calls a method of the voice agent.
    pub fn voice_call(&mut self, method: &str, params: &Value) -> Result<Value, ClientError> {
        let client = self.voice.as_ref().map_or(VOICE, |(c, _)| c.as_str());
        let call = json!({"client": client, "method": method, "params": params});
        self.rt.conn.call("methods.call", call)
    }
}

/// The anchor of a corner of the setting `position`.
fn anchor_of(corner: chora_draw::bubble::Corner) -> chora_kit::hud::Anchor {
    use chora_draw::bubble::Corner;
    use chora_kit::hud::Anchor;
    match corner {
        Corner::TopLeft => Anchor::TopLeft,
        Corner::TopRight => Anchor::TopRight,
        Corner::BottomLeft => Anchor::BottomLeft,
        Corner::BottomRight => Anchor::BottomRight,
    }
}

#[cfg(test)]
mod tests {
    use chora_draw::bubble::Corner;
    use chora_kit::hud::Anchor;

    use super::*;

    #[test]
    fn the_setting_position_names_a_corner_anchor() {
        assert_eq!(anchor_of(Corner::TopLeft), Anchor::TopLeft);
        assert_eq!(anchor_of(Corner::BottomRight), Anchor::BottomRight);
    }
}
