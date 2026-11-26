//! The life of a window of tiling through `kit::wm::book` (slice WM1):
//! the book decides which windows are tiling's, tiling settles its
//! proposals and keeps where and how the windows show.

use chora_kit::wm::{Book, Change, Origin, Phase, Why};
use chora_scene::{Event, ipc::ClientError};
use serde_json::Value;

use crate::{NAME, Tiling, commands::GROUP, wm};

/// The book of tiling: its windows carry groups `tw-`, and its meta keys
/// of before the versioned key stay its own.
pub(crate) fn book(filters: bool) -> Book {
    Book::new(NAME, filters)
        .with_groups(GROUP)
        .with_legacy(wm::legacy_key)
}

impl Tiling {
    /// An event of the life of a window: the outputs go to the model of
    /// tiling, then the book decides, and tiling settles and lays out.
    pub(crate) fn lifecycle(&mut self, event: &Event) -> Result<(), ClientError> {
        match event {
            Event::OutputChanged { output } => {
                let full = crate::full_area(output);
                self.wm.set_output(&output.name, output.layout_area(), full);
            }
            Event::OutputRemoved { name } => self.wm.remove_output(name),
            _ => {}
        }
        let changes = self.book.apply(event);
        self.settle(changes)?;
        self.relayout()
    }

    /// Does what the book says, in order. A detached window of tiling
    /// leaves only the model: tiling adopts it anew in the same frame. A
    /// window given in while a window or the desktop of another look holds
    /// the keyboard comes without it ([`Focus::takes_given`], D119).
    ///
    /// [`Focus::takes_given`]: chora_kit::wm::focus::Focus::takes_given
    pub(crate) fn settle(&mut self, changes: Vec<Change>) -> Result<(), ClientError> {
        for change in changes {
            match change {
                Change::Adopt { id, origin } => {
                    let keys =
                        origin != Origin::Given || self.focus.takes_given(NAME, |s| self.wm.has(s));
                    let adopted = self.adopt(&id)?;
                    if adopted && !keys {
                        self.focus.lose();
                    }
                    let next = if adopted {
                        self.book.confirm_keys(&id, keys)
                    } else {
                        self.book.refused(&id)
                    };
                    self.settle(next)?;
                }
                Change::Forget {
                    id,
                    why: Why::Detached,
                } => self.wm.remove_window(&id),
                Change::Forget { id, why } => self.forget_window(&id, &why),
                Change::Phase { id, phase } => self.tell_phase(&id, phase)?,
                Change::Apply(commands) => self.apply(commands)?,
                Change::Outputs => {}
            }
        }
        Ok(())
    }

    /// Emits `tiling.window` for a phase, with the rect of the window
    /// where tiling placed it. A refused emit only logs.
    fn tell_phase(&mut self, id: &str, phase: Phase) -> Result<(), ClientError> {
        let placed = self.sent.get(id);
        let rect = placed.and_then(|p| {
            let [x, y] = p.at?;
            Some([x, y, p.frame[0], p.frame[1]].map(f64::from))
        });
        let emit = self.book.phase_event(id, phase, rect);
        match self.rt.conn.call::<_, Value>("events.emit", emit) {
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: tiling.window: {err}");
                Ok(())
            }
            done => done.map(drop),
        }
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::testing::{FakeCore, Model};
    use chora_scene::{Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceInfo};
    use serde_json::{Value, json};

    use crate::{NAME, Tiling, tests::tiling_on};

    /// A tiling on the fake core of kit, with its group and the output
    /// `A`, as after its start.
    fn on_kit_core(core: &FakeCore, tag: &str) -> Tiling {
        let dir = chora_kit::testing::TempDir::new(&format!("chora-tiling-{tag}")).keep();
        std::fs::create_dir_all(&dir).unwrap();
        let mut tiling = tiling_on(core.runtime(NAME), &dir, &[]);
        let group = Command::Create {
            id: NAME.into(),
            kind: NodeKind::Group,
            parent: ROOT.into(),
            props: PropsUpdate::default(),
        };
        tiling.call_apply(vec![group]).unwrap();
        let output: OutputInfo = serde_json::from_value(json!({"name": "A",
            "size": [800, 600], "position": [0, 0], "scale": 1.0,
            "camera": chora_scene::Transform::default(),
            "work_area": [0, 0, 800, 600], "default_work_area": [0, 0, 800, 600]}))
        .unwrap();
        tiling.handle(Event::OutputChanged { output }).unwrap();
        tiling
    }

    fn open(core: &FakeCore, tiling: &mut Tiling, id: &str) {
        let info = SurfaceInfo {
            app_id: Some("probe".into()),
            ..SurfaceInfo::default()
        };
        core.model().lock().unwrap().scene.add_surface(id, info);
        let created = serde_json::from_value(json!({"event": "surface.created",
            "surface": id, "app_id": "probe", "title": null}))
        .unwrap();
        tiling.handle(created).unwrap();
    }

    fn phases(model: &Model) -> Vec<Value> {
        let core = model.lock().unwrap();
        (core.emits.iter())
            .filter(|e| e["event"] == "tiling.window")
            .map(|e| json!([e["data"]["surface"], e["data"]["phase"], e["data"]["node"]]))
            .collect()
    }

    fn props(model: &Model, id: &str) -> chora_scene::Props {
        model.lock().unwrap().scene.node(id).unwrap().props.clone()
    }

    #[test]
    fn a_window_that_opens_and_closes_tells_its_phases() {
        let core = FakeCore::start();
        let mut tiling = on_kit_core(&core, "wm1-phases");
        open(&core, &mut tiling, "s1");
        assert!(tiling.wm.has("s1"));
        let destroyed = Event::SurfaceDestroyed {
            surface: "s1".into(),
        };
        tiling.handle(destroyed).unwrap();
        assert!(!tiling.wm.has("s1"));
        assert_eq!(
            phases(&core.model()),
            [
                json!(["s1", "opened", "tw-s1"]),
                json!(["s1", "closed", "tw-s1"])
            ]
        );
    }

    #[test]
    fn a_foreign_look_passes_through_on_the_root_and_the_groups() {
        let core = FakeCore::start();
        let mut tiling = on_kit_core(&core, "wm1-d58");
        open(&core, &mut tiling, "s1");
        for node in [NAME, "tw-s1", "s1"] {
            let asked = Event::NodeRequested {
                node: node.into(),
                client: "effects".into(),
                action: "update".into(),
                args: json!({"opacity": 0.5, "meta": {"kept.1.window": "x", "window": "y", "role": "a"}}),
            };
            tiling.handle(asked).unwrap();
            let props = props(&core.model(), node);
            assert!((props.opacity - 0.5).abs() < 1e-9, "{node}: {props:?}");
            assert_eq!(
                props.meta.get("role").map(String::as_str),
                Some("a"),
                "{node}"
            );
            assert_ne!(
                props.meta.get("kept.1.window").map(String::as_str),
                Some("x"),
                "{node}"
            );
        }
    }

    #[test]
    fn the_meta_of_a_restart_goes_under_the_versioned_key() {
        let core = FakeCore::start();
        let mut tiling = on_kit_core(&core, "wm1-kept");
        open(&core, &mut tiling, "s1");
        let window = props(&core.model(), "s1").meta;
        assert!(window.contains_key("kept.1.window"), "{window:?}");
        assert!(!window.contains_key("window"), "{window:?}");
        let root = props(&core.model(), NAME).meta;
        assert!(root.contains_key("kept.1.current"), "{root:?}");
        assert!(root.contains_key("kept.1.tags:A"), "{root:?}");
        assert!(!root.contains_key("current"), "{root:?}");
    }

    /// The meta that the binary of 29e2cd77 wrote: tag 2 of `A` shows,
    /// in the layout `fair`; s2 has the focus.
    fn today(window: &str, current: &str, tags: &str) -> Vec<Value> {
        let s = |tags: u16, at: usize, rank: usize| {
            json!([{"output": "A", "tags": tags, "floating": false, "fullscreen": false,
                "maximized": [false, false], "minimized": false, "on_top": false,
                "urgent": false, "size": null, "at": null}, at, rank, rank])
            .to_string()
        };
        vec![
            json!({"id": NAME, "kind": "group", "meta": {current: "A",
                tags: r#"[2,1,[["tile",0.5,1,1],["fair",0.5,1,1]]]"#}}),
            json!({"id": "tw-s1", "kind": "group"}),
            json!({"id": "s1", "kind": "surface", "meta": {window: s(2, 1, 0)}}),
            json!({"id": "s2", "kind": "surface", "meta": {window: s(2, 0, 1)}}),
        ]
    }

    fn restored(nodes: &[Value]) {
        let core = FakeCore::start();
        let mut tiling = on_kit_core(&core, "wm1-restore");
        tiling.restore(nodes);
        let ids: Vec<&str> = tiling.wm.windows.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids, ["s2", "s1"]);
        assert_eq!(tiling.wm.focused(), Some("s2"));
        assert_eq!(tiling.wm.current_tag_index(), 1);
        let layout = tiling.wm.current().map(|(layout, _)| layout);
        assert_eq!(layout, Some(crate::layout::Layout::Fair));
        assert!(tiling.groups.contains("s1"));
    }

    #[test]
    fn a_restart_restores_from_the_keys_of_today() {
        restored(&today("window", "current", "tags:A"));
    }

    #[test]
    fn a_restart_restores_from_the_versioned_key() {
        restored(&today("kept.1.window", "kept.1.current", "kept.1.tags:A"));
    }
}
