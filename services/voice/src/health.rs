//! The health of the servers for `voice.health` and the state record
//! (docs/voice.md section 6): each actor reports its successes and
//! failures here. A server that is down gets a backoff for the actors
//! that may skip a try, such as the TTS.

use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, Instant, SystemTime},
};

use serde_json::{Value, json};

/// The servers and connections of `voice.health`.
pub const SERVERS: [&str; 5] = ["asr", "tts", "llm", "mcp", "chora"];

/// The longest backoff of a server that is down.
const MAX_BACKOFF: Duration = Duration::from_secs(30);

#[derive(Default)]
struct Status {
    /// None before the first report.
    up: Option<bool>,
    error: String,
    /// When the last error came.
    at: Option<SystemTime>,
    /// Failures in a row.
    failures: u32,
    retry_at: Option<Instant>,
}

// ponytail: one global lock for a few reports per turn; per-server
// atomics if it ever shows in a profile.
static HEALTH: Mutex<BTreeMap<&'static str, Status>> = Mutex::new(BTreeMap::new());

fn with<T>(server: &'static str, f: impl FnOnce(&mut Status) -> T) -> T {
    let mut health = HEALTH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f(health.entry(server).or_default())
}

/// The server answered.
pub fn ok(server: &'static str) {
    with(server, |s| {
        if s.up == Some(false) {
            crate::log::info(&format!("{server}: up again"));
        }
        s.up = Some(true);
        s.failures = 0;
        s.retry_at = None;
    });
}

/// The server failed with `error`; answers whether it was up before, so
/// that a caller tells the user once per outage.
pub fn down(server: &'static str, error: &str) -> bool {
    with(server, |s| {
        let was_up = s.up != Some(false);
        s.up = Some(false);
        s.error = error.chars().take(300).collect();
        s.at = Some(SystemTime::now());
        let backoff = Duration::from_secs(1) * 2u32.saturating_pow(s.failures.min(5));
        s.failures += 1;
        s.retry_at = Some(Instant::now() + backoff.min(MAX_BACKOFF));
        was_up
    })
}

/// Whether a try is due: the server is not down, or its backoff (1 s,
/// doubling per failure, at most 30 s) has passed.
pub fn ready(server: &'static str) -> bool {
    with(server, |s| s.retry_at.is_none_or(|at| Instant::now() >= at))
}

/// The backoff before the next try of a server, for a loop that waits.
pub fn wait(server: &'static str) -> Duration {
    with(server, |s| {
        s.retry_at.map_or(Duration::ZERO, |at| {
            at.saturating_duration_since(Instant::now())
        })
    })
}

/// `{SERVER: {status, error, error_at}}` for each of `SERVERS`; the
/// status is `ok`, `down` or `unknown` before the first report, and
/// `error_at` the Unix time of the last error.
pub fn record() -> Value {
    SERVERS
        .iter()
        .map(|&server| {
            let value = with(server, |s| {
                let status = match s.up {
                    Some(true) => "ok",
                    Some(false) => "down",
                    None => "unknown",
                };
                let at =
                    s.at.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs());
                json!({"status": status, "error": s.error, "error_at": at})
            });
            (server.to_owned(), value)
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_is_down_with_a_backoff_until_the_server_answers() {
        let s = "test-health";
        assert!(ready(s), "unknown is ready");
        assert!(down(s, "refused"), "the first failure of an outage");
        assert!(!down(s, "refused again"));
        assert!(
            !ready(s) && wait(s) > Duration::from_millis(1500),
            "2 s after two"
        );
        ok(s);
        assert!(ready(s) && wait(s) == Duration::ZERO);
        assert!(down(s, "x"), "a new outage");
        let record = record();
        assert_eq!(record.as_object().unwrap().len(), SERVERS.len());
        assert!(record["asr"]["status"].is_string());
    }
}
