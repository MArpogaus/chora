//! Unit tests of the construct's side of the cube (`construct.rs`): the
//! open, check, retry, turn and close of the cube, its nodes, its outputs
//! and its edge, on a fake core that keeps the scene and records the
//! requests with their parameters.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use chora_kit::{
    geometry::Cube,
    modes::Act,
    testing::{FakeCore, Log, focus_changed, has},
};
use chora_scene::{Command, Event, InputMode, NodeKind, OutputInfo, PropsUpdate, ROOT, Transform};
use serde_json::{Value, json};

use super::tests::host_of;
use super::{CUBE, EDGE, EDGE_GLOW, EDGE_WIDTH, FACES, Health, face_id, fill_id, mirror_id};
use crate::{Config, Construct, MIRRORS, NAME, PANES, config, settings};

/// A manager that runs until the cube closes it.
const RUNS: &str = "sh -c 'sleep 30' #";
/// A manager that writes a line to its error output and exits at once.
const EXITS: &str = "sh -c 'echo the manager broke >&2; exit 3' #";

/// A construct on a fake core that answers with `answer`. The fake core
/// keeps the scene, and each request is recorded as its method and its
/// parameters, one line each. The plugin's groups of the panes and of the
/// mirrors exist, as the plugin makes them at its start, and the virtual
/// outputs `VIRTUAL-1` to `VIRTUAL-4` are owned by the plugin, as the
/// core makes an output of `output.add`.
fn on_core(
    tag: &str,
    answer: impl Fn(&str, &Value) -> Option<Value> + Send + Sync + 'static,
) -> (Construct, FakeCore, Log) {
    let dir = chora_kit::testing::TempDir::new(&format!("chora-construct-cube-{tag}")).keep();
    std::fs::create_dir_all(&dir).unwrap();
    let core = FakeCore::start();
    core.respond(answer);
    let lines = core.record(|method, params| Some(format!("{method} {params}")));
    let mut conn = core.connect(NAME);
    let group = |id: &str| Command::Create {
        id: id.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    conn.apply(vec![group(PANES), group(MIRRORS)]).unwrap();
    for n in 1..=FACES {
        core.model()
            .lock()
            .unwrap()
            .scene
            .set_output_owner(&format!("VIRTUAL-{n}"), Some(NAME));
    }
    drop(conn);
    let mut rt = core.runtime(NAME);
    rt.pin_accepts_injected(false);
    let settings =
        settings::File::<Config>::load(dir.join("chora-construct.yaml"), config::DEFAULT).unwrap();
    let shaders = |name: &str, _: &str| Ok(format!("{}/{name}", dir.display()));
    let mut construct = Construct::new(rt, settings, false, &shaders).unwrap();
    construct.active = true;
    (construct, core, lines)
}

/// The answers of a core that lists `clients` as connected and adds the
/// virtual outputs `VIRTUAL-1`, `VIRTUAL-2`, ... as `output.add` asks.
fn cube_core(clients: Value) -> impl Fn(&str, &Value) -> Option<Value> + Send + Sync + 'static {
    let added = AtomicUsize::new(0);
    move |method, _| match method {
        "output.add" => {
            let n = added.fetch_add(1, Ordering::SeqCst) + 1;
            Some(json!(virtual_output(&format!("VIRTUAL-{n}"))))
        }
        "clients.list" => Some(clients.clone()),
        _ => None,
    }
}

/// A virtual output of this plugin, as the core answers `output.add`.
fn virtual_output(name: &str) -> OutputInfo {
    OutputInfo {
        name: name.into(),
        position: [0, 0],
        size: [1280, 800],
        scale: 1.0,
        camera: Transform::default(),
        work_area: [0, 0, 1280, 800],
        default_work_area: [0; 4],
        is_virtual: Some(true),
        owner: Some(NAME.into()),
    }
}

/// How many recorded requests of the method `method` there are.
fn sees(lines: &Log, method: &str) -> usize {
    let prefix = format!("{method} ");
    (lines.lock().unwrap().iter())
        .filter(|line| line.starts_with(&prefix))
        .count()
}

/// The id of a command that names a node.
fn id_of(command: &Command) -> Option<&str> {
    match command {
        Command::Create { id, .. } | Command::Update { id, .. } | Command::Animate { id, .. } => {
            Some(id)
        }
        _ => None,
    }
}

/// Whether some command of `commands` names the node `id`.
fn names(commands: &[Command], id: &str) -> bool {
    commands.iter().any(|c| id_of(c) == Some(id))
}

/// The created node `id` of `nodes`: its kind, its parent and its properties.
fn created<'a>(nodes: &'a [Command], id: &str) -> Option<(&'a NodeKind, &'a str, &'a PropsUpdate)> {
    nodes.iter().find_map(|c| match c {
        Command::Create {
            id: found,
            kind,
            parent,
            props,
        } if found == id => Some((kind, parent.as_str(), props)),
        _ => None,
    })
}

/// The ids of the cube group, its edge and its glow, and the mirror, the
/// face and the fill of each of `faces` faces.
fn cube_nodes(faces: usize) -> Vec<String> {
    let mut ids = vec![CUBE.to_owned(), EDGE.to_owned(), EDGE_GLOW.to_owned()];
    for i in 0..faces {
        ids.extend([face_id(i), fill_id(i), mirror_id(i)]);
    }
    ids
}

/// Waits until the faces of the open cube stop starting: the manager has
/// exited or failed to connect, or the test fails.
fn settled(construct: &mut Construct) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        construct.check_cube().unwrap();
        let starting = construct
            .cube
            .as_ref()
            .is_some_and(|c| c.health.contains(&Health::Starting));
        if !starting {
            return;
        }
        assert!(Instant::now() < deadline, "the manager is still starting");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A cube of two faces opened on a manager that runs.
fn open_two(tag: &str) -> (Construct, FakeCore, Log) {
    let (mut construct, core, lines) = on_core(tag, cube_core(json!([])));
    construct.settings.value.cube_manager = RUNS.into();
    construct.open_cube(2).unwrap();
    (construct, core, lines)
}

/// A cube of two faces that stands back, so that the pointer lights an
/// edge of it.
fn standing_back(tag: &str) -> Construct {
    let (mut construct, _core, _lines) = on_core(tag, cube_core(json!([])));
    construct.cube = Some(host_of(2));
    let stood = construct.cube.as_mut().unwrap().view.act(Act::StepBack);
    assert!(stood, "the cube stands back: navigate mode");
    construct
}

/// Opening refuses a face count out of range before it adds an output.
#[test]
fn open_refuses_a_face_count_out_of_range() {
    let (mut construct, _core, lines) = on_core("open-faces", cube_core(json!([])));
    construct.settings.value.cube_manager = RUNS.into();
    for faces in [0, FACES + 1] {
        let err = construct.open_cube(faces).unwrap_err().to_string();
        assert!(
            err.contains(&format!("faces: {faces} is not 1 to {FACES}")),
            "{err}"
        );
    }
    assert!(construct.cube.is_none());
    assert_eq!(sees(&lines, "output.add"), 0, "no output is added");
}

/// Opening refuses while a cube is open, and while a client of its
/// manager runs already: no output is added then either.
#[test]
fn open_refuses_while_a_cube_or_its_manager_runs() {
    let (mut open, _core, lines) = on_core("open-twice", cube_core(json!([])));
    open.cube = Some(host_of(2));
    let err = open.open_cube(2).unwrap_err().to_string();
    assert!(err.contains("a cube is open"), "{err}");
    assert_eq!(sees(&lines, "output.add"), 0);

    let (mut running, _core, lines) = on_core("open-running", cube_core(json!([{"name": "sh"}])));
    running.settings.value.cube_manager = RUNS.into();
    let err = running.open_cube(2).unwrap_err().to_string();
    assert!(err.contains("a client sh runs already"), "{err}");
    assert!(running.cube.is_none());
    assert_eq!(sees(&lines, "output.add"), 0);
}

/// A cube opens with one virtual output and one face per output, and its
/// manager starts on those outputs; the faces wait for it.
#[test]
fn open_adds_the_outputs_and_starts_the_manager_on_them() {
    let (construct, _core, lines) = open_two("open-ok");
    let host = construct.cube.as_ref().expect("the cube is open");
    assert_eq!(host.outputs, ["VIRTUAL-1", "VIRTUAL-2"]);
    assert_eq!(
        host.client(),
        "sh",
        "the manager is the client of its program"
    );
    assert!(
        host.command
            .ends_with("--output VIRTUAL-1 --output VIRTUAL-2"),
        "{}",
        host.command
    );
    assert!(
        host.health.iter().all(|h| matches!(h, Health::Starting)),
        "the faces wait for their manager"
    );
    assert_eq!(sees(&lines, "output.add"), 2);
}

/// The nodes of an open cube are in the scene; closing the cube ends its
/// manager, removes its outputs and takes its nodes away.
#[test]
fn the_cube_nodes_are_in_the_scene_until_the_cube_closes() {
    let (mut construct, core, lines) = open_two("open-scene");
    let model = core.model();
    let ids = cube_nodes(2);
    assert!(
        ids.iter().all(|id| has(&model, id)),
        "the cube, its edge and its faces are in the scene"
    );

    construct.close_cube().unwrap();
    assert!(construct.cube.is_none());
    assert_eq!(sees(&lines, "output.remove"), 2, "each output is removed");
    assert!(!has(&model, CUBE), "the cube group goes");
    assert!(!has(&model, &face_id(0)), "the faces go");
}

/// When the core refuses the nodes of the cube, the cube does not open:
/// the outputs it added go again.
#[test]
fn open_undoes_the_outputs_when_the_core_refuses_the_nodes() {
    let (mut construct, core, lines) = on_core("open-refused", cube_core(json!([])));
    construct.settings.value.cube_manager = RUNS.into();
    core.refuse_next("scene.apply");
    assert!(construct.open_cube(2).is_err());
    assert!(construct.cube.is_none());
    assert_eq!(sees(&lines, "output.add"), 2);
    assert_eq!(sees(&lines, "output.remove"), 2, "both outputs go");
}

/// A manager that exits fails its faces with a reason, and the cube stays.
#[test]
fn a_manager_that_exits_fails_its_faces_and_the_cube_stays() {
    let (mut construct, _core, _lines) = on_core("check-exit", cube_core(json!([])));
    construct.settings.value.cube_manager = EXITS.into();
    construct.open_cube(2).unwrap();
    settled(&mut construct);

    let host = construct.cube.as_ref().expect("the cube stays open");
    assert!(
        matches!(&host.health[0], Health::Failed(reason) if reason.starts_with("sh ")),
        "the reason names the manager: {:?}",
        host.health
    );
    assert!(host.health.iter().all(|h| matches!(h, Health::Failed(_))));
    construct.close_cube().unwrap();
}

/// A retry starts the manager again for the failed faces, with the same
/// command.
#[test]
fn a_retry_starts_the_manager_of_the_failed_faces_again() {
    let (mut construct, _core, _lines) = on_core("retry", cube_core(json!([])));
    construct.settings.value.cube_manager = EXITS.into();
    construct.open_cube(2).unwrap();
    settled(&mut construct);
    let command = construct.cube.as_ref().unwrap().command.clone();

    construct.retry_cube().unwrap();
    let host = construct.cube.as_ref().expect("the cube stays open");
    assert!(
        host.health.iter().all(|h| matches!(h, Health::Starting)),
        "the retry starts the faces again"
    );
    assert_eq!(host.command, command, "the same command starts again");
    construct.close_cube().unwrap();
}

/// A manager that does not connect within the limit fails its faces; one
/// that has not yet had its time keeps them starting.
#[test]
fn a_manager_that_does_not_connect_in_time_fails_its_faces() {
    let (mut construct, _core, _lines) = open_two("check-late");
    construct.check_cube().unwrap();
    let host = construct.cube.as_ref().expect("the cube is open");
    assert!(
        host.health.iter().all(|h| matches!(h, Health::Starting)),
        "within its time the manager may still connect"
    );

    let late = crate::faces::LIMIT + Duration::from_secs(1);
    let since = Instant::now()
        .checked_sub(late)
        .expect("the clock has run that long");
    construct.cube.as_mut().unwrap().since = since;
    construct.check_cube().unwrap();
    let host = construct.cube.as_ref().expect("the cube stays open");
    assert!(
        host.health.iter().all(|h| matches!(h, Health::Failed(_))),
        "the late manager fails its faces: {:?}",
        host.health
    );
    construct.close_cube().unwrap();
}

/// A connected manager brings its faces up and stops the wait.
#[test]
fn a_connected_manager_brings_the_faces_up() {
    let (mut construct, _core, _lines) = open_two("client-up");
    assert!(
        construct.cube_deadline().is_some(),
        "the start is waited for"
    );
    construct.cube_client(&focus_changed(None)).unwrap();
    let host = construct.cube.as_ref().expect("the cube is open");
    assert!(host.health.iter().all(|h| matches!(h, Health::Up)));
    assert!(
        host.start_deadline(Instant::now()).is_none(),
        "nothing starts now"
    );
    assert!(
        construct.cube_deadline().is_none(),
        "the construct waits for nothing once the manager is up"
    );
    construct.close_cube().unwrap();
}

/// The disconnect of the manager closes the cube, and its outputs go.
#[test]
fn the_disconnect_of_the_manager_closes_the_cube() {
    let (mut construct, _core, lines) = open_two("client-gone");
    construct
        .cube_client(&Event::ClientDisconnected { name: "sh".into() })
        .unwrap();
    assert!(construct.cube.is_none(), "the manager went, the cube goes");
    assert_eq!(sees(&lines, "output.remove"), 2);
}

/// Without a cube there is no turn, no face update and no mirror update;
/// the steps, the retry and the close do nothing.
#[test]
fn without_a_cube_there_is_no_turn_and_no_face_update() {
    let (mut construct, _core, _lines) = on_core("no-cube", cube_core(json!([])));
    assert!(construct.turn_cube(1, None, false).is_err());
    assert_eq!(construct.cube_faces().len(), 0);
    let look = construct.current_look();
    assert_eq!(construct.cube_mirror_looks(&look).len(), 0);
    assert!(!construct.cube_turns());
    assert!(construct.cube_deadline().is_none());
    construct.step_cube().unwrap();
    construct.retry_cube().unwrap();
    construct.close_cube().unwrap();
}

/// A turn of zero milliseconds is taken, and the next step ends it on the
/// next face.
#[test]
fn a_turn_ends_on_its_face_when_the_cube_steps() {
    let (mut construct, _core, _lines) = open_two("turn-ends");
    assert!(construct.turn_cube(1, Some(0), false).unwrap());
    assert!(construct.cube_turns(), "the turn is running");
    construct.step_cube().unwrap();
    let host = construct.cube.as_ref().unwrap();
    assert!(host.turn.is_none(), "the turn ended");
    assert_eq!(host.front, 1, "the front is the next face");
    assert!(
        (host.angle - Cube::turn_of(1)).abs() < 1e-9,
        "{}",
        host.angle
    );
    assert!(!construct.cube_turns());
    construct.close_cube().unwrap();
}

/// While a turn runs, a turn is ignored; a queued one goes on from the
/// running turn, and only one waits behind it.
#[test]
fn a_turn_while_one_runs_is_ignored_or_queued_once() {
    let (mut construct, _core, _lines) = open_two("turn-queue");
    assert!(construct.turn_cube(1, Some(60_000), false).unwrap());
    assert!(
        !construct.turn_cube(1, None, false).unwrap(),
        "a running turn is ignored"
    );
    assert!(
        construct.turn_cube(1, None, true).unwrap(),
        "a queued turn is taken"
    );
    assert!(construct.cube.as_ref().unwrap().queued);
    assert!(
        !construct.turn_cube(1, None, true).unwrap(),
        "only one turn waits behind the running one"
    );
    construct.close_cube().unwrap();
}

/// The edge of the face under the pointer lights with its glow, and fades
/// when the pointer leaves the face.
#[test]
fn the_edge_of_the_face_under_the_pointer_lights_and_fades() {
    let mut construct = standing_back("edge-lit");
    assert!(
        !names(&construct.cube_faces(), EDGE),
        "no pointer, no lit edge"
    );

    assert!(construct.cube.as_mut().unwrap().set_hover(Some(0)));
    let lit = construct.cube_faces();
    assert!(names(&lit, EDGE), "the edge of face 0 lights");
    assert!(names(&lit, EDGE_GLOW), "the glow lights with it");
    assert!(
        lit.iter()
            .any(|c| matches!(c, Command::Update { id, props, .. }
            if id == EDGE
                && props.border.as_ref().and_then(|b| b.as_ref()).is_some_and(|b| b.width > 0.0))),
        "the lit edge has a border"
    );

    construct.cube.as_mut().unwrap().set_hover(None);
    assert!(names(&construct.cube_faces(), EDGE), "the edge fades out");
    assert!(
        !names(&construct.cube_faces(), EDGE),
        "and is gone once faded"
    );
}

/// A turning cube hides the lit edge rather than fading it.
#[test]
fn a_turning_cube_hides_the_lit_edge() {
    let mut construct = standing_back("edge-hide");
    construct.cube.as_mut().unwrap().set_hover(Some(0));
    construct.cube_faces();
    construct.cube.as_mut().unwrap().set_hover(None);
    assert!(construct.turn_cube(1, Some(60_000), false).unwrap());

    assert!(
        construct
            .cube_faces()
            .iter()
            .any(|c| matches!(c, Command::Update { id, props, .. }
            if id == EDGE && props.visible == Some(false))),
        "a turning cube hides the edge"
    );
}

/// The mirror image of each face takes the look of the panes.
#[test]
fn the_mirror_images_of_the_faces_follow_the_look() {
    let (mut construct, _core, _lines) = on_core("mirror", cube_core(json!([])));
    construct.cube = Some(host_of(2));
    let look = construct.current_look();
    let mirrors = construct.cube_mirror_looks(&look);
    assert_eq!(mirrors.len(), 2, "one mirror a face");
    assert!(names(&mirrors, &mirror_id(0)));
    assert!(names(&mirrors, &mirror_id(1)));
}

/// The faces of a cube are a mirror, a shader and a fill per output, all
/// hidden until the first frame; the fill passes the pointer on.
#[test]
fn the_face_nodes_are_hidden_and_the_fills_pass_the_pointer_on() {
    let (construct, _core, _lines) = on_core("nodes-face", cube_core(json!([])));
    let outputs = ["VIRTUAL-1".to_owned(), "VIRTUAL-2".to_owned()];
    let cube = Cube {
        size: [1280.0, 800.0],
        caps: false,
    };
    let nodes = construct.face_nodes(&outputs, &cube);
    assert_eq!(
        nodes.len(),
        3 * outputs.len(),
        "a mirror, a face and a fill each"
    );
    for i in 0..outputs.len() {
        assert_face_nodes(&nodes, i);
    }
}

/// The shader of face `i` is hidden in the cube, its fill passes the
/// pointer on, and its mirror is there.
fn assert_face_nodes(nodes: &[Command], i: usize) {
    let (kind, parent, props) = created(nodes, &face_id(i)).expect("the face node");
    assert!(matches!(kind, NodeKind::Shader));
    assert_eq!(parent, CUBE);
    assert_eq!(props.visible, Some(false), "hidden until the first frame");
    let (kind, _, props) = created(nodes, &fill_id(i)).expect("the fill node");
    assert!(matches!(kind, NodeKind::Rect));
    assert_eq!(
        props.input,
        Some(InputMode::Pass),
        "the fill passes the pointer"
    );
    assert!(created(nodes, &mirror_id(i)).is_some(), "the mirror node");
}

/// The edge and its glow are hidden shaders and rects that pass the pointer
/// on, and the edge has the width of the accent border.
#[test]
fn the_edge_nodes_are_hidden_and_pass_the_pointer_on() {
    let (construct, _core, _lines) = on_core("nodes-edge", cube_core(json!([])));
    let [glow, edge] = construct.edge_nodes();
    assert!(
        matches!(&glow, Command::Create { id, kind: NodeKind::Shader, parent, props }
            if id == EDGE_GLOW && parent == CUBE && props.visible == Some(false)
                && props.input == Some(InputMode::Pass)),
        "the glow is hidden and passes the pointer"
    );
    assert!(
        matches!(&edge, Command::Create { id, kind: NodeKind::Rect, props, .. }
            if id == EDGE && props.visible == Some(false)
                && props.border.as_ref().and_then(|b| b.as_ref()).is_some_and(|b| b.width == EDGE_WIDTH)),
        "the edge is a hidden border of the set width"
    );
}

/// The outputs of a cube are added in one round, each of the size of the
/// view; a refused one takes the others back, and a refused removal does
/// not stop the others.
#[test]
fn the_outputs_are_added_together_and_a_refusal_takes_back_the_others() {
    let (mut construct, core, lines) = on_core("outputs", cube_core(json!([])));
    let added = construct.add_outputs(2, [1280.0, 800.0]).unwrap();
    assert_eq!(added, ["VIRTUAL-1", "VIRTUAL-2"]);
    let specs = (lines.lock().unwrap().iter())
        .filter(|l| l.starts_with("output.add") && l.contains("1280x800@1"))
        .count();
    assert_eq!(specs, 2, "each output is 1280x800 at scale 1");

    let (mut refused, core2, lines2) = on_core("outputs-refused", cube_core(json!([])));
    core2.refuse_next("output.add");
    assert!(refused.add_outputs(2, [1280.0, 800.0]).is_err());
    assert_eq!(
        sees(&lines2, "output.remove"),
        1,
        "the output that was added goes again"
    );

    core.refuse_next("output.remove");
    construct.remove_outputs(&["A".to_owned(), "B".to_owned()]);
    assert_eq!(
        sees(&lines, "output.remove"),
        2,
        "a refusal does not stop the others"
    );
}

/// A cube of two faces on a fake core, and a placed pane `name` of
/// `size` at `at` in front of it.
fn with_panes(tag: &str, panes: &[(&str, [f64; 2], [f64; 2])]) -> Construct {
    let (mut construct, _core, _lines) = on_core(tag, cube_core(json!([])));
    construct.cube = Some(host_of(2));
    for (name, at, size) in panes {
        construct.panes.push(crate::Pane {
            name: (*name).into(),
            source: format!("s-{name}"),
            content: crate::Content::Window { command: None },
            at: *at,
            yaw: 0.0,
            size: *size,
            taken: true,
            attached: false,
            placed: true,
        });
    }
    construct
}

/// Steps the cube of `construct` back as the person does, and lets the
/// panes that gave way reach their places.
fn stepped_back(construct: &mut Construct) {
    let stepped = construct
        .cube
        .as_mut()
        .unwrap()
        .act(Act::StepBack, Instant::now());
    assert!(stepped, "the cube steps back");
    construct.make_way();
    construct.cube.as_mut().unwrap().stand = 1.0;
    construct.step_asides();
}

/// The box that a pane covers on the screen, and the box of the cube
/// that stands back.
fn boxes(construct: &mut Construct, name: &str) -> ([f64; 4], [f64; 4]) {
    let (lens, pose) = (construct.lens, construct.pose);
    let pane = construct.panes.iter().find(|p| p.name == name).unwrap();
    let pane = construct.screen_box_of(pane);
    let host = construct.cube.as_mut().unwrap();
    host.aim(&lens, &pose);
    (pane, host.screen_box(&lens, &pose).unwrap())
}

fn at_of(construct: &Construct, name: &str) -> ([f64; 2], f64) {
    let p = construct.panes.iter().find(|p| p.name == name).unwrap();
    (p.at, p.yaw)
}

const FRONT: [f64; 2] = [0.0, 700.0];
const PANE: [f64; 2] = [400.0, 300.0];

/// A pane in front of the cube that would cover it once it stands back
/// goes beside it, clear of it, on the side where it was.
#[test]
fn a_placed_pane_in_front_of_the_cube_goes_beside_it_on_its_side() {
    for (x, side) in [(-200.0, -1.0), (200.0, 1.0)] {
        let mut construct = with_panes("aside-side", &[("p", [x, FRONT[1]], PANE)]);
        let (before, _) = boxes(&mut construct, "p");
        construct.cube.as_mut().unwrap().stand = 1.0;
        let (_, cube) = boxes(&mut construct, "p");
        construct.cube.as_mut().unwrap().stand = 0.0;
        assert!(crate::frame::overlaps(cube, before), "it covers the cube");
        stepped_back(&mut construct);
        let (pane, cube) = boxes(&mut construct, "p");
        assert!(
            !crate::frame::overlaps(cube, pane),
            "{pane:?} beside {cube:?}"
        );
        let centre = f64::midpoint(pane[0], pane[2]) - f64::midpoint(cube[0], cube[2]);
        assert_eq!(centre.signum(), side, "the side of x {x}");
    }
}

/// A pane behind the cube and one that clears it stay where they are; a
/// pane that nobody placed in front of it goes beside it as a placed one
/// does (co-54).
#[test]
fn a_pane_that_does_not_cover_the_cube_stays() {
    let mut construct = with_panes(
        "aside-stays",
        &[
            ("loose", [0.0, 700.0], PANE),
            ("behind", [0.0, 5000.0], PANE),
            ("clear", [-2500.0, 3000.0], PANE),
        ],
    );
    construct.panes[0].placed = false;
    stepped_back(&mut construct);
    let gave_way: Vec<&String> = construct.asides.keys().collect();
    assert_eq!(gave_way, ["loose"], "only the pane in front gave way");
    for (name, at) in [("behind", [0.0, 5000.0]), ("clear", [-2500.0, 3000.0])] {
        assert_eq!(at_of(&construct, name), (at, 0.0), "{name}");
    }
}

/// Two panes on one side take places that overlap neither each other
/// nor the cube (rv-1): the one nearer the cube takes the inner place.
#[test]
fn two_panes_on_one_side_go_beside_the_cube_without_overlap() {
    let mut construct = with_panes(
        "aside-two",
        &[
            ("far", [-300.0, 700.0], PANE),
            ("near", [-100.0, 700.0], PANE),
        ],
    );
    stepped_back(&mut construct);
    assert_eq!(construct.asides.len(), 2);
    let ((far, cube), (near, _)) = (boxes(&mut construct, "far"), boxes(&mut construct, "near"));
    assert!(!crate::frame::overlaps(far, near), "{far:?} and {near:?}");
    assert!(!crate::frame::overlaps(cube, far) && !crate::frame::overlaps(cube, near));
    assert!(
        near[2] > far[2],
        "the pane nearer the cube is the inner one"
    );
}

/// The panes on both sides do not take the room of each other.
#[test]
fn panes_on_both_sides_go_to_their_own_sides() {
    let mut construct = with_panes(
        "aside-both",
        &[("l", [-200.0, 700.0], PANE), ("r", [200.0, 700.0], PANE)],
    );
    stepped_back(&mut construct);
    let ((l, cube), (r, _)) = (boxes(&mut construct, "l"), boxes(&mut construct, "r"));
    assert!(l[2] <= cube[0] && r[0] >= cube[2], "{l:?} {cube:?} {r:?}");
}

/// A pane that gave way is home again, at the same place and the same
/// turn, when the cube has stepped in or is gone, and no longer aside.
#[test]
fn a_pane_that_gave_way_goes_home_exactly_when_the_cube_steps_in_or_closes() {
    for closes in [false, true] {
        let mut construct = with_panes("aside-home", &[("p", [-200.0, 700.0], PANE)]);
        construct.panes[0].yaw = 0.25;
        let home = at_of(&construct, "p");
        stepped_back(&mut construct);
        assert_ne!(at_of(&construct, "p"), home, "it gave way");
        assert_eq!(construct.asides.len(), 1);
        if closes {
            construct.cube = None;
        } else {
            let host = construct.cube.as_mut().unwrap();
            host.stand = 0.0;
            host.step = None;
        }
        construct.step_asides();
        assert_eq!(at_of(&construct, "p"), home, "closes: {closes}");
        assert!(construct.asides.is_empty(), "closes: {closes}");
    }
}

/// A pane that is still on its way (the cube steps in) is not home yet.
#[test]
fn a_pane_is_aside_until_the_step_in_has_ended() {
    let mut construct = with_panes("aside-glide", &[("p", [-200.0, 700.0], PANE)]);
    let home = at_of(&construct, "p");
    stepped_back(&mut construct);
    let beside = at_of(&construct, "p");
    construct.cube.as_mut().unwrap().stand = 0.5;
    construct.step_asides();
    let halfway = at_of(&construct, "p");
    assert!(halfway.0[0] > beside.0[0].min(home.0[0]) && halfway.0[0] < beside.0[0].max(home.0[0]));
    assert_eq!(construct.asides.len(), 1, "still aside");
}

/// A pane that the person moves while the cube stands back stays where
/// it was put: the move takes it out of the asides (panes.rs).
#[test]
fn a_pane_that_the_person_moves_while_aside_stays_there() {
    let mut construct = with_panes("aside-moved", &[("p", [-200.0, 700.0], PANE)]);
    stepped_back(&mut construct);
    construct
        .move_pane_facing(0, [900.0, 1500.0], [0.0, 0.0])
        .unwrap();
    assert!(construct.asides.is_empty(), "the move ends the aside");
    let put = at_of(&construct, "p");
    construct.cube.as_mut().unwrap().stand = 0.0;
    construct.step_asides();
    assert_eq!(at_of(&construct, "p"), put, "and the step in leaves it");
}

/// The place beside the cube is inside the leash (rv-4).
#[test]
fn the_place_beside_the_cube_is_inside_the_leash() {
    let mut construct = with_panes("aside-leash", &[("p", [-200.0, 700.0], PANE)]);
    construct.radius = Some(1000.0);
    stepped_back(&mut construct);
    let (at, _) = at_of(&construct, "p");
    assert!(
        at[0].hypot(at[1]) <= 1000.0 + 1e-6,
        "{at:?} is outside 1000"
    );
}

/// A pane that is wider than the room beside the cube shrinks its offset
/// to stay on the screen (rv-4).
#[test]
fn a_wide_pane_beside_the_cube_stays_on_the_screen() {
    let mut construct = with_panes("aside-wide", &[("p", [-200.0, 700.0], [1400.0, 600.0])]);
    stepped_back(&mut construct);
    let (pane, _) = boxes(&mut construct, "p");
    let width = construct.lens.w;
    assert!(pane[0] >= 0.0 && pane[2] <= width, "{pane:?} on {width} px");
}

/// A pane that has no room left on screen on its own side goes to the
/// other side, and does not take the room of a pane there (rv-1, rv-4).
#[test]
fn a_pane_with_no_room_on_its_side_goes_to_the_other_side() {
    let mut construct = with_panes(
        "aside-other",
        &[
            ("wide", [-100.0, 700.0], [1280.0, 800.0]),
            ("slim", [-300.0, 700.0], [600.0, 600.0]),
        ],
    );
    stepped_back(&mut construct);
    let ((wide, cube), (slim, _)) = (boxes(&mut construct, "wide"), boxes(&mut construct, "slim"));
    let width = construct.lens.w;
    assert!(!crate::frame::overlaps(wide, slim), "{wide:?} and {slim:?}");
    for pane in [wide, slim] {
        assert!(pane[0] >= 0.0 && pane[2] <= width, "{pane:?} on {width} px");
        assert!(
            !crate::frame::overlaps(cube, pane),
            "{pane:?} beside {cube:?}"
        );
    }
    assert!(
        slim[0] > cube[2],
        "the slim pane is right of the cube: {slim:?}"
    );
}

/// co-pane r1: the key sheet takes the keyboard from the window of a
/// face, so that Escape reaches the sheet and not the program (an XP
/// Start menu took it), and gives it back when the sheet closes.
#[test]
fn the_key_sheet_takes_the_keyboard_from_a_face_and_closes_on_escape() {
    let (mut construct, core, _lines) = on_core("help-keyboard", cube_core(json!([])));
    construct.cube = Some(host_of(2));
    let last = || core.model().lock().unwrap().focuses.last().cloned();
    construct.toggle_help().unwrap();
    assert!(construct.help, "the sheet is open");
    assert_eq!(last(), Some(None), "no surface keeps the keyboard");
    assert!(construct.walk_keys, "the look holds the keys");
    let escape = Event::NodeKey {
        node: crate::LOOK.into(),
        key: "Escape".into(),
        state: crate::PressState::Pressed,
        modifiers: Vec::new(),
        text: None,
        code: 1,
        injected: false,
    };
    chora_kit::runtime::Plugin::event(&mut construct, escape).unwrap();
    assert!(!construct.help, "Escape closes the sheet first");
    assert!(construct.cube.is_some(), "and nothing else");
}

/// co-pane r2 (rv-2): in the walk, a window of a pane that a press gave the
/// keyboard keeps it while the key sheet shows, so Escape and the typed
/// search went to the program; the sheet takes it as it does in a flown pane.
#[test]
fn the_key_sheet_takes_the_keyboard_from_a_pressed_pane_window() {
    let (mut construct, core, _lines) = on_core("help-pressed", |method, params| {
        let focused = params["filter"]["focused"].as_bool() == Some(true);
        match method {
            "scene.query" if focused => Some(json!([{"id": "s1"}])),
            _ => cube_core(json!([]))(method, params),
        }
    });
    construct.panes.push(crate::Pane {
        name: "probe".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [0.0, 1500.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: true,
        placed: false,
    });
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s1"))).unwrap();
    assert!(!construct.in_pane(), "the camera is in no pane");
    assert!(
        construct.keys_with_pane(),
        "the pressed window has the keys"
    );
    construct.toggle_help().unwrap();
    assert!(construct.help, "the sheet is open");
    assert_eq!(
        core.model().lock().unwrap().focuses.last().cloned(),
        Some(None),
        "the sheet holds the keyboard, not the program"
    );
    assert!(construct.walk_keys, "the look holds the keys");
}

/// co-pane r2 (rv-3): the key sheet gives the keyboard back to the window of
/// a face only while that face is at the front, as a step in does; after a
/// turn it stays with the walk, so no window that no one sees gets it.
#[test]
fn the_key_sheet_gives_the_keyboard_back_only_to_the_front_face() {
    let activated = |lines: &Log| {
        (lines.lock().unwrap().iter())
            .filter(|line| line.starts_with("methods.call ") && line.contains("\"w1\""))
            .count()
    };
    let answer = |method: &str, params: &Value| {
        let focused = params["filter"]["focused"].as_bool() == Some(true);
        match method {
            "scene.query" if focused => Some(json!([{"id": "w1"}])),
            _ => cube_core(json!([]))(method, params),
        }
    };
    let (mut still, _core, lines) = on_core("help-front", answer);
    still.cube = Some(host_of(2));
    still.toggle_help().unwrap();
    still.toggle_help().unwrap();
    assert_eq!(activated(&lines), 1, "the front face gets its window back");
    let (mut turned, _core, lines) = on_core("help-turned", answer);
    turned.cube = Some(host_of(2));
    turned.toggle_help().unwrap();
    turned.cube.as_mut().expect("the cube").front = 1;
    turned.toggle_help().unwrap();
    assert!(!turned.help, "the sheet is closed");
    assert_eq!(activated(&lines), 0, "another face is at the front now");
}

/// co-pane r2 (rv-6, by design): a press on the window of a pane gives that
/// window the keyboard and starts no look, so the first drag of a session
/// that begins on a shown window does not look (F5).
#[test]
fn a_press_on_a_pane_window_starts_no_look() {
    let (mut construct, core, _lines) = on_core("press-window", cube_core(json!([])));
    construct.active = true;
    construct.panes.push(crate::Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [0.0, 1500.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: true,
        placed: false,
    });
    let press = Event::PointerPressed {
        node: "s1".into(),
        button: 272,
        position: [640.0, 400.0],
        local: [320.0, 200.0],
        modifiers: Vec::new(),
        injected: false,
    };
    chora_kit::runtime::Plugin::event(&mut construct, press).unwrap();
    assert!(construct.drag.is_none(), "no look starts on the window");
    assert_eq!(
        core.model().lock().unwrap().focuses.last().cloned(),
        Some(Some("s1".into())),
        "the window has the keyboard"
    );
}

/// co-pane r2 (rv-8): the toggle of the key sheet regrabs, because the sheet
/// moves the keyboard from a pressed pane window to the walk and back, and
/// the chords held for that window go and come with it.
#[test]
fn the_key_sheet_lets_go_of_the_chords_of_a_pane_window_and_takes_them_back() {
    let (mut construct, _core, _lines) = on_core("help-regrab", |method, params| {
        let focused = params["filter"]["focused"].as_bool() == Some(true);
        match method {
            "scene.query" if focused => Some(json!([{"id": "s1"}])),
            _ => cube_core(json!([]))(method, params),
        }
    });
    construct.panes.push(crate::Pane {
        name: "probe".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [0.0, 1500.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: true,
        placed: false,
    });
    chora_kit::runtime::Plugin::event(&mut construct, focus_changed(Some("s1"))).unwrap();
    assert!(
        construct.rt.action("Super+q").is_some(),
        "held for the window"
    );
    construct.toggle_help().unwrap();
    assert!(
        construct.rt.action("Super+q").is_none(),
        "the sheet holds the keyboard, the window gets no chord"
    );
    construct.toggle_help().unwrap();
    assert!(
        construct.rt.action("Super+q").is_some(),
        "the window has the keyboard and its chords again"
    );
}

/// co-pane r1: the Close of the key sheet is a button.
#[test]
fn a_press_on_the_close_of_the_key_sheet_closes_it() {
    let (mut construct, _core, _lines) = on_core("help-close", cube_core(json!([])));
    construct.toggle_help().unwrap();
    assert!(construct.help);
    let press = Event::PointerPressed {
        node: "construct-help-body-foot-0-hit".into(),
        button: 272,
        position: [0.0; 2],
        local: [0.0; 2],
        modifiers: Vec::new(),
        injected: false,
    };
    chora_kit::runtime::Plugin::event(&mut construct, press).unwrap();
    assert!(!construct.help, "the Close button closes the sheet");
}

/// co-71: a press outside the key sheet, on its scrim, closes it, as Escape
/// and the Close do, and one on the sheet itself does not.
#[test]
fn a_press_outside_the_key_sheet_closes_it() {
    let (mut construct, _core, _lines) = on_core("help-outside", cube_core(json!([])));
    construct.toggle_help().unwrap();
    assert!(construct.help);
    let press = |node: &str| Event::PointerPressed {
        node: node.into(),
        button: 272,
        position: [0.0; 2],
        local: [0.0; 2],
        modifiers: Vec::new(),
        injected: false,
    };
    chora_kit::runtime::Plugin::event(&mut construct, press("construct-help-body-panel")).unwrap();
    assert!(construct.help, "a press on the sheet keeps it open");
    chora_kit::runtime::Plugin::event(&mut construct, press("construct-help-scrim")).unwrap();
    assert!(!construct.help, "a press outside the sheet closes it");
}

/// co-pane r1: a window of a pane that the cube hides does not get the
/// keyboard back when the key sheet closes: the focus stays a shown window.
#[test]
fn the_key_sheet_does_not_focus_a_pane_that_is_out_of_sight_on_close() {
    let (mut construct, core, _lines) = on_core("help-hidden", cube_core(json!([])));
    construct.panes.push(crate::Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [0.0, -1500.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: true,
        placed: false,
    });
    construct.help = true;
    construct.help_back = Some("s1".into());
    let before = core.model().lock().unwrap().focuses.len();
    construct.toggle_help().unwrap();
    assert!(!construct.help);
    let focuses = core.model().lock().unwrap().focuses.clone();
    assert_eq!(focuses.len(), before, "{focuses:?}");
}

/// co-46 r1: a cube that opens over the window that has the keyboard, so
/// that the window shows no more (the pane is behind the front face), lets
/// the keyboard go at once: the focus is a window that shows, never one no
/// one sees, so a key stays out of it until a face takes the keyboard. The
/// state says the same: no window that `shown: false` is the holder.
#[test]
fn a_cube_that_covers_the_focused_window_takes_the_keyboard_from_it() {
    let (mut construct, core, _lines) = on_core("cover-keys", cube_core(json!([])));
    construct.settings.value.cube_manager = RUNS.into();
    construct.panes.push(crate::Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [0.0, 1500.0],
        yaw: 0.0,
        size: [640.0, 400.0],
        taken: false,
        attached: true,
        placed: false,
    });
    construct.core_focus = Some("s1".into());
    construct.walk_keys = false;
    construct.open_cube(1).unwrap();
    let state = chora_kit::runtime::Plugin::state(&construct).unwrap();
    let window = state["windows"][0].clone();
    assert_eq!(window["shown"], json!(false), "the cube covers it: {state}");
    assert_ne!(
        state["focus"]["surface"],
        json!("s1"),
        "a window that shows no more holds no keys: {state}"
    );
    let last = core.model().lock().unwrap().focuses.last().cloned();
    assert_eq!(last, Some(None), "no surface keeps the keyboard");
}

/// The cube that the person opens, by `Super+C`, the row `A cube of
/// desktops` of the start card or the hint, stands back: the person sees
/// a cube of desktops and steps into a face, not a plain desktop with no
/// sign of the cube (co-52).
#[test]
fn the_cube_the_person_opens_stands_back() {
    let (mut construct, _core, _lines) = on_core("key-stands", cube_core(json!([])));
    construct.settings.value.cube_manager = RUNS.into();
    construct.cube_key();
    let host = construct.cube.as_ref().expect("the key opens a cube");
    assert_eq!(
        host.view.mode(),
        chora_kit::modes::Mode::Navigate,
        "the cube stands back"
    );
}

/// The window of a pane that the person did not place, ahead of the
/// camera as a terminal opens, shows beside the cube that the person
/// opens over it, and is not lost behind a face (co-54).
#[test]
fn a_window_ahead_shows_beside_the_cube_the_person_opens() {
    let (mut construct, _core, _lines) = on_core("key-aside", cube_core(json!([])));
    construct.settings.value.cube_manager = RUNS.into();
    let [fx, fz] = construct.pose.forward();
    let ahead = construct.lens.focal + construct.lens.w * 0.8;
    construct.panes.push(crate::Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [fx * ahead, fz * ahead],
        yaw: 0.0,
        size: [construct.lens.w, construct.lens.h],
        taken: false,
        attached: true,
        placed: false,
    });
    construct.cube_key();
    let host = construct.cube.as_mut().expect("the key opens a cube");
    if host.step.take().is_some() {
        host.stand = 1.0;
    }
    construct.step_asides();
    let pane = &construct.panes[0];
    assert!(
        construct.in_sight(pane) && !construct.covers_cube(pane),
        "the window shows: {:?}, {}",
        pane.at,
        construct.state_value()
    );
    let (pane, cube) = boxes(&mut construct, "foot");
    assert!(
        !crate::frame::overlaps(cube, pane),
        "{pane:?} beside {cube:?}"
    );
}

/// Stepped into a face, after the hints of the step went, the hint bar
/// still names the way out: `Step back` with its key, beside `All keys`;
/// a person who stepped in sees how to step out again (co-61).
#[test]
fn a_face_stepped_into_names_its_way_out_after_the_hints_went() {
    let (mut construct, core, _lines) = on_core("rest-way-out", cube_core(json!([])));
    core.model().lock().unwrap().scene.add_overlay("test");
    construct.output = Some(chora_kit::testing::output("test"));
    construct.metrics.keep_estimates();
    let group = Command::Create {
        id: NAME.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    construct.rt.conn.apply(vec![group]).unwrap();
    construct.cube = Some(host_of(2));
    construct.hints.started = true;
    construct.hints.engaged = true;
    construct.hints.until = None;
    construct.draw_hints().unwrap();
    let model = core.model();
    assert!(has(&model, "construct-hint-keys"), "the resting chip shows");
    assert!(
        has(&model, "construct-hint-step-back"),
        "the hint bar names the way out of the face"
    );
}

/// A click on a pane beside the cube that stands back flies into it, as
/// Tab and Return do, and its window takes the keys; the person who sees
/// the terminal beside the cube gets to it (co-62).
#[test]
fn a_click_on_a_pane_beside_the_cube_flies_into_it() {
    let (mut construct, core, _lines) = on_core("click-aside", cube_core(json!([])));
    construct.cube = Some(host_of(2));
    construct.panes.push(crate::Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [-200.0, 700.0],
        yaw: 0.0,
        size: PANE,
        taken: false,
        attached: true,
        placed: false,
    });
    stepped_back(&mut construct);
    let (pane, cube) = boxes(&mut construct, "foot");
    assert!(
        !crate::frame::overlaps(cube, pane),
        "{pane:?} beside {cube:?}"
    );
    let at = [
        f64::midpoint(pane[0], pane[2]),
        f64::midpoint(pane[1], pane[3]),
    ];
    let press = Event::PointerPressed {
        node: crate::LOOK.into(),
        button: 272,
        position: at,
        local: at,
        modifiers: Vec::new(),
        injected: false,
    };
    let release = Event::PointerReleased {
        node: crate::LOOK.into(),
        button: 272,
        position: at,
        local: at,
        modifiers: Vec::new(),
        injected: false,
        time: 0,
    };
    chora_kit::runtime::Plugin::event(&mut construct, press).unwrap();
    chora_kit::runtime::Plugin::event(&mut construct, release).unwrap();
    assert!(construct.in_pane(), "the camera flies into the pane");
    assert_eq!(
        core.model().lock().unwrap().focuses.last().cloned(),
        Some(Some("s1".into())),
        "the window of the pane takes the keys"
    );
}

/// A cube stood back with the hint bar on the screen: the overlay, the
/// output and the group of the construct, as the hints need them.
fn with_hint_bar(name: &str) -> (Construct, FakeCore, Log) {
    let (mut construct, core, lines) = on_core(name, cube_core(json!([])));
    core.model().lock().unwrap().scene.add_overlay("test");
    construct.output = Some(chora_kit::testing::output("test"));
    construct.metrics.keep_estimates();
    let group = Command::Create {
        id: NAME.into(),
        kind: NodeKind::Group,
        parent: ROOT.into(),
        props: PropsUpdate::default(),
    };
    construct.rt.conn.apply(vec![group]).unwrap();
    construct.cube = Some(host_of(2));
    construct.panes.push(crate::Pane {
        name: "foot".into(),
        source: "s1".into(),
        content: crate::Content::Window { command: None },
        at: [-200.0, 700.0],
        yaw: 0.0,
        size: PANE,
        taken: false,
        attached: true,
        placed: false,
    });
    stepped_back(&mut construct);
    construct.cube.as_mut().unwrap().step = None;
    construct.hints.started = true;
    construct.open_modes().unwrap();
    (construct, core, lines)
}

/// Stood back with a pane beside the cube, the hint bar names the click
/// on it that flies in (co-64).
#[test]
fn a_pane_beside_the_cube_names_its_click() {
    let (mut construct, core, _lines) = with_hint_bar("pane-acts");
    construct.draw_hints().unwrap();
    let model = core.model();
    assert!(has(&model, "construct-hint-keys"), "the cube's hints show");
    assert!(
        has(&model, "construct-hint-click-pane"),
        "the hint bar names the click on the pane"
    );
}

/// The hint bar names `Super Alt 4 Pull out` only where the key acts: it
/// gives the window with the keyboard to the construct, and the window of
/// a pane is the construct's already, so with the keyboard in the pane
/// beside the cube, or in no window at all, the key does nothing and
/// the hint is not told (co-68).
#[test]
fn the_pull_key_is_named_only_where_it_acts() {
    let (mut construct, core, _lines) = with_hint_bar("pull-named");
    construct.draw_hints().unwrap();
    assert!(
        !has(&core.model(), "construct-hint-pull-out"),
        "no window to pull: the key is not named"
    );
    construct.core_focus = Some("s1".into());
    construct.draw_hints().unwrap();
    assert!(
        !has(&core.model(), "construct-hint-pull-out"),
        "the keyboard is in the pane, the construct's own window: the key is not named"
    );
    construct.core_focus = Some("s7".into());
    construct.draw_hints().unwrap();
    let model = core.model();
    assert!(
        has(&model, "construct-hint-click-pane"),
        "the click on the pane stays named"
    );
    assert!(
        has(&model, "construct-hint-pull-out"),
        "the keyboard is in a window of a face: the key pulls it out, so it is named"
    );
    construct.core_focus = None;
    construct
        .cube
        .as_mut()
        .unwrap()
        .note_focus(Some("s7".into()));
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-pull-out"),
        "the window of a face noted at the step back is pulled out: named"
    );
}

/// co-70: stood back from a face window with the keyboard and no pane
/// beside the cube (row, Return, Super+t, Alt+Escape), the key pulls the
/// window out, so the hint bar names `Super Alt 4 Pull out`; with no
/// window to pull it stays silent (co-68 holds). The same predicate
/// decides both: the hint is told where the key acts.
#[test]
fn the_pull_key_is_named_in_a_face_window_stood_back_with_no_pane_beside() {
    let (mut construct, core, _lines) = with_hint_bar("pull-no-pane");
    construct.panes.clear();
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-keys"),
        "the cube's hints show"
    );
    assert!(
        !has(&core.model(), "construct-hint-click-pane"),
        "no pane beside the cube: no click on it is named"
    );
    assert!(
        !has(&core.model(), "construct-hint-pull-out"),
        "no window to pull: the key is not named"
    );
    construct.core_focus = Some("s7".into());
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-pull-out"),
        "the keyboard is in a window of a face: the key pulls it out, so it is named"
    );
    construct.core_focus = None;
    construct
        .cube
        .as_mut()
        .unwrap()
        .note_focus(Some("s7".into()));
    construct.draw_hints().unwrap();
    assert!(
        has(&core.model(), "construct-hint-pull-out"),
        "the window of a face noted at the step back is pulled out: named"
    );
}

/// Flown into a pane from the cube stood back, as a program started from
/// Programs or the click on a pane does, the hint bar names the way out,
/// `Step back` with its key, and no hint of the cube, whose keys do not
/// hold in a pane; the person is never stranded (co-65).
#[test]
fn a_pane_flown_into_from_the_cube_names_its_way_out() {
    let (mut construct, core, _lines) = with_hint_bar("pane-way-out");
    construct.fly_in(0).unwrap();
    assert!(construct.in_pane(), "the camera flies into the pane");
    for until in [true, false] {
        construct.draw_hints().unwrap();
        if !until {
            construct.hints.until = None;
            construct.draw_hints().unwrap();
        }
        let model = core.model();
        assert!(
            has(&model, "construct-hint-step-back"),
            "the hint bar names the way out (timed {until})"
        );
        assert!(
            !has(&model, "construct-hints-construct-mode-step-in-keys-0"),
            "no hint of the cube shows in the pane (timed {until})"
        );
    }
}

/// A program that starts while the cube stands back opens where the
/// person sees it: beside the cube, clear of it, not inside it, and it
/// stays there when the cube steps in (co-67).
#[test]
fn a_window_that_opens_while_the_cube_stands_back_shows_beside_it() {
    let mut construct = with_panes("aside-fresh", &[]);
    stepped_back(&mut construct);
    construct
        .adopt("s9", "foot", None, crate::Hold::Taken, true)
        .unwrap();
    let (pane, cube) = boxes(&mut construct, "foot");
    let screen = [0.0, 0.0, construct.lens.w, construct.lens.h];
    assert!(
        crate::frame::overlaps(screen, pane),
        "the new pane shows on the screen: {pane:?}"
    );
    assert!(
        !crate::frame::overlaps(cube, pane),
        "the new pane is clear of the cube: {pane:?} {cube:?}"
    );
    assert_eq!(construct.beside_cube(), Some(0), "it is the pane beside");
    let put = at_of(&construct, "foot");
    let host = construct.cube.as_mut().unwrap();
    host.stand = 0.0;
    host.step = None;
    construct.step_asides();
    assert_eq!(at_of(&construct, "foot"), put, "and the step in leaves it");
}

#[cfg(test)]
mod close_tests;
#[cfg(test)]
mod face_start_tests;
#[cfg(test)]
mod reach_tests;
