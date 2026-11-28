//! Starting the window manager: connecting, adopting the windows, creating the scene and the outputs.

use chora_kit::wm::Created;

use super::{
    BARS, Bar, ClientError, Command, Config, DESKTOP, DESKTOPS, DIALOGS, DesktopIcons, EVENTS,
    Effects, ExportMethod, HashMap, Icons, Instant, KeyAction, Launches, MENUS, METHODS, Menu,
    Mixer, NAME, NodeKind, OutputInfo, PropsUpdate, ROOT, RpcError, Runtime, Sound, Theme, Tooltip,
    WALLPAPERS, WINDOWS, Wm, apps, decoration, desk_id, desktop_icons, float, give, grab_keys,
    hosted::CubeKeys, max_scale, move_method, settings, wallpaper, window, without_reserve,
};
use chora_scene::Order;

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
        rt: Runtime<KeyAction>,
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
            dropped: None,
            front: 0,
            core_focus: None,
            key_node: None,
            heard: chora_kit::wm::focus::Focus::default(),
            arrival: super::Arrival::Settled,
            bars: Vec::new(),
            menu,
            drag: None,
            resize: None,
            last_title: None,
            places: window::Places::default(),
            desktop: 0,
            hover: None,
            pointer_output: None,
            cube_keys: CubeKeys::Own,
            host: None,
            left: None,
            floats: float::Floats::default(),
            dialog: None,
            open: chora_kit::wm::open::Open::default(),
            fade: None,
            fade_shader: None,
            context: None,
            sys_window: None,
            looks: Vec::new(),
            looks_failed: false,
            last_icon: None,
            panels: Vec::new(),
            hide_at: None,
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
            book: chora_kit::wm::Book::new(NAME, false).with_groups(decoration::GROUP),
            launches: Launches::new(),
            notepad,
            volume: None,
            voiced: chora_kit::helper::Helper::default(),
            mixer: Mixer::new(NAME),
            volume_read: Instant::now(),
        };
        wm.clock = wm.now();
        wm.voiced.ask(&mut wm.rt.conn);
        let old_root = wm.take_kept()?;
        wm.mixer.refresh();
        wm.rt.announce(&EVENTS, methods)?;
        wm.grab()?;
        wm.create_scene(old_root)?;
        wm.refresh_looks();
        wm.adopt_all()?;
        wm.catch_up_outputs()?;
        wm.sync_cube_keys();
        let pagers = wm.pagers();
        wm.apply(pagers)?;
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
    /// person did nothing here yet. So it does when xp joins a session
    /// where another look holds a claim (ti-38): the person is in that
    /// look. The desktop then shows as not focused and the system key
    /// stays free.
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
            let kept_at = self.kept.get(&node.id).map(|k| k.opened);
            let info = node.surface.clone().unwrap_or_default();
            let facts = Created {
                surface: node.id.clone(),
                app_id: info.app_id,
                title: info.title,
                to: node.to.clone(),
                pid: None,
                token: None,
                parent: None,
            };
            let changes = self.book.claim(&node.id, node.to.as_deref(), None);
            self.settle(changes, Some(&facts))?;
            if let Some(place) = kept_at.filter(|_| self.book.has(&node.id)) {
                self.book.keep(&node.id);
                opened.push((place, node.id));
            }
        }
        opened.sort_by_key(|(place, _)| *place);
        let kept: Vec<String> = opened.into_iter().map(|(_, id)| id).collect();
        self.opened.retain(|s| !kept.contains(s));
        self.opened.splice(0..0, kept);
        let top = self.topmost_except("");
        if self.opened.is_empty() && !self.cube_off() && self.another_look()? {
            self.arrival = super::Arrival::Joined;
        }
        let mut commands = if self.opened.is_empty() && (self.cube_off() || self.joined()) {
            self.grab_system_key(false);
            self.desk.focus(&mut self.theme, &mut self.icons, false)
        } else {
            self.focus(top.as_deref())
        };
        commands.extend(self.sync_tasks());
        self.apply(commands)
    }

    /// Whether xp joined beside another look and has had no keys since.
    pub(super) fn joined(&self) -> bool {
        self.arrival == super::Arrival::Joined
    }

    /// Whether the smallest claim at a point of the scene is one of another
    /// look, as the core chooses a look for a press (D55).
    pub(super) fn claim_of_another_look(&mut self, point: [f64; 2]) -> bool {
        let Ok(groups) = self.rt.conn.query("kind:group") else {
            return false;
        };
        chora_scene::claimant_at(&groups, point, |_| true).is_some_and(|owner| owner != NAME)
    }

    /// A client went: when xp joined beside another look and no look but xp
    /// holds a claim now, xp settles and takes the keyboard (ti-43,
    /// api.md "Window managers").
    pub(super) fn look_gone(&mut self) -> Vec<Command> {
        if !self.joined() || self.another_look().unwrap_or(true) {
            return Vec::new();
        }
        self.arrival = super::Arrival::Settled;
        let mut commands = self.focus(self.topmost_except("").as_deref());
        commands.push(self.desktop_keys());
        commands
    }

    /// Whether a look other than xp holds a claim.
    fn another_look(&mut self) -> Result<bool, ClientError> {
        let groups = self.rt.conn.query("kind:group")?;
        Ok((groups.iter())
            .any(|n| n.props.claim.is_some() && n.owner.as_deref().is_some_and(|o| o != NAME)))
    }

    /// The two roots of the plugin and their groups: the desktop in the
    /// background order (D89), and the windows, the taskbars, the dialogs
    /// and the menus in the root that holds the claim.
    fn roots(&self) -> Vec<Command> {
        let group = |id: &str, parent: &str| Command::Create {
            id: id.into(),
            kind: NodeKind::Group,
            parent: parent.into(),
            props: PropsUpdate::default(),
        };
        let mut commands = vec![
            group(NAME, ROOT),
            Command::Create {
                id: DESKTOP.into(),
                kind: NodeKind::Group,
                parent: ROOT.into(),
                props: PropsUpdate {
                    order: Some(Order::Background),
                    ..PropsUpdate::default()
                },
            },
        ];
        for id in [WALLPAPERS, desktop_icons::GROUP] {
            commands.push(group(id, DESKTOP));
        }
        let mut ids = [WINDOWS, BARS, DIALOGS, MENUS];
        if !self.taskbar.on_top {
            ids.swap(0, 1);
        }
        for id in ids {
            commands.push(group(id, NAME));
        }
        commands
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
        let mut commands: Vec<Command> = old_root
            .then(|| Command::Remove {
                id: NAME.into(),
                to: None,
            })
            .into_iter()
            .collect();
        commands.extend(self.roots());
        for desktop in 0..DESKTOPS {
            commands.push(Command::Create {
                id: desk_id(desktop),
                kind: NodeKind::Group,
                parent: WINDOWS.into(),
                props: PropsUpdate::default(),
            });
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
        let cube = self.wanted_keys();
        self.cube_keys = cube;
        self.rt.give_keys(Some(|to| {
            KeyAction::System(chora_kit::actions::System::Give(to.into()))
        }));
        grab_keys(&mut self.rt, &self.settings.value.common.bindings, cube)?;
        self.rt
            .accept_injected(self.settings.value.common.test_accept_injected_keys);
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

#[cfg(test)]
mod tests {
    use chora_kit::{runtime::Plugin as _, testing::FakeCore};
    use chora_scene::{Command, Event, NodeKind, PropsUpdate, ROOT};

    use super::super::{Arrival, Wm, tests::wm_on_kit_core};

    /// A look named `mc` that holds a claim, on a core that selects.
    fn core_with_look(look: bool) -> FakeCore {
        let core = FakeCore::start().selecting();
        if look {
            core.connect("mc")
                .apply(vec![Command::Create {
                    id: "mc".into(),
                    kind: NodeKind::Group,
                    parent: ROOT.into(),
                    props: PropsUpdate {
                        claim: Some(Some([900.0, 0.0, 300.0, 300.0])),
                        ..PropsUpdate::default()
                    },
                }])
                .unwrap();
        }
        core
    }

    /// The look `mc` goes: its claim leaves the scene, then the core says so.
    fn look_leaves(core: &FakeCore, wm: &mut Wm) {
        core.connect("mc")
            .apply(vec![Command::Remove {
                id: "mc".into(),
                to: None,
            }])
            .unwrap();
        wm.event(Event::ClientDisconnected { name: "mc".into() })
            .unwrap();
    }

    /// xp subscribes before its first grab (rss-8): the core sends no
    /// event to a connection without a subscription, so a grab refused
    /// before it would never hear `input.ungrabbed`, and a window manager
    /// that joined then would never be heard.
    #[test]
    fn xp_subscribes_before_it_grabs() {
        let core = core_with_look(false);
        let calls = core.record(|method, _| {
            let subscribe = method.starts_with("events.") && method.ends_with("subscribe");
            (subscribe || method == "input.grab").then(|| method.to_owned())
        });
        let _wm = wm_on_kit_core(&core, "subscribes-first");
        let calls = calls.lock().unwrap().clone();
        let subscribe = calls.iter().position(|m| m.starts_with("events."));
        let grab = calls.iter().position(|m| m == "input.grab");
        assert!(grab.is_some(), "xp grabs: {calls:?}");
        assert!(
            subscribe.is_some() && subscribe < grab,
            "rss-8: xp grabs before it subscribes: {:?}",
            &calls[..calls.len().min(3)]
        );
    }

    /// A chord that arrives while the scene stands and no key is held has
    /// no holder, and the core drops it (sh-44: the gates take the nodes of
    /// xp for the start, and Super+Space went nowhere). So xp holds its
    /// keys before its first node shows.
    #[test]
    fn xp_holds_its_keys_before_its_first_node_exists() {
        let core = core_with_look(false);
        let calls = core.record(|method, params| match method {
            "input.grab" => Some("grab".to_owned()),
            "scene.apply" if params.to_string().contains("\"create\"") => Some("create".to_owned()),
            _ => None,
        });
        let _wm = wm_on_kit_core(&core, "grabs-first");
        let calls = calls.lock().unwrap().clone();
        let grab = calls.iter().position(|m| m == "grab");
        let create = calls.iter().position(|m| m == "create");
        assert!(
            grab.is_some() && create.is_some(),
            "xp grabs and builds: {calls:?}"
        );
        assert!(
            grab < create,
            "sh-44: the first node exists before the first key is held: {:?}",
            &calls[..calls.len().min(4)]
        );
    }

    #[test]
    fn arrival_starting_alone_is_settled() {
        let core = core_with_look(false);
        let wm = wm_on_kit_core(&core, "arrival-alone");
        assert_eq!(wm.arrival, Arrival::Settled);
        assert!(!wm.joined());
    }

    #[test]
    fn arrival_joining_a_look_with_a_claim_is_joined() {
        let core = core_with_look(true);
        let wm = wm_on_kit_core(&core, "arrival-join");
        assert_eq!(wm.arrival, Arrival::Joined);
        assert!(wm.joined());
    }

    /// api.md "Window managers": xp holds the keyboard when no other look
    /// holds a claim (ti-43).
    #[test]
    fn arrival_other_look_leaves_settles() {
        let core = core_with_look(true);
        let mut wm = wm_on_kit_core(&core, "arrival-leaves");
        assert_eq!(wm.arrival, Arrival::Joined);
        look_leaves(&core, &mut wm);
        assert_eq!(wm.arrival, Arrival::Settled);
        assert_eq!(wm.state().unwrap()["focus"]["holder"], "xp");
    }

    #[test]
    fn arrival_a_client_that_holds_no_claim_leaving_keeps_joined() {
        let core = core_with_look(true);
        let mut wm = wm_on_kit_core(&core, "arrival-other-client");
        wm.event(Event::ClientDisconnected {
            name: "other".into(),
        })
        .unwrap();
        assert_eq!(wm.arrival, Arrival::Joined);
    }

    #[test]
    fn arrival_click_on_the_free_desktop_settles() {
        let core = core_with_look(true);
        let mut wm = wm_on_kit_core(&core, "arrival-click");
        let inside = wm.press_wallpaper([1000.0, 100.0]);
        assert!(
            inside.is_empty() && wm.joined(),
            "a click in the claim of the look"
        );
        wm.press_wallpaper([5.0, 5.0]);
        assert_eq!(wm.arrival, Arrival::Settled);
    }
}
