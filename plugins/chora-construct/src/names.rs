//! The names that the construct shows: the label of a pane, the display
//! name of a window manager and the notice of a refused give. The words
//! live here, as those of the start card do in `start.rs`; the word table
//! of the kit has none of them.

use chora_kit::words::Language;

/// The first word of a command line, without its path.
fn program(command: &str) -> &str {
    let first = command.split_whitespace().next().unwrap_or_default();
    first.rsplit('/').next().unwrap_or_default()
}

/// A name with its first letter in capitals.
fn capitalised(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
}

/// A program as a name: the last part of its app id, with spaces for `-`
/// and `_`, in capitals at the start.
fn app_name(app_id: &str) -> String {
    let last = app_id.rsplit('.').next().unwrap_or(app_id);
    capitalised(&last.replace(['-', '_'], " "))
}

/// The label of the pane of a window: its title, but not while the title
/// says nothing, which is when it is empty or only names the program, as
/// the app id or the first word of its command does. Then the terminal
/// that the construct started is `Terminal`, and any other program is its
/// app id as a name, never the id itself.
#[must_use]
pub fn pane_label(
    (title, app_id): (Option<&str>, Option<&str>),
    (command, terminal): (Option<&str>, &str),
    language: Language,
) -> String {
    let title = title.map_or("", str::trim);
    let app = app_id.unwrap_or_default();
    let called = command.map_or("", program);
    let says_nothing = title.is_empty()
        || title.eq_ignore_ascii_case(app)
        || (!called.is_empty() && title.eq_ignore_ascii_case(called));
    if !says_nothing {
        return title.to_owned();
    }
    if !called.is_empty() && called == program(terminal) {
        return crate::hints::terminal_word(language).to_owned();
    }
    if app.is_empty() {
        return match language {
            Language::En => "Window",
            Language::De => "Fenster",
        }
        .to_owned();
    }
    app_name(app)
}

/// The display name of a window manager.
#[must_use]
pub fn manager_name(manager: &str) -> String {
    match manager {
        "xp" => "XP".to_owned(),
        "mc" => "Media center".to_owned(),
        other => capitalised(other),
    }
}

/// The error of the core without the index of its command before it and
/// the code after it.
fn plain_error(error: &str) -> &str {
    let error = error.trim();
    let error = error.strip_prefix("command ").map_or(error, |rest| {
        let digits = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        digits.strip_prefix(": ").unwrap_or(error)
    });
    let error = error.rsplit_once(" (code ").map_or(error, |(text, _)| text);
    error.trim_end_matches('.')
}

/// What a refused give says as a reason, from the error of the core.
#[must_use]
pub fn reason(error: &str, language: Language) -> String {
    let error = plain_error(error);
    let words = |en: &str, de: &str| match language {
        Language::En => en.to_owned(),
        Language::De => de.to_owned(),
    };
    if error.is_empty() {
        return words("it said no", "es hat abgelehnt");
    }
    if error.contains("has no connection") || error.contains("holds no claim") {
        return words("it is not running", "es läuft nicht");
    }
    let mut chars = error.chars();
    chars
        .next()
        .map_or_else(String::new, |c| c.to_lowercase().chain(chars).collect())
}

/// The notice that a window manager did not take a window, with the
/// reason.
#[must_use]
pub fn refused(language: Language, manager: &str, reason: &str) -> String {
    let name = manager_name(manager);
    let reason = reason.trim_end_matches('.');
    match language {
        Language::En => format!("{name} did not take the window: {reason}."),
        Language::De => format!("{name} hat das Fenster nicht genommen: {reason}."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN: Language = Language::En;

    #[test]
    fn a_pane_is_labelled_with_its_title() {
        let label = |title| pane_label((Some(title), Some("foot")), (Some("foot"), "foot"), EN);
        assert_eq!(label("root@host: ~"), "root@host: ~");
        assert_eq!(label("  Notes  "), "Notes");
    }

    #[test]
    fn a_title_that_names_the_program_is_no_title() {
        let terminal = |title: &str, app: &str| {
            pane_label((Some(title), Some(app)), (Some("foot"), "foot"), EN)
        };
        assert_eq!(terminal("foot", "foot"), "Terminal");
        assert_eq!(terminal("", "foot"), "Terminal");
        assert_eq!(terminal("Foot", "startcell"), "Terminal");
        let german = pane_label((None, Some("foot")), (Some("foot"), "foot"), Language::De);
        assert_eq!(german, "Terminal");
    }

    #[test]
    fn a_program_that_is_no_terminal_is_its_app_id_as_a_name() {
        let label = |title, app| pane_label((title, Some(app)), (None, "foot"), EN);
        assert_eq!(label(None, "firefox"), "Firefox");
        assert_eq!(label(Some("firefox"), "firefox"), "Firefox");
        assert_eq!(label(None, "org.gnome.gedit"), "Gedit");
        assert_eq!(label(None, "my-editor"), "My editor");
        assert_eq!(pane_label((None, None), (None, "foot"), EN), "Window");
        assert_eq!(
            pane_label((None, None), (None, "foot"), Language::De),
            "Fenster"
        );
    }

    #[test]
    fn a_window_manager_has_a_display_name() {
        assert_eq!(manager_name("xp"), "XP");
        assert_eq!(manager_name("tiling"), "Tiling");
        assert_eq!(manager_name("mc"), "Media center");
        assert_eq!(manager_name("construct"), "Construct");
        assert_eq!(manager_name("other"), "Other");
    }

    #[test]
    fn a_refused_give_names_the_manager_and_the_reason() {
        assert_eq!(
            refused(EN, "tiling", "it is not running"),
            "Tiling did not take the window: it is not running."
        );
        assert_eq!(
            refused(Language::De, "xp", "es läuft nicht"),
            "XP hat das Fenster nicht genommen: es läuft nicht."
        );
        assert_eq!(reason("\"xp\" has no connection", EN), "it is not running");
        assert_eq!(
            reason("\"xp\" has no connection", Language::De),
            "es läuft nicht"
        );
        assert_eq!(
            reason("No window has the focus.", EN),
            "no window has the focus"
        );
        assert_eq!(reason("", EN), "it said no");
        let core = "command 0: \"xp\" holds no claim; only a window manager receives a window";
        assert_eq!(reason(core, EN), "it is not running");
        assert_eq!(reason(core, Language::De), "es läuft nicht");
        assert_eq!(
            reason("command 2: no window has the focus (code 1)", EN),
            "no window has the focus"
        );
    }
}
