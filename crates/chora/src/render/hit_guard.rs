//! Guard of the hit test of 2D scenes: the walk of `surface_under` and
//! `node_under` must give what the flat search of the 2D path gave before
//! planes. The reference below is that search, kept as it was: it walks
//! the draw list back to front and hits each node under its translation
//! and positive scale.

use chora_scene::{InputMode, NodeKind, Placed, Transform};
use serde_json::{Value, json};
use smithay::{
    desktop::WindowSurfaceType,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

use super::{node_under, surface_under};
use crate::{
    output::Spot,
    state::State,
    testing::{Client, Core, wayland::Wayland},
};

enum Hit {
    Surface(WlSurface, Point<f64, Logical>),
    Area,
}

fn translation_and_scale(t: &Transform) -> Option<(Point<f64, Logical>, (f64, f64))> {
    let (sx, sy) = t.scale_2d().filter(|(sx, sy)| *sx > 0.0 && *sy > 0.0)?;
    Some((t.translation_2d().into(), (sx, sy)))
}

fn inside(transform: &Transform, [w, h]: [f64; 2], pos: Point<f64, Logical>) -> bool {
    let Some((offset, (sx, sy))) = translation_and_scale(transform) else {
        return true;
    };
    let (x, y) = ((pos.x - offset.x) / sx, (pos.y - offset.y) / sy);
    (0.0..w).contains(&x) && (0.0..h).contains(&y)
}

fn flat_hit(state: &State, placed: &Placed<'_>, pos: Point<f64, Logical>) -> Option<Hit> {
    if !placed
        .clips
        .iter()
        .all(|(transform, size)| inside(transform, *size, pos))
    {
        return None;
    }
    let (offset, (sx, sy)) = translation_and_scale(&placed.transform)?;
    let local = Point::<f64, Logical>::from(((pos.x - offset.x) / sx, (pos.y - offset.y) / sy));
    match placed.node.kind {
        NodeKind::Surface => {
            let window = state.windows.get(&placed.node.id)?;
            let geo = window.geometry().loc.to_f64();
            let (surface, loc) = window.surface_under(local + geo, WindowSurfaceType::ALL)?;
            let on_surface = local - (loc.to_f64() - geo);
            Some(Hit::Surface(surface, pos - on_surface))
        }
        NodeKind::Rect | NodeKind::Image | NodeKind::Shader => {
            let [w, h] = placed.node.props.size.unwrap_or([0.0; 2]);
            ((0.0..w).contains(&local.x) && (0.0..h).contains(&local.y)).then_some(Hit::Area)
        }
        NodeKind::Group | NodeKind::Text => None,
    }
}

fn flat_found<'a>(
    state: &State,
    list: &'a [Placed<'a>],
    pos: Point<f64, Logical>,
) -> Option<(&'a Placed<'a>, Hit)> {
    list.iter()
        .rev()
        .filter(|p| p.input != InputMode::Pass)
        .find_map(|p| Some((p, flat_hit(state, p, pos)?)))
}

type SurfaceHit = (WlSurface, Point<f64, Logical>, Option<String>);

fn flat_surface_under(state: &State, spot: &Spot) -> Option<SurfaceHit> {
    let list = state.scene.draw_list();
    match flat_found(state, &list, spot.scene) {
        Some((placed, Hit::Surface(surface, location))) if placed.input == InputMode::Accept => {
            Some((surface, location, Some(placed.node.id.clone())))
        }
        _ => None,
    }
}

fn flat_node_under(state: &State, spot: &Spot) -> Option<String> {
    let list = state.scene.draw_list();
    let (placed, _) = flat_found(state, &list, spot.scene)?;
    (placed.input == InputMode::Accept).then(|| placed.node.id.clone())
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("a small number")
    }

    fn between(&mut self, low: u32, high: u32) -> f64 {
        f64::from(low + u32::try_from(self.below((high - low + 1) as usize)).expect("small"))
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

fn transform(rng: &mut Rng) -> Value {
    let scales = [0.5, 1.0, 1.5, 2.0];
    json!({"translate": [rng.between(0, 120) - 10.0, rng.between(0, 90) - 10.0],
           "scale": [rng.pick(&scales), rng.pick(&scales)]})
}

fn input(rng: &mut Rng) -> &'static str {
    ["accept", "accept", "accept", "pass", "block"][rng.below(5)]
}

fn random_scene(rng: &mut Rng, previous: bool) -> Vec<Value> {
    let mut commands = Vec::new();
    if previous {
        commands.push(json!({"remove": {"id": "world"}}));
    }
    commands.push(json!({"create": {"id": "world", "kind": "group", "parent": "root"}}));
    let mut groups = vec!["world".to_owned()];
    for number in 0..6 + rng.below(7) {
        let parent = rng.pick(&groups).clone();
        let id = format!("n{number}");
        if rng.below(2) == 0 {
            let mut props = json!({"transform": transform(rng), "input": input(rng)});
            if rng.below(3) == 0 {
                props["size"] = json!([rng.between(30, 160), rng.between(30, 120)]);
                props["clip"] = json!({"radius": rng.between(0, 8)});
            }
            commands.push(
                json!({"create": {"id": id, "kind": "group", "parent": parent, "props": props}}),
            );
            groups.push(id);
        } else {
            let props = json!({"transform": transform(rng), "input": input(rng),
                "size": [rng.between(20, 120), rng.between(20, 90)],
                "color": [0, 1, 0, 1], "radius": rng.pick(&[0.0, 0.0, 8.0])});
            commands.push(
                json!({"create": {"id": id, "kind": "rect", "parent": parent, "props": props}}),
            );
        }
    }
    for surface in ["s1", "s2"] {
        let parent = rng.pick(&groups).clone();
        commands.push(json!({"attach": {"surface": surface, "parent": parent}}));
        let props = json!({"transform": transform(rng), "input": input(rng)});
        commands.push(json!({"update": {"id": surface, "props": props}}));
    }
    commands
}

fn apply(wm: &mut Client, core: &mut Core, commands: &[Value]) {
    wm.ok(core, "scene.apply", json!({"commands": commands}));
}

#[test]
fn hits_match_the_flat_walk() {
    let mut core = Core::new();
    core.add_output("HDMI-1", (400, 300));
    let mut wayland = Wayland::connect(&mut core);
    for (name, size) in [("a", (100, 70)), ("b", (70, 60))] {
        let window = wayland.toplevel(&mut core, name);
        wayland.attach(&mut core, &window, size);
    }
    let mut wm = core.named("wm");
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let (mut surfaces, mut areas, mut nothing) = (0, 0, 0);
    for scene in 0..200 {
        let commands = random_scene(&mut rng, scene > 0);
        apply(&mut wm, &mut core, &commands);
        for _ in 0..40 {
            let at = Point::from((rng.between(0, 200), rng.between(0, 150)));
            let spot = core.state.spot(at).expect("a spot");
            let under = surface_under(&core.state, &spot).map(|u| (u.surface, u.location, u.node));
            assert_eq!(
                under,
                flat_surface_under(&core.state, &spot),
                "surface, scene {scene} at {at:?}"
            );
            let node = node_under(&core.state, &spot);
            assert_eq!(
                node,
                flat_node_under(&core.state, &spot),
                "node, scene {scene} at {at:?}"
            );
            surfaces += usize::from(under.is_some());
            areas += usize::from(under.is_none() && node.is_some());
            nothing += usize::from(node.is_none());
        }
    }
    assert!(
        surfaces > 500 && areas > 1000 && nothing > 1000,
        "the scenes cover surfaces, areas and misses: {surfaces} {areas} {nothing}"
    );
}
