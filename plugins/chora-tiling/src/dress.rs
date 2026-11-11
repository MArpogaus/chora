//! What a window wears besides its surface: the shadows under it, the
//! cross-fade of the focus mark and the motion of its group. The group of
//! a window holds, from the bottom: `ts` the shadow of an unfocused
//! window, `tf` the deeper shadow of the focused one, `tb` the border in
//! the line colour (or urgent), `tg` the gradient of the focus, and `tc`
//! the clip with the surface. `tf` and `tg` fade in and out with the mark.

use std::collections::BTreeMap;

use chora_scene::{Command, Easing, NodeKind, PropsUpdate, Transform, Uniform};

use crate::wm::{Mark, Placement};

/// The text of the shader of the shadows.
pub const SHADOW_SHADER: &str = include_str!("../shadow.frag");

/// How long a window takes to open, to move and to change its mark, and a
/// panel to fade in, in milliseconds.
pub const OPEN: f64 = 180.0;
pub const MOVE: f64 = 200.0;
pub const FOCUS: f64 = 120.0;
pub const FADE: f64 = 140.0;

/// The scale from which a window opens, about its centre.
const OPEN_SCALE: f64 = 0.94;

/// Blur, offset down and opacity of the shadow of a window without the
/// focus and of the one with it.
const SHADES: [(f64, f64, f64); 2] = [(12.0, 4.0, 0.25), (28.0, 8.0, 0.45)];

/// What `changes` needs besides the placement.
#[derive(Clone, Copy, Default)]
pub struct Dress<'a> {
    /// The colours of the border: accent, accent 2, line and urgent; none
    /// when the borders have no shader.
    pub colors: Option<&'a [[f64; 3]; 4]>,
    /// Whether the windows have shadows (`look.shadows`).
    pub shadows: bool,
    /// Whether changes move over time (`look.animations`).
    pub animate: bool,
    /// How far a window that a tag switch brings in slides, in pixels; the
    /// sign is the direction.
    pub slide: f64,
}

/// The id of the shadow of a window without the focus.
#[must_use]
pub fn shadow_of(surface: &str) -> String {
    format!("ts-{surface}")
}

/// The id of the deeper shadow of the focused window.
#[must_use]
pub fn glow_of(surface: &str) -> String {
    format!("tf-{surface}")
}

/// The id of the gradient border of the focused window.
#[must_use]
pub fn focus_of(surface: &str) -> String {
    format!("tg-{surface}")
}

/// The nodes of the two shadows of a window, hidden until it has a place;
/// the deep one starts transparent.
#[must_use]
pub fn shadow_nodes(surface: &str, shader: &str, parent: &str) -> Vec<Command> {
    let node = |id: String, opacity: f64| Command::Create {
        id,
        kind: NodeKind::Shader,
        parent: parent.into(),
        props: PropsUpdate {
            shader: Some(shader.into()),
            size: Some([1.0, 1.0]),
            visible: Some(false),
            opacity: Some(opacity),
            input: Some(chora_scene::InputMode::Pass),
            ..PropsUpdate::default()
        },
    };
    vec![node(shadow_of(surface), 1.0), node(glow_of(surface), 0.0)]
}

/// The updates of the geometry of the shadows around a placement.
#[must_use]
pub fn shadow_updates(placement: &Placement, on: bool) -> Vec<Command> {
    let width = f64::from(placement.border);
    let show = on && placement.border > 0;
    let [fw, fh] = placement.frame.map(f64::from);
    let boxed = [fw + 2.0 * width, fh + 2.0 * width];
    let radius = f64::from(placement.radius) + width;
    [
        (shadow_of(&placement.id), SHADES[0]),
        (glow_of(&placement.id), SHADES[1]),
    ]
    .into_iter()
    .map(|(id, (blur, down, opacity))| {
        let pad = (blur * 1.5).ceil();
        let size = [boxed[0] + 2.0 * pad, boxed[1] + 2.0 * pad];
        Command::Update {
            id,
            props: PropsUpdate {
                visible: Some(show),
                size: Some(size),
                transform: Some(Transform::translate(-width - pad, -width - pad + down, 0.0)),
                uniforms: Some(BTreeMap::from([
                    (
                        "color".to_owned(),
                        Uniform::Vector(vec![0.0, 0.0, 0.0, opacity]),
                    ),
                    ("radius".to_owned(), Uniform::Float(radius)),
                    ("blur".to_owned(), Uniform::Float(blur)),
                    ("box".to_owned(), Uniform::Vector(boxed.to_vec())),
                    ("logical".to_owned(), Uniform::Vector(size.to_vec())),
                ])),
                ..PropsUpdate::default()
            },
        }
    })
    .collect()
}

/// A change of `props` of a node: at once, or over `duration`.
#[must_use]
pub fn change(id: String, props: PropsUpdate, animate: Option<f64>) -> Command {
    match animate {
        Some(duration) => Command::Animate {
            id,
            props,
            duration,
            easing: Easing::EaseOut,
        },
        None => Command::Update { id, props },
    }
}

fn opacity(value: f64) -> PropsUpdate {
    PropsUpdate {
        opacity: Some(value),
        ..PropsUpdate::default()
    }
}

/// The cross-fade of the gradient border and of the deep shadow when the
/// mark of a window changes (or a window is new).
#[must_use]
pub fn fade(sent: Option<&Placement>, placement: &Placement, animate: bool) -> Vec<Command> {
    let focus = |p: &Placement| p.mark == Mark::Focus && p.border > 0;
    let now = focus(placement);
    if sent.is_some_and(|s| focus(s) == now) {
        return Vec::new();
    }
    let value = if now { 1.0 } else { 0.0 };
    let how = (animate && sent.is_some_and(|s| s.at.is_some())).then_some(FOCUS);
    [focus_of(&placement.id), glow_of(&placement.id)]
        .into_iter()
        .map(|id| change(id, opacity(value), how))
        .collect()
}

/// The transform of a window group at a place, scaled about the middle of
/// its frame.
#[must_use]
pub fn scaled(at: [f64; 2], frame: [f64; 2], scale: f64) -> Transform {
    let centre = [frame[0] / 2.0, frame[1] / 2.0];
    Transform::translate(at[0] + centre[0], at[1] + centre[1], 0.0).then(
        &Transform::scale(scale, scale, 1.0)
            .then(&Transform::translate(-centre[0], -centre[1], 0.0)),
    )
}

/// The commands that put a window group at its place: a new window opens
/// from 94 % and transparent, a window that a tag switch brings back
/// slides in, a window that moved slides, and without `animate` every one
/// jumps. A move carries the opacity 1 too: it ends an open that still runs,
/// and the window must not stay half transparent.
#[must_use]
pub fn place(
    sent: Option<&Placement>,
    placement: &Placement,
    at: [i32; 2],
    dress: &Dress,
) -> Vec<Command> {
    let group = format!("tw-{}", placement.id);
    let target = [f64::from(at[0]), f64::from(at[1])];
    let frame = placement.frame.map(f64::from);
    let end = Transform::translate(target[0], target[1], 0.0);
    let shown = |transform: Transform, opacity: Option<f64>| PropsUpdate {
        transform: Some(transform),
        visible: Some(true),
        opacity,
        ..PropsUpdate::default()
    };
    let to = |transform: Transform, opacity: Option<f64>| PropsUpdate {
        transform: Some(transform),
        opacity,
        ..PropsUpdate::default()
    };
    if !dress.animate || (placement.border == 0 && sent.is_none()) {
        return vec![Command::Update {
            id: group,
            props: shown(end, None),
        }];
    }
    match sent {
        None => vec![
            Command::Update {
                id: group.clone(),
                props: shown(scaled(target, frame, OPEN_SCALE), Some(0.0)),
            },
            change(group, to(end, Some(1.0)), Some(OPEN)),
        ],
        Some(s) if s.at.is_none() && dress.slide != 0.0 => vec![
            Command::Update {
                id: group.clone(),
                props: shown(
                    Transform::translate(target[0] + dress.slide, target[1], 0.0),
                    Some(1.0),
                ),
            },
            change(group, to(end, Some(1.0)), Some(MOVE)),
        ],
        Some(s) if s.at.is_none() => vec![Command::Update {
            id: group,
            props: shown(end, None),
        }],
        Some(_) => vec![change(group, to(end, Some(1.0)), Some(MOVE))],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wm::States;

    fn placed(at: Option<[i32; 2]>, mark: Mark) -> Placement {
        Placement {
            id: "s1".into(),
            at,
            size: Some([100, 80]),
            states: States::default(),
            radius: 10,
            frame: [100, 80],
            border: 2,
            mark,
        }
    }

    const ON: Dress<'static> = Dress {
        colors: None,
        shadows: true,
        animate: true,
        slide: 0.0,
    };

    fn animated(commands: &[Command]) -> Vec<(&str, f64)> {
        commands
            .iter()
            .filter_map(|c| match c {
                Command::Animate { id, duration, .. } => Some((id.as_str(), *duration)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_new_window_opens_in_180_ms_from_transparent() {
        let new = placed(Some([20, 20]), Mark::Focus);
        let commands = place(None, &new, [20, 20], &ON);
        assert_eq!(animated(&commands), [("tw-s1", 180.0)]);
        let Command::Update { props, .. } = &commands[0] else {
            panic!("{commands:?}")
        };
        assert_eq!(props.opacity, Some(0.0));
        assert_eq!(props.visible, Some(true));
    }

    #[test]
    fn a_moved_window_slides_in_200_ms_and_the_size_is_not_animated() {
        let before = placed(Some([20, 20]), Mark::Line);
        let after = placed(Some([300, 20]), Mark::Line);
        let commands = place(Some(&before), &after, [300, 20], &ON);
        assert_eq!(animated(&commands), [("tw-s1", 200.0)]);
        assert!(commands.iter().all(|c| match c {
            Command::Animate { props, .. } => props.size.is_none(),
            _ => true,
        }));
    }

    #[test]
    fn a_move_ends_an_open_that_still_runs_with_the_opacity_1() {
        let before = placed(Some([20, 20]), Mark::Line);
        let after = placed(Some([300, 20]), Mark::Line);
        let commands = place(Some(&before), &after, [300, 20], &ON);
        assert!(matches!(&commands[..], [Command::Animate { props, .. }]
            if props.opacity == Some(1.0)));
    }

    #[test]
    fn a_window_that_a_tag_switch_brings_in_slides_from_the_side() {
        let away = placed(None, Mark::Line);
        let back = placed(Some([20, 20]), Mark::Line);
        let dress = Dress {
            slide: 1280.0,
            ..ON
        };
        let commands = place(Some(&away), &back, [20, 20], &dress);
        let Command::Update { props, .. } = &commands[0] else {
            panic!("{commands:?}")
        };
        let matrix = serde_json::to_value(props.transform.as_ref().unwrap()).unwrap();
        let from = matrix["matrix"][12].as_f64().unwrap();
        assert!((from - 1300.0).abs() < 1e-9, "{from}");
        assert_eq!(animated(&commands), [("tw-s1", 200.0)]);
    }

    #[test]
    fn without_animations_everything_jumps() {
        let off = Dress {
            animate: false,
            ..ON
        };
        let first = placed(Some([20, 20]), Mark::Focus);
        let other = placed(Some([30, 20]), Mark::Line);
        for (sent, now) in [(None, &first), (Some(&first), &other)] {
            let at = now.at.unwrap();
            let mut all = place(sent, now, at, &off);
            all.extend(fade(sent, now, false));
            assert!(animated(&all).is_empty(), "{all:?}");
        }
    }

    #[test]
    fn the_mark_cross_fades_in_120_ms() {
        let line = placed(Some([20, 20]), Mark::Line);
        let focus = placed(Some([20, 20]), Mark::Focus);
        let commands = fade(Some(&line), &focus, true);
        assert_eq!(animated(&commands), [("tg-s1", 120.0), ("tf-s1", 120.0)]);
        assert_eq!(fade(Some(&focus), &focus, true), []);
    }

    #[test]
    fn the_shadows_are_as_the_design_has_them() {
        let updates = shadow_updates(&placed(Some([0, 0]), Mark::Focus), true);
        let sizes: Vec<[f64; 2]> = updates
            .iter()
            .filter_map(|c| match c {
                Command::Update { props, .. } => props.size,
                _ => None,
            })
            .collect();
        assert_eq!(sizes, [[140.0, 120.0], [188.0, 168.0]]);
        let none = shadow_updates(&placed(Some([0, 0]), Mark::Focus), false);
        assert!(none.iter().all(|c| matches!(c,
            Command::Update { props, .. } if props.visible == Some(false))));
    }
}
