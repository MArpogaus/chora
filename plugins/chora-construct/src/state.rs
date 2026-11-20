//! `state.get` (design 3.1): what the construct shows and holds, read
//! from its model until it uses `kit::wm`; the port deletes this adapter.
//! Only reads, besides `core_focus`, which follows `focus.changed` for
//! this answer alone. A pane of a window that the construct owns is a
//! window; a pane of a window of another client, of a plugin node or of a
//! virtual output is a view, never an owner. Panes lie in a space, so a
//! window has no rectangle on the output; desks are the cube's faces,
//! which the window manager on them reports.

use chora_kit::wm::open::{Kind, Open, Surface};
use serde_json::{Value, json};

use super::{Construct, Content, NAME, hud, space};

impl Construct {
    /// The windows that the construct owns, the views of the others, the
    /// focus, what is open and what still moves.
    pub(crate) fn state_value(&self) -> Value {
        let output = self.output.as_ref().map(|o| o.name.as_str());
        let far = self.radius();
        let inside = self.flown.as_ref().map(|f| f.surface.as_str());
        let in_sight = |p: &super::Pane| {
            if let Some(surface) = inside {
                return p.source == surface;
            }
            let covered = (self.cube.as_ref()).is_some_and(|c| c.covers(&self.pose, p.at));
            !covered
                && !self.backs.contains(&p.name)
                && space::sight(&self.lens, &self.pose, p.spot(), far) == "shown"
        };
        let origin = |surface: &str| self.book.origin(surface).map(chora_kit::wm::Origin::word);
        let mut windows = Vec::new();
        let mut views = Vec::new();
        for pane in &self.panes {
            let owner = match &pane.content {
                Content::Window { .. } if pane.attached => {
                    let mut window = json!({"surface": pane.source, "output": output,
                        "shown": self.active && in_sight(pane)});
                    if let Some(word) = origin(&pane.source) {
                        window["origin"] = json!(word);
                    }
                    windows.push(window);
                    continue;
                }
                Content::Window { .. } => None,
                Content::Plugin { plugin, .. } => Some(plugin.as_str()),
                Content::Manager { manager } => Some(manager.as_str()),
            };
            views.push(json!({"surface": pane.source, "owner": owner, "pane": pane.name}));
        }
        for surface in &self.hidden {
            let mut window = json!({"surface": surface, "output": output, "shown": false});
            if let Some(word) = origin(surface) {
                window["origin"] = json!(word);
            }
            windows.push(window);
        }
        json!({"windows": windows, "views": views, "focus": self.focus_value(),
            "open": self.open_value(), "busy": self.busy_value(),
            "helper": self.voiced.word()})
    }

    /// The construct holds the focus on a window it owns, or with its own
    /// space while that shows as the flavor and no surface has the focus.
    fn focus_value(&self) -> Value {
        let owned = |s: &str| {
            self.hidden.iter().any(|h| h == s)
                || (self.panes.iter()).any(|p| p.attached && p.window() == Some(s))
        };
        match self.core_focus.as_deref() {
            Some(surface) if owned(surface) => json!({"holder": NAME, "surface": surface}),
            None if self.active && !self.overlay => json!({"holder": NAME}),
            _ => json!({"holder": null}),
        }
    }

    /// The window list, the help and the window of the launcher, in the
    /// order Escape shuts them: the one that opened last first. The HUD host
    /// shows the questions of the voice agent.
    fn open_value(&self) -> Value {
        let shown = [
            (
                self.launcher_window.is_some(),
                "construct-launcher",
                Kind::Launcher,
                false,
            ),
            (self.help, hud::HELP, Kind::Sheet, true),
            (self.list.is_some(), hud::LIST, Kind::Popup, true),
        ];
        let mut open = Open::default();
        for (_, id, kind, modal) in shown.into_iter().filter(|(on, ..)| *on) {
            let surface = Surface::new(id, kind);
            open.open(if modal { surface.modal() } else { surface });
        }
        serde_json::to_value(open.state()).unwrap_or_default()
    }

    /// What still moves or waits: quiet once this is empty.
    fn busy_value(&self) -> Vec<&'static str> {
        let running = [
            (self.glide.is_some(), "glide"),
            (self.stride.is_some(), "walk"),
            (self.moving.is_some(), "move"),
            (self.effects.animating(), "effects"),
            (self.cube_turns(), "cube"),
            (!self.waiting.is_empty(), "waiting"),
        ];
        (running.into_iter().filter(|(on, _)| *on))
            .map(|(_, name)| name)
            .collect()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chora_kit::wm::open::{Kind, Surface};
    use chora_kit::{runtime::Plugin as _, settings, testing::FakeCore};
    use serde_json::json;

    use super::super::{Construct, Content, NAME, Pane, config};

    /// A construct on a fake core, as the flavor or beside another.
    pub(crate) fn construct(tag: &str, overlay: bool) -> (Construct, FakeCore) {
        let core = FakeCore::start();
        let rt = core.runtime(NAME);
        let dir = chora_kit::testing::TempDir::unmade(&format!("chora-construct-{tag}")).keep();
        let file = dir.join("chora-construct.yaml");
        let settings = settings::File::load(file, config::DEFAULT).unwrap();
        let shaders = |name: &str, _: &str| Ok(name.to_owned());
        let construct = Construct::new(rt, settings, overlay, &shaders).unwrap();
        (construct, core)
    }

    /// A pane straight ahead of the camera.
    fn pane(name: &str, source: &str, content: Content, attached: bool) -> Pane {
        Pane {
            name: name.into(),
            source: source.into(),
            content,
            at: [0.0, 1500.0],
            yaw: 0.0,
            size: [640.0, 400.0],
            taken: false,
            attached,
            placed: false,
        }
    }

    /// A fresh construct shows nothing, holds the focus with its space
    /// only as the flavor, and is quiet.
    #[test]
    fn a_fresh_construct_has_an_empty_state() {
        let (mut flavor, _core) = construct("state-empty", false);
        let state = flavor.state().expect("the construct has a state");
        let expected = json!({"windows": [], "views": [], "focus": {"holder": null},
            "open": [], "busy": [], "helper": "no_helper"});
        assert_eq!(state, expected);
        flavor.active = true;
        assert_eq!(
            flavor.state().unwrap()["focus"],
            json!({"holder": "construct"})
        );
        let (mut beside, _core) = construct("state-overlay", true);
        beside.active = true;
        assert_eq!(beside.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// An owned window is a window; the panes of a window of another
    /// client, of a plugin and of a manager are views.
    #[test]
    fn a_pane_of_another_owner_is_a_view() {
        let (mut c, _core) = construct("state-views", false);
        c.active = true;
        let window = || Content::Window { command: None };
        c.panes.push(pane("foot", "s1", window(), true));
        c.panes.push(pane("taken", "s2", window(), false));
        let agenda = Content::Plugin {
            plugin: "mc".into(),
            scene: "agenda".into(),
        };
        c.panes.push(pane("agenda", "mc-agenda", agenda, false));
        c.panes.push(pane(
            "xp",
            "VIRTUAL-1",
            Content::Manager {
                manager: "xp".into(),
            },
            false,
        ));
        c.hidden.push("s3".into());
        let state = c.state().unwrap();
        let surfaces: Vec<_> = state["windows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| (w["surface"].clone(), w["shown"].clone()))
            .collect();
        assert_eq!(
            surfaces,
            [(json!("s1"), json!(true)), (json!("s3"), json!(false))]
        );
        let views = json!([
            {"surface": "s2", "owner": null, "pane": "taken"},
            {"surface": "mc-agenda", "owner": "mc", "pane": "agenda"},
            {"surface": "VIRTUAL-1", "owner": "xp", "pane": "xp"},
        ]);
        assert_eq!(state["views"], views);
    }

    /// The construct holds the focus on a window that it owns, not on a
    /// view of another client's window.
    #[test]
    fn the_focus_is_held_only_on_an_owned_window() {
        let (mut c, _core) = construct("state-focus", false);
        let window = || Content::Window { command: None };
        c.panes.push(pane("foot", "s1", window(), true));
        c.panes.push(pane("taken", "s2", window(), false));
        c.core_focus = Some("s1".into());
        let held = json!({"holder": "construct", "surface": "s1"});
        assert_eq!(c.state().unwrap()["focus"], held);
        c.core_focus = Some("s2".into());
        assert_eq!(c.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// A look that takes the focused pane out of sight gives the keyboard
    /// back to the walk: the focus is never a window that is no shown
    /// window (co-pane r1, I3 of the persona round 3).
    #[test]
    fn a_look_away_from_the_focused_pane_takes_the_focus_off_it() {
        let (mut c, _core) = construct("state-look-away", false);
        c.active = true;
        c.panes
            .push(pane("foot", "s1", Content::Window { command: None }, true));
        c.core_focus = Some("s1".into());
        let held = json!({"holder": "construct", "surface": "s1"});
        assert_eq!(c.state().unwrap()["focus"], held);
        c.drag = Some(crate::Drag {
            press: [640.0, 400.0],
            last: [640.0, 400.0],
            moved: false,
        });
        c.look([640.0, 800.0]).unwrap();
        let state = c.state().unwrap();
        assert_eq!(state["windows"][0]["shown"], json!(false), "looked away");
        assert_eq!(state["focus"], json!({"holder": "construct"}));
    }

    /// A look that keeps the pane in sight leaves the focus where it is.
    #[test]
    fn a_small_look_keeps_the_focus_on_the_pane_in_sight() {
        let (mut c, _core) = construct("state-look-near", false);
        c.active = true;
        c.panes
            .push(pane("foot", "s1", Content::Window { command: None }, true));
        c.core_focus = Some("s1".into());
        c.drag = Some(crate::Drag {
            press: [640.0, 400.0],
            last: [640.0, 400.0],
            moved: false,
        });
        c.look([640.0, 410.0]).unwrap();
        let held = json!({"holder": "construct", "surface": "s1"});
        assert_eq!(c.state().unwrap()["focus"], held);
        assert_eq!(c.state().unwrap()["windows"][0]["shown"], json!(true));
    }

    /// The help is open while it shows.
    #[test]
    fn the_help_is_open_while_it_shows() {
        let (mut c, _core) = construct("state-help", false);
        c.help = true;
        let help = Surface::new("construct-help", Kind::Sheet).modal();
        let open = json!([help.entry()]);
        assert_eq!(c.state().unwrap()["open"], open);
    }

    /// `helper` is the word of the kit machine: `no_helper` until a helper
    /// exists, its state while it does, `no_helper` again when voice goes.
    #[test]
    fn the_helper_follows_voice() {
        let (mut x, _core) = construct("state-helper", false);
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
        let said = |event: &str, data| chora_scene::Event::Emitted {
            event: event.into(),
            client: "voice".into(),
            data,
        };
        x.event(said("voice.state", json!({"state": "idle"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
        x.event(said(
            "voice.state",
            json!({"name": "atlas", "state": "working"}),
        ))
        .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "working");
        x.event(said("voice.remove", json!({"name": "atlas"})))
            .unwrap();
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
        let gone = json!({"event": "client.disconnected", "name": "voice"});
        x.event(serde_json::from_value(gone).unwrap()).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "no_helper");
    }

    /// A look that hears voice connect asks voice for its state, and it
    /// hears every event of the helper machine, so that it says what the
    /// presenter says (rw-4).
    #[test]
    fn the_helper_is_seeded_from_voice_and_hears_every_event() {
        assert_eq!(
            chora_kit::helper::unheard(&crate::EVENTS),
            Vec::<&str>::new()
        );
        let (mut x, core) = construct("state-seed", false);
        core.answer_call(
            "voice",
            "state",
            json!({"state": "listening", "agents": [{"name": "coder", "state": "idle"}]}),
        );
        let connected = chora_scene::Event::ClientConnected {
            name: "voice".into(),
        };
        x.event(connected).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "listening");
        let said = chora_scene::Event::Emitted {
            event: "voice.say".into(),
            client: "voice".into(),
            data: json!({"state": "speaking", "text": "Hi"}),
        };
        x.event(said).unwrap();
        assert_eq!(x.state().unwrap()["helper"], "answered");
    }
}
