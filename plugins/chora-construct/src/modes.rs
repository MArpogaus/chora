//! The modes of the view on the cube (K4, `docs/cube-modes.md`): in use
//! mode the front face fills the view and takes the input; in navigate
//! mode the cube stands back, the construct has the keyboard, and the
//! person turns the cube or steps into a face. The chips at the top
//! right are the visible way: STEP BACK in use mode, TURN LEFT, TURN
//! RIGHT and STEP IN in navigate mode. `chora_kit::modes` holds the
//! model; this file draws it and takes its input.

use std::time::Instant;

use chora_kit::{
    modes::{Act, Mode},
    words::{Language, Word},
};
use chora_scene::{Event, ipc::PressState};
use serde_json::{Value, json};

use super::{
    BTN_LEFT, ClientError, Command, Construct, FLY_MS, Grab, LOOK, NAME, NodeKind, PropsUpdate,
    RpcError, create, cube,
};

/// The group of the chips, and the prefix of a chip.
const MODES: &str = "construct-modes";
const CHIP: &str = "construct-mode-";
/// The turn keys of a cube, as on the cube of xp on a screen.
const TURN_KEYS: [(&str, i32); 2] = [("Ctrl+Alt+Left", -1), ("Ctrl+Alt+Right", 1)];
/// The chips in the order of the screen, from the left.
const CHIPS: [Word; 4] = [
    Word::StepBack,
    Word::TurnLeft,
    Word::TurnRight,
    Word::StepIn,
];

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

fn no_cube() -> ClientError {
    ClientError::Rpc(RpcError::new(
        RpcError::REJECTED,
        "no cube is open; cube.open first",
    ))
}

impl Construct {
    /// Creates the chips of a new cube, hidden, and shows those of its
    /// mode.
    pub(crate) fn open_modes(&mut self) -> Result<(), ClientError> {
        let mut commands = vec![create(MODES, NodeKind::Group, NAME, PropsUpdate::default())];
        let language = language();
        for word in CHIPS {
            let (id, text) = (chip_id(word), word.text(language).to_uppercase());
            commands.extend(self.styled().chip(&id, MODES, &text));
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

    /// Places the chips of the mode at the top right, from the right
    /// edge, and hides the others.
    fn draw_modes(&mut self) -> Result<(), ClientError> {
        let Some(shown) = self.cube.as_ref().map(|c| c.view().controls()) else {
            return Ok(());
        };
        let language = language();
        let width = self.lens.w;
        let commands = self.fitted(|s| {
            let look = s.styled();
            let mut right = width;
            let mut commands = Vec::new();
            for word in CHIPS.into_iter().rev() {
                let (id, text) = (chip_id(word), word.text(language).to_uppercase());
                let show = shown.contains(&word);
                let (chip, used) = look.place_chip(&id, &text, right, show);
                commands.extend(chip);
                right -= if show { used } else { 0.0 };
            }
            commands
        })?;
        self.rt.conn.try_apply(commands).map(drop)
    }

    /// The bindings of the cube: the turn keys in both modes, and the
    /// fly-out binding in use mode, which steps back from the face.
    pub(crate) fn grab_cube_keys(&mut self) -> Result<(), ClientError> {
        let Some(mode) = self.cube.as_ref().map(|c| c.view().mode()) else {
            return Ok(());
        };
        let invalid = |b: &str, e: String| {
            ClientError::Rpc(RpcError::new(RpcError::INVALID_PARAMS, format!("{b}: {e}")))
        };
        for (binding, by) in TURN_KEYS {
            let parsed = binding
                .parse()
                .map_err(|e| invalid(binding, format!("{e}")))?;
            self.rt.grab(parsed, Grab::Turn(by))?;
        }
        if mode == Mode::Use && self.flown.is_none() {
            let binding = self.settings.value.fly_out.clone();
            let parsed = binding
                .parse()
                .map_err(|e| invalid(&binding, format!("{e}")))?;
            self.rt.grab(parsed, Grab::StepBack)?;
        }
        Ok(())
    }

    /// Does an act on the view of the cube, by a chip, a key, the voice
    /// or a method. An act that the mode has no use for does nothing.
    pub(crate) fn mode_act(&mut self, act: Act) -> Result<(), ClientError> {
        if let Act::Turn(by) = act {
            return self.turn_cube(by, None);
        }
        let host = self.cube.as_mut().ok_or_else(no_cube)?;
        if !host.act(act, Instant::now()) {
            return Ok(());
        }
        if act == Act::StepBack {
            self.take_the_keyboard()?;
        } else {
            self.give_the_keyboard_back();
            let home = self.cube.as_ref().map(cube::Host::home);
            if let Some(home) = home.filter(|home| *home != self.pose) {
                self.glide_to(home, FLY_MS, None);
            }
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
        self.mode_act(act).map_err(failed)?;
        Ok(json!({"mode": self.cube.as_ref().map(|c| mode_name(c.view().mode()))}))
    }

    /// The construct takes the keyboard for the keys of navigate mode,
    /// and notes the window that had it.
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
    /// again, through its window manager, when its face is at the front.
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

    /// The input of the modes, before the walk gets it: a grabbed key of
    /// the cube, a plain key in navigate mode, a click on a chip, and a
    /// click on a face that does not forward. Answers whether the event
    /// was for the modes. In use mode the keys of the walk do nothing, so
    /// that the face stays at 1:1.
    pub(crate) fn modes_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        let Some(view) = self.cube.as_ref().map(cube::Host::view) else {
            return Ok(false);
        };
        let act = match event {
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                ..
            } => match self.rt.action(binding) {
                Some(&Grab::StepBack) => Some(Act::StepBack),
                Some(&Grab::Turn(by)) => Some(Act::Turn(by)),
                _ => return Ok(false),
            },
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => match view.key(key) {
                Some(act) => Some(act),
                None if view.mode() == Mode::Use => None,
                None => return Ok(false),
            },
            Event::PointerPressed { node, button, .. } if *button == BTN_LEFT => {
                let chip = CHIPS.into_iter().find(|w| chip_id(*w) == *node);
                let face = cube::face_index(node).filter(|_| !view.forwards());
                match (chip, face) {
                    (Some(word), _) => Some(chip_act(word)),
                    (None, Some(face)) => return self.step_into(face).map(|()| true),
                    _ => return Ok(false),
                }
            }
            _ => return Ok(false),
        };
        if let Some(act) = act {
            self.mode_act(act)?;
        }
        Ok(true)
    }

    /// Steps into a face: the cube turns to it first when another one is
    /// at the front.
    fn step_into(&mut self, face: usize) -> Result<(), ClientError> {
        let by = self.cube.as_ref().map_or(0, |c| c.turns_to(face));
        if by != 0 {
            self.turn_cube(by, None)?;
        }
        self.mode_act(Act::StepIn)
    }
}
