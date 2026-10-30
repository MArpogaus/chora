//! The wallpaper of each output: an image node at the bottom of the
//! scene, with the picture of the desktop in its position on the color of
//! the desktop, as Display Properties > Desktop chooses them.

use std::path::{Path, PathBuf};

use chora_scene::{Command, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::{
    config::Wallpaper,
    theme::{Fit, Picture, Rect, Theme},
};

/// XP's blue of the desktop, behind a centered picture or without one.
pub const DESKTOP_BLUE: [u8; 3] = [0x00, 0x4E, 0x98];

/// How the picture fills the screen: XP's Center, Tile and Stretch, and
/// Fill of later Windows, which covers the screen without distortion and
/// cuts the overflow.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Deserialize,
    serde::Serialize,
    schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    /// At its own size in the middle, on the color of the desktop.
    Center,
    /// At its own size, repeated from the top-left corner.
    Tile,
    /// Over the whole screen, out of its aspect ratio.
    Stretch,
    /// Over the whole screen, keeping its aspect ratio.
    #[default]
    Fill,
}

impl Position {
    /// The positions in the order of the list.
    pub const ALL: [Self; 4] = [Self::Center, Self::Tile, Self::Stretch, Self::Fill];

    /// The name in the list.
    pub fn name(self) -> &'static str {
        match self {
            Self::Center => "Center",
            Self::Tile => "Tile",
            Self::Stretch => "Stretch",
            Self::Fill => "Fill",
        }
    }
}

/// Where a picture of a size goes on a screen of a size: the rectangle of
/// the picture, or of the first tile.
pub fn place(position: Position, [iw, ih]: [i32; 2], [w, h]: [i32; 2]) -> Rect {
    match position {
        Position::Center => [(w - iw) / 2, (h - ih) / 2, iw, ih],
        Position::Tile => [0, 0, iw, ih],
        Position::Stretch => [0, 0, w, h],
        Position::Fill => {
            // The larger of the two scales, so that no edge stays empty.
            let wide = i64::from(iw) * i64::from(h) >= i64::from(ih) * i64::from(w);
            let [fw, fh] = if wide {
                [scale(iw, h, ih), h]
            } else {
                [w, scale(ih, w, iw)]
            };
            [(w - fw) / 2, (h - fh) / 2, fw, fh]
        }
    }
}

/// `n * by / of`, rounded.
fn scale(n: i32, by: i32, of: i32) -> i32 {
    let exact = (i64::from(n) * i64::from(by) * 2 + i64::from(of)) / (2 * i64::from(of.max(1)));
    i32::try_from(exact).unwrap_or(i32::MAX)
}

/// The desktop of a screen of a size, drawn into a picture of another
/// size: the wallpaper itself, or its preview in Display Properties.
#[allow(clippy::many_single_char_names)] // coordinates and colors
pub fn picture(theme: &Theme, style: &Wallpaper, screen: [i32; 2], [w, h]: [i32; 2]) -> Picture {
    let [r, g, b] = style.color;
    let picture = Picture::new(w, h).svg(format!(
        r#"<rect width="{w}" height="{h}" fill="rgb({r},{g},{b})"/>"#
    ));
    let Some(file) = &style.file(theme) else {
        return picture;
    };
    let Some(size) = theme.image_size(file) else {
        // An unknown size: the picture covers the screen.
        return picture.fit(file, [0, 0, w, h], Fit::Cover);
    };
    let [x, y, pw, ph] = place(style.position, size, screen);
    // From the screen to the picture.
    let sx = |n: i32| scale(n, w, screen[0]);
    let sy = |n: i32| scale(n, h, screen[1]);
    let rect = [sx(x), sy(y), sx(pw).max(1), sy(ph).max(1)];
    match style.position {
        Position::Tile => picture.fit(file, [0, 0, w, h], Fit::Tile([rect[2], rect[3]])),
        _ => picture.fit(file, rect, Fit::Stretch),
    }
}

/// The id of the wallpaper of an output.
pub fn id(output: &str) -> String {
    format!("xp-wall-{output}")
}

/// Whether a node is a wallpaper.
pub fn owns(node: &str) -> bool {
    node.starts_with("xp-wall-")
}

/// Creates the wallpaper of an output.
pub fn create(
    theme: &mut Theme,
    style: &Wallpaper,
    output: &OutputInfo,
    parent: &str,
) -> Vec<Command> {
    vec![
        Command::Create {
            id: id(&output.name),
            kind: NodeKind::Image,
            parent: parent.into(),
            props: PropsUpdate {
                input: Some(InputMode::Accept),
                ..PropsUpdate::default()
            },
        },
        update(theme, style, output),
    ]
}

/// Draws the desktop over the work area of its output.
pub fn update(theme: &mut Theme, style: &Wallpaper, output: &OutputInfo) -> Command {
    // Where the default camera shows it, so that a camera move keeps it
    // in place in the scene.
    let [x, y, w, h] = output.layout_area();
    let image = theme.render(&picture(theme, style, [w, h], [w, h]));
    Command::Update {
        id: id(&output.name),
        props: PropsUpdate {
            image: Some(image),
            transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
            ..PropsUpdate::default()
        },
    }
}

/// The directories of the wallpapers: XP's in the XDG data directories,
/// the theme's and the system's.
pub fn dirs(theme: &Theme) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = chora_kit::xdg::data_dirs()
        .into_iter()
        .map(|d| d.join("wallpapers/WindowsXP"))
        .collect();
    dirs.push(theme.path("wallpaper"));
    dirs.push(Path::new("/usr/share/backgrounds").into());
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_positions_place_the_picture_like_xp() {
        let screen = [1280, 800];
        // Center: its own size in the middle, also when it is larger.
        assert_eq!(
            place(Position::Center, [800, 600], screen),
            [240, 100, 800, 600]
        );
        assert_eq!(
            place(Position::Center, [1600, 1000], screen),
            [-160, -100, 1600, 1000]
        );
        // Tile: the first tile at the top-left corner.
        assert_eq!(place(Position::Tile, [32, 32], screen), [0, 0, 32, 32]);
        // Stretch: the whole screen.
        assert_eq!(
            place(Position::Stretch, [800, 600], screen),
            [0, 0, 1280, 800]
        );
        // Fill: the whole screen at the aspect ratio of the picture; the
        // overflow hangs out on both sides.
        assert_eq!(
            place(Position::Fill, [800, 600], screen),
            [0, -80, 1280, 960]
        );
        assert_eq!(
            place(Position::Fill, [1920, 800], screen),
            [-320, 0, 1920, 800]
        );
    }
}
