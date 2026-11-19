//! The script of the life of a window (slice WM1, WM1b-d): one run for
//! every look that settles the proposals of `kit::wm::book`. A look is a
//! [`Subject`]; the script creates windows in the fake core, sends the
//! events that the core would send, and reads what the model of the core
//! says afterwards.
#![allow(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "a script for tests: a failure is a failed test"
)]

use chora_scene::{Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceInfo};
use serde_json::{Value, json};

use super::{FakeCore, Model};

/// A look that the script drives.
pub trait Subject {
    /// The look hears an event as `Plugin::event` does, and settles what
    /// follows from it.
    fn hear(&mut self, event: Event);

    /// Whether the window is the look's.
    fn has(&self, surface: &str) -> bool;

    /// The look gives the window to the window manager `to`; true when
    /// it went.
    fn give(&mut self, surface: &str, to: &str) -> bool;
}

/// The script, over a fake core.
pub struct Script<'a> {
    core: &'a FakeCore,
    name: &'a str,
    releases: bool,
}

impl<'a> Script<'a> {
    /// The script of the look `name` on `core`. The look settles its
    /// windows through a connection of that name.
    #[must_use]
    pub const fn new(core: &'a FakeCore, name: &'a str) -> Self {
        Self {
            core,
            name,
            releases: false,
        }
    }

    /// The look lets every window go when the last output that it
    /// manages goes (`--output`); else its windows stay.
    #[must_use]
    pub const fn releasing(mut self) -> Self {
        self.releases = true;
        self
    }

    /// Runs every cell in order: create, refused attach, reserved for
    /// another, detach, destroy, output removed, give in, refused give
    /// out, give out. `output` is an output that the look manages.
    pub fn run(&self, look: &mut impl Subject, output: &str) {
        if self.releases {
            look.hear(Event::OutputChanged {
                output: self::output(output),
            });
        }

        self.open_and_refuse(look);
        self.detach_and_destroy(look);
        self.output_removed(look, output);
        self.give_in_and_out(look);
    }

    fn open_and_refuse(&self, look: &mut impl Subject) {
        let (model, name) = (self.core.model(), self.name);

        surface(&model, "s1");
        look.hear(created("s1", None));
        assert!(look.has("s1"), "a created window is the look's");
        assert_eq!(owner(&model, "s1").as_deref(), Some(name));
        assert_eq!(phases(&model, name), [("s1".into(), "opened".into())]);
        look.hear(created("s1", None));
        assert_eq!(phases(&model, name).len(), 1, "a window opens once");

        surface(&model, "s2");
        self.core.refuse_next("scene.apply");
        look.hear(created("s2", None));
        assert!(!look.has("s2"), "a refused attach leaves the window");
        assert_eq!(phases(&model, name).len(), 1, "nothing opened");

        surface(&model, "s3");
        look.hear(created("s3", Some("xp")));
        assert!(!look.has("s3"), "a window reserved for xp is not claimed");
    }

    fn detach_and_destroy(&self, look: &mut impl Subject) {
        let (model, name) = (self.core.model(), self.name);

        detach(&model, "s1");
        look.hear(Event::SurfaceDetached {
            surface: "s1".into(),
        });
        assert!(look.has("s1"), "a detached window is the look's again");
        assert_eq!(owner(&model, "s1").as_deref(), Some(name));

        look.hear(Event::SurfaceDestroyed {
            surface: "s1".into(),
        });
        assert!(!look.has("s1"));
        assert_eq!(phases(&model, name).last().unwrap().1, "closed");
        let closed = phases(&model, name).len();
        look.hear(Event::SurfaceDestroyed {
            surface: "s9".into(),
        });
        assert_eq!(
            phases(&model, name).len(),
            closed,
            "a stranger closes nothing"
        );
    }

    fn output_removed(&self, look: &mut impl Subject, output: &str) {
        let model = self.core.model();

        surface(&model, "s4");
        look.hear(created("s4", None));
        assert!(look.has("s4"));
        look.hear(Event::OutputRemoved {
            name: output.into(),
        });
        if self.releases {
            assert!(
                !look.has("s4"),
                "the last output went: the window is let go"
            );
            assert_eq!(owner(&model, "s4"), None, "free for any manager");
            look.hear(Event::OutputChanged {
                output: self::output(output),
            });
        } else {
            assert!(look.has("s4"), "the window stays");
            look.hear(Event::SurfaceDestroyed {
                surface: "s4".into(),
            });
        }
    }

    fn give_in_and_out(&self, look: &mut impl Subject) {
        let (model, name) = (self.core.model(), self.name);

        surface(&model, "s5");
        model.lock().unwrap().scene.reserve("s5", Some(name));
        look.hear(created("s5", Some(name)));
        assert!(look.has("s5"), "a window given in is the look's");

        assert!(!look.give("s5", "xp"), "xp holds no claim yet");
        assert!(look.has("s5"), "a refused give keeps the window");
        let mut xp = self.core.connect("xp");
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

        assert!(look.give("s5", "xp"));
        assert!(!look.has("s5"));
        let to = model.lock().unwrap().scene.node("s5").unwrap().to.clone();
        assert_eq!(to.as_deref(), Some("xp"), "reserved for the receiver");
        assert_eq!(
            phases(&model, name).last(),
            Some(&("s5".into(), "given".into()))
        );
    }
}

/// Puts a free surface of the app `probe` into the model, as the core
/// does when a client maps one.
pub fn surface(model: &Model, id: &str) {
    let info = SurfaceInfo {
        app_id: Some("probe".into()),
        ..SurfaceInfo::default()
    };
    model.lock().unwrap().scene.add_surface(id, info);
}

/// The event `surface.created` of a surface, reserved for `to` when the
/// core says so, with the activation token `t1`.
#[must_use]
pub fn created(id: &str, to: Option<&str>) -> Event {
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

/// An output called `name`, 800 by 600.
#[must_use]
pub fn output(name: &str) -> OutputInfo {
    serde_json::from_value(json!({"name": name, "size": [800, 600], "position": [0, 0],
        "scale": 1.0, "camera": chora_scene::Transform::default(),
        "work_area": [0, 0, 800, 600], "default_work_area": [0, 0, 800, 600]}))
    .unwrap()
}

/// The owner of a node of the model.
#[must_use]
pub fn owner(model: &Model, id: &str) -> Option<String> {
    model.lock().unwrap().scene.node(id)?.owner.clone()
}

/// The `<look>.window` events that the look emitted: surface and phase.
#[must_use]
pub fn phases(model: &Model, look: &str) -> Vec<(String, String)> {
    let core = model.lock().unwrap();
    (core.emits.iter())
        .filter(|e| e["event"] == format!("{look}.window"))
        .map(|e| {
            let d: &Value = &e["data"];
            (
                d["surface"].as_str().unwrap().into(),
                d["phase"].as_str().unwrap().into(),
            )
        })
        .collect()
}

/// The core takes the parent and the owner of a surface, as when its
/// window manager goes.
pub fn detach(model: &Model, id: &str) {
    let mut core = model.lock().unwrap();
    core.scene.remove_surface(id);
    core.scene.add_surface(id, SurfaceInfo::default());
}
