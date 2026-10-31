//! The lock screen of `ext-session-lock`, part of the safety layer.
//!
//! While the session is locked, each output shows only its lock surface, or
//! black, and only the lock client gets input; plugin and popup grabs end
//! and pause. One lock client owns the lock. Other clients cannot lock or
//! unlock while it lives. When it dies, the session stays locked, and a new
//! lock client can take over.
//!
//! The core confirms a lock only after each output showed a frame that it
//! rendered under this lock, or after a second if an output shows nothing,
//! for example while the session is paused. Each lock has a generation, so
//! that a frame rendered before the lock, or under an earlier lock, does
//! not count.

use std::{
    collections::HashSet,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use smithay::{
    backend::input::InputTime,
    output::Output,
    reexports::{
        calloop::{
            RegistrationToken,
            timer::{TimeoutAction, Timer},
        },
        wayland_protocols::ext::session_lock::v1::server::ext_session_lock_v1::ExtSessionLockV1,
        wayland_server::{
            Resource,
            protocol::{wl_output::WlOutput, wl_surface::WlSurface},
        },
    },
    utils::SERIAL_COUNTER,
    wayland::session_lock::{
        LockSurface, SessionLockHandler, SessionLockManagerState, SessionLocker,
    },
};

use crate::{output::output_area, state::State};

/// The longest wait for locked frames before the core confirms a lock.
const CONFIRM_DEADLINE: Duration = Duration::from_secs(1);

/// The generation of the newest lock.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// A locked session.
pub struct Lock {
    /// The lock object of the client that owns the lock.
    owner: ExtSessionLockV1,
    /// The lock surfaces by output name.
    surfaces: Vec<(String, LockSurface)>,
    /// The confirmation, until each output showed a locked frame.
    pending: Option<SessionLocker>,
    /// The outputs that showed a frame of this lock.
    shown: HashSet<String>,
    /// Tells frames of this lock from earlier frames.
    generation: u64,
    /// The deadline timer of the confirmation.
    timer: Option<RegistrationToken>,
}

impl Lock {
    /// The lock surface of an output, while its client lives.
    pub fn surface(&self, output: &Output) -> Option<&WlSurface> {
        let name = output.name();
        self.surfaces
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, s)| s.wl_surface())
            .filter(|s| s.is_alive())
    }

    /// Whether a surface belongs to the client that owns the lock.
    pub fn owns(&self, surface: &WlSurface) -> bool {
        self.owner
            .client()
            .is_some_and(|c| surface.client().as_ref() == Some(&c))
    }
}

impl SessionLockHandler for State {
    fn lock_state(&mut self) -> &mut SessionLockManagerState {
        &mut self.lock_state
    }

    /// Takes the lock, unless a live client owns it. Dropping the
    /// confirmation tells a refused client that it failed.
    fn lock(&mut self, confirmation: SessionLocker) {
        if self.lock.as_ref().is_some_and(|l| l.owner.is_alive()) {
            refuse_lock();
            return;
        }
        self.drop_lock_timer();
        self.lock = Some(Lock {
            owner: confirmation.ext_session_lock().clone(),
            surfaces: Vec::new(),
            pending: Some(confirmation),
            shown: HashSet::new(),
            generation: GENERATION.fetch_add(1, Ordering::Relaxed) + 1,
            timer: None,
        });
        self.lock_changed();
        self.confirm_lock_later();
        tracing::info!("the session is locked");
    }

    fn unlock(&mut self) {
        self.drop_lock_timer();
        self.lock = None;
        self.lock_changed();
        tracing::info!("the session is unlocked");
    }

    fn new_surface(&mut self, surface: LockSurface, output: WlOutput) {
        let Some(output) = Output::from_resource(&output).filter(|o| self.outputs.contains(o))
        else {
            return;
        };
        let Some(lock) = self
            .lock
            .as_mut()
            .filter(|l| l.owner == *surface.ext_session_lock())
        else {
            return;
        };
        let size = output_area(&output).size;
        if let (Ok(w), Ok(h)) = (u32::try_from(size.w), u32::try_from(size.h)) {
            surface.with_pending_state(|state| state.size = Some((w, h).into()));
        }
        surface.send_configure();
        lock.surfaces.retain(|(name, _)| *name != output.name());
        lock.surfaces.push((output.name(), surface));
        self.update_focus();
        // The pointer enters the lock surface without a move.
        self.refresh_pointer();
    }
}

impl State {
    /// The lock surface that gets the keyboard: the one of the first output
    /// that has one.
    pub fn lock_focus(&self) -> Option<WlSurface> {
        let lock = self.lock.as_ref()?;
        self.outputs.iter().find_map(|o| lock.surface(o)).cloned()
    }

    /// The generation of the lock, for the frames that render under it.
    pub fn lock_generation(&self) -> Option<u64> {
        self.lock.as_ref().map(|l| l.generation)
    }

    /// Notes that an output showed a frame that rendered under the lock of
    /// a generation. A pending lock is confirmed when every output showed a
    /// frame of it.
    pub fn frame_shown(&mut self, output: &Output, generation: Option<u64>) {
        let Some(lock) = self
            .lock
            .as_mut()
            .filter(|l| l.pending.is_some() && Some(l.generation) == generation)
        else {
            return;
        };
        lock.shown.insert(output.name());
        if self.outputs.iter().all(|o| lock.shown.contains(&o.name())) {
            self.confirm_lock();
        }
    }

    /// Confirms the lock at the deadline, if the frames take longer.
    fn confirm_lock_later(&mut self) {
        let deadline = Timer::from_duration(CONFIRM_DEADLINE);
        let timer = self.loop_handle.insert_source(deadline, |_, (), state| {
            if let Some(lock) = &mut state.lock {
                lock.timer = None;
            }
            state.confirm_lock();
            TimeoutAction::Drop
        });
        match (timer, &mut self.lock) {
            (Ok(token), Some(lock)) => lock.timer = Some(token),
            _ => self.confirm_lock(),
        }
    }

    /// Stops the deadline timer of the lock, so that it cannot confirm a
    /// later lock.
    fn drop_lock_timer(&mut self) {
        if let Some(timer) = self.lock.as_mut().and_then(|l| l.timer.take()) {
            self.loop_handle.remove(timer);
        }
    }

    /// Sends each lock surface the size of its output again, after an
    /// output changed, for example its mode, scale or transform.
    pub fn resize_lock_surfaces(&self) {
        let Some(lock) = &self.lock else { return };
        for (name, surface) in &lock.surfaces {
            let Some(output) = self.outputs.iter().find(|o| o.name() == *name) else {
                continue;
            };
            let size = output_area(output).size;
            let (Ok(w), Ok(h)) = (u32::try_from(size.w), u32::try_from(size.h)) else {
                continue;
            };
            let changed = surface.with_pending_state(|state| {
                let size = Some((w, h).into());
                std::mem::replace(&mut state.size, size) != size
            });
            if changed {
                surface.send_configure();
            }
        }
    }

    fn confirm_lock(&mut self) {
        if let Some(pending) = self.lock.as_mut().and_then(|l| l.pending.take()) {
            pending.lock();
        }
    }

    /// Ends the grabs and moves the keyboard and the pointer after the
    /// session locked or unlocked.
    fn lock_changed(&mut self) {
        self.node_grab = None;
        self.button_grab = None;
        let pointer = self.pointer();
        pointer.unset_grab(self, SERIAL_COUNTER.next_serial(), InputTime::now());
        self.update_focus();
        self.refresh_pointer();
    }
}

/// Dropping the confirmation of a refused lock tells its client that it
/// failed.
fn refuse_lock() {
    tracing::warn!("a second client tried to lock the session");
}

#[cfg(test)]
mod tests {
    use smithay::output::Scale;

    use crate::testing::{Core, wayland::Wayland};

    #[test]
    fn a_lock_surface_follows_its_output_and_gets_the_pointer_at_once() {
        let mut core = Core::new();
        let output = core.add_output("HDMI-1", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let lock = wayland.lock(&mut core);
        let surface = wayland.lock_surface(&mut core, &lock);
        assert_eq!(wayland.lock_configures(&surface).size, (800, 600));
        let pointer = core.state.seat.get_pointer().expect("a pointer");
        assert_eq!(
            pointer.current_focus().as_ref(),
            core.state.lock_focus().as_ref()
        );
        assert!(pointer.current_focus().is_some());
        output.change_current_state(None, None, Some(Scale::Fractional(2.0)), None);
        core.state.output_changed();
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.lock_configures(&surface).size, (400, 300));
    }
}
