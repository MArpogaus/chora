//! The words of the actions and the two overlays that chora-tiling draws
//! itself: the key list and the layout chip. Both sit under the screen
//! root of an output, as glass: a rect with a backdrop blur.

use std::collections::BTreeMap;

use chora_kit::words::Language;
use chora_scene::{Backdrop, Border, Command, NodeKind, PropsUpdate, Transform};

use crate::{
    config::{Action, Config},
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
const TOP: f64 = 88.0;
/// The space below the last row.
const BOTTOM: f64 = 24.0;
/// The space left of the first column, and the distance of the columns.
const LEFT: f64 = 24.0;
const COLUMN: f64 = 416.0;
/// The pitches of the rows that the panel tries, in order; all on the 4 px
/// grid. The caps are 4 px less high.
const PITCHES: [f64; 2] = [24.0, 20.0];
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

/// The group of the key list that an action belongs to, and its words;
/// a `{}` stands for the number of a tag.
fn words(action: &Action, de: bool) -> Option<(usize, String)> {
    let pick = |en: &str, german: &str| (if de { german } else { en }).to_owned();
    if let Action::Run(command) = action {
        return Some((3, command.clone()));
    }
    if let Action::Give(to) = action {
        return Some((
            4,
            format!("{} {to}", pick("Give window to", "Fenster geben an")),
        ));
    }
    let window = window_words(action).map(|w| (0, w));
    let layout = layout_words(action).map(|w| (1, w));
    let tag = tag_words(action).map(|w| (2, w));
    let system = system_words(action).map(|w| (4, w));
    let (group, [en, german]) = window.or(layout).or(tag).or(system)?;
    Some((group, pick(en, german)))
}

type Words = [&'static str; 2];

fn window_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::FocusNext => ["Next window", "Nächstes Fenster"],
        Action::FocusPrevious => ["Previous window", "Voriges Fenster"],
        Action::FocusBack => ["Window before", "Fenster davor"],
        Action::FocusUrgent => ["Window that asks", "Fenster, das ruft"],
        Action::SwapNext => ["Move window down", "Fenster nach hinten"],
        Action::SwapPrevious => ["Move window up", "Fenster nach vorn"],
        Action::MoveToMaster => ["Make window main", "Fenster zum Hauptfenster"],
        Action::Close => ["Close window", "Fenster schließen"],
        Action::ToggleFloating => ["Float window", "Fenster schweben lassen"],
        Action::ToggleFullscreen => ["Full screen", "Vollbild"],
        Action::ToggleMaximized => ["Maximize", "Maximieren"],
        Action::ToggleMaximizedHorizontal => ["Maximize wide", "Breit maximieren"],
        Action::ToggleMaximizedVertical => ["Maximize tall", "Hoch maximieren"],
        Action::ToggleOnTop => ["Keep on top", "Immer oben"],
        Action::Minimize => ["Hide window", "Fenster verstecken"],
        Action::RestoreMinimized => ["Bring back window", "Fenster zurückholen"],
        Action::Drag => ["Drag window", "Fenster ziehen"],
        _ => return None,
    })
}

fn layout_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::IncMasterWidth(s) if *s >= 0.0 => ["Wider main area", "Hauptbereich breiter"],
        Action::IncMasterWidth(_) => ["Narrower main area", "Hauptbereich schmaler"],
        Action::IncMasters(s) if *s >= 0 => ["More main windows", "Mehr Hauptfenster"],
        Action::IncMasters(_) => ["Fewer main windows", "Weniger Hauptfenster"],
        Action::IncColumns(s) if *s >= 0 => ["More columns", "Mehr Spalten"],
        Action::IncColumns(_) => ["Fewer columns", "Weniger Spalten"],
        Action::NextLayout => ["Next layout", "Nächste Anordnung"],
        Action::PreviousLayout => ["Previous layout", "Vorige Anordnung"],
        Action::FocusNextOutput => ["Next screen", "Nächster Bildschirm"],
        Action::FocusPreviousOutput => ["Previous screen", "Voriger Bildschirm"],
        Action::MoveToNextOutput => ["Window to next screen", "Fenster auf nächsten Bildschirm"],
        _ => return None,
    })
}

fn tag_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::ViewTag(_) => ["Show tag {}", "Tag {} zeigen"],
        Action::ToggleTagView(_) => ["Add or remove tag {}", "Tag {} dazu oder weg"],
        Action::MoveToTag(_) => ["Window to tag {}", "Fenster auf Tag {}"],
        Action::ToggleWindowTag(_) => ["Window also on tag {}", "Fenster auch auf Tag {}"],
        Action::ViewNext => ["Next tag", "Nächster Tag"],
        Action::ViewPrevious => ["Previous tag", "Voriger Tag"],
        Action::ViewBack => ["Tag before", "Tag davor"],
        _ => return None,
    })
}

fn system_words(action: &Action) -> Option<Words> {
    Some(match action {
        Action::Reload => ["Read the settings again", "Einstellungen neu lesen"],
        Action::Volume(_) => ["Volume", "Lautstärke"],
        Action::ShowKeys => ["Show this list", "Diese Liste zeigen"],
        _ => return None,
    })
}

/// The title of a group of the key list.
fn title(group: usize, de: bool) -> &'static str {
    let names = [
        ["Windows", "Fenster"],
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
        Action::ViewTag(n)
        | Action::ToggleTagView(n)
        | Action::MoveToTag(n)
        | Action::ToggleWindowTag(n) => Some(*n),
        _ => None,
    }
}

/// A row before it is sorted: its rank, its keys and its words.
type Entry = (usize, String, String);

/// The English words of the rows in the order that the key list shows
/// them, by group; a row that is not here follows the others, by its keys.
const ORDER: [&str; 44] = [
    "Next window",
    "Previous window",
    "Window before",
    "Window that asks",
    "Move window down",
    "Move window up",
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
    "Terminal",
    "Run a program",
    "Chat with a helper",
    "Lock the screen",
    "Power menu",
    "Screenshot",
    "Screenshot of an area",
    "Read the settings again",
    "Show this list",
];

/// The keys as a person says them: `BTN_LEFT` is a drag, `slash` is `/`.
fn spell(keys: &str) -> String {
    keys.split('+')
        .map(|key| match key {
            "BTN_LEFT" => "drag",
            "slash" => "/",
            "Return" => "Enter",
            "Escape" => "Esc",
            other => other,
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// The rows of the key list by group: the keys as the file spells them
/// with `MOD` as the modifier, and the words. A `labels` entry names a
/// binding; the bindings of the tags 1 to 9 take one row.
#[must_use]
pub fn rows(config: &Config, lang: Language) -> Vec<(&'static str, Vec<(String, String)>)> {
    let de = lang == Language::De;
    let mut groups: Vec<Vec<Entry>> = vec![Vec::new(); 5];
    let mut spans: BTreeMap<(usize, usize, String, String), (usize, usize)> = BTreeMap::new();
    for (binding, action) in &config.bindings {
        let Some((group, label)) = words(action, de) else {
            continue;
        };
        if !binding.contains("MOD") {
            continue;
        }
        let english = words(action, false).map_or_else(String::new, |(_, en)| en);
        let named = config.labels.get(binding);
        let rank = ORDER
            .iter()
            .position(|w| *w == named.unwrap_or(&english))
            .unwrap_or(ORDER.len());
        let label = named.cloned().unwrap_or(label);
        let keys = spell(&binding.replace("MOD", &config.modifier));
        let Some(n) = number(action) else {
            match groups[group].iter_mut().find(|(_, _, l)| *l == label) {
                Some((_, known, _)) => *known = format!("{known} / {keys}"),
                None => groups[group].push((rank, keys, label)),
            }
            continue;
        };
        let stem = keys
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_owned();
        let span = spans.entry((group, rank, stem, label)).or_insert((n, n));
        (span.0, span.1) = (span.0.min(n), span.1.max(n));
    }
    for ((group, rank, stem, label), (low, high)) in spans {
        let range = if low == high {
            low.to_string()
        } else {
            format!("{low}-{high}")
        };
        let keys = format!("{stem}{range}");
        groups[group].push((rank, keys, label.replace("{}", &range)));
    }
    let named = groups.into_iter().enumerate();
    named
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
            color: Some(color(channels(&colors.surface), 0.92)),
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
    let height = TOP + tallest(&all, split, pitch) + BOTTOM;
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
            for (keys, label) in rows {
                items.push(Item::Cap {
                    at: [x, y + 2.0],
                    size: [cap_width(keys), pitch - 4.0],
                    keys: keys.clone(),
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
        color: Some([0.0, 0.0, 0.0, 0.25]),
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
            Item::Cap { at, size, keys } => {
                let keycap = PropsUpdate {
                    size: Some(*size),
                    color: Some(color(channels(&colors.surface_2), 1.0)),
                    radius: Some(6.0),
                    transform: Some(Transform::translate(at[0], at[1], 0.0)),
                    ..PropsUpdate::default()
                };
                commands.push(create(&format!("{id}-cap"), NodeKind::Rect, body, keycap));
                let mut mono = Face::new([at[0] + CAP_PAD, at[1] + 2.0], 13.0, 400, text_rgb);
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
    let (name, hint) = if de {
        (
            "Tasten",
            "Super+/ zeigt diese Liste wieder. Esc schließt sie.",
        )
    } else {
        ("Keys", "Super+/ shows this list again. Esc closes it.")
    };
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
            &Face::new([24.0, 48.0], 14.0, 400, channels(&colors.muted)),
        ),
    ]
}

/// The padding of the layout chip around its word, left and right, and
/// above and below.
const CHIP_PAD: [f64; 2] = [24.0, 16.0];

/// Where the chip and its word go for a word of the size `measured`, which
/// the renderer reports: the box of the chip, as big as the word and its
/// padding and centred low on the output, and the place of the word in the
/// middle of it, on both axes.
#[must_use]
pub fn chip_places(output: [f64; 2], measured: [f64; 2]) -> ([f64; 2], [f64; 2], [f64; 2]) {
    let size = [
        measured[0] + 2.0 * CHIP_PAD[0],
        measured[1] + 2.0 * CHIP_PAD[1],
    ];
    let at = [(output[0] - size[0]) / 2.0, output[1] - size[1] - 96.0];
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
        for (room, cap) in [(1000.0, 20.0), (696.0, 16.0)] {
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
    fn a_keycap_has_the_same_padding_left_and_right() {
        for item in laid_out(1000.0).items {
            if let Item::Cap { size, keys, .. } = item {
                let text = px(keys.chars().count()) * MONO_ADVANCE;
                assert!((size[0] - text - 2.0 * CAP_PAD).abs() < 1e-9, "{keys}");
            }
        }
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
    fn the_rows_follow_the_order_of_a_person_not_of_the_keys() {
        let groups = groups();
        let names: Vec<&str> = groups.iter().map(|(n, _)| *n).collect();
        assert_eq!(names, ["Windows", "Layouts", "Tags", "Programs", "System"]);
        let first = |g: usize| groups[g].1[0].1.clone();
        assert_eq!(first(0), "Next window");
        assert_eq!(first(1), "Next layout");
        assert_eq!(first(2), "Show tag 1-9");
        assert_eq!(first(3), "Terminal");
        assert_eq!(groups[0].1[1].1, "Previous window");
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
