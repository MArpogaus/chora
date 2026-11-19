//! Programs that a shell starts for the user, followed until their window
//! shows.
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

/// One way to start a program: the name that messages use and its shell
/// command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    /// The program, as a message names it.
    pub name: String,
    /// The shell command.
    pub command: String,
}

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
    /// The program did not open a window, and the next one of its
    /// fallbacks starts.
    Fallback {
        /// The name of what the user started.
        name: String,
        /// The program that did not open.
        from: String,
        /// The program that starts in its place.
        to: String,
        /// The message of the start, which this one replaces.
        note: Option<u32>,
        /// What went wrong with `from`.
        why: Why,
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
    /// The program ended without a window.
    Exited,
    /// No editor is installed that has a window.
    NoEditor,
}

impl Why {
    /// What went wrong, after the name of a program.
    fn clause(self) -> String {
        match self {
            Self::Missing | Self::NoEditor => "is not installed.".into(),
            Self::Refused => "could not be started.".into(),
            Self::Slow => format!("showed no window in {} seconds.", GIVE_UP.as_secs()),
            Self::Exited => "ended without a window.".into(),
        }
    }
}

impl Change {
    /// The title and the text of the message.
    #[must_use]
    pub fn text(&self) -> (String, String) {
        match self {
            Self::Fallback {
                name,
                from,
                to,
                why,
                ..
            } => (
                format!("{name} opened with {to}"),
                format!("{from} {}", why.clause()),
            ),
            Self::Starting { name, .. } => (format!("Starting {name}"), "Please wait.".into()),
            Self::Done { .. } => (String::new(), String::new()),
            Self::Failed { name, why, .. } => {
                let body = match why {
                    Why::Missing => "The program is not installed.".to_string(),
                    Why::Refused => "The session could not start it.".into(),
                    Why::Slow => format!("No window opened in {} seconds.", GIVE_UP.as_secs()),
                    Why::Exited => "The program ended without a window.".into(),
                    Why::NoEditor => "No text editor installed (for example mousepad).".into(),
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
    next: Next,
}

/// What a start falls back to when its program does not open.
#[derive(Clone, Debug, Default)]
struct Next {
    /// The name of the program that runs.
    from: String,
    /// The activation token of xp for the next start.
    activation: Option<String>,
    /// The programs that follow, in order.
    rest: Vec<Candidate>,
    /// Whether a start that ended without a window fails at once and not
    /// after [`GIVE_UP`].
    early: bool,
    /// The program before this one that did not open, and why; said
    /// only once this one shows its window.
    fell: Option<(String, Why)>,
    /// Where the shell of a start that ends early writes its failed
    /// status.
    status: Option<std::sync::Arc<StatusFile>>,
}

/// A file in which the shell around a start of an early program writes
/// its exit status when that is not zero. It is removed with the last
/// holder.
#[derive(Debug)]
struct StatusFile(std::path::PathBuf);

impl StatusFile {
    fn create() -> Option<Self> {
        static COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let dir = chora_scene::paths::runtime_home().unwrap_or_else(std::env::temp_dir);
        let n = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = dir.join(format!("chora-xp-exit-{}-{n}", std::process::id()));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()?;
        Some(Self(path))
    }

    /// The command that runs `command` and writes its status here when
    /// it fails. A program that replaces the shell leaves no status.
    fn wrap(&self, command: &str) -> String {
        let file = quoted(&self.0.to_string_lossy());
        format!(
            "{{ {command}\n}}; status=$?; [ $status -eq 0 ] || ! [ -e {file} ] || echo $status >{file}; exit $status"
        )
    }

    /// Whether the start ended with a status that is not zero.
    fn failed(&self) -> bool {
        std::fs::read_to_string(&self.0).is_ok_and(|text| !text.trim().is_empty())
    }
}

impl Drop for StatusFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The answer of `choractl run` for a command.
struct Started {
    name: String,
    token: Option<String>,
    pid: Option<u32>,
    since: Instant,
    windows_before: u64,
    next: Next,
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
    start: Starter,
}

/// How a command starts: it answers the activation token that finds the
/// window and the pid of the program, or nothing when it did not start.
pub type Starter =
    std::sync::Arc<dyn Fn(&str, Option<&str>) -> Option<(String, Option<u32>)> + Send + Sync>;

fn start(command: &str, token: Option<&str>) -> Option<(String, Option<u32>)> {
    crate::launch(command, token)
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
            start: std::sync::Arc::new(start),
        }
    }

    /// No program waits, and a command starts through `start`.
    #[must_use]
    pub fn with_starter(start: Starter) -> Self {
        Self {
            start,
            ..Self::new()
        }
    }

    /// Starts a shell command as an app. The wait for `choractl run` is
    /// on a thread of its own, so the window manager never blocks; its
    /// answer comes with [`Self::collect`]. A program that is not on
    /// `PATH` never starts: the change says so. The app gets the
    /// activation token of xp, if xp has one, as [`crate::launch`]
    /// takes it.
    pub fn run(
        &mut self,
        command: &str,
        (name, token): (String, Option<String>),
        now: Instant,
    ) -> Option<Change> {
        let candidate = Candidate {
            name: name.clone(),
            command: command.to_owned(),
        };
        let template = Next {
            activation: token,
            ..Next::default()
        };
        let mut changes = self.begin(&name, vec![candidate], template, now);
        changes.pop()
    }

    /// Starts the first of `chain` as the app `name`. When it does not
    /// open, the next one starts, and a change says which and why; when
    /// none opens, a change says `name` cannot open. `skipped` is a
    /// program that was left out because it is not installed.
    pub fn run_chain(
        &mut self,
        name: &str,
        (chain, skipped): (Vec<Candidate>, Option<String>),
        token: Option<String>,
        now: Instant,
    ) -> Vec<Change> {
        let template = Next {
            activation: token,
            early: true,
            fell: skipped.map(|from| (from, Why::Missing)),
            ..Next::default()
        };
        self.begin(name, chain, template, now)
    }

    fn begin(
        &mut self,
        name: &str,
        mut chain: Vec<Candidate>,
        template: Next,
        now: Instant,
    ) -> Vec<Change> {
        if chain.is_empty() {
            return vec![Change::Failed {
                name: name.to_owned(),
                note: None,
                why: Why::NoEditor,
            }];
        }
        let first = chain.remove(0);
        let status = template
            .early
            .then(StatusFile::create)
            .flatten()
            .map(std::sync::Arc::new);
        let next = Next {
            from: first.name,
            rest: chain,
            status,
            ..template
        };
        if missing(&first.command, crate::desktop_entry::on_path) {
            eprintln!(
                "chora-xp: {:?} is not on PATH, {:?} does not run",
                next.from, first.command
            );
            return self.fail((name.to_owned(), None), Why::Missing, next, now);
        }
        let command = next
            .status
            .as_ref()
            .map_or_else(|| first.command.clone(), |file| file.wrap(&first.command));
        let sender = self.sender.clone();
        let starter = self.start.clone();
        let windows_before = self.windows;
        let name = name.to_owned();
        std::thread::spawn(move || {
            let answer = starter(&command, next.activation.as_deref());
            let _ = sender.send(Started {
                name,
                pid: answer.as_ref().and_then(|(_, pid)| *pid),
                token: answer.map(|(token, _)| token),
                since: now,
                windows_before,
                next,
            });
        });
        Vec::new()
    }

    /// What a program that did not open calls for: the next of its
    /// fallbacks starts, or, when none is left, a change says the app
    /// cannot open. Which one served is said only when its window shows.
    fn fail(
        &mut self,
        (name, note): (String, Option<u32>),
        why: Why,
        next: Next,
        now: Instant,
    ) -> Vec<Change> {
        if next.rest.is_empty() {
            return vec![Change::Failed { name, note, why }];
        }
        let template = Next {
            activation: next.activation,
            early: next.early,
            fell: Some((next.from, why)),
            ..Next::default()
        };
        let mut changes = vec![Change::Done { note }];
        changes.extend(self.begin(&name, next.rest, template, now));
        changes
    }

    fn served(&mut self, name: String, next: Next) {
        if let Some((from, why)) = next.fell {
            self.failed.push(Change::Fallback {
                name,
                from,
                to: next.from,
                note: None,
                why,
            });
        }
    }

    /// Takes the answers of the starts that finished: the changes of
    /// those that failed.
    pub fn collect(&mut self) -> Vec<Change> {
        self.drain();
        std::mem::take(&mut self.failed)
    }

    fn drain(&mut self) {
        while let Ok(started) = self.started.try_recv() {
            if started.token.is_some() {
                self.add(started);
            } else {
                let name = (started.name, None);
                let changes = self.fail(name, Why::Refused, started.next, Instant::now());
                self.failed.extend(changes);
            }
        }
    }

    fn add(&mut self, started: Started) {
        let Some(token) = started.token else {
            return;
        };
        if self.opened.contains(&token) {
            self.served(started.name, started.next);
            return;
        }
        if DETACHED.contains(&started.name.as_str()) {
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
            next: started.next,
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
        let entry = self.entries.remove(i);
        self.served(entry.name, entry.next);
        entry.note
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
        let mut failed = Vec::new();
        let mut served = Vec::new();
        let windows = self.windows;
        self.entries.retain_mut(|entry| {
            let age = now.saturating_duration_since(entry.since);
            let silent = entry.windows_before == windows;
            let alive = entry.pid.is_some_and(crate::alive);
            let early = entry.next.early;
            if age >= GIVE_UP {
                if alive {
                    changes.push(Change::Done { note: entry.note });
                    let (name, _, next) = entry.take();
                    served.push((name, next));
                } else if silent {
                    failed.push((entry.take(), Why::Slow));
                }
                return false;
            }
            let crashed = entry.next.status.as_ref().is_some_and(|file| file.failed());
            if early && silent && crashed {
                failed.push((entry.take(), Why::Exited));
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
        for (name, next) in served {
            self.served(name, next);
        }
        for ((name, note, next), why) in failed {
            changes.extend(self.fail((name, note), why, next, now));
        }
        changes
    }
}

impl Entry {
    fn take(&mut self) -> (String, Option<u32>, Next) {
        (
            std::mem::take(&mut self.name),
            self.note,
            std::mem::take(&mut self.next),
        )
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

/// A word in single quotes for the shell.
fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}
