// SPDX-License-Identifier: GPL-3.0-or-later
// Adapted from Pinnacle, src/protocol/foreign_toplevel.rs (GPL-3.0-or-later),
// which follows niri.

//! `wlr-foreign-toplevel-management`: taskbars list the toplevels and ask
//! for changes to them.
//!
//! Each toplevel gets a handle after `surface.created`. The handle tells
//! the title, the application id, the outputs that show the surface and
//! the states. `activated` marks the surface that plugins focused.
//! `maximized` and `fullscreen` come from the last configure of a plugin.
//! The core never reports `minimized`, because plugins decide what it
//! means.
//!
//! The requests of a taskbar go to plugins as `surface.requested`. The core
//! changes nothing by itself, also not for `close`.

use std::collections::HashSet;

use chora_scene::{SurfaceInfo, SurfaceRequest, ToplevelState};
use smithay::{
    desktop::Window,
    output::Output,
    reexports::{
        wayland_protocols_wlr::foreign_toplevel::v1::server::{
            zwlr_foreign_toplevel_handle_v1::{self as handle, ZwlrForeignToplevelHandleV1},
            zwlr_foreign_toplevel_manager_v1::{self as manager, ZwlrForeignToplevelManagerV1},
        },
        wayland_server::{
            Client, DataInit, DisplayHandle, New, Resource, backend::ClientId,
            protocol::wl_output::WlOutput,
        },
    },
    wayland::{Dispatch2, GlobalDispatch2},
};

use crate::state::State;

/// The highest version of the protocol. Version 3 adds the parent of a
/// toplevel; the core sends no parent.
const VERSION: u32 = 3;

/// The managers of the taskbars and the toplevels that they see.
pub struct ForeignToplevelManagerState {
    display: DisplayHandle,
    /// The managers that did not stop.
    managers: Vec<ZwlrForeignToplevelManagerV1>,
    /// The toplevels in the order of their `surface.created`.
    toplevels: Vec<Toplevel>,
}

/// What taskbars know about one toplevel.
struct Toplevel {
    /// The surface node.
    surface: String,
    title: Option<String>,
    app_id: Option<String>,
    states: Vec<handle::State>,
    /// The outputs that show the surface.
    outputs: Vec<Output>,
    /// One handle per manager.
    handles: Vec<Handle>,
}

/// A handle and the `wl_output` objects that it entered.
struct Handle {
    resource: ZwlrForeignToplevelHandleV1,
    entered: Vec<WlOutput>,
}

impl ForeignToplevelManagerState {
    /// Creates the global. Only clients outside of a sandbox see it
    /// (see `crate::security`).
    pub fn new(display: &DisplayHandle) -> Self {
        display.create_global::<State, ZwlrForeignToplevelManagerV1, _>(VERSION, ManagerGlobal);
        Self {
            display: display.clone(),
            managers: Vec::new(),
            toplevels: Vec::new(),
        }
    }

    fn get_mut(&mut self, surface: &str) -> Option<&mut Toplevel> {
        self.toplevels.iter_mut().find(|t| t.surface == surface)
    }

    /// Adds a toplevel and announces it to all managers.
    pub fn add(&mut self, surface: &str, info: SurfaceInfo, states: Vec<handle::State>) {
        let mut toplevel = Toplevel {
            surface: surface.into(),
            title: info.title,
            app_id: info.app_id,
            states,
            outputs: Vec::new(),
            handles: Vec::new(),
        };
        for manager in &self.managers {
            toplevel.announce(&self.display, manager);
        }
        self.toplevels.push(toplevel);
    }

    /// Tells the managers a new title or application id. A client cannot
    /// unset them, so a missing value changes nothing.
    pub fn set_info(&mut self, surface: &str, info: SurfaceInfo) {
        let Some(toplevel) = self.get_mut(surface) else {
            return;
        };
        let title = info.title.filter(|t| toplevel.title.as_ref() != Some(t));
        let app_id = info.app_id.filter(|a| toplevel.app_id.as_ref() != Some(a));
        if title.is_none() && app_id.is_none() {
            return;
        }
        for handle in &toplevel.handles {
            send_info(&handle.resource, title.as_deref(), app_id.as_deref());
            handle.resource.done();
        }
        toplevel.title = title.or(toplevel.title.take());
        toplevel.app_id = app_id.or(toplevel.app_id.take());
    }

    /// Tells the managers new states of a toplevel, if they changed.
    pub fn set_states(&mut self, surface: &str, states: Vec<handle::State>) {
        let Some(toplevel) = self.get_mut(surface).filter(|t| t.states != states) else {
            return;
        };
        toplevel.states = states;
        for handle in &toplevel.handles {
            send_states(&handle.resource, &toplevel.states);
            handle.resource.done();
        }
    }

    /// Notes the toplevels that an output shows, by their surface nodes.
    /// The others leave the output.
    pub fn set_shown(&mut self, output: &Output, shown: &HashSet<String>) {
        for toplevel in &mut self.toplevels {
            let on = shown.contains(&toplevel.surface);
            if on == toplevel.outputs.contains(output) {
                continue;
            }
            if on {
                toplevel.outputs.push(output.clone());
            } else {
                toplevel.outputs.retain(|o| o != output);
            }
            toplevel.sync_outputs();
        }
    }

    /// Enters the `wl_output` objects that clients bound after their
    /// handles.
    pub fn output_bound(&mut self, output: &Output) {
        for toplevel in &mut self.toplevels {
            if toplevel.outputs.contains(output) {
                toplevel.sync_outputs();
            }
        }
    }

    /// Closes the handles of a destroyed toplevel.
    pub fn remove(&mut self, surface: &str) {
        let Some(index) = self.toplevels.iter().position(|t| t.surface == surface) else {
            return;
        };
        for handle in self.toplevels.remove(index).handles {
            handle.resource.closed();
        }
    }
}

impl Toplevel {
    /// Creates a handle for a manager and sends all details.
    fn announce(&mut self, display: &DisplayHandle, manager: &ZwlrForeignToplevelManagerV1) {
        let Some(client) = manager.client() else {
            return;
        };
        let data = HandleData {
            surface: self.surface.clone(),
        };
        let Ok(resource) = client.create_resource::<ZwlrForeignToplevelHandleV1, _, State>(
            display,
            manager.version(),
            data,
        ) else {
            return;
        };
        manager.toplevel(&resource);
        send_info(&resource, self.title.as_deref(), self.app_id.as_deref());
        send_states(&resource, &self.states);
        let mut handle = Handle {
            resource,
            entered: Vec::new(),
        };
        handle.sync_outputs(&self.outputs);
        handle.resource.done();
        self.handles.push(handle);
    }

    /// Makes every handle enter the outputs that show the surface, and
    /// leave the others.
    fn sync_outputs(&mut self) {
        for handle in &mut self.handles {
            if handle.sync_outputs(&self.outputs) {
                handle.resource.done();
            }
        }
    }
}

impl Handle {
    /// Sends `output_enter` and `output_leave` until the handle entered
    /// exactly the `wl_output` objects of its client for the outputs.
    /// Returns whether it sent an event.
    fn sync_outputs(&mut self, outputs: &[Output]) -> bool {
        let Some(client) = self.resource.client() else {
            return false;
        };
        let wanted: Vec<WlOutput> = outputs
            .iter()
            .flat_map(|o| o.client_outputs(&client))
            .collect();
        self.entered.retain(Resource::is_alive);
        let (kept, left): (Vec<_>, Vec<_>) =
            self.entered.drain(..).partition(|o| wanted.contains(o));
        let new: Vec<_> = wanted.into_iter().filter(|o| !kept.contains(o)).collect();
        for wl_output in &left {
            self.resource.output_leave(wl_output);
        }
        for wl_output in &new {
            self.resource.output_enter(wl_output);
        }
        let changed = !left.is_empty() || !new.is_empty();
        self.entered = kept;
        self.entered.extend(new);
        changed
    }
}

fn send_info(resource: &ZwlrForeignToplevelHandleV1, title: Option<&str>, app_id: Option<&str>) {
    if let Some(title) = title {
        resource.title(title.into());
    }
    if let Some(app_id) = app_id {
        resource.app_id(app_id.into());
    }
}

/// Sends the states. Version 1 of the protocol has no fullscreen state.
fn send_states(resource: &ZwlrForeignToplevelHandleV1, states: &[handle::State]) {
    let fullscreen = resource.version() >= 2;
    let bytes = states
        .iter()
        .filter(|s| fullscreen || **s != handle::State::Fullscreen)
        .flat_map(|s| u32::from(*s).to_ne_bytes())
        .collect();
    resource.state(bytes);
}

/// The request to plugins for a request of a taskbar. The core never
/// reports `minimized`, so `unset_minimized` has no meaning. The rectangle
/// of a taskbar entry is a hint for animations, which plugins draw.
fn surface_request(request: &handle::Request) -> Option<SurfaceRequest> {
    use handle::Request as R;
    match request {
        R::Activate { .. } => Some(SurfaceRequest::Activate),
        R::Close => Some(SurfaceRequest::Close),
        R::SetFullscreen { .. } => Some(SurfaceRequest::Fullscreen),
        R::UnsetFullscreen => Some(SurfaceRequest::Unfullscreen),
        R::SetMaximized => Some(SurfaceRequest::Maximize),
        R::UnsetMaximized => Some(SurfaceRequest::Unmaximize),
        R::SetMinimized => Some(SurfaceRequest::Minimize),
        _ => None,
    }
}

impl State {
    /// The states of a toplevel for taskbars.
    pub fn taskbar_states(&self, surface: &str) -> Vec<handle::State> {
        let configured = self.configured_states(surface);
        let mut states: Vec<_> = [
            (ToplevelState::Maximized, handle::State::Maximized),
            (ToplevelState::Fullscreen, handle::State::Fullscreen),
        ]
        .into_iter()
        .filter(|(state, _)| configured.contains(state))
        .map(|(_, wlr)| wlr)
        .collect();
        if self.focused_surface() == Some(surface) {
            states.push(handle::State::Activated);
        }
        states
    }

    /// Tells taskbars the states of a toplevel again, if they changed.
    pub fn restate_toplevel(&mut self, surface: &str) {
        let states = self.taskbar_states(surface);
        self.foreign_toplevel_manager_state
            .set_states(surface, states);
    }
}

/// The data of the global.
pub struct ManagerGlobal;

/// The data of a bound manager.
pub struct ManagerData;

/// The data of a handle: its surface node.
pub struct HandleData {
    surface: String,
}

impl GlobalDispatch2<ZwlrForeignToplevelManagerV1, State> for ManagerGlobal {
    fn bind(
        &self,
        state: &mut State,
        display: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrForeignToplevelManagerV1>,
        data_init: &mut DataInit<'_, State>,
    ) {
        let manager = data_init.init(resource, ManagerData);
        let protocol = &mut state.foreign_toplevel_manager_state;
        for toplevel in &mut protocol.toplevels {
            toplevel.announce(display, &manager);
        }
        protocol.managers.push(manager);
    }

    fn can_view(&self, client: &Client) -> bool {
        crate::security::unrestricted(client)
    }
}

impl Dispatch2<ZwlrForeignToplevelManagerV1, State> for ManagerData {
    fn request(
        &self,
        state: &mut State,
        _client: &Client,
        resource: &ZwlrForeignToplevelManagerV1,
        request: manager::Request,
        _display: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        if matches!(request, manager::Request::Stop) {
            let managers = &mut state.foreign_toplevel_manager_state.managers;
            managers.retain(|m| m != resource);
            resource.finished();
        }
    }

    fn destroyed(
        &self,
        state: &mut State,
        _client: ClientId,
        resource: &ZwlrForeignToplevelManagerV1,
    ) {
        let managers = &mut state.foreign_toplevel_manager_state.managers;
        managers.retain(|m| m != resource);
    }
}

impl Dispatch2<ZwlrForeignToplevelHandleV1, State> for HandleData {
    /// Passes the request to plugins. The handle of a destroyed toplevel
    /// has no window any more, so its requests change nothing.
    fn request(
        &self,
        state: &mut State,
        _client: &Client,
        _resource: &ZwlrForeignToplevelHandleV1,
        request: handle::Request,
        _display: &DisplayHandle,
        _data_init: &mut DataInit<'_, State>,
    ) {
        let toplevel = state
            .windows
            .get(&self.surface)
            .and_then(Window::toplevel)
            .cloned();
        if let (Some(toplevel), Some(request)) = (toplevel, surface_request(&request)) {
            state.request(&toplevel, request);
        }
    }

    fn destroyed(
        &self,
        state: &mut State,
        _client: ClientId,
        resource: &ZwlrForeignToplevelHandleV1,
    ) {
        let protocol = &mut state.foreign_toplevel_manager_state;
        if let Some(toplevel) = protocol.get_mut(&self.surface) {
            toplevel.handles.retain(|h| h.resource != *resource);
        }
    }
}

#[cfg(test)]
mod tests {
    use chora_scene::SurfaceRequest;
    use smithay::reexports::wayland_protocols_wlr::foreign_toplevel::v1::server::zwlr_foreign_toplevel_handle_v1::Request;

    use super::surface_request;

    #[test]
    fn taskbar_requests_go_to_plugins() {
        assert_eq!(
            surface_request(&Request::Close),
            Some(SurfaceRequest::Close)
        );
        assert_eq!(
            surface_request(&Request::SetMaximized),
            Some(SurfaceRequest::Maximize)
        );
        assert_eq!(
            surface_request(&Request::UnsetFullscreen),
            Some(SurfaceRequest::Unfullscreen)
        );
        assert_eq!(surface_request(&Request::UnsetMinimized), None);
    }
}
