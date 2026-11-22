//! The voice events, the questions and the lines of the chat session,
//! split from the root of the chat.

use chora_kit::{
    chat::{Line, Measure},
    words::{self, Language},
};
use serde_json::Value;

use super::{
    Answered, MAX_STATES, Mode, ONE_LINE_FIELD, Outside, PULLS, Pick, Question, Role, Session,
    Status, Transcript, Where, Word, json, names, sanitize, say, show_key, view, wrap,
};

impl Session {
    /// A voice event; the view is marked for drawing if it changed.
    pub fn voice_event(&mut self, out: &mut dyn Outside, event: &str, data: &Value) {
        if event == "voice.remove"
            && let Some(name) = data["name"].as_str()
        {
            self.states.remove(&name.to_lowercase());
            if self
                .helper()
                .is_some_and(|open| open.eq_ignore_ascii_case(name))
            {
                self.status.state = "done".into();
                self.status.task.clear();
                self.status.steps = None;
                self.dirty = true;
            }
            return;
        }
        if event == "voice.state"
            && let Some(name) = data["name"].as_str()
            && (self.states.len() < MAX_STATES || self.states.contains_key(&name.to_lowercase()))
        {
            self.states.insert(name.to_lowercase(), data.clone());
        }
        let Some(helper) = self.helper().map(str::to_owned) else {
            return;
        };
        if let Some(link) = self.link.as_mut() {
            link.voice_event(event, data, &helper);
        }
        let changed = match event {
            "voice.state" if names(data, &helper) => self.helper_state(out, data),
            "voice.confirm" => {
                if let Some(question) = Question::from_event(data, None) {
                    self.keys = question.keys;
                }
                self.card.is_some()
            }
            _ => false,
        };
        self.dirty |= changed;
    }

    pub(super) fn helper_state(&mut self, out: &mut dyn Outside, data: &Value) -> bool {
        let text = |key: &str| data[key].as_str().unwrap_or_default();
        let steps = data["step"].as_u64().zip(data["steps"].as_u64());
        let status = Status {
            name: sanitize(text("name")),
            state: text("state").to_owned(),
            task: sanitize(text("task")),
            steps,
        };
        let mut changed = status.state != self.status.state
            || status.task != self.status.task
            || status.steps != self.status.steps;
        self.status = status;
        let epoch = data["epoch"].as_u64();
        if epoch.is_some() && self.epoch.is_some() && epoch != self.epoch {
            self.transcript = Transcript::default();
            changed = true;
        }
        self.epoch = epoch.or(self.epoch);
        self.head = data["seq"].as_u64().unwrap_or(self.head);
        if self.head > self.transcript.after() {
            changed |= self.pull(out);
        }
        changed
    }

    /// A question of `agent` is about to show, before its card is placed:
    /// when it is the open helper's, or names none, the state is question
    /// at once, so the card and the hint show in the same frame.
    pub fn question_of(&mut self, agent: Option<&str>) {
        let ours = match (&self.mode, agent) {
            (Mode::Chat(open), Some(agent)) => open.eq_ignore_ascii_case(agent),
            (Mode::Chat(_), None) => true,
            _ => false,
        };
        if ours {
            "question".clone_into(&mut self.status.state);
        }
    }

    /// A question appeared: where its card goes, fixed until it ends.
    /// The field of a chat holds Enter while it waits. The field of the
    /// list does not, as in a list that opens after the question: the
    /// list has no helper whose question it could be, and its Enter picks.
    pub fn question_asked(&mut self) -> Where {
        self.answered = Answered::No;
        if self.helper().is_some()
            && let Some(link) = self.link.as_mut()
        {
            link.question(true);
        }
        let place = if self.is_open() {
            Where::Chat
        } else {
            Where::Figure
        };
        *self.card.insert(place)
    }

    /// The question ended.
    pub fn question_ended(&mut self) {
        self.answered = Answered::Yes;
        if let Some(link) = self.link.as_mut() {
            link.question(false);
        }
        self.card = None;
        self.card_height = None;
        self.card_top = None;
        self.dirty = true;
    }

    /// Where the card of the question that waits stays.
    #[cfg(test)]
    #[must_use]
    pub fn card_place(&self) -> Option<Where> {
        self.card
    }

    pub(super) fn label(&self) -> String {
        match &self.mode {
            Mode::Chat(name) if self.status.name.is_empty() => name.clone(),
            _ => self.status.name.clone(),
        }
    }

    /// The header line of the view.
    #[must_use]
    pub fn header(&self, language: Language) -> String {
        match &self.mode {
            Mode::Closed => String::new(),
            _ if self.card == Some(Where::Figure) => say(language, Word::Waiting, &[]),
            _ if self.offline => say(language, Word::Offline, &[]),
            Mode::List(picks, _) if picks.is_empty() => say(language, Word::NoHelper, &[]),
            Mode::List(..) => say(language, Word::ChatLauncher, &[]),
            Mode::Chat(_) => {
                let row = words::row(
                    language,
                    &self.label(),
                    &self.status.state,
                    &self.status.task,
                );
                match self.status.steps {
                    Some((step, steps)) => format!("{row} ({step}/{steps})"),
                    None => row,
                }
            }
        }
    }

    /// The header of the XP form in two lines: the name and the state
    /// word, then the task and the step. Outside of a chat the words of
    /// [`Self::header`] take the lines: their first sentence, then the
    /// rest.
    #[must_use]
    pub fn header_xp(&self, language: Language) -> (String, String) {
        let (Mode::Chat(_), false) = (&self.mode, self.card == Some(Where::Figure) || self.offline)
        else {
            let words = self.header(language);
            return match words.split_once(". ") {
                Some((first, rest)) => (format!("{first}."), rest.to_owned()),
                None => (words, String::new()),
            };
        };
        let state = words::state(&self.status.state).map_or_else(
            || self.status.state.clone(),
            |word| word.text(language).to_owned(),
        );
        let head = if state.is_empty() {
            self.label()
        } else {
            format!("{} \u{2014} {state}", self.label())
        };
        let step = self.status.steps.map(|(step, steps)| {
            say(
                language,
                Word::StepOf,
                &[("step", &step.to_string()), ("steps", &steps.to_string())],
            )
        });
        let sub = match (self.status.task.is_empty(), step) {
            (true, step) => step.unwrap_or_default(),
            (false, Some(step)) => format!("{}, {step}", self.status.task),
            (false, None) => self.status.task.clone(),
        };
        (head, sub)
    }

    /// The numbered entries of the list, empty outside of it.
    #[must_use]
    pub fn entries(&self, language: Language) -> Vec<String> {
        match &self.mode {
            Mode::List(picks, _) => (1..)
                .zip(picks)
                .map(|(n, pick)| format!("{n}  {}", self.entry(pick, language)))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The words of an entry of the list: a helper by its name as voice
    /// writes it, with its state word when voice named one.
    fn entry(&self, pick: &Pick, language: Language) -> String {
        let Pick::Agent(name) = pick else {
            return pick.label(language);
        };
        let Some(data) = self.states.get(&name.to_lowercase()) else {
            return name.clone();
        };
        let written = data["name"].as_str().map_or_else(|| name.clone(), sanitize);
        match data["state"].as_str().filter(|state| !state.is_empty()) {
            Some(state) => words::row(language, &written, state, ""),
            None => written,
        }
    }

    /// The hint above the field: while a question waits, that typing does
    /// not answer, with its keys unless the card in the view names them.
    #[must_use]
    pub fn hint(&self, language: Language) -> Option<String> {
        let waits = match self.mode {
            Mode::Chat(_) => self.waiting(),
            Mode::List(..) => self.card.is_some(),
            Mode::Closed => false,
        };
        waits.then(|| {
            if self.card == Some(Where::Chat) {
                return say(language, Word::TypingNoAnswer, &[]);
            }
            let (yes, no) = (show_key(&self.keys.yes), show_key(&self.keys.no));
            say(language, Word::TypedYes, &[("yes", &yes), ("no", &no)])
        })
    }

    /// The transcript as lines, none outside of a chat with a helper, and
    /// then what the agent said into the open balloon.
    pub fn lines(&mut self, language: Language, measure: &Measure) -> Vec<Line> {
        let mut lines = if matches!(self.mode, Mode::Chat(_)) {
            let label = self.label();
            self.transcript.lines(language, &label, measure)
        } else {
            Vec::new()
        };
        if matches!(self.mode, Mode::Chat(_)) {
            lines.extend(self.echoes(language, measure));
        }
        let spoken = self
            .spoken
            .as_deref()
            .map(|text| wrap(text, measure.max, measure.width));
        lines.extend(spoken.into_iter().flatten().map(|text| Line {
            role: Role::Notice,
            label: None,
            text,
        }));
        lines
    }

    /// The lines of what the person sent that the transcript does not hold
    /// yet: as the transcript will show it, the label over the first line.
    fn echoes(&self, language: Language, measure: &Measure) -> Vec<Line> {
        let label = say(language, Word::You, &[]);
        let mut lines = Vec::new();
        for (text, _) in &self.sent {
            for (n, part) in wrap(text, measure.max, measure.width)
                .into_iter()
                .enumerate()
            {
                lines.push(Line {
                    role: Role::User,
                    label: (n == 0).then(|| label.clone()),
                    text: part,
                });
            }
        }
        lines
    }

    /// Drops what the person sent once the transcript holds it.
    fn settle_sent(&mut self) {
        let entries = self.transcript.entries();
        self.sent.retain(|(text, after)| {
            !entries.iter().any(|entry| {
                entry.role == Role::User
                    && entry.seq.is_some_and(|seq| seq > *after)
                    && entry.text == *text
            })
        });
    }

    /// The height of the field: as it told it, else one line.
    #[must_use]
    pub fn field_height(&self) -> i32 {
        self.field_height.unwrap_or(ONE_LINE_FIELD)
    }

    /// The card of the question stands in the view, this high, its top at
    /// `top` on the output.
    pub fn card_shown(&mut self, top: i32, height: i32) {
        self.card_height = Some(height);
        self.card_top = Some(top);
        self.dirty = true;
    }

    /// The top of the card in the view, on the output.
    #[must_use]
    pub fn card_top(&self) -> Option<i32> {
        self.card_top.filter(|_| self.card == Some(Where::Chat))
    }

    /// The room that the card takes in the view, with its gap.
    #[must_use]
    pub fn card_room(&self) -> i32 {
        self.card_height
            .filter(|_| self.card == Some(Where::Chat))
            .map_or(0, |height| height + view::GAP)
    }

    /// A sentence of the agent, which shows in the view while it is the
    /// open balloon of the figure, until the view closes.
    pub fn agent_said(&mut self, text: &str) {
        self.spoken = Some(sanitize(text));
        self.dirty = true;
    }

    /// Asks voice again after it did not answer: the chat opens again and
    /// the transcript is pulled. True when the view changed.
    pub fn retry(&mut self, out: &mut dyn Outside) -> bool {
        if !self.offline {
            return false;
        }
        match &self.mode {
            Mode::Chat(name) => {
                let name = name.clone();
                self.announce_open(out, &name);
            }
            Mode::List(..) => self.open_list(out),
            Mode::Closed => return false,
        }
        !self.offline
    }

    /// Pulls the transcript from voice, until it is up to date. True when
    /// it grew or when voice stopped answering.
    pub fn pull(&mut self, out: &mut dyn Outside) -> bool {
        let Some(name) = self.helper().map(str::to_owned) else {
            return false;
        };
        let before = (self.transcript.after(), self.offline);
        for _ in 0..PULLS {
            let after = self.transcript.after();
            let params = json!({"name": name, "after": after});
            let answer = match out.voice("agent.read", params) {
                Ok(answer) => answer,
                Err(error) => {
                    self.refused_or_offline(&error);
                    break;
                }
            };
            self.offline = false;
            self.epoch = answer["epoch"].as_u64().or(self.epoch);
            self.transcript.apply(&answer);
            self.settle_sent();
            if self.transcript.after() == after || self.transcript.after() >= self.head {
                break;
            }
        }
        before != (self.transcript.after(), self.offline)
    }
}
