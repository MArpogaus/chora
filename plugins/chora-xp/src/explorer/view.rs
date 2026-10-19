//! The view of an Explorer window: XP's Tiles view, items of 48 pixel
//! icons with their name and gray lines beside them, in groups under
//! headings as My Computer shows them, and the Luna scroll bar when they
//! do not fit.

use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
};

use crate::{
    dialog::{self, BLACK},
    folder::Place,
    text::{Font, Text},
    theme::{Fit, Picture, Rect, Theme},
};

/// The step from one tile to the next, across and down.
pub const TILE: [i32; 2] = [218, 56];
/// The width of a tile that takes clicks.
const TILE_W: i32 = 214;
/// The margin left of the tiles.
const LEFT: i32 = 16;
/// The width of the scroll bar.
pub const SCROLL_W: i32 = 17;
/// The height of a heading with its line and the space below it.
const HEADING: i32 = 30;
/// The space below the tiles of a group.
const GROUP_GAP: i32 = 13;
/// The space above the tiles of a view without headings.
const TOP: i32 = 8;
/// The gray lines below the name of a tile.
const LINE_GRAY: [u8; 4] = [128, 128, 128, 255];
/// XP's selection blue.
const SELECTION: &str = "#316AC5";

/// What a double click on an item does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Open {
    /// Shows a place in the same window.
    Place(Place),
    /// Opens a file with `xdg-open`.
    File(PathBuf),
}

/// An item of the view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The name.
    pub title: String,
    /// The gray lines below the name, such as the type and the size.
    pub lines: Vec<String>,
    /// The icon names, the most fitting first.
    pub icons: &'static [&'static str],
    /// What a double click does.
    pub open: Option<Open>,
    /// The lines of the Details box while the item is selected, the bold
    /// one first.
    pub details: Vec<(String, bool)>,
}

/// Items under a heading; a folder has one group without a heading.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Group {
    /// The heading, as My Computer has them.
    pub heading: Option<String>,
    /// The items.
    pub items: Vec<Item>,
}

/// Where the headings and the tiles go in a view of a width, from the
/// top of the view without scrolling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// The top-left corner of each tile, item by item over the groups.
    pub tiles: Vec<[i32; 2]>,
    /// The headings and their tops.
    pub headings: Vec<(String, i32)>,
    /// The height of all.
    pub height: i32,
}

/// Lays out groups in a width: the tiles fill rows from the left, as
/// many as fit, at least one.
pub fn layout(groups: &[Group], width: i32) -> Layout {
    let columns = ((width - LEFT) / TILE[0]).max(1);
    let mut out = Layout {
        tiles: Vec::new(),
        headings: Vec::new(),
        height: 0,
    };
    let mut y = 0;
    for group in groups {
        if let Some(heading) = &group.heading {
            out.headings.push((heading.clone(), y));
            y += HEADING;
        } else {
            y += TOP;
        }
        let mut row = 0;
        for (i, _) in group.items.iter().enumerate() {
            let i = i32::try_from(i).unwrap_or(i32::MAX);
            row = i / columns;
            out.tiles
                .push([LEFT + (i % columns) * TILE[0], y + row * TILE[1]]);
        }
        if !group.items.is_empty() {
            y += (row + 1) * TILE[1];
        }
        y += GROUP_GAP;
    }
    out.height = y;
    out
}

/// The farthest the view scrolls: what does not fit.
pub fn max_scroll(height: i32, view: i32) -> i32 {
    (height - view).max(0)
}

/// The thumb of a scroll bar in a track of a height, for a view of a
/// height over content of a height, scrolled by an offset: its top in
/// the track and its height, at least 8 pixels.
pub fn thumb(track: i32, view: i32, height: i32, offset: i32) -> (i32, i32) {
    let max = max_scroll(height, view);
    if max == 0 || track <= 0 {
        return (0, track.max(0));
    }
    let size = (i64::from(track) * i64::from(view) / i64::from(height.max(1))).max(8);
    let size = i32::try_from(size).unwrap_or(track).min(track);
    let top = i64::from(track - size) * i64::from(offset.clamp(0, max)) / i64::from(max);
    (i32::try_from(top).unwrap_or(0), size)
}

/// The offset after the thumb moved by some pixels from an offset.
pub fn drag_offset(track: i32, view: i32, height: i32, (from, moved): (i32, i32)) -> i32 {
    let max = max_scroll(height, view);
    let (_, size) = thumb(track, view, height, from);
    let room = track - size;
    if room <= 0 {
        return from.clamp(0, max);
    }
    let offset = i64::from(from) + i64::from(moved) * i64::from(max) / i64::from(room);
    i32::try_from(offset.clamp(0, i64::from(max))).unwrap_or(0)
}

/// A tile: the icon, the name and the gray lines, white on XP's blue
/// when selected.
pub fn tile(theme: &Theme, item: &Item, icon: Option<&Path>, selected: bool) -> Picture {
    let [w, h] = [TILE_W, TILE[1]];
    let mut picture = Picture::new(w, h);
    if let Some(icon) = icon {
        picture = picture.file(icon, [0, 4, 48, 48]);
    }
    let lines: Vec<&str> = std::iter::once(item.title.as_str())
        .chain(item.lines.iter().map(String::as_str))
        .collect();
    let count = i32::try_from(lines.len()).unwrap_or(1);
    let top = (h - 13 * count) / 2;
    let room = w - 56;
    if selected {
        #[allow(clippy::cast_possible_truncation)]
        let widest = lines
            .iter()
            .map(|l| theme.text_width(l, Font::Ui).ceil() as i32)
            .max()
            .unwrap_or(0)
            .min(room);
        picture = picture.svg(format!(
            r#"<rect x="52" y="{}" width="{}" height="{}" fill="{SELECTION}"/>"#,
            top - 1,
            widest + 4,
            13 * count + 2
        ));
    }
    for (i, line) in lines.iter().enumerate() {
        let i = i32::try_from(i).unwrap_or(0);
        let color = match (selected, i) {
            (true, _) => [255, 255, 255, 255],
            (false, 0) => BLACK,
            (false, _) => LINE_GRAY,
        };
        let rect = [54, top + 13 * i, room, 13];
        picture = picture.text(Text::new(line, Font::Ui, color, rect));
    }
    picture
}

/// A heading of a view of a width: bold black text over XP's blue line,
/// which fades to white.
pub fn heading(heading: &str, w: i32) -> Picture {
    Picture::new(w, HEADING)
        .svg(r##"<defs><linearGradient id="l" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#3D95FF"/><stop offset="1" stop-color="#FFFFFF"/></linearGradient></defs><rect y="18" width="300" height="1" fill="url(#l)"/>"##)
        .text(Text::new(heading, Font::UiBold, BLACK, [14, 3, w - 28, 13]))
}

/// A part of a picture that shows inside a rectangle: the rectangle
/// where they overlap and the picture moved into it, or `None`. A part
/// is cut from the rendered picture, so that it costs no new layout of
/// its text and icons.
pub fn clip(
    theme: &mut Theme,
    picture: Picture,
    [x, y]: [i32; 2],
    [cx, cy, cw, ch]: Rect,
) -> Option<(Rect, Picture)> {
    let [w, h] = picture.size;
    let (left, top) = (x.max(cx), y.max(cy));
    let (right, bottom) = ((x + w).min(cx + cw), (y + h).min(cy + ch));
    if right <= left || bottom <= top {
        return None;
    }
    if [left, top, right, bottom] == [x, y, x + w, y + h] {
        return Some(([x, y, w, h], picture));
    }
    let (iw, ih) = (right - left, bottom - top);
    let whole = theme.render(&picture);
    let part = Picture::new(iw, ih).fit(Path::new(&whole), [x - left, y - top, w, h], Fit::Stretch);
    Some(([left, top, iw, ih], part))
}

/// The parts of XP's Luna scroll bar in a rectangle: the arrows at the
/// ends, the pale track and the blue thumb, as pictures with their
/// rectangles in the bar.
pub fn scroll_bar([w, h]: [i32; 2], (top, size): (i32, i32)) -> [(Rect, Picture); 4] {
    let track = Picture::new(w, h - 2 * w).svg(format!(
        r##"<rect width="{w}" height="{}" fill="#FEFEFB"/><rect width="1" height="{}" fill="#F3F1EC"/>"##,
        h - 2 * w,
        h - 2 * w
    ));
    let arrow = |down: bool| {
        let mut svg = dialog::arrow_button([0, 0, w, w], false);
        if !down {
            svg = format!(
                r#"<g transform="rotate(180 {c} {c})">{svg}</g>"#,
                c = f64::from(w) / 2.0
            );
        }
        Picture::new(w, w).svg(svg)
    };
    let thumb = Picture::new(w, size).svg(thumb_svg(w, size));
    [
        ([0, w, w, h - 2 * w], track),
        ([0, w + top, w, size], thumb),
        ([0, 0, w, w], arrow(false)),
        ([0, h - w, w, w], arrow(true)),
    ]
}

/// The blue thumb with its grip of four lines in the middle.
fn thumb_svg(w: i32, h: i32) -> String {
    let mut svg = format!(
        r##"<defs><linearGradient id="t" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#C9DAFC"/><stop offset="1" stop-color="#B1C8FA"/></linearGradient></defs><rect x="0.5" y="0.5" width="{}" height="{}" rx="2" fill="url(#t)" stroke="#9CB6F2"/>"##,
        w - 1,
        h - 1
    );
    if h > 14 {
        let mid = h / 2 - 4;
        for i in 0..4 {
            let _ = write!(
                svg,
                r##"<rect x="5" y="{}" width="7" height="1" fill="#EEF4FE"/><rect x="6" y="{}" width="7" height="1" fill="#8CB0F8"/>"##,
                mid + 2 * i,
                mid + 2 * i + 1
            );
        }
    }
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(n: usize) -> Vec<Item> {
        (0..n)
            .map(|i| Item {
                title: format!("{i}"),
                lines: Vec::new(),
                icons: &["folder"],
                open: None,
                details: Vec::new(),
            })
            .collect()
    }

    #[test]
    fn tiles_fill_rows_under_their_headings() {
        let groups = [
            Group {
                heading: Some("Files Stored on This Computer".into()),
                items: items(2),
            },
            Group {
                heading: Some("Hard Disk Drives".into()),
                items: items(3),
            },
        ];
        // Two columns in the view of XP's My Computer.
        let l = layout(&groups, 567);
        assert_eq!(l.headings[0].1, 0);
        assert_eq!(l.tiles[..2], [[16, 30], [234, 30]]);
        // The second heading sits 99 pixels lower, as in XP.
        assert_eq!(l.headings[1].1, 99);
        assert_eq!(l.tiles[2..], [[16, 129], [234, 129], [16, 185]]);
        assert_eq!(l.height, 99 + 30 + 112 + 13);
    }

    #[test]
    fn a_narrow_view_keeps_one_column() {
        let l = layout(
            &[Group {
                heading: None,
                items: items(2),
            }],
            100,
        );
        assert_eq!(l.tiles, [[16, 8], [16, 64]]);
    }

    #[test]
    fn the_thumb_follows_the_offset() {
        // Content of twice the view: the thumb is half the track.
        assert_eq!(thumb(400, 200, 400, 0), (0, 200));
        assert_eq!(thumb(400, 200, 400, 200), (200, 200));
        assert_eq!(thumb(400, 200, 400, 100), (100, 200));
        // Nothing to scroll: the thumb fills the track.
        assert_eq!(thumb(400, 500, 400, 0), (0, 400));
        // A drag of the thumb by 50 pixels scrolls by 50, up to the end.
        assert_eq!(drag_offset(400, 200, 400, (0, 50)), 50);
        assert_eq!(drag_offset(400, 200, 400, (100, 500)), 200);
        assert_eq!(drag_offset(400, 200, 400, (100, -500)), 0);
    }

    #[test]
    fn a_tile_shows_only_inside_the_view() {
        let (mut theme, cache) = crate::dialog::tests::theme("clip");
        let theme = &mut theme;
        let picture = Picture::new(214, 56);
        let view = [210, 86, 500, 400];
        assert_eq!(
            clip(theme, picture.clone(), [226, 100], view).map(|c| c.0),
            Some([226, 100, 214, 56])
        );
        assert_eq!(
            clip(theme, picture.clone(), [226, 60], view).map(|c| c.0),
            Some([226, 86, 214, 30])
        );
        assert_eq!(clip(theme, picture, [226, 500], view), None);
        let _ = std::fs::remove_dir_all(cache);
    }
}
