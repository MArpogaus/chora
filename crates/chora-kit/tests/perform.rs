//! Doing a system action (KIT2b): the hooks that a look gives meaning to
//! are called, the rest is the same in every look.

use chora_kit::actions::perform::{Failure, Host, perform};
use chora_kit::actions::{Step, System, Turn};
use chora_kit::conn::Conn;
use chora_kit::give::Windows;
use chora_kit::testing::FakeCore;
use chora_kit::volume::Change;
use chora_scene::ipc::RpcError;
use serde_json::Value;

struct Look {
    conn: Conn,
    done: Vec<String>,
    failures: Vec<Failure>,
    own_give: bool,
}

impl Look {
    fn new(core: &FakeCore) -> Self {
        Self {
            conn: core.connect("probe"),
            done: Vec::new(),
            failures: Vec::new(),
            own_give: false,
        }
    }
}

impl Windows for Look {
    fn conn(&mut self) -> &mut Conn {
        &mut self.conn
    }

    fn has(&self, _surface: &str) -> bool {
        false
    }

    fn hand(&mut self, _surface: &str, _to: &str) -> Result<Value, RpcError> {
        Ok(Value::Null)
    }
}

impl Host for Look {
    type Error = String;

    fn volume(&mut self, change: Change) -> Result<(), String> {
        self.done.push(format!("volume {change:?}"));
        Ok(())
    }

    fn keys(&mut self) -> Result<(), String> {
        self.done.push("keys".into());
        Ok(())
    }

    fn chat(&mut self) -> Result<(), String> {
        self.done.push("chat".into());
        Ok(())
    }

    fn failed(&mut self, failure: Failure) -> Result<(), String> {
        self.failures.push(failure);
        Ok(())
    }

    /// A look with a power menu of its own: the shared script stays out.
    fn power(&mut self) -> Result<bool, String> {
        self.done.push("power".into());
        Ok(true)
    }

    fn terminal(&mut self) -> Result<(), String> {
        self.done.push("terminal".into());
        Ok(())
    }

    fn launcher(&mut self) -> Result<(), String> {
        self.done.push("launcher".into());
        Ok(())
    }

    fn close(&mut self) -> Result<(), String> {
        self.done.push("close".into());
        Ok(())
    }

    fn desk(&mut self, step: Step) -> Result<(), String> {
        self.done.push(format!("desk {step:?}"));
        Ok(())
    }

    fn cube(&mut self, turn: Turn) -> Result<(), String> {
        self.done.push(format!("cube {turn:?}"));
        Ok(())
    }

    fn give(&mut self, to: &str) -> Result<(), String> {
        if !self.own_give {
            return Err("not the default".into());
        }
        self.done.push(format!("give {to}"));
        Ok(())
    }
}

/// A look that has no meaning for most actions: the defaults do nothing.
struct Bare(Look);

impl Windows for Bare {
    fn conn(&mut self) -> &mut Conn {
        self.0.conn()
    }

    fn has(&self, _surface: &str) -> bool {
        false
    }

    fn hand(&mut self, _surface: &str, _to: &str) -> Result<Value, RpcError> {
        Ok(Value::Null)
    }
}

impl Host for Bare {
    type Error = String;

    fn volume(&mut self, _change: Change) -> Result<(), String> {
        Ok(())
    }

    fn keys(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn chat(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn failed(&mut self, failure: Failure) -> Result<(), String> {
        self.0.failures.push(failure);
        Ok(())
    }
}

#[test]
fn each_system_action_reaches_its_hook() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    look.own_give = true;
    let table = [
        (System::Terminal, "terminal"),
        (System::Launcher, "launcher"),
        (System::Close, "close"),
        (System::Keys, "keys"),
        (System::Chat, "chat"),
        (System::Power, "power"),
        (System::Volume(Change::Up), "volume Up"),
        (System::Volume(Change::Set(30)), "volume Set(30)"),
        (System::Desk(Step::Next), "desk Next"),
        (System::Cube(Turn::Left), "cube Left"),
        (System::Give("xp".into()), "give xp"),
    ];
    for (system, hook) in table {
        look.done.clear();
        perform(&mut look, &system).unwrap();
        assert_eq!(look.done, [hook], "{system:?}");
    }
    assert!(look.failures.is_empty(), "{:?}", look.failures);
}

#[test]
fn a_drag_does_nothing_in_the_hooks() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    perform(&mut look, &System::Drag).unwrap();
    assert!(look.done.is_empty() && look.failures.is_empty());
}

#[test]
fn an_error_of_a_hook_comes_back() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    let err = perform(&mut look, &System::Give("xp".into())).unwrap_err();
    assert_eq!(err, "not the default");
}

/// A command runs through the activation of the look; where nothing
/// starts, the look is told which. (The shared scripts take the same
/// path; a test must not lock the screen of whoever runs it.)
#[test]
fn a_command_that_does_not_start_is_a_failure() {
    let core = FakeCore::start();
    let mut look = Look::new(&core);
    perform(&mut look, &System::Run("chora-no-such-program".into())).unwrap();
    assert_eq!(look.done, Vec::<String>::new());
    assert_eq!(
        look.failures,
        [Failure::NotStarted("chora-no-such-program".into())]
    );
}

/// Without a window, the default give is a refusal that names the target.
#[test]
fn the_default_give_without_a_window_is_a_failure() {
    let core = FakeCore::start();
    let mut look = Bare(Look::new(&core));
    perform(&mut look, &System::Give("xp".into())).unwrap();
    assert!(
        matches!(&look.0.failures[..], [Failure::NotGiven { to, .. }] if to == "xp"),
        "{:?}",
        look.0.failures
    );
}

#[test]
fn the_default_hooks_do_nothing() {
    let core = FakeCore::start();
    let mut look = Bare(Look::new(&core));
    for system in [
        System::Terminal,
        System::Launcher,
        System::Close,
        System::Desk(Step::Previous),
        System::Cube(Turn::Right),
    ] {
        perform(&mut look, &system).unwrap();
    }
    assert_eq!(look.0.failures, Vec::<Failure>::new());
}

/// A look without a power menu of its own leaves the action to the shared
/// script: its hook says no.
#[test]
fn the_default_power_hook_leaves_the_menu_to_the_shared_script() {
    let core = FakeCore::start();
    let mut look = Bare(Look::new(&core));
    assert_eq!(look.power(), Ok(false));
}

#[test]
fn the_keys_of_every_look_are_the_shared_ones() {
    let shared = [
        System::Launcher,
        System::Keys,
        System::Chat,
        System::Lock,
        System::Power,
        System::Screenshot,
        System::Volume(Change::Mute),
    ];
    assert!(shared.iter().all(System::in_every_look));
    let own = [
        System::Terminal,
        System::Close,
        System::Run("x".into()),
        System::Give("xp".into()),
        System::Drag,
        System::Desk(Step::Next),
        System::Cube(Turn::Left),
    ];
    assert!(own.iter().all(|s| !s.in_every_look()));
}
