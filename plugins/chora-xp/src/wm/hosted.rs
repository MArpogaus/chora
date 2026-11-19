//! The faces of a cube that another client hosts (D71, `chora_kit::cube`):
//! when all outputs of `chora-xp` are virtual ones of a client that
//! exports the cube role, the pager of each face turns that cube, and the
//! client holds the keys and the drag of the turn. When the host goes,
//! `chora-xp` goes flat on the screens with the windows of face `i` on
//! desktop `i`; when the host lists the same faces again, the windows go
//! back to their faces. Without a host, a desk switch is flat.

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
/// on a screen, the turn keys as flat desk switches without a cube, none
/// when a host turns the cube.
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
    /// hears its event, which tells when it came back after it went.
    pub(super) fn find_host(&mut self) {
        let outputs: Vec<String> = self.bars.iter().map(|b| b.output.clone()).collect();
        let outputs: Vec<&str> = outputs.iter().map(String::as_str).collect();
        let found = if self.cube_off() {
            self.listing()
                .and_then(|(client, l)| Host::of(&l, &client, &outputs))
        } else {
            None
        };
        if found.is_some() || self.left.is_none() {
            self.host = found;
        }
        let heard = self.host.as_ref().or(self.left.as_ref());
        let mut events: Vec<String> = EVENTS.iter().map(|e| (*e).to_owned()).collect();
        events.extend(heard.map(|h| cube::event(&h.client)));
        if let Err(err) = self.rt.subscribe(&events) {
            eprintln!("chora-xp: events.subscribe: {err}");
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
            eprintln!("chora-xp: {} {}: {err}", host.client, cube::TURN);
        }
        Vec::new()
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
