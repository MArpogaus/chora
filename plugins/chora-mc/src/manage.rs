//! The order of the scenes and their numbers: `order` of the settings
//! first, then the other scenes in the order they appeared. The number of
//! a scene is its place in that order.

use chora_scene::{Command, ipc::ClientError};

use crate::{Mc, chapter_id, config::Layout, scene, title_of};

/// The scene that lists all scenes.
pub const OVERVIEW: &str = "overview";
/// The panel of the overview with one line per scene.
const LIST: &str = "scenes";

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
}

#[cfg(test)]
mod tests {
    use crate::{chapter_id, region::Region, scene::Scene, testing};

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
}
