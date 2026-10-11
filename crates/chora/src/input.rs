//! Input routing: the safety layer first, then grabs, then the focused
//! surface. The core never changes the focus by itself.

use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
    },
    input::{
        keyboard::{FilterResult, Keysym, ModifiersState},
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    utils::SERIAL_COUNTER,
};

use crate::{render, state::State};

/// What a key press does besides reaching a client.
enum KeyAction {
    /// The release of a key whose press went elsewhere.
    Swallow,
    /// Stop the compositor.
    Quit,
    /// The emergency reset.
    Reset,
    /// Send `input.key` to the client that grabbed the binding.
    Grab(u64, String),
}

pub fn process<I: InputBackend>(state: &mut State, event: InputEvent<I>) {
    match event {
        InputEvent::Keyboard { event } => keyboard::<I>(state, &event),
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
        Some(KeyAction::Grab(conn, binding)) => state
            .ipc
            .send_to(conn, &chora_scene::Event::InputKey { binding }),
        Some(KeyAction::Swallow) | None => {}
    }
}

/// The fixed bindings of the safety layer (D23), then the grabs.
fn press_action(state: &State, mods: &ModifiersState, syms: &[Keysym]) -> Option<KeyAction> {
    if mods.ctrl && mods.alt {
        if syms.contains(&Keysym::BackSpace) {
            return Some(KeyAction::Quit);
        }
        if syms.contains(&Keysym::Escape) {
            return Some(KeyAction::Reset);
        }
    }
    let (conn, binding) = state.ipc.grab_for(mods, syms)?;
    Some(KeyAction::Grab(conn, binding))
}

fn motion<I: InputBackend>(state: &mut State, event: &I::PointerMotionAbsoluteEvent) {
    let pos = event.position_transformed(state.output_size());
    let under = render::surface_under(state, pos);
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    pointer.motion(
        state,
        under,
        &MotionEvent {
            location: pos,
            serial: SERIAL_COUNTER.next_serial(),
            time: event.time(),
        },
    );
    pointer.frame(state);
}

fn button<I: InputBackend>(state: &mut State, event: &I::PointerButtonEvent) {
    let pointer = state.seat.get_pointer().expect("the seat has a pointer");
    let button_state: ButtonState = event.state();
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
