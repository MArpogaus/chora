//! The drag of a node through a pane: where it starts, and the pointer grab that follows it.

use super::{
    AxisFrame, ButtonEvent, ButtonState, Focus, GestureHoldBeginEvent, GestureHoldEndEvent,
    GesturePinchBeginEvent, GesturePinchEndEvent, GesturePinchUpdateEvent, GestureSwipeBeginEvent,
    GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData, Logical, MotionEvent, Output,
    Point, PointerGrab, PointerHandle, PointerInnerHandle, PopupPointerGrab, RelativeMotionEvent,
    Serial, Spot, State, WlSurface, output_area, render,
};

/// Where a node is reached from: the list of the root or of the overlay
/// of an output, and the pointer in its coordinates. Under the overlay of
/// an output the coordinates are those of that output, also while a drag
/// crosses to another one; elsewhere they are scene coordinates.
pub(crate) fn anchor(
    scene: &chora_scene::Scene,
    outputs: &[Output],
    node: &str,
    spot: &Spot,
) -> (String, Point<f64, Logical>) {
    let overlay = outputs
        .iter()
        .find(|o| scene.is_within(node, &chora_scene::overlay_id(&o.name())));
    match overlay {
        Some(o) => (
            chora_scene::overlay_id(&o.name()),
            spot.local + (output_area(&spot.output).loc - output_area(o).loc).to_f64(),
        ),
        None => (chora_scene::ROOT.into(), spot.scene),
    }
}

/// The position of the pointer for a node, as `anchor` tells it.
pub(super) fn position_for(
    scene: &chora_scene::Scene,
    outputs: &[Output],
    node: &str,
    spot: &Spot,
) -> [f64; 2] {
    let (_, at) = anchor(scene, outputs, node, spot);
    [at.x, at.y]
}

/// The pointer in the coordinates of a node, mapped through its planes
/// without a test of the inside. `None` when a plane shows the pointer
/// behind the eye.
pub(super) fn local_for(
    scene: &chora_scene::Scene,
    outputs: &[Output],
    node: &str,
    spot: &Spot,
) -> Option<[f64; 2]> {
    let (start, at) = anchor(scene, outputs, node, spot);
    let local = render::local_in(scene, &start, at, node)?;
    Some([local.x, local.y])
}

/// Starts the press of a surface under a plane at the first button, in
/// place of the click grab that Smithay would start on that surface.
pub(super) fn start_plane_grab(
    state: &mut State,
    pointer: &PointerHandle<State>,
    button: u32,
    serial: Serial,
) {
    if pointer.is_grabbed() {
        return;
    }
    let focus = pointer.current_focus();
    let Some(pin) = Pin::under(state, focus.as_ref()) else {
        return;
    };
    let grab = PlaneGrab {
        start: GrabStartData {
            focus: Some((pin.surface.clone(), pointer.current_location() - pin.last)),
            button,
            location: pointer.current_location(),
        },
        popup: None,
        pin: Some(pin),
    };
    pointer.set_grab(state, grab, serial, Focus::Keep);
}

/// The press on a surface under a plane: its node, the surface, the
/// origin of the surface in the coordinates of the node, and the last
/// point on the surface that the client got.
pub(super) struct Pin {
    node: String,
    /// The pane that forwarded the press.
    via: Option<String>,
    surface: WlSurface,
    origin: Point<f64, Logical>,
    last: Point<f64, Logical>,
}

impl Pin {
    /// The press on the surface under the pointer, if the pointer has
    /// that surface as its focus and it is under a plane.
    fn under(state: &State, focus: Option<&WlSurface>) -> Option<Self> {
        let spot = state.spot(state.pointer_layout)?;
        let under = render::surface_under(state, &spot)?;
        let (Some(origin), Some(node)) = (under.plane, under.node) else {
            return None;
        };
        (focus == Some(&under.surface)).then(|| Self {
            node,
            via: spot.via,
            last: spot.scene - under.location,
            surface: under.surface,
            origin,
        })
    }

    /// The surface with the location that gives its client the point that
    /// the plane maps to the pointer at `location`. The node is reached
    /// from the root or the overlay of an output, as `anchor` tells. When
    /// the plane shows the pointer behind the eye, the client keeps its
    /// last point. `None` when the node left the scene, the last frame
    /// left its plane out, or the pointer left the pane of the press, as
    /// an injected drag does.
    fn mapped(
        &mut self,
        state: &State,
        location: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        let spot = state.spot(state.pointer_layout)?;
        let (start, at) = anchor(&state.scene, &state.all_outputs(), &self.node, &spot);
        if spot.via != self.via
            || !state.scene.is_within(&self.node, &start)
            || render::left_out(state, &spot.output, &self.node)
        {
            return None;
        }
        if let Some(local) = render::local_in(&state.scene, &start, at, &self.node) {
            self.last = local - self.origin;
        }
        Some((self.surface.clone(), location - self.last))
    }
}

/// The grab of a press on a surface under a plane, alone or around the
/// grab of a popup. Each motion of the press maps through the plane, also
/// outside the node, and goes to the surface of the press. Alone, the
/// grab ends when the last button goes up; around a popup, the press
/// ends and the popup keeps the pointer. Any other grab ends it, and with
/// it the press.
pub struct PlaneGrab {
    start: GrabStartData<State>,
    popup: Option<PopupPointerGrab<State>>,
    pin: Option<Pin>,
}

impl PlaneGrab {
    /// The grab of a popup, in which a press under a plane maps through
    /// the plane.
    pub fn popup(popup: PopupPointerGrab<State>) -> Self {
        Self {
            start: popup.start_data().clone(),
            popup: Some(popup),
            pin: None,
        }
    }
}

/// Passes an event of a gesture to the popup, or else to the focus.
macro_rules! gestures {
    ($($name:ident: $event:ty),* $(,)?) => {$(
        fn $name(
            &mut self,
            data: &mut State,
            handle: &mut PointerInnerHandle<'_, State>,
            event: &$event,
        ) {
            match &mut self.popup {
                Some(popup) => popup.$name(data, handle, event),
                None => handle.$name(data, event),
            }
        }
    )*};
}

impl PointerGrab<State> for PlaneGrab {
    fn motion(
        &mut self,
        data: &mut State,
        handle: &mut PointerInnerHandle<'_, State>,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &MotionEvent,
    ) {
        let mapped = self
            .pin
            .as_mut()
            .map(|pin| pin.mapped(data, event.location));
        if mapped.as_ref().is_some_and(Option::is_none) {
            self.pin = None;
            if self.popup.is_none() {
                handle.unset_grab(self, data, event.serial, event.time, true);
                return;
            }
        }
        let focus = mapped.flatten().or(focus);
        match &mut self.popup {
            Some(popup) => popup.motion(data, handle, focus, event),
            None => handle.motion(data, focus, event),
        }
    }

    fn relative_motion(
        &mut self,
        data: &mut State,
        handle: &mut PointerInnerHandle<'_, State>,
        focus: Option<(WlSurface, Point<f64, Logical>)>,
        event: &RelativeMotionEvent,
    ) {
        let location = handle.current_location();
        let focus = match &self.pin {
            Some(pin) => Some((pin.surface.clone(), location - pin.last)),
            None => focus,
        };
        match &mut self.popup {
            Some(popup) => popup.relative_motion(data, handle, focus, event),
            None => handle.relative_motion(data, focus, event),
        }
    }

    fn button(
        &mut self,
        data: &mut State,
        handle: &mut PointerInnerHandle<'_, State>,
        event: &ButtonEvent,
    ) {
        let first = handle.current_pressed().len() == 1;
        if event.state == ButtonState::Pressed && first && self.pin.is_none() {
            let focus = handle.current_focus().map(|(surface, _)| surface);
            self.pin = Pin::under(data, focus.as_ref());
        }
        let released = handle.current_pressed().is_empty();
        if let Some(popup) = &mut self.popup {
            popup.button(data, handle, event);
            if released {
                self.pin = None;
            }
        } else {
            handle.button(data, event);
            if released {
                handle.unset_grab(self, data, event.serial, event.time, true);
            }
        }
    }

    fn axis(
        &mut self,
        data: &mut State,
        handle: &mut PointerInnerHandle<'_, State>,
        details: AxisFrame,
    ) {
        match &mut self.popup {
            Some(popup) => popup.axis(data, handle, details),
            None => handle.axis(data, details),
        }
    }

    fn frame(&mut self, data: &mut State, handle: &mut PointerInnerHandle<'_, State>) {
        match &mut self.popup {
            Some(popup) => popup.frame(data, handle),
            None => handle.frame(data),
        }
    }

    gestures! {
        gesture_swipe_begin: GestureSwipeBeginEvent,
        gesture_swipe_update: GestureSwipeUpdateEvent,
        gesture_swipe_end: GestureSwipeEndEvent,
        gesture_pinch_begin: GesturePinchBeginEvent,
        gesture_pinch_update: GesturePinchUpdateEvent,
        gesture_pinch_end: GesturePinchEndEvent,
        gesture_hold_begin: GestureHoldBeginEvent,
        gesture_hold_end: GestureHoldEndEvent,
    }

    fn start_data(&self) -> &GrabStartData<State> {
        &self.start
    }

    fn unset(&mut self, data: &mut State) {
        if let Some(popup) = &mut self.popup {
            popup.unset(data);
        }
    }
}
