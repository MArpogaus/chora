//! The launcher of the programs: the installed applications of the
//! desktop files as the rows of `fuzzel --dmenu`, sorted by name. Text
//! that matches no row finds programs by their keywords, generic name or
//! command, then runs as a command when its program is installed, and
//! else the list shows again under a row that says that nothing was
//! found, so the launcher never closes without a word.

use std::{
    io::Write as _,
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{
    desktop_entry::{self, Entry},
    words::{Language, Word},
};

/// How often the list shows again after text that found nothing.
const ROUNDS: usize = 8;

/// A program of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    /// The name, the text of its row.
    pub name: String,
    /// The shell command, in the terminal if the program needs one.
    pub command: String,
    /// The icon name or path of the entry, empty if it has none.
    icon: String,
    /// The words that find it besides its name, in lower case.
    words: Vec<String>,
}

/// The programs of the entries that are shown and installed, sorted by
/// name without regard to case; of two with one name, the first.
/// `terminal` runs the programs that need one.
#[must_use]
pub fn apps(
    entries: impl IntoIterator<Item = Entry>,
    terminal: &str,
    on_path: impl Fn(&str) -> bool,
) -> Vec<App> {
    let mut apps: Vec<App> = entries
        .into_iter()
        .filter(|e| !e.hidden && !e.exec.is_empty() && e.installed(&on_path))
        .map(|e| {
            let program = e.exec.split_whitespace().next().unwrap_or_default();
            let program = program.rsplit('/').next().unwrap_or(program);
            let mut words: Vec<String> = e.keywords.iter().map(|k| k.to_lowercase()).collect();
            words.extend([e.generic.to_lowercase(), program.to_lowercase()]);
            words.retain(|w| !w.is_empty());
            App {
                name: e.name,
                command: desktop_entry::command(e.exec, e.terminal, terminal),
                icon: e.icon,
                words,
            }
        })
        .collect();
    apps.sort_by_cached_key(|a| a.name.to_lowercase());
    apps.dedup_by(|b, a| a.name == b.name);
    apps
}

impl App {
    /// The row for fuzzel's dmenu: the name, and when the entry has an icon
    /// the rofi syntax `name\0icon\x1fICON` that fuzzel (1.7 and later)
    /// draws as the icon of the row. It prints the name only, so the line it
    /// answers still matches the program; a fuzzel without the syntax or
    /// with no such icon shows the name alone.
    fn row(&self) -> String {
        if self.icon.is_empty() {
            self.name.clone()
        } else {
            format!("{}\0icon\x1f{}", self.name, self.icon)
        }
    }
}

/// The program of the row that opens the chat with a helper, and of the
/// helper word of the bar.
pub const HELPER_PROGRAM: &str = "chora-chat";

/// The programs of the rows that the flavor of tiling adds to its Apps
/// list: its theme rows, its settings row and its rows that move a window.
const FLAVOR_ROWS: [&str; 3] = ["look", "settings", "give"];

/// The word of the flavor row that the program is: `look`, `settings` or
/// `give` of `chora-tiling --launch look|settings|give`; none for a
/// program. The screenshot rows of the shared folder are programs.
fn flavor_program(app: &App) -> Option<&str> {
    let mut words = app.command.split_whitespace();
    (words.next() == Some("chora-tiling") && words.next() == Some("--launch"))
        .then(|| words.next())
        .flatten()
        .filter(|w| FLAVOR_ROWS.contains(w))
}

/// Whether the program is a row of the flavor of tiling.
fn flavor_row(app: &App) -> bool {
    flavor_program(app).is_some()
}

/// The list without the rows that move the focused window to another
/// look, which fail when no window has the focus: the one rule for the
/// Apps list and the Looks menu of the bar.
#[must_use]
pub fn without_moves(mut apps: Vec<App>) -> Vec<App> {
    apps.retain(|a| flavor_program(a) != Some("give"));
    apps
}

/// The Apps list of tiling: the row of the chat with a helper first, so
/// that the nine rows that fuzzel shows always hold it and Enter on the
/// preselected row opens a chat; then the programs; then the rows of the
/// flavor. Each group stays sorted as it was. The program of the helper row
/// is the last of the words.
#[must_use]
pub fn tiling_order(mut apps: Vec<App>) -> Vec<App> {
    let group = |a: &App| {
        if a.words.last().is_some_and(|w| w == HELPER_PROGRAM) {
            0
        } else if flavor_row(a) {
            2
        } else {
            1
        }
    };
    apps.sort_by_cached_key(group);
    apps
}

/// The name of the helper in the words of a door (`atlas · idle`, from
/// `chora-chat --door`); none for the door without a helper (`No helper`)
/// and for an empty one.
#[must_use]
pub fn helper_of_door(door: &str) -> Option<String> {
    let door = door.trim();
    let none = [Language::En, Language::De].map(|l| Word::NoHelperDoor.text(l));
    if door.is_empty() || none.contains(&door) {
        return None;
    }
    door.split(" · ")
        .next()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

/// The name of the helper that its door says, from `chora-chat --door`;
/// none without a helper, without the program, or when it does not answer
/// within two seconds.
#[must_use]
pub fn helper_name() -> Option<String> {
    let (send, receive) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let out = Command::new(HELPER_PROGRAM)
            .arg("--door")
            .stderr(Stdio::null())
            .output()
            .ok()
            .filter(|out| out.status.success());
        let _ = send.send(out.map(|out| String::from_utf8_lossy(&out.stdout).into_owned()));
    });
    let door = receive
        .recv_timeout(std::time::Duration::from_secs(2))
        .ok()??;
    helper_of_door(&door)
}

/// The row of the chat with a helper names the helper: `Chat with a
/// helper (atlas)`, so that its name finds it as the word helper does. The
/// name stands in plain ASCII: fuzzel in the C locale matches no row that has
/// a multibyte character such as the middle dot of a door word.
#[must_use]
pub fn helper_named(mut apps: Vec<App>, name: &str) -> Vec<App> {
    for app in apps
        .iter_mut()
        .filter(|a| a.words.last().is_some_and(|w| w == HELPER_PROGRAM))
    {
        app.name = format!("{} ({name})", app.name);
    }
    apps
}

/// What the answer of the list calls for.
#[derive(Debug, PartialEq, Eq)]
pub enum Answer<'a> {
    /// Nothing: the person closed the list.
    Closed,
    /// A shell command to run.
    Run(String),
    /// These programs match the text: the list shows them.
    Narrow(Vec<&'a App>),
    /// The text found nothing: the list shows again and says so.
    Nothing(String),
}

/// The row that says that `text` found nothing.
#[must_use]
pub fn nothing_found(text: &str) -> String {
    format!("Nothing found for \"{text}\"")
}

/// What the line that the list printed calls for: the row of a program
/// runs it; other text finds the programs with a word that holds it, one
/// of which runs at once; text whose first word is an installed program
/// runs as a command; any other text found nothing. The row of an earlier
/// `notice` closes the list.
#[must_use]
pub fn answer<'a>(
    apps: &'a [App],
    line: &str,
    notice: Option<&str>,
    on_path: impl Fn(&str) -> bool,
) -> Answer<'a> {
    let text = line.trim();
    if text.is_empty() || notice == Some(text) {
        return Answer::Closed;
    }
    if let Some(app) = apps.iter().find(|a| a.name == text) {
        return Answer::Run(app.command.clone());
    }
    let low = text.to_lowercase();
    let found: Vec<&App> = apps
        .iter()
        .filter(|a| {
            a.name.to_lowercase().contains(&low) || a.words.iter().any(|w| w.contains(&low))
        })
        .collect();
    match found.as_slice() {
        [] if desktop_entry::installed(text, on_path) => Answer::Run(text.to_owned()),
        [] => Answer::Nothing(text.to_owned()),
        [app] => Answer::Run(app.command.clone()),
        _ => Answer::Narrow(found),
    }
}

/// What the list of fuzzel came to.
enum Asked {
    /// The line that it printed: the row that the person chose, or the
    /// text that matched no row.
    Line(String),
    /// The person closed it.
    Closed,
    /// It did not run: the first line that tells why.
    Failed(String),
}

/// Shows the rows with `fuzzel --dmenu`. The exit code cannot tell a
/// close from a failure (fuzzel exits 1 on Escape and on every error),
/// so the stderr decides: a fuzzel that prints nothing and whose first
/// line starting with "err:" (its log prefix, after trimming) is absent
/// was closed by the person, whatever its exit code, a signal included;
/// one with such a line failed, and the text after the prefix is why.
/// A warning or info line alone is no failure. One that cannot start
/// failed too.
fn ask(fuzzel: &[String], rows: &[String]) -> Asked {
    let Some((program, args)) = fuzzel.split_first() else {
        return Asked::Failed("no launcher command".into());
    };
    let spawned = Command::new(program)
        .args(args)
        .arg("--dmenu")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => return Asked::Failed(format!("cannot start {program}: {err}")),
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(rows.join("\n").as_bytes());
    }
    let out = match child.wait_with_output() {
        Ok(out) => out,
        Err(err) => return Asked::Failed(format!("cannot wait for {program}: {err}")),
    };
    let line = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if !line.is_empty() {
        return Asked::Line(line);
    }
    let err = String::from_utf8_lossy(&out.stderr);
    match err.lines().find_map(|l| l.trim().strip_prefix("err:")) {
        Some(why) => Asked::Failed(without_position(why.trim()).to_owned()),
        None => Asked::Closed,
    }
}

/// [`ask`], and when a fuzzel with a `--config FILE` fails (a fuzzel that
/// does not know a key of the file, as 1.9.2 does not know `prompt`, exits
/// at once), once more without the config: the launcher in the plain look
/// is a way to start a program, a launcher that shows nothing is none.
fn ask_or_plain(fuzzel: &[String], rows: &[String]) -> Asked {
    let asked = ask(fuzzel, rows);
    match (&asked, without_config(fuzzel)) {
        (Asked::Failed(_), Some(plain)) => ask(&plain, rows),
        _ => asked,
    }
}

/// The command without its `--config FILE` (or `--config=FILE`), or
/// none when it has no config.
fn without_config(fuzzel: &[String]) -> Option<Vec<String>> {
    let at = fuzzel
        .iter()
        .position(|arg| arg == "--config" || arg.starts_with("--config="))?;
    let width = if fuzzel[at] == "--config" { 2 } else { 1 };
    let mut plain = fuzzel.to_vec();
    plain.drain(at..(at + width).min(fuzzel.len()));
    Some(plain)
}

/// The reason without the "<file>.c:<line>: " that fuzzel puts in front
/// of it: a person has no use for its source position.
fn without_position(why: &str) -> &str {
    let rest = why.split_once(".c:").filter(|(file, _)| {
        !file.is_empty()
            && file
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-".contains(c))
    });
    let Some((_, rest)) = rest else { return why };
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    match rest[digits..].strip_prefix(": ") {
        Some(reason) if digits > 0 => reason,
        _ => why,
    }
}

/// Lets the person choose a program with `fuzzel`, the command line of
/// fuzzel with its config file, and answers the shell command to run.
/// After text that found nothing the list shows again, with the row of
/// [`nothing_found`] first. A list that cannot start is logged.
#[must_use]
pub fn choose(fuzzel: &[String], apps: &[App]) -> Option<String> {
    choose_reporting(fuzzel, apps, &mut |why| eprintln!("{why}"))
}

/// Like [`choose`], and `report` gets, once, the line "Run could not
/// start: ..." when the list did not run. Closing the list is silent.
#[must_use]
pub fn choose_reporting(
    fuzzel: &[String],
    apps: &[App],
    report: &mut dyn FnMut(&str),
) -> Option<String> {
    let all: Vec<&App> = apps.iter().collect();
    let mut rows = all.clone();
    let mut notice: Option<String> = None;
    for _ in 0..ROUNDS {
        let shown: Vec<String> = notice
            .iter()
            .cloned()
            .chain(rows.iter().map(|a| a.row()))
            .collect();
        let line = match ask_or_plain(fuzzel, &shown) {
            Asked::Line(line) => line,
            Asked::Closed => return None,
            Asked::Failed(why) => {
                report(&format!("Run could not start: {why}"));
                return None;
            }
        };
        match answer(apps, &line, notice.as_deref(), desktop_entry::on_path) {
            Answer::Closed => return None,
            Answer::Run(command) => return Some(command),
            Answer::Narrow(found) => {
                rows = found;
                notice = None;
            }
            Answer::Nothing(text) => {
                rows.clone_from(&all);
                notice = Some(nothing_found(&text));
            }
        }
    }
    None
}

/// Shows `why` as a notice of the look, on a short connection of its
/// own (the launcher is a process that the look started); the log has it
/// too, so a look without a notice server loses nothing.
fn tell(why: &str) {
    eprintln!("{why}");
    let Ok(mut conn) = crate::conn::Conn::connect("launcher")
        .inspect_err(|err| eprintln!("launcher: the notice cannot connect: {err}"))
    else {
        return;
    };
    let _ = crate::notice::show_or_log(
        &mut conn,
        None,
        &crate::notice::Notice {
            app: "launcher",
            icon: "dialog-error",
            summary: "Run could not start",
            body: why.strip_prefix("Run could not start: ").unwrap_or(why),
            expire_ms: -1,
            urgency: crate::notice::Urgency::Normal,
        },
    );
}

/// The data directories of the launcher: `first`, the one of a flavor,
/// then the first folder of `shared` that has `applications` (Chat with a
/// helper, Screenshot), then the XDG ones.
fn entry_dirs(first: Option<PathBuf>, shared: &[PathBuf], xdg: Vec<PathBuf>) -> Vec<PathBuf> {
    (first.into_iter())
        .chain(crate::shared::entries_in(shared).cloned())
        .chain(xdg)
        .collect()
}

/// The launcher: the person chooses one of the installed applications,
/// which starts as an app with `choractl run`. `first` is a data
/// directory in front of the folder that every look shares and the XDG
/// ones, for the entries of a flavor.
pub fn run(fuzzel: &[String], terminal: &str, first: Option<PathBuf>) {
    run_ordered(fuzzel, terminal, first, None);
}

/// What the Apps list of tiling adds to [`run`].
#[derive(Clone, Copy, Debug)]
pub struct Tiling<'a> {
    /// The name of the helper that the row of the chat names, if any.
    pub helper: Option<&'a str>,
    /// Whether a window has the focus; without one the rows that move a
    /// window are left out ([`without_moves`]).
    pub window: bool,
}

/// Like [`run`], and with `tiling` the list is in the order of
/// [`tiling_order`], its chat row names the helper and its rows that move
/// a window are there only with a window.
pub fn run_ordered(
    fuzzel: &[String],
    terminal: &str,
    first: Option<PathBuf>,
    tiling: Option<Tiling<'_>>,
) {
    let dirs = entry_dirs(first, &crate::shared::dirs(), crate::xdg::data_dirs());
    let entries = desktop_entry::applications_in(&dirs);
    let mut apps = apps(entries, terminal, desktop_entry::on_path);
    if let Some(tiling) = tiling {
        if !tiling.window {
            apps = without_moves(apps);
        }
        apps = tiling_order(apps);
        if let Some(name) = tiling.helper {
            apps = helper_named(apps, name);
        }
    }
    if let Some(command) = choose_reporting(fuzzel, &apps, &mut tell) {
        crate::launch::run(&command, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, exec: &str) -> Entry {
        Entry {
            name: name.into(),
            exec: exec.into(),
            ..Entry::default()
        }
    }

    #[test]
    fn the_list_reads_the_flavor_then_the_shared_folder_then_the_system() {
        let shared = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/shared");
        let dirs = entry_dirs(
            Some("/flavor".into()),
            &["/none".into(), shared.clone()],
            vec!["/usr/share".into()],
        );
        assert_eq!(
            dirs,
            [PathBuf::from("/flavor"), shared, "/usr/share".into()]
        );
        let names: Vec<String> = (desktop_entry::applications_in(&dirs[1..2]).into_iter())
            .map(|entry| entry.name)
            .collect();
        assert!(
            names.contains(&"Chat with a helper".to_owned()),
            "{names:?}"
        );
    }

    #[test]
    fn the_helper_row_names_the_helper_that_the_door_names() {
        assert_eq!(helper_of_door("atlas · idle").as_deref(), Some("atlas"));
        assert_eq!(helper_of_door("coder").as_deref(), Some("coder"));
        assert_eq!(helper_of_door("No helper"), None);
        assert_eq!(helper_of_door("Kein Helfer"), None);
        assert_eq!(helper_of_door("  "), None);
        let list = apps(
            [
                entry("Chat with a helper", "chora-chat"),
                entry("Zed", "foot"),
            ],
            "foot",
            |_| true,
        );
        let names: Vec<String> = helper_named(list, "atlas")
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, ["Chat with a helper (atlas)", "Zed"]);
    }

    #[test]
    fn the_helper_row_stands_first_and_the_flavor_rows_last() {
        let list = apps(
            [
                entry("Dark theme", "chora-tiling --launch look dark"),
                entry("Chat with a helper", "chora-chat"),
                entry("Zed", "foot"),
                entry("Edit settings", "chora-tiling --launch settings"),
                entry("Alpha", "foot --alpha"),
                entry("Shot", "chora-tiling --launch screenshot"),
            ],
            "foot",
            |_| true,
        );
        let names: Vec<String> = tiling_order(list).into_iter().map(|a| a.name).collect();
        assert_eq!(
            names,
            [
                "Chat with a helper",
                "Alpha",
                "Shot",
                "Zed",
                "Dark theme",
                "Edit settings"
            ]
        );
    }

    #[test]
    fn the_rows_that_move_a_window_are_the_give_rows_only() {
        let list = apps(
            [
                entry(
                    "Move window to the xp desktop",
                    "chora-tiling --launch give xp",
                ),
                entry("Dark theme", "chora-tiling --launch look dark"),
                entry("Edit settings", "chora-tiling --launch settings"),
                entry("Give", "give xp"),
            ],
            "foot",
            |_| true,
        );
        let names: Vec<String> = without_moves(list).into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["Dark theme", "Edit settings", "Give"]);
    }

    fn on_path(program: &str) -> bool {
        matches!(program, "foot" | "shot" | "look" | "pad" | "htop")
    }

    fn list() -> Vec<App> {
        apps(
            [
                entry("Screenshot", "shot"),
                entry("terminal", "foot"),
                Entry {
                    keywords: vec!["Theme".into(), "colour".into()],
                    ..entry("Dark theme", "look dark")
                },
                Entry {
                    keywords: vec!["theme".into()],
                    ..entry("Light theme", "look light")
                },
                Entry {
                    generic: "Text Editor".into(),
                    ..entry("Pad", "/chora-no-such-dir/pad")
                },
                Entry {
                    generic: "Text Editor".into(),
                    ..entry("Notes", "pad --notes")
                },
                entry("Writer", "libreoffice --writer"),
                Entry {
                    hidden: true,
                    ..entry("Hidden", "foot")
                },
                Entry {
                    terminal: true,
                    ..entry("Monitor", "htop")
                },
                entry("Screenshot", "foot --second"),
            ],
            "foot",
            on_path,
        )
    }

    #[test]
    fn the_list_is_sorted_and_holds_only_installed_programs() {
        let list = list();
        let names: Vec<&str> = list.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Dark theme",
                "Light theme",
                "Monitor",
                "Notes",
                "Screenshot",
                "terminal"
            ]
        );
        assert_ne!(names[0], "Screenshot");
        assert_eq!(list[2].command, "foot -e htop");
        assert_eq!(list[4].command, "shot");
    }

    #[test]
    fn a_row_runs_its_program_and_no_text_closes() {
        let list = list();
        assert_eq!(
            answer(&list, "Screenshot\n", None, on_path),
            Answer::Run("shot".into())
        );
        assert_eq!(answer(&list, "  ", None, on_path), Answer::Closed);
    }

    #[test]
    fn text_finds_programs_by_their_words() {
        let list = list();
        assert_eq!(
            answer(&list, "editor", None, on_path),
            Answer::Run("pad --notes".into())
        );
        let Answer::Narrow(found) = answer(&list, "THEME", None, on_path) else {
            panic!("two programs")
        };
        let names: Vec<&str> = found.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Dark theme", "Light theme"]);
    }

    #[test]
    fn text_that_is_a_command_runs_and_other_text_finds_nothing() {
        let list = list();
        assert_eq!(
            answer(&list, "foot -e top", None, on_path),
            Answer::Run("foot -e top".into())
        );
        assert_eq!(
            answer(&list, "zzqq", None, on_path),
            Answer::Nothing("zzqq".into())
        );
        let notice = nothing_found("zzqq");
        assert_eq!(notice, "Nothing found for \"zzqq\"");
        assert_eq!(
            answer(&list, &notice, Some(&notice), on_path),
            Answer::Closed
        );
    }

    /// A stand-in for fuzzel: a shell script with the `body`, run by `sh`
    /// (as a program, a script that a parallel test just wrote can fail
    /// to exec with "text file busy").
    fn stand_in(name: &str, body: &str) -> (crate::testing::TempDir, Vec<String>) {
        let dir = crate::testing::TempDir::new(name);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("fuzzel");
        std::fs::write(&file, format!("#!/bin/sh\ncat >/dev/null\n{body}\n")).unwrap();
        let command = vec!["sh".to_owned(), file.to_string_lossy().into_owned()];
        (dir, command)
    }

    /// The rows that fuzzel gets on stdin carry the icon name of the desktop
    /// entry (`label\0icon\x1fNAME`, the rofi syntax of fuzzel's dmenu), a
    /// row of an entry without one stays plain text, and the label that
    /// fuzzel prints back finds the same program.
    #[test]
    fn the_rows_carry_the_icon_of_their_entry() {
        let apps = apps(
            [
                Entry {
                    icon: "accessories-text-editor".into(),
                    ..entry("Pad", "pad")
                },
                entry("Screenshot", "shot"),
            ],
            "foot",
            on_path,
        );
        let (dir, mut fuzzel) = stand_in("chora-launcher-icons", "echo Pad");
        let rows = dir.join("rows");
        let script = std::fs::read_to_string(&fuzzel[1]).unwrap();
        std::fs::write(
            &fuzzel[1],
            script.replace(
                "cat >/dev/null",
                &format!("cat >'{}'", rows.to_string_lossy()),
            ),
        )
        .unwrap();
        fuzzel.truncate(2);
        let got = choose_reporting(&fuzzel, &apps, &mut |_| {});
        assert_eq!(got.as_deref(), Some("pad"));
        assert_eq!(
            std::fs::read_to_string(&rows).unwrap(),
            "Pad\0icon\x1faccessories-text-editor\nScreenshot"
        );
    }

    fn reports(fuzzel: &[String]) -> (Option<String>, Vec<String>) {
        let mut said = Vec::new();
        let got = choose_reporting(fuzzel, &list(), &mut |line| said.push(line.to_owned()));
        (got, said)
    }

    #[test]
    fn a_launcher_that_fails_says_why_once() {
        let (_dir, fuzzel) = stand_in(
            "chora-launcher-fail",
            "echo ' err: config.c:17: [colors].prompt: invalid key' >&2; echo ' err: second' >&2; exit 1",
        );
        let (got, said) = reports(&fuzzel);
        assert_eq!(got, None);
        assert_eq!(said, ["Run could not start: [colors].prompt: invalid key"]);
    }

    #[test]
    fn an_err_line_after_a_warning_is_the_reason() {
        let (_dir, fuzzel) = stand_in(
            "chora-launcher-warn-err",
            "echo 'warn: something' >&2; echo ' err: bad ini' >&2; exit 1",
        );
        let (got, said) = reports(&fuzzel);
        assert_eq!(got, None);
        assert_eq!(said, ["Run could not start: bad ini"]);
    }

    #[test]
    fn a_warning_then_escape_stays_silent() {
        let (_dir, fuzzel) = stand_in(
            "chora-launcher-warn-escape",
            "echo 'warning: [colors].prompt ignored' >&2; exit 1",
        );
        assert_eq!(reports(&fuzzel), (None, vec![]));
    }

    #[test]
    fn a_launcher_that_cannot_spawn_says_so() {
        let (got, said) = reports(&["/chora-no-such-dir/fuzzel".to_owned()]);
        assert_eq!(got, None);
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].starts_with("Run could not start: "), "{said:?}");
    }

    /// A fuzzel that rejects the config file of the look (1.9.2 has no
    /// `prompt`) exits at once with an err line; the launcher then asks
    /// again without the config instead of showing nothing (rtr1-3).
    #[test]
    fn a_config_that_fuzzel_rejects_is_left_out() {
        let (_dir, mut fuzzel) = stand_in(
            "chora-launcher-config",
            "case \" $* \" in *\" --config \"*) echo ' err: config.c:866: x.ini:5: [colors].prompt: not a valid option: prompt' >&2; exit 1;; esac; echo Screenshot",
        );
        fuzzel.extend(["--config".to_owned(), "/run/chora-mc/fuzzel.ini".to_owned()]);
        assert_eq!(reports(&fuzzel), (Some("shot".to_owned()), vec![]));
    }

    /// Without a config the failure is the failure: it is said once.
    #[test]
    fn a_failure_without_a_config_is_still_said() {
        let (_dir, mut fuzzel) = stand_in(
            "chora-launcher-no-config",
            "echo ' err: no display' >&2; exit 1",
        );
        fuzzel.extend(["--config".to_owned(), "/run/chora-mc/fuzzel.ini".to_owned()]);
        let (got, said) = reports(&fuzzel);
        assert_eq!(got, None);
        assert_eq!(said, ["Run could not start: no display"]);
    }

    #[test]
    fn a_cancel_stays_silent() {
        let (_dir, fuzzel) = stand_in("chora-launcher-cancel", "exit 1");
        assert_eq!(reports(&fuzzel), (None, vec![]));
    }

    #[test]
    fn a_choice_runs_and_says_nothing() {
        let (_dir, fuzzel) = stand_in("chora-launcher-ok", "echo Screenshot");
        assert_eq!(reports(&fuzzel), (Some("shot".to_owned()), vec![]));
    }
}
