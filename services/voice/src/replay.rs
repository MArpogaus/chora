//! The replay harness (docs/voice.md section 13): runs the speech gate
//! and the barge-in gate over a recording and compares them with its
//! labels.
//!
//! A recording is a directory in the format of `record_dir`: `mic.wav`,
//! `events.jsonl` (optional; its `play` lines mark the agent's sound) and
//! `labels.txt`, one label per line, times in s of `mic.wav`:
//!
//! ```text
//! # a comment
//! turn 1.09 2.83    the user speaks from 1.09 s to 2.83 s
//! barge 2.08        the user starts to speak over the agent
//! ```

use std::{fmt, ops::Range, path::Path};

use crate::{
    audio::{Analyzer, FRAME},
    config::Config,
    gate::{self, BargeIn, FRAME_MS, Gate},
    pcm::{self, RATE},
    vad::Vad,
};

/// A detection matches a labelled barge-in from 100 ms before it to 1 s
/// after it.
const BARGE_WINDOW: (f64, f64) = (-0.1, 1.0);
/// The pre-roll of an utterance that a barge-in starts (section 4).
const BARGE_PREROLL_MS: u32 = 100;

/// Turns and barge-ins, in s.
#[derive(Debug, Default, PartialEq)]
pub struct Marks {
    pub turns: Vec<(f64, f64)>,
    pub barges: Vec<f64>,
}

/// Reads `labels.txt`.
///
/// # Errors
///
/// A line is neither a comment nor a label.
pub fn labels(text: &str) -> Result<Marks, String> {
    let mut marks = Marks::default();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or_default();
        let words: Vec<&str> = line.split_whitespace().collect();
        let times: Option<Vec<f64>> = words.iter().skip(1).map(|w| w.parse().ok()).collect();
        match (words.first(), times.as_deref()) {
            (None, _) => {}
            (Some(&"turn"), Some(&[start, end])) if start < end => marks.turns.push((start, end)),
            (Some(&"barge"), Some(&[at])) => marks.barges.push(at),
            _ => {
                return Err(format!(
                    "line {}: {line:?}: turn START END or barge AT",
                    n + 1
                ));
            }
        }
    }
    Ok(marks)
}

/// A recording with its labels.
pub struct Recording {
    pub mic: Vec<i16>,
    /// The agent's sound, in mic samples.
    pub plays: Vec<Range<usize>>,
    pub labels: Marks,
}

impl Recording {
    /// Loads a recording directory.
    ///
    /// # Errors
    ///
    /// A file is missing or wrong.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let (rate, mic) = pcm::read_wav(&dir.join("mic.wav")).map_err(|e| e.to_string())?;
        if rate != RATE {
            return Err(format!("{}: {rate} Hz; replay takes 24000", dir.display()));
        }
        let text = |name: &str| chora_kit::read_text(&dir.join(name), 64 << 20);
        let labels = text("labels.txt")
            .map_err(|e| format!("{}/labels.txt: {e}", dir.display()))
            .and_then(|t| labels(&t).map_err(|e| format!("{}/labels.txt: {e}", dir.display())))?;
        let mut plays = Vec::new();
        for line in text("events.jsonl").unwrap_or_default().lines() {
            let event: serde_json::Value = serde_json::from_str(line)
                .map_err(|e| format!("{}/events.jsonl: {e}", dir.display()))?;
            let field = |key: &str| event[key].as_u64().and_then(|v| usize::try_from(v).ok());
            if event["event"] == "play"
                && let (Some(at), Some(len)) = (field("at"), field("len"))
            {
                plays.push(at..at + len);
            }
        }
        Ok(Self { mic, plays, labels })
    }
}

/// Runs the gates over the recording as the agent would: while the agent
/// plays, only the barge-in gate listens; a barge-in stops the playback
/// and starts an utterance.
///
/// # Errors
///
/// The VAD fails.
pub fn detect(config: &Config, rec: &Recording) -> Result<Marks, String> {
    let vad = Vad::new().map_err(|e| format!("the VAD model: {e}"))?;
    let (mut analyzer, mut gate, mut barge) =
        (Analyzer::new(vad), Gate::new(config), BargeIn::new(config));
    let onset_s = seconds(config.onset_ms.div_ceil(FRAME_MS) * FRAME_MS);
    let barge_s = seconds(config.barge_in_ms.div_ceil(FRAME_MS) * FRAME_MS);
    let mut found = Marks::default();
    let (mut start, mut stopped) = (None, None);
    for (i, chunk) in rec.mic.as_chunks::<FRAME>().0.iter().enumerate() {
        let frame = analyzer.frame(chunk.to_vec())?;
        let now = seconds(u32::try_from(i + 1).unwrap_or(u32::MAX) * FRAME_MS);
        let play = rec.plays.iter().position(|p| p.contains(&(i * FRAME)));
        let playing = play.is_some() && play != stopped;
        if !playing {
            barge.reset();
        }
        if playing && barge.push(&frame) {
            found.barges.push(now);
            stopped = play;
            gate.reset();
            gate.open(BARGE_PREROLL_MS);
            start = Some(now - barge_s);
            continue;
        }
        match gate.push(&frame, !playing, false) {
            Some(gate::Event::Onset) => start = Some(now - onset_s),
            Some(gate::Event::Noise { .. }) => start = None,
            Some(gate::Event::End { end_wait_ms, .. }) => {
                if let Some(start) = start.take() {
                    found.turns.push((start, now - seconds(end_wait_ms)));
                }
            }
            None => {}
        }
    }
    Ok(found)
}

fn seconds(ms: u32) -> f64 {
    f64::from(ms) / 1000.0
}

/// The metrics of one or more recordings.
#[derive(Debug, Default)]
pub struct Metrics {
    pub turns: usize,
    pub missed_turns: usize,
    pub false_turns: usize,
    /// Detected minus labelled start and end of the matched turns, in s.
    pub onset_errors: Vec<f64>,
    pub offset_errors: Vec<f64>,
    pub barges: usize,
    pub missed_barges: usize,
    pub false_barges: usize,
    /// Detection minus label of the matched barge-ins, in s.
    pub barge_latencies: Vec<f64>,
    /// The length of the agent's sound, in s.
    pub played_s: f64,
}

impl Metrics {
    /// Compares the detections with the labels. A detected turn matches
    /// the first unmatched labelled turn that it overlaps.
    #[must_use]
    pub fn new(labels: &Marks, found: &Marks, played_s: f64) -> Self {
        let mut m = Self {
            turns: labels.turns.len(),
            barges: labels.barges.len(),
            played_s,
            ..Self::default()
        };
        let mut used = vec![false; labels.turns.len()];
        for &(start, end) in &found.turns {
            let hit = (0..labels.turns.len()).find(|&k| {
                let (s, e) = labels.turns[k];
                !used[k] && start < e && s < end
            });
            let Some(k) = hit else {
                m.false_turns += 1;
                continue;
            };
            used[k] = true;
            m.onset_errors.push(start - labels.turns[k].0);
            m.offset_errors.push(end - labels.turns[k].1);
        }
        m.missed_turns = used.iter().filter(|u| !**u).count();
        let mut used = vec![false; labels.barges.len()];
        for &at in &found.barges {
            let hit = (0..labels.barges.len()).find(|&k| {
                let late = at - labels.barges[k];
                !used[k] && (BARGE_WINDOW.0..=BARGE_WINDOW.1).contains(&late)
            });
            let Some(k) = hit else {
                m.false_barges += 1;
                continue;
            };
            used[k] = true;
            m.barge_latencies.push(at - labels.barges[k]);
        }
        m.missed_barges = used.iter().filter(|u| !**u).count();
        m
    }

    /// Adds the metrics of another recording.
    pub fn add(&mut self, other: Self) {
        self.turns += other.turns;
        self.missed_turns += other.missed_turns;
        self.false_turns += other.false_turns;
        self.onset_errors.extend(other.onset_errors);
        self.offset_errors.extend(other.offset_errors);
        self.barges += other.barges;
        self.missed_barges += other.missed_barges;
        self.false_barges += other.false_barges;
        self.barge_latencies.extend(other.barge_latencies);
        self.played_s += other.played_s;
    }

    /// False barge-ins per minute of the agent's sound.
    #[must_use]
    pub fn false_barges_per_min(&self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let n = self.false_barges as f64;
        if self.played_s > 0.0 {
            n * 60.0 / self.played_s
        } else {
            0.0
        }
    }
}

/// The mean of the absolute values, in ms; 0 for none.
#[must_use]
pub fn mean_abs_ms(values: &[f64]) -> f64 {
    #[allow(clippy::cast_precision_loss)]
    let n = values.len().max(1) as f64;
    values.iter().fold(0.0, |s, v| s + v.abs()) * 1000.0 / n
}

/// The largest absolute value, in ms; 0 for none.
#[must_use]
pub fn max_abs_ms(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |m, v| f64::max(m, v.abs())) * 1000.0
}

impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "turns={} missed={} false={} onset_err={:.0}ms (max {:.0}) \
             offset_err={:.0}ms (max {:.0}) barges={} missed={} false={} \
             ({:.1}/min of {:.1}s played) barge_latency={:.0}ms (max {:.0})",
            self.turns,
            self.missed_turns,
            self.false_turns,
            mean_abs_ms(&self.onset_errors),
            max_abs_ms(&self.onset_errors),
            mean_abs_ms(&self.offset_errors),
            max_abs_ms(&self.offset_errors),
            self.barges,
            self.missed_barges,
            self.false_barges,
            self.false_barges_per_min(),
            self.played_s,
            mean_abs_ms(&self.barge_latencies),
            max_abs_ms(&self.barge_latencies),
        )
    }
}

/// Replays each recording; one line per recording, then the sum.
///
/// # Errors
///
/// A recording does not load, or the VAD fails.
pub fn run(config: &Config, dirs: &[&Path]) -> Result<(Vec<String>, Metrics), String> {
    let mut lines = Vec::new();
    let mut all = Metrics::default();
    for dir in dirs {
        let rec = Recording::load(dir)?;
        let found = detect(config, &rec)?;
        let played: usize = rec.plays.iter().map(ExactSizeIterator::len).sum();
        #[allow(clippy::cast_precision_loss)]
        let played_s = played as f64 / f64::from(RATE);
        let m = Metrics::new(&rec.labels, &found, played_s);
        lines.push(format!("{}: {m}", dir.display()));
        all.add(m);
    }
    lines.push(format!("all: {all}"));
    Ok((lines, all))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_parse_with_comments_and_reject_mistakes() {
        let marks = labels("# note\nturn 1.0 2.5 # the user\n\nbarge 3\n").unwrap();
        assert_eq!(
            marks,
            Marks {
                turns: vec![(1.0, 2.5)],
                barges: vec![3.0]
            }
        );
        for bad in ["turn 2 1", "turn 1", "barge x", "talk 1 2"] {
            assert!(labels(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn metrics_count_missed_false_and_errors() {
        let labels = Marks {
            turns: vec![(1.0, 2.0), (4.0, 5.0)],
            barges: vec![3.0, 8.0],
        };
        let found = Marks {
            turns: vec![(1.1, 2.3), (1.9, 2.2), (6.0, 7.0)],
            barges: vec![3.25, 10.0],
        };
        let m = Metrics::new(&labels, &found, 30.0);
        assert_eq!(
            (
                m.missed_turns,
                m.false_turns,
                m.missed_barges,
                m.false_barges
            ),
            (1, 2, 1, 1)
        );
        assert!((mean_abs_ms(&m.onset_errors) - 100.0).abs() < 1e-6);
        assert!((mean_abs_ms(&m.offset_errors) - 300.0).abs() < 1e-6);
        assert!((mean_abs_ms(&m.barge_latencies) - 250.0).abs() < 1e-6);
        assert!((m.false_barges_per_min() - 2.0).abs() < 1e-9);
    }

    /// The CI corpus of tests/data/corpus (written by tests/corpus.py):
    /// the thresholds hold the gate at least as good as now.
    #[test]
    fn the_corpus_meets_the_thresholds() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/corpus");
        let mut dirs: Vec<_> = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        dirs.sort();
        let dirs: Vec<&Path> = dirs.iter().map(AsRef::as_ref).collect();
        let (lines, m) = run(&Config::default(), &dirs).unwrap();
        let report = lines.join("\n");
        assert!(m.turns >= 4 && m.barges >= 1, "{report}");
        assert_eq!((m.missed_turns, m.missed_barges), (0, 0), "{report}");
        // Known: the echo of RMS 600 in echo-barge is speech for Silero,
        // so it never teaches the echo level, and it barges in and opens a
        // turn. An echo level from the played sound (docs/voice.md, open
        // questions) would bring both to 0.
        assert!(m.false_barges <= 1 && m.false_turns <= 1, "{report}");
        assert!(max_abs_ms(&m.onset_errors) <= 200.0, "{report}");
        assert!(max_abs_ms(&m.offset_errors) <= 200.0, "{report}");
        assert!(max_abs_ms(&m.barge_latencies) <= 450.0, "{report}");
    }
}
