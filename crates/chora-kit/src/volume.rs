//! The volume of the default sound output, through `wpctl` of
//! `WirePlumber`, the session manager of `PipeWire`: no daemon of its own.
//! A [`Mixer`] runs `wpctl` on a thread of its own, each call at most
//! [`WAIT`] long, so that a slow or hung `wpctl` never stalls the main
//! loop of a plugin. A reading is honest: a level, or why there is none.

use std::{
    io::Read as _,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        mpsc::{Sender, channel},
    },
    time::{Duration, Instant},
};

use crate::{
    notify::{Message, Notifier},
    words::{Language, Word},
};

/// The default sound output of `wpctl`.
const SINK: &str = "@DEFAULT_AUDIO_SINK@";
/// One step of a volume key, in percent.
pub const STEP: u32 = 5;
/// How long the indicator of a volume key shows, in milliseconds.
const SHOWN_MS: i32 = 1500;
/// How long one call of `wpctl` may take.
pub const WAIT: Duration = Duration::from_secs(2);

/// The volume of the default output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Volume {
    /// From 0 to 100.
    pub percent: u32,
    /// The output is muted.
    pub muted: bool,
}

/// What a person learns about the volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    /// The volume of the default output.
    Level(Volume),
    /// `wpctl` is not installed.
    NoWpctl,
    /// `wpctl` runs, but there is no output device.
    NoOutput,
    /// A change or a reading failed or took too long.
    Failed,
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

impl Reading {
    /// The words of the reading: `40%`, `Muted`, `wpctl not found`, `No
    /// output device` or `Change failed`, in a language.
    #[must_use]
    pub fn words(self, language: Language) -> String {
        let word = match self {
            Self::Level(Volume {
                muted: false,
                percent,
            }) => return format!("{percent}%"),
            Self::Level(_) => Word::SoundMuted,
            Self::NoWpctl => Word::NoWpctl,
            Self::NoOutput => Word::NoOutputDevice,
            Self::Failed => Word::ChangeFailed,
        };
        word.text(language).to_owned()
    }
}

/// The reading in the answer of `wpctl get-volume`, such as
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

/// How a call of a program ended.
#[derive(Debug, PartialEq, Eq)]
enum Ran {
    /// It printed this and exited with success.
    Done(String),
    /// It exited with a failure.
    Refused,
    /// It is not installed.
    Missing,
    /// It took longer than the wait and was killed.
    Hung,
}

/// Runs a program with arguments, at most `wait` long.
fn run(program: &PathBuf, args: &[String], wait: Duration) -> Ran {
    let spawned = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ran::Missing,
        Err(_) => return Ran::Refused,
    };
    let deadline = Instant::now() + wait;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Ran::Hung;
            }
        }
    };
    let mut out = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        let _ = stdout.read_to_string(&mut out);
    }
    if status.success() {
        Ran::Done(out)
    } else {
        Ran::Refused
    }
}

/// Reads the volume with `wpctl get-volume`.
fn read(program: &PathBuf, wait: Duration) -> Reading {
    let args = ["get-volume".to_owned(), SINK.to_owned()];
    match run(program, &args, wait) {
        Ran::Done(answer) => parse(&answer).map_or(Reading::NoOutput, Reading::Level),
        Ran::Refused => Reading::NoOutput,
        Ran::Missing => Reading::NoWpctl,
        Ran::Hung => Reading::Failed,
    }
}

/// Changes the volume and reads it again; a call that fails ends the
/// change.
fn apply(program: &PathBuf, change: Change, wait: Duration) -> Reading {
    for args in calls(change) {
        match run(program, &args, wait) {
            Ran::Done(_) => {}
            Ran::Missing => return Reading::NoWpctl,
            Ran::Refused | Ran::Hung => return Reading::Failed,
        }
    }
    read(program, wait)
}

/// What a worker does next.
enum Job {
    Read,
    Change(Change, bool),
}

/// The worker that runs `wpctl`: [`Mixer::refresh`] and [`Mixer::change`]
/// return at once, and [`Mixer::take`] answers the newest reading once.
pub struct Mixer {
    jobs: Sender<Job>,
    newest: Arc<Mutex<Option<Reading>>>,
}

impl Mixer {
    /// A mixer of `wpctl` on the PATH, whose volume keys show the
    /// indicator as the app `app`.
    #[must_use]
    pub fn new(app: &str) -> Self {
        Self::with("wpctl".into(), app, WAIT)
    }

    /// A mixer of another `wpctl` program and wait, for tests.
    #[must_use]
    pub fn with(program: PathBuf, app: &str, wait: Duration) -> Self {
        let (jobs, inbox) = channel::<Job>();
        let newest = Arc::new(Mutex::new(None));
        let shared = Arc::clone(&newest);
        let owner = app.to_owned();
        let spawned = std::thread::Builder::new()
            .name("chora-volume".into())
            .spawn(move || {
                let mut notifier = Notifier::new();
                let mut shown = None;
                for job in inbox {
                    let (reading, indicate) = match job {
                        Job::Read => (read(&program, wait), false),
                        Job::Change(change, indicate) => (apply(&program, change, wait), indicate),
                    };
                    if indicate {
                        let language = Language::of_session();
                        let body = reading.words(language);
                        let message = Message {
                            icon: "audio-volume-medium",
                            summary: Word::Volume.text(language),
                            body: &body,
                            expire_ms: SHOWN_MS,
                        };
                        shown = notifier.show(&owner, shown, &message);
                    }
                    if let Ok(mut newest) = shared.lock() {
                        *newest = Some(reading);
                    }
                }
            });
        if let Err(err) = spawned {
            eprintln!("{app}: the volume thread does not start: {err}");
        }
        Self { jobs, newest }
    }

    /// Reads the volume again.
    pub fn refresh(&self) {
        let _ = self.jobs.send(Job::Read);
    }

    /// Changes the volume and reads it again; `indicate` shows the
    /// reading for a moment, as after a volume key.
    pub fn change(&self, change: Change, indicate: bool) {
        let _ = self.jobs.send(Job::Change(change, indicate));
    }

    /// The reading that came since the last call, if one did.
    #[must_use]
    pub fn take(&self) -> Option<Reading> {
        self.newest.lock().ok()?.take()
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

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
    fn every_reading_has_plain_words_in_english_and_german() {
        let level = |percent, muted| Reading::Level(Volume { percent, muted });
        for (reading, en, de) in [
            (level(40, false), "40%", "40%"),
            (level(40, true), "Muted", "Stumm"),
            (Reading::NoWpctl, "wpctl not found", "wpctl nicht gefunden"),
            (Reading::NoOutput, "No output device", "Kein Ausgabegerät"),
            (Reading::Failed, "Change failed", "Änderung fehlgeschlagen"),
        ] {
            assert_eq!(reading.words(Language::En), en);
            assert_eq!(reading.words(Language::De), de);
        }
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

    #[test]
    fn a_call_ends_after_its_wait() {
        let sh = PathBuf::from("/bin/sh");
        let args = |code: &str| vec!["-c".to_owned(), code.to_owned()];
        let wait = Duration::from_millis(300);
        assert_eq!(run(&sh, &args("echo hi"), wait), Ran::Done("hi\n".into()));
        assert_eq!(run(&sh, &args("exit 3"), wait), Ran::Refused);
        let start = Instant::now();
        assert_eq!(run(&sh, &args("sleep 5"), wait), Ran::Hung);
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "{:?}",
            start.elapsed()
        );
        let missing = PathBuf::from("/no/such/wpctl");
        assert_eq!(run(&missing, &[], wait), Ran::Missing);
    }

    /// A stand-in of `wpctl` in a directory of its own: `code` runs for
    /// every call.
    fn fake_wpctl(name: &str, code: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-wpctl-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a directory");
        let path = dir.join("wpctl");
        std::fs::write(&path, format!("#!/bin/sh\n{code}\n")).expect("a stand-in");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("mode");
        path
    }

    fn next(mixer: &Mixer) -> Option<Reading> {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(3) {
            if let Some(reading) = mixer.take() {
                return Some(reading);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }

    #[test]
    fn the_mixer_reads_off_the_main_loop_and_names_each_failure() {
        let wait = Duration::from_millis(300);
        let ok = fake_wpctl(
            "ok",
            r#"case "$1" in get-volume) echo 'Volume: 0.30' ;; esac"#,
        );
        let mixer = Mixer::with(ok, "test", wait);
        let start = Instant::now();
        mixer.refresh();
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "returns at once"
        );
        let thirty = Volume {
            percent: 30,
            muted: false,
        };
        assert_eq!(next(&mixer), Some(Reading::Level(thirty)));
        assert_eq!(mixer.take(), None, "a reading comes once");
        let none = fake_wpctl("none", "echo 'Translate ID error' >&2; exit 1");
        let mixer = Mixer::with(none, "test", wait);
        mixer.refresh();
        assert_eq!(next(&mixer), Some(Reading::NoOutput));
        let refused = fake_wpctl(
            "refused",
            r#"case "$1" in get-volume) echo 'Volume: 0.30' ;; *) exit 1 ;; esac"#,
        );
        let mixer = Mixer::with(refused, "test", wait);
        mixer.change(Change::Up, false);
        assert_eq!(next(&mixer), Some(Reading::Failed));
        let hung = fake_wpctl("hung", "sleep 5");
        let mixer = Mixer::with(hung, "test", wait);
        mixer.refresh();
        assert_eq!(next(&mixer), Some(Reading::Failed));
        let mixer = Mixer::with("/no/such/wpctl".into(), "test", wait);
        mixer.refresh();
        assert_eq!(next(&mixer), Some(Reading::NoWpctl));
    }
}
