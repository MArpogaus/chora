//! The Looks door of the bar: `chora-tiling --launch looks` lists the other
//! looks that run as the rows that move the focused window there, in the
//! words of the key list and the Apps rows, or says that none runs.

use chora_kit::{
    give,
    words::{Language, Word},
};
use serde_json::Value;

/// The other looks that a window can go to, by the name of their client.
const OTHERS: [&str; 3] = ["xp", "mc", "construct"];

/// The other looks among the clients of `clients.list`, in the order of
/// [`OTHERS`].
pub fn running(clients: &Value) -> Vec<&'static str> {
    let names: Vec<&str> = (clients.as_array().into_iter().flatten())
        .filter_map(|c| c["name"].as_str())
        .collect();
    OTHERS
        .into_iter()
        .filter(|look| names.contains(look))
        .collect()
}

/// The rows of the menu: one that moves the window to each running look,
/// or the one row that says that no other look runs, or that no window has
/// the focus (`window` false), where a row that moves one would fail.
pub fn rows(running: &[&str], window: bool, language: Language) -> Vec<String> {
    if running.is_empty() {
        return vec![Word::NoOtherLook.text(language).to_owned()];
    }
    if !window {
        return vec![Word::NoWindowToMove.text(language).to_owned()];
    }
    (running.iter())
        .map(|look| give::move_words(look, language))
        .collect()
}

/// The look that the row `line` moves the window to; none for the rows
/// that say that no look runs or no window has the focus, and for text
/// that is no row.
pub fn chosen<'a>(running: &[&'a str], language: Language, line: &str) -> Option<&'a str> {
    let line = line.trim();
    (running.iter())
        .copied()
        .find(|look| give::move_words(look, language) == line)
}

/// The word of the bar: "Looks" when no other look runs, else "Looks: " and
/// the names of the running ones, so that the word says where it leads.
pub fn word(running: &[&str]) -> String {
    if running.is_empty() {
        return "Looks".to_owned();
    }
    format!("Looks: {}", running.join(", "))
}

/// The prompt of the menu.
pub fn prompt(language: Language) -> &'static str {
    Word::MoveThisWindowTo.text(language)
}

/// The title and the text of the notice for a click on the Looks door
/// when the list of the running clients cannot be read.
pub fn unreachable_notice(language: Language) -> (&'static str, &'static str) {
    match language {
        Language::De => (
            "Andere Desktops nicht verfügbar",
            "Die Liste der anderen Desktops ließ sich nicht lesen.",
        ),
        Language::En => (
            "Other desktops not available",
            "The list of the other desktops could not be read.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_door_that_cannot_list_the_looks_says_so_in_both_languages() {
        let english = unreachable_notice(Language::En);
        let german = unreachable_notice(Language::De);
        assert_eq!(english.0, "Other desktops not available");
        assert_eq!(german.0, "Andere Desktops nicht verfügbar");
        assert!(english.1.contains("could not be read"), "{english:?}");
        assert!(german.1.contains("nicht lesen"), "{german:?}");
    }

    #[test]
    fn the_running_looks_are_the_other_clients_in_a_fixed_order() {
        let clients = json!([{"name": "construct"}, {"name": "tiling"}, {"name": "xp"}]);
        assert_eq!(running(&clients), ["xp", "construct"]);
        assert_eq!(running(&json!([{"name": "tiling"}])), Vec::<&str>::new());
        assert_eq!(running(&Value::Null), Vec::<&str>::new());
    }

    #[test]
    fn the_bar_word_names_the_looks_it_leads_to_ti_55() {
        assert_eq!(word(&[]), "Looks");
        assert_eq!(word(&["xp"]), "Looks: xp");
        assert_eq!(word(&["xp", "mc"]), "Looks: xp, mc");
    }

    #[test]
    fn a_row_moves_the_window_in_the_words_of_the_key_list() {
        let rows_en = rows(&["xp", "mc", "construct"], true, Language::En);
        assert_eq!(
            rows_en,
            [
                "Move window to the xp desktop",
                "Move window to the media center desktop",
                "Move window to the construct desktop"
            ]
        );
        assert_eq!(
            chosen(
                &["xp", "mc"],
                Language::En,
                "Move window to the xp desktop\n"
            ),
            Some("xp")
        );
        assert_eq!(
            chosen(
                &["xp"],
                Language::En,
                "Move window to the media center desktop"
            ),
            None
        );
        assert_eq!(
            rows(&["xp"], true, Language::De),
            ["Fenster auf den xp-Desktop verschieben"]
        );
    }

    #[test]
    fn no_other_look_is_one_row_that_chooses_nothing() {
        assert_eq!(rows(&[], true, Language::En), ["No other look is running"]);
        assert_eq!(rows(&[], true, Language::De), ["Kein anderer Look läuft"]);
        assert_eq!(chosen(&[], Language::En, "No other look is running"), None);
        assert_eq!(prompt(Language::En), "Move this window to ");
        assert_eq!(prompt(Language::De), "Dieses Fenster verschieben nach ");
    }
}
