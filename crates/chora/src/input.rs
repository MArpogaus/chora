//! Input routing: the safety layer first, then grabs, then the focused
//! surface.
//!
//! Plugins set the focus. The core changes it in three cases only: the
//! reset focuses the first surface, a click on a surface without an owner
//! focuses it, and a surface that becomes detached loses it. Protocols take
//! the keyboard for a while: a popup grab, a layer surface such as a
//! launcher, and the lock screen.

use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        InputTime, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
        PointerMotionEvent,
    },
    input::{
        keyboard::{FilterResult, Keysym, ModifiersState},
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    utils::{Logical, Point, SERIAL_COUNTER},
};

use crate::{output::output_area, render, state::State};

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
}

pub fn process<I: InputBackend>(state: &mut State, event: InputEvent<I>) {
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
    let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
    let (code, key_state) = (event.key_code(), event.state());
    let action = keyboard.input(
        state,
        code,
        key_state,
        SERIAL_COUNTER.next_serial(),
        event.time(),
        |state, mods, handle| {
            if key_state == KeyState::Released {
                let before = state.suppressed_keys.len();
                state.suppressed_keys.retain(|k| *k != code);
                return if state.suppressed_keys.len() < before {
                    FilterResult::Intercept(KeyAction::Swallow)
                } else {
                    FilterResult::Forward
                };
            }
            match press_action(state, mods, &handle.raw_syms()) {
                Some(action) => {
                    state.suppressed_keys.push(code);
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
        Some(KeyAction::Grab(conn, binding)) => state
            .ipc
            .send_to(conn, &chora_scene::Event::InputKey { binding }),
        Some(KeyAction::Swallow) | None => {}
    }
}

/// The fixed bindings of the safety layer (D23), then the grabs. The keymap
/// turns Ctrl+Alt+F1 to F12 into the keys that switch the VT.
fn press_action(state: &State, mods: &ModifiersState, syms: &[Keysym]) -> Option<KeyAction> {
    let first = Keysym::XF86_Switch_VT_1.raw();
    if let Some(vt) = syms
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
    if state.lock.is_some() {
        return None;
    }
    let (conn, binding) = state.ipc.grab_for(mods, syms)?;
    Some(KeyAction::Grab(conn, binding))
}

/// Relative motion, as from a mouse on real hardware. The pointer stays on
/// the outputs.
fn relative_motion<I: InputBackend>(state: &mut State, event: &I::PointerMotionEvent) {
    let layout = state.clamp_to_outputs(state.pointer_layout + event.delta());
    move_pointer(state, layout, event.time());
}

/// Absolute motion, as from the winit window, which shows the first output.
fn motion<I: InputBackend>(state: &mut State, event: &I::PointerMotionAbsoluteEvent) {
    let Some(area) = state.outputs.first().map(output_area) else {
        return;
    };
    let layout = area.loc.to_f64() + event.position_transformed(area.size);
    move_pointer(state, layout, event.time());
}

/// Moves the pointer to a point of the layout and tells the surface under
/// it.
fn move_pointer(state: &mut State, layout: Point<f64, Logical>, time: InputTime) {
    state.pointer_layout = layout;
    let Some(spot) = state.spot(layout) else {
        return;
    };
    let under = render::surface_under(state, &spot).map(|u| (u.surface, u.location));
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: spot.scene,
            serial: SERIAL_COUNTER.next_serial(),
            time,
        },
    );
    pointer.frame(state);
}

fn button<I: InputBackend>(state: &mut State, event: &I::PointerButtonEvent) {
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    let button_state: ButtonState = event.state();
    if button_state == ButtonState::Pressed {
        click_focus(state);
    }
    pointer.button(
        state,
        &ButtonEvent {
            button: event.button_code(),
            state: button_state,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time(),
        },
    );
    pointer.frame(state);
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
