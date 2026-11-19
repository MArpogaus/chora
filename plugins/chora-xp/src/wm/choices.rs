//! The Control Panel, the Log Off and Turn Off dialogs, the fade and sound.

use super::{
    Bar, Choice, Command, Dialog, FADE, Instant, Mode, NAME, OutputConfigure, OutputInfo, Picture,
    PropsUpdate, Reading, Setting, ShaderEffect, Sound, Uniform, Value, Wallpaper, Wm, apps,
    control, give, json, session, shared, sound, wallpaper, without_reserve,
};

impl Wm {
    /// The outputs of the plugin as the core has them, but with the work
    /// areas that the taskbars reserve: xp lays them out itself.
    pub(super) fn outputs(&mut self) -> Vec<OutputInfo> {
        let outputs = self.rt.outputs().unwrap_or_else(|err| {
            eprintln!("chora-xp: {err}");
            Vec::new()
        });
        (outputs.into_iter())
            .map(|o| without_reserve(&self.reserved, o))
            .collect()
    }

    /// Reserves the height of each taskbar of a screen at the bottom of
    /// the work area of the core, so that the chat, the notifications and
    /// other clients keep clear of it.
    pub(super) fn reserve_bars(&mut self) {
        let wanted: Vec<(String, i32)> = (self.bars.iter())
            .map(|bar| (bar.output.clone(), bar.reserve()))
            .filter(|(output, height)| self.reserved.get(output).copied().unwrap_or(0) != *height)
            .collect();
        for (output, height) in wanted {
            self.reserved.insert(output.clone(), height);
            let edges = json!({"output": output, "edges": {"bottom": height}});
            if let Err(err) = self.rt.conn.call::<_, Value>("output.reserve", edges) {
                eprintln!("chora-xp: output.reserve: {err}");
            }
        }
    }

    /// The Control Panel for an output, with the current choices.
    pub(super) fn control_panel(&mut self) -> Vec<apps::Node> {
        let walls = control::wallpapers(&wallpaper::dirs(&self.theme));
        let current = self.settings.value.wallpaper.file(&self.theme);
        control::menu(
            current.as_deref(),
            &walls,
            self.effects.toggles,
            self.volume.unwrap_or(Reading::Failed),
            &self.held_shortcuts(),
        )
    }

    /// Changes a setting: the scale of the output of the menu through the
    /// core, the wallpaper of every output, or an effect.
    pub(super) fn set(&mut self, setting: &Setting, output: Option<&str>) -> Vec<Command> {
        match setting {
            Setting::Scale(percent) => {
                let params = OutputConfigure {
                    output: output.map(String::from),
                    scale: Some(f64::from(*percent) / 100.0),
                    transform: None,
                };
                if let Err(err) = self.rt.conn.call::<_, Value>("output.configure", params) {
                    eprintln!("chora-xp: {err}");
                }
                Vec::new()
            }
            Setting::Wallpaper(file) => {
                let style = Wallpaper {
                    picture: Picture::File(file.clone()),
                    ..self.settings.value.wallpaper.clone()
                };
                self.change(json!({ "wallpaper": style }))
            }
            Setting::Effect(effect) => {
                let mut toggles = self.effects.toggles;
                toggles.flip(*effect);
                self.change(json!({ "effects": toggles }))
            }
            Setting::Volume(change) => self.change_volume(*change, false),
            Setting::Give(to) => {
                if let Err(err) = give::focused(self, to) {
                    self.notice(None, "Keyboard Shortcuts", &err.message, -1);
                }
                Vec::new()
            }
            Setting::Note => Vec::new(),
            Setting::Taskbar | Setting::DateTime | Setting::Display => {
                let area = output
                    .and_then(|o| self.bars.iter().find(|b| b.output == o))
                    .or(self.bars.first())
                    .map_or([0, 0, 1280, 770], Bar::usable);
                match setting {
                    Setting::Taskbar => self.open_taskbar_dialog(area),
                    Setting::DateTime => self.open_date_dialog(area),
                    _ => self.open_display_dialog(area),
                }
            }
        }
    }

    /// Shows a dialog in the middle of the output of the start menu, and
    /// lets the desktop behind it fade to gray.
    pub(super) fn open_dialog(&mut self, mode: Mode, output: Option<&str>) -> Vec<Command> {
        let bar = output
            .and_then(|o| self.bars.iter().find(|b| b.output == o))
            .or(self.bars.first());
        let Some(bar) = bar else {
            return Vec::new();
        };
        let [.., w, h] = bar.area();
        let suspend = (self.settings.value.session.stand_by.as_deref())
            .and_then(|c| c.split_whitespace().next())
            .is_some_and(session::on_path);
        let dialog = Dialog::new(mode, &bar.output, [w, h], suspend);
        let mut commands = dialog.create(&mut self.theme);
        self.dialog = Some(dialog);
        if self.fade_shader.is_none() {
            match self.theme.cache_file("fade.frag", session::FADE_SHADER) {
                Ok(path) => self.fade_shader = Some(path.to_string_lossy().into_owned()),
                Err(err) => eprintln!("chora-xp: the shader of the fade: {err}"),
            }
        }
        if self.fade_shader.is_some() && self.effects_on() {
            self.fade = Some(Instant::now());
            commands.extend(self.fade_step(0.0));
        }
        commands
    }

    /// The gray of the desktop: the whole plugin becomes a layer with the
    /// shader of the fade. The dialog sits on the overlay root, outside it.
    /// The gray starts at the origin of the scene; outputs left of or above
    /// it stay in color.
    pub(super) fn fade_step(&self, amount: f64) -> Option<Command> {
        let shader = self.fade_shader.clone()?;
        let size = self.bars.iter().fold([1.0_f64, 1.0_f64], |[w, h], bar| {
            let [x, y, bw, bh] = bar.area();
            [w.max(f64::from(x + bw)), h.max(f64::from(y + bh))]
        });
        Some(Command::Update {
            id: NAME.into(),
            props: PropsUpdate {
                size: Some(size),
                effects: Some(vec![ShaderEffect {
                    shader,
                    uniforms: [("amount".into(), Uniform::Float(amount))].into(),
                }]),
                ..PropsUpdate::default()
            },
        })
    }

    /// Whether the desktop still fades.
    pub fn fading(&self) -> bool {
        self.fade.is_some_and(|start| start.elapsed() < FADE)
    }

    /// The next frame of the fade.
    pub(crate) fn fade_frame(&self, now: Instant) -> Option<Command> {
        let start = self.fade?;
        let t = now.saturating_duration_since(start).as_secs_f64() / FADE.as_secs_f64();
        self.fade_step(t.min(1.0))
    }

    /// A press on a button of the dialog; a press outside it closes it,
    /// as Cancel does.
    pub(super) fn press_dialog(&mut self, node: &str) -> Vec<Command> {
        let Some(dialog) = &mut self.dialog else {
            return Vec::new();
        };
        match dialog.press(&mut self.theme, node) {
            Some(commands) => commands,
            None => self.close_dialog(),
        }
    }

    pub(super) fn close_dialog(&mut self) -> Vec<Command> {
        if self.dialog.take().is_none() {
            return Vec::new();
        }
        let mut commands = vec![Dialog::remove()];
        if self.fade.take().is_some() {
            commands.push(Command::Update {
                id: NAME.into(),
                props: PropsUpdate {
                    effects: Some(Vec::new()),
                    ..PropsUpdate::default()
                },
            });
        }
        commands
    }

    /// Plays one of XP's sounds, unless the Control Panel turned them off.
    pub(super) fn play(&self, sound: Sound) {
        if self.effects.toggles.sounds {
            sound::play(sound);
        }
    }

    /// Does what a button of a dialog says: runs the command of the
    /// settings, or the shared power script without one.
    pub(super) fn choose(&mut self, choice: Choice) -> Vec<Command> {
        let commands = self.close_dialog();
        let session = self.settings.value.session.clone();
        match choice {
            Choice::Lock => self.run_shared("power", &["lock"]),
            Choice::LogOff => {
                self.play(Sound::LogOff);
                self.run_session(session.log_off, "logout");
            }
            Choice::StandBy => session.stand_by.into_iter().for_each(session::run),
            Choice::TurnOff => {
                self.play(Sound::ShutDown);
                self.run_session(session.turn_off, "shutdown");
            }
            Choice::Restart => self.run_session(session.restart, "restart"),
            Choice::SwitchUser | Choice::Cancel => {}
        }
        commands
    }

    /// Runs the command that the person set, or else the action of the
    /// shared power script.
    pub(super) fn run_session(&mut self, own: Option<String>, action: &str) {
        match own {
            Some(command) => session::run(command),
            None => self.run_shared("power", &[action]),
        }
    }

    /// Runs a shared script; a notice says so when it is not installed.
    pub(super) fn run_shared(&mut self, script: &str, args: &[&str]) {
        if let Some(command) = shared::command(script, args) {
            session::run(command);
            return;
        }
        let body = format!("The file {script}.sh of the shared scripts is not installed.");
        self.notice(None, "Cannot do that", &body, 10_000);
    }
}
