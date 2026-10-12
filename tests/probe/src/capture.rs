//! The mode `capture output|toplevel`: captures the first output, or the
//! first toplevel, through `ext-image-copy-capture` into an shm buffer,
//! and prints the size and the number of colors of the picture.

use std::{fs::File, io::Read, io::Seek, os::fd::AsFd};

use rustix::fs::{MemfdFlags, memfd_create};
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_noop, event_created_child,
    protocol::{wl_output::WlOutput, wl_registry, wl_shm},
};
use wayland_protocols::ext::{
    foreign_toplevel_list::v1::client::{
        ext_foreign_toplevel_handle_v1::{self, ExtForeignToplevelHandleV1},
        ext_foreign_toplevel_list_v1::{self, ExtForeignToplevelListV1},
    },
    image_capture_source::v1::client::{
        ext_foreign_toplevel_image_capture_source_manager_v1::ExtForeignToplevelImageCaptureSourceManagerV1,
        ext_image_capture_source_v1::ExtImageCaptureSourceV1,
        ext_output_image_capture_source_manager_v1::ExtOutputImageCaptureSourceManagerV1,
    },
    image_copy_capture::v1::client::{
        ext_image_copy_capture_frame_v1::{self, ExtImageCopyCaptureFrameV1},
        ext_image_copy_capture_manager_v1::{ExtImageCopyCaptureManagerV1, Options},
        ext_image_copy_capture_session_v1::{self, ExtImageCopyCaptureSessionV1},
    },
};

use crate::{Probe, Queue, TIMEOUT, dispatch_until};

/// The globals of the capture and what chora answered.
#[derive(Default)]
pub struct Capture {
    output: Option<WlOutput>,
    outputs: Option<ExtOutputImageCaptureSourceManagerV1>,
    toplevel_sources: Option<ExtForeignToplevelImageCaptureSourceManagerV1>,
    manager: Option<ExtImageCopyCaptureManagerV1>,
    list: Option<ExtForeignToplevelListV1>,
    toplevels: Vec<ExtForeignToplevelHandleV1>,
    size: Option<(u32, u32)>,
    formats: Vec<wl_shm::Format>,
    constraints_done: bool,
    /// `Some(Ok(()))` after `ready`, `Some(Err(reason))` after `failed`.
    result: Option<Result<(), u32>>,
}

impl Capture {
    /// Binds a global of the capture, if it is one.
    pub fn bind(
        &mut self,
        registry: &wl_registry::WlRegistry,
        name: u32,
        interface: &str,
        qh: &QueueHandle<Probe>,
    ) {
        match interface {
            "wl_output" if self.output.is_none() => {
                self.output = Some(registry.bind(name, 1, qh, ()));
            }
            "ext_output_image_capture_source_manager_v1" => {
                self.outputs = Some(registry.bind(name, 1, qh, ()));
            }
            "ext_foreign_toplevel_image_capture_source_manager_v1" => {
                self.toplevel_sources = Some(registry.bind(name, 1, qh, ()));
            }
            "ext_image_copy_capture_manager_v1" => {
                self.manager = Some(registry.bind(name, 1, qh, ()));
            }
            "ext_foreign_toplevel_list_v1" => self.list = Some(registry.bind(name, 1, qh, ())),
            _ => {}
        }
    }
}

/// Captures the first output or the first toplevel.
pub fn capture(queue: &mut Queue, probe: &mut Probe) -> Result<String, String> {
    let qh = queue.handle();
    let what = std::env::args().nth(2).unwrap_or_default();
    let source = source(queue, probe, &what)?;
    let manager = probe
        .capture
        .manager
        .clone()
        .ok_or("no ext_image_copy_capture_manager_v1")?;
    let session = manager.create_session(&source, Options::empty(), &qh, ());
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.capture.constraints_done) {
        return Err("no buffer constraints".into());
    }
    let (w, h) = probe.capture.size.ok_or("no buffer size")?;
    let format = [wl_shm::Format::Xrgb8888, wl_shm::Format::Argb8888]
        .into_iter()
        .find(|f| probe.capture.formats.contains(f))
        .ok_or("no 32-bit shm format")?;
    let shm = probe.shm.clone().ok_or("no shm")?;
    let fd = memfd_create("chora-probe-capture", MemfdFlags::CLOEXEC).map_err(|e| e.to_string())?;
    let mut file = File::from(fd);
    let bytes = usize::try_from(w * h * 4).map_err(|e| e.to_string())?;
    file.set_len(bytes as u64).map_err(|e| e.to_string())?;
    let stride = i32::try_from(w * 4).map_err(|e| e.to_string())?;
    let height = i32::try_from(h).map_err(|e| e.to_string())?;
    let pool = shm.create_pool(file.as_fd(), stride * height, &qh, ());
    let buffer = pool.create_buffer(0, stride / 4, height, stride, format, &qh, ());
    let frame = session.create_frame(&qh, ());
    frame.attach_buffer(&buffer);
    frame.damage_buffer(0, 0, stride / 4, height);
    frame.capture();
    if !dispatch_until(queue, probe, TIMEOUT, |p| p.capture.result.is_some()) {
        return Err("the frame got no answer".into());
    }
    if let Some(Err(reason)) = probe.capture.result {
        return Err(format!("the frame failed: reason {reason}"));
    }
    let mut pixels = vec![0; bytes];
    file.rewind().map_err(|e| e.to_string())?;
    file.read_exact(&mut pixels).map_err(|e| e.to_string())?;
    let mut colors: Vec<&[u8]> = pixels.as_chunks::<4>().0.iter().map(|p| &p[..3]).collect();
    colors.sort_unstable();
    colors.dedup();
    Ok(format!("{w}x{h}, {} colors", colors.len()))
}

/// The source of the first output, or of the first toplevel.
fn source(
    queue: &mut Queue,
    probe: &mut Probe,
    what: &str,
) -> Result<ExtImageCaptureSourceV1, String> {
    let qh = queue.handle();
    match what {
        "output" => {
            let manager = probe
                .capture
                .outputs
                .clone()
                .ok_or("no output source manager")?;
            let output = probe.capture.output.clone().ok_or("no output")?;
            Ok(manager.create_source(&output, &qh, ()))
        }
        "toplevel" => {
            if !dispatch_until(queue, probe, TIMEOUT, |p| !p.capture.toplevels.is_empty()) {
                return Err("no toplevel in ext_foreign_toplevel_list_v1".into());
            }
            let manager = probe
                .capture
                .toplevel_sources
                .clone()
                .ok_or("no toplevel source manager")?;
            Ok(manager.create_source(&probe.capture.toplevels[0], &qh, ()))
        }
        _ => Err("usage: capture output|toplevel".into()),
    }
}

impl Dispatch<ExtImageCopyCaptureSessionV1, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &ExtImageCopyCaptureSessionV1,
        event: ext_image_copy_capture_session_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use ext_image_copy_capture_session_v1::Event;
        match event {
            Event::BufferSize { width, height } => probe.capture.size = Some((width, height)),
            Event::ShmFormat { format } => {
                if let Ok(format) = format.into_result() {
                    probe.capture.formats.push(format);
                }
            }
            Event::Done => probe.capture.constraints_done = true,
            _ => {}
        }
    }
}

impl Dispatch<ExtImageCopyCaptureFrameV1, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &ExtImageCopyCaptureFrameV1,
        event: ext_image_copy_capture_frame_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use ext_image_copy_capture_frame_v1::Event;
        match event {
            Event::Ready => probe.capture.result = Some(Ok(())),
            Event::Failed { reason } => {
                probe.capture.result = Some(Err(reason.into()));
            }
            _ => {}
        }
    }
}

impl Dispatch<ExtForeignToplevelListV1, ()> for Probe {
    fn event(
        probe: &mut Self,
        _: &ExtForeignToplevelListV1,
        event: ext_foreign_toplevel_list_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_foreign_toplevel_list_v1::Event::Toplevel { toplevel } = event {
            probe.capture.toplevels.push(toplevel);
        }
    }

    event_created_child!(Probe, ExtForeignToplevelListV1, [
        ext_foreign_toplevel_list_v1::EVT_TOPLEVEL_OPCODE => (ExtForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<ExtForeignToplevelHandleV1, ()> for Probe {
    fn event(
        probe: &mut Self,
        handle: &ExtForeignToplevelHandleV1,
        event: ext_foreign_toplevel_handle_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_foreign_toplevel_handle_v1::Event::Closed = event {
            probe.capture.toplevels.retain(|t| t != handle);
        }
    }
}

delegate_noop!(Probe: ignore WlOutput);
delegate_noop!(Probe: ignore ExtOutputImageCaptureSourceManagerV1);
delegate_noop!(Probe: ignore ExtForeignToplevelImageCaptureSourceManagerV1);
delegate_noop!(Probe: ignore ExtImageCaptureSourceV1);
delegate_noop!(Probe: ignore ExtImageCopyCaptureManagerV1);
