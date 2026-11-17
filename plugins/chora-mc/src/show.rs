use super::{
    Chat, ClientError, Command, Easing, FADE, Kind, Mc, Panel, PropsUpdate, RpcError, Scene,
    ShaderEffect, Shaders, TRAIL, Target, Uniform, invalid, keys, launch, name_of, page_id,
    pane_tag_update, panel_id, region, regions_on_top, rejected, scene_id, wait,
};

impl Mc {
    /// Writes the text of the panels of a scene anew: the effect starts at
    /// 0 and the core animates it to 1. Nothing when the setting is off.
    /// at most 4 KiB of lines
    /// ponytail: a line that wraps writes with the last line; the
    /// plugin does not know the layout of the core.
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn write(&self, name: &str, panels: &[&Panel]) -> Vec<Command> {
        let config = &self.settings.value;
        if !config.write.enabled || self.shaders == Shaders::Dropped {
            return Vec::new();
        }
        let mut commands = Vec::new();
        for panel in panels {
            let effect = |rows: [f64; 3], progress: f64| {
                let write = ShaderEffect {
                    shader: self.shader.to_string_lossy().into_owned(),
                    uniforms: [
                        ("progress".into(), Uniform::Float(progress)),
                        ("rows".into(), Uniform::Vector(rows.to_vec())),
                    ]
                    .into(),
                };
                Some(
                    std::iter::once(write)
                        .chain(self.blur_effect(panel))
                        .collect(),
                )
            };
            let Some(h) = self
                .place(name, panel)
                .map(|r| self.whole_lines(panel, r[3]))
            else {
                continue;
            };
            let line = self.ty("body", panel.style.as_ref()).line;
            let rows = [
                self.lines_top(panel).max(line) / h,
                line / h,
                1.0 + panel.lines.len() as f64,
            ];
            let id = panel_id(name, &panel.id);
            commands.push(Command::Update {
                id: id.clone(),
                props: PropsUpdate {
                    effects: effect(rows, 0.0),
                    ..PropsUpdate::default()
                },
            });
            commands.push(Command::Animate {
                id,
                props: PropsUpdate {
                    effects: effect(rows, 1.0),
                    ..PropsUpdate::default()
                },
                duration: config.write.duration_ms,
                easing: Easing::Linear,
            });
        }
        commands
    }

    /// Puts a window into the text block of its scene, below the head,
    /// without a frame, at the box that the render gives it too.
    pub(crate) fn place_window(&self, name: &str, surface: &str, attach: bool) -> Vec<Command> {
        let (Some(output), Some(window)) = (&self.output, self.window_box(name)) else {
            return Vec::new();
        };
        let parent = scene_id(name);
        let put = if attach {
            Command::Attach {
                surface: surface.into(),
                parent,
            }
        } else {
            Command::Move {
                id: surface.into(),
                parent: Some(parent),
                index: None,
            }
        };
        let mut commands = vec![put];
        commands.extend(region::place(surface, window, output.size));
        commands
    }

    /// Shows a scene. A switch of the user, a new window or the scene
    /// after a closed window gives the keyboard to the window of the
    /// scene, or to none: a method never takes it from a window.
    /// The regions switch in the same transaction as the scene.
    /// A hidden scene has no layout of the core: its notes align now.
    pub(crate) fn show(&mut self, name: &str, user: bool) -> Result<(), RpcError> {
        let Some(scene) = self.scenes.get(name) else {
            return Err(invalid(format!("no scene {name}")));
        };
        let app = scene.app.clone();
        let mut commands = Vec::new();
        if self.current.as_deref() != Some(name) {
            if let Some(old) = &self.current {
                commands.push(Command::Animate {
                    id: scene_id(old),
                    props: PropsUpdate {
                        opacity: Some(0.0),
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                    duration: FADE,
                    easing: Easing::EaseInOut,
                });
            }
            commands.push(Command::Update {
                id: scene_id(name),
                props: PropsUpdate {
                    visible: Some(true),
                    ..PropsUpdate::default()
                },
            });
            commands.push(Command::Animate {
                id: scene_id(name),
                props: PropsUpdate {
                    opacity: Some(1.0),
                    ..PropsUpdate::default()
                },
                duration: FADE,
                easing: Easing::EaseInOut,
            });
            let panels: Vec<&Panel> = scene.panels.iter().collect();
            commands.extend(self.write(name, &panels));
        }
        let window = self.windows.get(name).cloned();
        if user {
            commands.extend(self.keyboard_to(name, window.clone()));
        }
        self.step_to(name);
        commands.push(pane_tag_update(name));
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        commands.extend(self.regions().map_err(rejected)?);
        self.apply(commands).map_err(rejected)?;
        self.align(name).map_err(rejected)?;
        let waiting = (self.starting.values()).any(|t| *t == Target::Scene(name.into()));
        if let (Some(command), None, false) = (app, window, waiting) {
            self.launch_scene(name, &command)
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        Ok(())
    }

    /// The keyboard for a scene that the user shows: to its window, or to
    /// the node of a page, so that the plain keys of the page arrive. An
    /// open chat keeps the keyboard.
    pub(crate) fn keyboard_to(&mut self, name: &str, window: Option<String>) -> Vec<Command> {
        self.focused.clone_from(&window);
        if self.chat == Chat::Open {
            return Vec::new();
        }
        let plain = self.page_keys == keys::PageKeys::Plain;
        let page = (plain && window.is_none()).then(|| Command::FocusNode {
            node: Some(page_id(name)),
        });
        let focus = Command::Focus { surface: window };
        std::iter::once(focus).chain(page).collect()
    }

    /// The keyboard for the scene that shows, after the chat closed: the
    /// focus that `keyboard_to` held back while the chat had the keyboard,
    /// or the docked app that had it.
    pub(crate) fn keyboard_back(&mut self) -> Vec<Command> {
        let Some(name) = self.current.clone() else {
            return Vec::new();
        };
        let docked = (self.focused.clone()).filter(|f| self.docks.values().any(|d| d == f));
        let window = docked.or_else(|| self.windows.get(&name).cloned());
        self.keyboard_to(&name, window)
    }

    /// Makes `name` the current scene and puts the one before on the trail.
    pub(crate) fn step_to(&mut self, name: &str) {
        if self.current.as_deref() == Some(name) {
            return;
        }
        self.previous = self.current.replace(name.into());
        self.trail.extend(self.previous.clone());
        let extra = self.trail.len().saturating_sub(TRAIL);
        self.trail.drain(..extra);
    }

    /// Starts the app of a scene, whose window then fills its text block.
    /// A command that names a content of kind app runs that content.
    pub(crate) fn launch_scene(&mut self, name: &str, command: &str) -> Result<(), ClientError> {
        let config = &self.settings.value;
        let command = match config.contents.get(command) {
            Some(app) if app.kind == Kind::App => region::command(&app.command, config.colors()),
            _ => command.into(),
        };
        let mint = self.rt.conn.activation_token();
        let Some((token, pid)) = launch(&command, mint.as_deref()) else {
            return Ok(());
        };
        self.starting.insert(token, Target::Scene(name.into()));
        self.pids.insert(name.into(), pid);
        self.launched.push((name.into(), pid));
        self.apply(vec![wait()])
    }

    /// An app of a scene that ended without a window leaves its scene
    /// empty: the scene before it shows again (W11), and the scene of
    /// `app.open` goes.
    /// The next show of the scene starts the app again.
    /// The scene of app.open goes with its app, also of one such as
    /// ls that never had a window.
    pub(crate) fn look_at_launched(&mut self) -> Result<(), ClientError> {
        let windows = &self.windows;
        self.launched
            .retain(|(name, _)| !windows.contains_key(name));
        let (ended, running) = std::mem::take(&mut self.launched)
            .into_iter()
            .partition(|(_, pid)| !pid.is_some_and(chora_kit::alive));
        self.launched = running;
        let ended: Vec<(String, Option<u32>)> = ended;
        for (name, _) in ended {
            let target = Target::Scene(name.clone());
            self.starting.retain(|_, t| *t != target);
            let starting = &self.starting;
            self.quiet.retain(|t| starting.contains_key(t));
            if self.windows.contains_key(&name) {
                continue;
            }
            if self.transient.contains(&name) {
                self.remove_scene(&name)?;
            } else {
                self.leave(&name, true);
            }
        }
        if self.launched.is_empty() {
            return Ok(());
        }
        self.apply(vec![wait()])
    }

    /// The scene of `app.open`, named after the program.
    /// The scene of the same command again, or the first free name of
    /// the program: foot, foot-2, foot-3 ...
    /// The group with the head, which the build of a scene has too.
    pub(crate) fn app_scene(&mut self, command: &str) -> Result<String, RpcError> {
        let word = command
            .split_whitespace()
            .next()
            .ok_or_else(|| invalid("an empty command".into()))?;
        let base = name_of(word);
        let mut name = base.clone();
        for n in 2.. {
            match self.scenes.get(&name) {
                Some(scene) if scene.app.as_deref() == Some(command) => return Ok(name),
                Some(_) => name = format!("{base}-{n}"),
                None => break,
            }
        }
        self.add_scene(&name, Scene::app(command));
        self.transient.insert(name.clone());
        let mut commands = self.scene_commands(&name);
        commands.push(regions_on_top());
        self.apply(commands).map_err(rejected)?;
        self.renumber().map_err(rejected)?;
        Ok(name)
    }
}
