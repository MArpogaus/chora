//! The audio frames: the level played, barge-in and the interim text.

use super::{
    Control, Core, Duration, ECHO_TAIL, ECHO_WINDOW, Effect, FRAME_MS, Frame, Instant, Listen,
    Pending, SOUND_GAP, gate,
};

impl Core {
    /// The loudest played frame near `now`: the echo reaches the mic a
    /// little later.
    pub(super) fn played_level(&mut self, now: Instant) -> f64 {
        let window = Duration::from_millis(u64::from(FRAME_MS) * ECHO_WINDOW as u64);
        while self
            .levels
            .front()
            .is_some_and(|(at, _)| *at + window < now)
        {
            self.levels.pop_front();
        }
        self.levels
            .iter()
            .take_while(|(at, _)| *at <= now)
            .fold(0.0, |max, (_, level)| f64::max(max, *level))
    }

    /// Whether the reply speaks: its sound plays, or ran dry between
    /// two sentences a moment ago. A question that waits for its answer
    /// is the last sentence: only its echo tail counts.
    pub(super) fn sounding(&self, now: Instant) -> bool {
        // ponytail: a fixed gap; the speech could report its queue.
        let gap = if self.confirm.is_some() {
            ECHO_TAIL
        } else {
            SOUND_GAP
        };
        self.sound_end.is_some_and(|end| now < end + gap)
    }

    /// Speech over the agent's sound pauses it and starts an utterance.
    pub(super) fn barge_in(&mut self, now: Instant, level: f64) -> Vec<Effect> {
        crate::log::info(&format!(
            "barge-in played={level:.0} coupling={:.3}",
            self.barge.coupling()
        ));
        self.paused = true;
        self.sound_end = Some(now);
        let barge = Duration::from_millis(self.config.barge_in_ms.into());
        self.onset = now.checked_sub(barge).unwrap_or(now);
        // The speech that made the barge-in, and the pre-roll before it.
        self.gate
            .open(self.config.barge_in_ms + self.config.preroll_ms);
        self.recorded = 0;
        vec![Effect::Speech(Control::Pause), Effect::Mark("barge")]
    }

    pub(super) fn frame(&mut self, frame: &Frame) -> Vec<Effect> {
        let now = Instant::now();
        let playing = self.asking.is_some() && !self.paused && self.sounding(now);
        // Push to talk: no speech opens the gate, only the key does.
        let vad = !self.muted.mic && self.config.listen == Listen::Always;
        if playing && self.gate.pcm().is_none() && !self.key.down {
            // Only the barge-in gate listens over the agent's sound.
            let level = self.played_level(now);
            if vad && self.barge.push(frame, level) {
                return self.barge_in(now, level);
            }
            // The gate keeps its pre-roll.
            self.gate.push(frame, false, false);
            return Vec::new();
        }
        self.barge.reset();
        self.key.used |= self.key.down;
        match self.gate.push(frame, vad, self.key.down) {
            None => self.interim().into_iter().collect(),
            Some(gate::Event::Onset) => {
                self.onset = now;
                self.recorded = 0;
                vec![Effect::Mark("onset")]
            }
            Some(gate::Event::Noise { speech_ms }) => {
                self.key.used = false;
                crate::log::debug(&format!("noise speech={speech_ms}ms"));
                vec![Effect::Mark("noise")]
            }
            Some(gate::Event::End {
                pcm,
                speech_ms,
                end_wait_ms,
            }) => {
                self.turn += 1;
                self.ended = now;
                self.barge.heard(&pcm);
                self.pending.push(Pending {
                    turn: self.turn,
                    speech_ms,
                    heard: self.awake || std::mem::take(&mut self.key.used),
                    onset: self.onset,
                });
                vec![
                    Effect::Mark("end"),
                    Effect::Utterance {
                        turn: self.turn,
                        pcm,
                        speech_ms,
                        end_wait_ms,
                        awake: self.awake,
                    },
                ]
            }
        }
    }

    /// The interim effect, every `asr_interim_ms` of an utterance.
    pub(super) fn interim(&mut self) -> Option<Effect> {
        let every = self.config.asr_interim_ms.div_ceil(FRAME_MS);
        let pcm = self.gate.pcm()?;
        self.recorded += 1;
        (every > 0 && self.recorded.is_multiple_of(every)).then(|| Effect::Interim {
            turn: self.turn + 1,
            pcm: pcm.to_vec(),
        })
    }
}
