//! Construct files (F17): `$XDG_CONFIG_HOME/chora/constructs/NAME.yaml`
//! with the camera start pose, the leash radius R and the panes. A file
//! there is the user's own, as an mc scene file: its program panes start
//! their commands without a question (N5).

use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::space::Pose;

/// What a pane shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// The window of a shell command.
    Program,
    /// A node tree of another plugin, such as an mc scene.
    Plugin,
}

/// A pane as the file has it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaneFile {
    pub kind: Kind,
    /// The shell command of a program pane.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// The client name and the scene of a plugin pane.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,
    /// The centre on the ground, (x, z); without it, the pane goes ahead
    /// of the camera (C7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<[f64; 2]>,
    /// The size in logical pixels; without it, the size of the output or
    /// of the plugin's node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[f64; 2]>,
}

/// A construct as its file has it.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    /// The start pose of the camera.
    #[serde(default)]
    pub camera: Pose,
    /// The leash radius R; without it, `radius` of the settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(default)]
    pub panes: Vec<PaneFile>,
}

/// The directory of the construct files.
pub fn dir() -> Result<PathBuf, String> {
    chora_kit::xdg::config_home()
        .map(|d| d.join("chora/constructs"))
        .ok_or_else(|| "no $XDG_CONFIG_HOME or $HOME".into())
}

/// A name of a construct, a plugin or a scene: 1 to 64 letters, digits,
/// '.', '_' or '-', not from a dot. It is part of a path or a selector.
pub fn check_name(name: &str) -> Result<(), String> {
    let ok = (1..=64).contains(&name.len())
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
    if ok {
        Ok(())
    } else {
        Err(format!(
            "{name:?}: a name has 1 to 64 letters, digits, '.', '_' or '-'"
        ))
    }
}

fn finite(what: &str, v: &[f64]) -> Result<(), String> {
    if v.iter().all(|x| x.is_finite()) {
        Ok(())
    } else {
        Err(format!("{what}: not a finite number"))
    }
}

impl PaneFile {
    fn check(&self) -> Result<(), String> {
        match self.kind {
            Kind::Program => {
                if self.command.as_deref().is_none_or(|c| c.trim().is_empty()) {
                    return Err("a program pane needs a command".into());
                }
                if self.plugin.is_some() || self.scene.is_some() {
                    return Err("a program pane has no plugin or scene".into());
                }
            }
            Kind::Plugin => {
                check_name(
                    self.plugin
                        .as_deref()
                        .ok_or("a plugin pane needs a plugin")?,
                )?;
                check_name(self.scene.as_deref().ok_or("a plugin pane needs a scene")?)?;
                if self.command.is_some() {
                    return Err("a plugin pane has no command".into());
                }
            }
        }
        if let Some(at) = self.at {
            finite("at", &at)?;
        }
        if let Some(size) = self.size {
            finite("size", &size)?;
            if size.iter().any(|s| !(1.0..=16384.0).contains(s)) {
                return Err(format!("size: {size:?} is not from 1 to 16384"));
            }
        }
        Ok(())
    }
}

impl File {
    pub fn check(&self) -> Result<(), String> {
        let c = &self.camera;
        finite("camera", &[c.x, c.z, c.yaw, c.pitch])?;
        if let Some(r) = self.radius
            && !(r.is_finite() && r >= 100.0)
        {
            return Err(format!("radius: {r} is below 100"));
        }
        for (i, pane) in self.panes.iter().enumerate() {
            pane.check().map_err(|e| format!("panes[{i}]: {e}"))?;
        }
        Ok(())
    }
}

/// Parses and checks a construct.
pub fn parse(text: &str) -> Result<File, String> {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    let file: File =
        serde_saphyr::from_str_with_options(text, options).map_err(|e| e.to_string())?;
    file.check()?;
    Ok(file)
}

/// Reads the construct `name` in `dir`.
pub fn load(dir: &Path, name: &str) -> Result<File, String> {
    check_name(name)?;
    let path = dir.join(format!("{name}.yaml"));
    let text = chora_kit::read_text(&path, chora_kit::MAX_TEXT)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Writes the construct `name` in `dir`, through a temporary file, so
/// that a crash never leaves half a file.
pub fn save(dir: &Path, name: &str, file: &File) -> Result<(), String> {
    check_name(name)?;
    file.check()?;
    let text = serde_saphyr::to_string(file).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.yaml"));
    let tmp = dir.join(format!(".{name}.yaml.tmp"));
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let mut out = std::fs::File::create(&tmp)?;
        out.write_all(text.as_bytes())?;
        out.sync_all()?;
        std::fs::rename(&tmp, &path)
    };
    write().map_err(|e| format!("{}: {e}", path.display()))
}

/// The names of the constructs in `dir`, sorted.
pub fn list(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = (entries.flatten())
        .filter_map(|e| {
            let name = e.file_name().to_str()?.strip_suffix(".yaml")?.to_owned();
            check_name(&name).is_ok().then_some(name)
        })
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "chora-construct-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        drop(std::fs::remove_dir_all(&dir));
        dir
    }

    /// Save and load give the same construct back, to the last bit of
    /// each pose.
    #[test]
    fn a_construct_survives_a_round_trip() {
        let dir = temp_dir();
        let file = File {
            camera: Pose {
                x: 12.5,
                z: -300.25,
                yaw: 0.123_456_789_012_345_6,
                pitch: -0.2,
            },
            radius: Some(4000.0),
            panes: vec![
                PaneFile {
                    kind: Kind::Program,
                    command: Some("foot -e sh -c 'echo \"hi\"'".into()),
                    plugin: None,
                    scene: None,
                    at: Some([1.0 / 3.0, 2000.0]),
                    size: Some([1280.0, 800.0]),
                },
                PaneFile {
                    kind: Kind::Plugin,
                    command: None,
                    plugin: Some("mc".into()),
                    scene: Some("mission".into()),
                    at: Some([-1408.7, 1999.9]),
                    size: None,
                },
            ],
        };
        save(&dir, "desk", &file).unwrap();
        assert_eq!(load(&dir, "desk").unwrap(), file);
        save(&dir, "empty", &File::default()).unwrap();
        assert_eq!(list(&dir), ["desk", "empty"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn wrong_files_and_names_are_refused() {
        let dir = temp_dir();
        assert!(save(&dir, "../evil", &File::default()).is_err());
        assert!(load(&dir, ".hidden").is_err());
        for text in [
            "panes: [{kind: program}]",
            "panes: [{kind: program, command: ' '}]",
            "panes: [{kind: plugin, plugin: mc}]",
            "panes: [{kind: plugin, plugin: 'mc x', scene: a}]",
            "panes: [{kind: plugin, plugin: mc, scene: a, command: foot}]",
            "panes: [{kind: program, command: foot, size: [0, 10]}]",
            "radius: 10",
            "camera: {x: .nan}",
            "camera: {roll: 1}",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
        // A hand-written file may leave the poses out.
        let file = parse("panes: [{kind: program, command: foot}]").unwrap();
        assert_eq!(file.camera, Pose::default());
        assert_eq!(file.panes[0].at, None);
    }
}
