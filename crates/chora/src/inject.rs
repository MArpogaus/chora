//! `input.inject`: synthetic input of agents (D33). It needs a grant of the
//! user and fails while the session is locked, so that no client types into
//! the lock screen. Injected keys never reach the safety layer (D23); the
//! grabs of plugins see them. Keys and buttons that a request leaves down
//! go up at its end, so that no key sticks. A button press with `hold`
//! stays down until a later event releases it.
//!
//! Key names and text resolve through the first layout of the keymap: a
//! character needs a key of that layout, with Shift or `AltGr` at most.

use std::collections::HashMap;

use chora_scene::{
    Binding,
    ipc::{Capability, Inject, InjectEvent, Key, PressState, RpcError},
};
use serde_json::Value;
use smithay::{
    backend::input::{ButtonState, InputTime, KeyState},
    input::keyboard::{Keycode, Keysym, xkb},
    utils::{Logical, Point},
};

use crate::{
    input,
    ipc::{params_of, rejected},
    output::output_area,
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

/// One step of an injection, with its keys and points resolved.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Move(Point<f64, Logical>),
    /// The button, its state, and whether a press stays down at the end.
    Button(u32, ButtonState, bool),
    Scroll(f64, f64),
    Key(Keycode, KeyState),
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
            return Err(rejected(format!(
                "a request has at most {MAX_EVENTS} events"
            )));
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
            return Err(rejected(format!(
                "a request types at most {MAX_TEXT} characters"
            )));
        }
        let mut steps = Vec::new();
        for event in &inject.events {
            steps.extend(self.steps(event, keys.as_ref()).map_err(rejected)?);
        }
        if steps.len() > MAX_STEPS {
            return Err(rejected(format!("a request has at most {MAX_STEPS} steps")));
        }
        if inject.dry_run {
            return Ok(Value::Null);
        }
        self.run_steps(&steps);
        Ok(Value::Null)
    }

    /// The steps of one event.
    fn steps(&self, event: &InjectEvent, keys: Option<&Keys>) -> Result<Vec<Step>, String> {
        let keys = || keys.ok_or_else(|| String::from("no keymap"));
        Ok(match event {
            InjectEvent::PointerMove { position, output } => {
                vec![Step::Move(self.inject_point(*position, output.as_deref())?)]
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
            InjectEvent::Text { text } => keys()?.text(text)?,
        })
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

    fn run_steps(&mut self, steps: &[Step]) {
        let seat = self.seat.clone();
        self.idle_notifier_state.notify_activity(&seat);
        let mut keys_down = Vec::new();
        let mut buttons_down = Vec::new();
        for step in steps {
            let now = InputTime::now();
            match *step {
                Step::Move(point) => input::move_pointer(self, point, now),
                Step::Button(button, state, hold) => {
                    let pressed = state == ButtonState::Pressed && !hold;
                    note(&mut buttons_down, button, pressed);
                    input::press_button(self, button, state, now);
                }
                Step::Scroll(dx, dy) => input::scroll(self, dx, dy, now),
                Step::Key(code, state) => {
                    note(&mut keys_down, code, state == KeyState::Pressed);
                    input::key(self, code, state, now, true);
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
    use crate::testing::Core;
    use serde_json::json;

    /// An `input.inject` request that presses the left button, with `hold`.
    fn button_press(hold: bool) -> Value {
        let mut button = json!({"state": "pressed"});
        if hold {
            button["hold"] = json!(true);
        }
        json!({"events": [{"button": button}]})
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
    fn the_list_of_held_keys_follows_presses_and_releases() {
        let mut down = Vec::new();
        note(&mut down, 1, true);
        note(&mut down, 2, true);
        note(&mut down, 1, false);
        note(&mut down, 2, true);
        assert_eq!(down, vec![2]);
    }
}
