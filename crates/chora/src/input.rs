//! Input routing: the safety layer first, then grabs, then the focused
//! surface.
//!
//! Plugins set the focus. The core changes it in three cases only: the
//! reset focuses the first surface, a click on a surface without an owner
//! focuses it, and a surface that becomes detached loses it. Protocols take
//! the keyboard for a while: a popup grab, a layer surface such as a
//! launcher, and the lock screen.

use std::sync::atomic::Ordering;

use chora_scene::{Event as SceneEvent, NodeKind, ipc::PressState};
use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        InputTime, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent,
    },
    input::{
        keyboard::{FilterResult, Keycode, Keysym, ModifiersState},
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
    state::{ClientState, NodeGrab, State},
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
    Grab(u64, String),
    /// Send the release of a grabbed key to the client of the grab.
    Ungrab(u64, String),
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

/// A key goes down or up: the safety layer first, then the grabs, then the
/// focused surface. An injected key never reaches the safety layer (D23).
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
                let Some(i) = state.suppressed_keys.iter().position(|(k, _)| *k == code) else {
                    return FilterResult::Forward;
                };
                return FilterResult::Intercept(match state.suppressed_keys.remove(i).1 {
                    Some((conn, binding)) => KeyAction::Ungrab(conn, binding),
                    None => KeyAction::Swallow,
                });
            }
            let syms = (handle.raw_syms(), handle.modified_syms());
            match press_action(state, mods, &syms, injected) {
                Some(action) => {
                    let grab = match &action {
                        KeyAction::Grab(conn, binding) => Some((*conn, binding.clone())),
                        _ => None,
                    };
                    // A repeated press without a release keeps one entry.
                    state.suppressed_keys.retain(|(k, _)| *k != code);
                    state.suppressed_keys.push((code, grab));
                    FilterResult::Intercept(action)
                }
                None => FilterResult::Forward,
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
        Some(KeyAction::Grab(conn, binding)) => grabbed(state, conn, binding, PressState::Pressed),
        Some(KeyAction::Ungrab(conn, binding)) => {
            grabbed(state, conn, binding, PressState::Released);
        }
        Some(KeyAction::Swallow) => {}
        None => {
            if key_state == KeyState::Pressed {
                note_press(state, keyboard.current_focus(), serial);
            }
        }
    }
}

/// Tells the client of a grab that its key went down or up.
fn grabbed(state: &mut State, conn: u64, binding: String, press: PressState) {
    let event = chora_scene::Event::InputKey {
        binding,
        state: press,
    };
    state.ipc.send_to(conn, &event);
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
    let under = state
        .spot(state.pointer_layout)
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
    state.pointer_layout = layout;
    let Some(spot) = state.spot(layout) else {
        return;
    };
    if state.node_grab.is_none() {
        hover(state, &spot);
    }
    // The drag of a node goes to plugins only, with each new position.
    let under = if let Some(grab) = &mut state.node_grab {
        let position = position_for(&state.scene, &state.outputs, &grab.node, &spot);
        #[allow(clippy::float_cmp)] // the same input gives the same numbers
        if grab.position != position {
            grab.position = position;
            let node = grab.node.clone();
            state.emit(&SceneEvent::PointerMoved { node, position });
        }
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
    if let Some((surface, location)) = under {
        activate_constraint(&pointer, &surface, spot.scene - location);
    }
}

fn button<I: InputBackend>(state: &mut State, event: &I::PointerButtonEvent) {
    press_button(state, event.button_code(), event.state(), event.time());
}

/// A pointer button goes down or up.
pub fn press_button(state: &mut State, code: u32, button_state: ButtonState, time: InputTime) {
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    let serial = SERIAL_COUNTER.next_serial();
    if button_state == ButtonState::Pressed {
        click_focus(state);
        note_press(state, pointer.current_focus(), serial);
        press_node(state, code);
    } else {
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

/// Scrolls by logical pixels, as a touchpad does.
pub fn scroll(state: &mut State, dx: f64, dy: f64, time: InputTime) {
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
fn hover(state: &mut State, spot: &Spot) {
    let node = render::node_under(state, spot);
    if node == state.hovered {
        return;
    }
    if let Some(node) = std::mem::replace(&mut state.hovered, node.clone()) {
        state.emit(&SceneEvent::PointerLeft { node });
    }
    if let Some(node) = node {
        state.emit(&SceneEvent::PointerEntered { node });
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
    state.emit(&SceneEvent::PointerReleased {
        node,
        button,
        position,
    });
}

/// Notes the serial of a press for the client that gets it. Its popups may
/// grab the input with this serial only.
fn note_press(state: &State, surface: Option<WlSurface>, serial: Serial) {
    let client = surface.and_then(|s| state.display_handle.get_client(s.id()).ok());
    if let Some(data) = client.as_ref().and_then(|c| c.get_data::<ClientState>()) {
        data.last_press.store(u32::from(serial), Ordering::Relaxed);
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

fn axis<I: InputBackend>(state: &mut State, event: &I::PointerAxisEvent) {
    let source = event.source();
    let mut frame = AxisFrame::new(event.time()).source(source);
    for axis in [Axis::Horizontal, Axis::Vertical] {
        let discrete = event.amount_v120(axis);
        let amount = event
            .amount(axis)
            .unwrap_or_else(|| discrete.unwrap_or(0.0) * 15.0 / 120.0);
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
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    pointer.axis(state, frame);
    pointer.frame(state);
}
