//! XP's tooltip: a pale yellow box with a black 1 pixel border and one line
//! of text, which shows above a part of the taskbar after the pointer rested
//! on it, and goes when the pointer leaves or presses.

use std::time::{Duration, Instant};

use chora_scene::{Command, NodeKind, PropsUpdate, Transform};

use crate::{
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The id of the node of the tooltip.
pub const ID: &str = "xp-tooltip";
/// How long the pointer rests before the tooltip shows, as on XP.
pub const DELAY: Duration = Duration::from_millis(500);
const FILL: &str = "#FFFFE1";
const BORDER: &str = "#000000";
const BLACK: [u8; 4] = [0, 0, 0, 255];
const PAD_X: i32 = 8;
const HEIGHT: i32 = 28;
const GAP: i32 = 4;

/// What waits to show, and what shows.
#[derive(Default)]
pub struct Tooltip {
    pending: Option<Pending>,
    shown: bool,
}

struct Pending {
    due: Instant,
    node: String,
    text: String,
    target: Rect,
}

impl Tooltip {
    /// The pointer rests on a node whose tooltip is `text`, above `target`.
    pub fn rest(&mut self, node: &str, text: String, target: Rect, now: Instant) {
        self.pending = Some(Pending {
            due: now + DELAY,
            node: node.into(),
            text,
            target,
        });
    }

    /// The pointer left a node or pressed: no tooltip waits, and the one that
    /// shows goes.
    pub fn cancel(&mut self, node: Option<&str>) -> Vec<Command> {
        if node.is_some_and(|n| self.pending.as_ref().is_none_or(|p| p.node != n)) {
            return Vec::new();
        }
        self.pending = None;
        if std::mem::take(&mut self.shown) {
            return vec![Command::Remove {
                id: ID.into(),
                to: None,
            }];
        }
        Vec::new()
    }

    /// When the tooltip shows.
    pub fn due(&self) -> Option<Instant> {
        self.pending.as_ref().filter(|_| !self.shown).map(|p| p.due)
    }

    /// The commands that show the tooltip when its time came, in an area.
    pub fn show(
        &mut self,
        theme: &mut Theme,
        parent: &str,
        area: Rect,
        now: Instant,
    ) -> Vec<Command> {
        let Some(pending) = self
            .pending
            .as_ref()
            .filter(|p| now >= p.due && !self.shown)
        else {
            return Vec::new();
        };
        let (picture, size) = picture(theme, &pending.text);
        let [x, y] = place(pending.target, size, area);
        let image = theme.render(&picture);
        self.shown = true;
        vec![
            Command::Create {
                id: ID.into(),
                kind: NodeKind::Image,
                parent: parent.into(),
                props: PropsUpdate {
                    image: Some(image),
                    transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
                    ..PropsUpdate::default()
                },
            },
            Command::Move {
                id: ID.into(),
                parent: None,
                index: None,
            },
        ]
    }
}

/// The picture of a tooltip and its size.
pub fn picture(theme: &Theme, text: &str) -> (Picture, [i32; 2]) {
    #[allow(clippy::cast_possible_truncation)]
    let width = theme.text_width(text, Font::Body).ceil() as i32 + 2 * PAD_X;
    let svg = format!(
        r#"<rect width="{width}" height="{HEIGHT}" fill="{FILL}"/><rect x="0.5" y="0.5" width="{}" height="{}" fill="none" stroke="{BORDER}"/>"#,
        width - 1,
        HEIGHT - 1
    );
    let picture = Picture::new(width, HEIGHT).svg(svg).text(Text::new(
        text,
        Font::Body,
        BLACK,
        [PAD_X, 0, width - PAD_X, HEIGHT],
    ));
    (picture, [width, HEIGHT])
}

/// The top-left corner of a tooltip of a size over a target, centered above
/// it with a gap, inside an area; below the target when there is no room
/// above.
pub fn place(target: Rect, [w, h]: [i32; 2], area: Rect) -> [i32; 2] {
    let [tx, ty, tw, th] = target;
    let [ax, ay, aw, ah] = area;
    let x = (tx + (tw - w) / 2).clamp(ax, (ax + aw - w).max(ax));
    let above = ty - h - GAP;
    let y = if above >= ay { above } else { ty + th + GAP };
    [x, y.clamp(ay, (ay + ah - h).max(ay))]
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = [0, 0, 1280, 800];

    #[test]
    fn the_tooltip_stands_centered_above_the_target() {
        assert_eq!(place([600, 770, 24, 30], [100, 28], AREA), [562, 738]);
    }

    #[test]
    fn the_tooltip_stays_inside_the_area() {
        assert_eq!(place([1270, 770, 10, 30], [100, 28], AREA)[0], 1180);
        assert_eq!(place([0, 770, 10, 30], [100, 28], AREA)[0], 0);
    }

    #[test]
    fn without_room_above_it_goes_below() {
        assert_eq!(place([100, 10, 24, 24], [100, 28], AREA)[1], 38);
    }

    #[test]
    fn a_tooltip_waits_for_its_delay_and_a_leave_cancels_it() {
        let mut tip = Tooltip::default();
        let now = Instant::now();
        assert!(tip.due().is_none());
        tip.rest("xp-pager-1", "Desktop 2 of 4".into(), [0, 0, 10, 10], now);
        assert_eq!(tip.due(), Some(now + DELAY));
        assert_eq!(tip.cancel(Some("xp-pager-2")).len(), 0);
        assert!(tip.due().is_some());
        assert_eq!(tip.cancel(Some("xp-pager-1")).len(), 0);
        assert!(tip.due().is_none());
    }
}
