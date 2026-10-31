//! The order of the scenes and their numbers: `order` of the settings
//! first, then the other scenes in the order they appeared. The number of
//! a scene is its place in that order.

use chora_kit::settings::Store as _;
use chora_scene::{
    Command,
    ipc::{ClientError, RpcError},
};
use serde::Deserialize;

use crate::{Mc, chapter_id, config::Layout, invalid, rejected, scene, title_of};

/// The scene that lists all scenes.
pub const OVERVIEW: &str = "overview";
/// The panel of the overview with one line per scene.
const LIST: &str = "scenes";

/// Parameters of `scene.rename`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rename {
    name: String,
    to: String,
}

/// Parameters of `scene.move`: `to` is the new number, from 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Move {
    name: String,
    to: usize,
}

impl Mc {
    /// The names of the scenes in their order.
    pub fn order(&self) -> Vec<String> {
        let first = &self.settings.value.order;
        let rest = self.appeared.iter().filter(|n| !first.contains(n));
        (first.iter().chain(rest))
            .filter(|n| self.scenes.contains_key(*n))
            .cloned()
            .collect()
    }

    /// The number of a scene: its own, or its place in the order.
    pub fn number(&self, name: &str) -> String {
        let own = self.scenes.get(name).and_then(|s| s.chapter.number.clone());
        own.unwrap_or_else(|| {
            let at = self.order().iter().position(|n| n == name);
            at.map_or_else(String::new, |at| (at + 1).to_string())
        })
    }

    /// Adds a scene at the end of the order, or replaces one in place.
    pub fn add_scene(&mut self, name: &str, scene: scene::Scene) {
        if self.scenes.insert(name.into(), scene).is_none() {
            self.appeared.retain(|n| n != name);
            self.appeared.push(name.into());
        }
    }

    /// The lines of the overview: `N | Title` for each scene.
    fn overview_lines(&self) -> Vec<String> {
        (self.order().iter())
            .map(|name| {
                let title =
                    (self.scenes[name].chapter.title.clone()).unwrap_or_else(|| title_of(name));
                format!("{} | {title}", self.number(name))
            })
            .collect()
    }

    /// The list panel of the overview with the scenes now, if the scene
    /// has one.
    pub fn overview_panel(&self) -> Option<scene::Panel> {
        let scene = self.scenes.get(OVERVIEW)?;
        let panel = scene.panels.iter().find(|p| p.id == LIST)?;
        Some(scene::Panel {
            lines: self.overview_lines(),
            ..panel.clone()
        })
    }

    /// Shows the numbers of the order now: in each head, in the running
    /// head and in the overview, all in one go.
    pub fn renumber(&mut self) -> Result<(), ClientError> {
        if let Some(panel) = self.overview_panel()
            && let Err(err) = self.replace_panel(OVERVIEW, &panel, false)
        {
            eprintln!("chora-mc: {OVERVIEW}: {}", err.message);
        }
        let ty = self.ty("chapter_number", None);
        let heads: Vec<String> = (self.scenes.iter())
            .filter(|(_, s)| Self::has_chapter(s) && self.layout(s) == Layout::Number)
            .map(|(n, _)| n.clone())
            .collect();
        let updates = (heads.iter())
            .map(|name| Command::Update {
                id: chapter_id(name, "number"),
                props: ty.props(&self.number(name)),
            })
            .collect();
        if self.built {
            self.apply(updates)?;
            heads.iter().try_for_each(|name| self.align(name))?;
        }
        self.render()
    }

    /// Moves a scene to the number `to` and saves the order.
    pub fn move_scene(&mut self, Move { name, to }: Move) -> Result<(), RpcError> {
        let mut order = self.order();
        let at = (order.iter().position(|n| *n == name))
            .ok_or_else(|| invalid(format!("no scene {name}")))?;
        if self.transient.contains(&name) {
            return Err(invalid(format!(
                "{name} ends with its app; it stays at the end"
            )));
        }
        if to == 0 {
            return Err(invalid("to: a number from 1".into()));
        }
        order.remove(at);
        order.insert((to - 1).min(order.len()), name);
        order.retain(|n| !self.transient.contains(n));
        self.save_order(order)?;
        self.renumber().map_err(rejected)
    }

    /// Takes a new order into the settings and their file.
    fn save_order(&mut self, order: Vec<String>) -> Result<(), RpcError> {
        let old = std::mem::replace(&mut self.settings.value.order, order);
        self.settings.save().inspect_err(|_| {
            self.settings.value.order = old;
        })
    }

    /// Renames a scene: its file, its entry in the order and the scene.
    /// A built-in scene without a file has nothing to rename.
    pub fn rename_scene(&mut self, Rename { name, to }: Rename) -> Result<(), RpcError> {
        scene::check_name(&to).map_err(|e| invalid(format!("to: {e}")))?;
        if !self.scenes.contains_key(&name) {
            return Err(invalid(format!("no scene {name}")));
        }
        if self.scenes.contains_key(&to) {
            return Err(invalid(format!("a scene {to} exists")));
        }
        if !self.transient.contains(&name) {
            let from = self.dir.join(format!("{name}.yaml"));
            if !from.exists() {
                return Err(invalid(format!(
                    "{name} is built in and has no file; scene.save it first"
                )));
            }
            let path = self.dir.join(format!("{to}.yaml"));
            std::fs::rename(&from, &path).map_err(|e| {
                RpcError::new(RpcError::REJECTED, format!("{}: {e}", path.display()))
            })?;
        }
        if self.settings.value.order.contains(&name) {
            let order = (self.settings.value.order.iter())
                .map(|n| if *n == name { to.clone() } else { n.clone() })
                .collect();
            self.save_order(order)?;
        }
        self.rename_in_memory(&name, &to);
        self.build().map_err(rejected)
    }

    /// Moves everything that the plugin keeps by the name of a scene to
    /// its new name; the nodes follow with a build.
    fn rename_in_memory(&mut self, name: &str, to: &str) {
        let rename = |n: &mut String| {
            if n == name {
                to.clone_into(n);
            }
        };
        if let Some(scene) = self.scenes.remove(name) {
            self.scenes.insert(to.into(), scene);
        }
        self.appeared.iter_mut().for_each(rename);
        self.current.iter_mut().for_each(rename);
        self.previous.iter_mut().for_each(rename);
        self.launched.iter_mut().for_each(|(n, _)| rename(n));
        for target in self.starting.values_mut() {
            if let crate::Target::Scene(n) = target {
                rename(n);
            }
        }
        if let Some(v) = self.windows.remove(name) {
            self.windows.insert(to.into(), v);
        }
        if let Some(v) = self.pids.remove(name) {
            self.pids.insert(to.into(), v);
        }
        if let Some(v) = self.scroll.remove(name) {
            self.scroll.insert(to.into(), v);
        }
        if self.transient.remove(name) {
            self.transient.insert(to.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, Rename};
    use crate::{chapter_id, region::Region, scene, scene::Scene, scene_id, testing};

    /// The order of the settings first, then the scenes as they appeared;
    /// a new scene keeps the numbers before it, and a new order shows in
    /// the heads and the running head at once.
    #[test]
    fn the_numbers_follow_the_order() {
        let (mut mc, model) = testing::mc("order", [1920, 1080]);
        assert_eq!(
            mc.order(),
            ["agenda", "morning", "agents", "mission", "terminal"]
        );
        mc.add_scene("news", Scene::app("foot"));
        mc.add_scene("alpha", Scene::app("foot"));
        mc.build().unwrap();
        assert_eq!(mc.number("terminal"), "5");
        assert_eq!(mc.number("news"), "6");
        assert_eq!(mc.number("alpha"), "7", "in the order it appeared");
        mc.show("agenda").unwrap();
        mc.settings.value.order = vec!["mission".into(), "agenda".into()];
        mc.renumber().unwrap();
        assert_eq!(mc.number("agents"), "3", "alphabetical among the rest");
        let number = testing::text(&model, &chapter_id("agenda", "number"));
        assert_eq!(number.as_deref(), Some("2"));
        let head = format!("{}-head", mc.region_id(Region::Header));
        assert_eq!(testing::text(&model, &head).as_deref(), Some("2  AGENDA"));
    }

    fn rename(name: &str, to: &str) -> Rename {
        Rename {
            name: name.into(),
            to: to.into(),
        }
    }

    /// A rename takes the file, the entry in the order, the number and
    /// the nodes along; a built-in scene without a file has none.
    #[test]
    fn a_rename_moves_the_file_and_the_order_entry() {
        let (mut mc, model) = testing::mc("rename", [1920, 1080]);
        let day = Scene::app("foot");
        scene::save(&mc.dir, "day", &day).unwrap();
        mc.add_scene("day", day);
        mc.settings.value.order.insert(1, "day".into());
        mc.build().unwrap();
        mc.rename_scene(rename("day", "week")).unwrap();
        assert!(mc.dir.join("week.yaml").exists() && !mc.dir.join("day.yaml").exists());
        assert_eq!(mc.settings.value.order[1], "week");
        let saved = std::fs::read_to_string(mc.settings.file()).unwrap();
        assert!(saved.contains("week"), "{saved}");
        assert_eq!(mc.number("week"), "2");
        assert!(testing::has(&model, &scene_id("week")));
        assert!(!testing::has(&model, &scene_id("day")));
        assert!(mc.rename_scene(rename("agenda", "today")).is_err());
        assert!(mc.rename_scene(rename("week", "agents")).is_err());
        assert!(mc.rename_scene(rename("week", "../x")).is_err());
    }

    /// A move saves the whole order and shows the new numbers at once;
    /// the scene of an app stays at the end.
    #[test]
    fn a_move_renumbers_and_saves_the_order() {
        let (mut mc, model) = testing::mc("move", [1920, 1080]);
        mc.show("agenda").unwrap();
        let to = |name: &str, to| Move {
            name: name.into(),
            to,
        };
        mc.move_scene(to("mission", 1)).unwrap();
        let order = ["mission", "agenda", "morning", "agents", "terminal"];
        assert_eq!(mc.settings.value.order, order);
        let saved = std::fs::read_to_string(mc.settings.file()).unwrap();
        assert!(saved.contains("- mission\n- agenda"), "{saved}");
        let number = testing::text(&model, &chapter_id("agenda", "number"));
        assert_eq!(number.as_deref(), Some("2"));
        let head = format!("{}-head", mc.region_id(Region::Header));
        assert_eq!(testing::text(&model, &head).as_deref(), Some("2  AGENDA"));
        mc.move_scene(to("mission", 99)).unwrap();
        assert_eq!(mc.order().last().map(String::as_str), Some("mission"));
        mc.transient.insert("agenda".into());
        assert!(mc.move_scene(to("agenda", 1)).is_err());
        assert!(mc.move_scene(to("nothing", 1)).is_err());
    }
}
