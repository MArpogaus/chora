//! The hang watchdog of the safety layer (D23, D64): a thread of its own
//! watches a heartbeat that the main loop bumps after each dispatch. The
//! loop wakes at least every [`BEAT_EVERY`], so an idle loop beats too.
//! When the heartbeat stops for [`STALL_AFTER`], the thread logs the stall
//! and the action that follows. After `CHORA_HANG_ABORT_S` seconds, 30 by
//! default, it aborts chora with a core dump (SIGABRT), so that the
//! session can start again; 0 means never. A hung shader blocks the main
//! loop, and with it the frame watchdog and SIGUSR1, but not this thread.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use calloop::EventLoop;

/// The longest sleep of an idle main loop between two beats.
pub const BEAT_EVERY: Duration = Duration::from_secs(1);
/// How long the heartbeat may stop before the watchdog logs a stall. Under
/// the 5 s of gate check 3 of 0.9, and well above [`BEAT_EVERY`].
const STALL_AFTER: Duration = Duration::from_secs(3);
/// How long the main loop may hang before the watchdog aborts chora.
const ABORT_AFTER: Duration = Duration::from_secs(30);
/// How often the watchdog looks at the heartbeat.
const TICK: Duration = Duration::from_millis(250);

/// What the watchdog sees.
#[derive(Debug, PartialEq, Eq)]
enum Hang {
    /// The heartbeat stopped for this long.
    Stall(Duration),
    /// The heartbeat came back after a stall of this length.
    Runs(Duration),
    /// The heartbeat stopped for the abort time.
    Abort(Duration),
}

/// The state of the watchdog between two looks.
struct Watch {
    beat: u64,
    since: Instant,
    stalled: bool,
    abort: Option<Duration>,
}

impl Watch {
    fn new(abort: Option<Duration>, now: Instant) -> Self {
        Self {
            beat: 0,
            since: now,
            stalled: false,
            abort,
        }
    }

    /// Looks at the heartbeat at a time.
    fn check(&mut self, beat: u64, now: Instant) -> Option<Hang> {
        let gap = now - self.since;
        if beat != self.beat {
            self.beat = beat;
            self.since = now;
            return std::mem::take(&mut self.stalled).then_some(Hang::Runs(gap));
        }
        if self.abort.is_some_and(|abort| gap >= abort) {
            return Some(Hang::Abort(gap));
        }
        if !self.stalled && gap >= STALL_AFTER {
            self.stalled = true;
            return Some(Hang::Stall(gap));
        }
        None
    }
}

/// The abort time of `CHORA_HANG_ABORT_S`: unset or not a number is the
/// default, 0 is never.
fn abort_after(var: Option<&str>) -> Option<Duration> {
    match var.and_then(|v| v.trim().parse::<u64>().ok()) {
        None => Some(ABORT_AFTER),
        Some(0) => None,
        Some(s) => Some(Duration::from_secs(s)),
    }
}

/// Starts the watchdog thread on a heartbeat. It ends when the heartbeat
/// has no other owner.
fn spawn(
    beats: Arc<AtomicU64>,
    abort: Option<Duration>,
    mut report: impl FnMut(Hang, Option<Duration>) + Send + 'static,
) -> std::io::Result<thread::JoinHandle<()>> {
    thread::Builder::new()
        .name("chora-watchdog".into())
        .spawn(move || {
            let mut watch = Watch::new(abort, Instant::now());
            while Arc::strong_count(&beats) > 1 {
                thread::sleep(TICK);
                if let Some(hang) = watch.check(beats.load(Ordering::Relaxed), Instant::now()) {
                    report(hang, abort);
                }
            }
        })
}

/// Logs what the watchdog sees, and aborts on [`Hang::Abort`].
fn log(hang: Hang, abort: Option<Duration>) {
    let (gap, message) = match (hang, abort) {
        (Hang::Abort(gap), _) => abort_hung(gap),
        (Hang::Stall(gap), Some(abort)) => (
            gap,
            format!(
                "the main loop stalls; the watchdog aborts chora with a core dump after {} s",
                abort.as_secs()
            ),
        ),
        (Hang::Stall(gap), None) => (
            gap,
            "the main loop stalls; the watchdog only logs, CHORA_HANG_ABORT_S=0".into(),
        ),
        (Hang::Runs(gap), _) => (gap, "the main loop runs again".into()),
    };
    tracing::warn!(stalled_s = gap.as_secs(), "{message}");
}

/// Aborts chora with a core dump after a hang.
fn abort_hung(gap: Duration) -> ! {
    tracing::error!(
        stalled_s = gap.as_secs(),
        "the main loop hangs; the watchdog aborts chora with a core dump"
    );
    std::process::abort();
}

/// Runs an event loop until it stops, bumps the heartbeat after each
/// dispatch and wakes at least every [`BEAT_EVERY`]. The watchdog thread
/// on the heartbeat logs the stalls, and aborts as
/// `CHORA_HANG_ABORT_S` says.
///
/// # Errors
///
/// A failed start of the thread or a failed dispatch.
pub fn run<D>(
    event_loop: &mut EventLoop<'_, D>,
    data: &mut D,
    mut after: impl FnMut(&mut D),
) -> Result<(), Box<dyn std::error::Error>> {
    let beats = Arc::new(AtomicU64::new(0));
    spawn(
        beats.clone(),
        abort_after(std::env::var("CHORA_HANG_ABORT_S").ok().as_deref()),
        log,
    )?;
    Ok(run_beating(event_loop, data, &beats, &mut after)?)
}

/// Runs an event loop and bumps a heartbeat after each dispatch.
fn run_beating<D>(
    event_loop: &mut EventLoop<'_, D>,
    data: &mut D,
    beats: &AtomicU64,
    after: &mut impl FnMut(&mut D),
) -> calloop::Result<()> {
    event_loop.run(BEAT_EVERY, data, |data| {
        beats.fetch_add(1, Ordering::Relaxed);
        after(data);
    })
}

#[cfg(test)]
mod tests {
    use calloop::timer::{TimeoutAction, Timer};
    use std::sync::mpsc;

    use super::*;

    const S: Duration = Duration::from_secs(1);

    #[test]
    fn a_stall_logs_once_then_aborts_or_runs_again() {
        let start = Instant::now();
        let mut watch = Watch::new(Some(10 * S), start);
        assert_eq!(watch.check(0, start + 2 * S), None);
        assert_eq!(watch.check(0, start + 3 * S), Some(Hang::Stall(3 * S)));
        assert_eq!(watch.check(0, start + 4 * S), None);
        assert_eq!(watch.check(1, start + 5 * S), Some(Hang::Runs(5 * S)));
        assert_eq!(watch.check(2, start + 6 * S), None);
        assert_eq!(watch.check(2, start + 9 * S), Some(Hang::Stall(3 * S)));
        assert_eq!(watch.check(2, start + 16 * S), Some(Hang::Abort(10 * S)));
        let mut never = Watch::new(None, start);
        assert_eq!(never.check(0, start + 3 * S), Some(Hang::Stall(3 * S)));
        assert_eq!(never.check(0, start + 3600 * S), None);
    }

    #[test]
    fn chora_hang_abort_s_sets_the_abort_time() {
        assert_eq!(abort_after(None), Some(30 * S));
        assert_eq!(abort_after(Some("x")), Some(30 * S));
        assert_eq!(abort_after(Some("0")), None);
        assert_eq!(abort_after(Some("7")), Some(7 * S));
    }

    /// Gate check 3 of 0.9: a main loop that blocks for 5 s triggers the
    /// watchdog thread; an idle loop does not.
    #[test]
    fn a_main_loop_that_blocks_for_5_s_triggers_the_watchdog() {
        let mut event_loop: EventLoop<()> = EventLoop::try_new().unwrap();
        let beats = Arc::new(AtomicU64::new(0));
        let (seen, hangs) = mpsc::channel();
        let watchdog = spawn(beats.clone(), None, move |hang, _| {
            seen.send(hang).unwrap();
        })
        .unwrap();
        let handle = event_loop.handle();
        // The debug hook: a source that blocks the loop after 2 idle
        // seconds.
        handle
            .insert_source(Timer::from_duration(2 * S), |_, (), ()| {
                thread::sleep(5 * S);
                TimeoutAction::Drop
            })
            .unwrap();
        let signal = event_loop.get_signal();
        handle
            .insert_source(Timer::from_duration(9 * S), move |_, (), ()| {
                signal.stop();
                TimeoutAction::Drop
            })
            .unwrap();
        let start = Instant::now();
        run_beating(&mut event_loop, &mut (), &beats, &mut |()| {}).unwrap();
        assert!(start.elapsed() >= 9 * S);
        drop(beats);
        watchdog.join().unwrap();
        let hangs: Vec<Hang> = hangs.try_iter().collect();
        match hangs.as_slice() {
            [Hang::Stall(stall), Hang::Runs(gap)] => {
                assert!(*stall >= STALL_AFTER && *stall < 5 * S, "{hangs:?}");
                assert!(*gap >= 5 * S, "{hangs:?}");
            }
            _ => panic!("the watchdog saw {hangs:?}"),
        }
    }
}
