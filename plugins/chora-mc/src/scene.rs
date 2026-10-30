//! Scenes: what the whole screen shows for one task. A scene is a grid of
//! panels, or one app whose window fills the screen. Each scene is a YAML
//! file `NAME.yaml` in `$XDG_CONFIG_HOME/chora/mc/scenes/`.

use std::{collections::BTreeMap, io::Write as _, path::Path};

use serde::{Deserialize, Serialize};

/// The scenes that ship with the plugin; a file of the same name replaces
/// one.
pub const BUILT_IN: [(&str, &str); 2] = [
    ("agenda", include_str!("../scenes/agenda.yaml")),
    ("terminal", include_str!("../scenes/terminal.yaml")),
];

/// The most bytes of a text node of the core.
const MAX_TEXT: usize = 4096;

/// A scene as its file has it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// Columns and rows of the grid of the panels.
    #[serde(default = "one_cell")]
    pub grid: [u32; 2],
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panels: Vec<Panel>,
    /// A shell command whose window fills the screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
}

/// A title and lines of text.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub id: String,
    /// Column, row, width and height in cells of the grid.
    pub cell: [u32; 4],
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
}

fn one_cell() -> [u32; 2] {
    [1, 1]
}

/// An empty scene: the scene of a window that no scene started.
impl Default for Scene {
    fn default() -> Self {
        Self {
            grid: one_cell(),
            panels: Vec::new(),
            app: None,
        }
    }
}

impl Scene {
    /// A scene of one app.
    pub fn app(command: &str) -> Self {
        Self {
            app: Some(command.into()),
            ..Self::default()
        }
    }

    /// Checks what serde cannot.
    pub fn check(&self) -> Result<(), String> {
        let [cols, rows] = self.grid;
        if cols == 0 || rows == 0 {
            return Err("grid: at least one column and one row".into());
        }
        if self.app.as_deref().is_some_and(|a| a.trim().is_empty()) {
            return Err("app: an empty command".into());
        }
        for (i, panel) in self.panels.iter().enumerate() {
            let id = &panel.id;
            check_name(id).map_err(|e| format!("panel {id:?}: {e}"))?;
            if self.panels[..i].iter().any(|p| p.id == *id) {
                return Err(format!("panel {id}: twice"));
            }
            let [x, y, w, h] = panel.cell;
            if w == 0 || h == 0 || x + w > cols || y + h > rows {
                return Err(format!("panel {id}: the cell is outside the grid"));
            }
            panel.check_text()?;
        }
        Ok(())
    }
}

impl Panel {
    /// The lines as one text node shows them.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// With a tab in a line, the parts before and after the first tab
    /// of each line, as two columns.
    pub fn columns(&self) -> Option<(String, String)> {
        if !self.lines.iter().any(|l| l.contains('\t')) {
            return None;
        }
        let split = self
            .lines
            .iter()
            .map(|l| l.split_once('\t').unwrap_or(("", l)));
        let (times, texts): (Vec<&str>, Vec<&str>) = split.unzip();
        // A text node needs a text.
        let first = Some(times.join("\n")).filter(|t| !t.is_empty());
        Some((first.unwrap_or_else(|| " ".into()), texts.join("\n")))
    }

    /// A text node of the core takes at most 4 KiB.
    pub fn check_text(&self) -> Result<(), String> {
        if self.title.len() > MAX_TEXT || self.text().len() > MAX_TEXT {
            return Err(format!("panel {}: more than 4 KiB of text", self.id));
        }
        Ok(())
    }
}

/// A name of a scene or a panel is a file name and part of a node id.
pub fn check_name(name: &str) -> Result<(), String> {
    let ok = (1..=64).contains(&name.len())
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
    if ok {
        Ok(())
    } else {
        Err("a name has 1 to 64 letters, digits, '.', '_' or '-'".into())
    }
}

/// Parses and checks a scene.
pub fn parse(text: &str) -> Result<Scene, String> {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    let scene: Scene =
        serde_saphyr::from_str_with_options(text, options).map_err(|e| e.to_string())?;
    scene.check()?;
    Ok(scene)
}

/// The built-in scenes and the files in `dir`. A wrong file only logs.
pub fn load(dir: &Path) -> BTreeMap<String, Scene> {
    let mut scenes: BTreeMap<String, Scene> = BUILT_IN
        .iter()
        .map(|(name, text)| {
            (
                (*name).into(),
                parse(text).expect("built-in scenes are valid"),
            )
        })
        .collect();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return scenes;
    };
    for path in entries.flatten().map(|e| e.path()) {
        let Some(name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".yaml"))
        else {
            continue;
        };
        let read = std::fs::read_to_string(&path).map_err(|e| e.to_string());
        match read.and_then(|text| parse(&text)) {
            Ok(scene) if check_name(name).is_ok() => {
                scenes.insert(name.into(), scene);
            }
            Ok(_) => eprintln!("chora-mc: {}: a wrong name", path.display()),
            Err(err) => eprintln!("chora-mc: {}: {err}", path.display()),
        }
    }
    scenes
}

/// Writes `NAME.yaml` in `dir` through a temporary file, so that a crash
/// leaves the old file or the new one.
pub fn save(dir: &Path, name: &str, scene: &Scene) -> Result<(), String> {
    check_name(name)?;
    scene.check()?;
    let text = serde_saphyr::to_string(scene).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{name}.yaml"));
    let tmp = dir.join(format!(".{name}.yaml.tmp"));
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&tmp, &path)
    };
    write().map_err(|e| format!("{}: {e}", path.display()))
}

/// The box `[x, y, w, h]` of a cell on a screen of `size`, with `margin`
/// around the grid and between the cells.
pub fn place(cell: [u32; 4], grid: [u32; 2], size: [f64; 2], margin: f64) -> [f64; 4] {
    let [x, y, w, h] = cell.map(f64::from);
    let [cols, rows] = grid.map(f64::from);
    let cell_w = (size[0] - margin * (cols + 1.0)) / cols;
    let cell_h = (size[1] - margin * (rows + 1.0)) / rows;
    [
        margin + x * (cell_w + margin),
        margin + y * (cell_h + margin),
        w * cell_w + (w - 1.0) * margin,
        h * cell_h + (h - 1.0) * margin,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-mc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_saved_scene_loads_and_replaces_a_built_in_one() {
        let dir = dir("save");
        let scene = Scene {
            grid: [2, 1],
            panels: vec![Panel {
                id: "mail".into(),
                cell: [1, 0, 1, 1],
                title: "Mail".into(),
                lines: vec!["one".into(), "two".into()],
            }],
            app: None,
        };
        save(&dir, "agenda", &scene).unwrap();
        save(&dir, "news", &Scene::app("foot")).unwrap();
        std::fs::write(dir.join("broken.yaml"), "grid: [0, 1]\n").unwrap();
        let scenes = load(&dir);
        assert_eq!(scenes["agenda"], scene);
        assert_eq!(scenes["news"].app.as_deref(), Some("foot"));
        assert!(scenes.contains_key("terminal"), "the built-in scenes stay");
        assert!(!scenes.contains_key("broken"));
        assert!(!dir.join(".agenda.yaml.tmp").exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn mistakes_fail() {
        let dir = dir("wrong");
        assert!(save(&dir, "../evil", &Scene::app("foot")).is_err());
        assert!(!dir.exists(), "a wrong scene writes nothing");
        for wrong in [
            "grid: [0, 1]\n",
            "panels: [{id: a, cell: [0, 0, 2, 1]}]\n",
            "panels: [{id: a, cell: [0, 0, 1, 1]}, {id: a, cell: [0, 0, 1, 1]}]\n",
            "panels: [{id: a/b, cell: [0, 0, 1, 1]}]\n",
            "app: ' '\n",
            "frame: true\n",
        ] {
            assert!(parse(wrong).is_err(), "{wrong}");
        }
        let long = Panel {
            id: "a".into(),
            cell: [0, 0, 1, 1],
            title: String::new(),
            lines: vec!["x".repeat(MAX_TEXT + 1)],
        };
        assert!(long.check_text().is_err());
    }

    #[test]
    fn a_tab_makes_two_columns() {
        let mut panel = Panel {
            id: "a".into(),
            cell: [0, 0, 1, 1],
            title: String::new(),
            lines: vec!["9:00\tStandup".into(), "\tMail".into(), "Lunch".into()],
        };
        let columns = ("9:00\n\n".into(), "Standup\nMail\nLunch".into());
        assert_eq!(panel.columns(), Some(columns));
        panel.lines = vec!["\tMail".into()];
        assert_eq!(panel.columns(), Some((" ".into(), "Mail".into())));
        panel.lines = vec!["Mail".into()];
        assert_eq!(panel.columns(), None);
    }

    #[test]
    fn cells_share_the_screen_with_margins() {
        let size = [1000.0, 500.0];
        assert_eq!(
            place([0, 0, 1, 1], [1, 1], size, 20.0),
            [20.0, 20.0, 960.0, 460.0]
        );
        // Four columns of 225 with five margins of 20.
        assert_eq!(
            place([1, 0, 2, 1], [4, 1], size, 20.0),
            [265.0, 20.0, 470.0, 460.0]
        );
        assert_eq!(
            place([0, 1, 1, 1], [1, 2], size, 20.0),
            [20.0, 260.0, 960.0, 220.0]
        );
    }
}
