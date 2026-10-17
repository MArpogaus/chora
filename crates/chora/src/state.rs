//! The state of the compositor.

use std::{
    collections::HashMap,
    ffi::OsString,
    sync::{Arc, atomic::AtomicU32},
    time::Instant,
};

use chora_scene::{
    Binding, Decoration, Effect, Event, Scene, SurfaceInfo, SurfaceRequest, ToplevelState,
};
use smithay::{
    desktop::{PopupManager, Window},
    input::{
        Seat, SeatState,
        keyboard::{KeyboardHandle, Keycode},
        pointer::CursorImageStatus,
    },
    output::Output,
    reexports::{
        calloop::{Interest, LoopHandle, LoopSignal, Mode, PostAction, channel, generic::Generic},
        wayland_protocols::xdg::{
            decoration::zv1::server::zxdg_toplevel_decoration_v1 as xdg_toplevel_decoration,
            shell::server::xdg_toplevel,
        },
        wayland_server::{
            Display, DisplayHandle, backend::ClientData, protocol::wl_surface::WlSurface,
        },
    },
    utils::{Clock, Logical, Monotonic, Point, SERIAL_COUNTER, Serial},
    wayland::{
        compositor::{CompositorClientState, CompositorState, with_states},
        cursor_shape::CursorShapeManagerState,
        dmabuf::DmabufState,
        foreign_toplevel_list::{ForeignToplevelHandle, ForeignToplevelListState},
        fractional_scale::FractionalScaleManagerState,
        idle_inhibit::IdleInhibitManagerState,
        idle_notify::IdleNotifierState,
        input_method::InputMethodManagerState,
        output::OutputManagerState,
        pointer_constraints::PointerConstraintsState,
        presentation::PresentationState,
        relative_pointer::RelativePointerManagerState,
        selection::{data_device::DataDeviceState, primary_selection::PrimarySelectionState},
        session_lock::SessionLockManagerState,
        shell::{
            wlr_layer::WlrLayerShellState,
            xdg::{
                ToplevelSurface, XdgShellState, XdgToplevelSurfaceData,
                decoration::XdgDecorationState,
            },
        },
        shm::ShmState,
        socket::ListeningSocketSource,
        text_input::TextInputManagerState,
        viewporter::ViewporterState,
        xdg_activation::XdgActivationState,
    },
};

use smithay::wayland::security_context::{SecurityContext, SecurityContextState};

use crate::{
    backend::Backend, copies::Copies, foreign_toplevel::ForeignToplevelManagerState,
    grants::Grants, image_capture::ImageCapture, ipc::Ipc, lock::Lock, methods::Methods,
    output_management::OutputManagement, render::Assets, safety::Watchdog, security::trusted,
    time::Time, transaction::Transactions,
};

/// The drag of a rect or an image node: the node, the button, and the last
/// position that `pointer.moved` told.
#[derive(Clone)]
pub struct NodeGrab {
    pub node: String,
    pub button: u32,
    pub position: [f64; 2],
}

/// A grab of a button that is down: the connection of the grab, the
/// binding, and the button.
pub struct ButtonGrab {
    pub conn: u64,
    pub binding: String,
    pub button: u32,
}

/// A key grab whose key went down while all its modifiers stay down: the
/// connection of the grab and the binding. It ends with one `released`
/// when one of the modifiers goes up.
pub struct HeldGrab {
    pub conn: u64,
    pub binding: Binding,
}

/// The keys whose press did not reach a surface, and the held key grabs.
#[derive(Default)]
pub struct Held {
    /// Keys whose press went to the safety layer, a grab or a node, so that
    /// their release must not reach a surface.
    pub keys: Vec<(Keycode, Suppressed)>,
    /// The key grabs that are held.
    pub grabs: Vec<HeldGrab>,
}

/// Where the release of a key goes whose press did not reach a surface.
pub enum Suppressed {
    /// Nowhere, as after a binding of the safety layer.
    Swallow,
    /// To the owner of the node with the keyboard focus, with the node.
    Node(String, String),
}

/// The shortest time between two refreshes of the pointer while animations
/// run.
const POINTER_REFRESH: std::time::Duration = std::time::Duration::from_millis(8);

/// The state of the compositor, passed to every callback of the event loop.
#[allow(clippy::struct_field_names)] // the names of Smithay's handler traits
pub struct State {
    pub start_time: Instant,
    /// The time to which the animations of the scene advanced.
    animated_at: Instant,
    /// The last refresh of the pointer after a step of the animations.
    pointer_refreshed: Instant,
    /// The clock of presentation times.
    pub clock: Clock<Monotonic>,
    /// The time of the deadlines.
    pub time: Time,
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, State>,
    pub loop_signal: LoopSignal,
    pub socket_name: OsString,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<State>,
    pub data_device_state: DataDeviceState,
    pub primary_selection_state: PrimarySelectionState,
    pub popups: PopupManager,
    pub seat: Seat<State>,

    pub backend: Option<Backend>,
    pub dmabuf_state: DmabufState,
    /// The outputs that are on, in the order of their arrival. Outputs that
    /// a client turned off have a head in `output_management` only.
    pub outputs: Vec<Output>,
    pub output_management: OutputManagement,
    /// The pointer in layout coordinates. Smithay keeps it in scene
    /// coordinates; the two differ under a camera that is not the default.
    pub pointer_layout: Point<f64, Logical>,
    /// The cursor that the client under the pointer asks for.
    pub cursor_status: CursorImageStatus,
    pub layer_shell_state: WlrLayerShellState,
    pub lock_state: SessionLockManagerState,
    /// The lock screen, while the session is locked.
    pub lock: Option<Lock>,
    /// The surface node that plugins focused.
    focus_wanted: Option<String>,
    /// The watchdog of the render times.
    pub watchdog: Watchdog,
    /// The rect or image node of a press and its button, until the button
    /// goes up.
    pub node_grab: Option<NodeGrab>,
    /// The grab of a button with modifiers, until the button goes up.
    pub button_grab: Option<ButtonGrab>,
    /// The node of the last `pointer.entered`.
    pub hovered: Option<String>,
    /// A layer surface that a click focused.
    layer_focus: Option<WlSurface>,
    /// The last configure of each surface node: size and states.
    configured: HashMap<String, (Option<[i32; 2]>, Vec<ToplevelState>)>,
    pub idle_notifier_state: IdleNotifierState<State>,
    /// The surfaces that inhibit idleness.
    pub idle_inhibitors: Vec<WlSurface>,
    pub foreign_toplevel_list_state: ForeignToplevelListState,
    /// The entries of the toplevels in the foreign toplevel list.
    pub foreign_toplevels: HashMap<String, ForeignToplevelHandle>,
    pub foreign_toplevel_manager_state: ForeignToplevelManagerState,
    pub activation_state: XdgActivationState,
    /// Requests of surfaces before their first commit.
    pub early_requests: HashMap<String, Vec<SurfaceRequest>>,

    pub scene: Scene,
    /// The toplevels by the id of their surface node.
    pub windows: HashMap<String, Window>,
    pub assets: Assets,
    next_surface: u64,
    pub ipc: Ipc,
    pub transactions: Transactions,
    /// The grants of the user for `input.inject` and `screen.capture`.
    pub grants: Grants,
    /// The methods that plugins export, and the calls in flight.
    pub methods: Methods,
    /// `ext-image-copy-capture` and its sources.
    pub image_capture: ImageCapture,
    /// The screencopy frames that wait for the next tick.
    pub copies: Copies,
    /// The keys and the key grabs that are down.
    pub held: Held,
}

/// Creates the globals that live as long as the display and whose state
/// nothing reads.
fn create_globals(dh: &DisplayHandle, clock: &Clock<Monotonic>) {
    XdgDecorationState::new::<State>(dh);
    OutputManagerState::new_with_xdg_output::<State>(dh);
    ViewporterState::new::<State>(dh);
    IdleInhibitManagerState::new::<State>(dh);
    TextInputManagerState::new::<State>(dh);
    InputMethodManagerState::new::<State, _>(dh, trusted);
    SecurityContextState::new::<State, _>(dh, trusted);
    FractionalScaleManagerState::new::<State>(dh);
    CursorShapeManagerState::new::<State>(dh);
    RelativePointerManagerState::new::<State>(dh);
    PointerConstraintsState::new::<State>(dh);
    PresentationState::new::<State>(dh, clock.id() as u32);
    crate::screencopy::create_global(dh);
}

/// The assets, with the worker that decodes images for the event loop.
fn new_assets(loop_handle: &LoopHandle<'static, State>) -> Assets {
    let mut assets = Assets::default();
    let (done, decoded) = channel::channel();
    loop_handle
        .insert_source(decoded, |event, (), state| {
            if let channel::Event::Msg(decoded) = event {
                state.decoded(decoded);
            }
        })
        .expect("the event loop takes the channel of the images");
    assets.images.start_worker(done);
    assets
}

/// The one seat, with a keyboard of the default keymap and a pointer.
fn new_seat(dh: &DisplayHandle) -> (SeatState<State>, Seat<State>) {
    let mut seat_state = SeatState::new();
    let mut seat = seat_state.new_wl_seat(dh, "seat0");
    seat.add_keyboard(smithay::input::keyboard::XkbConfig::default(), 200, 25)
        .expect("the default keymap compiles");
    seat.add_pointer();
    (seat_state, seat)
}

impl State {
    pub fn new(
        display: Display<State>,
        loop_handle: LoopHandle<'static, State>,
        loop_signal: LoopSignal,
    ) -> Self {
        let dh = display.handle();
        let (seat_state, seat) = new_seat(&dh);
        add_display(display, &loop_handle);
        let idle_notifier_state = IdleNotifierState::new(&dh, loop_handle.clone());
        let clock = Clock::<Monotonic>::new();
        create_globals(&dh, &clock);
        let toplevel_list = ForeignToplevelListState::new_with_filter::<Self>(&dh, trusted);
        Self {
            start_time: Instant::now(),
            animated_at: Instant::now(),
            pointer_refreshed: Instant::now(),
            clock,
            time: Time::default(),
            compositor_state: CompositorState::new::<Self>(&dh),
            layer_shell_state: WlrLayerShellState::new_with_filter::<Self, _>(&dh, trusted),
            lock_state: SessionLockManagerState::new::<Self, _>(&dh, trusted),
            lock: None,
            xdg_shell_state: XdgShellState::new::<Self>(&dh),
            shm_state: ShmState::new::<Self>(&dh, vec![]),
            data_device_state: DataDeviceState::new::<Self>(&dh),
            primary_selection_state: PrimarySelectionState::new::<Self>(&dh),
            popups: PopupManager::default(),
            seat_state,
            seat,
            display_handle: dh.clone(),
            assets: new_assets(&loop_handle),
            loop_handle,
            loop_signal,
            socket_name: OsString::new(),
            backend: None,
            dmabuf_state: DmabufState::new(),
            outputs: Vec::new(),
            output_management: OutputManagement::new(&dh),
            pointer_layout: Point::default(),
            cursor_status: CursorImageStatus::default_named(),
            focus_wanted: None,
            node_grab: None,
            button_grab: None,
            hovered: None,
            watchdog: Watchdog::from_env(),
            layer_focus: None,
            configured: HashMap::new(),
            idle_notifier_state,
            idle_inhibitors: Vec::new(),
            foreign_toplevel_list_state: toplevel_list,
            foreign_toplevels: HashMap::new(),
            foreign_toplevel_manager_state: ForeignToplevelManagerState::new(&dh),
            activation_state: XdgActivationState::new::<Self>(&dh),
            early_requests: HashMap::new(),
            scene: Scene::default(),
            windows: HashMap::new(),
            next_surface: 1,
            ipc: Ipc::default(),
            transactions: Transactions::default(),
            grants: Grants::default(),
            methods: Methods::default(),
            image_capture: ImageCapture::new(&dh),
            copies: Copies::default(),
            held: Held::default(),
        }
    }

    /// The work after each turn of the event loop: animations, errors of
    /// effects, popups, the focus, and the output to all clients.
    pub fn after_events(&mut self) {
        self.advance_animations();
        self.report_effect_errors();
        self.popups.cleanup();
        // The focus follows the protocols, for example after a popup grab
        // ended.
        self.update_focus();
        self.ipc_flush();
        if let Err(err) = self.display_handle.flush_clients() {
            tracing::warn!(?err, "flushing the wayland clients failed");
        }
    }

    /// Tells the plugins about shaders of effects that failed in the last
    /// frames.
    pub fn report_effect_errors(&mut self) {
        for error in std::mem::take(&mut self.assets.effect_errors) {
            tracing::warn!(%error, "an effect failed");
            self.emit(&Event::EffectFailed { error });
        }
    }

    /// Moves the animations of the scene forward to now, and tells the
    /// plugins which ended.
    pub fn advance_animations(&mut self) {
        let now = Instant::now();
        let last = std::mem::replace(&mut self.animated_at, now);
        if !self.scene.animating() {
            return;
        }
        let ms = now.duration_since(last).as_secs_f64() * 1000.0;
        for node in self.scene.advance(ms) {
            self.emit(&Event::AnimationDone {
                node: Some(node),
                output: None,
            });
        }
        for output in self.scene.advance_cameras(ms) {
            self.emit(&Event::AnimationDone {
                node: None,
                output: Some(output.clone()),
            });
            self.execute(vec![chora_scene::Effect::Camera(output)]);
        }
        // The loop can run far more often than frames show.
        if now - self.pointer_refreshed >= POINTER_REFRESH {
            self.pointer_refreshed = now;
            self.refresh_pointer();
        }
    }

    /// Opens the Wayland socket with a free name.
    pub fn listen(&mut self) -> std::io::Result<()> {
        let socket = ListeningSocketSource::new_auto().map_err(std::io::Error::other)?;
        self.socket_name = socket.socket_name().to_os_string();
        self.loop_handle
            .insert_source(socket, |stream, (), state| {
                if let Err(err) = state
                    .display_handle
                    .insert_client(stream, Arc::new(ClientState::default()))
                {
                    tracing::warn!(?err, "a wayland client failed to connect");
                }
            })
            .map_err(|err| std::io::Error::other(err.to_string()))?;
        Ok(())
    }

    /// Registers a new toplevel. Its surface node follows with the first
    /// commit, so that no client configures it before the initial
    /// configure. Clients cannot create ids of the form `s1`, so the id
    /// stays free until then.
    pub fn add_window(&mut self, window: Window) {
        let id = format!("s{}", self.next_surface);
        self.next_surface += 1;
        self.windows.insert(id, window);
    }

    /// The id of the surface node of a toplevel surface.
    pub fn surface_id(&self, surface: &WlSurface) -> Option<String> {
        self.windows
            .iter()
            .find(|(_, w)| w.toplevel().is_some_and(|t| t.wl_surface() == surface))
            .map(|(id, _)| id.clone())
    }

    /// The app id, the title and the size of a toplevel.
    pub fn surface_info(&self, id: &str) -> SurfaceInfo {
        let Some(toplevel) = self.windows.get(id).and_then(Window::toplevel) else {
            return SurfaceInfo::default();
        };
        let size = self.windows[id].geometry().size;
        let server = toplevel.with_pending_state(|state| {
            state.decoration_mode == Some(xdg_toplevel_decoration::Mode::ServerSide)
        });
        let decoration = Some(if server {
            Decoration::Server
        } else {
            Decoration::Client
        });
        with_states(toplevel.wl_surface(), |states| {
            let data = states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .expect("every toplevel has role data")
                .lock()
                .expect("not poisoned");
            SurfaceInfo {
                app_id: data.app_id.clone(),
                title: data.title.clone(),
                size: Some([size.w, size.h]),
                decoration,
            }
        })
    }

    /// Sends an event to all subscribed clients.
    pub fn emit(&mut self, event: &Event) {
        self.ipc.broadcast(event);
    }

    /// Executes the effects of an applied transaction.
    pub fn execute(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                // The transaction sent its configure events when it started.
                Effect::Configure { .. } => {}
                Effect::Focus(surface) => self.focus_of_scene(surface.as_deref()),
                Effect::Close(surface) => {
                    if let Some(t) = self.windows.get(&surface).and_then(Window::toplevel) {
                        t.send_close();
                    }
                }
                Effect::Detached(surface) => {
                    self.unfocus(&surface);
                    self.emit(&Event::SurfaceDetached { surface });
                }
                Effect::Camera(name) => {
                    for output in self.output_info().into_iter().filter(|o| o.name == name) {
                        self.emit(&Event::OutputChanged { output });
                    }
                }
            }
        }
    }

    /// Sends a configure event and returns its serial, if the state changed.
    pub fn configure(
        &mut self,
        surface: &str,
        size: Option<[i32; 2]>,
        states: &[ToplevelState],
    ) -> Option<Serial> {
        let toplevel = self.windows.get(surface)?.toplevel()?.clone();
        set_pending(&toplevel, size, states);
        self.configured
            .insert(surface.into(), (size, states.to_vec()));
        self.restate_toplevel(surface);
        toplevel.send_pending_configure()
    }

    /// The toplevel states of the last configure of a surface node.
    pub fn configured_states(&self, surface: &str) -> &[ToplevelState] {
        self.configured
            .get(surface)
            .map_or(&[], |(_, states)| states.as_slice())
    }

    /// Forgets the configure of a surface that goes.
    pub fn configured_forget(&mut self, surface: &str) {
        self.configured.remove(surface);
    }

    /// The surface node that has the focus, as plugins set it.
    pub fn focused_surface(&self) -> Option<String> {
        self.focus_wanted.clone()
    }

    /// Focuses a surface node, or nothing. A layer surface that a click
    /// focused gives the keyboard back. Taskbars learn first which surface
    /// lost the focus, because some expect one active toplevel at a time.
    pub fn focus(&mut self, surface: Option<&str>) {
        self.scene.set_focused(surface);
        self.focus_of_scene(surface);
    }

    /// Clears the focus of a surface that goes or becomes detached. A layer
    /// surface that a click focused keeps the keyboard.
    pub fn unfocus(&mut self, surface: &str) {
        if self.focus_wanted.as_deref() != Some(surface) {
            return;
        }
        self.scene.set_focused(None);
        self.focus_wanted = None;
        self.update_focus();
        self.restate_toplevel(surface);
    }

    /// Moves the focus that a transaction set; the scene knows it already.
    fn focus_of_scene(&mut self, surface: Option<&str>) {
        let previous = std::mem::replace(&mut self.focus_wanted, surface.map(Into::into));
        self.layer_focus = None;
        self.update_focus();
        for id in [previous, self.focus_wanted.clone()].into_iter().flatten() {
            self.restate_toplevel(&id);
        }
    }

    /// Gives the keyboard to a layer surface after a click, or takes it
    /// back.
    pub fn set_layer_focus(&mut self, layer: Option<WlSurface>) {
        if self.layer_focus != layer {
            self.layer_focus = layer;
            self.update_focus();
        }
    }

    /// The node of `focus_node` that gets the keys, and its owner: not while
    /// the session is locked or a layer surface takes the keyboard.
    pub fn key_node(&self) -> Option<(String, String)> {
        if self.lock.is_some() || self.exclusive_layer().is_some() {
            return None;
        }
        let owner = self.scene.focused_node_owner()?;
        Some((self.scene.focused_node()?.into(), owner.into()))
    }

    /// Moves the keyboard to the surface that should have it: the lock
    /// screen, a layer surface that takes it exclusively, none while a node
    /// has the focus, a layer surface after a click, or the focused surface
    /// node. A popup grab keeps the keyboard, unless the session locks, a
    /// layer surface takes it or a node gets the focus.
    pub fn update_focus(&mut self) {
        let keyboard = self.seat.get_keyboard().expect("the seat has a keyboard");
        // A clicked layer surface keeps the keyboard only while it accepts it.
        self.layer_focus = self
            .layer_focus
            .take()
            .and_then(|l| self.keyboard_layer(&l));
        if self.lock.is_some() {
            self.scene.unfocus_node();
            let target = self.lock_focus();
            keyboard.unset_grab(self);
            if keyboard.current_focus() != target {
                keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
            }
            return;
        }
        // A launcher ends a popup grab, for example of an open menu.
        let exclusive = self.exclusive_layer();
        let node = self.key_node().is_some();
        if popup_grabbed(&keyboard) {
            if exclusive.is_none() && !node {
                return;
            }
            keyboard.unset_grab(self);
        }
        let window = self
            .focus_wanted
            .as_ref()
            .and_then(|s| self.windows.get(s))
            .and_then(Window::toplevel)
            .map(|t| t.wl_surface().clone());
        let clicked = self.layer_focus.clone();
        let target = if node {
            None
        } else {
            exclusive.or(clicked).or(window)
        };
        if keyboard.current_focus() != target {
            keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
        }
    }
}

/// Whether a popup grab holds the keyboard. The grab of an input method
/// starts without a focus and lets focus changes through, so it does not
/// count: it lasts as long as the input method runs.
pub fn popup_grabbed(keyboard: &KeyboardHandle<State>) -> bool {
    keyboard
        .grab_start_data()
        .is_some_and(|d| d.focus.is_some())
}

fn set_pending(toplevel: &ToplevelSurface, size: Option<[i32; 2]>, states: &[ToplevelState]) {
    toplevel.with_pending_state(|pending| {
        pending.size = size.map(|[w, h]| (w, h).into());
        for (state, xdg) in xdg_states() {
            set_xdg_states(&mut pending.states, xdg, states.contains(&state));
        }
    });
}

fn set_xdg_states(
    pending: &mut smithay::wayland::shell::xdg::ToplevelStateSet,
    xdg: &[xdg_toplevel::State],
    on: bool,
) {
    for s in xdg {
        if on {
            pending.set(*s);
        } else {
            pending.unset(*s);
        }
    }
}

/// The xdg states for each toplevel state of the API.
fn xdg_states() -> [(ToplevelState, &'static [xdg_toplevel::State]); 4] {
    use xdg_toplevel::State as X;
    [
        (ToplevelState::Activated, &[X::Activated]),
        (
            ToplevelState::Tiled,
            &[X::TiledLeft, X::TiledRight, X::TiledTop, X::TiledBottom],
        ),
        (ToplevelState::Maximized, &[X::Maximized]),
        (ToplevelState::Fullscreen, &[X::Fullscreen]),
    ]
}

/// Adds the display to the event loop, so that it dispatches the requests
/// of the clients.
fn add_display(display: Display<State>, loop_handle: &LoopHandle<'static, State>) {
    loop_handle
        .insert_source(
            Generic::new(display, Interest::READ, Mode::Level),
            |_, display, state| {
                // SAFETY: the display stays in this source and is never dropped
                // while the event loop runs.
                #[allow(unsafe_code)]
                unsafe {
                    display.get_mut().dispatch_clients(state)?;
                }
                Ok(PostAction::Continue)
            },
        )
        .expect("the event loop accepts the display");
}

/// Data of a Wayland client.
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    /// The serial of the last key or button press that the client got.
    pub last_press: AtomicU32,
    /// The serial of the last key or button event, press or release, that
    /// the client got.
    pub last_input: AtomicU32,
    /// The sandbox of a client that connected through
    /// `wp-security-context-v1`.
    pub security_context: Option<SecurityContext>,
}

impl ClientData for ClientState {}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use smithay::backend::input::{InputTime, KeyState};
    use wayland_client::Proxy;
    use wayland_protocols::xdg::shell::client::xdg_toplevel::State as Xdg;
    use wayland_protocols_wlr::layer_shell::v1::client::{
        zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::KeyboardInteractivity,
    };

    use crate::testing::{
        Client, Core,
        wayland::{Toplevel, Wayland},
    };

    /// The protocol id of the surface with the keyboard focus.
    fn keyboard(core: &Core) -> Option<u32> {
        let keyboard = core.state.seat.get_keyboard().expect("a keyboard");
        keyboard
            .current_focus()
            .map(|s| smithay::reexports::wayland_server::Resource::id(&s).protocol_id())
    }

    fn id(toplevel: &Toplevel) -> u32 {
        toplevel.surface.id().protocol_id()
    }

    fn apply(commands: &Value) -> Value {
        json!({ "commands": commands })
    }

    fn focus(surface: Option<&str>) -> Value {
        apply(&json!([{"focus": {"surface": surface}}]))
    }

    /// A core with a window manager that attached two toplevels `s1` and
    /// `s2` of one client.
    fn two_windows() -> (Core, Wayland, Client, Toplevel, Toplevel) {
        let mut core = Core::new();
        let mut wayland = Wayland::connect(&mut core);
        let first = wayland.toplevel(&mut core, "first");
        let second = wayland.toplevel(&mut core, "second");
        let mut wm = core.named("wm");
        let attach = apply(&json!([
            {"attach": {"surface": "s1", "parent": "root"}},
            {"attach": {"surface": "s2", "parent": "root"}},
        ]));
        wm.ok(&mut core, "scene.apply", attach);
        (core, wayland, wm, first, second)
    }

    #[test]
    fn the_focused_surface_gets_the_keyboard() {
        let (mut core, _wayland, mut wm, first, second) = two_windows();
        assert_eq!(keyboard(&core), None);
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        assert_eq!(keyboard(&core), Some(id(&first)));
        assert_eq!(core.state.focused_surface().as_deref(), Some("s1"));
        wm.ok(&mut core, "scene.apply", focus(Some("s2")));
        assert_eq!(keyboard(&core), Some(id(&second)));
        wm.ok(&mut core, "scene.apply", focus(None));
        assert_eq!(keyboard(&core), None);
    }

    #[test]
    fn a_client_focuses_only_its_own_surfaces() {
        let (mut core, _wayland, mut wm, first, _second) = two_windows();
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        let mut agent = core.named("agent");
        let (_, message) = agent.fails(&mut core, "scene.apply", focus(Some("s2")));
        assert!(message.contains("wm"), "{message}");
        assert_eq!(keyboard(&core), Some(id(&first)));
    }

    #[test]
    fn a_client_that_cleared_the_focus_can_give_it_back() {
        let (mut core, _wayland, mut wm, first, _second) = two_windows();
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        let mut launcher = core.named("launcher");
        launcher.ok(&mut core, "scene.apply", focus(None));
        assert_eq!(keyboard(&core), None);
        launcher.ok(&mut core, "scene.apply", focus(Some("s1")));
        assert_eq!(keyboard(&core), Some(id(&first)));
        let (_, message) = launcher.fails(&mut core, "scene.apply", focus(Some("s2")));
        assert!(
            message.contains("wm"),
            "only the surface that had it: {message}"
        );
    }

    #[test]
    fn the_keyboard_leaves_a_destroyed_window() {
        let (mut core, mut wayland, mut wm, first, _second) = two_windows();
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        let events = json!({"events": ["surface.destroyed"]});
        wm.ok(&mut core, "events.subscribe", events);
        wayland.destroy(&mut core, &first);
        assert_eq!(keyboard(&core), None);
        assert_eq!(wm.event_names(&mut core), ["surface.destroyed"]);
        let (_, message) = wm.fails(&mut core, "scene.apply", focus(Some("s1")));
        assert!(message.contains("s1"), "{message}");
    }

    #[test]
    fn an_exclusive_layer_surface_takes_the_keyboard_and_gives_it_back() {
        let (mut core, mut wayland, mut wm, first, second) = two_windows();
        core.add_output("HDMI-1", (800, 600));
        wayland.roundtrip(&mut core);
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        let bar = wayland.layer(&mut core, Layer::Top, KeyboardInteractivity::None);
        assert_eq!(keyboard(&core), Some(id(&first)), "a bar takes no keyboard");
        let launcher = wayland.layer(&mut core, Layer::Overlay, KeyboardInteractivity::Exclusive);
        assert_eq!(keyboard(&core), Some(launcher.surface.id().protocol_id()));
        wm.ok(&mut core, "scene.apply", focus(Some("s2")));
        assert_eq!(
            keyboard(&core),
            Some(launcher.surface.id().protocol_id()),
            "the launcher keeps the keyboard"
        );
        wayland.destroy_layer(&mut core, &launcher);
        assert_eq!(keyboard(&core), Some(id(&second)));
        wayland.destroy_layer(&mut core, &bar);
        assert_eq!(keyboard(&core), Some(id(&second)));
    }

    #[test]
    fn a_clicked_layer_surface_gives_the_keyboard_back_when_it_cannot_keep_it() {
        for case in ["role", "interactivity", "output"] {
            let (mut core, mut wayland, mut wm, first, _second) = two_windows();
            let output = core.add_output("HDMI-1", (800, 600));
            wayland.roundtrip(&mut core);
            wm.ok(&mut core, "scene.apply", focus(Some("s1")));
            let panel = wayland.layer(&mut core, Layer::Top, KeyboardInteractivity::OnDemand);
            let layer = smithay::desktop::layer_map_for_output(&output)
                .layers()
                .next()
                .map(|l| l.wl_surface().clone());
            core.state.set_layer_focus(layer);
            assert_eq!(keyboard(&core), Some(panel.surface.id().protocol_id()));
            match case {
                "role" => wayland.destroy_layer_role(&mut core, &panel),
                "interactivity" => {
                    wayland.set_layer_keyboard(&mut core, &panel, KeyboardInteractivity::None);
                }
                _ => {
                    core.state.remove_output(&output);
                    core.dispatch();
                }
            }
            assert_eq!(keyboard(&core), Some(id(&first)), "{case}");
            assert_eq!(core.state.layer_focus, None, "{case}");
        }
    }

    #[test]
    fn an_unmapped_toplevel_goes_with_the_keyboard_until_it_maps_again() {
        let (mut core, mut wayland, mut wm, first, _second) = two_windows();
        let events = json!({"events": ["surface.created", "surface.destroyed"]});
        wm.ok(&mut core, "events.subscribe", events);
        wayland.attach(&mut core, &first, (100, 100));
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        assert_eq!(keyboard(&core), Some(id(&first)));
        wayland.unmap(&mut core, &first);
        assert_eq!(keyboard(&core), None);
        assert_eq!(core.state.focused_surface(), None);
        assert!(core.state.scene.node("s1").is_none());
        assert_eq!(wm.event_names(&mut core), ["surface.destroyed"]);
        let configures = wayland.configures(&first).count;
        wayland.commit(&mut core, &first);
        assert_eq!(wayland.configures(&first).count, configures + 1);
        assert_eq!(wm.event_names(&mut core), ["surface.created"]);
        wayland.attach(&mut core, &first, (100, 100));
        assert!(core.state.scene.node("s1").is_some());
    }

    #[test]
    fn an_unmapped_toplevel_maps_again_without_its_old_configure() {
        let (mut core, mut wayland, mut wm, first, _second) = two_windows();
        let configure = apply(&json!([
            {"configure": {"surface": "s1", "size": [400, 300], "states": ["activated"]}},
        ]));
        let id = wm.send("scene.apply", configure);
        wayland.roundtrip(&mut core);
        wayland.answer(&mut core, &first);
        core.dispatch();
        assert_eq!(wm.answer(&mut core, id), Some(Ok(Value::Null)));
        wayland.attach(&mut core, &first, (400, 300));
        wayland.unmap(&mut core, &first);
        wayland.commit(&mut core, &first);
        let configures = wayland.configures(&first);
        assert_eq!(configures.size, (0, 0));
        assert_eq!(configures.states, []);
    }

    #[test]
    fn a_clicked_layer_surface_keeps_the_keyboard_when_the_focused_toplevel_goes() {
        for unmap in [false, true] {
            let (mut core, mut wayland, mut wm, first, _second) = two_windows();
            let output = core.add_output("HDMI-1", (800, 600));
            wayland.roundtrip(&mut core);
            wayland.attach(&mut core, &first, (100, 100));
            wm.ok(&mut core, "scene.apply", focus(Some("s1")));
            let panel = wayland.layer(&mut core, Layer::Top, KeyboardInteractivity::OnDemand);
            let layer = smithay::desktop::layer_map_for_output(&output)
                .layers()
                .next()
                .map(|l| l.wl_surface().clone());
            core.state.set_layer_focus(layer);
            if unmap {
                wayland.unmap(&mut core, &first);
            } else {
                wayland.destroy(&mut core, &first);
            }
            let panel = Some(panel.surface.id().protocol_id());
            assert_eq!(keyboard(&core), panel, "unmap={unmap}");
            assert_eq!(core.state.focused_surface(), None, "unmap={unmap}");
        }
    }

    #[test]
    fn the_emergency_reset_focuses_the_first_surface() {
        let (mut core, _wayland, mut wm, first, _second) = two_windows();
        core.add_output("HDMI-1", (800, 600));
        wm.ok(&mut core, "scene.apply", focus(Some("s2")));
        core.state.reset();
        core.dispatch();
        assert_eq!(core.state.focused_surface().as_deref(), Some("s1"));
        assert_eq!(keyboard(&core), Some(id(&first)));
    }

    #[test]
    fn a_configure_waits_for_the_commit_that_answers_it() {
        let (mut core, mut wayland, mut wm, first, _second) = two_windows();
        let configure = apply(&json!([
            {"configure": {"surface": "s1", "size": [400, 300], "states": ["activated"]}},
        ]));
        let id = wm.send("scene.apply", configure);
        wayland.roundtrip(&mut core);
        assert_eq!(wm.answer(&mut core, id), None, "the client did not commit");
        let configures = wayland.configures(&first);
        assert_eq!(configures.size, (400, 300));
        assert_eq!(configures.states, [Xdg::Activated]);
        wayland.answer(&mut core, &first);
        core.dispatch();
        assert_eq!(wm.answer(&mut core, id), Some(Ok(Value::Null)));
    }

    #[test]
    fn a_client_that_misses_the_deadline_delays_one_transaction_only() {
        let (mut core, mut wayland, mut wm, first, _second) = two_windows();
        let size = |w: i32| apply(&json!([{"configure": {"surface": "s1", "size": [w, 300]}}]));
        let id = wm.send("scene.apply", size(400));
        assert_eq!(wm.answer(&mut core, id), None);
        core.advance(crate::transaction::DEADLINE);
        assert_eq!(wm.answer(&mut core, id), Some(Ok(Value::Null)));
        let id = wm.send("scene.apply", size(500));
        assert_eq!(
            wm.answer(&mut core, id),
            Some(Ok(Value::Null)),
            "a late surface"
        );
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.configures(&first).size, (500, 300));
        wayland.answer(&mut core, &first);
        let id = wm.send("scene.apply", size(600));
        assert_eq!(wm.answer(&mut core, id), None, "the surface caught up");
    }

    #[test]
    fn a_popup_grabs_with_the_serial_of_a_key_release() {
        let (mut core, mut wayland, mut wm, first, _second) = two_windows();
        core.add_output("HDMI-1", (800, 600));
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        let f10 = smithay::input::keyboard::Keycode::new(76);
        for key_state in [KeyState::Pressed, KeyState::Released] {
            crate::input::key(&mut core.state, f10, key_state, InputTime::now(), false);
        }
        wayland.roundtrip(&mut core);
        let [press, release] = wayland.key_serials()[..] else {
            panic!("a press and a release: {:?}", wayland.key_serials());
        };
        let menu = wayland.popup(&mut core, &first, release);
        assert!(!wayland.popup_done(&menu), "the release opens the grab");
        let stale = wayland.popup(&mut core, &first, press.wrapping_sub(1));
        assert!(wayland.popup_done(&stale), "only an event of the client");
    }

    #[test]
    fn the_focus_moves_while_an_input_method_grabs_the_keyboard() {
        let (mut core, _wayland, mut wm, first, second) = two_windows();
        wm.ok(&mut core, "scene.apply", focus(Some("s1")));
        assert_eq!(keyboard(&core), Some(id(&first)));
        let keyboard_handle = core.state.seat.get_keyboard().unwrap();
        keyboard_handle.set_grab(
            &mut core.state,
            smithay::wayland::input_method::InputMethodKeyboardGrab::default(),
            smithay::utils::SERIAL_COUNTER.next_serial(),
        );
        wm.ok(&mut core, "scene.apply", focus(Some("s2")));
        assert_eq!(keyboard(&core), Some(id(&second)));
        assert!(keyboard_handle.is_grabbed());
    }
}
