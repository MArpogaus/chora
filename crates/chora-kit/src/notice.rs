//! The notice: the one way for a look to show, replace and dismiss a
//! message to the person. `chora-notify` is the only server; a look calls
//! its methods on the live API and draws nothing of its own.

use std::{fmt, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::conn::Conn;

/// The client that serves the notices.
pub const CLIENT: &str = "notify";
/// The method that shows or replaces a notice.
pub const SHOW: &str = "notify.show";
/// The method that dismisses a notice.
pub const CLOSE: &str = "notify.close";

/// How much a notice asks for the attention of the person.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    /// Shows for a while and goes.
    #[default]
    Normal,
    /// Stays until the person dismisses it.
    Critical,
}

/// What the server shows: a title, a text and an icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notice<'a> {
    /// The plugin that shows it.
    pub app: &'a str,
    /// The icon: a name of the icon theme or a path; empty for none.
    pub icon: &'a str,
    /// The bold title.
    pub summary: &'a str,
    /// The text below it.
    pub body: &'a str,
    /// How long it shows in milliseconds: -1 for the default of the
    /// server, 0 until a click.
    pub expire_ms: i32,
    /// How much it asks for the attention of the person.
    pub urgency: Urgency,
}

/// The server does not answer: it is not running, or it fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gone(pub String);

impl fmt::Display for Gone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no notification server: {}", self.0)
    }
}

impl std::error::Error for Gone {}

/// How long a call waits for the server, so that a server that hangs
/// stalls the plugin for this long at most.
const WAIT: Duration = Duration::from_secs(2);

fn call(conn: &mut Conn, method: &str, params: &Value) -> Result<Value, Gone> {
    let call = json!({"client": CLIENT, "method": method, "params": params});
    conn.call_within("methods.call", call, WAIT)
        .map_err(|err| Gone(err.to_string()))
}

/// Shows a notice, or, with the id of an earlier one in `replaces`, in
/// its place. Answers its id.
///
/// # Errors
///
/// The server is gone.
pub fn show(conn: &mut Conn, replaces: Option<u32>, notice: &Notice<'_>) -> Result<u32, Gone> {
    let params = json!({
        "app": notice.app,
        "replaces": replaces,
        "icon": notice.icon,
        "summary": notice.summary,
        "body": notice.body,
        "expire_ms": notice.expire_ms,
        "urgency": notice.urgency,
    });
    let answer = call(conn, SHOW, &params)?;
    answer["id"]
        .as_u64()
        .and_then(|id| u32::try_from(id).ok())
        .ok_or_else(|| Gone(format!("an answer without an id: {answer}")))
}

/// Dismisses a notice that is still showing or waiting.
///
/// # Errors
///
/// The server is gone.
pub fn dismiss(conn: &mut Conn, id: u32) -> Result<(), Gone> {
    call(conn, CLOSE, &json!({"id": id})).map(drop)
}

/// Shows a notice as [`show`] does; when the server is gone, the person
/// gets no notice, so the failure is logged once and the plugin goes on.
pub fn show_or_log(conn: &mut Conn, replaces: Option<u32>, notice: &Notice<'_>) -> Option<u32> {
    match show(conn, replaces, notice) {
        Ok(id) => Some(id),
        Err(err) => {
            eprintln!("{}: notice {:?}: {err}", notice.app, notice.summary);
            None
        }
    }
}
