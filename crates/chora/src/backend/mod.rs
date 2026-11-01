//! The backends. winit runs chora as a window inside another session; the
//! tty backend runs it on real hardware. The headless backend runs it with
//! virtual outputs, for tests, and renders in the unit tests.

pub mod headless;
mod host_keymap;
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
    /// Virtual outputs without a screen, for tests.
    Headless(Box<headless::Headless>),
}

impl Backend {
    /// Whether the GPU can use a buffer of a client.
    fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        match self {
            Self::Winit(winit) => winit.import_dmabuf(dmabuf),
            Self::Tty(tty) => tty.import_dmabuf(dmabuf),
            Self::Headless(headless) => headless.import_dmabuf(dmabuf),
        }
    }

    /// Compiles the shader of an effect on the renderer, to check it.
    pub fn check_effect(&mut self, path: &str) -> Result<(), String> {
        match self {
            Backend::Winit(winit) => winit.check_effect(path),
            Backend::Tty(tty) => tty.check_effect(path),
            Backend::Headless(headless) => crate::gl::check_effect(headless.gles(), path),
        }
    }

    /// Runs a function with the GLES renderer of the primary GPU, for
    /// example to capture the screen.
    pub fn with_gles<T>(
        &mut self,
        f: impl FnOnce(&mut smithay::backend::renderer::gles::GlesRenderer) -> T,
    ) -> Result<T, String> {
        match self {
            Self::Winit(winit) => Ok(f(winit.gles())),
            Self::Tty(tty) => tty.with_gles(f),
            Self::Headless(headless) => Ok(f(headless.gles())),
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
            Self::Headless(_) => Ok(()),
        }
    }

    /// Gives an output that is on a new mode.
    pub fn set_mode(&mut self, output: &Output, mode: Mode) -> Result<(), String> {
        match self {
            Self::Winit(_) => Err(winit::FIXED_MODE.into()),
            Self::Tty(tty) => tty.set_mode(output, mode),
            Self::Headless(_) => Ok(()),
        }
    }

    /// Turns an output on in a mode.
    pub fn enable_output(&mut self, output: &Output, mode: Mode) -> Result<(), String> {
        match self {
            Self::Winit(_) => Err(winit::ALWAYS_ON.into()),
            Self::Tty(tty) => tty.enable_output(output, mode),
            Self::Headless(_) => Ok(()),
        }
    }

    /// Turns an output off. Its connector stays, so that the output can
    /// turn on again.
    pub fn disable_output(&mut self, output: &Output) -> Result<(), String> {
        match self {
            Self::Winit(_) => Err(winit::ALWAYS_ON.into()),
            Self::Tty(tty) => tty.disable_output(output),
            Self::Headless(_) => Ok(()),
        }
    }
}

/// Starts the backend that fits: headless with virtual outputs if the
/// command line asks for it, winit inside another session, else the tty
/// backend. The startup command names the window of winit.
pub fn init(
    state: &mut State,
    headless: Option<&[headless::Spec]>,
    command: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(specs) = headless {
        return headless::init(state, specs);
    }
    let nested = ["WAYLAND_DISPLAY", "DISPLAY"]
        .iter()
        .any(|v| std::env::var_os(v).is_some());
    if nested {
        winit::init(state, command)
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
