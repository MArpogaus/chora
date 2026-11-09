//! The settings of the plugin (D62), a YAML file that lists every option.
//! The file `notify.yaml` next to the crate is the default and the
//! reference.

use std::time::Duration;

use chora_draw::bubble::Corner;
use chora_kit::settings::Settings;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The default settings.
pub const DEFAULT: &str = include_str!("../notify.yaml");

/// The settings of the plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The corner of the first output where the notifications show.
    pub position: Corner,
    /// The shortest time that a notification shows, in seconds.
    pub shortest: u64,
    /// The longest time that a notification shows, in seconds.
    pub longest: u64,
}

impl Settings for Config {
    fn check(&self) -> Result<(), String> {
        if self.shortest == 0 {
            return Err("shortest: at least 1 second".into());
        }
        if self.longest < self.shortest {
            return Err("longest: not less than shortest".into());
        }
        Ok(())
    }
}

impl Config {
    /// How long a notification shows: the shortest time by default, a time
    /// of the client between the shortest and the longest, and no end for
    /// 0.
    #[must_use]
    pub fn timeout(&self, expire_ms: i32) -> Option<Duration> {
        let [shortest, longest] = [self.shortest, self.longest].map(Duration::from_secs);
        match u64::try_from(expire_ms) {
            Err(_) => Some(shortest),
            Ok(0) => None,
            Ok(ms) => Some(Duration::from_millis(ms).clamp(shortest, longest)),
        }
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::settings::parse;

    use super::*;

    #[test]
    fn the_default_shows_ten_to_thirty_seconds() {
        let config: Config = parse(DEFAULT).unwrap();
        assert_eq!(config.position, Corner::BottomRight);
        assert_eq!([config.shortest, config.longest], [10, 30]);
    }

    #[test]
    fn the_time_follows_the_settings() {
        let config: Config = parse(DEFAULT).unwrap();
        assert_eq!(config.timeout(-1), Some(Duration::from_secs(10)));
        assert_eq!(config.timeout(0), None);
        assert_eq!(config.timeout(3000), Some(Duration::from_secs(10)));
        assert_eq!(config.timeout(20_000), Some(Duration::from_secs(20)));
        assert_eq!(config.timeout(60_000), Some(Duration::from_secs(30)));
    }

    #[test]
    fn a_wrong_value_fails() {
        for text in [
            "position: middle\nshortest: 10\nlongest: 30",
            "position: top_left\nshortest: 0\nlongest: 30",
            "position: top_left\nshortest: 20\nlongest: 10",
        ] {
            assert!(parse::<Config>(text).is_err(), "{text}");
        }
    }
}
