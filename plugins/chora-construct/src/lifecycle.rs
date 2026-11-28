//! The life of a window of the construct (WM1c): the windows that it
//! owns come and go through `kit::wm::book`. The book decides which
//! surfaces are free for the construct (the claim filter) and keeps where
//! each came from; the construct attaches them as panes and answers each
//! proposal with `confirm` or `refused`. What a pane does with its window
//! (placing, the camera, the keyboard) stays in `adopt.rs` and `list.rs`.

use chora_kit::wm::{Change, Created, Phase, Why};
use chora_scene::Event;
use serde_json::Value;

use super::{ClientError, Construct};

impl Construct {
    /// A surface was created: the book says whether it is free for the
    /// construct; the construct takes it as a pane, or only shows it as
    /// a pane when it started the program (overlay), and answers the
    /// proposal by whether the window is its own now.
    pub(crate) fn window_created(&mut self, created: &Created) -> Result<(), ClientError> {
        let changes = self.book.claim(
            &created.surface,
            created.to.as_deref(),
            created.token.clone(),
        );
        let free = !changes.is_empty();
        let done = self.created(created, free);
        self.settle_plain(changes)?;
        done?;
        self.keys_for_new(created)
    }

    /// A new window came as a pane and the camera is in no window, while
    /// a window of a pane has the keyboard from a press. A child of that
    /// window (a dialog with an xdg parent at its first commit) gets the
    /// keyboard, as a new window takes it in the other looks; any other
    /// new pane gives it back to the walk (D116). A focus of another
    /// window manager, of the launcher or of a window that the camera
    /// flies into stays.
    fn keys_for_new(&mut self, created: &Created) -> Result<(), ClientError> {
        let surface = created.surface.as_str();
        let pane = (self.panes.iter()).any(|p| p.window() == Some(surface));
        let pressed = (self.core_focus.as_deref()).filter(|f| *f != surface && self.own_window(f));
        let Some(pressed) = pressed.filter(|_| pane && self.inside().is_none()) else {
            return Ok(());
        };
        if created.parent.as_deref() == Some(pressed) {
            return self
                .rt
                .conn
                .apply(vec![chora_kit::wm::focus::give(surface)]);
        }
        self.walk_keys = true;
        self.rt.conn.apply(crate::modes::the_keyboard_to_the_look())
    }

    /// A surface lost its parent and owner: an own window leaves the
    /// book and comes back as a proposal, as the core says that it is
    /// free again or reserved for the construct, which attaches it anew.
    pub(crate) fn window_detached(&mut self, surface: &str) -> Result<(), ClientError> {
        self.take_free(surface, true)
    }

    /// The window of a surface that is free again, or comes with a drag
    /// (`fly` off): the book forgets it and claims it with the
    /// reservation that the core has for it; the construct attaches it.
    pub(crate) fn take_free(&mut self, surface: &str, fly: bool) -> Result<(), ClientError> {
        let node = self.rt.conn.query(&format!("id:{surface}"))?.pop();
        let to = node.and_then(|n| n.to);
        let was = self.book.forget(surface);
        let changes = self.book.claim(surface, to.as_deref(), None);
        let lost = was && changes.is_empty();
        self.settle(changes, &mut |construct, id| {
            construct.take_detached(id, fly)
        })?;
        if lost {
            self.forget_window(surface)?;
        }
        Ok(())
    }

    /// A window closed or went: the book tells the phase and lets it
    /// go; the pane of a window that the construct only shows goes too.
    pub(crate) fn window_destroyed(
        &mut self,
        event: &Event,
        surface: &str,
    ) -> Result<(), ClientError> {
        let owned = self.book.has(surface);
        let changes = self.book.apply(event);
        self.settle_plain(changes)?;
        if owned {
            Ok(())
        } else {
            self.forget_window(surface)
        }
    }

    /// Does what the book says, in order. `take` attaches the window of
    /// a proposal that has no pane yet; each proposal is answered by
    /// whether the window is the construct's now.
    pub(crate) fn settle(
        &mut self,
        changes: Vec<Change>,
        take: &mut dyn FnMut(&mut Self, &str) -> Result<(), ClientError>,
    ) -> Result<(), ClientError> {
        for change in changes {
            match change {
                Change::Adopt { id, .. } => {
                    let done = take(self, &id);
                    let next = if self.owns(&id) {
                        self.book.confirm(&id)
                    } else {
                        self.book.refused(&id)
                    };
                    self.settle_plain(next)?;
                    done?;
                }
                Change::Forget {
                    why: Why::Detached | Why::Given(_),
                    ..
                }
                | Change::Outputs => {}
                Change::Forget { id, .. } => self.forget_window(&id)?,
                Change::Phase { id, phase } => self.tell_phase(&id, phase),
                Change::Apply(commands) => self.rt.conn.apply(commands)?,
            }
        }
        Ok(())
    }

    /// Settles changes of a window that is attached already.
    pub(crate) fn settle_plain(&mut self, changes: Vec<Change>) -> Result<(), ClientError> {
        self.settle(changes, &mut |_, _| Ok(()))
    }

    /// Whether the window is the construct's: an attached pane, a window
    /// hidden under the space, or the window of the launcher.
    pub(crate) fn owns(&self, surface: &str) -> bool {
        self.panes
            .iter()
            .any(|p| p.attached && p.window() == Some(surface))
            || self.hidden.iter().any(|h| h == surface)
            || self.launcher_window.as_deref() == Some(surface)
    }

    /// The pane, the hidden window or the launcher window of a surface
    /// that closed: its data goes; the keyboard goes back to the space
    /// after the window of the launcher.
    fn forget_window(&mut self, surface: &str) -> Result<(), ClientError> {
        if self.launcher_window.as_deref() == Some(surface) {
            self.launcher_window = None;
            let walk = self.keys_to_the_walk();
            return self.rt.conn.apply(vec![walk]);
        }
        self.hidden.retain(|s| s != surface);
        self.forget(surface)
    }

    /// Emits `construct.window` for a phase. A refused emit only logs.
    pub(crate) fn tell_phase(&mut self, surface: &str, phase: Phase) {
        let emit = self.book.phase_event(surface, phase, None);
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", emit) {
            crate::log::log_warn!("chora-construct: construct.window: {err}");
        }
    }

    /// The construct gives an own window to `to`: a proposal of the
    /// book that the core settles. False when the window is not its own.
    pub(crate) fn propose_give(&mut self, surface: &str, to: &str) -> bool {
        self.book.give(surface, to)
    }

    /// The core took the give: the phase `given`, and the pane goes.
    pub(crate) fn gave(&mut self, surface: &str) -> Result<(), ClientError> {
        let next = self.book.confirm(surface);
        self.settle_plain(next)
    }

    /// The core refused the give: the window stays.
    pub(crate) fn give_refused(&mut self, surface: &str) {
        self.book.refused(surface);
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::{
        runtime::Plugin,
        settings,
        testing::{
            FakeCore, Script, Subject, created, created_child, focus_changed, output, owner,
            surface,
        },
    };
    use chora_scene::{Event, Node};

    use crate::{Construct, NAME, config};

    /// The construct as the flavor, built on a fake core of the test.
    fn flavor(core: &FakeCore, tag: &str) -> Construct {
        let mut rt = core.runtime(NAME);
        rt.pin_accepts_injected(false);
        let dir = chora_kit::testing::TempDir::unmade(&format!("chora-construct-{tag}")).keep();
        let file = dir.join("chora-construct.yaml");
        let settings = settings::File::load(file, config::DEFAULT).unwrap();
        let shaders = |name: &str, _: &str| Ok(dir.join(name).to_string_lossy().into_owned());
        let mut construct = Construct::new(rt, settings, false, &shaders).unwrap();
        core.model().lock().unwrap().scene.add_overlay("test");
        construct.output = Some(output("test"));
        construct.active = true;
        construct.metrics.keep_estimates();
        construct.build().unwrap();
        construct
    }

    /// A chord that arrives while the space of the construct stands and no
    /// key is held has no holder, and the core drops it (sh-44, sh-47: the
    /// gates take the nodes of the owner for the start). So the construct
    /// holds its keys before its first node exists.
    #[test]
    fn the_construct_holds_its_keys_before_its_first_node_exists() {
        let core = FakeCore::start();
        core.outputs(serde_json::json!([output("test")]));
        let calls = core.record(|method, params| match method {
            "input.grab" => Some("grab".to_owned()),
            "scene.apply" if params.to_string().contains("\"create\"") => Some("create".to_owned()),
            _ => None,
        });
        let mut rt = core.runtime(NAME);
        rt.pin_accepts_injected(false);
        let dir = chora_kit::testing::TempDir::unmade("chora-construct-grabs-first").keep();
        let file = dir.join("chora-construct.yaml");
        let settings = settings::File::load(file, config::DEFAULT).unwrap();
        let shaders = |name: &str, _: &str| Ok(dir.join(name).to_string_lossy().into_owned());
        let mut construct = Construct::new(rt, settings, false, &shaders).unwrap();
        core.model().lock().unwrap().scene.add_overlay("test");
        construct.start().unwrap();
        let calls = calls.lock().unwrap().clone();
        let grab = calls.iter().position(|m| m == "grab");
        let create = calls.iter().position(|m| m == "create");
        assert!(
            grab.is_some() && create.is_some(),
            "the construct grabs and builds: {calls:?}"
        );
        assert!(
            grab < create,
            "the first node exists before the first key is held: {:?}",
            &calls[..calls.len().min(4)]
        );
    }

    /// The construct under the script of the life of a window.
    struct Driven<'a>(&'a mut Construct);

    impl Subject for Driven<'_> {
        fn hear(&mut self, event: Event) {
            self.0.event(event).unwrap();
        }

        fn has(&self, surface: &str) -> bool {
            let state = self.0.state().unwrap();
            let windows = state["windows"].as_array().unwrap();
            windows.iter().any(|w| w["surface"] == surface)
        }

        fn give(&mut self, surface: &str, to: &str) -> bool {
            self.0.give(surface, to).is_ok()
        }
    }

    /// The script that every look runs: create, refused attach, reserved
    /// for another window manager, detach (the construct takes the window
    /// again), destroy, output removed (the windows stay), give in,
    /// refused give out, give out.
    #[test]
    fn the_construct_runs_the_script_of_the_life_of_a_window() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "script");
        Script::new(&core, NAME).run(&mut Driven(&mut construct), "test");
    }

    /// The state tells where a window came from, and no token.
    #[test]
    fn the_state_tells_the_origin_of_a_window() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "origin");
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        surface(&core.model(), "s2");
        core.model().lock().unwrap().scene.reserve("s2", Some(NAME));
        construct.event(created("s2", Some(NAME))).unwrap();
        let state = construct.state().unwrap();
        let origin = |surface: &str| {
            let windows = state["windows"].as_array().unwrap();
            let window = windows.iter().find(|w| w["surface"] == surface).unwrap();
            window["origin"].clone()
        };
        assert_eq!(origin("s1"), "new");
        assert_eq!(origin("s2"), "new");
        assert!(!state.to_string().contains("t1"), "no token in the state");
    }

    /// A restart takes the free surfaces and those reserved for the
    /// construct, and leaves those of another manager. The construct
    /// writes no meta for its windows (the binary of 29e2cd77 neither),
    /// so there is none to read.
    #[test]
    fn a_restart_adopts_the_free_windows_of_the_construct() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "restart");
        for id in ["s1", "s2", "s3", "s4"] {
            surface(&core.model(), id);
        }
        let node = |id: &str, owner: Option<&str>, to: Option<&str>| -> Node {
            let mut node = core.model().lock().unwrap().scene.node(id).unwrap().clone();
            node.owner = owner.map(Into::into);
            node.to = to.map(Into::into);
            node
        };
        let nodes = vec![
            node("s1", None, None),
            node("s2", None, Some(NAME)),
            node("s3", None, Some("xp")),
            node("s4", Some("xp"), None),
        ];
        construct.adopt_free(nodes).unwrap();
        let driven = Driven(&mut construct);
        assert!(driven.has("s1"));
        assert!(driven.has("s2"));
        assert!(!driven.has("s3"));
        assert!(!driven.has("s4"));
        assert_eq!(owner(&core.model(), "s1").as_deref(), Some(NAME));
        assert_eq!(owner(&core.model(), "s3"), None);
        let phases = chora_kit::testing::phases(&core.model(), NAME);
        let opened: Vec<_> = phases.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(opened, ["s1", "s2"], "each adopted window opens once");
    }

    /// Beside another flavor the construct takes no free window of its
    /// own: the book proposes it, the construct refuses, and nothing
    /// opens (F30).
    #[test]
    fn an_overlay_takes_no_free_window() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "overlay");
        construct.overlay = true;
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        assert!(!Driven(&mut construct).has("s1"));
        assert_eq!(owner(&core.model(), "s1"), None);
        assert_eq!(chora_kit::testing::phases(&core.model(), NAME).len(), 0);
    }

    /// A window that the core reserved for another window manager is
    /// no pane, even right after the construct launched a program: only
    /// a window that the book claimed is the construct's, so that its
    /// phases, its origin and its give agree with its panes (rw-1).
    #[test]
    fn a_launch_adopts_no_window_reserved_for_another_manager() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "reserved");
        construct.launched = Some(std::time::Instant::now());
        surface(&core.model(), "s1");
        construct.event(created("s1", Some("mc"))).unwrap();
        assert!(!Driven(&mut construct).has("s1"));
        assert_eq!(owner(&core.model(), "s1"), None);
        assert_eq!(chora_kit::testing::phases(&core.model(), NAME).len(), 0);
    }

    /// co-27 (D66): the window of a program that an overlay construct
    /// started is a pane of it even when the core reserved the surface for
    /// the window manager below (tiling), as the overlay only shows it: the
    /// book does not claim it, the construct owns it not, and no phase opens.
    #[test]
    fn an_overlay_shows_the_window_of_its_own_program_reserved_for_the_manager_below() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "overlay-own");
        construct.overlay = true;
        let pane = crate::PaneFile {
            kind: crate::Kind::Program,
            command: Some("foot".into()),
            plugin: None,
            scene: None,
            manager: None,
            at: None,
            yaw: None,
            size: None,
            placed: false,
        };
        construct
            .waiting
            .push(crate::Waiting::new(Some("t1".into()), pane));
        surface(&core.model(), "s1");
        core.model()
            .lock()
            .unwrap()
            .scene
            .reserve("s1", Some("tiling"));
        construct.event(created("s1", Some("tiling"))).unwrap();
        let shown = construct.panes.iter().any(|p| p.window() == Some("s1"));
        assert!(shown, "the own program of the overlay is a pane");
        assert!(
            !Driven(&mut construct).has("s1"),
            "the book claims no window of tiling"
        );
        assert_eq!(owner(&core.model(), "s1"), None);
        assert_eq!(chora_kit::testing::phases(&core.model(), NAME).len(), 0);
    }

    /// The guard of co-27 (rv-2): an overlay shows a window that the core
    /// reserved for the manager below only when the window carries the
    /// token of a program it waits for. A stranger without a token, one
    /// with another token, and one right after a launch are no pane, even
    /// beside a waiting pane without a token (a plugin or a manager), and
    /// the waited program keeps waiting.
    #[test]
    fn an_overlay_shows_no_reserved_window_without_a_waited_token() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "overlay-stranger");
        construct.overlay = true;
        construct.launched = Some(std::time::Instant::now());
        let pane = |kind| crate::PaneFile {
            kind,
            command: Some("foot".into()),
            plugin: None,
            scene: None,
            manager: None,
            at: None,
            yaw: None,
            size: None,
            placed: false,
        };
        let waiting = [
            crate::Waiting::new(Some("t1".into()), pane(crate::Kind::Program)),
            crate::Waiting::new(None, pane(crate::Kind::Manager)),
        ];
        construct.waiting.extend(waiting);
        for (surface, token) in [("s1", None), ("s2", Some("t2"))] {
            surface_of(&core, surface);
            let event = chora_kit::testing::created_by(surface, "probe", Some("tiling"), token);
            construct.event(event).unwrap();
            let shown = construct.panes.iter().any(|p| p.window() == Some(surface));
            assert!(!shown, "{surface} with the token {token:?} is no pane");
            assert_eq!(owner(&core.model(), surface), None);
        }
        assert_eq!(construct.waiting.len(), 2, "the waited program still waits");
        assert_eq!(chora_kit::testing::phases(&core.model(), NAME).len(), 0);
    }

    /// A surface `id` that the core reserved for tiling.
    fn surface_of(core: &FakeCore, id: &str) {
        surface(&core.model(), id);
        core.model()
            .lock()
            .unwrap()
            .scene
            .reserve(id, Some("tiling"));
    }

    /// co-29: the window list opens under a pointer that stood still, so a
    /// row is hovered at once and the list is drawn again within its fade;
    /// the redraw carries on the fade for the time left. The list opened
    /// 60 ms ago, so the carried fade ends within 110 ms, where neither the
    /// stopped fade nor a fresh `FADE_IN` of 160 ms has reached 1 (rv-1).
    #[test]
    fn a_row_hovered_while_the_list_fades_in_leaves_it_fully_opaque() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "list-hover");
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        construct.open_list().unwrap();
        let back = std::time::Duration::from_millis(60);
        let list = construct.list.as_mut().unwrap();
        list.opened = list.opened.checked_sub(back).unwrap();
        core.model().lock().unwrap().scene.advance(60.0);
        construct
            .event(Event::PointerEntered {
                node: format!("{}0", crate::command::LIST_ROW),
                moved: false,
                injected: false,
            })
            .unwrap();
        let model = core.model();
        let mut model = model.lock().unwrap();
        model.scene.advance(110.0);
        let opacity = model.scene.node(crate::hud::LIST).unwrap().props.opacity;
        assert!(
            (opacity - 1.0).abs() < f64::EPSILON,
            "the list fades in for the time left, not anew: {opacity}"
        );
    }

    /// FLK-DOORS-CO: a regrab lets go of the chords of the construct when
    /// the keyboard goes back to the walk, and the core may have a press
    /// of one on its way: the event comes for a binding that the plugin
    /// holds no more. In the walk the key is the key of the walk, so
    /// Super+space still starts the launcher (gate doors, cell
    /// launcher-key after a step out of a pane).
    #[test]
    fn a_chord_that_a_regrab_let_go_acts_as_a_key_of_the_walk() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "chord-let-go");
        assert!(!construct.rt.holds("Super+space"));
        assert!(construct.launched.is_none());
        construct
            .event(Event::InputKey {
                binding: "Super+space".into(),
                state: chora_scene::ipc::PressState::Pressed,
                injected: false,
            })
            .unwrap();
        assert!(
            construct.launched.is_some(),
            "the chord of the launcher did nothing in the walk"
        );
    }

    /// co-hud-load r1: under load the scene animation lags the wall time.
    /// A redraw after the 160 ms of the fade asks for no fade, and the
    /// update of the redraw stops the one that runs, short of 1; the list
    /// stays at 0.97 for good (gate-construct-hud, Tab after a pane.move).
    #[test]
    fn a_row_hovered_after_the_fade_time_leaves_the_list_fully_opaque() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "list-hover-late");
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        construct.open_list().unwrap();
        let back = std::time::Duration::from_millis(200);
        let list = construct.list.as_mut().unwrap();
        list.opened = list.opened.checked_sub(back).unwrap();
        core.model().lock().unwrap().scene.advance(150.0);
        construct
            .event(Event::PointerEntered {
                node: format!("{}0", crate::command::LIST_ROW),
                moved: false,
                injected: false,
            })
            .unwrap();
        let model = core.model();
        let mut model = model.lock().unwrap();
        model.scene.advance(1000.0);
        let opacity = model.scene.node(crate::hud::LIST).unwrap().props.opacity;
        assert!(
            (opacity - 1.0).abs() < f64::EPSILON,
            "the list is opaque after its fade time: {opacity}"
        );
    }

    /// FLK-HUD-TAB: under load the core starts and steps the fade later
    /// than the plugin counts it, so a redraw can come when the plugin's
    /// clock says the 160 ms are over and the core's fade has 20 ms left.
    /// The redraw must not leave the list at the opacity of that moment
    /// (0.998 in the failing runs, where Tab then "opened no list").
    #[test]
    fn a_redraw_after_the_fade_time_leaves_a_late_list_fully_opaque() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "list-late");
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        construct.open_list().unwrap();
        let back = std::time::Duration::from_millis(200);
        let list = construct.list.as_mut().unwrap();
        list.opened = list.opened.checked_sub(back).unwrap();
        core.model().lock().unwrap().scene.advance(140.0);
        construct
            .event(Event::PointerEntered {
                node: format!("{}0", crate::command::LIST_ROW),
                moved: false,
                injected: false,
            })
            .unwrap();
        let model = core.model();
        let mut model = model.lock().unwrap();
        model.scene.advance(1000.0);
        let opacity = model.scene.node(crate::hud::LIST).unwrap().props.opacity;
        assert!(
            (opacity - 1.0).abs() < f64::EPSILON,
            "the redraw stopped the fade of the list: {opacity}"
        );
    }

    /// kit7-a2 (D116): a press on the window of a pane gives it the
    /// keyboard; a new window that comes as a pane without a flight takes
    /// the keyboard back to the walk, as a new window takes it in the
    /// other looks, so that Tab opens the window list and no key goes to
    /// the window pointed at before.
    #[test]
    fn a_new_pane_takes_the_keyboard_back_to_the_walk_after_a_press() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "press-then-new");
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        construct
            .event(Event::PointerPressed {
                node: "s1".into(),
                button: 272,
                position: [0.0; 2],
                local: [0.0; 2],
                modifiers: Vec::new(),
                injected: false,
            })
            .unwrap();
        let last = || core.model().lock().unwrap().focuses.last().cloned();
        assert_eq!(last(), Some(Some("s1".into())), "the press");
        construct.event(focus_changed(Some("s1"))).unwrap();
        surface(&core.model(), "s2");
        construct.event(created("s2", None)).unwrap();
        assert!(Driven(&mut construct).has("s2"));
        assert_eq!(last(), Some(None), "the new pane gives no surface the keys");
        construct.event(focus_changed(None)).unwrap();
        let focus = construct.state().unwrap()["focus"].clone();
        assert_eq!(focus, serde_json::json!({"holder": NAME}), "the walk");
    }

    /// kit7-a2, rc-1 (D116): a window that the pressed window opens as its
    /// child (a dialog with an xdg parent) gets the keyboard, as in the
    /// other looks; a new window that is no child of it does not.
    #[test]
    fn a_dialog_of_the_pressed_window_gets_the_keyboard_and_an_unrelated_window_does_not() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "press-then-dialog");
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        construct
            .event(Event::PointerPressed {
                node: "s1".into(),
                button: 272,
                position: [0.0; 2],
                local: [0.0; 2],
                modifiers: Vec::new(),
                injected: false,
            })
            .unwrap();
        construct.event(focus_changed(Some("s1"))).unwrap();
        let last = || core.model().lock().unwrap().focuses.last().cloned();
        surface(&core.model(), "s2");
        construct.event(created_child("s2", "s1")).unwrap();
        assert!(Driven(&mut construct).has("s2"));
        assert_eq!(last(), Some(Some("s2".into())), "the dialog gets the keys");
        construct.event(focus_changed(Some("s2"))).unwrap();
        surface(&core.model(), "s3");
        construct.event(created("s3", None)).unwrap();
        assert_eq!(
            last(),
            Some(None),
            "an unrelated window gives them to the walk"
        );
    }

    /// A new pane that is the child of a window other than the pressed
    /// one is no child of it: the keys go back to the walk.
    #[test]
    fn a_child_of_another_window_gives_the_keyboard_to_the_walk() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "press-then-other-child");
        for id in ["s1", "s2"] {
            surface(&core.model(), id);
            construct.event(created(id, None)).unwrap();
        }
        construct
            .event(Event::PointerPressed {
                node: "s1".into(),
                button: 272,
                position: [0.0; 2],
                local: [0.0; 2],
                modifiers: Vec::new(),
                injected: false,
            })
            .unwrap();
        construct.event(focus_changed(Some("s1"))).unwrap();
        surface(&core.model(), "s3");
        construct.event(created_child("s3", "s2")).unwrap();
        let last = core.model().lock().unwrap().focuses.last().cloned();
        assert_eq!(last, Some(None));
    }

    /// A new pane leaves the keyboard where another window manager put
    /// it: the construct takes it only from its own window.
    #[test]
    fn a_new_pane_leaves_a_foreign_focus() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "foreign-then-new");
        construct.event(focus_changed(Some("x1"))).unwrap();
        let before = core.model().lock().unwrap().focuses.len();
        surface(&core.model(), "s2");
        construct.event(created("s2", None)).unwrap();
        assert_eq!(core.model().lock().unwrap().focuses.len(), before);
    }

    /// The views and the faces of the cube stay as they are while
    /// windows come and go: a window that closes takes only its own pane.
    #[test]
    fn the_views_stay_while_windows_come_and_go() {
        let core = FakeCore::start();
        let mut construct = flavor(&core, "faces");
        let before = construct.state().unwrap()["views"].clone();
        surface(&core.model(), "s1");
        construct.event(created("s1", None)).unwrap();
        construct
            .event(Event::SurfaceDestroyed {
                surface: "s1".into(),
            })
            .unwrap();
        assert_eq!(construct.state().unwrap()["views"], before);
    }
}
