//! The keymap of the host session, for chora as a window: winit
//! gives raw key codes, so chora asks the host for its keymap on a second
//! Wayland connection, and translates the codes with it.

use std::{fs::File, os::unix::fs::FileExt};

use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum,
    protocol::{wl_keyboard, wl_registry, wl_seat},
};

/// The largest keymap that chora takes; real ones have about 60 KiB.
const MAX_KEYMAP: u32 = 1 << 20;
/// The round trips to the keymap: the seat, the keyboard, the keymap.
const ROUND_TRIPS: usize = 3;

/// The keymap of the first seat of the host, as XKB text. `None` without a
/// host, a seat with a keyboard, or a keymap in XKB format.
pub fn fetch() -> Option<String> {
    let connection = Connection::connect_to_env().ok()?;
    let mut queue = connection.new_event_queue();
    connection.display().get_registry(&queue.handle(), ());
    let mut host = Host::default();
    for _ in 0..ROUND_TRIPS {
        queue.roundtrip(&mut host).ok()?;
        if host.keymap.is_some() {
            break;
        }
    }
    host.keymap
}

#[derive(Default)]
struct Host {
    seat: Option<wl_seat::WlSeat>,
    keymap: Option<String>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for Host {
    fn event(
        host: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "wl_seat"
            && host.seat.is_none()
        {
            host.seat = Some(registry.bind(name, version.min(5), queue, ()));
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for Host {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        (): &(),
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
            && capabilities.contains(wl_seat::Capability::Keyboard)
        {
            seat.get_keyboard(queue, ());
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for Host {
    fn event(
        host: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_keyboard::Event::Keymap {
            format: WEnum::Value(wl_keyboard::KeymapFormat::XkbV1),
            fd,
            size,
        } = event
            && size <= MAX_KEYMAP
        {
            host.keymap = read(&File::from(fd), size);
        }
    }
}

/// The text of a keymap file. It reads from the start, because the host
/// may share one open file with all clients.
fn read(file: &File, size: u32) -> Option<String> {
    let mut text = vec![0; usize::try_from(size).ok()?];
    file.read_exact_at(&mut text, 0).ok()?;
    let end = text.iter().position(|&b| b == 0).unwrap_or(text.len());
    text.truncate(end);
    String::from_utf8(text).ok()
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn a_keymap_ends_at_its_nul() {
        let path = std::env::temp_dir().join(format!("chora-keymap-{}", std::process::id()));
        File::create(&path)
            .unwrap()
            .write_all(b"xkb_keymap {};\0")
            .unwrap();
        let file = File::open(&path).unwrap();
        assert_eq!(read(&file, 15).as_deref(), Some("xkb_keymap {};"));
        assert_eq!(read(&file, 99), None, "a short file is no keymap");
        std::fs::remove_file(path).unwrap();
    }
}
