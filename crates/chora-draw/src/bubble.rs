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
    /// The size of the text in pixels, read by chora-notify: the card has
    /// the line height of that size. Left out, the card has the small text
    /// of the host.
    pub size: Option<u32>,
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
            size: None,
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

    /// Whether the tip of the tail stands over the box, so that the
    /// outline of the box carries the tail itself.
    fn tail_inside(&self, width: i32, tail_x: i32) -> bool {
        tail_x <= width - self.radius
    }

    /// The tail of a tip right of the box, which leans out of the corner
    /// to reach it: a picture of its own, so that the box keeps its size.
    #[must_use]
    pub fn lean(&self, [width, height]: [i32; 2], tail_x: i32) -> Option<Lean> {
        if self.tail <= 0 || self.tail_inside(width, tail_x) {
            return None;
        }
        let base_right = width - self.radius;
        let at = [base_right - self.tail_base, height - 1];
        let size = [tail_x - at[0] + 1, self.tail + 1];
        let (left, right) = (0.5, f64::from(self.tail_base) + 0.5);
        let (tip_x, tip_y) = (f64::from(tail_x - at[0]) - 0.5, f64::from(self.tail) + 0.5);
        let [fill_r, fill_g, fill_b] = self.fill;
        let [line_r, line_g, line_b] = self.border;
        let svg = format!(
            r#"<path d="M{left},0 H{right} L{tip_x},{tip_y} Z" fill="rgb({fill_r},{fill_g},{fill_b})"/><path d="M{right},0.5 L{tip_x},{tip_y} L{left},0.5" fill="none" stroke="rgb({line_r},{line_g},{line_b})" stroke-width="1"/>"#
        );
        Some(Lean { at, size, svg })
    }

    /// The rounded rectangle of a size with the tail at the x of its tip
    /// as one outline, so that the border runs around both; a tip right of
    /// the box has no tail here but in [`Style::lean`]; SVG elements
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
        if self.tail > 0 && self.tail_inside(width, tail_x) {
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

/// A tail that leans out of its box, as a picture below the box.
pub struct Lean {
    /// Its top-left corner from the top-left corner of the box.
    pub at: [i32; 2],
    /// Its size.
    pub size: [i32; 2],
    /// Its SVG elements for [`crate::theme::Picture::svg`].
    pub svg: String,
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

    /// rxt-3: the leaning tail is a wedge with a body, not a sliver: its
    /// thickness at the widest column is at least 12 px.
    #[test]
    fn the_leaning_tail_is_a_wedge_and_not_a_sliver() {
        let style = Style {
            tail: 18,
            ..Style::default()
        };
        for tip in [110_u32, 130, 160] {
            let lean = style
                .lean([100, 40], tip.cast_signed())
                .expect("a leaning tail");
            let path = lean.svg.split('"').nth(1).expect("the fill path");
            let n: Vec<f64> = path
                .split(|c: char| !(c.is_ascii_digit() || c == '.'))
                .filter_map(|t| t.parse().ok())
                .collect();
            let (left, right, tx, ty) = (n[0], n[2], n[3], n[4]);
            let widest = (0..=tip + 1)
                .map(|x| {
                    let x = f64::from(x) + 0.5;
                    if x < left || x > tx {
                        return 0.0;
                    }
                    let lower = ty * (x - left) / (tx - left);
                    let upper = if x <= right {
                        0.0
                    } else {
                        ty * (x - right) / (tx - right)
                    };
                    lower - upper
                })
                .fold(0.0, f64::max);
            assert!(widest >= 12.0, "tip {tip}: widest {widest} px");
        }
    }

    #[test]
    fn a_tip_right_of_the_box_leans_out_in_a_picture_of_its_own() {
        let style = Style {
            tail: 18,
            ..Style::default()
        };
        assert!(
            !style.shape([100, 40], 150).contains('L'),
            "no tail in the box"
        );
        let lean = style.lean([100, 40], 150).expect("a leaning tail");
        assert_eq!(lean.at[1], 39, "it starts on the bottom edge");
        assert_eq!(lean.at[0] + lean.size[0], 151, "it reaches the tip");
        assert!(
            style.lean([100, 40], 60).is_none(),
            "a tip over the box stays in it"
        );
    }
}
