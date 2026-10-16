//! XP's Log Off Windows and Turn off computer dialogs: a dark blue title
//! band with the flag, a light blue band with the big buttons, and Cancel
//! below. They sit in the middle of the overlay root of an output, above a
//! clear shield that takes the clicks outside. The measurements follow the
//! XP screenshots and `SessionDialogView.qml` of the reference.

use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform};

use crate::{
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The dialog with its one pixel black border.
pub const SIZE: [i32; 2] = [314, 200];
/// The big buttons: the icon and the label below it.
const BUTTON: [i32; 2] = [76, 54];
const ICON: i32 = 33;
const ICON_Y: i32 = 81;
const CANCEL: Rect = [244, 169, 58, 18];
const GROUP: &str = "xp-session";
const SHIELD: &str = "xp-session-shield";
const DIALOG: &str = "xp-session-dialog";
const FRAME: &str = "xp-session-frame";

/// The desktop behind the dialog loses its color, as on XP: `amount`
/// goes from 0 to 1 while the dialog opens.
pub const FADE_SHADER: &str = "precision mediump float;
uniform sampler2D tex;
uniform float alpha;
uniform float amount;
varying vec2 v_uv;

void main() {
    vec4 color = texture2D(tex, v_uv);
    float gray = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    vec3 rgb = mix(color.rgb, vec3(gray), amount) * (1.0 - 0.15 * amount);
    gl_FragColor = vec4(rgb, color.a) * alpha;
}
";

/// Which dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Log Off Windows: Switch User and Log Off.
    LogOff,
    /// Turn off computer: Stand By, Turn Off and Restart.
    TurnOff,
}

/// What a button of a dialog does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Always disabled: chora has no second session to switch to.
    SwitchUser,
    /// Asks every window to close.
    LogOff,
    /// Suspends the machine with `systemctl suspend`.
    StandBy,
    /// Ends chora-xp.
    TurnOff,
    /// Starts chora-xp again.
    Restart,
    /// Closes the dialog.
    Cancel,
}

/// A button of a dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Button {
    /// What it does.
    pub choice: Choice,
    /// The label.
    pub title: &'static str,
    /// The icon file of the theme, none for Cancel.
    pub icon: Option<&'static str>,
    /// A disabled button looks pale and does nothing.
    pub enabled: bool,
    /// The rectangle in the dialog.
    pub rect: Rect,
}

/// The buttons of a dialog, Cancel last. Stand By is enabled only when the
/// machine can suspend.
pub fn buttons(mode: Mode, suspend: bool) -> Vec<Button> {
    // The left edges of the icons in XP's screenshots.
    let list: &[(Choice, &str, &str, bool, i32)] = match mode {
        Mode::LogOff => &[
            (
                Choice::SwitchUser,
                "Switch User",
                "winxp/switch-user.svg",
                false,
                84,
            ),
            (Choice::LogOff, "Log Off", "winxp/log-off.svg", true, 199),
        ],
        Mode::TurnOff => &[
            (
                Choice::StandBy,
                "Stand By",
                "winxp/stand-by.svg",
                suspend,
                55,
            ),
            (Choice::TurnOff, "Turn Off", "winxp/turn-off.svg", true, 141),
            (Choice::Restart, "Restart", "winxp/restart.svg", true, 226),
        ],
    };
    let mut out: Vec<Button> = list
        .iter()
        .map(|&(choice, title, icon, enabled, x)| Button {
            choice,
            title,
            icon: Some(icon),
            enabled,
            rect: [x - (BUTTON[0] - ICON) / 2, ICON_Y, BUTTON[0], BUTTON[1]],
        })
        .collect();
    out.push(Button {
        choice: Choice::Cancel,
        title: "Cancel",
        icon: None,
        enabled: true,
        rect: CANCEL,
    });
    out
}

/// The top-left corner of the dialog in the middle of an output.
pub fn center([w, h]: [i32; 2]) -> [i32; 2] {
    [(w - SIZE[0]) / 2, (h - SIZE[1]) / 2]
}

/// An open dialog.
pub struct Dialog {
    /// The output it shows on.
    pub output: String,
    mode: Mode,
    /// The size of the output.
    size: [i32; 2],
    origin: [i32; 2],
    buttons: Vec<Button>,
    hovered: Option<usize>,
    pressed: Option<usize>,
}

impl Dialog {
    /// A dialog in the middle of an output of a size.
    pub fn new(mode: Mode, output: &str, size: [i32; 2], suspend: bool) -> Self {
        Self {
            output: output.into(),
            mode,
            size,
            origin: center(size),
            buttons: buttons(mode, suspend),
            hovered: None,
            pressed: None,
        }
    }

    /// Whether a node belongs to a dialog.
    pub fn owns(node: &str) -> bool {
        node.starts_with(GROUP)
    }

    /// Creates the shield, the dialog and its buttons on the overlay root
    /// of the output.
    pub fn create(&self, theme: &mut Theme) -> Vec<Command> {
        let title = match self.mode {
            Mode::LogOff => "Log Off Windows",
            Mode::TurnOff => "Turn off computer",
        };
        let frame = Picture::new(SIZE[0], SIZE[1])
            .file(&theme.path("winxp/dialog.svg"), [0, 0, SIZE[0], SIZE[1]])
            .text(Text::new(
                title,
                Font::Dialog,
                [255, 255, 255, 255],
                [13, 1, 250, 43],
            ))
            .file(&theme.path("winxp/start-flag.svg"), [269, 5, 38, 38]);
        let size = self.size.map(f64::from);
        let mut commands = vec![
            Command::Create {
                id: GROUP.into(),
                kind: NodeKind::Group,
                parent: format!("screen:{}", self.output),
                props: PropsUpdate::default(),
            },
            // Clear, so that the desktop shows through; it takes the
            // clicks outside the dialog.
            Command::Create {
                id: SHIELD.into(),
                kind: NodeKind::Rect,
                parent: GROUP.into(),
                props: PropsUpdate {
                    size: Some(size),
                    color: Some([0.0, 0.0, 0.0, 0.0]),
                    input: Some(InputMode::Accept),
                    ..PropsUpdate::default()
                },
            },
            Command::Create {
                id: DIALOG.into(),
                kind: NodeKind::Group,
                parent: GROUP.into(),
                props: at(self.origin),
            },
            Command::Create {
                id: FRAME.into(),
                kind: NodeKind::Image,
                parent: DIALOG.into(),
                props: PropsUpdate {
                    image: Some(theme.render(&frame)),
                    ..PropsUpdate::default()
                },
            },
        ];
        for (i, button) in self.buttons.iter().enumerate() {
            commands.push(Command::Create {
                id: button_id(i),
                kind: NodeKind::Image,
                parent: DIALOG.into(),
                props: PropsUpdate {
                    image: Some(self.render(theme, i)),
                    ..at([button.rect[0], button.rect[1]])
                },
            });
        }
        commands
    }

    /// Removes the dialog.
    pub fn remove() -> Command {
        Command::Remove { id: GROUP.into() }
    }

    /// A button lights up under the pointer.
    pub fn hover(&mut self, theme: &mut Theme, node: &str, on: bool) -> Vec<Command> {
        let Some(i) = self.button_of(node) else {
            return Vec::new();
        };
        if on {
            self.hovered = Some(i);
        } else if self.hovered == Some(i) {
            self.hovered = None;
        }
        vec![self.look(theme, i)]
    }

    /// A press: on a button it goes down, on the dialog nothing happens.
    /// `None` for a press outside the dialog, which closes it.
    pub fn press(&mut self, theme: &mut Theme, node: &str) -> Option<Vec<Command>> {
        if let Some(i) = self.button_of(node) {
            self.pressed = Some(i);
            return Some(vec![self.look(theme, i)]);
        }
        // A disabled button is part of the dialog too.
        (node == FRAME || node.starts_with("xp-session-button-")).then(Vec::new)
    }

    /// The choice of the button that went down, if the pointer is still on
    /// it, in output coordinates.
    pub fn release(
        &mut self,
        theme: &mut Theme,
        [px, py]: [f64; 2],
    ) -> (Vec<Command>, Option<Choice>) {
        let Some(index) = self.pressed.take() else {
            return (Vec::new(), None);
        };
        let [x, y, w, h] = self.buttons[index].rect;
        let [x, y] = [x + self.origin[0], y + self.origin[1]];
        let inside = (f64::from(x)..f64::from(x + w)).contains(&px)
            && (f64::from(y)..f64::from(y + h)).contains(&py);
        (
            vec![self.look(theme, index)],
            inside.then_some(self.buttons[index].choice),
        )
    }

    fn button_of(&self, node: &str) -> Option<usize> {
        let i: usize = node.strip_prefix("xp-session-button-")?.parse().ok()?;
        (i < self.buttons.len() && self.buttons[i].enabled).then_some(i)
    }

    fn look(&self, theme: &mut Theme, i: usize) -> Command {
        Command::Update {
            id: button_id(i),
            props: PropsUpdate {
                image: Some(self.render(theme, i)),
                ..PropsUpdate::default()
            },
        }
    }

    /// A button: the icon with its label, or Cancel, normal, under the
    /// pointer, down or disabled.
    fn render(&self, theme: &mut Theme, i: usize) -> String {
        let button = &self.buttons[i];
        let [_, _, w, h] = button.rect;
        let hovered = self.hovered == Some(i);
        let down = self.pressed == Some(i) && hovered;
        let shift = i32::from(down);
        let file = |name: &str| theme.path(name);
        let picture = match button.icon {
            None => {
                let face = match (down, hovered) {
                    (true, _) => "winxp/dialog-button-pressed.svg",
                    (false, true) => "winxp/dialog-button-hover.svg",
                    (false, false) => "winxp/dialog-button.svg",
                };
                Picture::new(w, h).file(&file(face), [0, 0, w, h]).text(
                    Text::new(button.title, Font::Ui, [0, 0, 0, 255], [shift, shift, w, h])
                        .centered(),
                )
            }
            Some(icon) => {
                let rect = [(w - ICON) / 2 + shift, shift, ICON, ICON];
                let mut picture = Picture::new(w, h).file(&file(icon), rect);
                let overlay = if !button.enabled {
                    Some("winxp/dialog-icon-disabled.svg")
                } else if hovered && !down {
                    Some("winxp/dialog-icon-hover.svg")
                } else {
                    None
                };
                if let Some(overlay) = overlay {
                    picture = picture.file(&file(overlay), rect);
                }
                let color = if button.enabled {
                    [255, 255, 255, 255]
                } else {
                    [178, 196, 240, 255]
                };
                picture
                    .text(Text::new(button.title, Font::UiBold, color, [0, 37, w, 16]).centered())
            }
        };
        theme.render(&picture)
    }
}

fn button_id(i: usize) -> String {
    format!("xp-session-button-{i}")
}

fn at([x, y]: [i32; 2]) -> PropsUpdate {
    PropsUpdate {
        transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
        ..PropsUpdate::default()
    }
}

/// Whether a program is on `$PATH`.
pub fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buttons_sit_where_xp_has_them() {
        let turn_off = buttons(Mode::TurnOff, true);
        let lefts: Vec<i32> = turn_off.iter().map(|b| b.rect[0]).collect();
        // The icons of Stand By, Turn Off and Restart start at 55, 141 and
        // 226; Cancel is last.
        assert_eq!(lefts, [55 - 21, 141 - 21, 226 - 21, 244]);
        assert_eq!(turn_off[3].choice, Choice::Cancel);
        let log_off = buttons(Mode::LogOff, true);
        assert_eq!(log_off.len(), 3);
        assert!(!log_off[0].enabled && log_off[1].enabled);
        // Stand By is disabled without a way to suspend.
        assert!(!buttons(Mode::TurnOff, false)[0].enabled);
    }

    #[test]
    fn the_dialog_sits_in_the_middle() {
        assert_eq!(center([1280, 800]), [483, 300]);
    }

    #[test]
    fn disabled_buttons_take_no_press() {
        let dialog = Dialog::new(Mode::LogOff, "o", [800, 600], false);
        assert_eq!(dialog.button_of("xp-session-button-0"), None);
        assert_eq!(dialog.button_of("xp-session-button-1"), Some(1));
        assert_eq!(dialog.button_of("xp-session-button-9"), None);
        assert!(Dialog::owns(SHIELD));
    }
}
