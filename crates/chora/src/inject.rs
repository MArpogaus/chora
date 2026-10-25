//! `input.inject`: synthetic input of agents (D33). It needs a grant of the
//! user and fails while the session is locked, so that no client types into
//! the lock screen. Injected keys never reach the safety layer (D23); the
//! grabs of plugins see them. Keys and buttons that a request leaves down
//! go up at its end, so that no key sticks. A button press with `hold`
//! stays down until a later event releases it.
//!
//! Every executed request shows a small yellow mark of the core next to
//! the pointer, gone 500 ms after the last injected event (D23).
//!
//! Key names and text resolve through the first layout of the keymap: a
//! character needs a key of that layout, with Shift or `AltGr` at most.
//! A text goes whole as one commit to the focus with an enabled text
//! input, so it also carries characters the keymap cannot reach.

use std::{collections::HashMap, time::Duration};

use chora_scene::{
    Binding, Node, NodeKind, Transform,
    ipc::{Capability, Inject, InjectEvent, Key, PressState, RpcError},
};
use serde_json::Value;
use smithay::{
    backend::input::{ButtonState, InputTime, KeyState},
    input::keyboard::{Keycode, Keysym, xkb},
    utils::{Logical, Point},
    wayland::text_input::TextInputSeat,
};

use crate::{
    input,
    ipc::{params_of, rejected},
    output::output_area,
    render::node_size,
    state::State,
};

/// The most events of one request.
const MAX_EVENTS: usize = 256;
/// The longest text of one request, in characters, over all its events.
const MAX_TEXT: usize = 4096;
/// The most steps of one request: key presses and releases, moves, button
/// presses and scrolls.
const MAX_STEPS: usize = 32_768;
/// The offset of XKB key codes from Linux key codes.
const XKB_OFFSET: u32 = 8;
/// How long the injection mark stays after the last injected event (D23).
const MARK_FOR: Duration = Duration::from_millis(500);
/// The distance of the mark from the pointer, in logical pixels.
const MARK_GAP: f64 = 8.0;

/// One step of an injection, with its keys and points resolved.
#[derive(Debug, Clone, PartialEq)]
enum Step {
    Move(Point<f64, Logical>),
    /// The button, its state, and whether a press stays down at the end.
    Button(u32, ButtonState, bool),
    Scroll(f64, f64),
    Key(Keycode, KeyState),
    /// A text that goes as one commit to the text input of the focus.
    Commit(String),
}

impl State {
    /// `input.inject`. Checks every event before the first one runs.
    pub fn ipc_inject(&mut self, conn: u64, params: Value) -> Result<Value, RpcError> {
        self.check_grant(conn, Capability::InputInject)?;
        if self.lock.is_some() {
            return Err(rejected("the session is locked"));
        }
        let inject: Inject = params_of(params)?;
        if inject.events.len() > MAX_EVENTS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a request has at most {MAX_EVENTS} events"),
            ));
        }
        let needs_keys = inject.events.iter().any(|e| {
            matches!(
                e,
                InjectEvent::Key {
                    key: Key::Name(_),
                    ..
                } | InjectEvent::Shortcut { .. }
                    | InjectEvent::Text { .. }
            )
        });
        let keys = if needs_keys {
            Some(Keys::of(self).map_err(rejected)?)
        } else {
            None
        };
        let text: usize = inject
            .events
            .iter()
            .map(|e| match e {
                InjectEvent::Text { text } => text.chars().count(),
                _ => 0,
            })
            .sum();
        if text > MAX_TEXT {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a request types at most {MAX_TEXT} characters"),
            ));
        }
        let mut steps = Vec::new();
        for event in &inject.events {
            steps.extend(self.steps(event, keys.as_ref()).map_err(rejected)?);
        }
        if steps.len() > MAX_STEPS {
            return Err(RpcError::new(
                RpcError::LIMIT_REACHED,
                format!("a request has at most {MAX_STEPS} steps"),
            ));
        }
        if inject.dry_run {
            return Ok(Value::Null);
        }
        self.run_steps(&steps);
        self.mark_injection();
        Ok(Value::Null)
    }

    /// Shows the injection mark (D23): a small yellow rect of the core
    /// next to the pointer, gone `MARK_FOR` after the last injected
    /// event. The mark belongs to the core, so no client can remove it.
    fn mark_injection(&mut self) {
        let until = self.time.now() + MARK_FOR;
        self.inject_mark = Some(until);
        let _ = self.expire_at(until);
        if let Some(spot) = self.spot(self.pointer_layout) {
            let at = [spot.local.x + MARK_GAP, spot.local.y + MARK_GAP];
            self.scene.show_injection(&spot.output.name(), at);
        }
    }

    /// Removes the injection mark.
    pub(crate) fn unmark_injection(&mut self) {
        self.inject_mark = None;
        self.scene.remove_injection();
    }

    /// The steps of one event.
    fn steps(&self, event: &InjectEvent, keys: Option<&Keys>) -> Result<Vec<Step>, String> {
        let keys = || keys.ok_or_else(|| String::from("no keymap"));
        Ok(match event {
            InjectEvent::PointerMove {
                position,
                output,
                node,
                point,
            } => {
                let at = match (position, node, point) {
                    (Some(position), None, None) => {
                        self.inject_point(*position, output.as_deref())?
                    }
                    (None, Some(node), Some(point)) => {
                        self.node_point(node, *point, output.as_deref())?
                    }
                    _ => return Err("pointer_move needs position or node and point".into()),
                };
                vec![Step::Move(at)]
            }
            InjectEvent::Button {
                button,
                state,
                hold,
            } => vec![Step::Button(*button, button_state(*state), *hold)],
            InjectEvent::Click { button } => vec![
                Step::Button(*button, ButtonState::Pressed, false),
                Step::Button(*button, ButtonState::Released, false),
            ],
            InjectEvent::Scroll { dx, dy } => vec![Step::Scroll(*dx, *dy)],
            InjectEvent::Key { key, state } => {
                let code = match key {
                    Key::Code(code) => linux_key(*code)?,
                    Key::Name(name) => keys()?.named(name)?,
                };
                vec![Step::Key(code, key_state(*state))]
            }
            InjectEvent::Shortcut { keys: binding } => keys()?.shortcut(binding)?,
            InjectEvent::Text { text } => {
                if self.commits_text() {
                    vec![Step::Commit(text.clone())]
                } else {
                    keys()?.text(text)?
                }
            }
        })
    }

    /// Whether the focus has an enabled text input: the core then commits
    /// its texts to it instead of typing keys.
    fn commits_text(&self) -> bool {
        let mut enabled = false;
        self.seat
            .text_input()
            .with_active_text_input(|_, _| enabled = true);
        enabled
    }

    /// A point in the logical coordinates of an output, in the layout.
    fn inject_point(
        &self,
        [x, y]: [f64; 2],
        output: Option<&str>,
    ) -> Result<Point<f64, Logical>, String> {
        if !(x.is_finite() && y.is_finite()) {
            return Err("the position is not finite".into());
        }
        let output = match output {
            Some(name) => self.outputs.iter().find(|o| o.name() == name),
            None => self.outputs.first(),
        }
        .ok_or("no such output")?;
        let point = output_area(output).loc.to_f64() + Point::from((x, y));
        Ok(self.clamp_to_outputs(point))
    }

    /// A point inside a node, in the coordinates of the node, as a point
    /// of the layout. Maps it through the same transforms as the `rects`
    /// of a query, so pan, zoom and scaled parents need no client math.
    /// The point must lie in the box the node draws; `output`, if given,
    /// limits the search to that output. A mesh, a perspective or a clip
    /// may draw the point elsewhere.
    fn node_point(
        &self,
        node: &str,
        [x, y]: [f64; 2],
        output: Option<&str>,
    ) -> Result<Point<f64, Logical>, String> {
        if !(x.is_finite() && y.is_finite()) {
            return Err("the point is not finite".into());
        }
        let outputs: &[_] = match output {
            Some(name) => std::slice::from_ref(
                self.outputs
                    .iter()
                    .find(|o| o.name() == name)
                    .ok_or("no such output")?,
            ),
            None => &self.outputs,
        };
        for output in outputs {
            let name = output.name();
            let area = output_area(output);
            // The overlay sits in the coordinates of its output, without
            // its camera (D43).
            let views = [
                (Transform::IDENTITY, self.scene.overlay_draw_list(&name)),
                (self.camera(output), self.scene.draw_list()),
            ];
            for (camera, list) in views {
                let Some(placed) = list
                    .iter()
                    .find(|p| p.node.id == node && self.draws_something(p.node))
                else {
                    continue;
                };
                if !self.shows_point(placed.node, x, y) {
                    continue;
                }
                let (at_x, at_y) = camera.then(&placed.transform).apply(x, y);
                if at_x >= 0.0
                    && at_y >= 0.0
                    && at_x < f64::from(area.size.w)
                    && at_y < f64::from(area.size.h)
                {
                    return Ok(area.loc.to_f64() + Point::from((at_x, at_y)));
                }
            }
        }
        Err(format!("the node {node:?} shows no point [{x}, {y}]"))
    }

    /// Whether a point of the node's own coordinates lies in the box the
    /// node draws. The box of a surface is its bounds, which include
    /// subsurfaces and decorations. The bounds are the box of the bounds:
    /// `hit` in the renderer is stricter, it also takes the clips and the
    /// surface tree into account.
    fn shows_point(&self, node: &Node, x: f64, y: f64) -> bool {
        let (at_x, at_y, w, h) = match node.kind {
            NodeKind::Rect | NodeKind::Image => {
                let [w, h] = node_size(self, node);
                (0.0, 0.0, w, h)
            }
            NodeKind::Surface => {
                let Some(window) = self.windows.get(node.id.as_str()) else {
                    return false;
                };
                let bbox = window.bbox().to_f64();
                let geo = window.geometry().loc.to_f64();
                (
                    bbox.loc.x - geo.x,
                    bbox.loc.y - geo.y,
                    bbox.size.w,
                    bbox.size.h,
                )
            }
            // A text node draws inside the box of its layout.
            NodeKind::Text => match self.assets.texts.size(node) {
                Some([w, h]) => (0.0, 0.0, w, h),
                None => return false,
            },
            NodeKind::Group | NodeKind::Shader => return false,
        };
        (at_x..at_x + w).contains(&x) && (at_y..at_y + h).contains(&y)
    }

    /// Whether a node draws at a place of its own: groups and shaders
    /// draw nothing, a surface without a window draws nothing, and a
    /// text node draws when its layout exists.
    fn draws_something(&self, node: &Node) -> bool {
        match node.kind {
            NodeKind::Surface => self.windows.contains_key(node.id.as_str()),
            NodeKind::Rect | NodeKind::Image => true,
            NodeKind::Text => node.props.text.is_some(),
            NodeKind::Group | NodeKind::Shader => false,
        }
    }

    fn run_steps(&mut self, steps: &[Step]) {
        let seat = self.seat.clone();
        self.idle_notifier_state.notify_activity(&seat);
        let mut keys_down = Vec::new();
        let mut buttons_down = Vec::new();
        for step in steps {
            let now = InputTime::now();
            match step {
                Step::Move(point) => input::move_pointer(self, *point, now),
                Step::Button(button, state, hold) => {
                    let pressed = *state == ButtonState::Pressed && !hold;
                    note(&mut buttons_down, *button, pressed);
                    input::press_button(self, *button, *state, now);
                }
                Step::Scroll(dx, dy) => input::scroll(self, *dx, *dy, now),
                Step::Key(code, state) => {
                    note(&mut keys_down, *code, *state == KeyState::Pressed);
                    input::key(self, *code, *state, now, true);
                }
                Step::Commit(text) => {
                    let text_input = self.seat.text_input();
                    text_input
                        .with_active_text_input(|input, _| input.commit_string(Some(text.clone())));
                    text_input.done(false);
                }
            }
        }
        for code in keys_down.into_iter().rev() {
            input::key(self, code, KeyState::Released, InputTime::now(), true);
        }
        for button in buttons_down {
            input::press_button(self, button, ButtonState::Released, InputTime::now());
        }
    }
}

/// Keeps the list of keys or buttons that are down.
fn note<T: PartialEq>(down: &mut Vec<T>, item: T, pressed: bool) {
    down.retain(|d| *d != item);
    if pressed {
        down.push(item);
    }
}

fn button_state(state: PressState) -> ButtonState {
    match state {
        PressState::Pressed => ButtonState::Pressed,
        PressState::Released => ButtonState::Released,
    }
}

fn key_state(state: PressState) -> KeyState {
    match state {
        PressState::Pressed => KeyState::Pressed,
        PressState::Released => KeyState::Released,
    }
}

/// The XKB key code of a Linux key code.
fn linux_key(code: u32) -> Result<Keycode, String> {
    if code == 0 || code > 767 {
        return Err(format!("no key code {code}"));
    }
    Ok(Keycode::new(code + XKB_OFFSET))
}

/// The keymap of the seat, compiled again, so that safe code can read it.
struct Keys {
    /// For each keysym of the first layout: the key, and the modifier keys
    /// with the fewest modifiers.
    ways: HashMap<Keysym, (Keycode, Vec<Keycode>)>,
    /// For each keysym: the first key whose first level gives it.
    plain: HashMap<Keysym, Keycode>,
}

impl Keys {
    fn of(state: &mut State) -> Result<Self, String> {
        let keyboard = state.seat.get_keyboard().expect("the seat has a keyboard");
        let source = keyboard
            .with_xkb_state(state, |context| {
                context.xkb().lock().map(|xkb| xkb.keymap_as_string()).ok()
            })
            .ok_or("the keymap is in use")?;
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_string(
            &context,
            source,
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or("the keymap does not compile")?;
        Ok(Self::from_keymap(&keymap))
    }

    /// Reads every key of the first layout once.
    fn from_keymap(keymap: &xkb::Keymap) -> Self {
        let codes = || (keymap.min_keycode().raw()..=keymap.max_keycode().raw()).map(Keycode::new);
        let mut plain = HashMap::new();
        for code in codes() {
            for sym in keymap.key_get_syms_by_level(code, 0, 0) {
                plain.entry(*sym).or_insert(code);
            }
        }
        let mut ways = HashMap::new();
        for code in codes() {
            for level in 0..keymap.num_levels_for_key(code, 0) {
                add_level(keymap, &plain, &mut ways, code, level);
            }
        }
        Self { ways, plain }
    }

    /// The key of a keysym name, without its modifiers.
    fn named(&self, name: &str) -> Result<Keycode, String> {
        let mut sym = xkb::keysym_from_name(name, xkb::KEYSYM_NO_FLAGS);
        if sym.raw() == 0 {
            sym = xkb::keysym_from_name(name, xkb::KEYSYM_CASE_INSENSITIVE);
        }
        if sym.raw() == 0 {
            return Err(format!("unknown key {name:?}"));
        }
        self.find(sym)
            .map(|(code, _)| code)
            .ok_or_else(|| format!("the keymap has no key {name:?}"))
    }

    /// The steps of a shortcut: the modifiers down, the key down and up,
    /// the modifiers up.
    fn shortcut(&self, binding: &Binding) -> Result<Vec<Step>, String> {
        let m = binding.mods;
        let mut mods = Vec::new();
        for (on, sym) in [
            (m.ctrl, Keysym::Control_L),
            (m.alt, Keysym::Alt_L),
            (m.shift, Keysym::Shift_L),
            (m.logo, Keysym::Super_L),
        ] {
            if on {
                mods.push(
                    self.plain(sym)
                        .ok_or_else(|| format!("the keymap has no {sym:?}"))?,
                );
            }
        }
        Ok(press(&mods, self.named(&binding.key)?))
    }

    /// The steps that type a text.
    fn text(&self, text: &str) -> Result<Vec<Step>, String> {
        let mut steps = Vec::new();
        for c in text.chars() {
            let sym = match c {
                '\n' => Keysym::Return,
                '\t' => Keysym::Tab,
                _ => xkb::utf32_to_keysym(u32::from(c)),
            };
            let (code, mods) = self
                .find(sym)
                .ok_or_else(|| format!("the keymap cannot type {c:?}"))?;
            steps.extend(press(&mods, code));
        }
        Ok(steps)
    }

    /// The key and the modifier keys that give a keysym on the first
    /// layout, with the fewest modifiers.
    fn find(&self, sym: Keysym) -> Option<(Keycode, Vec<Keycode>)> {
        self.ways.get(&sym).cloned()
    }

    /// The key whose first level gives a keysym.
    fn plain(&self, sym: Keysym) -> Option<Keycode> {
        self.plain.get(&sym).copied()
    }
}

/// Notes the keysyms of a level of a key, unless a key with fewer
/// modifiers gives them.
fn add_level(
    keymap: &xkb::Keymap,
    plain: &HashMap<Keysym, Keycode>,
    ways: &mut HashMap<Keysym, (Keycode, Vec<Keycode>)>,
    code: Keycode,
    level: xkb::LevelIndex,
) {
    let mut masks = [0; 8];
    let n = keymap.key_get_mods_for_level(code, 0, level, &mut masks);
    let Some(mods) = masks
        .into_iter()
        .take(n)
        .filter_map(|mask| modifier_keys(keymap, plain, mask))
        .min_by_key(Vec::len)
    else {
        return;
    };
    for sym in keymap.key_get_syms_by_level(code, 0, level) {
        if ways
            .get(sym)
            .is_none_or(|(_, old): &(Keycode, Vec<Keycode>)| mods.len() < old.len())
        {
            ways.insert(*sym, (code, mods.clone()));
        }
    }
}

/// The keys that set the modifiers of a mask: Shift and `AltGr` only.
fn modifier_keys(
    keymap: &xkb::Keymap,
    plain: &HashMap<Keysym, Keycode>,
    mask: xkb::ModMask,
) -> Option<Vec<Keycode>> {
    let mut keys = Vec::new();
    for index in 0..keymap.num_mods().min(32) {
        if mask & (1 << index) == 0 {
            continue;
        }
        let sym = match keymap.mod_get_name(index) {
            "Shift" => Keysym::Shift_L,
            "Mod5" | "LevelThree" => Keysym::ISO_Level3_Shift,
            _ => return None,
        };
        keys.push(*plain.get(&sym)?);
    }
    Some(keys)
}

/// Modifiers down, a key down and up, the modifiers up again.
fn press(mods: &[Keycode], code: Keycode) -> Vec<Step> {
    let mut steps: Vec<Step> = mods
        .iter()
        .map(|m| Step::Key(*m, KeyState::Pressed))
        .collect();
    steps.push(Step::Key(code, KeyState::Pressed));
    steps.push(Step::Key(code, KeyState::Released));
    steps.extend(mods.iter().rev().map(|m| Step::Key(*m, KeyState::Released)));
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{Core, wayland::Wayland};
    use chora_scene::INJECTION;
    use serde_json::json;

    /// An `input.inject` request that presses the left button, with `hold`.
    fn button_press(hold: bool) -> Value {
        let mut button = json!({"state": "pressed"});
        if hold {
            button["hold"] = json!(true);
        }
        json!({"events": [{"button": button}]})
    }

    /// Gate check 9 of 0.7: the limit of a request has its own code.
    #[test]
    fn more_events_than_the_limit_answer_limit_reached() {
        let mut core = Core::new();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        core.admin().ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        let events: Vec<Value> = (0..=MAX_EVENTS)
            .map(|_| json!({"click": {"button": 272}}))
            .collect();
        let (error_code, message) = agent.fails(
            &mut core,
            "input.inject",
            json!({"events": events, "dry_run": true}),
        );
        assert_eq!(error_code, RpcError::LIMIT_REACHED);
        assert!(message.contains("at most"), "{message}");
    }

    #[test]
    fn a_dry_run_checks_the_events_and_sends_nothing() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // The watcher reports the pointer events of its rect at the origin.
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"create": {"id": "menu", "kind": "rect",
                "parent": "root", "props": {"size": [100, 100],
                "color": [1, 1, 1, 1]}}}]}),
        );
        watcher.ok(&mut core, "events.subscribe", json!({"events": []}));
        // A wrong key symbol fails the check.
        let bad = json!({"dry_run": true,
            "events": [{"key": {"key": "NoSuchKey", "state": "pressed"}}]});
        let (_, message) = agent.fails(&mut core, "input.inject", bad);
        assert!(message.contains("NoSuchKey"), "{message}");
        // A good batch passes and sends nothing.
        let batch = json!({"events": [
            {"pointer_move": {"position": [50.0, 50.0]}},
            {"button": {"state": "pressed"}}
        ]});
        let mut dry = batch.clone();
        dry["dry_run"] = json!(true);
        agent.ok(&mut core, "input.inject", dry);
        let names = watcher.event_names(&mut core);
        assert!(
            !names.iter().any(|e| e.starts_with("pointer.")),
            "the dry run sends nothing: {names:?}"
        );
        // The same batch without the dry run does send.
        agent.ok(&mut core, "input.inject", batch);
        let names = watcher.event_names(&mut core);
        assert!(names.iter().any(|e| e == "pointer.pressed"), "{names:?}");
    }

    #[test]
    fn a_held_button_stays_down_until_a_later_request_releases_it() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // The watcher reports the button events of its rect at the origin.
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"create": {"id": "menu", "kind": "rect",
                "parent": "root", "props": {"size": [100, 100],
                "color": [1, 1, 1, 1]}}}]}),
        );
        watcher.ok(&mut core, "events.subscribe", json!({"events": []}));
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [{"pointer_move": {"position": [50.0, 50.0]}}]}),
        );
        watcher.event_names(&mut core);
        agent.ok(&mut core, "input.inject", button_press(true));
        let names = watcher.event_names(&mut core);
        assert!(names.iter().any(|e| e == "pointer.pressed"), "{names:?}");
        assert!(
            !names.iter().any(|e| e == "pointer.released"),
            "the held button stays down: {names:?}"
        );
        // A second request releases it.
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [{"button": {"state": "released"}}]}),
        );
        let names = watcher.event_names(&mut core);
        assert!(names.iter().any(|e| e == "pointer.released"), "{names:?}");
        // Without hold the core releases the button at the end.
        agent.ok(&mut core, "input.inject", button_press(false));
        let names = watcher.event_names(&mut core);
        assert!(names.iter().any(|e| e == "pointer.pressed"), "{names:?}");
        assert!(
            names.iter().any(|e| e == "pointer.released"),
            "the request ends its buttons: {names:?}"
        );
    }

    #[test]
    fn a_node_point_lands_where_the_query_shows_the_rect() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // The watcher owns a child under a parent that doubles it and
        // moves it, so node coordinates and screen coordinates differ.
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"create": {"id": "big", "kind": "group", "parent": "root",
                    "props": {"transform": {"scale": [2.0, 2.0],
                                            "translate": [40.0, 20.0]}}}},
                {"create": {"id": "child", "kind": "rect", "parent": "big",
                    "props": {"size": [100.0, 100.0], "color": [1, 1, 1, 1],
                              "transform": {"translate": [10.0, 20.0]}}}},
            ]}),
        );
        watcher.ok(&mut core, "events.subscribe", json!({"events": []}));
        let found = watcher.ok(&mut core, "scene.query", json!({"selector": "id:child"}));
        let rect: Vec<f64> = found[0]["rects"]["HDMI-1"]
            .as_array()
            .expect("a rect")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect();
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [
                {"pointer_move": {"node": "child", "point": [50.0, 50.0]}},
                {"button": {"state": "pressed"}}
            ]}),
        );
        let events = watcher.events_now(&mut core);
        let pressed = events
            .iter()
            .find(|e| e["params"]["event"] == json!("pointer.pressed"))
            .expect("the click lands on the child");
        let at: Vec<f64> = pressed["params"]["position"]
            .as_array()
            .expect("a position")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect();
        // The middle of the node, read through the rects of the query.
        assert_eq!(at, vec![rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0]);
        // A move with neither position nor node and point fails.
        let (_, message) = agent.fails(
            &mut core,
            "input.inject",
            json!({"events": [{"pointer_move": {}}]}),
        );
        assert!(message.contains("position or node"), "{message}");
        // A move to a node that shows nowhere fails.
        let (_, message) = agent.fails(
            &mut core,
            "input.inject",
            json!({"events": [{"pointer_move": {"node": "gone", "point": [1.0, 1.0]}}]}),
        );
        assert!(message.contains("gone"), "{message}");
    }

    #[test]
    fn a_node_point_is_sought_on_the_named_output_only() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        core.add_output("DP-1", (800, 600));
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // A node fixed to the overlay of DP-1 shows on that output only.
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"create": {"id": "badge", "kind": "rect",
                "parent": "screen:DP-1", "props": {"size": [100.0, 100.0],
                "color": [1, 1, 1, 1]}}}]}),
        );
        watcher.ok(&mut core, "events.subscribe", json!({"events": []}));
        let found = watcher.ok(&mut core, "scene.query", json!({"selector": "id:badge"}));
        let rect: Vec<f64> = found[0]["rects"]["DP-1"]
            .as_array()
            .expect("a rect on DP-1")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect();
        // The output that shows the node nowhere rejects the request.
        let bad = json!({"pointer_move": {"node": "badge", "point": [10.0, 10.0],
                                          "output": "HDMI-1"}});
        let (_, message) = agent.fails(&mut core, "input.inject", json!({"events": [bad]}));
        assert!(message.contains("badge"), "{message}");
        // A point outside the box the node draws fails on its own output.
        let bad = json!({"pointer_move": {"node": "badge", "point": [150.0, 50.0],
                                          "output": "DP-1"}});
        let (_, message) = agent.fails(&mut core, "input.inject", json!({"events": [bad]}));
        assert!(message.contains("badge"), "{message}");
        // The named output takes the pointer to the node there.
        let tap = json!({"pointer_move": {"node": "badge", "point": [10.0, 10.0],
                                          "output": "DP-1"}});
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [tap, {"button": {"state": "pressed"}}]}),
        );
        let events = watcher.events_now(&mut core);
        let pressed = events
            .iter()
            .find(|e| e["params"]["event"] == json!("pointer.pressed"))
            .expect("the click lands on DP-1");
        let at: Vec<f64> = pressed["params"]["position"]
            .as_array()
            .expect("a position")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect();
        assert_eq!(at, vec![rect[0] + 10.0, rect[1] + 10.0]);
    }

    #[test]
    fn a_node_point_of_a_surface_lands_where_the_query_shows_its_rect() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // A toplevel attached to a parent that doubles it: node
        // coordinates and screen coordinates differ.
        let mut windows = Wayland::connect(&mut core);
        let window = windows.toplevel(&mut core, "app");
        windows.attach(&mut core, &window, (100, 80));
        let mut watcher = core.named("watcher");
        watcher.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"create": {"id": "big", "kind": "group", "parent": "root",
                    "props": {"transform": {"scale": [2.0, 2.0]}}}},
                {"attach": {"surface": "s1", "parent": "big"}},
            ]}),
        );
        watcher.ok(&mut core, "events.subscribe", json!({"events": []}));
        let found = watcher.ok(&mut core, "scene.query", json!({"selector": "id:s1"}));
        let rect: Vec<f64> = found[0]["rects"]["HDMI-1"]
            .as_array()
            .expect("a rect")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect();
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [
                {"pointer_move": {"node": "s1", "point": [30.0, 25.0]}},
                {"button": {"state": "pressed"}}
            ]}),
        );
        let events = watcher.events_now(&mut core);
        let pressed = events
            .iter()
            .find(|e| e["params"]["event"] == json!("pointer.pressed"))
            .expect("the click lands on the surface");
        let at: Vec<f64> = pressed["params"]["position"]
            .as_array()
            .expect("a position")
            .iter()
            .map(|v| v.as_f64().expect("a number"))
            .collect();
        assert_eq!(at, vec![rect[0] + 60.0, rect[1] + 50.0]);
    }

    fn us() -> Keys {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "",
            "pc105",
            "us",
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .expect("the us keymap compiles");
        Keys::from_keymap(&keymap)
    }

    fn key(code: u32, state: KeyState) -> Step {
        Step::Key(Keycode::new(code + XKB_OFFSET), state)
    }

    #[test]
    fn a_text_goes_as_one_commit_to_the_text_input_of_the_focus() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        // A client with a US keymap and an enabled text input.
        let mut wayland = Wayland::connect(&mut core);
        let _window = wayland.toplevel(&mut core, "app");
        let mut wm = core.named("wm");
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"attach": {"surface": "s1", "parent": "root"}},
                {"focus": {"surface": "s1"}},
            ]}),
        );
        wayland.enable_text_input(&mut core);
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // Umlauts that the US keymap cannot reach.
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [{"text": {"text": "Grüße äöüß"}}]}),
        );
        wayland.roundtrip(&mut core);
        assert_eq!(wayland.commits(), ["Grüße äöüß"]);
        // Without a text input the same text still needs the keymap.
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"focus": {"surface": null}}]}),
        );
        let node = json!({"events": [{"text": {"text": "ü"}}]});
        let (_, message) = agent.fails(&mut core, "input.inject", node);
        assert!(message.contains("the keymap cannot type 'ü'"), "{message}");
    }

    #[test]
    fn text_types_with_shift_where_needed() {
        use KeyState::{Pressed, Released};
        let keys = us();
        // a is key 30, Shift_L 42, Return 28.
        assert_eq!(
            keys.text("aA\n").unwrap(),
            vec![
                key(30, Pressed),
                key(30, Released),
                key(42, Pressed),
                key(30, Pressed),
                key(30, Released),
                key(42, Released),
                key(28, Pressed),
                key(28, Released),
            ]
        );
        assert!(keys.text("é").is_err());
    }

    #[test]
    fn a_shortcut_holds_its_modifiers() {
        use KeyState::{Pressed, Released};
        let keys = us();
        // Control_L is key 29, t is key 20.
        let binding: Binding = "Ctrl+t".parse().unwrap();
        assert_eq!(
            keys.shortcut(&binding).unwrap(),
            vec![
                key(29, Pressed),
                key(20, Pressed),
                key(20, Released),
                key(29, Released)
            ]
        );
    }

    #[test]
    fn keys_go_by_name_or_code() {
        let keys = us();
        assert_eq!(keys.named("Return").unwrap(), Keycode::new(36));
        assert_eq!(keys.named("return").unwrap(), Keycode::new(36));
        assert!(keys.named("NoSuchKey").is_err());
        assert_eq!(linux_key(28).unwrap(), Keycode::new(36));
        assert!(linux_key(0).is_err());
    }

    #[test]
    fn a_mark_shows_injection_and_goes_after_half_a_second() {
        let mut core = Core::new();
        core.add_output("HDMI-1", (800, 600));
        let mut ctl = core.admin();
        let grant = json!({"client": "agent", "capabilities": ["input.inject"]});
        ctl.ok(&mut core, "grants.set", grant);
        let mut agent = core.named("agent");
        // A rejected request shows nothing.
        let bad = json!({"events": [{"key": {"key": "NoSuchKey", "state": "pressed"}}]});
        agent.fails(&mut core, "input.inject", bad);
        assert!(core.state.scene.node(INJECTION).is_none());
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [{"pointer_move": {"position": [100.0, 80.0]}}]}),
        );
        let mark = core.state.scene.node(INJECTION).expect("the mark shows");
        assert_eq!(mark.parent.as_deref(), Some("screen:HDMI-1"));
        assert!(mark.owner.is_none(), "the mark belongs to the core");
        assert_eq!(
            mark.props.transform.apply(0.0, 0.0),
            (108.0, 88.0),
            "the mark sits next to the pointer"
        );
        // A new injection moves the mark and starts the time again.
        core.advance(Duration::from_millis(400));
        agent.ok(
            &mut core,
            "input.inject",
            json!({"events": [{"pointer_move": {"position": [300.0, 200.0]}}]}),
        );
        core.advance(Duration::from_millis(400));
        let mark = core
            .state
            .scene
            .node(INJECTION)
            .expect("the mark stays with the last event");
        assert_eq!(mark.props.transform.apply(0.0, 0.0).0, 308.0);
        core.advance(Duration::from_millis(200));
        assert!(
            core.state.scene.node(INJECTION).is_none(),
            "the mark goes 500 ms after the last event"
        );
    }

    #[test]
    fn the_list_of_held_keys_follows_presses_and_releases() {
        let mut down = Vec::new();
        note(&mut down, 1, true);
        note(&mut down, 2, true);
        note(&mut down, 1, false);
        note(&mut down, 2, true);
        assert_eq!(down, vec![2]);
    }
}
