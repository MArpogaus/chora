//! The events of the compositor for the window of the field.

use std::sync::Mutex;

use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum, delegate_noop, event_created_child,
    protocol::{
        wl_buffer::{self, WlBuffer},
        wl_compositor::WlCompositor,
        wl_data_device::{self, WlDataDevice},
        wl_data_device_manager::WlDataDeviceManager,
        wl_data_offer::{self, WlDataOffer},
        wl_keyboard::{self, KeyState, KeymapFormat, WlKeyboard},
        wl_output::{self, WlOutput},
        wl_registry,
        wl_seat::{self, Capability, WlSeat},
        wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::{
    wp::text_input::zv3::client::{
        zwp_text_input_manager_v3::ZwpTextInputManagerV3,
        zwp_text_input_v3::{self, ZwpTextInputV3},
    },
    xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
};
use xkbcommon::xkb;

use crate::window::Window;

/// The mime types that an offer lists.
#[derive(Default)]
pub struct Mimes(pub Mutex<Vec<String>>);

delegate_noop!(Window: ignore WlCompositor);
delegate_noop!(Window: ignore WlShm);
delegate_noop!(Window: ignore WlShmPool);
delegate_noop!(Window: ignore WlSurface);
delegate_noop!(Window: ignore ZwlrLayerShellV1);
delegate_noop!(Window: ignore XdgActivationV1);
delegate_noop!(Window: ignore ZwpTextInputManagerV3);
delegate_noop!(Window: ignore WlDataDeviceManager);

impl Dispatch<wl_registry::WlRegistry, ()> for Window {
    fn event(
        window: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };
        let globals = &mut window.globals;
        match interface.as_str() {
            "wl_compositor" => globals.compositor = Some(registry.bind(name, 4, qh, ())),
            "wl_shm" => globals.shm = Some(registry.bind(name, 1, qh, ())),
            "zwlr_layer_shell_v1" => globals.layer_shell = Some(registry.bind(name, 1, qh, ())),
            "wl_seat" => globals.seat = Some(registry.bind(name, version.min(5), qh, ())),
            "xdg_activation_v1" => globals.activation = Some(registry.bind(name, 1, qh, ())),
            "zwp_text_input_manager_v3" => {
                globals.text_manager = Some(registry.bind(name, 1, qh, ()));
            }
            "wl_data_device_manager" => {
                globals.data_manager = Some(registry.bind(name, version.min(3), qh, ()));
            }
            "wl_output" => {
                let output = registry.bind(name, version.min(4), qh, ());
                window.outputs.push((output, None));
            }
            _ => {}
        }
    }
}

impl Dispatch<WlOutput, ()> for Window {
    fn event(
        window: &mut Self,
        output: &WlOutput,
        event: wl_output::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event
            && let Some(entry) = window.outputs.iter_mut().find(|(o, _)| o == output)
        {
            entry.1 = Some(name);
        }
    }
}

impl Dispatch<WlSeat, ()> for Window {
    fn event(
        window: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        (): &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        else {
            return;
        };
        if capabilities.contains(Capability::Keyboard) && window.globals.keyboard.is_none() {
            window.globals.keyboard = Some(seat.get_keyboard(qh, ()));
        }
    }
}

impl Dispatch<WlKeyboard, ()> for Window {
    fn event(
        window: &mut Self,
        _: &WlKeyboard,
        event: wl_keyboard::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Keymap {
                format: WEnum::Value(KeymapFormat::XkbV1),
                fd,
                size,
            } => window.keymap(fd, size),
            wl_keyboard::Event::Enter { .. } => window.set_focused(true),
            wl_keyboard::Event::Leave { .. } => window.set_focused(false),
            wl_keyboard::Event::Key { key, state, .. } => {
                window.key(key, state == WEnum::Value(KeyState::Pressed));
            }
            wl_keyboard::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                if let Some(state) = &mut window.xkb {
                    state.update_mask(mods_depressed, mods_latched, mods_locked, 0, 0, group);
                }
            }
            wl_keyboard::Event::RepeatInfo { rate, delay } => window.rate = (rate, delay),
            _ => {}
        }
    }
}

impl Window {
    fn keymap(&mut self, fd: std::os::fd::OwnedFd, size: u32) {
        use std::io::{Read, Seek, SeekFrom};
        let mut text = String::new();
        let mut file = std::fs::File::from(fd);
        if file.seek(SeekFrom::Start(0)).is_err()
            || file
                .take(u64::from(size))
                .read_to_string(&mut text)
                .is_err()
        {
            return;
        }
        let text = text.trim_end_matches('\0').to_owned();
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_string(
            &context,
            text,
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );
        self.xkb = keymap.map(|keymap| xkb::State::new(&keymap));
    }

    /// The seat is known: the input method and the clipboard attach to it.
    pub fn attach_seat(&mut self, qh: &QueueHandle<Self>) {
        let globals = &mut self.globals;
        if let (Some(seat), Some(manager)) = (&globals.seat, &globals.text_manager) {
            globals.text_input = Some(manager.get_text_input(seat, qh, ()));
        }
        if let (Some(seat), Some(manager)) = (&globals.seat, &globals.data_manager) {
            globals.data_device = Some(manager.get_data_device(seat, qh, ()));
        }
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for Window {
    fn event(
        window: &mut Self,
        _: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, .. } => window.configured(serial),
            zwlr_layer_surface_v1::Event::Closed => window.closed_by_compositor(),
            _ => {}
        }
    }
}

impl Dispatch<WlBuffer, ()> for Window {
    fn event(
        _: &mut Self,
        buffer: &WlBuffer,
        event: wl_buffer::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            buffer.destroy();
        }
    }
}

impl Dispatch<ZwpTextInputV3, ()> for Window {
    fn event(
        window: &mut Self,
        _: &ZwpTextInputV3,
        event: zwp_text_input_v3::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_text_input_v3::Event::PreeditString { text, .. } => {
                window.pending.preedit = Some(text.unwrap_or_default());
            }
            zwp_text_input_v3::Event::CommitString { text } => {
                window.pending.commit = text;
            }
            zwp_text_input_v3::Event::DeleteSurroundingText {
                before_length,
                after_length,
            } => window.pending.delete = Some((before_length, after_length)),
            zwp_text_input_v3::Event::Done { .. } => window.ime_done(),
            _ => {}
        }
    }
}

impl Dispatch<WlDataDevice, ()> for Window {
    fn event(
        window: &mut Self,
        _: &WlDataDevice,
        event: wl_data_device::Event,
        (): &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_data_device::Event::Selection { id } = event
            && let Some(old) = std::mem::replace(&mut window.selection, id)
        {
            old.destroy();
        }
    }

    event_created_child!(Window, WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (WlDataOffer, Mimes::default()),
    ]);
}

impl Dispatch<WlDataOffer, Mimes> for Window {
    fn event(
        _: &mut Self,
        _: &WlDataOffer,
        event: wl_data_offer::Event,
        mimes: &Mimes,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_data_offer::Event::Offer { mime_type } = event
            && let Ok(mut list) = mimes.0.lock()
        {
            list.push(mime_type);
        }
    }
}
