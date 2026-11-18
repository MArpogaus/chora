//! The event loop methods: reading events, handling them, answering calls and the periodic tick.

use super::{
    Bar, Call, ClientError, Command, Duration, Event, Helper, Instant, Language, MENUS, Panel,
    Reading, RpcError, VOLUME_AGAIN, Value, Wm, control, give, is_own, known, taskbar, volume,
    without_reserve,
};

impl Wm {
    /// The next event, or `None` after a timeout.
    pub fn conn_event(&mut self, timeout: Duration) -> Result<Option<Event>, ClientError> {
        self.rt.conn.next_event_timeout(timeout)
    }

    /// Handles one event of the core, with one arm per event.
    #[allow(clippy::too_many_lines)]
    pub(super) fn handle(&mut self, event: Event) -> Result<(), ClientError> {
        self.voiced.hear(&event);
        let commands = match event {
            Event::SurfaceCreated {
                surface,
                app_id,
                title,
                to,
                token,
                ..
            } => {
                if let Some(note) = self.launches.window(token.as_deref()) {
                    self.notifier.close(note);
                }
                return self.adopt(&surface, app_id, title.as_deref(), to.as_deref());
            }
            Event::SurfaceDetached { surface } => {
                let commands = self.forget(&surface);
                self.apply(commands)?;
                return self.adopt(&surface, None, None, None);
            }
            Event::SurfaceDestroyed { surface } => {
                self.core_focus.take_if(|s| *s == surface);
                self.forget(&surface)
            }
            Event::SurfaceChanged {
                surface,
                app_id,
                title,
                size,
                decoration,
            } => {
                let mut commands = self.redecorate(&surface, decoration);
                commands.extend(self.retitle(&surface, app_id, title.as_deref()));
                if let Some(size) = size.filter(|[w, h]| *w > 0 && *h > 0) {
                    commands.extend(self.resized(&surface, size));
                }
                commands
            }
            Event::SurfaceRequested { surface, request } => self.request(&surface, request),
            Event::NodeRequested {
                node, action, args, ..
            } if action == "update" => self.requested_look(&node, &args),
            Event::PointerPressed {
                node,
                button,
                position,
                .. // injected: hears drops an injected press unless xp accepts them
            } => self.pressed(&node, button, position),
            Event::PointerMoved { position, .. } => self.dragged(position), // injected: hears drops an injected move unless xp accepts them
            Event::PointerReleased { node, position, .. } => self.released(&node, position), // injected: hears drops an injected release unless xp accepts them
            Event::OutputChanged { output } => {
                let output = without_reserve(&self.reserved, output);
                [self.rescale()?, self.output_changed(&output)].concat()
            }
            Event::OutputRemoved { name } => [self.rescale()?, self.output_removed(&name)].concat(),
            Event::Emitted {
                event,
                client,
                data,
            } if event == "voice.state" && client == "voice" => self.voice_state(&data, false),
            Event::Emitted {
                event,
                client,
                data,
            } if event == "voice.remove" && client == "voice" => self.voice_state(&data, true),
            Event::ClientDisconnected { name } => {
                self.floats.owner_gone(&name);
                if name == "voice" && self.helper != Helper::default() {
                    self.helper = Helper::default();
                    self.trays()
                } else {
                    Vec::new()
                }
            }
            event @ (Event::InputKey { .. } // injected: hears drops an injected key unless xp accepts them
            | Event::InputButton { .. } // injected: hears drops an injected event unless xp accepts them
            | Event::InputMoved { .. } // injected: hears drops an injected event unless xp accepts them
            | Event::NodeKey { .. }) => self.input(event), // injected: hears drops an injected key unless xp accepts them
            Event::PointerEntered { node, .. } => self.hovered(&node, true), // injected: hears drops an injected event unless xp accepts them
            Event::PointerLeft { node } => self.hovered(&node, false),
            Event::EffectsDropped { client } if is_own(&client) => self.effects_dropped(),
            Event::FocusChanged {
                surface: Some(surface),
            } => self.focus_moved(&surface),
            _ => Vec::new(),
        };
        self.apply(commands)
    }

    /// Runs an exported method, as the matching click does, and answers
    /// with nothing.
    pub(super) fn call(&mut self, call: Call) -> Result<Value, RpcError> {
        let commands = match call {
            Call::StartMenu { open } => {
                let bar = self.bars.first().map(|bar| bar.output.clone());
                if open && self.dialog.is_some() {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        "the Log Off or Turn Off dialog is open",
                    ));
                }
                if open == self.menu.is_open() {
                    Vec::new()
                } else if open {
                    let output =
                        bar.ok_or_else(|| RpcError::new(RpcError::REJECTED, "no output"))?;
                    self.press_taskbar(taskbar::Part::Start(output))
                } else {
                    self.close_menu()
                }
            }
            Call::Minimize { surface } => {
                known(self.index(&surface).is_some(), &surface)?;
                self.minimize(&surface)
            }
            Call::Restore { surface } | Call::Activate { surface } => {
                known(self.index(&surface).is_some(), &surface)?;
                self.activate(&surface)
            }
            Call::Restart => {
                self.floats.land_all();
                self.apply(self.keep_for_restart())
                    .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
                self.done = true;
                self.restart = true;
                return Ok(Value::Null);
            }
            Call::TurnTo { desktop } => self.turn_to(desktop),
            Call::Move {
                surface,
                output,
                at,
            } => self.move_to_output(&surface, &output, at)?,
            Call::Float(float) => return self.float(float),
            Call::Give(give::Give { surface, to, grab }) => {
                known(self.index(&surface).is_some(), &surface)?;
                let (answer, commands) = self.give_dragged(&surface, &to, grab)?;
                self.apply(commands)
                    .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
                return Ok(answer);
            }
        };
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(Value::Null)
    }

    /// When the pointer has rested on the start menu long enough to open
    /// or close a cascade.
    pub fn menu_due(&self) -> Option<Instant> {
        [self.menu.due(), self.tip.due()]
            .into_iter()
            .flatten()
            .min()
    }

    /// Opens or closes the cascade where the pointer rests, and shows the
    /// tooltip of a part of the taskbar that it rests on.
    pub fn menu_rest(&mut self, now: Instant) -> Result<(), ClientError> {
        let mut commands = self.menu.rest(&mut self.theme, &mut self.icons, now);
        let area = self.bars.first().map_or([0, 0, 1280, 800], Bar::area);
        commands.extend(self.tip.show(&mut self.theme, MENUS, area, now));
        if commands.is_empty() {
            return Ok(());
        }
        self.apply(commands)
    }

    /// Updates the clock.
    pub fn tick(&mut self) -> Result<(), ClientError> {
        self.check_launches();
        let mut commands = self.refresh_icons();
        for i in 0..self.panels.len() {
            if matches!(self.panels[i], Panel::Clock(_)) {
                commands.extend(self.redraw_panel(i));
            }
        }
        if !commands.is_empty() {
            self.apply(commands)?;
        }
        if self.volume_read.elapsed() >= VOLUME_AGAIN {
            self.volume_read = Instant::now();
            self.mixer.refresh();
        }
        let now = self.now();
        let reading = self.mixer.take();
        let changed = reading.is_some_and(|r| Some(r) != self.volume);
        if changed {
            self.volume = reading;
            let commands = self.show_volume();
            self.apply(commands)?;
        }
        if now != self.clock {
            self.clock = now;
            let commands = self.trays();
            self.apply(commands)?;
        }
        Ok(())
    }

    /// The words of the speaker, in the language of the session.
    pub(super) fn volume_words(&self) -> String {
        self.volume
            .map(|r| r.words(Language::of_session()))
            .unwrap_or_default()
    }

    /// What the speaker of the taskbars shows: the icon only.
    pub(super) fn speaker(&self) -> taskbar::Speaker {
        match self.volume {
            Some(Reading::Level(level)) if !level.muted => taskbar::Speaker::On(level.percent),
            _ => taskbar::Speaker::Off,
        }
    }

    /// What the icons of the notification area show.
    pub(super) fn icons(&self) -> taskbar::Icons {
        taskbar::Icons {
            speaker: self.speaker(),
            mic_off: self.helper.mic_off,
        }
    }

    /// The tooltip of the helper: its state word, or "Voice off" while the
    /// microphone is off.
    pub(super) fn helper_tip(&self) -> String {
        self.helper.tip(Language::of_session())
    }

    /// The state of the helper that voice announces: the tray icon shows
    /// the microphone and the tooltip the state word.
    pub(super) fn voice_state(&mut self, data: &Value, gone: bool) -> Vec<Command> {
        let mut helper = self.helper.clone();
        if gone {
            helper.forget(data);
        } else {
            helper.follow(data);
        }
        if helper == self.helper {
            return Vec::new();
        }
        self.helper = helper;
        self.trays()
    }

    /// The tooltip of the speaker: the level, or what is wrong.
    pub(super) fn speaker_tip(&self) -> String {
        match self.volume {
            Some(Reading::Level(level)) if !level.muted => format!("Volume {}%", level.percent),
            _ => self.volume_words(),
        }
    }

    /// The notification areas of every taskbar, with the clock and the
    /// volume.
    pub(super) fn trays(&mut self) -> Vec<Command> {
        let icons = self.icons();
        let mut commands = Vec::new();
        for bar in &self.bars {
            commands.extend(bar.tray(&mut self.theme, &self.clock, icons));
        }
        commands
    }

    /// Shows a new reading: the speakers change their icon and keep their
    /// width, so that nothing moves. A tooltip that shows says the new
    /// level.
    pub(super) fn show_volume(&mut self) -> Vec<Command> {
        self.trays()
    }

    /// The speakers show the reading after it at the next tick.
    pub(super) fn change_volume(&mut self, change: volume::Change, by_key: bool) -> Vec<Command> {
        self.mixer.change(change, by_key);
        self.volume_read = Instant::now();
        Vec::new()
    }

    /// The bindings of Keyboard Shortcuts that the plugin holds.
    pub(super) fn held_shortcuts(&self) -> Vec<&'static str> {
        (give::KEYS.iter().map(|(key, _)| *key))
            .chain([control::CHAT_KEY])
            .filter(|key| self.rt.holds(key))
            .collect()
    }

    pub(super) fn now(&self) -> String {
        chrono::Local::now()
            .format(&self.settings.value.clock_format)
            .to_string()
    }
}
