//! The voice agent in the plugin: its events (D65) drive the figure of
//! [`crate::assistant`], the question card and the badge of a muted mic
//! of [`crate::card`]. A click on the figure mutes the mic; only a real
//! click on the badge turns it on again.

use std::time::{Duration, Instant};

use chora_kit::{
    ask::Question,
    hud::{Anchor, ChatForm},
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
    rows::Helper,
};

/// The client name of the voice agent before its first event.
const VOICE: &str = "voice";

/// The most helpers that the context counts.
const MAX_HELPERS: usize = 64;

/// The time between two tries to ask the voice agent for its state.
const ASK_EVERY: Duration = Duration::from_millis(250);

/// How long one call to `state` of the voice agent may take.
const ASK_WAIT: Duration = Duration::from_millis(500);

/// The most tries to ask the voice agent for its state after it connected.
const ASK_TRIES: u8 = 20;

impl Agent {
    /// Asks the voice agent for its state, so that the figure shows after
    /// a start or restart of chora-assistant while the agent is awake.
    pub fn ask_voice(&mut self) -> Result<(), ClientError> {
        self.claim_indicator(false);
        self.ask_state()?;
        Ok(())
    }

    /// Calls `state` of the voice agent and shows what it answers. False
    /// when the call is worth a new try: no answer in [`ASK_WAIT`], or the
    /// method is not exported yet. An answer without a state leaves the
    /// state as it is.
    fn ask_state(&mut self) -> Result<bool, ClientError> {
        let client = self.voice.as_ref().map_or(VOICE, |(c, _)| c.as_str());
        let call = json!({"client": client, "method": "state", "params": {}});
        let state: Value = match self.rt.conn.call_within("methods.call", call, ASK_WAIT) {
            Ok(state) => state,
            Err(ClientError::Io(_)) => return Ok(false),
            Err(ClientError::Rpc(error)) if error.message.ends_with("is no exported method") => {
                return Ok(false);
            }
            Err(ClientError::Rpc(_) | ClientError::Json(_)) => return Ok(true),
            Err(error) => return Err(error),
        };
        let Some(name) = state["state"].as_str() else {
            return Ok(true);
        };
        self.voice = Some((VOICE.to_owned(), assistant::context_state(name)));
        self.muted = state["muted"]["mic"].as_bool().unwrap_or(false);
        let mut commands = self.sync_assistant(Instant::now())?;
        commands.extend(self.sync_badge()?);
        self.apply(commands)?;
        Ok(true)
    }

    /// When the button that took no click shows its normal state again.
    pub fn flash_due(&self) -> Option<Instant> {
        self.flash.as_ref().map(|(_, at)| *at)
    }

    /// Puts the button that took no click back.
    pub fn flash_step(&mut self, now: Instant) -> Result<(), ClientError> {
        let Some((node, _)) = self.flash.take_if(|(_, at)| *at <= now) else {
            return Ok(());
        };
        if self.card.is_none() {
            return Ok(());
        }
        self.apply(vec![card::pressed(&node, false)])
    }

    /// When the next try to ask the voice agent for its state is due.
    pub fn ask_due(&self) -> Option<Instant> {
        self.state_ask.map(|(at, _)| at)
    }

    /// Tries to ask the voice agent for its state after it connected. The
    /// agent exports its methods after its `hello`, so a try can come too
    /// early; it repeats [`ASK_TRIES`] times.
    pub fn ask_step(&mut self, now: Instant) -> Result<(), ClientError> {
        let Some((_, left)) = self.state_ask.filter(|(at, _)| *at <= now) else {
            return Ok(());
        };
        self.state_ask = if self.ask_state()? || left <= 1 {
            None
        } else {
            Some((now + ASK_EVERY, left - 1))
        };
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
        let now = Instant::now();
        if let Some(helper) = helper_of(kind, data, client) {
            if kind == Some("state") {
                self.helper_state(helper, data, now);
                self.chat_voice(event, data);
            }
            if kind == Some("remove") {
                self.helper_gone(helper, now);
                self.chat_voice(event, data);
            }
            let mut commands = self.sync_row()?;
            commands.extend(self.sync_task());
            return Ok(commands);
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
                    self.said = Some((text.to_owned(), now + time));
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
        let helper = Helper {
            state: assistant::context_state(data["state"].as_str().unwrap_or("")),
            task: chora_kit::chat::sanitize(data["task"].as_str().unwrap_or("")),
            steps: data["step"].as_u64().zip(data["steps"].as_u64()),
        };
        if self.helpers.len() < MAX_HELPERS || self.helpers.contains_key(name) {
            self.helpers.insert(name.to_owned(), helper);
        }
        let agents = context::agents_of(self.helpers.values().map(|helper| helper.state));
        self.switcher.set_agents(agents, now);
    }

    /// The task of the busy helper in the island of the folder.
    fn sync_task(&mut self) -> Vec<Command> {
        let task = crate::rows::island_task(self.language, &self.helpers);
        match self.assistant.as_mut() {
            Some(a) => a.set_task(&mut self.theme, &task),
            None => Vec::new(),
        }
    }

    /// A helper ended: it no longer counts for the context.
    fn helper_gone(&mut self, name: &str, now: Instant) {
        self.helpers.remove(name);
        let agents = context::agents_of(self.helpers.values().map(|helper| helper.state));
        self.switcher.set_agents(agents, now);
    }

    /// Shows a sentence in the balloon, through the template and the length
    /// of the folder.
    fn say(&mut self, text: &str, time: Duration, now: Instant) -> Vec<Command> {
        if self.chat.is_open() {
            self.chat.agent_said(text);
            return Vec::new();
        }
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

    /// A question opens the balloon of the figure, so that its card shows
    /// inside: with the helper that asks, else with the list.
    fn open_balloon_for_question(&mut self) {
        if self.card.is_some() || !self.balloon_chat() || self.chat.is_open() {
            return;
        }
        if let Err(err) = self.chat_show(self.asking().as_deref(), None) {
            eprintln!("chora-assistant: the balloon for the question: {err}");
        }
    }

    /// Whether the chat is the balloon of the figure: a `bubble` chat of a
    /// folder whose figure shows.
    pub fn balloon_chat(&self) -> bool {
        self.assistant.is_some() && self.values.chat().style == chora_kit::hud::ChatStyle::Bubble
    }

    /// The chat opened or closed. Open, it is the one surface: it takes
    /// in the sentence, the last one when it is the balloon of the figure,
    /// else the one still spoken, and the balloon of the sentence goes.
    /// Closed, the balloon of the figure shows the last sentence again, and
    /// another look the sentence that the agent still speaks.
    pub fn chat_toggled(&mut self, open: bool, now: Instant) -> Vec<Command> {
        let balloon = self.balloon_chat();
        let last = (self.said.as_ref())
            .filter(|(_, until)| balloon || *until > now)
            .map(|(text, _)| text.clone());
        if open {
            if let Some(text) = &last {
                self.chat.agent_said(text);
            }
            return self
                .assistant
                .as_mut()
                .map(Assistant::hush)
                .unwrap_or_default();
        }
        if !balloon {
            return self.say_again(now);
        }
        last.map(|text| self.say(&text, Duration::ZERO, now))
            .unwrap_or_default()
    }

    /// Shows the sentence again that the agent still speaks, on a figure
    /// that was built again.
    pub fn say_again(&mut self, now: Instant) -> Vec<Command> {
        match self.said.clone() {
            Some((text, until)) if until > now && self.card.is_none() => {
                self.say(&text, until - now, now)
            }
            _ => Vec::new(),
        }
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

    /// The helper that has a question: the title of a card names it when
    /// the event does not.
    fn asking(&self) -> Option<String> {
        (self.helpers.iter())
            .find(|(_, helper)| helper.state == "question")
            .map(|(name, _)| name.clone())
    }

    /// A card for a question on an output, not placed yet.
    fn new_card(&self, question: Question, (buttons, output): (bool, &str), now: Instant) -> Shown {
        let (ink, face) = self.card_ink();
        Shown {
            id: question.answer_id.clone(),
            card: chora_kit::ask::Card::new(question, now),
            buttons,
            output: output.to_owned(),
            origin: [0, 0],
            notice: false,
            too_long: false,
            max_width: None,
            glass: (self.values.chat().style == chora_kit::hud::ChatStyle::Glass)
                .then_some(self.values.dark),
            ink,
            face,
        }
    }

    /// The question of `voice.confirm` as a card, once and in one place
    /// until it ends; its end takes the card away. A question whose whole
    /// text does not fit has no buttons and is refused at once.
    fn confirm(&mut self, data: &Value, now: Instant) -> Result<Vec<Command>, ClientError> {
        let Some(mut question) = Question::from_event(data, None) else {
            return Ok(self.end_card(data));
        };
        if question.agent.is_none() {
            question.agent = self.asking();
        }
        let id = question.answer_id.clone();
        if !self.settings.value.enabled || self.card.as_ref().is_some_and(|c| c.id == id) {
            return Ok(Vec::new());
        }
        self.open_balloon_for_question();
        let Some((output, scale)) = self.output()? else {
            return Ok(Vec::new());
        };
        self.card_fonts();
        self.card_theme.set_scale(scale);
        let figure = self.figure_rect(&output);
        let mut commands = self.ensure_top(&output.name);
        let mut shown = self.new_card(question, (data["buttons"] == true, &output.name), now);
        self.place_card(&mut shown, figure, &output)?;
        let language = self.language;
        if let Some(old) = self.card.take() {
            commands.extend(card::hide(&old));
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

    /// The family of the card: the sans of the look where the folder names
    /// a slot, else the one of the host. The fonts stay those of the host:
    /// a family by name, never a font file of a folder.
    fn card_fonts(&mut self) {
        let want = if self.values.slot || self.values.chat().form == ChatForm::Xp {
            self.values.font.clone()
        } else {
            crate::HOST_FAMILY.to_owned()
        };
        if want == self.card_family {
            return;
        }
        match card::theme(&self.source.cache, &want) {
            Ok(theme) => {
                self.card_theme = theme;
                self.card_family = want;
            }
            Err(err) => eprintln!("chora-assistant: card fonts: {err}"),
        }
    }

    /// What the card is drawn in: the page of the look where the folder
    /// names a slot, else the card of the host.
    fn card_ink(&self) -> (card::Ink, Option<chat::Face>) {
        let chat = self.values.chat();
        if chat.form == ChatForm::Xp {
            return (card::Ink::XP, chat::Face::of(&chat));
        }
        if !self.values.slot {
            return (card::Ink::HOST, None);
        }
        let (fill, text, muted, face) = chat::page_ink(&self.values.chat());
        (card::Ink::page(fill, text, muted), face)
    }

    /// The end of a question, by `done` with its id.
    fn end_card(&mut self, data: &Value) -> Vec<Command> {
        let Some(id) = chora_kit::ask::event_id(data) else {
            return Vec::new();
        };
        if self.card.as_ref().is_some_and(|c| c.id == id) {
            return self.close_card();
        }
        Vec::new()
    }

    /// Takes the card away: its answer reached voice, which ends the
    /// question with `done` too, but an agent that does not say so leaves
    /// no card on the screen after the answer.
    fn close_card(&mut self) -> Vec<Command> {
        let old = self.card.take();
        self.chat.question_ended();
        old.as_ref().map(card::hide).unwrap_or_default()
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
            (None, Some((frame, _))) => {
                let origin = chat::card_origin(&frame, size);
                self.chat.card_shown(origin[1], size[1]);
                origin
            }
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
            let width = chat::card_width(&frame);
            (frame, width)
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
            self.state_ask = Some((Instant::now(), ASK_TRIES));
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
            self.state_ask = None;
        }
        if self.voice.as_ref().is_none_or(|(client, _)| client != name) {
            return Ok(Vec::new());
        }
        self.voice = None;
        self.helpers.clear();
        self.switcher.set_agents("none", Instant::now());
        let mut commands = self.sync_assistant(Instant::now())?;
        commands.extend(self.sync_row()?);
        Ok(commands)
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
                let mut commands = commands;
                if self.in_pane {
                    commands.extend(a.island_dimmed(true));
                }
                self.assistant = Some(a);
                commands.extend(self.sync_task());
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

    /// The card in the view of the chat follows the view when the field
    /// changes its height, so that it stands above the hint. A re-layout is
    /// no new question: the guard of its buttons does not start again.
    pub fn card_follows(&mut self) -> Result<Vec<Command>, ClientError> {
        if self.chat.card_top().is_none() {
            return Ok(Vec::new());
        }
        let (Some(frame), Some((output, _))) = (self.chat_frame()?, self.output()?) else {
            return Ok(Vec::new());
        };
        let Some(mut shown) = self.card.take() else {
            return Ok(Vec::new());
        };
        let size = card::size(&self.card_theme, &shown, self.language, output.size);
        let origin = chat::card_origin(&frame, size);
        if !card::follow(&mut shown, origin) {
            self.card = Some(shown);
            return Ok(Vec::new());
        }
        self.chat.card_shown(origin[1], size[1]);
        let mut commands = card::hide(&shown);
        let (theme, language) = (&mut self.card_theme, self.language);
        commands.extend(card::show(theme, &shown, language, output.size, true));
        self.card = Some(shown);
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
                let answered = self.voice_call("confirm.answer", &params);
                if let Err(err) = &answered {
                    eprintln!("chora-assistant: voice: confirm.answer: {err}");
                    return Ok((true, Vec::new()));
                }
                Ok((true, self.close_card()))
            }
            Outcome::TooSoon => {
                eprintln!("chora-assistant: a press on {node} came too soon after the card");
                self.flash = Some((node.to_owned(), Instant::now() + card::FLASH));
                Ok((true, vec![card::pressed(node, true)]))
            }
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
/// The helper that an event of the agent is about: the `name` of a state
/// or a removal of a helper. A question names the helper that asks, for
/// the title of its card, and is the agent's own event.
fn helper_of<'a>(kind: Option<&str>, data: &'a Value, client: &str) -> Option<&'a str> {
    if kind == Some("confirm") {
        return None;
    }
    data.get("name")
        .and_then(Value::as_str)
        .filter(|name| *name != client)
}

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
    fn a_question_that_names_its_helper_is_a_question_of_the_agent() {
        let asked = serde_json::json!({"id": "51", "prompt": "Write notes.txt?", "name": "Atlas"});
        assert_eq!(helper_of(Some("confirm"), &asked, VOICE), None);
        let working = serde_json::json!({"name": "Atlas", "state": "working"});
        assert_eq!(helper_of(Some("state"), &working, VOICE), Some("Atlas"));
        assert_eq!(helper_of(Some("remove"), &working, VOICE), Some("Atlas"));
        let own = serde_json::json!({"name": VOICE, "state": "working"});
        assert_eq!(helper_of(Some("state"), &own, VOICE), None);
    }

    #[test]
    fn the_setting_position_names_a_corner_anchor() {
        assert_eq!(anchor_of(Corner::TopLeft), Anchor::TopLeft);
        assert_eq!(anchor_of(Corner::BottomRight), Anchor::BottomRight);
    }
}
