//! The hang watchdog of the safety layer (D23, D64): a thread of its own
//! watches a heartbeat that the main loop bumps after each dispatch. The
//! loop wakes at least every [`BEAT_EVERY`], so an idle loop beats too.
//! When the heartbeat stops for [`STALL_AFTER`], the thread logs the stall
//! and the action that follows. After `CHORA_HANG_ABORT_S` seconds, 30 by
//! default, it aborts chora with a core dump (SIGABRT), so that the
//! session can start again; 0 means never. A hung shader blocks the main
//! loop, and with it the frame watchdog and SIGUSR1, but not this thread.
//!
//! The watchdog counts only the time that it watched: a look that comes
//! later than [`LATE`] after the one before means that the thread itself
//! did not run, after SIGSTOP or on a starved host, and that time is no
//! stall of the main loop. This has a ceiling in wall time, so that a
//! host on which every look comes late cannot keep a hung core alive:
//! [`CEILING`] times the abort time without a beat aborts too, once the
//! watchdog watched for longer than an idle loop sleeps.
//!
//! The stall and the abort lines go to fd 2 with a raw `write(2)`, not
//! through tracing or `std::io::stderr`: a main loop that hangs inside a
//! log write holds their lock, and the watchdog must not wait for it. The
//! SIGABRT handler then prints the backtrace of the watchdog thread, not
//! that of the main loop; the core dump has all threads.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use calloop::EventLoop;

use crate::words::say;

/// The longest sleep of an idle main loop between two beats.
pub const BEAT_EVERY: Duration = Duration::from_secs(1);
/// How long the heartbeat may stop before the watchdog logs a stall (D64).
const STALL_AFTER: Duration = Duration::from_secs(3);
/// How long the main loop may hang before the watchdog aborts chora.
const ABORT_AFTER: Duration = Duration::from_secs(30);
/// How often the watchdog looks at the heartbeat.
const TICK: Duration = Duration::from_millis(250);
/// A look this long after the one before means that the watchdog thread
/// did not run in between.
const LATE: Duration = Duration::from_secs(1);

/// How many abort times of wall time without a beat end a hang of which
/// the watchdog watched only a part.
const CEILING: u32 = 4;
/// The watched time without a beat that the ceiling needs: more than the
/// sleep of an idle loop, so that a core that was stopped beats first.
const WATCHED: Duration = Duration::from_millis(1250);

/// What the watchdog sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hang {
    /// The heartbeat stopped for this long; the abort time follows.
    Stall(Duration, Option<Duration>),
    /// The heartbeat came back after a stall of this length.
    Runs(Duration),
    /// The heartbeat stopped for the abort time.
    Abort(Duration),
}

/// The state of the watchdog between two looks.
struct Watch {
    beat: u64,
    since: Instant,
    /// The last beat in wall time, also over the time not watched.
    beat_at: Instant,
    looked: Instant,
    stalled: bool,
    abort: Option<Duration>,
}

impl Watch {
    fn new(abort: Option<Duration>, now: Instant) -> Self {
        Self {
            beat: 0,
            since: now,
            beat_at: now,
            looked: now,
            stalled: false,
            abort,
        }
    }

    /// Looks at the heartbeat at a time. The time that the watchdog
    /// itself did not run counts for no stall.
    fn check(&mut self, beat: u64, now: Instant) -> Option<Hang> {
        let step = now - self.looked;
        self.looked = now;
        if step >= LATE {
            self.since += step.saturating_sub(TICK);
        }
        let gap = now - self.since;
        if beat != self.beat {
            self.beat = beat;
            self.since = now;
            self.beat_at = now;
            return std::mem::take(&mut self.stalled).then_some(Hang::Runs(gap));
        }
        if self.abort.is_some_and(|abort| gap >= abort) {
            return Some(Hang::Abort(gap));
        }
        let wall = now - self.beat_at;
        if gap >= WATCHED && self.abort.is_some_and(|abort| wall >= abort * CEILING) {
            return Some(Hang::Abort(wall));
        }
        if !self.stalled && gap >= STALL_AFTER {
            self.stalled = true;
            return Some(Hang::Stall(gap, self.abort));
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
    mut report: impl FnMut(Hang) + Send + 'static,
) -> std::io::Result<thread::JoinHandle<()>> {
    thread::Builder::new()
        .name("chora-watchdog".into())
        .spawn(move || {
            crate::words::roomy_signal_stack();
            let mut watch = Watch::new(abort, Instant::now());
            while Arc::strong_count(&beats) > 1 {
                thread::sleep(TICK);
                if let Some(hang) = watch.check(beats.load(Ordering::Relaxed), Instant::now()) {
                    report(hang);
                }
            }
        })
}

/// Logs what the watchdog sees, and aborts on [`Hang::Abort`]. Only the
/// line of a loop that runs again goes through tracing.
fn log(hang: Hang) {
    match hang {
        Hang::Abort(gap) => {
            say(format_args!(
                "the main loop hangs; the watchdog aborts chora with a core dump \
                 (stalled {} s); the backtrace below is the watchdog thread's",
                gap.as_secs()
            ));
            std::process::abort();
        }
        Hang::Stall(gap, Some(abort)) => say(format_args!(
            "the main loop stalls; the watchdog aborts chora with a core dump after {} s \
             (stalled {} s)",
            abort.as_secs(),
            gap.as_secs()
        )),
        Hang::Stall(gap, None) => say(format_args!(
            "the main loop stalls; the watchdog only logs, CHORA_HANG_ABORT_S=0 (stalled {} s)",
            gap.as_secs()
        )),
        Hang::Runs(gap) => {
            tracing::warn!(stalled_s = gap.as_secs(), "the main loop runs again");
        }
    }
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

    /// Looks at a beat every [`TICK`] after a time, until another time.
    fn looks(watch: &mut Watch, beat: u64, after: Instant, until: Instant) -> Vec<Hang> {
        let ticks = (until - after).as_millis() / TICK.as_millis();
        (1..=u32::try_from(ticks).unwrap())
            .filter_map(|tick| watch.check(beat, after + tick * TICK))
            .collect()
    }

    #[test]
    fn a_stall_logs_once_then_aborts_or_runs_again() {
        let start = Instant::now();
        let at = |s: u32| start + s * S;
        let mut watch = Watch::new(Some(10 * S), start);
        assert_eq!(
            looks(&mut watch, 0, start, at(4)),
            [Hang::Stall(3 * S, Some(10 * S))]
        );
        assert_eq!(
            looks(&mut watch, 1, at(4), at(5)),
            [Hang::Runs(4 * S + TICK)]
        );
        assert_eq!(looks(&mut watch, 2, at(5), at(6)), []);
        assert_eq!(
            looks(&mut watch, 2, at(6), at(15) + TICK),
            [Hang::Stall(3 * S, Some(10 * S)), Hang::Abort(10 * S)]
        );
        let mut never = Watch::new(None, start);
        assert_eq!(
            looks(&mut never, 0, start, at(3600)),
            [Hang::Stall(3 * S, None)]
        );
    }

    /// A stopped or starved process stops the watchdog thread too: the
    /// time that the watchdog did not watch is no stall of the main loop.
    #[test]
    fn a_watchdog_that_did_not_run_counts_no_stall() {
        let start = Instant::now();
        let mut watch = Watch::new(Some(10 * S), start);
        assert_eq!(watch.check(1, start + TICK), None);
        assert_eq!(watch.check(1, start + 40 * S), None, "after SIGCONT");
        assert_eq!(watch.check(2, start + 40 * S + TICK), None);
        let later = start + 40 * S + TICK;
        let seen: Vec<Hang> = (1..=40)
            .filter_map(|tick| watch.check(2, later + tick * TICK))
            .collect();
        assert_eq!(
            seen,
            [Hang::Stall(3 * S, Some(10 * S)), Hang::Abort(10 * S)],
            "the stall counts from the last beat"
        );
    }

    /// On a starved host every look of the watchdog can come late, and
    /// then almost no time counts as a stall. A main loop that did not
    /// beat for four times the abort time in wall time hangs all the
    /// same: the watchdog aborts.
    #[test]
    fn a_hang_aborts_after_a_ceiling_of_wall_time_when_every_look_is_late() {
        let start = Instant::now();
        let mut watch = Watch::new(Some(10 * S), start);
        let seen: Vec<Hang> = (1..=6)
            .filter_map(|look| watch.check(0, start + look * 10 * S))
            .collect();
        assert!(
            matches!(seen.last(), Some(Hang::Abort(wall)) if *wall >= 40 * S),
            "{seen:?}"
        );
    }

    /// The stall line does not wait for the stdout lock of tracing, which
    /// a main loop that hangs inside a log write holds.
    #[test]
    fn the_stall_line_skips_the_log_lock() {
        let held = std::io::stdout().lock();
        let (done, finished) = mpsc::channel();
        thread::spawn(move || {
            let subscriber = tracing_subscriber::fmt().finish();
            tracing::subscriber::with_default(subscriber, || log(Hang::Stall(3 * S, None)));
            done.send(()).unwrap();
        });
        let result = finished.recv_timeout(2 * S);
        drop(held);
        assert!(result.is_ok(), "the watchdog waits for the stdout lock");
    }

    #[test]
    fn chora_hang_abort_s_sets_the_abort_time() {
        assert_eq!(abort_after(None), Some(30 * S));
        assert_eq!(abort_after(Some("x")), Some(30 * S));
        assert_eq!(abort_after(Some("0")), None);
        assert_eq!(abort_after(Some("7")), Some(7 * S));
    }

    /// A main loop that blocks for 5 s triggers the watchdog thread; an
    /// idle loop does not.
    #[test]
    fn a_main_loop_that_blocks_for_5_s_triggers_the_watchdog() {
        let mut event_loop: EventLoop<()> = EventLoop::try_new().unwrap();
        let beats = Arc::new(AtomicU64::new(0));
        let (seen, hangs) = mpsc::channel();
        let watchdog = spawn(beats.clone(), None, move |hang| {
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
            [Hang::Stall(stall, None), Hang::Runs(gap)] => {
                assert!(*stall >= STALL_AFTER && *stall < 5 * S, "{hangs:?}");
                assert!(*gap >= 5 * S, "{hangs:?}");
            }
            _ => panic!("the watchdog saw {hangs:?}"),
        }
    }
}
