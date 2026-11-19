//! The life of a window of the construct (WM1c): the windows that it
//! owns come and go through `kit::wm::book`.

#[cfg(test)]
mod tests {
    use chora_kit::{
        runtime::{Plugin, Runtime},
        settings,
        testing::{FakeCore, Script, Subject, created, output, owner, surface},
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
