//! The core's last words: no end of the core is silent. A faulting
//! signal writes the fault address and a backtrace, a signal that ends
//! the core writes its name and its sender, and a panic writes its
//! thread, its message and a backtrace, all to stderr before the default
//! action. The lines go to fd 2 with a raw `write(2)`, without the lock
//! of tracing or of `std::io::stderr`.

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

/// Best effort: `Backtrace::force_capture` is not async signal safe, so a
/// second fault inside it stays unlogged.
#[allow(unsafe_code, reason = "reads the siginfo of the kernel")]
extern "C" fn fault(sig: libc::c_int, info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    // SAFETY: the kernel passes a valid siginfo for a faulting signal.
    let address = if info.is_null() {
        0
    } else {
        unsafe { (*info).si_addr() as usize }
    };
    let text = format!(
        "chora: {} at address {address:#x}, backtrace follows\n{}\n",
        name(sig),
        Backtrace::force_capture()
    );
    write_all(2, text.as_bytes());
    default_action(sig);
}

/// The line names the sender when a process sent the signal: its code
/// is then at most 0.
#[allow(unsafe_code, reason = "reads the siginfo of the kernel")]
extern "C" fn end(sig: libc::c_int, info: *mut libc::siginfo_t, _ctx: *mut libc::c_void) {
    // SAFETY: the kernel passes a valid siginfo, and si_pid is set for a code of at most 0.
    let sender = (!info.is_null())
        .then(|| unsafe { ((*info).si_code <= 0).then(|| (*info).si_pid()) })
        .flatten();
    match sender {
        Some(pid) => say(format_args!("{} from pid {pid} ends chora", name(sig))),
        None => say(format_args!("{} from the kernel ends chora", name(sig))),
    }
    default_action(sig);
}

#[allow(unsafe_code, reason = "installs signal handlers")]
fn handle(signals: &[libc::c_int], handler: Handler) {
    for sig in signals {
        // SAFETY: an all-zero sigaction with a plain handler is valid.
        unsafe {
            let mut act: libc::sigaction = std::mem::zeroed();
            act.sa_flags = libc::SA_SIGINFO;
            act.sa_sigaction = handler as usize;
            libc::sigaction(*sig, &raw const act, std::ptr::null_mut());
        }
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
    handle(&FAULTS, fault);
    handle(&ENDS, end);
    panics_to(2);
}

#[cfg(test)]
mod tests {
    use std::{io::Read as _, os::fd::AsRawFd as _};

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

    /// The hook runs in a thread that is not the main thread.
    #[test]
    fn a_panic_in_a_worker_thread_leaves_its_words() {
        let (mut reader, writer) = std::io::pipe().unwrap();
        let before = std::panic::take_hook();
        panics_to(writer.as_raw_fd());
        let worker = std::thread::Builder::new()
            .name("chora-worker".into())
            .spawn(|| panic!("the worker gives up"))
            .unwrap();
        assert!(worker.join().is_err());
        std::panic::set_hook(before);
        drop(writer);
        let mut words = String::new();
        reader.read_to_string(&mut words).unwrap();
        assert!(
            words.contains("chora: the thread chora-worker panicked at"),
            "{words}"
        );
        assert!(words.contains("the worker gives up"), "{words}");
        assert!(words.contains("panics_to"), "no backtrace: {words}");
    }
}
