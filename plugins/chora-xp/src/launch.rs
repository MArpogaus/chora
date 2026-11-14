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

/// Programs that show no toplevel window, a layer-shell launcher or a
/// toggle that hands over to a running instance: their start is no wait.
const DETACHED: [&str; 2] = ["fuzzel", "chora-chat"];

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
    /// The program runs and needs no window: its message ends.
    Done {
        /// The message of its start, which this one closes.
        note: Option<u32>,
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
    /// No window showed within [`GIVE_UP`].
    Slow,
}

impl Change {
    /// The title and the text of the message.
    #[must_use]
    pub fn text(&self) -> (String, String) {
        match self {
            Self::Starting { name, .. } => (format!("Starting {name}"), "Please wait.".into()),
            Self::Done { .. } => (String::new(), String::new()),
            Self::Failed { name, why, .. } => {
                let body = match why {
                    Why::Missing => "The program is not installed.".to_string(),
                    Why::Refused => "The session could not start it.".into(),
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
    token: Option<String>,
    pid: Option<u32>,
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
    /// `PATH` never starts: the change says so. The app gets the
    /// activation token of xp, if xp has one, as [`chora_kit::launch`]
    /// takes it.
    pub fn run(
        &mut self,
        command: &str,
        (name, token): (String, Option<String>),
        now: Instant,
    ) -> Option<Change> {
        if missing(command, crate::session::on_path) {
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
            let answer = chora_kit::launch(&command, token.as_deref());
            let _ = sender.send(Started {
                name,
                pid: answer.as_ref().and_then(|(_, pid)| *pid),
                token: answer.map(|(token, _)| token),
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
            match started.token {
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
        let Some(token) = started.token else {
            return;
        };
        if self.opened.contains(&token) || DETACHED.contains(&started.name.as_str()) {
            return;
        }
        self.entries.push(Entry {
            name: started.name,
            token,
            pid: started.pid,
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

    /// What the programs that wait for a window call for at `now`. A
    /// program that shows no window within [`GIVE_UP`] fails, unless a
    /// window of another start opened meanwhile, which a launcher that
    /// hands its work to a running instance explains.
    pub fn check(&mut self, now: Instant) -> Vec<Change> {
        let mut changes = Vec::new();
        let windows = self.windows;
        self.entries.retain_mut(|entry| {
            let age = now.saturating_duration_since(entry.since);
            if age >= GIVE_UP {
                if entry.pid.is_some_and(chora_kit::alive) {
                    changes.push(Change::Done { note: entry.note });
                } else if entry.windows_before == windows {
                    changes.push(Change::Failed {
                        name: entry.name.clone(),
                        note: entry.note,
                        why: Why::Slow,
                    });
                }
                return false;
            }
            if age >= SHOW_AFTER && !entry.shown {
                entry.shown = true;
                changes.push(Change::Starting {
                    token: entry.token.clone(),
                    name: entry.name.clone(),
                });
            }
            true
        });
        changes
    }
}

/// Words that the shell runs without a program on `PATH`.
const BUILTINS: [&str; 18] = [
    ".", ":", "alias", "cd", "command", "echo", "eval", "exec", "exit", "export", "printf", "pwd",
    "read", "set", "source", "test", "trap", "unset",
];

/// Whether a command is a plain program that `on_path` does not know:
/// its first word is a bare name, no shell builtin, and the command has
/// no assignment, quote around the program, path, operator or
/// substitution that could make the shell find the program elsewhere.
/// Such a command never runs, so a message can say so at once; any other
/// command runs and the limit of [`GIVE_UP`] speaks.
fn missing(command: &str, on_path: impl Fn(&str) -> bool) -> bool {
    let Some(word) = command.split_whitespace().next() else {
        return false;
    };
    let plain = word
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'));
    let shell = command.chars().any(|c| {
        matches!(
            c,
            '&' | '|' | ';' | '<' | '>' | '(' | ')' | '$' | '`' | '\\'
        )
    });
    plain && !shell && !BUILTINS.contains(&word) && !on_path(word)
}

/// The launcher command in the look of xp: `fuzzel` with the `file` of
/// the flavor as its configuration, unless the command names one or the
/// file is missing. `fuzzel` alone is the Run box, [`RUN_BOX`] of `exe`
/// with the file and the terminal. Another command stays as it is.
#[must_use]
pub fn themed(command: &str, file: &std::path::Path, exe: &str, terminal: &str) -> String {
    let mut words = command.split_whitespace();
    let named = words.clone().any(|w| w == "--config" || w == "-c");
    if words.next() != Some("fuzzel") || named || !file.is_file() {
        return command.to_owned();
    }
    let rest = command.trim_start().trim_start_matches("fuzzel");
    let file = quoted(&file.to_string_lossy());
    if rest.trim().is_empty() {
        return format!("{} {RUN_BOX} {file} {}", quoted(exe), quoted(terminal));
    }
    format!("fuzzel --config {file}{rest}")
}

/// The argument of chora-xp that shows the Run box.
pub const RUN_BOX: &str = "--run-box";

/// A word in single quotes for the shell.
fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// The Run box, `chora-xp --run-box FILE TERMINAL`: fuzzel with the
/// config file of the flavor lists the installed programs, sorted, and
/// says when text finds nothing, as [`chora_kit::launcher`] does.
pub fn run_box(args: &[String]) {
    let [file, terminal] = args else {
        eprintln!("usage: chora-xp {RUN_BOX} FILE TERMINAL");
        return;
    };
    let fuzzel = ["fuzzel", "--config", file].map(String::from);
    chora_kit::launcher::run(&fuzzel, terminal, None);
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

    fn launches() -> (Launches, Instant) {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(started(now));
        (launches, now)
    }

    fn started(since: Instant) -> Started {
        Started {
            name: "writer".into(),
            token: Some("t1".into()),
            pid: None,
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
    fn only_a_plain_command_of_a_program_that_is_not_there_counts_as_missing() {
        let on_path = |word: &str| matches!(word, "env" | "foot");
        let refused = ["chora-no-such", "chora-no-such --flag '/home/user'"];
        let run = [
            "foot",
            "foot -e htop",
            "/opt/app/bin/start",
            "./app",
            "'my app' --flag",
            "\"my app\"",
            "VAR=1 chora-no-such",
            "env VAR=1 chora-no-such",
            "cd /tmp && chora-no-such",
            "chora-no-such || foot",
            "chora-no-such $HOME",
            "cd /tmp",
            "exec chora-no-such",
            "",
        ];
        for command in refused {
            assert!(missing(command, on_path), "{command}");
        }
        for command in run {
            assert!(!missing(command, on_path), "{command}");
        }
    }

    #[test]
    fn a_missing_program_gives_a_message_and_starts_nothing() {
        let mut launches = Launches::new();
        let change = launches.run(
            "chora-no-such-program --flag",
            ("chora-no-such-program".into(), None),
            Instant::now(),
        );
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
        let (mut launches, now) = launches();
        assert_eq!(launches.check(seconds(now, 1)), none());
        let changes = launches.check(seconds(now, 2));
        assert_eq!(
            changes,
            [Change::Starting {
                token: "t1".into(),
                name: "writer".into()
            }]
        );
        assert_eq!(launches.check(seconds(now, 3)), none());
    }

    #[test]
    fn the_window_of_the_token_ends_the_wait_and_names_the_message_to_close() {
        let (mut launches, now) = launches();
        launches.check(seconds(now, 2));
        launches.noted("t1", 12);
        assert_eq!(launches.window(Some("other")), None);
        assert_eq!(launches.window(Some("t1")), Some(12));
        assert_eq!(launches.check(seconds(now, 40)), none());
    }

    #[test]
    fn a_window_that_shows_before_the_answer_of_its_start_ends_the_wait() {
        let now = Instant::now();
        let mut launches = Launches::new();
        assert_eq!(launches.window(Some("t1")), None);
        launches.add(started(now));
        assert_eq!(launches.check(seconds(now, 40)), none());
    }

    #[test]
    fn a_launcher_that_exits_at_once_is_followed_like_any_program() {
        let (mut launches, now) = launches();
        assert_eq!(launches.check(seconds(now, 1)), none());
        assert_eq!(
            launches.check(seconds(now, 2)),
            [Change::Starting {
                token: "t1".into(),
                name: "writer".into()
            }]
        );
        launches.noted("t1", 12);
        assert_eq!(launches.window(Some("t1")), Some(12));
    }

    #[test]
    fn a_start_that_left_no_pid_waits_like_any_other() {
        let (mut launches, now) = launches();
        assert_eq!(launches.check(seconds(now, 1)), none());
    }

    #[test]
    fn a_launcher_that_hands_over_to_a_running_instance_is_no_failure() {
        let (mut launches, now) = launches();
        assert_eq!(launches.window(None), None);
        assert_eq!(launches.check(seconds(now, GIVE_UP.as_secs())), none());
    }

    #[test]
    fn a_program_without_a_window_gives_up_after_the_limit() {
        let (mut launches, now) = launches();
        assert!(
            launches
                .check(seconds(now, GIVE_UP.as_secs() - 1))
                .iter()
                .all(|c| matches!(c, Change::Starting { .. }))
        );
        let changes = launches.check(seconds(now, GIVE_UP.as_secs()));
        assert_eq!(
            changes,
            [Change::Failed {
                name: "writer".into(),
                note: None,
                why: Why::Slow
            }]
        );
        assert_eq!(launches.check(seconds(now, 60)), none());
    }

    #[test]
    fn a_launcher_without_a_toplevel_is_not_followed() {
        for name in ["fuzzel", "chora-chat"] {
            let now = Instant::now();
            let mut launches = Launches::new();
            launches.add(Started {
                name: name.into(),
                ..started(now)
            });
            assert_eq!(launches.check(seconds(now, 2)), none(), "{name}");
            assert_eq!(launches.check(seconds(now, 40)), none(), "{name}");
        }
    }

    #[test]
    fn a_program_that_still_runs_at_the_limit_is_no_failure() {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(Started {
            pid: Some(std::process::id()),
            ..started(now)
        });
        launches.check(seconds(now, 2));
        launches.noted("t1", 12);
        assert_eq!(
            launches.check(seconds(now, GIVE_UP.as_secs())),
            [Change::Done { note: Some(12) }]
        );
        assert_eq!(launches.check(seconds(now, 60)), none());
    }

    #[test]
    fn a_program_that_ended_without_a_window_is_a_failure() {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(Started {
            pid: Some(u32::MAX),
            ..started(now)
        });
        assert!(matches!(
            launches.check(seconds(now, GIVE_UP.as_secs())).as_slice(),
            [Change::Failed { why: Why::Slow, .. }]
        ));
    }

    #[test]
    fn fuzzel_takes_the_look_of_xp() {
        let dir = std::env::temp_dir().join(format!("chora-xp-themed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("fuzzel.ini");
        std::fs::write(&file, "[main]\n").unwrap();
        let themed = |command: &str, file: &std::path::Path| {
            super::themed(command, file, "/opt/it's/chora-xp", "foot")
        };
        let line = themed("fuzzel --dmenu", &file);
        assert_eq!(
            line,
            format!("fuzzel --config '{}' --dmenu", file.display())
        );
        assert_eq!(
            themed("fuzzel --config /x.ini", &file),
            "fuzzel --config /x.ini"
        );
        assert_eq!(themed("rofi -show run", &file), "rofi -show run");
        assert_eq!(themed("fuzzel", &dir.join("none.ini")), "fuzzel");
        assert_eq!(
            themed("fuzzel", &file),
            format!(
                "'/opt/it'\\''s/chora-xp' --run-box '{}' 'foot'",
                file.display()
            )
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_shipped_fuzzel_file_is_xp_and_opaque() {
        let text = include_str!("../../../flavors/xp/theme/fuzzel.ini");
        assert!(text.contains("font=Tahoma:pixelsize=14"), "{text}");
        assert!(text.contains("prompt=\"Open: \""), "{text}");
        for key in ["background", "text", "selection", "border"] {
            let value = text
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{key}=")))
                .unwrap_or_else(|| panic!("{key}"));
            assert!(value.ends_with("ff"), "{key} is not opaque: {value}");
        }
        assert!(text.contains("selection=316ac5ff"), "{text}");
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
        let (title, body) = failed(Why::Refused).text();
        assert_eq!(title, "Cannot open thunar");
        assert!(body.contains("could not start"), "{body}");
    }
}
