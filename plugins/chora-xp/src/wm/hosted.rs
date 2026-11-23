//! The faces of a cube that another client hosts (D71, `chora_kit::cube`):
//! when all outputs of `chora-xp` are virtual ones of a client that
//! exports the cube role, the pager of each face turns that cube, and the
//! client holds the keys and the drag of the turn. When the host goes,
//! `chora-xp` goes flat on the screens with the windows of face `i` on
//! desktop `i`; when the host lists the same faces again, the windows go
//! back to their faces. Without a host there is no desk switch.

use chora_kit::cube;
use chora_scene::{Command, OutputInfo};

use super::{Bar, EVENTS, Wm, desk_id};
use crate::decoration;

/// The client that hosts the faces, and the virtual outputs of its faces
/// in turn order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Host {
    pub client: String,
    pub faces: Vec<String>,
}

/// The bindings of the cube that `chora-xp` holds: all with its own cube
/// on a screen, none on virtual outputs without a host and none when a
/// host turns the cube.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CubeKeys {
    Own,
    Flat,
    Hosted,
}

impl Host {
    /// The host of `outputs`: the exporter of the cube role, when its
    /// faces hold every output.
    fn of(listing: &cube::Listing, client: &str, outputs: &[&str]) -> Option<Self> {
        let faces: Vec<String> = listing.faces.iter().map(|f| f.output.clone()).collect();
        let held = !outputs.is_empty() && outputs.iter().all(|o| faces.iter().any(|f| f == o));
        held.then(|| Self {
            client: client.to_owned(),
            faces,
        })
    }

    /// The face of an output.
    pub fn face(&self, output: &str) -> Option<usize> {
        self.faces.iter().position(|f| f == output)
    }
}

impl Wm {
    /// The bindings of the cube that the outputs and the host ask for.
    pub(super) fn wanted_keys(&self) -> CubeKeys {
        match (self.cube_off(), &self.host) {
            (false, _) => CubeKeys::Own,
            (true, Some(_)) => CubeKeys::Hosted,
            (true, None) => CubeKeys::Flat,
        }
    }

    /// Looks for the host of the faces after the outputs changed, and
    /// hears its event, which tells when it came back after it went. A
    /// host that removes its faces one by one stays the host until the
    /// last one went. The face at the front is the listing's, also when
    /// xp starts again on a cube that turned before (rcf-4).
    pub(super) fn find_host(&mut self) {
        let outputs: Vec<String> = self.bars.iter().map(|b| b.output.clone()).collect();
        let outputs: Vec<&str> = outputs.iter().map(String::as_str).collect();
        let found = if self.cube_off() {
            self.listing()
                .and_then(|(client, l)| Host::of(&l, &client, &outputs).map(|h| (h, l.front)))
        } else {
            None
        };
        let closing = found.is_none()
            && (self.host.as_ref()).is_some_and(|h| outputs.iter().all(|o| h.face(o).is_some()));
        if !closing && (found.is_some() || self.left.is_none()) {
            if let Some((_, front)) = &found {
                self.front = *front;
            }
            self.host = found.map(|(h, _)| h);
        }
        let heard = self.host.as_ref().or(self.left.as_ref());
        let mut events: Vec<String> = EVENTS.iter().map(|e| (*e).to_owned()).collect();
        events.extend(heard.map(|h| cube::event(&h.client)));
        if let Err(err) = self.rt.subscribe(&events) {
            warn_subscribe(&err);
        }
    }

    /// The exporter of the cube role and its listing.
    fn listing(&mut self) -> Option<(String, cube::Listing)> {
        let client = cube::exporter(&mut self.rt.conn).ok().flatten()?;
        let listing = cube::list(&mut self.rt.conn, &client).ok()?;
        Some((client, listing))
    }

    /// The desktop that the pager of a taskbar lights: the face of its
    /// output on a hosted cube, else the desktop that shows.
    pub(super) fn desk_of(&self, bar: &Bar) -> usize {
        (self.host.as_ref())
            .and_then(|h| h.face(&bar.output))
            .unwrap_or(self.desktop)
    }

    /// A desk chosen in a pager: the host turns its cube to the face; else
    /// `chora-xp` turns its own cube or switches flat.
    pub(super) fn go_to_desk(&mut self, desktop: usize) -> Vec<Command> {
        if self.host.is_none() && self.cube_off() {
            self.find_host();
        }
        let Some(host) = self.host.clone() else {
            return self.turn_to(desktop);
        };
        let turn = cube::Turn {
            to: Some(desktop),
            ..cube::Turn::default()
        };
        if let Err(err) = cube::turn(&mut self.rt.conn, &host.client, turn) {
            warn_turn(&host.client, &err);
        }
        Vec::new()
    }

    /// Whether `event` is the event of the cube of the host of the faces.
    pub(super) fn is_host_event(&self, event: &str) -> bool {
        (self.host.as_ref()).is_some_and(|h| event == cube::event(&h.client))
    }

    /// Whether the window `surface` lies on the face at the front of the
    /// cube of the host; every window does without a host.
    pub(super) fn on_front(&self, surface: &str) -> bool {
        let front = (self.host.as_ref()).and_then(|h| h.faces.get(self.front));
        front.is_none_or(|output| self.output_of(surface).as_ref() == Some(output))
    }

    /// The host turned its cube (`NAME.cube`). At rest on a face, the
    /// Start menu and a context menu of another face shut (rcf-3). Keys
    /// that xp holds on a window of another face, which does not show,
    /// or on its own nodes, go to the topmost window of the front face,
    /// else to its icons (co-50); keys on the icons go to the topmost
    /// window of the face. Keys that another client holds stay there.
    pub(super) fn host_turned(&mut self, turned: cube::Turned) -> Vec<Command> {
        if turned.turning {
            return Vec::new();
        }
        self.front = turned.front;
        let mut commands = self.shut_menus_off_front();
        if self.own_node() && (self.menu.is_open() || self.context.is_some()) {
            return commands;
        }
        let window = (self.focused.clone())
            .filter(|s| !self.node_away() && self.core_focus.as_deref() == Some(s));
        if window.is_none() && !self.own_node() {
            return commands;
        }
        if window.as_deref().is_some_and(|s| self.on_front(s)) {
            return commands;
        }
        let next = self.topmost_except("");
        commands.extend(self.focus(next.as_deref()));
        commands
    }

    /// Shuts the Start menu and a context menu that are open on a face out
    /// of sight: the keys that they hold would go there.
    fn shut_menus_off_front(&mut self) -> Vec<Command> {
        let front = (self.host.as_ref()).and_then(|h| h.faces.get(self.front).cloned());
        let Some(front) = front else {
            return Vec::new();
        };
        let mut commands = Vec::new();
        if self.menu.output().is_some_and(|o| o != front) {
            commands.extend(self.close_menu());
        }
        let context = (self.context.as_ref())
            .and_then(|(_, area)| self.bar_at(*area))
            .map(|b| b.output.clone());
        if context.is_some_and(|o| o != front) {
            commands.extend(self.close_context());
        }
        commands
    }

    /// A client went: when it hosted the faces, `chora-xp` goes flat on
    /// the screens, and the windows of face `i` go to desktop `i`.
    pub(super) fn host_gone(&mut self, name: &str) -> Vec<Command> {
        if self.host.as_ref().is_none_or(|h| h.client != name) {
            return Vec::new();
        }
        let Some(host) = self.host.take() else {
            return Vec::new();
        };
        let screens: Vec<OutputInfo> = (self.rt.all_outputs().unwrap_or_default().into_iter())
            .filter(|o| o.is_virtual != Some(true))
            .collect();
        if screens.is_empty() {
            self.host = Some(host);
            return Vec::new();
        }
        self.rt
            .set_outputs(screens.iter().map(|o| o.name.clone()).collect());
        let mut commands = Vec::new();
        for screen in &screens {
            commands.extend(self.output_changed(screen));
        }
        let screen = screens[0].name.clone();
        for (face, output) in host.faces.iter().enumerate() {
            commands.extend(self.leave_output(output, Some((&screen, face))));
        }
        let shown = (self.focused.as_deref())
            .and_then(|s| self.index(s))
            .map_or(0, |i| self.windows[i].desktop);
        self.left = Some(host);
        commands.extend(self.after_move(shown));
        commands
    }

    /// The event of the cube of a host that went: when its faces are the
    /// same again, the windows go back to them.
    pub(super) fn host_back(&mut self, event: &str) -> Vec<Command> {
        let Some(left) = self.left.clone() else {
            return Vec::new();
        };
        if event != cube::event(&left.client) {
            return Vec::new();
        }
        let listed = cube::list(&mut self.rt.conn, &left.client).ok();
        let front = listed.as_ref().map_or(0, |l| l.front);
        let faces: Option<Vec<String>> =
            listed.map(|l| l.faces.into_iter().map(|f| f.output).collect());
        if faces.as_ref() != Some(&left.faces) {
            return Vec::new();
        }
        let outputs = self.rt.all_outputs().unwrap_or_default();
        let face_infos: Vec<OutputInfo> = (left.faces.iter())
            .filter_map(|f| outputs.iter().find(|o| &o.name == f).cloned())
            .collect();
        if face_infos.len() != left.faces.len() {
            return Vec::new();
        }
        let screens: Vec<String> = self.bars.iter().map(|b| b.output.clone()).collect();
        self.rt.set_outputs(left.faces.clone());
        let mut commands = Vec::new();
        for face in &face_infos {
            commands.extend(self.output_changed(face));
        }
        for i in 0..self.windows.len() {
            let face = self.windows[i].desktop.min(left.faces.len() - 1);
            commands.extend(self.carry_to(i, &left.faces[face]));
        }
        for screen in &screens {
            commands.extend(self.leave_output(screen, None));
        }
        self.left = None;
        self.host = Some(left);
        self.front = front;
        commands.extend(self.after_move(0));
        commands
    }

    /// Moves window `i` to the work area of `output`, on desktop 0.
    fn carry_to(&mut self, i: usize, output: &str) -> Vec<Command> {
        let from = self.bar_at(self.windows[i].rect()).map(Bar::usable);
        let to = self
            .bars
            .iter()
            .find(|b| b.output == output)
            .map(Bar::usable);
        let mut commands = Vec::new();
        if let (Some(from), Some(to)) = (from, to) {
            commands.extend(self.move_window(i, from, to));
        }
        commands.extend(self.put_on_desk(i, 0));
        commands
    }

    /// Puts window `i` on a desktop.
    fn put_on_desk(&mut self, i: usize, desktop: usize) -> Vec<Command> {
        self.windows[i].desktop = desktop;
        vec![Command::Move {
            id: decoration::group_id(&self.windows[i].surface),
            parent: Some(desk_id(desktop)),
            index: None,
        }]
    }

    /// Takes the taskbar of an output away and moves its windows to the
    /// first other taskbar, or to the taskbar of the output `to` onto its
    /// desktop when it is given.
    fn leave_output(&mut self, output: &str, to: Option<(&str, usize)>) -> Vec<Command> {
        let Some(at) = self.bars.iter().position(|b| b.output == output) else {
            return Vec::new();
        };
        let bar = self.bars.remove(at);
        let mut commands = vec![
            bar.remove(),
            Command::Remove {
                id: super::wallpaper::id(output),
                to: None,
            },
        ];
        let target = match to {
            Some((to, _)) => self.bars.iter().find(|b| b.output == to),
            None => self.bars.first(),
        };
        let Some(target) = target.map(Bar::usable) else {
            return commands;
        };
        let from = bar.usable();
        let moved: Vec<usize> = (0..self.windows.len())
            .filter(|&i| bar.contains(self.windows[i].pos))
            .collect();
        for i in moved {
            commands.extend(self.move_window(i, from, target));
            commands.extend(to.map(|(_, d)| self.put_on_desk(i, d)).unwrap_or_default());
        }
        commands
    }

    /// The scene after the windows moved between faces and screens: the
    /// desktop `shown` shows, the grabs, the claim and the taskbars follow.
    fn after_move(&mut self, shown: usize) -> Vec<Command> {
        let mut commands = self.show_desktop(shown);
        commands.extend(self.claim());
        self.sync_cube_keys();
        self.find_host();
        self.reserve_bars();
        commands.extend(self.pagers());
        commands
    }
}

fn warn_subscribe(err: &dyn std::fmt::Display) {
    tracing::warn!("events.subscribe: {err}");
}

fn warn_turn(client: &str, err: &dyn std::fmt::Display) {
    tracing::warn!("{client} {}: {err}", cube::TURN);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::Window;

    /// A virtual output of a face, far from the screen as the construct
    /// places them.
    fn face_output(name: &str, i: i32) -> OutputInfo {
        OutputInfo {
            name: name.into(),
            position: [-262_144 * (i + 1), -262_144],
            size: [1280, 800],
            scale: 1.0,
            camera: chora_scene::Transform::default(),
            work_area: [-262_144 * (i + 1), -262_144, 1280, 800],
            default_work_area: [-262_144 * (i + 1), -262_144, 1280, 800],
            is_virtual: Some(true),
            owner: Some("construct".into()),
        }
    }

    /// The host goes: the window of face `i` lands on desktop `i` of the
    /// screen, and the desktop of the focused window shows.
    #[test]
    fn the_window_of_face_i_goes_to_desktop_i_when_the_host_goes() {
        let (mut wm, _methods) = super::super::tests::wm_on_fake_core("host-gone");
        let faces: Vec<String> = (1..=4).map(|i| format!("V-{i}")).collect();
        for (i, face) in (0..).zip(&faces) {
            let _ = wm.output_changed(&face_output(face, i));
        }
        let _ = wm.leave_output("X", None);
        for (i, face) in faces.iter().enumerate() {
            let usable = wm.bars.iter().find(|b| &b.output == face).unwrap().usable();
            let mut w = Window::new(&format!("s{i}"), None, None);
            w.size = Some([200, 100]);
            w.shown = true;
            w.pos = [usable[0] + 40, usable[1] + 40];
            wm.windows.push(w);
        }
        wm.focused = Some("s0".into());
        wm.host = Some(Host {
            client: "construct".into(),
            faces,
        });
        let _ = wm.host_gone("construct");
        let desks: Vec<usize> = wm.windows.iter().map(|w| w.desktop).collect();
        assert_eq!(desks, [0, 1, 2, 3]);
        assert_eq!(wm.desktop, 0, "the desktop of the focused window shows");
        let on_screen = |w: &Window| wm.bars[0].contains(w.pos);
        assert!(wm.windows.iter().all(on_screen));
    }

    fn listing(faces: &[&str]) -> cube::Listing {
        cube::Listing {
            faces: (faces.iter())
                .map(|o| cube::Face {
                    output: (*o).into(),
                    manager: "xp".into(),
                    health: "up".into(),
                })
                .collect(),
            front: 0,
            turning: false,
            mode: "use".into(),
        }
    }

    /// The host closes its cube: its faces go one by one, and xp ends
    /// with the last one, also when the host listed no faces between.
    #[test]
    fn xp_ends_when_the_host_removes_the_last_face() {
        let (mut wm, _methods) = super::super::tests::wm_on_fake_core("host-closes");
        let faces: Vec<String> = (1..=4).map(|i| format!("V-{i}")).collect();
        for (i, face) in (0..).zip(&faces) {
            let _ = wm.output_changed(&face_output(face, i));
        }
        let _ = wm.leave_output("X", None);
        wm.host = Some(Host {
            client: "construct".into(),
            faces: faces.clone(),
        });
        for face in &faces {
            assert!(!wm.done, "xp ends only with the last face");
            let _ = wm.output_removed(face);
        }
        assert!(wm.done);
    }

    /// xp on the four faces of a construct, with the window `s0` on face 0
    /// and `s2` on face 2, `s0` focused, as the core says.
    fn hosted(tag: &str) -> Wm {
        let (mut wm, _methods) = super::super::tests::wm_on_fake_core(tag);
        let faces: Vec<String> = (1..=4).map(|i| format!("V-{i}")).collect();
        for (i, face) in (0..).zip(&faces) {
            let _ = wm.output_changed(&face_output(face, i));
        }
        let _ = wm.leave_output("X", None);
        for (i, face) in [(0, &faces[0]), (2, &faces[2])] {
            let usable = wm.bars.iter().find(|b| &b.output == face).unwrap().usable();
            let mut w = Window::new(&format!("s{i}"), None, None);
            w.size = Some([200, 100]);
            w.shown = true;
            w.pos = [usable[0] + 40, usable[1] + 40];
            wm.windows.push(w);
        }
        wm.host = Some(Host {
            client: "construct".into(),
            faces,
        });
        wm.focused = Some("s0".into());
        wm.core_focus = Some("s0".into());
        wm
    }

    fn turned(front: usize) -> cube::Turned {
        cube::Turned {
            front,
            turning: false,
        }
    }

    fn icons() -> Command {
        Command::FocusNode {
            node: Some(crate::desktop_icons::GROUP.into()),
        }
    }

    /// co-50: the host turns its cube away from the face of the focused
    /// window. That window shows no more, so it must not keep the keys:
    /// they go to the icons of the front face, which has no window, and to
    /// `s2` once the cube is at its face.
    #[test]
    fn a_turn_of_the_host_takes_the_keys_to_the_front_face() {
        let mut wm = hosted("host-turn-keys");
        let commands = wm.host_turned(turned(1));
        assert_eq!(
            wm.focused, None,
            "s0 of face 0 keeps the keys: {commands:?}"
        );
        assert!(
            commands.contains(&chora_kit::wm::focus::release()),
            "{commands:?}"
        );
        assert!(
            commands.contains(&icons()),
            "the icons get no keys: {commands:?}"
        );
        wm.core_focus = None;
        wm.key_node = Some(super::super::NAME.into());
        let commands = wm.host_turned(turned(2));
        assert_eq!(wm.focused.as_deref(), Some("s2"), "{commands:?}");
        assert!(
            commands.contains(&chora_kit::wm::focus::give("s2")),
            "{commands:?}"
        );
    }

    /// A turn while another client holds the keys, as the walk of the
    /// construct in navigate mode, leaves them there.
    #[test]
    fn a_turn_of_the_host_leaves_the_keys_of_another_client() {
        let mut wm = hosted("host-turn-other");
        wm.focused = None;
        wm.core_focus = None;
        wm.key_node = Some("construct".into());
        assert_eq!(wm.host_turned(turned(2)), Vec::new());
        assert_eq!(wm.focused, None);
    }

    /// co-51: the focused window of the front face closes (the window of
    /// the lock). The core first focuses nothing and then destroys it: the
    /// keys go to the icons of the front face, never to nobody and never
    /// to `s0` of a face that does not show.
    #[test]
    fn the_keys_stay_on_the_front_face_when_its_focused_window_closes() {
        let mut wm = hosted("host-close-keys");
        let _ = wm.host_turned(turned(1));
        let usable = wm.bars.iter().find(|b| b.output == "V-2").unwrap().usable();
        let mut lock = Window::new("s9", None, None);
        lock.size = Some([200, 100]);
        lock.shown = true;
        lock.pos = [usable[0] + 40, usable[1] + 40];
        wm.windows.push(lock);
        wm.focused = Some("s9".into());
        wm.core_focus = Some("s9".into());
        let _ = wm.focus_moved(None);
        let commands = wm.forget("s9");
        assert_eq!(wm.focused, None, "{commands:?}");
        assert!(
            !commands.contains(&chora_kit::wm::focus::give("s0")),
            "{commands:?}"
        );
        assert!(
            commands.contains(&icons()),
            "the keys go nowhere: {commands:?}"
        );
    }

    /// rcf-3: the Start menu is open on face 0 and holds the keys when the
    /// host turns to face 2: the menu shuts, out of sight it takes no
    /// keys, and the keys go to `s2` of the front face.
    #[test]
    fn a_turn_at_rest_shuts_the_start_menu_of_a_face_out_of_sight() {
        let mut wm = hosted("host-turn-menu");
        let _ = wm.press_taskbar(crate::taskbar::Part::Start("V-1".into()));
        assert_eq!(wm.menu.output(), Some("V-1"), "the menu opens on face 0");
        wm.key_node = Some(super::super::NAME.into());
        let commands = wm.host_turned(turned(2));
        assert_eq!(wm.menu.output(), None, "the menu stays open: {commands:?}");
        assert!(
            commands.contains(&chora_kit::wm::focus::give("s2")),
            "{commands:?}"
        );
    }

    /// rcf-3: the Start menu that the icons opened on face 0 holds the
    /// keys when the host turns to face 1, which has no window: the menu
    /// shuts and the keys go to the icons, not to the menu out of sight.
    #[test]
    fn a_turn_at_rest_to_a_face_without_windows_takes_the_keys_off_the_menu() {
        let mut wm = hosted("host-turn-menu-icons");
        wm.focused = None;
        wm.core_focus = None;
        let _ = wm.press_taskbar(crate::taskbar::Part::Start("V-1".into()));
        wm.key_node = Some(super::super::NAME.into());
        let commands = wm.host_turned(turned(1));
        assert_eq!(wm.menu.output(), None, "the menu stays open: {commands:?}");
        assert!(
            commands.contains(&icons()),
            "the icons get no keys: {commands:?}"
        );
    }

    /// A cube event at rest on the face of an open Start menu leaves the
    /// menu open with the keys.
    #[test]
    fn a_turn_at_rest_keeps_the_start_menu_of_the_front_face() {
        let mut wm = hosted("host-turn-menu-front");
        let _ = wm.press_taskbar(crate::taskbar::Part::Start("V-2".into()));
        wm.key_node = Some(super::super::NAME.into());
        let commands = wm.host_turned(turned(1));
        assert_eq!(wm.menu.output(), Some("V-2"), "{commands:?}");
        assert!(
            !commands
                .iter()
                .any(|c| matches!(c, Command::FocusNode { .. })),
            "{commands:?}"
        );
    }

    /// A fake core with the four faces of a construct whose front is
    /// `front`, as `cube.list` tells, and with `screen` besides them.
    fn construct_core(front: usize, screen: bool) -> chora_kit::testing::FakeCore {
        let core = chora_kit::testing::FakeCore::start();
        let mut outputs: Vec<OutputInfo> = (0..4)
            .map(|i| face_output(&format!("V-{}", i + 1), i))
            .collect();
        if screen {
            let mut x = face_output("X", 0);
            x.position = [0, 0];
            x.work_area = [0, 0, 1280, 800];
            x.default_work_area = [0, 0, 1280, 800];
            x.is_virtual = None;
            x.owner = None;
            outputs.insert(0, x);
        }
        core.outputs(serde_json::json!(outputs));
        let mut faces = listing(&["V-1", "V-2", "V-3", "V-4"]);
        faces.front = front;
        let faces = serde_json::to_value(faces).unwrap();
        core.respond(move |method, params| match method {
            "methods.list" => Some(serde_json::json!([
                {"name": cube::LIST, "client": "construct"}
            ])),
            "methods.call" if params["method"] == cube::LIST => Some(faces.clone()),
            _ => None,
        });
        core
    }

    /// A window `surface` on the work area of `output`.
    fn window_on(wm: &mut Wm, surface: &str, output: &str, desktop: usize) {
        let usable = wm
            .bars
            .iter()
            .find(|b| b.output == output)
            .unwrap()
            .usable();
        let mut w = Window::new(surface, None, None);
        w.size = Some([200, 100]);
        w.shown = true;
        w.desktop = desktop;
        w.pos = [usable[0] + 40, usable[1] + 40];
        wm.windows.push(w);
    }

    /// rcf-4: the construct starts xp again on its faces (its retry) while
    /// face 1 is at the front. xp reads the front from the listing: when
    /// the focused window of face 1 closes, the keys go to the icons, not
    /// to `s0` of face 0, which does not show.
    #[test]
    fn xp_started_on_a_hosted_cube_reads_its_front_from_the_listing() {
        let core = construct_core(1, false);
        let mut wm = super::super::tests::wm_on_kit_core(&core, "host-retry-front");
        assert!(wm.host.is_some(), "xp finds no host");
        assert_eq!(wm.front, 1, "the front is not the listing's");
        window_on(&mut wm, "s1", "V-2", 0);
        window_on(&mut wm, "s0", "V-1", 0);
        wm.focused = Some("s1".into());
        wm.core_focus = Some("s1".into());
        let commands = wm.forget("s1");
        assert!(
            !commands.contains(&chora_kit::wm::focus::give("s0")),
            "{commands:?}"
        );
    }

    /// rcf-4: the host comes back with face 1 at the front: xp reads the
    /// front from the listing, and a close of the focused window of face
    /// 1 gives no keys to `s0` of face 0.
    #[test]
    fn a_host_that_comes_back_gives_xp_its_front() {
        let core = construct_core(1, true);
        let mut wm = super::super::tests::wm_on_kit_core(&core, "host-back-front");
        for face in ["V-1", "V-2", "V-3", "V-4"] {
            let _ = wm.leave_output(face, None);
        }
        wm.host = None;
        wm.front = 0;
        wm.left = Some(Host {
            client: "construct".into(),
            faces: (1..=4).map(|i| format!("V-{i}")).collect(),
        });
        window_on(&mut wm, "s1", "X", 1);
        window_on(&mut wm, "s0", "X", 0);
        let _ = wm.host_back(&cube::event("construct"));
        assert!(wm.host.is_some(), "the host is not back");
        assert_eq!(wm.front, 1, "the front is not the listing's");
        assert_eq!(wm.output_of("s1").as_deref(), Some("V-2"));
        wm.focused = Some("s1".into());
        wm.core_focus = Some("s1".into());
        let commands = wm.forget("s1");
        assert!(
            !commands.contains(&chora_kit::wm::focus::give("s0")),
            "{commands:?}"
        );
    }

    #[test]
    fn a_host_lists_every_output_as_a_face() {
        let faces = listing(&["V-1", "V-2", "V-3", "V-4"]);
        let host = Host::of(&faces, "construct", &["V-2", "V-1"]).expect("a host");
        assert_eq!(host.face("V-3"), Some(2));
        assert_eq!(host.client, "construct");
        assert!(Host::of(&faces, "construct", &["V-1", "HEADLESS-1"]).is_none());
        assert!(Host::of(&faces, "construct", &[]).is_none());
    }
}
