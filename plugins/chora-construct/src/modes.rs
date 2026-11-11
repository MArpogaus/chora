//! The modes of the view on the cube (K4, `docs/cube-modes.md`): in use
//! mode the front face fills the view and takes the input; in navigate
//! mode the cube stands back, the construct has the keyboard, and the
//! person turns the cube or steps into a face. The chips at the bottom
//! right, above the taskbar of the face, are the visible way: STEP BACK
//! in use mode, TURN LEFT, TURN RIGHT and STEP IN in navigate mode. `chora_kit::modes` holds the
//! model; this file draws it and takes its input.

use std::time::Instant;

use chora_kit::{
    modes::{Act, Mode, View},
    words::{Language, Word},
};
use chora_scene::{Event, ipc::PressState};
use serde_json::{Value, json};

use super::{
    Action, BTN_LEFT, ClientError, Command, Construct, FLY_MS, Grab, LOOK, NAME, NodeKind,
    PropsUpdate, Query, RpcError, create, cube,
};

/// The group of the chips, and the prefix of a chip.
const MODES: &str = "construct-modes";
const CHIP: &str = "construct-mode-";
/// The turn keys of a cube, as on the cube of xp on a screen.
const TURN_KEYS: [(&str, i32); 2] = [("Ctrl+Alt+Left", -1), ("Ctrl+Alt+Right", 1)];
/// The event of the window manager of the cube while its window
/// switcher shows; the switcher needs the binding of the step back.
pub(crate) const SWITCHER: &str = "xp.switcher";
/// The room that the chips leave for a taskbar that the construct
/// cannot see yet.
const BAR: f64 = 40.0;
/// The chips in the order of the screen, from the left.
const CHIPS: [Word; 4] = [
    Word::StepBack,
    Word::TurnLeft,
    Word::TurnRight,
    Word::StepIn,
];

/// Who does an act. Only a person moves the keyboard (principle 11): a
/// method or the voice leaves it where it is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum By {
    Person,
    Method,
}

/// The name of a chip in its node id.
fn chip_name(word: Word) -> &'static str {
    match word {
        Word::StepBack => "step-back",
        Word::TurnLeft => "turn-left",
        Word::TurnRight => "turn-right",
        _ => "step-in",
    }
}

fn chip_id(word: Word) -> String {
    format!("{CHIP}{}", chip_name(word))
}

/// The act of a chip.
fn chip_act(word: Word) -> Act {
    match word {
        Word::StepBack => Act::StepBack,
        Word::TurnLeft => Act::Turn(-1),
        Word::TurnRight => Act::Turn(1),
        _ => Act::StepIn,
    }
}

/// The text of a chip: the word in capitals, an arrow beside a turn, and
/// the key of the step back, which is the one chip of use mode.
fn chip_text(word: Word, language: Language, step_back: &str) -> String {
    let text = word.text(language).to_uppercase();
    match word {
        Word::StepBack => format!("{text}  {}", step_back.replace("Escape", "Esc")),
        Word::TurnLeft => format!("\u{2190} {text}"),
        Word::TurnRight => format!("{text} \u{2192}"),
        _ => text,
    }
}

/// The language of the words, from the locale of the session.
fn language() -> Language {
    let tag = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()));
    tag.map_or(Language::En, |tag| Language::from_tag(&tag))
}

/// The name of a mode in an answer.
fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Use => "use",
        Mode::Navigate => "navigate",
    }
}

/// The bindings that the construct holds for a cube: the turn keys, and
/// in use mode `step_back`, unless the window switcher of the faces
/// shows and needs it.
fn cube_bindings(mode: Mode, switching: bool, step_back: Option<&String>) -> Vec<(String, Grab)> {
    let mut wanted: Vec<(String, Grab)> = (TURN_KEYS.iter())
        .map(|(binding, by)| ((*binding).to_owned(), Grab::Turn(*by)))
        .collect();
    if let Some(binding) = step_back.filter(|_| mode == Mode::Use && !switching) {
        wanted.push((binding.clone(), Grab::StepBack));
    }
    wanted
}

fn no_cube() -> ClientError {
    ClientError::Rpc(RpcError::new(
        RpcError::REJECTED,
        "no cube is open; cube.open first",
    ))
}

impl Construct {
    /// The texts of the chips, in the order of `CHIPS`.
    fn chip_texts(&self) -> Vec<(Word, String)> {
        let (language, key) = (language(), &self.settings.value.fly_out);
        (CHIPS.into_iter())
            .map(|word| (word, chip_text(word, language, key)))
            .collect()
    }

    /// Creates the chips of a new cube, hidden, and shows those of its
    /// mode.
    pub(crate) fn open_modes(&mut self) -> Result<(), ClientError> {
        let mut commands = vec![create(MODES, NodeKind::Group, NAME, PropsUpdate::default())];
        for (word, text) in self.chip_texts() {
            commands.extend(self.styled().chip(&chip_id(word), MODES, &text));
        }
        self.rt.conn.try_apply(commands)?;
        self.regrab()?;
        self.draw_modes()
    }

    /// Removes the chips of a cube that went.
    pub(crate) fn close_modes(&mut self) -> Result<(), ClientError> {
        self.rt.conn.try_apply(vec![Command::Remove {
            id: MODES.into(),
            to: None,
        }])?;
        self.regrab()
    }

    /// The top of the taskbar of the front face on the screen: the chips
    /// stay above it. Before the window manager drew it, a guess.
    fn bar_top(&mut self) -> Result<f64, ClientError> {
        let Some(output) = self.cube.as_ref().map(cube::Host::front_output) else {
            return Ok(self.lens.h);
        };
        let selector = format!("id:{}-panel-{output}", cube::MANAGER);
        let bars: Vec<Value> = (self.rt.conn).call("scene.query", Query::new(&selector))?;
        let top = bars
            .first()
            .and_then(|bar| bar["rects"][&output][1].as_f64());
        Ok(top.unwrap_or(self.lens.h - BAR))
    }

    /// Places the chips of the mode at the bottom right, above the
    /// taskbar of the face, where they cover no title band and no button
    /// of a window, and hides the others. Also after the output changed.
    pub(crate) fn draw_modes(&mut self) -> Result<(), ClientError> {
        let Some(shown) = self.cube.as_ref().map(|c| c.view().controls()) else {
            return Ok(());
        };
        let texts = self.chip_texts();
        let (width, bottom) = (self.lens.w, self.bar_top()?);
        let commands = self.fitted(|s| {
            let look = s.styled();
            let mut right = width;
            let mut commands = Vec::new();
            for (word, text) in texts.iter().rev() {
                let show = shown.contains(word);
                let (chip, used) = look.place_chip(&chip_id(*word), text, [right, bottom], show);
                commands.extend(chip);
                right -= if show { used } else { 0.0 };
            }
            commands
        })?;
        self.rt.conn.try_apply(commands).map(drop)
    }

    /// The bindings of the cube: the turn keys in both modes, and the
    /// fly-out binding in use mode, which steps back from the face. The
    /// step back gives its binding up while the window switcher of the
    /// faces shows, which cancels with it. A binding that another client
    /// holds is logged, and the chips still work.
    pub(crate) fn grab_cube_keys(&mut self) {
        let Some(host) = &self.cube else {
            return;
        };
        let step_back = self.flown.is_none().then_some(&self.settings.value.fly_out);
        let wanted = cube_bindings(host.view().mode(), host.switching, step_back);
        for (binding, action) in wanted {
            let grabbed = (binding.parse().map_err(|e| format!("{e}")))
                .and_then(|parsed| self.rt.grab(parsed, action).map_err(|e| e.to_string()));
            if let Err(err) = grabbed {
                eprintln!("chora-construct: the cube key {binding}: {err}");
            }
        }
    }

    /// Does an act on the view of the cube, by a chip, a key, the voice
    /// or a method. An act that the mode has no use for does nothing. A
    /// step back from inside a pane flies out of the pane first.
    pub(crate) fn mode_act(&mut self, act: Act, by: By) -> Result<(), ClientError> {
        if let Act::Turn(turns) = act {
            return self.turn_cube(turns, None, by == By::Person);
        }
        if self.cube.is_none() {
            return Err(no_cube());
        }
        if act == Act::StepBack && self.flown.is_some() {
            self.fly_out()?;
        }
        let changed = (self.cube.as_mut()).is_some_and(|host| host.act(act, Instant::now()));
        if !changed {
            return Ok(());
        }
        if act == Act::StepIn {
            self.give_the_keyboard_back();
            let home = self.cube.as_ref().map(cube::Host::home);
            if let Some(home) = home.filter(|home| *home != self.pose) {
                self.glide_to(home, FLY_MS, None);
            }
        } else if by == By::Person {
            self.take_the_keyboard()?;
        }
        self.regrab()?;
        self.draw_modes()?;
        self.frame()
    }

    /// The methods `view.step_back` and `view.step_in`.
    pub(crate) fn mode_method(&mut self, act: Act) -> Result<Value, RpcError> {
        let failed = |err| match err {
            ClientError::Rpc(err) => err,
            err => RpcError::new(RpcError::REJECTED, err.to_string()),
        };
        self.mode_act(act, By::Method).map_err(failed)?;
        Ok(json!({"mode": self.cube.as_ref().map(|c| mode_name(c.view().mode()))}))
    }

    /// The construct takes the keyboard for the keys of navigate mode,
    /// because the person stepped back, and notes the window that had it.
    fn take_the_keyboard(&mut self) -> Result<(), ClientError> {
        let window = self.focused_surface()?;
        if let Some(host) = &mut self.cube {
            host.note_focus(window);
        }
        self.rt.conn.apply(vec![Command::FocusNode {
            node: Some(LOOK.into()),
        }])
    }

    /// The window that had the keyboard before the step back gets it
    /// again, through its window manager, when the face of then is at
    /// the front again. After a turn to another face it stays without.
    fn give_the_keyboard_back(&mut self) {
        let Some(surface) = self.cube.as_mut().and_then(cube::Host::take_focus) else {
            return;
        };
        let call = json!({"client": cube::MANAGER, "method": "activate",
            "params": {"surface": surface}});
        let activated: Result<Value, _> = self.rt.conn.call("methods.call", call);
        if let Err(err) = activated {
            eprintln!("chora-construct: the focus after the step in: {err}");
        }
    }

    /// The window switcher of the faces shows or went: the binding of
    /// the step back goes to it, or comes back.
    fn switcher(&mut self, shown: bool) -> Result<(), ClientError> {
        match &mut self.cube {
            Some(host) if host.switching != shown => host.switching = shown,
            _ => return Ok(()),
        }
        self.regrab()
    }

    /// The act of a key that the look node got. In navigate mode the
    /// keys of the view act and the others go on to the walk. In use
    /// mode the keys of the walk do nothing, so that the face stays at
    /// 1:1; the keys that open something go on.
    fn key_act(&self, view: View, key: &str) -> Option<Pressed> {
        if let Some(act) = view.key(key) {
            return Some(Pressed::Act(act));
        }
        let opens = matches!(
            self.settings.value.keys.get(key),
            Some(Action::Terminal | Action::Launcher | Action::Windows | Action::Help)
        );
        (view.mode() == Mode::Use && !opens).then_some(Pressed::Nothing)
    }

    /// The act of a press of the left button on a chip, or on a face in
    /// navigate mode while the cube rests. An injected press does
    /// nothing, as for the other controls.
    fn press_act(&mut self, view: View, node: &str, injected: bool) -> Option<Pressed> {
        let chip = CHIPS.into_iter().find(|w| chip_id(*w) == node);
        let face = cube::face_index(node).filter(|_| view.mode() == Mode::Navigate);
        if chip.is_none() && face.is_none() {
            return None;
        }
        if injected && !self.rt.accepts_injected() {
            return Some(Pressed::Nothing);
        }
        Some(match (chip, face) {
            (Some(word), _) => Pressed::Act(chip_act(word)),
            (None, Some(face)) if !view.turning() => Pressed::Face(face),
            _ => Pressed::Nothing,
        })
    }

    /// The input of the modes, before the walk gets it: a grabbed key of
    /// the cube, a plain key, a click on a chip, and a click on a face
    /// in navigate mode. Answers whether the event was for the modes.
    pub(crate) fn modes_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        let Some(view) = self.cube.as_ref().map(cube::Host::view) else {
            return Ok(false);
        };
        let pressed = match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == cube::MANAGER && event == SWITCHER => {
                let shown = data["shown"].as_bool().unwrap_or(false);
                return self.switcher(shown).map(|()| true);
            }
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                ..
            } => match self.rt.action(binding) {
                Some(&Grab::StepBack) => Some(Pressed::Act(Act::StepBack)),
                Some(&Grab::Turn(by)) => Some(Pressed::Act(Act::Turn(by))),
                _ => None,
            },
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => self.key_act(view, key),
            Event::PointerPressed {
                node,
                button,
                injected,
                ..
            } if *button == BTN_LEFT => self.press_act(view, node, *injected),
            _ => None,
        };
        match pressed {
            Some(Pressed::Act(act)) => self.mode_act(act, By::Person)?,
            Some(Pressed::Face(face)) => self.step_into(face)?,
            Some(Pressed::Nothing) => {}
            None => return Ok(false),
        }
        Ok(true)
    }

    /// Steps into a face: the cube turns to it first when another one is
    /// at the front.
    fn step_into(&mut self, face: usize) -> Result<(), ClientError> {
        let by = self.cube.as_ref().map_or(0, |c| c.turns_to(face));
        if by != 0 {
            self.turn_cube(by, None, false)?;
        }
        self.mode_act(Act::StepIn, By::Person)
    }
}

/// What a key or a press of the left button does to the view.
enum Pressed {
    Act(Act),
    Face(usize),
    Nothing,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step_back_bound(mode: Mode, switching: bool) -> bool {
        let key = "Alt+Escape".to_owned();
        let wanted = cube_bindings(mode, switching, Some(&key));
        let turns = wanted.iter().filter(|(_, g)| matches!(g, Grab::Turn(_)));
        assert_eq!(turns.count(), 2, "the turn keys are bound in every state");
        (wanted.iter()).any(|(b, g)| b == "Alt+Escape" && matches!(g, Grab::StepBack))
    }

    #[test]
    fn the_step_back_binding_goes_to_the_window_switcher_while_it_shows() {
        assert!(step_back_bound(Mode::Use, false));
        assert!(
            !step_back_bound(Mode::Use, true),
            "the switcher cancels with it"
        );
        assert!(
            !step_back_bound(Mode::Navigate, false),
            "stood back already"
        );
    }

    #[test]
    fn a_chip_names_its_key_or_its_way() {
        let text = |word| chip_text(word, Language::En, "Alt+Escape");
        assert_eq!(text(Word::StepBack), "STEP BACK  Alt+Esc");
        assert_eq!(text(Word::TurnLeft), "\u{2190} TURN LEFT");
        assert_eq!(text(Word::TurnRight), "TURN RIGHT \u{2192}");
        assert_eq!(text(Word::StepIn), "STEP IN");
    }
}
