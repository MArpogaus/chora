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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use chora_kit::runtime::Plugin as _;

    /// The recipe pages of the tour (demo/tour/chapters/mc.py), as
    /// `scene.save` takes them.
    const PAGES: &str = r#"[
      {"name": "ingredients", "chapter": {"title": "Spaghetti"}, "panels": [
        {"id": "lead", "cell": [0, 0, 2, 2], "style": {"base": "lead"}, "empty": "No recipe yet."},
        {"id": "list", "cell": [2, 0, 2, 2], "rule": "faint", "empty": "No list yet."},
        {"id": "tip", "cell": [0, 2, 4, 1], "empty": "No tip yet."}]},
      {"name": "boil", "chapter": {"title": "Step 1: the water"}, "panels": [
        {"id": "lead", "cell": [0, 0, 3, 2], "style": {"base": "lead"}, "empty": "Nothing yet."},
        {"id": "side", "cell": [3, 0, 1, 2], "rule": "faint", "empty": "Nothing yet."}]},
      {"name": "sauce", "chapter": {"title": "Step 2: the sauce"}, "panels": [
        {"id": "lead", "cell": [0, 0, 3, 2], "style": {"base": "lead"}, "empty": "Nothing yet."},
        {"id": "side", "cell": [3, 0, 1, 2], "rule": "faint", "empty": "Nothing yet."}]}
    ]"#;

    /// The lines of the tour, and of the morning page, as `panel.set`
    /// takes them.
    const LINES: &str = r#"[
      {"scene": "ingredients", "panel": "lead", "lines": ["Spaghetti with garlic and oil",
        "Serves two. Twenty minutes from the first pot to the plate."]},
      {"scene": "ingredients", "panel": "list", "lines": ["For two", "200 g spaghetti",
        "4 cloves of garlic", "6 tbsp olive oil"]},
      {"scene": "ingredients", "panel": "tip", "lines": ["Tip",
        "Keep a cup of the pasta water for the sauce."]},
      {"scene": "boil", "panel": "lead", "lines": ["Boil the water",
        "Fill the large pot with two litres of water, add a spoon of salt and bring it to a rolling boil. Then add the spaghetti and cook it for nine minutes."]},
      {"scene": "boil", "panel": "side", "lines": ["Timer", "9 minutes once the spaghetti is in."]},
      {"scene": "sauce", "panel": "lead", "lines": ["Warm the oil",
        "Take the oil on low heat. Add the sliced garlic and the chili for two minutes, until golden. Do not let the garlic turn brown."]},
      {"scene": "sauce", "panel": "side", "lines": ["Then",
        "Toss the drained spaghetti in the pan with a splash of the pasta water and the parsley."]},
      {"scene": "morning", "panel": "lead", "lines": ["A quiet day", "Two meetings and a walk."]},
      {"scene": "morning", "panel": "reading", "lines": ["Reading", "The long read", "A short one"]}
    ]"#;

    /// Saves the recipe pages and fills them and the morning page; gives
    /// the panel of the last row of each and the lines it holds at
    /// 1280x720 with the right column: the list 4, the Timer 3 (its second
    /// line wraps), "Then" 5, the reading 3.
    fn recipe(mc: &mut crate::Mc) -> [(&'static str, &'static str, usize); 4] {
        let pages: Vec<serde_json::Value> = serde_json::from_str(PAGES).unwrap();
        for page in pages {
            mc.method("scene.save", page).unwrap();
        }
        let lines: Vec<serde_json::Value> = serde_json::from_str(LINES).unwrap();
        for set in lines {
            mc.method("panel.set", set).unwrap();
        }
        [
            ("ingredients", "list", 4),
            ("boil", "side", 3),
            ("sauce", "side", 5),
            ("morning", "reading", 3),
        ]
    }

    /// How many lines of a panel show on the page as it stands, unscrolled:
    /// the whole lines of its box above the end of the page.
    fn lines_shown(mc: &crate::Mc, scene: &str, panel: &str) -> usize {
        let panel = (mc.scenes[scene].panels.iter())
            .find(|p| p.id == panel)
            .unwrap()
            .clone();
        let [_, y, _, h] = mc.place(scene, &panel).unwrap();
        let geo = mc.geometry(scene).unwrap();
        let end = geo.text[1] + geo.text[3];
        let line = mc.ty("body", panel.style.as_ref()).line;
        let top = mc.lines_top(&panel);
        let seen = mc.whole_lines(&panel, h.min(end - y).max(0.0));
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a few lines"
        )]
        let shown = ((seen - top) / line).round().max(0.0) as usize;
        shown
    }

    /// With no card on screen the last row of the recipe pages shows every
    /// line at 1280x720 (rv9ka-1): the page reaches the footer, not the
    /// top of a card that is not there.
    #[test]
    fn with_no_card_the_last_row_of_a_recipe_shows_every_line() {
        let core = chora_kit::testing::FakeCore::start().selecting();
        core.answer_with("state.get", json!({"shown": []}));
        let mut mc = crate::testing::mc_on(&core, "page-end-none", [1280, 720]);
        let want = recipe(&mut mc);
        mc.learn_rooms().unwrap();
        let cut: Vec<String> = (want.iter())
            .map(|&(scene, panel, n)| (scene, panel, n, lines_shown(&mc, scene, panel)))
            .filter(|&(_, _, n, shown)| shown < n)
            .map(|(scene, panel, n, shown)| format!("{scene}/{panel} {shown} of {n}"))
            .collect();
        assert!(cut.is_empty(), "lines cut with no card: {}", cut.join(", "));
    }
}
