//! The rule of key grabs between window managers: where the person is
//! when a key goes down, and which grabs wait for that place. The safety
//! layer (D23) is decided before this rule and stays above every plugin.

use smithay::{
    input::keyboard::{Keysym, ModifiersState},
    utils::{Logical, Point},
};

use super::{KeyAction, State};

/// The grab of a binding, unless the session is locked. A grab that waits
/// for the manager of another place swallows the key: it must not reach
/// the focused client as a raw key (sh-24). An injected key follows the
/// same place, the keyboard of the person, but never the pointer, which
/// an injection moves too: without a keyboard place it goes to the grab.
/// In the test mode of `injected_press_is_real` it counts as a key of
/// the person.
pub(super) fn grab_action(
    state: &State,
    mods: &ModifiersState,
    syms: &[Keysym],
    injected: bool,
) -> Option<KeyAction> {
    if state.lock.is_some() {
        return None;
    }
    state.ipc.grab_for_where(mods, syms, |_| true)?;
    let by_pointer = !injected || state.injected_press_is_real;
    let place = key_place(state, by_pointer);
    let held_back = |conn| {
        place
            .as_deref()
            .is_some_and(|place| holds_other_claim(state, conn, place))
    };
    Some(
        match state.ipc.grab_for_where(mods, syms, |c| !held_back(c)) {
            Some((conn, binding)) => KeyAction::Grab(conn, binding, injected),
            None => KeyAction::Swallow,
        },
    )
}

/// The window manager where the person is when a key goes down: the one
/// that a click on its empty claim chose (D55), else the owner of the
/// keyboard focus, of a node or of a window (no window while a clicked
/// layer surface has the keyboard), else, `by_pointer`, the one whose
/// claim holds the pointer. The keyboard goes before the pointer, which
/// may rest on any pane while the person types.
fn key_place(state: &State, by_pointer: bool) -> Option<String> {
    if let Some(chosen) = state.chosen_manager() {
        return Some(chosen.into());
    }
    let scene = &state.scene;
    let window_owner = || {
        let surface = state
            .focused_surface()
            .filter(|_| !state.layer_has_keyboard())?;
        scene.node(surface)?.owner.as_deref()
    };
    let focus_owner = (scene.focused_node_owner())
        .or_else(window_owner)
        .filter(|owner| state.ipc.live(owner) && scene.holds_claim(owner));
    if let Some(owner) = focus_owner {
        return Some(owner.into());
    }
    if !by_pointer {
        return None;
    }
    let spot = state.spot(state.pointer_layout)?;
    let point = [spot.scene.x, spot.scene.y];
    chora_scene::claimant_at(scene.nodes(), point, |n| state.ipc.live(n)).map(String::from)
}

/// Whether the client of `conn` is a window manager of virtual outputs,
/// other than `place`, where the person is: its claims all lie on
/// virtual outputs, as those of a manager in a pane or on a face of a
/// cube do. A space that shows them on a screen is no such manager, and
/// keeps its keys.
pub(crate) fn holds_other_claim(state: &State, conn: u64, place: &str) -> bool {
    let Some(name) = state.ipc.name(conn).filter(|name| *name != place) else {
        return false;
    };
    let virtuals: Vec<_> = (state.assets.virtuals.iter())
        .map(|v| crate::output::output_area(v.output()).to_f64())
        .collect();
    let mut claims = (state.scene.nodes())
        .filter(|n| n.owner.as_deref() == Some(name))
        .filter_map(|n| n.props.claim)
        .peekable();
    claims.peek().is_some()
        && claims.all(|[x, y, _, _]| {
            virtuals
                .iter()
                .any(|area| area.contains(Point::<f64, Logical>::from((x, y))))
        })
}
