//! A Wayland test client for `tests/nested.sh`. Each mode checks one
//! behavior of chora and exits with 0 when chora behaves as it should:
//!
//! - `second-lock`: a lock fails while another client holds the lock.
//! - `take-lock`: a lock succeeds after the client of the lock died, and
//!   unlocks again.
//! - `popup-grab`: a popup that asks for a grab gets no keyboard while the
//!   session is locked.
//! - `presentation`: a frame reports when it showed, on the monotonic
//!   clock. The test script shows the toplevel `chora-probe`.
//! - `held-commit W H`: the buffer that answers the configure of a
//!   transaction shows no earlier than the transaction. The test script
//!   shows the toplevel, and delays the transaction with a stopped client.
//! - `drag X0 Y0 X1 Y1`: drags with the left button from one point of the
//!   first output to another, through a virtual pointer of the host
//!   compositor, which has no pointer device of its own.
//! - `keys CODE+CODE+...`: holds evdev keys down in order and lets them go
//!   in reverse, through a virtual keyboard of the host with the default
//!   US keymap, so that a nested compositor with the same default keymap
//!   reads the same keys.
//! - `wiggle X0 Y0 X1 Y1 MS`: presses at the first point, moves to the
//!   second one, keeps circling around it with the button down for MS
//!   milliseconds, and releases there; the drag of a wobbly window.
//! - `point X Y MS`: moves the pointer to a point without a button and
//!   stays there for MS milliseconds, for hover looks.
//! - `capture output|toplevel`: captures the first output or the first
//!   toplevel through `ext-image-copy-capture`, and prints the size and
//!   the number of colors.

mod capture;

use std::{
    fs::File,
    io::Write,
    os::fd::AsFd,
    process::ExitCode,
    time::{Duration, Instant},
};

use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{MemfdFlags, memfd_create},
    time::{ClockId, clock_gettime},
};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_keyboard, wl_pointer, wl_registry,
        wl_seat, wl_shm, wl_shm_pool::WlShmPool, wl_surface::WlSurface,
    },
};
use wayland_protocols::{
    ext::session_lock::v1::client::{
        ext_session_lock_manager_v1::ExtSessionLockManagerV1,
        ext_session_lock_v1::{self, ExtSessionLockV1},
    },
    wp::presentation_time::client::{
        wp_presentation::{self, WpPresentation},
        wp_presentation_feedback::{self, WpPresentationFeedback},
    },
    xdg::shell::client::{
        xdg_popup::XdgPopup,
        xdg_positioner::XdgPositioner,
        xdg_surface::{self, XdgSurface},
        xdg_toplevel::{self, XdgToplevel},
        xdg_wm_base::{self, XdgWmBase},
    },
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1,
    zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
};

/// The longest wait for an answer of chora.
const TIMEOUT: Duration = Duration::from_secs(10);
/// The time in which a refused grab must not give the keyboard.
const GRAB_WAIT: Duration = Duration::from_millis(1500);
/// The shortest wait of a held commit: the deadline of a transaction is
/// 150 ms.
const HELD: Duration = Duration::from_millis(100);
/// The id of `CLOCK_MONOTONIC` on Linux.
const CLOCK_MONOTONIC: u32 = 1;

/// The globals and what chora answered so far.
#[derive(Default)]
struct Probe {
    compositor: Option<WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    wm_base: Option<XdgWmBase>,
    seat: Option<wl_seat::WlSeat>,
    lock_manager: Option<ExtSessionLockManagerV1>,
    presentation: Option<WpPresentation>,
    pointer_manager: Option<ZwlrVirtualPointerManagerV1>,
    keyboard_manager: Option<ZwpVirtualKeyboardManagerV1>,
    /// `Some(true)` after `locked`, `Some(false)` after `finished`.
    locked: Option<bool>,
    /// The xdg surfaces that got a configure, by their number.
    configured: Vec<u32>,
    /// The size of the next configure of the toplevel.
    pending_size: [i32; 2],
    /// The sizes of the configures of the toplevel.
    sizes: Vec<[i32; 2]>,
    /// Whether one of the surfaces of the probe got the keyboard.
    keyboard: bool,
    /// The clock of presentation times.
    clock: Option<u32>,
    /// The answer to the request for presentation feedback.
    feedback: Option<Feedback>,
    /// The globals and the answers of the mode `capture`.
    capture: capture::Capture,
}

/// The answer to a request for presentation feedback.
#[derive(Clone, Copy)]
enum Feedback {
    /// The frame showed at a time of the clock.
    Presented(Duration),
    Discarded,
}

type Queue = EventQueue<Probe>;

fn main() -> ExitCode {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let conn = Connection::connect_to_env().expect("a Wayland display");
    let mut queue = conn.new_event_queue();
    let qh = queue.handle();
    conn.display().get_registry(&qh, ());
    let mut probe = Probe::default();
    queue.roundtrip(&mut probe).expect("the globals");
    let result = match mode.as_str() {
        "second-lock" => second_lock(&mut queue, &mut probe),
        "take-lock" => take_lock(&mut queue, &mut probe),
        "popup-grab" => popup_grab(&mut queue, &mut probe),
        "presentation" => presentation(&mut queue, &mut probe),
        "drag" => drag(&mut queue, &mut probe),
        "keys" => keys(&mut queue, &mut probe),
        "wiggle" => wiggle(&mut queue, &mut probe),
        "point" => point(&mut queue, &mut probe),
        "capture" => capture::capture(&mut queue, &mut probe),
        "held-commit" => match size_args() {
            Some(size) => held_commit(&mut queue, &mut probe, size),
            None => Err("usage: held-commit W H".into()),
        },
        _ => Err(format!("unknown mode {mode:?}")),
    };
    match result {
        Ok(message) => {
            println!("{mode}: {message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{mode}: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Locks while another client holds the lock.
fn second_lock(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let lock = lock(queue, probe)?;
    match probe.locked {
        Some(false) => Ok("refused".into()),
        Some(true) => {
            lock.unlock_and_destroy();
            let _ = queue.roundtrip(probe);
            Err("a second client locked the session".into())
        }
        None => Err("no answer".into()),
    }
}

/// Locks after the client of the lock died, and unlocks.
fn take_lock(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let lock = lock(queue, probe)?;
    if probe.locked != Some(true) {
        return Err("the lock failed".into());
    }
    lock.unlock_and_destroy();
    queue.roundtrip(probe).map_err(|e| e.to_string())?;
    Ok("locked and unlocked".into())
}

/// Asks for a lock and waits for the answer.
fn lock(queue: &mut Queue, probe: &mut Probe) -> Result<ExtSessionLockV1, String> {
    let manager = probe.lock_manager.clone().ok_or("no session lock")?;
    let lock = manager.lock(&queue.handle(), ());
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.locked.is_some()) {
        return Err("no answer to the lock".into());
    }
    Ok(lock)
}

/// Maps a toplevel and a popup that asks for a grab, and watches the
/// keyboard.
fn popup_grab(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let qh = queue.handle();
    let seat = probe.seat.clone().ok_or("no seat")?;
    seat.get_keyboard(&qh, ());
    let (surface, xdg_surface) = toplevel(queue, probe)?;
    let wm_base = probe.wm_base.clone().ok_or("no xdg shell")?;
    let popup_surface = create_surface(probe, &qh)?;
    let popup_xdg = wm_base.get_xdg_surface(&popup_surface, &qh, 2);
    let positioner = wm_base.create_positioner(&qh, ());
    positioner.set_size(50, 50);
    positioner.set_anchor_rect(0, 0, 10, 10);
    let popup = popup_xdg.get_popup(Some(&xdg_surface), &positioner, &qh, ());
    popup.grab(&seat, 1);
    popup_surface.commit();
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.configured.contains(&2)) {
        return Err("no configure of the popup".into());
    }
    show(probe, &qh, &popup_surface, [50, 50])?;
    dispatch_until(queue, probe, GRAB_WAIT, |p| p.keyboard);
    drop(surface);
    if probe.keyboard {
        Err("a surface of the probe got the keyboard".into())
    } else {
        Ok("no keyboard".into())
    }
}

/// Maps the toplevel `chora-probe` with a request for presentation
/// feedback and waits until its frame showed.
fn presentation(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let qh = queue.handle();
    let presentation = probe.presentation.clone().ok_or("no presentation time")?;
    let (surface, _xdg_surface) = toplevel(queue, probe)?;
    presentation.feedback(&surface, &qh, ());
    show(probe, &qh, &surface, [200, 200])?;
    let start = monotonic();
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.feedback.is_some()) {
        return Err("no presentation feedback".into());
    }
    let Some(Feedback::Presented(time)) = probe.feedback else {
        return Err("the frame was discarded".into());
    };
    if probe.clock != Some(CLOCK_MONOTONIC) {
        return Err(format!("the clock is {:?}", probe.clock));
    }
    if time < start || time > monotonic() {
        return Err(format!("the time {time:?} is not after {start:?}"));
    }
    Ok(format!("shown after {:?}", time.saturating_sub(start)))
}

/// Shows the toplevel `chora-probe`, then answers the configure of a size
/// with a buffer of that size, and checks how long the buffer waited.
fn held_commit(queue: &mut Queue, probe: &mut Probe, size: [i32; 2]) -> Result<String, String> {
    let qh = queue.handle();
    let presentation = probe.presentation.clone().ok_or("no presentation time")?;
    let (surface, _xdg_surface) = toplevel(queue, probe)?;
    presentation.feedback(&surface, &qh, ());
    show(probe, &qh, &surface, [200, 200])?;
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.feedback.is_some()) {
        return Err("the toplevel did not show".into());
    }
    println!("ready");
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.sizes.contains(&size)) {
        return Err(format!("no configure of the size {size:?}"));
    }
    let start = monotonic();
    probe.feedback = None;
    presentation.feedback(&surface, &qh, ());
    show(probe, &qh, &surface, size)?;
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.feedback.is_some()) {
        return Err("no presentation feedback".into());
    }
    let Some(Feedback::Presented(time)) = probe.feedback else {
        return Err("the frame was discarded".into());
    };
    let waited = time.saturating_sub(start);
    if waited < HELD {
        return Err(format!(
            "the buffer showed after {waited:?}, before its transaction"
        ));
    }
    Ok(format!("the buffer waited {waited:?} for its transaction"))
}

/// The size of the area of a virtual pointer: the output of the test.
const EXTENT: [u32; 2] = [1280, 800];
/// The left button.
const BTN_LEFT: u32 = 272;

/// Drags with the left button from one point to another, in steps.
fn drag(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let points: Vec<u32> = std::env::args()
        .skip(2)
        .filter_map(|a| a.parse().ok())
        .collect();
    let [x0, y0, x1, y1] = points[..] else {
        return Err("usage: drag X0 Y0 X1 Y1".into());
    };
    let manager = probe.pointer_manager.clone().ok_or("no virtual pointer")?;
    let pointer = manager.create_virtual_pointer(probe.seat.as_ref(), &queue.handle(), ());
    let start = Instant::now();
    let time = || u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    let pause = |queue: &mut Queue, probe: &mut Probe| {
        // Lets the client under the pointer see the new seat capability
        // and each step.
        dispatch_until(queue, probe, Duration::from_millis(150), |_| false);
    };
    let to = |x: u32, y: u32| pointer.motion_absolute(time(), x, y, EXTENT[0], EXTENT[1]);
    pause(queue, probe);
    to(x0, y0);
    pointer.frame();
    pause(queue, probe);
    pointer.button(time(), BTN_LEFT, wl_pointer::ButtonState::Pressed);
    pointer.frame();
    pause(queue, probe);
    for step in 1..=4 {
        let along = |a: u32, b: u32| (a * (4 - step) + b * step) / 4;
        to(along(x0, x1), along(y0, y1));
        pointer.frame();
        pause(queue, probe);
    }
    pointer.button(time(), BTN_LEFT, wl_pointer::ButtonState::Released);
    pointer.frame();
    pause(queue, probe);
    pointer.destroy();
    queue.roundtrip(probe).map_err(|e| e.to_string())?;
    Ok(format!("dragged from {x0},{y0} to {x1},{y1}"))
}

/// Holds the left button while the pointer moves to a point and circles
/// around it, then releases there.
fn wiggle(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let args: Vec<u32> = std::env::args()
        .skip(2)
        .filter_map(|a| a.parse().ok())
        .collect();
    let [x0, y0, x1, y1, ms] = args[..] else {
        return Err("usage: wiggle X0 Y0 X1 Y1 MS".into());
    };
    let manager = probe.pointer_manager.clone().ok_or("no virtual pointer")?;
    let pointer = manager.create_virtual_pointer(probe.seat.as_ref(), &queue.handle(), ());
    let start = Instant::now();
    let time = || u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    let to = |x: f64, y: f64| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped
        let (x, y) = (
            x.clamp(0.0, f64::from(EXTENT[0] - 1)) as u32,
            y.clamp(0.0, f64::from(EXTENT[1] - 1)) as u32,
        );
        pointer.motion_absolute(time(), x, y, EXTENT[0], EXTENT[1]);
        pointer.frame();
    };
    let (a, b) = (
        [f64::from(x0), f64::from(y0)],
        [f64::from(x1), f64::from(y1)],
    );
    // As in drag: the client under the pointer sees the new seat capability
    // first, then the first point, then the press.
    dispatch_until(queue, probe, Duration::from_millis(150), |_| false);
    to(a[0], a[1]);
    dispatch_until(queue, probe, Duration::from_millis(150), |_| false);
    pointer.button(time(), BTN_LEFT, wl_pointer::ButtonState::Pressed);
    pointer.frame();
    dispatch_until(queue, probe, Duration::from_millis(150), |_| false);
    let held = Instant::now();
    while held.elapsed() < Duration::from_millis(u64::from(ms)) {
        let s = held.elapsed().as_secs_f64();
        // A quarter of a second to the point, then circles around it.
        let reach = (s / 0.25).min(1.0);
        let (sin, cos) = (s * 9.0).sin_cos();
        let swing = 70.0 * reach;
        to(
            a[0] + (b[0] - a[0]) * reach + swing * cos,
            a[1] + (b[1] - a[1]) * reach + swing * sin,
        );
        dispatch_until(queue, probe, Duration::from_millis(16), |_| false);
    }
    to(b[0], b[1]);
    dispatch_until(queue, probe, Duration::from_millis(100), |_| false);
    pointer.button(time(), BTN_LEFT, wl_pointer::ButtonState::Released);
    pointer.frame();
    dispatch_until(queue, probe, Duration::from_millis(150), |_| false);
    pointer.destroy();
    queue.roundtrip(probe).map_err(|e| e.to_string())?;
    Ok(format!(
        "wiggled from {x0},{y0} around {x1},{y1} for {ms} ms"
    ))
}

/// Moves the pointer to a point and stays there.
fn point(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let args: Vec<u32> = std::env::args()
        .skip(2)
        .filter_map(|a| a.parse().ok())
        .collect();
    let [x, y, ms] = args[..] else {
        return Err("usage: point X Y MS".into());
    };
    let manager = probe.pointer_manager.clone().ok_or("no virtual pointer")?;
    let pointer = manager.create_virtual_pointer(probe.seat.as_ref(), &queue.handle(), ());
    let start = Instant::now();
    let time = || u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    dispatch_until(queue, probe, Duration::from_millis(150), |_| false);
    pointer.motion_absolute(time(), x, y, EXTENT[0], EXTENT[1]);
    pointer.frame();
    dispatch_until(queue, probe, Duration::from_millis(u64::from(ms)), |_| {
        false
    });
    pointer.destroy();
    queue.roundtrip(probe).map_err(|e| e.to_string())?;
    Ok(format!("pointed at {x},{y} for {ms} ms"))
}

/// The default keymap of xkbcommon, US with evdev keycodes: the one of a
/// compositor without XKB variables.
const KEYMAP: &str = "xkb_keymap {
    xkb_keycodes { include \"evdev+aliases(qwerty)\" };
    xkb_types { include \"complete\" };
    xkb_compat { include \"complete\" };
    xkb_symbols { include \"pc+us+inet(evdev)\" };
};
";

/// Holds keys down in order and lets them go in reverse.
fn keys(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let combo = std::env::args().nth(2).unwrap_or_default();
    let codes: Vec<u32> = combo.split('+').filter_map(|c| c.parse().ok()).collect();
    if codes.is_empty() {
        return Err("usage: keys CODE+CODE+...".into());
    }
    let qh = queue.handle();
    let seat = probe.seat.clone().ok_or("no seat")?;
    let manager = probe
        .keyboard_manager
        .clone()
        .ok_or("no virtual keyboard")?;
    let keyboard = manager.create_virtual_keyboard(&seat, &qh, ());
    // The keymap ends with a nul byte, which its size counts.
    let fd = memfd_create("chora-probe-keymap", MemfdFlags::CLOEXEC).map_err(|e| e.to_string())?;
    let mut file = File::from(fd);
    file.write_all(KEYMAP.as_bytes())
        .and_then(|()| file.write_all(&[0]))
        .map_err(|e| e.to_string())?;
    let size = u32::try_from(KEYMAP.len() + 1).unwrap_or(0);
    keyboard.keymap(wl_keyboard::KeymapFormat::XkbV1.into(), file.as_fd(), size);
    let start = Instant::now();
    let time = || u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);
    let pause = |queue: &mut Queue, probe: &mut Probe| {
        dispatch_until(queue, probe, Duration::from_millis(100), |_| false);
    };
    pause(queue, probe);
    for code in &codes {
        keyboard.key(time(), *code, wl_keyboard::KeyState::Pressed.into());
        pause(queue, probe);
    }
    for code in codes.iter().rev() {
        keyboard.key(time(), *code, wl_keyboard::KeyState::Released.into());
        pause(queue, probe);
    }
    keyboard.destroy();
    queue.roundtrip(probe).map_err(|e| e.to_string())?;
    Ok(format!("pressed {combo}"))
}

/// The width and the height in the arguments after the mode.
fn size_args() -> Option<[i32; 2]> {
    let mut args = std::env::args().skip(2).map(|a| a.parse().ok());
    Some([args.next()??, args.next()??])
}

/// Creates a toplevel and waits for its configure.
fn toplevel(queue: &mut Queue, probe: &mut Probe) -> Result<(WlSurface, XdgSurface), String> {
    let qh = queue.handle();
    let wm_base = probe.wm_base.clone().ok_or("no xdg shell")?;
    let surface = create_surface(probe, &qh)?;
    let xdg_surface = wm_base.get_xdg_surface(&surface, &qh, 1);
    let toplevel = xdg_surface.get_toplevel(&qh, ());
    toplevel.set_app_id("chora-probe".into());
    surface.commit();
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.configured.contains(&1)) {
        return Err("no configure of the toplevel".into());
    }
    Ok((surface, xdg_surface))
}

fn create_surface(probe: &Probe, qh: &QueueHandle<Probe>) -> Result<WlSurface, String> {
    let compositor = probe.compositor.as_ref().ok_or("no compositor")?;
    Ok(compositor.create_surface(qh, ()))
}

/// Attaches a gray buffer of a size and commits.
fn show(
    probe: &Probe,
    qh: &QueueHandle<Probe>,
    surface: &WlSurface,
    [w, h]: [i32; 2],
) -> Result<(), String> {
    let shm = probe.shm.as_ref().ok_or("no shm")?;
    let fd = memfd_create("chora-probe", MemfdFlags::CLOEXEC).map_err(|e| e.to_string())?;
    let mut file = File::from(fd);
    let pixels = usize::try_from(w * h).map_err(|e| e.to_string())?;
    file.write_all(&[0x80; 4].repeat(pixels))
        .map_err(|e| e.to_string())?;
    let pool = shm.create_pool(file.as_fd(), w * h * 4, qh, ());
    let buffer = pool.create_buffer(0, w, h, w * 4, wl_shm::Format::Xrgb8888, qh, ());
    surface.attach(Some(&buffer), 0, 0);
    surface.damage_buffer(0, 0, w, h);
    surface.commit();
    Ok(())
}

/// Dispatches events until a condition holds or the time is up. Returns
/// whether the condition holds.
fn dispatch_until(
    queue: &mut Queue,
    probe: &mut Probe,
    timeout: Duration,
    done: impl Fn(&Probe) -> bool,
) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if queue.dispatch_pending(probe).is_err() {
            return false;
        }
        if done(probe) {
            return true;
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() || queue.flush().is_err() {
            return false;
        }
        if let Some(guard) = queue.prepare_read() {
            let fd = guard.connection_fd();
            let mut fds = [PollFd::new(&fd, PollFlags::IN)];
            let timeout = Timespec::try_from(left).unwrap_or(Timespec {
                tv_sec: 1,
                tv_nsec: 0,
            });
            if poll(&mut fds, Some(&timeout)).unwrap_or(0) > 0 && guard.read().is_err() {
                return false;
            }
        }
    }
}

/// The time of the monotonic clock.
fn monotonic() -> Duration {
    let now = clock_gettime(ClockId::Monotonic);
    Duration::new(
        u64::try_from(now.tv_sec).unwrap_or(0),
        u32::try_from(now.tv_nsec).unwrap_or(0),
    )
}

impl Dispatch<wl_registry::WlRegistry, ()> for Probe {
    fn event(
        probe: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name, interface, ..
        } = event
        else {
            return;
        };
        match interface.as_str() {
            "wl_compositor" => probe.compositor = Some(registry.bind(name, 4, qh, ())),
            "wl_shm" => probe.shm = Some(registry.bind(name, 1, qh, ())),
            "xdg_wm_base" => probe.wm_base = Some(registry.bind(name, 1, qh, ())),
            "wl_seat" => probe.seat = Some(registry.bind(name, 1, qh, ())),
            "ext_session_lock_manager_v1" => {
                probe.lock_manager = Some(registry.bind(name, 1, qh, ()));
            }
            "wp_presentation" => probe.presentation = Some(registry.bind(name, 1, qh, ())),
            "zwp_virtual_keyboard_manager_v1" => {
                probe.keyboard_manager = Some(registry.bind(name, 1, qh, ()));
            }
            "zwlr_virtual_pointer_manager_v1" => {
                probe.pointer_manager = Some(registry.bind(name, 1, qh, ()));
            }
            other => probe.capture.bind(registry, name, other, qh),
        }
    }
}

impl Dispatch<XdgWmBase, ()> for Probe {
    fn event(
        _: &mut Self,
        wm_base: &XdgWmBase,
        event: xdg_wm_base::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<XdgSurface, u32> for Probe {
    fn event(
        probe: &mut Self,
        surface: &XdgSurface,
        event: xdg_surface::Event,
        number: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            surface.ack_configure(serial);
            probe.configured.push(*number);
            if *number == 1 {
                probe.sizes.push(probe.pending_size);
            }
        }
    }
}

impl Dispatch<XdgToplevel, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &XdgToplevel,
        event: xdg_toplevel::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Configure { width, height, .. } = event {
            probe.pending_size = [width, height];
        }
    }
}

impl Dispatch<ExtSessionLockV1, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            ext_session_lock_v1::Event::Locked => probe.locked = Some(true),
            ext_session_lock_v1::Event::Finished => probe.locked = Some(false),
            _ => {}
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Enter { .. } = event {
            probe.keyboard = true;
        }
    }
}

impl Dispatch<WpPresentation, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &WpPresentation,
        event: wp_presentation::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_presentation::Event::ClockId { clk_id } = event {
            probe.clock = Some(clk_id);
        }
    }
}

impl Dispatch<WpPresentationFeedback, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wp_presentation_feedback::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                ..
            } => {
                let seconds = (u64::from(tv_sec_hi) << 32) | u64::from(tv_sec_lo);
                probe.feedback = Some(Feedback::Presented(Duration::new(seconds, tv_nsec)));
            }
            wp_presentation_feedback::Event::Discarded => {
                probe.feedback = Some(Feedback::Discarded);
            }
            _ => {}
        }
    }
}

delegate_noop!(Probe: ignore WlCompositor);
delegate_noop!(Probe: ignore WlSurface);
delegate_noop!(Probe: ignore wl_shm::WlShm);
delegate_noop!(Probe: ignore WlShmPool);
delegate_noop!(Probe: ignore WlBuffer);
delegate_noop!(Probe: ignore wl_seat::WlSeat);
delegate_noop!(Probe: ignore XdgPopup);
delegate_noop!(Probe: ignore XdgPositioner);
delegate_noop!(Probe: ignore ExtSessionLockManagerV1);
delegate_noop!(Probe: ignore ZwlrVirtualPointerManagerV1);
delegate_noop!(Probe: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(Probe: ignore ZwpVirtualKeyboardV1);
delegate_noop!(Probe: ignore ZwlrVirtualPointerV1);
