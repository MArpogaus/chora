//! Protocol handlers: the parts of the Wayland protocols that Smithay leaves
//! to the compositor.

use std::sync::atomic::Ordering;

use chora_scene::{Event, SurfaceRequest};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_dispatch2,
    desktop::{
        PopupKeyboardGrab, PopupKind, PopupPointerGrab, PopupUngrabStrategy, Window,
        WindowSurfaceType, find_popup_root_surface, get_popup_toplevel_coords,
        layer_map_for_output,
    },
    input::{
        Seat, SeatHandler, SeatState,
        pointer::{CursorImageStatus, Focus, PointerHandle},
    },
    output::Output,
    reexports::{
        wayland_protocols::xdg::decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode,
        wayland_server::{
            Client, Resource,
            protocol::{
                wl_buffer::WlBuffer, wl_output::WlOutput, wl_seat::WlSeat, wl_surface::WlSurface,
            },
        },
    },
    utils::{Logical, Point, Rectangle, Serial},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, add_pre_commit_hook,
            get_parent, is_sync_subsurface,
        },
        output::OutputHandler,
        pointer_constraints::{PointerConstraintsHandler, with_pointer_constraint},
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

use crate::{
    output::output_area,
    state::{ClientState, State},
};

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
            } else {
                self.layer_committed(&root);
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
    /// Removes the node of a toplevel that the client destroyed or
    /// unmapped, and tells the plugins with `surface.destroyed`.
    fn surface_gone(&mut self, id: &str) {
        if self.focused_surface() == Some(id) {
            self.focus(None);
        }
        self.configured_forget(id);
        self.unlist_toplevel(id);
        self.early_requests.remove(id);
        let existed = self.scene.remove_surface(id);
        self.transactions.forget(id);
        self.complete_transactions();
        // A toplevel without a first commit never had a node or an event.
        if existed {
            self.emit(&Event::SurfaceDestroyed { surface: id.into() });
        }
    }

    /// Handles a commit of a toplevel: the initial configure and the
    /// `surface.created` event.
    fn toplevel_committed(&mut self, id: &str) {
        let toplevel = self.windows[id]
            .toplevel()
            .expect("windows are toplevels")
            .clone();
        if !toplevel.is_initial_configure_sent() {
            // A commit without a buffer unmapped this toplevel, and Smithay
            // reset its role state. Its next commit is an initial commit, so
            // the toplevel goes as a destroyed one does.
            if self.scene.node(id).is_some() {
                self.surface_gone(id);
                return;
            }
            toplevel.send_configure();
            let info = self.surface_info(id);
            self.scene.add_surface(id, info.clone());
            // Without an output, the pointer is in no claim.
            let pointer = self
                .spot(self.pointer_layout)
                .map_or([f64::NAN; 2], |s| [s.scene.x, s.scene.y]);
            let focused = self.focused_surface().map(String::from);
            let parent = toplevel.parent().and_then(|p| self.surface_id(&p));
            let ipc = &self.ipc;
            let to = (self.scene)
                .manager_of_new(parent.as_deref(), focused.as_deref(), pointer, |n| {
                    ipc.live(n)
                })
                .map(String::from);
            self.scene.reserve(id, to.as_deref());
            self.list_toplevel(id);
            self.emit(&Event::SurfaceCreated {
                surface: id.into(),
                app_id: info.app_id,
                title: info.title,
                decoration: info.decoration,
                to,
            });
            self.send_early_requests(id);
            return;
        }
        let info = self.surface_info(id);
        let resized = self
            .scene
            .node(id)
            .and_then(|n| n.surface.as_ref())
            .is_some_and(|old| old.size != info.size);
        self.scene.set_surface_info(id, info.clone());
        if resized {
            self.emit(&Event::SurfaceChanged {
                surface: id.into(),
                app_id: info.app_id,
                title: info.title,
                size: info.size,
                decoration: info.decoration,
            });
        }
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

    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {
        self.cursor_status = image;
    }

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

impl OutputHandler for State {
    /// A taskbar that binds an output after it got its handles still
    /// learns the toplevels on that output.
    fn output_bound(&mut self, output: Output, _wl_output: WlOutput) {
        self.foreign_toplevel_manager_state.output_bound(&output);
    }
}

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        add_pre_commit_hook::<Self, _>(surface.wl_surface(), |state, _, surface| {
            state.hold_commit(surface);
        });
        self.add_window(Window::new_wayland_window(surface));
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let Some(id) = self.surface_id(surface.wl_surface()) else {
            return;
        };
        self.windows.remove(&id);
        self.surface_gone(&id);
    }

    fn title_changed(&mut self, surface: ToplevelSurface) {
        self.surface_changed(&surface);
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        self.surface_changed(&surface);
    }

    fn fullscreen_request(&mut self, surface: ToplevelSurface, _output: Option<WlOutput>) {
        self.request(&surface, SurfaceRequest::Fullscreen);
        Self::answer(&surface);
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        self.request(&surface, SurfaceRequest::Unfullscreen);
        Self::answer(&surface);
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        self.request(&surface, SurfaceRequest::Maximize);
        Self::answer(&surface);
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        self.request(&surface, SurfaceRequest::Unmaximize);
        Self::answer(&surface);
    }

    fn minimize_request(&mut self, surface: ToplevelSurface) {
        self.request(&surface, SurfaceRequest::Minimize);
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
        // A popup grabs the input only for the input event of its client
        // that opened it, and never while the session is locked.
        if self.lock.is_some() || !self.pressed(surface.wl_surface(), serial) {
            surface.send_popup_done();
            return;
        }
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
            let keyboard_busy = crate::state::popup_grabbed(&keyboard)
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
    /// Whether a serial is the one of the last press or of the last key or
    /// button event that the client of a surface got. A client of a client
    /// socket needs its last press, as for a token.
    fn pressed(&self, surface: &WlSurface, serial: Serial) -> bool {
        let client = self.display_handle.get_client(surface.id()).ok();
        let serial = u32::from(serial);
        client
            .as_ref()
            .and_then(|c| c.get_data::<ClientState>())
            .is_some_and(|d| {
                if d.name.is_some() {
                    return crate::security::last_press(d, serial)
                        && self.focused_or_unnamed(surface);
                }
                [&d.last_press, &d.last_input]
                    .iter()
                    .any(|s| s.load(Ordering::Relaxed) == serial)
            })
    }

    fn surface_changed(&mut self, surface: &ToplevelSurface) {
        let Some(id) = self.surface_id(surface.wl_surface()) else {
            return;
        };
        let info = self.surface_info(&id);
        self.scene.set_surface_info(&id, info.clone());
        self.relist_toplevel(&id);
        self.emit(&Event::SurfaceChanged {
            surface: id,
            app_id: info.app_id,
            title: info.title,
            size: info.size,
            decoration: info.decoration,
        });
    }

    /// Keeps a popup inside the output that shows its toplevel or its
    /// layer surface.
    pub fn unconstrain_popup(&self, popup: &PopupSurface) {
        let kind = PopupKind::Xdg(popup.clone());
        let Some(mut target) = find_popup_root_surface(&kind)
            .ok()
            .and_then(|root| self.popup_area(&root))
        else {
            return;
        };
        target.loc -= get_popup_toplevel_coords(&kind);
        popup.with_pending_state(|state| {
            state.geometry = state.positioner.get_unconstrained_geometry(target);
        });
    }

    /// The area for the popups of a toplevel or a layer surface, relative to
    /// that surface: the area of the output that shows it.
    fn popup_area(&self, root: &WlSurface) -> Option<Rectangle<i32, Logical>> {
        if let Some(origin) = self
            .surface_id(root)
            .and_then(|id| crate::render::window_origin(self, &id))
        {
            let mut area = self.scene_area_at(origin.to_f64());
            area.loc -= origin;
            return Some(area);
        }
        let output = self.output_of_layer(root)?;
        let map = layer_map_for_output(&output);
        let layer = map.layer_for_surface(root, WindowSurfaceType::TOPLEVEL)?;
        let geo = map.layer_geometry(layer)?;
        Some(Rectangle::new(
            Point::default() - geo.loc,
            output_area(&output).size,
        ))
    }
}

impl XdgDecorationHandler for State {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        toplevel
            .with_pending_state(|state| state.decoration_mode = Some(DecorationMode::ClientSide));
    }

    /// Grants the mode that the client asks for, and tells the plugins.
    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: DecorationMode) {
        toplevel.with_pending_state(|state| state.decoration_mode = Some(mode));
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
            self.surface_changed(&toplevel);
        }
    }

    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        toplevel
            .with_pending_state(|state| state.decoration_mode = Some(DecorationMode::ClientSide));
        if toplevel.is_initial_configure_sent() {
            toplevel.send_pending_configure();
            self.surface_changed(&toplevel);
        }
    }
}

impl PointerConstraintsHandler for State {
    /// Activates a new constraint at once if the pointer is on its surface.
    fn new_constraint(&mut self, surface: &WlSurface, pointer: &PointerHandle<Self>) {
        if pointer.current_focus().as_ref() == Some(surface) && self.focused_or_unnamed(surface) {
            with_pointer_constraint(surface, pointer, |constraint| {
                if let Some(constraint) = constraint {
                    constraint.activate();
                }
            });
        }
    }
}

delegate_dispatch2!(State);
