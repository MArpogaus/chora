//! Tests that an element draws only inside the damage that the damage
//! tracker gives it. The tracker clears the damage and nothing else; an
//! element that draws past it blends its translucent pixels a second time
//! over the frame before.

use serde_json::json;

use crate::{
    backend::headless::{render_all, screen_row},
    testing::{Core, scratch_dir},
};

const ROW: i32 = 200;
const VEIL_PIXEL: i32 = 200;

fn red_channel(core: &mut Core, x: i32) -> u8 {
    screen_row(&mut core.state, 0, ROW, (x, 1))[0][0]
}

#[test]
fn a_translucent_shader_blends_once_when_only_a_corner_of_it_changes() {
    let dir = scratch_dir("chora-damage");
    let shader = dir.join("veil.frag");
    std::fs::write(
        &shader,
        "precision mediump float;\nvoid main() { gl_FragColor = vec4(0.5, 0.0, 0.0, 0.5); }\n",
    )
    .unwrap();
    let mut core = Core::with_renderer();
    let mut wm = core.named("wm");
    let grant = json!({"client": "wm", "capabilities": ["screen.capture"]});
    core.admin().ok(&mut core, "grants.set", grant);
    wm.ok(&mut core, "output.add", json!({"spec": "640x480"}));
    let commands = json!({"commands": [
        {"create": {"id": "dot", "kind": "rect", "parent": "root",
            "props": {"size": [20, 20], "color": [0, 0, 1, 1],
                      "transform": {"translate": [400, 300]}}}},
        {"create": {"id": "veil", "kind": "shader", "parent": "root",
            "props": {"size": [400, 300], "shader": shader,
                      "transform": {"translate": [100, 100]}}}},
    ]});
    wm.ok(&mut core, "scene.apply", commands);
    render_all(&mut core.state);
    render_all(&mut core.state);
    let first = red_channel(&mut core, VEIL_PIXEL);
    assert!(
        (126..=130).contains(&first),
        "the veil over black is half red: {first}"
    );
    let recolor = json!({"commands": [
        {"update": {"id": "dot", "props": {"color": [0, 1, 0, 1]}}},
    ]});
    wm.ok(&mut core, "scene.apply", recolor);
    render_all(&mut core.state);
    let second = red_channel(&mut core, VEIL_PIXEL);
    assert_eq!(second, first, "the veil blended again outside the damage");
    std::fs::remove_dir_all(&dir).unwrap();
}
