//! Starting the window manager: connecting, adopting the windows, creating the scene and the outputs.

use super::{
    BARS, Bar, ClientError, Command, Config, DESKTOPS, DIALOGS, DesktopIcons, EVENTS, Effects,
    ExportMethod, HashMap, Helper, Icons, Instant, Key, Launches, MENUS, METHODS, Menu, Mixer,
    NAME, NodeKind, Notifier, OutputInfo, PropsUpdate, ROOT, RpcError, Runtime, Sound, Theme,
    Tooltip, WALLPAPERS, WINDOWS, Wm, apps, desk_id, desktop_icons, float, give, grab_keys,
    max_scale, move_method, settings, wallpaper, window, without_reserve,
};

impl Wm {
    /// Connects, creates the scene of the plugin and adopts the windows
    /// that have no owner. `outputs` are the names of `--output`.
    pub fn start(
        outputs: Vec<String>,
        theme: Theme,
        settings: settings::File<Config>,
        slow: f64,
    ) -> Result<Self, ClientError> {
        Self::start_on(Runtime::connect(NAME, outputs)?, theme, settings, slow)
    }

    /// As [`Self::start`], on a runtime that is connected already.
    #[allow(clippy::too_many_lines, reason = "one line per field of the plugin")]
    pub(super) fn start_on(
        rt: Runtime<Key>,
        theme: Theme,
        settings: settings::File<Config>,
        slow: f64,
    ) -> Result<Self, ClientError> {
        let mut methods = METHODS.map(ExportMethod::from).to_vec();
        methods.push(give::method());
        methods.push(move_method());
        methods.push(chora_kit::float::method());
        methods.push(chora_kit::runtime::state_method());
        methods.extend(settings::File::<Config>::methods());
        let config = &settings.value;
        let mut icons = Icons::new(crate::icons::THEMES);
        let notepad = apps::system_editors(config.editor.as_deref());
        let menu = Menu::new(
            config,
            apps::load(&config.terminal, config.editor.as_deref()),
            &theme,
            &mut icons,
        );
        let mut wm = Self {
            rt,
            clock: String::new(),
            desk: DesktopIcons::new(&config.terminal),
            effects: Effects::new(config.effects),
            taskbar: config.taskbar,
            reserved: HashMap::new(),
            theme,
            icons,
            settings,
            slow,
            windows: Vec::new(),
            opened: Vec::new(),
            focused: None,
            core_focus: None,
            bars: Vec::new(),
            menu,
            drag: None,
            resize: None,
            last_title: None,
            places: window::Places::default(),
            desktop: 0,
            hover: None,
            pointer_output: None,
            cube_keys: false,
            floats: float::Floats::default(),
            dialog: None,
            fade: None,
            fade_shader: None,
            context: None,
            sys_window: None,
            last_icon: None,
            panels: Vec::new(),
            hide_at: None,
            last_clock: None,
            display_tab: 0,
            item: None,
            done: false,
            restart: false,
            switcher: None,
            switch_escape: None,
            system_key: None,
            tip: Tooltip::default(),
            tap: None,
            hidden: Vec::new(),
            cube_grab: None,
            kept: HashMap::new(),
            launches: Launches::new(),
            notepad,
            notifier: Notifier::new(),
            volume: None,
            helper: Helper::default(),
            mixer: Mixer::new(NAME),
            volume_read: Instant::now(),
        };
        wm.clock = wm.now();
        let old_root = wm.take_kept()?;
        wm.mixer.refresh();
        wm.create_scene(old_root)?;
        wm.grab()?;
        wm.rt.announce(&EVENTS, methods)?;
        wm.adopt_all()?;
        wm.catch_up_outputs()?;
        Ok(wm)
    }

    /// Lays out the outputs that changed before the subscription, which
    /// sent no event for it.
    pub(super) fn catch_up_outputs(&mut self) -> Result<(), ClientError> {
        let stale: Vec<OutputInfo> = (self.outputs().into_iter())
            .filter(|o| self.bars.iter().any(|b| b.is_stale(o)))
            .collect();
        for output in &stale {
            let commands = self.output_changed(output);
            self.apply(commands)?;
        }
        Ok(())
    }

    /// Adopts the surfaces without an owner, as at a start, in their order
    /// before a restart. Only a window of the current desktop gets the
    /// focus. Without a window, on virtual outputs only, the keyboard stays
    /// where it is: the client that shows the outputs has it, and the
    /// person did nothing here yet. The desktop then shows as not focused
    /// and the system key stays free.
    pub(super) fn adopt_all(&mut self) -> Result<(), ClientError> {
        let mut nodes: Vec<chora_scene::Node> = self
            .rt
            .conn
            .query("kind:surface")?
            .into_iter()
            .filter(|n| n.owner.is_none())
            .collect();
        nodes.sort_by_key(|n| self.kept.get(&n.id).map_or(usize::MAX, |k| k.z));
        let mut opened = Vec::new();
        for node in nodes {
            if let Some(k) = self.kept.get(&node.id) {
                opened.push((k.opened, node.id.clone()));
            }
            let info = node.surface.unwrap_or_default();
            self.adopt(&node.id, info.app_id, info.title.as_deref(), None)?;
        }
        opened.sort_by_key(|(place, _)| *place);
        let kept: Vec<String> = opened.into_iter().map(|(_, id)| id).collect();
        self.opened.retain(|s| !kept.contains(s));
        self.opened.splice(0..0, kept);
        let top = self.topmost_except("");
        let mut commands = if self.opened.is_empty() && self.cube_off() {
            self.grab_system_key(false);
            self.desk.focus(&mut self.theme, &mut self.icons, false)
        } else {
            self.focus(top.as_deref())
        };
        commands.extend(self.sync_tasks());
        self.apply(commands)
    }

    /// The groups, the wallpapers, the taskbars and the hidden menu, in
    /// one transaction, and XP's logon sound. It fails if another instance
    /// runs.
    /// Builds the scene. The root group that a restart kept goes in the
    /// same transaction, so that the plugin holds a claim throughout and
    /// the surfaces reserved for it stay so (D55).
    pub(super) fn create_scene(&mut self, old_root: bool) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = (self.rt.outputs()?.into_iter())
            .map(|o| without_reserve(&self.reserved, o))
            .collect();
        self.theme.set_scale(max_scale(&outputs));
        let group = |id: &str, parent: &str| Command::Create {
            id: id.into(),
            kind: NodeKind::Group,
            parent: parent.into(),
            props: PropsUpdate::default(),
        };
        let mut commands: Vec<Command> = old_root
            .then(|| Command::Remove {
                id: NAME.into(),
                to: None,
            })
            .into_iter()
            .collect();
        commands.push(group(NAME, ROOT));
        let mut ids = [
            WALLPAPERS,
            desktop_icons::GROUP,
            WINDOWS,
            BARS,
            DIALOGS,
            MENUS,
        ];
        if !self.taskbar.on_top {
            ids.swap(2, 3);
        }
        for id in ids {
            commands.push(group(id, NAME));
        }
        for desktop in 0..DESKTOPS {
            commands.push(group(&desk_id(desktop), WINDOWS));
            if desktop != self.desktop {
                commands.push(Command::Update {
                    id: desk_id(desktop),
                    props: PropsUpdate {
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                });
            }
        }
        for output in &outputs {
            commands.extend(self.add_output(output));
        }
        commands.extend(self.claim());
        commands.extend(self.menu.create(&mut self.theme, MENUS));
        commands.extend(self.refresh_icons());
        match self.try_apply(commands)? {
            None => {
                self.reserve_bars();
                self.play(Sound::Logon);
                Ok(())
            }
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    pub(super) fn add_output(&mut self, output: &OutputInfo) -> Vec<Command> {
        let mut commands = wallpaper::create(
            &mut self.theme,
            &self.settings.value.wallpaper,
            output,
            WALLPAPERS,
        );
        let mut bar = Bar::new(output);
        bar.configure(self.taskbar);
        let tray = (self.clock.as_str(), self.icons());
        commands.extend(bar.create(&mut self.theme, BARS, tray, self.desktop));
        self.bars.push(bar);
        commands
    }

    pub(super) fn grab(&mut self) -> Result<(), ClientError> {
        let cube = !self.cube_off();
        self.cube_keys = cube;
        grab_keys(&mut self.rt, cube)?;
        self.rt
            .accept_injected(self.settings.value.test_accept_injected_keys);
        self.rt.give_keys(Some(Key::Give));
        self.rt.grab_give_keys()
    }

    /// Applies a transaction, with the logical sizes of the pictures.
    pub(super) fn apply(&mut self, mut commands: Vec<Command>) -> Result<(), ClientError> {
        let floated = self.sync_floats(&mut commands);
        self.theme.size_images(&mut commands);
        self.rt.conn.apply(commands)?;
        self.tell_floats(floated);
        Ok(())
    }

    /// Applies a transaction, as [`Self::apply`], and returns the error of
    /// the core.
    pub(super) fn try_apply(
        &mut self,
        mut commands: Vec<Command>,
    ) -> Result<Option<RpcError>, ClientError> {
        self.theme.size_images(&mut commands);
        self.rt.conn.try_apply(commands)
    }

    /// Renders the shown pictures again when the largest scale of the
    /// outputs changes, so that they stay sharp. Its commands go before
    /// those of the change, so that the new pictures of the change win.
    pub(super) fn rescale(&mut self) -> Result<Vec<Command>, ClientError> {
        let outputs = self.outputs();
        if !self.theme.set_scale(max_scale(&outputs)) {
            return Ok(Vec::new());
        }
        let mut commands = Vec::new();
        for node in self.rt.conn.query(&format!("owner:{NAME} kind:image"))? {
            let props = &node.props;
            let Some((image, size)) = props.image.as_deref().and_then(|p| self.theme.rerender(p))
            else {
                continue;
            };
            commands.push(Command::Update {
                id: node.id,
                props: PropsUpdate {
                    image: Some(image),
                    size: props.size.is_none().then_some(size),
                    ..PropsUpdate::default()
                },
            });
        }
        Ok(commands)
    }
}
