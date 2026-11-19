//! The answer of `scene.boxes`: what each output shows, as boxes in the
//! logical pixels of the output, bottom first. It is read-only and holds
//! no policy: a check such as "no text overlaps" or "the faces of a cube
//! meet" reads it and decides itself.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Node, NodeKind, Order, Transform};

/// Where a box sits in the stack of an output, bottom first: the layer
/// surfaces below the scene, the scene, the overlay of the output, and
/// the layer surfaces above them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// Layer surfaces of the `background` layer, such as a wallpaper.
    Background,
    /// Layer surfaces of the `bottom` layer.
    Bottom,
    /// The nodes below the root, through the camera of the output.
    Scene,
    /// The nodes under the overlay root of the output, without its camera
    /// (D43).
    Overlay,
    /// Layer surfaces of the `top` layer, such as a panel.
    Top,
    /// Layer surfaces of the `overlay` layer.
    ShellOverlay,
}

/// One box that an output shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeBox {
    /// The node; `None` for a layer surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// The kind of the node; `None` for a layer surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<NodeKind>,
    /// The client name of the owner of the node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// The namespace of a layer surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Where the box sits in the stack of the output.
    pub layer: Layer,
    /// The order of the root group that holds a node of the scene (D89).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<Order>,
    /// `[x, y, w, h]`, as the `rects` of `scene.query` give it.
    pub rect: [f64; 4],
    /// Whether the node covers its box with opaque pixels, so that
    /// nothing below it shows there (see [`opaque`]).
    pub opaque: bool,
    /// The four corners of a node that draws as a plane: top left, top
    /// right, bottom right, bottom left of its size, projected (see
    /// [`corners`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corners: Option<[[f64; 2]; 4]>,
    /// The text of a text node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The font size of a text node in its own logical pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
}

/// The boxes of one output.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OutputBoxes {
    /// The logical size of the output.
    pub size: [f64; 2],
    /// The boxes, bottom first.
    pub boxes: Vec<NodeBox>,
}

/// The answer of `scene.boxes`, by output name.
pub type Boxes = BTreeMap<String, OutputBoxes>;

/// Whether a placed node covers its box with opaque pixels: it shows at
/// full opacity after its ancestors, and it is a window, or a rect with
/// an opaque fill, square corners and no blurred frame behind it. Images,
/// text, shaders and layer surfaces can be clear anywhere and count as
/// not opaque.
#[must_use]
pub fn opaque(node: &Node, opacity: f64) -> bool {
    if opacity < 1.0 {
        return false;
    }
    let props = &node.props;
    match node.kind {
        NodeKind::Surface => true,
        NodeKind::Rect => {
            props.color.is_some_and(|c| c[3] >= 1.0)
                && props.radius <= 0.0
                && props.backdrop.is_none()
        }
        NodeKind::Group | NodeKind::Image | NodeKind::Shader | NodeKind::Text => false,
    }
}

/// The corners of a quad of `size` after `transform`, divided by w: top
/// left, top right, bottom right, bottom left. `None` when a corner lies
/// at or behind the near plane `w = near`, since the quad does not show
/// whole there.
#[must_use]
pub fn corners(transform: &Transform, [w, h]: [f64; 2], near: f64) -> Option<[[f64; 2]; 4]> {
    let m = transform.matrix();
    let quad = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
    let mut out = [[0.0; 2]; 4];
    for (corner, [x, y]) in out.iter_mut().zip(quad) {
        let depth = m[3] * x + m[7] * y + m[15];
        if depth <= near {
            return None;
        }
        *corner = [
            (m[0] * x + m[4] * y + m[12]) / depth,
            (m[1] * x + m[5] * y + m[13]) / depth,
        ];
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Props;

    fn node(kind: NodeKind, props: Props) -> Node {
        Node {
            id: "n".into(),
            kind,
            owner: Some("wm".into()),
            parent: Some(crate::ROOT.into()),
            children: Vec::new(),
            props,
            surface: None,
            to: None,
        }
    }

    fn rect(color: [f64; 4]) -> Props {
        Props {
            size: Some([10.0, 10.0]),
            color: Some(color),
            ..Props::default()
        }
    }

    #[test]
    fn an_opaque_fill_at_full_opacity_is_opaque() {
        assert!(opaque(
            &node(NodeKind::Rect, rect([0.0, 0.0, 0.0, 1.0])),
            1.0
        ));
        assert!(opaque(&node(NodeKind::Surface, Props::default()), 1.0));
    }

    #[test]
    fn a_clear_fill_round_corners_a_backdrop_or_fading_are_not_opaque() {
        let clear = node(NodeKind::Rect, rect([0.0, 0.0, 0.0, 0.5]));
        let round = node(
            NodeKind::Rect,
            Props {
                radius: 4.0,
                ..rect([0.0, 0.0, 0.0, 1.0])
            },
        );
        let full = node(NodeKind::Rect, rect([0.0, 0.0, 0.0, 1.0]));
        assert!(!opaque(&clear, 1.0), "a fill with alpha");
        assert!(!opaque(&round, 1.0), "round corners show what is below");
        assert!(!opaque(&full, 0.99), "an ancestor fades it");
        assert!(
            !opaque(&node(NodeKind::Rect, Props::default()), 1.0),
            "no fill"
        );
    }

    #[test]
    fn images_text_and_shaders_are_never_opaque() {
        for kind in [
            NodeKind::Image,
            NodeKind::Text,
            NodeKind::Shader,
            NodeKind::Group,
        ] {
            assert!(!opaque(&node(kind, rect([1.0; 4])), 1.0), "{kind:?}");
        }
    }

    #[test]
    fn the_corners_of_a_plain_quad_are_its_translated_rect() {
        let at = Transform::translate(5.0, 7.0, 0.0);
        assert_eq!(
            corners(&at, [10.0, 4.0], 0.0),
            Some([[5.0, 7.0], [15.0, 7.0], [15.0, 11.0], [5.0, 11.0]])
        );
    }

    #[test]
    fn the_corners_divide_by_the_depth_of_a_perspective() {
        let lens = Transform::perspective([0.0, 0.0], 100.0);
        let pushed = lens.then(&Transform::translate(0.0, 0.0, 100.0));
        let [a, b, c, d] = corners(&pushed, [10.0, 10.0], 0.0).expect("in front");
        assert_eq!(
            [a, b, c, d],
            [[0.0, 0.0], [5.0, 0.0], [5.0, 5.0], [0.0, 5.0]]
        );
    }

    #[test]
    fn a_quad_through_the_near_plane_has_no_corners() {
        let lens = Transform::perspective([0.0, 0.0], 100.0);
        let behind = lens.then(&Transform::translate(0.0, 0.0, -150.0));
        assert_eq!(corners(&behind, [10.0, 10.0], 1.0 / 64.0), None);
    }
}
