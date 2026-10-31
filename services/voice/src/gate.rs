//! The speech gate: Silero probabilities to utterances, with onset,
//! release, pre-roll, the push-to-talk key and the limits of a turn
//! (docs/voice.md section 4).

use std::collections::VecDeque;

use crate::config::Config;

/// The length of a frame: 768 samples at 24 kHz.
pub const FRAME_MS: u32 = 32;

/// A mic frame with its speech probability and level.
#[derive(Clone, Debug)]
pub struct Frame {
    pub samples: Vec<i16>,
    pub prob: f32,
}

/// What a frame did to the gate.
#[derive(Debug, PartialEq)]
pub enum Event {
    /// An utterance started.
    Onset,
    /// An utterance ended: its PCM with the pre-roll, its speech, and the
    /// wait from the last speech to the end.
    End {
        pcm: Vec<i16>,
        speech_ms: u32,
        end_wait_ms: u32,
    },
    /// An utterance ended with too little speech, for example a breath.
    Noise { speech_ms: u32 },
}

/// The settings of the gate, in frames.
#[derive(Clone, Debug, PartialEq)]
struct Limits {
    threshold: f64,
    release: f64,
    onset: u32,
    silence: u32,
    min_speech: u32,
    max_turn: u32,
    preroll: usize,
    /// The speech that starts a barge-in.
    barge: usize,
}

impl From<&Config> for Limits {
    fn from(c: &Config) -> Self {
        let frames = |ms: u32| ms.div_ceil(FRAME_MS);
        Self {
            threshold: c.vad_threshold,
            release: c.vad_release,
            onset: frames(c.onset_ms).max(1),
            silence: frames(c.silence_ms).max(1),
            min_speech: frames(c.min_speech_ms),
            max_turn: frames(c.max_turn_ms).max(1),
            preroll: frames(c.preroll_ms) as usize,
            barge: frames(c.barge_in_ms) as usize,
        }
    }
}

struct Utterance {
    pcm: Vec<i16>,
    frames: u32,
    speech: u32,
    quiet: u32,
    /// Started or held by the key: only the key ends it.
    by_key: bool,
}

/// The speech gate.
pub struct Gate {
    limits: Limits,
    /// The frames before an onset: the onset frames and the pre-roll.
    before: VecDeque<Vec<i16>>,
    /// The frames over the threshold in a row.
    run: u32,
    utterance: Option<Utterance>,
}

impl Gate {
    pub fn new(config: &Config) -> Self {
        Self {
            limits: config.into(),
            before: VecDeque::new(),
            run: 0,
            utterance: None,
        }
    }

    /// Takes new settings; an utterance in progress goes on.
    pub fn configure(&mut self, config: &Config) {
        self.limits = config.into();
    }

    /// Drops an utterance in progress.
    pub fn reset(&mut self) {
        self.utterance = None;
        self.run = 0;
    }

    /// The PCM of the utterance in progress.
    pub fn pcm(&self) -> Option<&[i16]> {
        self.utterance.as_ref().map(|u| u.pcm.as_slice())
    }

    /// Starts an utterance now, for example on a barge-in, with at most
    /// `preroll_ms` of the frames before.
    pub fn open(&mut self, preroll_ms: u32) {
        let take = (preroll_ms.div_ceil(FRAME_MS) as usize).min(self.before.len());
        let pcm = self.before.iter().skip(self.before.len() - take).flatten();
        self.utterance = Some(Utterance {
            pcm: pcm.copied().collect(),
            frames: 1,
            speech: 1,
            quiet: 0,
            by_key: false,
        });
        self.before.clear();
        self.run = 0;
    }

    /// Takes the next frame. `listen` lets speech start an utterance; it
    /// is false, for example, while the mic is muted. `key` is the
    /// push-to-talk key: down starts an utterance at once, and up ends
    /// one that it started or held.
    pub fn push(&mut self, frame: &Frame, listen: bool, key: bool) -> Option<Event> {
        let l = &self.limits;
        let Some(u) = &mut self.utterance else {
            self.before.push_back(frame.samples.clone());
            let keep = (l.onset as usize).max(l.barge) + l.preroll;
            while self.before.len() > keep {
                self.before.pop_front();
            }
            self.run = if listen && f64::from(frame.prob) > l.threshold {
                self.run + 1
            } else {
                0
            };
            if !key && self.run < l.onset {
                return None;
            }
            // The onset frames, and the pre-roll before them.
            let take = (self.run as usize + l.preroll).min(self.before.len());
            let pcm = self
                .before
                .iter()
                .skip(self.before.len() - take)
                .flatten()
                .copied();
            self.utterance = Some(Utterance {
                pcm: pcm.collect(),
                frames: self.run.max(1),
                speech: self.run,
                quiet: 0,
                by_key: key,
            });
            self.before.clear();
            self.run = 0;
            return Some(Event::Onset);
        };
        u.pcm.extend_from_slice(&frame.samples);
        u.frames += 1;
        u.by_key |= key;
        // The key counts as speech: a held key is an explicit request.
        if key || (listen && f64::from(frame.prob) >= l.release) {
            u.speech += 1;
            u.quiet = 0;
        } else {
            u.quiet += 1;
        }
        let ended = if u.by_key {
            !key
        } else {
            u.quiet >= l.silence || u.frames >= l.max_turn
        };
        if !ended {
            return None;
        }
        let u = self.utterance.take()?;
        let speech_ms = u.speech * FRAME_MS;
        if u.speech < l.min_speech {
            return Some(Event::Noise { speech_ms });
        }
        Some(Event::End {
            pcm: u.pcm,
            speech_ms,
            end_wait_ms: u.quiet * FRAME_MS,
        })
    }
}

/// The frames of played sound, before and with a mic frame, whose
/// loudest one is the reference of the echo: the sound reaches the mic
/// a little later than it is written.
pub const ECHO_WINDOW: usize = 5;

/// Played sound under this level teaches the coupling nothing.
const LEARN_FROM: f64 = 200.0;
/// The step of the coupling per frame, and the step up on a frame that
/// counts as speech.
const LEARN_STEP: f64 = 1.05;
const LEARN_STEP_SPEECH: f64 = 1.01;
/// The lowest coupling: the `PipeWire` echo canceller leaves an echo of
/// RMS 0.1 for a played level of 4000 (live recording 2026-10-31).
const MIN_COUPLING: f64 = 0.000_01;

/// The adaptive level floor of the barge-in: this part of the median
/// level of the last `TURNS` turns, at least `MIN_FLOOR`; `FIRST_FLOOR`
/// before the first turn.
const FLOOR_PART: f64 = 0.4;
const TURNS: usize = 5;
const MIN_FLOOR: f64 = 5.0;
const FIRST_FLOOR: f64 = 15.0;

/// The median; none for no values.
fn median(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut v: Vec<f64> = values.into_iter().collect();
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied()
}

/// The barge-in gate (docs/voice.md section 4): speech over the agent's
/// own sound. The echo level is the played level (the loudest frame of
/// the last `ECHO_WINDOW`) times a coupling that the gate learns.
pub struct BargeIn {
    threshold: f64,
    ratio: f64,
    min_rms: f64,
    frames: u32,
    /// The mic level over the played level, learned while the agent
    /// speaks.
    coupling: f64,
    run: u32,
    /// The median frame level of the user's last turns.
    turns: VecDeque<f64>,
}

impl BargeIn {
    pub fn new(c: &Config) -> Self {
        Self {
            threshold: c.barge_in_vad_threshold,
            ratio: c.barge_in_echo_ratio,
            min_rms: c.barge_in_min_rms,
            frames: c.barge_in_ms.div_ceil(FRAME_MS).max(1),
            coupling: c.barge_in_coupling,
            run: 0,
            turns: VecDeque::new(),
        }
    }

    /// Takes new settings; the learned coupling and levels stay.
    pub fn configure(&mut self, c: &Config) {
        let (coupling, turns) = (self.coupling, std::mem::take(&mut self.turns));
        *self = Self::new(c);
        (self.coupling, self.turns) = (coupling, turns);
    }

    /// Learns the level of the user's speech from the PCM of a turn.
    pub fn heard(&mut self, pcm: &[i16]) {
        if let Some(level) = median(pcm.chunks(crate::audio::FRAME).map(crate::pcm::rms)) {
            self.turns.push_back(level);
            if self.turns.len() > TURNS {
                self.turns.pop_front();
            }
        }
    }

    /// The level that speech must pass besides the echo:
    /// `barge_in_min_rms`, or else adaptive.
    pub fn floor(&self) -> f64 {
        if self.min_rms > 0.0 {
            return self.min_rms;
        }
        median(self.turns.iter().copied()).map_or(FIRST_FLOOR, |m| (m * FLOOR_PART).max(MIN_FLOOR))
    }

    /// The learned coupling.
    pub fn coupling(&self) -> f64 {
        self.coupling
    }

    /// Takes a mic frame during playback and the played level with it;
    /// true once the speech over the gate lasted `barge_in_ms`.
    pub fn push(&mut self, frame: &Frame, played: f64) -> bool {
        let rms = crate::pcm::rms(&frame.samples);
        let echo = self.coupling * played;
        let speech =
            f64::from(frame.prob) > self.threshold && rms > self.floor().max(echo * self.ratio);
        // ponytail: a median tracker of mic over played level. Frames
        // that count as speech raise it only slowly: the user's words
        // until the barge-in, or echo over a coupling that is still too
        // low. An adaptive filter on the played PCM would know more.
        if played > LEARN_FROM {
            let step = match (rms > echo, speech) {
                (false, _) => 1.0 / LEARN_STEP,
                (true, false) => LEARN_STEP,
                (true, true) => LEARN_STEP_SPEECH,
            };
            self.coupling = (self.coupling * step).clamp(MIN_COUPLING, 10.0);
        }
        self.run = if speech { self.run + 1 } else { 0 };
        self.run == self.frames
    }

    /// The playback ended: a new one counts from zero.
    pub fn reset(&mut self) {
        self.run = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N: usize = 768;

    fn frame(prob: f32, sample: i16) -> Frame {
        Frame {
            samples: vec![sample; N],
            prob,
        }
    }

    fn config() -> Config {
        Config {
            onset_ms: 64,
            silence_ms: 96,
            min_speech_ms: 128,
            max_turn_ms: 640,
            preroll_ms: 64,
            ..Config::default()
        }
    }

    /// Runs probabilities through a gate; frame i has the sample value i.
    fn run(gate: &mut Gate, probs: &[f32], key: &[bool]) -> Vec<(usize, Event)> {
        probs
            .iter()
            .enumerate()
            .filter_map(|(i, &p)| {
                let down = key.get(i).copied().unwrap_or(false);
                gate.push(&frame(p, i16::try_from(i).unwrap()), true, down)
                    .map(|e| (i, e))
            })
            .collect()
    }

    fn values(pcm: &[i16]) -> Vec<i16> {
        pcm.chunks(N).map(|c| c[0]).collect()
    }

    #[test]
    fn speech_starts_after_onset_and_keeps_the_preroll() {
        let mut gate = Gate::new(&config());
        // Quiet 0..5, speech 5..12, quiet after.
        let probs = [[0.1; 5].as_slice(), &[0.9; 7], &[0.1; 6]].concat();
        let events = run(&mut gate, &probs, &[]);
        assert_eq!(events[0], (6, Event::Onset), "two frames of onset");
        let (
            at,
            Event::End {
                pcm,
                speech_ms,
                end_wait_ms,
            },
        ) = &events[1]
        else {
            panic!("{events:?}");
        };
        assert_eq!(*at, 14, "three quiet frames end it");
        assert_eq!(
            values(pcm),
            (3..=14).collect::<Vec<_>>(),
            "two frames of pre-roll"
        );
        assert_eq!((*speech_ms, *end_wait_ms), (7 * 32, 3 * 32));
    }

    #[test]
    fn the_preroll_never_reaches_further_back_than_its_limit() {
        let mut gate = Gate::new(&Config {
            preroll_ms: 0,
            ..config()
        });
        let probs = [[0.1; 5].as_slice(), &[0.9; 7], &[0.1; 3]].concat();
        let events = run(&mut gate, &probs, &[]);
        let Event::End { pcm, .. } = &events[1].1 else {
            panic!("{events:?}");
        };
        assert_eq!(values(pcm)[0], 5, "starts at the first speech frame");
    }

    #[test]
    fn a_short_sound_is_noise_and_a_dip_does_not_end_speech() {
        let mut gate = Gate::new(&config());
        let probs = [0.9, 0.9, 0.9, 0.1, 0.1, 0.1];
        let events = run(&mut gate, &probs, &[]);
        assert_eq!(events[1].1, Event::Noise { speech_ms: 96 });
        // A dip of two frames under the release stays in one utterance.
        let probs = [0.9, 0.9, 0.9, 0.2, 0.2, 0.9, 0.9, 0.1, 0.1, 0.1];
        let events = run(&mut gate, &probs, &[]);
        assert_eq!(events.len(), 2, "{events:?}");
        assert!(matches!(events[1], (9, Event::End { speech_ms: 160, .. })));
    }

    #[test]
    fn a_long_turn_ends_at_max_turn() {
        let mut gate = Gate::new(&config());
        let events = run(&mut gate, &[0.9; 30], &[]);
        // Onset at frame 1 with 2 frames, then 18 more: 20 frames = 640 ms.
        assert!(matches!(events[1], (19, Event::End { .. })), "{events:?}");
    }

    #[test]
    fn the_key_records_without_speech_until_it_goes_up() {
        let mut gate = Gate::new(&config());
        let key = [false, true, true, true, true, true, true, false, false];
        let events = run(&mut gate, &[0.0; 9], &key);
        assert_eq!(events[0], (1, Event::Onset));
        let (at, Event::End { pcm, .. }) = &events[1] else {
            panic!("{events:?}");
        };
        assert_eq!(*at, 7, "the end comes with the key");
        assert_eq!(values(pcm), (0..=7).collect::<Vec<_>>());
    }

    #[test]
    fn open_starts_an_utterance_with_a_short_preroll() {
        let mut gate = Gate::new(&config());
        run(&mut gate, &[0.0; 4], &[]);
        gate.open(64);
        let events = run(&mut gate, &[0.9, 0.0, 0.0, 0.0], &[]);
        let (_, Event::Noise { speech_ms }) = events[0] else {
            panic!("{events:?}");
        };
        assert_eq!(speech_ms, 64, "the open frame and one of speech");
    }

    /// The frames of `level` until a barge-in fires, of `n`.
    fn fires(barge: &mut BargeIn, prob: f32, level: i16, played: f64, n: usize) -> usize {
        (0..n)
            .filter(|_| barge.push(&frame(prob, level), played))
            .count()
    }

    #[test]
    fn the_barge_in_floor_follows_the_level_of_the_turns() {
        let b = &mut BargeIn::new(&Config {
            barge_in_ms: 96,
            ..Config::default()
        });
        assert_eq!(fires(b, 0.9, 12, 0.0, 10), 0, "under the first floor");
        // Quiet turns (RMS 30) lower the floor to 12.
        for _ in 0..3 {
            b.heard(&[30; 768 * 4]);
        }
        assert!((b.floor() - 12.0).abs() < 1e-9, "{}", b.floor());
        assert_eq!(fires(b, 0.9, 20, 0.0, 10), 1, "quiet speech");
        assert_eq!(fires(b, 0.9, 11, 0.0, 10), 0, "under the floor");
    }

    #[test]
    fn barge_in_needs_loud_speech_for_barge_in_ms() {
        let b = &mut BargeIn::new(&Config {
            barge_in_ms: 96,
            barge_in_min_rms: 300.0,
            ..Config::default()
        });
        assert_eq!(fires(b, 0.9, 200, 0.0, 10), 0, "under barge_in_min_rms");
        assert_eq!(fires(b, 0.9, 1000, 0.0, 10), 1, "once, after 3 frames");
        // Echo that Silero calls speech teaches a coupling of 0.2.
        assert_eq!(fires(b, 0.9, 800, 4000.0, 100), 0, "loud echo");
        assert!((b.coupling() - 0.2).abs() < 0.02, "{}", b.coupling());
        assert_eq!(fires(b, 0.9, 1500, 4000.0, 3), 0, "under twice the echo");
        assert_eq!(fires(b, 0.9, 2000, 4000.0, 10), 1);
    }

    #[test]
    fn without_listen_speech_starts_nothing() {
        let mut gate = Gate::new(&config());
        let starts = (0..10)
            .filter_map(|_| gate.push(&frame(0.99, 0), false, false))
            .count();
        assert_eq!(starts, 0);
    }
}
