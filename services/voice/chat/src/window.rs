//! The layer surface of the field: where it sits, what it draws and what
//! its keyboard, input method and clipboard do to the draft. The surface
//! is on layer `top` with the keyboard on demand. It takes the keyboard
//! with the activation token of the act that opened the chat, or when the
//! person clicks it, and never otherwise.

use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
    os::fd::{AsFd, OwnedFd},
    sync::Arc,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::{MemfdFlags, memfd_create},
};
use serde::Deserialize;
use serde_json::json;
use wayland_client::{
    EventQueue, Proxy, QueueHandle,
    backend::WaylandError,
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::WlCompositor,
        wl_data_device::WlDataDevice,
        wl_data_device_manager::WlDataDeviceManager,
        wl_data_offer::WlDataOffer,
        wl_keyboard::WlKeyboard,
        wl_output::WlOutput,
        wl_seat::WlSeat,
        wl_shm::{self, WlShm},
        wl_surface::WlSurface,
    },
};
use wayland_protocols::{
    wp::text_input::zv3::client::{
        zwp_text_input_manager_v3::ZwpTextInputManagerV3,
        zwp_text_input_v3::{ChangeCause, ContentHint, ContentPurpose, ZwpTextInputV3},
    },
    xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};
use xkbcommon::xkb;

use crate::{
    draw::{Look, Painter},
    host::{Msg, Request, Wake},
    keys::{self, Decoded, Mods},
    paste,
    session::{Effect, Key, Session},
};

/// The width of the field when a place names none.
const DEFAULT_WIDTH: u32 = 400;
/// The narrowest and the widest field.
const MIN_WIDTH: u32 = 1;
const MAX_WIDTH: u32 = 1600;
/// The most bytes of the surrounding text that the input method gets.
const SURROUNDING: usize = 4000;

/// Where the field sits and how it looks.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Place {
    /// The output by its name; any output when missing.
    #[serde(default)]
    pub output: Option<String>,
    /// The edges that the field sticks to: `top`, `bottom`, `left`,
    /// `right`, or two of them joined by `_`, such as `bottom_right`.
    pub anchor: String,
    /// The distance to the edges: top, right, bottom, left.
    #[serde(default)]
    pub margin: [i32; 4],
    /// The width in pixels.
    #[serde(default = "default_width")]
    pub width: u32,
    /// The activation token of the act that opened the chat.
    #[serde(default)]
    pub token: Option<String>,
    /// The colors.
    #[serde(default)]
    pub look: Look,
    /// The scale of the output: the buffer has this many pixels for each
    /// logical pixel, 1 to 4.
    #[serde(default = "default_scale")]
    pub scale: u32,
}

fn default_scale() -> u32 {
    1
}

fn default_width() -> u32 {
    DEFAULT_WIDTH
}

impl Place {
    /// The place with a width that a surface can have.
    #[must_use]
    pub fn clamped(mut self) -> Self {
        self.width = self.width.clamp(MIN_WIDTH, MAX_WIDTH);
        self.scale = self.scale.clamp(1, 4);
        self
    }
}

/// The anchor of a place: the edges that its name lists.
#[must_use]
pub fn anchor_of(name: &str) -> Anchor {
    name.split('_')
        .fold(Anchor::empty(), |anchor, edge| match edge {
            "top" => anchor | Anchor::Top,
            "bottom" => anchor | Anchor::Bottom,
            "left" => anchor | Anchor::Left,
            "right" => anchor | Anchor::Right,
            _ => anchor,
        })
}

/// The globals of the compositor that the field uses.
#[derive(Default)]
pub struct Globals {
    pub compositor: Option<WlCompositor>,
    pub shm: Option<WlShm>,
    pub layer_shell: Option<ZwlrLayerShellV1>,
    pub seat: Option<WlSeat>,
    pub activation: Option<XdgActivationV1>,
    pub text_manager: Option<ZwpTextInputManagerV3>,
    pub data_manager: Option<WlDataDeviceManager>,
    pub keyboard: Option<WlKeyboard>,
    pub text_input: Option<ZwpTextInputV3>,
    pub data_device: Option<WlDataDevice>,
}

/// The surface while the field shows.
struct Shown {
    surface: WlSurface,
    layer: ZwlrLayerSurfaceV1,
    output: Option<String>,
    configured: bool,
    size: [u32; 2],
}

/// The key that repeats.
struct Repeat {
    code: u32,
    key: Key,
    next: Instant,
}

/// What the input method sent since its last `done`.
#[derive(Default)]
pub struct Pending {
    pub commit: Option<String>,
    pub preedit: Option<String>,
    pub delete: Option<(u32, u32)>,
}

/// The window of the field.
pub struct Window {
    pub qh: QueueHandle<Self>,
    pub globals: Globals,
    pub outputs: Vec<(WlOutput, Option<String>)>,
    pub xkb: Option<xkb::State>,
    pub pending: Pending,
    pub selection: Option<WlDataOffer>,
    pub focused: bool,
    pub running: bool,
    pub rate: (i32, i32),
    shown: Option<Shown>,
    place: Option<Place>,
    token: Option<String>,
    session: Session,
    painter: Painter,
    repeat: Option<Repeat>,
    requests: Sender<Request>,
    wake: Arc<Wake>,
}

impl Window {
    /// A window with no surface yet.
    #[must_use]
    pub fn new(
        qh: QueueHandle<Self>,
        session: Session,
        requests: Sender<Request>,
        wake: Arc<Wake>,
    ) -> Self {
        Self {
            qh,
            globals: Globals::default(),
            outputs: Vec::new(),
            xkb: None,
            pending: Pending::default(),
            selection: None,
            focused: false,
            running: true,
            rate: (25, 400),
            shown: None,
            place: None,
            token: None,
            session,
            painter: Painter::new(),
            repeat: None,
            requests,
            wake,
        }
    }

    /// Serves the compositor and the messages of the connection until the
    /// program ends.
    ///
    /// # Errors
    ///
    /// The Wayland connection fails.
    pub fn run(
        &mut self,
        queue: &mut EventQueue<Self>,
        msgs: &Receiver<Msg>,
        wake: OwnedFd,
    ) -> Result<(), String> {
        let mut wake = File::from(wake);
        while self.running {
            while let Ok(msg) = msgs.try_recv() {
                self.msg(msg);
            }
            queue.flush().map_err(|e| e.to_string())?;
            queue.dispatch_pending(self).map_err(|e| e.to_string())?;
            self.repeat_due();
            queue.flush().map_err(|e| e.to_string())?;
            let Some(guard) = queue.prepare_read() else {
                continue;
            };
            let timeout = self.repeat_wait();
            let ready = {
                let display = guard.connection_fd();
                let mut fds = [
                    PollFd::new(&display, PollFlags::IN),
                    PollFd::new(&wake, PollFlags::IN),
                ];
                let timeout = timeout.and_then(|t| Timespec::try_from(t).ok());
                poll(&mut fds, timeout.as_ref()).unwrap_or(0);
                [fds[0].revents(), fds[1].revents()]
            };
            if ready[0].contains(PollFlags::IN) {
                match guard.read() {
                    Err(WaylandError::Io(e)) if e.kind() == ErrorKind::WouldBlock => {}
                    other => {
                        other.map_err(|e| e.to_string())?;
                    }
                }
            } else {
                drop(guard);
            }
            if ready[1].contains(PollFlags::IN) {
                let _ = wake.read(&mut [0_u8; 64]);
            }
        }
        Ok(())
    }

    fn msg(&mut self, msg: Msg) {
        match msg {
            Msg::Place(place) => self.apply_place(*place),
            Msg::Question(waiting) => self.session.set_waiting(waiting),
            Msg::Keys(keys) => self.session.set_keys(keys),
            Msg::SayFailed(text) => self.session.say_failed(&text),
            Msg::PickFirst(text) => self.session.pick_first(&text),
            Msg::Clear => self.session.clear(),
            Msg::Paste(text) => {
                let effect = self.session.key(Key::Text(text));
                self.effect(effect);
            }
            Msg::Hide => {
                self.token = None;
                self.destroy_surface();
            }
            Msg::Quit => self.running = false,
        }
        self.changed();
    }

    fn output_named(&self, name: &str) -> Option<&WlOutput> {
        (self.outputs.iter())
            .find(|(_, n)| n.as_deref() == Some(name))
            .map(|(o, _)| o)
    }

    fn apply_place(&mut self, place: Place) {
        let moved = (self.shown.as_ref()).is_some_and(|s| s.output != place.output);
        if moved {
            self.destroy_surface();
        }
        if self.shown.is_none() {
            self.create_surface(&place);
        } else if let Some(shown) = &self.shown {
            shown.layer.set_anchor(anchor_of(&place.anchor));
            let [top, right, bottom, left] = place.margin;
            shown.layer.set_margin(top, right, bottom, left);
        }
        if place.token.is_some() {
            self.token.clone_from(&place.token);
        }
        self.place = Some(place);
    }

    fn create_surface(&mut self, place: &Place) {
        let (Some(compositor), Some(shell)) = (&self.globals.compositor, &self.globals.layer_shell)
        else {
            eprintln!("chora-chat: the compositor has no layer shell");
            self.running = false;
            return;
        };
        let surface = compositor.create_surface(&self.qh, ());
        let output = place.output.as_deref().and_then(|n| self.output_named(n));
        let layer = shell.get_layer_surface(
            &surface,
            output,
            Layer::Top,
            "chora-chat".into(),
            &self.qh,
            (),
        );
        layer.set_anchor(anchor_of(&place.anchor));
        let [top, right, bottom, left] = place.margin;
        layer.set_margin(top, right, bottom, left);
        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer.set_exclusive_zone(0);
        let layout = self.painter.layout(&self.session, place.width, &place.look);
        let size = [place.width, layout.height()];
        layer.set_size(size[0], size[1]);
        surface.commit();
        self.shown = Some(Shown {
            surface,
            layer,
            output: place.output.clone(),
            configured: false,
            size,
        });
    }

    fn destroy_surface(&mut self) {
        if let Some(shown) = self.shown.take() {
            shown.layer.destroy();
            shown.surface.destroy();
        }
        self.repeat = None;
        self.set_focused(false);
    }

    /// The compositor configured the surface.
    pub fn configured(&mut self, serial: u32) {
        let mut first = false;
        if let Some(shown) = &mut self.shown {
            shown.layer.ack_configure(serial);
            first = !std::mem::replace(&mut shown.configured, true);
        }
        self.changed();
        if first {
            self.emit("shown", json!({}));
        }
    }

    /// The compositor closed the surface, for example with its output.
    pub fn closed_by_compositor(&mut self) {
        self.close();
    }

    /// Esc: the field goes and the draft stays.
    fn close(&mut self) {
        self.token = None;
        self.destroy_surface();
        self.emit("closed", json!({}));
    }

    fn emit(&self, event: &'static str, data: serde_json::Value) {
        let _ = self.requests.send(Request::Emit(event, data));
    }

    /// The keyboard came or went.
    pub fn set_focused(&mut self, focused: bool) {
        if self.focused == focused {
            return;
        }
        self.focused = focused;
        self.emit("keyboard", json!({"on": focused}));
        if let Some(input) = &self.globals.text_input {
            if focused {
                input.enable();
            } else {
                input.disable();
            }
            input.commit();
        }
        if !focused {
            self.repeat = None;
        }
        self.changed();
    }

    /// The draft or the surface changed: draw again and tell the input method.
    pub fn changed(&mut self) {
        self.draw();
        self.sync_ime();
    }

    fn draw(&mut self) {
        let Some(place) = &self.place else {
            return;
        };
        let layout = self.painter.layout(&self.session, place.width, &place.look);
        let size = [place.width, layout.height()];
        let scale = place.scale;
        let pixels = self
            .painter
            .paint(&layout, &place.look, (size[0], scale), self.focused);
        let token = self.token.take();
        let Some(shown) = self.shown.as_mut().filter(|s| s.configured) else {
            self.token = token;
            return;
        };
        if shown.size != size {
            shown.layer.set_size(size[0], size[1]);
            shown.size = size;
        }
        shown
            .surface
            .set_buffer_scale(i32::try_from(scale).unwrap_or(1));
        if let Some(buffer) = buffer(&self.globals, &self.qh, &pixels, size.map(|v| v * scale)) {
            let [w, h] = size.map(|v| i32::try_from(v * scale).unwrap_or(i32::MAX));
            shown.surface.attach(Some(&buffer), 0, 0);
            shown.surface.damage_buffer(0, 0, w, h);
        }
        shown.surface.commit();
        if let (Some(token), Some(activation)) = (token, &self.globals.activation) {
            activation.activate(token, &shown.surface);
        }
    }

    fn sync_ime(&self) {
        let (Some(input), true) = (&self.globals.text_input, self.focused) else {
            return;
        };
        let cursor = self.session.cursor();
        let (from, text) = surrounding(self.session.text(), cursor);
        let at = i32::try_from(cursor - from).unwrap_or(0);
        input.set_surrounding_text(text.to_owned(), at, at);
        input.set_text_change_cause(ChangeCause::Other);
        input.set_content_type(ContentHint::None, ContentPurpose::Normal);
        input.set_cursor_rectangle(8, 8, 1, 20);
        input.commit();
    }

    /// The input method finished a batch of changes.
    pub fn ime_done(&mut self) {
        let pending = std::mem::take(&mut self.pending);
        if let Some((before, after)) = pending.delete {
            self.session
                .delete_surrounding(before as usize, after as usize);
        }
        if let Some(text) = pending.commit {
            self.session.commit(&text);
        }
        self.session
            .set_preedit(&pending.preedit.unwrap_or_default());
        self.changed();
    }

    /// A key went down or up, with the keycode of evdev.
    pub fn key(&mut self, code: u32, pressed: bool) {
        if !pressed {
            if self.repeat.as_ref().is_some_and(|r| r.code == code) {
                self.repeat = None;
            }
            return;
        }
        let Some(key) = self.decode(code) else {
            return;
        };
        if keys::repeats(&key) && self.xkb_repeats(code) {
            let (rate, delay) = self.rate;
            if rate > 0 {
                let next =
                    Instant::now() + Duration::from_millis(u64::try_from(delay).unwrap_or(0));
                self.repeat = Some(Repeat {
                    code,
                    key: key.clone(),
                    next,
                });
            }
        }
        self.press(key);
    }

    fn xkb_repeats(&self, code: u32) -> bool {
        (self.xkb.as_ref()).is_some_and(|s| s.get_keymap().key_repeats(xkb::Keycode::new(code + 8)))
    }

    fn decode(&mut self, code: u32) -> Option<Key> {
        let state = self.xkb.as_ref()?;
        let keycode = xkb::Keycode::new(code + 8);
        let active = |name: &str| state.mod_name_is_active(name, xkb::STATE_MODS_EFFECTIVE);
        let mods = Mods {
            ctrl: active(xkb::MOD_NAME_CTRL),
            shift: active(xkb::MOD_NAME_SHIFT),
            alt: active(xkb::MOD_NAME_ALT),
        };
        let sym = state.key_get_one_sym(keycode).raw();
        match keys::decode(sym, mods, &state.key_get_utf8(keycode)) {
            Decoded::Key(key) => Some(key),
            Decoded::Paste => {
                self.paste();
                None
            }
            Decoded::Stop => {
                self.emit("stop", json!({}));
                None
            }
            Decoded::Ignore => None,
        }
    }

    /// Asks the selection for its text; a thread reads it, so that a slow
    /// source does not stop the window.
    fn paste(&mut self) {
        let Some(offer) = &self.selection else {
            return;
        };
        let mimes = (offer.data::<crate::events::Mimes>())
            .and_then(|m| m.0.lock().ok().map(|list| list.clone()))
            .unwrap_or_default();
        let (Some(mime), Ok((read, write))) = (
            paste::text_type(&mimes),
            rustix::pipe::pipe_with(rustix::pipe::PipeFlags::CLOEXEC),
        ) else {
            return;
        };
        offer.receive(mime.to_owned(), write.as_fd());
        drop(write);
        let wake = Arc::clone(&self.wake);
        std::thread::spawn(move || wake.tell(Msg::Paste(paste::read_text(read))));
    }

    fn press(&mut self, key: Key) {
        let effect = self.session.key(key);
        self.effect(effect);
        self.changed();
    }

    fn effect(&mut self, effect: Option<Effect>) {
        match effect {
            Some(Effect::Say(text)) => {
                let _ = self.requests.send(Request::Say(text));
            }
            Some(Effect::Close) => self.close(),
            None => {}
        }
    }

    fn repeat_wait(&self) -> Option<Duration> {
        let repeat = self.repeat.as_ref()?;
        Some(repeat.next.saturating_duration_since(Instant::now()))
    }

    fn repeat_due(&mut self) {
        let (rate, _) = self.rate;
        let Some(repeat) = self.repeat.as_mut().filter(|r| r.next <= Instant::now()) else {
            return;
        };
        let key = repeat.key.clone();
        let interval = Duration::from_millis(1000 / u64::try_from(rate.max(1)).unwrap_or(1));
        repeat.next = Instant::now() + interval;
        self.press(key);
    }
}

/// The text around the cursor that fits the limit of the input method,
/// and where it starts in the text.
fn surrounding(text: &str, cursor: usize) -> (usize, &str) {
    let mut from = cursor.saturating_sub(SURROUNDING / 2);
    while !text.is_char_boundary(from) {
        from += 1;
    }
    let mut to = (from + SURROUNDING).min(text.len());
    while !text.is_char_boundary(to) {
        to -= 1;
    }
    (from, &text[from..to])
}

/// A shm buffer with the pixels, one pool per buffer.
fn buffer(
    globals: &Globals,
    qh: &QueueHandle<Window>,
    pixels: &[u8],
    [w, h]: [u32; 2],
) -> Option<WlBuffer> {
    let shm = globals.shm.as_ref()?;
    let fd = memfd_create("chora-chat", MemfdFlags::CLOEXEC).ok()?;
    let mut file = File::from(fd);
    file.write_all(pixels).ok()?;
    let [w, h] = [i32::try_from(w).ok()?, i32::try_from(h).ok()?];
    let pool = shm.create_pool(file.as_fd(), w * h * 4, qh, ());
    let buffer = pool.create_buffer(0, w, h, w * 4, wl_shm::Format::Argb8888, qh, ());
    pool.destroy();
    Some(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_width_is_clamped_to_what_a_surface_can_be() {
        let place = |width| {
            serde_json::from_value::<Place>(json!({"anchor": "top", "width": width}))
                .expect("a place")
                .clamped()
                .width
        };
        assert_eq!((place(0), place(400), place(100_000)), (1, 400, 1600));
    }

    #[test]
    fn an_anchor_names_its_edges() {
        assert_eq!(anchor_of("bottom"), Anchor::Bottom);
        assert_eq!(anchor_of("top_right"), Anchor::Top | Anchor::Right);
        assert_eq!(anchor_of("middle"), Anchor::empty());
    }

    #[test]
    fn a_place_refuses_unknown_keys_and_has_defaults() {
        let place: Place = serde_json::from_value(json!({"anchor": "bottom"})).expect("a place");
        assert_eq!((place.width, place.margin), (400, [0; 4]));
        let wrong = serde_json::from_value::<Place>(json!({"anchor": "bottom", "size": 3}));
        assert!(wrong.is_err());
    }

    #[test]
    fn the_surrounding_text_stays_on_char_boundaries() {
        let text = "ä".repeat(5000);
        let (from, around) = surrounding(&text, 6000);
        assert!(around.len() <= SURROUNDING && text.is_char_boundary(from));
    }
}
