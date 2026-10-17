//! Taskbar and Start Menu Properties, as XP lays it out: the tab Taskbar
//! with the taskbar appearance and the notification area, each under a
//! preview of the taskbar, and the tab Start Menu with the two styles of
//! the start menu. What chora-xp lacks is gray: grouping of similar task
//! buttons, Quick Launch, hiding inactive icons, the classic start menu
//! and the Customize buttons.

use std::fmt::Write as _;

use crate::{
    dialog::{self, Control, Dialog, Hit, Page},
    taskbar::{self, Settings},
    theme::{Picture, Rect, Sheet, Theme},
};

/// The client area: XP's dialog is 404 by 455 pixels with its frame.
const SIZE: [i32; 2] = [398, 423];
/// The white page below the tabs.
const PAGE: Rect = [6, 27, 384, 358];
const TABS: [&str; 2] = ["Taskbar", "Start Menu"];
/// The previews, 337 pixels wide as on XP.
const PREVIEW_WIDTH: i32 = 337;
const CHECK: i32 = 17;

/// What a click asks the window manager to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Show the page again.
    Redraw,
    /// Take the settings, and close the dialog or not.
    Apply(Settings, bool),
    /// Close without a change.
    Close,
}

/// The open dialog: the settings of the session and the ones that the
/// check boxes show.
pub struct TaskbarDialog {
    /// The window.
    pub dialog: Dialog,
    tab: usize,
    applied: Settings,
    shown: Settings,
}

impl TaskbarDialog {
    /// The dialog for the settings of the session, in the middle of a
    /// work area.
    pub fn new(settings: Settings, area: Rect) -> Self {
        Self {
            dialog: Dialog::new("taskbar", "Taskbar and Start Menu Properties", SIZE, area),
            tab: 0,
            applied: settings,
            shown: settings,
        }
    }

    /// The page of the current tab.
    pub fn page(&self, theme: &mut Theme, clock: &str) -> Page {
        let mut body = dialog::face(SIZE).svg(dialog::tab_page(PAGE));
        let mut controls = dialog::tabs(theme, [PAGE[0], PAGE[1]], &TABS, self.tab);
        if self.tab == 0 {
            self.taskbar_tab(theme, clock, &mut body, &mut controls);
        } else {
            start_menu_tab(theme, &mut body, &mut controls);
        }
        let [_, h] = SIZE;
        controls.extend([
            Control::push("ok", "OK", [156, h - 29]).default(),
            Control::push("cancel", "Cancel", [237, h - 29]),
            Control::push("apply", "Apply", [318, h - 29]).enabled(self.shown != self.applied),
        ]);
        Page {
            body: Some(body),
            controls,
        }
    }

    fn taskbar_tab(
        &self,
        theme: &mut Theme,
        clock: &str,
        body: &mut Picture,
        controls: &mut Vec<Control>,
    ) {
        let s = self.shown;
        let mut picture = std::mem::replace(body, Picture::new(1, 1));
        picture = dialog::group_box(theme, picture, [13, 37, 370, 172], "Taskbar appearance");
        let bar = taskbar::preview(
            theme,
            [PREVIEW_WIDTH, 46],
            &["Untitled - Notepad", "My Documents"],
            [s.auto_hide, s.on_top],
            None,
        );
        picture = framed(theme, picture, &bar, [25, 56]);
        let row = |i: i32| [25, 108 + i * 19, 340, CHECK];
        controls.extend([
            Control::check("locked", "Lock the taskbar", row(0), s.locked),
            Control::check("auto_hide", "Auto-hide the taskbar", row(1), s.auto_hide),
            Control::check(
                "on_top",
                "Keep the taskbar on top of other windows",
                row(2),
                s.on_top,
            ),
            Control::check("group", "Group similar taskbar buttons", row(3), false).enabled(false),
            Control::check("quick", "Show Quick Launch", row(4), false).enabled(false),
        ]);
        picture = dialog::group_box(theme, picture, [13, 217, 370, 127], "Notification area");
        let shown_clock = if s.clock { clock } else { "" };
        let tray = taskbar::preview(
            theme,
            [PREVIEW_WIDTH, 46],
            &[],
            [false, true],
            Some(shown_clock),
        );
        picture = framed(theme, picture, &tray, [25, 236]);
        controls.extend([
            Control::check("clock", "Show the clock", [25, 288, 200, CHECK], s.clock),
            Control::check(
                "inactive",
                "Hide inactive icons",
                [25, 307, 200, CHECK],
                true,
            )
            .enabled(false),
            Control::push("customize", "Customize...", [292, 312]).enabled(false),
        ]);
        *body = picture;
    }

    /// A click on a control: a tab shows its page, a check box flips,
    /// OK, Cancel and Apply do what they say.
    pub fn click(&mut self, hit: &Hit) -> Outcome {
        let Hit::Control(name, _) = hit else {
            return Outcome::Close;
        };
        let s = &mut self.shown;
        match name.as_str() {
            "tab-0" => self.tab = 0,
            "tab-1" => self.tab = 1,
            "locked" => s.locked = !s.locked,
            "auto_hide" => s.auto_hide = !s.auto_hide,
            "on_top" => s.on_top = !s.on_top,
            "clock" => s.clock = !s.clock,
            "ok" => return Outcome::Apply(self.shown, true),
            "cancel" => return Outcome::Close,
            "apply" => {
                self.applied = self.shown;
                return Outcome::Apply(self.shown, false);
            }
            _ => {}
        }
        Outcome::Redraw
    }

    /// The settings changed elsewhere, as by Lock the Taskbar of the
    /// context menu: the dialog shows them, keeping its own changes.
    pub fn changed(&mut self, settings: Settings) {
        if self.shown == self.applied {
            self.shown = settings;
        }
        self.applied = settings;
    }
}

/// The tab Start Menu: a preview of XP's start menu, and the two styles,
/// of which chora-xp has only the first.
fn start_menu_tab(theme: &mut Theme, body: &mut Picture, controls: &mut Vec<Control>) {
    let mut picture = std::mem::replace(body, Picture::new(1, 1));
    picture = framed(theme, picture, &start_menu_preview(theme), [25, 40]);
    let text = |p: Picture, line: &str, y: i32| dialog::label(p, line, [43, y, 245, 13], true);
    let gray = |p: Picture, line: &str, y: i32| dialog::label(p, line, [43, y, 245, 13], false);
    picture = text(
        picture,
        "Select this menu style for easy access to the",
        246,
    );
    picture = text(
        picture,
        "Internet, e-mail, and your favorite programs.",
        259,
    );
    picture = gray(
        picture,
        "Select this option to use the menu style from",
        300,
    );
    picture = gray(picture, "earlier versions of Windows.", 313);
    controls.extend([
        Control::radio("xp", "Start menu", [25, 226, 200, CHECK], true),
        Control::push("customize", "Customize...", [303, 226]).enabled(false),
        Control::radio(
            "classic",
            "Classic Start menu",
            [25, 280, 200, CHECK],
            false,
        )
        .enabled(false),
        Control::push("customize-classic", "Customize...", [303, 280]).enabled(false),
    ]);
    *body = picture;
}

/// XP's start menu in small, above the start button: the blue header,
/// the white and the blue column and the footer of the theme.
fn start_menu_preview(theme: &Theme) -> Picture {
    let [w, h] = [PREVIEW_WIDTH, 176];
    let bar = taskbar::preview(theme, [w, h], &[], [false, true], None);
    let [mx, mw] = [8, 170];
    let mut picture = bar
        .element(Sheet::Winxp, "startmenu-header", [mx, 6, mw, 22])
        .element(Sheet::Winxp, "startmenu-body", [mx, 28, mw, 98])
        .element(Sheet::Winxp, "startmenu-footer", [mx, 126, mw, 16]);
    // Lines for the entries of the two columns.
    let mut svg = String::new();
    for i in 0..6 {
        let y = 36 + i * 14;
        let _ = write!(
            svg,
            r##"<rect x="{}" y="{y}" width="10" height="10" fill="#7A96DF"/><rect x="{}" y="{}" width="44" height="3" fill="#8A8A8A"/>"##,
            mx + 6,
            mx + 20,
            y + 4
        );
        let _ = write!(
            svg,
            r##"<rect x="{}" y="{y}" width="10" height="10" fill="#F2C14F"/><rect x="{}" y="{}" width="44" height="3" fill="#1E3287"/>"##,
            mx + mw / 2 + 6,
            mx + mw / 2 + 20,
            y + 4
        );
    }
    picture = picture.svg(svg);
    picture
}

/// A preview with XP's thin frame, at a point of a picture.
fn framed(theme: &mut Theme, picture: Picture, preview: &Picture, [x, y]: [i32; 2]) -> Picture {
    let [w, h] = preview.size;
    let file = std::path::PathBuf::from(theme.render(preview));
    picture.file(&file, [x, y, w, h]).svg(format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#919B9C"/>"##,
        f64::from(x) - 0.5,
        f64::from(y) - 0.5,
        w + 1,
        h + 1
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::tests::{preview, theme};

    #[test]
    fn check_boxes_change_the_settings_on_apply() {
        let mut panel = TaskbarDialog::new(Settings::default(), [0, 0, 1280, 770]);
        let click = |p: &mut TaskbarDialog, name: &str| p.click(&Hit::Control(name.into(), [1, 1]));
        assert_eq!(click(&mut panel, "auto_hide"), Outcome::Redraw);
        assert_eq!(click(&mut panel, "clock"), Outcome::Redraw);
        let changed = Settings {
            auto_hide: true,
            clock: false,
            ..Settings::default()
        };
        assert_eq!(click(&mut panel, "apply"), Outcome::Apply(changed, false));
        assert_eq!(click(&mut panel, "ok"), Outcome::Apply(changed, true));
        assert_eq!(panel.click(&Hit::Close), Outcome::Close);
    }

    #[test]
    fn the_tabs_look_like_xp() {
        let (mut theme, cache) = theme("taskbar-dialog");
        let mut panel = TaskbarDialog::new(Settings::default(), [0, 0, 1280, 770]);
        for (tab, name) in [(0, "taskbar"), (1, "startmenu")] {
            panel.tab = tab;
            let page = panel.page(&mut theme, "4:09 PM");
            panel.dialog.create(&mut theme, "root", page);
            preview(&mut theme, &panel.dialog, name);
        }
        let _ = std::fs::remove_dir_all(cache);
    }
}
