//! `state.get` (design 3.1): what tiling shows and holds, read from its
//! model and from what the last transaction placed, until tiling uses
//! `kit::wm`; the port deletes this adapter. Only reads. Tags count from
//! 1, as the panel shows them. Title and app id are core facts that
//! tiling does not keep; `views` and `texts` stay out.

use serde_json::{Value, json};

use crate::{NAME, Tiling, keys};

impl Tiling {
    /// The windows bottom first, the tags that show, the focus, the key
    /// list while it is open and what still moves.
    pub(crate) fn state_value(&self) -> Value {
        let z = |id: &str| self.order.iter().position(|o| o == id);
        let windows: Vec<Value> = (self.wm.windows.iter())
            .map(|w| {
                let placed = self.sent.get(&w.id);
                let at = placed.and_then(|p| p.at);
                let rect = at
                    .zip(placed)
                    .map(|([x, y], p)| [x, y, p.frame[0], p.frame[1]]);
                json!({
                    "surface": w.id,
                    "output": w.output,
                    "desks": tags_of(w.tags),
                    "rect": rect,
                    "shown": at.is_some(),
                    "minimized": w.minimized,
                    "maximized": w.maximized == (true, true),
                    "fullscreen": w.fullscreen,
                    "floating": w.floating,
                    "z": z(&w.id),
                })
            })
            .collect();
        let desks: Vec<Value> = (self.wm.tags().into_iter())
            .map(|o| {
                let shown: Vec<usize> = o.tags.iter().filter(|t| t.shown).map(|t| t.tag).collect();
                json!({"output": o.name, "shown": shown})
            })
            .collect();
        let held = (self.focus.surface.as_deref())
            .filter(|s| self.focus.sent && !self.focus.lost && self.wm.has(s));
        let focus = match held {
            Some(surface) => json!({"holder": NAME, "surface": surface}),
            None => json!({"holder": null}),
        };
        let open: Vec<Value> = (self
            .keys_open
            .then(|| json!({"id": keys::KEYS, "kind": "sheet", "modal": true})))
        .into_iter()
        .collect();
        let running = [
            (self.effects.animating(), "effects"),
            (self.chip_until.is_some(), "chip"),
        ];
        let busy: Vec<&str> = (running.into_iter().filter(|(on, _)| *on))
            .map(|(_, name)| name)
            .collect();
        json!({"windows": windows, "desks": desks, "focus": focus, "open": open,
            "busy": busy})
    }
}

/// The tag numbers of a set of tag bits, from 1.
fn tags_of(bits: u16) -> Vec<usize> {
    (0..16)
        .filter(|i| bits & (1 << i) != 0)
        .map(|i| i + 1)
        .collect()
}

#[cfg(test)]
mod tests {
    use chora_kit::runtime::Plugin as _;
    use serde_json::json;

    use crate::tests::on_fake_core;

    /// The tag bits read as tag numbers from 1.
    #[test]
    fn tag_bits_read_as_numbers() {
        assert_eq!(super::tags_of(0b101), [1, 3]);
        assert_eq!(super::tags_of(0), Vec::<usize>::new());
    }

    /// A fresh tiling shows nothing, holds no focus and is quiet.
    #[test]
    fn a_fresh_tiling_has_an_empty_state() {
        let (tiling, _) = on_fake_core("state-empty");
        let state = tiling.state().expect("tiling has a state");
        let expected = json!({"windows": [], "desks": [], "focus": {"holder": null},
            "open": [], "busy": []});
        assert_eq!(state, expected);
    }

    /// The fields of a window that a test reads.
    fn read(state: &serde_json::Value, i: usize) -> serde_json::Value {
        let w = &state["windows"][i];
        json!([w["surface"], w["output"], w["desks"], w["shown"], w["z"]])
    }

    /// A window placed on the first tag is in the state with its tag and
    /// place, and tiling holds the focus on it until another window
    /// manager takes it.
    #[test]
    fn a_placed_window_is_in_the_state_with_the_focus() {
        let (mut tiling, _) = on_fake_core("state-window");
        let area = [0, 0, 400, 300];
        tiling.wm.set_output("X", area, area);
        tiling.wm.add_window("s1");
        tiling.relayout().unwrap();
        let state = tiling.state().unwrap();
        assert_eq!(read(&state, 0), json!(["s1", "X", [1], true, 0]));
        assert_eq!(state["desks"], json!([{"output": "X", "shown": [1]}]));
        assert_eq!(state["focus"], json!({"holder": "tiling", "surface": "s1"}));
        tiling.focus.moved(Some("other"));
        assert_eq!(tiling.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// The key list is open while it shows, and holds the keyboard.
    #[test]
    fn the_key_list_is_open_while_it_shows() {
        let (mut tiling, _) = on_fake_core("state-keys");
        tiling.keys_open = true;
        let state = tiling.state().unwrap();
        let open = json!([{"id": "tiling-keys", "kind": "sheet", "modal": true}]);
        assert_eq!(state["open"], open);
        assert_eq!(state["focus"], json!({"holder": "tiling"}));
    }
}
