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

use crate::desktop_entry::{self, Entry};

/// How often the list shows again after text that found nothing.
const ROUNDS: usize = 8;

/// A program of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct App {
    /// The name, the text of its row.
    pub name: String,
    /// The shell command, in the terminal if the program needs one.
    pub command: String,
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
                words,
            }
        })
        .collect();
    apps.sort_by_cached_key(|a| a.name.to_lowercase());
    apps.dedup_by(|b, a| a.name == b.name);
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

/// Shows the rows with `fuzzel --dmenu` and answers the line that it
/// printed: the row that the person chose, the text that matched no row,
/// or nothing when the person closed it or fuzzel did not run.
fn ask(fuzzel: &[String], rows: &[&str]) -> Option<String> {
    let (program, args) = fuzzel.split_first()?;
    let mut child = Command::new(program)
        .args(args)
        .arg("--dmenu")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| eprintln!("cannot start {program}: {err}"))
        .ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(rows.join("\n").as_bytes());
    }
    let out = child.wait_with_output().ok()?;
    let line = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!line.is_empty()).then_some(line)
}

/// Lets the person choose a program with `fuzzel`, the command line of
/// fuzzel with its config file, and answers the shell command to run.
/// After text that found nothing the list shows again, with the row of
/// [`nothing_found`] first.
#[must_use]
pub fn choose(fuzzel: &[String], apps: &[App]) -> Option<String> {
    let all: Vec<&str> = apps.iter().map(|a| a.name.as_str()).collect();
    let mut rows = all.clone();
    let mut notice: Option<String> = None;
    for _ in 0..ROUNDS {
        let shown: Vec<&str> = notice
            .as_deref()
            .into_iter()
            .chain(rows.iter().copied())
            .collect();
        let line = ask(fuzzel, &shown)?;
        match answer(apps, &line, notice.as_deref(), desktop_entry::on_path) {
            Answer::Closed => return None,
            Answer::Run(command) => return Some(command),
            Answer::Narrow(found) => {
                rows = found.iter().map(|a| a.name.as_str()).collect();
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

/// The data directories of the launcher: `first`, the one of a flavor,
/// then the first folder of `shared` that has `applications` (Chat with a
/// helper, Screenshot), then the XDG ones.
fn entry_dirs(first: Option<PathBuf>, shared: &[PathBuf], xdg: Vec<PathBuf>) -> Vec<PathBuf> {
    let shared = shared.iter().find(|d| d.join("applications").is_dir());
    (first.into_iter())
        .chain(shared.cloned())
        .chain(xdg)
        .collect()
}

/// The launcher: the person chooses one of the installed applications,
/// which starts as an app with `choractl run`. `first` is a data
/// directory in front of the folder that every look shares and the XDG
/// ones, for the entries of a flavor.
pub fn run(fuzzel: &[String], terminal: &str, first: Option<PathBuf>) {
    let dirs = entry_dirs(first, &crate::shared::dirs(), crate::xdg::data_dirs());
    let entries = desktop_entry::applications_in(&dirs);
    let apps = apps(entries, terminal, desktop_entry::on_path);
    if let Some(command) = choose(fuzzel, &apps) {
        crate::run(&command, None);
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
                    ..entry("Dark look", "look dark")
                },
                Entry {
                    keywords: vec!["theme".into()],
                    ..entry("Light look", "look light")
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
                "Dark look",
                "Light look",
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
        assert_eq!(names, ["Dark look", "Light look"]);
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
}
