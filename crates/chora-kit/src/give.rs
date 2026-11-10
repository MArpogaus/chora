//! Windows that move between window managers (D55): the claim under a
//! point, and the method `window.give` that every window manager exports.

use chora_scene::{
    Command, Node, claimant_at,
    ipc::{ClientError, ExportMethod, Filter, MethodCall, Query, RpcError},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{conn::Conn, runtime};

/// The name of the method that hands a window to another window manager.
pub const METHOD: &str = "window.give";

pub use chora_scene::MOVE;

/// The parameters of `window.give`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Give {
    /// The surface id of the window.
    pub surface: String,
    /// The client name of the window manager that gets it.
    pub to: String,
}

/// The export of `window.give`, with its description and schema. It is
/// privileged: a give hides a window from view and hands it to another
/// client, which may then focus, close or configure it.
#[must_use]
pub fn method() -> ExportMethod {
    runtime::privileged(runtime::method(
        METHOD,
        "Hands a window to another window manager (D55), and answers with its surface id.",
        json!({
            "surface": {"type": "string", "description": "The surface id of the window."},
            "to": {"type": "string", "description": "The client name of the window manager."},
        }),
        &["surface", "to"],
    ))
}

/// The windows of a window manager, for the key action `give` (D55).
pub trait Windows {
    /// The connection of the window manager.
    fn conn(&mut self) -> &mut Conn;

    /// Whether the window manager has this window.
    fn has(&self, surface: &str) -> bool;

    /// Hands an own window to `to` and forgets it: a `Conn::give`, the
    /// bookkeeping and the relayout of the window manager. The answer of
    /// `window.give`.
    ///
    /// # Errors
    ///
    /// The core refuses, or the relayout fails.
    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError>;
}

/// What the key action does with the focused window.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    /// The window is an own one: the window manager hands it over.
    Hand(String),
    /// Another window manager owns it: it hands it over with its own
    /// `window.give`.
    Forward {
        /// The client name of the owner.
        owner: String,
        /// The surface id of the window.
        surface: String,
    },
}

/// Decides what a key `give: to` does with the focused window of the
/// window manager `me`: it refuses without a focused window or owner, a
/// window of `me` that `windows` does not know, and a give to `me` of an
/// own window.
///
/// # Errors
///
/// One of the refusals above.
pub fn plan(
    focused: Option<&Node>,
    me: &str,
    to: &str,
    windows: &impl Windows,
) -> Result<Step, RpcError> {
    let refuse = |m: String| Err(RpcError::new(RpcError::REJECTED, m));
    let Some(node) = focused else {
        return refuse("no window has the focus".into());
    };
    let Some(owner) = node.owner.as_deref() else {
        return refuse(format!("{} has no window manager", node.id));
    };
    if owner != me {
        return Ok(Step::Forward {
            owner: owner.into(),
            surface: node.id.clone(),
        });
    }
    if to == me {
        return refuse(format!("{} is with {me} already", node.id));
    }
    if !windows.has(&node.id) {
        return refuse(format!("{me} has no window {}", node.id));
    }
    Ok(Step::Hand(node.id.clone()))
}

/// The keys that give the focused window to the window manager of this
/// name (D55): the same on every window manager.
pub const KEYS: [(&str, &str); 4] = [
    ("Super+Alt+1", "tiling"),
    ("Super+Alt+2", "xp"),
    ("Super+Alt+3", "mc"),
    ("Super+Alt+4", "construct"),
];

/// The time that a key waits for the owner of another window.
pub const FORWARD_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

/// As [`focused`], with the time to wait for the owner of another window.
///
/// # Errors
///
/// As [`focused`], and the time ends.
pub fn focused_within(
    _windows: &mut impl Windows,
    _to: &str,
    _wait: std::time::Duration,
) -> Result<Value, RpcError> {
    Err(RpcError::new(RpcError::REJECTED, "not written yet"))
}

/// The key action `give: to`: gives the focused window to the window
/// manager `to`. A window of the plugin goes through [`Windows::hand`],
/// the window of another window manager through its `window.give`, since
/// a key has one holder, whoever owns the window. Answers the surface id.
///
/// # Errors
///
/// The window cannot go, the owner refuses, or the socket fails.
pub fn focused(windows: &mut impl Windows, to: &str) -> Result<Value, RpcError> {
    let node = windows.conn().focused_surface()?;
    let me = windows.conn().name().to_owned();
    match plan(node.as_ref(), &me, to, windows)? {
        Step::Hand(surface) => windows.hand(&surface, to),
        Step::Forward { owner, surface } => windows
            .conn()
            .call(
                "methods.call",
                MethodCall {
                    client: owner,
                    method: METHOD.into(),
                    params: json!({"surface": surface, "to": to}),
                },
            )
            .map_err(|e| match e {
                ClientError::Rpc(err) => err,
                other => rejected(other),
            }),
    }
}

impl Conn {
    /// The focused window, `None` without one.
    ///
    /// # Errors
    ///
    /// The query fails.
    pub fn focused_surface(&mut self) -> Result<Option<Node>, RpcError> {
        let query = Query {
            selector: "kind:surface".into(),
            filter: Some(Filter {
                focused: true,
                ..Filter::default()
            }),
        };
        let nodes: Vec<Value> = self.call("scene.query", query).map_err(rejected)?;
        nodes
            .into_iter()
            .next()
            .map(|v| serde_json::from_value(v).map_err(rejected))
            .transpose()
    }

    /// The window manager under a point in scene coordinates: the owner of
    /// the smallest claim of another client that contains it, as the core
    /// picks one for a new toplevel under the pointer; `None` outside the
    /// claims of others.
    ///
    /// # Errors
    ///
    /// The query fails.
    pub fn claimant_at(&mut self, point: [f64; 2]) -> Result<Option<String>, RpcError> {
        let nodes = self.query("*").map_err(rejected)?;
        let me = self.name();
        Ok(claimant_at(&nodes, point, |o| o != me).map(String::from))
    }

    /// Hands a surface of the plugin to the window manager `to` with a
    /// `remove` with `to`, and returns the answer of `window.give`: the
    /// surface id. The core refuses a `to` that is not connected or holds
    /// no claim, and the surface then stays with the plugin.
    ///
    /// # Errors
    ///
    /// `to` is the plugin itself, the core refuses, or the socket fails.
    pub fn give(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        if to == self.name() {
            return Err(RpcError::new(
                RpcError::INVALID_PARAMS,
                format!("{surface} is with {to} already"),
            ));
        }
        let remove = Command::Remove {
            id: surface.into(),
            to: Some(to.into()),
        };
        match self.try_apply(vec![remove]).map_err(rejected)? {
            Some(err) => Err(err),
            None => Ok(Value::String(surface.into())),
        }
    }
}

fn rejected(err: impl std::fmt::Display) -> RpcError {
    RpcError::new(RpcError::REJECTED, err.to_string())
}

#[cfg(test)]
mod tests {
    use super::focused_within;
    use super::{Step, Windows, plan};
    use crate::conn::{
        Conn,
        fake_core::{self, Answer},
    };
    use std::time::Duration;

    use chora_scene::{
        Node, NodeKind, Props,
        ipc::{ExportMethod, RpcError},
    };
    use serde_json::json;

    struct Own(Vec<&'static str>);

    impl Windows for Own {
        fn conn(&mut self) -> &mut Conn {
            unreachable!("plan never talks to the core")
        }

        fn has(&self, surface: &str) -> bool {
            self.0.contains(&surface)
        }

        fn hand(
            &mut self,
            _: &str,
            _: &str,
        ) -> Result<serde_json::Value, chora_scene::ipc::RpcError> {
            unreachable!("plan never hands a window")
        }
    }

    fn surface(id: &str, owner: Option<&str>) -> Node {
        Node {
            id: id.into(),
            kind: NodeKind::Surface,
            owner: owner.map(String::from),
            parent: None,
            children: Vec::new(),
            props: Props::default(),
            surface: None,
            to: None,
        }
    }

    #[test]
    fn an_own_window_is_handed_over() {
        let node = surface("s1", Some("xp"));
        let step = plan(Some(&node), "xp", "tiling", &Own(vec!["s1"]));
        assert_eq!(step.unwrap(), Step::Hand("s1".into()));
    }

    #[test]
    fn a_window_of_another_manager_goes_through_its_owner() {
        let node = surface("s1", Some("tiling"));
        let step = plan(Some(&node), "xp", "mc", &Own(vec![]));
        let forward = Step::Forward {
            owner: "tiling".into(),
            surface: "s1".into(),
        };
        assert_eq!(step.unwrap(), forward);
    }

    #[test]
    fn a_window_that_the_manager_does_not_know_is_refused() {
        let node = surface("s1", Some("xp"));
        assert!(plan(Some(&node), "xp", "tiling", &Own(vec![])).is_err());
    }

    #[test]
    fn a_window_without_owner_or_focus_is_refused() {
        let node = surface("s1", None);
        assert!(plan(Some(&node), "xp", "tiling", &Own(vec![])).is_err());
        assert!(plan(None, "xp", "tiling", &Own(vec![])).is_err());
    }

    #[test]
    fn a_give_to_self_is_refused() {
        let node = surface("s1", Some("xp"));
        let err = plan(Some(&node), "xp", "xp", &Own(vec!["s1"])).unwrap_err();
        assert!(err.message.contains("already"), "{}", err.message);
    }

    #[test]
    fn window_give_is_privileged() {
        assert!(matches!(super::method(), ExportMethod::Described(d) if d.privileged));
    }

    /// `choractl give` always names the surface, so the schema needs it.
    #[test]
    fn window_give_needs_a_surface() {
        let ExportMethod::Described(d) = super::method() else {
            panic!("a described method");
        };
        let required = &d.input_schema.unwrap()["required"];
        assert_eq!(required, &serde_json::json!(["surface", "to"]));
        let params = serde_json::json!({"to": "xp"});
        assert!(serde_json::from_value::<super::Give>(params).is_err());
    }

    #[test]
    fn a_window_that_is_with_the_target_already_is_not_forwarded() {
        let node = surface("s1", Some("tiling"));
        let err = plan(Some(&node), "xp", "tiling", &Own(vec![])).unwrap_err();
        assert!(err.message.contains("already"), "{}", err.message);
    }

    #[test]
    fn an_own_window_is_never_forwarded() {
        let node = surface("s1", Some("tiling"));
        let step = plan(Some(&node), "tiling", "xp", &Own(vec!["s1"]));
        assert_eq!(step.unwrap(), Step::Hand("s1".into()));
    }

    /// A window manager on a fake core: the focused window is `focus`.
    struct Wm {
        conn: Conn,
        own: Vec<&'static str>,
        handed: Vec<(String, String)>,
    }

    impl Windows for Wm {
        fn conn(&mut self) -> &mut Conn {
            &mut self.conn
        }

        fn has(&self, surface: &str) -> bool {
            self.own.contains(&surface)
        }

        fn hand(&mut self, surface: &str, to: &str) -> Result<serde_json::Value, RpcError> {
            self.handed.push((surface.into(), to.into()));
            Ok(json!(surface))
        }
    }

    const WAIT: Duration = Duration::from_millis(150);

    fn wm_on(
        owner: &str,
        calls: impl Fn(&serde_json::Value) -> Answer + Send + 'static,
    ) -> (Wm, std::thread::JoinHandle<Vec<String>>) {
        let focus = serde_json::to_value(surface("s1", Some(owner))).unwrap();
        let (path, core) = fake_core::serve(move |method, params| match method {
            "scene.query" => Answer::Result(json!([focus])),
            "methods.call" => calls(params),
            _ => Answer::Result(json!(null)),
        });
        let conn = Conn::connect_at("xp", &path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let wm = Wm {
            conn,
            own: vec!["s1"],
            handed: Vec::new(),
        };
        (wm, core)
    }

    #[test]
    fn a_window_of_another_manager_is_given_by_its_owner() {
        let (mut wm, core) = wm_on("tiling", |params| {
            assert_eq!(params["client"], "tiling");
            assert_eq!(params["method"], "window.give");
            assert_eq!(params["params"], json!({"surface": "s1", "to": "mc"}));
            Answer::Result(json!("s1"))
        });
        assert_eq!(focused_within(&mut wm, "mc", WAIT).unwrap(), json!("s1"));
        assert_eq!(wm.handed, []);
        drop(wm);
        assert!(core.join().unwrap().contains(&"methods.call".to_owned()));
    }

    #[test]
    fn an_own_window_makes_no_call_to_another_manager() {
        let (mut wm, core) = wm_on("xp", |_| Answer::Error("never called"));
        focused_within(&mut wm, "tiling", WAIT).unwrap();
        assert_eq!(wm.handed, [("s1".to_owned(), "tiling".to_owned())]);
        drop(wm);
        assert!(!core.join().unwrap().contains(&"methods.call".to_owned()));
    }

    #[test]
    fn an_owner_that_is_gone_fails_the_key() {
        let (mut wm, _core) = wm_on("tiling", |_| Answer::Error("tiling is not connected"));
        let err = focused_within(&mut wm, "mc", WAIT).unwrap_err();
        assert!(err.message.contains("not connected"), "{}", err.message);
    }

    #[test]
    fn an_owner_that_refuses_fails_the_key() {
        let (mut wm, _core) = wm_on("tiling", |_| Answer::Error("tiling has no window s1"));
        let err = focused_within(&mut wm, "mc", WAIT).unwrap_err();
        assert!(err.message.contains("no window"), "{}", err.message);
    }

    #[test]
    fn an_owner_that_does_not_answer_ends_the_wait() {
        let (mut wm, _core) = wm_on("tiling", |_| Answer::Silence);
        let start = std::time::Instant::now();
        assert!(focused_within(&mut wm, "mc", WAIT).is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
