//! The words of the actions and the two overlays that chora-tiling draws
//! itself: the key list and the layout chip. Both sit under the screen
//! root of an output, as glass: a rect with a backdrop blur.

use std::collections::BTreeMap;

use chora_kit::words::Language;
use chora_scene::{Backdrop, Border, Command, NodeKind, PropsUpdate, Transform};

use crate::{
    config::{Action, Config, Palette},
    layout::Layout,
};

/// The group of the key list, and of the layout chip.
pub const KEYS: &str = "tiling-keys";
pub const CHIP: &str = "tiling-chip";

/// The panel of the key list: its width and its least height.
const PANEL: [f64; 2] = [880.0, 560.0];
const ROW: f64 = 20.0;

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

/// The rows of the key list by group: the keys as the file spells them
/// with `MOD` as the modifier, and the words. A `labels` entry names a
/// binding; the bindings of the tags 1 to 9 take one row.
#[must_use]
pub fn rows(config: &Config, lang: Language) -> Vec<(&'static str, Vec<(String, String)>)> {
    let de = lang == Language::De;
    let mut groups: Vec<Vec<(String, String)>> = vec![Vec::new(); 5];
    let mut spans: BTreeMap<(usize, String, String), (usize, usize)> = BTreeMap::new();
    for (binding, action) in &config.bindings {
        let Some((group, label)) = words(action, de) else {
            continue;
        };
        if !binding.contains("MOD") {
            continue;
        }
        let label = config.labels.get(binding).cloned().unwrap_or(label);
        let keys = binding.replace("MOD", &config.modifier);
        let Some(n) = number(action) else {
            match groups[group].iter_mut().find(|(_, l)| *l == label) {
                Some((known, _)) => *known = format!("{known} / {keys}"),
                None => groups[group].push((keys, label)),
            }
            continue;
        };
        let stem = keys
            .trim_end_matches(|c: char| c.is_ascii_digit())
            .to_owned();
        let span = spans.entry((group, stem, label)).or_insert((n, n));
        (span.0, span.1) = (span.0.min(n), span.1.max(n));
    }
    for ((group, stem, label), (low, high)) in spans {
        let range = if low == high {
            low.to_string()
        } else {
            format!("{low}-{high}")
        };
        groups[group].push((format!("{stem}{range}"), label.replace("{}", &range)));
    }
    let named = groups.into_iter().enumerate();
    named
        .filter(|(_, rows)| !rows.is_empty())
        .map(|(group, rows)| (title(group, de), rows))
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
            family: face
                .mono
                .then(|| "JetBrains Mono, DejaVu Sans Mono, monospace".to_owned()),
            color: Some(face.rgba),
            ..PropsUpdate::default()
        },
    )
}

/// The `surface`, `text`, `muted` and keycap colours of Ink, which the
/// overlays use.
const SURFACE: [f64; 3] = [0.102, 0.114, 0.153];
const TEXT: [f64; 3] = [0.902, 0.910, 0.937];
const MUTED: [f64; 3] = [0.604, 0.631, 0.706];
const KEYCAP: [f64; 3] = [0.141, 0.157, 0.212];

/// A glass rect: the surface colour, a hairline in the line colour and a
/// backdrop blur, at a place.
fn glass(id: &str, parent: &str, at: [f64; 2], size: [f64; 2], line: [f64; 3]) -> Command {
    create(
        id,
        NodeKind::Rect,
        parent,
        PropsUpdate {
            transform: Some(Transform::translate(at[0], at[1], 0.0)),
            size: Some(size),
            color: Some(color(SURFACE, 0.82)),
            radius: Some(14.0),
            border: Some(Some(Border {
                width: 1.0,
                color: color(line, 1.0),
            })),
            backdrop: Some(Some(Backdrop {
                blur: 24.0,
                saturation: 1.2,
            })),
            ..PropsUpdate::default()
        },
    )
}

/// One line of the flow of the key list: a group title, or keys and words.
enum Line<'a> {
    Title(&'a str),
    Row(&'a str, &'a str),
}

/// The commands that draw the key list centred on an output of the given
/// size, under its screen root `parent`.
#[must_use]
pub fn key_list(
    parent: &str,
    output: [f64; 2],
    groups: &[(&str, Vec<(String, String)>)],
    palette: &Palette,
    lang: Language,
) -> Vec<Command> {
    let line = palette.rgb().unwrap_or_default()[2];
    let mut flow: Vec<Line> = Vec::new();
    for (name, rows) in groups {
        flow.push(Line::Title(name));
        flow.extend(rows.iter().map(|(k, l)| Line::Row(k, l)));
    }
    let per_column = (flow.len() + 1).div_ceil(2);
    let wanted = 96.0 + ROW * px(per_column + 1) + 24.0;
    let height = PANEL[1].max(wanted).min(output[1] - 24.0);
    let origin = [(output[0] - PANEL[0]) / 2.0, (output[1] - height) / 2.0];
    let scrim = PropsUpdate {
        size: Some(output),
        color: Some([0.0, 0.0, 0.0, 0.25]),
        ..PropsUpdate::default()
    };
    let body = format!("{KEYS}-body");
    let mut commands = vec![
        create(KEYS, NodeKind::Group, parent, PropsUpdate::default()),
        create(&format!("{KEYS}-scrim"), NodeKind::Rect, KEYS, scrim),
        glass(
            &format!("{KEYS}-panel"),
            KEYS,
            origin,
            [PANEL[0], height],
            line,
        ),
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
    commands.extend(header(&body, lang));
    commands.extend(rows_of(&body, &flow, per_column));
    commands
}

fn header(body: &str, lang: Language) -> Vec<Command> {
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
            &Face::new([24.0, 20.0], 20.0, 600, TEXT),
        ),
        text(
            &format!("{body}-h"),
            body,
            hint,
            &Face::new([24.0, 48.0], 14.0, 400, MUTED),
        ),
    ]
}

/// The nodes of the lines in two columns; a group that a column break
/// splits gets its title again at the top of the second column.
fn rows_of(body: &str, flow: &[Line], per_column: usize) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut group = "";
    for (n, entry) in flow.iter().enumerate() {
        let (column, at) = (n / per_column, n % per_column);
        let x = 24.0 + 416.0 * px(column);
        let mut y = 88.0 + ROW * px(at);
        let id = format!("{body}-{column}-{at}");
        let (keys, label) = match entry {
            Line::Title(name) => {
                group = name;
                let face = Face::new([x, y], 14.0, 600, MUTED);
                commands.push(text(&id, body, name, &face));
                continue;
            }
            Line::Row(keys, label) => (*keys, *label),
        };
        if at == 0 && column > 0 {
            let face = Face::new([x, y], 14.0, 600, MUTED);
            commands.push(text(&format!("{id}-g"), body, group, &face));
            y += ROW;
        }
        let cap = px(keys.chars().count()) * 7.9 + 12.0;
        let keycap = PropsUpdate {
            size: Some([cap, ROW - 2.0]),
            color: Some(color(KEYCAP, 1.0)),
            radius: Some(5.0),
            transform: Some(Transform::translate(x, y, 0.0)),
            ..PropsUpdate::default()
        };
        commands.push(create(&format!("{id}-cap"), NodeKind::Rect, body, keycap));
        let mut mono = Face::new([x + 6.0, y + 1.0], 13.0, 400, TEXT);
        mono.mono = true;
        commands.push(text(&format!("{id}-k"), body, keys, &mono));
        let face = Face::new([x + cap + 10.0, y], 14.0, 400, TEXT);
        commands.push(text(&format!("{id}-l"), body, label, &face));
    }
    commands
}

/// The commands that draw the layout chip, centred low on an output of the
/// given size, under its screen root `parent`.
#[must_use]
pub fn chip(parent: &str, output: [f64; 2], word: &str, palette: &Palette) -> Vec<Command> {
    let line = palette.rgb().unwrap_or_default()[2];
    let size = [px(word.chars().count()) * 12.0 + 56.0, 56.0];
    let at = [(output[0] - size[0]) / 2.0, output[1] - size[1] - 96.0];
    let face = Face::new([at[0] + 28.0, at[1] + 14.0], 20.0, 600, TEXT);
    vec![
        create(CHIP, NodeKind::Group, parent, PropsUpdate::default()),
        glass(&format!("{CHIP}-glass"), CHIP, at, size, line),
        text(&format!("{CHIP}-word"), CHIP, word, &face),
    ]
}
