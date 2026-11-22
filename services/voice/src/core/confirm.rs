//! The question of the user: ask, answer, and the yes, no, key and
//! button that answer it.

use super::{
    BUTTON_DELAY, Control, Core, Duration, Earcon, Effect, KEY_DELAY, Ordering, Sender, State,
};

impl Core {
    /// Starts a reply to the queued texts and `text`; with `say`, a
    /// reply that says `text`.
    pub(super) fn ask(&mut self, text: String, say: bool) -> Vec<Effect> {
        let text = if say {
            text
        } else {
            if !text.is_empty() {
                self.queue.push(text);
            }
            std::mem::take(&mut self.queue).join(" ")
        };
        self.replies += 1;
        self.asking = Some(self.replies);
        if !say {
            self.request.clone_from(&text);
        }
        self.doing = State::Thinking;
        self.unclear = 0;
        self.played.clear();
        self.sound_end = None;
        vec![Effect::Ask {
            reply: self.replies,
            text,
            speak: !self.muted.speaker,
            ended: self.ended,
            say,
            voice: None,
        }]
    }

    /// Whether a confirmation waits: of the reply or of a sub-agent.
    pub(super) fn confirming(&self) -> bool {
        self.confirm.is_some() || self.sub_confirm.is_some()
    }

    /// Whether the confirmation that an answer goes to, that of the
    /// reply first, takes only a key.
    pub(super) fn needs_key(&self) -> bool {
        if self.confirm.is_some() {
            self.physical
        } else {
            self.sub_confirm.is_some() && self.sub_physical
        }
    }

    /// Answers the confirmation that waits, that of the reply first: yes,
    /// no, or unclear (`None`, a no to a sub-agent).
    pub(super) fn answer(&mut self, yes: Option<bool>) {
        let said = match yes {
            Some(true) => "yes",
            Some(false) => "no",
            None => "unclear",
        };
        if let Some(answer) = self.confirm.take() {
            crate::log::info(&format!("confirm: {said}"));
            let _ = answer.send(yes);
            self.cancel_prompt = Some(self.confirm_prompt.clone());
            self.doing = if yes == Some(true) {
                State::Tool
            } else {
                State::Thinking
            };
        } else if let Some((name, answer)) = self.sub_confirm.take() {
            crate::log::info(&format!("confirm {name}: {said}"));
            let _ = answer.send(yes == Some(true));
        }
    }

    /// A yes with an always word. A question of a sub-agent takes none:
    /// it has no action to remember, so the answer is a plain yes or
    /// the key.
    pub(super) fn always(&mut self) -> Vec<Effect> {
        if self.confirm.is_none() {
            crate::log::info("confirm: an always word answers no question of a sub-agent");
            return Vec::new();
        }
        self.confirm_always.store(true, Ordering::Relaxed);
        self.yes_no(true)
    }

    /// A spoken yes or no, or one of `voice.confirm`: the cut prompt
    /// does not play again. A yes does not answer a question that waits
    /// for a key; for a question of a helper it hears `phrases.need_key`.
    pub(super) fn yes_no(&mut self, yes: bool) -> Vec<Effect> {
        if yes && self.needs_key() {
            crate::log::info("confirm: a yes without the key counts as nothing");
            let mut effects: Vec<Effect> = self.earcon(Earcon::Error).into_iter().collect();
            if self.confirm.is_none() {
                effects.extend(self.ask(self.config.phrases.need_key.clone(), true));
            }
            return effects;
        }
        self.answered(yes)
    }

    /// A key of `confirm_keys` answers the question that waits for it,
    /// once the question showed for `KEY_DELAY`; an earlier press counts
    /// as nothing.
    pub(super) fn confirm_key(&mut self, yes: bool) -> Vec<Effect> {
        if !self.needs_key() {
            return Vec::new();
        }
        if self.too_soon(KEY_DELAY, "a key") {
            return Vec::new();
        }
        crate::log::info("confirm: by the key");
        self.answered(yes)
    }

    /// A button answers the question on the screen, also one that waits
    /// for a key: a click is a physical act as a key press is. Another
    /// id, such as that of a question that ended, answers nothing; nor
    /// does a click within `BUTTON_DELAY` of the question, also of one
    /// that replaced another.
    pub(super) fn confirm_button(
        &mut self,
        id: u64,
        yes: bool,
        open: &Sender<bool>,
    ) -> Vec<Effect> {
        let taken =
            self.confirming() && self.shown == Some(id) && !self.too_soon(BUTTON_DELAY, "a button");
        let _ = open.send(taken);
        if !taken {
            return Vec::new();
        }
        crate::log::info("confirm: by a button");
        self.answered(yes)
    }

    /// The question `id` cannot show whole: a no, with the error earcon;
    /// another id, such as that of a question that ended, is nothing. The
    /// card of a sub-agent question that a newer card replaced on the
    /// screen is a no for that question alone: it is never left to wait
    /// for an answer without a screen.
    pub(super) fn unshowable(&mut self, id: u64) -> Vec<Effect> {
        if self.sub_shown == Some(id) && self.shown != Some(id) {
            self.sub_shown = None;
            if let Some((name, answer)) = self.sub_confirm.take() {
                crate::log::info(&format!(
                    "confirm {name}: the replaced question is too long for the own HUD, a no"
                ));
                let _ = answer.send(false);
                return self.earcon(Earcon::Error).into_iter().collect();
            }
            return Vec::new();
        }
        if !self.confirming() || self.shown != Some(id) {
            return Vec::new();
        }
        crate::log::info("confirm: the question is too long for the own HUD, a no");
        let mut effects: Vec<Effect> = self.earcon(Earcon::Error).into_iter().collect();
        effects.extend(self.answered(false));
        effects
    }

    /// Whether the question on the screen showed for less than `delay`:
    /// a press `by` a key or a button then counts as nothing.
    pub(super) fn too_soon(&self, delay: Duration, by: &str) -> bool {
        let soon = self.shown_at.elapsed() < delay;
        if soon {
            crate::log::info(&format!(
                "confirm: {by} too soon after the question counts as nothing"
            ));
        }
        soon
    }

    pub(super) fn answered(&mut self, yes: bool) -> Vec<Effect> {
        self.answer(Some(yes));
        if !self.paused {
            return Vec::new();
        }
        self.paused = false;
        vec![Effect::Speech(Control::Resume { skip: true })]
    }
}
