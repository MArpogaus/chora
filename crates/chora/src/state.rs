//! The state of the compositor.

use std::{
    collections::HashMap,
    ffi::OsString,
    sync::{Arc, atomic::AtomicU32},
    time::Instant,
};

use chora_scene::{Effect, Event, Scene, SurfaceInfo, SurfaceRequest, ToplevelState};
use smithay::{
    backend::renderer::element::solid::SolidColorBuffer,
    desktop::{PopupManager, Window},
    input::{Seat, SeatState, keyboard::Keycode, pointer::CursorImageStatus},
    output::Output,
    reexports::{
        calloop::{Interest, LoopHandle, LoopSignal, Mode, PostAction, generic::Generic},
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            Display, DisplayHandle, Resource, backend::ClientData, protocol::wl_surface::WlSurface,
        },
    },
    utils::{Logical, Point, SERIAL_COUNTER, Serial},
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

use crate::{backend::Backend, ipc::Ipc, lock::Lock, transaction::Transactions};

/// The state of the compositor, passed to every callback of the event loop.
#[allow(clippy::struct_field_names)] // the names of Smithay's handler traits
pub struct State {
    pub start_time: Instant,
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
    /// The outputs, in the order of their arrival.
    pub outputs: Vec<Output>,
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
    pub activation_state: XdgActivationState,
    /// Requests of surfaces before their first commit.
    pub early_requests: HashMap<String, Vec<SurfaceRequest>>,

    pub scene: Scene,
    /// The toplevels by the id of their surface node.
    pub windows: HashMap<String, Window>,
    /// One color buffer per rect node.
    pub rects: HashMap<String, SolidColorBuffer>,
    next_surface: u64,
    pub ipc: Ipc,
    pub transactions: Transactions,
    /// Keys whose press went to the safety layer or to a grab, so that their
    /// release must not reach a client.
    pub suppressed_keys: Vec<Keycode>,
}

impl State {
    pub fn new(
        display: Display<State>,
        loop_handle: LoopHandle<'static, State>,
        loop_signal: LoopSignal,
    ) -> Self {
        let dh = display.handle();
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "seat0");
        seat.add_keyboard(smithay::input::keyboard::XkbConfig::default(), 200, 25)
            .expect("the default keymap compiles");
        seat.add_pointer();
        let socket_name = listen(display, &loop_handle);
        let idle_notifier_state = IdleNotifierState::new(&dh, loop_handle.clone());
        // These globals live as long as the display; nothing reads their
        // state.
        XdgDecorationState::new::<Self>(&dh);
        OutputManagerState::new_with_xdg_output::<Self>(&dh);
        ViewporterState::new::<Self>(&dh);
        IdleInhibitManagerState::new::<Self>(&dh);
        TextInputManagerState::new::<Self>(&dh);
        InputMethodManagerState::new::<Self, _>(&dh, |_| true);
        FractionalScaleManagerState::new::<Self>(&dh);
        CursorShapeManagerState::new::<Self>(&dh);
        RelativePointerManagerState::new::<Self>(&dh);
        PointerConstraintsState::new::<Self>(&dh);
        Self {
            start_time: Instant::now(),
            compositor_state: CompositorState::new::<Self>(&dh),
            layer_shell_state: WlrLayerShellState::new::<Self>(&dh),
            lock_state: SessionLockManagerState::new::<Self, _>(&dh, |_| true),
            lock: None,
            xdg_shell_state: XdgShellState::new::<Self>(&dh),
            shm_state: ShmState::new::<Self>(&dh, vec![]),
            data_device_state: DataDeviceState::new::<Self>(&dh),
            primary_selection_state: PrimarySelectionState::new::<Self>(&dh),
            popups: PopupManager::default(),
            seat_state,
            seat,
            display_handle: dh.clone(),
            loop_handle,
            loop_signal,
            socket_name,
            backend: None,
            dmabuf_state: DmabufState::new(),
            outputs: Vec::new(),
            pointer_layout: Point::default(),
            cursor_status: CursorImageStatus::default_named(),
            focus_wanted: None,
            layer_focus: None,
            configured: HashMap::new(),
            idle_notifier_state,
            idle_inhibitors: Vec::new(),
            foreign_toplevel_list_state: ForeignToplevelListState::new::<Self>(&dh),
            foreign_toplevels: HashMap::new(),
            activation_state: XdgActivationState::new::<Self>(&dh),
            early_requests: HashMap::new(),
            scene: Scene::default(),
            windows: HashMap::new(),
            rects: HashMap::new(),
            next_surface: 1,
            ipc: Ipc::default(),
            transactions: Transactions::default(),
            suppressed_keys: Vec::new(),
        }
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
                Effect::Focus(surface) => self.focus(surface.as_deref()),
                Effect::Close(surface) => {
                    if let Some(t) = self.windows.get(&surface).and_then(Window::toplevel) {
                        t.send_close();
                    }
                }
                Effect::Detached(surface) => {
                    if self.focused_surface().as_deref() == Some(surface.as_str()) {
                        self.focus(None);
                    }
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
    /// The configure stays known, for a client that maps the surface again.
    pub fn configure(
        &mut self,
        surface: &str,
        size: Option<[i32; 2]>,
        states: &[ToplevelState],
    ) -> Option<Serial> {
        let toplevel = self.windows.get(surface)?.toplevel()?;
        set_pending(toplevel, size, states);
        self.configured
            .insert(surface.into(), (size, states.to_vec()));
        toplevel.send_pending_configure()
    }

    /// Forgets the configure of a destroyed surface.
    pub fn configured_forget(&mut self, surface: &str) {
        self.configured.remove(surface);
    }

    /// Prepares the initial configure of a surface that a client maps again:
    /// the last configure of its plugin.
    pub fn reconfigure(&self, surface: &str) {
        let toplevel = self.windows.get(surface).and_then(Window::toplevel);
        if let (Some(toplevel), Some((size, states))) = (toplevel, self.configured.get(surface)) {
            set_pending(toplevel, *size, states);
        }
    }

    /// The surface node that has the focus, as plugins set it.
    pub fn focused_surface(&self) -> Option<String> {
        self.focus_wanted.clone()
    }

    /// Focuses a surface node, or nothing. A layer surface that a click
    /// focused gives the keyboard back.
    pub fn focus(&mut self, surface: Option<&str>) {
        self.focus_wanted = surface.map(Into::into);
        self.layer_focus = None;
        self.update_focus();
    }

    /// Gives the keyboard to a layer surface after a click, or takes it
    /// back.
    pub fn set_layer_focus(&mut self, layer: Option<WlSurface>) {
        if self.layer_focus != layer {
            self.layer_focus = layer;
            self.update_focus();
        }
    }

    /// Moves the keyboard to the surface that should have it: the lock
    /// screen, a layer surface that takes it exclusively, a layer surface
    /// after a click, or the focused surface node. A popup grab keeps the
    /// keyboard, unless the session locks.
    pub fn update_focus(&mut self) {
        let keyboard = self.seat.get_keyboard().expect("the seat has a keyboard");
        if self.lock.is_some() {
            let target = self.lock_focus();
            keyboard.unset_grab(self);
            if keyboard.current_focus() != target {
                keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
            }
            return;
        }
        // A launcher ends a popup grab, for example of an open menu.
        let exclusive = self.exclusive_layer();
        if keyboard.is_grabbed() {
            if exclusive.is_none() {
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
        let clicked = self.layer_focus.clone().filter(Resource::is_alive);
        let target = exclusive.or(clicked).or(window);
        if keyboard.current_focus() != target {
            keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
        }
    }
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

/// Opens the Wayland socket and adds the display to the event loop.
fn listen(display: Display<State>, loop_handle: &LoopHandle<'static, State>) -> OsString {
    let socket = ListeningSocketSource::new_auto().expect("a free wayland socket name");
    let name = socket.socket_name().to_os_string();
    loop_handle
        .insert_source(socket, |stream, (), state| {
            if let Err(err) = state
                .display_handle
                .insert_client(stream, Arc::new(ClientState::default()))
            {
                tracing::warn!(?err, "a wayland client failed to connect");
            }
        })
        .expect("the event loop accepts the socket");
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
    name
}

/// Data of a Wayland client.
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    /// The serial of the last key or button press that the client got.
    pub last_press: AtomicU32,
}

impl ClientData for ClientState {}
