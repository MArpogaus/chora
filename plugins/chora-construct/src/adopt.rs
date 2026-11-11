//! Adoption of windows as panes: place, hide, launcher start, a window
//! found, resize and forget, and the launcher pid check.

use super::{
    ClientError, Command, Construct, Content, Hold, Instant, Kind, LAUNCHED, NAME, NO_ROOM, Pane,
    PaneFile, PropsUpdate, Transform, WINDOWS, Waiting, configure, update,
};

/// Whether a window with the token of the launcher is the launcher's own:
/// a launcher without a window, such as fuzzel, may pass its token on to
/// the program that it starts, another process. The launcher runs under
/// `sh -c`, which does not always exec its last command, so the pid of the
/// launcher may be that of the shell and the window the shell's child.
/// Without both pids, the window is the launcher's.
pub(crate) fn launchers_own(window: Option<u32>, launcher: Option<u32>) -> bool {
    launchers_own_by(window, launcher, process)
}

pub(crate) fn launchers_own_by(
    window: Option<u32>,
    launcher: Option<u32>,
    process: impl Fn(u32) -> Option<(u32, String)>,
) -> bool {
    let Some((window, launcher)) = window.zip(launcher) else {
        return true;
    };
    window == launcher
        || (process(launcher).is_some_and(|(_, comm)| SHELLS.contains(&comm.as_str()))
            && process(window).is_some_and(|(parent, _)| parent == launcher))
}

/// The names of the shells that wrap a launcher.
pub(crate) const SHELLS: [&str; 3] = ["sh", "dash", "bash"];

/// The parent pid and the command name of a process, from `/proc`.
pub(crate) fn process(pid: u32) -> Option<(u32, String)> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let (comm, rest) = stat.get(stat.find('(')? + 1..)?.rsplit_once(')')?;
    let parent = rest.split_whitespace().nth(1)?.parse().ok()?;
    Some((parent, comm.into()))
}

impl Construct {
    /// Attaches a window under the construct, hidden, with no pane.
    pub(crate) fn attach_hidden(&mut self, surface: &str) -> Result<(), ClientError> {
        let hidden = PropsUpdate {
            opacity: Some(0.0),
            ..PropsUpdate::default()
        };
        let attach = Command::Attach {
            surface: surface.into(),
            parent: WINDOWS.into(),
        };
        if let Some(err) = self
            .rt
            .conn
            .try_apply(vec![attach, update(surface, hidden)])?
        {
            eprintln!("chora-construct: {surface}: {err}");
        } else {
            self.hidden.push(surface.into());
        }
        Ok(())
    }

    /// A new window becomes a pane, ahead of the camera or where its file
    /// puts it. The construct puts its own window into the pane, where it
    /// takes the input through the ray of the core; the pane shows the
    /// window of another owner through a node texture.
    #[allow(clippy::too_many_arguments, reason = "the window and how it comes")]
    pub(crate) fn adopt(
        &mut self,
        surface: &str,
        label: &str,
        file: Option<&PaneFile>,
        hold: Hold,
        force: bool,
    ) -> Result<(), ClientError> {
        if !self.active || self.panes.iter().any(|p| p.window() == Some(surface)) {
            return Ok(());
        }
        let size = (file.and_then(|f| f.size)).unwrap_or([self.lens.w, self.lens.h]);
        let Some(at) = self.place(file.and_then(|f| f.at), size, force) else {
            self.voice.notice(NO_ROOM, Instant::now());
            if hold == Hold::Own {
                self.attach_hidden(surface)?;
            }
            return self.draw_hud();
        };
        let pane = Pane {
            name: self.label(label),
            source: surface.into(),
            content: Content::Window {
                command: file.and_then(|f| f.command.clone()),
            },
            at,
            yaw: self.yaw_at(file, at),
            size,
            taken: hold == Hold::Taken,
            attached: hold == Hold::Own,
        };
        let mut commands = self.pane_nodes(&pane);
        self.add_placeholder(&mut commands, &pane)?;
        if pane.attached {
            let fresh = !self.hidden.contains(&surface.to_string());
            self.hidden.retain(|s| s != surface);
            commands.extend([
                if fresh {
                    Command::Attach {
                        surface: surface.into(),
                        parent: pane.node(),
                    }
                } else {
                    Command::Move {
                        id: surface.into(),
                        parent: Some(pane.node()),
                        index: None,
                    }
                },
                update(
                    surface,
                    PropsUpdate {
                        opacity: Some(1.0),
                        transform: Some(Transform::translate(0.0, 0.0, 0.0)),
                        ..PropsUpdate::default()
                    },
                ),
                configure(surface, size),
            ]);
        }
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            eprintln!("chora-construct: {surface}: {err}");
            return Ok(());
        }
        self.panes.push(pane);
        self.frame()
    }

    /// Whether the window has no buffer yet: the core knows no size of it.
    fn no_buffer(&mut self, surface: &str) -> bool {
        let size = (self.rt.conn.query(&format!("id:{surface}")).ok())
            .and_then(|nodes| nodes.into_iter().next()?.surface?.size);
        size.is_none_or(|[w, h]| w <= 0 || h <= 0)
    }

    /// Puts the placeholder of the pane into its commands, right after
    /// the node of the pane, so that it lies under the window, unless the
    /// window has its first buffer.
    fn add_placeholder(
        &mut self,
        commands: &mut Vec<Command>,
        pane: &Pane,
    ) -> Result<(), ClientError> {
        if !self.no_buffer(&pane.source) {
            return Ok(());
        }
        self.unfilled.push(pane.source.clone());
        let command = match &pane.content {
            Content::Window { command } => command.clone(),
            _ => None,
        };
        let name = self.window_label(&pane.source, command.as_deref());
        let node = pane.node();
        let wait = self.fitted(|s| {
            crate::placeholder::draw(&s.styled(), pane.size, &node, &pane.source, &name)
        })?;
        let after = (commands.iter())
            .position(|c| matches!(c, Command::Create { id, .. } if *id == node))
            .map_or(commands.len(), |at| at + 1);
        for (k, command) in wait.into_iter().enumerate() {
            commands.insert(after + k, command);
        }
        Ok(())
    }

    /// A new window: the launcher's own window, a pane of the construct,
    /// the program of the launcher, or, for the flavor, a free window.
    #[allow(clippy::too_many_arguments, reason = "the fields of surface.created")]
    pub(crate) fn created(
        &mut self,
        surface: &str,
        label: &str,
        to: Option<&str>,
        pid: Option<u32>,
        token: Option<&str>,
    ) -> Result<(), ClientError> {
        let free = to.is_none_or(|t| t == NAME);
        self.waiting.retain(|w| !w.stale());
        let started = (self.launcher.as_ref()).filter(|(t, _)| token == Some(t.as_str()));
        if let Some(&(_, launcher_pid)) = started {
            self.launcher = None;
            if !launchers_own(pid, launcher_pid) {
                self.launched = None;
                self.adopt(surface, label, None, self.started(), true)?;
                return self.fly_into_window(surface);
            }
            if free && !self.overlay {
                return self.show_launcher(surface);
            }
            return Ok(());
        }
        let i = (self.waiting.iter()).position(|w| token.is_some() && w.token.as_deref() == token);
        let waited = i.map(|i| self.waiting.remove(i));
        let by_key = waited.as_ref().is_some_and(|w| w.key);
        let pane = waited.map(|w| w.pane);
        let launched =
            pane.is_none() && (self.launched.take()).is_some_and(|t| t.elapsed() < LAUNCHED);
        let force = pane.is_some() || launched;
        if pane.is_some() || launched || (free && !self.overlay) {
            self.adopt(surface, label, pane.as_ref(), self.started(), force)?;
            if by_key || launched {
                return self.fly_into_window(surface);
            }
        }
        Ok(())
    }

    /// Flies into the pane of a window that the person just opened, so
    /// that it shows at 1:1 and has the keyboard.
    fn fly_into_window(&mut self, surface: &str) -> Result<(), ClientError> {
        match (self.panes.iter()).position(|p| p.window() == Some(surface)) {
            Some(i) => self.fly_in(i),
            None => Ok(()),
        }
    }

    /// The window of a launcher shows over the space, in the middle of
    /// the output once its size comes, with the keyboard; its close gives
    /// the keyboard back to the walk.
    pub(crate) fn show_launcher(&mut self, surface: &str) -> Result<(), ClientError> {
        self.launcher_window = Some(surface.into());
        let commands = vec![
            Command::Attach {
                surface: surface.into(),
                parent: NAME.into(),
            },
            Command::Focus {
                surface: Some(surface.into()),
            },
        ];
        if let Some(err) = self.rt.conn.try_apply(commands)? {
            eprintln!("chora-construct: launcher {surface}: {err}");
        }
        Ok(())
    }

    /// Starts the program of the key `terminal`; its window becomes a
    /// pane ahead, as a program pane of a construct.
    pub(crate) fn start_terminal(&mut self) {
        self.start_terminal_at(None);
    }

    /// Starts the terminal; its pane stands at the place and yaw `spot`,
    /// or where a new pane goes.
    pub(crate) fn start_terminal_at(&mut self, spot: Option<([f64; 2], f64)>) {
        self.waiting.retain(|w| !w.stale());
        let command = self.settings.value.terminal.clone();
        let mint = self.rt.conn.activation_token();
        let token = chora_kit::launch(&command, mint.as_deref()).map(|(t, _)| t);
        let pane = PaneFile {
            kind: Kind::Program,
            command: Some(command),
            plugin: None,
            scene: None,
            manager: None,
            at: spot.map(|(at, _)| at),
            yaw: spot.map(|(_, yaw)| yaw),
            size: None,
        };
        self.waiting.push(Waiting {
            key: true,
            ..Waiting::new(token, pane)
        });
    }

    /// Starts the launcher. Its own window, if it has one and the
    /// token, is no pane; the next window after it is.
    pub(crate) fn start_launcher(&mut self) {
        let mint = self.rt.conn.activation_token();
        self.launcher = chora_kit::launch(&self.settings.value.launcher, mint.as_deref());
        self.launched = Some(Instant::now());
    }

    /// Asks the window of the pane ahead to close; its pane goes when
    /// it does. A plugin pane goes at once.
    pub(crate) fn close_ahead(&mut self) -> Result<(), ClientError> {
        match self.pane_ahead() {
            Some(i) => self.remove_pane(i),
            None => Ok(()),
        }
    }

    pub(crate) fn resize(&mut self, surface: &str, size: [f64; 2]) -> Result<(), ClientError> {
        if self.launcher_window.as_deref() == Some(surface) {
            return self.rt.conn.apply(vec![self.snap_place(size, surface)]);
        }
        if size[0] > 0.0 && size[1] > 0.0 && self.unfilled.iter().any(|s| s == surface) {
            self.unfilled.retain(|s| s != surface);
            let gone = Command::Remove {
                id: crate::placeholder::id(surface),
                to: None,
            };
            self.rt.conn.try_apply(vec![gone])?;
        }
        let Some(i) = (self.panes.iter()).position(|p| p.window() == Some(surface)) else {
            return Ok(());
        };
        let pane = &mut self.panes[i];
        if pane.size.iter().zip(size).all(|(a, b)| (a - b).abs() < 0.5) {
            return Ok(());
        }
        pane.size = size;
        let pane = &self.panes[i];
        let mut commands = self.placed(pane);
        if (self.flown.as_ref()).is_some_and(|f| f.surface == surface && f.attached) {
            commands.push(self.snap_place(size, surface));
        }
        self.rt.conn.apply(commands)?;
        self.frame()
    }

    /// A window closed or left: its pane goes. A pane that the camera
    /// is in flies back out first.
    pub(crate) fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        self.forget_with(surface, true)
    }

    /// Removes the pane of a window; the camera in it goes back, and the
    /// keyboard to the walk when `walk` is set. A window given away
    /// leaves the keyboard to its new owner.
    pub(crate) fn forget_with(&mut self, surface: &str, walk: bool) -> Result<(), ClientError> {
        let Some(i) = (self.panes.iter()).position(|p| p.window() == Some(surface)) else {
            return Ok(());
        };
        let pane = self.panes.remove(i);
        self.unfilled.retain(|s| s != surface);
        self.rt.conn.apply(pane.remove().into())?;
        if let Some(flown) = self.flown.take_if(|f| f.surface == surface) {
            self.unsnap(None, walk)?;
            self.pose = flown.back;
        }
        if let Some(into) = (self
            .glide
            .take_if(|g| g.into.as_ref().is_some_and(|f| f.surface == surface)))
        .and_then(|g| g.into)
        {
            self.pose = into.back;
        }
        self.frame()
    }
}
