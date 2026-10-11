//! The state of the compositor.

use std::{collections::HashMap, ffi::OsString, sync::Arc, time::Instant};

use chora_scene::{Effect, Event, OutputInfo, Scene, SurfaceInfo, ToplevelState};
use smithay::{
    backend::renderer::element::solid::SolidColorBuffer,
    desktop::{PopupManager, Window},
    input::{Seat, SeatState, keyboard::Keycode},
    output::Output,
    reexports::{
        calloop::{Interest, LoopHandle, LoopSignal, Mode, PostAction, generic::Generic},
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            Display, DisplayHandle,
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
        },
    },
    utils::{Logical, SERIAL_COUNTER, Serial, Size},
    wayland::{
        compositor::{CompositorClientState, CompositorState, with_states},
        output::OutputManagerState,
        selection::{data_device::DataDeviceState, primary_selection::PrimarySelectionState},
        shell::xdg::{XdgShellState, XdgToplevelSurfaceData, decoration::XdgDecorationState},
        shm::ShmState,
        socket::ListeningSocketSource,
        viewporter::ViewporterState,
    },
};

use crate::{ipc::Ipc, transaction::Transactions, winit::Backend};

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
    pub _xdg_decoration_state: XdgDecorationState,
    pub shm_state: ShmState,
    pub _output_manager_state: OutputManagerState,
    pub seat_state: SeatState<State>,
    pub data_device_state: DataDeviceState,
    pub primary_selection_state: PrimarySelectionState,
    pub _viewporter_state: ViewporterState,
    pub popups: PopupManager,
    pub seat: Seat<State>,

    pub backend: Option<Backend>,
    pub output: Option<Output>,

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
        Self {
            start_time: Instant::now(),
            compositor_state: CompositorState::new::<Self>(&dh),
            xdg_shell_state: XdgShellState::new::<Self>(&dh),
            _xdg_decoration_state: XdgDecorationState::new::<Self>(&dh),
            shm_state: ShmState::new::<Self>(&dh, vec![]),
            _output_manager_state: OutputManagerState::new_with_xdg_output::<Self>(&dh),
            data_device_state: DataDeviceState::new::<Self>(&dh),
            primary_selection_state: PrimarySelectionState::new::<Self>(&dh),
            _viewporter_state: ViewporterState::new::<Self>(&dh),
            popups: PopupManager::default(),
            seat_state,
            seat,
            display_handle: dh,
            loop_handle,
            loop_signal,
            socket_name,
            backend: None,
            output: None,
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
    /// configure.
    pub fn add_window(&mut self, window: Window) {
        let mut id = format!("s{}", self.next_surface);
        while self.scene.node(&id).is_some() || self.windows.contains_key(&id) {
            self.next_surface += 1;
            id = format!("s{}", self.next_surface);
        }
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

    /// Reads the app id and the title of a toplevel into its surface node.
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

    /// The logical size of the output.
    pub fn output_size(&self) -> Size<i32, Logical> {
        self.output
            .as_ref()
            .and_then(Output::current_mode)
            .map_or((0, 0).into(), |m| m.size.to_logical(1))
    }

    /// The outputs as the API describes them.
    pub fn output_info(&self) -> Vec<OutputInfo> {
        let size = self.output_size();
        self.output
            .iter()
            .map(|output| {
                let position = output.current_location();
                OutputInfo {
                    name: output.name(),
                    position: [position.x, position.y],
                    size: [size.w, size.h],
                    scale: output.current_scale().fractional_scale(),
                }
            })
            .collect()
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
            }
        }
    }

    /// Sends a configure event and returns its serial, if the state changed.
    pub fn configure(
        &self,
        surface: &str,
        size: Option<[i32; 2]>,
        states: &[ToplevelState],
    ) -> Option<Serial> {
        let toplevel = self.windows.get(surface)?.toplevel()?;
        toplevel.with_pending_state(|pending| {
            pending.size = size.map(|[w, h]| (w, h).into());
            for (state, xdg) in xdg_states() {
                set_xdg_states(&mut pending.states, xdg, states.contains(&state));
            }
        });
        toplevel.send_pending_configure()
    }

    /// The surface node with the keyboard focus.
    pub fn focused_surface(&self) -> Option<String> {
        let keyboard = self.seat.get_keyboard()?;
        self.surface_id(&keyboard.current_focus()?)
    }

    /// Sets the keyboard focus to a surface node, or to nothing.
    pub fn focus(&mut self, surface: Option<&str>) {
        let target = surface
            .and_then(|s| self.windows.get(s))
            .and_then(Window::toplevel)
            .map(|t| t.wl_surface().clone());
        let keyboard = self.seat.get_keyboard().expect("the seat has a keyboard");
        keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
    }
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
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}
