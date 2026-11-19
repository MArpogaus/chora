//! The rows of a key list (KIT3): what each of the four looks draws, made
//! once from the bindings. Words come from [`System`] (one wording per
//! action and language), a chord is spelled one way, and the actions of
//! one look are named by the look. A look draws its own costume around
//! the rows: its groups, its caps, its order.

use crate::actions::{Action, Step, System, Turn};
use crate::volume::Change;
use crate::words::Language;

/// A row of a key list: what the bindings of one wording do.
#[derive(Clone, Debug, PartialEq)]
pub struct Row<L> {
    /// The action of the first binding of the row.
    pub action: Action<L>,
    /// What it does, in words.
    pub words: String,
    /// The chords that do it, as people read them, in the order of the
    /// bindings.
    pub chords: Vec<String>,
    /// The first binding, as the file spells it: the one that a click on
    /// the row acts like.
    pub binding: String,
}

/// The thin space that keeps an arrow off the plus before it.
const THIN: char = '\u{2009}';

/// The caps of a binding, one per key: the caps of [`crate::glass`], and
/// `drag` for the left button of the pointer.
#[must_use]
pub fn caps(binding: &str) -> Vec<String> {
    binding
        .split('+')
        .filter(|key| !key.is_empty())
        .map(|key| match key {
            "BTN_LEFT" => "drag".to_owned(),
            other => crate::glass::cap(other),
        })
        .collect()
}

/// A binding as people read it: its caps joined by a plus, and a thin
/// space between the plus and an arrow, so that the arrow does not touch it.
#[must_use]
pub fn chord(binding: &str) -> String {
    let mut out = String::new();
    for (i, cap) in caps(binding).iter().enumerate() {
        if i > 0 {
            out.push('+');
            if cap.contains(['←', '→', '↑', '↓']) {
                out.push(THIN);
            }
        }
        out.push_str(cap);
    }
    out
}

/// The words of an action of the system, none for a command without a
/// program.
#[must_use]
pub fn words(system: &System, language: Language) -> Option<String> {
    let de = language == Language::De;
    let pick = |en: &str, german: &str| (if de { german } else { en }).to_owned();
    Some(match system {
        System::Run(command) if command.starts_with("fuzzel") => launch(de),
        System::Run(command) => {
            let program = command.split_whitespace().next()?;
            pick(&format!("Run {program}"), &format!("{program} ausführen"))
        }
        System::Terminal => pick("Open a terminal", "Terminal öffnen"),
        System::Launcher => launch(de),
        System::Keys => pick("All keys", "Alle Tasten"),
        System::Chat => pick("Chat with a helper", "Mit einem Helfer chatten"),
        System::Lock => pick("Lock the screen", "Bildschirm sperren"),
        System::Power => pick("Power menu", "Ausschalt-Menü"),
        System::Screenshot => pick("Take a screenshot", "Bildschirmfoto aufnehmen"),
        System::Volume(Change::Up) => pick("Volume up", "Lauter"),
        System::Volume(Change::Down) => pick("Volume down", "Leiser"),
        System::Volume(Change::Mute) => pick("Mute", "Stumm"),
        System::Volume(Change::Set(percent)) => pick(
            &format!("Set the volume to {percent}%"),
            &format!("Lautstärke auf {percent} %"),
        ),
        System::Close => pick("Close window", "Fenster schließen"),
        System::Give(to) => pick(
            &format!("Give window to {to}"),
            &format!("Fenster geben an {to}"),
        ),
        System::Drag => pick("Drag window", "Fenster ziehen"),
        System::Desk(Step::Next) => pick("Next desk", "Nächster Desktop"),
        System::Desk(Step::Previous) => pick("Previous desk", "Voriger Desktop"),
        System::Cube(Turn::Left) => pick("Turn the cube to the left", "Würfel nach links drehen"),
        System::Cube(Turn::Right) => {
            pick("Turn the cube to the right", "Würfel nach rechts drehen")
        }
    })
}

/// The words of the launcher.
fn launch(de: bool) -> String {
    if de {
        "Programm starten oder Befehl ausführen"
    } else {
        "Start a program or run a command"
    }
    .to_owned()
}

/// The place of an action of the system in a key list: the order of
/// [`System`], the three volume keys in the order of the keys.
const fn rank(system: &System) -> u8 {
    match system {
        System::Run(_) => 0,
        System::Terminal => 1,
        System::Launcher => 2,
        System::Keys => 3,
        System::Chat => 4,
        System::Lock => 5,
        System::Power => 6,
        System::Screenshot => 7,
        System::Volume(Change::Up) => 8,
        System::Volume(Change::Down) => 9,
        System::Volume(Change::Mute) => 10,
        System::Volume(Change::Set(_)) => 11,
        System::Close => 12,
        System::Give(_) => 13,
        System::Drag => 14,
        System::Desk(Step::Next) => 15,
        System::Desk(Step::Previous) => 16,
        System::Cube(Turn::Left) => 17,
        System::Cube(Turn::Right) => 18,
    }
}

/// The rows of `bindings`, one per wording: the actions of the system
/// first, in the order of [`System`], then the actions of the look in the
/// order of the bindings. `named` answers for a binding and its action
/// the words that the look gives it: for an action of the look they are
/// the only words (none: no row), for an action of the system they
/// replace the words of kit (none: those words). Bindings of one wording
/// share a row.
pub fn rows<'a, L: Clone + 'a>(
    bindings: impl IntoIterator<Item = (String, &'a Action<L>)>,
    language: Language,
    named: impl Fn(&str, &Action<L>) -> Option<String>,
) -> Vec<Row<L>> {
    let mut found: Vec<(Option<u8>, Row<L>)> = Vec::new();
    for (binding, action) in bindings {
        let (place, words) = match action {
            Action::System(system) => (
                Some(rank(system)),
                named(&binding, action).or_else(|| words(system, language)),
            ),
            Action::Look(_) => (None, named(&binding, action)),
        };
        let Some(words) = words else {
            continue;
        };
        let chord = chord(&binding);
        if let Some((rank, row)) = found
            .iter_mut()
            .find(|(_, row)| row.words == words && row.action_is_system() == place.is_some())
        {
            *rank = (*rank).min(place);
            row.chords.push(chord);
            continue;
        }
        let row = Row {
            action: action.clone(),
            words,
            chords: vec![chord],
            binding,
        };
        found.push((place, row));
    }
    found.sort_by_key(|(place, _)| place.unwrap_or(u8::MAX));
    found.into_iter().map(|(_, row)| row).collect()
}

/// The keys that give a window to a look, as one row: the span of the
/// keys and the words. No file binds them.
#[must_use]
pub fn give_row(language: Language) -> (String, String) {
    let (keys, _) = crate::give::keys_row("-");
    let words = match language {
        Language::En => "Give window to a look",
        Language::De => "Fenster an einen Fenstermanager geben",
    };
    (keys, words.to_owned())
}

impl<L> Row<L> {
    /// Whether the action of the row is one of the system.
    #[must_use]
    pub const fn action_is_system(&self) -> bool {
        matches!(self.action, Action::System(_))
    }
}
