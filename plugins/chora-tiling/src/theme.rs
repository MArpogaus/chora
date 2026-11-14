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
                "#11131a", "#1a1d27", "#242836", "#2e3344", "#e6e8ef", "#9aa1b4", "#a78bfa",
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

/// How long a question to the system may take.
const ASK_LIMIT: std::time::Duration = std::time::Duration::from_secs(2);

/// What a program prints, when it succeeds within `limit`; a program that
/// takes longer is killed.
#[must_use]
pub fn run_within(program: &str, args: &[&str], limit: std::time::Duration) -> Option<String> {
    use std::io::Read;
    let mut child = std::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let end = std::time::Instant::now() + limit;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if std::time::Instant::now() < end => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let mut text = String::new();
    child.stdout.take()?.read_to_string(&mut text).ok()?;
    status.success().then_some(text)
}

/// What the system asks for: the settings portal, else gsettings, each
/// within two seconds; `None` when neither answers or the person has no
/// preference. It blocks, so the loop of the window manager runs it in a
/// thread of its own.
#[must_use]
pub fn system_scheme() -> Option<Scheme> {
    let portal = run_within(
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
        ASK_LIMIT,
    )
    .and_then(|a| scheme_of_portal(&a));
    portal.or_else(|| {
        run_within(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "color-scheme"],
            ASK_LIMIT,
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
        ("mako.conf", mako(colors)),
        ("icons/apps.svg", icon(&colors.text, APPS, 16)),
        ("icons/apps@2.svg", icon(&colors.text, APPS, 32)),
        ("icons/keys.svg", icon(&colors.text, KEYS_ICON, 16)),
        ("icons/keys@2.svg", icon(&colors.text, KEYS_ICON, 32)),
        ("icons/power.svg", icon(&colors.text, POWER, 16)),
        ("icons/power@2.svg", icon(&colors.text, POWER, 32)),
    ]
}

/// The shapes of the icons of the panel on a 16 px grid: a 2 x 2 grid of
/// tiles, a keyboard and a power sign. The panel draws them as
/// backgrounds, so they carry their colour, which changes with the theme.
const APPS: &str = r#"<g fill="C"><rect x="1.5" y="1.5" width="5.5" height="5.5" rx="1.5"/><rect x="9" y="1.5" width="5.5" height="5.5" rx="1.5"/><rect x="1.5" y="9" width="5.5" height="5.5" rx="1.5"/><rect x="9" y="9" width="5.5" height="5.5" rx="1.5"/></g>"#;
const KEYS_ICON: &str = r#"<rect x="1" y="3.5" width="14" height="9" rx="2.5" fill="none" stroke="C" stroke-width="1.5"/><g fill="C"><rect x="3.5" y="6" width="1.5" height="1.5" rx=".4"/><rect x="7.25" y="6" width="1.5" height="1.5" rx=".4"/><rect x="11" y="6" width="1.5" height="1.5" rx=".4"/><rect x="4.5" y="9" width="7" height="1.5" rx=".5"/></g>"#;
const POWER: &str = r#"<path d="M4.6 4.2a5.5 5.5 0 1 0 6.8 0M8 1.5v6" fill="none" stroke="C" stroke-width="1.5" stroke-linecap="round"/>"#;

/// An icon drawn at `size` pixels of its 16 px grid: the panel takes the
/// 16 px file at scale 1 and the 32 px file at a higher scale
/// (`-gtk-scaled`), so that it is drawn at the scale of the output.
fn icon(color: &str, shapes: &str, size: u32) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 16 16\">{}</svg>\n",
        shapes.replace("\"C\"", &format!("\"{color}\""))
    )
}

fn fuzzel(c: &Colors) -> String {
    let [surface, text, accent, on_accent, line] =
        [&c.surface, &c.text, &c.accent, &c.on_accent, &c.line].map(|x| format!("{}ff", bare(x)));
    format!(
        "# The launcher of the tiling flavor, themed and opaque. Written by\n\
         # chora-tiling --apply-theme from the palette of chora-tiling.yaml.\n\
         [main]\n\
         font=Inter:pixelsize=14\n\
         prompt=\"Apps  \"\n\
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
         selection={accent}\n\
         selection-text={on_accent}\n\
         selection-match={on_accent}\n\
         border={line}\n\
         \n\
         [border]\n\
         width=1\n\
         radius=12\n\
         selection-radius=8\n"
    )
}

fn mako(c: &Colors) -> String {
    format!(
        "# The notifications of the tiling flavor: an island in the palette, above\n\
         # the dot of the helper and 12 inside the border of a tile. Written by\n\
         # chora-tiling --apply-theme from the palette of chora-tiling.yaml.\n\
         font=Inter 14\n\
         layer=overlay\n\
         anchor=bottom-right\n\
         margin=24,24,62,24\n\
         padding=12\n\
         width=360\n\
         border-size=1\n\
         border-radius=10\n\
         icons=0\n\
         default-timeout=10000\n\
         background-color={surface}\n\
         text-color={text}\n\
         border-color={line}\n\
         progress-color=over {accent}\n\
         \n\
         [mode=chat]\n\
         margin=24,{beside},62,24\n\
         \n\
         [urgency=high]\n\
         border-color={urgent}\n\
         default-timeout=0\n",
        surface = c.surface,
        text = c.text,
        line = c.line,
        accent = c.accent,
        urgent = c.urgent,
        beside = CHAT_BESIDE,
    )
}

/// The right margin of a notification while the chat panel is open: the
/// panel is 400 wide, 13 from the edge, and the notification stands 12
/// left of it.
const CHAT_BESIDE: i32 = 424;

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

/// Writes a file with `text`, unless it has that already: the text goes to
/// a temporary file beside it, which replaces it in one step, so a reader
/// never sees half a file.
fn write_changed(path: &Path, text: &str) -> std::io::Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|old| old == text) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".tmp{}", std::process::id()));
    let temp = path.with_file_name(name);
    let written = std::fs::write(&temp, text).and_then(|()| std::fs::rename(&temp, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

/// Writes the files of [`files`] and the name of the scheme into a
/// directory, which it creates; a file that has the text already stays
/// as it is.
///
/// # Errors
///
/// A file cannot be written.
pub fn apply(dir: &Path, colors: &Colors, scheme: Scheme) -> std::io::Result<()> {
    for (name, text) in files(colors) {
        write_changed(&dir.join(name), &text)?;
    }
    write_changed(&dir.join("scheme"), &format!("{}\n", scheme.name()))
}

/// `chora-tiling` in the runtime directory of the user: the theme that
/// [`apply`] writes and the pids of the panel.
#[must_use]
pub fn runtime_dir() -> Option<std::path::PathBuf> {
    let runtime = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?);
    runtime.is_absolute().then(|| runtime.join("chora-tiling"))
}

/// The directory that [`apply`] writes by default: `chora-tiling/theme`
/// in the runtime directory of the user, written again at each start.
#[must_use]
pub fn default_dir() -> Option<std::path::PathBuf> {
    runtime_dir().map(|d| d.join("theme"))
}

/// Whether a command line (`/proc/PID/cmdline`) is the program `word`, by
/// the exact name of its first word, or the script `word` that a shell
/// runs, by the exact name of its second.
fn runs(cmdline: &[u8], word: &str) -> bool {
    let mut words = cmdline
        .split(|&b| b == 0)
        .map(|w| String::from_utf8_lossy(w).into_owned());
    let name = |w: &str| w.rsplit('/').next().unwrap_or_default().to_owned();
    let Some(first) = words.next().map(|w| name(&w)) else {
        return false;
    };
    if first == word {
        return true;
    }
    matches!(first.as_str(), "sh" | "bash" | "dash")
        && words.next().is_some_and(|w| name(&w) == word)
}

/// The pid that a companion of the flavor left in `NAME.pid` of the
/// runtime directory, when that process still runs and is the program
/// or script `word`, so that the pid of another program is never taken.
#[must_use]
pub fn companion(name: &str, word: &str) -> Option<u32> {
    let file = runtime_dir()?.join(format!("{name}.pid"));
    let pid: u32 = std::fs::read_to_string(file).ok()?.trim().parse().ok()?;
    let line = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    runs(&line, word).then_some(pid)
}

/// Sends a signal, by the name that `kill -s` takes, to a process.
pub fn signal(pid: u32, name: &str) {
    let _ = std::process::Command::new("kill")
        .args(["-s", name, &pid.to_string()])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
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
    dirs.extend(flavor_dirs());
    dirs
}

/// Where the files of the flavor are, first the one that wins:
/// `CHORA_FLAVOR_DIR`, the installed path, and the checkout that the
/// running binary comes from (`target/debug/chora-tiling` of a build in
/// a repository), so a session without the variable finds them too.
#[must_use]
pub fn flavor_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    dirs.extend(
        std::env::var_os("CHORA_FLAVOR_DIR")
            .filter(|v| !v.is_empty())
            .map(std::path::PathBuf::from),
    );
    dirs.push(INSTALLED.into());
    let exe = std::env::current_exe().ok();
    dirs.extend(
        exe.iter()
            .flat_map(|e| e.ancestors().skip(1))
            .map(|a| a.join("flavors").join("tiling"))
            .find(|d| d.is_dir()),
    );
    dirs
}

/// The value of `XDG_DATA_DIRS` that lets the launcher list the
/// `applications` folder of the flavor (the entries Screenshot, Screenshot
/// of an area and the looks): the first directory that has one, in front
/// of what `current` says, or of the default of the specification.
#[must_use]
pub fn data_dirs(dirs: &[std::path::PathBuf], current: Option<&str>) -> Option<String> {
    let flavor = dirs.iter().find(|d| d.join("applications").is_dir())?;
    let rest = current
        .filter(|c| !c.is_empty())
        .unwrap_or("/usr/local/share:/usr/share");
    Some(format!("{}:{rest}", flavor.display()))
}

/// The directory of the flavor in an installed system.
pub const INSTALLED: &str = "/usr/share/chora/flavors/tiling";

/// The script that a program of the flavor runs, `sh power.sh` for
/// `power`, `sh screenshot.sh` for `screenshot` and `sh look.sh` for `look`, or the name of the
/// script that no directory has.
fn script_line(program: &str, dirs: &[std::path::PathBuf]) -> Result<Vec<String>, LaunchError> {
    let name = format!("{program}.sh");
    dirs.iter()
        .map(|d| d.join(&name))
        .find(|f| f.is_file())
        .map(|f| vec!["sh".to_owned(), f.to_string_lossy().into_owned()])
        .ok_or_else(|| LaunchError::Missing {
            program: program.to_owned(),
            name,
            searched: dirs.to_vec(),
        })
}

/// Why a program of the flavor does not start.
#[derive(Debug, PartialEq, Eq)]
pub enum LaunchError {
    /// The script of the program is in none of the directories.
    Missing {
        program: String,
        name: String,
        searched: Vec<std::path::PathBuf>,
    },
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self::Missing { name, searched, .. } = self;
        let list: Vec<_> = searched.iter().map(|d| d.display().to_string()).collect();
        write!(f, "{name} is in none of {}", list.join(", "))
    }
}

impl LaunchError {
    /// The title and the text of the notice for a person.
    #[must_use]
    pub fn notice(&self) -> (&'static str, &'static str) {
        let german = matches!(
            std::env::var("LC_ALL")
                .or_else(|_| std::env::var("LC_MESSAGES"))
                .or_else(|_| std::env::var("LANG"))
                .as_deref(),
            Ok(l) if l.starts_with("de")
        );
        let Self::Missing { program, .. } = self;
        match (program.as_str(), german) {
            ("power", false) => (
                "Power menu not installed",
                "The file power.sh of the tiling flavor was not found.",
            ),
            ("power", true) => (
                "Ausschaltmenü nicht installiert",
                "Die Datei power.sh des Tiling-Flavors fehlt.",
            ),
            ("look", false) => (
                "Look switch not installed",
                "The file look.sh of the tiling flavor was not found.",
            ),
            ("look", true) => (
                "Look-Umschalter nicht installiert",
                "Die Datei look.sh des Tiling-Flavors fehlt.",
            ),
            (_, false) => (
                "Screenshot not installed",
                "The file screenshot.sh of the tiling flavor was not found.",
            ),
            (_, true) => (
                "Bildschirmfoto nicht installiert",
                "Die Datei screenshot.sh des Tiling-Flavors fehlt.",
            ),
        }
    }
}

/// The command line of a program of the flavor with its config file:
/// `foot -c FILE`, `fuzzel --config FILE`, `swaylock -C FILE` and `swaybg
/// -i WALLPAPER -m fill`, `screenshot` and `power` as `sh screenshot.sh`
/// and `sh power.sh` of the flavor, then the arguments of the caller; the program
/// alone when no file exists. The wallpaper of a scheme is
/// `wallpaper.png` for Ink and `wallpaper-paper.png` for Paper; a
/// `wallpaper.png` of the user, found first, serves both.
///
/// # Errors
///
/// `screenshot` or `power` has no script in any directory.
pub fn launch_line(
    program: &str,
    rest: &[String],
    dirs: &[std::path::PathBuf],
    scheme: Scheme,
) -> Result<Vec<String>, LaunchError> {
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
        "mako" => line.extend(
            find(&["mako.conf"])
                .into_iter()
                .flat_map(|f| ["--config".into(), f]),
        ),
        "screenshot" | "power" | "look" => line = script_line(program, dirs)?,
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
    Ok(line)
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
        let line = |p: &str, s| launch_line(p, &rest, &dirs, s).unwrap();
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

    fn fuzzel_value(file: &str, key: &str) -> String {
        let line = file
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key}=")))
            .unwrap_or_default();
        line.to_owned()
    }

    #[test]
    fn the_selected_row_of_the_launcher_stands_out_from_the_panel() {
        for scheme in [Scheme::Ink, Scheme::Paper] {
            let colors = Colors::of(scheme);
            let all = files(&colors);
            let file = &all.iter().find(|(n, _)| *n == "fuzzel.ini").unwrap().1;
            let hex = |key: &str| format!("#{}", &fuzzel_value(file, key)[..6]);
            let ratio = contrast(&hex("selection"), &hex("background"));
            assert!(ratio >= 3.0, "{scheme:?}: the selection is {ratio:.2}:1");
            for text in ["selection-text", "selection-match"] {
                let ratio = contrast(&hex(text), &hex("selection"));
                assert!(ratio >= 4.5, "{scheme:?}: {text} is {ratio:.2}:1");
            }
        }
    }

    #[test]
    fn a_question_to_the_system_ends_within_its_limit() {
        let limit = std::time::Duration::from_millis(200);
        let start = std::time::Instant::now();
        assert_eq!(run_within("sleep", &["5"], limit), None);
        assert!(start.elapsed() < std::time::Duration::from_secs(2));
        assert_eq!(run_within("echo", &["a"], limit).as_deref(), Some("a\n"));
        assert_eq!(run_within("false", &[], limit), None);
    }

    #[test]
    fn the_files_of_the_theme_are_written_only_when_they_change() {
        use std::os::unix::fs::MetadataExt;
        let dir = std::env::temp_dir().join(format!("tiling-apply-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ink = Colors::of(Scheme::Ink);
        apply(&dir, &ink, Scheme::Ink).unwrap();
        let inode = |name: &str| std::fs::metadata(dir.join(name)).unwrap().ino();
        let names = ["colors.css", "scheme"];
        let before = names.map(inode);
        apply(&dir, &ink, Scheme::Ink).unwrap();
        assert_eq!(names.map(inode), before, "the same theme rewrote a file");
        apply(&dir, &Colors::of(Scheme::Paper), Scheme::Paper).unwrap();
        assert_ne!(inode("colors.css"), before[0]);
        assert_eq!(
            std::fs::read_to_string(dir.join("scheme")).unwrap(),
            "paper\n"
        );
        let stray = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|e| e.file_name().to_string_lossy().contains(".tmp"));
        assert!(!stray, "a temporary file stayed");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_companion_is_the_program_or_the_script_by_its_exact_name() {
        let line = |s: &str| s.replace('|', "\0").into_bytes();
        assert!(runs(&line("/usr/bin/waybar|-c|x"), "waybar"));
        assert!(runs(&line("sh|/home/u/waybar.sh|/home/u"), "waybar.sh"));
        assert!(!runs(&line("sh|/home/u/waybar.sh|/home/u"), "waybar"));
        assert!(!runs(&line("/usr/bin/waybar-extra"), "waybar"));
        assert!(!runs(&line("vim|waybar.sh"), "waybar.sh"));
        assert!(!runs(&line("/home/waybar/bin/tool"), "waybar"));
    }

    #[test]
    fn the_launcher_has_a_prompt_and_14_pixel_text() {
        let all = files(&Colors::of(Scheme::Ink));
        let file = &all.iter().find(|(n, _)| *n == "fuzzel.ini").unwrap().1;
        assert!(fuzzel_value(file, "prompt").trim_matches('"').trim().len() > 1);
        assert_eq!(fuzzel_value(file, "font"), "Inter:pixelsize=14");
    }

    #[test]
    fn the_launcher_lists_the_applications_of_the_flavor() {
        let temp = std::env::temp_dir().join(format!("tiling-data-{}", std::process::id()));
        let (bare_dir, flavor) = (temp.join("b"), temp.join("f"));
        std::fs::create_dir_all(flavor.join("applications")).unwrap();
        std::fs::create_dir_all(&bare_dir).unwrap();
        let dirs = [bare_dir, flavor.clone()];
        let want = format!("{}:/a:/b", flavor.display());
        assert_eq!(data_dirs(&dirs, Some("/a:/b")), Some(want));
        assert!(
            data_dirs(&dirs, None)
                .unwrap()
                .ends_with(":/usr/local/share:/usr/share")
        );
        assert_eq!(data_dirs(&dirs[..1], None), None);
        std::fs::remove_dir_all(temp).unwrap();
        let apps = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/tiling/applications");
        let shot = std::fs::read_to_string(apps.join("chora-screenshot.desktop")).unwrap();
        assert!(shot.contains("\nName=Screenshot\n"));
        assert!(shot.contains("\nExec=chora-tiling --launch screenshot\n"));
        for (file, name, word) in [
            ("chora-look-light.desktop", "Light look", "light"),
            ("chora-look-dark.desktop", "Dark look", "dark"),
            ("chora-look-system.desktop", "System look", "system"),
        ] {
            let entry = std::fs::read_to_string(apps.join(file)).unwrap();
            assert!(entry.contains(&format!("\nName={name}\n")), "{file}");
            assert!(
                entry.contains(&format!("\nExec=chora-tiling --launch look {word}\n")),
                "{file}"
            );
        }
    }

    #[test]
    fn a_missing_script_is_an_error_and_never_true() {
        let none = [std::path::PathBuf::from("/nonexistent/flavor")];
        for program in ["power", "screenshot", "look"] {
            let err = launch_line(program, &[], &none, Scheme::Ink).unwrap_err();
            assert!(err.to_string().contains(&format!("{program}.sh")));
            let (title, text) = err.notice();
            assert!(text.contains(&format!("{program}.sh")), "{title}: {text}");
        }
    }

    #[test]
    fn the_checkout_of_the_binary_is_a_flavor_directory() {
        let dirs = flavor_dirs();
        assert!(
            dirs.iter().any(|d| d.join("power.sh").is_file()),
            "{dirs:?}"
        );
    }

    #[test]
    fn the_launcher_is_opaque_and_the_terminal_is_padded() {
        let ink = Colors::of(Scheme::Ink);
        let all = files(&ink);
        let text = |n: &str| all.iter().find(|(k, _)| *k == n).unwrap().1.clone();
        assert!(text("fuzzel.ini").contains("background=1a1d27ff"));
        assert!(text("fuzzel.ini").contains("selection-radius=8"));
        let mako = text("mako.conf");
        assert!(mako.contains("background-color=#1a1d27"));
        assert!(mako.contains("border-color=#2e3344"));
        assert!(mako.contains("border-radius=10"));
        assert!(text("foot.ini").contains("pad=16x16"));
        assert!(text("foot.ini").contains("font=JetBrains Mono:size=10.5"));
    }

    #[test]
    fn the_three_icons_of_the_panel_have_the_colour_of_its_text() {
        for scheme in [Scheme::Ink, Scheme::Paper] {
            let colors = Colors::of(scheme);
            for (name, svg) in files(&colors)
                .iter()
                .filter(|(k, _)| k.starts_with("icons/"))
            {
                assert!(
                    svg.to_lowercase().contains(&colors.text.to_lowercase()),
                    "{name} in {scheme:?}"
                );
            }
        }
    }

    #[test]
    fn the_notifications_keep_clear_of_the_dot_and_the_border() {
        let all = files(&Colors::of(Scheme::Ink));
        let mako = &all.iter().find(|(k, _)| *k == "mako.conf").unwrap().1;
        assert!(mako.contains("margin=24,24,62,24"));
        assert!(mako.contains("[mode=chat]\nmargin=24,424,62,24"));
        assert!(!mako.contains("max-icon-size"), "mako refuses a size of 0");
    }
}
