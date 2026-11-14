//! The doors of the construct: one glass capsule in the top left corner
//! with the clock and the plain words Programs, Keys, Lock, Power,
//! Screenshot and the volume word. It shows in the walk and flown into
//! a pane. A click on a word does what its key does (the keys of every
//! look, `Action::SHARED`); the clock stands still and is re-drawn each
//! minute. The words carry no button chrome (docs/construct-style.md,
//! rule 20): the pointer over a word shows the hover fill.

use std::time::{Duration, Instant};

use chora_kit::{
    glass::{CAPSULE, Material, Type, margin},
    volume::{Mixer, Reading},
    words::{Language, Word},
};
use chora_scene::{Command, Event, InputMode};

use crate::{
    BTN_LEFT, ClientError, Construct, NAME,
    config::Action,
    glass::{Surface, face, group, rect, surface, text},
    hud::Look,
};

/// The group of the doors, and the group of the nodes that take clicks,
/// which sits at the origin of the output.
pub const DOORS: &str = "construct-doors";
pub const HITS: &str = "construct-doors-hit";
/// The prefix of the ids of the doors, and the id of the clock.
pub const DOOR: &str = "construct-door-";
pub const CLOCK: &str = "construct-door-clock";

const PAD: f64 = 8.0;
const ITEM: f64 = 32.0;
const INSET: f64 = 12.0;
const SEPARATOR: f64 = 9.0;
/// How often the answer of the mixer is asked for, and how many times.
const READING: Duration = Duration::from_millis(100);
const READINGS: u8 = 50;

/// A word of the capsule that takes a click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Door {
    Programs,
    Keys,
    Lock,
    Power,
    Screenshot,
    Volume,
}

const ALL: [Door; 6] = [
    Door::Programs,
    Door::Keys,
    Door::Lock,
    Door::Power,
    Door::Screenshot,
    Door::Volume,
];

impl Door {
    fn name(self) -> &'static str {
        match self {
            Self::Programs => "programs",
            Self::Keys => "keys",
            Self::Lock => "lock",
            Self::Power => "power",
            Self::Screenshot => "screenshot",
            Self::Volume => "volume",
        }
    }

    /// The id of its click node.
    #[must_use]
    pub fn id(self) -> String {
        format!("{DOOR}{}", self.name())
    }

    /// The door that a node is.
    #[must_use]
    pub fn of(node: &str) -> Option<Self> {
        ALL.into_iter().find(|d| node == d.id())
    }

    /// The key that does what the door does.
    #[must_use]
    pub const fn action(self) -> Action {
        match self {
            Self::Programs => Action::Launcher,
            Self::Keys => Action::Help,
            Self::Lock => Action::Lock,
            Self::Power => Action::Power,
            Self::Screenshot => Action::Screenshot,
            Self::Volume => Action::Mute,
        }
    }

    /// Its word, the volume word with the reading.
    #[must_use]
    pub fn word(self, language: Language, volume: Option<Reading>) -> String {
        let (en, de) = match self {
            Self::Programs => ("Programs", "Programme"),
            Self::Keys => ("Keys", "Tasten"),
            Self::Lock => ("Lock", "Sperren"),
            Self::Power => ("Power", "Ausschalten"),
            Self::Screenshot => ("Screenshot", "Bildschirmfoto"),
            Self::Volume => return volume_word(language, volume),
        };
        match language {
            Language::En => en,
            Language::De => de,
        }
        .to_owned()
    }
}

/// `Volume 40%`, or the words of a reading without a level.
#[must_use]
pub fn volume_word(language: Language, volume: Option<Reading>) -> String {
    let title = Word::Volume.text(language);
    match volume {
        None => title.to_owned(),
        Some(reading @ Reading::Level(_)) => format!("{title} {}", reading.words(language)),
        Some(reading) => reading.words(language),
    }
}

/// What the capsule shows and how: the draw is repeated when it changes.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub out: [f64; 2],
    pub over: bool,
    pub dark: bool,
    pub plain: bool,
    pub clock: String,
    pub words: Vec<String>,
    pub hover: Option<Door>,
}

/// The state of the doors: the time, the volume reading and what is
/// drawn.
pub struct State {
    volume: Option<Reading>,
    clock: String,
    clock_due: Option<Instant>,
    reading_due: Option<Instant>,
    readings: u8,
    pub hover: Option<Door>,
    pub shown: Option<Shown>,
    pub stale: bool,
}

impl State {
    #[must_use]
    pub fn new() -> Self {
        Self {
            volume: None,
            clock: String::new(),
            clock_due: None,
            reading_due: None,
            readings: 0,
            hover: None,
            shown: None,
            stale: false,
        }
    }

    /// The next time that the doors need a look.
    #[must_use]
    pub fn deadline(&self) -> Option<Instant> {
        [self.clock_due, self.reading_due]
            .into_iter()
            .flatten()
            .min()
    }

    pub fn ask_for_reading(&mut self, now: Instant) {
        self.reading_due = Some(now + READING);
        self.readings = 0;
    }

    /// The clock of the minute and the newest reading of the mixer.
    fn refresh(&mut self, now: Instant, mixer: &Mixer) {
        if self.clock_due.is_none_or(|due| due <= now) {
            self.clock = local_time();
            self.clock_due = Some(now + to_next_minute());
            mixer.refresh();
            self.ask_for_reading(now);
        }
        if self.reading_due.is_some_and(|due| due <= now) {
            if let Some(reading) = mixer.take() {
                self.volume = Some(reading);
                self.reading_due = None;
            } else {
                self.readings += 1;
                self.reading_due = (self.readings < READINGS).then_some(now + READING);
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// The time now as `HH:MM`; `--:--` when `date` fails.
fn local_time() -> String {
    std::process::Command::new("date")
        .arg("+%H:%M")
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .filter(|time| !time.is_empty())
        .unwrap_or_else(|| "--:--".into())
}

/// The time to the next full minute, and a little more.
fn to_next_minute() -> Duration {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let ms = 60_000 - u64::try_from(now % 60_000).unwrap_or(0);
    Duration::from_millis(ms + 50)
}

/// The width of the capsule with `clock` and the `words`.
#[must_use]
pub fn capsule_width(look: &Look, clock: &str, words: &[String]) -> f64 {
    let small = |role, line: &str| look.size(role, line, None)[0].ceil();
    let clock_w = small(face(Type::Small, 500), clock);
    let doors: f64 = (words.iter())
        .map(|w| small(face(Type::Small, 400), w) + 2.0 * INSET)
        .sum();
    PAD + INSET + clock_w + INSET + SEPARATOR + doors + PAD
}

/// The capsule in the top left corner of an output of `out`, made anew.
#[must_use]
pub fn draw(look: &Look, shown: &Shown) -> Vec<Command> {
    let m: &Material = &look.glass;
    let w = capsule_width(look, &shown.clock, &shown.words);
    let x = margin(shown.out[0]);
    let y = margin(shown.out[0]);
    let top = (CAPSULE - ITEM) / 2.0;
    let line = top + (ITEM - 20.0) / 2.0;
    let mut commands = surface(
        look,
        m,
        &Surface {
            id: DOORS,
            parent: NAME,
            rect: [x, y, w, CAPSULE],
            radius: CAPSULE / 2.0,
            strong: shown.over,
            name: "doors",
            input: InputMode::Pass,
        },
    );
    commands.push(group(HITS, NAME, [0.0, 0.0], InputMode::Accept));
    let clock_w = look.size(face(Type::Small, 500), &shown.clock, None)[0].ceil();
    let mut left = PAD + INSET;
    commands.push(rect(
        CLOCK,
        DOORS,
        [left, top, clock_w, ITEM],
        0.0,
        [0.0; 4],
    ));
    commands.push(text(
        look,
        &format!("{CLOCK}-text"),
        DOORS,
        (face(Type::Small, 500), &shown.clock),
        [left, line],
        m.text,
    ));
    left += clock_w + INSET;
    commands.push(rect(
        &format!("{DOORS}-separator"),
        DOORS,
        [left + 4.0, (CAPSULE - 16.0) / 2.0, 1.0, 16.0],
        0.0,
        m.separator,
    ));
    left += SEPARATOR;
    commands.extend(words(look, shown, [x, y], left));
    commands
}

/// The words from `left` on: a click node and a text each.
fn words(look: &Look, shown: &Shown, [x, y]: [f64; 2], mut left: f64) -> Vec<Command> {
    let m: &Material = &look.glass;
    let top = (CAPSULE - ITEM) / 2.0;
    let line = top + (ITEM - 20.0) / 2.0;
    let mut commands = Vec::new();
    for (door, word) in ALL.into_iter().zip(&shown.words) {
        let item_w = look.size(face(Type::Small, 400), word, None)[0].ceil() + 2.0 * INSET;
        let fill = if shown.hover == Some(door) {
            m.hover
        } else {
            [0.0; 4]
        };
        let mut hit = rect(
            &door.id(),
            HITS,
            [x + left, y + top, item_w, ITEM],
            ITEM / 2.0,
            fill,
        );
        if let Command::Create { props, .. } = &mut hit {
            props.input = Some(InputMode::Accept);
        }
        commands.push(hit);
        commands.push(text(
            look,
            &format!("{}-text", door.id()),
            DOORS,
            (face(Type::Small, 400), word),
            [left + INSET, line],
            m.text,
        ));
        left += item_w;
    }
    commands
}

/// The commands that remove the capsule.
#[must_use]
pub fn remove() -> Vec<Command> {
    [DOORS, HITS]
        .map(|id| Command::Remove {
            id: id.into(),
            to: None,
        })
        .into()
}

impl Construct {
    /// Whether the capsule shows: with the HUD, and not under the help or
    /// the window list, which stand over the space.
    fn shows_doors(&self) -> bool {
        self.shows_hud() && !self.help && self.list.is_none()
    }

    /// The capsule anew when what it shows changed.
    pub(crate) fn draw_doors(&mut self) -> Result<(), ClientError> {
        self.doors.refresh(Instant::now(), &self.mixer);
        let wanted = self.shows_doors().then(|| self.doors_shown());
        if wanted == self.doors.shown && !std::mem::take(&mut self.doors.stale) {
            return Ok(());
        }
        let mut commands = if self.doors.shown.is_some() {
            remove()
        } else {
            Vec::new()
        };
        if let Some(shown) = &wanted {
            let capsule = self.fitted(|s| {
                let base = s.styled();
                let look = if shown.over {
                    crate::hints::over_content(&base)
                } else {
                    base
                };
                draw(&look, shown)
            })?;
            commands.extend(capsule);
        }
        self.doors.shown = wanted;
        self.rt.conn.apply(commands)
    }

    fn doors_shown(&self) -> Shown {
        let language = Language::of_session();
        Shown {
            out: [self.lens.w, self.lens.h],
            over: self.flown.is_some(),
            dark: self.settings.value.theme == crate::config::Theme::Dark,
            plain: self.glass.plain,
            clock: self.doors.clock.clone(),
            words: (ALL.into_iter())
                .map(|d| d.word(language, self.doors.volume))
                .collect(),
            hover: self.doors.hover,
        }
    }

    /// A click on a word, and the pointer over one; whether a word took
    /// the event.
    pub(crate) fn door_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        match event {
            Event::PointerPressed {
                node,
                button,
                injected,
                ..
            } if *button == BTN_LEFT => {
                let Some(door) = Door::of(node) else {
                    return Ok(false);
                };
                if !*injected || self.rt.accepts_injected() {
                    self.grabbed_key(door.action())?;
                }
                Ok(true)
            }
            Event::PointerEntered { node, .. } => {
                self.doors.hover = Door::of(node).or(self.doors.hover);
                Ok(false)
            }
            Event::PointerLeft { node } => {
                if Door::of(node) == self.doors.hover {
                    self.doors.hover = None;
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_door_has_its_node_and_a_key_of_every_look() {
        for door in ALL {
            assert_eq!(Door::of(&door.id()), Some(door));
            assert!(Action::SHARED.contains(&door.action()));
        }
        assert_eq!(Door::of("construct-door-clock"), None);
        assert_eq!(Door::of("construct-door-volume-text"), None);
    }

    #[test]
    fn the_volume_word_names_the_level_or_the_reason() {
        use chora_kit::volume::Volume;
        let level = Reading::Level(Volume {
            percent: 50,
            muted: false,
        });
        assert_eq!(volume_word(Language::En, Some(level)), "Volume 50%");
        assert_eq!(volume_word(Language::De, Some(level)), "Lautstärke 50%");
        assert_eq!(volume_word(Language::En, None), "Volume");
        assert_eq!(
            volume_word(Language::En, Some(Reading::NoWpctl)),
            "Sound unavailable"
        );
    }

    #[test]
    fn the_words_come_in_english_and_german() {
        assert_eq!(Door::Programs.word(Language::En, None), "Programs");
        assert_eq!(Door::Lock.word(Language::De, None), "Sperren");
    }

    #[test]
    fn the_time_to_the_next_minute_is_within_a_minute() {
        let wait = to_next_minute();
        assert!(wait > Duration::from_millis(50) && wait <= Duration::from_millis(60_050));
    }
}
