//! The rows of a key list (KIT3): one wording per `System` action and
//! language, one spelling per key, and the spec table that the acts of
//! the four looks read too (`tests/keysheet/words.table`).

use chora_kit::actions::{Action, Step, System, Turn};
use chora_kit::keysheet::{self, Row};
use chora_kit::volume::Change;
use chora_kit::words::Language;

const TABLE: &str = include_str!("../../../tests/keysheet/words.table");

/// The lines of one kind of the table, split at tabs, without the kind.
fn lines(kind: &str) -> Vec<Vec<&'static str>> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .filter(|f| f[0] == kind)
        .map(|f| f[1..].to_vec())
        .collect()
}

/// The action of a name of the table, with `fuzzel` or `x` for an argument.
fn system(name: &str) -> System {
    match name {
        "keys" => System::Keys,
        "chat" => System::Chat,
        "lock" => System::Lock,
        "power" => System::Power,
        "screenshot" => System::Screenshot,
        "volume:up" => System::Volume(Change::Up),
        "volume:down" => System::Volume(Change::Down),
        "volume:mute" => System::Volume(Change::Mute),
        "volume:set" => System::Volume(Change::Set(30)),
        "close" => System::Close,
        "drag" => System::Drag,
        "terminal" => System::Terminal,
        "launcher" => System::Launcher,
        "run:fuzzel" => System::Run("fuzzel --dmenu".into()),
        "run" => System::Run("foot -e htop".into()),
        "give" => System::Give("xp".into()),
        "desk:next" => System::Desk(Step::Next),
        "desk:previous" => System::Desk(Step::Previous),
        "cube:left" => System::Cube(Turn::Left),
        "cube:right" => System::Cube(Turn::Right),
        other => panic!("the table names {other}"),
    }
}

/// The argument that [`system`] gave an action.
fn argument(name: &str) -> &'static str {
    match name {
        "volume:set" => "30",
        "run" => "foot",
        "give" => "xp",
        _ => "",
    }
}

#[test]
fn every_action_has_the_words_of_the_table_in_both_languages() {
    for fields in lines("words") {
        let (name, english, german) = (fields[0], fields[1], fields[2]);
        let (en, de) = (
            english.replace("{}", argument(name)),
            german.replace("{}", argument(name)),
        );
        let action = system(name);
        assert_eq!(
            keysheet::words(&action, Language::En).as_deref(),
            Some(en.as_str()),
            "{name}"
        );
        assert_eq!(
            keysheet::words(&action, Language::De).as_deref(),
            Some(de.as_str()),
            "{name}"
        );
    }
}

#[test]
fn a_key_is_spelled_one_way() {
    for fields in lines("cap") {
        let (key, cap) = (fields[0], fields[1]);
        assert_eq!(keysheet::caps(key), vec![cap.to_owned()], "{key}");
        assert_eq!(
            keysheet::caps(&format!("Super+{key}")),
            ["Super", cap],
            "Super+{key}"
        );
    }
}

#[test]
fn a_chord_joins_its_caps_and_keeps_an_arrow_off_the_plus() {
    assert_eq!(keysheet::chord("Super+Return"), "Super+Return");
    assert_eq!(keysheet::chord("Super+Left"), "Super+\u{2009}←");
    assert_eq!(keysheet::chord("Super+Alt+slash"), "Super+Alt+/");
    assert_eq!(keysheet::chord("XF86AudioRaiseVolume"), "Vol+");
}

/// rpt3-1: a chord is spelled as the key list spells it, whatever order
/// the core names its modifiers in and whatever the keysym is called.
#[test]
fn a_chord_has_one_order_of_modifiers_and_the_names_of_the_keys_on_the_keyboard() {
    assert_eq!(keysheet::chord("Alt+Super+x"), "Super+Alt+X");
    assert_eq!(
        keysheet::chord("Ctrl+Shift+Super+Next"),
        "Super+Ctrl+Shift+Page Down"
    );
    assert_eq!(
        keysheet::chord("Ctrl+Alt+Shift+Super+Prior"),
        "Super+Ctrl+Alt+Shift+Page Up"
    );
    assert_eq!(keysheet::chord("Super+bracketleft"), "Super+[");
    assert_eq!(keysheet::chord("Super+bracketright"), "Super+]");
    for (key, cap) in [
        ("apostrophe", "'"),
        ("grave", "`"),
        ("minus", "-"),
        ("equal", "="),
        ("backslash", "\\"),
        ("comma", ","),
        ("semicolon", ";"),
        ("BackSpace", "Backspace"),
    ] {
        assert_eq!(
            keysheet::chord(&format!("Super+{key}")),
            format!("Super+{cap}")
        );
    }
    assert_eq!(keysheet::chord("Super+Shift+Escape"), "Super+Shift+Esc");
    assert_eq!(keysheet::chord("super+SHIFT+Return"), "Super+Shift+Return");
}

#[derive(Clone, Debug, PartialEq)]
enum Own {
    Page(u8),
    Hide,
}

fn named(binding: &str, action: &Action<Own>) -> Option<String> {
    match action {
        Action::Look(Own::Page(n)) => Some(format!("Page {n}")),
        Action::System(System::Run(_)) if binding == "Super+Return" => Some("Terminal".into()),
        _ => None,
    }
}

fn bound(list: &[(&str, Action<Own>)]) -> Vec<Row<Own>> {
    let bindings: Vec<(String, &Action<Own>)> =
        list.iter().map(|(k, a)| ((*k).to_owned(), a)).collect();
    keysheet::rows(bindings, Language::En, named)
}

#[test]
fn bindings_of_one_wording_are_one_row_with_every_chord() {
    let rows = bound(&[
        ("Super+F1", Action::System(System::Keys)),
        ("Super+slash", Action::System(System::Keys)),
        ("Super+l", Action::System(System::Lock)),
    ]);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0].words, "All keys");
    assert_eq!(rows[0].chords, ["Super+F1", "Super+/"]);
    assert_eq!(rows[0].binding, "Super+F1");
    assert_eq!(rows[1].words, "Lock the screen");
    assert_eq!(rows[1].chords, ["Super+L"]);
}

#[test]
fn rows_of_the_system_come_first_in_the_order_of_the_actions_then_those_of_the_look() {
    let rows = bound(&[
        ("a", Action::Look(Own::Page(1))),
        ("b", Action::System(System::Power)),
        ("c", Action::System(System::Keys)),
        ("d", Action::Look(Own::Hide)),
        ("e", Action::Look(Own::Page(2))),
        ("f", Action::System(System::Volume(Change::Mute))),
    ]);
    let words: Vec<&str> = rows.iter().map(|r| r.words.as_str()).collect();
    assert_eq!(
        words,
        ["All keys", "Power menu", "Mute", "Page 1", "Page 2"]
    );
}

#[test]
fn the_give_keys_are_one_row_in_both_languages() {
    assert_eq!(
        keysheet::give_row(Language::En),
        (
            "Super+Alt+1-4".to_owned(),
            "Give window to a look".to_owned()
        )
    );
    let wording = lines("giverow");
    let (english, german) = (wording[0][0], wording[0][1]);
    assert_eq!(keysheet::give_row(Language::En).1, english);
    assert_eq!(keysheet::give_row(Language::De).1, german);
}

#[test]
fn the_give_row_and_the_give_action_have_one_wording() {
    for (language, noun) in [
        (Language::En, "a look"),
        (Language::De, "einen Fenstermanager"),
    ] {
        assert_eq!(
            keysheet::words(&System::Give(noun.into()), language),
            Some(keysheet::give_row(language).1),
            "{language:?}"
        );
    }
}

#[test]
fn the_words_of_a_look_replace_those_of_kit_for_one_binding_only() {
    let rows = bound(&[
        ("Super+Return", Action::System(System::Run("foot".into()))),
        ("Super+r", Action::System(System::Run("foot".into()))),
    ]);
    let words: Vec<&str> = rows.iter().map(|r| r.words.as_str()).collect();
    assert_eq!(words, ["Terminal", "Run foot"]);
}
