//! chora-xp: a stacking window manager in the style of Windows XP, the
//! showroom flavor of chora. Window frames, the taskbar and the start menu
//! are image nodes that the plugin rasterizes from the SVG files of the
//! Exposé theme and of the Windows XP pack. Windows wobble, minimize
//! through a magic lamp, and live on the four faces of a desktop cube.

mod apps;
mod balloon;
mod cascade;

mod context;
mod control;
mod cube;
mod datetime;
mod decoration;
mod desktop_icons;
mod dialog;
mod display;
mod icons;
mod notify;
mod session;
mod sound;
mod startmenu;
mod switcher;
mod taskbar;
mod taskbar_dialog;
mod text;
mod theme;
mod wallpaper;
mod window;
mod wm;

use std::{
    path::PathBuf,
    process::ExitCode,
    thread::sleep,
    time::{Duration, Instant},
};

use chora_kit::{conn::Conn, icons::Icons};
use chora_scene::{Event, ipc::ClientError};
use clap::Parser;

use crate::{startmenu::Menu, theme::Theme, wm::Wm};

/// How often the plugin checks the clock.
const TICK: Duration = Duration::from_secs(1);
/// The time between two frames of an effect: 60 Hz.
const FRAME: Duration = Duration::from_micros(16_667);

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
    /// The shell command of Run, and of All Programs when no desktop file
    /// lists a program.
    #[arg(long, default_value = "fuzzel")]
    launcher: String,
    /// The shell command of the file manager, for the places of the start
    /// menu and the folders of the desktop; the folder follows it.
    #[arg(long, default_value = "thunar")]
    file_manager: String,
    /// More programs for the start menu, as TITLE=COMMAND.
    #[arg(long = "program", value_name = "TITLE=COMMAND")]
    programs: Vec<String>,
    /// The format of the clock, as in strftime.
    #[arg(long, default_value = "%-I:%M %p")]
    clock: String,
    /// The wallpaper, a PNG, JPEG or SVG file; XP's Bliss when it is
    /// installed, else Bløff of the theme.
    #[arg(long)]
    wallpaper: Option<PathBuf>,
    /// Stretches the time of the magic lamp and of the cube by a factor,
    /// to show them slowly.
    #[arg(long, value_name = "FACTOR", default_value_t = 1.0)]
    slow_motion: f64,
    /// Plays no sounds; Control Panel > Effects > Sounds turns them on.
    #[arg(long)]
    no_sounds: bool,
    /// Manages only this output, by name, and leaves the others to other
    /// window managers; all outputs without it. Repeat it for more.
    #[arg(long = "output", value_name = "NAME")]
    outputs: Vec<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    // The core may close the connection and stay: the emergency reset
    // revokes grants and disconnects, and the scene survives it. Come back
    // in that case; give up when only failures follow within the episode.
    let mut failing = Duration::ZERO;
    loop {
        let start = Instant::now();
        match run(&args) {
            Ok(()) => return ExitCode::SUCCESS,
            Err(err) => {
                failing = next_failing(start.elapsed(), failing);
                if failing >= COME_BACK {
                    eprintln!("chora-xp: {err}");
                    return ExitCode::FAILURE;
                }
                eprintln!("chora-xp: {err}; I come back");
                sleep(TRY_AGAIN);
            }
        }
    }
}

/// How long a served session must last to start a new episode of errors.
const SERVED: Duration = Duration::from_millis(500);
/// How long one episode of failures may last before the plugin gives up.
const COME_BACK: Duration = Duration::from_secs(30);
/// The wait between two attempts.
const TRY_AGAIN: Duration = Duration::from_millis(500);

/// The failures of the current episode after a session of `served`.
fn next_failing(served: Duration, failing: Duration) -> Duration {
    if served >= SERVED {
        TRY_AGAIN
    } else {
        failing + served + TRY_AGAIN
    }
}

fn run(args: &Args) -> Result<(), String> {
    let conn = Conn::connect(wm::NAME).map_err(|e| e.to_string())?;
    let theme = Theme::load(&args.theme, &cache_dir()).map_err(|e| format!("theme: {e}"))?;
    let mut icons = Icons::new(crate::icons::THEMES);
    let menu = Menu::new(&menu_config(args), &theme, &mut icons);
    let config = wm::Config {
        terminal: args.terminal.clone(),
        file_manager: args.file_manager.clone(),
        clock: args.clock.clone(),
        wallpaper: wallpaper::Style::new(
            args.wallpaper
                .clone()
                .or_else(|| chora_kit::xdg::data_file("wallpapers/WindowsXP/bliss.jpg"))
                .unwrap_or_else(|| theme.path("wallpaper/bloff.png")),
        ),
        slow: args.slow_motion,
        outputs: args.outputs.clone(),
        toggles: control::Toggles {
            sounds: !args.no_sounds,
            ..control::Toggles::default()
        },
    };
    let mut wm = Wm::start(conn, theme, menu, icons, config).map_err(|e| e.to_string())?;
    event_loop(&mut wm).map_err(|e| e.to_string())?;
    if wm.restart {
        drop(wm);
        return Err(restart());
    }
    // Nodes of a plugin with resume outlive it for the grace time.
    wm.leave().map_err(|e| e.to_string())
}

/// Restart of the Turn Off dialog: the plugin runs itself again with the
/// same arguments, in the same process, so that the core and the session
/// keep it. The connection closes with the exec, and the new instance
/// adopts the windows as at any start. Returns only on an error.
fn restart() -> String {
    use std::os::unix::process::CommandExt as _;
    let mut args = std::env::args_os();
    let program = args.next().unwrap_or_else(|| "chora-xp".into());
    let err = std::process::Command::new(program).args(args).exec();
    format!("restart: {err}")
}

/// Handles events until Turn Off. In between, the clock ticks, cascades
/// of the start menu open where the pointer rests, and while an effect
/// runs, frames follow each other at 60 Hz. Notifications come in and
/// expire.
fn event_loop(wm: &mut Wm) -> Result<(), ClientError> {
    let mut tick = Instant::now() + TICK;
    let mut frame = Instant::now();
    while !wm.done {
        let until = if wm.animating() {
            frame.min(tick)
        } else {
            tick
        };
        let until = wm.menu_due().map_or(until, |due| until.min(due));
        let until = wm
            .notes_due(Instant::now())
            .map_or(until, |due| until.min(due));
        if let Some(event) = wm.conn_event(until.saturating_duration_since(Instant::now()))? {
            let (event, next) = last_move(wm, event)?;
            wm.handle(event)?;
            if let Some(next) = next {
                wm.handle(next)?;
            }
        }
        let now = Instant::now();
        wm.menu_rest(now)?;
        wm.notifications(now)?;
        if wm.animating() && now >= frame {
            wm.frame(now)?;
            frame = now + FRAME;
        }
        if now >= tick {
            wm.tick()?;
            tick = now + TICK;
        }
    }
    Ok(())
}

/// Moves of the pointer that queued up, as during a slow redraw of a
/// drag, count as the last one: the event to handle, and the one after
/// the moves.
fn last_move(wm: &mut Wm, mut event: Event) -> Result<(Event, Option<Event>), ClientError> {
    while matches!(event, Event::PointerMoved { .. }) {
        match wm.conn_event(Duration::from_micros(1))? {
            Some(next @ Event::PointerMoved { .. }) => event = next,
            next => return Ok((event, next)),
        }
    }
    Ok((event, None))
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
        file_manager: args.file_manager.clone(),
        apps: apps::load(&args.terminal),
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

#[cfg(test)]
mod supervision_tests {
    use super::{COME_BACK, SERVED, TRY_AGAIN, next_failing};
    use std::time::Duration;

    #[test]
    fn a_served_session_starts_a_new_episode_of_failures() {
        assert_eq!(next_failing(SERVED * 2, COME_BACK / 2), TRY_AGAIN);
    }

    #[test]
    fn an_episode_of_pure_failures_gives_up_after_come_back() {
        let mut failing = Duration::ZERO;
        for _ in 0..1000 {
            failing = next_failing(Duration::ZERO, failing);
            if failing >= COME_BACK {
                return;
            }
        }
        panic!("the plugin never gave up");
    }
}
