//! The key list of the margin column: the keys that the bindings hold,
//! with the words for what they do. Six rows show at the foot of the
//! column; the key Super+F1 shows all of them.

use chora_kit::actions::{Action as Bound, System};
use chora_kit::keysheet;
use chora_kit::words::Language;

use crate::config::{Action, Config, Look};

/// The words of an empty list of notifications.
pub fn nothing_new(language: chora_kit::words::Language) -> &'static str {
    match language {
        chora_kit::words::Language::En => "Nothing new.",
        chora_kit::words::Language::De => "Nichts Neues.",
    }
}

/// The keys of a page, which has no program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageKeys {
    /// The page takes the keyboard: Right and Left alone turn it, Home
    /// goes to the first page, Up and Down scroll it by a line, Page Up
    /// and Page Down by a screen, and Escape goes back.
    Plain,
    /// Only the bindings: beside other window managers (`--output`) the
    /// node with the keyboard would draw their new windows to mc (D55).
    Bound,
}

/// How much of the key list shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum List {
    /// Six rows.
    #[default]
    Six,
    /// Every binding.
    All,
}

impl List {
    /// The other one.
    pub fn flip(self) -> Self {
        match self {
            Self::Six => Self::All,
            Self::All => Self::Six,
        }
    }
}

/// A row of the key list: the key as people read it, its words, and the
/// binding that a click on the row acts like.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub key: String,
    pub words: String,
    pub binding: Option<String>,
}

impl Row {
    fn new(key: &str, words: &str, binding: Option<&str>) -> Self {
        Self {
            key: key.replace("+\u{2009}", "+"),
            words: words.to_owned(),
            binding: binding.map(str::to_owned),
        }
    }
}

/// The keys of the page row: on a page the plain arrows turn it, so the
/// row names them without their modifier; a program in a scene takes the
/// plain arrows itself, and the row names the whole binding.
fn page_keys(previous: &str, next: &str, plain: bool) -> String {
    let both = pair(previous, next);
    match both.rsplit_once('+') {
        Some((_, arrows)) if plain => arrows.trim_start_matches(THIN).to_owned(),
        _ => both,
    }
}

/// The rows of the bindings of the file, with the words of kit and the
/// words that mc gives to the actions of its own.
fn sheet(config: &Config) -> Vec<keysheet::Row<Look>> {
    let bindings = (config.common.bindings.iter()).map(|(key, action)| (key.clone(), action));
    keysheet::rows(bindings, Language::En, |_, action| {
        let Bound::Look(own) = action else {
            return None;
        };
        Some(match own {
            Look::NextScene => "Next page".to_owned(),
            Look::PreviousScene => "Previous page".to_owned(),
            Look::Back => "Back one step".to_owned(),
            Look::Scene(name) => format!("Show {}", crate::title_of(name).to_lowercase()),
            Look::Toggle(crate::region::Region::Right) => "Hide this column".to_owned(),
            Look::Toggle(region) => format!("Show or hide the {}", region.name()),
            Look::Solo { content, .. } => format!("Show the {content}"),
            Look::Swallow => return None,
        })
    })
}

/// The row of the page keys: the keys of both turns as one.
fn page_row(config: &Config, next: &keysheet::Row<Look>, plain: bool) -> Option<Row> {
    let previous = (config.common.bindings.iter())
        .find(|(_, a)| **a == Action::Look(Look::PreviousScene))
        .map(|(k, _)| k.as_str())?;
    let keys = page_keys(previous, &next.binding, plain);
    Some(Row::new(&keys, "Turn the page", Some(&next.binding)))
}

/// A row of kit as a row of the column: its first key, as a click on the
/// row acts like it.
fn row_of(row: &keysheet::Row<Look>) -> Row {
    let key = row.chords.first().cloned().unwrap_or_default();
    Row::new(&key, &row.words, Some(&row.binding))
}

/// The rows that always show; `plain` on a page, where the arrows alone
/// turn it.
pub fn short(config: &Config, plain: bool) -> Vec<Row> {
    let sheet = sheet(config);
    let find = |want: &dyn Fn(&Bound<Look>) -> bool| sheet.iter().find(|row| want(&row.action));
    let mut rows = Vec::new();
    rows.extend(find(&|a| *a == Action::System(System::Keys)).map(row_of));
    rows.extend(
        find(&|a| *a == Action::Look(Look::NextScene)).and_then(|n| page_row(config, n, plain)),
    );
    rows.extend(find(&|a| matches!(a, Action::System(System::Run(_)))).map(row_of));
    rows.extend(find(&|a| *a == Action::System(System::Chat)).map(row_of));
    rows.extend(
        find(&|a| *a == Action::Look(Look::Toggle(crate::region::Region::Right))).map(row_of),
    );
    rows.push(Row::new("Esc", "Back one step", None));
    rows
}

/// Every binding as key and words, in the order of the tasks: the keys,
/// the page, the scenes, programs, the chat, the column, the header and
/// the footer, the rest, and Esc last. The two keys of the page are one
/// row, as in the short list, and only the first key of the key list has
/// a row: the others are aliases. The give keys, which no file binds,
/// are one row: `Super+Alt+1-4`.
pub fn all(config: &Config, plain: bool) -> Vec<Row> {
    let scene_at = |name: &str| config.order.iter().position(|n| n == name);
    let mut found: Vec<(u8, usize, Row)> = Vec::new();
    for (place, row) in sheet(config).into_iter().enumerate() {
        let (rank, at) = match &row.action {
            Action::System(System::Keys) => (0, 0),
            Action::Look(Look::NextScene) => (1, 0),
            Action::Look(Look::Scene(name)) => (2, scene_at(name).unwrap_or(usize::MAX)),
            Action::System(System::Run(_)) => (3, 0),
            Action::System(System::Chat) => (4, 0),
            Action::Look(Look::Toggle(crate::region::Region::Right)) => (5, 0),
            Action::Look(Look::Toggle(region)) => {
                (6, usize::from(*region != crate::region::Region::Header))
            }
            Action::Look(Look::Solo { .. }) => (7, 0),
            Action::System(System::Give(_)) => (8, 0),
            Action::System(System::Lock) => (9, 0),
            Action::System(System::Power) => (9, 1),
            Action::System(System::Screenshot) => (9, 2),
            Action::System(System::Volume(_)) => (9, 3 + place),
            Action::Look(Look::PreviousScene | Look::Back | Look::Swallow)
            | Action::System(
                System::Drag
                | System::Terminal
                | System::Launcher
                | System::Close
                | System::Desk(_)
                | System::Cube(_),
            ) => continue,
        };
        let shown = match &row.action {
            Action::Look(Look::NextScene) => page_row(config, &row, plain),
            _ => Some(row_of(&row)),
        };
        found.extend(shown.map(|shown| (rank, at, shown)));
        if plain && matches!(row.action, Action::Look(Look::NextScene)) {
            let first = Action::Look(Look::Scene(config.default_scene.clone()));
            let binding = sheet(config).into_iter().find(|r| r.action == first);
            let binding = binding.map(|r| r.binding);
            let home = Row::new("Home", "First page", binding.as_deref());
            found.push((rank, 1, home));
            let scroll = Row::new("\u{2191}\u{2009}\u{2193}", "Scroll the page", None);
            found.push((rank, 2, scroll));
        }
    }
    let (give, words) = keysheet::give_row(Language::En);
    found.push((8, usize::MAX, Row::new(&give, &words, None)));
    found.sort_by_key(|(rank, at, _)| (*rank, *at));
    let mut rows: Vec<Row> = found.into_iter().map(|(_, _, row)| row).collect();
    rows.push(Row::new("Esc", "Back one step", None));
    rows
}

/// A thin space.
const THIN: char = '\u{2009}';

/// Two keys that differ in their last part, as `Super+←→` with a thin space
/// between the arrows too, so that the pair reads as one key.
fn pair(first: &str, second: &str) -> String {
    let (a, b) = (keysheet::chord(first), keysheet::chord(second));
    match (a.rsplit_once('+'), b.rsplit_once('+')) {
        (Some((head, x)), Some((other, y))) if head == other => {
            let arrows = [x, y].iter().all(|key| key.contains(['←', '→']));
            let gap = if arrows { THIN } else { ' ' };
            format!("{head}+{x}{gap}{}", y.trim_start_matches(THIN))
        }
        _ => format!("{a} {b}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DEFAULT};

    fn config() -> Config {
        chora_kit::settings::parse(DEFAULT).expect("the default settings parse")
    }

    #[test]
    fn six_rows_name_the_keys_of_the_bindings() {
        let rows = short(&config(), false);
        assert_eq!(rows.len(), 6, "{rows:?}");
        assert_eq!(rows[0].key, "Super+F1");
        assert_eq!(rows[0].words, "All keys");
        assert_eq!(rows[1].key, "Super+←\u{2009}→");
        assert_eq!(rows[1].binding.as_deref(), Some("Super+Right"));
        assert_eq!(rows[2].key, "Super+Space");
        assert_eq!(rows[5].binding, None);
    }

    #[test]
    fn the_full_list_has_a_row_for_each_task_but_the_drag_the_aliases_and_one_for_the_page_and_the_give_keys()
     {
        let config = config();
        let rows = all(&config, false);
        assert_eq!(rows.len(), config.common.bindings.len() - 3, "{rows:?}");
    }

    #[test]
    fn the_full_list_goes_by_task_in_the_words_of_the_short_list() {
        let words: Vec<String> = all(&config(), true).into_iter().map(|r| r.words).collect();
        let want = [
            "All keys",
            "Turn the page",
            "First page",
            "Scroll the page",
            "Show agenda",
            "Show terminal",
            "Show overview",
            "Start a program or run a command",
            "Chat with a helper",
            "Hide this column",
            "Show or hide the header",
            "Show or hide the footer",
            "Show the overlay",
            "Give window to a look",
            "Lock the screen",
            "Power menu",
            "Take a screenshot",
            "Volume up",
            "Volume down",
            "Mute",
            "Back one step",
        ];
        assert_eq!(words, want);
    }

    #[test]
    fn a_page_names_the_plain_arrows_and_keeps_the_binding_of_the_click() {
        for rows in [short(&config(), true), all(&config(), true)] {
            assert_eq!(rows[1].key, "←\u{2009}→", "{rows:?}");
            assert_eq!(rows[1].words, "Turn the page");
            assert_eq!(rows[1].binding.as_deref(), Some("Super+Right"));
        }
        let rows = all(&config(), true);
        assert_eq!(rows[2].key, "Home");
        assert_eq!(rows[2].words, "First page");
        assert_eq!(rows[2].binding.as_deref(), Some("Super+Home"));
    }

    #[test]
    fn a_page_names_the_scroll_keys_beside_home() {
        let rows = all(&config(), true);
        assert_eq!(rows[3].key, "↑\u{2009}↓", "{rows:?}");
        assert_eq!(rows[3].words, "Scroll the page");
        assert_eq!(rows[3].binding, None);
        let rows = all(&config(), false);
        assert!(rows.iter().all(|r| r.words != "Scroll the page"));
    }
}
