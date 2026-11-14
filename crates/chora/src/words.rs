//! The core's last words: no end of the core is silent. A faulting
//! signal writes the fault address and a backtrace, a signal that ends
//! the core writes its name and its sender, and a panic writes its
//! thread, its message and a backtrace, all to stderr before the default
//! action. The lines go to fd 2 with a raw `write(2)`, without the lock
//! of tracing or of `std::io::stderr`.
//!
//! A fault writes its fixed line first, from a buffer on the stack, and
//! only then the backtrace, which needs malloc and the lock of the
//! unwinder: after a fault inside malloc that can hang, so an alarm asks
//! every [`GIVE_UP_AFTER`] milliseconds whether the handler got on, and
//! ends the core with the signal of the fault when it did not. A slow
//! backtrace, under load, is not stuck and goes on. The handlers run on the
//! alternate signal stack of their thread, so a stack overflow has its
//! words too, and a fault handler runs once: a second fault ends the
//! core at once. The main thread and the hang watchdog, which aborts,
//! get a signal stack with room for the backtrace; on the small stack
//! that Rust gives the other threads only the fixed line is sure.
//! `CHORA_TEST_PANIC` starts a thread that panics, for
//! `tests/last-words.sh`.

use std::{
    backtrace::Backtrace,
    fmt::Display,
    os::fd::RawFd,
    sync::atomic::{AtomicI32, AtomicI64, AtomicU32, Ordering},
};

/// The signals of a fault, with a backtrace.
const FAULTS: [libc::c_int; 6] = [
    libc::SIGSEGV,
    libc::SIGBUS,
    libc::SIGILL,
    libc::SIGABRT,
    libc::SIGFPE,
    libc::SIGSYS,
];
/// The signals whose default action ends the core without a fault.
const ENDS: [libc::c_int; 8] = [
    libc::SIGTERM,
    libc::SIGINT,
    libc::SIGHUP,
    libc::SIGQUIT,
    libc::SIGUSR2,
    libc::SIGALRM,
    libc::SIGXCPU,
    libc::SIGXFSZ,
];

/// The milliseconds between two alarms of a fault handler, each of
/// which asks whether the handler got on. A test sets a shorter time.
static GIVE_UP_AFTER: AtomicU32 = AtomicU32::new(2000);

/// The milliseconds after which a fault handler ends even when it gets
/// on: a handler that spins never ends by itself.
const GIVE_UP_AT_LAST: u64 = 60_000;

/// The size of the signal stack of the main thread: the backtrace and
/// its symbols need more than the few pages of the default.
const SIGNAL_STACK: usize = 512 * 1024;

type Handler = extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void);

fn name(sig: libc::c_int) -> &'static str {
    match sig {
        libc::SIGSEGV => "SIGSEGV",
        libc::SIGBUS => "SIGBUS",
        libc::SIGILL => "SIGILL",
        libc::SIGABRT => "SIGABRT",
        libc::SIGFPE => "SIGFPE",
        libc::SIGSYS => "SIGSYS",
        libc::SIGTERM => "SIGTERM",
        libc::SIGINT => "SIGINT",
        libc::SIGHUP => "SIGHUP",
        libc::SIGQUIT => "SIGQUIT",
        libc::SIGUSR2 => "SIGUSR2",
        libc::SIGALRM => "SIGALRM",
        libc::SIGXCPU => "SIGXCPU",
        libc::SIGXFSZ => "SIGXFSZ",
        _ => "a signal",
    }
}

/// Writes `chora: <line>` to fd 2 with one raw `write(2)`, from a buffer
/// on the stack: no lock of tracing, of `std::io::stderr` or of malloc,
/// so a signal handler and the hang watchdog may call it. A line longer
/// than the buffer ends short.
pub fn say(line: std::fmt::Arguments<'_>) {
    use std::io::Write as _;
    let mut buf = std::io::Cursor::new([0_u8; 256]);
    let _ = writeln!(buf, "chora: {line}");
    let len = usize::try_from(buf.position()).unwrap_or(0);
    write_all(2, &buf.into_inner()[..len]);
}

#[allow(unsafe_code, reason = "one libc write to a fixed fd")]
fn write_all(fd: RawFd, text: &[u8]) {
    // SAFETY: the pointer and the length come from a live slice.
    unsafe {
        libc::write(fd, text.as_ptr().cast(), text.len());
    }
}

/// Hands a signal to its default action after the handler returns.
#[allow(unsafe_code, reason = "libc calls that a signal handler may make")]
fn default_action(sig: libc::c_int) {
    // SAFETY: both calls are async signal safe.
    unsafe {
        libc::signal(sig, libc::SIG_DFL);
        libc::raise(sig);
    }
}

/// What the alarm of a fault handler knows of it: the signal of the
/// fault, the thread of the handler with the clock of its CPU time, the
/// CPU time at the last alarm, and the alarms so far.
struct Watch {
    signal: AtomicI32,
    thread: AtomicI32,
    clock: AtomicI32,
    cpu_ns: AtomicI64,
    alarms: AtomicU32,
}

static WATCH: Watch = Watch {
    signal: AtomicI32::new(0),
    thread: AtomicI32::new(0),
    clock: AtomicI32::new(0),
    cpu_ns: AtomicI64::new(0),
    alarms: AtomicU32::new(0),
};

/// The CPU time of a thread by its clock, in nanoseconds.
#[allow(unsafe_code, reason = "a libc call that a signal handler may make")]
fn cpu_ns(clock: libc::clockid_t) -> i64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: the call is async signal safe and writes a live timespec.
    unsafe {
        libc::clock_gettime(clock, &raw mut time);
    }
    time.tv_sec * 1_000_000_000 + time.tv_nsec
}

/// The state of a thread in a line of `/proc/<pid>/task/<tid>/stat`:
/// the letter after the name in brackets, which may hold any byte.
fn state_in(stat: &[u8]) -> Option<u8> {
    let name_ends = stat.iter().rposition(|byte| *byte == b')')?;
    stat.get(name_ends + 2).copied()
}

/// Whether the scheduler has a thread of this process as running or as
/// ready to run, by `/proc`, read without malloc.
#[allow(unsafe_code, reason = "libc calls that a signal handler may make")]
fn runnable(thread: libc::pid_t) -> bool {
    use std::io::Write as _;
    let mut path = std::io::Cursor::new([0_u8; 64]);
    let _ = write!(path, "/proc/self/task/{thread}/stat\0");
    let mut stat = [0_u8; 128];
    // SAFETY: the path ends with a NUL and the buffer is live.
    let read = unsafe {
        let fd = libc::open(path.get_ref().as_ptr().cast(), libc::O_RDONLY);
        if fd < 0 {
            return false;
        }
        let read = libc::read(fd, stat.as_mut_ptr().cast(), stat.len());
        libc::close(fd);
        read
    };
    let read = usize::try_from(read).unwrap_or(0);
    state_in(&stat[..read]) == Some(b'R')
}

/// Whether a fault handler that the alarm found at this point may go
/// on: it used CPU time since the last alarm or waits only for a CPU,
/// and it has not run for longer than [`GIVE_UP_AT_LAST`]. A handler
/// that waits for a lock does neither.
fn goes_on(used_cpu: bool, runnable: bool, ran_ms: u64) -> bool {
    (used_cpu || runnable) && ran_ms < GIVE_UP_AT_LAST
}

/// The alarm of a fault handler. A handler that is slow goes on, so
/// that load does not cut the backtrace; one that is stuck ends here,
/// with the signal of its fault and not with SIGALRM.
extern "C" fn stuck(_sig: libc::c_int, _info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    let now = cpu_ns(WATCH.clock.load(Ordering::Relaxed));
    let used_cpu = now > WATCH.cpu_ns.swap(now, Ordering::Relaxed);
    let alarms = WATCH.alarms.fetch_add(1, Ordering::Relaxed) + 1;
    let ran_ms = u64::from(alarms) * u64::from(GIVE_UP_AFTER.load(Ordering::Relaxed));
    if goes_on(
        used_cpu,
        runnable(WATCH.thread.load(Ordering::Relaxed)),
        ran_ms,
    ) {
        return;
    }
    let sig = WATCH.signal.load(Ordering::Relaxed);
    say(format_args!(
        "the handler of {} is stuck after {ran_ms} ms: no backtrace",
        name(sig)
    ));
    end_with(sig);
}

/// Ends the core with a signal from inside a handler, where the signal
/// may be blocked.
#[allow(unsafe_code, reason = "libc calls that a signal handler may make")]
fn end_with(sig: libc::c_int) {
    // SAFETY: the set is a live value and the calls are async signal safe.
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&raw mut set);
        libc::sigaddset(&raw mut set, sig);
        libc::pthread_sigmask(libc::SIG_UNBLOCK, &raw const set, std::ptr::null_mut());
    }
    default_action(sig);
}

/// Watches the fault handler of the calling thread: an alarm every
/// [`GIVE_UP_AFTER`] milliseconds ends the core with the signal of the
/// fault when the handler made no progress since the alarm before.
#[allow(unsafe_code, reason = "libc calls that a signal handler may make")]
fn give_up_when_stuck(sig: libc::c_int) {
    let ms = GIVE_UP_AFTER.load(Ordering::Relaxed);
    let every = libc::timeval {
        tv_sec: libc::time_t::from(ms / 1000),
        tv_usec: libc::suseconds_t::from(ms % 1000 * 1000),
    };
    let timer = libc::itimerval {
        it_interval: every,
        it_value: every,
    };
    let mut clock: libc::clockid_t = libc::CLOCK_THREAD_CPUTIME_ID;
    // SAFETY: the clock and the timer are live values.
    unsafe {
        libc::pthread_getcpuclockid(libc::pthread_self(), &raw mut clock);
        WATCH.signal.store(sig, Ordering::Relaxed);
        WATCH.thread.store(libc::gettid(), Ordering::Relaxed);
        WATCH.clock.store(clock, Ordering::Relaxed);
        WATCH.cpu_ns.store(cpu_ns(clock), Ordering::Relaxed);
        WATCH.alarms.store(0, Ordering::Relaxed);
        handle(&[libc::SIGALRM], stuck, libc::SA_RESTART);
        libc::setitimer(libc::ITIMER_REAL, &raw const timer, std::ptr::null_mut());
    }
}

/// The fixed line comes first and needs no lock. The backtrace is best
/// effort: `Backtrace::force_capture` is not async signal safe, so the
/// alarm ends a handler that is stuck in it, and a second fault inside it
/// ends the core by the default action.
#[allow(unsafe_code, reason = "reads the siginfo of the kernel")]
extern "C" fn fault(sig: libc::c_int, info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    // SAFETY: the kernel passes a valid siginfo for a faulting signal.
    let address = if info.is_null() {
        0
    } else {
        unsafe { (*info).si_addr() as usize }
    };
    say(format_args!(
        "{} at address {address:#x}, backtrace follows",
        name(sig)
    ));
    give_up_when_stuck(sig);
    let text = format!("{}\n", Backtrace::force_capture());
    write_all(2, text.as_bytes());
    default_action(sig);
}

/// Whether a signal code says that a process sent the signal with
/// `kill`, `sigqueue` or `tgkill`: only then the siginfo has its pid.
fn from_a_process(code: libc::c_int) -> bool {
    [libc::SI_USER, libc::SI_QUEUE, libc::SI_TKILL].contains(&code)
}

/// The line names the sender when a process sent the signal.
#[allow(unsafe_code, reason = "reads the siginfo of the kernel")]
extern "C" fn end(sig: libc::c_int, info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    // SAFETY: the kernel passes a valid siginfo, and si_pid is set for these codes.
    let sender = (!info.is_null())
        .then(|| unsafe { from_a_process((*info).si_code).then(|| (*info).si_pid()) })
        .flatten();
    match sender {
        Some(pid) => say(format_args!("{} from pid {pid} ends chora", name(sig))),
        None => say(format_args!("{} from the kernel ends chora", name(sig))),
    }
    default_action(sig);
}

/// Installs a handler that runs on the alternate signal stack, with
/// more flags for some signals.
#[allow(unsafe_code, reason = "installs signal handlers")]
fn handle(signals: &[libc::c_int], handler: Handler, flags: libc::c_int) {
    for sig in signals {
        // SAFETY: an all-zero sigaction with a plain handler is valid.
        unsafe {
            let mut act: libc::sigaction = std::mem::zeroed();
            act.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK | flags;
            act.sa_sigaction = handler as usize;
            libc::sigaction(*sig, &raw const act, std::ptr::null_mut());
        }
    }
}

/// Gives the calling thread an alternate signal stack with room for a
/// backtrace. The stack lives as long as the process.
#[allow(unsafe_code, reason = "sets the signal stack of the calling thread")]
pub fn roomy_signal_stack() {
    let stack = Box::leak(vec![0_u8; SIGNAL_STACK].into_boxed_slice());
    let alternate = libc::stack_t {
        ss_sp: stack.as_mut_ptr().cast(),
        ss_flags: 0,
        ss_size: stack.len(),
    };
    // SAFETY: the stack is a live allocation that is never freed.
    unsafe {
        libc::sigaltstack(&raw const alternate, std::ptr::null_mut());
    }
}

/// What a panic leaves in the log: the thread, the message with its
/// place, what happens to the core, and the backtrace.
fn panic_words(thread: Option<&str>, panic: &dyn Display, trace: &dyn Display) -> String {
    let then = if thread == Some("main") {
        "chora ends"
    } else {
        "the core goes on without this thread"
    };
    format!(
        "chora: the thread {} {panic}; {then}; backtrace follows\n{trace}\n",
        thread.unwrap_or("without a name")
    )
}

/// Writes the words of every panic, in every thread, to a file
/// descriptor.
fn panics_to(fd: RawFd) {
    std::panic::set_hook(Box::new(move |panic| {
        let text = panic_words(
            std::thread::current().name(),
            panic,
            &Backtrace::force_capture(),
        );
        write_all(fd, text.as_bytes());
    }));
}

/// Installs the last words for the signals and the panics of all threads.
pub fn install() {
    roomy_signal_stack();
    handle(&FAULTS, fault, libc::SA_RESETHAND);
    handle(&ENDS, end, 0);
    panics_to(2);
    if std::env::var_os("CHORA_TEST_PANIC").is_some() {
        let test = std::thread::Builder::new().name("chora-test".into());
        let _ = test.spawn(|| panic!("CHORA_TEST_PANIC is set"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panic_names_its_thread_and_what_follows() {
        let words = panic_words(Some("images"), &"panicked at a.rs:1:2:\nboom", &"0: frame");
        assert_eq!(
            words,
            "chora: the thread images panicked at a.rs:1:2:\nboom; \
             the core goes on without this thread; backtrace follows\n0: frame\n"
        );
        let words = panic_words(Some("main"), &"panicked", &"");
        assert!(words.contains("; chora ends;"), "{words}");
        let words = panic_words(None, &"panicked", &"");
        assert!(
            words.contains("the thread without a name panicked"),
            "{words}"
        );
    }

    #[test]
    fn only_a_signal_of_a_process_names_a_sender() {
        assert!(from_a_process(libc::SI_USER));
        assert!(from_a_process(libc::SI_QUEUE));
        assert!(from_a_process(libc::SI_TKILL));
        assert!(!from_a_process(libc::SI_TIMER));
        assert!(!from_a_process(libc::SI_KERNEL));
    }

    #[test]
    fn a_handler_goes_on_while_it_works_or_waits_for_a_cpu() {
        assert!(goes_on(true, false, 2000));
        assert!(goes_on(false, true, 2000), "starved of the CPU, not stuck");
        assert!(!goes_on(false, false, 2000), "it waits for a lock");
        assert!(!goes_on(true, true, GIVE_UP_AT_LAST), "it spins");
    }

    #[test]
    fn the_state_of_a_thread_follows_its_name() {
        assert_eq!(state_in(b"7 (chora) R 1 7"), Some(b'R'));
        assert_eq!(state_in(b"7 (a) R (b) S 1 7"), Some(b'S'));
        assert_eq!(state_in(b"7 (chora"), None);
        assert_eq!(state_in(b""), None);
    }

    #[test]
    fn the_thread_that_asks_is_runnable() {
        #[allow(unsafe_code, reason = "the id of the calling thread")]
        // SAFETY: gettid has no arguments and cannot fail.
        let thread = unsafe { libc::gettid() };
        assert!(runnable(thread));
        assert!(!runnable(0));
    }

    /// Runs one test of this binary again in a child with a variable
    /// set, so that a fault or a hook of the whole process stays apart
    /// from the other tests; returns how the child ended and its stderr.
    fn again(test: &str, var: &str) -> (std::process::ExitStatus, String) {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--nocapture", "--test-threads=1"])
            .env(var, "1")
            .output()
            .unwrap();
        (
            child.status,
            String::from_utf8_lossy(&child.stderr).into_owned(),
        )
    }

    #[allow(unconditional_recursion, reason = "the test overflows the stack")]
    fn overflow(depth: u64) -> u64 {
        let frame = std::hint::black_box([depth; 64]);
        overflow(depth + 1) + frame[0]
    }

    /// A stack overflow faults with no room on the stack of its thread:
    /// the handler runs on the alternate stack that Rust gives each
    /// thread, and the signal still ends the process.
    #[test]
    fn a_stack_overflow_leaves_its_words() {
        use std::os::unix::process::ExitStatusExt as _;
        const CHILD: &str = "CHORA_WORDS_OVERFLOW";
        if std::env::var_os(CHILD).is_some() {
            install();
            overflow(0);
        }
        let (status, words) = again("words::tests::a_stack_overflow_leaves_its_words", CHILD);
        assert!(words.contains("chora: SIGSEGV at address"), "{words}");
        assert_eq!(status.signal(), Some(libc::SIGSEGV), "{words}");
    }

    /// A backtrace that takes longer than the time of the alarm is slow,
    /// not stuck: under load the alarm must not end the core with
    /// SIGALRM in the middle of the backtrace (F10). The child gives the
    /// handler 5 ms, which no backtrace of a deep stack meets.
    #[test]
    fn a_slow_backtrace_is_not_cut_by_the_alarm() {
        use std::os::unix::process::ExitStatusExt as _;
        const CHILD: &str = "CHORA_WORDS_SLOW";
        if std::env::var_os(CHILD).is_some() {
            GIVE_UP_AFTER.store(5, Ordering::Relaxed);
            install();
            overflow(0);
        }
        let test = "words::tests::a_slow_backtrace_is_not_cut_by_the_alarm";
        let (status, words) = again(test, CHILD);
        assert_eq!(status.signal(), Some(libc::SIGSEGV), "{words}");
        assert!(words.contains("words::tests::overflow"), "{words}");
    }

    /// A handler that sleeps, as one that waits for the lock of malloc
    /// does, is stuck: the alarm says so and ends the process with the
    /// signal of the fault.
    #[test]
    fn a_stuck_handler_ends_with_the_signal_of_its_fault() {
        use std::os::unix::process::ExitStatusExt as _;
        const CHILD: &str = "CHORA_WORDS_STUCK";
        if std::env::var_os(CHILD).is_some() {
            GIVE_UP_AFTER.store(50, Ordering::Relaxed);
            give_up_when_stuck(libc::SIGBUS);
            std::thread::sleep(std::time::Duration::from_secs(30));
            return;
        }
        let test = "words::tests::a_stuck_handler_ends_with_the_signal_of_its_fault";
        let (status, words) = again(test, CHILD);
        assert_eq!(status.signal(), Some(libc::SIGBUS), "{words}");
        assert!(
            words.contains("chora: the handler of SIGBUS is stuck after"),
            "{words}"
        );
    }

    /// The hook runs in a thread that is not the main thread, here in a
    /// child, because the hook belongs to the whole process.
    #[test]
    fn a_panic_in_a_worker_thread_leaves_its_words() {
        const CHILD: &str = "CHORA_WORDS_PANIC";
        if std::env::var_os(CHILD).is_some() {
            panics_to(2);
            let worker = std::thread::Builder::new()
                .name("chora-worker".into())
                .spawn(|| panic!("the worker gives up"))
                .unwrap();
            assert!(worker.join().is_err());
            return;
        }
        let test = "words::tests::a_panic_in_a_worker_thread_leaves_its_words";
        let (status, words) = again(test, CHILD);
        assert!(status.success(), "{words}");
        assert!(
            words.contains("chora: the thread chora-worker panicked at"),
            "{words}"
        );
        assert!(words.contains("the worker gives up"), "{words}");
        assert!(words.contains("panics_to"), "no backtrace: {words}");
    }
}
