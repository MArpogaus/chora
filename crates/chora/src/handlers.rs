//! Protocol handlers: the parts of the Wayland protocols that Smithay leaves
//! to the compositor.

use chora_scene::Event;
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_dispatch2,
    desktop::{
        PopupKeyboardGrab, PopupKind, PopupPointerGrab, PopupUngrabStrategy, Window,
        find_popup_root_surface, get_popup_toplevel_coords,
    },
    input::{
        Seat, SeatHandler, SeatState,
        pointer::{CursorImageStatus, Focus},
    },
    reexports::{
        wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode,
        wayland_server::{
            Client, Resource,
            protocol::{wl_buffer::WlBuffer, wl_seat::WlSeat, wl_surface::WlSurface},
        },
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, get_parent,
            is_sync_subsurface,
        },
        output::OutputHandler,
        selection::{
            SelectionHandler,
            data_device::{
                DataDeviceHandler, DataDeviceState, WaylandDndGrabHandler, set_data_device_focus,
            },
            primary_selection::{
                PrimarySelectionHandler, PrimarySelectionState, set_primary_focus,
            },
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
            decoration::XdgDecorationHandler,
        },
        shm::{ShmHandler, ShmState},
    },
};

use crate::state::{ClientState, State};

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("every client has client state")
            .compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(id) = self.surface_id(&root) {
                self.windows[&id].on_commit();
                self.toplevel_committed(&id);
            }
        }
        self.popups.commit(surface);
        if let Some(PopupKind::Xdg(popup)) = self.popups.find_popup(surface)
            && !popup.is_initial_configure_sent()
        {
            popup
                .send_configure()
                .expect("initial configure is always allowed");
        }
    }
}

impl State {
    /// Handles a commit of a toplevel: the initial configure, the
    /// `surface.created` event and pending transactions.
    fn toplevel_committed(&mut self, id: &str) {
        let toplevel = self.windows[id]
            .toplevel()
            .expect("windows are toplevels")
            .clone();
        if !toplevel.is_initial_configure_sent() {
            toplevel.send_configure();
            // A client that maps a surface again after it unmapped it, with
            // a commit without a buffer, keeps its node.
            let info = self.surface_info(id);
            if self.scene.add_surface(id, info.clone()) {
                self.emit(&Event::SurfaceCreated {
                    surface: id.into(),
                    app_id: info.app_id,
                    title: info.title,
                });
            }
            return;
        }
        let info = self.surface_info(id);
        self.scene.set_surface_info(id, info);
        self.transactions.committed(id, &toplevel);
        self.complete_transactions();
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) {
        let dh = &self.display_handle;
        let client = focused.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client.clone());
        set_primary_focus(dh, seat, client);
    }
}

impl SelectionHandler for State {
    type SelectionUserData = ();
}

impl DataDeviceHandler for State {
    fn data_device_state(&mut self) -> &mut DataDeviceState {
        &mut self.data_device_state
    }
}

impl smithay::input::dnd::DndGrabHandler for State {}
impl WaylandDndGrabHandler for State {}

impl PrimarySelectionHandler for State {
    fn primary_selection_state(&mut self) -> &mut PrimarySelectionState {
        &mut self.primary_selection_state
    }
}

impl OutputHandler for State {}

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        self.add_window(Window::new_wayland_window(surface));
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let Some(id) = self.surface_id(surface.wl_surface()) else {
            return;
        };
        self.windows.remove(&id);
        let existed = self.scene.remove_surface(&id);
        self.transactions.forget(&id);
        self.complete_transactions();
        // A toplevel without a first commit never had a node or an event.
        if existed {
            self.emit(&Event::SurfaceDestroyed { surface: id });
        }
    }

    fn title_changed(&mut self, surface: ToplevelSurface) {
        self.surface_changed(&surface);
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        self.surface_changed(&surface);
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        self.unconstrain_popup(&surface);
        if let Err(err) = self.popups.track_popup(PopupKind::Xdg(surface)) {
            tracing::warn!(?err, "a popup without a parent");
        }
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        surface.with_pending_state(|state| {
            state.geometry = positioner.get_geometry();
            state.positioner = positioner;
        });
        self.unconstrain_popup(&surface);
        surface.send_repositioned(token);
    }

    /// A popup grab, for example of a menu: input goes to the popup until a
    /// click outside closes it.
    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        let Some(seat) = Seat::<Self>::from_resource(&seat) else {
            return;
        };
        let kind = PopupKind::Xdg(surface);
        let Ok(root) = find_popup_root_surface(&kind) else {
            return;
        };
        if let Ok(mut grab) = self.popups.grab_popup(root, kind, &seat, serial) {
            let previous = grab.previous_serial().unwrap_or(serial);
            let keyboard = seat.get_keyboard().expect("the seat has a keyboard");
            let pointer = seat.get_pointer().expect("the seat has a pointer");
            let keyboard_busy = keyboard.is_grabbed()
                && !(keyboard.has_grab(serial) || keyboard.has_grab(previous));
            let pointer_busy =
                pointer.is_grabbed() && !(pointer.has_grab(serial) || pointer.has_grab(previous));
            if keyboard_busy || pointer_busy {
                grab.ungrab(PopupUngrabStrategy::All);
                return;
            }
            keyboard.set_focus(self, grab.current_grab(), serial);
            keyboard.set_grab(self, PopupKeyboardGrab::new(&grab), serial);
            pointer.set_grab(self, PopupPointerGrab::new(&grab), serial, Focus::Keep);
        }
    }
}

impl State {
    fn surface_changed(&mut self, surface: &ToplevelSurface) {
        let Some(id) = self.surface_id(surface.wl_surface()) else {
            return;
        };
        let info = self.surface_info(&id);
        self.scene.set_surface_info(&id, info.clone());
        self.emit(&Event::SurfaceChanged {
            surface: id,
            app_id: info.app_id,
            title: info.title,
        });
    }

    /// Keeps a popup inside the output that shows its toplevel.
    fn unconstrain_popup(&self, popup: &PopupSurface) {
        let kind = PopupKind::Xdg(popup.clone());
        let Ok(root) = find_popup_root_surface(&kind) else {
            return;
        };
        let Some(origin) = self
            .surface_id(&root)
            .and_then(|id| crate::render::window_origin(self, &id))
        else {
            return;
        };
        let mut target = self.scene_area_at(origin.to_f64());
        target.loc -= origin;
        target.loc -= get_popup_toplevel_coords(&kind);
        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }
}

impl XdgDecorationHandler for State {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel
            .with_pending_state(|state| state.decoration_mode = Some(DecorationMode::ClientSide));
    }

    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: DecorationMode) {
        toplevel.with_pending_state(|state| state.decoration_mode = Some(mode));
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        toplevel
            .with_pending_state(|state| state.decoration_mode = Some(DecorationMode::ClientSide));
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
        }
    }
}

impl smithay::wayland::pointer_constraints::PointerConstraintsHandler for State {}

delegate_dispatch2!(State);
