//! chora-xp: a stacking window manager in the style of Windows XP, the
//! showroom flavor of chora. Window frames, the taskbar and the start menu
//! are image nodes that the plugin rasterizes from the SVG files of the
//! Exposé theme and of the Windows XP pack.

mod conn;
mod decoration;
mod icons;
mod startmenu;
mod taskbar;
mod text;
mod theme;
mod wallpaper;
mod window;
mod wm;

use std::{
    path::PathBuf,
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_scene::ipc::ClientError;
use clap::Parser;

use crate::{conn::Conn, icons::Icons, startmenu::Menu, theme::Theme, wm::Wm};

/// How often the plugin checks the clock and the sizes of the windows.
const TICK: Duration = Duration::from_millis(250);

/// A stacking window manager in the style of Windows XP.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The theme directory: `flavors/xp/theme` of the source tree, or of the
    /// installed flavor.
    #[arg(
        long,
        env = "CHORA_XP_THEME",
        default_value = "/usr/share/chora/flavors/xp/theme"
    )]
    theme: PathBuf,
    /// The shell command of the terminal, for the start menu and Super+e.
    #[arg(long, default_value = "foot")]
    terminal: String,
    /// The shell command of All Programs and Run.
    #[arg(long, default_value = "fuzzel")]
    launcher: String,
    /// More programs for the start menu, as TITLE=COMMAND.
    #[arg(long = "program", value_name = "TITLE=COMMAND")]
    programs: Vec<String>,
    /// The format of the clock, as in strftime.
    #[arg(long, default_value = "%-I:%M %p")]
    clock: String,
    /// The wallpaper, a PNG or SVG file; Bløff of the theme by default.
    #[arg(long)]
    wallpaper: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chora-xp: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    let conn = Conn::connect(wm::NAME).map_err(|e| e.to_string())?;
    let theme = Theme::load(&args.theme, &cache_dir()).map_err(|e| format!("theme: {e}"))?;
    let mut icons = Icons::default();
    let menu = Menu::new(&menu_config(args), &theme, &mut icons);
    let config = wm::Config {
        terminal: args.terminal.clone(),
        clock: args.clock.clone(),
        wallpaper: args
            .wallpaper
            .clone()
            .unwrap_or_else(|| theme.path("wallpaper/bloff.png")),
    };
    let mut wm = Wm::start(conn, theme, menu, icons, config).map_err(|e| e.to_string())?;
    event_loop(&mut wm).map_err(|e| e.to_string())
}

/// Handles events until Turn Off; ticks in between.
fn event_loop(wm: &mut Wm) -> Result<(), ClientError> {
    let mut next = Instant::now() + TICK;
    while !wm.done {
        let wait = next.saturating_duration_since(Instant::now());
        if let Some(event) = wm.conn_event(wait)? {
            wm.handle(event)?;
        }
        if Instant::now() >= next {
            wm.tick()?;
            next = Instant::now() + TICK;
        }
    }
    Ok(())
}

/// The programs of the start menu: the terminal, then the extra ones.
fn menu_config(args: &Args) -> startmenu::Config {
    let mut programs = vec![(
        "Terminal".to_string(),
        args.terminal.clone(),
        "utilities-terminal".to_string(),
    )];
    for program in &args.programs {
        let (title, command) = program.split_once('=').unwrap_or((program, program));
        let icon = command.split_whitespace().next().unwrap_or_default();
        programs.push((title.into(), command.into(), icon.into()));
    }
    startmenu::Config {
        user: std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "User".into()),
        programs,
        launcher: args.launcher.clone(),
    }
}

/// The directory of the rendered files: one per chora instance, below
/// `$XDG_RUNTIME_DIR/chora-xp`, which the core can read.
fn cache_dir() -> PathBuf {
    let runtime =
        std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from);
    let instance = std::env::var_os(chora_scene::ipc::SOCKET_ENV)
        .map(PathBuf::from)
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "default".into());
    runtime.join("chora-xp").join(instance)
}
