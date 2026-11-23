//! The words of the actions and the two overlays that chora-tiling draws
//! itself: the key list and the layout chip. Both sit under the screen
//! root of an output, as glass: a rect with a backdrop blur.

use std::collections::BTreeMap;

use chora_kit::{actions::System, keysheet, volume::Change, words::Language};
use chora_scene::{Backdrop, Border, Command, NodeKind, PropsUpdate, Transform};

use crate::{
    config::{Action, Config, Own},
    layout::Layout,
    theme::{Colors, channels},
};

/// The group of the key list, and of the layout chip.
pub const KEYS: &str = "tiling-keys";
pub const CHIP: &str = "tiling-chip";

/// The space that the panel leaves above and below: the bar (8 + 32) and
/// the gap of 12 under it, on both sides, so that it stays clear of it.
const BAR_CLEARANCE: f64 = 52.0;
/// The width of the panel of the key list.
const PANEL_WIDTH: f64 = 880.0;
/// The height of the header, above the first row.
const TOP: f64 = 84.0;
/// The space below the last row.
const BOTTOM: f64 = 24.0;
/// The space left of the first column, and the distance of the columns.
const LEFT: f64 = 24.0;
const COLUMN: f64 = 416.0;
/// The pitches of the rows that the panel tries, in order. The caps are 4
/// px less high, and never less than [`CAP_MIN`].
const PITCHES: [f64; 3] = [24.0, 22.0, 20.0];
/// How close to the room the panel may come before it takes all of it, so
/// that its top stands on the grid line under the bar.
const SNAP: f64 = 8.0;
/// The least height of a keycap: the keys in 13 px with their descenders
/// and 2 px to spare above and below.
const CAP_MIN: f64 = 18.0;
/// How far the top of the 13 px keys sits below the top of the 14 px label
/// beside them, so that both stand on one baseline.
const CAP_TEXT_DY: f64 = 1.0;
/// The air above a group title that follows a group in its column.
const GROUP_GAP: f64 = 8.0;
/// The padding of a keycap left and right, and the space to its label.
const CAP_PAD: f64 = 6.0;
const CAP_GAP: f64 = 10.0;
/// The height of a line of 14 px text, which sits in the middle of a pitch.
const LINE: f64 = 17.0;
/// The advance of a character of the keycap font at 13 px: 0.6 em.
const MONO_ADVANCE: f64 = 7.8;
/// The faces of the overlays: single family names, as the renderer takes
/// no list.
const UI_FONT: &str = "Inter";
const MONO_FONT: &str = "JetBrains Mono";

/// The word of a layout, as the panel shows it.
#[must_use]
pub fn layout_word(layout: Layout, lang: Language) -> &'static str {
    let de = lang == Language::De;
    match (layout, de) {
        (Layout::Tile, false) => "Tile",
        (Layout::Tile, true) => "Kacheln",
        (Layout::TileLeft, false) => "Tile left",
        (Layout::TileLeft, true) => "Kacheln links",
        (Layout::TileBottom, false) => "Tile bottom",
        (Layout::TileBottom, true) => "Kacheln unten",
        (Layout::TileTop, false) => "Tile top",
        (Layout::TileTop, true) => "Kacheln oben",
        (Layout::Fair, false) => "Fair",
        (Layout::Fair, true) => "Raster",
        (Layout::FairHorizontal, false) => "Fair rows",
        (Layout::FairHorizontal, true) => "Raster quer",
        (Layout::Spiral, false) => "Spiral",
        (Layout::Spiral, true) => "Spirale",
        (Layout::Dwindle, false) => "Dwindle",
        (Layout::Dwindle, true) => "Treppe",
        (Layout::Max, false) => "Max",
        (Layout::Max, true) => "Voll",
        (Layout::Fullscreen, false) => "Fullscreen",
        (Layout::Fullscreen, true) => "Vollbild",
        (Layout::Magnifier, false) => "Magnifier",
        (Layout::Magnifier, true) => "Lupe",
        (Layout::Floating, false) => "Floating",
        (Layout::Floating, true) => "Frei",
    }
}

/// The groups of the key list, in the order they are shown: the windows,
/// whose first row is the focus keys, then the give keys, which no file
/// binds.
const WINDOWS: usize = 0;
const LOOKS: usize = 1;
const LAYOUTS: usize = 2;
const TAGS: usize = 3;
const PROGRAMS: usize = 4;
const SYSTEM: usize = 5;

/// Whether an action is the settings door, which the list shows beside the
/// key that reads the settings again.
fn opens_settings(action: &Action) -> bool {
    matches!(action, Action::System(System::Run(command)) if command.ends_with("--launch settings"))
}

/// The group of the key list that an action belongs to.
fn group_of(action: &Action) -> usize {
    match action {
        Action::System(System::Run(_)) if opens_settings(action) => SYSTEM,
        Action::System(
            System::Run(_)
            | System::Terminal
            | System::Launcher
            | System::Chat
            | System::Lock
            | System::Power
            | System::Screenshot,
        ) => PROGRAMS,
        Action::System(System::Close | System::Drag) => WINDOWS,
        Action::System(_) => SYSTEM,
        Action::Look(own) => {
            let own = Action::Look(own.clone());
            if window_words(&own).is_some() {
                WINDOWS
            } else if layout_words(&own).is_some() {
                LAYOUTS
            } else if tag_words(&own).is_some() {
                TAGS
            } else {
                SYSTEM
            }
        }
    }
}

/// The words of an action of the look, or the label that the file gives
/// its binding; a `{}` stands for the number of a tag. None for an action
/// of the system without a label: kit names it.
fn named(config: &Config, binding: &str, action: &Action, de: bool) -> Option<String> {
    if let Some(label) = config
        .labels
        .iter()
        .find(|(bound, _)| bound.replace("MOD", &config.modifier) == binding)
        .map(|(_, label)| label)
    {
        return Some(label.clone());
    }
    let [en, german] = window_words(action)
        .or_else(|| layout_words(action))
        .or_else(|| tag_words(action))
        .or_else(|| own_words(action))?;
    Some((if de { german } else { en }).to_owned())
}

type Words = [&'static str; 2];

fn window_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::Look(Own::FocusNext | Own::FocusPrevious) => ["Move focus", "Fokus bewegen"],
        Action::Look(Own::FocusBack) => ["Window before", "Fenster davor"],
        Action::Look(Own::FocusUrgent) => ["Window that asks", "Fenster, das ruft"],
        Action::Look(Own::SwapNext | Own::SwapPrevious) => ["Swap window", "Fenster tauschen"],
        Action::Look(Own::MoveToMaster) => ["Make window main", "Fenster zum Hauptfenster"],
        Action::Look(Own::ToggleFloating) => ["Float window", "Fenster schweben lassen"],
        Action::Look(Own::ToggleFullscreen) => ["Full screen", "Vollbild"],
        Action::Look(Own::ToggleMaximized) => ["Maximize", "Maximieren"],
        Action::Look(Own::ToggleMaximizedHorizontal) => ["Maximize wide", "Breit maximieren"],
        Action::Look(Own::ToggleMaximizedVertical) => ["Maximize tall", "Hoch maximieren"],
        Action::Look(Own::ToggleOnTop) => ["Keep on top", "Immer oben"],
        Action::Look(Own::Minimize) => ["Hide window", "Fenster verstecken"],
        Action::Look(Own::RestoreMinimized) => ["Bring back window", "Fenster zurückholen"],
        _ => return None,
    })
}

fn layout_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::Look(Own::IncMasterWidth(s)) if *s >= 0.0 => {
            ["Wider main area", "Hauptbereich breiter"]
        }
        Action::Look(Own::IncMasterWidth(_)) => ["Narrower main area", "Hauptbereich schmaler"],
        Action::Look(Own::IncMasters(s)) if *s >= 0 => ["More main windows", "Mehr Hauptfenster"],
        Action::Look(Own::IncMasters(_)) => ["Fewer main windows", "Weniger Hauptfenster"],
        Action::Look(Own::IncColumns(s)) if *s >= 0 => ["More columns", "Mehr Spalten"],
        Action::Look(Own::IncColumns(_)) => ["Fewer columns", "Weniger Spalten"],
        Action::Look(Own::NextLayout) => ["Next layout", "Nächste Anordnung"],
        Action::Look(Own::PreviousLayout) => ["Previous layout", "Vorige Anordnung"],
        Action::Look(Own::FocusNextOutput) => ["Next screen", "Nächster Bildschirm"],
        Action::Look(Own::FocusPreviousOutput) => ["Previous screen", "Voriger Bildschirm"],
        Action::Look(Own::MoveToNextOutput) => {
            ["Window to next screen", "Fenster auf nächsten Bildschirm"]
        }
        _ => return None,
    })
}

fn tag_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::Look(Own::ViewTag(_)) => ["Show tag {}", "Tag {} zeigen"],
        Action::Look(Own::ToggleTagView(_)) => ["Add or remove tag {}", "Tag {} dazu oder weg"],
        Action::Look(Own::MoveToTag(_)) => ["Window to tag {}", "Fenster auf Tag {}"],
        Action::Look(Own::ToggleWindowTag(_)) => {
            ["Window also on tag {}", "Fenster auch auf Tag {}"]
        }
        Action::Look(Own::ViewNext) => ["Next tag", "Nächster Tag"],
        Action::Look(Own::ViewPrevious) => ["Previous tag", "Voriger Tag"],
        Action::Look(Own::ViewBack) => ["Tag before", "Tag davor"],
        _ => return None,
    })
}

fn own_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::Look(Own::Reload) => ["Read the settings again", "Einstellungen neu lesen"],
        _ => return None,
    })
}

/// The title of a group of the key list.
fn title(group: usize, de: bool) -> &'static str {
    let names = [
        ["Windows", "Fenster"],
        ["Other looks", "Andere Fenstermanager"],
        ["Layouts", "Anordnung"],
        ["Tags", "Tags"],
        ["Programs", "Programme"],
        ["System", "System"],
    ];
    names[group][usize::from(de)]
}

/// The number at the end of a key, as `view_tag` binds it.
fn number(action: &Action) -> Option<usize> {
    match action {
        Action::Look(
            Own::ViewTag(n) | Own::ToggleTagView(n) | Own::MoveToTag(n) | Own::ToggleWindowTag(n),
        ) => Some(*n),
        _ => None,
    }
}

/// Whether an action moves the focus to another window: its key shows in
/// the list without `MOD`, as Alt+Tab does.
fn moves_focus(action: &Action) -> bool {
    matches!(action, Action::Look(Own::FocusNext | Own::FocusPrevious))
}

/// A row before it is sorted: its rank, its keys and its words.
type Entry = (usize, String, String);

/// The English words of the rows in the order that the key list shows
/// them, by group; a row that is not here follows the others, by its keys.
const ORDER: [&str; 46] = [
    "Move focus",
    "Window before",
    "Window that asks",
    "Swap window",
    "Make window main",
    "Close window",
    "Float window",
    "Full screen",
    "Maximize",
    "Maximize wide",
    "Maximize tall",
    "Keep on top",
    "Hide window",
    "Bring back window",
    "Drag window",
    "Next layout",
    "Previous layout",
    "Wider main area",
    "Narrower main area",
    "More main windows",
    "Fewer main windows",
    "More columns",
    "Fewer columns",
    "Next screen",
    "Previous screen",
    "Window to next screen",
    "Show tag {}",
    "Add or remove tag {}",
    "Window to tag {}",
    "Window also on tag {}",
    "Next tag",
    "Previous tag",
    "Tag before",
    "Open a terminal",
    "Start a program or run a command",
    "Chat with a helper",
    "Lock the screen",
    "Power menu",
    "Take a screenshot",
    "Screenshot of an area",
    "Volume up",
    "Volume down",
    "Mute",
    "Edit settings",
    "Read the settings again",
    "All keys",
];

/// The keys of the rows of the tags: the bindings of one stem with their
/// digits as a span, `Super+1-9`, one span per stem.
fn spans(chords: &[String]) -> String {
    let mut spans: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for chord in chords {
        let stem = chord.trim_end_matches(|c: char| c.is_ascii_digit());
        let Ok(n) = chord[stem.len()..].parse::<usize>() else {
            continue;
        };
        let span = spans.entry(stem.to_owned()).or_insert((n, n));
        (span.0, span.1) = (span.0.min(n), span.1.max(n));
    }
    let named: Vec<String> = (spans.into_iter())
        .map(|(stem, (low, high))| {
            let range = if low == high {
                low.to_string()
            } else {
                format!("{low}-{high}")
            };
            format!("{stem}{range}")
        })
        .collect();
    named.join(" / ")
}

/// A row of kit as a row of the list: its rank by the English words, its
/// keys and its words, with the tags as a span.
fn entry(config: &Config, row: &keysheet::Row<Own>) -> Entry {
    let english =
        named(config, &row.binding, &row.action, false).unwrap_or_else(|| match &row.action {
            Action::System(system) => keysheet::words(system, Language::En).unwrap_or_default(),
            Action::Look(_) => String::new(),
        });
    let rank = ORDER
        .iter()
        .position(|w| *w == english)
        .unwrap_or(ORDER.len());
    if number(&row.action).is_none() {
        let keys = row.chords.clone();
        return (rank, keys.join(" / "), row.words.clone());
    }
    let keys = spans(&row.chords);
    let range = keys.rsplit(['+', ' ']).next().unwrap_or_default();
    let words = row.words.replace("{}", range);
    (rank, keys, words)
}

/// The rows of the key list by group: the keys as kit spells them, with the
/// modifier of the file for `MOD`, and the words. The bindings of the tags
/// 1 to 9 take one row, a binding with the words of another joins its row,
/// the media keys of the volume take one, and the give keys, which no
/// file binds, another.
#[must_use]
pub fn rows(config: &Config, lang: Language) -> Vec<(&'static str, Vec<(String, String)>)> {
    let de = lang == Language::De;
    let media = |action: &Action| {
        matches!(
            action,
            Action::System(System::Volume(Change::Up | Change::Down | Change::Mute))
        )
    };
    let resolved = (config.common.bindings.iter())
        .filter(|(binding, action)| binding.contains("MOD") || media(action) || moves_focus(action))
        .map(|(binding, action)| (binding.replace("MOD", &config.modifier), action));
    let sheet = keysheet::rows(resolved, lang, |binding, action| {
        named(config, binding, action, de)
    });
    let mut groups: Vec<Vec<Entry>> = vec![Vec::new(); SYSTEM + 1];
    let mut volume_keys: Vec<Entry> = Vec::new();
    for row in &sheet {
        let entry = entry(config, row);
        let volume = matches!(
            row.action,
            Action::System(System::Volume(Change::Up | Change::Down | Change::Mute))
        );
        if volume {
            volume_keys.push(entry);
        } else {
            groups[group_of(&row.action)].push(entry);
        }
    }
    if let Some((rank, _, _)) = volume_keys.first() {
        let rank = *rank;
        let join =
            |part: fn(&Entry) -> &str| volume_keys.iter().map(part).collect::<Vec<_>>().join(" / ");
        let row = (rank, join(|e| &e.1), join(|e| &e.2));
        groups[SYSTEM].push(row);
    }
    let (keys, words) = keysheet::give_row(lang);
    groups[LOOKS].push((ORDER.len(), keys, words));
    groups
        .into_iter()
        .enumerate()
        .filter(|(_, rows)| !rows.is_empty())
        .map(|(group, mut rows)| {
            rows.sort();
            let rows = rows.into_iter().map(|(_, k, l)| (k, l)).collect();
            (title(group, de), rows)
        })
        .collect()
}

fn color(rgb: [f64; 3], alpha: f64) -> [f64; 4] {
    [rgb[0], rgb[1], rgb[2], alpha]
}

/// A count as a length in pixels.
fn px(n: usize) -> f64 {
    f64::from(u32::try_from(n).unwrap_or(u32::MAX))
}

fn create(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

/// How a text looks: its place, size, weight, face and colour.
struct Face {
    at: [f64; 2],
    size: f64,
    weight: u16,
    mono: bool,
    rgba: [f64; 4],
}

impl Face {
    fn new(at: [f64; 2], size: f64, weight: u16, rgb: [f64; 3]) -> Self {
        Self {
            at,
            size,
            weight,
            mono: false,
            rgba: color(rgb, 1.0),
        }
    }
}

fn text(id: &str, parent: &str, words: &str, face: &Face) -> Command {
    create(
        id,
        NodeKind::Text,
        parent,
        PropsUpdate {
            transform: Some(Transform::translate(face.at[0], face.at[1], 0.0)),
            text: Some(words.into()),
            font_size: Some(face.size),
            weight: Some(face.weight),
            family: Some(if face.mono { MONO_FONT } else { UI_FONT }.to_owned()),
            color: Some(face.rgba),
            ..PropsUpdate::default()
        },
    )
}

/// A glass rect: the surface colour, a hairline in the line colour and a
/// backdrop blur, at a place.
fn glass(id: &str, parent: &str, at: [f64; 2], size: [f64; 2], colors: &Colors) -> Command {
    create(
        id,
        NodeKind::Rect,
        parent,
        PropsUpdate {
            transform: Some(Transform::translate(at[0], at[1], 0.0)),
            size: Some(size),
            color: Some(color(channels(&colors.surface), 0.97)),
            radius: Some(14.0),
            border: Some(Some(Border {
                width: 1.0,
                color: color(channels(&colors.line), 1.0),
            })),
            backdrop: Some(Some(Backdrop {
                blur: 24.0,
                saturation: 1.2,
            })),
            ..PropsUpdate::default()
        },
    )
}

/// What a row of the key list draws, in the coordinates of the panel.
#[derive(Debug, PartialEq)]
pub enum Item {
    /// The title of a group.
    Title { at: [f64; 2], text: String },
    /// A keycap: its box and the keys in it.
    Cap {
        at: [f64; 2],
        size: [f64; 2],
        keys: String,
        text_y: f64,
    },
    /// The words of a row.
    Label { at: [f64; 2], text: String },
}

/// The key list laid out: the size of the panel and its items.
#[derive(Debug, PartialEq)]
pub struct Plan {
    pub size: [f64; 2],
    pub items: Vec<Item>,
}

/// The height of a keycap on a pitch.
fn cap_height(pitch: f64) -> f64 {
    (pitch - 4.0).max(CAP_MIN)
}

/// The width of a keycap: the keys in the monospace face with the same
/// padding left and right.
fn cap_width(keys: &str) -> f64 {
    px(keys.chars().count()) * MONO_ADVANCE + 2.0 * CAP_PAD
}

/// The height of a column of groups at a pitch: a title and a row each
/// take a pitch, and a group after another has air above it.
fn column_height(groups: &[&(&str, Vec<(String, String)>)], pitch: f64) -> f64 {
    let lines: usize = groups.iter().map(|(_, rows)| rows.len() + 1).sum();
    px(lines) * pitch + GROUP_GAP * px(groups.len().saturating_sub(1))
}

/// Lays out the groups in two columns, each group whole in one of them, so
/// that the taller column is least tall; at the first pitch that fits
/// `room`, the height that the panel may have.
#[must_use]
pub fn plan(groups: &[(&str, Vec<(String, String)>)], room: f64) -> Plan {
    let all: Vec<&(&str, Vec<(String, String)>)> = groups.iter().collect();
    let pitch = PITCHES
        .into_iter()
        .find(|&p| TOP + tallest(&all, best_split(&all, p), p) + BOTTOM <= room)
        .unwrap_or(PITCHES[PITCHES.len() - 1]);
    let split = best_split(&all, pitch);
    let content = TOP + tallest(&all, split, pitch) + BOTTOM;
    let height = if content >= room - SNAP {
        room
    } else {
        content
    };
    let line_gap = ((pitch - LINE) / 2.0).round();
    let mut items = Vec::new();
    for column in 0..2 {
        let members = members(&all, split, column);
        let x = LEFT + COLUMN * px(column);
        let widest = members
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|(keys, _)| cap_width(keys)))
            .fold(0.0, f64::max);
        let mut y = TOP;
        for (n, (title, rows)) in members.iter().enumerate() {
            if n > 0 {
                y += GROUP_GAP;
            }
            items.push(Item::Title {
                at: [x, y + line_gap],
                text: (*title).to_owned(),
            });
            y += pitch;
            let cap_height = cap_height(pitch);
            for (keys, label) in rows {
                items.push(Item::Cap {
                    at: [x, y + ((pitch - cap_height) / 2.0).floor()],
                    size: [widest, cap_height],
                    keys: keys.clone(),
                    text_y: y + line_gap + CAP_TEXT_DY,
                });
                items.push(Item::Label {
                    at: [x + widest + CAP_GAP, y + line_gap],
                    text: label.clone(),
                });
                y += pitch;
            }
        }
    }
    Plan {
        size: [PANEL_WIDTH, height],
        items,
    }
}

/// The groups of a column, in their order; bit n of `split` puts group n
/// in the second column.
fn members<'a>(
    all: &[&'a (&str, Vec<(String, String)>)],
    split: u32,
    column: usize,
) -> Vec<&'a (&'a str, Vec<(String, String)>)> {
    all.iter()
        .enumerate()
        .filter(|(n, _)| usize::from(split >> n & 1 == 1) == column)
        .map(|(_, g)| *g)
        .collect()
}

/// The height of the taller column of a split.
fn tallest(all: &[&(&str, Vec<(String, String)>)], split: u32, pitch: f64) -> f64 {
    let height = |column| column_height(&members(all, split, column), pitch);
    height(0).max(height(1))
}

/// The split of the groups that makes the taller column least tall; of
/// equal ones the one that keeps the first groups in the first column.
fn best_split(all: &[&(&str, Vec<(String, String)>)], pitch: f64) -> u32 {
    (0..1_u32 << all.len().min(16))
        .filter(|split| split & 1 == 0)
        .min_by(|&a, &b| tallest(all, a, pitch).total_cmp(&tallest(all, b, pitch)))
        .unwrap_or(0)
}

/// The alpha of the scrim in the `base` colour of the theme: little enough
/// that the bar, which no scrim reaches, stays within a few levels of the
/// surfaces below it.
const SCRIM: f64 = 0.20;

/// The commands that draw the key list centred on an output of the given
/// size, under its screen root `parent`.
#[must_use]
pub fn key_list(
    parent: &str,
    output: [f64; 2],
    groups: &[(&str, Vec<(String, String)>)],
    colors: &Colors,
    lang: Language,
) -> Vec<Command> {
    let plan = plan(groups, output[1] - 2.0 * BAR_CLEARANCE);
    let origin = [
        (output[0] - plan.size[0]) / 2.0,
        (output[1] - plan.size[1]) / 2.0,
    ];
    let scrim = PropsUpdate {
        size: Some(output),
        color: Some(color(channels(&colors.base), SCRIM)),
        ..PropsUpdate::default()
    };
    let body = format!("{KEYS}-body");
    let mut commands = vec![
        create(
            KEYS,
            NodeKind::Group,
            parent,
            PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        ),
        create(&format!("{KEYS}-scrim"), NodeKind::Rect, KEYS, scrim),
        glass(&format!("{KEYS}-panel"), KEYS, origin, plan.size, colors),
        create(
            &body,
            NodeKind::Group,
            KEYS,
            PropsUpdate {
                transform: Some(Transform::translate(origin[0], origin[1], 0.0)),
                ..PropsUpdate::default()
            },
        ),
    ];
    commands.extend(header(&body, colors, lang));
    commands.extend(draw(&plan.items, &body, colors));
    commands
}

/// Whether the renderer's size of the text node `id` counts for the width of
/// the panel: the labels of the rows and the Mouse line.
#[must_use]
pub fn sets_width(id: &str) -> bool {
    id.starts_with(KEYS) && (id.ends_with("-l") || id.ends_with("-b"))
}

/// The updates that make the panel as wide as its content: `right` is where
/// the longest word ends, in the coordinates of the panel, which the
/// renderer reports once it drew them. The panel keeps its height, takes
/// [`LEFT`] of room right of the word, as it has left of the keycaps, and
/// stays centred on an output of the given size.
#[must_use]
pub fn fit_list(output: [f64; 2], height: f64, right: f64) -> Vec<Command> {
    let width = (right + LEFT).ceil();
    let origin = [
        ((output[0] - width) / 2.0).floor(),
        ((output[1] - height) / 2.0).floor(),
    ];
    let place = || Some(Transform::translate(origin[0], origin[1], 0.0));
    vec![
        Command::Update {
            id: format!("{KEYS}-panel"),
            props: PropsUpdate {
                size: Some([width, height]),
                transform: place(),
                ..PropsUpdate::default()
            },
        },
        Command::Update {
            id: format!("{KEYS}-body"),
            props: PropsUpdate {
                transform: place(),
                ..PropsUpdate::default()
            },
        },
    ]
}

/// The commands of the items of a plan, under the group `body`.
fn draw(items: &[Item], body: &str, colors: &Colors) -> Vec<Command> {
    let mut commands = Vec::new();
    let (text_rgb, muted_rgb) = (channels(&colors.text), channels(&colors.muted));
    for (n, item) in items.iter().enumerate() {
        let id = format!("{body}-{n}");
        match item {
            Item::Title { at, text: words } => {
                let face = Face::new(*at, 14.0, 600, muted_rgb);
                commands.push(text(&id, body, words, &face));
            }
            Item::Cap {
                at,
                size,
                keys,
                text_y,
            } => {
                let keycap = PropsUpdate {
                    size: Some(*size),
                    color: Some(color(channels(&colors.surface_2), 1.0)),
                    radius: Some(6.0),
                    transform: Some(Transform::translate(at[0], at[1], 0.0)),
                    ..PropsUpdate::default()
                };
                commands.push(create(&format!("{id}-cap"), NodeKind::Rect, body, keycap));
                let inset = (size[0] - cap_width(keys)) / 2.0 + CAP_PAD;
                let mut mono = Face::new([at[0] + inset, *text_y], 13.0, 400, text_rgb);
                mono.mono = true;
                commands.push(text(&format!("{id}-k"), body, keys, &mono));
            }
            Item::Label { at, text: words } => {
                let face = Face::new(*at, 14.0, 400, text_rgb);
                commands.push(text(&format!("{id}-l"), body, words, &face));
            }
        }
    }
    commands
}

fn header(body: &str, colors: &Colors, lang: Language) -> Vec<Command> {
    let de = lang == Language::De;
    let (name, hint, mouse, basics) = if de {
        (
            "Tasten",
            "Super+F1 zeigt diese Liste wieder. Esc schließt sie.",
            "Maus",
            "Apps startet Programme · Tag anklicken zeigt ihn · Super+ziehen verschiebt · \
             Ein/Aus-Symbol: Sitzung, Bildschirmfoto",
        )
    } else {
        (
            "Keys",
            "Super+F1 shows this list again. Esc closes it.",
            "Mouse",
            "Apps starts a program · click a tag to show it · Super+drag moves a window · \
             power icon: lock, log out, screenshot",
        )
    };
    let muted = channels(&colors.muted);
    vec![
        text(
            &format!("{body}-t"),
            body,
            name,
            &Face::new([24.0, 20.0], 20.0, 600, channels(&colors.text)),
        ),
        text(
            &format!("{body}-h"),
            body,
            hint,
            &Face::new([120.0, 26.0], 14.0, 400, muted),
        ),
        text(
            &format!("{body}-m"),
            body,
            mouse,
            &Face::new([24.0, 56.0], 14.0, 600, muted),
        ),
        text(
            &format!("{body}-b"),
            body,
            basics,
            &Face::new([84.0, 56.0], 14.0, 400, channels(&colors.text)),
        ),
    ]
}

/// The padding of the layout chip around its word, left and right, and
/// above and below.
const CHIP_PAD: [f64; 2] = [24.0, 16.0];

/// Where the chip and its word go for a word of the size `measured`, which
/// the renderer reports: the box of the chip, as big as the word and its
/// padding and centred on the output, and the place of the word in the
/// middle of it, on both axes.
#[must_use]
pub fn chip_places(output: [f64; 2], measured: [f64; 2]) -> ([f64; 2], [f64; 2], [f64; 2]) {
    let size = [
        measured[0] + 2.0 * CHIP_PAD[0],
        measured[1] + 2.0 * CHIP_PAD[1],
    ];
    let at = [
        (output[0] - size[0]) / 2.0,
        ((output[1] - size[1]) / 2.0).floor(),
    ];
    (at, size, [at[0] + CHIP_PAD[0], at[1] + CHIP_PAD[1]])
}

/// What the renderer reports for a word that it did not draw yet: a guess
/// from the number of letters.
#[must_use]
pub fn guess(word: &str) -> [f64; 2] {
    [px(word.chars().count()) * 12.0, 24.0]
}

/// The commands that draw the layout chip under the screen root `parent`.
/// The renderer knows the size of the word only once it drew it, so the
/// chip starts invisible with a guess; [`chip_fit`] then sets it.
#[must_use]
pub fn chip(parent: &str, output: [f64; 2], word: &str, colors: &Colors) -> Vec<Command> {
    let (at, size, word_at) = chip_places(output, guess(word));
    let face = Face::new(word_at, 20.0, 600, channels(&colors.text));
    vec![
        create(
            CHIP,
            NodeKind::Group,
            parent,
            PropsUpdate {
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        ),
        glass(&format!("{CHIP}-glass"), CHIP, at, size, colors),
        text(&format!("{CHIP}-word"), CHIP, word, &face),
    ]
}

/// The updates that fit the chip to the measured word and show it, in a
/// fade when `animate`.
#[must_use]
pub fn chip_fit(output: [f64; 2], measured: [f64; 2], animate: bool) -> Vec<Command> {
    let (at, size, word_at) = chip_places(output, measured);
    let update = |id: String, props| Command::Update { id, props };
    vec![
        update(
            format!("{CHIP}-glass"),
            PropsUpdate {
                size: Some(size),
                transform: Some(Transform::translate(at[0], at[1], 0.0)),
                ..PropsUpdate::default()
            },
        ),
        update(
            format!("{CHIP}-word"),
            PropsUpdate {
                transform: Some(Transform::translate(word_at[0], word_at[1], 0.0)),
                ..PropsUpdate::default()
            },
        ),
        crate::dress::change(
            CHIP.into(),
            PropsUpdate {
                opacity: Some(1.0),
                ..PropsUpdate::default()
            },
            animate.then_some(crate::dress::FADE),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::parse;

    use super::*;
    use crate::{config::DEFAULT, theme::Scheme};

    fn groups() -> Vec<(&'static str, Vec<(String, String)>)> {
        rows(&parse::<Config>(DEFAULT).unwrap(), Language::En)
    }

    fn laid_out(room: f64) -> Plan {
        plan(&groups(), room)
    }

    fn column(x: f64) -> usize {
        usize::from(x >= LEFT + COLUMN - 1.0)
    }

    #[test]
    fn a_binding_without_mod_has_no_row_but_the_media_keys_do() {
        let mut config = parse::<Config>(DEFAULT).unwrap();
        config
            .common
            .bindings
            .insert("Print".into(), Action::System(System::Screenshot));
        config.common.bindings.insert(
            "XF86AudioMute".into(),
            Action::System(System::Volume(Change::Mute)),
        );
        let keys: Vec<String> = rows(&config, Language::En)
            .into_iter()
            .flat_map(|(_, rows)| rows.into_iter().map(|(keys, _)| keys))
            .collect();
        assert!(
            !keys
                .iter()
                .any(|k| k.split(" / ").any(|key| key == "Print")),
            "{keys:?}"
        );
        assert!(keys.iter().any(|k| k.contains("Mute")), "{keys:?}");
    }

    #[test]
    fn an_arrow_keeps_the_thin_space_that_kit_spells_before_it() {
        let config = parse::<Config>(DEFAULT).unwrap();
        let keys: Vec<String> = rows(&config, Language::En)
            .into_iter()
            .flat_map(|(_, rows)| rows.into_iter().map(|(keys, _)| keys))
            .filter(|k| k.contains(['←', '→', '↑', '↓']))
            .collect();
        assert!(!keys.is_empty(), "no arrow key in the list");
        for key in keys {
            for (at, c) in key.char_indices().filter(|(_, c)| "←→↑↓".contains(*c)) {
                assert!(key[..at].ends_with("+\u{2009}"), "{key:?} at {c}");
            }
        }
    }

    #[test]
    fn no_two_lines_of_a_column_share_a_height() {
        for room in [1000.0, 696.0] {
            let plan = laid_out(room);
            let mut seen = Vec::new();
            for item in &plan.items {
                let (Item::Title { at, .. } | Item::Label { at, .. }) = item else {
                    continue;
                };
                let key = (column(at[0]), at[1].to_bits());
                assert!(!seen.contains(&key), "two lines at {at:?}");
                seen.push(key);
            }
        }
    }

    #[test]
    fn rows_keep_a_pitch_of_the_grid_and_caps_leave_a_gap() {
        for (room, cap) in [(1000.0, 20.0), (696.0, 18.0)] {
            let plan = laid_out(room);
            let mut caps: Vec<(usize, f64, f64)> = plan
                .items
                .iter()
                .filter_map(|i| match i {
                    Item::Cap { at, size, .. } => Some((column(at[0]), at[1], size[1])),
                    _ => None,
                })
                .collect();
            caps.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
            for pair in caps.windows(2).filter(|p| p[0].0 == p[1].0) {
                let gap = pair[1].1 - pair[0].1 - pair[0].2;
                assert!(gap >= 4.0, "caps {pair:?} are {gap} px apart");
            }
            assert!(
                caps.iter().all(|c| (c.2 - cap).abs() < 1e-9),
                "caps are {cap} px high in a room of {room}"
            );
        }
    }

    #[test]
    fn the_caps_of_a_column_are_as_wide_as_its_widest_and_their_right_edges_meet() {
        let plan = laid_out(1000.0);
        for c in 0..2 {
            let mut widths = Vec::new();
            for item in &plan.items {
                if let Item::Cap { at, size, keys, .. } = item
                    && column(at[0]) == c
                {
                    let text = px(keys.chars().count()) * MONO_ADVANCE;
                    assert!(size[0] >= text + 2.0 * CAP_PAD - 1e-9, "{keys}");
                    widths.push(size[0]);
                }
            }
            assert!(
                widths.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-9),
                "{widths:?}"
            );
        }
    }

    #[test]
    fn a_panel_that_nearly_fills_the_room_takes_all_of_it() {
        let own = laid_out(2000.0).size[1];
        assert!((laid_out(own + 4.0).size[1] - (own + 4.0)).abs() < 1e-9);
        assert!((laid_out(own + 40.0).size[1] - own).abs() < 1e-9);
    }

    #[test]
    fn the_labels_of_a_column_start_at_one_x() {
        let plan = laid_out(1000.0);
        for c in 0..2 {
            let mut xs: Vec<f64> = plan
                .items
                .iter()
                .filter_map(|i| match i {
                    Item::Label { at, .. } if column(at[0]) == c => Some(at[0]),
                    _ => None,
                })
                .collect();
            xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
            assert_eq!(xs.len(), 1, "column {c}: {xs:?}");
        }
    }

    #[test]
    fn a_group_is_whole_in_one_column_and_has_one_title() {
        let titles: Vec<String> = laid_out(1000.0)
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Title { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        let mut names: Vec<String> = groups().iter().map(|(n, _)| (*n).to_owned()).collect();
        names.sort();
        let mut seen = titles.clone();
        seen.sort();
        assert_eq!(seen, names, "{titles:?}");
    }

    #[test]
    fn the_panel_is_its_content_and_24_px() {
        let plan = laid_out(1000.0);
        let low = plan
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Cap { at, size, .. } => Some(at[1] + size[1] + 2.0),
                _ => None,
            })
            .fold(0.0, f64::max);
        assert!((plan.size[1] - low - BOTTOM).abs() < 1e-9, "{plan:?}");
        assert_eq!(plan.size[0], 880.0);
        let tight = laid_out(696.0);
        assert!(tight.size[1] <= 696.0, "the panel is {}", tight.size[1]);
    }

    #[test]
    fn the_tooltips_of_the_bar_spell_keys_as_the_key_list_does() {
        let flavor =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling/waybar");
        let list: Vec<String> = groups()
            .iter()
            .flat_map(|(_, rows)| rows.iter().flat_map(|(keys, _)| keys.split(" / ")))
            .map(str::to_owned)
            .collect();
        let mut named = 0;
        for file in ["config.jsonc", "layout.sh"] {
            let text = std::fs::read_to_string(flavor.join(file)).unwrap();
            for line in text
                .lines()
                .filter(|l| l.contains("tip") && !l.contains("de*"))
            {
                for word in line
                    .split(['(', ')', '"', '\\'])
                    .filter(|w| w.starts_with("Super+"))
                {
                    named += 1;
                    assert!(
                        list.iter().any(|keys| keys == word),
                        "{file}: {word} is not spelled like the key list:\n{list:?}"
                    );
                }
            }
        }
        assert_eq!(named, 1);
    }

    #[test]
    fn the_doors_of_the_bar_have_no_tooltip() {
        let flavor =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling/waybar");
        let text = std::fs::read_to_string(flavor.join("config.jsonc")).unwrap();
        for door in ["apps", "helper", "keys", "power"] {
            let from = text
                .find(&format!("\"custom/{door}\": {{"))
                .unwrap_or_else(|| panic!("no module custom/{door}"));
            let block = &text[from..from + text[from..].find("\n  }").unwrap()];
            assert!(
                block.contains("\"tooltip\": false"),
                "custom/{door} has a tooltip:\n{block}"
            );
        }
    }

    #[test]
    fn the_panel_fits_its_longest_word_with_the_margin_of_the_left() {
        let commands = fit_list([1280.0, 800.0], 640.0, 764.0);
        let Command::Update { props, .. } = &commands[0] else {
            panic!("no update of the panel");
        };
        assert_eq!(props.size, Some([788.0, 640.0]));
        assert_eq!(
            props.transform,
            Some(Transform::translate(246.0, 80.0, 0.0))
        );
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn the_rows_follow_the_order_of_a_person_not_of_the_keys() {
        let groups = groups();
        let names: Vec<&str> = groups.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names,
            [
                "Windows",
                "Other looks",
                "Layouts",
                "Tags",
                "Programs",
                "System"
            ]
        );
        let first = |g: usize| groups[g].1[0].1.clone();
        assert_eq!(first(0), "Move focus");
        assert_eq!(first(2), "Next layout");
        assert_eq!(first(3), "Show tag 1-9");
        assert_eq!(first(4), "Open a terminal");
        assert_eq!(groups[0].1[1].1, "Window before");
    }

    #[test]
    fn the_focus_keys_are_the_first_row_in_plain_words() {
        let groups = groups();
        let (keys, words) = &groups[0].1[0];
        assert_eq!(words, "Move focus");
        for key in ["Super+J", "Super+K", "Alt+Tab"] {
            assert!(keys.split(" / ").any(|k| k == key), "{key} in {keys}");
        }
    }

    #[test]
    fn edit_settings_stands_beside_the_reload_key() {
        let groups = groups();
        let system = &groups[groups.len() - 1].1;
        let at = system
            .iter()
            .position(|(_, words)| words == "Edit settings")
            .expect("a row Edit settings in the system group");
        assert_eq!(system[at].0, "Super+S");
        assert_eq!(
            system[at + 1],
            (
                "Super+Ctrl+R".to_owned(),
                "Read the settings again".to_owned()
            )
        );
    }

    #[test]
    fn the_give_keys_have_a_group_of_their_own_next_to_the_windows() {
        let groups = groups();
        assert_eq!(groups[1].0, "Other looks");
        assert_eq!(
            groups[1].1,
            [(
                "Super+Alt+1-4".to_owned(),
                "Give window to a look".to_owned()
            )]
        );
        let de = rows(&parse::<Config>(DEFAULT).unwrap(), Language::De);
        assert_eq!(de[1].0, "Andere Fenstermanager");
    }

    fn words_of(commands: &[Command]) -> Vec<String> {
        commands
            .iter()
            .filter_map(|c| match c {
                Command::Create { props, .. } => props.text.clone(),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_list_has_a_mouse_line_under_its_header() {
        let colors = Colors::of(Scheme::Ink);
        let config = parse::<Config>(DEFAULT).unwrap();
        let drawn = |lang| {
            let rows = rows(&config, lang);
            words_of(&key_list("screen:X", [1280.0, 800.0], &rows, &colors, lang))
        };
        let en = drawn(Language::En);
        assert!(en.contains(&"Mouse".to_owned()), "{en:?}");
        let line = en.iter().find(|w| w.starts_with("Apps")).expect("a line");
        for part in ["Apps", "click a tag", "Super+drag", "power icon"] {
            assert!(line.contains(part), "{part} in {line}");
        }
        let de = drawn(Language::De);
        assert!(de.contains(&"Maus".to_owned()), "{de:?}");
    }

    #[test]
    fn the_mouse_line_of_each_language_counts_for_the_width_of_the_panel() {
        let colors = Colors::of(Scheme::Ink);
        for (lang, start) in [(Language::En, "Apps"), (Language::De, "Apps")] {
            let commands = header(&format!("{KEYS}-body"), &colors, lang);
            let line = commands
                .iter()
                .find_map(|c| match c {
                    Command::Create { id, props, .. }
                        if props.text.as_deref().is_some_and(|t| t.starts_with(start)) =>
                    {
                        Some(id.clone())
                    }
                    _ => None,
                })
                .expect("a line");
            assert!(sets_width(&line), "{line}");
        }
        let wide = fit_list([1280.0, 800.0], 640.0, 900.0);
        let Command::Update { props, .. } = &wide[0] else {
            panic!("no update of the panel");
        };
        assert_eq!(props.size, Some([924.0, 640.0]));
    }

    #[test]
    fn the_volume_keys_have_the_caps_and_the_words_of_every_look() {
        let rows: Vec<(String, String)> = groups().into_iter().flat_map(|(_, rows)| rows).collect();
        let row = (
            "Vol+ / Vol- / Mute".to_owned(),
            "Volume up / Volume down / Mute".to_owned(),
        );
        assert!(rows.contains(&row), "{row:?} is no row of {rows:?}");
    }

    #[test]
    fn the_keys_are_words() {
        let all: String = groups()
            .iter()
            .flat_map(|(_, rows)| rows.iter().map(|(k, _)| k.clone()))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(!all.contains("BTN_LEFT") && !all.contains("slash"), "{all}");
        assert!(
            all.contains("Super+drag") && all.contains("Super+/"),
            "{all}"
        );
    }

    fn texts(commands: &[Command]) -> Vec<(String, Option<String>, Option<f64>)> {
        commands
            .iter()
            .filter_map(|c| match c {
                Command::Create {
                    id,
                    kind: NodeKind::Text,
                    props,
                    ..
                } => Some((id.clone(), props.family.clone(), props.font_size)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_overlays_use_inter_and_one_monospace_family() {
        let colors = Colors::of(Scheme::Ink);
        let list = key_list(
            "screen:X",
            [1280.0, 800.0],
            &groups(),
            &colors,
            Language::En,
        );
        let chip = chip("screen:X", [1280.0, 800.0], "Tile", &colors);
        for (id, family, _) in texts(&list).into_iter().chain(texts(&chip)) {
            let want = if id.ends_with("-k") {
                MONO_FONT
            } else {
                UI_FONT
            };
            assert_eq!(family.as_deref(), Some(want), "{id}");
        }
    }

    #[test]
    fn the_scrim_is_tinted_from_the_base_token_of_each_theme() {
        for scheme in [Scheme::Ink, Scheme::Paper] {
            let colors = Colors::of(scheme);
            let list = key_list(
                "screen:X",
                [1280.0, 800.0],
                &groups(),
                &colors,
                Language::En,
            );
            let color = list
                .iter()
                .find_map(|c| match c {
                    Command::Create { id, props, .. } if id == "tiling-keys-scrim" => props.color,
                    _ => None,
                })
                .expect("a scrim");
            let [r, g, b] = channels(&colors.base);
            assert_eq!(color, [r, g, b, SCRIM], "{scheme:?}");
        }
    }

    #[test]
    fn the_chip_stands_in_the_middle_of_the_output() {
        let (at, size, _) = chip_places([1280.0, 800.0], [68.0, 24.0]);
        assert!(((at[0] + size[0] / 2.0) - 640.0).abs() < 1e-9);
        assert!(((at[1] + size[1] / 2.0) - 400.0).abs() <= 0.5);
    }

    #[test]
    fn the_word_sits_in_the_middle_of_the_chip() {
        for measured in [[29.0, 24.0], [68.0, 24.0], [140.0, 30.0]] {
            let (at, size, word) = chip_places([1280.0, 800.0], measured);
            let centre = |a: [f64; 2], s: [f64; 2]| [a[0] + s[0] / 2.0, a[1] + s[1] / 2.0];
            let (c, w) = (centre(at, size), centre(word, measured));
            assert!((c[0] - w[0]).abs() < 1e-9 && (c[1] - w[1]).abs() < 1e-9);
            assert_eq!(word[0] - at[0], 24.0);
            assert_eq!(word[1] - at[1], 16.0);
        }
    }
}
