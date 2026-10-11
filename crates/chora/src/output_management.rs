// Adapted from Pinnacle, src/protocol/output_management.rs (GPL-3.0-or-later, https://github.com/pinnacle-comp/pinnacle).

//! The `wlr-output-management` protocol (D27): kanshi and wlr-randr read
//! the outputs as heads and configure them.
//!
//! Every output of the backend is a head, also while it is off. A head
//! shows the modes, the current mode, the position, the transform and the
//! scale of its output, and the make, model and serial number of the
//! monitor. [`State::configure_outputs`] tests and applies a
//! configuration. The backends have no adaptive sync, so the global has
//! version 3.

use std::sync::Mutex;

use smithay::{
    output::{Mode, Output, PhysicalProperties},
    reexports::{
        wayland_protocols_wlr::output_management::v1::server::{
            zwlr_output_configuration_head_v1::{
                self as config_head, ZwlrOutputConfigurationHeadV1,
            },
            zwlr_output_configuration_v1::{self as config, ZwlrOutputConfigurationV1},
            zwlr_output_head_v1::{self, ZwlrOutputHeadV1},
            zwlr_output_manager_v1::{self, ZwlrOutputManagerV1},
            zwlr_output_mode_v1::{self, ZwlrOutputModeV1},
        },
        wayland_server::{
            Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource, WEnum,
            backend::ClientId, protocol::wl_output,
        },
    },
    utils::{Logical, Point, SERIAL_COUNTER, Transform},
};

use crate::{
    output::{ModeChoice, OutputChanges, OutputConfig},
    state::State,
};

/// The version of the global. Version 4 adds adaptive sync.
const VERSION: u32 = 3;

/// The heads and the managers that clients bound.
pub struct OutputManagement {
    display: DisplayHandle,
    /// Every output of the backend, on or off, in the order of arrival.
    heads: Vec<Head>,
    managers: Vec<Manager>,
    /// The serial of the last `done` event.
    serial: u32,
    /// A head appeared or went away since the last `done` event.
    dirty: bool,
}

/// An output and the state that the clients know of it.
struct Head {
    output: Output,
    sent: HeadState,
}

/// The properties of a head that can change.
#[derive(Clone, PartialEq)]
struct HeadState {
    enabled: bool,
    modes: Vec<Mode>,
    mode: Option<Mode>,
    position: Point<i32, Logical>,
    transform: Transform,
    scale: f64,
}

/// A bound manager with its head objects and their mode objects.
struct Manager {
    manager: ZwlrOutputManagerV1,
    heads: Vec<(ZwlrOutputHeadV1, Vec<ZwlrOutputModeV1>)>,
}

/// A configuration that a client builds, and the serial that it is for.
pub struct Configuration {
    serial: u32,
    pending: Mutex<Pending>,
}

/// The parts of a configuration that change with its requests.
struct Pending {
    /// The client applied or tested the configuration.
    used: bool,
    /// The configuration got the `cancelled` event.
    cancelled: bool,
    /// The heads of the manager when the configuration started, with the
    /// choice of the client so far.
    heads: Vec<(ZwlrOutputHeadV1, Option<Choice>)>,
}

/// What a configuration does with a head.
enum Choice {
    Enabled(ZwlrOutputConfigurationHeadV1),
    Disabled,
}

impl HeadState {
    fn of(output: &Output, enabled: bool) -> Self {
        Self {
            enabled,
            modes: output.modes(),
            mode: output.current_mode(),
            position: output.current_location(),
            transform: output.current_transform(),
            scale: output.current_scale().fractional_scale(),
        }
    }
}

impl OutputManagement {
    /// Creates the global.
    pub fn new(display: &DisplayHandle) -> Self {
        display.create_global::<State, ZwlrOutputManagerV1, ()>(VERSION, ());
        Self {
            display: display.clone(),
            heads: Vec::new(),
            managers: Vec::new(),
            serial: SERIAL_COUNTER.next_serial().into(),
            dirty: false,
        }
    }

    /// Adds the head of a new output. [`Self::update`] sends `done`.
    pub fn add_head(&mut self, output: &Output, enabled: bool) {
        if self.heads.iter().any(|h| h.output == *output) {
            return;
        }
        let state = HeadState::of(output, enabled);
        for manager in &mut self.managers {
            manager.advertise(&self.display, output, &state);
        }
        self.heads.push(Head {
            output: output.clone(),
            sent: state,
        });
        self.dirty = true;
    }

    /// Removes the head of an output that went away. [`Self::update`] sends
    /// `done`.
    pub fn remove_head(&mut self, output: &Output) {
        self.heads.retain(|h| h.output != *output);
        for manager in &mut self.managers {
            manager.heads.retain(|(head, modes)| {
                let other = head.data::<Output>() != Some(output);
                if !other {
                    finish(head, modes);
                }
                other
            });
        }
        self.dirty = true;
    }

    /// Sends the changes of the heads to all managers, then `done`.
    /// `enabled` holds the outputs that are on.
    pub fn update(&mut self, enabled: &[Output]) {
        for head in &mut self.heads {
            let state = HeadState::of(&head.output, enabled.contains(&head.output));
            if state == head.sent {
                continue;
            }
            for manager in &mut self.managers {
                manager.send_changes(&self.display, &head.output, &head.sent, &state);
            }
            head.sent = state;
            self.dirty = true;
        }
        if std::mem::take(&mut self.dirty) {
            self.serial = SERIAL_COUNTER.next_serial().into();
            for manager in &self.managers {
                manager.manager.done(self.serial);
            }
        }
    }

    /// Advertises all heads to a new manager.
    fn bind(&mut self, manager: ZwlrOutputManagerV1) {
        let mut data = Manager {
            manager,
            heads: Vec::new(),
        };
        for head in &self.heads {
            data.advertise(&self.display, &head.output, &head.sent);
        }
        data.manager.done(self.serial);
        self.managers.push(data);
    }

    /// The head objects of a manager.
    fn heads_of(&self, manager: &ZwlrOutputManagerV1) -> Vec<ZwlrOutputHeadV1> {
        self.managers
            .iter()
            .filter(|m| m.manager == *manager)
            .flat_map(|m| m.heads.iter().map(|(head, _)| head.clone()))
            .collect()
    }

    /// Whether a configuration takes requests: it is unused, not
    /// cancelled, and for the current serial. A configuration for an old
    /// serial gets `cancelled`; a used one is a protocol error.
    fn usable(
        &self,
        object: &ZwlrOutputConfigurationV1,
        serial: u32,
        pending: &mut Pending,
    ) -> bool {
        if pending.used {
            object.post_error(
                config::Error::AlreadyUsed,
                "the configuration was applied or tested",
            );
            return false;
        }
        if !pending.cancelled && serial != self.serial {
            object.cancelled();
            pending.cancelled = true;
        }
        !pending.cancelled
    }
}

impl Manager {
    /// Creates the head object of an output and sends all its properties.
    fn advertise(&mut self, display: &DisplayHandle, output: &Output, state: &HeadState) {
        let Some(client) = self.manager.client() else {
            return;
        };
        let version = self.manager.version();
        let Ok(head) =
            client.create_resource::<ZwlrOutputHeadV1, _, State>(display, version, output.clone())
        else {
            return;
        };
        self.manager.head(&head);
        head.name(output.name());
        head.description(output.description());
        send_identity(&head, &output.physical_properties());
        let preferred = output.preferred_mode();
        let modes: Vec<_> = state
            .modes
            .iter()
            .filter_map(|m| new_mode(display, &head, *m, preferred == Some(*m)))
            .collect();
        send_state(&head, &modes, state, None);
        self.heads.push((head, modes));
    }

    /// Sends the changes of an output to its head objects.
    fn send_changes(
        &mut self,
        display: &DisplayHandle,
        output: &Output,
        old: &HeadState,
        new: &HeadState,
    ) {
        let preferred = output.preferred_mode();
        for (head, modes) in &mut self.heads {
            if head.data::<Output>() == Some(output) {
                sync_modes(display, head, modes, &new.modes, preferred);
                send_state(head, modes, new, Some(old));
            }
        }
    }
}

/// Sends the physical size, the make, the model and the serial number,
/// where they are known.
fn send_identity(head: &ZwlrOutputHeadV1, properties: &PhysicalProperties) {
    let size = properties.size;
    if size.w > 0 && size.h > 0 {
        head.physical_size(size.w, size.h);
    }
    if head.version() < zwlr_output_head_v1::EVT_MAKE_SINCE {
        return;
    }
    if !properties.make.is_empty() {
        head.make(properties.make.clone());
    }
    if !properties.model.is_empty() {
        head.model(properties.model.clone());
    }
    if !properties.serial_number.is_empty() {
        head.serial_number(properties.serial_number.clone());
    }
}

/// Creates a mode object of a head and sends its properties.
fn new_mode(
    display: &DisplayHandle,
    head: &ZwlrOutputHeadV1,
    mode: Mode,
    preferred: bool,
) -> Option<ZwlrOutputModeV1> {
    let client = head.client()?;
    let object = client
        .create_resource::<ZwlrOutputModeV1, _, State>(display, head.version(), mode)
        .ok()?;
    // The mode event comes first, because it creates the object for the
    // client. wlr-randr shows 0x0 modes otherwise.
    head.mode(&object);
    object.size(mode.size.w, mode.size.h);
    if mode.refresh > 0 {
        object.refresh(mode.refresh);
    }
    if preferred {
        object.preferred();
    }
    Some(object)
}

/// Finishes the mode objects that went away and creates the new ones.
fn sync_modes(
    display: &DisplayHandle,
    head: &ZwlrOutputHeadV1,
    objects: &mut Vec<ZwlrOutputModeV1>,
    modes: &[Mode],
    preferred: Option<Mode>,
) {
    objects.retain(|object| {
        let kept = object.data::<Mode>().is_some_and(|m| modes.contains(m));
        if !kept {
            object.finished();
        }
        kept
    });
    for mode in modes {
        if !objects.iter().any(|o| o.data::<Mode>() == Some(mode)) {
            objects.extend(new_mode(display, head, *mode, preferred == Some(*mode)));
        }
    }
}

/// Sends the properties of a head that differ from `old`, or all of them
/// without `old`.
fn send_state(
    head: &ZwlrOutputHeadV1,
    modes: &[ZwlrOutputModeV1],
    new: &HeadState,
    old: Option<&HeadState>,
) {
    if old.is_none_or(|o| o.enabled != new.enabled) {
        head.enabled(i32::from(new.enabled));
    }
    // The other properties count only while the head is on; a head that
    // turns on gets all of them.
    if new.enabled {
        send_properties(head, modes, new, old.filter(|o| o.enabled));
    }
}

/// Sends the current mode, the position, the transform and the scale of a
/// head that is on.
fn send_properties(
    head: &ZwlrOutputHeadV1,
    modes: &[ZwlrOutputModeV1],
    new: &HeadState,
    old: Option<&HeadState>,
) {
    let current = new
        .mode
        .filter(|m| old.is_none_or(|o| o.mode != Some(*m)))
        .and_then(|m| modes.iter().find(|o| o.data::<Mode>() == Some(&m)));
    if let Some(mode) = current {
        head.current_mode(mode);
    }
    if old.is_none_or(|o| o.position != new.position) {
        head.position(new.position.x, new.position.y);
    }
    if old.is_none_or(|o| o.transform != new.transform) {
        head.transform(new.transform.into());
    }
    if old.is_none_or(|o| o.scale.to_bits() != new.scale.to_bits()) {
        head.scale(new.scale);
    }
}

/// Tells a client that a head and its modes went away.
fn finish(head: &ZwlrOutputHeadV1, modes: &[ZwlrOutputModeV1]) {
    for mode in modes {
        mode.finished();
    }
    head.finished();
}

impl Pending {
    /// The configuration of each output, or `None` while a head has no
    /// choice.
    fn config(&self) -> Option<Vec<(Output, OutputConfig)>> {
        self.heads
            .iter()
            .map(|(head, choice)| {
                let config = match choice.as_ref()? {
                    Choice::Disabled => OutputConfig::Disabled,
                    Choice::Enabled(object) => OutputConfig::Enabled(
                        object
                            .data::<Mutex<OutputChanges>>()?
                            .lock()
                            .expect("not poisoned")
                            .clone(),
                    ),
                };
                Some((head.data::<Output>()?.clone(), config))
            })
            .collect()
    }
}

impl State {
    /// Records the choice of a client for one head of a configuration.
    fn choose(
        &self,
        object: &ZwlrOutputConfigurationV1,
        configuration: &Configuration,
        head: &ZwlrOutputHeadV1,
        choice: Choice,
    ) {
        let mut pending = configuration.pending.lock().expect("not poisoned");
        if !self
            .output_management
            .usable(object, configuration.serial, &mut pending)
        {
            return;
        }
        match pending.heads.iter_mut().find(|(h, _)| h == head) {
            Some((_, slot @ None)) => *slot = Some(choice),
            Some(_) => object.post_error(
                config::Error::AlreadyConfiguredHead,
                "the head is already configured",
            ),
            // A head of another manager.
            None => {}
        }
    }

    /// Tests a configuration, and applies it if `apply`, then sends the
    /// result.
    fn finish_configuration(
        &mut self,
        object: &ZwlrOutputConfigurationV1,
        configuration: &Configuration,
        apply: bool,
    ) {
        let config = {
            let mut pending = configuration.pending.lock().expect("not poisoned");
            if !self
                .output_management
                .usable(object, configuration.serial, &mut pending)
            {
                return;
            }
            pending.used = true;
            pending.config()
        };
        let Some(config) = config else {
            object.post_error(config::Error::UnconfiguredHead, "a head is not configured");
            return;
        };
        match self.configure_outputs(config, apply) {
            Ok(()) => {
                object.succeeded();
                if apply {
                    self.outputs_configured();
                }
            }
            Err(err) => {
                tracing::info!(%err, apply, "an output configuration failed");
                object.failed();
            }
        }
    }
}

impl GlobalDispatch<ZwlrOutputManagerV1, ()> for State {
    fn bind(
        state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<ZwlrOutputManagerV1>,
        (): &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        let manager = data_init.init(resource, ());
        state.output_management.bind(manager);
    }
}

impl Dispatch<ZwlrOutputManagerV1, ()> for State {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &ZwlrOutputManagerV1,
        request: zwlr_output_manager_v1::Request,
        (): &(),
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            zwlr_output_manager_v1::Request::CreateConfiguration { id, serial } => {
                let heads = state.output_management.heads_of(resource);
                let cancelled = serial != state.output_management.serial;
                let configuration = Configuration {
                    serial,
                    pending: Mutex::new(Pending {
                        used: false,
                        cancelled,
                        heads: heads.into_iter().map(|h| (h, None)).collect(),
                    }),
                };
                let object = data_init.init(id, configuration);
                if cancelled {
                    object.cancelled();
                }
            }
            zwlr_output_manager_v1::Request::Stop => {
                state
                    .output_management
                    .managers
                    .retain(|m| m.manager != *resource);
                resource.finished();
            }
            _ => {}
        }
    }

    fn destroyed(state: &mut Self, _client: ClientId, resource: &ZwlrOutputManagerV1, (): &()) {
        state
            .output_management
            .managers
            .retain(|m| m.manager != *resource);
    }
}

impl Dispatch<ZwlrOutputHeadV1, Output> for State {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &ZwlrOutputHeadV1,
        request: zwlr_output_head_v1::Request,
        _output: &Output,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        if let zwlr_output_head_v1::Request::Release = request {
            forget_head(state, resource);
        }
    }

    fn destroyed(
        state: &mut Self,
        _client: ClientId,
        resource: &ZwlrOutputHeadV1,
        _output: &Output,
    ) {
        forget_head(state, resource);
    }
}

/// Forgets a head object that its client released.
fn forget_head(state: &mut State, head: &ZwlrOutputHeadV1) {
    for manager in &mut state.output_management.managers {
        manager.heads.retain(|(h, _)| h != head);
    }
}

impl Dispatch<ZwlrOutputModeV1, Mode> for State {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &ZwlrOutputModeV1,
        request: zwlr_output_mode_v1::Request,
        _mode: &Mode,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        if let zwlr_output_mode_v1::Request::Release = request {
            forget_mode(state, resource);
        }
    }

    fn destroyed(state: &mut Self, _client: ClientId, resource: &ZwlrOutputModeV1, _mode: &Mode) {
        forget_mode(state, resource);
    }
}

/// Forgets a mode object that its client released.
fn forget_mode(state: &mut State, mode: &ZwlrOutputModeV1) {
    for manager in &mut state.output_management.managers {
        for (_, modes) in &mut manager.heads {
            modes.retain(|m| m != mode);
        }
    }
}

impl Dispatch<ZwlrOutputConfigurationV1, Configuration> for State {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &ZwlrOutputConfigurationV1,
        request: config::Request,
        configuration: &Configuration,
        _handle: &DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            config::Request::EnableHead { id, head } => {
                let object = data_init.init(id, Mutex::new(OutputChanges::default()));
                state.choose(resource, configuration, &head, Choice::Enabled(object));
            }
            config::Request::DisableHead { head } => {
                state.choose(resource, configuration, &head, Choice::Disabled);
            }
            config::Request::Apply => state.finish_configuration(resource, configuration, true),
            config::Request::Test => state.finish_configuration(resource, configuration, false),
            _ => {}
        }
    }
}

impl Dispatch<ZwlrOutputConfigurationHeadV1, Mutex<OutputChanges>> for State {
    fn request(
        _state: &mut Self,
        _client: &Client,
        resource: &ZwlrOutputConfigurationHeadV1,
        request: config_head::Request,
        changes: &Mutex<OutputChanges>,
        _handle: &DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        let mut changes = changes.lock().expect("not poisoned");
        match request {
            config_head::Request::SetMode { mode } => {
                if let Some(mode) = mode.data::<Mode>() {
                    set_once(resource, &mut changes.mode, ModeChoice::Listed(*mode));
                }
            }
            config_head::Request::SetCustomMode {
                width,
                height,
                refresh,
            } => set_custom_mode(resource, &mut changes, (width, height, refresh)),
            config_head::Request::SetPosition { x, y } => {
                set_once(resource, &mut changes.position, (x, y).into());
            }
            config_head::Request::SetTransform { transform } => {
                set_transform(resource, &mut changes, transform);
            }
            config_head::Request::SetScale { scale } => set_scale(resource, &mut changes, scale),
            _ => {}
        }
    }
}

/// Sets a property of a head configuration. A second time is a protocol
/// error.
fn set_once<T>(object: &ZwlrOutputConfigurationHeadV1, slot: &mut Option<T>, value: T) {
    if slot.is_some() {
        object.post_error(
            config_head::Error::AlreadySet,
            "the property is already set",
        );
    } else {
        *slot = Some(value);
    }
}

/// Sets a custom mode: a size and a refresh rate, or 0 for any rate.
fn set_custom_mode(
    object: &ZwlrOutputConfigurationHeadV1,
    changes: &mut OutputChanges,
    (width, height, refresh): (i32, i32, i32),
) {
    if width <= 0 || height <= 0 || refresh < 0 {
        object.post_error(config_head::Error::InvalidCustomMode, "the mode is invalid");
        return;
    }
    let choice = ModeChoice::Custom {
        size: (width, height).into(),
        refresh,
    };
    set_once(object, &mut changes.mode, choice);
}

/// Sets a transform of the enum of `wl_output`.
fn set_transform(
    object: &ZwlrOutputConfigurationHeadV1,
    changes: &mut OutputChanges,
    transform: WEnum<wl_output::Transform>,
) {
    match transform {
        WEnum::Value(transform) => set_once(object, &mut changes.transform, transform.into()),
        WEnum::Unknown(value) => object.post_error(
            config_head::Error::InvalidTransform,
            format!("the transform {value} is unknown"),
        ),
    }
}

/// Sets a scale above 0.
fn set_scale(object: &ZwlrOutputConfigurationHeadV1, changes: &mut OutputChanges, scale: f64) {
    if scale > 0.0 {
        set_once(object, &mut changes.scale, scale);
    } else {
        object.post_error(
            config_head::Error::InvalidScale,
            "the scale is not positive",
        );
    }
}
