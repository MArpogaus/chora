//! The `scene.boxes` call: per output the boxes it shows, bottom first,
//! with layer, opacity and owner, and the projected corners of planes
//! and of the nodes of a 3D context.

use chora_scene::boxes::{self, Layer, NodeBox, OutputBoxes};
use smithay::{desktop::layer_map_for_output, output::Output, wayland::shell::wlr_layer};

use super::{
    HashMap, Logical, NodeKind, Placed, Rectangle, State, Transform, Value, clients, output_area,
};

impl State {
    /// The boxes of every output (`scene.boxes`); a client socket sees
    /// only its own nodes and no layer surface, as in `scene.query`.
    pub(super) fn boxes(&self, conn: u64) -> Value {
        let fixed = self.ipc.fixed(conn);
        let all: boxes::Boxes = self
            .all_outputs()
            .iter()
            .map(|output| (output.name(), self.output_boxes(output, fixed)))
            .collect();
        serde_json::json!(all)
    }

    fn output_boxes(&self, output: &Output, fixed: Option<&str>) -> OutputBoxes {
        let area = output_area(output);
        let screen = Rectangle::from_size(area.size).to_f64();
        let name = output.name();
        let shell = |layers: &[(wlr_layer::Layer, Layer)]| {
            if fixed.is_some() {
                return Vec::new();
            }
            shell_boxes(output, layers)
        };
        let mut out = shell(&[
            (wlr_layer::Layer::Background, Layer::Background),
            (wlr_layer::Layer::Bottom, Layer::Bottom),
        ]);
        let views = [
            (Layer::Scene, self.camera(output), self.scene.draw_list()),
            (
                Layer::Overlay,
                Transform::IDENTITY,
                self.scene.overlay_draw_list(&name),
            ),
        ];
        for (layer, camera, list) in &views {
            let at: HashMap<&str, Transform> = list
                .iter()
                .map(|placed| (placed.node.id.as_str(), camera.then(&placed.transform)))
                .collect();
            let shown = list
                .iter()
                .filter(|p| p.opacity > 0.0)
                .filter(|p| fixed.is_none_or(|f| clients::own_node(p.node, f)));
            out.extend(shown.filter_map(|placed| self.node_box(placed, &at, screen, *layer)));
        }
        out.extend(shell(&[
            (wlr_layer::Layer::Top, Layer::Top),
            (wlr_layer::Layer::Overlay, Layer::ShellOverlay),
        ]));
        OutputBoxes {
            size: [screen.size.w, screen.size.h],
            boxes: out,
        }
    }

    /// The box of one placed node, `None` where it does not show.
    fn node_box(
        &self,
        placed: &Placed<'_>,
        at: &HashMap<&str, Transform>,
        screen: Rectangle<f64, Logical>,
        layer: Layer,
    ) -> Option<NodeBox> {
        let node = placed.node;
        let rect = self.screen_rect(placed, at, screen)?;
        let transform = at.get(node.id.as_str())?;
        let (_, near) = self.lineage(node, at)?;
        let in_context = near > crate::mesh::NEAR;
        let corners = (in_context || !transform.is_plain())
            .then(|| self.quad_size(node))
            .flatten()
            .and_then(|size| boxes::corners(transform, size, near));
        let text = node.kind == NodeKind::Text;
        Some(NodeBox {
            node: Some(node.id.clone()),
            kind: Some(node.kind),
            owner: node.owner.clone(),
            namespace: None,
            layer,
            order: self.scene.root_order(&node.id),
            rect: [rect.loc.x, rect.loc.y, rect.size.w, rect.size.h],
            opaque: boxes::opaque(node, placed.opacity),
            corners,
            text: node.props.text.clone().filter(|_| text),
            font_size: text.then(|| node.props.font_size.unwrap_or(crate::text::FONT_SIZE)),
        })
    }

    /// The size of the quad that a node draws as a plane: its size, or
    /// the geometry of its window.
    fn quad_size(&self, node: &chora_scene::Node) -> Option<[f64; 2]> {
        if let Some(size) = node.props.size {
            return Some(size);
        }
        let window = self.windows.get(node.id.as_str())?;
        let size = window.geometry().size;
        Some([f64::from(size.w), f64::from(size.h)])
    }
}

/// The layer surfaces of an output on some layers, bottom first.
fn shell_boxes(output: &Output, layers: &[(wlr_layer::Layer, Layer)]) -> Vec<NodeBox> {
    let map = layer_map_for_output(output);
    let mut out = Vec::new();
    for (shell, layer) in layers {
        for surface in map.layers_on(*shell) {
            let Some(geo) = map.layer_geometry(surface) else {
                continue;
            };
            let geo = geo.to_f64();
            out.push(NodeBox {
                node: None,
                kind: None,
                owner: None,
                namespace: Some(surface.namespace().to_owned()),
                layer: *layer,
                order: None,
                rect: [geo.loc.x, geo.loc.y, geo.size.w, geo.size.h],
                opaque: false,
                corners: None,
                text: None,
                font_size: None,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use wayland_protocols_wlr::layer_shell::v1::client::{
        zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::KeyboardInteractivity,
    };

    use crate::testing::{Client, Core, wayland::Wayland};

    fn boxes(client: &mut Client, core: &mut Core, output: &str) -> Vec<Value> {
        let all = client.ok(core, "scene.boxes", Value::Null);
        all[output]["boxes"].as_array().expect("boxes").clone()
    }

    fn of<'a>(boxes: &'a [Value], node: &str) -> &'a Value {
        boxes
            .iter()
            .find(|b| b["node"] == node)
            .unwrap_or_else(|| panic!("no box of {node} in {boxes:?}"))
    }

    fn order(boxes: &[Value]) -> Vec<&str> {
        boxes.iter().filter_map(|b| b["node"].as_str()).collect()
    }

    /// A scene of one client: an opaque floor, a clear glass, a faded,
    /// a fully clear and a hidden rect, a badge in the overlay, and a
    /// camera that pans by 5 px.
    fn floor_and_badge() -> (Core, Client) {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "floor", "kind": "rect", "parent": "root",
                "props": {"size": [100, 50], "color": [0, 0, 1, 1],
                          "transform": {"translate": [10, 20]}}}},
            {"create": {"id": "glass", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [1, 1, 1, 0.5]}}},
            {"create": {"id": "faded", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [1, 1, 1, 1], "opacity": 0.5}}},
            {"create": {"id": "gone", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [1, 1, 1, 1], "opacity": 0}}},
            {"create": {"id": "hidden", "kind": "rect", "parent": "root",
                "props": {"size": [10, 10], "color": [1, 1, 1, 1], "visible": false}}},
            {"create": {"id": "badge", "kind": "rect", "parent": "screen:out",
                "props": {"size": [8, 8], "color": [1, 0, 0, 1]}}},
            {"camera": {"output": "out", "transform": {"translate": [-5, 0]}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        (core, wm)
    }

    #[test]
    fn boxes_list_each_output_bottom_first_with_layer_and_owner() {
        let (mut core, mut wm) = floor_and_badge();
        let all = wm.ok(&mut core, "scene.boxes", Value::Null);
        assert_eq!(all["out"]["size"], json!([800.0, 600.0]));
        let list = boxes(&mut wm, &mut core, "out");
        assert_eq!(order(&list), ["floor", "glass", "faded", "badge"]);
        let floor = of(&list, "floor");
        assert_eq!(floor["rect"], json!([5.0, 20.0, 100.0, 50.0]), "camera");
        assert_eq!(floor["layer"], "scene");
        assert_eq!(floor["owner"], "wm");
        assert_eq!(floor["kind"], "rect");
        assert!(floor.get("corners").is_none(), "a flat node has no corners");
        let badge = of(&list, "badge");
        assert_eq!(badge["layer"], "overlay");
        assert_eq!(badge["rect"], json!([0.0, 0.0, 8.0, 8.0]), "no camera");
    }

    #[test]
    fn a_box_names_the_order_of_its_root_group() {
        let (mut core, mut wm) = floor_and_badge();
        let mut bar = core.named("bar");
        bar.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [
                {"create": {"id": "bar", "kind": "group", "parent": "root",
                    "props": {"order": "overlay"}}},
                {"create": {"id": "pill", "kind": "rect", "parent": "bar",
                    "props": {"size": [40, 10], "color": [0, 1, 0, 1]}}},
            ]}),
        );
        wm.ok(
            &mut core,
            "scene.apply",
            json!({"commands": [{"create": {"id": "page", "kind": "group", "parent": "root",
                "props": {"order": "background", "size": [800, 600], "clip": {}}}},
                {"create": {"id": "paper", "kind": "rect", "parent": "page",
                    "props": {"size": [800, 600], "color": [1, 1, 1, 1]}}}]}),
        );
        let list = boxes(&mut wm, &mut core, "out");
        assert_eq!(
            order(&list),
            ["page", "paper", "floor", "glass", "faded", "pill", "badge"]
        );
        assert_eq!(of(&list, "paper")["order"], "background");
        assert_eq!(of(&list, "pill")["order"], "overlay");
        assert_eq!(of(&list, "floor")["order"], "normal");
        assert!(
            of(&list, "badge").get("order").is_none(),
            "outside the root"
        );
    }

    #[test]
    fn a_box_says_whether_it_is_opaque() {
        let (mut core, mut wm) = floor_and_badge();
        let list = boxes(&mut wm, &mut core, "out");
        assert_eq!(of(&list, "floor")["opaque"], true);
        assert_eq!(of(&list, "glass")["opaque"], false, "a clear fill");
        assert_eq!(of(&list, "faded")["opaque"], false, "a faded node");
    }

    #[test]
    fn the_rect_of_a_box_is_the_rect_of_the_query() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "frame", "kind": "group", "parent": "root",
                "props": {"size": [100, 100], "clip": {},
                          "transform": {"translate": [20, 30]}}}},
            {"create": {"id": "half", "kind": "rect", "parent": "frame",
                "props": {"size": [20, 10], "color": [0, 0, 1, 1],
                          "transform": {"translate": [90, 10]}}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        let query = wm.ok(&mut core, "scene.query", json!({"selector": "id:half"}));
        let list = boxes(&mut wm, &mut core, "out");
        assert_eq!(of(&list, "half")["rect"], query[0]["rects"]["out"]);
    }

    /// Two faces of a cube in a 3D context, the eye in front of the centre
    /// of the front face: the right edge of the front face is the left
    /// edge of the side face, which runs away from the eye, so their
    /// projected corners meet and the far edge of the side face shrinks.
    #[test]
    fn the_faces_of_a_cube_report_projected_corners_that_meet() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let side = [
            0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 100.0, 0.0, 0.0, 1.0,
        ];
        let setup = json!({"commands": [
            {"create": {"id": "lens", "kind": "group", "parent": "root",
                "props": {"size": [100, 100], "perspective": 400,
                          "transform": {"translate": [50, 50]}}}},
            {"create": {"id": "front", "kind": "rect", "parent": "lens",
                "props": {"size": [100, 100], "color": [1, 0, 0, 1]}}},
            {"create": {"id": "side", "kind": "rect", "parent": "lens",
                "props": {"size": [100, 100], "color": [0, 1, 0, 1],
                          "transform": {"matrix": side}}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        let list = boxes(&mut wm, &mut core, "out");
        let corners = |node: &str| -> Vec<[f64; 2]> {
            serde_json::from_value(of(&list, node)["corners"].clone())
                .unwrap_or_else(|_| panic!("corners of {node}: {list:?}"))
        };
        let (front, side) = (corners("front"), corners("side"));
        assert_eq!(
            front[0],
            [50.0, 50.0],
            "the front face lies in the plane z = 0"
        );
        assert_eq!(front[2], [150.0, 150.0]);
        let near = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) < 1e-9;
        assert!(near(front[1], side[0]), "top edge: {front:?} {side:?}");
        assert!(near(front[2], side[3]), "bottom edge: {front:?} {side:?}");
        let far_height = side[2][1] - side[1][1];
        assert!(far_height < 100.0, "the far edge recedes: {side:?}");
    }

    /// A font check: a text node reports its text and font size, and its
    /// box is about one line of that size high and grows with the text.
    #[test]
    fn a_text_box_carries_its_text_and_a_line_of_its_font_size() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "short", "kind": "text", "parent": "root",
                "props": {"text": "Hi", "font_size": 20,
                          "transform": {"translate": [10, 10]}}}},
            {"create": {"id": "long", "kind": "text", "parent": "root",
                "props": {"text": "Hi there, a longer line", "font_size": 20,
                          "transform": {"translate": [10, 100]}}}},
            {"create": {"id": "plain", "kind": "text", "parent": "root",
                "props": {"text": "Hi"}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        let list = boxes(&mut wm, &mut core, "out");
        let short = of(&list, "short");
        assert_eq!(short["text"], "Hi");
        assert_eq!(short["font_size"], 20.0);
        assert_eq!(short["opaque"], false);
        assert_eq!(of(&list, "plain")["font_size"], 16.0, "the default size");
        let h = short["rect"][3].as_f64().unwrap();
        assert!((20.0..=40.0).contains(&h), "one line of 20 px: {short}");
        let width = |b: &Value| b["rect"][2].as_f64().unwrap();
        assert!(width(of(&list, "long")) > 4.0 * width(short), "{list:?}");
    }

    #[test]
    fn layer_surfaces_box_below_and_above_the_scene() {
        let mut core = Core::new();
        core.add_output("out", (800, 600));
        let mut wayland = Wayland::connect(&mut core);
        let mut wm = core.named("wm");
        wm.ok(&mut core, "scene.apply", crate::testing::rect("r"));
        let _wall = wayland.layer(&mut core, Layer::Background, KeyboardInteractivity::None);
        let _bar = wayland.layer(&mut core, Layer::Top, KeyboardInteractivity::None);
        let list = boxes(&mut wm, &mut core, "out");
        let layers: Vec<&str> = list.iter().map(|b| b["layer"].as_str().unwrap()).collect();
        assert_eq!(layers, ["background", "scene", "top"], "{list:?}");
        assert_eq!(list[2]["namespace"], "test");
        assert!(list[2].get("node").is_none());
        assert_eq!(list[2]["opaque"], false);
    }

    #[test]
    fn every_output_has_its_own_boxes() {
        let mut core = Core::new();
        core.add_output("left", (400, 300));
        core.add_output("right", (400, 300));
        let mut wm = core.named("wm");
        let setup = json!({"commands": [
            {"create": {"id": "mark", "kind": "rect", "parent": "screen:right",
                "props": {"size": [4, 4], "color": [1, 0, 0, 1]}}},
        ]});
        wm.ok(&mut core, "scene.apply", setup);
        assert_eq!(
            order(&boxes(&mut wm, &mut core, "left")),
            Vec::<&str>::new()
        );
        assert_eq!(order(&boxes(&mut wm, &mut core, "right")), ["mark"]);
    }
}
