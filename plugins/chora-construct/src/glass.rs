//! The glass of the construct HUD in nodes of the core (construct HUD
//! spec, section 2; docs/construct-style.md, rules 18 to 25): a surface is
//! its shadow, its outer hairline, the tint with its inner ring, and the
//! highlight along its top edge; keycaps, wells, rows and the focus ring
//! are rects, the type is text nodes in the four sizes. The tokens are
//! `chora_kit::glass`. Without shaders, after the watchdog, the shadow
//! goes and the rest stays.
#![allow(
    clippy::many_single_char_names,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps,
    reason = "builders of nodes: x, y, w, h and r of shapes, one prop after the other"
)]

use std::collections::BTreeMap;

use chora_kit::glass::{KEY_GAP, KEY_HEIGHT, KEY_PAD, KEY_RADIUS, Material, Type, caps};
use chora_scene::{Border, Command, InputMode, NodeKind, PropsUpdate, Transform, Uniform};

use crate::hud::{Look, Role};

/// How far the shadow reaches out of a surface.
pub const REACH: f64 = 96.0;

fn rgb(c: [f64; 4]) -> [f64; 3] {
    [c[0], c[1], c[2]]
}

fn at(x: f64, y: f64) -> Option<Transform> {
    Some(Transform::translate(x, y, 0.0))
}

/// A node.
fn node(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

/// A group at a point.
#[must_use]
pub fn group(id: &str, parent: &str, [x, y]: [f64; 2], input: InputMode) -> Command {
    node(
        id,
        NodeKind::Group,
        parent,
        PropsUpdate {
            transform: at(x, y),
            input: Some(input),
            ..PropsUpdate::default()
        },
    )
}

/// A rounded rect in a colour.
#[must_use]
pub fn rect(id: &str, parent: &str, [x, y, w, h]: [f64; 4], r: f64, color: [f64; 4]) -> Command {
    node(
        id,
        NodeKind::Rect,
        parent,
        PropsUpdate {
            transform: at(x, y),
            size: Some([w, h]),
            radius: (r > 0.0).then_some(r),
            color: Some(color),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        },
    )
}

/// A line of text in a face of the four sizes, its line box at `[x, y]`.
#[must_use]
pub fn text(
    look: &Look,
    id: &str,
    parent: &str,
    (role, line): (Role, &str),
    [x, y]: [f64; 2],
    color: [f64; 4],
) -> Command {
    node(
        id,
        NodeKind::Text,
        parent,
        PropsUpdate {
            transform: at(x, y),
            input: Some(InputMode::Pass),
            ..look.text_props(role, line, rgb(color))
        },
    )
}

/// A face of the four sizes in a weight.
#[must_use]
pub const fn face(face: Type, weight: u16) -> Role {
    Role::Face(face, weight)
}

/// The meta that names a helper surface for the layout audit, and the
/// surface that holds it.
fn surface_meta(name: &str, inside: Option<&str>) -> BTreeMap<String, String> {
    let mut meta = BTreeMap::from([("surface".to_owned(), name.to_owned())]);
    if let Some(inside) = inside {
        meta.insert("in".into(), inside.into());
    }
    meta
}

/// What a surface is: its id, its parent, its place and size, its corner
/// radius, whether it takes the strong tint, and its name for the audit.
pub struct Surface<'a> {
    pub id: &'a str,
    pub parent: &'a str,
    pub rect: [f64; 4],
    pub radius: f64,
    pub strong: bool,
    pub name: &'a str,
    pub input: InputMode,
}

/// The nodes of a glass surface: the group `id` at its place, with the
/// shadow, the hairline, the tint and the highlight. The content goes
/// into the group after them.
#[must_use]
pub fn surface(look: &Look, m: &Material, s: &Surface<'_>) -> Vec<Command> {
    let [x, y, w, h] = s.rect;
    let r = s.radius.min(w / 2.0).min(h / 2.0);
    let id = s.id;
    let mut commands = vec![group(id, s.parent, [x, y], s.input)];
    if !look.plain {
        commands.push(shadow(look, m, &format!("{id}-shadow"), id, [w, h, r]));
    }
    commands.push(node(
        &format!("{id}-edge"),
        NodeKind::Rect,
        id,
        PropsUpdate {
            transform: at(-1.0, -1.0),
            size: Some([w + 2.0, h + 2.0]),
            radius: Some(r + 1.0),
            color: Some([0.0; 4]),
            border: Some(Some(Border {
                width: 1.0,
                color: m.edge_out,
            })),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        },
    ));
    commands.push(node(
        &format!("{id}-glass"),
        NodeKind::Rect,
        id,
        PropsUpdate {
            size: Some([w, h]),
            radius: Some(r),
            color: Some(if look.plain {
                m.plain(s.strong)
            } else {
                m.fill(s.strong)
            }),
            backdrop: Some(backdrop(m).filter(|_| !look.plain)),
            border: Some(Some(Border {
                width: 1.0,
                color: m.edge_in,
            })),
            input: Some(s.input),
            meta: Some(surface_meta(s.name, None)),
            ..PropsUpdate::default()
        },
    ));
    let inset = r - (r * r - (r - 1.0).powi(2)).max(0.0).sqrt();
    commands.push(rect(
        &format!("{id}-hi"),
        id,
        [inset, 0.0, w - 2.0 * inset, 1.0],
        0.0,
        m.highlight,
    ));
    commands
}

/// The blur of the core behind a surface, while `chora_kit::glass` asks
/// for it.
#[must_use]
pub fn backdrop(m: &Material) -> Option<chora_scene::Backdrop> {
    m.backdrop().map(|b| chora_scene::Backdrop {
        blur: b.blur,
        saturation: b.saturation,
    })
}

/// The shadow node of a surface of `[w, h, r]`.
fn shadow(look: &Look, m: &Material, id: &str, parent: &str, [w, h, r]: [f64; 3]) -> Command {
    let layer = |k: usize| {
        let l = m.shadows[k];
        Uniform::Vector(vec![l.y, l.blur / 2.0, l.alpha, 0.0])
    };
    let uniforms = BTreeMap::from([
        ("box".into(), Uniform::Vector(vec![w, h, r, REACH])),
        ("shade".into(), Uniform::Vector(m.shadow_rgb.to_vec())),
        ("near".into(), layer(0)),
        ("mid".into(), layer(1)),
        ("far".into(), layer(2)),
    ]);
    node(
        id,
        NodeKind::Shader,
        parent,
        PropsUpdate {
            shader: Some(look.shadow.into()),
            transform: at(-REACH, -REACH),
            size: Some([w + 2.0 * REACH, h + 2.0 * REACH]),
            uniforms: Some(uniforms),
            input: Some(InputMode::Pass),
            ..PropsUpdate::default()
        },
    )
}

/// The width of the keycaps of a chord, as the core measures their names.
#[must_use]
pub fn keycaps_width(look: &Look, keys: &[String]) -> f64 {
    let caps: f64 = keys.iter().map(|k| cap_width(look, k)).sum();
    #[allow(clippy::cast_precision_loss, reason = "a few keys")]
    let gaps = keys.len().saturating_sub(1) as f64 * KEY_GAP;
    caps + gaps
}

fn cap_width(look: &Look, key: &str) -> f64 {
    let ink = look.size(face(Type::Small, 500), key, None)[0];
    (ink + 2.0 * KEY_PAD).round().max(KEY_HEIGHT)
}

/// The keycaps of a chord side by side from `[x, y]`, 24 high, named
/// `{id}-0` on: the nodes and their width.
#[must_use]
pub fn keycaps(
    look: &Look,
    m: &Material,
    (id, parent): (&str, &str),
    keys: &[String],
    [x, y]: [f64; 2],
) -> (Vec<Command>, f64) {
    let mut commands = Vec::new();
    let mut left = x;
    for (k, key) in keys.iter().enumerate() {
        if k > 0 {
            left += KEY_GAP;
        }
        let w = cap_width(look, key);
        let cap = format!("{id}-{k}");
        commands.push(node(
            &cap,
            NodeKind::Rect,
            parent,
            PropsUpdate {
                transform: at(left, y),
                size: Some([w, KEY_HEIGHT]),
                radius: Some(KEY_RADIUS),
                color: Some(m.cap),
                border: Some(Some(Border {
                    width: 1.0,
                    color: m.cap_edge,
                })),
                input: Some(InputMode::Pass),
                ..PropsUpdate::default()
            },
        ));
        commands.push(rect(
            &format!("{cap}-low"),
            parent,
            [
                left + KEY_RADIUS,
                y + KEY_HEIGHT - 1.0,
                w - 2.0 * KEY_RADIUS,
                1.0,
            ],
            0.0,
            m.cap_low,
        ));
        let ink = look.size(face(Type::Small, 500), key, None)[0];
        commands.push(text(
            look,
            &format!("{cap}-text"),
            parent,
            (face(Type::Small, 500), key),
            [left + ((w - ink) / 2.0).round(), y + 2.0],
            m.text,
        ));
        left += w;
    }
    (commands, left - x)
}

/// The caps of the keys of a binding: `Super+Alt+1` as Super, Alt, 1.
#[must_use]
pub fn chord(binding: &str) -> Vec<String> {
    caps(binding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, hud::Metrics};

    #[test]
    fn a_surface_is_its_shadow_hairline_tint_and_highlight_named_for_the_audit() {
        let config: Config = serde_saphyr::from_str(crate::config::DEFAULT).unwrap();
        let metrics = Metrics::default();
        let look = Look {
            style: &config.style,
            colors: &config.style.themes.light,
            plain: false,
            shader: "/glass.frag",
            shadow: "/shadow.frag",
            glass: Material::of(false),
            metrics: &metrics,
        };
        let m = Material::of(false);
        let s = Surface {
            id: "construct-hints",
            parent: crate::NAME,
            rect: [100.0, 640.0, 600.0, 40.0],
            radius: 20.0,
            strong: false,
            name: "hints",
            input: InputMode::Pass,
        };
        let commands = surface(&look, &m, &s);
        let ids: Vec<&str> = commands
            .iter()
            .filter_map(|c| match c {
                Command::Create { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            ids,
            [
                "construct-hints",
                "construct-hints-shadow",
                "construct-hints-edge",
                "construct-hints-glass",
                "construct-hints-hi"
            ]
        );
        let Command::Create { props, .. } = &commands[3] else {
            panic!("the glass")
        };
        assert_eq!(props.meta.as_ref().unwrap()["surface"], "hints");
        assert!(
            (props.color.unwrap()[3] - 0.64).abs() < 1e-9,
            "the tint over the blur"
        );
        assert_eq!(props.backdrop.unwrap().map(|b| b.blur), Some(40.0));
        assert_eq!(props.radius, Some(20.0));
    }
}
