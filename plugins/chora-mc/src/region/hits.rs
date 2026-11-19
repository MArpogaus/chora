//! The hits and close marks of a region: the nodes that take the pointer,
//! the focus mark under it and the clicks that they answer.

use super::{
    ClientError, Command, Ink, Item, Mc, NodeKind, PropsUpdate, Put, Transform, move_to, rect,
};

/// The id of the focus mark of a hit.
pub fn mark_id(base: &str) -> String {
    format!("{base}-mark")
}

/// The transparent rect that takes the pointer over something that a
/// click acts on, and the focus mark that shows beside it under the
/// pointer: a bar of 2 pixels in the color `ink`, as high as the capitals that it
/// marks, which `hit_place` sets.
pub fn hit_nodes(base: &str, parent: &str, ink: [f64; 4]) -> [Command; 2] {
    let mark = PropsUpdate {
        input: Some(chora_scene::InputMode::Pass),
        opacity: Some(0.0),
        ..PropsUpdate::default()
    };
    [
        rect(
            &format!("{base}-hit"),
            parent,
            [0.0, 0.0, 1.0, 1.0],
            [0.0; 4],
        ),
        Command::Create {
            id: mark_id(base),
            kind: NodeKind::Rect,
            parent: parent.into(),
            props: PropsUpdate {
                size: Some([MARK_WIDTH, MARK_WIDTH]),
                color: Some(ink),
                ..mark
            },
        },
    ]
}

/// Sizes the hit of an item to its box, and puts its mark 12 pixels left
/// of the item at `item_x`, as high as the capitals of its text, which
/// start `top` pixels below the box.
pub fn hit_place(base: &str, [x, y, w, h]: [f64; 4], item_x: f64, ink: Ink) -> [Command; 4] {
    [
        move_to(format!("{base}-hit"), x, y),
        Command::Update {
            id: format!("{base}-hit"),
            props: PropsUpdate {
                size: Some([w.max(1.0), h.max(1.0)]),
                ..PropsUpdate::default()
            },
        },
        move_to(mark_id(base), item_x - MARK_GAP, y + ink.top),
        Command::Update {
            id: mark_id(base),
            props: PropsUpdate {
                size: Some([MARK_WIDTH, ink.height.max(1.0)]),
                ..PropsUpdate::default()
            },
        },
    ]
}

/// The side of the close mark of an urgent line and its hit; the bars are
/// as thick as the hairline of the hover mark.
pub const CLOSE: f64 = 14.0;
/// The space between the close mark and the words.
const CLOSE_GAP: f64 = 8.0;

/// The left edge of an item of a side: flush right at `flush`, right
/// after the item before it at `after`, else at its own `x`.
pub(super) fn left_of(item: &Item, flush: f64, after: Option<f64>) -> f64 {
    match item.put {
        Put::End => flush.max(0.0),
        Put::After => after.unwrap_or(item.x),
        _ => item.x,
    }
}

/// The room that an item keeps before its text for its close mark.
pub(super) fn lead(item: &Item) -> f64 {
    if item.mark { CLOSE + CLOSE_GAP } else { 0.0 }
}

/// The id of a bar of the close mark of an item: `n` is 0 or 1.
pub fn close_id(base: &str, n: usize) -> String {
    format!("{base}-close-{n}")
}

/// The two bars of the close mark of an item, if it has one, in the color
/// of its text; `close_place` crosses them at 45 degrees.
pub(super) fn close_nodes(item: &Item, parent: &str, fallback: [f64; 4]) -> Vec<Command> {
    let color = item.props.color.unwrap_or(fallback);
    (0..2)
        .filter(|_| item.mark)
        .map(|n| {
            let mut bar = rect(
                &close_id(&item.id, n),
                parent,
                [0.0, 0.0, CLOSE, 2.0],
                color,
            );
            if let Command::Create { props, .. } = &mut bar {
                props.input = Some(chora_scene::InputMode::Pass);
            }
            bar
        })
        .collect()
}

/// Puts the bars on the cross of a mark whose left edge is `x` and whose
/// centre stands at `y`.
pub(super) fn close_place(item: &Item, [x, y]: [f64; 2]) -> Vec<Command> {
    let centre = [x + CLOSE / 2.0, y];
    (0..2)
        .filter(|_| item.mark)
        .map(|n| {
            let turn = if n == 0 { 45.0 } else { -45.0 };
            let at = Transform::translate(centre[0], centre[1], 0.0)
                .then(&Transform::rotate_z(turn))
                .then(&Transform::translate(-CLOSE / 2.0, -1.0, 0.0));
            Command::Update {
                id: close_id(&item.id, n),
                props: PropsUpdate {
                    transform: Some(at),
                    ..PropsUpdate::default()
                },
            }
        })
        .collect()
}

/// The size of the focus mark, and its distance from what it marks.
const MARK_WIDTH: f64 = 2.0;
const MARK_GAP: f64 = 12.0;

impl Mc {
    /// Shows the focus mark of a hit while the pointer is over it, and
    /// hides it when the pointer leaves.
    pub fn mark(&mut self, node: &str, on: bool) -> Result<(), ClientError> {
        let Some(base) = node
            .strip_suffix("-hit")
            .filter(|_| node.starts_with("mc-"))
        else {
            return Ok(());
        };
        self.apply(vec![Command::Update {
            id: mark_id(base),
            props: PropsUpdate {
                opacity: Some(if on { 1.0 } else { 0.0 }),
                ..PropsUpdate::default()
            },
        }])
    }

    /// What a click on a hit does: a key row acts as its key, a helper
    /// opens the chat with it, a row of the overview shows its scene, a
    /// row of another page is announced as `mc.row.chosen`. Answers
    /// whether the node is a hit.
    pub fn click_hit(&mut self, node: &str) -> bool {
        let Some(base) = node.strip_suffix("-hit") else {
            return false;
        };
        if base.ends_with("-chapter-head") {
            return self.show_overview();
        }
        let token = self.hit_token(node);
        match token.as_deref().and_then(|t| t.split_once(':')) {
            Some(("key", binding)) => self.act(binding),
            Some(("agent", agent)) => self.chat_with(Some(agent)),
            Some(("chat", "open")) => self.chat_with(None),
            Some(("chat", "close")) => self.chat_close(),
            Some(("door", what)) => self.door(what),
            _ => return self.click_row(base),
        }
        true
    }

    /// Shows the page list, as a click on the head of a page does.
    fn show_overview(&mut self) -> bool {
        if let Err(err) = self.show(crate::manage::OVERVIEW, true) {
            eprintln!("chora-mc: {}: {}", crate::manage::OVERVIEW, err.message);
        }
        true
    }

    /// A click on a row of a page: `mc-s-SCENE-panels-PANEL-row-N`.
    fn click_row(&mut self, base: &str) -> bool {
        let Some((scene, rest)) = base
            .strip_prefix("mc-s-")
            .and_then(|b| b.split_once("-panels-"))
        else {
            return false;
        };
        let Some((panel, index)) = rest.rsplit_once("-row-") else {
            return false;
        };
        let Ok(index) = index.parse::<usize>() else {
            return false;
        };
        if scene == crate::manage::OVERVIEW {
            if let Some(target) = self.order().get(index).cloned()
                && let Err(err) = self.show(&target, true)
            {
                eprintln!("chora-mc: {target}: {}", err.message);
            }
            return true;
        }
        let emit = serde_json::json!({"event": "mc.row.chosen",
            "data": {"scene": scene, "panel": panel, "index": index}});
        if let Err(err) = self
            .rt
            .conn
            .call::<_, serde_json::Value>("events.emit", emit)
        {
            eprintln!("chora-mc: mc.row.chosen: {err}");
        }
        true
    }
}

#[cfg(test)]
mod tests_moved;
