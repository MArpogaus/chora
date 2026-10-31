//! chora-xp: a stacking window manager in the style of Windows XP, the
//! showroom flavor of chora. Window frames, the taskbar and the start menu
//! are image nodes that the plugin rasterizes from the SVG files of the
//! Exposé theme and of the Windows XP pack. Windows wobble, minimize
//! through a magic lamp, and live on the four faces of a desktop cube.

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod apps;
mod assistant;
mod balloon;
mod cascade;
mod config;
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
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_kit::{runtime, settings};
use chora_scene::{Event, ipc::ClientError};
use clap::Parser;

use crate::{config::Config, theme::Theme, wm::Wm};

/// How often the plugin checks the clock.
const TICK: Duration = Duration::from_secs(1);
/// The time between two frames of an effect: 60 Hz.
const FRAME: Duration = Duration::from_micros(16_667);

/// The theme of the installed flavor.
const INSTALLED_THEME: &str = "/usr/share/chora/flavors/xp/theme";

/// A stacking window manager in the style of Windows XP.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The theme directory: `flavors/xp/theme` of the source tree, or of the
    /// installed flavor. Without it, the installed flavor, else the source
    /// tree of the executable in `target/<profile>/`.
    #[arg(long, env = "CHORA_XP_THEME")]
    theme: Option<PathBuf>,
    /// The settings file. Without it, chora-xp reads
    /// $XDG_CONFIG_HOME/chora/chora-xp.yaml, by default in ~/.config, if
    /// it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Stretches the time of the magic lamp and of the cube by a factor,
    /// to show them slowly.
    #[arg(long, value_name = "FACTOR", default_value_t = 1.0)]
    slow_motion: f64,
    /// Manages only this output, by name, and leaves the others to other
    /// window managers; all outputs without it. Repeat it for more.
    #[arg(long = "output", value_name = "NAME")]
    outputs: Vec<String>,
}

fn main() -> ExitCode {
    let args = Args::try_parse().unwrap_or_else(|err| {
        if err.kind() == clap::error::ErrorKind::UnknownArgument {
            eprintln!(
                "chora-xp: --wallpaper, --clock, --terminal, --launcher, --program, \
                 --no-sounds and --file-manager moved into chora-xp.yaml (D62)"
            );
        }
        err.exit()
    });
    runtime::run(wm::NAME, || play(&args))
}

/// One connection to the core: window management until Turn Off.
fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-xp")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    let dir = resolve_theme(
        args.theme.clone(),
        Path::new(INSTALLED_THEME),
        exe.as_deref(),
    );
    let theme = Theme::load(&dir, &cache_dir()?).map_err(|e| format!("theme: {e}"))?;
    let mut wm = Wm::start(args.outputs.clone(), theme, settings, args.slow_motion)?;
    event_loop(&mut wm)?;
    if wm.restart {
        drop(wm);
        return Err(restart().into());
    }
    // Nodes of a plugin with resume outlive it for the grace time.
    Ok(wm.leave()?)
}

/// The theme directory: the given one, else the installed flavor if it
/// exists, else `flavors/xp/theme` two levels above the directory of `exe`
/// (the source tree of `target/<profile>/chora-xp`). Falls back to the
/// installed path, so that the error of a missing theme names it.
fn resolve_theme(given: Option<PathBuf>, installed: &Path, exe: Option<&Path>) -> PathBuf {
    if let Some(dir) = given {
        return dir;
    }
    if installed.is_dir() {
        return installed.to_path_buf();
    }
    exe.and_then(Path::parent)
        .and_then(|dir| dir.join("../../flavors/xp/theme").canonicalize().ok())
        .filter(|dir| dir.is_dir())
        .unwrap_or_else(|| installed.to_path_buf())
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
        let until = wm.assistant_due().map_or(until, |due| until.min(due));
        if let Some(event) = wm.conn_event(until.saturating_duration_since(Instant::now()))? {
            let (event, next) = last_move(wm, event)?;
            runtime::dispatch(wm, event)?;
            if let Some(next) = next {
                runtime::dispatch(wm, next)?;
            }
        }
        let now = Instant::now();
        wm.menu_rest(now)?;
        wm.notifications(now)?;
        wm.assistant_step(now)?;
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

/// The directory of the rendered files: one per chora instance, below
/// `$XDG_RUNTIME_DIR/chora-xp`, which the core can read.
fn cache_dir() -> Result<PathBuf, String> {
    // No fallback to the shared temporary directory: another local user
    // could plant files there first.
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or("no absolute $XDG_RUNTIME_DIR for the cache")?;
    let instance = std::env::var_os(chora_scene::ipc::SOCKET_ENV)
        .map(PathBuf::from)
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "default".into());
    Ok(runtime.join("chora-xp").join(instance))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_resolution_order() {
        let root = std::env::temp_dir().join(format!("chora-xp-theme-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let installed = root.join("usr/flavors/xp/theme");
        let exe = root.join("tree/target/release/chora-xp");
        let source = root.join("tree/flavors/xp/theme");
        let given = root.join("given");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        // Nothing exists: the installed path, so that the error names it.
        assert_eq!(resolve_theme(None, &installed, Some(&exe)), installed);
        std::fs::create_dir_all(&source).unwrap();
        assert_eq!(
            resolve_theme(None, &installed, Some(&exe)),
            source.canonicalize().unwrap()
        );
        // The installed flavor wins over the source tree.
        std::fs::create_dir_all(&installed).unwrap();
        assert_eq!(resolve_theme(None, &installed, Some(&exe)), installed);
        // The given path wins over everything.
        assert_eq!(
            resolve_theme(Some(given.clone()), &installed, Some(&exe)),
            given
        );
        // Without an executable path, the installed path.
        assert_eq!(resolve_theme(None, &installed, None), installed);
        let _ = std::fs::remove_dir_all(&root);
    }
}
