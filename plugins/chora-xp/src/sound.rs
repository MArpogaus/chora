//! XP's sounds, from the sound theme `WindowsXP` in the XDG data
//! directories. A player of the session plays them in the background;
//! without the theme or a player, chora-xp stays silent.

use std::process::Command;

use crate::session::on_path;

/// The players to try, with their arguments before the file.
const PLAYERS: [&str; 3] = ["pw-play", "paplay", "canberra-gtk-play -f"];

/// The events with a sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// chora-xp starts.
    Logon,
    /// Log Off of the Log Off dialog.
    LogOff,
    /// Turn Off of the Turn Off dialog.
    ShutDown,
    /// A balloon tip opens.
    Balloon,
}

impl Sound {
    /// The file of the sound in the theme.
    fn file(self) -> &'static str {
        match self {
            Self::Logon => "login.wav",
            Self::LogOff => "logout.wav",
            Self::ShutDown => "shutdown.wav",
            Self::Balloon => "notify.wav",
        }
    }
}

/// Plays a sound, if the theme has it and a player is installed. Returns
/// at once: the shell starts the player in the background.
pub fn play(sound: Sound) {
    let Some(file) =
        chora_kit::xdg::data_file(&format!("sounds/WindowsXP/stereo/{}", sound.file()))
    else {
        return;
    };
    let Some(player) = player(on_path) else {
        return;
    };
    let spawned = Command::new("sh")
        .args(["-c", &format!("{player} \"$1\" >/dev/null 2>&1 &"), "sh"])
        .arg(&file)
        .status();
    if let Err(err) = spawned {
        eprintln!("chora-xp: cannot play {}: {err}", file.display());
    }
}

/// The first player whose program is installed.
fn player(installed: impl Fn(&str) -> bool) -> Option<&'static str> {
    PLAYERS
        .into_iter()
        .find(|p| installed(p.split(' ').next().unwrap_or(p)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_have_xp_sounds_and_a_player() {
        assert_eq!(Sound::Logon.file(), "login.wav");
        assert_eq!(Sound::LogOff.file(), "logout.wav");
        assert_eq!(Sound::Balloon.file(), "notify.wav");
        assert_eq!(player(|p| p != "pw-play"), Some("paplay"));
        assert_eq!(
            player(|p| p == "canberra-gtk-play"),
            Some("canberra-gtk-play -f")
        );
        assert_eq!(player(|_| false), None);
    }
}
