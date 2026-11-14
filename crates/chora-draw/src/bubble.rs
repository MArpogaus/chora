//! The bubble that blocks draw around text: a rounded rectangle with a
//! border and, if the values of the asset folder ask for it, a tail below
//! it. Also where a block puts a picture on an output: a corner and a
//! margin.

use std::fmt::Write as _;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A corner of an output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    /// The top left corner.
    TopLeft,
    /// The top right corner.
    TopRight,
    /// The bottom left corner.
    BottomLeft,
    /// The bottom right corner.
    BottomRight,
}

impl Corner {
    /// The top-left corner of a picture of a size in this corner of an
    /// output of a size, with the margin to the edges.
    #[must_use]
    pub fn origin(self, screen: [i32; 2], size: [i32; 2], margin: [i32; 2]) -> [i32; 2] {
        let left = margin[0];
        let right = screen[0] - margin[0] - size[0];
        let top = margin[1];
        let bottom = screen[1] - margin[1] - size[1];
        match self {
            Self::TopLeft => [left, top],
            Self::TopRight => [right, top],
            Self::BottomLeft => [left, bottom],
            Self::BottomRight => [right, bottom],
        }
    }

    /// Whether the corner is at the right edge.
    #[must_use]
    pub fn right(self) -> bool {
        matches!(self, Self::TopRight | Self::BottomRight)
    }
}

/// The look of a bubble: values of an asset folder. A key that the file
/// leaves out takes the neutral value: a light card without a tail.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Style {
    /// Red, green and blue of the inside.
    pub fill: [u8; 3],
    /// Red, green and blue of the one pixel border.
    pub border: [u8; 3],
    /// Red, green and blue of the text.
    pub text: [u8; 3],
    /// The radius of the corners.
    pub radius: i32,
    /// The height of the tail below the bubble; 0 for none.
    pub tail: i32,
    /// The width of the tail at the bubble.
    pub tail_base: i32,
    /// The distance of the tip of the tail from the edge of the bubble
    /// that is nearer to its corner of the output.
    pub tail_inset: i32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: [250, 250, 250],
            border: [140, 140, 140],
            text: [32, 32, 32],
            radius: 8,
            tail: 0,
            tail_base: 16,
            tail_inset: 40,
        }
    }
}

impl Style {
    /// The text color with full opacity.
    #[must_use]
    pub fn text_rgba(&self) -> [u8; 4] {
        let [r, g, b] = self.text;
        [r, g, b, 255]
    }

    /// The x of the tip of the tail in a bubble of a width, for the
    /// corner of the output that the bubble sits in.
    #[must_use]
    pub fn tail_x(&self, width: i32, corner: Corner) -> i32 {
        let x = if corner.right() {
            width - self.tail_inset
        } else {
            self.tail_inset
        };
        x.clamp(
            self.radius + self.tail_base,
            (width - self.radius).max(self.radius + self.tail_base),
        )
    }

    /// The rounded rectangle of a size with the tail at the x of its tip
    /// as one outline, so that the border runs around both; SVG elements
    /// for [`crate::theme::Picture::svg`].
    #[must_use]
    pub fn shape(&self, [width, height]: [i32; 2], tail_x: i32) -> String {
        let (right, bottom) = (f64::from(width) - 0.5, f64::from(height) - 0.5);
        let radius = f64::from(self.radius);
        let near = 0.5 + radius;
        let (inner_x, inner_y) = (right - radius, bottom - radius);
        let arc = format!("A{radius},{radius} 0 0 1");
        let mut path = format!(
            "M{near},0.5 H{inner_x} {arc} {right},{near} V{inner_y} {arc} {inner_x},{bottom} "
        );
        if self.tail > 0 {
            let tip_x = f64::from(tail_x) - 0.5;
            let tip_y = bottom + f64::from(self.tail);
            let base_x = tip_x - f64::from(self.tail_base);
            let _ = write!(path, "H{tip_x} V{tip_y} L{base_x},{bottom} ");
        }
        let _ = write!(
            path,
            "H{near} {arc} 0.5,{inner_y} V{near} {arc} {near},0.5 Z"
        );
        let [fill_r, fill_g, fill_b] = self.fill;
        let [line_r, line_g, line_b] = self.border;
        format!(
            r#"<path d="{path}" fill="rgb({fill_r},{fill_g},{fill_b})" stroke="rgb({line_r},{line_g},{line_b})" stroke-width="1"/>"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_picture_sits_in_its_corner() {
        let at = |corner: Corner| corner.origin([1000, 800], [200, 100], [10, 20]);
        assert_eq!(at(Corner::TopLeft), [10, 20]);
        assert_eq!(at(Corner::TopRight), [790, 20]);
        assert_eq!(at(Corner::BottomLeft), [10, 680]);
        assert_eq!(at(Corner::BottomRight), [790, 680]);
    }

    #[test]
    fn the_tail_stays_between_the_corners() {
        let style = Style {
            tail: 18,
            ..Style::default()
        };
        assert_eq!(style.tail_x(200, Corner::BottomRight), 160);
        assert_eq!(style.tail_x(200, Corner::BottomLeft), 40);
        assert_eq!(style.tail_x(50, Corner::BottomRight), 24);
    }

    #[test]
    fn the_outline_has_a_tail_only_when_asked() {
        let plain = Style::default().shape([100, 40], 60);
        let tailed = Style {
            tail: 18,
            ..Style::default()
        }
        .shape([100, 40], 60);
        assert!(
            !plain.contains('L') && tailed.contains("V57.5 L43.5,39.5"),
            "{tailed}"
        );
    }
}
