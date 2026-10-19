//! The notification server of the freedesktop specification on the
//! session bus, so that `notify-send` and programs show XP's balloons.
//! zbus serves the bus on its own thread; the calls reach the main loop
//! through a channel, and the main loop sends the signals.

use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};

use zbus::{
    blocking::Connection,
    fdo::{RequestNameFlags, RequestNameReply},
    zvariant::OwnedValue,
};

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";
/// XP showed a balloon for at least 10 and at most 30 seconds.
const SHORTEST: Duration = Duration::from_secs(10);
const LONGEST: Duration = Duration::from_secs(30);

/// A notification as the main loop gets it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    /// Its id, which `replaces_id` names.
    pub id: u32,
    /// The icon: a name of the icon theme or a path; empty for none.
    pub icon: String,
    /// The bold title.
    pub summary: String,
    /// The text below it.
    pub body: String,
    /// The keys of the actions, without their labels.
    pub actions: Vec<String>,
    /// How long it shows; `None` until a click.
    pub timeout: Option<Duration>,
}

/// A call of a client.
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    /// Show a notification, or update the one with the same id.
    Notify(Note),
    /// `CloseNotification`.
    Close(u32),
}

/// Why a notification closed, as the specification numbers it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// Its time ran out.
    Expired = 1,
    /// A click closed it.
    Dismissed = 2,
    /// `CloseNotification` closed it.
    Closed = 3,
}

/// A signal for the clients.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signal {
    /// `NotificationClosed`.
    Closed(u32, Reason),
    /// `ActionInvoked`.
    Action(u32, String),
}

/// How long a notification shows: XP's 10 seconds by default, a time of
/// the client between XP's 10 and 30 seconds, and no end for 0.
pub fn timeout(expire_timeout: i32) -> Option<Duration> {
    match u64::try_from(expire_timeout) {
        Err(_) => Some(SHORTEST),
        Ok(0) => None,
        Ok(ms) => Some(Duration::from_millis(ms).clamp(SHORTEST, LONGEST)),
    }
}

/// The icon of a notification: the `image-path` hint before `app_icon`,
/// as the specification orders them.
fn icon(app_icon: &str, hints: &HashMap<String, OwnedValue>) -> String {
    ["image-path", "image_path"]
        .iter()
        .find_map(|key| {
            let value = hints.get(*key)?;
            String::try_from(value.try_clone().ok()?).ok()
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| app_icon.into())
        .trim_start_matches("file://")
        .into()
}

/// The keys of an action list, which alternates keys and labels.
fn action_keys(actions: &[String]) -> Vec<String> {
    let (pairs, _) = actions.as_chunks::<2>();
    pairs.iter().map(|[key, _]| key.clone()).collect()
}

/// The object on the bus.
struct Server {
    requests: Sender<Request>,
    last: u32,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl Server {
    // The arguments of the specification, as zbus hands them over.
    #[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        // XP's balloons do not show the name of the program.
        let _ = app_name;
        let id = if replaces_id == 0 {
            self.last = self.last.wrapping_add(1).max(1);
            self.last
        } else {
            replaces_id
        };
        let _ = self.requests.send(Request::Notify(Note {
            id,
            icon: icon(&app_icon, &hints),
            summary,
            body,
            actions: action_keys(&actions),
            timeout: timeout(expire_timeout),
        }));
        id
    }

    fn close_notification(&self, id: u32) {
        let _ = self.requests.send(Request::Close(id));
    }

    #[allow(clippy::unused_self)] // zbus calls it on the object
    fn get_capabilities(&self) -> Vec<&str> {
        vec!["actions", "body", "icon-static"]
    }

    #[allow(clippy::unused_self)]
    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("chora-xp", "chora", env!("CARGO_PKG_VERSION"), "1.2")
    }
}

/// The server on the session bus.
pub struct Bus {
    conn: Connection,
    /// The calls of the clients.
    pub requests: Receiver<Request>,
}

impl Bus {
    /// Takes the name on the session bus, if no other server has it: it
    /// never queues for the name, nor takes it from its owner.
    pub fn start() -> Result<Self, String> {
        let (tx, requests) = channel();
        let conn = Connection::session().map_err(|e| e.to_string())?;
        let server = Server {
            requests: tx,
            last: 0,
        };
        conn.object_server()
            .at(PATH, server)
            .map_err(|e| e.to_string())?;
        match conn.request_name_with_flags(NAME, RequestNameFlags::DoNotQueue.into()) {
            Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner) => {
                Ok(Self { conn, requests })
            }
            Ok(_) => Err(format!("{NAME} has another owner")),
            Err(err) => Err(err.to_string()),
        }
    }

    /// The server, or `None` with a line in the log when it cannot run.
    pub fn start_or_log() -> Option<Self> {
        Self::start()
            .map_err(|err| eprintln!("chora-xp: no notifications: {err}"))
            .ok()
    }

    /// Sends a signal to the clients.
    pub fn emit(&self, signal: &Signal) {
        let sent = match signal {
            Signal::Closed(id, reason) => self.conn.emit_signal(
                None::<()>,
                PATH,
                NAME,
                "NotificationClosed",
                &(*id, *reason as u32),
            ),
            Signal::Action(id, key) => {
                self.conn
                    .emit_signal(None::<()>, PATH, NAME, "ActionInvoked", &(*id, key))
            }
        };
        if let Err(err) = sent {
            eprintln!("chora-xp: notifications: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_time_follows_xp() {
        assert_eq!(timeout(-1), Some(SHORTEST));
        assert_eq!(timeout(0), None);
        assert_eq!(timeout(3000), Some(SHORTEST));
        assert_eq!(timeout(20_000), Some(Duration::from_secs(20)));
        assert_eq!(timeout(60_000), Some(LONGEST));
    }

    #[test]
    fn actions_come_in_pairs() {
        let list = ["default", "Open", "later", "Later", "odd"].map(String::from);
        assert_eq!(action_keys(&list), ["default", "later"]);
    }

    #[test]
    fn the_image_path_comes_before_the_app_icon() {
        let mut hints = HashMap::new();
        assert_eq!(icon("mail", &hints), "mail");
        hints.insert(
            "image-path".into(),
            OwnedValue::try_from(zbus::zvariant::Value::from("file:///tmp/a.png")).unwrap(),
        );
        assert_eq!(icon("mail", &hints), "/tmp/a.png");
    }

    #[test]
    fn ids_count_up_and_replace() {
        let (tx, rx) = channel();
        let mut server = Server {
            requests: tx,
            last: 0,
        };
        let mut notify = |replaces| {
            server.notify(
                "app".into(),
                replaces,
                String::new(),
                "s".into(),
                "b".into(),
                Vec::new(),
                HashMap::new(),
                -1,
            )
        };
        assert_eq!(notify(0), 1);
        assert_eq!(notify(0), 2);
        assert_eq!(notify(7), 7);
        let ids: Vec<u32> = rx
            .try_iter()
            .map(|r| match r {
                Request::Notify(note) => note.id,
                Request::Close(_) => 0,
            })
            .collect();
        assert_eq!(ids, [1, 2, 7]);
    }
}
