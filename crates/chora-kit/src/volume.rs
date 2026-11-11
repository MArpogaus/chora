//! The volume of the default sound output, through `wpctl` of
//! `WirePlumber`, the session manager of `PipeWire`. Without `wpctl` or an
//! output there is no volume, and a change does nothing.

use std::process::{Command, Stdio};

use crate::notify::{Message, Notifier};

/// The default sound output of `wpctl`.
const SINK: &str = "@DEFAULT_AUDIO_SINK@";
/// One step of a volume key, in percent.
pub const STEP: u32 = 5;
/// How long the indicator of a volume key shows, in milliseconds.
const SHOWN_MS: i32 = 1500;

/// The volume of the default output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Volume {
    /// From 0 to 100.
    pub percent: u32,
    /// The output is muted.
    pub muted: bool,
}

/// A change of the volume.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    /// One step louder; it unmutes.
    Up,
    /// One step quieter.
    Down,
    /// Mutes or unmutes.
    Mute,
    /// Sets a volume in percent; it unmutes.
    Set(u32),
}

impl Volume {
    /// The words of the volume: `40%` or `Muted`.
    #[must_use]
    pub fn label(self) -> String {
        if self.muted {
            "Muted".into()
        } else {
            format!("{}%", self.percent)
        }
    }
}

/// The words of a volume, or `No sound` without one.
#[must_use]
pub fn label(volume: Option<Volume>) -> String {
    volume.map_or_else(|| "No sound".into(), Volume::label)
}

/// The volume in the answer of `wpctl get-volume`, such as
/// `Volume: 0.40 [MUTED]`.
#[must_use]
pub fn parse(answer: &str) -> Option<Volume> {
    let rest = answer.trim().strip_prefix("Volume:")?;
    let mut words = rest.split_whitespace();
    let level: f64 = words.next()?.parse().ok()?;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped to 0 to 100"
    )]
    let percent = (level * 100.0).round().clamp(0.0, 100.0) as u32;
    let muted = words.any(|w| w == "[MUTED]");
    Some(Volume { percent, muted })
}

/// The `wpctl` calls of a change.
#[must_use]
pub fn calls(change: Change) -> Vec<Vec<String>> {
    let call = |args: &[&str]| args.iter().map(|a| (*a).to_owned()).collect();
    let unmute = call(&["set-mute", SINK, "0"]);
    match change {
        Change::Up => vec![
            unmute,
            call(&["set-volume", "-l", "1.0", SINK, &format!("{STEP}%+")]),
        ],
        Change::Down => vec![call(&["set-volume", SINK, &format!("{STEP}%-")])],
        Change::Mute => vec![call(&["set-mute", SINK, "toggle"])],
        Change::Set(percent) => vec![
            unmute,
            call(&["set-volume", SINK, &format!("{}%", percent.min(100))]),
        ],
    }
}

/// The volume now, if `wpctl` and an output are there.
#[must_use]
pub fn read() -> Option<Volume> {
    let out = Command::new("wpctl")
        .args(["get-volume", SINK])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    parse(&String::from_utf8_lossy(&out.stdout))
}

/// Changes the volume and answers the volume after it; `None` when
/// `wpctl` or the output is missing.
#[must_use]
pub fn change(change: Change) -> Option<Volume> {
    for args in calls(change) {
        let done = Command::new("wpctl")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if !done.is_ok_and(|s| s.success()) {
            return None;
        }
    }
    read()
}

/// The short indicator of the volume keys: one notification that each
/// press replaces, so that a held key shows one, not a stack.
#[derive(Default)]
pub struct Indicator {
    notifier: Notifier,
    shown: Option<u32>,
}

impl Indicator {
    /// Changes the volume and shows the volume after it, or that there
    /// is no sound, as the app `app`.
    pub fn change(&mut self, app: &str, change: Change) -> Option<Volume> {
        let volume = self::change(change);
        let body = label(volume);
        let message = Message {
            icon: "audio-volume-medium",
            summary: "Volume",
            body: &body,
            expire_ms: SHOWN_MS,
        };
        self.shown = self.notifier.show(app, self.shown, &message);
        volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_answer_of_wpctl_reads_as_a_volume() {
        let volume = |percent, muted| Some(Volume { percent, muted });
        assert_eq!(parse("Volume: 0.40\n"), volume(40, false));
        assert_eq!(parse("Volume: 0.55 [MUTED]\n"), volume(55, true));
        assert_eq!(parse("Volume: 1.00"), volume(100, false));
        assert_eq!(parse(""), None);
        assert_eq!(parse("Translate ID error"), None);
    }

    #[test]
    fn a_volume_has_plain_words() {
        let at = |percent, muted| Some(Volume { percent, muted });
        assert_eq!(label(at(40, false)), "40%");
        assert_eq!(label(at(40, true)), "Muted");
        assert_eq!(label(None), "No sound");
    }

    #[test]
    fn a_change_never_goes_past_full() {
        let sink = |args: &[&str]| args.iter().map(|a| (*a).to_owned()).collect::<Vec<_>>();
        assert_eq!(
            calls(Change::Up),
            [
                sink(&["set-mute", SINK, "0"]),
                sink(&["set-volume", "-l", "1.0", SINK, "5%+"])
            ]
        );
        assert_eq!(calls(Change::Down), [sink(&["set-volume", SINK, "5%-"])]);
        assert_eq!(calls(Change::Mute), [sink(&["set-mute", SINK, "toggle"])]);
        assert_eq!(
            calls(Change::Set(150)),
            [
                sink(&["set-mute", SINK, "0"]),
                sink(&["set-volume", SINK, "100%"])
            ]
        );
    }
}
