//! The life of a window of the construct (WM1c): the windows that it
//! owns come and go through `kit::wm::book`. The book decides which
//! surfaces are free for the construct (the claim filter) and keeps where
//! each came from; the construct attaches them as panes and answers each
//! proposal with `confirm` or `refused`. What a pane does with its window
//! (placing, the camera, the keyboard) stays in `adopt.rs` and `list.rs`.

use chora_kit::wm::{Change, Created, Phase, Why};
use chora_scene::Event;
use serde_json::Value;

use super::{ClientError, Command, Construct, LOOK};

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
        done
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
            return self.rt.conn.apply(vec![Command::FocusNode {
                node: Some(LOOK.into()),
            }]);
        }
        self.hidden.retain(|s| s != surface);
        self.forget(surface)
    }

    /// Emits `construct.window` for a phase. A refused emit only logs.
    pub(crate) fn tell_phase(&mut self, surface: &str, phase: Phase) {
        let emit = self.book.phase_event(surface, phase, None);
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", emit) {
            eprintln!("chora-construct: construct.window: {err}");
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
        runtime::{Plugin, Runtime},
        settings,
        testing::{FakeCore, Script, Subject, created, focus_changed, output, owner, surface},
    };
    use chora_scene::{Event, Node};

    use crate::{Construct, NAME, config};

    /// The construct as the flavor, built on a fake core of the test.
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    fn flavor(core: &FakeCore, tag: &str) -> Construct {
        let mut rt = Runtime::with(core.connect(NAME), NAME, Vec::new());
        rt.pin_accepts_injected(false);
        let dir =
            std::env::temp_dir().join(format!("chora-construct-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
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
        assert_eq!(origin("s2"), "given");
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

    /// kit7-a2 (D107): a press on the window of a pane gives it the
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
