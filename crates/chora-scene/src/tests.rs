//! Tests of the scene rules.

use crate::{
    Command, Easing, Effect, Event, MAX_CAMERAS, MAX_DURATION, MAX_NODES, NodeKind, PropsUpdate,
    ROOT, Scene, SceneError, Selector, SurfaceInfo, SurfaceRequest, Transform, overlay_id,
};

fn group(id: &str, parent: &str) -> Command {
    Command::Create {
        id: id.into(),
        kind: NodeKind::Group,
        parent: parent.into(),
        props: PropsUpdate::default(),
    }
}

fn attach(surface: &str, parent: &str) -> Command {
    Command::Attach {
        surface: surface.into(),
        parent: parent.into(),
    }
}

fn scene_with_surface() -> Scene {
    let mut scene = Scene::default();
    scene.add_surface("s1", SurfaceInfo::default());
    scene
}

#[test]
fn a_failed_command_rolls_back_the_whole_transaction() {
    let mut scene = Scene::default();
    let before = scene.clone();
    let err = scene
        .apply("wm", &[group("work", ROOT), group("x", "missing")])
        .unwrap_err();
    assert_eq!(err.index, 1);
    assert_eq!(err.error, SceneError::Missing("missing".into()));
    assert_eq!(scene, before);
}

#[test]
fn only_the_owner_changes_a_node() {
    let mut scene = Scene::default();
    scene.apply("view", &[group("work", ROOT)]).unwrap();
    let update = Command::Update {
        id: "work".into(),
        props: PropsUpdate {
            opacity: Some(0.5),
            ..PropsUpdate::default()
        },
    };
    let err = scene
        .apply("agent", std::slice::from_ref(&update))
        .unwrap_err();
    assert_eq!(
        err.error,
        SceneError::NotOwner("work".into(), "view".into())
    );
    scene.apply("view", &[update]).unwrap();
}

#[test]
fn a_child_under_a_foreign_group_keeps_its_own_owner() {
    let mut scene = scene_with_surface();
    scene.apply("view", &[group("work", ROOT)]).unwrap();
    scene.apply("wm", &[attach("s1", "work")]).unwrap();
    assert_eq!(scene.node("s1").unwrap().owner.as_deref(), Some("wm"));
    assert_eq!(scene.node("work").unwrap().owner.as_deref(), Some("view"));
}

#[test]
fn an_owned_surface_cannot_be_taken() {
    let mut scene = scene_with_surface();
    scene.apply("wm", &[attach("s1", ROOT)]).unwrap();
    let err = scene.apply("agent", &[attach("s1", ROOT)]).unwrap_err();
    assert_eq!(err.error, SceneError::NotOwner("s1".into(), "wm".into()));
}

#[test]
fn disconnect_removes_nodes_and_detaches_surfaces() {
    let mut scene = scene_with_surface();
    scene
        .apply("wm", &[group("tiles", ROOT), attach("s1", "tiles")])
        .unwrap();
    assert_eq!(scene.disconnect("wm"), vec![Effect::Detached("s1".into())]);
    assert!(scene.node("tiles").is_none());
    let s1 = scene.node("s1").unwrap();
    assert_eq!((s1.parent.as_deref(), s1.owner.as_deref()), (None, None));
    assert_eq!(scene.node(ROOT).unwrap().children, Vec::<String>::new());
}

#[test]
fn removing_a_group_detaches_foreign_surfaces_inside() {
    let mut scene = scene_with_surface();
    scene.apply("view", &[group("work", ROOT)]).unwrap();
    scene.apply("wm", &[attach("s1", "work")]).unwrap();
    scene
        .apply("view", &[Command::Remove { id: "work".into() }])
        .unwrap();
    assert_eq!(scene.node("s1").unwrap().parent, None);
}

#[test]
fn move_into_own_subtree_is_rejected() {
    let mut scene = Scene::default();
    scene
        .apply("wm", &[group("a", ROOT), group("b", "a")])
        .unwrap();
    let mv = Command::Move {
        id: "a".into(),
        parent: Some("b".into()),
        index: None,
    };
    let err = scene.apply("wm", &[mv]).unwrap_err();
    assert_eq!(err.error, SceneError::Cycle("a".into()));
}

#[test]
fn configure_and_focus_become_effects_for_owned_surfaces() {
    let mut scene = scene_with_surface();
    let effects = scene
        .apply(
            "wm",
            &[
                attach("s1", ROOT),
                Command::Configure {
                    surface: "s1".into(),
                    size: Some([800, 600]),
                    states: vec![],
                },
                Command::Focus {
                    surface: Some("s1".into()),
                },
            ],
        )
        .unwrap();
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[1], Effect::Focus(Some("s1".into())));
}

#[test]
fn reset_puts_surfaces_in_a_grid_without_owner() {
    let mut scene = scene_with_surface();
    scene.add_surface("s2", SurfaceInfo::default());
    scene
        .apply("wm", &[group("tiles", ROOT), attach("s1", "tiles")])
        .unwrap();
    let effects = scene.reset([100.0, 0.0, 1000.0, 500.0]);
    assert!(scene.node("tiles").is_none());
    assert_eq!(scene.node(ROOT).unwrap().children, vec!["s1", "s2"]);
    assert_eq!(
        scene.node("s2").unwrap().props.transform,
        Transform::translate(600.0, 0.0, 0.0)
    );
    assert!(scene.node("s1").unwrap().owner.is_none());
    assert_eq!(effects.len(), 2);
}

#[test]
fn draw_list_composes_transforms_and_skips_hidden_nodes() {
    let mut scene = scene_with_surface();
    let moved = PropsUpdate {
        transform: Some(Transform::translate(10.0, 20.0, 0.0)),
        ..PropsUpdate::default()
    };
    let hidden = PropsUpdate {
        visible: Some(false),
        ..PropsUpdate::default()
    };
    let create = |id: &str, props: PropsUpdate| Command::Create {
        id: id.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props,
    };
    scene
        .apply(
            "wm",
            &[
                create("a", moved),
                create("hidden", hidden),
                attach("s1", "a"),
            ],
        )
        .unwrap();
    let list = scene.draw_list();
    let ids: Vec<&str> = list.iter().map(|p| p.node.id.as_str()).collect();
    assert_eq!(ids, ["a", "s1"]);
    assert_eq!(list[1].transform.translation_2d(), (10.0, 20.0));
}

#[test]
fn selectors_combine_conditions() {
    let mut scene = Scene::default();
    scene.add_surface(
        "s1",
        SurfaceInfo {
            app_id: Some("foot".into()),
            title: Some("make test".into()),
            ..SurfaceInfo::default()
        },
    );
    scene.add_surface("s2", SurfaceInfo::default());
    let sel: Selector = "kind:surface title~:test".parse().unwrap();
    let ids: Vec<&str> = scene.query(&sel).iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, ["s1"]);
    assert_eq!(scene.query(&"*".parse().unwrap()).len(), 3);
    assert!("colour:red".parse::<Selector>().is_err());
}

#[test]
fn commands_reject_unknown_fields() {
    let json = r#"[{"update": {"id": "a", "props": {"opacty": 0.5}}}]"#;
    assert!(serde_json::from_str::<Vec<Command>>(json).is_err());
    let json = r#"[{"update": {"id": "a", "props": {"opacity": 0.5}}}]"#;
    assert!(serde_json::from_str::<Vec<Command>>(json).is_ok());
}

#[test]
fn deep_trees_do_not_overflow_the_stack() {
    let mut scene = Scene::default();
    // As deep as the node limit allows.
    let commands: Vec<Command> = (0..MAX_NODES - 1)
        .map(|i| {
            let parent = if i == 0 {
                ROOT.to_owned()
            } else {
                format!("g{}", i - 1)
            };
            group(&format!("g{i}"), &parent)
        })
        .collect();
    scene.apply("wm", &commands).unwrap();
    assert_eq!(scene.draw_list().len(), MAX_NODES - 1);
}

#[test]
fn disconnect_lists_each_surface_once() {
    let mut scene = scene_with_surface();
    scene
        .apply("wm", &[group("a", ROOT), attach("s1", "a")])
        .unwrap();
    assert_eq!(scene.disconnect("wm"), vec![Effect::Detached("s1".into())]);
}

#[test]
fn remove_reports_the_surfaces_it_detaches() {
    let mut scene = scene_with_surface();
    scene.apply("view", &[group("work", ROOT)]).unwrap();
    scene.apply("wm", &[attach("s1", "work")]).unwrap();
    let effects = scene
        .apply("view", &[Command::Remove { id: "work".into() }])
        .unwrap();
    assert_eq!(effects, vec![Effect::Detached("s1".into())]);
}

#[test]
fn a_detached_surface_forgets_the_properties_of_its_old_owner() {
    let mut scene = scene_with_surface();
    let hide = Command::Update {
        id: "s1".into(),
        props: PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        },
    };
    scene.apply("wm", &[attach("s1", ROOT), hide]).unwrap();
    scene.disconnect("wm");
    assert!(scene.node("s1").unwrap().props.visible);
}

#[test]
fn the_owner_moves_its_surface_instead_of_attaching_it_again() {
    let mut scene = scene_with_surface();
    scene
        .apply("wm", &[attach("s1", ROOT), group("g", ROOT)])
        .unwrap();
    let err = scene.apply("wm", &[attach("s1", "g")]).unwrap_err();
    assert_eq!(err.error, SceneError::Attached("s1".into()));
}

#[test]
fn color_belongs_to_rects() {
    let mut scene = Scene::default();
    let sized = PropsUpdate {
        size: Some([10.0, 10.0]),
        color: Some([1.0, 0.0, 0.0, 1.0]),
        ..PropsUpdate::default()
    };
    let create = |kind| Command::Create {
        id: "n".into(),
        kind,
        parent: ROOT.into(),
        props: sized.clone(),
    };
    let err = scene.apply("wm", &[create(NodeKind::Group)]).unwrap_err();
    assert_eq!(err.error, SceneError::WrongKind("n".into(), "rect"));
    scene.apply("wm", &[create(NodeKind::Rect)]).unwrap();
}

#[test]
fn only_images_have_a_file_and_it_has_an_absolute_path() {
    let mut scene = Scene::default();
    let create = |id: &str, kind, image: &str| Command::Create {
        id: id.into(),
        kind,
        parent: ROOT.into(),
        props: PropsUpdate {
            image: Some(image.into()),
            slice: Some([4.0, 4.0, 4.0, 4.0]),
            ..PropsUpdate::default()
        },
    };
    let err = scene
        .apply("wm", &[create("r", NodeKind::Rect, "/a.png")])
        .unwrap_err();
    assert_eq!(err.error, SceneError::WrongKind("r".into(), "image"));
    let err = scene
        .apply("wm", &[create("i", NodeKind::Image, "a.png")])
        .unwrap_err();
    assert!(matches!(err.error, SceneError::Invalid(_)));
    scene
        .apply("wm", &[create("i", NodeKind::Image, "/a.png")])
        .unwrap();
    assert_eq!(
        scene.node("i").unwrap().props.image.as_deref(),
        Some("/a.png")
    );
}

#[test]
fn selector_values_can_hold_spaces_in_quotes() {
    let mut scene = Scene::default();
    scene.add_surface(
        "s1",
        SurfaceInfo {
            title: Some("make test".into()),
            ..SurfaceInfo::default()
        },
    );
    let sel: Selector = r#"kind:surface title:"make test""#.parse().unwrap();
    assert_eq!(scene.query(&sel).len(), 1);
    assert!(r#"title:"open"#.parse::<Selector>().is_err());
}

#[test]
fn invalid_sizes_and_transforms_are_rejected() {
    let mut scene = scene_with_surface();
    scene.apply("wm", &[attach("s1", ROOT)]).unwrap();
    let configure = Command::Configure {
        surface: "s1".into(),
        size: Some([-5, 10]),
        states: vec![],
    };
    let rect = |props| Command::Create {
        id: "r".into(),
        kind: NodeKind::Rect,
        parent: ROOT.into(),
        props,
    };
    let negative = PropsUpdate {
        size: Some([-1.0, 10.0]),
        ..PropsUpdate::default()
    };
    let infinite = PropsUpdate {
        transform: Some(Transform::translate(f64::INFINITY, 0.0, 0.0)),
        ..PropsUpdate::default()
    };
    for command in [configure, rect(negative), rect(infinite)] {
        let err = scene.apply("wm", &[command]).unwrap_err();
        assert!(matches!(err.error, SceneError::Invalid(_)), "{err}");
    }
}

#[test]
fn surface_ids_are_reserved_for_surfaces() {
    let mut scene = Scene::default();
    let err = scene.apply("wm", &[group("s7", ROOT)]).unwrap_err();
    assert!(matches!(err.error, SceneError::Invalid(_)), "{err}");
    for id in ["s", "sx", "s1a", "work"] {
        scene.apply("wm", &[group(id, ROOT)]).unwrap();
    }
}

#[test]
fn the_scene_has_a_node_limit() {
    let mut scene = Scene::default();
    let groups = |range: std::ops::Range<usize>| -> Vec<Command> {
        range.map(|i| group(&format!("g{i}"), ROOT)).collect()
    };
    // Neither surfaces nor the root count.
    for i in 1..=10 {
        scene.add_surface(&format!("s{i}"), SurfaceInfo::default());
    }
    scene.apply("wm", &groups(0..MAX_NODES)).unwrap();
    let err = scene
        .apply("wm", &groups(MAX_NODES..MAX_NODES + 1))
        .unwrap_err();
    assert!(matches!(err.error, SceneError::Invalid(_)), "{err}");
}

fn camera(output: &str, transform: Option<Transform>) -> Command {
    Command::Camera {
        output: output.into(),
        transform,
        duration: None,
        easing: Easing::Linear,
    }
}

fn camera_move(output: &str, transform: Option<Transform>, duration: f64) -> Command {
    Command::Camera {
        output: output.into(),
        transform,
        duration: Some(duration),
        easing: Easing::Linear,
    }
}

#[test]
fn a_camera_moves_from_where_it_is_and_tells_its_end() {
    let mut scene = Scene::default();
    scene.set_default_camera("DP-1", Transform::translate(-1920.0, 0.0, 0.0));
    let pan = Transform::translate(-2920.0, 0.0, 0.0);
    scene
        .apply("wm", &[camera_move("DP-1", Some(pan), 100.0)])
        .unwrap();
    assert!(scene.animating());
    assert_eq!(scene.camera("DP-1"), Some(&pan), "the target");
    let x = |scene: &Scene| scene.current_camera("DP-1").map(|t| t.translation_2d().0);
    assert_eq!(x(&scene), Some(-1920.0));
    assert_eq!(scene.advance_cameras(50.0), Vec::<String>::new());
    assert_eq!(x(&scene), Some(-2420.0));
    // A new move starts in the middle, here back to the default camera.
    scene
        .apply("wm", &[camera_move("DP-1", None, 100.0)])
        .unwrap();
    assert_eq!(x(&scene), Some(-2420.0));
    assert_eq!(scene.advance_cameras(100.0), vec!["DP-1".to_string()]);
    assert!(!scene.animating());
    assert_eq!(scene.current_camera("DP-1"), None);
}

#[test]
fn a_camera_move_needs_a_duration_up_to_a_minute_and_ends_with_its_owner() {
    let mut scene = Scene::default();
    scene.set_default_camera("DP-1", Transform::IDENTITY);
    let zoom = Transform::scale(2.0, 2.0, 1.0);
    for duration in [-1.0, f64::NAN, MAX_DURATION + 1.0] {
        let err = scene
            .apply("wm", &[camera_move("DP-1", Some(zoom), duration)])
            .unwrap_err();
        assert!(matches!(err.error, SceneError::Invalid(_)), "{err}");
    }
    scene
        .apply("wm", &[camera_move("DP-1", Some(zoom), 0.0)])
        .unwrap();
    assert!(!scene.animating(), "no duration, no move");
    scene
        .apply("wm", &[camera_move("DP-1", Some(zoom.then(&zoom)), 500.0)])
        .unwrap();
    assert!(scene.animating());
    scene.disconnect("wm");
    assert!(!scene.animating());
    assert_eq!(scene.current_camera("DP-1"), None);
}

#[test]
fn a_camera_belongs_to_the_client_that_sets_it_first() {
    let mut scene = Scene::default();
    let zoom = Transform::scale(2.0, 2.0, 1.0);
    let effects = scene.apply("wm", &[camera("DP-1", Some(zoom))]).unwrap();
    assert_eq!(effects, vec![Effect::Camera("DP-1".into())]);
    assert_eq!(scene.camera("DP-1"), Some(&zoom));
    let err = scene.apply("agent", &[camera("DP-1", None)]).unwrap_err();
    assert_eq!(
        err.error,
        SceneError::CameraOwner("DP-1".into(), "wm".into())
    );
    scene.apply("wm", &[camera("DP-1", None)]).unwrap();
    assert_eq!(scene.camera("DP-1"), None);
    scene.apply("agent", &[camera("DP-1", Some(zoom))]).unwrap();
}

#[test]
fn a_disconnect_restores_the_default_camera() {
    let mut scene = Scene::default();
    let pan = Transform::translate(-100.0, 0.0, 0.0);
    scene.apply("wm", &[camera("DP-1", Some(pan))]).unwrap();
    assert_eq!(scene.disconnect("wm"), vec![Effect::Camera("DP-1".into())]);
    assert_eq!(scene.camera("DP-1"), None);
    scene.apply("wm", &[camera("DP-1", Some(pan))]).unwrap();
    scene.reset([0.0, 0.0, 100.0, 100.0]);
    assert_eq!(scene.camera("DP-1"), None);
}

#[test]
fn a_camera_needs_a_positive_2d_scale() {
    let mut scene = Scene::default();
    let mut perspective = [0.0; 16];
    for i in [0, 5, 10, 15] {
        perspective[i] = 1.0;
    }
    perspective[11] = 0.5;
    let perspective: Transform =
        serde_json::from_value(serde_json::json!({ "matrix": perspective })).unwrap();
    for transform in [
        Transform::scale(0.0, 1.0, 1.0),
        Transform::scale(1e-9, 1e-9, 1.0),
        Transform::rotate_z(30.0),
        Transform::translate(f64::NAN, 0.0, 0.0),
        Transform::translate(1e12, 0.0, 0.0),
        perspective,
    ] {
        let err = scene
            .apply("wm", &[camera("DP-1", Some(transform))])
            .unwrap_err();
        assert!(matches!(err.error, SceneError::Invalid(_)), "{err}");
    }
}

#[test]
fn a_camera_ignores_the_z_axis() {
    let mut scene = Scene::default();
    let uniform: Transform = serde_json::from_value(serde_json::json!({"scale": 1.5})).unwrap();
    scene.apply("wm", &[camera("DP-1", Some(uniform))]).unwrap();
    assert_eq!(scene.camera("DP-1"), Some(&Transform::scale(1.5, 1.5, 1.0)));
    let lifted = Transform::translate(10.0, 0.0, 5.0);
    scene.apply("wm", &[camera("DP-1", Some(lifted))]).unwrap();
    assert_eq!(
        scene.camera("DP-1"),
        Some(&Transform::translate(10.0, 0.0, 0.0))
    );
}

#[test]
fn adding_a_surface_again_keeps_its_node() {
    let mut scene = scene_with_surface();
    scene
        .apply("wm", &[group("g", ROOT), attach("s1", "g")])
        .unwrap();
    assert!(!scene.add_surface("s1", SurfaceInfo::default()));
    assert_eq!(scene.node("s1").unwrap().parent.as_deref(), Some("g"));
    scene.remove_surface("s1");
    assert!(scene.draw_list().iter().all(|p| p.node.id != "s1"));
}

#[test]
fn the_number_of_cameras_has_a_limit() {
    let mut scene = Scene::default();
    let pan = Some(Transform::translate(-10.0, 0.0, 0.0));
    let cameras: Vec<Command> = (0..MAX_CAMERAS)
        .map(|i| camera(&format!("out-{i}"), pan))
        .collect();
    scene.apply("wm", &cameras).unwrap();
    // A camera of a known output can still change.
    scene
        .apply("wm", &[camera("out-0", None), camera("out-0", pan)])
        .unwrap();
    let err = scene.apply("wm", &[camera("one-more", pan)]).unwrap_err();
    assert!(matches!(err.error, SceneError::Invalid(_)), "{err}");
}

#[test]
fn a_request_event_names_the_request() {
    let event = Event::SurfaceRequested {
        surface: "s1".into(),
        request: SurfaceRequest::Fullscreen,
    };
    assert_eq!(
        serde_json::to_value(&event).unwrap(),
        serde_json::json!({"event": "surface.requested", "surface": "s1", "request": "fullscreen"})
    );
}

#[test]
fn an_animation_moves_a_node_and_ends_with_the_targets() {
    let mut scene = Scene::default();
    let rect = Command::Create {
        id: "r".into(),
        kind: NodeKind::Rect,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    let animate = |x: f64, duration: f64| Command::Animate {
        id: "r".into(),
        props: PropsUpdate {
            transform: Some(Transform::translate(x, 0.0, 0.0)),
            visible: Some(false),
            ..PropsUpdate::default()
        },
        duration,
        easing: Easing::Linear,
    };
    scene.apply("wm", &[rect, animate(100.0, 200.0)]).unwrap();
    assert!(scene.animating());
    assert_eq!(scene.advance(100.0), Vec::<String>::new());
    let props = &scene.node("r").unwrap().props;
    assert_eq!(props.transform.translation_2d(), (50.0, 0.0));
    assert!(props.visible);
    assert_eq!(scene.advance(150.0), vec!["r".to_string()]);
    let props = &scene.node("r").unwrap().props;
    assert_eq!(props.transform.translation_2d(), (100.0, 0.0));
    assert!(!props.visible);
    assert!(!scene.animating());
}

#[test]
fn an_animation_needs_a_duration_up_to_a_minute_and_the_owner() {
    let mut scene = Scene::default();
    let rect = Command::Create {
        id: "r".into(),
        kind: NodeKind::Rect,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    scene.apply("wm", &[rect]).unwrap();
    let animate = |duration: f64| Command::Animate {
        id: "r".into(),
        props: PropsUpdate::default(),
        duration,
        easing: Easing::Linear,
    };
    for duration in [-1.0, 60_001.0, f64::NAN] {
        let err = scene.apply("wm", &[animate(duration)]).unwrap_err();
        assert!(matches!(err.error, SceneError::Invalid(_)));
    }
    let err = scene.apply("other", &[animate(10.0)]).unwrap_err();
    assert!(matches!(err.error, SceneError::NotOwner(..)));
}

#[test]
fn an_update_or_a_removal_ends_an_animation() {
    let mut scene = Scene::default();
    let rect = Command::Create {
        id: "r".into(),
        kind: NodeKind::Rect,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    let animate = Command::Animate {
        id: "r".into(),
        props: PropsUpdate {
            opacity: Some(0.0),
            ..PropsUpdate::default()
        },
        duration: 100.0,
        easing: Easing::EaseOut,
    };
    let update = Command::Update {
        id: "r".into(),
        props: PropsUpdate::default(),
    };
    scene.apply("wm", &[rect, animate.clone(), update]).unwrap();
    assert!(!scene.animating());
    scene.apply("wm", &[animate]).unwrap();
    scene
        .apply("wm", &[Command::Remove { id: "r".into() }])
        .unwrap();
    assert!(!scene.animating());
    assert_eq!(scene.advance(200.0), Vec::<String>::new());
}

#[test]
fn a_node_reads_back_from_its_json() {
    let mut scene = Scene::default();
    let rect = Command::Create {
        id: "r".into(),
        kind: NodeKind::Rect,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    scene.apply("wm", &[rect]).unwrap();
    let node = scene.node("r").unwrap();
    let json = serde_json::to_value(node).unwrap();
    let back: crate::Node = serde_json::from_value(json).unwrap();
    assert_eq!(&back, node);
}

#[test]
fn dropped_effects_stay_gone_during_an_animation() {
    let mut scene = Scene::default();
    let rect = Command::Create {
        id: "r".into(),
        kind: NodeKind::Rect,
        parent: ROOT.into(),
        props: PropsUpdate {
            size: Some([10.0, 10.0]),
            mesh: Some(crate::Mesh::flat(1, 1, [10.0, 10.0])),
            ..PropsUpdate::default()
        },
    };
    let fade = Command::Animate {
        id: "r".into(),
        props: PropsUpdate {
            opacity: Some(0.0),
            ..PropsUpdate::default()
        },
        duration: 1000.0,
        easing: Easing::Linear,
    };
    scene.apply("wm", &[rect, fade]).unwrap();
    assert!(
        !scene.drop_effects("panel"),
        "only the effects of the owner"
    );
    assert!(scene.drop_effects("wm"));
    scene.advance(100.0);
    assert!(scene.node("r").unwrap().props.mesh.is_none());
    assert!(!scene.drop_effects("wm"));
}

/// A create command from JSON props.
fn create(id: &str, kind: &str, props: &serde_json::Value) -> Command {
    serde_json::from_value(serde_json::json!({
        "create": {"id": id, "kind": kind, "parent": ROOT, "props": props}
    }))
    .unwrap()
}

#[test]
fn a_rounded_rect_with_a_border_is_a_layer() {
    let mut scene = Scene::default();
    let props = serde_json::json!({
        "size": [100, 40], "color": [1, 1, 1, 1], "radius": 8,
        "border": {"width": 2, "color": [0, 0, 0, 1]}
    });
    scene
        .apply("wm", &[create("button", "rect", &props)])
        .unwrap();
    let node = scene.node("button").unwrap();
    assert!(node.is_layer());
    assert!((node.props.radius - 8.0).abs() < f64::EPSILON);
    let remove: Command = serde_json::from_value(serde_json::json!({
        "update": {"id": "button", "props": {"border": null, "radius": 0}}
    }))
    .unwrap();
    scene.apply("wm", &[remove]).unwrap();
    let node = scene.node("button").unwrap();
    assert!(node.props.border.is_none());
    assert!(!node.is_layer());
}

#[test]
fn corners_belong_to_rects_and_shaders_to_shader_nodes() {
    let mut scene = Scene::default();
    for (kind, props) in [
        ("group", serde_json::json!({"radius": 4})),
        ("rect", serde_json::json!({"shader": "/x.frag"})),
        ("rect", serde_json::json!({"radius": -1})),
        ("rect", serde_json::json!({"border": {"width": 1}})),
        ("shader", serde_json::json!({"size": [10, 10]})),
        ("shader", serde_json::json!({"shader": "x.frag"})),
    ] {
        // A border without a color does not even parse.
        let command: Result<Command, _> = serde_json::from_value(serde_json::json!({
            "create": {"id": "n", "kind": kind, "parent": ROOT, "props": props}
        }));
        if let Ok(command) = command {
            assert!(scene.apply("wm", &[command]).is_err(), "{kind} {props}");
        }
    }
}

#[test]
fn a_shader_node_reads_other_nodes_but_not_itself() {
    let mut scene = Scene::default();
    let props = serde_json::json!({
        "size": [100, 100], "shader": "/s.frag",
        "uniforms": {"strength": 0.5, "picture": {"node": "other"}}
    });
    scene
        .apply("wm", &[create("glow", "shader", &props)])
        .unwrap();
    assert!(scene.node("glow").unwrap().is_layer());
    let own = serde_json::json!({
        "size": [1, 1], "shader": "/s.frag", "uniforms": {"me": {"node": "me"}}
    });
    assert!(scene.apply("wm", &[create("me", "shader", &own)]).is_err());
    let effect = serde_json::json!({
        "size": [1, 1], "effects": [{"shader": "/e.frag", "uniforms": {"t": {"node": "glow"}}}]
    });
    assert!(scene.apply("wm", &[create("e", "group", &effect)]).is_err());
}

#[test]
fn a_clip_needs_no_surface_and_makes_a_layer() {
    let mut scene = scene_with_surface();
    let clip = serde_json::json!({"size": [50, 50], "clip": {"radius": 10}});
    scene.apply("wm", &[create("box", "group", &clip)]).unwrap();
    assert!(scene.node("box").unwrap().is_layer());
    let on_surface: Command = serde_json::from_value(serde_json::json!({
        "update": {"id": "s1", "props": {"clip": {}}}
    }))
    .unwrap();
    scene.apply("wm", &[attach("s1", "box")]).unwrap();
    assert!(scene.apply("wm", &[on_surface]).is_err());
}

#[test]
fn the_watchdog_weighs_the_visible_nodes_of_each_owner() {
    let mut scene = Scene::default();
    let plain = serde_json::json!({"size": [10, 10], "color": [1, 1, 1, 1]});
    let round = serde_json::json!({"size": [10, 10], "color": [1, 1, 1, 1], "radius": 4});
    let effect = serde_json::json!({"size": [10, 10], "effects": [{"shader": "/e.frag"}]});
    let hidden = serde_json::json!({"size": [10, 10], "visible": false});
    scene.apply("wm", &[create("a", "rect", &plain)]).unwrap();
    scene.apply("wm", &[create("b", "rect", &hidden)]).unwrap();
    scene
        .apply("panel", &[create("c", "rect", &round)])
        .unwrap();
    scene
        .apply("loader", &[create("d", "group", &effect)])
        .unwrap();
    let load = scene.load_by_owner();
    assert_eq!(load.get("wm"), Some(&1), "a hidden node costs nothing");
    assert_eq!(load.get("panel"), Some(&26));
    assert_eq!(load.get("loader"), Some(&51));
}

#[test]
fn the_watchdog_takes_the_shader_of_a_shader_node() {
    let mut scene = Scene::default();
    let props = serde_json::json!({"size": [10, 10], "shader": "/s.frag"});
    scene.apply("wm", &[create("s", "shader", &props)]).unwrap();
    assert!(scene.drop_effects("wm"));
    assert!(scene.node("s").unwrap().props.shader.is_none());
}

#[test]
fn a_client_that_cleared_the_focus_gives_it_back() {
    let mut scene = scene_with_surface();
    scene
        .apply(
            "wm",
            &[Command::Attach {
                surface: "s1".into(),
                parent: ROOT.into(),
            }],
        )
        .unwrap();
    scene.set_focused(Some("s1"));
    let focus = |surface: Option<&str>| Command::Focus {
        surface: surface.map(Into::into),
    };
    // Another client cannot take the focus of a surface of the wm ...
    assert!(scene.apply("walk", &[focus(Some("s1"))]).is_err());
    // ... but after it cleared the focus, it gives it back, once.
    scene.apply("walk", &[focus(None)]).unwrap();
    scene.apply("walk", &[focus(Some("s1"))]).unwrap();
    assert!(scene.apply("walk", &[focus(Some("s1"))]).is_err());
    // A focus that the user gave elsewhere since ends the return.
    scene.apply("walk", &[focus(None)]).unwrap();
    scene.set_focused(Some("s2"));
    assert!(scene.apply("walk", &[focus(Some("s1"))]).is_err());
    // A transaction that focuses an own surface and clears the focus gives
    // it back to that surface.
    scene.add_surface("s5", SurfaceInfo::default());
    scene
        .apply(
            "walk",
            &[Command::Attach {
                surface: "s5".into(),
                parent: ROOT.into(),
            }],
        )
        .unwrap();
    scene.set_focused(Some("s1"));
    scene
        .apply("walk", &[focus(Some("s5")), focus(None)])
        .unwrap();
    assert!(scene.apply("walk", &[focus(Some("s1"))]).is_err());
    scene.apply("walk", &[focus(Some("s5"))]).unwrap();
    // So does the end of the client.
    scene.set_focused(Some("s1"));
    scene.apply("walk", &[focus(None)]).unwrap();
    scene.disconnect("walk");
    assert!(scene.apply("walk", &[focus(Some("s1"))]).is_err());
}

#[test]
fn a_gone_surface_is_no_missing_node_of_the_client() {
    let mut scene = Scene::default();
    let focus = Command::Focus {
        surface: Some("s7".into()),
    };
    assert!(scene.apply("wm", &[focus]).unwrap_err().is_gone_surface());
    let remove = Command::Remove { id: "tiles".into() };
    assert!(!scene.apply("wm", &[remove]).unwrap_err().is_gone_surface());
}

#[test]
fn an_overlay_root_holds_nodes_fixed_to_its_output() {
    let mut scene = Scene::default();
    scene.add_overlay("DP-1");
    let hud = Command::Create {
        id: "hud".into(),
        kind: NodeKind::Rect,
        parent: overlay_id("DP-1"),
        props: PropsUpdate {
            size: Some([100.0, 20.0]),
            transform: Some(Transform::translate(10.0, 5.0, 0.0)),
            ..PropsUpdate::default()
        },
    };
    scene.apply("agent", &[hud]).unwrap();
    // The overlay is no part of the scene of the cameras.
    assert!(scene.draw_list().iter().all(|p| p.node.id != "hud"));
    let overlay = scene.overlay_draw_list("DP-1");
    let placed = overlay.iter().find(|p| p.node.id == "hud").unwrap();
    assert_eq!(placed.transform.translation_2d(), (10.0, 5.0));
    // Clients cannot create or remove an overlay root.
    let fake = Command::Create {
        id: overlay_id("HDMI-1"),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    assert!(scene.apply("agent", &[fake]).is_err());
    let remove = Command::Remove {
        id: overlay_id("DP-1"),
    };
    assert!(scene.apply("agent", &[remove]).is_err());
    // The reset empties the overlay, and keeps it.
    scene.reset([0.0, 0.0, 100.0, 100.0]);
    assert_eq!(
        scene.node(&overlay_id("DP-1")).unwrap().children,
        Vec::<String>::new()
    );
}

#[test]
fn a_client_gives_the_keyboard_to_its_own_node_until_a_surface_takes_it() {
    let mut scene = scene_with_surface();
    scene.apply("wm", &[attach("s1", ROOT)]).unwrap();
    scene.apply("xp", &[group("menu", ROOT)]).unwrap();
    let focus_node = |node: Option<&str>| Command::FocusNode {
        node: node.map(Into::into),
    };
    assert!(scene.apply("wm", &[focus_node(Some("menu"))]).is_err());
    assert!(scene.apply("wm", &[focus_node(Some("s1"))]).is_err());
    scene.apply("xp", &[focus_node(Some("menu"))]).unwrap();
    assert_eq!(scene.focused_node(), Some("menu"));
    assert_eq!(scene.focused_node_owner(), Some("xp"));
    // Only the owner gives it back.
    scene.apply("wm", &[focus_node(None)]).unwrap();
    assert_eq!(scene.focused_node(), Some("menu"));
    scene.apply("xp", &[focus_node(None)]).unwrap();
    assert_eq!(scene.focused_node(), None);
    // The focus of a surface and the removal of the node end it.
    scene.apply("xp", &[focus_node(Some("menu"))]).unwrap();
    let focus = Command::Focus {
        surface: Some("s1".into()),
    };
    scene.apply("wm", &[focus]).unwrap();
    assert_eq!(scene.focused_node(), None);
    scene.apply("xp", &[focus_node(Some("menu"))]).unwrap();
    scene
        .apply("xp", &[Command::Remove { id: "menu".into() }])
        .unwrap();
    assert_eq!(scene.focused_node(), None);
}
