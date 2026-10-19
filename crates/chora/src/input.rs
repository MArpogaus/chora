//! Input routing: the safety layer first, then grabs, then a node with the
//! keyboard focus, then the focused surface.
//!
//! Plugins set the focus. The core changes it only in the cases that the
//! input flow of `docs/architecture.md` lists. Protocols take the keyboard
//! for a while: a popup grab, a layer surface such as a launcher, and the
//! lock screen.

use std::{sync::atomic::Ordering, time::Duration};

use chora_scene::{Binding, Event as SceneEvent, NodeKind, ScrollSource, ipc::PressState};
use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        InputTime, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent,
    },
    input::{
        keyboard::{FilterResult, Keycode, Keysym, KeysymHandle, ModifiersState, xkb},
        pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerHandle, RelativeMotionEvent},
    },
    output::Output,
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point, SERIAL_COUNTER, Serial},
    wayland::pointer_constraints::{PointerConstraint, with_pointer_constraint},
};

use crate::{
    output::{Spot, output_area},
    render,
    state::{
        ButtonGrab, ClientState, HeldGrab, NodeGrab, REPEAT_DELAY, REPEAT_RATE, Repeat, State,
        Suppressed,
    },
};

/// What a key press does besides reaching a client.
enum KeyAction {
    /// The release of a key whose press went elsewhere.
    Swallow,
    /// Stop the compositor.
    Quit,
    /// The emergency reset.
    Reset,
    /// Switch to another virtual terminal.
    SwitchVt(i32),
    /// Send `input.key` to the client that grabbed the binding.
    Grab(u64, Binding),
    /// Send a key to the owner of the node with the keyboard focus.
    Node(String, SceneEvent),
}

pub fn process<I: InputBackend>(state: &mut State, event: InputEvent<I>) {
    let seat = state.seat.clone();
    state.idle_notifier_state.notify_activity(&seat);
    match event {
        InputEvent::Keyboard { event } => keyboard::<I>(state, &event),
        InputEvent::PointerMotion { event } => relative_motion::<I>(state, &event),
        InputEvent::PointerMotionAbsolute { event } => motion::<I>(state, &event),
        InputEvent::PointerButton { event } => button::<I>(state, &event),
        InputEvent::PointerAxis { event } => axis::<I>(state, &event),
        _ => {}
    }
}

fn keyboard<I: InputBackend>(state: &mut State, event: &I::KeyboardKeyEvent) {
    key(state, event.key_code(), event.state(), event.time(), false);
}

/// Logs the layouts of the keymap of the seat, so that a wrong layout
/// shows.
pub fn log_layout(state: &mut State) {
    let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
    let layouts = keyboard.with_xkb_state(state, |context| {
        let xkb = context.xkb().lock().ok()?;
        let names: Vec<_> = xkb
            .layouts()
            .map(|l| xkb.layout_name(l).to_owned())
            .collect();
        Some(names.join(", "))
    });
    tracing::info!(
        layouts = layouts.as_deref().unwrap_or("?"),
        "the keyboard layouts; XKB_DEFAULT_LAYOUT sets others"
    );
}

/// Releases every key that is down, for example when the window of a
/// nested chora loses the focus and the host keeps the releases.
pub fn release_keys(state: &mut State) {
    let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
    for code in keyboard.pressed_keys() {
        key(state, code, KeyState::Released, InputTime::now(), false);
    }
}

/// A key goes down or up: the safety layer first, then the grabs, then a
/// node with the keyboard focus, then the focused surface. An injected key never reaches the safety layer (D23).
pub fn key(state: &mut State, code: Keycode, key_state: KeyState, time: InputTime, injected: bool) {
    let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
    // A grab that a client took in this loop iteration, for example of an
    // input method, must not see a key of the lock screen.
    if state.lock.is_some() && keyboard.is_grabbed() {
        keyboard.unset_grab(state);
    }
    let serial = SERIAL_COUNTER.next_serial();
    let action = keyboard.input(
        state,
        code,
        key_state,
        serial,
        time,
        |state, mods, handle| {
            if key_state == KeyState::Released {
                released_key(state, code, mods, &handle)
            } else {
                pressed_key(state, code, mods, &handle, injected)
            }
        },
    );
    match action {
        Some(KeyAction::Quit) => state.loop_signal.stop(),
        Some(KeyAction::Reset) => state.reset(),
        Some(KeyAction::SwitchVt(vt)) => {
            if let Some(backend) = state.backend.as_mut() {
                backend.change_vt(vt);
            }
        }
        Some(KeyAction::Grab(conn, binding)) => {
            grabbed(state, conn, &binding, PressState::Pressed);
            if !state
                .held
                .grabs
                .iter()
                .any(|h| h.conn == conn && h.binding == binding)
            {
                state.held.grabs.push(HeldGrab {
                    conn,
                    binding,
                    code,
                });
            }
        }
        Some(KeyAction::Node(owner, event)) => state.ipc.send_to_client(&owner, &event),
        Some(KeyAction::Swallow) => {}
        None => note_input(
            state,
            keyboard.current_focus(),
            serial,
            key_state == KeyState::Pressed,
        ),
    }
    if key_state == KeyState::Released {
        release_grabs(state, &keyboard.modifier_state());
    }
}

/// Where the release of a key goes: to the surface, unless its press went
/// elsewhere.
fn released_key(
    state: &mut State,
    code: Keycode,
    mods: &ModifiersState,
    handle: &KeysymHandle<'_>,
) -> FilterResult<KeyAction> {
    if state.held.repeat.as_ref().is_some_and(|r| r.code == code) {
        state.held.repeat = None;
    }
    // A grab without Ctrl, Alt or Super ends with its own key.
    let (free, held_grabs) = std::mem::take(&mut state.held.grabs)
        .into_iter()
        .partition(|h| h.code == code && free_binding(&h.binding));
    state.held.grabs = held_grabs;
    for grab in free {
        grabbed(state, grab.conn, &grab.binding, PressState::Released);
    }
    let Some(i) = state.held.keys.iter().position(|(k, _)| *k == code) else {
        return FilterResult::Forward;
    };
    FilterResult::Intercept(match state.held.keys.remove(i).1 {
        Suppressed::Node(owner, node) => {
            KeyAction::Node(owner, node_key(node, mods, handle, PressState::Released))
        }
        Suppressed::Swallow => KeyAction::Swallow,
    })
}

/// Where the press of a key goes: the safety layer, a grab, a node with the
/// keyboard focus, or the surface. The core notes where its release goes.
fn pressed_key(
    state: &mut State,
    code: Keycode,
    mods: &ModifiersState,
    handle: &KeysymHandle<'_>,
    injected: bool,
) -> FilterResult<KeyAction> {
    // Another key ends the repeat of a node key.
    state.held.repeat = None;
    let syms = (handle.raw_syms(), handle.modified_syms());
    let (action, suppressed) = if let Some(action) = press_action(state, mods, &syms, injected) {
        (action, Suppressed::Swallow)
    } else if let Some((node, owner)) = state.key_node() {
        let event = node_key(node.clone(), mods, handle, PressState::Pressed);
        if !handle.modified_sym().is_modifier_key() {
            let next = state.time.now() + Duration::from_millis(REPEAT_DELAY);
            let _ = state.expire_at(next);
            state.held.repeat = Some(Repeat {
                code,
                owner: owner.clone(),
                event: event.clone(),
                next,
            });
        }
        (
            KeyAction::Node(owner.clone(), event),
            Suppressed::Node(owner, node),
        )
    } else {
        return FilterResult::Forward;
    };
    // A repeated press without a release keeps one entry.
    state.held.keys.retain(|(k, _)| *k != code);
    state.held.keys.push((code, suppressed));
    FilterResult::Intercept(action)
}

/// Tells the client of a grab that its key went down or up.
fn grabbed(state: &mut State, conn: u64, binding: &Binding, press: PressState) {
    let event = chora_scene::Event::InputKey {
        binding: binding.to_string(),
        state: press,
    };
    state.ipc.send_to(conn, &event);
}

/// True for a binding without Ctrl, Alt or Super. It needs the capability
/// `input.grab-any`, and it ends with its own key, not with a modifier.
pub fn free_binding(binding: &Binding) -> bool {
    !(binding.mods.ctrl || binding.mods.alt || binding.mods.logo)
}

/// Ends the held key grabs of which Ctrl, Alt or Super went up, each with
/// one `released`, so that an Alt+Tab switcher learns when Alt went up.
/// Shift does not end a grab: Alt+Shift+Tab steps back while Alt stays.
fn release_grabs(state: &mut State, mods: &ModifiersState) {
    let (ended, held) = std::mem::take(&mut state.held.grabs)
        .into_iter()
        .partition(|h| {
            let m = h.binding.mods;
            (m.ctrl && !mods.ctrl) || (m.alt && !mods.alt) || (m.logo && !mods.logo)
        });
    state.held.grabs = held;
    let ended: Vec<HeldGrab> = ended;
    for grab in ended {
        grabbed(state, grab.conn, &grab.binding, PressState::Released);
    }
}

impl State {
    /// Takes the grabs without Ctrl, Alt or Super away from a client name:
    /// a held one ends with one `released` and the registrations go with
    /// the capability `input.grab-any` that allowed them.
    pub fn revoke_free_grabs(&mut self, name: &str) {
        let conns = self.ipc.conns_of(name);
        for conn in &conns {
            self.ipc.drop_free_grabs(*conn);
        }
        let (ends, keep) = std::mem::take(&mut self.held.grabs)
            .into_iter()
            .partition(|h| conns.contains(&h.conn) && free_binding(&h.binding));
        self.held.grabs = keep;
        for grab in ends {
            grabbed(self, grab.conn, &grab.binding, PressState::Released);
        }
    }

    /// Sends the held key of a node again when its time came, as a client
    /// repeats a held key of `wl_keyboard`. The repeat ends with the focus
    /// of the node.
    pub fn repeat_key(&mut self) {
        let Some(repeat) = self.held.repeat.take() else {
            return;
        };
        let SceneEvent::NodeKey { node, .. } = &repeat.event else {
            return;
        };
        if self.key_node() != Some((node.clone(), repeat.owner.clone())) {
            return;
        }
        if repeat.next > self.time.now() {
            self.held.repeat = Some(repeat);
            return;
        }
        self.ipc.send_to_client(&repeat.owner, &repeat.event);
        let next = repeat.next + Duration::from_millis(1000 / REPEAT_RATE);
        let _ = self.expire_at(next);
        self.held.repeat = Some(Repeat { next, ..repeat });
    }
}

/// The `node.key` event of a key for the node with the keyboard focus.
fn node_key(
    node: String,
    mods: &ModifiersState,
    handle: &KeysymHandle<'_>,
    state: PressState,
) -> SceneEvent {
    let sym = handle.modified_sym();
    let plain = !(mods.ctrl || mods.alt || mods.logo);
    let text = sym
        .key_char()
        .filter(|c| plain && state == PressState::Pressed && !c.is_control())
        .map(String::from);
    SceneEvent::NodeKey {
        node,
        key: xkb::keysym_get_name(sym),
        code: handle.raw_code().raw().saturating_sub(8),
        state,
        modifiers: modifier_names(mods),
        text,
    }
}

/// The names of the modifiers of the keyboard that are down.
fn modifiers(state: &State) -> Vec<String> {
    let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
    modifier_names(&keyboard.modifier_state())
}

/// The names of the modifiers that are down, as bindings name them.
pub fn modifier_names(mods: &ModifiersState) -> Vec<String> {
    [
        (mods.ctrl, "Ctrl"),
        (mods.alt, "Alt"),
        (mods.shift, "Shift"),
        (mods.logo, "Super"),
    ]
    .into_iter()
    .filter(|(on, _)| *on)
    .map(|(_, name)| name.to_owned())
    .collect()
}

/// The fixed bindings of the safety layer (D23), then the grabs. The keymap
/// turns Ctrl+Alt+F1 to F12 into the keys that switch the VT; only the
/// modified keysyms show them.
fn press_action(
    state: &State,
    mods: &ModifiersState,
    (syms, modified): &(Vec<Keysym>, Vec<Keysym>),
    injected: bool,
) -> Option<KeyAction> {
    if injected {
        return grab_action(state, mods, syms);
    }
    let first = Keysym::XF86_Switch_VT_1.raw();
    if let Some(vt) = modified
        .iter()
        .find_map(|s| s.raw().checked_sub(first).filter(|n| *n < 12))
    {
        return Some(KeyAction::SwitchVt(i32::try_from(vt).ok()? + 1));
    }
    if mods.ctrl && mods.alt {
        if syms.contains(&Keysym::BackSpace) {
            return Some(KeyAction::Quit);
        }
        if syms.contains(&Keysym::Escape) {
            return Some(KeyAction::Reset);
        }
    }
    grab_action(state, mods, syms)
}

/// The grab of a binding, unless the session is locked.
fn grab_action(state: &State, mods: &ModifiersState, syms: &[Keysym]) -> Option<KeyAction> {
    if state.lock.is_some() {
        return None;
    }
    let (conn, binding) = state.ipc.grab_for(mods, syms)?;
    Some(KeyAction::Grab(conn, binding))
}

/// Relative motion, as from a mouse on real hardware. The pointer stays on
/// the outputs.
fn relative_motion<I: InputBackend>(state: &mut State, event: &I::PointerMotionEvent) {
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    // A grabbed button keeps the motion from the surfaces.
    let under = state
        .spot(state.pointer_layout)
        .filter(|_| state.button_grab.is_none())
        .and_then(|spot| render::surface_under(state, &spot));
    let focus = under.as_ref().map(|u| (u.surface.clone(), u.location));
    let relative = RelativeMotionEvent {
        delta: event.delta(),
        delta_unaccel: event.delta_unaccel(),
        time: event.time(),
    };
    pointer.relative_motion(state, focus, &relative);
    let target = state.clamp_to_outputs(state.pointer_layout + event.delta());
    let local = under
        .as_ref()
        .map(|u| pointer.current_location() - u.location);
    let constraint = under
        .as_ref()
        .zip(local)
        .and_then(|(u, local)| active_constraint(&pointer, &u.surface, local));
    match constraint {
        // A locked pointer only reports relative motion, as games want.
        Some(Constraint::Locked) => pointer.frame(state),
        Some(Constraint::Confined) if leaves(state, target, under.as_ref()) => {
            pointer.frame(state);
        }
        _ => move_pointer(state, target, event.time()),
    }
}

/// A pointer constraint of `pointer-constraints`.
enum Constraint {
    Locked,
    Confined,
}

/// The active constraint of a surface, if the pointer is in its region.
fn active_constraint(
    pointer: &PointerHandle<State>,
    surface: &WlSurface,
    local: Point<f64, Logical>,
) -> Option<Constraint> {
    with_pointer_constraint(surface, pointer, |constraint| {
        let constraint = constraint.filter(|c| c.is_active())?;
        let inside = constraint
            .region()
            .is_none_or(|r| r.contains(local.to_i32_round()));
        inside.then(|| match &*constraint {
            PointerConstraint::Locked(_) => Constraint::Locked,
            PointerConstraint::Confined(_) => Constraint::Confined,
        })
    })
}

/// Whether a move to a layout point leaves the surface that confines the
/// pointer.
fn leaves(state: &State, target: Point<f64, Logical>, under: Option<&render::Under>) -> bool {
    let next = state
        .spot(target)
        .and_then(|spot| render::surface_under(state, &spot));
    next.map(|n| n.surface) != under.map(|u| u.surface.clone())
}

/// Activates the constraint of the surface under the pointer, if the
/// pointer is in its region.
fn activate_constraint(
    pointer: &PointerHandle<State>,
    surface: &WlSurface,
    local: Point<f64, Logical>,
) {
    with_pointer_constraint(surface, pointer, |constraint| {
        let waiting = constraint.filter(|c| !c.is_active())?;
        let inside = waiting
            .region()
            .is_none_or(|r| r.contains(local.to_i32_round()));
        inside.then(|| waiting.activate())
    });
}

/// Absolute motion, as from the winit window, which shows the first output.
/// The renderer shows the output through the inverse of its transform, so
/// the transform maps a point of the window back to the output.
fn motion<I: InputBackend>(state: &mut State, event: &I::PointerMotionAbsoluteEvent) {
    let Some(output) = state.outputs.first() else {
        return;
    };
    let area = output_area(output);
    let transform = output.current_transform();
    let window = transform.transform_size(area.size);
    let local = transform.transform_point_in(event.position_transformed(window), &window.to_f64());
    // A drag past the edge of the window reports points outside of it.
    let layout = state.clamp_to_outputs(area.loc.to_f64() + local);
    move_pointer(state, layout, event.time());
}

impl State {
    /// Tells the surface under the pointer after the scene moved under it,
    /// for example after a transaction or a camera change.
    pub fn refresh_pointer(&mut self) {
        move_pointer(self, self.pointer_layout, InputTime::now());
    }
}

/// Moves the pointer to a point of the layout and tells the surface under
/// it.
pub fn move_pointer(state: &mut State, layout: Point<f64, Logical>, time: InputTime) {
    let moved = state.pointer_layout != layout;
    state.pointer_layout = layout;
    let Some(spot) = state.spot(layout) else {
        return;
    };
    if state.node_grab.is_none() && state.button_grab.is_none() {
        hover(state, &spot, moved);
    }
    if let Some(grab) = state.button_grab.as_ref().filter(|_| moved) {
        let (binding, conn) = (grab.binding.clone(), grab.conn);
        let event = SceneEvent::InputMoved {
            binding,
            position: [spot.scene.x, spot.scene.y],
            output: spot.output.name(),
            local: [spot.local.x, spot.local.y],
        };
        state.ipc.send_to(conn, &event);
    }
    // The drag of a node goes to plugins only, with each new position.
    let under = if let Some(grab) = &mut state.node_grab {
        let position = position_for(&state.scene, &state.outputs, &grab.node, &spot);
        #[allow(clippy::float_cmp)] // the same input gives the same numbers
        if grab.position != position {
            grab.position = position;
            let node = grab.node.clone();
            let modifiers = modifiers(state);
            state.emit(&SceneEvent::PointerMoved {
                node,
                position,
                modifiers,
            });
        }
        None
    } else if state.button_grab.is_some() {
        None
    } else {
        render::surface_under(state, &spot).map(|u| (u.surface, u.location))
    };
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    pointer.motion(
        state,
        under.clone(),
        &MotionEvent {
            location: spot.scene,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );
    pointer.frame(state);
    if let Some((surface, location)) = under.filter(|(s, _)| state.focused_or_unnamed(s)) {
        activate_constraint(&pointer, &surface, spot.scene - location);
    }
}

fn button<I: InputBackend>(state: &mut State, event: &I::PointerButtonEvent) {
    press_button(state, event.button_code(), event.state(), event.time());
}

/// A pointer button goes down or up.
pub fn press_button(state: &mut State, code: u32, button_state: ButtonState, time: InputTime) {
    if grab_button(state, code, button_state, time) {
        return;
    }
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    let serial = SERIAL_COUNTER.next_serial();
    if button_state == ButtonState::Pressed {
        click_focus(state);
        note_input(state, pointer.current_focus(), serial, true);
        press_node(state, code);
    } else {
        note_input(state, pointer.current_focus(), serial, false);
        release_node(state, code);
    }
    pointer.button(
        state,
        &ButtonEvent {
            button: code,
            state: button_state,
            serial,
            time,
        },
    );
    pointer.frame(state);
}

/// A button with modifiers that a client grabbed: the press, the moves and
/// the release go to the client of the grab and to no surface. The release
/// comes also when the modifiers went up first. Returns whether the grab
/// took the button.
fn grab_button(state: &mut State, code: u32, button_state: ButtonState, time: InputTime) -> bool {
    let grab = if button_state == ButtonState::Pressed {
        if state.button_grab.is_some() || state.lock.is_some() {
            return false;
        }
        let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
        let mods = keyboard.modifier_state();
        let Some((conn, binding)) = state.ipc.button_grab_for(&mods, code) else {
            return false;
        };
        let binding = binding.to_string();
        state.button_grab = Some(ButtonGrab {
            conn,
            binding: binding.clone(),
            button: code,
        });
        (conn, binding, PressState::Pressed)
    } else {
        let Some(grab) = state.button_grab.take_if(|g| g.button == code) else {
            return false;
        };
        (grab.conn, grab.binding, PressState::Released)
    };
    let Some(spot) = state.spot(state.pointer_layout) else {
        return true;
    };
    let (conn, binding, press) = grab;
    let event = SceneEvent::InputButton {
        binding,
        state: press,
        position: [spot.scene.x, spot.scene.y],
        output: spot.output.name(),
        local: [spot.local.x, spot.local.y],
    };
    state.ipc.send_to(conn, &event);
    // The surface under the pointer gets a leave, and an enter after the
    // release.
    move_pointer(state, state.pointer_layout, time);
    true
}

/// Scrolls by logical pixels, as a touchpad does.
pub fn scroll(state: &mut State, dx: f64, dy: f64, time: InputTime) {
    scroll_node(state, [dx, dy], [0.0; 2], AxisSource::Continuous);
    let mut frame = AxisFrame::new(time).source(AxisSource::Continuous);
    for (axis, amount) in [(Axis::Horizontal, dx), (Axis::Vertical, dy)] {
        if amount != 0.0 {
            frame = frame.value(axis, amount);
        }
    }
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    pointer.axis(state, frame);
    pointer.frame(state);
}

/// Tells plugins when the pointer comes over another node.
fn hover(state: &mut State, spot: &Spot, moved: bool) {
    let node = render::node_under(state, spot);
    if node == state.hovered {
        return;
    }
    if let Some(node) = std::mem::replace(&mut state.hovered, node.clone()) {
        state.emit(&SceneEvent::PointerLeft { node });
    }
    if let Some(node) = node {
        state.emit(&SceneEvent::PointerEntered { node, moved });
    }
}

/// Tells plugins about a press over a node. Over a rect or an image, the
/// moves and the release of the button follow.
fn press_node(state: &mut State, button: u32) {
    if state.node_grab.is_some() {
        return;
    }
    let Some(spot) = state.spot(state.pointer_layout) else {
        return;
    };
    let Some(node) = render::node_under(state, &spot) else {
        return;
    };
    let area = state
        .scene
        .node(&node)
        .is_some_and(|n| n.kind != NodeKind::Surface);
    let position = position_for(&state.scene, &state.outputs, &node, &spot);
    state.emit(&SceneEvent::PointerPressed {
        node: node.clone(),
        button,
        position,
        modifiers: modifiers(state),
    });
    if area {
        state.node_grab = Some(NodeGrab {
            node,
            button,
            position,
        });
    }
}

/// The position of the pointer for a node: under the overlay of an output
/// in the coordinates of that output, also while a drag crosses to another
/// one, and in scene coordinates elsewhere.
fn position_for(
    scene: &chora_scene::Scene,
    outputs: &[Output],
    node: &str,
    spot: &Spot,
) -> [f64; 2] {
    let overlay = outputs
        .iter()
        .find(|o| scene.is_within(node, &chora_scene::overlay_id(&o.name())));
    let at = overlay.map_or(spot.scene, |o| {
        spot.local + (output_area(&spot.output).loc - output_area(o).loc).to_f64()
    });
    [at.x, at.y]
}

/// Ends the drag of a node when its button goes up.
fn release_node(state: &mut State, button: u32) {
    let Some(NodeGrab { node, .. }) = state.node_grab.take_if(|g| g.button == button) else {
        return;
    };
    let position = state.spot(state.pointer_layout).map_or([0.0; 2], |s| {
        position_for(&state.scene, &state.outputs, &node, &s)
    });
    let modifiers = modifiers(state);
    state.emit(&SceneEvent::PointerReleased {
        node,
        button,
        position,
        modifiers,
    });
}

/// Notes the serial of a key or button event for the client that gets it.
/// Its popups may grab the input with the serial of its last press or of
/// its last event: toolkits name the newest event, which for a menu that
/// opens from the keyboard is the release of the key.
fn note_input(state: &State, surface: Option<WlSurface>, serial: Serial, press: bool) {
    let client = surface.and_then(|s| state.display_handle.get_client(s.id()).ok());
    if let Some(data) = client.as_ref().and_then(|c| c.get_data::<ClientState>()) {
        if press {
            data.last_press.store(u32::from(serial), Ordering::Relaxed);
        }
        data.last_input.store(u32::from(serial), Ordering::Relaxed);
    }
}

/// The focus changes of a click. A layer surface that accepts the keyboard,
/// for example the search field of a panel, keeps it until a click
/// elsewhere. A click on a surface without an owner, for example in the
/// reset grid, focuses it. The module docs list all focus changes.
fn click_focus(state: &mut State) {
    let under = state
        .spot(state.pointer_layout)
        .and_then(|spot| render::surface_under(state, &spot));
    let layer = under
        .as_ref()
        .filter(|u| u.node.is_none())
        .and_then(|u| state.keyboard_layer(&u.surface));
    state.set_layer_focus(layer);
    let ownerless = under
        .and_then(|u| u.node)
        .filter(|id| state.scene.node(id).is_some_and(|n| n.owner.is_none()));
    if let Some(id) = ownerless {
        state.focus(Some(&id));
    }
}

/// Tells the owner of the node under the pointer about scrolling, unless
/// the node is a surface, whose client gets it through Wayland.
fn scroll_node(state: &mut State, delta: [f64; 2], discrete: [f64; 2], source: AxisSource) {
    #[allow(clippy::float_cmp)] // the end of a scroll on a touchpad is exactly 0
    if delta == [0.0; 2] && discrete == [0.0; 2] {
        return;
    }
    let Some(spot) = state.spot(state.pointer_layout) else {
        return;
    };
    let Some(node) = render::node_under(state, &spot) else {
        return;
    };
    let owner = state.scene.node(&node).and_then(|n| {
        (n.kind != NodeKind::Surface)
            .then(|| n.owner.clone())
            .flatten()
    });
    let Some(owner) = owner else {
        return;
    };
    let position = position_for(&state.scene, &state.outputs, &node, &spot);
    let event = SceneEvent::PointerScroll {
        node,
        position,
        dx: delta[0],
        dy: delta[1],
        discrete,
        source: match source {
            AxisSource::Wheel => ScrollSource::Wheel,
            AxisSource::Finger => ScrollSource::Finger,
            AxisSource::WheelTilt => ScrollSource::WheelTilt,
            AxisSource::Continuous => ScrollSource::Continuous,
        },
        modifiers: modifiers(state),
    };
    state.ipc.send_to_client(&owner, &event);
}

fn axis<I: InputBackend>(state: &mut State, event: &I::PointerAxisEvent) {
    let source = event.source();
    let mut frame = AxisFrame::new(event.time()).source(source);
    let mut delta = [0.0; 2];
    let mut steps = [0.0; 2];
    for (i, axis) in [Axis::Horizontal, Axis::Vertical].into_iter().enumerate() {
        let discrete = event.amount_v120(axis);
        let amount = event
            .amount(axis)
            .unwrap_or_else(|| discrete.unwrap_or(0.0) * 15.0 / 120.0);
        delta[i] = amount;
        steps[i] = discrete.unwrap_or(0.0) / 120.0;
        if amount != 0.0 {
            frame = frame.value(axis, amount);
            if let Some(d) = discrete {
                #[allow(clippy::cast_possible_truncation)]
                let d = d as i32;
                frame = frame.v120(axis, d);
            }
        }
        if source == AxisSource::Finger && event.amount(axis) == Some(0.0) {
            frame = frame.stop(axis);
        }
    }
    scroll_node(state, delta, steps, source);
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    pointer.axis(state, frame);
    pointer.frame(state);
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use smithay::{
        backend::input::{ButtonState, InputTime, KeyState},
        input::keyboard::Keycode,
    };

    use crate::testing::{
        Client, Core,
        wayland::{Toplevel, Wayland},
    };

    const A: u32 = 30;
    const SHIFT: u32 = 42;
    const CTRL: u32 = 29;
    const ALT: u32 = 56;
    const ESCAPE: u32 = 1;

    fn press(core: &mut Core, linux: u32, down: bool) {
        let state = if down {
            KeyState::Pressed
        } else {
            KeyState::Released
        };
        super::key(
            &mut core.state,
            Keycode::new(linux + 8),
            state,
            InputTime::now(),
            false,
        );
    }

    fn tap(core: &mut Core, linux: u32) {
        press(core, linux, true);
        press(core, linux, false);
    }

    fn apply(client: &mut Client, core: &mut Core, commands: &Value) {
        client.ok(core, "scene.apply", json!({ "commands": commands }));
    }

    /// Whether a surface has the keyboard.
    fn surface_has_keyboard(core: &Core) -> bool {
        let keyboard = core.state.seat.get_keyboard().expect("a keyboard");
        keyboard.current_focus().is_some()
    }

    /// The `params` of the events of a client after a turn of the core.
    fn events(client: &mut Client, core: &mut Core) -> Vec<Value> {
        client
            .events_now(core)
            .into_iter()
            .map(|e| e["params"].clone())
            .collect()
    }

    fn menu(xp: &mut Client, core: &mut Core) {
        rect(xp, core, "menu");
    }

    fn rect(client: &mut Client, core: &mut Core, id: &str) {
        apply(
            client,
            core,
            &json!([{"create": {"id": id, "kind": "rect", "parent": "root",
                "props": {"size": [100, 100], "color": [1, 1, 1, 1]}}}]),
        );
    }

    /// A core with an output, a window `s1` with the focus, and a plugin
    /// `xp` with a rect `menu` at the origin that subscribed to all events.
    fn window_and_menu() -> (Core, Wayland, Toplevel, Client, Client) {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "app");
        let mut wm = core.named("wm");
        let commands = json!([
            {"attach": {"surface": "s1", "parent": "root"}},
            {"focus": {"surface": "s1"}},
        ]);
        apply(&mut wm, &mut core, &commands);
        let mut xp = core.named("xp");
        menu(&mut xp, &mut core);
        xp.ok(&mut core, "events.subscribe", json!({"events": []}));
        (core, wayland, window, wm, xp)
    }

    fn focus_node(node: Option<&str>) -> Value {
        json!([{"focus_node": {"node": node}}])
    }

    #[test]
    fn a_focused_node_gets_the_keys_and_the_text_instead_of_the_surface() {
        let (mut core, _wayland, _window, mut wm, mut xp) = window_and_menu();
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        assert!(surface_has_keyboard(&core));
        apply(&mut xp, &mut core, &focus_node(Some("menu")));
        assert!(!surface_has_keyboard(&core), "the window gets a leave");
        tap(&mut core, A);
        press(&mut core, SHIFT, true);
        tap(&mut core, A);
        press(&mut core, SHIFT, false);
        let keys: Vec<Value> = events(&mut xp, &mut core)
            .into_iter()
            .filter(|e| e["event"] == "node.key")
            .collect();
        assert_eq!(
            keys[0],
            json!({"event": "node.key", "node": "menu", "key": "a", "code": A,
                   "state": "pressed", "modifiers": [], "text": "a"})
        );
        assert_eq!(keys[1]["state"], "released");
        assert_eq!(keys[1].get("text"), None);
        assert_eq!(
            (&keys[2]["key"], &keys[2].get("text")),
            (&json!("Shift_L"), &None)
        );
        assert_eq!(
            keys[3],
            json!({"event": "node.key", "node": "menu", "key": "A", "code": A,
                   "state": "pressed", "modifiers": ["Shift"], "text": "A"})
        );
        assert_eq!(keys.len(), 6);
        let wm_keys = events(&mut wm, &mut core);
        assert!(
            wm_keys.iter().all(|e| e["event"] != "node.key"),
            "only the owner: {wm_keys:?}"
        );
    }

    /// The states of the `node.key` events of a key code.
    fn node_keys(xp: &mut Client, core: &mut Core, linux: u32) -> Vec<String> {
        events(xp, core)
            .into_iter()
            .filter(|e| e["event"] == "node.key" && e["code"] == linux)
            .map(|e| e["state"].as_str().unwrap_or_default().to_owned())
            .collect()
    }

    #[test]
    fn a_held_key_repeats_for_a_node_as_for_a_surface() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        apply(&mut xp, &mut core, &focus_node(Some("menu")));
        press(&mut core, A, true);
        assert_eq!(node_keys(&mut xp, &mut core, A), ["pressed"]);
        let ms = std::time::Duration::from_millis;
        core.advance(ms(199));
        assert!(node_keys(&mut xp, &mut core, A).is_empty(), "the delay");
        core.advance(ms(1));
        assert_eq!(node_keys(&mut xp, &mut core, A), ["pressed"]);
        core.advance(ms(40));
        assert_eq!(node_keys(&mut xp, &mut core, A), ["pressed"], "25 a second");
        press(&mut core, A, false);
        core.advance(ms(400));
        assert_eq!(node_keys(&mut xp, &mut core, A), ["released"]);
        // Another key ends the repeat; a modifier does not repeat.
        press(&mut core, A, true);
        assert_eq!(node_keys(&mut xp, &mut core, A), ["pressed"]);
        press(&mut core, SHIFT, true);
        core.advance(ms(400));
        let codes: Vec<Value> = events(&mut xp, &mut core)
            .into_iter()
            .filter(|e| e["event"] == "node.key")
            .map(|e| e["code"].clone())
            .collect();
        assert_eq!(codes, [SHIFT]);
    }

    #[test]
    fn the_surface_gets_the_keys_again_after_the_focus_of_a_node() {
        let (mut core, mut wayland, _window, _wm, mut xp) = window_and_menu();
        apply(&mut xp, &mut core, &focus_node(Some("menu")));
        tap(&mut core, A);
        wayland.roundtrip(&mut core);
        assert!(wayland.key_serials().is_empty(), "the window got no key");
        apply(&mut xp, &mut core, &focus_node(None));
        assert!(surface_has_keyboard(&core), "the window gets an enter");
        tap(&mut core, A);
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.key_serials().len(), 2);
    }

    #[test]
    fn the_safety_layer_works_while_a_node_has_the_focus() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        apply(&mut xp, &mut core, &focus_node(Some("menu")));
        press(&mut core, CTRL, true);
        press(&mut core, ALT, true);
        tap(&mut core, ESCAPE);
        // A forked child of another test may hold the socket for a while,
        // so the test asks the core.
        assert!(core.state.ipc.conn_ids().is_empty(), "the reset closes all");
        assert_eq!(core.state.scene.focused_node(), None);
        assert!(surface_has_keyboard(&core));
    }

    #[test]
    fn the_focus_returns_when_the_node_or_its_client_goes() {
        let (mut core, _wayland, _window, mut wm, mut xp) = window_and_menu();
        apply(&mut xp, &mut core, &focus_node(Some("menu")));
        apply(&mut xp, &mut core, &json!([{"remove": {"id": "menu"}}]));
        assert!(surface_has_keyboard(&core), "after the removal");
        // A client that keeps its nodes gives the focus back as it goes.
        let mut kept = core.connect();
        kept.ok(
            &mut core,
            "hello",
            json!({"name": "panel", "keep_nodes": true}),
        );
        menu(&mut kept, &mut core);
        apply(&mut kept, &mut core, &focus_node(Some("menu")));
        assert!(!surface_has_keyboard(&core));
        drop(kept);
        core.dispatch();
        assert!(core.state.scene.node("menu").is_some());
        assert!(surface_has_keyboard(&core), "after the disconnect");
        // The focus of a surface ends it.
        rect(&mut xp, &mut core, "list");
        apply(&mut xp, &mut core, &focus_node(Some("list")));
        assert!(!surface_has_keyboard(&core));
        apply(&mut wm, &mut core, &json!([{"focus": {"surface": "s1"}}]));
        assert!(surface_has_keyboard(&core), "after the focus of a surface");
    }

    #[test]
    fn the_lock_ends_the_focus_of_a_node() {
        let (mut core, mut wayland, _window, _wm, mut xp) = window_and_menu();
        apply(&mut xp, &mut core, &focus_node(Some("menu")));
        let _lock = wayland.lock(&mut core);
        assert!(core.state.lock.is_some());
        assert_eq!(core.state.scene.focused_node(), None);
        xp.take_events();
        tap(&mut core, A);
        let keys = events(&mut xp, &mut core);
        assert!(keys.iter().all(|e| e["event"] != "node.key"), "{keys:?}");
    }

    fn move_to(core: &mut Core, x: f64, y: f64) {
        super::move_pointer(&mut core.state, (x, y).into(), InputTime::now());
    }

    fn click(core: &mut Core, down: bool) {
        let state = if down {
            ButtonState::Pressed
        } else {
            ButtonState::Released
        };
        super::press_button(&mut core.state, 272, state, InputTime::now());
    }

    /// The events of a kind.
    fn named(events: &[Value], name: &str) -> Vec<Value> {
        events
            .iter()
            .filter(|e| e["event"] == name)
            .cloned()
            .collect()
    }

    #[test]
    fn pointer_events_carry_the_modifiers() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        apply(
            &mut xp,
            &mut core,
            &json!([{"update": {"id": "menu", "props": {"transform": {"translate": [400, 0]}}}}]),
        );
        move_to(&mut core, 450.0, 50.0);
        press(&mut core, CTRL, true);
        click(&mut core, true);
        move_to(&mut core, 460.0, 50.0);
        press(&mut core, CTRL, false);
        click(&mut core, false);
        let all = events(&mut xp, &mut core);
        let ctrl = json!(["Ctrl"]);
        assert_eq!(named(&all, "pointer.pressed")[0]["modifiers"], ctrl);
        assert_eq!(named(&all, "pointer.moved")[0]["modifiers"], ctrl);
        assert_eq!(named(&all, "pointer.released")[0]["modifiers"], json!([]));
    }

    #[test]
    fn scrolling_over_a_node_goes_to_its_owner_and_over_a_surface_to_its_client() {
        let (mut core, mut wayland, window, mut wm, mut xp) = window_and_menu();
        wayland.attach(&mut core, &window, (100, 100));
        apply(
            &mut xp,
            &mut core,
            &json!([{"update": {"id": "menu", "props": {"transform": {"translate": [400, 0]}}}}]),
        );
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        move_to(&mut core, 450.0, 50.0);
        press(&mut core, SHIFT, true);
        super::scroll(&mut core.state, 0.0, 15.0, InputTime::now());
        press(&mut core, SHIFT, false);
        let scrolls = named(&events(&mut xp, &mut core), "pointer.scroll");
        assert_eq!(
            scrolls,
            [
                json!({"event": "pointer.scroll", "node": "menu", "position": [450.0, 50.0],
                    "dx": 0.0, "dy": 15.0, "discrete": [0.0, 0.0], "source": "continuous",
                    "modifiers": ["Shift"]})
            ]
        );
        assert_eq!(
            named(&events(&mut wm, &mut core), "pointer.scroll"),
            Vec::<Value>::new()
        );
        move_to(&mut core, 50.0, 50.0);
        let entered = named(&events(&mut xp, &mut core), "pointer.entered");
        assert_eq!(entered[0]["node"], "s1", "the pointer is over the window");
        super::scroll(&mut core.state, 0.0, 15.0, InputTime::now());
        assert_eq!(
            named(&events(&mut xp, &mut core), "pointer.scroll"),
            Vec::<Value>::new()
        );
    }

    const SUPER: u32 = 125;

    fn pointer_focused(core: &Core) -> bool {
        let pointer = core.state.seat.get_pointer().expect("a pointer");
        pointer.current_focus().is_some()
    }

    #[test]
    fn a_grabbed_button_goes_with_its_moves_to_the_grab_and_to_no_surface() {
        let (mut core, mut wayland, window, mut wm, mut xp) = window_and_menu();
        wayland.attach(&mut core, &window, (100, 100));
        apply(&mut xp, &mut core, &json!([{"remove": {"id": "menu"}}]));
        let grab = json!({"binding": "Super+BTN_LEFT"});
        assert_eq!(wm.ok(&mut core, "input.grab", grab), "Super+BTN_LEFT");
        wm.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["pointer.pressed"]}),
        );
        move_to(&mut core, 50.0, 50.0);
        assert!(pointer_focused(&core));
        press(&mut core, SUPER, true);
        click(&mut core, true);
        assert!(!pointer_focused(&core), "the window gets a leave");
        move_to(&mut core, 60.0, 70.0);
        // The release comes also after the modifier went up.
        press(&mut core, SUPER, false);
        click(&mut core, false);
        assert!(pointer_focused(&core), "the window gets an enter");
        let at = |x: f64, y: f64| json!({"position": [x, y], "output": "HDMI-1", "local": [x, y]});
        let mut expected = Vec::new();
        for (name, state, x, y) in [
            ("input.button", Some("pressed"), 50.0, 50.0),
            ("input.moved", None, 60.0, 70.0),
            ("input.button", Some("released"), 60.0, 70.0),
        ] {
            let mut event = at(x, y);
            event["event"] = json!(name);
            event["binding"] = json!("Super+BTN_LEFT");
            if let Some(state) = state {
                event["state"] = json!(state);
            }
            expected.push(event);
        }
        assert_eq!(events(&mut wm, &mut core), expected);
        // Without the modifier, the window gets the click.
        click(&mut core, true);
        click(&mut core, false);
        let pressed = named(&events(&mut wm, &mut core), "pointer.pressed");
        assert_eq!(pressed[0]["node"], "s1");
    }

    #[test]
    fn a_button_grab_needs_a_modifier_and_a_known_button() {
        let mut core = Core::new();
        let mut wm = core.named("wm");
        let grab = |binding: &str| json!({"binding": binding});
        let (_, message) = wm.fails(&mut core, "input.grab", grab("BTN_LEFT"));
        assert_eq!(
            message,
            "BTN_LEFT needs Ctrl, Alt or Super or the capability input.grab-any"
        );
        let (_, message) = wm.fails(&mut core, "input.grab", grab("Alt+BTN_NOPE"));
        assert_eq!(message, "unknown key \"BTN_NOPE\"");
        wm.ok(&mut core, "input.grab", grab("Ctrl+Alt+BTN_RIGHT"));
        wm.ok(&mut core, "input.ungrab", grab("Ctrl+Alt+BTN_RIGHT"));
    }

    const TAB: u32 = 15;

    /// The states of the `input.key` events of a binding.
    fn key_states(client: &mut Client, core: &mut Core, binding: &str) -> Vec<String> {
        events(client, core)
            .into_iter()
            .filter(|e| e["event"] == "input.key" && e["binding"] == binding)
            .map(|e| e["state"].as_str().unwrap_or_default().to_owned())
            .collect()
    }

    #[test]
    fn a_held_grab_ends_when_its_modifier_goes_up() {
        let (mut core, mut wayland, _window, mut wm, _xp) = window_and_menu();
        wm.ok(&mut core, "input.grab", json!({"binding": "Alt+Tab"}));
        press(&mut core, ALT, true);
        tap(&mut core, TAB);
        tap(&mut core, TAB);
        assert_eq!(
            key_states(&mut wm, &mut core, "Alt+Tab"),
            ["pressed", "pressed"]
        );
        press(&mut core, ALT, false);
        assert_eq!(key_states(&mut wm, &mut core, "Alt+Tab"), ["released"]);
        // A modifier that goes up before the key ends the grab as well, and
        // the release of the key reaches no surface.
        press(&mut core, ALT, true);
        press(&mut core, TAB, true);
        press(&mut core, ALT, false);
        assert_eq!(
            key_states(&mut wm, &mut core, "Alt+Tab"),
            ["pressed", "released"]
        );
        press(&mut core, TAB, false);
        assert_eq!(
            key_states(&mut wm, &mut core, "Alt+Tab"),
            Vec::<String>::new()
        );
        wayland.roundtrip(&mut core);
        // Only the presses and releases of Alt reach the window.
        assert_eq!(wayland.key_serials().len(), 4);
    }

    #[test]
    fn shift_does_not_end_a_held_grab() {
        let (mut core, _wayland, _window, mut wm, _xp) = window_and_menu();
        wm.ok(&mut core, "input.grab", json!({"binding": "Alt+Shift+Tab"}));
        press(&mut core, ALT, true);
        press(&mut core, SHIFT, true);
        tap(&mut core, TAB);
        press(&mut core, SHIFT, false);
        let binding = "Alt+Shift+Tab";
        assert_eq!(key_states(&mut wm, &mut core, binding), ["pressed"]);
        press(&mut core, ALT, false);
        assert_eq!(key_states(&mut wm, &mut core, binding), ["released"]);
    }

    #[test]
    fn a_held_grab_without_a_modifier_ends_with_its_own_key() {
        let mut core = Core::new();
        let grant = json!({"client": "agent", "capabilities": ["input.grab-any"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        agent.ok(&mut core, "input.grab", json!({"binding": "Tab"}));
        tap(&mut core, TAB);
        assert_eq!(
            key_states(&mut agent, &mut core, "Tab"),
            ["pressed", "released"]
        );
    }

    #[test]
    fn a_revoke_ends_a_free_grab_that_is_held() {
        let mut core = Core::new();
        let grant = json!({"client": "agent", "capabilities": ["input.grab-any"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        agent.ok(&mut core, "input.grab", json!({"binding": "Tab"}));
        press(&mut core, TAB, true);
        assert_eq!(key_states(&mut agent, &mut core, "Tab"), ["pressed"]);
        core.admin()
            .ok(&mut core, "grants.revoke", json!({"client": "agent"}));
        assert_eq!(key_states(&mut agent, &mut core, "Tab"), ["released"]);
    }

    #[test]
    fn a_revoke_of_grab_any_ends_the_grabs_without_a_modifier() {
        let mut core = Core::new();
        let grant = json!({"client": "agent", "capabilities": ["input.grab-any"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        agent.ok(&mut core, "input.grab", json!({"binding": "Tab"}));
        core.admin()
            .ok(&mut core, "grants.revoke", json!({"client": "agent"}));
        tap(&mut core, TAB);
        assert_eq!(
            key_states(&mut agent, &mut core, "Tab"),
            Vec::<String>::new()
        );
    }
}
