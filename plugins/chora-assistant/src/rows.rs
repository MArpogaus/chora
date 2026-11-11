//! The rows of the helpers: one row for each helper that the voice agent
//! names, with its state word. The words are the host's; a folder gives
//! the place, the number of rows and the colours.

use std::collections::BTreeMap;

use chora_kit::words::{self, Language};

use crate::folders::good_name;

/// What stands between the parts of a row.
pub const SEPARATOR: &str = " · ";
/// The most characters of the task of a row.
const MAX_TASK: usize = 60;
/// The name of the "Chat ..." row after [`ROW`], which no helper takes.
const CHAT: &str = "chat";

/// The start of the id of the row of a helper; the name follows.
pub const ROW: &str = "hud-row-";

/// What the voice agent last said of a helper.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Helper {
    /// The state, as the context names it.
    pub state: &'static str,
    /// What the helper does.
    pub task: String,
    /// The step of the task and the number of steps.
    pub steps: Option<(u64, u64)>,
}

/// The parts of the row of a helper: its name, its state word and its
/// task with the step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The id of the node.
    pub id: String,
    /// The name of the helper.
    pub name: String,
    /// The state word.
    pub state: String,
    /// The task, with `(step/steps)` when the helper counts them.
    pub task: String,
}

impl Row {
    /// The words of the row: `Atlas · working · Write file (2/2)`.
    #[must_use]
    pub fn text(&self) -> String {
        [&self.name, &self.state, &self.task]
            .into_iter()
            .filter(|part| !part.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(SEPARATOR)
    }
}

/// The rows of the helpers by name, at most `most`. A name that is no
/// plain name, or that is the name of the "Chat ..." row, has no row.
#[must_use]
pub fn rows(language: Language, helpers: &BTreeMap<String, Helper>, most: usize) -> Vec<Row> {
    helpers
        .iter()
        .filter(|(name, _)| good_name(name) && name.as_str() != CHAT)
        .take(most)
        .map(|(name, helper)| Row {
            id: format!("{ROW}{name}"),
            name: name.clone(),
            state: words::state(helper.state).map_or_else(
                || helper.state.to_owned(),
                |word| word.text(language).to_lowercase(),
            ),
            task: match (task_of(helper), helper.steps) {
                (task, Some((step, steps))) if !task.is_empty() => {
                    format!("{task} ({step}/{steps})")
                }
                (task, _) => task,
            },
        })
        .collect()
}

/// The task of a helper, cut to [`MAX_TASK`] characters.
fn task_of(helper: &Helper) -> String {
    helper.task.chars().take(MAX_TASK).collect()
}

/// The helper of the row that a press on `node` names.
#[must_use]
pub fn helper_of(node: &str) -> Option<&str> {
    node.strip_prefix(ROW).filter(|name| *name != CHAT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn helper(state: &'static str, task: &str, steps: Option<(u64, u64)>) -> Helper {
        Helper {
            state,
            task: task.into(),
            steps,
        }
    }

    #[test]
    fn one_row_per_helper_with_its_state_word() {
        let helpers: BTreeMap<String, Helper> = [
            (
                "Atlas".to_owned(),
                helper("working", "Write file", Some((2, 2))),
            ),
            ("Bert".to_owned(), helper("idle", "", None)),
        ]
        .into();
        let shown = rows(Language::En, &helpers, 4);
        let texts: Vec<(String, String)> = shown.iter().map(|r| (r.id.clone(), r.text())).collect();
        assert_eq!(
            texts,
            [
                (
                    "hud-row-Atlas".to_owned(),
                    "Atlas · working · Write file (2/2)".to_owned()
                ),
                ("hud-row-Bert".to_owned(), "Bert · idle".to_owned()),
            ]
        );
        assert_eq!(shown[0].state, "working");
        let german = rows(Language::De, &helpers, 4);
        assert_eq!(german[0].text(), "Atlas · arbeitet · Write file (2/2)");
        assert_eq!(rows(Language::En, &helpers, 1).len(), 1);
    }

    #[test]
    fn a_name_that_is_no_plain_name_has_no_row() {
        let helpers: BTreeMap<String, Helper> = ["chat", "a b", "../x", ""]
            .into_iter()
            .map(|name| (name.to_owned(), helper("working", "", None)))
            .collect();
        assert_eq!(rows(Language::En, &helpers, 4), Vec::new());
    }

    #[test]
    fn a_press_on_a_row_names_its_helper() {
        assert_eq!(helper_of("hud-row-Atlas"), Some("Atlas"));
        assert_eq!(helper_of("hud-row-chat"), None);
        assert_eq!(helper_of("hud-card"), None);
    }
}
