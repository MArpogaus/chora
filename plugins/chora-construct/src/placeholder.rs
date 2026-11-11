//! The placeholder of a pane whose first buffer has not come (construct
//! look v2, section 2): a glass fill the size of the pane with the working
//! ring and the name of the window, so that a new pane never shows a grey
//! slab. It sits under the window and goes with the first buffer.

use chora_kit::glass::Type;
use chora_scene::{Command, InputMode, NodeKind, PropsUpdate, Transform, Uniform};

use crate::{
    glass::{face, group, rect, text},
    hud::Look,
    panes::create,
};

/// The prefix of the group of a placeholder; the source of the pane
/// follows.
pub const WAIT: &str = "construct-wait-";

/// The diameter of the ring, and its mode in `glass.frag`: working.
const RING: f64 = 80.0;
const WORKING: f64 = 3.0;
/// The side of the node that draws the ring: the ring with a margin.
const SPOT: f64 = RING + 8.0;

/// The id of the placeholder of the pane of `source`.
#[must_use]
pub fn id(source: &str) -> String {
    format!("{WAIT}{source}")
}

/// The nodes of the placeholder of a pane of `size`, in the pane node
/// `pane`: the fill, the ring in the working mode, and `name` under it.
#[must_use]
pub fn draw(look: &Look, [w, h]: [f64; 2], pane: &str, source: &str, name: &str) -> Vec<Command> {
    let id = id(source);
    let m = &look.glass;
    let fill = if look.plain {
        m.plain(true)
    } else {
        m.fill(true)
    };
    let mut commands = vec![
        group(&id, pane, [0.0, 0.0], InputMode::Pass),
        rect(&format!("{id}-fill"), &id, [0.0, 0.0, w, h], 0.0, fill),
    ];
    let centre = [w / 2.0, h / 2.0 - RING / 2.0 - 8.0];
    if !look.plain {
        let ring = [SPOT / 2.0, SPOT / 2.0, RING, WORKING];
        let mut uniforms = look.glow_uniforms([SPOT, SPOT], 0.0, 0.0, ring, 0.0);
        if let Some(Uniform::Vector(accent)) = uniforms.get_mut("accent") {
            accent[3] = 0.0;
        }
        commands.push(create(
            &format!("{id}-ring"),
            NodeKind::Shader,
            &id,
            PropsUpdate {
                size: Some([SPOT, SPOT]),
                transform: Some(Transform::translate(
                    centre[0] - SPOT / 2.0,
                    centre[1] - SPOT / 2.0,
                    0.0,
                )),
                shader: Some(look.shader.into()),
                uniforms: Some(uniforms),
                input: Some(InputMode::Pass),
                ..PropsUpdate::default()
            },
        ));
    }
    let role = face(Type::Display, 300);
    let name_w = look.size(role, name, None)[0];
    commands.push(text(
        look,
        &format!("{id}-name"),
        &id,
        (role, name),
        [((w - name_w) / 2.0).round(), h / 2.0 + 8.0],
        m.text,
    ));
    commands
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, hud::Metrics};
    use chora_kit::glass::Material;
    use chora_scene::{InputMode, NodeKind, Uniform};

    fn drawn(plain: bool) -> Vec<Command> {
        let config: Config = serde_saphyr::from_str(crate::config::DEFAULT).unwrap();
        let metrics = Metrics::default();
        let look = Look {
            style: &config.style,
            colors: &config.style.themes.dark,
            plain,
            shader: "/glass.frag",
            shadow: "/shadow.frag",
            glass: Material::of(true),
            metrics: &metrics,
        };
        draw(
            &look,
            [1280.0, 720.0],
            "construct-pane-s1",
            "s1",
            "Terminal",
        )
    }

    fn created<'a>(commands: &'a [Command], id: &str) -> &'a chora_scene::PropsUpdate {
        commands
            .iter()
            .find_map(|c| match c {
                Command::Create { id: i, props, .. } if i == id => Some(props),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no node {id}"))
    }

    #[test]
    fn the_placeholder_fills_the_pane_and_names_the_window() {
        let commands = drawn(false);
        let group = commands.iter().find_map(|c| match c {
            Command::Create {
                id, kind, parent, ..
            } if id == "construct-wait-s1" => Some((kind, parent)),
            _ => None,
        });
        assert_eq!(
            group,
            Some((&NodeKind::Group, &"construct-pane-s1".to_owned()))
        );
        let fill = created(&commands, "construct-wait-s1-fill");
        assert_eq!(fill.size, Some([1280.0, 720.0]));
        assert_eq!(fill.input, Some(InputMode::Pass));
        let name = created(&commands, "construct-wait-s1-name");
        assert_eq!(name.text.as_deref(), Some("Terminal"));
    }

    #[test]
    fn the_placeholder_shows_the_working_ring_without_a_glow() {
        let commands = drawn(false);
        let ring = created(&commands, "construct-wait-s1-ring");
        let uniforms = ring.uniforms.as_ref().unwrap();
        let Some(Uniform::Vector(ring)) = uniforms.get("ring") else {
            panic!("no ring uniform");
        };
        assert_eq!((ring[2], ring[3]), (80.0, 3.0));
        let Some(Uniform::Vector(accent)) = uniforms.get("accent") else {
            panic!("no accent uniform");
        };
        assert_eq!(accent[3], 0.0);
    }

    #[test]
    fn without_shaders_the_placeholder_has_no_ring() {
        let commands = drawn(true);
        assert!(commands.iter().all(|c| !matches!(
            c,
            Command::Create { id, .. } if id.ends_with("-ring")
        )));
        created(&commands, "construct-wait-s1-fill");
    }
}
