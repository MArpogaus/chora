//! The vertical fit of the items of a region: which lines stay, which give
//! way, and the commands that place a window and a rect.

use super::{
    Command, Item, NodeKind, PropsUpdate, ToplevelState, create,
    fonts::{self, Ink},
    move_to,
};

/// The width of the margin column on an output of `output` pixels: 26 per
/// cent of it in whole lines of the grid, from `lines` lines up to a
/// third more.
pub fn column_width(lines: f64, baseline: f64, output: f64) -> f64 {
    let most = (lines * 4.0 / 3.0).round() * baseline;
    ((0.26 * output / baseline).round() * baseline).clamp(lines * baseline, most)
}

/// The space that a text carries after its last letter: the tracking, which
/// an item flush right gives back so that its ink meets the edge.
pub(super) fn trailing(props: &PropsUpdate) -> f64 {
    props.letter_spacing.unwrap_or(0.0) * props.font_size.unwrap_or(0.0)
}

/// The capitals of the text that the props describe, from the top of its box.
pub(super) fn props_ink(props: &PropsUpdate) -> Ink {
    let size = props.font_size.unwrap_or(16.0);
    let line = size * props.line_height.unwrap_or(1.2);
    fonts::ink(props.family.as_deref(), props.weight, size, line)
}

/// The vertical places of the items that sit in the foot, or of those
/// that do not, from 0, and where they end.
pub fn lines(
    items: &[Item],
    heights: &[f64],
    foot: bool,
    baseline: f64,
) -> (Vec<(usize, f64)>, f64) {
    lines_of(items, heights, foot, baseline, &vec![true; items.len()])
}

pub(super) fn lines_of(
    items: &[Item],
    heights: &[f64],
    foot: bool,
    baseline: f64,
    keep: &[bool],
) -> (Vec<(usize, f64)>, f64) {
    let (mut y, mut line, mut out) = (0.0, 0.0, Vec::new());
    for (i, item) in items
        .iter()
        .enumerate()
        .filter(|(i, it)| keep[*i] && it.put.foot() == foot)
    {
        if item.put.row() {
            out.push((i, line));
            y = f64::max(y, line + heights[i]);
            continue;
        }
        if item.first && y > 0.0 {
            y += baseline;
        }
        y += item.before;
        line = y;
        out.push((i, y));
        y += heights[i];
    }
    (out, y)
}

/// Shows the items that give way where they stay and hides the others.
pub(super) fn shed_updates(items: &[Item], keep: &[bool]) -> Vec<Command> {
    (items.iter().zip(keep))
        .filter(|(item, _)| item.shed)
        .map(|(item, kept)| Command::Update {
            id: item.id.clone(),
            props: PropsUpdate {
                visible: Some(*kept),
                ..PropsUpdate::default()
            },
        })
        .collect()
}

/// Which items of a side stay when its flow must end within `room`: the
/// items that give way go from the last one, and a title with no entry
/// left goes with them.
pub fn fit(items: &[Item], heights: &[f64], room: f64, baseline: f64) -> Vec<bool> {
    let mut keep = vec![true; items.len()];
    let end = |keep: &[bool]| lines_of(items, heights, false, baseline, keep).1;
    while end(&keep) > room {
        let Some(last) = (0..items.len()).rev().find(|&i| keep[i] && items[i].shed) else {
            return keep;
        };
        keep[last] = false;
    }
    let shed = |i: &usize| items[*i].shed && keep[*i];
    let left: Vec<usize> = (0..items.len()).filter(shed).collect();
    let dropped = items
        .iter()
        .zip(&keep)
        .any(|(item, kept)| item.shed && !kept);
    if dropped && left.iter().all(|&i| items[i].id.ends_with("-title")) {
        for i in left {
            keep[i] = false;
        }
    }
    keep
}

/// The number of letters of `count` that a share of the width keeps.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "a line of a screen"
)]
fn kept(count: usize, share: f64) -> usize {
    (count as f64 * share).floor().max(0.0) as usize
}

/// Shortens the clock of the end of a line by the `over` that the end
/// takes beyond the room left by the head and the middle, so that the head
/// keeps its letters; the clock keeps its last words, its width falls.
pub(super) fn clock_gives_way(end: &mut [(&Item, f64, f64)], over: f64) -> Vec<Command> {
    let mut commands = Vec::new();
    for (item, w, _) in end {
        if item.id.ends_with("-clock") && over > 0.0 && *w > over {
            let room = *w - over;
            commands.push(shorten_front(item, room / *w));
            *w = room;
        }
    }
    commands
}

/// Cuts the text of an item to a share of its letters from the front, by
/// whole words: a date gives up its weekday first and keeps its time. The
/// last word stays, however narrow the room.
pub(super) fn shorten_front(item: &Item, share: f64) -> Command {
    let text = item.props.text.as_deref().unwrap_or_default();
    let count = text.chars().count();
    let keep = kept(count, share);
    let mut rest = text;
    while rest.chars().count() > keep {
        match rest.split_once(char::is_whitespace) {
            Some((_, tail)) if !tail.trim().is_empty() => rest = tail.trim_start(),
            _ => break,
        }
    }
    Command::Update {
        id: item.id.clone(),
        props: PropsUpdate {
            text: Some(rest.to_owned()),
            ..PropsUpdate::default()
        },
    }
}

/// Cuts the text of an item to a share of its width, with an ellipsis;
/// nothing below a few letters. It cuts by the share of the letters, not
/// by their widths, and one frame shows the whole text first; an ellipsis
/// of the core would fix both.
pub(super) fn shorten(item: &Item, share: f64) -> Command {
    let text = item.props.text.as_deref().unwrap_or_default();
    let count = text.chars().count();
    let keep = kept(count, share);
    let short = if keep < 4 {
        " ".to_owned()
    } else {
        let cut: String = text.chars().take(keep - 1).collect();
        format!("{}…", cut.trim_end())
    };
    Command::Update {
        id: item.id.clone(),
        props: PropsUpdate {
            text: Some(short),
            ..PropsUpdate::default()
        },
    }
}

/// Moves a surface under a group.
pub(super) fn park(surface: &str, parent: &str) -> Command {
    Command::Move {
        id: surface.into(),
        parent: Some(parent.into()),
        index: None,
    }
}

/// Puts a window at a box of its parent: full screen at the size of the
/// output, else tiled.
pub fn place(surface: &str, [x, y, w, h]: [f64; 4], output: [i32; 2]) -> [Command; 2] {
    #[allow(clippy::cast_possible_truncation, reason = "pixels of a screen")]
    let size = [w.round() as i32, h.round() as i32];
    let full = size == output && x == 0.0 && y == 0.0;
    let state = if full {
        ToplevelState::Fullscreen
    } else {
        ToplevelState::Tiled
    };
    [
        move_to(surface.into(), x, y),
        Command::Configure {
            surface: surface.into(),
            size: Some(size),
            states: vec![state],
        },
    ]
}

pub fn rect(id: &str, parent: &str, [x, y, w, h]: [f64; 4], color: [f64; 4]) -> Command {
    create(
        id,
        NodeKind::Rect,
        parent,
        PropsUpdate {
            transform: Some(crate::translate(x, y)),
            size: Some([w, h]),
            color: Some(color),
            ..PropsUpdate::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stacked(id: &str, shed: bool, first: bool) -> Item {
        Item {
            shed,
            ..Item::text(id.into(), 0, PropsUpdate::default(), first)
        }
    }

    #[test]
    fn the_oldest_notifications_give_way_when_the_side_is_full() {
        let items = [
            stacked("door", false, true),
            stacked("notify-title", true, true),
            stacked("notify-0", true, false),
            stacked("notify-1", true, false),
        ];
        let heights = [28.0; 4];
        let kept = |room| fit(&items, &heights, room, 28.0);
        assert_eq!(kept(400.0), [true; 4], "all fit");
        assert_eq!(
            kept(130.0),
            [true, true, true, false],
            "the oldest goes first"
        );
        assert_eq!(
            kept(60.0),
            [true, false, false, false],
            "no title stands over no entry"
        );
        assert_eq!(
            kept(10.0),
            [true, false, false, false],
            "the rest never gives way"
        );
    }

    #[test]
    fn the_column_grows_with_the_output_in_whole_lines() {
        assert_eq!(column_width(12.0, 28.0, 1280.0), 336.0);
        assert_eq!(column_width(12.0, 28.0, 1920.0), 448.0);
        assert_eq!(column_width(12.0, 28.0, 1440.0), 364.0);
        assert_eq!(column_width(12.0, 28.0, 800.0), 336.0);
    }
}
