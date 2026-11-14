//! The key list of the margin column: the keys that the bindings hold,
//! with the words for what they do. Six rows show at the foot of the
//! column; the key Super+F1 shows all of them.

use chora_kit::volume::Change;

use crate::config::{Action, Config};

/// The words of an empty list of notifications.
pub fn nothing_new(language: chora_kit::words::Language) -> &'static str {
    match language {
        chora_kit::words::Language::En => "Nothing new.",
        chora_kit::words::Language::De => "Nichts Neues.",
    }
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
    fn new(key: String, words: &str, binding: Option<&str>) -> Self {
        Self {
            key,
            words: words.to_owned(),
            binding: binding.map(str::to_owned),
        }
    }
}

/// The rows that always show.
pub fn short(config: &Config) -> Vec<Row> {
    let mut rows = Vec::new();
    let find = |want: &dyn Fn(&Action) -> bool| {
        config
            .bindings
            .iter()
            .find(|(_, a)| want(a))
            .map(|(k, _)| k.as_str())
    };
    if let Some(key) = find(&|a| *a == Action::Keys) {
        rows.push(Row::new(human(key), "All keys", Some(key)));
    }
    if let (Some(next), Some(previous)) = (
        find(&|a| *a == Action::NextScene),
        find(&|a| *a == Action::PreviousScene),
    ) {
        rows.push(Row::new(pair(previous, next), "Turn the page", Some(next)));
    }
    if let Some(key) = find(&|a| matches!(a, Action::Run(_))) {
        rows.push(Row::new(human(key), "Start a program", Some(key)));
    }
    if let Some(key) = find(&|a| *a == Action::Chat) {
        rows.push(Row::new(human(key), "Chat with a helper", Some(key)));
    }
    if let Some(key) = find(&|a| *a == Action::Toggle(crate::region::Region::Right)) {
        rows.push(Row::new(human(key), "Hide this column", Some(key)));
    }
    rows.push(Row::new("Esc".to_owned(), "Back one step", None));
    rows
}

/// Every binding as key and words, in the order of the tasks: the keys,
/// the page, the scenes, programs, the chat, the column, the header and
/// the footer, the rest, and Esc last. The two keys of the page are one
/// row, as in the short list, and only the first key of the key list has
/// a row: the others are aliases.
pub fn all(config: &Config) -> Vec<Row> {
    let scene_at = |name: &str| config.order.iter().position(|n| n == name);
    let mut found: Vec<(u8, usize, Row)> = Vec::new();
    let first_keys = (config.bindings.iter())
        .find(|(_, a)| **a == Action::Keys)
        .map(|(k, _)| k);
    for (key, action) in &config.bindings {
        if *action == Action::Keys && Some(key) != first_keys {
            continue;
        }
        let (rank, at) = match action {
            Action::Keys => (0, 0),
            Action::NextScene => (1, 0),
            Action::Scene(name) => (2, scene_at(name).unwrap_or(usize::MAX)),
            Action::Run(_) => (3, 0),
            Action::Chat => (4, 0),
            Action::Toggle(crate::region::Region::Right) => (5, 0),
            Action::Toggle(region) => (6, usize::from(*region != crate::region::Region::Header)),
            Action::Solo { .. } => (7, 0),
            Action::Give(_) => (8, 0),
            Action::Lock => (9, 0),
            Action::Power => (9, 1),
            Action::Screenshot => (9, 2),
            Action::Volume(change) => (9, 3 + volume_order(*change)),
            Action::PreviousScene | Action::Drag => continue,
        };
        let row = match action {
            Action::NextScene => short_page(config, key),
            _ => words(action).map(|words| Row::new(human(key), &words, Some(key))),
        };
        found.extend(row.map(|row| (rank, at, row)));
    }
    found.sort_by_key(|(rank, at, _)| (*rank, *at));
    let mut rows: Vec<Row> = found.into_iter().map(|(_, _, row)| row).collect();
    rows.push(Row::new("Esc".to_owned(), "Back one step", None));
    rows
}

fn short_page(config: &Config, next: &str) -> Option<Row> {
    let previous = (config.bindings.iter())
        .find(|(_, a)| **a == Action::PreviousScene)
        .map(|(k, _)| k.as_str())?;
    Some(Row::new(pair(previous, next), "Turn the page", Some(next)))
}

/// What an action does, in words, the same words as the short list; none
/// for a drag, which the pointer does.
fn words(action: &Action) -> Option<String> {
    Some(match action {
        Action::NextScene => "Next page".to_owned(),
        Action::PreviousScene => "Previous page".to_owned(),
        Action::Scene(name) => format!("Show {}", crate::title_of(name).to_lowercase()),
        Action::Run(command) if command.starts_with("fuzzel") => "Start a program".to_owned(),
        Action::Run(command) => format!("Run {}", command.split_whitespace().next()?),
        Action::Give(to) => format!("Give the window to {to}"),
        Action::Drag => return None,
        Action::Toggle(crate::region::Region::Right) => "Hide this column".to_owned(),
        Action::Toggle(region) => format!("Show or hide the {}", region.name()),
        Action::Solo { content, .. } => format!("Show the {content}"),
        Action::Keys => "All keys".to_owned(),
        Action::Chat => "Chat with a helper".to_owned(),
        Action::Lock => "Lock the screen".to_owned(),
        Action::Power => "Power menu".to_owned(),
        Action::Screenshot => "Take a screenshot".to_owned(),
        Action::Volume(Change::Up) => "Volume up".to_owned(),
        Action::Volume(Change::Down) => "Volume down".to_owned(),
        Action::Volume(Change::Mute) => "Mute or unmute".to_owned(),
        Action::Volume(Change::Set(percent)) => format!("Set the volume to {percent}%"),
    })
}

fn volume_order(change: Change) -> usize {
    match change {
        Change::Up => 0,
        Change::Down => 1,
        Change::Mute => 2,
        Change::Set(_) => 3,
    }
}

/// A key as people read it: arrows for the arrow keys, a capital Space.
/// A thin space after the plus keeps an arrow off it.
pub fn human(key: &str) -> String {
    let mut out = String::new();
    for (i, part) in key.split('+').enumerate() {
        let (shown, arrow) = match part {
            "Left" => ("←", true),
            "Right" => ("→", true),
            "space" => ("Space", false),
            other => (other, false),
        };
        if i > 0 {
            out.push('+');
            if arrow {
                out.push(THIN);
            }
        }
        out.push_str(shown);
    }
    out
}

/// A thin space.
const THIN: char = '\u{2009}';

/// Two keys that differ in their last part, as `Super+←→` with a thin space
/// between the arrows too, so that the pair reads as one key.
fn pair(first: &str, second: &str) -> String {
    let (a, b) = (human(first), human(second));
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
        let rows = short(&config());
        assert_eq!(rows.len(), 6, "{rows:?}");
        assert_eq!(rows[0].key, "Super+F1");
        assert_eq!(rows[0].words, "All keys");
        assert_eq!(rows[1].key, "Super+\u{2009}←\u{2009}→");
        assert_eq!(rows[1].binding.as_deref(), Some("Super+Right"));
        assert_eq!(rows[2].key, "Super+Space");
        assert_eq!(rows[5].binding, None);
    }

    #[test]
    fn the_full_list_has_a_row_for_each_task_but_the_drag_the_alias_and_one_for_the_page() {
        let config = config();
        let rows = all(&config);
        assert_eq!(rows.len(), config.bindings.len() - 2, "{rows:?}");
    }

    #[test]
    fn the_full_list_goes_by_task_in_the_words_of_the_short_list() {
        let words: Vec<String> = all(&config()).into_iter().map(|r| r.words).collect();
        let want = [
            "All keys",
            "Turn the page",
            "Show agenda",
            "Show terminal",
            "Show overview",
            "Start a program",
            "Chat with a helper",
            "Hide this column",
            "Show or hide the header",
            "Show or hide the footer",
            "Show the overlay",
            "Lock the screen",
            "Power menu",
            "Take a screenshot",
            "Volume up",
            "Volume down",
            "Mute or unmute",
            "Back one step",
        ];
        assert_eq!(words, want);
    }

    #[test]
    fn an_arrow_keeps_a_thin_space_off_the_plus() {
        assert_eq!(human("Super+Left"), "Super+\u{2009}←");
        assert_eq!(human("Super+Return"), "Super+Return");
    }
}
