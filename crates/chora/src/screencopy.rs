//! `wlr-screencopy`: grim, OBS and `xdg-desktop-portal-wlr` capture
//! outputs (D37).
//!
//! A frame offers one shm buffer in the format XRGB8888, in the size of the
//! output or of its region. The core renders the output into it as on
//! screen, without the cursor; `overlay_cursor` has no effect.
//! Frames wait for the next tick of `copies`, one refresh later, so that a
//! stream gets at most one frame per refresh. `copy_with_damage` reports
//! the whole frame as damaged. A frame fails while the session is locked.

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use smithay::{
    output::Output,
    reexports::{
        wayland_protocols_wlr::screencopy::v1::server::{
            zwlr_screencopy_frame_v1::{self as frame, ZwlrScreencopyFrameV1},
            zwlr_screencopy_manager_v1::{self as manager, ZwlrScreencopyManagerV1},
        },
        wayland_server::{
            Client, DataInit, DisplayHandle, New, Resource,
            backend::ClientId,
            protocol::{wl_buffer::WlBuffer, wl_output::WlOutput, wl_shm},
        },
    },
    wayland::{Dispatch2, GlobalDispatch2},
};

use crate::{
    copies::{self, Pictures},
    render, shm,
    state::State,
};

/// Version 3 adds `buffer_done` and dmabuf buffers; the core offers shm
/// buffers only.
const VERSION: u32 = 3;

/// Creates the global.
pub fn create_global(display: &DisplayHandle) {
    display.create_global::<State, ZwlrScreencopyManagerV1, _>(VERSION, ScreencopyGlobal);
}

/// The data of the global.
pub struct ScreencopyGlobal;

/// The data of a bound manager.
pub struct ManagerData;

/// The data of a frame: the output and the part of it in physical pixels,
/// or nothing for a frame that failed at once.
pub struct FrameData {
    target: Option<(Output, [u32; 4])>,
    used: AtomicBool,
}

impl GlobalDispatch2<ZwlrScreencopyManagerV1, State> for ScreencopyGlobal {
    fn bind(
        &self,
        _state: &mut State,
        _display: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrScreencopyManagerV1>,
        data_init: &mut DataInit<'_, State>,
    ) {
        data_init.init(resource, ManagerData);
    }

    fn can_view(&self, client: &Client) -> bool {
        crate::security::may_capture(client)
    }
}

impl Dispatch2<ZwlrScreencopyManagerV1, State> for ManagerData {
    fn request(
        &self,
        _state: &mut State,
        _client: &Client,
        _resource: &ZwlrScreencopyManagerV1,
        request: manager::Request,
        _display: &DisplayHandle,
        data_init: &mut DataInit<'_, State>,
    ) {
        match request {
            manager::Request::CaptureOutput { frame, output, .. } => {
                start(data_init, frame, &output, None);
            }
            manager::Request::CaptureOutputRegion {
                frame,
                output,
                x,
                y,
                width,
                height,
                ..
            } => start(data_init, frame, &output, Some([x, y, width, height])),
            _ => {}
        }
    }
}

/// Offers the buffer of a new frame, or fails it.
fn start(
    data_init: &mut DataInit<'_, State>,
    frame: New<ZwlrScreencopyFrameV1>,
    output: &WlOutput,
    region: Option<[i32; 4]>,
) {
    let target = Output::from_resource(output)
        .and_then(|output| Some((part(&output, region)?, output)))
        .map(|(part, output)| (output, part));
    let size = target.as_ref().map(|(_, [_, _, w, h])| (*w, *h));
    let frame = data_init.init(
        frame,
        FrameData {
            target,
            used: AtomicBool::new(false),
        },
    );
    let Some((w, h)) = size else {
        frame.failed();
        return;
    };
    frame.buffer(wl_shm::Format::Xrgb8888, w, h, w * 4);
    if frame.version() >= 3 {
        frame.buffer_done();
    }
}

/// The part of an output in physical pixels: all of it, or a region in its
/// logical coordinates, cut to the output. `None` when nothing is left.
fn part(output: &Output, region: Option<[i32; 4]>) -> Option<[u32; 4]> {
    let size = render::physical_size(output);
    let (w, h) = (f64::from(size.w), f64::from(size.h));
    let Some([x, y, rw, rh]) = region else {
        return Some([0, 0, size.w.try_into().ok()?, size.h.try_into().ok()?]);
    };
    let scale = output.current_scale().fractional_scale();
    let clamp = |v: f64, max: f64| v.clamp(0.0, max);
    let left = clamp((f64::from(x) * scale).floor(), w);
    let top = clamp((f64::from(y) * scale).floor(), h);
    let right = clamp((f64::from(x.saturating_add(rw)) * scale).ceil(), w);
    let bottom = clamp((f64::from(y.saturating_add(rh)) * scale).ceil(), h);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped
    let px = |v: f64| v as u32;
    (right > left && bottom > top).then(|| [px(left), px(top), px(right - left), px(bottom - top)])
}

impl Dispatch2<ZwlrScreencopyFrameV1, State> for FrameData {
    fn request(
        &self,
        state: &mut State,
        _client: &Client,
        resource: &ZwlrScreencopyFrameV1,
        request: frame::Request,
        _display: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        let (buffer, damage) = match request {
            frame::Request::Copy { buffer } => (buffer, false),
            frame::Request::CopyWithDamage { buffer } => (buffer, true),
            _ => return,
        };
        if self.used.swap(true, Ordering::Relaxed) {
            resource.post_error(frame::Error::AlreadyUsed, "the frame was copied already");
            return;
        }
        let Some((output, part)) = self.target.clone() else {
            resource.failed();
            return;
        };
        let fits = shm::buffer_data(&buffer).is_some_and(|data| {
            [u32::try_from(data.width), u32::try_from(data.height)] == [Ok(part[2]), Ok(part[3])]
                && shm::supported(data.format)
                && data.stride >= data.width * 4
        });
        if !fits {
            resource.post_error(
                frame::Error::InvalidBuffer,
                "the buffer is no shm buffer in the offered size and format",
            );
            return;
        }
        let copy = Copy {
            frame: resource.clone(),
            buffer,
            part,
            damage,
            output,
        };
        if state.waiting_frames(copy.client().as_ref()) >= copies::MAX_WAITING {
            copy.frame.failed();
            return;
        }
        state.copies.wlr.push(copy);
        state.arm_copies();
    }
}

/// A copy that a frame asked for; it waits for the next tick.
pub struct Copy {
    frame: ZwlrScreencopyFrameV1,
    buffer: WlBuffer,
    output: Output,
    part: [u32; 4],
    damage: bool,
}

impl Copy {
    /// The client of the frame.
    pub fn client(&self) -> Option<ClientId> {
        self.frame.client().map(|c| c.id())
    }

    /// Writes the picture of the output into the buffer and tells the
    /// client, or fails the frame.
    pub fn serve(self, state: &mut State, pictures: &mut Pictures) {
        if !self.frame.is_alive() {
            return;
        }
        if let Err(err) = self.write(state, pictures) {
            tracing::warn!(%err, "a screencopy frame failed");
            self.frame.failed();
            return;
        }
        self.frame.flags(frame::Flags::empty());
        if self.damage {
            self.frame.damage(0, 0, self.part[2], self.part[3]);
        }
        let now: Duration = state.clock.now().into();
        let secs = now.as_secs();
        #[allow(clippy::cast_possible_truncation)] // split into two halves
        self.frame
            .ready((secs >> 32) as u32, secs as u32, now.subsec_nanos());
    }

    /// Writes the part of the output into the buffer, unless the session is
    /// locked.
    fn write(&self, state: &mut State, pictures: &mut Pictures) -> Result<(), String> {
        if state.lock.is_some() {
            return Err("the session is locked".into());
        }
        let pixels = state.output_picture(pictures, &self.output)?;
        let part = pixels
            .crop(self.part)
            .ok_or("the output changed its size")?;
        shm::write(&self.buffer, part.width, part.height, &part.rgba)
    }
}

#[cfg(test)]
mod tests {
    use smithay::output::{Mode, PhysicalProperties, Scale, Subpixel};

    use super::*;

    fn output(scale: f64) -> Output {
        let output = Output::new(
            "test".into(),
            PhysicalProperties {
                size: (0, 0).into(),
                subpixel: Subpixel::Unknown,
                make: String::new(),
                model: String::new(),
                serial_number: String::new(),
            },
        );
        let mode = Mode {
            size: (200, 100).into(),
            refresh: 60_000,
        };
        output.change_current_state(Some(mode), None, Some(Scale::Fractional(scale)), None);
        output
    }

    #[test]
    fn a_region_turns_into_pixels_inside_the_output() {
        assert_eq!(part(&output(1.0), None), Some([0, 0, 200, 100]));
        assert_eq!(
            part(&output(2.0), Some([10, 10, 20, 20])),
            Some([20, 20, 40, 40])
        );
        assert_eq!(
            part(&output(1.0), Some([150, 50, 100, 100])),
            Some([150, 50, 50, 50])
        );
        assert_eq!(part(&output(1.0), Some([300, 0, 10, 10])), None);
    }
}
