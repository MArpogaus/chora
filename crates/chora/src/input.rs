//! Input routing: the safety layer first, then grabs, then a node with the
//! keyboard focus, then the focused surface.
//!
//! Plugins set the focus. The core changes it only in the cases that the
//! input flow of `docs/architecture.md` lists. Protocols take the keyboard
//! for a while: a popup grab, a layer surface such as a launcher, and the
//! lock screen.

use std::{sync::atomic::Ordering, time::Duration};

use chora_scene::{
    Binding, Event as SceneEvent, GivenGrab, MOVE, NodeKind, ScrollSource, ipc::PressState,
};
use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        InputTime, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent,
    },
    desktop::PopupPointerGrab,
    input::{
        keyboard::{
            FilterResult, KeyboardSource, Keycode, Keysym, KeysymHandle, ModifiersState, xkb,
        },
        pointer::{
            AxisFrame, ButtonEvent, Focus, GestureHoldBeginEvent, GestureHoldEndEvent,
            GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent,
            GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData,
            MotionEvent, PointerGrab, PointerHandle, PointerInnerHandle, RelativeMotionEvent,
        },
    },
    output::Output,
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point, SERIAL_COUNTER, Serial},
    wayland::pointer_constraints::{PointerConstraint, with_pointer_constraint},
};

use crate::{
    output::{Spot, output_area},
    render,
    state::{ButtonGrab, ClientState, HeldGrab, NodeGrab, Repeat, State, Suppressed},
};

mod grabs;
mod place;
mod plane;

use place::grab_action;
#[cfg(test)]
pub(crate) use place::holds_other_claim;
pub use plane::PlaneGrab;
pub(crate) use plane::anchor;
use plane::{local_for, position_for, start_plane_grab};

/// What a key press does besides reaching a client.
enum KeyAction {
    /// The release of a key whose press went elsewhere.
    Swallow,
    /// Stop the compositor.
    Quit,
    /// The emergency reset.
    Reset,
    /// Revokes the capabilities of every client (D23).
    RevokeAll,
    /// Switch to another virtual terminal.
    SwitchVt(i32),
    /// Send `input.key` to the client that grabbed the binding.
    /// The connection and the binding of a grab, and whether the key was
    /// injected.
    Grab(u64, Binding, bool),
    /// Send a key to the owner of the node with the keyboard focus.
    Node(String, Box<SceneEvent>),
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
    let keyboard = state.keyboard();
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

/// The keyboard of `input.inject`: a source of its own, so that an
/// injected release lifts no key or modifier that the person holds.
const INJECTED: KeyboardSource = KeyboardSource::Auxiliary(0);

/// The keyboard that a key comes from.
fn source(injected: bool) -> KeyboardSource {
    if injected {
        INJECTED
    } else {
        KeyboardSource::MAIN
    }
}

/// Releases every key that is down, for example when the window of a
/// nested chora loses the focus and the host keeps the releases.
pub fn release_keys(state: &mut State) {
    let keyboard = state.keyboard();
    for code in keyboard.pressed_keys() {
        key(state, code, KeyState::Released, InputTime::now(), false);
    }
}

/// A key goes down or up: the safety layer first, then the grabs, then a
/// node with the keyboard focus, then the focused surface. An injected key never reaches the safety layer (D23).
pub fn key(state: &mut State, code: Keycode, key_state: KeyState, time: InputTime, injected: bool) {
    let keyboard = state.keyboard();
    // A grab that a client took in this loop iteration, for example of an
    // input method, must not see a key of the lock screen.
    if state.lock.is_some() && keyboard.is_grabbed() {
        keyboard.unset_grab(state);
    }
    if key_state == KeyState::Pressed {
        state.pressed = None;
    }
    let serial = SERIAL_COUNTER.next_serial();
    if key_state == KeyState::Pressed {
        state.track_press(serial, injected);
    }
    let action = keyboard.input_from_source(
        source(injected),
        state,
        code,
        key_state,
        serial,
        time,
        |state, mods, handle| {
            if key_state == KeyState::Released {
                released_key(state, code, mods, &handle, injected)
            } else {
                pressed_key(state, code, mods, &handle, injected)
            }
        },
    );
    match action {
        Some(KeyAction::Quit) => state.quit("the quit binding"),
        Some(KeyAction::Reset) => state.reset(),
        Some(KeyAction::RevokeAll) => state.revoke_all(),
        Some(KeyAction::SwitchVt(vt)) => {
            if let Some(backend) = state.backend.as_mut() {
                backend.change_vt(vt);
            }
        }
        Some(KeyAction::Grab(conn, binding, injected)) => {
            state.note_act(vec![conn]);
            if !injected && key_state == KeyState::Pressed {
                state.choose_manager_of(conn);
            }
            hold_grab(state, conn, binding, code, injected);
        }
        Some(KeyAction::Node(owner, event)) => {
            if key_state == KeyState::Pressed {
                state.note_act(state.owner_conns(Some(&owner)));
            }
            state.ipc.send_to_client(&owner, &event);
        }
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

/// Tells the client of a key grab about the press and keeps the grab until
/// the release.
fn hold_grab(state: &mut State, conn: u64, binding: Binding, code: Keycode, injected: bool) {
    grabbed(state, conn, &binding, PressState::Pressed, injected);
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
            injected,
        });
    }
}

/// Where the release of a key goes: to the surface, unless its press went
/// elsewhere.
fn released_key(
    state: &mut State,
    code: Keycode,
    mods: &ModifiersState,
    handle: &KeysymHandle<'_>,
    injected: bool,
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
        grabbed(
            state,
            grab.conn,
            &grab.binding,
            PressState::Released,
            grab.injected,
        );
    }
    let Some(i) = state.held.keys.iter().position(|(k, _)| *k == code) else {
        return FilterResult::Forward;
    };
    FilterResult::Intercept(match state.held.keys.remove(i).1 {
        Suppressed::Node(owner, node) => {
            let release = node_key(node, mods, handle, PressState::Released, injected);
            KeyAction::Node(owner, Box::new(release))
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
        let event = node_key(node.clone(), mods, handle, PressState::Pressed, injected);
        let (rate, delay) = state.repeat();
        if rate > 0 && !handle.modified_sym().is_modifier_key() {
            let next = state.time.now() + Duration::from_millis(delay.into());
            let _ = state.expire_at(next);
            state.held.repeat = Some(Repeat {
                code,
                owner: owner.clone(),
                event: event.clone(),
                next,
            });
        }
        (
            KeyAction::Node(owner.clone(), Box::new(event)),
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
fn grabbed(state: &mut State, conn: u64, binding: &Binding, press: PressState, injected: bool) {
    let event = chora_scene::Event::InputKey {
        binding: binding.to_string(),
        state: press,
        injected,
    };
    state.ipc.send_to(conn, &event);
}

/// True for a binding whose key is a modifier itself, as `Super+Super_L`:
/// a tap of the modifier.
fn lone_modifier(binding: &Binding) -> bool {
    ["Super_", "Alt_", "Control_", "Shift_", "Meta_"]
        .iter()
        .any(|m| binding.key.starts_with(m))
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
        grabbed(
            state,
            grab.conn,
            &grab.binding,
            PressState::Released,
            grab.injected,
        );
    }
}

impl State {
    /// Takes the grabs without Ctrl, Alt or Super away from a client name:
    /// a held one ends with one `released` and the registrations go with
    /// the capability `input.grab-any` that allowed them.
    pub fn revoke_free_grabs(&mut self, name: &str) {
        let conns = self.ipc.conns_of(name);
        let dropped = (conns.iter())
            .flat_map(|conn| self.ipc.drop_free_grabs(*conn))
            .collect::<Vec<_>>();
        let (ends, keep) = std::mem::take(&mut self.held.grabs)
            .into_iter()
            .partition(|h| conns.contains(&h.conn) && free_binding(&h.binding));
        self.held.grabs = keep;
        for grab in ends {
            grabbed(
                self,
                grab.conn,
                &grab.binding,
                PressState::Released,
                grab.injected,
            );
        }
        self.tell_ungrabbed(dropped);
    }

    /// Sends the held key of a node again when its time came, as a client
    /// repeats a held key of `wl_keyboard`. The repeat ends with the focus
    /// of the node.
    pub fn repeat_key(&mut self) {
        let Some(repeat) = self.held.repeat.take() else {
            return;
        };
        let SceneEvent::NodeKey { node, .. } = &repeat.event else {
            // injected: the repeat resends a key with the flag it was sent with
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
        let (rate, _) = self.repeat();
        if rate == 0 {
            return;
        }
        let next = repeat.next + Duration::from_millis(1000 / u64::from(rate));
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
    injected: bool,
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
        injected,
    }
}

/// The names of the modifiers of the keyboard that are down.
fn modifiers(state: &State) -> Vec<String> {
    let keyboard = state.keyboard();
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

/// The fixed bindings of the safety layer (D23), then the grabs. Quit,
/// reset and revoke wait while the session is locked. The keymap
/// turns Ctrl+Alt+F1 to F12 into the keys that switch the VT; only the
/// modified keysyms show them.
fn press_action(
    state: &State,
    mods: &ModifiersState,
    (syms, modified): &(Vec<Keysym>, Vec<Keysym>),
    injected: bool,
) -> Option<KeyAction> {
    if injected {
        return grab_action(state, mods, syms, true);
    }
    let first = Keysym::XF86_Switch_VT_1.raw();
    if let Some(vt) = modified
        .iter()
        .find_map(|s| s.raw().checked_sub(first).filter(|n| *n < 12))
    {
        return Some(KeyAction::SwitchVt(i32::try_from(vt).ok()? + 1));
    }
    if mods.ctrl && mods.alt && state.lock.is_none() {
        if syms.contains(&Keysym::BackSpace) {
            return Some(KeyAction::Quit);
        }
        if syms.contains(&Keysym::Escape) {
            return Some(KeyAction::Reset);
        }
        if syms.contains(&Keysym::i) || syms.contains(&Keysym::I) {
            return Some(KeyAction::RevokeAll);
        }
    }
    grab_action(state, mods, syms, false)
}

/// Relative motion, as from a mouse on real hardware. The pointer stays on
/// the outputs.
fn relative_motion<I: InputBackend>(state: &mut State, event: &I::PointerMotionEvent) {
    let pointer = state.pointer();
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
    /// for example after a transaction or a camera change, with the time
    /// of the last input of the pointer: the scene moved, not the hand.
    pub fn refresh_pointer(&mut self) {
        move_pointer(self, self.pointer_layout, self.pointer_time);
    }
}

/// The drag of a node goes to plugins only, with each new position.
fn drag_node(state: &mut State, spot: &Spot, time: InputTime) {
    let outputs = state.all_outputs();
    let Some(grab) = &mut state.node_grab else {
        return;
    };
    let position = position_for(&state.scene, &outputs, &grab.node, spot);
    let local = local_for(&state.scene, &outputs, &grab.node, spot).unwrap_or(grab.local);
    if (grab.position, grab.local) != (position, local) {
        grab.position = position;
        grab.local = local;
        let node = grab.node.clone();
        let modifiers = modifiers(state);
        let injected = state.injecting;
        state.emit(&SceneEvent::PointerMoved {
            node,
            position,
            local,
            modifiers,
            injected,
            time: u64::from(time.millis()),
        });
    }
}

/// Moves the pointer to a point of the layout and tells the surface under
/// it.
pub fn move_pointer(state: &mut State, layout: Point<f64, Logical>, time: InputTime) {
    let moved = state.pointer_layout != layout;
    state.pointer_layout = layout;
    state.pointer_time = time;
    let Some(spot) = state.spot(layout) else {
        return;
    };
    state.drop_stray_grab();
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
            injected: state.injecting,
        };
        state.ipc.send_to(conn, &event);
    }
    let under = if state.node_grab.is_some() {
        drag_node(state, &spot, time);
        None
    } else if state.button_grab.is_some() {
        None
    } else {
        render::surface_under(state, &spot).map(|u| (u.surface, u.location))
    };
    let pointer = state.pointer();
    let under =
        under.filter(|(surface, _)| !state.client_drag || state.takes_part_in_drags(surface));
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
    press_button(
        state,
        event.button_code(),
        event.state(),
        event.time(),
        false,
    );
}

/// Ends the drag and drop of a client without a drop: injected input never
/// moves or drops one.
pub fn cancel_client_drag(state: &mut State) {
    if state.client_drag {
        let pointer = state.pointer();
        pointer.unset_grab(state, SERIAL_COUNTER.next_serial(), InputTime::now());
        state.client_drag = false;
    }
}

/// A pointer button goes down or up.
pub fn press_button(
    state: &mut State,
    code: u32,
    button_state: ButtonState,
    time: InputTime,
    injected: bool,
) {
    state.pointer_time = time;
    if injected {
        cancel_client_drag(state);
    }
    if button_state == ButtonState::Pressed {
        state.pressed = None;
    }
    let left_a_pane = !injected
        && match button_state {
            ButtonState::Pressed => {
                state.hold_press(code);
                false
            }
            ButtonState::Released => state.release_press(code),
        };
    let serial = SERIAL_COUNTER.next_serial();
    if button_state == ButtonState::Pressed {
        state.track_press(serial, injected);
        let real = !injected || state.injected_press_is_real;
        state.last_real_button = real.then_some((serial, code));
        if real {
            state.choose_manager();
        }
    } else {
        state
            .last_real_button
            .take_if(|(_, button)| *button == code);
    }
    if grab_button(state, code, button_state, time) {
        return;
    }
    let pointer = state.pointer();
    if button_state == ButtonState::Pressed {
        click_focus(state);
        note_input(state, pointer.current_focus(), serial, true);
        press_node(state, code, injected);
        start_plane_grab(state, &pointer, code, serial);
    } else {
        note_input(state, pointer.current_focus(), serial, false);
        release_node(state, code, left_a_pane, time);
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
    if button_state == ButtonState::Pressed {
        start_plane_grab(state, &pointer, code, serial);
    }
}

/// The connection, the binding and the state of the grab that a button
/// press or release started or ended: whether the grab took the pointer,
/// and whether its press did not come from the person.
type Grabbed = (u64, String, PressState, bool, bool);

/// The grab that a press of a button with modifiers starts, if a client
/// grabbed the binding. As for a new toplevel: without an output, the
/// pointer is in no claim.
fn open_button_grab(state: &mut State, code: u32) -> Option<Grabbed> {
    if state.button_grab.is_some() || state.lock.is_some() {
        return None;
    }
    let keyboard = state.keyboard();
    let mods = keyboard.modifier_state();
    let pointer =
        (state.spot(state.pointer_layout)).map_or([f64::NAN; 2], |s| [s.scene.x, s.scene.y]);
    let (conn, binding) = state
        .ipc
        .button_grab_for(&mods, code, &state.scene, pointer)?;
    let binding = binding.to_string();
    let to_pointer = binding == MOVE;
    let window = window_under_pointer(state);
    let real = real_press_of(state, code);
    state.button_grab = Some(ButtonGrab {
        conn,
        binding: binding.clone(),
        button: code,
        real,
        to_pointer,
        given: None,
        window,
    });
    Some((
        conn,
        binding,
        PressState::Pressed,
        to_pointer,
        real.is_none(),
    ))
}

/// The grab that the release of its button ends.
fn close_button_grab(state: &mut State, code: u32) -> Option<Grabbed> {
    if state.drop_stray_grab() {
        state.refresh_pointer();
    }
    let grab = state.button_grab.take_if(|g| g.button == code)?;
    Some((
        grab.conn,
        grab.binding,
        PressState::Released,
        grab.to_pointer,
        grab.real.is_none(),
    ))
}

/// A button with modifiers that a client grabbed: the press, the moves and
/// the release go to the client of the grab and to no surface. The release
/// comes also when the modifiers went up first. Returns whether the grab
/// took the button.
fn grab_button(state: &mut State, code: u32, button_state: ButtonState, time: InputTime) -> bool {
    let grab = if button_state == ButtonState::Pressed {
        open_button_grab(state, code)
    } else {
        close_button_grab(state, code)
    };
    let Some(grab) = grab else {
        return false;
    };
    let (conn, binding, press, moved, injected) = grab;
    if press == PressState::Pressed {
        state.note_act(vec![conn]);
    }
    send_button(state, (conn, binding), press, None, injected);
    if moved {
        // The press of a move reached the pointer, so its release does too,
        // to no surface.
        let pointer = state.pointer();
        let event = ButtonEvent {
            button: code,
            state: button_state,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        };
        pointer.button(state, &event);
        pointer.frame(state);
    }
    // The surface under the pointer gets a leave, and an enter after the
    // release.
    move_pointer(state, state.pointer_layout, time);
    true
}

/// The window under the pointer, for the press of a button binding.
fn window_under_pointer(state: &State) -> Option<String> {
    let spot = state.spot(state.pointer_layout)?;
    render::surface_under(state, &spot)?.node
}

/// Starts the drag of an interactive move (`xdg_toplevel.move`) while a
/// button is down on the window: an owner of the window that grabbed
/// `move` gets the press, the moves and the release as the events of a
/// button grab with the binding `move`, and the window gets a leave.
///
/// The press of `serial` must still hold the pointer, on `toplevel` or one
/// of its subsurfaces: a stale serial of a window in the background does
/// not cut off a drag in another window.
pub fn start_move(state: &mut State, toplevel: &WlSurface, surface: &str, serial: Serial) {
    let pointer = state.pointer();
    if !pointer.has_grab(serial) {
        return;
    }
    let Some(start) = pointer.grab_start_data() else {
        return;
    };
    let on_it = start
        .focus
        .is_some_and(|(focus, _)| crate::handlers::root_surface(&focus) == *toplevel);
    if !on_it {
        return;
    }
    let button = start.button;
    if state.button_grab.is_some() || state.lock.is_some() {
        return;
    }
    let owner = state.scene.node(surface).and_then(|n| n.owner.as_deref());
    let Some(conn) = owner.and_then(|o| state.ipc.move_grab_of(o)) else {
        return;
    };
    // The click of the client ends here; the core keeps the button.
    pointer.unset_grab(state, SERIAL_COUNTER.next_serial(), InputTime::now());
    let real = real_press_of(state, button);
    state.button_grab = Some(ButtonGrab {
        conn,
        binding: MOVE.into(),
        button,
        real,
        to_pointer: true,
        given: None,
        window: Some(surface.into()),
    });
    let window = Some((surface, None));
    let injected = real.is_none();
    send_button(
        state,
        (conn, MOVE.into()),
        PressState::Pressed,
        window,
        injected,
    );
    move_pointer(state, state.pointer_layout, InputTime::now());
}

/// Sends `input.button` to the connection of a button grab, with the
/// window of a move and the state of a drag that came with it. Without
/// an output under the pointer the event has the pointer in the layout
/// and no output name, so that a release always arrives. `injected` is
/// set when the press of the grab did not come from the person. A press
/// ends the held taps of a lone modifier of other clients without their
/// release: a drag with Super is no tap of Super.
fn send_button(
    state: &mut State,
    (conn, binding): (u64, String),
    press: PressState,
    window: Option<(&str, Option<GivenGrab>)>,
    injected: bool,
) {
    if press == PressState::Pressed {
        (state.held.grabs).retain(|h| h.conn == conn || !lone_modifier(&h.binding));
    }
    let (surface, grab) = window.unzip();
    let layout = [state.pointer_layout.x, state.pointer_layout.y];
    let (position, output, local) =
        state
            .spot(state.pointer_layout)
            .map_or((layout, String::new(), layout), |spot| {
                let scene = [spot.scene.x, spot.scene.y];
                (scene, spot.output.name(), [spot.local.x, spot.local.y])
            });
    let event = SceneEvent::InputButton {
        binding,
        state: press,
        position,
        output,
        local,
        surface: surface.map(String::from),
        grab: grab.flatten(),
        injected,
    };
    state.ipc.send_to(conn, &event);
}

/// The serial of the press of a button when the user made it and the
/// button is still down; `None` after an injected press.
fn real_press_of(state: &State, button: u32) -> Option<Serial> {
    let (serial, pressed) = state.last_real_button?;
    (pressed == button).then_some(serial)
}

impl State {}

/// The press of a drag: its serial and its button.
pub type Press = (Serial, u32);

/// The most bytes of the `wobble` of a `give_grab`, as JSON: the springs
/// of a 4x4 grid take about 2 kB.
const MAX_WOBBLE: usize = 16 * 1024;

/// The press that a client holds and may give with its window.
struct HeldPress {
    button: u32,
    /// The serial of the press when the user made it.
    real: Option<Serial>,
    /// The window of a button grab.
    window: Option<String>,
    /// The node of a press on a node.
    node: Option<String>,
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
    let pointer = state.pointer();
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
        let injected = state.injecting;
        state.emit(&SceneEvent::PointerEntered {
            node,
            moved,
            injected,
        });
    }
}

/// Tells plugins about a press over a node. Over a rect or an image, the
/// moves and the release of the button follow.
fn press_node(state: &mut State, button: u32, injected: bool) {
    if state.node_grab.is_some() {
        return;
    }
    let Some(spot) = state.spot(state.pointer_layout) else {
        return;
    };
    let Some((node, local)) = render::node_hit(state, &spot) else {
        return;
    };
    let local = [local.x, local.y];
    let hit = state.scene.node(&node);
    let area = hit.is_some_and(|n| n.kind != NodeKind::Surface);
    // A window counts for the client that it came from.
    if !injected {
        state.pressed = hit.and_then(|n| {
            n.surface
                .as_ref()
                .map_or(n.owner.clone(), |s| s.client.clone())
        });
    }
    if area {
        let owner = hit.and_then(|n| n.owner.clone());
        state.note_act(state.owner_conns(owner.as_deref()));
    }
    let position = position_for(&state.scene, &state.all_outputs(), &node, &spot);
    state.emit(&SceneEvent::PointerPressed {
        node: node.clone(),
        button,
        position,
        local,
        modifiers: modifiers(state),
        injected: injected && !state.injected_press_is_real,
    });
    if area {
        let real =
            real_press_of(state, button).filter(|_| !injected || state.injected_press_is_real);
        state.node_grab = Some(NodeGrab {
            node,
            button,
            position,
            local,
            real,
        });
    }
}

/// Ends the drag of a node when its button goes up. After a press through
/// a pane, the release is on the screen and the node keeps its last point.
fn release_node(state: &mut State, button: u32, left_a_pane: bool, time: InputTime) {
    let Some(NodeGrab { node, local, .. }) = state.node_grab.take_if(|g| g.button == button) else {
        return;
    };
    let spot = state.spot(state.pointer_layout);
    let position = spot.as_ref().map_or([0.0; 2], |s| {
        position_for(&state.scene, &state.all_outputs(), &node, s)
    });
    let local = spot
        .filter(|_| !left_a_pane)
        .and_then(|s| local_for(&state.scene, &state.all_outputs(), &node, &s))
        .unwrap_or(local);
    let modifiers = modifiers(state);
    let injected = state.injecting;
    state.emit(&SceneEvent::PointerReleased {
        node,
        button,
        position,
        local,
        modifiers,
        injected,
        time: u64::from(time.millis()),
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
    #[allow(
        clippy::float_cmp,
        reason = "the end of a scroll on a touchpad is exactly 0"
    )]
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
    let position = position_for(&state.scene, &state.all_outputs(), &node, &spot);
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
        injected: state.injecting,
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
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "a scroll distance in wheel steps, far below i32::MAX"
                )]
                let d = d as i32;
                frame = frame.v120(axis, d);
            }
        }
        if source == AxisSource::Finger && event.amount(axis) == Some(0.0) {
            frame = frame.stop(axis);
        }
    }
    scroll_node(state, delta, steps, source);
    let pointer = state.pointer();
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
    /// The params of the events, without `seq`: these tests check
    /// payloads, the sequence numbers have a test of their own.
    fn events(client: &mut Client, core: &mut Core) -> Vec<Value> {
        client
            .events_now(core)
            .into_iter()
            .map(|e| {
                let mut params = e["params"].clone();
                if let Some(params) = params.as_object_mut() {
                    params.remove("seq");
                }
                params
            })
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

    /// A key with Super that nobody grabbed reaches the window, as any
    /// key does: a nested compositor, a virtual machine or a game may
    /// want it. The window managers grab only the keys they document.
    #[test]
    fn an_unbound_super_key_reaches_the_window() {
        let (mut core, mut wayland, _window, _wm, _xp) = window_and_menu();
        press(&mut core, SUPER, true);
        tap(&mut core, A);
        press(&mut core, SUPER, false);
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.key_serials().len(), 4, "Super and Super+a");
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
        super::press_button(&mut core.state, 272, state, InputTime::now(), false);
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
    fn a_move_and_a_release_carry_the_time_of_their_input() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        apply(
            &mut xp,
            &mut core,
            &json!([{"update": {"id": "menu", "props": {"transform": {"translate": [400, 0]}}}}]),
        );
        let at = InputTime::from_millis;
        move_to(&mut core, 450.0, 50.0);
        click(&mut core, true);
        super::move_pointer(&mut core.state, (460.0, 50.0).into(), at(81_234));
        super::press_button(
            &mut core.state,
            272,
            ButtonState::Released,
            at(81_250),
            false,
        );
        let all = events(&mut xp, &mut core);
        assert_eq!(named(&all, "pointer.moved")[0]["time"], 81_234);
        assert_eq!(named(&all, "pointer.released")[0]["time"], 81_250);
    }

    #[test]
    fn the_time_of_an_input_is_a_32_bit_count_of_milliseconds() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        apply(
            &mut xp,
            &mut core,
            &json!([{"update": {"id": "menu", "props": {"transform": {"translate": [400, 0]}}}}]),
        );
        let past_the_wrap = InputTime::from_micros(((1 << 32) + 5) * 1000);
        move_to(&mut core, 450.0, 50.0);
        click(&mut core, true);
        super::move_pointer(&mut core.state, (460.0, 50.0).into(), past_the_wrap);
        let all = events(&mut xp, &mut core);
        assert_eq!(named(&all, "pointer.moved")[0]["time"], 5);
    }

    #[test]
    fn a_refresh_during_a_press_keeps_the_time_of_the_last_input() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        let translate = |x: i32| json!([{"update": {"id": "menu", "props": {"transform": {"translate": [x, 0]}}}}]);
        apply(&mut xp, &mut core, &translate(400));
        let at = InputTime::from_millis;
        move_to(&mut core, 450.0, 50.0);
        super::press_button(
            &mut core.state,
            272,
            ButtonState::Pressed,
            at(81_200),
            false,
        );
        apply(&mut xp, &mut core, &translate(410));
        super::move_pointer(&mut core.state, (460.0, 50.0).into(), at(81_234));
        apply(&mut xp, &mut core, &translate(420));
        super::press_button(
            &mut core.state,
            272,
            ButtonState::Released,
            at(81_250),
            false,
        );
        let all = events(&mut xp, &mut core);
        let times: Vec<Value> = (named(&all, "pointer.moved").iter())
            .map(|moved| moved["time"].clone())
            .collect();
        assert_eq!(times, [81_200, 81_234, 81_234]);
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

    /// A button grab of one client during the tap of a lone modifier
    /// that another client holds, such as Super of a start menu, ends the
    /// tap without its release: the drag with Super is no tap of Super.
    #[test]
    fn a_grabbed_button_ends_the_tap_of_a_lone_modifier_of_another_client() {
        let (mut core, _wayland, _window, mut wm, mut xp) = window_and_menu();
        let drag = json!({"binding": "Super+BTN_LEFT"});
        assert_eq!(wm.ok(&mut core, "input.grab", drag), "Super+BTN_LEFT");
        let tap = json!({"binding": "Super+Super_L"});
        assert_eq!(xp.ok(&mut core, "input.grab", tap), "Super+Super_L");
        move_to(&mut core, 50.0, 50.0);
        press(&mut core, SUPER, true);
        click(&mut core, true);
        click(&mut core, false);
        press(&mut core, SUPER, false);
        assert_eq!(key_states(&mut xp, &mut core, "Super+Super_L"), ["pressed"]);
        press(&mut core, SUPER, true);
        press(&mut core, SUPER, false);
        assert_eq!(
            key_states(&mut xp, &mut core, "Super+Super_L"),
            ["pressed", "released"],
            "a tap without a button ends with its release"
        );
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

    /// The events of a client of one of the kinds, each with its `injected`
    /// flag, `false` when the event has none.
    fn flags(events: &[Value], names: &[&str]) -> Vec<(String, bool)> {
        events
            .iter()
            .filter_map(|e| {
                let name = e["event"].as_str().filter(|n| names.contains(n))?;
                Some((name.to_string(), e["injected"].as_bool().unwrap_or(false)))
            })
            .collect()
    }

    #[test]
    fn an_injected_click_with_an_injected_modifier_reaches_the_button_grab_flagged() {
        let (mut core, mut wayland, window, mut wm, _xp) = window_and_menu();
        wayland.attach(&mut core, &window, (100, 100));
        wm.ok(
            &mut core,
            "input.grab",
            json!({"binding": "Super+BTN_LEFT"}),
        );
        move_to(&mut core, 50.0, 50.0);
        events(&mut wm, &mut core);
        core.state.injecting = true;
        super::key(
            &mut core.state,
            Keycode::new(SUPER + 8),
            KeyState::Pressed,
            InputTime::now(),
            true,
        );
        let down = ButtonState::Pressed;
        super::press_button(&mut core.state, 272, down, InputTime::now(), true);
        move_to(&mut core, 60.0, 70.0);
        let up = ButtonState::Released;
        super::press_button(&mut core.state, 272, up, InputTime::now(), true);
        core.state.injecting = false;
        let got = flags(
            &events(&mut wm, &mut core),
            &["input.button", "input.moved"],
        );
        let injected = |name: &str| (name.to_string(), true);
        assert_eq!(
            got,
            [
                injected("input.button"),
                injected("input.moved"),
                injected("input.button")
            ]
        );
    }

    #[test]
    fn a_click_of_the_person_reaches_the_button_grab_unflagged() {
        let (mut core, mut wayland, window, mut wm, _xp) = window_and_menu();
        wayland.attach(&mut core, &window, (100, 100));
        wm.ok(
            &mut core,
            "input.grab",
            json!({"binding": "Super+BTN_LEFT"}),
        );
        move_to(&mut core, 50.0, 50.0);
        events(&mut wm, &mut core);
        press(&mut core, SUPER, true);
        click(&mut core, true);
        move_to(&mut core, 60.0, 70.0);
        click(&mut core, false);
        let got = flags(
            &events(&mut wm, &mut core),
            &["input.button", "input.moved"],
        );
        assert_eq!(got.len(), 3, "{got:?}");
        assert!(got.iter().all(|(_, injected)| !injected), "{got:?}");
    }

    #[test]
    fn an_injected_move_and_an_injected_scroll_reach_the_owner_flagged() {
        let (mut core, _wayland, _window, _wm, mut xp) = window_and_menu();
        move_to(&mut core, 300.0, 300.0);
        events(&mut xp, &mut core);
        core.state.injecting = true;
        move_to(&mut core, 50.0, 50.0);
        super::scroll(&mut core.state, 0.0, 15.0, InputTime::now());
        core.state.injecting = false;
        let got = flags(
            &events(&mut xp, &mut core),
            &["pointer.entered", "pointer.scroll"],
        );
        let injected = |name: &str| (name.to_string(), true);
        assert_eq!(
            got,
            [injected("pointer.entered"), injected("pointer.scroll")]
        );
        move_to(&mut core, 300.0, 300.0);
        move_to(&mut core, 50.0, 50.0);
        super::scroll(&mut core.state, 0.0, 15.0, InputTime::now());
        let got = flags(
            &events(&mut xp, &mut core),
            &["pointer.entered", "pointer.scroll"],
        );
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(got.iter().all(|(_, injected)| !injected), "{got:?}");
    }

    /// A window manager whose group claims an area.
    fn claimer(core: &mut Core, name: &str, claim: [f64; 4]) -> Client {
        let mut wm = core.named(name);
        apply(
            &mut wm,
            core,
            &json!([{"create": {"id": name, "kind": "group", "parent": "root",
                "props": {"claim": claim}}}]),
        );
        wm.ok(core, "events.subscribe", json!({"events": []}));
        wm
    }

    /// The `state` of the `input.button` events and the moves of a client.
    fn buttons(client: &mut Client, core: &mut Core) -> Vec<String> {
        events(client, core)
            .iter()
            .filter(|e| e["event"] == "input.button" || e["event"] == "input.moved")
            .map(|e| e["state"].as_str().unwrap_or("moved").to_owned())
            .collect()
    }

    #[test]
    fn a_shared_button_grab_goes_to_the_claim_under_the_pointer() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let grab = json!({"binding": "Super+BTN_LEFT"});
        let mut left = claimer(&mut core, "left", [0.0, 0.0, 400.0, 600.0]);
        let mut right = claimer(&mut core, "right", [400.0, 0.0, 400.0, 600.0]);
        let mut inner = claimer(&mut core, "inner", [500.0, 100.0, 100.0, 100.0]);
        left.ok(&mut core, "input.grab", grab.clone());
        right.ok(&mut core, "input.grab", grab.clone());
        inner.ok(&mut core, "input.grab", grab.clone());
        let drag = |core: &mut Core, from: (f64, f64), to: (f64, f64)| {
            move_to(core, from.0, from.1);
            press(core, SUPER, true);
            click(core, true);
            move_to(core, to.0, to.1);
            click(core, false);
            press(core, SUPER, false);
        };
        let all = ["pressed", "moved", "released"];
        // The drag stays with the client of the press, across claims.
        drag(&mut core, (100.0, 100.0), (700.0, 500.0));
        assert_eq!(buttons(&mut left, &mut core), all);
        assert_eq!(buttons(&mut right, &mut core), Vec::<String>::new());
        drag(&mut core, (700.0, 500.0), (100.0, 100.0));
        assert_eq!(buttons(&mut right, &mut core), all);
        assert_eq!(buttons(&mut left, &mut core), Vec::<String>::new());
        // The smallest claim wins.
        drag(&mut core, (550.0, 150.0), (560.0, 160.0));
        assert_eq!(buttons(&mut inner, &mut core), all);
        assert_eq!(buttons(&mut right, &mut core), Vec::<String>::new());
        // No claim under the pointer: the first grab.
        apply(
            &mut right,
            &mut core,
            &json!([{"update": {"id": "right", "props": {"claim": null}}}]),
        );
        drag(&mut core, (700.0, 500.0), (710.0, 510.0));
        assert_eq!(buttons(&mut left, &mut core), all);
        assert_eq!(buttons(&mut right, &mut core), Vec::<String>::new());
    }

    #[test]
    fn a_button_grab_without_a_claim_has_one_owner() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let grab = json!({"binding": "Super+BTN_LEFT"});
        let mut plain = core.named("plain");
        let mut claimed = claimer(&mut core, "claimed", [0.0, 0.0, 400.0, 600.0]);
        let mut other = claimer(&mut core, "other", [400.0, 0.0, 400.0, 600.0]);
        // A first grab without a claim keeps the binding to itself.
        plain.ok(&mut core, "input.grab", grab.clone());
        let (_, message) = claimed.fails(&mut core, "input.grab", grab.clone());
        assert_eq!(message, "Super+BTN_LEFT is grabbed already");
        plain.ok(&mut core, "input.ungrab", grab.clone());
        // A second grab needs a claim too.
        claimed.ok(&mut core, "input.grab", grab.clone());
        let (_, message) = plain.fails(&mut core, "input.grab", grab.clone());
        assert_eq!(message, "Super+BTN_LEFT is grabbed already");
        other.ok(&mut core, "input.grab", grab);
        // Key bindings stay with the first grab, claims or not.
        let key = json!({"binding": "Super+a"});
        claimed.ok(&mut core, "input.grab", key.clone());
        let (_, message) = other.fails(&mut core, "input.grab", key);
        assert_eq!(message, "Super+a is grabbed already");
    }

    /// The serial of the last press that the client of a window got.
    fn last_press(core: &Core, window: &str) -> u32 {
        let surface = core.state.windows[window].toplevel().unwrap().wl_surface();
        let id = smithay::reexports::wayland_server::Resource::id(surface);
        let client = core.state.display_handle.get_client(id).unwrap();
        let data = client.get_data::<super::ClientState>().unwrap();
        data.last_press.load(super::Ordering::Relaxed)
    }

    #[test]
    fn a_move_of_a_client_is_a_drag_of_the_window_manager_that_grabbed_move() {
        let (mut core, mut wayland, window, mut wm, mut xp) = window_and_menu();
        wayland.attach(&mut core, &window, (100, 100));
        apply(&mut xp, &mut core, &json!([{"remove": {"id": "menu"}}]));
        move_to(&mut core, 50.0, 50.0);
        let drag = |core: &mut Core, wayland: &mut Wayland| {
            click(core, true);
            let serial = last_press(core, "s1");
            wayland.request_move(core, &window, serial);
            move_to(core, 60.0, 70.0);
            click(core, false);
            move_to(core, 50.0, 50.0);
        };
        // Without the grab of move, the client keeps its press.
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        drag(&mut core, &mut wayland);
        assert_eq!(buttons(&mut wm, &mut core), Vec::<String>::new());
        assert_eq!(
            wm.ok(&mut core, "input.grab", json!({"binding": "move"})),
            "move"
        );
        drag(&mut core, &mut wayland);
        let grabbed = events(&mut wm, &mut core);
        let grabbed: Vec<&Value> = (grabbed.iter())
            .filter(|e| e["event"] == "input.button" || e["event"] == "input.moved")
            .collect();
        let states: Vec<&str> = (grabbed.iter())
            .map(|e| e["state"].as_str().unwrap_or("moved"))
            .collect();
        assert_eq!(states, ["pressed", "moved", "released"]);
        // The press names the window that asked for the move.
        assert_eq!(grabbed[0]["surface"], "s1");
        assert!(grabbed[2].get("surface").is_none());
        assert!(pointer_focused(&core), "the window gets an enter");
        // A second press works as before.
        click(&mut core, true);
        click(&mut core, false);
        let pressed = named(&events(&mut wm, &mut core), "pointer.pressed");
        assert_eq!(pressed[0]["node"], "s1");
    }

    /// A client that kept the serial of an old press cannot cut off a drag
    /// in another window with a move (4.7).
    #[test]
    fn a_move_with_a_stale_serial_starts_no_drag() {
        let (mut core, mut wayland, window, mut wm, mut xp) = window_and_menu();
        wayland.attach(&mut core, &window, (100, 100));
        apply(&mut xp, &mut core, &json!([{"remove": {"id": "menu"}}]));
        let mut other = Wayland::connect(&mut core);
        let second = other.toplevel(&mut core, "two");
        apply(
            &mut wm,
            &mut core,
            &json!([{"attach": {"surface": "s2", "parent": "root"}},
                {"update": {"id": "s2", "props": {"transform": {"translate": [300, 300]}}}}]),
        );
        other.attach(&mut core, &second, (100, 100));
        wm.ok(&mut core, "input.grab", json!({"binding": "move"}));
        wm.ok(&mut core, "events.subscribe", json!({"events": []}));
        move_to(&mut core, 50.0, 50.0);
        click(&mut core, true);
        let stale = last_press(&core, "s1");
        click(&mut core, false);
        // The user holds the button on the other window.
        move_to(&mut core, 350.0, 350.0);
        click(&mut core, true);
        wayland.request_move(&mut core, &window, stale);
        move_to(&mut core, 360.0, 370.0);
        click(&mut core, false);
        assert_eq!(buttons(&mut wm, &mut core), Vec::<String>::new());
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

    /// A revoke of `input.grab-any` ends its grabs as `input.ungrab` does:
    /// the subscribers of `input.ungrabbed` are told (rd-7).
    #[test]
    fn a_revoke_of_grab_any_tells_the_ungrab() {
        let mut core = Core::new();
        let grant = json!({"client": "agent", "capabilities": ["input.grab-any"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        let mut waiting = core.named("waiting");
        waiting.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["input.ungrabbed"]}),
        );
        agent.ok(&mut core, "input.grab", json!({"binding": "Tab"}));
        agent.ok(&mut core, "input.grab", json!({"binding": "Alt+Escape"}));
        waiting.events_now(&mut core);
        core.admin()
            .ok(&mut core, "grants.revoke", json!({"client": "agent"}));
        let told = (waiting.events_now(&mut core).into_iter())
            .map(|e| e["params"].clone())
            .filter(|e| e["event"] == "input.ungrabbed")
            .map(|e| e["binding"].clone())
            .collect::<Vec<_>>();
        assert_eq!(told, [json!("Tab")], "only the grab that ended is told");
    }

    /// tiling on the left half and xp on the right half, each with a
    /// claim, the button binding and `move`. tiling has the window s1 and
    /// drags it: Super is down and the left button went down on the
    /// window at (10, 10), as an injected press when `injected`.
    fn mid_drag(injected: bool) -> (Core, Wayland, Client, Client) {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "win");
        wayland.attach(&mut core, &window, (40, 30));
        let mut tiling = claimer(&mut core, "tiling", [0.0, 0.0, 400.0, 600.0]);
        let mut xp = claimer(&mut core, "xp", [400.0, 0.0, 400.0, 600.0]);
        for wm in [&mut tiling, &mut xp] {
            wm.ok(
                &mut core,
                "input.grab",
                json!({"binding": "Super+BTN_LEFT"}),
            );
            wm.ok(&mut core, "input.grab", json!({"binding": "move"}));
        }
        let attach = json!([{"attach": {"surface": "s1", "parent": "tiling"}}]);
        apply(&mut tiling, &mut core, &attach);
        move_to(&mut core, 10.0, 10.0);
        press(&mut core, SUPER, true);
        let down = ButtonState::Pressed;
        super::press_button(&mut core.state, 272, down, InputTime::now(), injected);
        events(&mut tiling, &mut core);
        events(&mut xp, &mut core);
        (core, wayland, tiling, xp)
    }

    /// The transaction of tiling that gives s1 to xp with the drag.
    fn give_with_the_drag() -> Value {
        json!({"commands": [
            {"remove": {"id": "s1", "to": "xp"}},
            {"give_grab": {"surface": "s1", "to": "xp", "point": [12.0, 8.0],
                "velocity": [300.0, 0.0], "wobble": {"points": [1.5, 2.5]}}},
        ]})
    }

    /// The press that the receiver of a drag gets: `move` with the window
    /// at the pointer, and the state that the giver sent.
    fn is_the_press_of_the_given_drag(pressed: &Value) {
        assert_eq!(pressed["binding"], "move");
        assert_eq!(pressed["state"], "pressed");
        assert_eq!(pressed["surface"], "s1");
        assert_eq!(pressed["position"], json!([10.0, 10.0]));
        let grab = json!({"point": [12.0, 8.0], "velocity": [300.0, 0.0],
            "wobble": {"points": [1.5, 2.5]}});
        assert_eq!(pressed["grab"], grab);
    }

    /// E3: tiling gives a window to xp while the button is down. The core
    /// moves the pointer grab in the same transaction: xp gets the press
    /// of `move` with the window and the state of the drag, then the moves
    /// and the release, and tiling gets nothing more.
    #[test]
    fn a_give_in_the_middle_of_a_drag_moves_the_grab_to_the_receiver() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        tiling.ok(&mut core, "scene.apply", give_with_the_drag());
        let got = events(&mut xp, &mut core);
        let names: Vec<&str> = got.iter().filter_map(|e| e["event"].as_str()).collect();
        assert_eq!(names, ["surface.detached", "input.button"], "{got:?}");
        is_the_press_of_the_given_drag(&got[1]);
        let take = json!([{"attach": {"surface": "s1", "parent": "xp"}}]);
        apply(&mut xp, &mut core, &take);
        move_to(&mut core, 500.0, 300.0);
        click(&mut core, false);
        press(&mut core, SUPER, false);
        assert_eq!(buttons(&mut xp, &mut core), ["moved", "released"]);
        assert_eq!(buttons(&mut tiling, &mut core), Vec::<String>::new());
        assert!(core.state.button_grab.is_none());
        assert!(core.pointer_free());
    }

    /// The drag of an injected press stays with its window manager: the
    /// whole transaction fails and the window stays too.
    #[test]
    fn a_give_with_the_drag_of_an_injected_press_is_refused() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(true);
        let (_, message) = tiling.fails(&mut core, "scene.apply", give_with_the_drag());
        assert!(message.contains("real press"), "{message}");
        let owner = core.state.scene.node("s1").unwrap().owner.clone();
        assert_eq!(owner.as_deref(), Some("tiling"));
        assert_eq!(buttons(&mut xp, &mut core), Vec::<String>::new());
        click(&mut core, false);
        assert_eq!(buttons(&mut tiling, &mut core), ["released"]);
    }

    /// A window manager that answers a method call of a confined caller
    /// gives no drag: a confined agent must not move the pointer grab of
    /// the user through `window.give`.
    #[test]
    fn a_give_with_the_drag_for_a_confined_caller_is_refused() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        tiling.ok(
            &mut core,
            "methods.export",
            json!({"names": ["window.give"]}),
        );
        let mut admin = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["methods.call"]});
        admin.ok(&mut core, "grants.set", grant);
        let mut agent = core.connect();
        agent.ok(
            &mut core,
            "hello",
            json!({"name": "agent", "confined": true}),
        );
        let call = json!({"client": "tiling", "method": "window.give", "params": {}});
        agent.send("methods.call", call);
        core.dispatch();
        let (_, message) = tiling.fails(&mut core, "scene.apply", give_with_the_drag());
        assert!(message.contains("confined"), "{message}");
        assert_eq!(buttons(&mut xp, &mut core), Vec::<String>::new());
        let mut boxed = core.connect();
        boxed.ok(&mut core, "hello", json!({"name": "box", "confined": true}));
        let (_, message) = boxed.fails(&mut core, "scene.apply", give_with_the_drag());
        assert!(message.contains("confined"), "{message}");
    }

    /// The receiver ends in the middle of the drag: the grab goes with
    /// it, the release of the button reaches no client and the pointer is
    /// free again.
    #[test]
    fn a_receiver_that_dies_in_the_middle_of_the_drag_frees_the_press() {
        let (mut core, _wayland, mut tiling, xp) = mid_drag(false);
        tiling.ok(&mut core, "scene.apply", give_with_the_drag());
        assert!(core.state.button_grab.is_some());
        drop(xp);
        core.dispatch();
        assert!(core.state.button_grab.is_none(), "the grab outlived xp");
        move_to(&mut core, 500.0, 300.0);
        click(&mut core, false);
        press(&mut core, SUPER, false);
        assert_eq!(buttons(&mut tiling, &mut core), Vec::<String>::new());
        assert!(core.pointer_free());
    }

    /// The grab follows the window: a receiver that does not attach the
    /// window loses the grab when the core frees the window again, and
    /// gets the release, so that its drag ends too.
    #[test]
    fn a_receiver_that_does_not_take_the_window_loses_the_grab() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        tiling.ok(&mut core, "scene.apply", give_with_the_drag());
        events(&mut xp, &mut core);
        core.advance(std::time::Duration::from_secs(6));
        move_to(&mut core, 500.0, 300.0);
        assert_eq!(buttons(&mut xp, &mut core), ["released"]);
        assert!(core.state.button_grab.is_none());
    }

    /// xp drags a window by a press on its own title bar, a rect node: it
    /// gives that press with the window, and tiling gets the rest of the
    /// drag as `move`. xp hears no release.
    #[test]
    fn a_press_on_an_own_node_goes_with_the_window_too() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        click(&mut core, false);
        press(&mut core, SUPER, false);
        tiling.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"remove": {"id": "s1", "to": "xp"}}]}),
        );
        let title = json!({"create": {"id": "title", "kind": "rect", "parent": "xp",
            "props": {"size": [100, 20], "transform": {"translate": [500, 100]}}}});
        let take = json!({"attach": {"surface": "s1", "parent": "xp"}});
        apply(&mut xp, &mut core, &json!([take, title]));
        xp.ok(
            &mut core,
            "events.subscribe",
            json!({"events": ["pointer.*"]}),
        );
        move_to(&mut core, 510.0, 110.0);
        click(&mut core, true);
        events(&mut xp, &mut core);
        events(&mut tiling, &mut core);
        let give = json!({"commands": [
            {"remove": {"id": "s1", "to": "tiling"}},
            {"give_grab": {"surface": "s1", "to": "tiling", "point": [10.0, 10.0]}},
        ]});
        xp.ok(&mut core, "scene.apply", give);
        assert!(core.state.node_grab.is_none());
        move_to(&mut core, 100.0, 100.0);
        click(&mut core, false);
        assert_eq!(
            buttons(&mut tiling, &mut core),
            ["pressed", "moved", "released"]
        );
        let after = events(&mut xp, &mut core);
        let released = named(&after, "pointer.released");
        assert_eq!(released.len(), 0, "{after:?}");
        assert!(core.pointer_free());
    }

    /// The answer of tiling's give with the drag, as an error message.
    fn refused_give(core: &mut Core, tiling: &mut Client) -> String {
        tiling.fails(core, "scene.apply", give_with_the_drag()).1
    }

    /// The owner of the window s1.
    fn owner_of_s1(core: &Core) -> Option<String> {
        core.state.scene.node("s1").unwrap().owner.clone()
    }

    /// Review of E3: the press that the core saw at the check must still
    /// be the press when the transaction applies. The transaction waits
    /// for a configure; the user lets go and presses again in between, and
    /// the whole transaction fails: the window and the new drag stay.
    #[test]
    fn a_give_that_waited_fails_when_the_press_is_another_one() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        let mut give = give_with_the_drag();
        let resize = json!({"configure": {"surface": "s1", "size": [50, 40], "states": []}});
        give["commands"].as_array_mut().unwrap().insert(0, resize);
        let id = tiling.send("scene.apply", give);
        core.dispatch();
        assert_eq!(tiling.answer(&mut core, id), None, "the transaction waits");
        click(&mut core, false);
        click(&mut core, true);
        core.advance(crate::transaction::DEADLINE);
        let answer = tiling.answer(&mut core, id).expect("the deadline passed");
        let error = answer.expect_err("the press is another one");
        assert!(
            error["message"].as_str().unwrap().contains("press"),
            "{error}"
        );
        assert_eq!(owner_of_s1(&core).as_deref(), Some("tiling"));
        assert_eq!(buttons(&mut xp, &mut core), Vec::<String>::new());
        click(&mut core, false);
        let seen = buttons(&mut tiling, &mut core);
        assert_eq!(
            seen.last().map(String::as_str),
            Some("released"),
            "{seen:?}"
        );
    }

    /// Review of E3: a lock in the middle of the drag ends it, and with
    /// it the give.
    #[test]
    fn a_give_after_a_lock_in_the_middle_of_the_drag_is_refused() {
        let (mut core, mut wayland, mut tiling, mut xp) = mid_drag(false);
        let _lock = wayland.lock(&mut core);
        let message = refused_give(&mut core, &mut tiling);
        assert!(message.contains("holds no pointer grab"), "{message}");
        assert_eq!(owner_of_s1(&core).as_deref(), Some("tiling"));
        assert_eq!(buttons(&mut xp, &mut core), Vec::<String>::new());
    }

    /// Review of E3: any injected input ends the real press, also a move
    /// of the pointer: an agent must not steer a drag that then changes
    /// its window manager.
    #[test]
    fn a_give_after_an_injected_move_is_refused() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        let mut admin = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        admin.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        let moved = json!({"events": [{"pointer_move": {"position": [500.0, 300.0]}}]});
        agent.ok(&mut core, "input.inject", moved);
        let message = refused_give(&mut core, &mut tiling);
        assert!(message.contains("real press"), "{message}");
        assert_eq!(buttons(&mut xp, &mut core), Vec::<String>::new());
    }

    /// Review of E3: a real press that ended is no licence for a later
    /// injected one of the same button.
    #[test]
    fn a_give_with_an_injected_press_after_a_real_one_is_refused() {
        let (mut core, _wayland, mut tiling, _xp) = mid_drag(false);
        click(&mut core, false);
        let down = ButtonState::Pressed;
        super::press_button(&mut core.state, 272, down, InputTime::now(), true);
        let message = refused_give(&mut core, &mut tiling);
        assert!(message.contains("real press"), "{message}");
    }

    /// Review of E3: only the drag of the window itself goes. A press
    /// beside the window is no drag of it.
    #[test]
    fn a_give_of_a_window_that_the_press_did_not_hit_is_refused() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        click(&mut core, false);
        move_to(&mut core, 200.0, 200.0);
        click(&mut core, true);
        let message = refused_give(&mut core, &mut tiling);
        assert!(message.contains("s1"), "{message}");
        assert_eq!(owner_of_s1(&core).as_deref(), Some("tiling"));
        assert_eq!(buttons(&mut xp, &mut core), Vec::<String>::new());
    }

    /// Review of E3: xp holds a press on its own node while tiling holds
    /// the grab of a button binding. xp must not give a drag then, and
    /// the grab of tiling stays.
    #[test]
    fn a_give_does_not_take_the_grab_of_another_client() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        click(&mut core, false);
        press(&mut core, SUPER, false);
        let hand = json!({"commands": [{"remove": {"id": "s1", "to": "xp"}}]});
        tiling.ok(&mut core, "scene.apply", hand);
        let title = json!({"create": {"id": "title", "kind": "rect", "parent": "xp",
            "props": {"size": [100, 20], "transform": {"translate": [500, 100]}}}});
        let take = json!({"attach": {"surface": "s1", "parent": "xp"}});
        apply(&mut xp, &mut core, &json!([take, title]));
        move_to(&mut core, 510.0, 110.0);
        click(&mut core, true);
        assert!(core.state.node_grab.is_some());
        let other = json!({"binding": "Super+BTN_RIGHT"});
        tiling.ok(&mut core, "input.grab", other);
        press(&mut core, SUPER, true);
        let right = ButtonState::Pressed;
        super::press_button(&mut core.state, 273, right, InputTime::now(), false);
        let held = core.state.button_grab.as_ref().map(|g| g.conn);
        assert!(held.is_some(), "tiling holds the right button");
        let give = json!({"commands": [
            {"remove": {"id": "s1", "to": "tiling"}},
            {"give_grab": {"surface": "s1", "to": "tiling", "point": [10.0, 10.0]}},
        ]});
        let (_, message) = xp.fails(&mut core, "scene.apply", give);
        assert!(message.contains("another client"), "{message}");
        assert_eq!(core.state.button_grab.as_ref().map(|g| g.conn), held);
    }

    /// The test mode of the headless backend: an injected button press
    /// gives its drag, so that a gate can pull a window out of a face.
    /// Without the mode the same give is refused.
    #[test]
    fn the_test_mode_lets_an_injected_drag_give() {
        let (mut core, _wayland, mut tiling, mut xp) = mid_drag(false);
        click(&mut core, false);
        core.state.injected_press_is_real = true;
        let down = ButtonState::Pressed;
        super::press_button(&mut core.state, 272, down, InputTime::now(), true);
        tiling.ok(&mut core, "scene.apply", give_with_the_drag());
        let got = events(&mut xp, &mut core);
        assert!(got.iter().any(|e| e["event"] == "input.button"), "{got:?}");
    }

    /// Review of E3: the state of a drag is data of another client: the
    /// core refuses a wobble past its size limit.
    #[test]
    fn a_give_with_a_huge_wobble_is_refused() {
        let (mut core, _wayland, mut tiling, _xp) = mid_drag(false);
        let mut give = give_with_the_drag();
        give["commands"][1]["give_grab"]["wobble"] = json!("w".repeat(20_000));
        let (_, message) = tiling.fails(&mut core, "scene.apply", give);
        assert!(message.contains("wobble"), "{message}");
    }
    const SLOT_1: f64 = -262_144.0;

    /// A manager of a virtual output that grabbed Alt+y, one on the
    /// screen under the pointer, and the pointer there.
    fn manager_of_a_face_and_one_under_the_pointer() -> (Core, Client, Client) {
        let mut core = Core::new();
        let mut face = core.named("face");
        face.ok(
            &mut core,
            "output.add",
            json!({"spec": "32x32", "virtual": true}),
        );
        core.add_output("screen", (64, 48));
        apply(
            &mut face,
            &mut core,
            &json!([{"create": {"id": "face", "kind": "group", "parent": "root",
                "props": {"claim": [SLOT_1, SLOT_1, 32.0, 32.0]}}}]),
        );
        face.ok(&mut core, "input.grab", json!({"binding": "Alt+y"}));
        let space = claimer(&mut core, "space", [0.0, 0.0, 64.0, 48.0]);
        move_to(&mut core, 5.0, 5.0);
        (core, face, space)
    }

    fn alt_y(core: &Core, injected: bool) -> Option<super::KeyAction> {
        let mods = smithay::input::keyboard::ModifiersState {
            alt: true,
            ..Default::default()
        };
        super::grab_action(
            &core.state,
            &mods,
            &[smithay::input::keyboard::Keysym::y],
            injected,
        )
    }

    #[test]
    fn a_grab_held_back_by_the_pointer_is_swallowed_not_passed_on() {
        let (core, _face, _space) = manager_of_a_face_and_one_under_the_pointer();
        assert!(
            matches!(alt_y(&core, false), Some(super::KeyAction::Swallow)),
            "the key must not reach the focused client"
        );
    }

    #[test]
    fn an_injected_key_skips_the_pointer_gate() {
        let (core, _face, _space) = manager_of_a_face_and_one_under_the_pointer();
        assert!(
            matches!(alt_y(&core, true), Some(super::KeyAction::Grab(_, _, true))),
            "an injected key goes to the grab"
        );
    }

    #[test]
    fn a_key_without_a_grab_is_not_swallowed() {
        let (core, _face, _space) = manager_of_a_face_and_one_under_the_pointer();
        let mods = smithay::input::keyboard::ModifiersState::default();
        let none = super::grab_action(
            &core.state,
            &mods,
            &[smithay::input::keyboard::Keysym::y],
            false,
        );
        assert!(none.is_none(), "a plain key goes to the client");
    }
    #[test]
    fn an_injected_key_that_counts_as_real_keeps_the_pointer_gate() {
        let (mut core, _face, _space) = manager_of_a_face_and_one_under_the_pointer();
        core.state.injected_press_is_real = true;
        assert!(
            matches!(alt_y(&core, true), Some(super::KeyAction::Swallow)),
            "in the test mode an injected key is a key of the person"
        );
    }

    const Y: u32 = 21;

    /// The `input.key` presses that a client got.
    fn key_presses(client: &mut Client, core: &mut Core) -> Vec<Value> {
        events(client, core)
            .into_iter()
            .filter(|e| e["event"] == "input.key" && e["state"] == "pressed")
            .collect()
    }

    /// I11 (sh-24): the manager of a face bound Alt+y, and its window, a
    /// probe client, has the keyboard; the pointer rests on the claim of
    /// another manager. The key goes to the manager of the face, and the
    /// probe gets Alt, no raw y.
    #[test]
    fn a_key_that_a_look_binds_skips_its_focused_probe_and_reaches_the_look() {
        let (mut core, mut face, mut space) = manager_of_a_face_and_one_under_the_pointer();
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "probe");
        wayland.attach(&mut core, &window, (20, 20));
        let give = json!([{"attach": {"surface": "s1", "parent": "space"}},
            {"remove": {"id": "s1", "to": "face"}}]);
        apply(&mut space, &mut core, &give);
        let take = json!([{"attach": {"surface": "s1", "parent": "face"}},
            {"focus": {"surface": "s1"}}]);
        apply(&mut face, &mut core, &take);
        key_presses(&mut face, &mut core);
        press(&mut core, ALT, true);
        tap(&mut core, Y);
        press(&mut core, ALT, false);
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.key_serials().len(), 2, "the probe gets Alt only");
        let got = key_presses(&mut face, &mut core);
        assert_eq!(got.len(), 1, "the look gets Alt+y: {got:?}");
    }

    /// Keyboard after a stale pointer (sh-24): the manager of a face took
    /// the keyboard for its node while the pointer rests on the claim of
    /// another manager. Its key goes to it; the pointer is stale.
    #[test]
    fn the_keys_follow_the_keyboard_not_a_stale_pointer() {
        let (mut core, mut face, _space) = manager_of_a_face_and_one_under_the_pointer();
        let menu = json!([{"create": {"id": "menu", "kind": "rect", "parent": "face",
            "props": {"size": [10, 10], "color": [1, 1, 1, 1]}}}]);
        apply(&mut face, &mut core, &menu);
        apply(&mut face, &mut core, &focus_node(Some("menu")));
        assert!(
            matches!(
                alt_y(&core, false),
                Some(super::KeyAction::Grab(_, _, false))
            ),
            "the manager with the keyboard gets its key"
        );
    }

    /// rw-3: a clicked layer surface (the search field of a panel) has the
    /// keyboard while the window of a manager of a face still is the
    /// focus the core wants. The place is not that manager: the keys go
    /// where the pointer is.
    #[test]
    fn the_key_place_is_not_the_window_owner_while_a_layer_has_the_keyboard() {
        let (mut core, mut face, mut space) = manager_of_a_face_and_one_under_the_pointer();
        let mut wayland = Wayland::connect(&mut core);
        let window = wayland.toplevel(&mut core, "probe");
        wayland.attach(&mut core, &window, (20, 20));
        let give = json!([{"attach": {"surface": "s1", "parent": "space"}},
            {"remove": {"id": "s1", "to": "face"}}]);
        apply(&mut space, &mut core, &give);
        let take = json!([{"attach": {"surface": "s1", "parent": "face"}},
            {"focus": {"surface": "s1"}}]);
        apply(&mut face, &mut core, &take);
        assert!(
            matches!(
                alt_y(&core, false),
                Some(super::KeyAction::Grab(_, _, false))
            ),
            "the manager of the window that has the keyboard gets its key"
        );
        wayland.layer(
            &mut core,
            smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer::Top,
            smithay::reexports::wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
        );
        let field = core.state.all_outputs().iter().find_map(|output| {
            let map = smithay::desktop::layer_map_for_output(output);
            map.layers().next().map(|l| l.wl_surface().clone())
        });
        assert!(field.is_some(), "the layer is mapped");
        core.state.set_layer_focus(field);
        assert!(
            matches!(alt_y(&core, false), Some(super::KeyAction::Swallow)),
            "the layer has the keyboard: the key place is where the pointer is"
        );
    }

    /// sh-24: the keyboard is on a node of the manager on the screen, and
    /// the manager of a face bound Alt+y. An injected Alt+y follows the
    /// rule of a real one: the person is not on the face, so the key
    /// reaches no client, neither the face nor the focused node.
    #[test]
    fn an_injected_key_follows_the_keyboard_place_as_a_real_one() {
        let (mut core, _face, mut space) = manager_of_a_face_and_one_under_the_pointer();
        let field = json!([{"create": {"id": "field", "kind": "rect", "parent": "space",
            "props": {"size": [10, 10], "color": [1, 1, 1, 1]}}}]);
        apply(&mut space, &mut core, &field);
        apply(&mut space, &mut core, &focus_node(Some("field")));
        assert!(
            matches!(alt_y(&core, false), Some(super::KeyAction::Swallow)),
            "a real key of the face waits for the face"
        );
        assert!(
            matches!(alt_y(&core, true), Some(super::KeyAction::Swallow)),
            "an injected key waits for the face too"
        );
    }
}
