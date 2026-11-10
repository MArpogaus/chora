//! The two modes of a view on a shape of panes, such as a cube of faces
//! (K4): in use mode the person works in the front pane, which takes the
//! pointer and the keys; in navigate mode the person stands back, sees
//! the shape, turns it and steps into a pane again. `docs/cube-modes.md`
//! gives the flows and the words. The model is the same in every plugin
//! that shows such a shape, so the same act does the same thing
//! everywhere.

use crate::words::Word;

/// Where the input of the person goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// The front pane fills the view and takes the input.
    #[default]
    Use,
    /// The view stands back; the input moves the view and the shape.
    Navigate,
}

/// What a person does to the view, by a control, a key, the voice or a
/// method.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// Stands back from the front pane.
    StepBack,
    /// Steps into the front pane.
    StepIn,
    /// Turns the shape by a number of panes; negative is to the left.
    Turn(i32),
}

/// The mode of a view and whether its shape turns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct View {
    mode: Mode,
    turning: bool,
}

impl View {
    /// The mode.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Whether the shape turns.
    #[must_use]
    pub fn turning(&self) -> bool {
        self.turning
    }

    /// Does an act. Returns whether the view changed: an act that the
    /// mode has no use for, and a turn during a turn, change nothing.
    pub fn act(&mut self, act: Act) -> bool {
        match (act, self.mode) {
            (Act::StepBack, Mode::Use) => {
                self.mode = Mode::Navigate;
                true
            }
            (Act::StepIn, Mode::Navigate) => {
                self.mode = Mode::Use;
                true
            }
            (Act::Turn(by), _) if by != 0 && !self.turning => {
                self.turning = true;
                true
            }
            _ => false,
        }
    }

    /// The turn ended; the mode is the one of before the turn.
    pub fn turn_ended(&mut self) {
        self.turning = false;
    }

    /// Whether the front pane takes the input: in use mode, and not while
    /// the shape turns.
    #[must_use]
    pub fn forwards(&self) -> bool {
        self.mode == Mode::Use && !self.turning
    }

    /// The controls that show, in their order.
    #[must_use]
    pub fn controls(&self) -> &'static [Word] {
        match self.mode {
            Mode::Use => &[Word::StepBack],
            Mode::Navigate => &[Word::TurnLeft, Word::TurnRight, Word::StepIn],
        }
    }

    /// The act of a key that the plugin has without a modifier: only in
    /// navigate mode, where the plugin has the keyboard. `Escape` backs
    /// out of the step back and `Return` steps in.
    #[must_use]
    pub fn key(&self, key: &str) -> Option<Act> {
        let stood_back = self.mode == Mode::Navigate;
        (stood_back && matches!(key, "Escape" | "Return")).then_some(Act::StepIn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_view_starts_in_use_mode_and_steps_back_and_in() {
        let mut view = View::default();
        assert_eq!((view.mode(), view.forwards()), (Mode::Use, true));
        assert!(!view.act(Act::StepIn), "it is in the pane already");
        assert!(view.act(Act::StepBack));
        assert_eq!((view.mode(), view.forwards()), (Mode::Navigate, false));
        assert!(!view.act(Act::StepBack), "it stands back already");
        assert!(view.act(Act::StepIn));
        assert_eq!((view.mode(), view.forwards()), (Mode::Use, true));
    }

    #[test]
    fn a_turn_keeps_the_mode_and_forwards_nothing_while_it_runs() {
        for mode in [Mode::Use, Mode::Navigate] {
            let mut view = View::default();
            if mode == Mode::Navigate {
                view.act(Act::StepBack);
            }
            assert!(view.act(Act::Turn(1)));
            assert!(view.turning() && !view.forwards(), "{mode:?}");
            assert!(!view.act(Act::Turn(-1)), "one turn at a time");
            assert!(!view.act(Act::Turn(0)));
            view.turn_ended();
            assert_eq!(view.mode(), mode);
            assert_eq!(view.forwards(), mode == Mode::Use);
        }
    }

    #[test]
    fn each_mode_shows_its_controls() {
        let mut view = View::default();
        assert_eq!(view.controls(), [Word::StepBack]);
        view.act(Act::StepBack);
        let stood_back = [Word::TurnLeft, Word::TurnRight, Word::StepIn];
        assert_eq!(view.controls(), stood_back);
    }

    /// In use mode the pane has the keyboard: a plain key is not for the
    /// view, so a program in the pane keeps its `Escape`.
    #[test]
    fn plain_keys_act_only_in_navigate_mode() {
        let mut view = View::default();
        assert_eq!(view.key("Escape"), None);
        assert_eq!(view.key("Return"), None);
        view.act(Act::StepBack);
        assert_eq!(view.key("Escape"), Some(Act::StepIn));
        assert_eq!(view.key("Return"), Some(Act::StepIn));
        assert_eq!(view.key("a"), None);
    }
}
