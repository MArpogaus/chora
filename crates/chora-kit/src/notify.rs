//! A client of the notification server on the session bus
//! (`org.freedesktop.Notifications`, served by `chora-notify`), for a
//! plugin that has something to tell the user. The connection opens at
//! the first call and again after a failure; without a server the calls
//! fail, are logged, and the plugin goes on.

use std::collections::HashMap;

use zbus::{blocking::Connection, zvariant::Value};

const NAME: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";

/// What the server shows: a title, a text and an icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message<'a> {
    /// The icon: a name of the icon theme or a path; empty for none.
    pub icon: &'a str,
    /// The bold title.
    pub summary: &'a str,
    /// The text below it.
    pub body: &'a str,
    /// How long it shows in milliseconds: -1 for the default of the
    /// server, 0 until a click.
    pub expire_ms: i32,
}

/// The connection to the session bus.
#[derive(Default)]
pub struct Notifier {
    conn: Option<Connection>,
}

impl Notifier {
    /// A client that connects at its first call.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A client on a given connection.
    #[must_use]
    pub fn on(conn: Connection) -> Self {
        Self { conn: Some(conn) }
    }

    /// Shows a message as the app `app`, or, with the id of an earlier one
    /// in `replaces`, in its place. Answers its id, or `None` when the
    /// server could not be reached.
    pub fn show(&mut self, app: &str, replaces: Option<u32>, message: &Message<'_>) -> Option<u32> {
        let sent = self.connection().and_then(|conn| {
            conn.call_method(
                Some(NAME),
                PATH,
                Some(NAME),
                "Notify",
                &(
                    app,
                    replaces.unwrap_or(0),
                    message.icon,
                    message.summary,
                    message.body,
                    Vec::<&str>::new(),
                    HashMap::<&str, Value<'_>>::new(),
                    message.expire_ms,
                ),
            )
            .and_then(|reply| reply.body().deserialize::<u32>())
            .map_err(|err| err.to_string())
        });
        match sent {
            Ok(id) => Some(id),
            Err(err) => {
                eprintln!("notify {:?}: {err}", message.summary);
                self.conn = None;
                None
            }
        }
    }

    /// Closes a message that is still showing or waiting.
    pub fn close(&mut self, id: u32) {
        let sent = self.connection().and_then(|conn| {
            conn.call_method(Some(NAME), PATH, Some(NAME), "CloseNotification", &(id,))
                .map(drop)
                .map_err(|err| err.to_string())
        });
        if let Err(err) = sent {
            eprintln!("notify close {id}: {err}");
            self.conn = None;
        }
    }

    fn connection(&mut self) -> Result<&Connection, String> {
        if self.conn.is_none() {
            self.conn = Some(Connection::session().map_err(|err| err.to_string())?);
        }
        self.conn.as_ref().ok_or_else(|| "no connection".into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    struct Server(Arc<Mutex<Vec<(u32, String)>>>);

    #[zbus::interface(name = "org.freedesktop.Notifications")]
    impl Server {
        #[allow(
            clippy::too_many_arguments,
            reason = "the signature of the specification"
        )]
        fn notify(
            &mut self,
            app: String,
            replaces: u32,
            icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: HashMap<String, zbus::zvariant::OwnedValue>,
            expire: i32,
        ) -> u32 {
            let _ = (app, icon, body, actions, hints, expire);
            self.0.lock().unwrap().push((replaces, summary));
            replaces.max(41) + 1
        }

        fn close_notification(&self, id: u32) {
            self.0.lock().unwrap().push((id, "closed".into()));
        }
    }

    struct KillOnDrop(std::process::Child);

    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// A private session bus, or `None` without `dbus-daemon`.
    fn bus() -> Option<(KillOnDrop, String)> {
        let daemon = std::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .ok()?;
        let mut daemon = KillOnDrop(daemon);
        let mut address = String::new();
        let stdout = daemon.0.stdout.take()?;
        std::io::BufRead::read_line(&mut std::io::BufReader::new(stdout), &mut address).ok()?;
        Some((daemon, address.trim().into()))
    }

    fn connect(address: &str) -> zbus::blocking::connection::Builder<'_> {
        zbus::blocking::connection::Builder::address(address).unwrap()
    }

    #[test]
    fn a_message_reaches_the_server_and_is_replaced_and_closed_by_its_id() {
        let Some((_daemon, address)) = bus() else {
            return;
        };
        let seen = Arc::new(Mutex::new(Vec::new()));
        let _server = connect(&address)
            .name(NAME)
            .unwrap()
            .serve_at(PATH, Server(seen.clone()))
            .unwrap()
            .build()
            .unwrap();
        let mut notifier = Notifier::on(connect(&address).build().unwrap());
        let message = Message {
            icon: "",
            summary: "Starting writer",
            body: "",
            expire_ms: 5000,
        };
        let id = notifier.show("chora-xp", None, &message);
        assert_eq!(id, Some(42));
        assert_eq!(notifier.show("chora-xp", id, &message), Some(43));
        notifier.close(43);
        assert_eq!(
            *seen.lock().unwrap(),
            [
                (0, "Starting writer".to_string()),
                (42, "Starting writer".to_string()),
                (43, "closed".to_string())
            ]
        );
    }

    #[test]
    fn a_bus_without_a_server_loses_the_message() {
        let Some((_daemon, address)) = bus() else {
            return;
        };
        let mut notifier = Notifier::on(connect(&address).build().unwrap());
        let message = Message {
            icon: "",
            summary: "lost",
            body: "",
            expire_ms: -1,
        };
        assert_eq!(notifier.show("chora-xp", None, &message), None);
    }
}
