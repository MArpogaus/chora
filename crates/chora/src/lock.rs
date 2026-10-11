//! The lock screen of `ext-session-lock`, part of the safety layer.
//!
//! While the session is locked, each output shows only its lock surface, or
//! black, and only the lock surfaces get input; plugin grabs pause. The lock
//! stays when its client dies, so a crash never unlocks the session. A new
//! lock client can then take over.

use std::collections::HashMap;

use smithay::{
    output::Output,
    reexports::wayland_server::{
        Resource,
        protocol::{wl_output::WlOutput, wl_surface::WlSurface},
    },
    wayland::session_lock::{
        LockSurface, SessionLockHandler, SessionLockManagerState, SessionLocker,
    },
};

use crate::{output::output_area, state::State};

/// A locked session: the lock surfaces by output name.
#[derive(Default)]
pub struct Lock {
    surfaces: HashMap<String, LockSurface>,
}

impl Lock {
    /// The lock surface of an output, while its client lives.
    pub fn surface(&self, output: &Output) -> Option<&WlSurface> {
        self.surfaces
            .get(&output.name())
            .map(LockSurface::wl_surface)
            .filter(|s| s.is_alive())
    }
}

impl SessionLockHandler for State {
    fn lock_state(&mut self) -> &mut SessionLockManagerState {
        &mut self.lock_state
    }

    /// Locks at once. The next frame of each output shows only the lock.
    fn lock(&mut self, confirmation: SessionLocker) {
        self.lock = Some(Lock::default());
        confirmation.lock();
        self.update_focus();
        tracing::info!("the session is locked");
    }

    fn unlock(&mut self) {
        self.lock = None;
        self.update_focus();
        tracing::info!("the session is unlocked");
    }

    fn new_surface(&mut self, surface: LockSurface, output: WlOutput) {
        let Some(output) = Output::from_resource(&output) else {
            return;
        };
        let size = output_area(&output).size;
        let size = (u32::try_from(size.w), u32::try_from(size.h));
        if let (Ok(w), Ok(h)) = size {
            surface.with_pending_state(|state| state.size = Some((w, h).into()));
        }
        surface.send_configure();
        if let Some(lock) = &mut self.lock {
            lock.surfaces.insert(output.name(), surface);
        }
        self.update_focus();
    }
}

impl State {
    /// The lock surface that gets the keyboard: the one of the first output
    /// that has one.
    pub fn lock_focus(&self) -> Option<WlSurface> {
        let lock = self.lock.as_ref()?;
        self.outputs.iter().find_map(|o| lock.surface(o)).cloned()
    }
}
