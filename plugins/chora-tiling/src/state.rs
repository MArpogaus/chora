//! `state.get` (design 3.1): what tiling shows and holds, in the shape of
//! `kit::wm::state`, read from its model, its book and from what the last
//! transaction placed. Only reads. Tags count from 1, as the panel shows
//! them. Title and app id are core facts that tiling does not keep;
//! `views` and `texts` stay out.

use chora_kit::wm::state::{Desks, State, Window};
use serde_json::{Value, json};

use crate::{NAME, Tiling};

impl Tiling {
    /// What still moves.
    fn busy(&self) -> Vec<String> {
        let running = [
            (self.effects.animating(), "effects"),
            (self.chip_until.is_some(), "chip"),
        ];
        (running.into_iter().filter(|(on, _)| *on))
            .map(|(_, name)| name.to_owned())
            .collect()
    }

    /// The windows bottom first, the tags that show, the focus, the key
    /// list while it is open and what still moves.
    pub(crate) fn state_value(&self) -> Value {
        let z = |id: &str| self.order.iter().position(|o| o == id);
        let windows = (self.wm.windows.iter())
            .map(|w| {
                let rect = self.sent.get(&w.id).and_then(|p| {
                    let [x, y] = p.at?;
                    Some([x, y, p.frame[0], p.frame[1]].map(f64::from))
                });
                Window {
                    surface: w.id.clone(),
                    origin: self.book.origin(&w.id).map(chora_kit::wm::Origin::word),
                    output: w.output.clone(),
                    desks: tags_of(w.tags),
                    rect,
                    shown: rect.is_some(),
                    minimized: w.minimized,
                    maximized: w.maximized == (true, true),
                    fullscreen: w.fullscreen,
                    floating: w.floating,
                    held: self.dragged.as_ref().is_some_and(|(id, _)| *id == w.id),
                    z: z(&w.id),
                    ..Window::default()
                }
            })
            .collect();
        let desks = (self.wm.tags().into_iter())
            .map(|o| Desks {
                shown: o.tags.iter().filter(|t| t.shown).map(|t| t.tag).collect(),
                output: o.name,
            })
            .collect();
        let mut focus = self.focus.state(NAME, |s| self.wm.has(s));
        if self.keys_open() {
            focus.holder = Some(NAME.to_owned());
        }
        let open = self.open.state();
        State {
            windows,
            desks,
            focus,
            open,
            busy: self.busy(),
            more: [("helper".to_owned(), json!(self.voiced.word()))]
                .into_iter()
                .collect(),
            ..State::default()
        }
        .value()
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
            "open": [], "busy": [], "helper": "none"});
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
        tiling.open.open(
            chora_kit::wm::open::Surface::new(crate::keys::KEYS, chora_kit::wm::open::Kind::Sheet)
                .modal(),
        );
        let state = tiling.state().unwrap();
        let open = json!([{"id": "tiling-keys", "kind": "sheet", "modal": true}]);
        assert_eq!(state["open"], open);
        assert_eq!(state["focus"], json!({"holder": "tiling"}));
    }

    /// `helper` is `none` until voice reports, follows its latest
    /// `voice.state`, and is `none` again when voice goes.
    #[test]
    fn the_helper_follows_voice() {
        let (mut x, _) = on_fake_core("state-helper");
        assert_eq!(x.state().unwrap()["helper"], "none");
        let said = |event: &str, data| chora_scene::Event::Emitted {
            event: event.into(),
            client: "voice".into(),
            data,
        };
        x.event(said("voice.state", json!({"state": "idle"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "idle");
        x.event(said(
            "voice.state",
            json!({"name": "atlas", "state": "working"}),
        ))
        .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "working");
        x.event(said("voice.remove", json!({"name": "atlas"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "idle");
        let gone = json!({"event": "client.disconnected", "name": "voice"});
        x.event(serde_json::from_value(gone).unwrap()).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "none");
    }
}
