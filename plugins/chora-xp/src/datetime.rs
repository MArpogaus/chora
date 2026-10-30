//! XP's Date and Time Properties: the tab Date & Time with the month and
//! the year, the month calendar with today in blue, the analog clock
//! with its moving hands and the digital time, and the tab Time Zone with
//! the zone of the system. chora cannot set the clock of the system
//! without privileges, so OK closes the dialog without a change and
//! Apply stays gray. The sizes follow XP's dialog of 404 by 348 pixels.

use std::{fmt::Write as _, path::Path};

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

use crate::{
    context::{Act, Entry},
    dialog::{self, Control, Dialog, Hit, Page},
    text::{Font, Text},
    theme::{Picture, Rect, Theme},
};

/// The client area.
const SIZE: [i32; 2] = [398, 316];
const PAGE: Rect = [6, 27, 384, 251];
const TABS: [&str; 2] = ["Date & Time", "Time Zone"];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTH: Rect = [27, 57, 75, 21];
const YEAR: Rect = [112, 57, 63, 21];
const YEAR_SPIN: Rect = [176, 57, 16, 21];
const CALENDAR: Rect = [27, 89, 162, 128];
const CLOCK: Rect = [222, 51, 140, 140];
const TIME: Rect = [245, 200, 108, 20];
/// The rows of the calendar.
const HEADER: i32 = 18;
const CELL_HEIGHT: i32 = 18;
/// The selection of XP.
const BLUE: &str = "#316AC5";
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// The days of a month.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first = |y, m| NaiveDate::from_ymd_opt(y, m, 1);
    match (first(year, month), first(y, m)) {
        (Some(a), Some(b)) => u32::try_from((b - a).num_days()).unwrap_or(30),
        _ => 30,
    }
}

/// The cells of a month in XP's calendar: six weeks of seven days from
/// Sunday, as XP shows them in English, with the day in each cell.
pub fn grid(year: i32, month: u32) -> [[Option<u32>; 7]; 6] {
    let offset =
        NaiveDate::from_ymd_opt(year, month, 1).map_or(0, |d| d.weekday().num_days_from_sunday());
    let days = days_in_month(year, month);
    let mut cells = [[None; 7]; 6];
    for day in 1..=days {
        let n = (offset + day - 1) as usize;
        if let Some(cell) = cells.get_mut(n / 7).and_then(|row| row.get_mut(n % 7)) {
            *cell = Some(day);
        }
    }
    cells
}

/// The angles of the hour, minute and second hands, in degrees
/// clockwise from 12.
pub fn hands(hour: u32, minute: u32, second: u32) -> [f64; 3] {
    let [h, m, s] = [hour % 12, minute, second].map(f64::from);
    [h * 30.0 + m * 0.5 + s / 120.0, m * 6.0 + s * 0.1, s * 6.0]
}

/// The time zone of the system: `TZ`, else the link `/etc/localtime`.
fn zone() -> String {
    if let Ok(tz) = std::env::var("TZ")
        && !tz.is_empty()
    {
        return tz.trim_start_matches(':').to_string();
    }
    std::fs::read_link("/etc/localtime")
        .ok()
        .and_then(|link| zone_of(&link))
        .unwrap_or_else(|| "UTC".into())
}

/// The zone name in the path of a zoneinfo file.
fn zone_of(link: &Path) -> Option<String> {
    let link = link.to_string_lossy();
    let (_, name) = link.split_once("zoneinfo/")?;
    Some(name.to_string())
}

/// What a click asks the window manager to do.
#[derive(Clone, Debug, PartialEq)]
pub enum Click {
    /// Show the page again.
    Redraw,
    /// Close the dialog.
    Close,
    /// Open the month list below a rectangle of the client area.
    Months(Rect, Vec<Entry>),
}

/// The open dialog, with the month that it shows and the chosen day.
pub struct DateDialog {
    /// The window.
    pub dialog: Dialog,
    tab: usize,
    year: i32,
    month: u32,
    day: u32,
}

impl DateDialog {
    /// The dialog for today, in the middle of a work area.
    pub fn new(now: NaiveDateTime, area: Rect) -> Self {
        Self {
            dialog: Dialog::new("datetime", "Date and Time Properties", SIZE, area),
            tab: 0,
            year: now.year(),
            month: now.month(),
            day: now.day(),
        }
    }

    /// The page of the current tab at a time.
    pub fn page(&self, theme: &Theme, now: NaiveDateTime) -> Page {
        let mut body = dialog::face(SIZE).svg(dialog::tab_page(PAGE));
        let mut controls = dialog::tabs(theme, [PAGE[0], PAGE[1]], &TABS, self.tab);
        if self.tab == 0 {
            body = dialog::group_box(theme, body, [16, 33, 183, 202], "Date");
            body = dialog::group_box(theme, body, [208, 33, 170, 202], "Time");
            body = body.svg(dialog::field(YEAR)).text(Text::new(
                &self.year.to_string(),
                Font::Ui,
                dialog::BLACK,
                [YEAR[0] + 5, YEAR[1], YEAR[2] - 6, YEAR[3]],
            ));
            body = dialog::label(
                body,
                &format!("Current time zone:  {}", zone()),
                [16, 250, 360, 13],
                true,
            );
            controls.extend([
                Control::area("month", MONTH, self.month_box()),
                Control::area("year", YEAR_SPIN, spin(YEAR_SPIN)),
                Control::area("calendar", CALENDAR, self.calendar()),
                Control::area("clock", CLOCK, clock_face(now)).enabled(false),
                Control::area("time", TIME, time_box(now)).enabled(false),
            ]);
        } else {
            let offset = chrono::Local::now().format("%:z").to_string();
            let line = format!("(GMT{offset}) {}", zone());
            let combo = [13, 40, 370, 21];
            body = body
                .svg(dialog::field(combo))
                .svg(dialog::arrow_button(
                    [combo[0] + combo[2] - 18, 42, 16, 17],
                    false,
                ))
                .text(Text::new(&line, Font::Ui, dialog::BLACK, [18, 40, 330, 21]));
            controls.push(
                Control::check(
                    "dst",
                    "Automatically adjust clock for daylight saving changes",
                    [16, 250, 360, 17],
                    true,
                )
                .enabled(false),
            );
        }
        controls.extend([
            Control::push("ok", "OK", [156, 287]).default(),
            Control::push("cancel", "Cancel", [237, 287]),
            Control::push("apply", "Apply", [318, 287]).enabled(false),
        ]);
        Page {
            body: Some(body),
            controls,
        }
    }

    /// The month in its drop-down field, selected as XP shows it.
    fn month_box(&self) -> Picture {
        let [_, _, w, h] = MONTH;
        let name = MONTHS[(self.month - 1) as usize];
        Picture::new(w, h)
            .svg(dialog::field([0, 0, w, h]))
            .svg(format!(
                r#"<rect x="3" y="3" width="{}" height="{}" fill="{BLUE}"/>"#,
                w - 22,
                h - 6
            ))
            .svg(dialog::arrow_button([w - 18, 2, 16, h - 4], false))
            .text(Text::new(name, Font::Ui, WHITE, [5, 0, w - 26, h]))
    }

    /// The month calendar: a sunken frame, the blue row of the weekdays
    /// and the days, the chosen one in XP's blue.
    fn calendar(&self) -> Picture {
        let [_, _, w, h] = CALENDAR;
        let [iw, ih] = [w - 4, h - 4];
        let mut svg = format!(
            r##"<rect width="{w}" height="{h}" fill="#fff"/><path d="M0.5 {h}V0.5H{w}" fill="none" stroke="#ACA899"/><path d="M1.5 {}V1.5H{}" fill="none" stroke="#716F64"/><path d="M{} 1V{}H1" fill="none" stroke="#FFFFFF"/><path d="M{} 2V{}H2" fill="none" stroke="#F1EFE2"/><rect x="2" y="2" width="{iw}" height="{HEADER}" fill="#7A96DF"/>"##,
            h - 1,
            w - 1,
            f64::from(w) - 0.5,
            f64::from(h) - 0.5,
            f64::from(w) - 1.5,
            f64::from(h) - 1.5,
        );
        let mut texts = Vec::new();
        for (i, letter) in ["S", "M", "T", "W", "T", "F", "S"].iter().enumerate() {
            texts.push(
                Text::new(
                    letter,
                    Font::Ui,
                    [216, 228, 248, 255],
                    cell(i, 0, iw, -HEADER),
                )
                .centered(),
            );
        }
        for (row, week) in grid(self.year, self.month).iter().enumerate() {
            for (col, day) in week.iter().enumerate() {
                let Some(day) = day else { continue };
                let [x, y, cw, ch] = cell(col, row, iw, 0);
                let chosen = *day == self.day;
                if chosen {
                    let _ = write!(
                        svg,
                        r#"<rect x="{}" y="{}" width="14" height="13" fill="{BLUE}"/>"#,
                        x + cw / 2 - 7,
                        y + (ch - 13) / 2
                    );
                }
                let color = if chosen { WHITE } else { dialog::BLACK };
                texts.push(Text::new(&day.to_string(), Font::Ui, color, [x, y, cw, ch]).centered());
            }
        }
        let _ = ih;
        let mut picture = Picture::new(w, h).svg(svg);
        for text in texts {
            picture = picture.text(text);
        }
        picture
    }

    /// The day under a point of the calendar.
    fn day_at(&self, [px, py]: [i32; 2]) -> Option<u32> {
        let [_, _, w, _] = CALENDAR;
        let iw = w - 4;
        let row = usize::try_from((py - 2 - HEADER).div_euclid(CELL_HEIGHT)).ok()?;
        let col = usize::try_from((px - 2) * 7 / iw).ok()?;
        *grid(self.year, self.month).get(row)?.get(col)?
    }

    /// A click on a control.
    pub fn click(&mut self, hit: &Hit) -> Click {
        let Hit::Control(name, point) = hit else {
            return Click::Close;
        };
        match name.as_str() {
            "tab-0" => self.tab = 0,
            "tab-1" => self.tab = 1,
            "month" => {
                let entries = MONTHS
                    .iter()
                    .zip(1..)
                    .map(|(m, i)| Entry::item(m, Act::Month(i)).checked(i == self.month))
                    .collect();
                return Click::Months(MONTH, entries);
            }
            "year" => {
                let up = point[1] < YEAR_SPIN[3] / 2;
                self.year = (self.year + if up { 1 } else { -1 }).clamp(1980, 2099);
                self.clamp_day();
            }
            "calendar" => {
                if let Some(day) = self.day_at(*point) {
                    self.day = day;
                }
            }
            "ok" | "cancel" => return Click::Close,
            _ => {}
        }
        Click::Redraw
    }

    /// Shows a month of the month list.
    pub fn set_month(&mut self, month: u32) {
        self.month = month.clamp(1, 12);
        self.clamp_day();
    }

    fn clamp_day(&mut self) {
        self.day = self.day.min(days_in_month(self.year, self.month));
    }

    /// The point of the client area where the month list opens, in the
    /// scene, and its width.
    pub fn list_at(&self, [x, y, w, h]: Rect) -> ([i32; 2], i32) {
        let [cx, cy] = self.dialog.client();
        ([cx + x, cy + y + h], w)
    }
}

/// The cell of a column and a row of the calendar in a width; `top`
/// moves it, as for the row of the weekdays.
fn cell(col: usize, row: usize, width: i32, top: i32) -> Rect {
    let [c, r] = [col, row].map(|n| i32::try_from(n).unwrap_or(0));
    let x = 2 + c * width / 7;
    let next = 2 + (c + 1) * width / 7;
    [x, 2 + HEADER + top + r * CELL_HEIGHT, next - x, CELL_HEIGHT]
}

/// The two arrow buttons of a spin box of a size.
fn spin([_, _, w, h]: Rect) -> Picture {
    Picture::new(w, h).svg(dialog::arrow_button([0, 0, w, h], true))
}

/// The digital time in its field, with the spin box at the right.
fn time_box(now: NaiveDateTime) -> Picture {
    let [_, _, w, h] = TIME;
    let field = w - 18;
    Picture::new(w, h)
        .svg(dialog::field([0, 0, field, h]))
        .svg(dialog::arrow_button([field + 1, 0, 17, h], true))
        .text(Text::new(
            &now.format("%-I:%M:%S %p").to_string(),
            Font::Ui,
            dialog::BLACK,
            [4, 0, field - 6, h],
        ))
}

/// XP's analog clock: gray dots for the minutes, turquoise squares for
/// the hours, the turquoise hour and minute hands with their gray shadow,
/// and the second hand, which inverts what lies below it.
fn clock_face(now: NaiveDateTime) -> Picture {
    let [_, top, w, h] = CLOCK;
    let c = f64::from(w) / 2.0;
    // The page grows gray to the bottom; the clock carries its piece of
    // it, so that the second hand can invert it.
    let shade = |y: i32| {
        let t = f64::from(y - PAGE[1]) / f64::from(PAGE[3]);
        let mix = |from: f64, to: f64| (from + (to - from) * t).round();
        format!(
            "rgb({},{},{})",
            mix(252.0, 244.0),
            mix(252.0, 243.0),
            mix(254.0, 238.0)
        )
    };
    let mut svg = format!(
        r#"<defs><linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{}"/><stop offset="1" stop-color="{}"/></linearGradient></defs><rect width="{w}" height="{h}" fill="url(#bg)"/>"#,
        shade(top),
        shade(top + h)
    );
    svg += &dial(c, 61.0);
    let [hour, minute, second] = hands(now.hour(), now.minute(), now.second());
    for (angle, length) in [(hour, 35.0), (minute, 47.0)] {
        // A kite, widest near the middle, with a short tail.
        let kite = format!(
            "M0 -{length}L3 -{:.1}L0 5L-3 -{:.1}Z",
            length * 0.18,
            length * 0.18
        );
        let _ = write!(
            svg,
            r##"<path d="{kite}" fill="#ACA899" transform="translate({} {}) rotate({angle})"/><path d="{kite}" fill="#008080" transform="translate({c} {c}) rotate({angle})"/>"##,
            c + 2.0,
            c + 2.0
        );
    }
    let _ = write!(
        svg,
        r##"<g style="mix-blend-mode:difference"><path d="M0 0V-48" stroke="#fff" stroke-width="1.2" transform="translate({c} {c}) rotate({second})"/></g>"##
    );
    Picture::new(w, h).svg(svg)
}

/// The dial of a radius around a center: a gray dot for each minute and
/// a turquoise square for each hour.
fn dial(center: f64, radius: f64) -> String {
    let mut svg = String::new();
    for minute in 0..60 {
        let angle = f64::from(minute * 6).to_radians();
        let x = (center + radius * angle.sin()).round();
        let y = (center - radius * angle.cos()).round();
        if minute % 5 == 0 {
            let _ = write!(
                svg,
                r##"<rect x="{}" y="{}" width="3" height="3" fill="#000"/><rect x="{}" y="{}" width="3" height="3" fill="#00FFFF"/><rect x="{x}" y="{y}" width="2" height="2" fill="#008080" transform="translate(-1 -1)"/>"##,
                x - 1.0,
                y - 1.0,
                x - 2.0,
                y - 2.0,
            );
        } else {
            let _ = write!(
                svg,
                r##"<rect x="{}" y="{}" width="2" height="2" fill="#ACA899"/><rect x="{x}" y="{y}" width="1" height="1" fill="#FFFFFF"/>"##,
                x - 1.0,
                y - 1.0,
            );
        }
    }
    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialog::tests::{preview, theme};

    #[test]
    fn months_have_their_days_and_leap_years_a_29th_of_february() {
        assert_eq!(days_in_month(2003, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
    }

    #[test]
    fn weeks_start_on_sunday() {
        // April 2003 of XP's screenshot starts on a Tuesday.
        let april = grid(2003, 4);
        assert_eq!(
            april[0],
            [None, None, Some(1), Some(2), Some(3), Some(4), Some(5)]
        );
        assert_eq!(april[2][2], Some(15));
        assert_eq!(april[4][3], Some(30));
        assert_eq!(april[5], [None; 7]);
        // March 2026 starts on a Sunday; August 2026 on a Saturday and
        // needs the sixth week.
        assert_eq!(grid(2026, 3)[0][0], Some(1));
        let august = grid(2026, 8);
        assert_eq!(august[0][6], Some(1));
        assert_eq!(august[5][1], Some(31));
    }

    #[test]
    fn the_hands_point_at_the_time() {
        assert_eq!(hands(3, 0, 0), [90.0, 0.0, 0.0]);
        assert_eq!(hands(15, 30, 0), [105.0, 180.0, 0.0]);
        let [h, m, s] = hands(11, 41, 57);
        assert!((h - 350.975).abs() < 1e-9 && (m - 251.7).abs() < 1e-9);
        assert_eq!(s, 342.0);
    }

    #[test]
    fn a_click_on_the_calendar_chooses_the_day() {
        let april = NaiveDate::from_ymd_opt(2003, 4, 15)
            .and_then(|d| d.and_hms_opt(11, 41, 57))
            .expect("a date");
        let mut panel = DateDialog::new(april, [0, 0, 1280, 770]);
        // The cell of the 30th: the fifth row, the fourth column.
        let [x, y, ..] = cell(3, 4, CALENDAR[2] - 4, 0);
        assert_eq!(panel.day_at([x + 3, y + 3]), Some(30));
        assert_eq!(panel.day_at([5, 25]), None);
        panel.click(&Hit::Control("calendar".into(), [x + 3, y + 3]));
        assert_eq!(panel.day, 30);
        // February has no 30th.
        panel.set_month(2);
        assert_eq!(panel.day, 28);
        assert_eq!(
            zone_of(Path::new("/usr/share/zoneinfo/Europe/Zurich")).as_deref(),
            Some("Europe/Zurich")
        );
    }

    #[test]
    fn the_dialog_looks_like_xp() {
        let (mut theme, cache) = theme("datetime");
        let april = NaiveDate::from_ymd_opt(2003, 4, 15)
            .and_then(|d| d.and_hms_opt(11, 41, 57))
            .expect("a date");
        let mut panel = DateDialog::new(april, [0, 0, 1280, 770]);
        for (tab, name) in [(0, "datetime"), (1, "timezone")] {
            panel.tab = tab;
            let page = panel.page(&theme, april);
            panel.dialog.create(&mut theme, "root", page);
            preview(&mut theme, &panel.dialog, name);
        }
        let _ = std::fs::remove_dir_all(cache);
    }
}
