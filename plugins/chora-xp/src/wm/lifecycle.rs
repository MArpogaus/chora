//! The life of a window of xp (WM1d): the windows that it owns come and
//! go through `kit::wm::book`. The book decides which surfaces are free
//! for xp (the claim filter) and keeps where each came from; xp attaches
//! them with their frames and answers each proposal with `confirm` or
//! `refused`. What a window does once it is xp's (the frame, the focus,
//! the taskbar) stays in `windows.rs`.

use chora_kit::wm::{Change, Created, Origin, Phase, Why};
use chora_scene::{Event, ipc::ClientError};
use serde_json::Value;

use super::Wm;

impl Wm {
    /// A surface was created: the launch that waited for it is done, and
    /// the book says whether the surface is free for xp.
    pub(super) fn window_created(&mut self, created: &Created) -> Result<(), ClientError> {
        if let Some(note) = self.launches.window(created.token.as_deref()) {
            self.dismiss_notice(note);
        }
        let changes = self.book.claim(
            &created.surface,
            created.to.as_deref(),
            created.token.clone(),
        );
        self.settle(changes, Some(created))
    }

    /// A surface lost its parent and owner, or closed: an own window
    /// leaves xp, and a detached one comes back as a proposal, as the
    /// core says that it is free again.
    pub(super) fn window_left(&mut self, event: &Event) -> Result<(), ClientError> {
        let changes = self.book.apply(event);
        self.settle(changes, None)
    }

    /// Does what the book says, in order. `facts` are what the core said
    /// of a created surface; a proposal without them asks the core.
    pub(super) fn settle(
        &mut self,
        changes: Vec<Change>,
        facts: Option<&Created>,
    ) -> Result<(), ClientError> {
        for change in changes {
            match change {
                Change::Adopt { id, origin } => {
                    let (app_id, title) = facts
                        .filter(|created| created.surface == id)
                        .map_or((None, None), |c| (c.app_id.clone(), c.title.as_deref()));
                    let keys = self.takes_keys(&origin);
                    let next = match self.adopt(&id, app_id, title, keys) {
                        Ok(true) => self.book.confirm_keys(&id, keys),
                        Ok(false) => self.book.refused(&id),
                        Err(err) => {
                            self.book.refused(&id);
                            return Err(err);
                        }
                    };
                    self.settle(next, facts)?;
                }
                Change::Forget {
                    why: Why::Given(_), ..
                }
                | Change::Outputs => {}
                Change::Forget { id, .. } => {
                    let commands = self.forget(&id);
                    self.apply(commands)?;
                }
                Change::Phase { id, phase } => self.tell_phase(&id, phase),
                Change::Apply(commands) => self.apply(commands)?,
            }
        }
        Ok(())
    }

    /// Whether a window that comes from `origin` takes the keyboard: a
    /// window given in does not while a window or the desktop of another
    /// look holds it, as tiling keeps it on the output where the person
    /// is when it sends a window to xp on another output (D119). Another
    /// node, as the walk of the construct, holds it only for a moment.
    fn takes_keys(&self, origin: &Origin) -> bool {
        *origin != Origin::Given || !(self.focus_away() || self.desk_away())
    }

    /// Emits `xp.window` for a phase, with the frame of the window while
    /// it shows. A refused emit only logs.
    pub(super) fn tell_phase(&mut self, surface: &str, phase: Phase) {
        let rect = (self.index(surface))
            .filter(|&i| self.windows[i].shown)
            .map(|i| self.windows[i].rect().map(f64::from));
        let emit = self.book.phase_event(surface, phase, rect);
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", emit) {
            tracing::warn!("xp.window: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chora_kit::{
        give::Windows as _,
        runtime::Plugin as _,
        testing::{
            FakeCore, Model, Script, Subject, created, focus_changed, owner, phases, surface,
        },
    };
    use chora_scene::{Command, Event, NodeKind, PropsUpdate, ROOT, SurfaceInfo};
    use serde_json::Value;

    use crate::wm::{NAME, Wm, tests::wm_on_kit_core};

    /// xp under the script of the life of a window.
    struct Driven<'a>(&'a mut Wm);

    impl Subject for Driven<'_> {
        fn hear(&mut self, event: Event) {
            self.0.event(event).unwrap();
        }

        fn has(&self, surface: &str) -> bool {
            self.0.has(surface)
        }

        fn give(&mut self, surface: &str, to: &str) -> bool {
            self.0.hand(surface, to).is_ok()
        }
    }

    /// The script that every look runs: create, refused attach, reserved
    /// for another window manager, detach (xp attaches the window anew),
    /// destroy, output removed (the windows stay), give in, refused give
    /// out, give out.
    #[test]
    fn xp_runs_the_script_of_the_life_of_a_window() {
        let core = FakeCore::start();
        let mut xp = wm_on_kit_core(&core, "script");
        Script::new(&core, NAME)
            .beside("mc")
            .run(&mut Driven(&mut xp), "test");
    }

    /// The origin that the state tells of a window.
    fn origin(xp: &Wm, id: &str) -> Value {
        let state = xp.state().unwrap();
        let windows = state["windows"].as_array().unwrap();
        windows.iter().find(|w| w["surface"] == id).unwrap()["origin"].clone()
    }

    /// The state tells where a window came from, and no token.
    #[test]
    fn the_state_tells_the_origin_of_a_window() {
        let core = FakeCore::start();
        let mut xp = wm_on_kit_core(&core, "origin");
        surface(&core.model(), "s1");
        xp.event(created("s1", None)).unwrap();
        surface(&core.model(), "s2");
        core.model().lock().unwrap().scene.reserve("s2", Some(NAME));
        xp.event(created("s2", Some(NAME))).unwrap();
        assert_eq!(origin(&xp, "s1"), "new");
        assert_eq!(origin(&xp, "s2"), "new");
        let state = xp.state().unwrap();
        assert!(!state.to_string().contains("t1"), "no token: {state}");
    }

    /// D119, ti-57: a window given to xp while a window of another look
    /// holds the keyboard (tiling kept it on the output where the person
    /// is) comes without the keyboard: xp does not focus it and holds
    /// none. Without another holder the next window given in takes the
    /// keyboard, as before.
    #[test]
    fn a_window_given_in_leaves_the_keyboard_with_another_holder() {
        let core = FakeCore::start();
        let mut xp = wm_on_kit_core(&core, "given-keys");
        let model = core.model();
        surface(&model, "s9");
        xp.event(focus_changed(Some("s9"))).unwrap();
        surface(&model, "s1");
        model.lock().unwrap().scene.reserve("s1", Some(NAME));
        xp.event(Event::SurfaceDetached {
            surface: "s1".into(),
        })
        .unwrap();
        assert!(xp.has("s1"), "xp took the window");
        assert_eq!(origin(&xp, "s1"), "given");
        let focused = model
            .lock()
            .unwrap()
            .scene
            .focused_surface()
            .map(String::from);
        assert_ne!(
            focused.as_deref(),
            Some("s1"),
            "xp focused the window given in"
        );
        let state = xp.state().unwrap();
        assert_eq!(state["focus"]["holder"], Value::Null, "{}", state["focus"]);

        xp.event(focus_changed(None)).unwrap();
        surface(&model, "s2");
        model.lock().unwrap().scene.reserve("s2", Some(NAME));
        xp.event(Event::SurfaceDetached {
            surface: "s2".into(),
        })
        .unwrap();
        let focused = model
            .lock()
            .unwrap()
            .scene
            .focused_surface()
            .map(String::from);
        assert_eq!(focused.as_deref(), Some("s2"));
    }

    /// A surface of the core with a size, as a client that mapped it.
    fn mapped(model: &Model, id: &str) {
        let info = SurfaceInfo {
            app_id: Some("probe".into()),
            size: Some([200, 100]),
            ..SurfaceInfo::default()
        };
        model.lock().unwrap().scene.add_surface(id, info);
    }

    /// What the binary of 29e2cd77 left in the meta of a group before its
    /// restart: the same keys and the same JSON.
    fn left_by_the_old_binary(core: &FakeCore) {
        let model = core.model();
        let meta = |json: &str| PropsUpdate {
            meta: Some(BTreeMap::from([("xp".to_owned(), json.to_owned())])),
            ..PropsUpdate::default()
        };
        let group = |id: &str, parent: &str, json: &str| Command::Create {
            id: id.into(),
            kind: NodeKind::Group,
            parent: parent.into(),
            props: meta(json),
        };
        let mut commands = vec![group("xp", ROOT, "1")];
        let windows = [
            (
                "s1",
                r#"{"frame":[300,120,216,130],"desktop":1,"restore":null,"z":2,"opened":1,"minimized":false}"#,
            ),
            (
                "s2",
                r#"{"frame":[40,60,216,130],"desktop":1,"restore":null,"z":0,"opened":2,"minimized":true}"#,
            ),
            (
                "s3",
                r#"{"frame":[500,200,216,130],"desktop":0,"restore":null,"z":1,"opened":0,"minimized":false}"#,
            ),
        ];
        for (id, json) in windows {
            mapped(&model, id);
            let node = format!("xp-win-{id}");
            commands.push(group(&node, "xp", json));
            commands.push(Command::Attach {
                surface: id.into(),
                parent: node,
            });
        }
        core.connect("xp").apply(commands).unwrap();
    }

    /// xp after a restart: the kept windows of the old binary, one that
    /// appeared meanwhile, and one that the core reserved for mc.
    fn restarted(tag: &str) -> (FakeCore, Wm) {
        let core = FakeCore::start().selecting();
        left_by_the_old_binary(&core);
        mapped(&core.model(), "s4");
        mapped(&core.model(), "s5");
        core.model().lock().unwrap().scene.reserve("s5", Some("mc"));
        let xp = wm_on_kit_core(&core, tag);
        (core, xp)
    }

    /// A restart reads the meta that the binary of 29e2cd77 wrote: the
    /// windows come back by their stacking order, frame, desktop and
    /// minimized state, and the taskbar keeps its order, with a window
    /// that appeared meanwhile after the kept ones.
    #[test]
    fn a_restart_reads_the_meta_of_the_old_binary() {
        let (_core, xp) = restarted("restart-meta");
        let order: Vec<&str> = xp.windows.iter().map(|w| w.surface.as_str()).collect();
        assert_eq!(order[..3], ["s2", "s3", "s1"], "the stacking order");
        assert_eq!(xp.opened, ["s3", "s1", "s2", "s4"], "the taskbar order");
        let at = |id: &str| xp.windows[xp.index(id).unwrap()].pos;
        assert_eq!(at("s1"), [300, 120]);
        assert_eq!(at("s3"), [500, 200]);
        assert!(xp.windows[xp.index("s2").unwrap()].minimized);
        assert_eq!(xp.desktop, 1, "the desktop of the root group");
    }

    /// A restart leaves a window that the core reserved for another
    /// window manager, and each window that it adopts opens once, as
    /// kept or as new.
    #[test]
    fn a_restart_opens_each_adopted_window_once() {
        let (core, xp) = restarted("restart-phases");
        assert_eq!(owner(&core.model(), "s5"), None, "reserved for mc: left");
        assert!(!xp.has("s5"));
        let opened: Vec<String> = phases(&core.model(), NAME)
            .into_iter()
            .map(|(surface, phase)| format!("{surface} {phase}"))
            .collect();
        let want = ["s2", "s3", "s1", "s4"].map(|s| format!("{s} opened"));
        assert_eq!(opened, want, "each adopted window opens once");
        let origins = ["s1", "s2", "s3", "s4"].map(|id| origin(&xp, id));
        assert_eq!(origins, ["kept", "kept", "kept", "new"]);
    }
}
