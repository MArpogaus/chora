//! Writes pictures into the shared memory buffers of clients, for the
//! screencopy protocols. Smithay hands out the memory of a buffer as a raw
//! pointer, so this module is the second place with `unsafe` code, after
//! `gl`.

#![allow(unsafe_code)]

use smithay::{
    reexports::wayland_server::protocol::{wl_buffer::WlBuffer, wl_shm},
    wayland::shm::{BufferData, with_buffer_contents, with_buffer_contents_mut},
};

/// The size and the format of an shm buffer, if it is one.
pub fn buffer_data(buffer: &WlBuffer) -> Option<BufferData> {
    with_buffer_contents(buffer, |_, _, data| data).ok()
}

/// Whether a format takes the pixels of a picture: 32 bits per pixel, blue
/// in the lowest byte.
pub fn supported(format: wl_shm::Format) -> bool {
    matches!(format, wl_shm::Format::Argb8888 | wl_shm::Format::Xrgb8888)
}

/// Writes premultiplied RGBA pixels, the top row first, into an shm buffer
/// of the same size in a [`supported`] format.
pub fn write(buffer: &WlBuffer, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    with_buffer_contents_mut(buffer, |ptr, len, data| {
        let (w, h) = (width as usize, height as usize);
        let offset = usize::try_from(data.offset).map_err(|_| "a negative offset")?;
        let stride = usize::try_from(data.stride).map_err(|_| "a negative stride")?;
        let fits = (data.width, data.height)
            == (
                width.try_into().unwrap_or(-1),
                height.try_into().unwrap_or(-1),
            )
            && supported(data.format)
            && stride >= w * 4
            && rgba.len() >= w * h * 4
            && h.checked_sub(1)
                .and_then(|rows| {
                    offset
                        .checked_add(rows.checked_mul(stride)?)?
                        .checked_add(w * 4)
                })
                .is_some_and(|end| end <= len);
        if !fits {
            return Err("the buffer does not fit the picture".to_string());
        }
        // SAFETY: Smithay maps `len` bytes of the pool at `ptr` for the
        // duration of this closure and catches a SIGBUS of a truncated
        // file. The client shares the memory and may write it at the same
        // time; that garbles at most its own picture.
        let memory = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
        for (row, line) in rgba.chunks_exact(w * 4).take(h).enumerate() {
            let start = offset + row * stride;
            let target = &mut memory[start..start + w * 4];
            for (out, px) in target
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(line.as_chunks::<4>().0)
            {
                *out = [px[2], px[1], px[0], px[3]];
            }
        }
        Ok(())
    })
    .map_err(|e| format!("{e:?}"))?
}
