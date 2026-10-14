//! The headless backend of the unit tests: a GLES renderer on the software
//! EGL device of Mesa, with no screen. Captures render with it in the test
//! process, so that a test checks pixels.

use smithay::backend::{
    allocator::dmabuf::Dmabuf,
    egl::{EGLContext, EGLDevice, EGLDisplay},
    renderer::{ImportDma, gles::GlesRenderer},
};

/// The renderer of the headless backend.
pub struct Headless {
    renderer: GlesRenderer,
}

impl Headless {
    /// A renderer on the software device of Mesa, so that the pixels do
    /// not depend on a GPU.
    pub fn new() -> Result<Self, String> {
        let device = EGLDevice::enumerate()
            .map_err(|e| format!("no EGL devices: {e}"))?
            .find(EGLDevice::is_software)
            .ok_or("no software EGL device; the tests need the Mesa drivers")?;
        // SAFETY: the display and its context live as long as the renderer,
        // and only the thread of the test uses them.
        #[allow(unsafe_code)]
        let renderer = unsafe {
            let display = EGLDisplay::new(device).map_err(|e| e.to_string())?;
            let context = EGLContext::new(&display).map_err(|e| e.to_string())?;
            GlesRenderer::new(context).map_err(|e| e.to_string())?
        };
        Ok(Self { renderer })
    }

    /// The renderer.
    pub fn gles(&mut self) -> &mut GlesRenderer {
        &mut self.renderer
    }

    /// Whether the renderer can use a buffer of a client.
    pub fn import_dmabuf(&mut self, dmabuf: &Dmabuf) -> bool {
        self.renderer.import_dmabuf(dmabuf, None).is_ok()
    }
}
