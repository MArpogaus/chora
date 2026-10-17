//! The cascading menus of All Programs: XP's frame with the blue band and
//! the shadow, rows of 22 pixels, and the placement next to the menu they
//! come from (`CascadeMenu.qml` and `CascadeMenuView.qml` of the
//! reference).

use std::path::{Path, PathBuf};

use chora_scene::{Command, NodeKind, PropsUpdate, Transform};

use crate::{
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
};

/// The height of a row.
pub const ROW: i32 = 22;
/// A cascade overlaps the menu it comes from.
const OVERLAP: i32 = 2;
/// The text starts 23 pixels into a row, after a 16 pixel icon.
const TEXT_X: i32 = 23;

/// The distance of the rows from the outer edges of the frame, left, top,
/// right and bottom: the `cascade-hint-*-margin` elements of the theme.
/// The bottom and right margins include the shadow, which is the bottom
/// margin less the top one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Margins(pub [i32; 4]);

impl Margins {
    /// The margins of the theme.
    pub fn of(theme: &Theme) -> Self {
        let hint = |side: &str, axis: usize, default: i32| {
            theme
                .size(Sheet::Winxp, &format!("cascade-hint-{side}-margin"))
                .map_or(default, |s| s[axis])
        };
        Self([
            hint("left", 0, 6),
            hint("top", 1, 1),
            hint("right", 0, 5),
            hint("bottom", 1, 5),
        ])
    }

    /// The width of the shadow at the right and the bottom.
    pub fn shadow(self) -> i32 {
        self.0[3] - self.0[1]
    }
}

/// An open cascade.
#[derive(Debug)]
pub struct Cascade {
    /// The depth: 0 for All Programs.
    pub level: usize,
    /// In scene coordinates, with the shadow.
    pub rect: Rect,
    /// The number of rows.
    pub count: usize,
    column_width: i32,
    /// Rows per column.
    rows: i32,
    margins: Margins,
    /// The row under the pointer.
    pub hovered: Option<usize>,
    /// The row whose cascade is open.
    pub open: Option<usize>,
    /// The icon files of the rows.
    pub icons: Vec<Option<PathBuf>>,
}

impl Cascade {
    /// Lays out a cascade for rows of text widths in a work area: long
    /// menus wrap into more columns. `row` is the row it opens from in
    /// scene coordinates, `parent` the cascade of that row.
    pub fn new(
        level: usize,
        widths: &[f64],
        margins: Margins,
        [row, area]: [Rect; 2],
        parent: Option<Rect>,
    ) -> Self {
        let [left, top, right, bottom] = margins.0;
        let max_rows = ((area[3] - top - top - margins.shadow()) / ROW).max(5);
        let count = i32::try_from(widths.len()).unwrap_or(i32::MAX).max(1);
        let columns = (count + max_rows - 1) / max_rows;
        let rows = count.min(max_rows);
        let widest = widths.iter().copied().fold(0.0, f64::max);
        #[allow(clippy::cast_possible_truncation)]
        let column_width = (widest.ceil() as i32 + 2 * TEXT_X).clamp(130, 380);
        let size = [
            columns * column_width + left + right,
            rows * ROW + top + bottom,
        ];
        let [x, y] = place(size, margins, row, parent, area);
        Self {
            level,
            rect: [x, y, size[0], size[1]],
            count: widths.len(),
            column_width,
            rows,
            margins,
            hovered: None,
            open: None,
            icons: Vec::new(),
        }
    }

    /// A row in scene coordinates; columns fill top to bottom.
    pub fn row(&self, index: usize) -> Rect {
        let i = i32::try_from(index).unwrap_or(0);
        let [x, y, ..] = self.rect;
        [
            x + self.margins.0[0] + i / self.rows * self.column_width,
            y + self.margins.0[1] + i % self.rows * ROW,
            self.column_width,
            ROW,
        ]
    }

    /// Whether a row shows highlighted: under the pointer, or with its
    /// cascade open while the pointer is on no other row.
    pub fn lit(&self, index: usize) -> bool {
        lit(self.hovered, self.open, index)
    }

    /// Creates the cascade with its rows under a parent node.
    pub fn create(&self, theme: &mut Theme, parent: &str, rows: &[Row<'_>]) -> Vec<Command> {
        let [x, y, w, h] = self.rect;
        let frame = Picture::new(w, h).frame(Sheet::Winxp, "cascade", [0, 0, w, h]);
        let mut commands = vec![
            Command::Create {
                id: group_id(self.level),
                kind: NodeKind::Group,
                parent: parent.into(),
                props: at(x, y),
            },
            Command::Create {
                id: format!("{}-frame", group_id(self.level)),
                kind: NodeKind::Image,
                parent: group_id(self.level),
                props: PropsUpdate {
                    image: Some(theme.render(&frame)),
                    ..PropsUpdate::default()
                },
            },
        ];
        for (i, row) in rows.iter().enumerate() {
            let [rx, ry, ..] = self.row(i);
            commands.push(Command::Create {
                id: row_id(self.level, i),
                kind: NodeKind::Image,
                parent: group_id(self.level),
                props: PropsUpdate {
                    image: Some(theme.render(&self.picture(theme, row, self.lit(i)))),
                    ..at(rx - x, ry - y)
                },
            });
        }
        commands
    }

    /// The new look of a row.
    pub fn relook(&self, theme: &mut Theme, index: usize, row: &Row<'_>) -> Command {
        Command::Update {
            id: row_id(self.level, index),
            props: PropsUpdate {
                image: Some(theme.render(&self.picture(theme, row, self.lit(index)))),
                ..PropsUpdate::default()
            },
        }
    }

    /// Removes the cascade.
    pub fn remove(&self) -> Command {
        Command::Remove {
            id: group_id(self.level),
            to: None,
        }
    }

    /// A row: highlight, icon, name and, for a group, the arrow.
    fn picture(&self, theme: &Theme, row: &Row<'_>, lit: bool) -> Picture {
        let w = self.column_width;
        let mut picture = Picture::new(w, ROW);
        if lit {
            picture = picture.frame(Sheet::Winxp, "cascade-highlight", [0, 0, w, ROW]);
        }
        if let Some(icon) = row.icon {
            picture = picture.file(icon, [4, (ROW - 16) / 2, 16, 16]);
        }
        if row.checked {
            let check = if lit {
                "winxp/check-selected.svg"
            } else {
                "winxp/check.svg"
            };
            // XP's 8 pixel check box, in the middle of the icon column.
            picture = picture.file(&theme.path(check), [8, (ROW - 8) / 2, 8, 8]);
        }
        let color = if lit {
            [255, 255, 255, 255]
        } else {
            [0, 0, 0, 255]
        };
        let end = if row.group { 14 } else { 4 };
        picture = picture.text(Text::new(
            row.title,
            Font::Ui,
            color,
            [TEXT_X, 0, w - TEXT_X - end, ROW],
        ));
        if row.group {
            picture = arrow(theme, picture, lit);
        }
        picture
    }
}

/// The arrow of a group at the right of a picture: XP's 8 pixel arrow
/// box, 4 pixels from the right, white when the row is lit.
pub fn arrow(theme: &Theme, picture: Picture, lit: bool) -> Picture {
    let [w, h] = picture.size;
    let file = if lit {
        "winxp/cascade-arrow-selected.svg"
    } else {
        "winxp/cascade-arrow.svg"
    };
    picture.file(&theme.path(file), [w - 4 - 8, (h - 8) / 2, 8, 8])
}

/// Whether a row shows highlighted, for the row under the pointer and the
/// row whose cascade is open.
pub fn lit(hovered: Option<usize>, open: Option<usize>, index: usize) -> bool {
    hovered == Some(index) || (open == Some(index) && hovered.is_none())
}

/// What a row shows.
pub struct Row<'a> {
    /// The name.
    pub title: &'a str,
    /// The icon file.
    pub icon: Option<&'a Path>,
    /// A group opens a cascade.
    pub group: bool,
    /// A check mark instead of the icon.
    pub checked: bool,
}

/// The top-left corner of a cascade of a size (with its shadow): right of
/// the menu it comes from, overlapping it, with the first row level with
/// the row it opens from; mirrored when there is no room on the right,
/// and ending level with the row when there is none below. Without a
/// parent menu (All Programs), it starts right of the row and ends level
/// with its bottom.
fn place(
    [w, h]: [i32; 2],
    margins: Margins,
    [rx, ry, rw, rh]: Rect,
    parent: Option<Rect>,
    [ax, ay, aw, ah]: Rect,
) -> [i32; 2] {
    let [_, top, ..] = margins.0;
    let shadow = margins.shadow();
    let (right, bottom) = (ax + aw, ay + ah);
    let (x, y) = match parent {
        Some([px, _, pw, _]) => {
            let mut x = px + pw - shadow - OVERLAP;
            if x + w > right {
                x = px + OVERLAP - (w - shadow);
            }
            let mut y = ry - top;
            if y + h > bottom {
                y = ry + rh + top - (h - shadow);
            }
            (x, y)
        }
        None => ((rx + rw).min(right - w), ry + rh + top - (h - shadow)),
    };
    [x.max(ax), y.max(ay)]
}

fn group_id(level: usize) -> String {
    format!("xp-menu-cascade-{level}")
}

fn row_id(level: usize, index: usize) -> String {
    format!("xp-menu-row-{level}-{index}")
}

/// The level and the index of a row node.
pub fn row_of(node: &str) -> Option<(usize, usize)> {
    let (level, index) = node.strip_prefix("xp-menu-row-")?.split_once('-')?;
    Some((level.parse().ok()?, index.parse().ok()?))
}

fn at(x: i32, y: i32) -> PropsUpdate {
    PropsUpdate {
        transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
        ..PropsUpdate::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARGINS: Margins = Margins([6, 1, 5, 5]);
    const AREA: Rect = [0, 0, 1024, 738];

    #[test]
    fn the_margins_come_from_the_theme() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme");
        let cache = std::env::temp_dir().join(format!("chora-xp-margins-{}", std::process::id()));
        let theme = Theme::load(&dir, &cache).expect("the theme loads");
        assert_eq!(Margins::of(&theme), MARGINS);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn all_programs_grows_up_from_its_button() {
        let button = [6, 674, 179, 24];
        let c = Cascade::new(0, &[60.0; 10], MARGINS, [button, AREA], None);
        // 130 wide at least; ten rows of 22 and the frame.
        assert_eq!(c.rect, [185, 477, 141, 226]);
        // The last row ends level with the button.
        assert_eq!(c.row(9)[1] + ROW, 698);
        // No room on the right: it shifts left.
        let c = Cascade::new(0, &[60.0; 10], MARGINS, [[900, 674, 179, 24], AREA], None);
        assert_eq!(c.rect[0], 1024 - 141);
    }

    #[test]
    fn a_cascade_opens_beside_its_row() {
        let parent = [185, 100, 141, 226];
        let row = [191, 167, 130, 22];
        let c = Cascade::new(1, &[200.0; 3], MARGINS, [row, AREA], Some(parent));
        // Overlapping the parent by 2 pixels, the first row level with
        // the row.
        assert_eq!(c.rect[..2], [185 + 141 - 4 - 2, 166]);
        assert_eq!(c.row(0)[..2], [c.rect[0] + 6, 167]);
        assert_eq!(c.rect[2], 246 + 11);
    }

    #[test]
    fn a_cascade_flips_at_the_edges() {
        let parent = [800, 500, 141, 226];
        let row = [806, 700, 130, 22];
        let c = Cascade::new(1, &[200.0; 3], MARGINS, [row, AREA], Some(parent));
        // Left of the parent, ending level with the row.
        assert_eq!(c.rect[0], 800 + 2 - (257 - 4));
        assert_eq!(c.row(2)[1] + ROW, 722);
    }

    #[test]
    fn long_menus_wrap_into_columns() {
        // (738 - 6) / 22 = 33 rows a column.
        let c = Cascade::new(0, &[10.0; 70], MARGINS, [[6, 674, 179, 24], AREA], None);
        assert_eq!(c.rect[2], 3 * 130 + 11);
        assert_eq!(c.rect[3], 33 * ROW + 6);
        assert_eq!(c.row(33)[..2], [c.rect[0] + 6 + 130, c.rect[1] + 1]);
        // At least five rows on a tiny screen.
        let c = Cascade::new(
            0,
            &[10.0; 7],
            MARGINS,
            [[0, 0, 10, 10], [0, 0, 400, 50]],
            None,
        );
        assert_eq!(c.rect[3], 5 * ROW + 6);
    }

    #[test]
    fn rows_parse_their_node() {
        assert_eq!(row_of(&row_id(2, 17)), Some((2, 17)));
        assert_eq!(row_of("xp-menu-cascade-1-frame"), None);
    }
}
