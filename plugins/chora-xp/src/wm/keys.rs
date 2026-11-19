//! Keys: bindings, the Alt+Tab switcher and the system key.

use std::convert::Infallible;

use chora_kit::actions::perform::{Failure, Host, perform};
use chora_kit::actions::{Step, Turn};

use super::{
    Binding, Command, ESCAPE_TRIES, ESCAPE_WAIT, Event, Grab, HELP_TASK, KeyAction, Look, MENU_KEY,
    MENUS, Mode, NodeKind, PressState, PropsUpdate, SWITCHER, Switcher, Transform, Value, Window,
    Wm, add_help, close, control, give, help, switcher, taskbar,
};

/// A turn of the cube in quarter turns: left is back.
const fn quarter(turn: Turn) -> i32 {
    match turn {
        Turn::Left => -1,
        Turn::Right => 1,
    }
}

/// The window manager while it does an action of every look: the hooks
/// leave the scene commands of the action in `commands`.
struct Acting<'a> {
    wm: &'a mut Wm,
    commands: Vec<Command>,
}

impl give::Windows for Acting<'_> {
    fn conn(&mut self) -> &mut chora_kit::conn::Conn {
        give::Windows::conn(self.wm)
    }

    fn has(&self, surface: &str) -> bool {
        give::Windows::has(self.wm, surface)
    }

    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, chora_scene::ipc::RpcError> {
        give::Windows::hand(self.wm, surface, to)
    }
}

/// What xp does for the actions of every look: the key list, the Turn Off
/// dialog, the launcher, the chat, the volume, a close and a turn of the
/// cube. The scripts (lock, screenshot) are the shared ones.
impl Host for Acting<'_> {
    type Error = Infallible;

    fn volume(&mut self, change: chora_kit::volume::Change) -> Result<(), Infallible> {
        self.commands.extend(self.wm.change_volume(change, true));
        Ok(())
    }

    fn keys(&mut self) -> Result<(), Infallible> {
        self.commands.extend(self.wm.open_help(help::KEYBOARD));
        Ok(())
    }

    fn chat(&mut self) -> Result<(), Infallible> {
        self.wm.start_program(control::CHAT);
        Ok(())
    }

    fn failed(&mut self, failure: Failure) -> Result<(), Infallible> {
        match failure {
            Failure::NoScript(script) => self.wm.cannot_do(&script),
            Failure::NotStarted(name) => eprintln!("chora-xp: {name} did not start"),
            Failure::NotGiven { to, why } => eprintln!("chora-xp: give to {to}: {why}"),
        }
        Ok(())
    }

    /// The power key opens the Turn Off dialog of XP, not the shared menu.
    fn power(&mut self) -> Result<bool, Infallible> {
        self.commands
            .extend(self.wm.open_dialog(Mode::TurnOff, None));
        Ok(true)
    }

    fn launcher(&mut self) -> Result<(), Infallible> {
        let launcher = self.wm.settings.value.launcher.clone();
        self.wm.start_program(&launcher);
        Ok(())
    }

    fn close(&mut self) -> Result<(), Infallible> {
        let closing = self.wm.focused.as_deref().map(close).unwrap_or_default();
        self.commands.extend(closing);
        Ok(())
    }

    fn cube(&mut self, turn: Turn) -> Result<(), Infallible> {
        self.commands.extend(self.wm.turn(quarter(turn)));
        Ok(())
    }
}

impl Wm {
    /// The events of the grabs of keys and buttons.
    pub(super) fn input(&mut self, event: Event) -> Vec<Command> {
        match event {
            Event::NodeKey {
                key,
                state: PressState::Pressed,
                modifiers,
                node: _,
                code: _,
                text: _,
                injected: _, // injected: hears drops an injected key unless xp accepts them
            } => {
                let shift = modifiers.iter().any(|m| m == "Shift");
                let menu = key == "Menu" || (key == "F10" && shift);
                self.node_key(if menu { MENU_KEY } else { &key })
            }
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                .. // injected: the runtime drops an injected binding key unless it accepts them
            } => self.key(&binding),
            Event::InputKey {
                binding,
                state: PressState::Released,
                .. // injected: the runtime drops an injected binding key unless it accepts them
            } => self.key_released(&binding),
            Event::InputButton {
                binding, state, position, .. // injected: hears drops an injected event unless xp accepts them
            } => self.button(&binding, state, position),
            Event::InputMoved { position, .. } => self.drag_cube(position), // injected: hears drops an injected event unless xp accepts them
            _ => Vec::new(),
        }
    }

    /// A grabbed pointer button: the pointer turns the cube for `cube`,
    /// and any other action runs at the press.
    fn button(&mut self, binding: &str, state: PressState, position: [f64; 2]) -> Vec<Command> {
        match (self.rt.action(binding), state) {
            (Some(KeyAction::Look(Look::Cube)), PressState::Pressed) => self.grab_cube(position),
            (Some(KeyAction::Look(Look::Cube)), PressState::Released) => {
                self.release_cube();
                Vec::new()
            }
            (Some(_), PressState::Pressed) => self.key(binding),
            (_, PressState::Released) | (None, _) => Vec::new(),
        }
    }

    pub(super) fn key(&mut self, binding: &str) -> Vec<Command> {
        if self.system_key.as_deref() == Some(binding) {
            return self.system_key_pressed();
        }
        if self.switch_escape.as_deref() == Some(binding) {
            return self.cancel_switch();
        }
        let action = self.rt.action(binding).cloned();
        let tap = matches!(action, Some(KeyAction::Look(Look::Tap)));
        match (&mut self.tap, tap) {
            (tap, true) => *tap = Some(false),
            (Some(used), _) => *used = true,
            _ => {}
        }
        match action {
            Some(KeyAction::System(system)) => {
                let mut acting = Acting {
                    wm: self,
                    commands: Vec::new(),
                };
                let Ok(()) = perform(&mut acting, &system);
                acting.commands
            }
            Some(KeyAction::Look(look)) => self.look(look),
            None => Vec::new(),
        }
    }

    /// Does an action of xp: the Start menu, the switcher, the desktop and
    /// the windows that go with a turn.
    fn look(&mut self, look: Look) -> Vec<Command> {
        match look {
            Look::Files => self.open_files(),
            Look::Desktop => self.toggle_desktop(),
            Look::Carry(turn) => self.carry(quarter(turn)),
            Look::Switch(step) => self.switch(match step {
                Step::Next => 1,
                Step::Previous => -1,
            }),
            Look::Start => self.toggle_start(),
            Look::Tap | Look::Cube => Vec::new(),
        }
    }

    /// Ctrl+Escape and Super alone: opens or closes the start menu, but
    /// not under the shade of Log Off or Turn Off.
    pub(super) fn toggle_start(&mut self) -> Vec<Command> {
        if self.dialog.is_some() {
            return Vec::new();
        }
        let mut commands = self.close_context();
        if let Some(output) = self.bars.first().map(|b| b.output.clone()) {
            commands.extend(self.press_taskbar(taskbar::Part::Start(output)));
        }
        commands
    }

    /// Alt+Tab: opens XP's switcher, or moves its choice.
    pub(super) fn switch(&mut self, step: i32) -> Vec<Command> {
        if self.dialog.is_some() {
            return Vec::new();
        }
        let open = self.switcher.is_some();
        let mut commands = Vec::new();
        if let Some(switcher) = &mut self.switcher {
            switcher.step(step);
        } else {
            commands.extend(self.close_menu());
            commands.extend(self.close_context());
            let mut windows: Vec<&Window> = self
                .windows
                .iter()
                .rev()
                .filter(|w| w.shown && w.desktop == self.desktop)
                .collect();
            windows.sort_by_key(|w| w.minimized);
            let mut entries = windows
                .into_iter()
                .map(|w| switcher::Entry {
                    surface: w.surface.clone(),
                    title: w.title.clone(),
                    icon: self
                        .icons
                        .program_sized(w.app_id.as_deref().unwrap_or_default(), 32),
                })
                .collect::<Vec<_>>();
            if self.has_help() {
                let help = switcher::Entry {
                    surface: HELP_TASK.into(),
                    title: help::TITLE.into(),
                    icon: self.icons.program_sized("help-browser", 32),
                };
                add_help(&mut entries, help, self.help_active());
            }
            let Some(switcher) = Switcher::new(entries, step) else {
                return commands;
            };
            self.switcher = Some(switcher);
            self.switcher_opened();
            self.tell_switcher(true);
            self.grab_switch_escape();
        }
        commands.extend(self.show_switcher(open));
        commands
    }

    /// The box of the switcher, new or again.
    pub(super) fn show_switcher(&mut self, again: bool) -> Vec<Command> {
        let Some(switcher) = &self.switcher else {
            return Vec::new();
        };
        let picture = switcher.picture(&self.theme);
        let [w, h] = picture.size;
        let [x, y, aw, ah] = self.work_area();
        let image = self.theme.render(&picture);
        let props = PropsUpdate {
            image: Some(image),
            transform: Some(Transform::translate(
                f64::from(x + (aw - w) / 2),
                f64::from(y + (ah - h) / 2),
                0.0,
            )),
            ..PropsUpdate::default()
        };
        vec![if again {
            Command::Update {
                id: SWITCHER.into(),
                props,
            }
        } else {
            Command::Create {
                id: SWITCHER.into(),
                kind: NodeKind::Image,
                parent: MENUS.into(),
                props,
            }
        }]
    }

    /// A window that closes leaves the switcher; the last one closes it.
    pub(super) fn drop_from_switcher(&mut self, surface: &str) -> Vec<Command> {
        match self.switcher.as_mut().map(|s| s.remove(surface)) {
            Some(true) => self.show_switcher(true),
            Some(false) => self.cancel_switch(),
            None => Vec::new(),
        }
    }

    /// Alt+Space: the system menu of the focused window, after the open
    /// menus close; nothing under the shade of Log Off or Turn Off.
    pub(super) fn system_key_pressed(&mut self) -> Vec<Command> {
        if self.dialog.is_some() {
            return Vec::new();
        }
        let mut commands = self.close_context();
        commands.extend(self.close_menu());
        if let Some(surface) = self.focused.clone() {
            commands.extend(self.system_below_icon(&surface));
        }
        commands
    }

    /// The core moved the keyboard focus to a surface: when it is no window
    /// of `chora-xp`, as one of another window manager, the window of
    /// `chora-xp` looks inactive and Alt+Space goes back to the clients.
    /// A window of `chora-xp` that gets the focus while none of them has
    /// it looks active again and takes Alt+Space. Without a focused
    /// surface, as when a node of another client takes the keyboard, no
    /// window of `chora-xp` has it either.
    pub(super) fn focus_moved(&mut self, surface: Option<&str>) -> Vec<Command> {
        self.core_focus = surface.map(Into::into);
        if let Some(surface) = surface.filter(|s| self.index(s).is_some()) {
            if self.focused.is_some() {
                return Vec::new();
            }
            self.focused = Some(surface.into());
            self.grab_system_key(true);
            let mut commands = self.restyle(surface);
            commands.extend(self.sync_tasks());
            return commands;
        }
        let Some(previous) = self.focused.take() else {
            return Vec::new();
        };
        self.grab_system_key(false);
        let mut commands = self.restyle(&previous);
        commands.extend(self.sync_tasks());
        commands
    }

    /// Takes Alt+Space while a window of `chora-xp` has the focus, and
    /// gives it back to the clients without one.
    pub(super) fn grab_system_key(&mut self, on: bool) {
        if on == self.system_key.is_some() {
            return;
        }
        let parsed: Result<Binding, _> = match self.system_key.take() {
            Some(held) => held.parse(),
            None => "Alt+space".parse(),
        };
        let binding = match parsed {
            Ok(binding) => binding,
            Err(err) => {
                eprintln!("chora-xp: {err}");
                return;
            }
        };
        if on {
            match self
                .rt
                .conn
                .call::<_, String>("input.grab", Grab { binding })
            {
                Ok(normal) => self.system_key = Some(normal),
                Err(err) => eprintln!("chora-xp: {err}"),
            }
        } else if let Err(err) = self
            .rt
            .conn
            .call::<_, Value>("input.ungrab", Grab { binding })
        {
            eprintln!("chora-xp: {err}");
        }
    }

    /// Alt+Escape: the switcher closes without a switch.
    pub(super) fn cancel_switch(&mut self) -> Vec<Command> {
        if self.end_switch().is_none() {
            return Vec::new();
        }
        vec![Command::Remove {
            id: SWITCHER.into(),
            to: None,
        }]
    }

    /// The release of the modifier of a key: Alt up after Alt+Tab
    /// activates the chosen window.
    pub(super) fn key_released(&mut self, binding: &str) -> Vec<Command> {
        if matches!(self.rt.action(binding), Some(KeyAction::Look(Look::Tap))) {
            return match self.tap.take() {
                Some(false) => self.toggle_start(),
                _ => Vec::new(),
            };
        }
        let Some(switcher) = self.end_switch() else {
            return Vec::new();
        };
        let mut commands = vec![Command::Remove {
            id: SWITCHER.into(),
            to: None,
        }];
        commands.extend(self.activate(switcher.chosen()));
        commands
    }

    /// Takes Alt+Escape from the clients only while the switcher shows,
    /// so that it cancels the switch.
    pub(super) fn grab_switch_escape(&mut self) {
        let binding = match "Alt+Escape".parse() {
            Ok(binding) => binding,
            Err(err) => {
                eprintln!("chora-xp: {err}");
                return;
            }
        };
        let mut grabbed = Err(String::new());
        for attempt in 0..ESCAPE_TRIES {
            if attempt > 0 {
                std::thread::sleep(ESCAPE_WAIT);
            }
            let grab = Grab {
                binding: Binding::clone(&binding),
            };
            grabbed =
                (self.rt.conn.call::<_, String>("input.grab", grab)).map_err(|e| e.to_string());
            if grabbed.is_ok() {
                break;
            }
        }
        match grabbed {
            Ok(normal) => self.switch_escape = Some(normal),
            Err(err) => eprintln!("chora-xp: {err}"),
        }
    }

    /// Tells the other clients that the switcher shows or went, with the
    /// event `xp.switcher`: one that holds Alt+Escape gives it up for
    /// that time.
    pub(super) fn tell_switcher(&mut self, shown: bool) {
        let event = serde_json::json!({"event": "xp.switcher", "data": {"shown": shown}});
        if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", event) {
            eprintln!("chora-xp: {err}");
        }
    }

    /// Closes the switcher and gives Alt+Escape back.
    pub(super) fn end_switch(&mut self) -> Option<Switcher> {
        if let Some(normal) = self.switch_escape.take() {
            let ungrab = normal
                .parse::<Binding>()
                .map_err(|e| e.to_string())
                .and_then(|binding| {
                    (self.rt.conn)
                        .call::<_, Value>("input.ungrab", Grab { binding })
                        .map_err(|e| e.to_string())
                });
            if let Err(err) = ungrab {
                eprintln!("chora-xp: {err}");
            }
        }
        let switcher = self.switcher.take();
        self.open.shut(super::surfaces::SWITCHER);
        if switcher.is_some() {
            self.tell_switcher(false);
        }
        switcher
    }
}
