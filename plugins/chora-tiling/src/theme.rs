//! The palette of the look: Ink (dark) and Paper (light), the choice
//! between them, and the files of the flavor that carry the same colours
//! (the panel, the launcher, the terminal and the lock screen).

use std::{fmt::Write as _, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Which palette the look takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Paper when the system asks for the light colour scheme, else Ink.
    #[default]
    System,
    /// Dark.
    Ink,
    /// Light.
    Paper,
}

/// A colour scheme: the two palettes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheme {
    Ink,
    Paper,
}

impl Scheme {
    /// The name that the files of the flavor use.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Scheme::Ink => "ink",
            Scheme::Paper => "paper",
        }
    }
}

impl Theme {
    /// The scheme that the setting gives; `system` is what the system
    /// says, or Ink when it says nothing.
    #[must_use]
    pub fn scheme(self, system: Option<Scheme>) -> Scheme {
        match self {
            Theme::Ink => Scheme::Ink,
            Theme::Paper => Scheme::Paper,
            Theme::System => system.unwrap_or(Scheme::Ink),
        }
    }
}

/// The colours of a look, each `#rrggbb`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Colors {
    pub base: String,
    pub surface: String,
    pub surface_2: String,
    pub line: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub on_accent: String,
    pub accent_2: String,
    pub urgent: String,
    pub terminal: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
}

/// The names of the colours, in the order of [`Colors::values`].
pub const NAMES: [&str; 14] = [
    "base",
    "surface",
    "surface-2",
    "line",
    "text",
    "muted",
    "accent",
    "on-accent",
    "accent-2",
    "urgent",
    "terminal",
    "green",
    "yellow",
    "blue",
];

impl Colors {
    /// The palette of a scheme.
    #[must_use]
    pub fn of(scheme: Scheme) -> Self {
        let make = |v: [&str; 14]| {
            let [
                base,
                surface,
                surface_2,
                line,
                text,
                muted,
                accent,
                on_accent,
                accent_2,
                urgent,
                terminal,
                green,
                yellow,
                blue,
            ] = v.map(str::to_owned);
            Self {
                base,
                surface,
                surface_2,
                line,
                text,
                muted,
                accent,
                on_accent,
                accent_2,
                urgent,
                terminal,
                green,
                yellow,
                blue,
            }
        };
        match scheme {
            Scheme::Ink => make([
                "#11131a", "#1a1d27", "#242836", "#3a4054", "#e6e8ef", "#9aa1b4", "#a78bfa",
                "#11131a", "#5eead4", "#f87171", "#161922", "#86e0a0", "#f0cd6e", "#7aa2f7",
            ]),
            Scheme::Paper => make([
                "#e9ebf1", "#ffffff", "#f2f3f7", "#d5d8e1", "#1b1e27", "#5b6274", "#6d4aff",
                "#ffffff", "#0e9f8e", "#d33f3f", "#ffffff", "#1f7a43", "#8a5a00", "#2f5bd8",
            ]),
        }
    }

    /// The colours in the order of [`NAMES`].
    #[must_use]
    pub fn values(&self) -> [&str; 14] {
        [
            &self.base,
            &self.surface,
            &self.surface_2,
            &self.line,
            &self.text,
            &self.muted,
            &self.accent,
            &self.on_accent,
            &self.accent_2,
            &self.urgent,
            &self.terminal,
            &self.green,
            &self.yellow,
            &self.blue,
        ]
    }

    /// The colour with a name of [`NAMES`] for a setting.
    pub fn set(&mut self, name: &str, value: &str) {
        let slot = match name {
            "base" => &mut self.base,
            "surface" => &mut self.surface,
            "surface_2" => &mut self.surface_2,
            "line" => &mut self.line,
            "text" => &mut self.text,
            "muted" => &mut self.muted,
            "accent" => &mut self.accent,
            "on_accent" => &mut self.on_accent,
            "accent_2" => &mut self.accent_2,
            "urgent" => &mut self.urgent,
            "terminal" => &mut self.terminal,
            "green" => &mut self.green,
            "yellow" => &mut self.yellow,
            "blue" => &mut self.blue,
            _ => return,
        };
        value.clone_into(slot);
    }
}

/// A `#rrggbb` as channels from 0 to 1, or `None` for any other text.
#[must_use]
pub fn rgb(text: &str) -> Option<[f64; 3]> {
    let digits = text.strip_prefix('#').filter(|d| d.len() == 6)?;
    let value = u32::from_str_radix(digits, 16).ok()?;
    Some([16, 8, 0].map(|shift| f64::from((value >> shift) & 255) / 255.0))
}

/// The channels of a colour that was checked; black for a bad one.
#[must_use]
pub fn channels(text: &str) -> [f64; 3] {
    rgb(text).unwrap_or_default()
}

/// The relative luminance of WCAG 2.
#[must_use]
#[cfg(test)]
pub fn luminance(text: &str) -> f64 {
    let [r, g, b] = channels(text).map(|c| {
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// The contrast ratio of two colours, from 1 to 21.
#[must_use]
#[cfg(test)]
pub fn contrast(a: &str, b: &str) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// The colour scheme that the system asks for, from the answer of the
/// settings portal (`v u 1`: 1 dark, 2 light, 0 no preference).
#[must_use]
pub fn scheme_of_portal(answer: &str) -> Option<Scheme> {
    match answer.trim().rsplit(' ').next()? {
        "1" => Some(Scheme::Ink),
        "2" => Some(Scheme::Paper),
        _ => None,
    }
}

/// The colour scheme from the answer of `gsettings get
/// org.gnome.desktop.interface color-scheme`.
#[must_use]
pub fn scheme_of_gsettings(answer: &str) -> Option<Scheme> {
    match answer.trim().trim_matches('\'') {
        "prefer-dark" => Some(Scheme::Ink),
        "prefer-light" => Some(Scheme::Paper),
        _ => None,
    }
}

/// What the system asks for: the settings portal, else gsettings; `None`
/// when neither answers or the person has no preference.
#[must_use]
pub fn system_scheme() -> Option<Scheme> {
    let ask = |program: &str, args: &[&str]| {
        let out = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let portal = ask(
        "busctl",
        &[
            "--user",
            "--timeout=2",
            "call",
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Settings",
            "ReadOne",
            "ss",
            "org.freedesktop.appearance",
            "color-scheme",
        ],
    )
    .and_then(|a| scheme_of_portal(&a));
    portal.or_else(|| {
        ask(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "color-scheme"],
        )
        .and_then(|a| scheme_of_gsettings(&a))
    })
}

fn bare(hex: &str) -> &str {
    hex.trim_start_matches('#')
}

/// The files of the flavor that carry the palette, by name: the colours
/// of the panel, the launcher, the terminal and the lock screen.
#[must_use]
pub fn files(colors: &Colors) -> Vec<(&'static str, String)> {
    let mut css = String::from(
        "/* The palette of the tiling flavor. One accent; accent-2 is only the far end\n \
         * of the focus gradient. Written by chora-tiling --apply-theme from the\n \
         * theme and the palette of chora-tiling.yaml; change them there. */\n",
    );
    for (name, value) in NAMES.iter().zip(colors.values()) {
        let _ = writeln!(css, "@define-color {name} {value};");
    }
    vec![
        ("colors.css", css),
        ("fuzzel.ini", fuzzel(colors)),
        ("foot.ini", foot(colors)),
        ("swaylock.conf", swaylock(colors)),
        ("icons/apps.svg", icon(&colors.text, APPS)),
        ("icons/keys.svg", icon(&colors.muted, KEYS_ICON)),
        ("icons/power.svg", icon(&colors.muted, POWER)),
    ]
}

/// The shapes of the icons of the panel on a 16 px grid: a 2 x 2 grid of
/// tiles, a keyboard and a power sign. The panel draws them as
/// backgrounds, so they carry their colour, which changes with the theme.
const APPS: &str = r#"<g fill="C"><rect x="1.5" y="1.5" width="5.5" height="5.5" rx="1.5"/><rect x="9" y="1.5" width="5.5" height="5.5" rx="1.5"/><rect x="1.5" y="9" width="5.5" height="5.5" rx="1.5"/><rect x="9" y="9" width="5.5" height="5.5" rx="1.5"/></g>"#;
const KEYS_ICON: &str = r#"<rect x="1" y="3.5" width="14" height="9" rx="2.5" fill="none" stroke="C" stroke-width="1.5"/><g fill="C"><rect x="3.5" y="6" width="1.5" height="1.5" rx=".4"/><rect x="7.25" y="6" width="1.5" height="1.5" rx=".4"/><rect x="11" y="6" width="1.5" height="1.5" rx=".4"/><rect x="4.5" y="9" width="7" height="1.5" rx=".5"/></g>"#;
const POWER: &str = r#"<path d="M4.6 4.2a5.5 5.5 0 1 0 6.8 0M8 1.5v6" fill="none" stroke="C" stroke-width="1.5" stroke-linecap="round"/>"#;

fn icon(color: &str, shapes: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\">{}</svg>\n",
        shapes.replace("\"C\"", &format!("\"{color}\""))
    )
}

fn fuzzel(c: &Colors) -> String {
    let [surface, text, accent, surface_2, line] =
        [&c.surface, &c.text, &c.accent, &c.surface_2, &c.line].map(|x| format!("{}ff", bare(x)));
    format!(
        "# The launcher of the tiling flavor, themed and opaque. Written by\n\
         # chora-tiling --apply-theme from the palette of chora-tiling.yaml.\n\
         [main]\n\
         font=Inter:size=14\n\
         prompt=\"  \"\n\
         width=36\n\
         lines=9\n\
         horizontal-pad=20\n\
         vertical-pad=16\n\
         inner-pad=8\n\
         line-height=28\n\
         icons-enabled=no\n\
         \n\
         [colors]\n\
         background={surface}\n\
         text={text}\n\
         match={accent}\n\
         selection={surface_2}\n\
         selection-text={text}\n\
         selection-match={accent}\n\
         border={line}\n\
         \n\
         [border]\n\
         width=1\n\
         radius=12\n"
    )
}

fn foot(c: &Colors) -> String {
    let b = bare;
    format!(
        "# The terminal of the tiling flavor: JetBrains Mono, padding inside\n\
         # the rounded corners and the palette. Written by chora-tiling\n\
         # --apply-theme from the palette of chora-tiling.yaml.\n\
         [main]\n\
         font=JetBrains Mono:size=10.5\n\
         pad=16x16\n\
         \n\
         [cursor]\n\
         color={bg} {accent}\n\
         \n\
         [colors]\n\
         background={bg}\n\
         foreground={text}\n\
         selection-foreground={text}\n\
         selection-background={selection}\n\
         regular0={surface_2}\n\
         regular1={urgent}\n\
         regular2={green}\n\
         regular3={yellow}\n\
         regular4={blue}\n\
         regular5={accent}\n\
         regular6={accent_2}\n\
         regular7={text}\n\
         bright0={muted}\n\
         bright1={urgent}\n\
         bright2={green}\n\
         bright3={yellow}\n\
         bright4={blue}\n\
         bright5={accent}\n\
         bright6={accent_2}\n\
         bright7={text}\n",
        bg = b(&c.terminal),
        text = b(&c.text),
        accent = b(&c.accent),
        accent_2 = b(&c.accent_2),
        selection = b(&c.line),
        surface_2 = b(&c.surface_2),
        urgent = b(&c.urgent),
        green = b(&c.green),
        yellow = b(&c.yellow),
        blue = b(&c.blue),
        muted = b(&c.muted),
    )
}

fn swaylock(c: &Colors) -> String {
    let b = bare;
    format!(
        "# The lock screen of the tiling flavor. Written by chora-tiling\n\
         # --apply-theme from the palette of chora-tiling.yaml.\n\
         color={base}\n\
         font=Inter\n\
         indicator-radius=72\n\
         indicator-thickness=6\n\
         inside-color={surface}\n\
         inside-clear-color={surface}\n\
         inside-ver-color={surface}\n\
         inside-wrong-color={surface}\n\
         ring-color={line}\n\
         ring-clear-color={accent_2}\n\
         ring-ver-color={accent}\n\
         ring-wrong-color={urgent}\n\
         key-hl-color={accent}\n\
         bs-hl-color={urgent}\n\
         line-color=00000000\n\
         separator-color=00000000\n\
         text-color={text}\n\
         text-clear-color={text}\n\
         text-ver-color={text}\n\
         text-wrong-color={urgent}\n",
        base = b(&c.base),
        surface = b(&c.surface),
        line = b(&c.line),
        accent = b(&c.accent),
        accent_2 = b(&c.accent_2),
        urgent = b(&c.urgent),
        text = b(&c.text),
    )
}

/// Writes the files of [`files`] and the name of the scheme into a
/// directory, which it creates.
///
/// # Errors
///
/// A file cannot be written.
pub fn apply(dir: &Path, colors: &Colors, scheme: Scheme) -> std::io::Result<()> {
    for (name, text) in files(colors) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)?;
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("scheme"), format!("{}\n", scheme.name()))
}

/// The directory that [`apply`] writes by default: `chora-tiling/theme`
/// in the runtime directory of the user, written again at each start.
#[must_use]
pub fn default_dir() -> Option<std::path::PathBuf> {
    let runtime = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?);
    runtime
        .is_absolute()
        .then(|| runtime.join("chora-tiling").join("theme"))
}

/// The directories that hold the config of a program, first the one
/// that wins: the copy of the user in `chora/tiling` of the config
/// directory, the files that [`apply`] wrote, and the flavor.
#[must_use]
pub fn config_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".config")));
    dirs.extend(config.map(|c| c.join("chora").join("tiling")));
    dirs.extend(default_dir());
    let flavor = std::env::var_os("CHORA_FLAVOR_DIR")
        .map_or_else(|| "/usr/share/chora/flavors/tiling".into(), Into::into);
    dirs.push(flavor);
    dirs
}

/// The command line of a program of the flavor with its config file:
/// `foot -c FILE`, `fuzzel --config FILE`, `swaylock -C FILE` and `swaybg
/// -i WALLPAPER -m fill`, `screenshot` and `power` as `sh screenshot.sh`
/// and `sh power.sh` of the flavor, then the arguments of the caller; the program
/// alone when no file exists. The wallpaper of a scheme is
/// `wallpaper.png` for Ink and `wallpaper-paper.png` for Paper; a
/// `wallpaper.png` of the user, found first, serves both.
#[must_use]
pub fn launch_line(
    program: &str,
    rest: &[String],
    dirs: &[std::path::PathBuf],
    scheme: Scheme,
) -> Vec<String> {
    let find = |names: &[&str]| {
        dirs.iter()
            .flat_map(|d| names.iter().map(move |n| d.join(n)))
            .find(|f| f.is_file())
            .map(|f| f.to_string_lossy().into_owned())
    };
    let mut line = vec![program.to_owned()];
    match program {
        "foot" => line.extend(
            find(&["foot.ini"])
                .into_iter()
                .flat_map(|f| ["-c".into(), f]),
        ),
        "fuzzel" => line.extend(
            find(&["fuzzel.ini"])
                .into_iter()
                .flat_map(|f| ["--config".into(), f]),
        ),
        "swaylock" => line.extend(
            find(&["swaylock.conf"])
                .into_iter()
                .flat_map(|f| ["-C".into(), f]),
        ),
        "screenshot" | "power" => {
            let name = format!("{program}.sh");
            line = find(&[&name])
                .map_or_else(|| vec!["true".to_owned()], |f| vec!["sh".to_owned(), f]);
        }
        "swaybg" => {
            let names = [scheme_wallpaper(scheme), "wallpaper.png"];
            match find(&names) {
                Some(file) => line.extend(["-i".into(), file, "-m".into(), "fill".into()]),
                None => line.extend(["-c".into(), Colors::of(scheme).base]),
            }
        }
        _ => {}
    }
    line.extend(rest.iter().cloned());
    line
}

/// The file name of the wallpaper of a scheme.
fn scheme_wallpaper(scheme: Scheme) -> &'static str {
    match scheme {
        Scheme::Ink => "wallpaper.png",
        Scheme::Paper => "wallpaper-paper.png",
    }
}

/// The scheme that the files of [`apply`] name, or Ink.
#[must_use]
pub fn written_scheme(dirs: &[std::path::PathBuf]) -> Scheme {
    let name = dirs
        .iter()
        .find_map(|d| std::fs::read_to_string(d.join("scheme")).ok());
    match name.as_deref().map(str::trim) {
        Some("paper") => Scheme::Paper,
        _ => Scheme::Ink,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LABELS: [(&str, &str); 4] = [
        ("text", "surface"),
        ("muted", "surface"),
        ("accent", "surface"),
        ("urgent", "surface"),
    ];

    fn pick<'a>(c: &'a Colors, name: &str) -> &'a str {
        let at = NAMES.iter().position(|n| n.replace('-', "_") == name);
        c.values()[at.unwrap()]
    }

    #[test]
    fn every_label_of_both_palettes_reads_at_4_5_to_1() {
        for scheme in [Scheme::Ink, Scheme::Paper] {
            let c = Colors::of(scheme);
            for (fg, bg) in LABELS {
                let ratio = contrast(pick(&c, fg), pick(&c, bg));
                assert!(ratio >= 4.5, "{scheme:?} {fg} on {bg}: {ratio:.2}");
            }
            let on = contrast(&c.on_accent, &c.accent);
            assert!(on >= 4.5, "{scheme:?} on-accent on accent: {on:.2}");
            for ansi in [&c.green, &c.yellow, &c.blue, &c.urgent, &c.text, &c.muted] {
                let ratio = contrast(ansi, &c.terminal);
                assert!(
                    ratio >= 4.5,
                    "{scheme:?} {ansi} on the terminal: {ratio:.2}"
                );
            }
        }
    }

    #[test]
    fn the_system_decides_only_when_the_theme_says_so() {
        assert_eq!(Theme::System.scheme(None), Scheme::Ink);
        assert_eq!(Theme::System.scheme(Some(Scheme::Paper)), Scheme::Paper);
        assert_eq!(Theme::Ink.scheme(Some(Scheme::Paper)), Scheme::Ink);
        assert_eq!(Theme::Paper.scheme(None), Scheme::Paper);
    }

    #[test]
    fn the_answers_of_the_system_are_read() {
        assert_eq!(scheme_of_portal("v u 2\n"), Some(Scheme::Paper));
        assert_eq!(scheme_of_portal("v u 1"), Some(Scheme::Ink));
        assert_eq!(scheme_of_portal("v u 0"), None);
        assert_eq!(scheme_of_portal(""), None);
        assert_eq!(scheme_of_gsettings("'prefer-light'\n"), Some(Scheme::Paper));
        assert_eq!(scheme_of_gsettings("'prefer-dark'"), Some(Scheme::Ink));
        assert_eq!(scheme_of_gsettings("'default'"), None);
    }

    #[test]
    fn the_files_of_the_flavor_are_the_ink_palette() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling");
        let ink = Colors::of(Scheme::Ink);
        for (name, text) in files(&ink) {
            let path = match name {
                "colors.css" => dir.join("waybar/colors.css"),
                icons if icons.starts_with("icons/") => dir.join("waybar").join(icons),
                other => dir.join(other),
            };
            let have = std::fs::read_to_string(&path).unwrap_or_default();
            assert_eq!(
                have, text,
                "{name} is not what --apply-theme writes for Ink"
            );
        }
    }

    #[test]
    fn a_program_starts_with_the_file_of_the_user_before_the_others() {
        let temp = std::env::temp_dir().join(format!("tiling-launch-{}", std::process::id()));
        let (user, generated, flavor) = (temp.join("u"), temp.join("g"), temp.join("f"));
        for dir in [&user, &generated, &flavor] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let dirs = [user.clone(), generated.clone(), flavor.clone()];
        let rest = vec!["-f".to_owned()];
        let line = |p: &str, s| launch_line(p, &rest, &dirs, s);
        assert_eq!(line("foot", Scheme::Ink), ["foot", "-f"]);
        assert_eq!(line("swaybg", Scheme::Ink)[..2], ["swaybg", "-c"]);
        std::fs::write(flavor.join("foot.ini"), "").unwrap();
        std::fs::write(generated.join("foot.ini"), "").unwrap();
        let flavored = generated.join("foot.ini").to_string_lossy().into_owned();
        assert_eq!(line("foot", Scheme::Ink), ["foot", "-c", &flavored, "-f"]);
        std::fs::write(user.join("foot.ini"), "").unwrap();
        let own = user.join("foot.ini").to_string_lossy().into_owned();
        assert_eq!(line("foot", Scheme::Ink)[2], own);
        std::fs::write(flavor.join("wallpaper.png"), "").unwrap();
        std::fs::write(flavor.join("wallpaper-paper.png"), "").unwrap();
        let name = |s| line("swaybg", s)[2].rsplit('/').next().unwrap().to_owned();
        assert_eq!(name(Scheme::Ink), "wallpaper.png");
        assert_eq!(name(Scheme::Paper), "wallpaper-paper.png");
        std::fs::write(user.join("wallpaper.png"), "").unwrap();
        assert_eq!(
            line("swaybg", Scheme::Paper)[2],
            user.join("wallpaper.png").to_string_lossy()
        );
        std::fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn the_launcher_is_opaque_and_the_terminal_is_padded() {
        let ink = Colors::of(Scheme::Ink);
        let all = files(&ink);
        let text = |n: &str| all.iter().find(|(k, _)| *k == n).unwrap().1.clone();
        assert!(text("fuzzel.ini").contains("background=1a1d27ff"));
        assert!(text("foot.ini").contains("pad=16x16"));
        assert!(text("foot.ini").contains("font=JetBrains Mono:size=10.5"));
    }
}
