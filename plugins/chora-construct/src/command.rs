//! The command surfaces of the construct HUD (spec, sections 5 and 7):
//! the window list and the help, each a strong glass panel in the centre
//! over a scrim, with a search line in `display` that filters as the
//! person types, a divider, the rows and a footer of keys. A chosen row
//! has the selection fill; the row under the pointer the hover fill.
#![allow(
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::single_match_else,
    reason = "builders of nodes: x, y, w, h of shapes, one node after the other"
)]

use chora_kit::glass::{INNER_RADIUS, PANEL_RADIUS, ROW, Type, UNIT, caps};
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform, Uniform};

use crate::{
    glass::{Surface, face, group, keycaps, keycaps_width, rect, surface, text},
    hud::Look,
};

/// The body of the window list in its group, made anew on each change.
pub const LIST_BODY: &str = "construct-list-body";
/// The prefix of a row of the list; a click chooses it.
pub const LIST_ROW: &str = "construct-list-row-";
/// The scrim behind the window list.
pub const SCRIM: &str = "construct-list-scrim";
/// The width of the list, its top, and of the help with and without a
/// search, and its top.
pub const LIST_WIDTH: f64 = 560.0;
const LIST_TOP: f64 = 104.0;
const HELP_WIDE: f64 = 880.0;
const HELP_NARROW: f64 = 560.0;
const HELP_TOP: f64 = 64.0;
/// The search line, the footer and a row of the help.
const SEARCH: f64 = 56.0;
const FOOTER: f64 = 48.0;
const KEY_ROW: f64 = 32.0;
/// The thumbnail of a window in a row.
const THUMB: [f64; 2] = [64.0, 40.0];

/// A window in the list: its name, its note, and its surface with its
/// size for a thumbnail while its pane shows.
pub struct WindowRow {
    pub name: String,
    pub note: String,
    pub thumb: Option<(String, [f64; 2])>,
}

/// What the list shows.
pub struct ListModel<'a> {
    pub rows: &'a [WindowRow],
    pub sel: usize,
    pub hovered: Option<usize>,
    pub query: &'a str,
    pub count: String,
    pub placeholder: String,
    pub footer: [(Vec<String>, String); 3],
    pub thumb_shader: Option<&'a str>,
}

/// The tint of a command surface: nothing of the space reads through.
const COMMAND_TINT: f64 = 0.92;
/// The alpha of the scrim behind a command surface, light and dark.
const SCRIM_LIGHT: f64 = 0.48;
const SCRIM_DARK: f64 = 0.56;

/// The glass of a command surface (polish 4): the strong tint at 0.92
/// over a scrim of 0.48 light or 0.56 dark.
fn command_glass(look: &Look) -> chora_kit::glass::Material {
    let mut m = look.glass;
    let dark = m.tint[0] < 0.5;
    m.strong[3] = COMMAND_TINT;
    m.scrim[3] = if dark { SCRIM_DARK } else { SCRIM_LIGHT };
    m
}

/// The scrim over the whole output, which lets a click through to the
/// space behind it, where it closes the surface.
#[must_use]
pub fn scrim(look: &Look, (id, parent): (&str, &str), out: [f64; 2]) -> Command {
    let mut scrim = rect(
        id,
        parent,
        [0.0, 0.0, out[0], out[1]],
        0.0,
        command_glass(look).scrim,
    );
    if let Command::Create { props, .. } = &mut scrim {
        props.meta = Some([("surface".into(), "backdrop".into())].into());
    }
    scrim
}

/// The search line at `y` of a panel `w` wide: the glass, the text or
/// its placeholder with the caret, and the count or the scope at the
/// right.
fn search(
    look: &Look,
    parent: &str,
    (query, placeholder, scope): (&str, &str, &str),
    w: f64,
) -> Vec<Command> {
    let m = &look.glass;
    let display = face(Type::Display, 300);
    let (shown, color) = if query.is_empty() {
        (placeholder, m.muted)
    } else {
        (query, m.text)
    };
    let x = UNIT + 16.0;
    let mut commands = icon(
        look,
        &format!("{parent}-icon"),
        parent,
        [x, UNIT + (SEARCH - 20.0) / 2.0],
    );
    commands.push(text(
        look,
        &format!("{parent}-query"),
        parent,
        (display, shown),
        [x + 32.0, UNIT + 12.0],
        color,
    ));
    if !query.is_empty() {
        let end = look.size(display, query, None)[0];
        commands.push(rect(
            &format!("{parent}-caret"),
            parent,
            [x + 32.0 + end + 2.0, UNIT + 16.0, 2.0, 24.0],
            1.0,
            m.accent,
        ));
    }
    let label = face(Type::Label, 500);
    let scope_w = look.size(label, scope, None)[0];
    commands.push(text(
        look,
        &format!("{parent}-scope"),
        parent,
        (label, scope),
        [(w - UNIT - 16.0 - scope_w).round(), UNIT + 20.0],
        m.muted,
    ));
    commands.push(rect(
        &format!("{parent}-divider"),
        parent,
        [2.0 * UNIT, UNIT + SEARCH, w - 4.0 * UNIT, 1.0],
        0.0,
        m.separator,
    ));
    commands
}

/// The magnifier of a search line, 20 square at `[x, y]`: a ring of 11
/// and a handle at 45 degrees, as in the mockups.
fn icon(look: &Look, id: &str, parent: &str, [x, y]: [f64; 2]) -> Vec<Command> {
    let m = &look.glass;
    let mut ring = rect(
        &format!("{id}-ring"),
        parent,
        [x + 3.0, y + 3.0, 11.0, 11.0],
        5.5,
        [0.0; 4],
    );
    if let Command::Create { props, .. } = &mut ring {
        props.border = Some(Some(chora_scene::Border {
            width: 1.6,
            color: m.muted,
        }));
    }
    let mut handle = rect(
        &format!("{id}-handle"),
        parent,
        [0.0, 0.0, 6.4, 1.6],
        0.8,
        m.muted,
    );
    if let Command::Create { props, .. } = &mut handle {
        let turn = Transform::translate(x + 13.0, y + 12.4, 0.0).then(&Transform::rotate_z(45.0));
        props.transform = Some(turn);
    }
    vec![ring, handle]
}

/// The footer at `y` of a panel `w` wide: hints of keys at the left, one
/// at the right.
fn footer(
    look: &Look,
    parent: &str,
    items: &[(Vec<String>, String)],
    y: f64,
    w: f64,
) -> Vec<Command> {
    let m = &look.glass;
    let small = face(Type::Small, 400);
    let mut commands = Vec::new();
    let top = y + (FOOTER - 24.0) / 2.0;
    let mut x = UNIT + 16.0;
    let last = items.len().saturating_sub(1);
    for (k, (keys, word)) in items.iter().enumerate() {
        let word_w = look.size(small, word, None)[0];
        let caps_w = keycaps_width(look, keys);
        let gap = if keys.is_empty() { 0.0 } else { 6.0 };
        if k == last {
            x = (w - UNIT - 16.0 - caps_w - gap - word_w).round();
        }
        let (caps, used) = keycaps(
            look,
            m,
            (&format!("{parent}-foot-{k}"), parent),
            keys,
            [x, top],
        );
        commands.extend(caps);
        commands.push(text(
            look,
            &format!("{parent}-foot-{k}-word"),
            parent,
            (small, word),
            [x + used + gap, top + 2.0],
            m.muted,
        ));
        x += used + gap + word_w.ceil() + 16.0;
    }
    commands
}

/// The window list over the scrim, made anew: the group of its body in
/// the group `parent` of the list.
#[must_use]
pub fn list(look: &Look, model: &ListModel<'_>, parent: &str, out: [f64; 2]) -> Vec<Command> {
    let m = &command_glass(look);
    let rows = model.rows.len().max(1);
    #[allow(clippy::cast_precision_loss, reason = "a dozen rows")]
    let rows_h = rows as f64 * ROW + (rows - 1) as f64 * 2.0;
    let h = UNIT + SEARCH + 1.0 + UNIT + rows_h + FOOTER;
    let x = ((out[0] - LIST_WIDTH) / 2.0).round();
    let y = LIST_TOP.min((out[1] - h) / 2.0).max(UNIT).round();
    let mut commands = vec![scrim(look, (SCRIM, parent), out)];
    commands.extend(surface(
        look,
        m,
        &Surface {
            id: LIST_BODY,
            parent,
            rect: [x, y, LIST_WIDTH, h],
            radius: PANEL_RADIUS,
            strong: true,
            name: "window-list",
            input: InputMode::Accept,
        },
    ));
    commands.extend(search(
        look,
        LIST_BODY,
        (model.query, &model.placeholder, &model.count),
        LIST_WIDTH,
    ));
    let top = UNIT + SEARCH + 1.0 + UNIT;
    for (k, row) in model.rows.iter().enumerate() {
        #[allow(clippy::cast_precision_loss, reason = "a dozen rows")]
        let ry = top + k as f64 * (ROW + 2.0);
        commands.extend(window_row(look, model, (k, row), ry));
    }
    commands.extend(footer(
        look,
        LIST_BODY,
        &model.footer,
        top + rows_h,
        LIST_WIDTH,
    ));
    commands
}

/// One row of the list at `y` in the body.
fn window_row(
    look: &Look,
    model: &ListModel<'_>,
    (k, row): (usize, &WindowRow),
    y: f64,
) -> Vec<Command> {
    let m = &look.glass;
    let fill = if k == model.sel {
        m.select
    } else if model.hovered == Some(k) {
        m.hover
    } else {
        [0.0; 4]
    };
    let id = format!("{LIST_ROW}{k}");
    let w = LIST_WIDTH - 2.0 * UNIT;
    let mut hit = rect(&id, LIST_BODY, [UNIT, y, w, ROW], INNER_RADIUS, fill);
    if let Command::Create { props, .. } = &mut hit {
        props.input = Some(InputMode::Accept);
    }
    let mut commands = vec![hit];
    let thumb = [2.0 * UNIT, y + (ROW - THUMB[1]) / 2.0];
    commands.extend(thumbnail(look, &id, row, thumb, model.thumb_shader));
    let data = face(Type::Data, 400);
    let note_w = look.size(data, &row.note, None)[0];
    let name_x = thumb[0] + THUMB[0] + 12.0;
    let room = w - (name_x - UNIT) - 12.0 - note_w - 16.0;
    let name = look.one_line(face(Type::Text, 400), &row.name, room);
    commands.push(text(
        look,
        &format!("{id}-name"),
        LIST_BODY,
        (face(Type::Text, 400), &name),
        [name_x, y + 16.0],
        m.text,
    ));
    if !row.note.is_empty() {
        commands.push(text(
            look,
            &format!("{id}-note"),
            LIST_BODY,
            (data, &row.note),
            [(UNIT + w - 12.0 - note_w).round(), y + 18.0],
            m.muted,
        ));
    }
    commands
}

/// The thumbnail of a window: its picture, cut to 64 by 40 with corners of
/// 8, while its pane shows; else a well with its first letter.
fn thumbnail(
    look: &Look,
    id: &str,
    row: &WindowRow,
    [x, y]: [f64; 2],
    shader: Option<&str>,
) -> Vec<Command> {
    let m = &look.glass;
    let well = rect(
        &format!("{id}-thumb"),
        LIST_BODY,
        [x, y, THUMB[0], THUMB[1]],
        8.0,
        m.well,
    );
    match (shader.filter(|_| !look.plain), &row.thumb) {
        (Some(shader), Some((surface, size))) => {
            let (sa, ta) = (size[0] / size[1].max(1.0), THUMB[0] / THUMB[1]);
            let crop = if sa > ta {
                [ta / sa, 1.0]
            } else {
                [1.0, sa / ta]
            };
            let uniforms = [
                (
                    "win".to_owned(),
                    Uniform::Texture(chora_scene::NodeTexture {
                        node: surface.clone(),
                    }),
                ),
                (
                    "box".to_owned(),
                    Uniform::Vector(vec![THUMB[0], THUMB[1], 8.0, 0.0]),
                ),
                ("crop".to_owned(), Uniform::Vector(crop.to_vec())),
            ];
            let mut edge = rect(
                &format!("{id}-thumb-edge"),
                LIST_BODY,
                [x, y, THUMB[0], THUMB[1]],
                8.0,
                [0.0; 4],
            );
            if let Command::Create { props, .. } = &mut edge {
                props.border = Some(Some(chora_scene::Border {
                    width: 1.0,
                    color: m.cap_edge,
                }));
            }
            vec![
                well,
                Command::Create {
                    id: format!("{id}-thumb-picture"),
                    kind: NodeKind::Shader,
                    parent: LIST_BODY.into(),
                    props: PropsUpdate {
                        shader: Some(shader.into()),
                        transform: Some(Transform::translate(x, y, 0.0)),
                        size: Some(THUMB),
                        uniforms: Some(uniforms.into_iter().collect()),
                        input: Some(InputMode::Pass),
                        ..PropsUpdate::default()
                    },
                },
                edge,
            ]
        }
        _ => {
            let letter: String = row
                .name
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect())
                .unwrap_or_default();
            let small = face(Type::Small, 500);
            let lw = look.size(small, &letter, None)[0];
            vec![
                well,
                text(
                    look,
                    &format!("{id}-letter"),
                    LIST_BODY,
                    (small, &letter),
                    [(x + (THUMB[0] - lw) / 2.0).round(), y + 10.0],
                    m.muted,
                ),
            ]
        }
    }
}

/// A row of the help: what it does, its keys and, after `or`, other keys.
pub struct KeyRow {
    pub what: String,
    pub keys: Vec<String>,
    pub alt: Vec<String>,
    /// A pointer gesture after the keys, such as `drag`.
    pub gesture: Option<String>,
}

/// A group of rows: its head and whether every look shares it.
pub struct KeyGroup {
    pub head: String,
    pub shared: bool,
    pub rows: Vec<KeyRow>,
}

/// What the help shows.
pub struct HelpModel<'a> {
    pub groups: &'a [KeyGroup],
    pub query: &'a str,
    pub sel: usize,
    pub scope: String,
    pub placeholder: String,
    pub every_look: String,
    pub or: String,
    pub note: String,
    pub footer: Vec<(Vec<String>, String)>,
}

/// Whether a row matches a search: its words or the names of its keys,
/// ignoring case.
#[must_use]
pub fn matches(row: &KeyRow, query: &str) -> bool {
    let query = query.to_lowercase();
    query.is_empty()
        || row.what.to_lowercase().contains(&query)
        || row
            .keys
            .iter()
            .chain(&row.alt)
            .any(|k| k.to_lowercase() == query)
}

/// The groups that a search leaves, with the rows that match.
#[must_use]
pub fn found<'a>(groups: &'a [KeyGroup], query: &str) -> Vec<(&'a KeyGroup, Vec<&'a KeyRow>)> {
    groups
        .iter()
        .map(|g| {
            (
                g,
                g.rows
                    .iter()
                    .filter(|r| matches(r, query))
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(_, rows)| !rows.is_empty())
        .collect()
}

/// The help over the scrim, in the group `id` made anew: two columns
/// without a search, one with it.
#[must_use]
pub fn help(
    look: &Look,
    model: &HelpModel<'_>,
    (id, parent): (&str, &str),
    out: [f64; 2],
) -> Vec<Command> {
    let m = &command_glass(look);
    let narrow = !model.query.is_empty();
    let w = if narrow { HELP_NARROW } else { HELP_WIDE }.min(out[0] - 2.0 * UNIT);
    let shown = found(model.groups, model.query);
    let columns: Vec<Vec<(&KeyGroup, Vec<&KeyRow>)>> =
        if narrow { vec![shown] } else { split(shown) };
    let col_w = if narrow {
        w - 4.0 * UNIT
    } else {
        (w - 4.0 * UNIT - 24.0) / 2.0
    };
    let col_h = |col: &[(&KeyGroup, Vec<&KeyRow>)]| -> f64 {
        #[allow(clippy::cast_precision_loss, reason = "a few rows")]
        col.iter()
            .map(|(_, rows)| UNIT + KEY_ROW + rows.len() as f64 * KEY_ROW)
            .sum()
    };
    let body_h = columns
        .iter()
        .map(|c| col_h(c))
        .fold(0.0, f64::max)
        .max(KEY_ROW);
    let top = UNIT + SEARCH + 1.0 + UNIT;
    let h = top + body_h + UNIT + 1.0 + FOOTER;
    let x = ((out[0] - w) / 2.0).round();
    let y = HELP_TOP.min((out[1] - h) / 2.0).max(UNIT).round();
    let mut commands = vec![
        group(id, parent, [0.0, 0.0], InputMode::Pass),
        scrim(look, (&format!("{id}-scrim"), id), out),
    ];
    let body = format!("{id}-body");
    commands.extend(surface(
        look,
        m,
        &Surface {
            id: &body,
            parent: id,
            rect: [x, y, w, h],
            radius: PANEL_RADIUS,
            strong: true,
            name: "help",
            input: InputMode::Pass,
        },
    ));
    commands.extend(search(
        look,
        &body,
        (model.query, &model.placeholder, &model.scope),
        w,
    ));
    let mut index = 0;
    for (c, column) in columns.iter().enumerate() {
        #[allow(clippy::cast_precision_loss, reason = "two columns")]
        let cx = 2.0 * UNIT + c as f64 * (col_w + 24.0);
        let mut cy = top;
        for (group, rows) in column {
            cy += UNIT;
            commands.extend(head(
                look,
                &body,
                group,
                (&model.every_look, index),
                [cx, cy],
            ));
            cy += KEY_ROW;
            for row in rows {
                let selected = narrow && index == model.sel;
                commands.extend(key_row(
                    look,
                    (&body, index),
                    row,
                    (model, selected),
                    [cx, cy, col_w],
                ));
                cy += KEY_ROW;
                index += 1;
            }
        }
    }
    let foot = top + body_h + UNIT;
    commands.push(rect(
        &format!("{body}-divider-foot"),
        &body,
        [2.0 * UNIT, foot, w - 4.0 * UNIT, 1.0],
        0.0,
        m.separator,
    ));
    if !narrow {
        commands.push(text(
            look,
            &format!("{body}-note"),
            &body,
            (face(Type::Small, 400), &model.note),
            [UNIT + 16.0, foot + 1.0 + (FOOTER - 20.0) / 2.0],
            m.muted,
        ));
    }
    commands.extend(footer(look, &body, &model.footer, foot + 1.0, w));
    commands
}

/// The two columns of the help: the first groups at the left and the
/// rest at the right, cut where the taller column is the least tall.
type Shown<'a> = Vec<(&'a KeyGroup, Vec<&'a KeyRow>)>;

#[allow(clippy::cast_precision_loss, reason = "a few rows")]
fn split(mut shown: Shown<'_>) -> Vec<Shown<'_>> {
    let heights: Vec<f64> = (shown.iter())
        .map(|(_, rows)| UNIT + KEY_ROW + rows.len() as f64 * KEY_ROW)
        .collect();
    let total: f64 = heights.iter().sum();
    let mut left = 0.0;
    let mut cut = 1;
    let mut best = f64::INFINITY;
    for (k, height) in heights.iter().enumerate() {
        left += height;
        let taller = left.max(total - left);
        if taller < best {
            best = taller;
            cut = k + 1;
        }
    }
    let right = shown.split_off(cut.min(shown.len()));
    vec![shown, right]
}

fn head(
    look: &Look,
    body: &str,
    group: &KeyGroup,
    (every, index): (&str, usize),
    [x, y]: [f64; 2],
) -> Vec<Command> {
    let m = &look.glass;
    let label = face(Type::Label, 500);
    let id = format!("{body}-head-{index}");
    let mut commands = vec![text(
        look,
        &id,
        body,
        (label, &group.head),
        [x + 12.0, y + 8.0],
        m.muted,
    )];
    if group.shared {
        let w = look.size(label, &group.head, None)[0];
        commands.push(text(
            look,
            &format!("{id}-every"),
            body,
            (label, &format!("· {every}")),
            [x + 12.0 + w + 8.0, y + 8.0],
            m.muted,
        ));
    }
    commands
}

/// One row of the help at `[x, y]` in a column `w` wide.
fn key_row(
    look: &Look,
    (body, index): (&str, usize),
    row: &KeyRow,
    (model, selected): (&HelpModel<'_>, bool),
    [x, y, w]: [f64; 3],
) -> Vec<Command> {
    let m = &look.glass;
    let id = format!("{body}-row-{index}");
    let mut commands = Vec::new();
    if selected {
        commands.push(rect(
            &format!("{id}-sel"),
            body,
            [x, y, w, KEY_ROW],
            12.0,
            m.select,
        ));
    }
    let what = face(Type::Text, 400);
    if let Some((start, len)) = find(&row.what, model.query) {
        let before = look.size(what, &row.what[..start], None)[0];
        let mark = look.size(what, &row.what[start..start + len], None)[0];
        commands.push(rect(
            &format!("{id}-mark"),
            body,
            [x + 12.0 + before - 1.0, y + 4.0, mark + 2.0, 24.0],
            3.0,
            m.mark,
        ));
    }
    commands.push(text(
        look,
        &format!("{id}-what"),
        body,
        (what, &row.what),
        [x + 12.0, y + 4.0],
        m.text,
    ));
    let small = face(Type::Small, 400);
    let gesture_w = row
        .gesture
        .as_ref()
        .map_or(0.0, |g| look.size(small, g, None)[0] + 4.0);
    let or_w = if row.alt.is_empty() {
        0.0
    } else {
        look.size(small, &model.or, None)[0] + 12.0
    };
    let total = keycaps_width(look, &row.keys) + or_w + keycaps_width(look, &row.alt) + gesture_w;
    let mut kx = (x + w - 8.0 - total).round();
    let (caps, used) = keycaps(
        look,
        m,
        (&format!("{id}-keys"), body),
        &row.keys,
        [kx, y + 4.0],
    );
    commands.extend(caps);
    kx += used;
    if let Some(gesture) = &row.gesture {
        commands.push(text(
            look,
            &format!("{id}-gesture"),
            body,
            (small, gesture),
            [kx + 4.0, y + 6.0],
            m.muted,
        ));
    }
    if !row.alt.is_empty() {
        commands.push(text(
            look,
            &format!("{id}-or"),
            body,
            (small, &model.or),
            [kx + 6.0, y + 6.0],
            m.muted,
        ));
        kx += or_w;
        commands.extend(
            keycaps(
                look,
                m,
                (&format!("{id}-alt"), body),
                &row.alt,
                [kx, y + 4.0],
            )
            .0,
        );
    }
    commands
}

/// Where a search matches in a text, ignoring case: its start and its
/// length in bytes.
fn find(text: &str, query: &str) -> Option<(usize, usize)> {
    if query.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    let start = lower.find(&query.to_lowercase())?;
    (lower.len() == text.len()).then_some((start, query.len()))
}

/// The caps of a binding of the settings.
#[must_use]
pub fn chord(binding: &str) -> Vec<String> {
    caps(binding)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(what: &str, keys: &[&str]) -> KeyRow {
        KeyRow {
            what: what.into(),
            keys: keys.iter().map(|k| (*k).to_owned()).collect(),
            alt: Vec::new(),
            gesture: None,
        }
    }

    #[test]
    fn a_search_keeps_the_rows_whose_words_or_keys_match_in_their_groups() {
        let groups = vec![
            KeyGroup {
                head: "Walk".into(),
                shared: false,
                rows: vec![
                    row("Fly into the pane ahead", &["Return"]),
                    row("Back to the start", &["H"]),
                ],
            },
            KeyGroup {
                head: "Panes".into(),
                shared: false,
                rows: vec![
                    row("Close the pane ahead", &["Super", "X"]),
                    row("Open a terminal", &["Super", "T"]),
                ],
            },
        ];
        let kept = found(&groups, "PANE");
        let names: Vec<(&str, usize)> = kept
            .iter()
            .map(|(g, r)| (g.head.as_str(), r.len()))
            .collect();
        assert_eq!(names, [("Walk", 1), ("Panes", 1)]);
        assert_eq!(
            found(&groups, "super")
                .iter()
                .map(|(_, r)| r.len())
                .sum::<usize>(),
            2,
            "a key name matches whole"
        );
        assert_eq!(find("Close the pane ahead", "PANE"), Some((10, 4)));
    }

    #[test]
    fn the_help_cuts_its_columns_where_the_taller_one_is_the_least_tall() {
        let group = |rows: usize| KeyGroup {
            head: "Group".into(),
            shared: false,
            rows: (0..rows)
                .map(|k| row(&format!("row {k}"), &["X"]))
                .collect(),
        };
        let groups: Vec<KeyGroup> = [7, 4, 3, 4, 9].into_iter().map(group).collect();
        let columns = split(found(&groups, ""));
        assert_eq!(columns.iter().map(Vec::len).collect::<Vec<_>>(), [3, 2]);
        let one = split(found(&groups[..1], ""));
        assert_eq!(one.iter().map(Vec::len).collect::<Vec<_>>(), [1, 0]);
    }

    fn alpha_of(commands: &[Command], id: &str) -> f64 {
        commands
            .iter()
            .find_map(|c| match c {
                Command::Create {
                    id: made, props, ..
                } if made == id => props.color,
                _ => None,
            })
            .map_or_else(|| panic!("no node {id}"), |c| c[3])
    }

    fn help_in(dark: bool) -> Vec<Command> {
        let config: crate::config::Config = serde_saphyr::from_str(crate::config::DEFAULT).unwrap();
        let metrics = crate::Metrics::default();
        let look = Look {
            style: &config.style,
            colors: &config.style.themes.light,
            plain: false,
            shader: "/glass.frag",
            shadow: "/shadow.frag",
            glass: chora_kit::glass::Material::of(dark),
            metrics: &metrics,
        };
        let model = HelpModel {
            groups: &[],
            query: "",
            sel: 0,
            scope: String::new(),
            placeholder: String::new(),
            every_look: String::new(),
            or: String::new(),
            note: String::new(),
            footer: Vec::new(),
        };
        help(&look, &model, ("help", "root"), [1280.0, 720.0])
    }

    /// Polish 4: nothing of the space reads through a command surface.
    #[test]
    fn the_help_tints_092_over_a_scrim_of_048_light_and_056_dark() {
        for (dark, scrim) in [(false, 0.48), (true, 0.56)] {
            let commands = help_in(dark);
            assert!(
                (alpha_of(&commands, "help-scrim") - scrim).abs() < 1e-9,
                "{dark}"
            );
            assert!(
                (alpha_of(&commands, "help-body-glass") - 0.92).abs() < 1e-9,
                "{dark}"
            );
        }
    }
}
