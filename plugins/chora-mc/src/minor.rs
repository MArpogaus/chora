//! mc-minor r1: the checks of mc-41, mc-14 and mc-13.

use chora_kit::testing::FakeCore;
use chora_scene::{Command, PropsUpdate};

use crate::region::Region;

/// The create commands of the panels of a built-in scene at 1280 x 800.
fn made(scene: &str) -> Vec<(String, PropsUpdate)> {
    let (mc, _) = crate::testing::mc("minor-made", [1280, 800]);
    let scene_of = mc.scenes.get(scene).cloned().expect("a built-in scene");
    scene_of
        .panels
        .iter()
        .flat_map(|panel| mc.panel(scene, panel))
        .filter_map(|c| match c {
            Command::Create { id, props, .. } => Some((id, props)),
            _ => None,
        })
        .collect()
}

#[test]
fn mc_13_no_collector_in_the_words_of_a_scene() {
    let (mc, _) = crate::testing::mc("minor-words", [1280, 800]);
    for (name, scene) in &mc.scenes {
        for panel in &scene.panels {
            let words = format!("{} {} {}", panel.title, panel.empty, panel.text());
            assert!(
                !words.to_lowercase().contains("collector"),
                "{name}/{}: {words}",
                panel.id
            );
        }
    }
}

#[test]
fn mc_13_the_empty_lines_of_all_pages_share_one_family_and_color() {
    let mut seen = Vec::new();
    for scene in ["morning", "mission", "agenda"] {
        for (id, props) in made(scene) {
            if id.ends_with("-empty") {
                seen.push((id, props.family.clone(), props.color));
            }
        }
    }
    assert!(seen.len() >= 4, "{seen:?}");
    for s in &seen {
        assert_eq!((&s.1, s.2), (&seen[0].1, seen[0].2), "{seen:?}");
    }
}

#[test]
fn mc_13_a_head_over_nothing_is_not_red() {
    let (mc, _) = crate::testing::mc("minor-red", [1280, 800]);
    let red = mc.settings.value.colors().get("red").copied();
    let heads = made("mission");
    let (_, errors) = heads
        .iter()
        .find(|(id, _)| id.ends_with("-errors-title"))
        .expect("the errors head");
    assert_ne!(errors.color, red, "the head of an empty panel is red");
}

#[test]
fn mc_13_the_divider_stands_midway_between_the_texts() {
    let (mc, _) = crate::testing::mc("minor-rule", [1920, 1080]);
    let scene = mc.scenes.get("morning").unwrap();
    let panel = |id: &str| scene.panels.iter().find(|p| p.id == id).unwrap();
    let lead = mc.place("morning", panel("lead")).unwrap();
    let weather = mc.place("morning", panel("weather")).unwrap();
    let Some(Command::Create { props, .. }) = mc.rule("morning", panel("weather")) else {
        panic!("the rule of the weather");
    };
    let rule = props.transform.unwrap().translation_2d().0;
    let middle = f64::midpoint(lead[0] + lead[2], weather[0]);
    assert!(
        (rule - middle).abs() <= 1.0,
        "rule at {rule}, middle {middle}"
    );
}

#[test]
fn mc_41_the_hit_of_a_wrapped_key_row_covers_every_line() {
    let core = FakeCore::start();
    let seen = core.model();
    core.respond(move |method, params| {
        let id = params["selector"].as_str()?.strip_prefix("id:")?;
        let model = seen.lock().unwrap();
        let props = &model.scene.node(id)?.props;
        let size = props.font_size.unwrap_or(16.0);
        let letters = f64::from(u16::try_from(props.text.as_deref().map_or(0, str::len)).unwrap());
        let natural = letters * 0.6 * size;
        let room = props.max_width.unwrap_or(natural).max(1.0);
        let rect = [
            0.0,
            0.0,
            natural.min(room),
            (natural / room).ceil().max(1.0) * size * props.line_height.unwrap_or(1.2),
        ];
        (method == "scene.query").then(|| serde_json::json!([{"rects": {"test": rect}}]))
    });
    let mut mc = crate::testing::mc_on(&core, "minor-hit", [1280, 800]);
    mc.show("agenda", true).unwrap();
    let moves = mc.regions().unwrap();
    let base = format!("{}-keys-2", mc.region_id(Region::Right));
    let words = core
        .model()
        .lock()
        .unwrap()
        .scene
        .node(&format!("{base}-words"))
        .unwrap()
        .props
        .clone();
    let size = words.font_size.unwrap_or(16.0);
    let letters = f64::from(u16::try_from(words.text.as_deref().unwrap().len()).unwrap());
    let lines = (letters * 0.6 * size / words.max_width.unwrap()).ceil();
    assert!(lines > 1.0, "the row does not wrap: {words:?}");
    let line = size * words.line_height.unwrap_or(1.2);
    let high = moves.iter().rev().find_map(|c| match c {
        Command::Update { id, props } if *id == format!("{base}-key-hit") => {
            props.size.map(|s| s[1])
        }
        _ => None,
    });
    let high = high.expect("the hit of the key row is sized");
    assert!(
        high + 0.5 >= lines * line,
        "hit {high} < {lines} lines of {line}"
    );
}

#[test]
fn mc_14_the_chat_body_is_19_and_the_arrow_keys_have_no_space_after_the_plus() {
    let hud = include_str!("../hud/hud.yaml");
    assert!(hud.contains("chat:\n  style: plain\n  font: TeX Gyre Pagella\n  size: 19"));
    let (mc, _) = crate::testing::mc("minor-keys", [1280, 800]);
    let rows = crate::keys::short(&mc.settings.value, false);
    assert_eq!(rows[1].key, "Super+←\u{2009}→", "{rows:?}");
}
