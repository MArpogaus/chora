//! The SVG files of the XP theme that the pictures of the plugin name.

use chora_draw::theme::Sheet;

/// The window frame of Exposé.
pub const DECORATION: Sheet = Sheet("expose/decoration.svg");
/// The close button.
pub const CLOSE: Sheet = Sheet("expose/close.svg");
/// The maximize button.
pub const MAXIMIZE: Sheet = Sheet("expose/maximize.svg");
/// The minimize button.
pub const MINIMIZE: Sheet = Sheet("expose/minimize.svg");
/// The restore button of a maximized window.
pub const RESTORE: Sheet = Sheet("expose/restore.svg");
/// The XP artwork: start button, start menu, notification area.
pub const WINXP: Sheet = Sheet("winxp/winxp.svg");
/// The task buttons.
pub const TASKS: Sheet = Sheet("winxp/tasks.svg");

/// All of them, for [`chora_draw::theme::Theme::load`].
pub const ALL: [Sheet; 7] = [DECORATION, CLOSE, MAXIMIZE, MINIMIZE, RESTORE, WINXP, TASKS];

/// The family of the text of the theme.
pub const FAMILY: &str = "Tahoma";

#[cfg(test)]
pub mod tests {
    use std::path::{Path, PathBuf};

    use chora_draw::theme::{Picture, Theme};
    use resvg::tiny_skia::Pixmap;

    use super::*;

    /// The theme of the source tree, with a cache of its own for a test.
    pub fn theme(test: &str) -> (Theme, PathBuf) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/theme");
        let cache = std::env::temp_dir().join(format!("chora-xp-{test}-{}", std::process::id()));
        let theme = Theme::load(&dir, &cache, &ALL, FAMILY).map(Theme::large_chrome);
        (theme.expect("the theme loads"), cache)
    }

    /// The sides of the icon slots of the look.
    pub const ICON_SIZES: [i32; 4] = [16, 24, 32, 48];

    /// The audit kind `icon-size-in-{16,24,32,48}`: the file layers of a
    /// picture whose slot is no square of one of the sides.
    pub fn icon_size_findings(picture: &Picture) -> Vec<String> {
        picture
            .layers
            .iter()
            .filter_map(|layer| match layer {
                chora_draw::theme::Layer::File { path, rect, .. }
                    if !path.to_string_lossy().contains("arrow")
                        && (rect[2] != rect[3] || !ICON_SIZES.contains(&rect[2])) =>
                {
                    Some(format!(
                        "{} in a slot of {}x{}",
                        path.display(),
                        rect[2],
                        rect[3]
                    ))
                }
                _ => None,
            })
            .collect()
    }

    /// The audit kind `text-clipped-by-box`: the text layers of a picture
    /// that are wider than their box, which would end in an ellipsis or
    /// cross the edge of the box.
    pub fn clipped_text_findings(theme: &Theme, picture: &Picture) -> Vec<String> {
        picture
            .layers
            .iter()
            .filter_map(|layer| match layer {
                chora_draw::theme::Layer::Text(text)
                    if theme.text_width(&text.line, text.font) > f64::from(text.rect[2]) =>
                {
                    Some(format!(
                        "{:?} is wider than its box of {}",
                        text.line, text.rect[2]
                    ))
                }
                _ => None,
            })
            .collect()
    }

    /// A picture drawn at scale 1.
    pub fn draw(theme: &mut Theme, picture: &Picture) -> Pixmap {
        Pixmap::load_png(theme.render(picture)).expect("a PNG file")
    }

    /// The color of a pixel of an opaque picture.
    pub fn rgb(image: &Pixmap, x: i32, y: i32) -> [u8; 3] {
        let p = image
            .pixel(u32::try_from(x).expect("x"), u32::try_from(y).expect("y"))
            .expect("inside")
            .demultiply();
        [p.red(), p.green(), p.blue()]
    }

    /// A run of columns of an image with ink, as pixels: the box of the
    /// ink and how many pixels it has.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Ink {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
        pub count: i32,
    }

    impl Ink {
        pub fn width(&self) -> i32 {
            self.right - self.left
        }

        pub fn height(&self) -> i32 {
            self.bottom - self.top
        }
    }

    /// The runs of columns with ink in the image, divided by at least
    /// `gap` columns without.
    pub fn runs(image: &Pixmap, gap: i32, is_ink: impl Fn([u8; 3]) -> bool) -> Vec<Ink> {
        let (w, h) = (
            i32::try_from(image.width()).expect("w"),
            i32::try_from(image.height()).expect("h"),
        );
        let mut found: Vec<Ink> = Vec::new();
        let mut empty = gap;
        for x in 0..w {
            let ys: Vec<i32> = (0..h).filter(|y| is_ink(rgb(image, x, *y))).collect();
            if ys.is_empty() {
                empty += 1;
                continue;
            }
            let (top, bottom) = (ys[0], ys[ys.len() - 1] + 1);
            let count = i32::try_from(ys.len()).expect("count");
            match found.last_mut() {
                Some(run) if empty < gap => {
                    run.right = x + 1;
                    run.top = run.top.min(top);
                    run.bottom = run.bottom.max(bottom);
                    run.count += count;
                }
                _ => found.push(Ink {
                    left: x,
                    top,
                    right: x + 1,
                    bottom,
                    count,
                }),
            }
            empty = 0;
        }
        found
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn frames_have_the_sizes_of_their_parts() {
        let (mut theme, cache) = theme("frames");
        let frame = theme.frame(DECORATION, "decoration");
        assert_eq!(frame.slice, [34.0, 40.0, 34.0, 34.0]);
        let button = theme.frame(CLOSE, "active");
        assert_eq!(button.slice, [3.0, 3.0, 3.0, 3.0]);
        let image = Pixmap::load_png(&frame.image).expect("a PNG file");
        assert_eq!([image.width(), image.height()], [156, 162]);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_start_button_is_green() {
        let (mut theme, cache) = theme("start");
        let picture = Picture::new(100, 30).element(WINXP, "startbutton-normal", [0, 0, 100, 30]);
        let image = Pixmap::load_png(theme.render(&picture)).expect("a PNG file");
        let pixel = image.pixel(50, 15).expect("inside");
        assert!(pixel.green() > pixel.red() && pixel.green() > pixel.blue());
        let _ = std::fs::remove_dir_all(cache);
    }
}
