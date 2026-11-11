//! The key list of the margin column: the keys that the bindings hold,
//! with the words for what they do. Six rows show at the foot of the
//! column; the key Super+F1 shows all of them.

use crate::config::{Action, Config};

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

/// The rows that always show, as key and words.
pub fn short(config: &Config) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    let find = |want: &dyn Fn(&Action) -> bool| {
        config
            .bindings
            .iter()
            .find(|(_, a)| want(a))
            .map(|(k, _)| k.as_str())
    };
    if let Some(key) = find(&|a| *a == Action::Keys) {
        rows.push((human(key), "All keys".to_owned()));
    }
    if let (Some(next), Some(previous)) = (
        find(&|a| *a == Action::NextScene),
        find(&|a| *a == Action::PreviousScene),
    ) {
        rows.push((pair(previous, next), "Turn the page".to_owned()));
    }
    if let Some(key) = find(&|a| matches!(a, Action::Run(_))) {
        rows.push((human(key), "Start a program".to_owned()));
    }
    if let Some(key) = find(&|a| matches!(a, Action::Scene(s) if s == "agenda")) {
        rows.push((human(key), "Show the agenda".to_owned()));
    }
    if let Some(key) = find(&|a| *a == Action::Toggle(crate::region::Region::Right)) {
        rows.push((human(key), "Hide this column".to_owned()));
    }
    rows.push(("Esc".to_owned(), "Back one step".to_owned()));
    rows
}

/// Every binding as key and words, then the keys of other plugins that
/// the list names.
pub fn all(config: &Config) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = (config.bindings.iter())
        .filter_map(|(key, action)| Some((human(key), words(action)?)))
        .collect();
    rows.push(("Esc".to_owned(), "Back one step".to_owned()));
    rows
}

/// What an action does, in words; none for a drag, which the pointer does.
fn words(action: &Action) -> Option<String> {
    Some(match action {
        Action::NextScene => "Next page".to_owned(),
        Action::PreviousScene => "Previous page".to_owned(),
        Action::Scene(name) => format!("Show {}", crate::title_of(name).to_lowercase()),
        Action::Run(command) if command.starts_with("fuzzel") => "Start a program".to_owned(),
        Action::Run(command) => format!("Run {}", command.split_whitespace().next()?),
        Action::Give(to) => format!("Give the window to {to}"),
        Action::Drag => return None,
        Action::Toggle(region) => format!("Show or hide the {}", region.name()),
        Action::Solo { content, .. } => format!("Show the {content}"),
        Action::Keys => "All keys".to_owned(),
    })
}

/// A key as people read it: arrows for the arrow keys, a capital Space.
fn human(key: &str) -> String {
    key.split('+')
        .map(|part| match part {
            "Left" => "←".to_owned(),
            "Right" => "→".to_owned(),
            "space" => "Space".to_owned(),
            other => other.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// Two keys that differ in their last part, as `Super+← →`.
fn pair(first: &str, second: &str) -> String {
    let (a, b) = (human(first), human(second));
    match (a.rsplit_once('+'), b.rsplit_once('+')) {
        (Some((head, x)), Some((other, y))) if head == other => format!("{head}+{x} {y}"),
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
        assert_eq!(rows[0], ("Super+F1".into(), "All keys".into()));
        assert_eq!(rows[1].0, "Super+← →");
        assert_eq!(rows[2], ("Super+Space".into(), "Start a program".into()));
    }

    #[test]
    fn the_full_list_has_a_row_for_each_binding_but_the_drag() {
        let config = config();
        let rows = all(&config);
        assert_eq!(rows.len(), config.bindings.len(), "{rows:?}");
    }
}
