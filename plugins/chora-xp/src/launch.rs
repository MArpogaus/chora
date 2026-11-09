//! Programs that the user starts, followed until their window shows.
//!
//! A program that cannot start, or ends without a window, or takes too
//! long gives a message that names it; one that is still starting after a
//! moment shows that it is. The window manager answers each start with
//! [`Change`]s that it shows through the notification server.

use std::{
    sync::mpsc::{Receiver, Sender, channel},
    time::{Duration, Instant},
};

/// How long a program starts unnoticed before a message says so.
pub const SHOW_AFTER: Duration = Duration::from_secs(2);

/// How long a program may take to open its first window.
pub const GIVE_UP: Duration = Duration::from_secs(30);

/// How many tokens of opened windows are kept.
const OPENED: usize = 32;

/// What a message tells the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    /// The program is still starting.
    Starting {
        /// The activation token of the start.
        token: String,
        /// The name of the program.
        name: String,
    },
    /// The program did not open a window.
    Failed {
        /// The name of the program.
        name: String,
        /// The message of its start, which this one replaces.
        note: Option<u32>,
        /// What went wrong.
        why: Why,
    },
}

/// Why a program did not open a window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    /// No such program on `PATH`.
    Missing,
    /// `choractl run` could not start it.
    Refused,
    /// It ended before its first window showed.
    Stopped,
    /// No window showed within [`GIVE_UP`].
    Slow,
}

impl Change {
    /// The title and the text of the message.
    #[must_use]
    pub fn text(&self) -> (String, String) {
        match self {
            Self::Starting { name, .. } => (format!("Starting {name}"), "Please wait.".into()),
            Self::Failed { name, why, .. } => {
                let body = match why {
                    Why::Missing => "The program is not installed.".into(),
                    Why::Refused => "The session could not start it.".into(),
                    Why::Stopped => "It ended before it opened a window.".to_string(),
                    Why::Slow => format!("No window opened in {} seconds.", GIVE_UP.as_secs()),
                };
                (format!("Cannot open {name}"), body)
            }
        }
    }
}

/// A program that started and shows no window yet.
#[derive(Debug)]
struct Entry {
    name: String,
    token: String,
    pid: Option<u32>,
    since: Instant,
    windows_before: u64,
    note: Option<u32>,
    shown: bool,
}

/// The answer of `choractl run` for a command.
struct Started {
    name: String,
    launched: Option<(String, Option<u32>)>,
    since: Instant,
    windows_before: u64,
}

/// The programs that started and wait for their first window.
pub struct Launches {
    entries: Vec<Entry>,
    windows: u64,
    /// The tokens of the latest windows, for a window that shows before
    /// the answer of its start arrives.
    opened: Vec<String>,
    failed: Vec<Change>,
    started: Receiver<Started>,
    sender: Sender<Started>,
}

impl Default for Launches {
    fn default() -> Self {
        Self::new()
    }
}

impl Launches {
    /// No program waits.
    #[must_use]
    pub fn new() -> Self {
        let (sender, started) = channel();
        Self {
            entries: Vec::new(),
            windows: 0,
            opened: Vec::new(),
            failed: Vec::new(),
            started,
            sender,
        }
    }

    /// Starts a shell command as an app. The wait for `choractl run` is
    /// on a thread of its own, so the window manager never blocks; its
    /// answer comes with [`Self::collect`]. A program that is not on
    /// `PATH` never starts: the change says so.
    pub fn run(&mut self, command: &str, now: Instant) -> Option<Change> {
        let name = program_name(command);
        if !crate::session::on_path(command.split_whitespace().next().unwrap_or_default()) {
            eprintln!("chora-xp: {name:?} is not on PATH, {command:?} does not run");
            return Some(Change::Failed {
                name,
                note: None,
                why: Why::Missing,
            });
        }
        let command = command.to_owned();
        let sender = self.sender.clone();
        let windows_before = self.windows;
        std::thread::spawn(move || {
            let launched = chora_kit::launch(&command);
            let _ = sender.send(Started {
                name,
                launched,
                since: now,
                windows_before,
            });
        });
        None
    }

    /// Takes the answers of the starts that finished: the changes of
    /// those that failed.
    pub fn collect(&mut self) -> Vec<Change> {
        self.drain();
        std::mem::take(&mut self.failed)
    }

    fn drain(&mut self) {
        while let Ok(started) = self.started.try_recv() {
            match started.launched {
                Some(_) => self.add(started),
                None => self.failed.push(Change::Failed {
                    name: started.name,
                    note: None,
                    why: Why::Refused,
                }),
            }
        }
    }

    fn add(&mut self, started: Started) {
        let Some((token, pid)) = started.launched else {
            return;
        };
        if self.opened.contains(&token) {
            return;
        }
        self.entries.push(Entry {
            name: started.name,
            token,
            pid,
            since: started.since,
            windows_before: started.windows_before,
            note: None,
            shown: false,
        });
    }

    /// A window opened, with the activation token that started it if it
    /// has one. Answers the message of the start to close, if there is one.
    pub fn window(&mut self, token: Option<&str>) -> Option<u32> {
        self.drain();
        self.windows += 1;
        let token = token?;
        if self.opened.len() >= OPENED {
            self.opened.remove(0);
        }
        self.opened.push(token.to_owned());
        let i = self.entries.iter().position(|e| e.token == token)?;
        self.entries.remove(i).note
    }

    /// Keeps the id of the message that says a program starts.
    pub fn noted(&mut self, token: &str, id: u32) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.token == token) {
            entry.note = Some(id);
        }
    }

    /// What the programs that wait for a window call for at `now`.
    /// `alive` tells whether a process runs.
    pub fn check(&mut self, now: Instant, alive: impl Fn(u32) -> bool) -> Vec<Change> {
        let mut changes = Vec::new();
        let windows = self.windows;
        self.entries.retain_mut(|entry| {
            let age = now.saturating_duration_since(entry.since);
            let gone = !entry.pid.is_some_and(&alive);
            let why = if gone && entry.windows_before != windows {
                return false;
            } else if gone {
                Why::Stopped
            } else if age >= GIVE_UP {
                Why::Slow
            } else {
                if age >= SHOW_AFTER && !entry.shown {
                    entry.shown = true;
                    changes.push(Change::Starting {
                        token: entry.token.clone(),
                        name: entry.name.clone(),
                    });
                }
                return true;
            };
            changes.push(Change::Failed {
                name: entry.name.clone(),
                note: entry.note,
                why,
            });
            false
        });
        changes
    }
}

/// The name of the program of a shell command: its first word without
/// quotes and directory.
#[must_use]
pub fn program_name(command: &str) -> String {
    let word = command.split_whitespace().next().unwrap_or_default();
    let word = word.trim_matches(['\'', '"']);
    word.rsplit('/').next().unwrap_or(word).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launches(pid: Option<u32>) -> (Launches, Instant) {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(started(pid, now));
        (launches, now)
    }

    fn started(pid: Option<u32>, since: Instant) -> Started {
        Started {
            name: "writer".into(),
            launched: Some(("t1".into(), pid)),
            since,
            windows_before: 0,
        }
    }

    fn none() -> Vec<Change> {
        Vec::new()
    }

    fn seconds(now: Instant, s: u64) -> Instant {
        now + Duration::from_secs(s)
    }

    #[test]
    fn the_name_is_the_program_without_directory_and_quotes() {
        assert_eq!(program_name("libreoffice --writer"), "libreoffice");
        assert_eq!(program_name("/usr/bin/thunar '/'"), "thunar");
        assert_eq!(program_name("'my app' x"), "my");
        assert_eq!(program_name(""), "");
    }

    #[test]
    fn a_missing_program_gives_a_message_and_starts_nothing() {
        let mut launches = Launches::new();
        let change = launches.run("chora-no-such-program --flag", Instant::now());
        assert_eq!(
            change,
            Some(Change::Failed {
                name: "chora-no-such-program".into(),
                note: None,
                why: Why::Missing
            })
        );
        assert_eq!(launches.collect(), none());
    }

    #[test]
    fn a_program_that_runs_shows_a_message_only_after_a_moment() {
        let (mut launches, now) = launches(Some(7));
        assert_eq!(launches.check(seconds(now, 1), |_| true), none());
        let changes = launches.check(seconds(now, 2), |_| true);
        assert_eq!(
            changes,
            [Change::Starting {
                token: "t1".into(),
                name: "writer".into()
            }]
        );
        assert_eq!(launches.check(seconds(now, 3), |_| true), none());
    }

    #[test]
    fn the_window_of_the_token_ends_the_wait_and_names_the_message_to_close() {
        let (mut launches, now) = launches(Some(7));
        launches.check(seconds(now, 2), |_| true);
        launches.noted("t1", 12);
        assert_eq!(launches.window(Some("other")), None);
        assert_eq!(launches.window(Some("t1")), Some(12));
        assert_eq!(launches.check(seconds(now, 40), |_| true), none());
    }

    #[test]
    fn a_window_that_shows_before_the_answer_of_its_start_ends_the_wait() {
        let now = Instant::now();
        let mut launches = Launches::new();
        assert_eq!(launches.window(Some("t1")), None);
        launches.add(started(Some(7), now));
        assert_eq!(launches.check(seconds(now, 40), |_| true), none());
    }

    #[test]
    fn a_program_that_ended_without_a_window_says_so_in_place_of_its_message() {
        let (mut launches, now) = launches(Some(7));
        launches.check(seconds(now, 2), |_| true);
        launches.noted("t1", 12);
        let changes = launches.check(seconds(now, 3), |_| false);
        assert_eq!(
            changes,
            [Change::Failed {
                name: "writer".into(),
                note: Some(12),
                why: Why::Stopped
            }]
        );
        assert_eq!(launches.check(seconds(now, 4), |_| false), none());
    }

    #[test]
    fn a_start_that_left_no_pid_counts_as_ended() {
        let (mut launches, now) = launches(None);
        assert_eq!(launches.check(seconds(now, 1), |_| true).len(), 1);
    }

    #[test]
    fn a_launcher_that_hands_over_to_a_running_instance_is_no_failure() {
        let (mut launches, now) = launches(Some(7));
        assert_eq!(launches.window(None), None);
        assert_eq!(launches.check(seconds(now, 1), |_| false), none());
        assert_eq!(launches.check(seconds(now, 40), |_| true), none());
    }

    #[test]
    fn a_program_without_a_window_gives_up_after_the_limit() {
        let (mut launches, now) = launches(Some(7));
        assert!(
            launches
                .check(seconds(now, GIVE_UP.as_secs() - 1), |_| true)
                .iter()
                .all(|c| matches!(c, Change::Starting { .. }))
        );
        let changes = launches.check(seconds(now, GIVE_UP.as_secs()), |_| true);
        assert_eq!(
            changes,
            [Change::Failed {
                name: "writer".into(),
                note: None,
                why: Why::Slow
            }]
        );
        assert_eq!(launches.check(seconds(now, 60), |_| true), none());
    }

    #[test]
    fn the_messages_name_the_program() {
        let failed = |why| Change::Failed {
            name: "thunar".into(),
            note: None,
            why,
        };
        let (title, body) = failed(Why::Missing).text();
        assert_eq!(title, "Cannot open thunar");
        assert!(body.contains("not installed"), "{body}");
        assert!(failed(Why::Slow).text().1.contains("30 seconds"));
        let starting = Change::Starting {
            token: "t".into(),
            name: "writer".into(),
        };
        assert_eq!(starting.text().0, "Starting writer");
    }
}
