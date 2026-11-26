//! The bar, theme, chip and key list that tiling draws over the windows.

use chora_kit::{actions::System, keysheet};
use chora_scene::{Command, PropsUpdate, ipc::ClientError};
use serde_json::Value;

use crate::{
    ANSWER_CHECK, Action, CHIP_TIME, HINT_AGAIN, HINT_TIME, NAME, PANEL_UNIT, THEME_CHECK, Tiling,
    WALLPAPER_UNIT, dress, keys, theme,
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
            if self.keys_open() {
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
        let dark = self.scheme == theme::Scheme::Ink;
        let emit = chora_kit::hud::theme_event(NAME, dark);
        self.rt.conn.call::<_, Value>("events.emit", emit)?;
        let look = chora_kit::hud::look_theme_event(dark);
        self.rt.conn.call::<_, Value>("events.emit", look).map(drop)
    }

    /// Hides the panel with `SIGUSR1` while a window is fullscreen and shows
    /// it again after: the islands never float over the content.
    pub(super) fn hide_bar_for_fullscreen(&mut self) {
        let want = self.wm.fullscreen_shown();
        let (record, toggle) = bar_step(self.bar_hidden, want, || {
            theme::companion("waybar", "waybar")
        });
        self.bar_hidden = record;
        if let Some(pid) = toggle {
            theme::signal(pid, "USR1");
        }
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
        let Some((layout, _)) = self.wm.current() else {
            return Ok(());
        };
        let lang = chora_kit::words::Language::of_session();
        self.show_words(keys::layout_word(layout, lang), CHIP_TIME)
    }

    /// Says on the chip that nothing is bound to the chord `binding`, and
    /// which key shows them all: for 8 seconds (slow eyes and a look 4.3 s after the key can read it), at most once in 10 seconds,
    /// and not while the key list shows. The key stays swallowed.
    pub(super) fn hint_unbound(&mut self, binding: &str) -> Result<(), ClientError> {
        let now = std::time::Instant::now();
        let due = self
            .hinted
            .is_none_or(|at| now.duration_since(at) >= HINT_AGAIN);
        if self.keys_open() || !due {
            return Ok(());
        }
        self.hinted = Some(now);
        let config = &self.wm.config;
        let help = (config.common.bindings.iter())
            .find(|(_, action)| **action == Action::System(System::Keys))
            .and_then(|(text, _)| config.binding(text).ok())
            .map(|b| b.to_string());
        let language = chora_kit::words::Language::of_session();
        let chord = keysheet::chord(binding);
        let words = match help {
            Some(help) => chora_kit::words::say(
                language,
                chora_kit::words::Word::KeyUnbound,
                &[("chord", &chord), ("key", &keysheet::chord(&help))],
            ),
            None => chora_kit::words::say(
                language,
                chora_kit::words::Word::KeyUnboundBare,
                &[("chord", &chord)],
            ),
        };
        self.show_words(&words, HINT_TIME)
    }

    /// Shows the chip with `word` in the middle of the current output for
    /// `time`; a chip that still shows gives way to it.
    pub(super) fn show_words(
        &mut self,
        word: &str,
        time: std::time::Duration,
    ) -> Result<(), ClientError> {
        let Some((output, size)) = self.wm.current_output() else {
            return Ok(());
        };
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
            word,
            &self.colors,
        ));
        self.chip_until = Some(std::time::Instant::now() + time);
        if self.apply_report(commands)? {
            let guess = keys::guess(word);
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
        if self.keys_open() {
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
        self.open.open(
            chora_kit::wm::open::Surface::new(keys::KEYS, chora_kit::wm::open::Kind::Sheet).modal(),
        );
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

    /// Whether the key list shows, and holds the keyboard.
    pub(super) fn keys_open(&self) -> bool {
        self.open.contains(keys::KEYS)
    }

    /// Hides the key list when it shows, and does nothing when it does not.
    pub(super) fn hide_keys_if_open(&mut self) -> Result<(), ClientError> {
        if self.keys_open() {
            self.hide_keys()?;
        }
        Ok(())
    }

    /// Hides the key list and gives the keyboard back.
    pub(super) fn hide_keys(&mut self) -> Result<(), ClientError> {
        self.open.shut(keys::KEYS);
        self.apply(vec![
            Command::FocusNode { node: None },
            Command::Remove {
                id: keys::KEYS.into(),
                to: None,
            },
        ])
    }
}

/// What the bar needs after an event: the new record of the panel that was
/// told to hide (its pid; none while it shows) and the pid that gets
/// `SIGUSR1`, which toggles the panel, when its state is not the wanted
/// one. `panel` names the running panel. It is asked at each event while a
/// window is fullscreen or the record names a hidden panel, and the pid is
/// compared each time: a panel that started again under a fullscreen
/// window shows, whatever the record says, and is told anew.
pub(super) fn bar_step(
    record: Option<(u32, bool)>,
    want: bool,
    panel: impl FnOnce() -> Option<u32>,
) -> (Option<(u32, bool)>, Option<u32>) {
    if !want && record.is_none() {
        return (None, None);
    }
    let Some(pid) = panel() else {
        return (None, None);
    };
    let hidden = record
        .filter(|(old, _)| *old == pid)
        .is_some_and(|(_, hidden)| hidden);
    (want.then_some((pid, true)), (hidden != want).then_some(pid))
}

#[cfg(test)]
mod tests_moved;
