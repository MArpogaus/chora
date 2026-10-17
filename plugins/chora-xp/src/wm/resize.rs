//! The resize of a window from the edges and corners of its frame: the
//! drag asks for the new size once per frame of the effects, and the
//! opposite edges stay where they are.

use chora_scene::Command;

use super::{Resize, Wm};
use crate::{theme::Rect, window};

impl Wm {
    /// A press on the border of a window starts a resize. A window that
    /// still wobbles stops: it does not wobble while it resizes.
    pub(super) fn start_resize(
        &mut self,
        surface: &str,
        edges: window::Edges,
        from: Rect,
        start: [f64; 2],
    ) -> Vec<Command> {
        let mut commands = Vec::new();
        if self.effects.wobbles.remove(surface).is_some()
            && let Some(i) = self.index(surface)
        {
            commands.push(self.windows[i].unmesh());
        }
        self.resize = Some(Resize {
            surface: surface.into(),
            edges,
            start,
            from,
            pending: None,
            target: from,
            asked: None,
            released: false,
        });
        commands
    }

    /// The frame of a resize with the pointer at a point.
    pub(super) fn resize_target(&self, r: &Resize, [px, py]: [f64; 2]) -> Rect {
        let min = self
            .index(&r.surface)
            .map_or([1, 1], |i| self.windows[i].min_frame());
        #[allow(clippy::cast_possible_truncation)]
        let delta = [
            (px - r.start[0]).round() as i32,
            (py - r.start[1]).round() as i32,
        ];
        window::resized(r.from, r.edges, delta, min)
    }

    /// The release of the button ends a resize at the last frame that
    /// the pointer wants; `None` without a resize.
    pub(super) fn release_resize(&mut self, position: [f64; 2]) -> Option<Vec<Command>> {
        let r = self.resize.as_ref().filter(|r| !r.released)?;
        let target = self.resize_target(r, position);
        if let Some(r) = &mut self.resize {
            r.released = true;
            if r.target != target {
                r.target = target;
                r.pending = Some(target);
            }
        }
        let commands = self.resize_frame();
        if let Some(i) = self.resize.as_ref().and_then(|r| self.index(&r.surface)) {
            self.anchor(i);
        }
        Some(commands)
    }

    /// Whether a resize waits for its next frame.
    pub(super) fn resize_due(&self) -> bool {
        self.resize.as_ref().is_some_and(|r| r.pending.is_some())
    }

    /// Asks for the frame that the pointer wants: a configure to a
    /// client, or the new layout of an Explorer window.
    pub(super) fn resize_frame(&mut self) -> Vec<Command> {
        let Some(r) = &mut self.resize else {
            return Vec::new();
        };
        let (Some(rect), surface) = (r.pending.take(), r.surface.clone()) else {
            return Vec::new();
        };
        let Some(i) = self.index(&surface) else {
            return Vec::new();
        };
        let active = self.is_active(&surface);
        let mut commands = Vec::new();
        match self.windows[i].resize_to(rect, active) {
            Some(Command::Configure { size, .. }) if size == self.resize_asked() => {}
            Some(configure) => {
                if let (Some(r), Command::Configure { size, .. }) = (&mut self.resize, &configure) {
                    r.asked = *size;
                }
                commands.push(configure);
            }
            // A window of the plugin takes its frame at once.
            None => {
                commands.extend(self.windows[i].style(&mut self.theme, active));
                commands.extend(self.relayout_explorers());
            }
        }
        commands
    }

    fn resize_asked(&self) -> Option<[i32; 2]> {
        self.resize.as_ref().and_then(|r| r.asked)
    }

    /// A client committed a size during or right after a resize: its
    /// frame keeps the edges that the drag does not move. The resize ends
    /// with the commit of the last size it asked for.
    pub(super) fn anchor(&mut self, i: usize) {
        let win = &mut self.windows[i];
        let Some(resize) = self.resize.as_ref().filter(|r| r.surface == win.surface) else {
            return;
        };
        let ([x, y, fw, fh], [ow, oh]) = (resize.from, win.outer());
        win.pos = [
            if resize.edges.left { x + fw - ow } else { x },
            if resize.edges.top { y + fh - oh } else { y },
        ];
        if resize.released && resize.asked.is_none_or(|asked| Some(asked) == win.size) {
            self.resize = None;
        }
    }
}
