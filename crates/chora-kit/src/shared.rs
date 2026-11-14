//! The files that every look shares, in the folder `shared` beside the
//! flavors: `power.sh` (the power menu, lock, log out, restart and shut
//! down), `screenshot.sh`, and `applications/` with the launcher entries
//! Chat with a helper, Screenshot and Screenshot of an area. A look runs
//! the scripts from here and keeps no copy.

use std::path::{Path, PathBuf};

/// The folder in an installed system.
pub const INSTALLED: &str = "/usr/share/chora/flavors/shared";

/// Where the shared files are, first the one that wins: `CHORA_SHARED_DIR`,
/// the folder `shared` beside `CHORA_FLAVOR_DIR`, the installed path, and
/// the checkout that the running binary comes from, so that a session
/// without the variables finds them too.
#[must_use]
pub fn dirs() -> Vec<PathBuf> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let exe = std::env::current_exe().ok();
    dirs_of(
        var("CHORA_SHARED_DIR"),
        var("CHORA_FLAVOR_DIR").as_deref(),
        exe.as_deref(),
    )
}

fn dirs_of(shared: Option<PathBuf>, flavor: Option<&Path>, exe: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = shared.into_iter().collect();
    dirs.extend(
        flavor
            .and_then(Path::parent)
            .map(|flavors| flavors.join("shared")),
    );
    dirs.push(INSTALLED.into());
    dirs.extend(
        exe.into_iter()
            .flat_map(|e| e.ancestors().skip(1))
            .map(|a| a.join("flavors").join("shared"))
            .find(|d| d.is_dir()),
    );
    dirs
}

fn script_in(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let file = format!("{name}.sh");
    dirs.iter().map(|d| d.join(&file)).find(|f| f.is_file())
}

/// The shell command that runs a shared script with its arguments, for
/// [`crate::run`]: `sh '/…/shared/power.sh' lock`. None when no folder
/// has the script.
#[must_use]
pub fn command(name: &str, args: &[&str]) -> Option<String> {
    let script = script_in(name, &dirs())?;
    let mut line = format!("sh {}", quote(&script.to_string_lossy()));
    for arg in args {
        line.push(' ');
        line.push_str(&quote(arg));
    }
    Some(line)
}

fn quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// The shell command that runs a launcher with the shared entries in its
/// list, for [`crate::launch`]: `fuzzel`, the launcher of every look,
/// gets `XDG_DATA_DIRS` with the shared folder in front, as one quoted
/// word. Another command, or one without a shared folder, stays as it
/// is.
#[must_use]
pub fn launcher(command: &str) -> String {
    let current = std::env::var("XDG_DATA_DIRS").ok();
    launcher_with(
        command,
        data_dirs_in(&dirs(), current.as_deref()).as_deref(),
    )
}

fn launcher_with(command: &str, data: Option<&str>) -> String {
    let program = command.split_whitespace().next().unwrap_or_default();
    match data {
        Some(data) if program.rsplit('/').next() == Some("fuzzel") => {
            format!("XDG_DATA_DIRS={} {command}", quote(data))
        }
        _ => command.to_owned(),
    }
}

/// The value of `XDG_DATA_DIRS` that lets a launcher list the shared
/// entries: the first folder with `applications` in front of what
/// `current` says, or of the default of the specification. None when no
/// folder has entries.
fn data_dirs_in(dirs: &[PathBuf], current: Option<&str>) -> Option<String> {
    let shared = dirs.iter().find(|d| d.join("applications").is_dir())?;
    let rest = current
        .filter(|c| !c.is_empty())
        .unwrap_or("/usr/local/share:/usr/share");
    Some(format!("{}:{rest}", shared.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkout() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/shared")
    }

    #[test]
    fn the_variable_wins_then_the_folder_beside_the_flavor() {
        let dirs = dirs_of(
            Some("/own".into()),
            Some(Path::new("/usr/share/chora/flavors/xp")),
            None,
        );
        assert_eq!(
            dirs,
            [
                PathBuf::from("/own"),
                PathBuf::from("/usr/share/chora/flavors/shared"),
                PathBuf::from(INSTALLED),
            ]
        );
    }

    #[test]
    fn the_checkout_of_the_binary_has_the_shared_scripts() {
        let dirs = dirs();
        for name in ["power", "screenshot"] {
            assert!(script_in(name, &dirs).is_some(), "{name}: {dirs:?}");
        }
        assert!(script_in("nothing", &dirs).is_none());
    }

    #[test]
    fn the_shared_folder_holds_one_entry_per_shared_launcher_row() {
        let apps = checkout().join("applications");
        for (file, name) in [
            ("chora-chat.desktop", "Name=Chat with a helper"),
            ("chora-screenshot.desktop", "Name=Screenshot"),
            (
                "chora-screenshot-area.desktop",
                "Name=Screenshot of an area",
            ),
        ] {
            let text = std::fs::read_to_string(apps.join(file)).expect(file);
            assert!(text.lines().any(|l| l == name), "{file}: {text}");
        }
    }

    #[test]
    fn the_launcher_lists_the_shared_entries_first() {
        let dirs = [PathBuf::from("/none"), checkout()];
        let want = format!("{}:/usr/share", checkout().display());
        assert_eq!(data_dirs_in(&dirs, Some("/usr/share")), Some(want));
        assert_eq!(data_dirs_in(&dirs[..1], None), None);
    }

    #[test]
    fn only_fuzzel_gets_the_shared_entries_as_one_quoted_word() {
        assert_eq!(
            launcher_with("fuzzel --dmenu", Some("/s:/usr/share")),
            "XDG_DATA_DIRS='/s:/usr/share' fuzzel --dmenu"
        );
        assert_eq!(
            launcher_with("/usr/bin/fuzzel", Some("/it's here:/usr/share")),
            "XDG_DATA_DIRS='/it'\\''s here:/usr/share' /usr/bin/fuzzel"
        );
        assert_eq!(launcher_with("fuzzel", None), "fuzzel");
        assert_eq!(launcher_with("rofi", Some("/s")), "rofi");
        assert_eq!(launcher_with("foot fuzzel", Some("/s")), "foot fuzzel");
    }

    #[test]
    fn a_quote_in_a_path_stays_one_word() {
        assert_eq!(quote("a'b c"), "'a'\\''b c'");
    }
}
