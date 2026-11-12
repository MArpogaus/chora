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
