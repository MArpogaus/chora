//! The backends. winit runs chora as a window inside another session; the
//! tty backend runs it on real hardware.

pub mod tty;
pub mod winit;

use smithay::{
    backend::allocator::dmabuf::Dmabuf,
    output::{Mode, Output},
    wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
};

use crate::state::State;

/// The backend that drives the outputs and the input devices.
pub enum Backend {
    /// A window inside another session.
    Winit(Box<winit::Winit>),
    /// Real hardware.
    Tty(Box<tty::Tty>),
}

impl Backend {
    /// Whether the GPU can use a buffer of a client.
    fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        match self {
            Self::Winit(winit) => winit.import_dmabuf(dmabuf),
            Self::Tty(tty) => tty.import_dmabuf(dmabuf),
        }
    }

    /// Switches to another virtual terminal. Only the tty backend can.
    pub fn change_vt(&mut self, vt: i32) {
        if let Self::Tty(tty) = self {
            tty.change_vt(vt);
        }
    }

    /// Checks, without a change, whether an output can show a mode, or
    /// turn off for `None`.
    pub fn test_output(&self, output: &Output, mode: Option<Mode>) -> Result<(), String> {
        match self {
            Self::Winit(winit) => winit.test_output(mode),
            Self::Tty(tty) => tty.test_output(output, mode),
        }
    }

    /// Gives an output that is on a new mode.
    pub fn set_mode(&mut self, output: &Output, mode: Mode) -> Result<(), String> {
        match self {
            Self::Winit(_) => Err(winit::FIXED_MODE.into()),
            Self::Tty(tty) => tty.set_mode(output, mode),
        }
    }

    /// Turns an output on in a mode.
    pub fn enable_output(&mut self, output: &Output, mode: Mode) -> Result<(), String> {
        match self {
            Self::Winit(_) => Err(winit::ALWAYS_ON.into()),
            Self::Tty(tty) => tty.enable_output(output, mode),
        }
    }

    /// Turns an output off. Its connector stays, so that the output can
    /// turn on again.
    pub fn disable_output(&mut self, output: &Output) -> Result<(), String> {
        match self {
            Self::Winit(_) => Err(winit::ALWAYS_ON.into()),
            Self::Tty(tty) => tty.disable_output(output),
        }
    }
}

/// Starts the backend that fits: winit inside another session, else the
/// tty backend.
pub fn init(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    let nested = ["WAYLAND_DISPLAY", "DISPLAY"]
        .iter()
        .any(|v| std::env::var_os(v).is_some());
    if nested {
        winit::init(state)
    } else {
        tty::init(state)
    }
}

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.dmabuf_state
    }

    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        if self
            .backend
            .as_mut()
            .is_some_and(|b| b.import_dmabuf(&dmabuf))
        {
            let _ = notifier.successful::<State>();
        } else {
            notifier.failed();
        }
    }
}
