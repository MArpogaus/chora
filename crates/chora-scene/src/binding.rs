//! Key bindings such as `Super+Return`.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Modifier keys of a binding.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers {
    /// Control.
    pub ctrl: bool,
    /// Alt.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Super, also called logo or Mod4.
    pub logo: bool,
}

/// Modifiers and one key, for example `Super+Shift+Return`. The key is an
/// XKB keysym name; the core resolves it without case.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Binding {
    /// The modifiers.
    pub mods: Modifiers,
    /// The keysym name.
    pub key: String,
}

/// A binding with a syntax error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid binding {0:?}")]
pub struct BindingError(String);

impl FromStr for Binding {
    type Err = BindingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || BindingError(s.into());
        let mut parts: Vec<&str> = s.split('+').collect();
        let key = parts.pop().filter(|k| !k.is_empty()).ok_or_else(err)?;
        let mut mods = Modifiers::default();
        for part in parts {
            let flag = match part.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => &mut mods.ctrl,
                "alt" | "mod1" => &mut mods.alt,
                "shift" => &mut mods.shift,
                "super" | "logo" | "mod4" => &mut mods.logo,
                _ => return Err(err()),
            };
            *flag = true;
        }
        Ok(Self {
            mods,
            key: key.into(),
        })
    }
}

impl TryFrom<String> for Binding {
    type Error = BindingError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Binding> for String {
    fn from(b: Binding) -> Self {
        b.to_string()
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = self.mods;
        for (on, name) in [
            (m.ctrl, "Ctrl"),
            (m.alt, "Alt"),
            (m.shift, "Shift"),
            (m.logo, "Super"),
        ] {
            if on {
                write!(f, "{name}+")?;
            }
        }
        f.write_str(&self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_in_any_case() {
        let b: Binding = "super+SHIFT+Return".parse().unwrap();
        assert!(b.mods.logo && b.mods.shift && !b.mods.ctrl);
        assert_eq!(b.to_string(), "Shift+Super+Return");
    }

    #[test]
    fn rejects_unknown_modifiers_and_empty_keys() {
        assert!("Hyper+x".parse::<Binding>().is_err());
        assert!("Ctrl+".parse::<Binding>().is_err());
    }
}
