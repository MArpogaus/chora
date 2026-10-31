//! Scenes: what the whole screen shows for one task. A scene is a grid of
//! panels, or one app whose window fills its text block. Each scene is a YAML
//! file `NAME.yaml` in `$XDG_CONFIG_HOME/chora/mc/scenes/`.

use std::{collections::BTreeMap, io::Write as _, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    config::{Layout, Page, Style},
    region::Region,
};

/// The scenes that ship with the plugin; a file of the same name replaces
/// one.
pub const BUILT_IN: [(&str, &str); 5] = [
    ("agenda", include_str!("../scenes/agenda.yaml")),
    ("agents", include_str!("../scenes/agents.yaml")),
    ("mission", include_str!("../scenes/mission.yaml")),
    ("morning", include_str!("../scenes/morning.yaml")),
    ("terminal", include_str!("../scenes/terminal.yaml")),
];

/// The most bytes of a text node of the core.
pub const MAX_TEXT: usize = 4096;

/// The most columns and rows of the grid of a scene.
pub const MAX_GRID: u32 = 24;

/// A scene as its file has it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// Columns and rows of the grid of the panels, over `grid` of the
    /// settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid: Option<[u32; 2]>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panels: Vec<Panel>,
    /// A shell command whose window fills the text block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// The head of the scene.
    #[serde(default, skip_serializing_if = "Chapter::is_empty")]
    pub chapter: Chapter,
    /// Whether a region shows in this scene, over the settings.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub regions: BTreeMap<Region, bool>,
    /// The text block of this scene, over `page` of the settings: for
    /// example narrow columns side by side on a wide screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<Page>,
}

/// The head of a scene. Without them, the number is the place of the
/// scene, the title its name, and the running head the date.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Chapter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running_head: Option<String>,
    /// The layout of the head, over `chapter.layout` of the settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
}

impl Chapter {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// A title and lines of text.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub id: String,
    /// Column, row, width and height in cells of the grid.
    pub cell: [u32; 4],
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<String>,
    /// The style of the lines, `body` without a base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
    /// The style of the title, `heading` without a base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_style: Option<Style>,
    /// The style of the notes in the margin, `margin` without a base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_style: Option<Style>,
    /// A hairline in this color of the theme left of the panel, in the
    /// gap: a rule between columns, or a line for now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    /// The opacity of the panel, from 0 to 1: context that steps back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// A blur of the panel, its radius in pixels up to 16.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blur: Option<f64>,
}

/// An empty scene: the scene of a window that no scene started.
impl Default for Scene {
    fn default() -> Self {
        Self {
            grid: None,
            panels: Vec::new(),
            app: None,
            chapter: Chapter::default(),
            regions: BTreeMap::new(),
            page: None,
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

    /// Checks what serde cannot; `grid` is the grid of the settings, for
    /// a scene without its own.
    pub fn check(&self, grid: [u32; 2]) -> Result<(), String> {
        let [cols, rows] = self.grid.unwrap_or(grid);
        check_grid([cols, rows])?;
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
            let inside = |at: u32, size: u32, max| at.checked_add(size).is_some_and(|e| e <= max);
            if w == 0 || h == 0 || !inside(x, w, cols) || !inside(y, h, rows) {
                return Err(format!("panel {id}: the cell is outside the grid"));
            }
            panel.check_text()?;
            if panel.opacity.is_some_and(|o| !(0.0..=1.0).contains(&o))
                || panel.blur.is_some_and(|b| !(0.0..=16.0).contains(&b))
            {
                return Err(format!("panel {id}: an opacity from 0 to 1, a blur to 16"));
            }
        }
        if let Some(page) = &self.page {
            page.check()?;
        }
        let chapter = &self.chapter;
        let texts = [&chapter.number, &chapter.title, &chapter.running_head];
        if texts
            .iter()
            .flat_map(|t| t.iter())
            .any(|t| t.len() > MAX_TEXT)
        {
            return Err("chapter: more than 4 KiB of text".into());
        }
        Ok(())
    }
}

impl Panel {
    /// The lines as one text node shows them.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Whether the panel has neither a title nor lines.
    pub fn is_empty(&self) -> bool {
        self.title.is_empty() && self.lines.is_empty()
    }

    /// With a tab in a line, the parts before and after the first tab
    /// of each line: a note in the margin and its line.
    pub fn columns(&self) -> Option<Vec<(&str, &str)>> {
        if !self.lines.iter().any(|l| l.contains('\t')) {
            return None;
        }
        let split = self.lines.iter();
        Some(
            split
                .map(|l| l.split_once('\t').unwrap_or(("", l)))
                .collect(),
        )
    }

    /// A text node of the core takes at most 4 KiB.
    pub fn check_text(&self) -> Result<(), String> {
        if self.title.len() > MAX_TEXT || self.text().len() > MAX_TEXT {
            return Err(format!("panel {}: more than 4 KiB of text", self.id));
        }
        Ok(())
    }
}

/// A grid has 1 to `MAX_GRID` columns and rows.
pub fn check_grid([cols, rows]: [u32; 2]) -> Result<(), String> {
    if !(1..=MAX_GRID).contains(&cols) || !(1..=MAX_GRID).contains(&rows) {
        return Err(format!("grid: 1 to {MAX_GRID} columns and rows"));
    }
    Ok(())
}

/// The grid of the settings: a grid that the built-in scenes fit.
pub fn check_default_grid(grid: [u32; 2]) -> Result<(), String> {
    check_grid(grid)?;
    for (name, text) in BUILT_IN {
        parse(text, grid).map_err(|e| format!("grid: the built-in scene {name}: {e}"))?;
    }
    Ok(())
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
pub fn parse(text: &str, grid: [u32; 2]) -> Result<Scene, String> {
    let mut options = serde_saphyr::Options::default();
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    let scene: Scene =
        serde_saphyr::from_str_with_options(text, options).map_err(|e| e.to_string())?;
    scene.check(grid)?;
    Ok(scene)
}

/// The built-in scenes and the files in `dir`, on the `grid` of the
/// settings. A wrong file only logs.
#[allow(clippy::expect_used, reason = "the settings check the built-in scenes")]
pub fn load(dir: &Path, grid: [u32; 2]) -> BTreeMap<String, Scene> {
    let mut scenes: BTreeMap<String, Scene> = BUILT_IN
        .iter()
        .map(|(name, text)| {
            (
                (*name).into(),
                parse(text, grid).expect("the built-in scenes fit the grid"),
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
        let read = chora_kit::read_text(&path, chora_kit::MAX_TEXT).map_err(|e| e.to_string());
        match read.and_then(|text| parse(&text, grid)) {
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
pub fn save(dir: &Path, name: &str, scene: &Scene, grid: [u32; 2]) -> Result<(), String> {
    check_name(name)?;
    scene.check(grid)?;
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

/// The box `[x, y, w, h]` of a cell in the `area` `[x, y, w, h]`, with
/// `gap` between the columns and between the rows.
pub fn place(cell: [u32; 4], grid: [u32; 2], area: [f64; 4], gap: [f64; 2]) -> [f64; 4] {
    let [x, y, w, h] = cell.map(f64::from);
    let [cols, rows] = grid.map(f64::from);
    let cell_w = (area[2] - gap[0] * (cols - 1.0)) / cols;
    let cell_h = (area[3] - gap[1] * (rows - 1.0)) / rows;
    [
        area[0] + x * (cell_w + gap[0]),
        area[1] + y * (cell_h + gap[1]),
        // A small output leaves no room for the gaps.
        (w * cell_w + (w - 1.0) * gap[0]).max(0.0),
        (h * cell_h + (h - 1.0) * gap[1]).max(0.0),
    ]
}

/// The cells of panels on a grid of `cols` columns: in reading order,
/// each at most `cols` wide, at the top-most, then left-most place below
/// the panels before it. Answers the cells and the number of rows.
pub fn flow(cells: &[[u32; 4]], cols: u32) -> (Vec<[u32; 4]>, u32) {
    let cols = cols.max(1);
    let mut order: Vec<usize> = (0..cells.len()).collect();
    order.sort_by_key(|&i| (cells[i][1], cells[i][0]));
    let mut tops = vec![0; cols as usize];
    let mut flowed = cells.to_vec();
    for i in order {
        let [_, _, w, h] = cells[i];
        let w = w.clamp(1, cols);
        let top = |x: u32| tops[x as usize..(x + w) as usize].iter().copied().max();
        let (y, x) = (0..=cols - w)
            .filter_map(|x| Some((top(x)?, x)))
            .min()
            .unwrap_or_default();
        tops[x as usize..(x + w) as usize].fill(y + h);
        flowed[i] = [x, y, w, h];
    }
    (flowed, tops.into_iter().max().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID: [u32; 2] = [4, 3];

    fn dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("chora-mc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_saved_scene_loads_and_replaces_a_built_in_one() {
        let dir = dir("save");
        let scene = Scene {
            grid: Some([2, 1]),
            panels: vec![Panel {
                id: "mail".into(),
                cell: [1, 0, 1, 1],
                title: "Mail".into(),
                lines: vec!["one".into(), "two".into()],
                style: Some(Style {
                    base: Some("margin".into()),
                    size: Some(12.0),
                    ..Style::default()
                }),
                ..Panel::default()
            }],
            app: None,
            chapter: Chapter {
                number: Some("IV".into()),
                ..Chapter::default()
            },
            regions: [(Region::Right, true)].into(),
            page: Some(Page {
                measure: 20.0,
                margin: 0.0,
                gap: 2.0,
            }),
        };
        save(&dir, "agenda", &scene, GRID).unwrap();
        save(&dir, "news", &Scene::app("foot"), GRID).unwrap();
        std::fs::write(dir.join("broken.yaml"), "grid: [0, 1]\n").unwrap();
        let scenes = load(&dir, GRID);
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
        assert!(save(&dir, "../evil", &Scene::app("foot"), GRID).is_err());
        assert!(!dir.exists(), "a wrong scene writes nothing");
        for wrong in [
            "grid: [0, 1]\n",
            // Without a grid of its own, the cell is outside the grid of
            // the settings.
            "panels: [{id: a, cell: [4, 0, 1, 1]}]\n",
            "panels: [{id: a, cell: [0, 0, 1, 1]}, {id: a, cell: [0, 0, 1, 1]}]\n",
            "panels: [{id: a/b, cell: [0, 0, 1, 1]}]\n",
            "app: ' '\n",
            "frame: true\n",
            "page: {measure: 5, margin: 0, gap: 1}\n",
            "panels: [{id: a, cell: [0, 0, 1, 1], opacity: 2}]\n",
            // W11: a huge grid blanked mc, a cell near u32::MAX overflowed.
            "grid: [4294967295, 4294967295]\n",
            "grid: [25, 1]\n",
            "grid: [2, 1]\npanels: [{id: a, cell: [4294967295, 0, 1, 1]}]\n",
            "grid: [2, 1]\npanels: [{id: a, cell: [1, 0, 4294967295, 1]}]\n",
        ] {
            assert!(parse(wrong, GRID).is_err(), "{wrong}");
        }
        let long = Panel {
            lines: vec!["x".repeat(MAX_TEXT + 1)],
            ..Panel::default()
        };
        assert!(long.check_text().is_err());
    }

    #[test]
    fn a_tab_makes_two_columns() {
        let mut panel = Panel {
            lines: vec!["9:00\tStandup".into(), "\tMail".into(), "Lunch".into()],
            ..Panel::default()
        };
        let rows = vec![("9:00", "Standup"), ("", "Mail"), ("", "Lunch")];
        assert_eq!(panel.columns(), Some(rows));
        panel.lines = vec!["Mail".into()];
        assert_eq!(panel.columns(), None);
    }

    #[test]
    fn cells_share_the_area_with_gaps() {
        let area = [20.0, 20.0, 960.0, 460.0];
        assert_eq!(place([0, 0, 1, 1], [1, 1], area, [20.0, 20.0]), area);
        // Four columns of 225 with three gaps of 20.
        assert_eq!(
            place([1, 0, 2, 1], [4, 1], area, [20.0, 20.0]),
            [265.0, 20.0, 470.0, 460.0]
        );
        assert_eq!(
            place([0, 1, 1, 1], [1, 2], area, [20.0, 20.0]),
            [20.0, 260.0, 960.0, 220.0]
        );
    }

    #[test]
    fn panels_that_do_not_fit_wrap_in_reading_order() {
        // The morning scene on two columns: the lead, then the rest below.
        let cells = [
            [0, 0, 2, 2],
            [2, 0, 1, 1],
            [3, 0, 1, 2],
            [2, 1, 1, 1],
            [0, 2, 2, 1],
            [2, 2, 2, 1],
        ];
        let (flowed, rows) = flow(&cells, 2);
        assert_eq!(
            flowed,
            [
                [0, 0, 2, 2],
                [0, 2, 1, 1],
                [1, 2, 1, 2],
                [0, 3, 1, 1],
                [0, 4, 2, 1],
                [0, 5, 2, 1],
            ]
        );
        assert_eq!(rows, 6);
        assert_eq!(flow(&cells, 4).0, cells);
    }
}
