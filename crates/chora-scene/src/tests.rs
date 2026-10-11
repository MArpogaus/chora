//! Tests of the scene rules.

use crate::{
    Command, Effect, NodeKind, PropsUpdate, ROOT, Scene, SceneError, Selector, SurfaceInfo,
    Transform,
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
    assert_eq!(scene.disconnect("wm"), vec!["s1".to_owned()]);
    assert!(scene.node("tiles").is_none());
    let s1 = scene.node("s1").unwrap();
    assert_eq!((s1.parent.as_deref(), s1.owner.as_deref()), (None, None));
    assert!(scene.node(ROOT).unwrap().children.is_empty());
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
    let effects = scene.reset([1000.0, 500.0]);
    assert!(scene.node("tiles").is_none());
    assert_eq!(scene.node(ROOT).unwrap().children, vec!["s1", "s2"]);
    assert_eq!(
        scene.node("s2").unwrap().props.transform,
        Transform::translate(500.0, 0.0, 0.0)
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
            size: None,
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
    let commands: Vec<Command> = (0..100_000)
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
    assert_eq!(scene.draw_list().len(), 100_000);
}

#[test]
fn disconnect_lists_each_surface_once() {
    let mut scene = scene_with_surface();
    scene
        .apply("wm", &[group("a", ROOT), attach("s1", "a")])
        .unwrap();
    assert_eq!(scene.disconnect("wm"), vec!["s1".to_owned()]);
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
fn size_and_color_belong_to_rects() {
    let mut scene = Scene::default();
    let sized = PropsUpdate {
        size: Some([10.0, 10.0]),
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
