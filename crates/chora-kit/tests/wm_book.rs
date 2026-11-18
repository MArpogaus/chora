//! The `FakeCore` script of `kit::wm::book` (slice WM1): a small look that
//! settles the proposals of its book against the fake core, as tiling
//! does. Each cell is one step of the life of a window.
#![allow(clippy::unwrap_used, reason = "a failure is a failed test")]

use chora_kit::{
    conn::Conn,
    testing::{FakeCore, Model},
    wm::{Book, Change, Origin, Phase, Why, give::pass_on},
};
use chora_scene::{Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceInfo};
use serde_json::{Value, json};

const NAME: &str = "look";

/// A look that keeps nothing but its book: a window shows as soon as the
/// core took its attach.
struct Look {
    book: Book,
    conn: Conn,
    /// What the look heard from its book, in order.
    heard: Vec<Change>,
}

impl Look {
    fn new(core: &FakeCore, filters: bool) -> Self {
        let mut conn = core.connect(NAME);
        conn.apply(vec![Command::Create {
            id: NAME.into(),
            kind: NodeKind::Group,
            parent: ROOT.into(),
            props: PropsUpdate::default(),
        }])
        .unwrap();
        let book = Book::new(NAME, filters).with_groups("g-");
        Self {
            book,
            conn,
            heard: Vec::new(),
        }
    }

    fn hear(&mut self, event: &Event) {
        let changes = self.book.apply(event);
        self.settle(changes);
    }

    fn settle(&mut self, changes: Vec<Change>) {
        for change in changes {
            self.heard.push(change.clone());
            match change {
                Change::Adopt { id, .. } => {
                    let attach = Command::Attach {
                        surface: id.clone(),
                        parent: NAME.into(),
                    };
                    let next = match self.conn.try_apply(vec![attach]) {
                        Ok(None) => self.book.confirm(&id),
                        _ => self.book.refused(&id),
                    };
                    self.settle(next);
                }
                Change::Phase { id, phase } => {
                    let emit = self.book.phase_event(&id, phase, None);
                    self.conn.call::<_, Value>("events.emit", emit).unwrap();
                }
                Change::Apply(commands) => {
                    let _ = self.conn.try_apply(commands).unwrap();
                }
                Change::Forget { .. } | Change::Outputs => {}
            }
        }
    }

    fn give(&mut self, id: &str, to: &str) -> bool {
        assert!(self.book.give(id, to), "{id} is not the look's");
        let next = match self.conn.give(id, to) {
            Ok(_) => self.book.confirm(id),
            Err(_) => self.book.refused(id),
        };
        let gone = next.iter().any(|c| matches!(c, Change::Forget { .. }));
        self.settle(next);
        gone
    }
}

fn surface(model: &Model, id: &str) {
    let info = SurfaceInfo {
        app_id: Some("probe".into()),
        ..SurfaceInfo::default()
    };
    model.lock().unwrap().scene.add_surface(id, info);
}

fn created(id: &str, to: Option<&str>) -> Event {
    Event::SurfaceCreated {
        surface: id.into(),
        app_id: Some("probe".into()),
        title: None,
        decoration: None,
        to: to.map(Into::into),
        pid: None,
        token: Some("t1".into()),
    }
}

fn owner(model: &Model, id: &str) -> Option<String> {
    model.lock().unwrap().scene.node(id)?.owner.clone()
}

fn phases(model: &Model) -> Vec<(String, String)> {
    let core = model.lock().unwrap();
    core.emits
        .iter()
        .map(|e| {
            assert_eq!(e["event"], format!("{NAME}.window"));
            let d = &e["data"];
            (
                d["surface"].as_str().unwrap().into(),
                d["phase"].as_str().unwrap().into(),
            )
        })
        .collect()
}

fn output(name: &str) -> OutputInfo {
    serde_json::from_value(json!({"name": name, "size": [800, 600], "position": [0, 0],
        "scale": 1.0, "camera": chora_scene::Transform::default(),
        "work_area": [0, 0, 800, 600], "default_work_area": [0, 0, 800, 600]}))
    .unwrap()
}

/// The core takes the parent and the owner of a surface, as when its
/// window manager goes.
fn detach(model: &Model, id: &str) {
    let mut core = model.lock().unwrap();
    core.scene.remove_surface(id);
    core.scene.add_surface(id, SurfaceInfo::default());
}

#[test]
fn a_created_window_is_adopted_and_opens() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    look.hear(&created("s1", None));
    assert!(look.book.has("s1"));
    assert_eq!(
        look.book.origin("s1"),
        Some(&Origin::New {
            token: Some("t1".into())
        })
    );
    assert_eq!(owner(&core.model(), "s1").as_deref(), Some(NAME));
    assert_eq!(phases(&core.model()), [("s1".into(), "opened".into())]);
    look.hear(&created("s1", None));
    assert_eq!(phases(&core.model()).len(), 1);
}

#[test]
fn a_refused_attach_leaves_the_window_to_another_manager() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    core.refuse_next("scene.apply");
    look.hear(&created("s1", None));
    assert!(!look.book.has("s1"), "another manager was first");
    assert!(phases(&core.model()).is_empty(), "nothing opened");
    assert!(
        matches!(&look.heard[..], [Change::Adopt { .. }]),
        "{:?}",
        look.heard
    );
}

#[test]
fn a_window_reserved_for_another_manager_is_not_claimed() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    look.hear(&created("s1", Some("xp")));
    assert!(look.heard.is_empty(), "{:?}", look.heard);
    assert!(!look.book.has("s1"));
}

#[test]
fn a_detached_window_is_forgotten_and_adopted_anew() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    look.hear(&created("s1", None));
    look.heard.clear();
    detach(&core.model(), "s1");
    look.hear(&Event::SurfaceDetached {
        surface: "s1".into(),
    });
    assert!(
        matches!(
            &look.heard[..2],
            [
                Change::Forget {
                    why: Why::Detached,
                    ..
                },
                Change::Adopt {
                    origin: Origin::New { token: None },
                    ..
                },
            ]
        ),
        "{:?}",
        look.heard
    );
    assert!(look.book.has("s1"));
}

#[test]
fn a_destroyed_window_is_forgotten_and_closes() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    look.hear(&created("s1", None));
    look.hear(&Event::SurfaceDestroyed {
        surface: "s1".into(),
    });
    assert!(!look.book.has("s1"));
    assert_eq!(
        phases(&core.model()),
        [
            ("s1".into(), "opened".into()),
            ("s1".into(), "closed".into())
        ]
    );
    look.hear(&Event::SurfaceDestroyed {
        surface: "s9".into(),
    });
    assert_eq!(phases(&core.model()).len(), 2);
}

#[test]
fn the_last_named_output_removed_releases_every_window() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, true);
    let changes = look.book.set_output("DP-1");
    look.settle(changes);
    surface(&core.model(), "s1");
    look.hear(&created("s1", None));
    look.hear(&Event::OutputRemoved {
        name: "DP-1".into(),
    });
    assert!(!look.book.has("s1"));
    assert_eq!(owner(&core.model(), "s1"), None, "free for any manager");
    assert!(
        look.heard.contains(&Change::Forget {
            id: "s1".into(),
            why: Why::Released
        }),
        "{:?}",
        look.heard
    );
    surface(&core.model(), "s2");
    look.heard.clear();
    look.hear(&created("s2", None));
    assert!(look.heard.is_empty(), "{:?}", look.heard);
    look.hear(&created("s2", Some(NAME)));
    assert_eq!(look.heard, [Change::Apply(pass_on("s2", NAME))]);
    let changes = look.book.set_output("DP-2");
    look.settle(changes);
    look.hear(&Event::OutputChanged {
        output: output("DP-2"),
    });
    surface(&core.model(), "s3");
    look.hear(&created("s3", None));
    assert!(look.book.has("s3"));
}

#[test]
fn a_window_given_in_has_the_origin_given() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    core.model().lock().unwrap().scene.reserve("s1", Some(NAME));
    look.hear(&created("s1", Some(NAME)));
    assert_eq!(look.book.origin("s1"), Some(&Origin::Given));
}

#[test]
fn a_refused_give_out_keeps_the_window() {
    let core = FakeCore::start();
    let mut look = Look::new(&core, false);
    surface(&core.model(), "s1");
    look.hear(&created("s1", None));
    assert!(!look.give("s1", "xp"), "xp holds no claim yet");
    assert!(look.book.has("s1"), "a refused give keeps the window");
    let mut xp = core.connect("xp");
    xp.apply(vec![Command::Create {
        id: "xp".into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate {
            claim: Some(Some([0.0, 0.0, 800.0, 600.0])),
            ..PropsUpdate::default()
        },
    }])
    .unwrap();
    assert!(look.give("s1", "xp"));
    assert!(!look.book.has("s1"));
    assert!(
        look.heard.contains(&Change::Forget {
            id: "s1".into(),
            why: Why::Given("xp".into())
        }),
        "{:?}",
        look.heard
    );
    assert_eq!(
        phases(&core.model()).last(),
        Some(&("s1".into(), "given".into()))
    );
    assert!(!look.book.give("s1", "xp"), "not the look's any more");
}

#[test]
fn a_foreign_look_update_passes_through_on_groups_and_root() {
    let core = FakeCore::start();
    let mut look = Book::new(NAME, false)
        .with_groups("g-")
        .with_legacy(|k| k == "window");
    let _ = core;
    look.keep("s1");
    let asked = |node: &str| Event::NodeRequested {
        node: node.into(),
        client: "effects".into(),
        action: "update".into(),
        args: json!({"opacity": 0.5, "effects": [], "meta": {"kept.1.window": "x", "window": "y", "role": "a"}}),
    };
    for node in [NAME, "g-s1", "s1"] {
        let changes = look.apply(&asked(node));
        let [Change::Apply(commands)] = &changes[..] else {
            panic!("{node}: {changes:?}");
        };
        let [Command::Update { id, props }] = &commands[..] else {
            panic!("{node}: {commands:?}");
        };
        assert_eq!(id, node);
        assert_eq!(props.opacity, Some(0.5));
        let meta = props.meta.clone().unwrap();
        assert_eq!(
            meta.keys().collect::<Vec<_>>(),
            ["role"],
            "kept keys stay the look's"
        );
    }
    assert_eq!(look.apply(&asked("g-s9")), []);
    let focus = Event::NodeRequested {
        node: NAME.into(),
        client: "effects".into(),
        action: "focus".into(),
        args: Value::Null,
    };
    assert_eq!(look.apply(&focus), []);
}

#[test]
fn the_phase_event_names_the_look_window_node_and_rect() {
    let book = Book::new(NAME, false).with_groups("g-");
    let emit = book.phase_event("s1", Phase::Opened, Some([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(emit.event, "look.window");
    assert_eq!(
        emit.data,
        json!({"surface": "s1", "node": "g-s1", "phase": "opened", "rect": [1.0, 2.0, 3.0, 4.0]})
    );
}

mod kept {
    use chora_kit::wm::kept::{self, Meta};
    use serde_json::json;

    fn legacy(key: &str) -> bool {
        key == "window" || key == "current" || key.starts_with("tags:")
    }

    #[test]
    fn the_versioned_key_wins_and_today_keys_are_read_without_it() {
        let nodes = [
            json!({"id": "look", "kind": "group", "meta": {"current": "DP-1", "tags:DP-1": "[1]", "other": "x"}}),
            json!({"id": "s1", "kind": "surface", "meta": {"window": "old"}}),
            json!({"id": "s2", "kind": "surface", "meta": {"window": "old", "kept.1.window": "new"}}),
            json!({"id": "s3", "kind": "surface"}),
            json!({"id": "g-s1", "kind": "group", "meta": {"window": "not a window"}}),
        ];
        let read = kept::read(&nodes, "look", legacy);
        let meta = |pairs: &[(&str, &str)]| -> Meta {
            pairs
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect()
        };
        assert_eq!(
            read.root,
            meta(&[("current", "DP-1"), ("tags:DP-1", "[1]")])
        );
        assert_eq!(
            read.windows,
            [
                ("s1".into(), meta(&[("window", "old")])),
                ("s2".into(), meta(&[("window", "new")])),
                ("s3".into(), Meta::new()),
            ]
        );
    }

    #[test]
    fn a_write_uses_the_versioned_key_and_retires_today_keys() {
        let sub: Meta = [("window".to_owned(), "w".to_owned())].into();
        let written = kept::write(&sub, &["window".into()]);
        let want: Meta = [
            ("kept.1.window".to_owned(), "w".to_owned()),
            ("window".to_owned(), String::new()),
        ]
        .into();
        assert_eq!(written, want);
        assert!(kept::reserved("kept.1.window"));
        assert!(kept::reserved(kept::KEY));
        assert!(!kept::reserved("role"));
    }
}
