//! XP's Display Properties, 404 by 455 pixels with its five tabs:
//!
//! - Themes: the sample of the theme, the wallpaper with an active window
//!   and the Recycle Bin;
//! - Desktop: the monitor with the chosen background, the list of the
//!   wallpapers, their position and the color of the desktop;
//! - Screen Saver and Appearance: as XP lays them out, Effects... with the
//!   effects of chora-xp;
//! - Settings: the scale of the output as XP's screen resolution, and the
//!   monitor.
//!
//! What chora-xp lacks is gray: other themes, Browse... (a file chooser),
//! Customize Desktop..., screen savers, power settings, color schemes and
//! fonts, the color depth and the advanced settings. Apply and OK take the
//! background and the scale; Cancel closes without them.

use std::{fmt::Write as _, path::PathBuf};

use crate::{
    config::{self, Wallpaper},
    context::{Act, Entry},
    control::{self, SCALES, Toggles},
    decoration::{self, BORDER, TITLE},
    dialog::{self, BLACK, Control, Dialog, Hit, Page},
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
    wallpaper::{self, Position},
};

/// The client area: XP's dialog is 404 by 455 pixels with its frame.
const SIZE: [i32; 2] = [398, 423];
const PAGE: Rect = [6, 27, 384, 358];
const TABS: [&str; 5] = [
    "Themes",
    "Desktop",
    "Screen Saver",
    "Appearance",
    "Settings",
];
/// The monitor of Desktop, Screen Saver and Settings, and its screen.
const MONITOR: Rect = [111, 47, 177, 160];
const SCREEN: Rect = [122, 62, 152, 112];
/// The list of the backgrounds and the height of its rows.
const LIST: Rect = [21, 235, 272, 114];
const ROW: i32 = 17;
/// The width of a scroll bar.
const SCROLL: i32 = 17;
const POSITION: Rect = [303, 286, 75, 21];
const COLOR: Rect = [303, 333, 74, 21];
/// The slider of the screen resolution.
const SLIDER: Rect = [56, 290, 100, 26];
/// The sample of Themes and Appearance.
const SAMPLE: Rect = [21, 145, 356, 224];
const APPEARANCE: Rect = [21, 40, 356, 150];
/// The colors of the drop-down palette: XP's blue of the desktop, and
/// the colors of XP's basic palette.
const COLORS: [(&str, [u8; 3]); 12] = [
    ("Desktop blue", wallpaper::DESKTOP_BLUE),
    ("Teal blue", [0x3A, 0x6E, 0xA5]),
    ("Black", [0, 0, 0]),
    ("Gray", [0x80, 0x80, 0x80]),
    ("Silver", [0xC0, 0xC0, 0xC0]),
    ("White", [0xFF, 0xFF, 0xFF]),
    ("Navy", [0, 0, 0x80]),
    ("Teal", [0, 0x80, 0x80]),
    ("Green", [0, 0x80, 0]),
    ("Olive", [0x80, 0x80, 0]),
    ("Maroon", [0x80, 0, 0]),
    ("Purple", [0x80, 0, 0x80]),
];
const WHITE: [u8; 4] = [255, 255, 255, 255];
const BLUE: &str = "#316AC5";

/// The output of the dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    /// Its name.
    pub output: String,
    /// Its size in pixels of the hardware.
    pub physical: [i32; 2],
    /// Its scale in percent.
    pub scale: u32,
}

/// What a click asks the window manager to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Click {
    /// Show the page again.
    Redraw,
    /// Close without a change.
    Close,
    /// Take the background and the scale of the output, and close the
    /// dialog or not.
    Apply {
        /// The background of the desktop.
        style: Wallpaper,
        /// The scale in percent.
        scale: u32,
        /// OK closes the dialog, Apply keeps it.
        close: bool,
    },
    /// Open a drop-down list below a rectangle of the client area.
    List(Rect, Vec<Entry>),
    /// Open the effects.
    Effects,
}

/// The files of the pictures that the pages show.
#[derive(Clone, Debug, Default)]
pub struct Art {
    /// The icon of a picture file in the list, 16 pixels.
    pub picture: Option<PathBuf>,
    /// The Recycle Bin on the sample of Themes.
    pub bin: Option<PathBuf>,
    /// The wallpapers of the list.
    pub walls: Vec<PathBuf>,
}

/// The open dialog.
pub struct DisplayDialog {
    /// The window.
    pub dialog: Dialog,
    /// The tab that shows.
    pub tab: usize,
    walls: Vec<PathBuf>,
    applied: Wallpaper,
    shown: Wallpaper,
    screen: Screen,
    /// The scale that the slider shows.
    scale: u32,
    /// The first row of the list that shows.
    top: usize,
    art: Art,
}

/// The file of a resolved wallpaper.
fn file(wallpaper: &Wallpaper) -> Option<&PathBuf> {
    match &wallpaper.picture {
        config::Picture::File(file) => Some(file),
        _ => None,
    }
}

impl DisplayDialog {
    /// The dialog for the background of the session and an output, on a
    /// tab, in the middle of a work area. A wallpaper that is not in the
    /// list joins it. The caller resolves the default picture.
    pub fn new(style: Wallpaper, screen: Screen, tab: usize, area: Rect, mut art: Art) -> Self {
        let mut walls = std::mem::take(&mut art.walls);
        if let Some(file) = file(&style).filter(|f| !walls.contains(f)) {
            walls.push(file.clone());
        }
        let mut panel = Self {
            dialog: Dialog::new("display", "Display Properties", SIZE, area),
            tab: tab.min(TABS.len() - 1),
            walls,
            applied: style.clone(),
            shown: style,
            scale: screen.scale,
            screen,
            top: 0,
            art,
        };
        panel.reveal();
        panel
    }

    /// The names of the list: (None) and the wallpapers.
    fn names(&self) -> Vec<String> {
        let mut names = vec!["(None)".to_string()];
        names.extend(
            self.walls
                .iter()
                .map(|w| control::name_of(&w.file_stem().unwrap_or_default().to_string_lossy())),
        );
        names
    }

    /// The row of the chosen background.
    fn chosen(&self) -> usize {
        file(&self.shown)
            .and_then(|f| self.walls.iter().position(|w| w == f))
            .map_or(0, |i| i + 1)
    }

    /// The rows that the list shows whole.
    fn visible() -> usize {
        usize::try_from((LIST[3] - 2) / ROW).unwrap_or(1)
    }

    fn max_top(&self) -> usize {
        (self.walls.len() + 1).saturating_sub(Self::visible())
    }

    /// Scrolls the list so that the chosen row shows.
    fn reveal(&mut self) {
        let row = self.chosen();
        if row < self.top {
            self.top = row;
        } else if row >= self.top + Self::visible() {
            self.top = row + 1 - Self::visible();
        }
        self.top = self.top.min(self.max_top());
    }

    /// The logical size of the output at a scale.
    fn logical(&self, scale: u32) -> [i32; 2] {
        let scale = i64::from(scale.max(1));
        self.screen
            .physical
            .map(|n| i32::try_from((i64::from(n) * 100 + scale / 2) / scale).unwrap_or(n))
    }

    /// The page of the current tab.
    pub fn page(&self, theme: &mut Theme) -> Page {
        let mut body = dialog::face(SIZE).svg(dialog::tab_page(PAGE));
        let mut controls = dialog::tabs(theme, [PAGE[0], PAGE[1]], &TABS, self.tab);
        match self.tab {
            0 => body = self.themes_tab(theme, body, &mut controls),
            1 => body = self.desktop_tab(theme, body, &mut controls),
            2 => body = screen_saver_tab(theme, body, &mut controls),
            3 => body = self.appearance_tab(theme, body, &mut controls),
            _ => body = self.settings_tab(theme, body, &mut controls),
        }
        let [_, h] = SIZE;
        let changed = self.shown != self.applied || self.scale != self.screen.scale;
        controls.extend([
            Control::push("ok", "OK", [156, h - 29]).default(),
            Control::push("cancel", "Cancel", [237, h - 29]),
            Control::push("apply", "Apply", [318, h - 29]).enabled(changed),
        ]);
        Page {
            body: Some(body),
            controls,
        }
    }

    /// The desktop as the monitor shows it.
    fn screen_picture(&self, theme: &Theme) -> Picture {
        let [.., w, h] = SCREEN;
        wallpaper::picture(theme, &self.shown, self.logical(self.screen.scale), [w, h])
    }

    fn themes_tab(&self, theme: &mut Theme, body: Picture, controls: &mut Vec<Control>) -> Picture {
        let text = |p: Picture, line: &str, y: i32| dialog::label(p, line, [21, y, 356, 13], true);
        let mut body = text(
            body,
            "A theme is a background plus a set of sounds, icons, and other elements",
            38,
        );
        body = text(
            body,
            "to help you personalize your computer with one click.",
            51,
        );
        body = text(body, "Theme:", 80);
        body = text(body, "Sample:", 131);
        controls.extend([
            Control::area(
                "theme",
                [21, 97, 192, 21],
                dialog::combo([192, 21], "Windows XP", false),
            )
            .enabled(false),
            Control::push("save-as", "Save As...", [221, 95]).enabled(false),
            Control::push("delete", "Delete", [302, 95]).enabled(false),
        ]);
        let [x, y, w, h] = SAMPLE;
        let [iw, ih] = [w - 4, h - 4];
        let mut sample = wallpaper::picture(
            theme,
            &self.shown,
            self.logical(self.screen.scale),
            [iw, ih],
        );
        sample = sample.layers_of(
            window_sample(theme, "Active Window", [216, 123], true),
            [35, 35],
        );
        if let Some(bin) = &self.art.bin {
            sample = sample.file(bin, [iw - 52, ih - 44, 32, 32]);
        }
        let file = PathBuf::from(theme.render(&sample));
        body.svg(dialog::sunken(SAMPLE))
            .file(&file, [x + 2, y + 2, iw, ih])
    }

    fn desktop_tab(
        &self,
        theme: &mut Theme,
        body: Picture,
        controls: &mut Vec<Control>,
    ) -> Picture {
        let mut body = monitor(theme, body, &self.screen_picture(theme));
        body = dialog::label(body, "Background:", [21, 217, 200, 13], true);
        let has_picture = file(&self.shown).is_some();
        body = dialog::label(body, "Position:", [303, 270, 75, 13], has_picture);
        body = dialog::label(body, "Color:", [303, 317, 75, 13], true);
        let [.., cw, ch] = COLOR;
        controls.extend([
            Control::area("list", LIST, self.list(theme)),
            Control::push("browse", "Browse...", [304, 236]).enabled(false),
            Control::area(
                "position",
                POSITION,
                dialog::combo(
                    [POSITION[2], POSITION[3]],
                    self.shown.position.name(),
                    has_picture,
                ),
            )
            .enabled(has_picture),
            Control::area("color", COLOR, color_button([cw, ch], self.shown.color)),
            {
                let mut c = Control::push("customize", "Customize Desktop...", [21, 354]);
                c.rect[2] = 134;
                c.enabled(false)
            },
        ]);
        body
    }

    /// The list of the backgrounds: an icon and a name in each row, the
    /// chosen name in XP's blue, and the scroll bar.
    fn list(&self, theme: &Theme) -> Picture {
        let [.., w, h] = LIST;
        let mut picture =
            Picture::new(w, h).svg(format!(r##"<rect width="{w}" height="{h}" fill="#fff"/>"##));
        let names = self.names();
        let chosen = self.chosen();
        for (i, name) in names
            .iter()
            .enumerate()
            .skip(self.top)
            .take(Self::visible() + 1)
        {
            let y = 1 + i32::try_from(i - self.top).unwrap_or(0) * ROW;
            if i == 0 {
                picture = picture.svg(format!(
                    r##"<g transform="translate(5 {y})"><circle cx="8" cy="8" r="6" fill="none" stroke="#F52F15" stroke-width="2.2"/><path d="M3.8 3.8L12.2 12.2" stroke="#F52F15" stroke-width="2.2"/></g>"##
                ));
            } else if let Some(icon) = &self.art.picture {
                picture = picture.file(icon, [5, y, 16, 16]);
            }
            #[allow(clippy::cast_possible_truncation)]
            let tw = theme.text_width(name, Font::Ui).ceil() as i32;
            let color = if i == chosen {
                picture = picture.svg(format!(
                    r#"<rect x="24" y="{y}" width="{}" height="{}" fill="{BLUE}"/>"#,
                    tw + 4,
                    ROW - 1
                ));
                WHITE
            } else {
                BLACK
            };
            picture = picture.text(Text::new(
                name,
                Font::Ui,
                color,
                [26, y, w - SCROLL - 30, ROW - 1],
            ));
        }
        let rows = i32::try_from(names.len()).unwrap_or(0);
        let top = i32::try_from(self.top).unwrap_or(0);
        let track = h - 2 - 2 * SCROLL;
        let thumb = thumb(track, h - 2, rows * ROW, top * ROW);
        for (rect, part) in scroll_bar([SCROLL, h - 2], thumb) {
            picture = picture.layers_of(part, [w - 1 - SCROLL + rect[0], 1 + rect[1]]);
        }
        picture.svg(format!(
            r##"<rect x="0.5" y="0.5" width="{}" height="{}" fill="none" stroke="#7F9DB9"/>"##,
            w - 1,
            h - 1
        ))
    }

    #[allow(clippy::many_single_char_names)] // coordinates and colors
    fn appearance_tab(
        &self,
        theme: &mut Theme,
        body: Picture,
        controls: &mut Vec<Control>,
    ) -> Picture {
        let [x, y, w, h] = APPEARANCE;
        let [iw, ih] = [w - 4, h - 4];
        let [r, g, b] = self.shown.color;
        let mut sample = Picture::new(iw, ih).svg(format!(
            r#"<rect width="{iw}" height="{ih}" fill="rgb({r},{g},{b})"/>"#
        ));
        sample = sample
            .layers_of(
                window_sample(theme, "Inactive Window", [230, 80], false),
                [12, 10],
            )
            .layers_of(
                window_sample(theme, "Active Window", [240, 100], true),
                [26, 36],
            );
        let message = window_sample(theme, "Message Box", [160, 76], true)
            .svg(format!(
                r#"<rect x="{BORDER}" y="{TITLE}" width="{}" height="{}" fill="{}"/>"#,
                160 - 2 * BORDER,
                76 - TITLE - BORDER,
                dialog::FACE
            ))
            .text(Text::new(
                "Message Text",
                Font::Ui,
                BLACK,
                [12, TITLE + 4, 120, 13],
            ))
            .layers_of(
                Picture::new(52, 19)
                    .svg(dialog::push_face(52, 19, true))
                    .text(Text::new("OK", Font::Ui, BLACK, [0, 0, 52, 19]).centered()),
                [54, TITLE + 21],
            );
        sample = sample.layers_of(message, [178, 64]);
        let file = PathBuf::from(theme.render(&sample));
        let mut body = body
            .svg(dialog::sunken(APPEARANCE))
            .file(&file, [x + 2, y + 2, iw, ih]);
        for (label, value, y) in [
            ("Windows and buttons:", "Windows XP style", 204),
            ("Color scheme:", "Default (blue)", 250),
            ("Font size:", "Normal", 296),
        ] {
            body = dialog::label(body, label, [21, y, 200, 13], true);
            controls.push(
                Control::area(
                    label,
                    [21, y + 16, 224, 21],
                    dialog::combo([224, 21], value, false),
                )
                .enabled(false),
            );
        }
        controls.extend([
            Control::push("effects", "Effects...", [292, 266]),
            Control::push("advanced", "Advanced", [292, 312]).enabled(false),
        ]);
        body
    }

    fn settings_tab(
        &self,
        theme: &mut Theme,
        body: Picture,
        controls: &mut Vec<Control>,
    ) -> Picture {
        let [.., sw, sh] = SCREEN;
        // The desktop with a taskbar, as XP's small picture of it.
        let screen = self.screen_picture(theme).svg(format!(
            r##"<rect y="{}" width="{sw}" height="6" fill="#245EDB"/><rect y="{}" width="20" height="6" rx="1.5" fill="#3C9A3C"/>"##,
            sh - 6,
            sh - 6
        ));
        let mut body = monitor(theme, body, &screen);
        body = dialog::label(body, "Display:", [21, 230, 356, 13], true);
        body = dialog::label(
            body,
            &format!("{} on chora", self.screen.output),
            [21, 246, 356, 13],
            true,
        );
        body = dialog::group_box(theme, body, [21, 274, 173, 70], "Screen resolution");
        body = dialog::group_box(theme, body, [204, 274, 174, 70], "Color quality");
        body = dialog::label(body, "Less", [31, 294, 30, 13], true);
        body = dialog::label(body, "More", [157, 294, 30, 13], true);
        let [w, h] = self.logical(self.scale);
        body = body.text(
            Text::new(
                &format!("{w} by {h} pixels"),
                Font::Ui,
                BLACK,
                [21, 320, 173, 13],
            )
            .centered(),
        );
        body = body.svg(spectrum([213, 318, 155, 14]));
        controls.extend([
            Control::area("slider", SLIDER, slider(tick_of(self.scale))),
            Control::area(
                "depth",
                [213, 291, 155, 21],
                dialog::combo([155, 21], "Highest (32 bit)", false),
            )
            .enabled(false),
            {
                let mut c = Control::push("troubleshoot", "Troubleshoot...", [200, 353]);
                c.rect[2] = 87;
                c.enabled(false)
            },
            {
                let mut c = Control::push("advanced", "Advanced", [295, 353]);
                c.rect[2] = 82;
                c.enabled(false)
            },
        ]);
        body
    }

    /// A click on a control.
    pub fn click(&mut self, hit: &Hit) -> Click {
        let Hit::Control(name, point) = hit else {
            return Click::Close;
        };
        if let Some(tab) = name.strip_prefix("tab-").and_then(|t| t.parse().ok()) {
            self.tab = tab;
            return Click::Redraw;
        }
        match name.as_str() {
            "list" => self.click_list(*point),
            "position" => {
                let entries = Position::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        Entry::item(p.name(), Act::Position(i)).checked(*p == self.shown.position)
                    })
                    .collect();
                return Click::List(POSITION, entries);
            }
            "color" => {
                let entries = COLORS
                    .iter()
                    .map(|(name, c)| {
                        Entry::item(name, Act::Color(*c))
                            .swatch(*c)
                            .checked(*c == self.shown.color)
                    })
                    .collect();
                return Click::List(COLOR, entries);
            }
            "slider" => {
                let tick = nearest_tick(point[0]);
                self.scale = SCALES[SCALES.len() - 1 - tick];
            }
            "effects" => return Click::Effects,
            "ok" | "apply" => {
                let close = name == "ok";
                if !close {
                    self.applied = self.shown.clone();
                    self.screen.scale = self.scale;
                }
                return Click::Apply {
                    style: self.shown.clone(),
                    scale: self.scale,
                    close,
                };
            }
            "cancel" => return Click::Close,
            _ => {}
        }
        Click::Redraw
    }

    /// A click in the list: a row chooses its background, the scroll bar
    /// scrolls by a row or by a page.
    fn click_list(&mut self, [x, y]: [i32; 2]) {
        let [.., w, h] = LIST;
        let page = Self::visible();
        if x >= w - 1 - SCROLL {
            let rows = i32::try_from(self.walls.len() + 1).unwrap_or(0);
            let top = i32::try_from(self.top).unwrap_or(0);
            let (thumb, size) = thumb(h - 2 - 2 * SCROLL, h - 2, rows * ROW, top * ROW);
            let y = y - 1 - SCROLL;
            self.top = if y < 0 {
                self.top.saturating_sub(1)
            } else if y >= h - 2 - 2 * SCROLL {
                self.top + 1
            } else if y < thumb {
                self.top.saturating_sub(page)
            } else if y >= thumb + size {
                self.top + page
            } else {
                self.top
            }
            .min(self.max_top());
            return;
        }
        let row = self.top + usize::try_from((y - 1).max(0) / ROW).unwrap_or(0);
        if row == 0 {
            self.shown.picture = config::Picture::None;
        } else if let Some(wall) = self.walls.get(row - 1) {
            self.shown.picture = config::Picture::File(wall.clone());
        }
    }

    /// A row of the list of Position.
    pub fn set_position(&mut self, index: usize) {
        if let Some(p) = Position::ALL.get(index) {
            self.shown.position = *p;
        }
    }

    /// A color of the palette.
    pub fn set_color(&mut self, color: [u8; 3]) {
        self.shown.color = color;
    }

    /// The point of the client area where a list opens, in the scene,
    /// and its width.
    pub fn list_at(&self, [x, y, w, h]: Rect) -> ([i32; 2], i32) {
        let [cx, cy] = self.dialog.client();
        ([cx + x, cy + y + h], w)
    }

    /// The output of the dialog.
    pub fn output(&self) -> &str {
        &self.screen.output
    }
}

/// The ticks of the slider, from Less to More: the scales from the
/// largest to the smallest.
fn tick_of(scale: u32) -> usize {
    SCALES
        .iter()
        .position(|s| *s == scale)
        .map_or(SCALES.len() - 1, |i| SCALES.len() - 1 - i)
}

/// The x of a tick in the slider.
fn tick_x(tick: usize) -> i32 {
    let steps = i32::try_from(SCALES.len() - 1).unwrap_or(1);
    8 + i32::try_from(tick).unwrap_or(0) * (SLIDER[2] - 16) / steps
}

/// The tick nearest to a point of the slider.
fn nearest_tick(x: i32) -> usize {
    (0..SCALES.len())
        .min_by_key(|t| (tick_x(*t) - x).abs())
        .unwrap_or(0)
}

/// XP's track bar: the sunken channel, the ticks below it and the thumb
/// that points down at its tick.
fn slider(tick: usize) -> Picture {
    let [.., w, h] = SLIDER;
    let mut svg = format!(
        r##"<rect x="8.5" y="7.5" width="{}" height="3" fill="#F4F3EE" stroke="#9D9C99"/><path d="M9 10.5H{}" stroke="#fff"/>"##,
        w - 17,
        w - 8
    );
    for t in 0..SCALES.len() {
        let _ = write!(
            svg,
            r##"<rect x="{}" y="21" width="1" height="4" fill="#000"/>"##,
            tick_x(t)
        );
    }
    let x = f64::from(tick_x(tick));
    let _ = write!(
        svg,
        r##"<defs><linearGradient id="th" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#fff"/><stop offset="1" stop-color="#E4E3DB"/></linearGradient></defs><path d="M{l} 0.5H{r}V13.5L{x} 18.5L{l} 13.5Z" fill="url(#th)" stroke="#778892"/><path d="M{l2} 13L{x} 17L{r2} 13" fill="none" stroke="#21A121" stroke-width="1.2"/>"##,
        l = x - 5.0,
        r = x + 6.0,
        l2 = x - 4.0,
        r2 = x + 5.0,
    );
    Picture::new(w, h).svg(svg)
}

/// The colors of the color depth, three rainbows side by side.
fn spectrum([x, y, w, h]: Rect) -> String {
    let stops = [
        "#FF00BF", "#FF0000", "#FFAC00", "#00FF00", "#00FF93", "#0000FF", "#8000FF",
    ];
    let mut svg = r#"<defs><linearGradient id="spec" x1="0" y1="0" x2="1" y2="0">"#.to_string();
    for (i, s) in stops.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let offset = i as f64 / (stops.len() - 1) as f64;
        let _ = write!(svg, r#"<stop offset="{offset:.3}" stop-color="{s}"/>"#);
    }
    svg += "</linearGradient></defs>";
    let third = w / 3;
    for i in 0..3 {
        let _ = write!(
            svg,
            r#"<rect x="{}" y="{y}" width="{third}" height="{h}" fill="url(#spec)"/>"#,
            x + i * third
        );
    }
    svg + &format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="#ACA899"/>"##,
        f64::from(x) - 0.5,
        f64::from(y) - 0.5,
        w + 1,
        h + 1
    )
}

/// The color button: the push button with the color in a black frame, a
/// line and the arrow of its palette.
#[allow(clippy::many_single_char_names)] // sizes and colors
fn color_button([w, h]: [i32; 2], [r, g, b]: [u8; 3]) -> Picture {
    Picture::new(w, h).svg(dialog::push_face(w, h, true) + &format!(
        r##"<rect x="6.5" y="5.5" width="52" height="{}" fill="rgb({r},{g},{b})" stroke="#000"/><rect x="62" y="4" width="1" height="{}" fill="#ACA899"/><rect x="63" y="4" width="1" height="{}" fill="#fff"/><path d="M65 9h5l-2.5 3z" fill="#000"/>"##,
        h - 11,
        h - 8,
        h - 8
    ))
}

/// XP's monitor around a picture of its screen, on a body.
fn monitor(theme: &mut Theme, body: Picture, screen: &Picture) -> Picture {
    let [mx, my, ..] = MONITOR;
    let [sx, sy, sw, sh] = SCREEN;
    let (l, t) = (f64::from(mx) + 0.5, f64::from(my) + 0.5);
    let svg = format!(
        r##"<defs><linearGradient id="bezel" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="0.6" stop-color="#F1F0EA"/><stop offset="1" stop-color="#D9D9D2"/></linearGradient><linearGradient id="foot" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#D8D9DB"/><stop offset="0.5" stop-color="#9EA1A8"/><stop offset="1" stop-color="#5E6370"/></linearGradient></defs><path d="M{n1} {t2}h{neck}v6h-{neck}z" fill="#6E717A"/><ellipse cx="{c}" cy="{fy}" rx="52" ry="6" fill="url(#foot)" stroke="#3F4552"/><rect x="{l}" y="{t}" width="176" height="145" rx="6" fill="url(#bezel)" stroke="#9CA2B2"/><path d="M{l2} {b2}V{t3}Q{l2} {t1} {l4} {t1}H{r4}" fill="none" stroke="#fff"/><rect x="{dx}" y="{dy}" width="85" height="12" rx="1" fill="#F2F1EB" stroke="#C5C8CD"/><rect x="{led}" y="{ly}" width="2" height="2" fill="#12CB15"/><circle cx="{bx}" cy="{by}" r="3.5" fill="#E7E7E3" stroke="#64656A"/><rect x="{fx}" y="{fy2}" width="{fw}" height="{fh}" fill="none" stroke="#64656A"/>"##,
        n1 = mx + 58,
        t2 = my + 145,
        neck = 60,
        c = f64::from(mx) + 88.5,
        fy = my + 153,
        l2 = l + 1.0,
        b2 = t + 140.0,
        t3 = t + 6.0,
        t1 = t + 1.0,
        l4 = l + 6.0,
        r4 = l + 170.0,
        dx = f64::from(mx + 45) + 0.5,
        dy = f64::from(my + 130) + 0.5,
        led = mx + 137,
        ly = my + 136,
        bx = f64::from(mx) + 148.5,
        by = f64::from(my) + 135.5,
        fx = f64::from(sx) - 0.5,
        fy2 = f64::from(sy) - 0.5,
        fw = sw + 1,
        fh = sh + 1,
    );
    let file = PathBuf::from(theme.render(screen));
    body.svg(svg).file(&file, SCREEN)
}

/// A window of the theme as XP's samples show it: the frame with its
/// title and buttons, the white client with Window Text and a scroll bar.
fn window_sample(_theme: &Theme, title: &str, [w, h]: [i32; 2], active: bool) -> Picture {
    let [cw, ch] = [w - 2 * BORDER, h - TITLE - BORDER];
    let mut picture = decoration::sample(title, [w, h], active).svg(format!(
        r##"<rect x="{BORDER}" y="{TITLE}" width="{cw}" height="{ch}" fill="#fff"/>"##
    ));
    if active {
        picture = picture.text(Text::new(
            "Window Text",
            Font::Ui,
            BLACK,
            [BORDER + 2, TITLE + 1, cw - SCROLL - 4, 13],
        ));
        for (rect, part) in scroll_bar([SCROLL, ch], (0, (ch - 2 * SCROLL) / 4)) {
            picture = picture.layers_of(part, [w - BORDER - SCROLL + rect[0], TITLE + rect[1]]);
        }
    }
    picture
}

/// Screen Saver: the monitor without a picture, and the screen saver
/// and the power settings, gray: chora-xp has neither.
fn screen_saver_tab(theme: &mut Theme, body: Picture, controls: &mut Vec<Control>) -> Picture {
    let [.., sw, sh] = SCREEN;
    let black = Picture::new(sw, sh).svg(format!(
        r##"<rect width="{sw}" height="{sh}" fill="#000"/>"##
    ));
    let mut body = monitor(theme, body, &black);
    body = dialog::group_box(theme, body, [21, 213, 357, 78], "Screen saver");
    body = dialog::group_box(theme, body, [21, 294, 357, 85], "Monitor power");
    body = dialog::label(body, "Wait:", [30, 263, 26, 13], false);
    body = dialog::label(body, "minutes", [115, 263, 45, 13], false);
    body = dialog::label(
        body,
        "To adjust monitor power settings and save energy,",
        [117, 311, 255, 13],
        true,
    );
    body = dialog::label(body, "click Power.", [117, 324, 255, 13], true);
    let spin = Picture::new(50, 21)
        .svg(dialog::field([0, 0, 50, 21]))
        .svg(dialog::arrow_button([33, 1, 16, 19], true))
        .svg(r##"<rect x="33" y="1" width="16" height="19" fill="#fff" fill-opacity="0.55"/>"##)
        .text(Text::new("10", Font::Ui, dialog::GRAY, [4, 0, 26, 21]));
    controls.extend([
        Control::area(
            "saver",
            [30, 231, 170, 21],
            dialog::combo([170, 21], "(None)", false),
        )
        .enabled(false),
        Control::push("saver-settings", "Settings", [208, 230]).enabled(false),
        Control::push("preview", "Preview", [289, 230]).enabled(false),
        Control::area("wait", [58, 259, 50, 21], spin).enabled(false),
        Control::check(
            "password",
            "On resume, password protect",
            [165, 261, 205, 17],
            false,
        )
        .enabled(false),
        Control::push("power", "Power...", [292, 348]).enabled(false),
    ]);
    body
}

/// What a click on Effects does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effects {
    /// Show the page again.
    Redraw,
    /// Close without a change.
    Close,
    /// Take the effects and close.
    Take(Toggles),
}

/// XP's Effects of Appearance, with the effects of chora-xp: wobbly
/// windows, the magic lamp, the desktop cube and the sounds.
pub struct EffectsDialog {
    /// The window.
    pub dialog: Dialog,
    toggles: Toggles,
}

const EFFECTS_SIZE: [i32; 2] = [345, 168];

impl EffectsDialog {
    /// The dialog for the effects of the session, above a work area.
    pub fn new(toggles: Toggles, area: Rect) -> Self {
        Self {
            dialog: Dialog::new("effects", "Effects", EFFECTS_SIZE, area),
            toggles,
        }
    }

    /// The check boxes and OK and Cancel.
    pub fn page(&self) -> Page {
        let t = self.toggles;
        let row = |i: i32| [13, 14 + i * 24, 320, 17];
        let [w, h] = EFFECTS_SIZE;
        Page {
            body: Some(dialog::face(EFFECTS_SIZE)),
            controls: vec![
                Control::check(
                    "wobbly",
                    "Let windows wobble while they move",
                    row(0),
                    t.wobbly,
                ),
                Control::check(
                    "lamp",
                    "Minimize windows through the magic lamp",
                    row(1),
                    t.lamp,
                ),
                Control::check("cube", "Turn between desktops on a cube", row(2), t.cube),
                Control::check("sounds", "Play Windows XP's sounds", row(3), t.sounds),
                Control::push("ok", "OK", [w - 2 * 73 - 6 - 11, h - 32]).default(),
                Control::push("cancel", "Cancel", [w - 73 - 11, h - 32]),
            ],
        }
    }

    /// A click: a check box flips; OK takes the effects, Cancel and the
    /// close button close without them.
    pub fn click(&mut self, hit: &Hit) -> Effects {
        let Hit::Control(name, _) = hit else {
            return Effects::Close;
        };
        let t = &mut self.toggles;
        match name.as_str() {
            "wobbly" => t.wobbly = !t.wobbly,
            "lamp" => t.lamp = !t.lamp,
            "cube" => t.cube = !t.cube,
            "sounds" => t.sounds = !t.sounds,
            "ok" => return Effects::Take(self.toggles),
            "cancel" => return Effects::Close,
            _ => {}
        }
        Effects::Redraw
    }
}

/// The farthest the view scrolls: what does not fit.
fn max_scroll(height: i32, view: i32) -> i32 {
    (height - view).max(0)
}

/// The thumb of a scroll bar in a track of a height, for a view of a
/// height over content of a height, scrolled by an offset: its top in
/// the track and its height, at least 8 pixels.
fn thumb(track: i32, view: i32, height: i32, offset: i32) -> (i32, i32) {
    let max = max_scroll(height, view);
    if max == 0 || track <= 0 {
        return (0, track.max(0));
    }
    let size = (i64::from(track) * i64::from(view) / i64::from(height.max(1))).max(8);
    let size = i32::try_from(size).unwrap_or(track).min(track);
    let top = i64::from(track - size) * i64::from(offset.clamp(0, max)) / i64::from(max);
    (i32::try_from(top).unwrap_or(0), size)
}

/// The parts of XP's Luna scroll bar in a rectangle: the arrows at the
/// ends, the pale track and the blue thumb, as pictures with their
/// rectangles in the bar.
fn scroll_bar([w, h]: [i32; 2], (top, size): (i32, i32)) -> [(Rect, Picture); 4] {
    let track = Picture::new(w, h - 2 * w).svg(format!(
        r##"<rect width="{w}" height="{}" fill="#FEFEFB"/><rect width="1" height="{}" fill="#F3F1EC"/>"##,
        h - 2 * w,
        h - 2 * w
    ));
    let arrow = |down: bool| {
        let mut svg = dialog::arrow_button([0, 0, w, w], false);
        if !down {
            svg = format!(
                r#"<g transform="rotate(180 {c} {c})">{svg}</g>"#,
                c = f64::from(w) / 2.0
            );
        }
        Picture::new(w, w).svg(svg)
    };
    let thumb = Picture::new(w, size).svg(thumb_svg(w, size));
    [
        ([0, w, w, h - 2 * w], track),
        ([0, w + top, w, size], thumb),
        ([0, 0, w, w], arrow(false)),
        ([0, h - w, w, w], arrow(true)),
    ]
}

/// The blue thumb with its grip of four lines in the middle.
fn thumb_svg(w: i32, h: i32) -> String {
    let mut svg = format!(
        r##"<defs><linearGradient id="t" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#C9DAFC"/><stop offset="1" stop-color="#B1C8FA"/></linearGradient></defs><rect x="0.5" y="0.5" width="{}" height="{}" rx="2" fill="url(#t)" stroke="#9CB6F2"/>"##,
        w - 1,
        h - 1
    );
    if h > 14 {
        let mid = h / 2 - 4;
        for i in 0..4 {
            let _ = write!(
                svg,
                r##"<rect x="5" y="{}" width="7" height="1" fill="#EEF4FE"/><rect x="6" y="{}" width="7" height="1" fill="#8CB0F8"/>"##,
                mid + 2 * i,
                mid + 2 * i + 1
            );
        }
    }
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::tests::{preview, theme};

    fn panel(tab: usize) -> DisplayDialog {
        let walls = (0..9)
            .map(|i| PathBuf::from(format!("/w/wall{i}.png")))
            .collect();
        let screen = Screen {
            output: "WL-1".into(),
            physical: [1600, 1000],
            scale: 125,
        };
        let style = Wallpaper::new("/w/wall3.png".into());
        let art = Art {
            walls,
            ..Art::default()
        };
        DisplayDialog::new(style, screen, tab, [0, 0, 1280, 770], art)
    }

    fn click(p: &mut DisplayDialog, name: &str, at: [i32; 2]) -> Click {
        p.click(&Hit::Control(name.into(), at))
    }

    #[test]
    fn the_thumb_follows_the_offset() {
        // Content of twice the view: the thumb is half the track.
        assert_eq!(thumb(400, 200, 400, 0), (0, 200));
        assert_eq!(thumb(400, 200, 400, 200), (200, 200));
        assert_eq!(thumb(400, 200, 400, 100), (100, 200));
        // Nothing to scroll: the thumb fills the track.
        assert_eq!(thumb(400, 500, 400, 0), (0, 400));
    }

    #[test]
    fn the_list_chooses_the_background_and_apply_takes_it() {
        let mut p = panel(1);
        assert_eq!(p.chosen(), 4);
        // (None) is the first row.
        assert_eq!(click(&mut p, "list", [40, 3]), Click::Redraw);
        assert_eq!(file(&p.shown), None);
        // The second row is the first wallpaper.
        click(&mut p, "list", [40, 1 + ROW + 3]);
        assert_eq!(file(&p.shown), Some(&"/w/wall0.png".into()));
        p.set_position(0);
        p.set_color([0, 0, 0]);
        let Click::Apply {
            style,
            scale,
            close,
        } = click(&mut p, "apply", [1, 1])
        else {
            panic!("apply");
        };
        assert_eq!(style.position, Position::Center);
        assert_eq!((style.color, scale, close), ([0, 0, 0], 125, false));
        assert_eq!(click(&mut p, "cancel", [1, 1]), Click::Close);
    }

    #[test]
    fn the_scroll_bar_scrolls_the_list() {
        let mut p = panel(1);
        // Ten rows, six show.
        assert_eq!((p.top, p.max_top()), (0, 4));
        let bar = LIST[2] - 1 - SCROLL + 3;
        click(&mut p, "list", [bar, LIST[3] - 5]);
        assert_eq!(p.top, 1);
        click(&mut p, "list", [bar, 3]);
        assert_eq!(p.top, 0);
        // A click on the track below the thumb goes a page down, to the
        // end at most.
        click(&mut p, "list", [bar, LIST[3] - SCROLL - 4]);
        assert_eq!(p.top, 4);
    }

    #[test]
    fn the_slider_chooses_the_scale_as_the_resolution() {
        let mut p = panel(4);
        assert_eq!(tick_of(125), 2);
        assert_eq!(p.logical(125), [1280, 800]);
        click(&mut p, "slider", [tick_x(0) + 2, 10]);
        assert_eq!(p.scale, 200);
        assert_eq!(p.logical(p.scale), [800, 500]);
        click(&mut p, "slider", [SLIDER[2], 10]);
        assert_eq!(p.scale, 100);
        let Click::Apply { scale, close, .. } = click(&mut p, "ok", [1, 1]) else {
            panic!("ok");
        };
        assert_eq!((scale, close), (100, true));
    }

    #[test]
    fn the_effects_change_on_ok() {
        let mut e = EffectsDialog::new(crate::config::defaults().effects, [0, 0, 800, 600]);
        let hit = |n: &str| Hit::Control(n.into(), [1, 1]);
        assert_eq!(e.click(&hit("cube")), Effects::Redraw);
        let Effects::Take(t) = e.click(&hit("ok")) else {
            panic!("ok");
        };
        assert!(!t.cube && t.wobbly);
        assert_eq!(e.click(&Hit::Close), Effects::Close);
    }

    #[test]
    fn the_tabs_look_like_xp() {
        let (mut theme, cache) = theme("display");
        let bliss = chora_kit::xdg::data_file("wallpapers/WindowsXP/bliss.jpg")
            .unwrap_or_else(|| theme.path("wallpaper/bloff.png"));
        let mut walls = vec![bliss.clone()];
        walls.extend((0..8).map(|i| PathBuf::from(format!("/w/wall-{i}.png"))));
        let screen = Screen {
            output: "WL-1".into(),
            physical: [800, 600],
            scale: 100,
        };
        let mut style = Wallpaper::new(bliss);
        style.position = Position::Stretch;
        let art = Art {
            walls,
            ..Art::default()
        };
        let mut p = DisplayDialog::new(style, screen, 0, [0, 0, 1280, 770], art);
        for (tab, name) in TABS.iter().enumerate() {
            p.tab = tab;
            let page = p.page(&mut theme);
            p.dialog.create(&mut theme, "root", page);
            preview(
                &mut theme,
                &p.dialog,
                &format!("display-{}", name.replace(' ', "")),
            );
        }
        let mut e = EffectsDialog::new(crate::config::defaults().effects, [0, 0, 800, 600]);
        let page = e.page();
        e.dialog.create(&mut theme, "root", page);
        preview(&mut theme, &e.dialog, "effects");
        let _ = std::fs::remove_dir_all(cache);
    }
}
