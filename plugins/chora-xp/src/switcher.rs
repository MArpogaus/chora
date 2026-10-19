//! XP's Alt+Tab: the icons of the windows, up to seven a row, in XP's
//! beige box with the selection frame around the chosen one and its
//! title in a sunken field below. The layout follows the `winxp_alttab`
//! switcher of `kde-plasma-winxp-launcher`.

use std::path::PathBuf;

use crate::{
    dialog::BLACK,
    text::{Font, Text},
    theme::{Picture, Rect, Sheet, Theme},
};

/// A cell of an icon.
const CELL: i32 = 43;
/// Icons a row.
const COLUMNS: i32 = 7;
/// The height of the title field.
const TITLE: i32 = 22;
/// The space between the icons and the title.
const GAP: i32 = 12;

/// A window of the switcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The surface.
    pub surface: String,
    /// The title.
    pub title: String,
    /// The icon file.
    pub icon: Option<PathBuf>,
}

/// The open switcher: the windows, the most recent first, and the chosen
/// one.
#[derive(Debug, PartialEq, Eq)]
pub struct Switcher {
    pub entries: Vec<Entry>,
    chosen: usize,
}

impl Switcher {
    /// A switcher over windows, the most recent first; the first step goes
    /// to the next window, or back to the last one.
    pub fn new(entries: Vec<Entry>, step: i32) -> Option<Self> {
        if entries.is_empty() {
            return None;
        }
        let mut switcher = Self { entries, chosen: 0 };
        switcher.step(step);
        Some(switcher)
    }

    /// Moves the choice to the next window, or with a negative step to the
    /// previous one, around the ends.
    pub fn step(&mut self, step: i32) {
        let n = self.entries.len();
        self.chosen = if step < 0 {
            (self.chosen + n - 1) % n
        } else {
            (self.chosen + 1) % n
        };
    }

    /// Drops a window that closed. Returns whether windows are left.
    pub fn remove(&mut self, surface: &str) -> bool {
        if let Some(i) = self.entries.iter().position(|e| e.surface == surface) {
            self.entries.remove(i);
            if i < self.chosen || self.chosen == self.entries.len() {
                self.chosen = self.chosen.saturating_sub(1);
            }
        }
        !self.entries.is_empty()
    }

    /// The chosen window.
    pub fn chosen(&self) -> &str {
        &self.entries[self.chosen].surface
    }

    /// The picture of the box.
    #[allow(clippy::many_single_char_names)] // coordinates
    pub fn picture(&self, theme: &Theme) -> Picture {
        let pad = theme
            .size(Sheet::Winxp, "alttab-hint-left-margin")
            .map_or(12, |s| s[0]);
        let inset = theme
            .size(Sheet::Winxp, "alttab-title-hint-left-margin")
            .map_or(3, |s| s[0]);
        let count = i32::try_from(self.entries.len()).unwrap_or(1);
        let columns = count.clamp(1, COLUMNS);
        let rows = (count + columns - 1) / columns;
        let [gw, gh] = [columns * CELL, rows * CELL];
        let [w, h] = [gw + 2 * pad, gh + GAP + TITLE + 2 * pad];
        let mut picture = Picture::new(w, h).frame(Sheet::Winxp, "alttab", [0, 0, w, h]);
        for (i, entry) in self.entries.iter().enumerate() {
            let i = i32::try_from(i).unwrap_or(0);
            let [x, y] = [pad + i % columns * CELL, pad + i / columns * CELL];
            if usize::try_from(i) == Ok(self.chosen) {
                picture = picture.frame(Sheet::Winxp, "alttab-selection", [x, y, CELL, CELL]);
            }
            if let Some(icon) = &entry.icon {
                let at = (CELL - 32) / 2;
                picture = picture.file(icon, [x + at, y + at, 32, 32]);
            }
        }
        let title: Rect = [pad, pad + gh + GAP, gw, TITLE];
        picture = picture.frame(Sheet::Winxp, "alttab-title", title);
        let [x, y, tw, th] = title;
        let text = &self.entries[self.chosen].title;
        picture
            .text(Text::new(text, Font::Ui, BLACK, [x + inset, y, tw - 2 * inset, th]).centered())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(n: usize) -> Vec<Entry> {
        (0..n)
            .map(|i| Entry {
                surface: format!("s{i}"),
                title: format!("window {i}"),
                icon: None,
            })
            .collect()
    }

    #[test]
    fn tab_goes_to_the_next_window_and_around() {
        let mut s = Switcher::new(entries(3), 1).expect("windows");
        assert_eq!(s.chosen(), "s1");
        s.step(1);
        s.step(1);
        assert_eq!(s.chosen(), "s0");
        // Shift+Tab goes back to the last one.
        let back = Switcher::new(entries(3), -1).expect("windows");
        assert_eq!(back.chosen(), "s2");
        assert!(Switcher::new(Vec::new(), 1).is_none());
        // A window that closes leaves; the choice stays on its window.
        let mut s = Switcher::new(entries(3), 1).expect("windows");
        s.step(1);
        assert!(s.remove("s0"));
        assert_eq!(s.chosen(), "s2");
        assert!(s.remove("s2"));
        assert_eq!(s.chosen(), "s1");
        assert!(!s.remove("s1"));
    }

    #[test]
    fn the_box_holds_seven_icons_a_row() {
        let (mut theme, cache) = crate::dialog::tests::theme("alttab");
        let mut list = entries(9);
        let mut icons = chora_kit::icons::Icons::new(crate::icons::THEMES);
        for entry in &mut list {
            entry.icon = icons.program_sized("", 32);
        }
        let s = Switcher::new(list, 1).expect("windows");
        let picture = s.picture(&theme);
        // CHORA_XP_PREVIEW=dir cargo test writes the box there.
        if let Some(dir) = std::env::var_os("CHORA_XP_PREVIEW") {
            let file = theme.render(&picture);
            std::fs::copy(file, std::path::Path::new(&dir).join("alttab.png")).expect("a copy");
        }
        let [w, h] = picture.size;
        let pad = theme
            .size(Sheet::Winxp, "alttab-hint-left-margin")
            .map_or(12, |s| s[0]);
        assert_eq!(
            [w, h],
            [7 * CELL + 2 * pad, 2 * CELL + GAP + TITLE + 2 * pad]
        );
        let _ = std::fs::remove_dir_all(cache);
    }
}
