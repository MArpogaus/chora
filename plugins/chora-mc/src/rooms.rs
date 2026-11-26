//! The room that the other servers of the output take: the state pill of
//! the HUD host at the top right and the card of the notice server at the
//! foot. mc reads their rects (the nodes `hud-word` and `hud-figure`, and
//! `state.get` of the notice server) instead of keeping a copy of their
//! geometry, so that a pill or a card that stands elsewhere moves the end
//! of the header and of the page with it (rmm1-2).

use serde_json::{Value, json};

use crate::{ClientError, Mc, region::Region};

/// The nodes of the pill of the HUD host: its word and its mark, and the
/// badge of a muted mic, which is wider than the word.
const PILL: [&str; 3] = ["hud-word", "hud-figure", "hud-badge"];
/// The most of the height of the output that a card at the foot may take;
/// a card higher up stands in the middle or at the top, not at the foot.
const MOST_OF_HEIGHT: f64 = 3.0;

/// What mc read of the other servers last.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Rooms {
    /// The left edge of the state pill, if the HUD host draws one.
    pub pill_left: Option<f64>,
    /// The height from the top of the card that shows now at the foot to
    /// the foot, when it is more than the room of the footer: the page
    /// leaves that band free only while the card shows, and comes back
    /// whole when it closed (rv9ka-1); a card that fits the footer room
    /// never moves the pages (rmm2-2).
    pub card: Option<f64>,
}

/// The left edge of the leftmost node of the HUD host in `PILL` that
/// `rect` finds.
fn leftmost(
    mut rect: impl FnMut(&str) -> Result<Option<[f64; 4]>, ClientError>,
) -> Result<Option<f64>, ClientError> {
    let mut left: Option<f64> = None;
    for id in PILL {
        if let Some([x, ..]) = rect(id)? {
            left = Some(left.map_or(x, |left| left.min(x)));
        }
    }
    Ok(left)
}

impl Mc {
    /// The height at the foot that the page leaves free: the card's while
    /// one shows at the foot, else the footer's.
    pub(crate) fn card_room(&self) -> f64 {
        self.rooms.card.unwrap_or_else(|| self.foot_room())
    }

    /// The room of the footer line at the foot: what the page leaves free
    /// while no card shows there (rv9ka-1).
    pub(crate) fn foot_room(&self) -> f64 {
        let config = &self.settings.value;
        config.regions.get(Region::Footer).size * config.scale.baseline
    }

    /// Where the header ends at the right: a baseline before the pill, or
    /// at the output width `width` when the host draws none. While the
    /// chat is open the host moves the pill into it for a moment
    /// (Super+a), and the header ignores it.
    pub(crate) fn header_end(&self, width: f64) -> f64 {
        let baseline = self.settings.value.scale.baseline;
        let pill = (self.rooms.pill_left).filter(|_| self.chat != crate::Chat::Open);
        pill.map_or(width, |left| left - baseline)
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
    /// takes beyond the room of the footer, and none once it closed. The
    /// rect is output-local already (rmm3-1); a card is at the foot by its
    /// bottom in the lower third, and its room is clamped to a third (rmm3-2).
    pub(crate) fn read_rooms(&mut self) -> Result<Rooms, ClientError> {
        let Some(output) = self.output.clone() else {
            return Ok(self.rooms);
        };
        let [width, height] = output.size.map(f64::from);
        let page_top = self.page_top();
        let pill_left = leftmost(|id| self.rect(id))?;
        let mut rooms = Rooms {
            pill_left: pill_left.map(f64::round),
            card: None,
        };
        if let Some([x, y, _, h]) = self.card_rect() {
            let third = (height / MOST_OF_HEIGHT).round();
            let room = (height - y).round().min(third);
            let at_foot = y + h >= height - third;
            if (0.0..width).contains(&x) && y > page_top && at_foot {
                rooms.card = (room > self.foot_room()).then_some(room);
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

    /// Reads the rooms right after a notice came or went, so that a card
    /// never covers the last lines of the page until the next round.
    pub(crate) fn learn_card(&mut self) {
        if let Err(err) = self.learn_rooms() {
            tracing::warn!("rooms after a notice: {err}");
        }
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

    /// The badge of a muted mic is wider than the word of the state; it
    /// takes room like the pill, so that the header ends before it
    /// (rpm2-3).
    #[test]
    fn the_header_ends_before_the_badge_of_a_muted_mic() {
        let nodes = [
            ("hud-badge", [947.0, 27.0, 252.0, 32.0]),
            ("hud-figure", [1204.0, 34.0, 16.0, 16.0]),
        ];
        let left = super::leftmost(|id| {
            Ok(nodes
                .iter()
                .find(|(name, _)| *name == id)
                .map(|(_, rect)| *rect))
        });
        assert_eq!(left.unwrap(), Some(947.0));
    }

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

    /// How many lines of a panel show on the page as it stands, scrolled
    /// as it is: the whole lines of its box above the end of the page.
    fn lines_shown(mc: &crate::Mc, scene: &str, panel: &str) -> usize {
        let panel = (mc.scenes[scene].panels.iter())
            .find(|p| p.id == panel)
            .unwrap()
            .clone();
        let [_, y, _, h] = mc.place(scene, &panel).unwrap();
        let geo = mc.geometry(scene).unwrap();
        let scroll = mc.scroll.get(scene).copied().unwrap_or_default();
        let end = geo.text[1] + geo.text[3] + scroll;
        let line = mc.ty("body", panel.style.as_ref()).line;
        let top = mc.lines_top(&panel);
        let seen = mc.whole_lines(&panel, h.min(end - y).max(0.0));
        (1_u32..=200)
            .take_while(|&i| top + f64::from(i) * line <= seen + 0.5)
            .count()
    }

    /// Whether the mark that more follows shows on the page.
    fn more_shows(mc: &crate::Mc, scene: &str) -> bool {
        let rects = crate::HashMap::new();
        match mc.more_update(scene, &rects) {
            Some(crate::Command::Update { props, .. }) => props.visible == Some(true),
            other => panic!("no update of the mark: {other:?}"),
        }
    }

    /// The panels of `want` that lose lines with no word of it: fewer
    /// lines show than the panel holds, and the page shows no mark that
    /// more follows.
    fn cut_unseen(mc: &crate::Mc, want: &[(&str, &str, usize)]) -> Vec<String> {
        (want.iter())
            .map(|&(scene, panel, n)| (scene, panel, n, lines_shown(mc, scene, panel)))
            .filter(|&(scene, _, n, shown)| shown < n && !more_shows(mc, scene))
            .map(|(scene, panel, n, shown)| format!("{scene}/{panel} {shown} of {n}"))
            .collect()
    }

    /// With no card on screen the last row of the recipe pages shows every
    /// line at 1280x720 (rv9ka-1): the page reaches the footer, not the
    /// top of a card that is not there, and a page that is longer still
    /// says that it goes on.
    #[test]
    fn with_no_card_the_last_row_of_a_recipe_shows_every_line() {
        let core = chora_kit::testing::FakeCore::start().selecting();
        core.answer_with("state.get", json!({"shown": []}));
        let mut mc = crate::testing::mc_on(&core, "page-end-none", [1280, 720]);
        let want = recipe(&mut mc);
        mc.learn_rooms().unwrap();
        let cut = cut_unseen(&mc, &want);
        assert!(cut.is_empty(), "lines cut with no card: {}", cut.join(", "));
        let (h, b) = (720.0, mc.settings.value.scale.baseline);
        for (scene, panel, n) in want {
            let [_, top, _, height] = mc.geometry(scene).unwrap().text;
            assert!(top + height + b >= h - mc.foot_room(), "{scene} ends short");
            if scene != "sauce" && scene != "morning" {
                assert!(lines_shown(&mc, scene, panel) >= n, "{scene}/{panel}");
            }
        }
    }

    /// While a card shows at the foot the page keeps its band free and says
    /// that it goes on; Down of a remote or a keyboard scrolls it to its
    /// last line, Up back; and once the card closed the page is whole
    /// again (rv9ka-1).
    #[test]
    fn with_a_card_the_page_goes_on_and_the_keys_scroll_it() {
        let core = chora_kit::testing::FakeCore::start().selecting();
        core.answer_with("state.get", json!({"shown": []}));
        let mut mc = crate::testing::mc_on(&core, "page-end-card", [1280, 720]);
        let want = recipe(&mut mc);
        mc.show("boil", false).unwrap();
        core.answer_with(
            "state.get",
            json!({"shown": [{"rect": [160, 585, 291, 79]}]}),
        );
        mc.learn_rooms().unwrap();
        assert_eq!(mc.card_room(), 135.0, "the card's band");
        assert!(lines_shown(&mc, "boil", "side") < 3, "the card takes lines");
        assert!(more_shows(&mc, "boil"), "the page says that it goes on");
        assert_eq!(cut_unseen(&mc, &want[1..2]), Vec::<String>::new());
        for _ in 0..4 {
            crate::tests::press_key_from(&mut mc, "Down", &[], false);
        }
        assert!(
            lines_shown(&mc, "boil", "side") >= 3,
            "Down reaches the end"
        );
        assert!(!more_shows(&mc, "boil"), "at its end the mark hides");
        crate::tests::press_key_from(&mut mc, "Up", &[], false);
        assert!(more_shows(&mc, "boil"), "Up goes back");
        core.answer_with("state.get", json!({"shown": []}));
        mc.learn_rooms().unwrap();
        assert_eq!(mc.card_room(), mc.foot_room(), "the band comes back");
        assert_eq!(mc.scroll.get("boil").copied().unwrap_or_default(), 0.0);
        assert!(lines_shown(&mc, "boil", "side") >= 3, "the page is whole");
    }

    /// The panels of the pages whose lines the edge of the page cuts
    /// through a line: the page edge is nearer than the end of the panel's
    /// clip, and what shows of the lines node (from its top to the clip or
    /// the edge) is neither the whole text nor a whole number of lines
    /// (rv9ka-1, rtr3-1).
    fn lines_cut_through(mc: &crate::Mc) -> Vec<String> {
        use chora_scene::Command;
        let mut cut = Vec::new();
        for (scene, page) in &mc.scenes {
            let geo = mc.geometry(scene).unwrap();
            let scroll = mc.scroll.get(scene).copied().unwrap_or_default();
            let edge = geo.text[1] + geo.text[3];
            for panel in page.panels.iter().filter(|p| !p.lines.is_empty()) {
                let id = crate::panel_id(scene, &panel.id);
                let commands = mc.panel(scene, panel);
                let made = |want: &str| {
                    commands.iter().find_map(|c| match c {
                        Command::Create { id, props, .. } if id == want => Some(props.clone()),
                        _ => None,
                    })
                };
                let clip = made(&id).and_then(|p| p.size).map_or(0.0, |s| s[1]);
                let lines = made(&format!("{id}-lines")).unwrap();
                let top = lines.transform.map_or(0.0, |t| t.translation_2d().1);
                let [_, y, w, h] = mc.place(scene, panel).unwrap();
                let line = mc.ty("body", panel.style.as_ref()).line;
                let total = mc.text_height(panel, w) - mc.lines_top(panel);
                let room = edge - (y - scroll);
                let seen = (clip.min(h).min(room) - top).max(0.0);
                let whole = ((seen / line) - (seen / line).round()).abs() < 0.02;
                if room < clip.min(h) && seen + 0.5 < total && !whole {
                    cut.push(format!("{scene}/{} {seen} of {total} px", panel.id));
                }
            }
        }
        cut
    }

    /// With no card, and with a timer card at the foot, no panel of a page
    /// shows a part of a line at the edge of the page: the panel takes
    /// whole lines and the page says that it goes on (rtr3-1).
    #[test]
    fn a_panel_at_the_page_edge_shows_whole_lines_with_and_without_a_card() {
        let core = chora_kit::testing::FakeCore::start().selecting();
        core.answer_with("state.get", json!({"shown": []}));
        let mut mc = crate::testing::mc_on(&core, "page-end-whole", [1280, 720]);
        recipe(&mut mc);
        mc.learn_rooms().unwrap();
        let none = lines_cut_through(&mc);
        core.answer_with(
            "state.get",
            json!({"shown": [{"rect": [160, 585, 291, 79]}]}),
        );
        mc.learn_rooms().unwrap();
        let card = lines_cut_through(&mc);
        assert!(
            none.is_empty() && card.is_empty(),
            "lines cut through, no card: {}; a card: {}",
            none.join(", "),
            card.join(", ")
        );
    }

    /// A foot card on an output that stands right of another one takes
    /// its room: the rect of the card is output-local (rmm3-1).
    #[test]
    fn a_foot_card_on_an_output_off_the_origin_takes_room() {
        let core = chora_kit::testing::FakeCore::start().selecting();
        core.answer_with("state.get", json!({"shown": []}));
        let mut mc = crate::testing::mc_on(&core, "page-end-offset", [1280, 720]);
        mc.output.as_mut().unwrap().position = [1280, 0];
        core.answer_with(
            "state.get",
            json!({"shown": [{"rect": [160, 585, 291, 79]}]}),
        );
        mc.learn_rooms().unwrap();
        assert_eq!(mc.card_room(), 135.0, "the card's band");
    }

    /// A long foot card (8 body lines) on 1280x720 is a foot card all the
    /// same; its room is clamped to a third of the height (rmm3-2).
    #[test]
    fn a_long_foot_card_takes_the_clamped_third() {
        let core = chora_kit::testing::FakeCore::start().selecting();
        core.answer_with("state.get", json!({"shown": []}));
        let mut mc = crate::testing::mc_on(&core, "page-end-long", [1280, 720]);
        core.answer_with(
            "state.get",
            json!({"shown": [{"rect": [160, 400, 291, 264]}]}),
        );
        mc.learn_rooms().unwrap();
        assert_eq!(mc.card_room(), 240.0, "a third of 720");
    }
}
