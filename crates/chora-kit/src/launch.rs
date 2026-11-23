//! Programs that a shell starts for the user, followed until their window
//! shows (D104).
//!
//! `choractl run` starts a program in a unit of its own and answers the
//! activation token, the pid of the main process and the unit. The shell
//! owns that process with a pidfd, which needs no parent: when the main
//! process is gone and the unit has no process left, the program ended;
//! with a failed exit status, which the shell around the command writes
//! to a file, it ended without a window and fails at once. A program that
//! is still starting after a moment shows that it is, and it is until its
//! window shows or the program ends: a program that runs without a window
//! is neither served nor failed, and no timer decides for it. One that
//! ended without a failure and shows no window within [`HAND_OVER`] fails
//! too, since a launcher may have handed its work to a running instance
//! and a window of that shows soon. The tracking answers each start with
//! [`Change`]s that a look shows through the notification server.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    time::{Duration, Instant},
};

/// How long a program starts unnoticed before a message says so.
pub const SHOW_AFTER: Duration = Duration::from_secs(2);

/// How long after a program ended without a failure its window may still
/// show: a launcher that hands its work to a running instance exits at
/// once. It is also the wait for a start whose end nobody can see, which
/// has neither a pid nor a unit.
pub const HAND_OVER: Duration = Duration::from_secs(30);

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
    /// A start that nobody follows showed no window within [`HAND_OVER`].
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
            Self::Slow => format!("showed no window in {} seconds.", HAND_OVER.as_secs()),
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
                    Why::Slow => format!("No window opened in {} seconds.", HAND_OVER.as_secs()),
                    Why::Exited => "The program ended without a window.".into(),
                    Why::NoEditor => "No text editor installed (for example mousepad).\nTo name one yourself: editor in chora-xp.yaml".into(),
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
    since: Instant,
    windows_before: u64,
    bare_before: u64,
    note: Option<u32>,
    shown: bool,
    watched: bool,
    gone: bool,
    ended: Option<Instant>,
    _watch: Option<Watch>,
    next: Next,
}

/// Stops the thread that follows a program once its start is not waited
/// for any more.
#[derive(Debug)]
struct Watch(Arc<AtomicBool>);

impl Drop for Watch {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// What a start falls back to when its program does not open.
#[derive(Clone, Debug, Default)]
struct Next {
    /// The name of the program that runs.
    from: String,
    /// The activation token of the look for the next start.
    activation: Option<String>,
    /// The programs that follow, in order.
    rest: Vec<Candidate>,
    /// The program before this one that did not open, and why; said
    /// only once this one shows its window.
    fell: Option<(String, Why)>,
    /// Where the shell of a start writes its failed exit status. A pidfd
    /// tells that a program ended, not how, and the main process of a unit
    /// is a child of the user manager, not of the look, so there is no
    /// status to wait for; the unit is started with `--collect`, which
    /// removes it, a failed one too, with its result. A status of zero may
    /// be a hand-over to a running instance, so only a failure ends the
    /// wait at once.
    status: Option<Arc<StatusFile>>,
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
        let path = dir.join(format!("chora-launch-exit-{}-{n}", std::process::id()));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()?;
        Some(Self(path))
    }

    /// The command that runs `command` and writes its status here when
    /// it fails. The command runs in a subshell, so that an `exit` or an
    /// `exec` that fails ends the subshell and leaves its status to the
    /// shell around it.
    fn wrap(&self, command: &str) -> String {
        let file = quoted(&self.0.to_string_lossy());
        format!(
            "( {command}\n); status=$?; [ $status -eq 0 ] || ! [ -e {file} ] || echo $status >{file}; exit $status"
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    /// The activation token that finds the window of the program.
    pub token: String,
    /// The pid of its main process, when `choractl` knows it.
    pub pid: Option<u32>,
    /// The systemd unit that runs it, when a user manager does.
    pub unit: Option<String>,
}

/// How a command starts: it answers what `choractl run` answers, or
/// nothing when the program did not start. The command gets the
/// activation token of the look, if the look has one.
pub type Starter = Arc<dyn Fn(&str, Option<&str>) -> Option<Answer> + Send + Sync>;

/// The start of a command that finished, or the end of a program.
enum Heard {
    Started(Started),
    Gone(String),
}

struct Started {
    name: String,
    answer: Option<Answer>,
    since: Instant,
    windows_before: u64,
    bare_before: u64,
    next: Next,
}

/// The programs that started and wait for their first window.
pub struct Launches {
    entries: Vec<Entry>,
    windows: u64,
    /// How many of the windows carried no token: only those can be the
    /// window of a start that loads, so only they end its message.
    bare: u64,
    /// The tokens of the latest windows, for a window that shows before
    /// the answer of its start arrives.
    opened: Vec<String>,
    failed: Vec<Change>,
    heard: Receiver<Heard>,
    sender: Sender<Heard>,
    start: Starter,
}

impl Default for Launches {
    fn default() -> Self {
        Self::new()
    }
}

impl Launches {
    /// No program waits, and a command starts with `choractl run`.
    #[must_use]
    pub fn new() -> Self {
        Self::with_starter(Arc::new(choractl))
    }

    /// No program waits, and a command starts through `start`.
    #[must_use]
    pub fn with_starter(start: Starter) -> Self {
        let (sender, heard) = channel();
        Self {
            entries: Vec::new(),
            windows: 0,
            bare: 0,
            opened: Vec::new(),
            failed: Vec::new(),
            heard,
            sender,
            start,
        }
    }

    /// Starts a shell command as an app. The wait for `choractl run` is
    /// on a thread of its own, so the window manager never blocks; its
    /// answer comes with [`Self::collect`]. A program that is not on
    /// `PATH` never starts: the change says so. The app gets the
    /// activation token of the look, if it has one.
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
        let next = Next {
            from: first.name,
            rest: chain,
            ..template
        };
        if missing(&first.command, crate::desktop_entry::on_path) {
            eprintln!(
                "chora-kit: {:?} is not on PATH, {:?} does not run",
                next.from, first.command
            );
            return self.fail((name.to_owned(), None), Why::Missing, next, now);
        }
        let status = (!DETACHED.contains(&name))
            .then(StatusFile::create)
            .flatten()
            .map(Arc::new);
        let command = status
            .as_ref()
            .map_or_else(|| first.command.clone(), |file| file.wrap(&first.command));
        let next = Next { status, ..next };
        let sender = self.sender.clone();
        let starter = self.start.clone();
        let windows_before = self.windows;
        let bare_before = self.bare;
        let name = name.to_owned();
        std::thread::spawn(move || {
            let answer = starter(&command, next.activation.as_deref());
            let _ = sender.send(Heard::Started(Started {
                name,
                answer,
                since: now,
                windows_before,
                bare_before,
                next,
            }));
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
        while let Ok(heard) = self.heard.try_recv() {
            match heard {
                Heard::Started(started) if started.answer.is_some() => self.add(started),
                Heard::Started(started) => {
                    let name = (started.name, None);
                    let changes = self.fail(name, Why::Refused, started.next, Instant::now());
                    self.failed.extend(changes);
                }
                Heard::Gone(token) => {
                    if let Some(entry) = self.entries.iter_mut().find(|e| e.token == token) {
                        entry.gone = true;
                    }
                }
            }
        }
    }

    fn add(&mut self, started: Started) {
        let Some(Answer { token, pid, unit }) = started.answer else {
            return;
        };
        if self.opened.contains(&token) {
            self.served(started.name, started.next);
            return;
        }
        if DETACHED.contains(&started.name.as_str()) {
            return;
        }
        let watched = pid.is_some() || unit.is_some();
        let stop = Arc::new(AtomicBool::new(false));
        if watched {
            watch(
                self.sender.clone(),
                token.clone(),
                (pid, unit),
                stop.clone(),
            );
        }
        self.entries.push(Entry {
            name: started.name,
            token,
            since: started.since,
            windows_before: started.windows_before,
            bare_before: started.bare_before,
            note: None,
            shown: false,
            watched,
            gone: false,
            ended: None,
            _watch: watched.then(|| Watch(stop)),
            next: started.next,
        });
    }

    /// A window opened, with the activation token that started it if it
    /// has one. Answers the message of the start to close, if there is one.
    pub fn window(&mut self, token: Option<&str>) -> Option<u32> {
        self.drain();
        self.windows += 1;
        let Some(token) = token else {
            self.bare += 1;
            return None;
        };
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

    /// The messages that say a program starts, taken from their starts:
    /// a menu or a popup is about to open where they show. The starts go
    /// on, and one that fails still says so.
    pub fn take_notes(&mut self) -> Vec<u32> {
        self.entries
            .iter_mut()
            .filter_map(|e| e.note.take())
            .collect()
    }

    /// What the programs that wait for a window call for at `now`. A
    /// program that ended with a failure and without a window fails at
    /// once, unless a window of another start opened meanwhile, which a
    /// launcher that hands its work to a running instance explains. One
    /// that ended without a failure fails when no window shows within
    /// [`HAND_OVER`] of its end. A program that runs waits for its window
    /// or its end, however long that takes.
    pub fn check(&mut self, now: Instant) -> Vec<Change> {
        let mut changes = Vec::new();
        let mut failed = Vec::new();
        let (windows, bare) = (self.windows, self.bare);
        self.entries.retain_mut(|entry| {
            let age = now.saturating_duration_since(entry.since);
            let silent = entry.windows_before == windows;
            let crashed = entry.next.status.as_ref().is_some_and(|file| file.failed());
            if entry.gone && silent && crashed {
                failed.push((entry.take(), Why::Exited));
                return false;
            }
            if let Some(why) = entry.over(now) {
                if silent {
                    failed.push((entry.take(), why));
                } else if let Some(note) = entry.note {
                    changes.push(Change::Done { note: Some(note) });
                }
                return false;
            }
            if entry.shown
                && entry.bare_before != bare
                && let Some(note) = entry.note.take()
            {
                changes.push(Change::Done { note: Some(note) });
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
        for ((name, note, next), why) in failed {
            changes.extend(self.fail((name, note), why, next, now));
        }
        changes
    }
}

impl Entry {
    /// Why the wait is over at `now`, if it is: the program ended
    /// [`HAND_OVER`] ago, or nobody can see its end and it started that
    /// long ago.
    fn over(&mut self, now: Instant) -> Option<Why> {
        let (since, why) = if !self.watched {
            (self.since, Why::Slow)
        } else if self.gone {
            (*self.ended.get_or_insert(now), Why::Exited)
        } else {
            return None;
        };
        (now.saturating_duration_since(since) >= HAND_OVER).then_some(why)
    }

    fn take(&mut self) -> (String, Option<u32>, Next) {
        (
            std::mem::take(&mut self.name),
            self.note,
            std::mem::take(&mut self.next),
        )
    }
}

/// Follows the program of a start on a thread of its own and tells
/// `sender` when it ended: its main process is gone and its unit has no
/// process left. A process that a pidfd cannot hold, and a unit without a
/// main process, are looked at once a second. The thread ends with the
/// program, or when `stop` is set because the start is not waited for.
fn watch(
    sender: Sender<Heard>,
    token: String,
    (pid, unit): (Option<u32>, Option<String>),
    stop: Arc<AtomicBool>,
) {
    std::thread::spawn(move || {
        let stopped = || stop.load(Ordering::Relaxed);
        let unit = unit.as_deref();
        let main_gone = pid.is_none_or(|pid| {
            main_process_gone(pid, unit, (&stopped, POLL)).unwrap_or_else(|| {
                gone_by_polling((pid, unit), (&stopped, POLL), (alive, unit_main_pid))
            })
        });
        let unit_empty = || unit.is_none_or(|u| becomes(stopped, POLL, || !unit_active(u)));
        if main_gone && unit_empty() {
            let _ = sender.send(Heard::Gone(token));
        }
    });
}

/// Whether the process `pid` is gone, waiting for its end until `stopped`
/// tells to give up, which it is asked once every `every`; `None` when it
/// cannot be held: the pidfd cannot be opened or the wait was
/// interrupted. The process is held with a pidfd, which tells the end of a
/// process that is no child; a pid that the unit no longer names has
/// ended, and may name another process by now.
fn main_process_gone(
    pid: u32,
    unit: Option<&str>,
    (stopped, every): (&dyn Fn() -> bool, Duration),
) -> Option<bool> {
    use rustix::{
        event::{PollFd, PollFlags, Timespec, poll},
        process::{Pid, PidfdFlags, pidfd_open},
    };
    let Some(raw) = i32::try_from(pid).ok().and_then(Pid::from_raw) else {
        return Some(true);
    };
    let fd = match pidfd_open(raw, PidfdFlags::empty()) {
        Ok(fd) => fd,
        Err(rustix::io::Errno::SRCH) => return Some(true),
        Err(err) => {
            eprintln!("chora-kit: pidfd_open({pid}): {err}");
            return None;
        }
    };
    if unit.is_some_and(|unit| unit_main_pid(unit) != Some(pid)) {
        return Some(true);
    }
    let timeout = Timespec {
        tv_sec: i64::try_from(every.as_secs()).unwrap_or(i64::MAX),
        tv_nsec: i64::from(every.subsec_nanos()),
    };
    loop {
        let mut fds = [PollFd::new(&fd, PollFlags::IN)];
        match poll(&mut fds, Some(&timeout)) {
            Ok(0) if stopped() => return Some(false),
            Ok(0) => {}
            Ok(_) => return Some(true),
            Err(_) => return None,
        }
    }
}

/// The time between two looks at a program that cannot be held with a pidfd.
const POLL: Duration = Duration::from_secs(1);

/// Whether `probe` turns true before `stopped` tells to give up, asked
/// once every `every`.
fn becomes(stopped: impl Fn() -> bool, every: Duration, mut probe: impl FnMut() -> bool) -> bool {
    loop {
        if probe() {
            return true;
        }
        if stopped() {
            return false;
        }
        std::thread::sleep(every);
    }
}

/// Whether the process `pid` is gone before `stopped` tells to give up,
/// found by looking at it once every `every`: it is no process any more, or the unit names
/// another main process.
fn gone_by_polling(
    (pid, unit): (u32, Option<&str>),
    (stopped, every): (impl Fn() -> bool, Duration),
    (alive, main_pid): (impl Fn(u32) -> bool, impl Fn(&str) -> Option<u32>),
) -> bool {
    becomes(stopped, every, || {
        !alive(pid) || unit.is_some_and(|unit| main_pid(unit) != Some(pid))
    })
}

/// The trimmed stdout of `systemctl --user ARGS`; empty when it fails to
/// run.
fn systemctl(args: &[&str]) -> String {
    std::process::Command::new("systemctl")
        .arg("--user")
        .args(args)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// Whether the unit has processes: it is active, or on its way up. A
/// unit that ended is collected and has no state.
fn unit_active(unit: &str) -> bool {
    matches!(
        systemctl(&["show", "--property=ActiveState", "--value", unit]).as_str(),
        "active" | "activating" | "reloading"
    )
}

/// The pid of the main process of the unit, if it has one.
fn unit_main_pid(unit: &str) -> Option<u32> {
    systemctl(&["show", "--property=MainPID", "--value", unit])
        .parse()
        .ok()
        .filter(|pid| *pid > 0)
}

/// A command that runs `choractl run` for a shell command through
/// `program`, with the activation token of the plugin, when it has one,
/// in `CHORA_RUN_TOKEN` and not in the arguments, which every user can
/// read.
fn run_command(program: &[&str], command: &str, token: Option<&str>) -> std::process::Command {
    let mut run = std::process::Command::new(program[0]);
    run.args(&program[1..])
        .args(["run", "--", "sh", "-c", command]);
    if let Some(token) = token {
        run.env("CHORA_RUN_TOKEN", token);
    }
    run
}

/// Starts a shell command as an app with `choractl run` (D42). The app
/// gets the activation token `token`, which the plugin minted on its own
/// connection with [`crate::conn::Conn::activation_token`] so that it
/// carries the act that the plugin received, or else a token of
/// `choractl`; and a unit of its own when a user manager runs. `&` lets
/// the shell exit at once, so no zombie stays. The program is not
/// followed: [`Launches`] does that.
pub fn run(command: &str, token: Option<&str>) {
    let program = ["sh", "-c", r#"choractl "$@" >/dev/null &"#, "choractl"];
    let started = run_command(&program, command, token).status();
    if let Err(err) = started {
        eprintln!("cannot run {command:?}: {err}");
    }
}

/// Starts a shell command as [`run`] does, and answers the activation
/// token that finds its window and the pid of the app, if `choractl`
/// knows it.
#[must_use]
pub fn start(command: &str, token: Option<&str>) -> Option<(String, Option<u32>)> {
    let answer = choractl(command, token)?;
    Some((answer.token, answer.pid))
}

/// Starts a shell command with `choractl run` and answers what it
/// prints: token, pid and unit.
fn choractl(command: &str, token: Option<&str>) -> Option<Answer> {
    let run = run_command(&["choractl"], command, token)
        .stdin(std::process::Stdio::null())
        .output();
    let line = match run {
        Ok(out) if out.status.success() => out.stdout,
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            eprintln!("cannot run {command:?}: {}", err.trim());
            return None;
        }
        Err(err) => {
            eprintln!("cannot run {command:?}: choractl run: {err}");
            return None;
        }
    };
    let answer: serde_json::Value = serde_json::from_slice(&line).unwrap_or_default();
    Some(Answer {
        token: answer["token"].as_str()?.into(),
        pid: answer["pid"].as_u64().and_then(|p| u32::try_from(p).ok()),
        unit: answer["unit"].as_str().map(str::to_owned),
    })
}

/// Whether a process runs: it has a `/proc` entry and is no zombie. A
/// pid can be reused, so the answer is a hint.
#[must_use]
pub fn alive(pid: u32) -> bool {
    let stat = crate::read_text(std::path::Path::new(&format!("/proc/{pid}/stat")), 4096);
    stat.ok()
        .and_then(|s| Some(s[s.rfind(')')? + 1..].trim_start().starts_with('Z')))
        .is_some_and(|zombie| !zombie)
}

/// A word in single quotes for the shell.
fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
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
/// command runs, and the shell finds the program or does not.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn launches() -> (Launches, Instant) {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(started(now));
        (launches, now)
    }

    fn answer(pid: Option<u32>) -> Answer {
        Answer {
            token: "t1".into(),
            pid,
            unit: None,
        }
    }

    fn started(since: Instant) -> Started {
        Started {
            name: "writer".into(),
            answer: Some(answer(None)),
            since,
            windows_before: 0,
            bare_before: 0,
            next: Next::default(),
        }
    }

    fn none() -> Vec<Change> {
        Vec::new()
    }

    fn seconds(now: Instant, s: u64) -> Instant {
        now + Duration::from_secs(s)
    }

    fn never() -> bool {
        false
    }

    #[test]
    fn a_probe_that_turns_true_before_the_deadline_is_true() {
        let mut asked = 0;
        let turned = becomes(never, Duration::from_millis(5), || {
            asked += 1;
            asked == 3
        });
        assert!(turned);
        assert_eq!(asked, 3);
    }

    #[test]
    fn a_probe_that_stays_false_is_false_at_the_deadline() {
        let begun = Instant::now();
        let turned = becomes(
            || begun.elapsed() >= Duration::from_millis(100),
            Duration::from_millis(5),
            || false,
        );
        assert!(!turned);
        assert!(begun.elapsed() >= Duration::from_millis(100));
    }

    #[test]
    fn a_process_that_cannot_be_held_is_followed_by_looking_at_it() {
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap();
        let pid = child.id();
        let every = Duration::from_millis(5);
        let probes = (alive, |_: &str| None);
        let begun = Instant::now();
        let soon = || begun.elapsed() >= Duration::from_millis(100);
        assert!(!gone_by_polling((pid, None), (soon, every), probes));
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(gone_by_polling((pid, None), (never, every), probes));
    }

    #[test]
    fn a_unit_that_names_another_main_process_ends_the_wait() {
        let every = Duration::from_millis(5);
        let own = std::process::id();
        let probes = (alive, |_: &str| Some(own + 1));
        assert!(gone_by_polling(
            (own, Some("app.service")),
            (never, every),
            probes
        ));
    }

    #[test]
    fn a_wait_for_a_process_that_runs_ends_when_it_is_told_to_stop() {
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap();
        let every = Duration::from_millis(10);
        let stop = AtomicBool::new(false);
        let stopped = || stop.load(Ordering::Relaxed);
        std::thread::scope(|scope| {
            let wait = scope.spawn(|| main_process_gone(child.id(), None, (&stopped, every)));
            std::thread::sleep(Duration::from_millis(50));
            assert!(!wait.is_finished(), "a running process is waited for");
            stop.store(true, Ordering::Relaxed);
            assert_eq!(wait.join().unwrap(), Some(false));
        });
        child.kill().unwrap();
        child.wait().unwrap();
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
    fn a_window_without_the_token_ends_the_message_of_a_start_that_shows_one() {
        let (mut launches, now) = launches();
        launches.check(seconds(now, 2));
        launches.noted("t1", 12);
        assert_eq!(launches.window(None), None);
        assert_eq!(
            launches.check(seconds(now, 3)),
            [Change::Done { note: Some(12) }]
        );
        assert_eq!(launches.check(seconds(now, 4)), none());
    }

    #[test]
    fn the_window_of_another_start_leaves_the_message_of_a_start() {
        let (mut launches, now) = launches();
        launches.add(Started {
            answer: Some(Answer {
                token: "t2".into(),
                pid: None,
                unit: None,
            }),
            name: "browser".into(),
            ..started(now)
        });
        assert_eq!(launches.check(seconds(now, 2)).len(), 2, "both start");
        launches.noted("t1", 12);
        launches.noted("t2", 13);
        assert_eq!(launches.window(Some("t2")), Some(13));
        assert_eq!(launches.check(seconds(now, 3)), none(), "t1 still loads");
        assert_eq!(launches.window(Some("t1")), Some(12));
    }

    #[test]
    fn a_popup_takes_the_messages_and_the_start_still_gives_up() {
        let (mut launches, now) = launches();
        launches.check(seconds(now, 2));
        launches.noted("t1", 12);
        assert_eq!(launches.take_notes(), [12]);
        assert_eq!(launches.take_notes(), Vec::<u32>::new());
        assert_eq!(
            launches.check(seconds(now, HAND_OVER.as_secs())),
            [Change::Failed {
                name: "writer".into(),
                note: None,
                why: Why::Slow
            }]
        );
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
        assert_eq!(launches.check(seconds(now, HAND_OVER.as_secs())), none());
    }

    #[test]
    fn a_program_without_a_window_gives_up_after_the_limit() {
        let (mut launches, now) = launches();
        assert!(
            launches
                .check(seconds(now, HAND_OVER.as_secs() - 1))
                .iter()
                .all(|c| matches!(c, Change::Starting { .. }))
        );
        let changes = launches.check(seconds(now, HAND_OVER.as_secs()));
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

    /// A start whose main process is this test process: it never ends.
    fn running(now: Instant, fell: Option<(String, Why)>) -> Launches {
        let mut launches = Launches::new();
        launches.add(Started {
            answer: Some(answer(Some(std::process::id()))),
            next: Next {
                fell,
                ..Next::default()
            },
            ..started(now)
        });
        launches
    }

    #[test]
    fn a_running_program_without_a_window_is_not_served_at_the_old_limit() {
        let now = Instant::now();
        let mut launches = running(now, Some(("pad".to_owned(), Why::Missing)));
        launches.check(seconds(now, 2));
        for s in [30, 31, 300] {
            launches.check(seconds(now, s));
            assert_eq!(launches.collect(), none(), "{s} s: served without a window");
        }
    }

    #[test]
    fn a_running_program_without_a_window_gives_no_message_after_the_old_limit() {
        let now = Instant::now();
        let mut launches = running(now, None);
        launches.check(seconds(now, 2));
        launches.noted("t1", 12);
        for s in [30, 31, 300] {
            assert_eq!(launches.check(seconds(now, s)), none(), "{s} s");
        }
        assert_eq!(launches.window(Some("t1")), Some(12), "it still starts");
    }

    #[test]
    fn a_program_that_ended_without_a_window_is_a_failure_at_once() {
        let now = Instant::now();
        let file = StatusFile::create().expect("a status file");
        std::fs::write(&file.0, "1\n").unwrap();
        let mut launches = Launches::new();
        launches.add(Started {
            answer: Some(answer(Some(u32::MAX))),
            next: Next {
                status: Some(Arc::new(file)),
                ..Next::default()
            },
            ..started(now)
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !launches.entries[0].gone && Instant::now() < deadline {
            launches.drain();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(matches!(
            launches.check(now).as_slice(),
            [Change::Failed {
                why: Why::Exited,
                ..
            }]
        ));
    }

    #[test]
    fn a_program_that_ended_without_a_failure_or_a_window_ends_the_wait_late() {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(Started {
            answer: Some(answer(Some(u32::MAX))),
            ..started(now)
        });
        launches.entries[0].gone = true;
        launches.check(seconds(now, 5));
        assert!(
            launches
                .check(seconds(now, 5 + HAND_OVER.as_secs() - 1))
                .iter()
                .all(|c| !matches!(c, Change::Failed { .. }))
        );
        let changes = launches.check(seconds(now, 5 + HAND_OVER.as_secs()));
        assert!(
            matches!(
                changes.as_slice(),
                [Change::Failed {
                    why: Why::Exited,
                    ..
                }]
            ),
            "{changes:?}"
        );
    }

    #[test]
    fn a_hand_over_that_showed_a_window_of_another_start_closes_its_message() {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(Started {
            answer: Some(answer(Some(u32::MAX))),
            ..started(now)
        });
        launches.check(seconds(now, 2));
        launches.noted("t1", 12);
        launches.entries[0].gone = true;
        launches.window(Some("other"));
        launches.check(seconds(now, 3));
        assert_eq!(
            launches.check(seconds(now, 3 + HAND_OVER.as_secs())),
            [Change::Done { note: Some(12) }]
        );
        assert_eq!(launches.check(seconds(now, 200)), none());
    }

    #[test]
    fn an_end_with_a_window_of_another_start_is_waited_for_and_not_told() {
        let now = Instant::now();
        let mut launches = Launches::new();
        launches.add(started(now));
        launches.entries[0].gone = true;
        launches.window(Some("other"));
        assert_eq!(launches.check(seconds(now, 1)), none());
        assert_eq!(launches.check(seconds(now, HAND_OVER.as_secs())), none());
    }

    fn chain() -> Vec<Candidate> {
        ["writer", "gedit", "nano"]
            .map(|name| Candidate {
                name: name.into(),
                command: format!("chora-no-such-{name}"),
            })
            .to_vec()
    }

    #[test]
    fn a_chain_of_missing_programs_says_only_that_the_app_cannot_open() {
        let mut launches = Launches::new();
        let changes = launches.run_chain(
            "Notepad",
            (chain(), Some("pad".into())),
            None,
            Instant::now(),
        );
        let closes = |changes: &[Change]| {
            changes
                .iter()
                .all(|c| matches!(c, Change::Done { note: None }))
        };
        let (last, before) = changes.split_last().expect("a change");
        assert!(closes(before), "{changes:?}");
        assert_eq!(
            last,
            &Change::Failed {
                name: "Notepad".into(),
                note: None,
                why: Why::Missing
            }
        );
        assert_eq!(launches.collect(), none());
    }

    #[test]
    fn an_empty_chain_says_that_no_text_editor_is_installed() {
        let mut launches = Launches::new();
        let changes = launches.run_chain("Notepad", (Vec::new(), None), None, Instant::now());
        let [
            change @ Change::Failed {
                why: Why::NoEditor, ..
            },
        ] = changes.as_slice()
        else {
            panic!("{changes:?}");
        };
        assert_eq!(
            change.text(),
            (
                "Cannot open Notepad".to_owned(),
                "No text editor installed (for example mousepad).\nTo name one yourself: editor in chora-xp.yaml"
                    .to_owned()
            )
        );
    }

    fn ended_editor(now: Instant, status: Option<&str>, rest: Vec<Candidate>) -> Launches {
        let file = StatusFile::create().expect("a status file");
        if let Some(text) = status {
            std::fs::write(&file.0, text).unwrap();
        }
        let mut launches = Launches::new();
        launches.add(Started {
            name: "Notepad".into(),
            next: Next {
                from: "writer".into(),
                rest,
                status: Some(Arc::new(file)),
                ..Next::default()
            },
            ..started(now)
        });
        launches.entries[0].gone = true;
        launches
    }

    #[test]
    fn an_editor_that_failed_gives_way_to_the_next_and_says_so_when_it_shows() {
        let now = Instant::now();
        let mut launches = ended_editor(now, Some("1\n"), chain()[1..].to_vec());
        let changes = launches.check(seconds(now, 0));
        assert!(
            changes
                .iter()
                .all(|c| !matches!(c, Change::Fallback { .. })),
            "{changes:?}"
        );
        let fell = Some(("writer".to_owned(), Why::Exited));
        let next = Next {
            from: "gedit".into(),
            fell,
            ..Next::default()
        };
        launches.add(Started {
            name: "Notepad".into(),
            answer: Some(Answer {
                token: "t2".into(),
                pid: None,
                unit: None,
            }),
            next,
            ..started(now)
        });
        let waiting = launches.check(seconds(now, 5));
        assert!(
            waiting.iter().all(|c| matches!(c, Change::Starting { .. })),
            "{waiting:?}"
        );
        launches.window(Some("t2"));
        let told = launches.collect();
        assert_eq!(
            told.first().map(Change::text),
            Some((
                "Notepad opened with gedit".into(),
                "writer ended without a window.".into()
            ))
        );
    }

    #[test]
    fn a_fallback_that_ends_without_a_window_leaves_no_notice_of_success() {
        let now = Instant::now();
        let mut launches = ended_editor(now, Some("1\n"), Vec::new());
        let fell = Some(("writer".to_owned(), Why::Exited));
        launches.entries[0].next.fell = fell;
        let changes = launches.check(seconds(now, 0));
        assert!(
            matches!(
                changes.as_slice(),
                [Change::Failed { name, why: Why::Exited, .. }] if name == "Notepad"
            ),
            "{changes:?}"
        );
        assert_eq!(launches.collect(), none());
    }

    #[test]
    fn an_editor_that_handed_over_with_status_zero_is_waited_for() {
        let now = Instant::now();
        let mut launches = ended_editor(now, None, chain()[1..].to_vec());
        for s in [2, 5, 10, HAND_OVER.as_secs() - 1] {
            let changes = launches.check(seconds(now, s));
            assert!(
                changes.iter().all(|c| matches!(c, Change::Starting { .. })),
                "{s}: {changes:?}"
            );
        }
        launches.window(Some("t1"));
        assert_eq!(launches.check(seconds(now, 40)), none());
        assert_eq!(launches.collect(), none());
    }

    #[test]
    fn the_status_file_goes_with_the_start() {
        let file = StatusFile::create().expect("a status file");
        let path = file.0.clone();
        assert!(path.exists() && !file.failed());
        assert!(file.wrap("lowriter -n").contains("lowriter -n"));
        drop(file);
        assert!(!path.exists());
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

    #[test]
    fn the_token_of_a_run_goes_in_the_environment_not_the_arguments() {
        let run = run_command(&["choractl"], "foot", Some("secret"));
        assert!(run.get_args().all(|arg| arg != "secret"));
        let env: Vec<_> = run.get_envs().collect();
        let token = std::ffi::OsStr::new("secret");
        assert_eq!(
            env,
            [(std::ffi::OsStr::new("CHORA_RUN_TOKEN"), Some(token))]
        );
    }

    #[test]
    fn a_running_process_is_alive_and_a_missing_one_not() {
        assert!(alive(std::process::id()));
        assert!(!alive(u32::MAX));
    }
}
