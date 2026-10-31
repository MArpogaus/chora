//! The layouts: how the tiled windows of one output share its work area.
//! The names and the behavior follow the layouts of awesome.

use serde::{Deserialize, Serialize};

/// A rectangle: x, y, width and height in logical pixels.
pub type Rect = [i32; 4];

/// How the tiled windows share the work area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    /// The masters on the left, the stack in columns on the right.
    Tile,
    /// The masters on the right.
    TileLeft,
    /// The masters at the top.
    TileBottom,
    /// The masters at the bottom.
    TileTop,
    /// A grid of columns.
    Fair,
    /// A grid of rows.
    FairHorizontal,
    /// Each window takes half of the rest, turning clockwise.
    Spiral,
    /// Each window takes half of the rest, to the right and down.
    Dwindle,
    /// Every window fills the work area; the focused one is on top.
    Max,
    /// Every window fills the output; the focused one is on top.
    Fullscreen,
    /// The focused window floats in the middle; the others share a column.
    Magnifier,
    /// Every window floats.
    Floating,
}

/// The parameters of a tag that the tile and magnifier layouts use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub master_width: f64,
    pub masters: usize,
    pub columns: usize,
}

/// The cells of `n` tiled windows, in their order. `focused` is the index
/// of the focused window. The floating layout has no cells.
pub fn arrange(
    layout: Layout,
    n: usize,
    (area, output): (Rect, Rect),
    focused: Option<usize>,
    params: Params,
) -> Vec<Rect> {
    match layout {
        Layout::Tile => tile(n, area, params, false, false),
        Layout::TileLeft => tile(n, area, params, true, false),
        Layout::TileBottom => tile(n, area, params, false, true),
        Layout::TileTop => tile(n, area, params, true, true),
        Layout::Fair => fair(n, area),
        Layout::FairHorizontal => fair(n, transpose(area))
            .into_iter()
            .map(transpose)
            .collect(),
        Layout::Spiral => halves(n, area, true),
        Layout::Dwindle => halves(n, area, false),
        Layout::Max => vec![area; n],
        Layout::Fullscreen => vec![output; n],
        Layout::Magnifier => magnifier(n, area, focused, params.master_width),
        Layout::Floating => Vec::new(),
    }
}

/// The tile layout with the masters on the left, then mirrored and turned
/// for the other sides.
fn tile(n: usize, area: Rect, params: Params, mirror: bool, turn: bool) -> Vec<Rect> {
    let frame = if turn { transpose(area) } else { area };
    tile_right(n, frame, params)
        .into_iter()
        .map(|cell| if mirror { mirrored(cell, frame) } else { cell })
        .map(|cell| if turn { transpose(cell) } else { cell })
        .collect()
}

fn tile_right(count: usize, [x, y, w, h]: Rect, params: Params) -> Vec<Rect> {
    let masters = params.masters.min(count);
    let stack = count - masters;
    let master_width = match (masters, stack) {
        (0, _) => 0,
        (_, 0) => w,
        #[allow(clippy::cast_possible_truncation)] // a share of an i32
        _ => (f64::from(w) * params.master_width).round() as i32,
    };
    let mut cells = split(masters, [x, y, master_width, h], false);
    let columns = params.columns.clamp(1, stack.max(1));
    let mut left = stack;
    for (i, column) in split(columns, [x + master_width, y, w - master_width, h], true)
        .into_iter()
        .enumerate()
    {
        let count = left.div_ceil(columns - i);
        cells.extend(split(count, column, false));
        left -= count;
    }
    cells
}

/// Columns of windows: `ceil(sqrt(n))` per column, the rest in the last
/// column; two windows stand side by side.
fn fair(n: usize, area: Rect) -> Vec<Rect> {
    if n == 0 {
        return Vec::new();
    }
    let rows = if n == 2 {
        1
    } else {
        n.isqrt() + usize::from(n.isqrt().pow(2) < n)
    };
    let columns = n.div_ceil(rows);
    split(columns, area, true)
        .into_iter()
        .enumerate()
        .flat_map(|(c, column)| split(rows.min(n - c * rows), column, false))
        .collect()
}

/// Each window takes one half of the rest, and the last one the whole rest.
/// The spiral turns clockwise: right, down, left, up. The dwindle goes
/// right and down only.
fn halves(count: usize, mut rest: Rect, spiral: bool) -> Vec<Rect> {
    let mut cells = Vec::with_capacity(count);
    for i in 0..count {
        if i + 1 == count {
            cells.push(rest);
            break;
        }
        let [x, y, w, h] = rest;
        let side = if spiral { i % 4 } else { i % 2 };
        let (cell, next) = match side {
            0 => ([x, y, w / 2, h], [x + w / 2, y, w - w / 2, h]),
            1 => ([x, y, w, h / 2], [x, y + h / 2, w, h - h / 2]),
            2 => ([x + w - w / 2, y, w / 2, h], [x, y, w - w / 2, h]),
            _ => ([x, y + h - h / 2, w, h / 2], [x, y, w, h - h / 2]),
        };
        cells.push(cell);
        rest = next;
    }
    cells
}

/// The focused window in the middle, with `sqrt(share)` of each side; the
/// others in rows over the whole area.
fn magnifier(count: usize, area: Rect, focused: Option<usize>, share: f64) -> Vec<Rect> {
    let Some(focused) = focused.filter(|f| *f < count && count > 1) else {
        return vec![area; count];
    };
    let [x, y, w, h] = area;
    #[allow(clippy::cast_possible_truncation)] // a share of an i32
    let scale = |side: i32| (f64::from(side) * share.sqrt()).round() as i32;
    let (fw, fh) = (scale(w), scale(h));
    let mut cells = split(count - 1, area, false);
    cells.insert(focused, [x + (w - fw) / 2, y + (h - fh) / 2, fw, fh]);
    cells
}

/// Splits an area into `n` equal columns, or rows.
pub fn split(n: usize, [left, top, width, height]: Rect, columns: bool) -> Vec<Rect> {
    let Ok(n) = i32::try_from(n) else {
        return Vec::new();
    };
    (0..n)
        .map(|i| {
            if columns {
                let (start, end) = (width * i / n, width * (i + 1) / n);
                [left + start, top, end - start, height]
            } else {
                let (start, end) = (height * i / n, height * (i + 1) / n);
                [left, top + start, width, end - start]
            }
        })
        .collect()
}

/// Shrinks a cell by a gap on all sides, but never below 1 pixel and never
/// out of the cell.
pub fn inset([x, y, w, h]: Rect, gap: i32) -> Rect {
    let (gx, gy) = (gap.min((w - 1) / 2).max(0), gap.min((h - 1) / 2).max(0));
    [x + gx, y + gy, (w - 2 * gx).max(1), (h - 2 * gy).max(1)]
}

fn transpose([x, y, w, h]: Rect) -> Rect {
    [y, x, h, w]
}

fn mirrored([x, y, w, h]: Rect, [fx, _, fw, _]: Rect) -> Rect {
    [fx + fw - (x - fx) - w, y, w, h]
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = [0, 0, 100, 60];
    const ONE: Params = Params {
        master_width: 0.5,
        masters: 1,
        columns: 1,
    };

    fn cells(layout: Layout, n: usize) -> Vec<Rect> {
        arrange(layout, n, (AREA, [0, 0, 100, 80]), Some(0), ONE)
    }

    #[test]
    fn one_window_fills_the_area() {
        assert_eq!(cells(Layout::Tile, 1), vec![AREA]);
        assert_eq!(cells(Layout::Fair, 1), vec![AREA]);
        assert_eq!(cells(Layout::Spiral, 1), vec![AREA]);
    }

    #[test]
    fn tile_puts_the_master_on_its_side() {
        let stack = [[50, 0, 50, 30], [50, 30, 50, 30]];
        assert_eq!(
            cells(Layout::Tile, 3),
            [[[0, 0, 50, 60]].as_slice(), &stack].concat()
        );
        assert_eq!(
            cells(Layout::TileLeft, 2),
            vec![[50, 0, 50, 60], [0, 0, 50, 60]]
        );
        assert_eq!(
            cells(Layout::TileBottom, 2),
            vec![[0, 0, 100, 30], [0, 30, 100, 30]]
        );
        assert_eq!(
            cells(Layout::TileTop, 2),
            vec![[0, 30, 100, 30], [0, 0, 100, 30]]
        );
    }

    #[test]
    fn tile_follows_the_tag_parameters() {
        let params = Params {
            master_width: 0.6,
            masters: 2,
            columns: 2,
        };
        let cells = arrange(Layout::Tile, 4, (AREA, AREA), None, params);
        assert_eq!(
            cells,
            vec![
                [0, 0, 60, 30],
                [0, 30, 60, 30],
                [60, 0, 20, 60],
                [80, 0, 20, 60]
            ]
        );
    }

    #[test]
    fn fair_makes_columns_and_its_turn_rows() {
        assert_eq!(
            cells(Layout::Fair, 2),
            vec![[0, 0, 50, 60], [50, 0, 50, 60]]
        );
        assert_eq!(
            cells(Layout::FairHorizontal, 2),
            vec![[0, 0, 100, 30], [0, 30, 100, 30]]
        );
        assert_eq!(
            cells(Layout::Fair, 3),
            vec![[0, 0, 50, 30], [0, 30, 50, 30], [50, 0, 50, 60]]
        );
    }

    #[test]
    fn spiral_turns_and_dwindle_does_not() {
        assert_eq!(
            cells(Layout::Spiral, 4),
            vec![
                [0, 0, 50, 60],
                [50, 0, 50, 30],
                [75, 30, 25, 30],
                [50, 30, 25, 30]
            ]
        );
        assert_eq!(
            cells(Layout::Dwindle, 3),
            vec![[0, 0, 50, 60], [50, 0, 50, 30], [50, 30, 50, 30]]
        );
    }

    #[test]
    fn max_fullscreen_and_magnifier() {
        assert_eq!(cells(Layout::Max, 2), vec![AREA; 2]);
        assert_eq!(cells(Layout::Fullscreen, 1), vec![[0, 0, 100, 80]]);
        let magnified = arrange(Layout::Magnifier, 3, (AREA, AREA), Some(1), ONE);
        assert_eq!(magnified[1], [14, 9, 71, 42]);
        assert_eq!(magnified[0], [0, 0, 100, 30]);
        assert_eq!(cells(Layout::Floating, 2), Vec::<Rect>::new());
    }

    #[test]
    fn a_huge_gap_stays_inside_the_cell() {
        let [x, y, w, h] = inset(AREA, 700);
        assert!(x + w <= 100 && y + h <= 60 && w >= 1 && h >= 1);
    }
}
