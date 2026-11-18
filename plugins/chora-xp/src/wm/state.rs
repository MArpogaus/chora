//! `state.get` (design 3.1): what chora-xp shows and holds, read from its
//! model until xp uses `kit::wm`; the port deletes this adapter. Only
//! reads. Desktops count from 1, as the pager shows them. `views` and
//! `texts` stay out: xp shows no window of another look, and its drawn
//! text runs come with NET3.

use serde_json::{Value, json};

use super::{NAME, Wm};

impl Wm {
    /// The client windows bottom first, the desktops, the focus, the open
    /// menus and dialogs, and the effects that still run.
    pub(super) fn state_value(&self) -> Value {
        let windows: Vec<Value> = (self.windows.iter().enumerate())
            .map(|(z, w)| {
                json!({
                    "surface": w.surface,
                    "title": w.title,
                    "app_id": w.app_id,
                    "output": self.output_of(&w.surface),
                    "desks": [w.desktop + 1],
                    "rect": w.rect(),
                    "shown": w.shown && !w.minimized && w.desktop == self.desktop,
                    "minimized": w.minimized,
                    "maximized": w.maximized(),
                    "fullscreen": false,
                    "floating": self.floats.floats(&w.surface),
                    "z": z,
                })
            })
            .collect();
        let desks: Vec<Value> = (self.bars.iter())
            .map(|b| json!({"output": b.output, "shown": [self.desktop + 1]}))
            .collect();
        let focus = match self.focused.as_deref().filter(|s| self.index(s).is_some()) {
            Some(surface) => json!({"holder": NAME, "surface": surface}),
            None => json!({"holder": null}),
        };
        json!({"windows": windows, "desks": desks, "focus": focus,
            "open": self.open_value(), "busy": self.busy_value()})
    }

    /// The menus and dialogs that show, in the order Escape shuts them.
    fn open_value(&self) -> Vec<Value> {
        let entry =
            |id: &str, kind: &str, modal: bool| json!({"id": id, "kind": kind, "modal": modal});
        let mut open = Vec::new();
        if self.switcher.is_some() {
            open.push(entry("xp-switcher", "switcher", false));
        }
        if self.dialog.is_some() {
            open.push(entry("xp-session", "dialog", true));
        }
        if self.context.is_some() {
            let mut menu = entry("xp-ctx", "menu", false);
            if let Some(surface) = &self.sys_window {
                menu["for"] = json!(surface);
            }
            open.push(menu);
        }
        if self.menu.is_open() {
            open.push(entry("xp-menu", "menu", false));
        }
        for panel in self.panels.iter().rev() {
            open.push(entry(&panel.window_ref().group(), "dialog", false));
        }
        open
    }

    /// What still moves: quiet once this is empty.
    fn busy_value(&self) -> Vec<&'static str> {
        let running = [
            (self.effects.driver.animating(), "effects"),
            (self.effects.turn.is_some(), "cube"),
            (self.fading(), "fade"),
            (self.sliding(), "slide"),
            (self.resize_due(), "resize"),
        ];
        running
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::runtime::Plugin as _;
    use serde_json::json;

    use super::super::tests::wm_on_fake_core;
    use crate::window::Window;

    /// A window with a size on a desktop.
    fn window(surface: &str, desktop: usize) -> Window {
        let mut w = Window::new(surface, Some("notepad".into()), Some(surface));
        w.size = Some([200, 100]);
        w.shown = true;
        w.desktop = desktop;
        w
    }

    /// A fresh xp shows no window, holds no focus and is quiet.
    #[test]
    fn a_fresh_xp_has_an_empty_state() {
        let (wm, _methods) = wm_on_fake_core("state-empty");
        let state = wm.state().expect("xp has a state");
        let expected = json!({"windows": [], "focus": {"holder": null}, "open": [],
            "busy": [], "desks": [{"output": "X", "shown": [1]}]});
        assert_eq!(state, expected);
    }

    /// The fields of a window that a test reads.
    fn read(state: &serde_json::Value, i: usize) -> serde_json::Value {
        let w = &state["windows"][i];
        json!([
            w["surface"],
            w["output"],
            w["z"],
            w["shown"],
            w["desks"],
            w["minimized"]
        ])
    }

    /// The windows come bottom first with their desktop, and only those
    /// of the desktop that shows show.
    #[test]
    fn the_state_lists_the_windows_bottom_first() {
        let (mut wm, _methods) = wm_on_fake_core("state-windows");
        wm.windows.push(window("s1", 0));
        wm.windows.push(window("s2", 1));
        let state = wm.state().unwrap();
        assert_eq!(read(&state, 0), json!(["s1", "X", 0, true, [1], false]));
        assert_eq!(read(&state, 1), json!(["s2", "X", 1, false, [2], false]));
        assert_eq!(state["windows"][0]["app_id"], "notepad");
        wm.windows[0].minimized = true;
        let state = wm.state().unwrap();
        assert_eq!(read(&state, 0), json!(["s1", "X", 0, false, [1], true]));
    }

    /// xp holds the focus only while it is on one of its windows.
    #[test]
    fn xp_holds_the_focus_only_on_its_window() {
        let (mut wm, _methods) = wm_on_fake_core("state-focus");
        wm.windows.push(window("s1", 0));
        wm.focused = Some("s1".into());
        let focus = json!({"holder": "xp", "surface": "s1"});
        assert_eq!(wm.state().unwrap()["focus"], focus);
        wm.focused = Some("other".into());
        assert_eq!(wm.state().unwrap()["focus"], json!({"holder": null}));
    }

    /// The start menu is open after `start_menu {open: true}` and shut
    /// after `{open: false}`.
    #[test]
    fn the_start_menu_is_open_in_the_state() {
        let (mut wm, _methods) = wm_on_fake_core("state-menu");
        wm.method("start_menu", json!({"open": true})).unwrap();
        let open = &wm.state().unwrap()["open"];
        assert_eq!(
            *open,
            json!([{"id": "xp-menu", "kind": "menu", "modal": false}])
        );
        wm.method("start_menu", json!({"open": false})).unwrap();
        assert_eq!(wm.state().unwrap()["open"], json!([]));
    }
}
