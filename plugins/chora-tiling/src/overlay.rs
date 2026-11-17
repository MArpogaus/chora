//! The bar, theme, chip and key list that tiling draws over the windows.

use chora_scene::{Command, PropsUpdate, ipc::ClientError};
use serde_json::Value;

use crate::{
    ANSWER_CHECK, CHIP_TIME, NAME, PANEL_UNIT, THEME_CHECK, Tiling, WALLPAPER_UNIT, dress, keys,
    theme,
};

impl Tiling {
    /// When the loop must wake without an event: the next frame of an
    /// effect, or the end of the layout chip.
    pub(super) fn wake(&self, frame: std::time::Instant) -> Option<std::time::Instant> {
        let frame = self.effects.animating().then_some(frame);
        [frame, self.chip_until, self.theme_check]
            .into_iter()
            .flatten()
            .min()
    }

    /// Takes the colours of the theme and writes the files of the flavor
    /// that carry them; `reload` asks the panel to read its style again,
    /// which it must when the colours changed under it, and the wallpaper
    /// to start again when the scheme changed. A panel or wallpaper that
    /// `waybar.sh` or `wallpaper.sh` runs gets `SIGUSR2`, else the unit is
    /// reloaded or restarted; without a user manager there is nothing
    /// to do.
    pub(super) fn apply_theme(&mut self, reload: bool) {
        let look = &self.settings.value.look;
        let scheme = look.theme.scheme(
            (look.theme == theme::Theme::System)
                .then_some(self.system)
                .flatten(),
        );
        let colors = look.palette.over(scheme);
        let changed = colors != self.colors;
        let new_scheme = scheme != self.scheme;
        self.theme_check =
            (look.theme == theme::Theme::System).then(|| std::time::Instant::now() + THEME_CHECK);
        self.scheme = scheme;
        self.colors = colors;
        let _ = self.name_theme();
        let Some(dir) = theme::default_dir() else {
            return;
        };
        if let Err(err) = theme::apply(&dir, &self.colors, scheme) {
            eprintln!("chora-tiling: the files of the theme: {err}");
            return;
        }
        let user_unit = |verb: &str, unit: &str| {
            let _ = std::process::Command::new("systemctl")
                .args(["--user", "--no-block", verb, unit])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        };
        if reload && changed {
            match theme::companion("panel", "waybar.sh") {
                Some(pid) => theme::signal(pid, "USR2"),
                None => user_unit("reload", PANEL_UNIT),
            }
        }
        if reload && changed {
            let _ = std::process::Command::new("makoctl")
                .arg("reload")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
        if reload && new_scheme {
            match theme::companion("wallpaper", "wallpaper.sh") {
                Some(pid) => theme::signal(pid, "USR2"),
                None => user_unit("restart", WALLPAPER_UNIT),
            }
        }
    }

    /// Asks the system for its colour scheme when it is time, in a thread
    /// so that the loop never waits for the answer, and takes the new
    /// colours of the theme when the answer changed them.
    pub(super) fn follow_system(&mut self, now: std::time::Instant) -> Result<(), ClientError> {
        use std::sync::mpsc::TryRecvError;
        let Some(query) = &self.theme_query else {
            if self.theme_check.is_none_or(|at| now < at) {
                return Ok(());
            }
            let (send, receive) = std::sync::mpsc::channel();
            let ask = self.ask_system;
            std::thread::spawn(move || {
                let _ = send.send(ask());
            });
            self.theme_query = Some(receive);
            self.theme_check = Some(now + ANSWER_CHECK);
            return Ok(());
        };
        match query.try_recv() {
            Ok(answer) => self.system = answer,
            Err(TryRecvError::Empty) => {
                self.theme_check = Some(now + ANSWER_CHECK);
                return Ok(());
            }
            Err(TryRecvError::Disconnected) => {}
        }
        self.theme_query = None;
        let before = self.colors.clone();
        self.apply_theme(true);
        if self.colors != before {
            self.sent.clear();
            self.still = true;
            let done = self.relayout();
            self.still = false;
            done?;
            if self.keys_open {
                self.hide_keys()?;
                self.toggle_keys()?;
            }
        }
        Ok(())
    }

    /// Tells the panel at once when the tags or the layout changed, with
    /// `SIGRTMIN+1` to waybar, which the modules of the tags and of the
    /// layout answer by asking again; without a panel nothing happens.
    pub(super) fn push_bar(&mut self) {
        self.hide_bar_for_fullscreen();
        if !self.bar_changed() {
            return;
        }
        if let Some(pid) = theme::companion("waybar", "waybar") {
            theme::signal(pid, "RTMIN+1");
        }
    }

    /// Names the theme to the HUD host, which colours its panels, chat and
    /// question card by it.
    pub(super) fn name_theme(&mut self) -> Result<(), ClientError> {
        let emit = chora_kit::hud::theme_event(NAME, self.scheme == theme::Scheme::Ink);
        self.rt.conn.call::<_, Value>("events.emit", emit).map(drop)
    }

    /// Hides the panel with `SIGUSR1` while a window is fullscreen and shows
    /// it again after: the islands never float over the content.
    pub(super) fn hide_bar_for_fullscreen(&mut self) {
        let want = self.wm.fullscreen_shown();
        if self.bar_hidden.is_some_and(|(_, hidden)| hidden == want)
            || (!want && self.bar_hidden.is_none())
        {
            return;
        }
        let Some(pid) = theme::companion("waybar", "waybar") else {
            self.bar_hidden = None;
            return;
        };
        let hidden = self
            .bar_hidden
            .filter(|(old, _)| *old == pid)
            .is_some_and(|(_, hidden)| hidden);
        if hidden != want {
            theme::signal(pid, "USR1");
        }
        self.bar_hidden = Some((pid, want));
    }

    /// Whether the layout or the tags differ from what the panel was last
    /// told; takes the new state.
    pub(super) fn bar_changed(&mut self) -> bool {
        let layout = self.wm.current().map(|(layout, _)| layout);
        let state = (layout, self.wm.tags());
        if self.bar.as_ref() == Some(&state) {
            return false;
        }
        self.bar = Some(state);
        true
    }

    /// Shows the chip that names the layout of the current tag for 700 ms.
    pub(super) fn show_chip(&mut self) -> Result<(), ClientError> {
        let (Some((output, size)), Some((layout, _))) =
            (self.wm.current_output(), self.wm.current())
        else {
            return Ok(());
        };
        let lang = chora_kit::words::Language::of_session();
        let mut commands = Vec::new();
        if self.chip_until.is_some() {
            commands.push(Command::Remove {
                id: keys::CHIP.into(),
                to: None,
            });
        }
        commands.extend(keys::chip(
            &format!("screen:{output}"),
            size,
            keys::layout_word(layout, lang),
            &self.colors,
        ));
        self.chip_until = Some(std::time::Instant::now() + CHIP_TIME);
        if self.apply_report(commands)? {
            let guess = keys::guess(keys::layout_word(layout, lang));
            let measured = self.measured(&format!("{}-word", keys::CHIP));
            let animate = self.wm.config.look.animations;
            self.apply(keys::chip_fit(size, measured.unwrap_or(guess), animate))?;
        }
        Ok(())
    }

    /// The size that the renderer gave a text node, in logical pixels.
    pub(super) fn measured(&mut self, id: &str) -> Option<[f64; 2]> {
        let nodes = self.query(&format!("id:{id}")).ok()?;
        let rect = nodes
            .first()
            .and_then(|n| n["rects"].as_object())
            .and_then(|r| r.values().next())
            .and_then(Value::as_array);
        rect.and_then(|r| Some([r.get(2)?.as_f64()?, r.get(3)?.as_f64()?]))
    }

    /// Takes the chip away when its time is up.
    pub(super) fn end_chip(&mut self, now: std::time::Instant) -> Result<(), ClientError> {
        if self.chip_until.is_some_and(|at| now >= at) {
            self.chip_until = None;
            self.apply(vec![Command::Remove {
                id: keys::CHIP.into(),
                to: None,
            }])?;
        }
        Ok(())
    }

    /// Shows the key list when it is hidden, and hides it when it shows.
    pub(super) fn toggle_keys(&mut self) -> Result<(), ClientError> {
        if self.keys_open {
            return self.hide_keys();
        }
        let Some((output, size)) = self.wm.current_output() else {
            return Ok(());
        };
        let lang = chora_kit::words::Language::of_session();
        let rows = keys::rows(&self.wm.config, lang);
        let commands = keys::key_list(&format!("screen:{output}"), size, &rows, &self.colors, lang);
        if self.apply_report(commands)? {
            let fit = self.key_list_fit(size);
            if !fit.is_empty() {
                self.apply(fit)?;
            }
        }
        let fade = self.wm.config.look.animations.then_some(dress::FADE);
        let mut commands = vec![dress::change(
            keys::KEYS.into(),
            PropsUpdate {
                opacity: Some(1.0),
                ..PropsUpdate::default()
            },
            fade,
        )];
        commands.push(Command::FocusNode {
            node: Some(keys::KEYS.into()),
        });
        self.apply(commands)?;
        self.keys_open = true;
        Ok(())
    }

    /// The updates that make the panel of the key list as wide as the
    /// longest word that the renderer drew; none when it reports nothing.
    pub(super) fn key_list_fit(&mut self, output: [f64; 2]) -> Vec<Command> {
        let rect = |node: &Value| {
            let r = node["rects"].as_object()?.values().next()?.as_array()?;
            Some([
                r.first()?.as_f64()?,
                r.get(1)?.as_f64()?,
                r.get(2)?.as_f64()?,
                r.get(3)?.as_f64()?,
            ])
        };
        let Some(panel) = self
            .query(&format!("id:{}-panel", keys::KEYS))
            .ok()
            .and_then(|nodes| nodes.first().and_then(rect))
        else {
            return Vec::new();
        };
        let words = self.query("kind:text").unwrap_or_default();
        let right = words
            .iter()
            .filter(|n| n["id"].as_str().is_some_and(keys::sets_width))
            .filter_map(rect)
            .map(|r| r[0] + r[2] - panel[0])
            .fold(0.0, f64::max);
        if right <= 0.0 {
            return Vec::new();
        }
        keys::fit_list(output, panel[3], right)
    }

    /// Hides the key list and gives the keyboard back.
    pub(super) fn hide_keys(&mut self) -> Result<(), ClientError> {
        self.keys_open = false;
        self.apply(vec![
            Command::FocusNode { node: None },
            Command::Remove {
                id: keys::KEYS.into(),
                to: None,
            },
        ])
    }
}
