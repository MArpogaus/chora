//! The core's last words: no end of the core is silent. A faulting
//! signal writes the fault address and a backtrace, a signal that ends
//! the core writes its name and its sender, and a panic writes its
//! thread, its message and a backtrace, all to stderr before the default
//! action. The lines go to fd 2 with a raw `write(2)`, without the lock
//! of tracing or of `std::io::stderr`.
//!
//! A fault writes its fixed line first, from a buffer on the stack, and
//! only then the backtrace, which needs malloc and the lock of the
//! unwinder: after a fault inside malloc that can hang, so an alarm ends
//! the core [`GIVE_UP_AFTER`] seconds later. The handlers run on the
//! alternate signal stack of their thread, so a stack overflow has its
//! words too, and a fault handler runs once: a second fault ends the
//! core at once. The main thread and the hang watchdog, which aborts,
//! get a signal stack with room for the backtrace; on the small stack
//! that Rust gives the other threads only the fixed line is sure.
//! `CHORA_TEST_PANIC` starts a thread that panics, for
//! `tests/last-words.sh`.

use std::{backtrace::Backtrace, fmt::Display, os::fd::RawFd};

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

/// The seconds that a fault handler may take for the backtrace.
const GIVE_UP_AFTER: libc::c_uint = 2;

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

/// Ends the core with SIGALRM after some seconds, whatever the handler
/// that runs then waits for.
#[allow(unsafe_code, reason = "libc calls that a signal handler may make")]
fn give_up_after(seconds: libc::c_uint) {
    // SAFETY: both calls are async signal safe.
    unsafe {
        libc::signal(libc::SIGALRM, libc::SIG_DFL);
        libc::alarm(seconds);
    }
}

/// The fixed line comes first and needs no lock. The backtrace is best
/// effort: `Backtrace::force_capture` is not async signal safe, so the
/// alarm ends a handler that hangs in it, and a second fault inside it
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
    give_up_after(GIVE_UP_AFTER);
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
