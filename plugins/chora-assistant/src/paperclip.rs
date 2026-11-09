//! The paperclip of the XP flavor: a wire with eyes and eyebrows, drawn in
//! code. It is drawn for chora, an original character; no artwork of
//! Microsoft's Office assistant was used. The look moves into the asset
//! folder of the XP flavor as SVG files with the XP step X7.

use chora_draw::theme::Picture;

use crate::assistant::Pose;

/// The size of the picture.
pub const SIZE: [i32; 2] = [80, 116];
/// The point that the body turns around: the bottom of the wire.
pub const PIVOT: [f64; 2] = [41.0, 110.0];
/// Where the tail of the balloon points: above the eyes.
pub const HEAD: [i32; 2] = [38, 10];

/// The face of a pose.
struct Face {
    /// The height of the eyes, 1 for the normal one.
    eyes: f64,
    /// Where the pupils look, from the middle of the eyes.
    look: [f64; 2],
    /// How far each eyebrow goes up, left and right.
    brows: [f64; 2],
}

impl Pose {
    fn face(self) -> Face {
        let (eyes, look, brows) = match self {
            Self::Idle => (1.0, [0.5, 1.0], [0.0, 0.0]),
            Self::Listening => (1.18, [-1.5, 0.0], [2.0, 2.0]),
            Self::Thinking => (1.0, [2.0, -3.6], [-1.0, 3.5]),
            Self::Speaking => (1.0, [-2.0, 0.5], [1.0, 1.0]),
            Self::Asking => (1.1, [0.0, -0.5], [6.0, 6.0]),
            Self::Sleeping => (1.0, [0.0, 0.0], [-1.5, -1.5]),
        };
        Face { eyes, look, brows }
    }

    /// How far the picture leans, in degrees, to the right.
    fn tilt(self) -> f64 {
        match self {
            Self::Listening => -6.0,
            Self::Asking => 4.0,
            Self::Sleeping => 3.0,
            _ => 0.0,
        }
    }
}

/// The wire of the paperclip: three bends, the small one inside the
/// large one.
const WIRE: &str =
    "M33,48 V84 A8.5,8.5 0 0 0 50,84 V34 A14,14 0 0 0 22,34 V90 A19,19 0 0 0 60,90 V44";

/// The paperclip in a pose, with its eyes open or shut.
pub fn draw(pose: Pose, shut: bool) -> Picture {
    use std::fmt::Write as _;
    let shut = shut || pose == Pose::Sleeping;
    let face = pose.face();
    let (tilt, [px, py]) = (pose.tilt(), PIVOT);
    let mut svg = format!(
        r##"<defs><linearGradient id="assistant-metal" x1="0" x2="1" y1="0" y2="0.3"><stop offset="0" stop-color="#f4f6f9"/><stop offset="0.45" stop-color="#b9c0ca"/><stop offset="1" stop-color="#8a929e"/></linearGradient></defs><ellipse cx="41" cy="111" rx="20" ry="3.2" fill="#000" opacity="0.22"/><g transform="rotate({tilt} {px} {py})"><path d="{WIRE}" fill="none" stroke="#3d434c" stroke-width="7" stroke-linecap="round" stroke-linejoin="round"/><path d="{WIRE}" fill="none" stroke="url(#assistant-metal)" stroke-width="4.4" stroke-linecap="round"/><path d="{WIRE}" fill="none" stroke="#fff" stroke-opacity="0.75" stroke-width="1.1" stroke-linecap="round" transform="translate(-0.9,-0.6)"/>"##
    );
    let [lx, ly] = face.look;
    for ((cx, side), lift) in [(30.5, -1.0), (50.5, 1.0)].into_iter().zip(face.brows) {
        let (cy, rx, ry) = (37.0, 8.0, 9.0 * face.eyes);
        let _ = if shut {
            write!(
                svg,
                r##"<path d="M{},{} Q{cx},{} {},{}" fill="none" stroke="#1d1f24" stroke-width="1.8" stroke-linecap="round"/>"##,
                cx - rx + 1.0,
                cy + 1.0,
                cy + 6.0,
                cx + rx - 1.0,
                cy + 1.0
            )
        } else {
            write!(
                svg,
                r##"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="#fff" stroke="#1d1f24" stroke-width="1.5"/><circle cx="{}" cy="{}" r="3.4" fill="#16181c"/><circle cx="{}" cy="{}" r="1.1" fill="#fff"/>"##,
                cx + lx,
                cy + ly + 1.0,
                cx + lx + 1.2,
                cy + ly - 0.2
            )
        };
        let y = cy - ry - 5.0 - lift;
        let _ = write!(
            svg,
            r##"<path d="M{},{y} Q{cx},{} {},{}" fill="none" stroke="#2b2118" stroke-width="2.6" stroke-linecap="round"/>"##,
            cx - 5.0 * side,
            y - 3.0,
            cx + 6.5 * side,
            y + 1.0
        );
    }
    svg.push_str("</g>");
    Picture::new(SIZE[0], SIZE[1]).svg(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sleeping_paperclip_has_its_eyes_shut() {
        assert_eq!(
            draw(Pose::Sleeping, false),
            draw(Pose::Sleeping, true),
            "eyes shut"
        );
        assert_ne!(draw(Pose::Idle, false), draw(Pose::Idle, true));
    }
}
