//! Scene commands that place, dress and hide the windows of tiling.

use std::collections::BTreeMap;

use chora_scene::{
    Clip, Command, InputMode, NodeKind, PropsUpdate, ToplevelState, Transform, Uniform,
};
use serde_json::Value;

use crate::{
    NAME, config,
    dress::{self, Dress},
    wm::{Mark, Placement, States, Wm},
};

/// The window that a drag takes: the window of tiling that asked for a
/// move, from its title bar for example, else the window `under` the
/// pointer, which may be another window where windows overlap.
pub(super) fn taken(
    asked: Option<String>,
    ours: impl Fn(&str) -> bool,
    under: impl FnOnce() -> Option<String>,
) -> Option<String> {
    match asked {
        Some(id) => ours(&id).then_some(id),
        None => under(),
    }
}

/// The prefix of the id of the group that frames a window.
pub(super) const GROUP: &str = "tw-";

/// The id of the node that clips a window to its corners.
pub(super) fn clip_of(surface: &str) -> String {
    format!("tc-{surface}")
}

/// The id of the node that draws the border of a window.
pub(super) fn border_of(surface: &str) -> String {
    format!("tb-{surface}")
}

/// The nodes of a new window: the group that carries it, the border and
/// the clip that holds the surface.
pub(super) fn frame_nodes(
    surface: &str,
    border: Option<&str>,
    shadow: Option<&str>,
) -> Vec<Command> {
    let create = |id: String, kind, parent: String, props| Command::Create {
        id,
        kind,
        parent,
        props,
    };
    let hidden = PropsUpdate {
        size: Some([1.0, 1.0]),
        visible: Some(false),
        ..PropsUpdate::default()
    };
    let mut nodes = vec![create(
        group_of(surface),
        NodeKind::Group,
        NAME.into(),
        hidden.clone(),
    )];
    if let Some(shader) = shadow {
        nodes.extend(dress::shadow_nodes(surface, shader, &group_of(surface)));
    }
    if let Some(shader) = border {
        let props = PropsUpdate {
            shader: Some(shader.into()),
            input: Some(InputMode::Pass),
            ..hidden
        };
        nodes.push(create(
            border_of(surface),
            NodeKind::Shader,
            group_of(surface),
            props.clone(),
        ));
        let focus = PropsUpdate {
            opacity: Some(0.0),
            ..props
        };
        nodes.push(create(
            dress::focus_of(surface),
            NodeKind::Shader,
            group_of(surface),
            focus,
        ));
    }
    nodes.push(create(
        clip_of(surface),
        NodeKind::Group,
        group_of(surface),
        PropsUpdate {
            size: Some([1.0, 1.0]),
            ..PropsUpdate::default()
        },
    ));
    nodes
}

/// A shader of the plugin, written into the runtime directory of the user;
/// none when there is none, or it is no absolute path (the shared temporary
/// directory would let another user plant it first).
pub(super) fn runtime_shader(name: &str, text: &str) -> Option<String> {
    let runtime = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?);
    let path = runtime.join("chora-tiling").join(name);
    runtime.is_absolute().then_some(())?;
    if chora_kit::read_text(&path, chora_kit::MAX_TEXT)
        .ok()
        .as_deref()
        != Some(text)
    {
        chora_kit::write_file(&path, text.as_bytes()).ok()?;
    }
    Some(path.to_string_lossy().into_owned())
}

/// The id of the group of a window: it holds the surface, clips it to the
/// rounded corners and carries its position.
pub(super) fn group_of(surface: &str) -> String {
    format!("{GROUP}{surface}")
}

/// Whether an answered request takes the window out of tiling: the core
/// accepted a `move` to a parent that is not the tiling group. A rejected
/// transaction leaves the window managed (D31).
pub(super) fn leaves_tiling(applied: bool, action: &str, args: &Value) -> bool {
    applied
        && action == "move"
        && args
            .get("parent")
            .and_then(Value::as_str)
            .is_some_and(|parent| parent != NAME)
}

/// The answer to a request of another client for one of the windows
/// (D31): the commands that do it. `focus` goes through the model and
/// answers without commands. `None`: the action, for example `resize`,
/// stays unanswered; `kit::wm::book` answers `update` (D58).
pub(super) fn answers(wm: &mut Wm, node: &str, action: &str, args: &Value) -> Option<Vec<Command>> {
    match action {
        "focus" => {
            wm.focus(node);
            Some(Vec::new())
        }
        "close" => Some(vec![Command::Close {
            surface: node.into(),
        }]),
        "move" => Some(vec![Command::Move {
            id: node.into(),
            parent: args.get("parent").and_then(Value::as_str).map(|p| {
                if p == NAME {
                    group_of(node)
                } else {
                    p.to_owned()
                }
            }),
            index: args
                .get("index")
                .and_then(Value::as_u64)
                .and_then(|i| usize::try_from(i).ok()),
        }]),
        _ => None,
    }
}

/// The update that gives a new window a style.
pub(super) fn styled(style: &config::Style, surface: &str) -> Command {
    Command::Update {
        id: surface.into(),
        props: PropsUpdate {
            opacity: Some(style.opacity),
            effects: Some(style.effects.clone()),
            ..PropsUpdate::default()
        },
    }
}

/// The commands that bring a window from what was sent to its placement.
/// `colors` are the colours of the palette when the window has a border.
pub(super) fn changes(
    sent: Option<&Placement>,
    placement: &Placement,
    dress: &Dress,
) -> Vec<Command> {
    let id = &placement.id;
    let Some(at) = placement.at else {
        return hide(sent, id);
    };
    let mut commands = Vec::new();
    if sent.is_none_or(|s| s.size != placement.size || s.states != placement.states) {
        commands.push(configure(id, placement.size, placement.states));
    }
    if sent.is_none_or(|s| s.at != placement.at) {
        commands.extend(dress::place(sent, placement, at, dress));
    }
    let reshaped = sent.is_none_or(|s| s.frame != placement.frame);
    let rounded = sent.is_none_or(|s| s.radius != placement.radius);
    if reshaped || rounded {
        let mut props = PropsUpdate::default();
        if reshaped {
            props.size = Some(placement.frame.map(f64::from));
        }
        if rounded {
            props.clip = Some((placement.radius > 0).then(|| Clip {
                radius: f64::from(placement.radius),
            }));
        }
        commands.push(update(clip_of(id), props));
    }
    if let Some(colors) = dress.colors {
        let restyled = sent.is_none_or(|s| {
            s.frame != placement.frame
                || s.radius != placement.radius
                || s.border != placement.border
                || s.mark != placement.mark
        });
        if restyled {
            commands.extend(border_updates(placement, colors));
            commands.extend(dress::shadow_updates(placement, dress.shadows, colors[0]));
        }
        commands.extend(dress::fade(sent, placement, dress.animate, restyled));
    }
    commands
}

pub(super) fn update(id: String, props: PropsUpdate) -> Command {
    Command::Update { id, props }
}

/// The updates of the two border nodes of a window: the ring in the line
/// colour (urgent for a window that asked for attention) and the gradient
/// of the focus, which shows through its opacity.
pub(super) fn border_updates(placement: &Placement, colors: &[[f64; 3]; 4]) -> Vec<Command> {
    let line = match placement.mark {
        Mark::Urgent => colors[3],
        Mark::Line | Mark::Focus => colors[2],
    };
    vec![
        ring_update(border_of(&placement.id), placement, [line, line]),
        ring_update(
            dress::focus_of(&placement.id),
            placement,
            [colors[0], colors[1]],
        ),
    ]
}

/// The update of a border node: its size and place around the window, its
/// colours and whether it shows.
pub(super) fn ring_update(
    node: String,
    placement: &Placement,
    [from, to]: [[f64; 3]; 2],
) -> Command {
    let width = f64::from(placement.border);
    let color = |[r, g, b]: [f64; 3]| Uniform::Vector(vec![r, g, b, 1.0]);
    let [w, h] = placement.frame.map(f64::from);
    update(
        node,
        PropsUpdate {
            visible: Some(placement.border > 0),
            size: Some([w + 2.0 * width, h + 2.0 * width]),
            transform: Some(Transform::translate(-width, -width, 0.0)),
            uniforms: Some(BTreeMap::from([
                ("from".to_owned(), color(from)),
                ("to".to_owned(), color(to)),
                (
                    "radius".to_owned(),
                    Uniform::Float(f64::from(placement.radius) + width),
                ),
                ("width".to_owned(), Uniform::Float(width)),
                (
                    "logical".to_owned(),
                    Uniform::Vector(vec![w + 2.0 * width, h + 2.0 * width]),
                ),
            ])),
            ..PropsUpdate::default()
        },
    )
}

/// The update from the saved meta of a node to its new meta. A key that
/// went stays in the core; the restore skips it.
pub(super) fn meta_change(
    saved: Option<&BTreeMap<String, String>>,
    id: &str,
    meta: &BTreeMap<String, String>,
) -> Option<Command> {
    (saved != Some(meta)).then(|| Command::Update {
        id: id.into(),
        props: PropsUpdate {
            meta: Some(meta.clone()),
            ..PropsUpdate::default()
        },
    })
}

/// Hides a window that showed or is new to the core, and takes its focus
/// state away.
pub(super) fn hide(sent: Option<&Placement>, id: &str) -> Vec<Command> {
    if sent.is_some_and(|s| s.at.is_none()) {
        return Vec::new();
    }
    let mut commands = Vec::new();
    if let Some(sent) = sent.filter(|s| s.states.activated) {
        let states = States {
            activated: false,
            ..sent.states
        };
        commands.push(configure(id, sent.size, states));
    }
    commands.push(Command::Update {
        id: group_of(id),
        props: PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        },
    });
    commands
}

pub(super) fn configure(id: &str, size: Option<[i32; 2]>, states: States) -> Command {
    let states = [
        (states.tiled, ToplevelState::Tiled),
        (states.maximized, ToplevelState::Maximized),
        (states.fullscreen, ToplevelState::Fullscreen),
        (states.activated, ToplevelState::Activated),
    ];
    Command::Configure {
        surface: id.into(),
        size,
        states: states
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, s)| s)
            .collect(),
    }
}
