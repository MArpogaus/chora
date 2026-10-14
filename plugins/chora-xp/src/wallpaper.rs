//! The wallpaper of each output: an image node at the bottom of the
//! scene, with the picture scaled to cover the work area.

use std::path::Path;

use chora_scene::{Command, InputMode, NodeKind, OutputInfo, PropsUpdate, Transform};

use crate::theme::{Picture, Theme};

/// The id of the wallpaper of an output.
pub fn id(output: &str) -> String {
    format!("xp-wall-{output}")
}

/// Whether a node is a wallpaper.
pub fn owns(node: &str) -> bool {
    node.starts_with("xp-wall-")
}

/// Creates the wallpaper of an output.
pub fn create(theme: &mut Theme, file: &Path, output: &OutputInfo, parent: &str) -> Vec<Command> {
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
        update(theme, file, output),
    ]
}

/// Fits the wallpaper to the work area of its output.
pub fn update(theme: &mut Theme, file: &Path, output: &OutputInfo) -> Command {
    // Where the default camera shows it, so that a camera move keeps it
    // in place in the scene.
    let [x, y, w, h] = output.layout_area();
    let image = theme.render(&Picture::new(w, h).cover(file));
    Command::Update {
        id: id(&output.name),
        props: PropsUpdate {
            image: Some(image),
            transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
            ..PropsUpdate::default()
        },
    }
}

/// The PNG file of the wallpaper at a size, for the faces of the cube.
pub fn image(theme: &mut Theme, file: &Path, [w, h]: [i32; 2]) -> String {
    theme.render(&Picture::new(w, h).cover(file))
}
