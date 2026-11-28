//! The put of the give key that waits for the face manager (co-79 r7):
//! the window of a pane, when the construct looks again and until when,
//! and the question whether the face manager answers calls yet. A thread
//! asks it on a connection of its own, as the HUD thread of voice draws
//! on its own, and the answer is read on a later look: the frame loop
//! never waits on a manager that starts (co-79 r8, rcrr7-5).

use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    time::{Duration, Instant},
};

use chora_scene::ipc::MethodCall;
use serde_json::{Value, json};

/// How long the thread waits for the answer of the face manager.
const ASK_LIMIT: Duration = Duration::from_secs(2);

/// A put of the window of a pane into the face at the front that waits
/// until the face manager can place it.
pub(crate) struct Put {
    /// The window of the pane.
    pub(crate) surface: String,
    /// When the construct looks again.
    pub(crate) at: Instant,
    /// When the wait is over.
    pub(crate) until: Instant,
    /// The answer to the question out: whether the face manager answered.
    asked: Option<Receiver<bool>>,
}

impl Put {
    /// A put of `surface` that looks at once and waits up to `limit`.
    pub(crate) fn new(surface: &str, now: Instant, limit: Duration) -> Self {
        Self {
            surface: surface.to_owned(),
            at: now,
            until: now + limit,
            asked: None,
        }
    }

    /// Whether the face manager `to` answered a call, without waiting:
    /// the answer of the question out, else a new question on a
    /// connection of its own to `socket`, whose answer a later look
    /// reads. With no socket to ask on, the put goes as it did before.
    pub(crate) fn answered(&mut self, socket: Option<&Path>, to: &str) -> bool {
        let Some(socket) = socket else {
            return true;
        };
        if let Some(asked) = &self.asked {
            match asked.try_recv() {
                Ok(true) => return true,
                Err(TryRecvError::Empty) => return false,
                Ok(false) | Err(TryRecvError::Disconnected) => {}
            }
        }
        self.asked = ask(socket.to_owned(), to);
        false
    }
}

/// Asks the face manager `to` for its state in a thread, on a connection
/// of its own to the core at `socket`, which ends with the answer: the
/// receiver gets whether it answered within `ASK_LIMIT`.
fn ask(socket: PathBuf, to: &str) -> Option<Receiver<bool>> {
    static ASKED: AtomicUsize = AtomicUsize::new(0);
    let name = format!(
        "{}-ask-{}",
        crate::NAME,
        ASKED.fetch_add(1, Ordering::Relaxed)
    );
    let call = MethodCall {
        client: to.to_owned(),
        method: "state.get".into(),
        params: json!({}),
    };
    let (tell, answer) = mpsc::channel();
    let asking = move || {
        #[allow(
            clippy::disallowed_methods,
            reason = "a question in a thread: it asks one thing on a connection of its own and \
                      ends, so that the frame loop never waits on the face manager"
        )]
        let conn = chora_kit::conn::Conn::connect_at(&name, &socket);
        let answered =
            conn.and_then(|mut conn| conn.call_within::<_, Value>("methods.call", call, ASK_LIMIT));
        let _ = tell.send(answered.is_ok());
    };
    let thread = std::thread::Builder::new().name("construct-ask".into());
    thread.spawn(asking).ok().map(|_| answer)
}
