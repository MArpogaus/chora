//! The room that the other servers of the output take: the state pill of
//! the HUD host at the top right and the card of the notice server at the
//! foot. mc reads their rects (the nodes `hud-word` and `hud-figure`, and
//! `state.get` of the notice server) instead of keeping a copy of their
//! geometry, so that a pill or a card that stands elsewhere moves the end
//! of the header and of the page with it (rmm1-2).

use serde_json::{Value, json};

use crate::{ClientError, Mc};

/// The nodes of the pill of the HUD host: its word and its mark.
const PILL: [&str; 2] = ["hud-word", "hud-figure"];
/// The room that the page keeps at the foot until a card has shown: the
/// margin of the card of the mc folder of the notice server, 56 pixels,
/// and its two lines.
const CARD_ROOM_AT_START: f64 = 135.0;
/// The most of the height of the output that a card at the foot may take;
/// a card higher up stands in the middle or at the top, not at the foot.
const MOST_OF_HEIGHT: f64 = 3.0;

/// What mc read of the other servers last.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Rooms {
    /// The left edge of the state pill, if the HUD host draws one.
    pub pill_left: Option<f64>,
    /// The height from the top of the card that shows now to the foot, when
    /// it is more than the room at the start (rmm2-2): a card of another
    /// height that fits the room never moves the pages.
    pub card: Option<f64>,
}

impl Mc {
    /// The height at the foot that the page leaves free for the card.
    pub(crate) fn card_room(&self) -> f64 {
        self.rooms.card.unwrap_or(CARD_ROOM_AT_START)
    }

    /// Where the header ends at the right: a baseline before the pill, or
    /// at the output width `width` when the host draws none.
    pub(crate) fn header_end(&self, width: f64) -> f64 {
        let baseline = self.settings.value.scale.baseline;
        self.rooms.pill_left.map_or(width, |left| left - baseline)
    }

    /// The top of the pages in output coordinates: where the text block of
    /// a scene begins.
    fn page_top(&self) -> f64 {
        let first = self.scenes.keys().next();
        first
            .and_then(|name| self.geometry(name))
            .map_or(0.0, |geo| geo.text[1])
    }

    /// What the pill and the card take now. A card counts only when it
    /// stands on the output of mc and its top lies under the top of the
    /// page, in output coordinates (rmm2-1); the room is what the card
    /// takes beyond the room at the start, and none once it closed.
    pub(crate) fn read_rooms(&mut self) -> Result<Rooms, ClientError> {
        let Some(output) = self.output.clone() else {
            return Ok(self.rooms);
        };
        let [width, height] = output.size.map(f64::from);
        let [left, top] = output.position.map(f64::from);
        let page_top = self.page_top();
        let mut pill_left: Option<f64> = None;
        for id in PILL {
            if let Some([x, ..]) = self.rect(id)? {
                pill_left = Some(pill_left.map_or(x, |left| left.min(x)));
            }
        }
        let mut rooms = Rooms {
            pill_left: pill_left.map(f64::round),
            card: None,
        };
        if let Some([x, y, ..]) = self.card_rect() {
            let (across, down) = (x - left, y - top);
            let room = (height - down).round();
            let at_foot = room <= (height / MOST_OF_HEIGHT).round();
            if (0.0..width).contains(&across) && down > page_top && at_foot {
                rooms.card = (room > CARD_ROOM_AT_START).then_some(room);
            }
        }
        Ok(rooms)
    }

    /// Reads the pill and the card again. A pill that stands elsewhere
    /// (its word is as wide as the state it says) moves the end of the
    /// header; a card that stands elsewhere moves the end of the pages
    /// too, so only that builds them anew.
    pub(crate) fn learn_rooms(&mut self) -> Result<(), ClientError> {
        let rooms = self.read_rooms()?;
        if rooms == self.rooms {
            return Ok(());
        }
        let card_moved = rooms.card != self.rooms.card;
        self.rooms = rooms;
        self.render()?;
        if card_moved {
            self.rebuild_pages()?;
        }
        Ok(())
    }

    /// The box of the card on screen, if the notice server runs and shows one.
    fn card_rect(&mut self) -> Option<[f64; 4]> {
        let call = json!({"client": "notify", "method": "state.get", "params": {}});
        let state: Value = self.rt.conn.call("methods.call", call).ok()?;
        let rect = state["shown"].get(0)?["rect"].as_array()?;
        let rect: Vec<f64> = rect.iter().filter_map(Value::as_f64).collect();
        <[f64; 4]>::try_from(rect).ok()
    }
}
