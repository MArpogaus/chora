//! Floating windows (`chora_kit::float`): a client that shows a virtual
//! output of chora-xp on a shape, such as the construct on a face of its
//! cube, asks with `windows.float` that the windows of that output
//! float. While they float, their groups have opacity 0, so the output
//! shows the wallpaper, the icons and the taskbar without them, and the
//! caller shows each group on a plane of its own. chora-xp names the
//! groups in the answer, and again in the event `xp.floating` when they
//! change. A window that leaves a floating output, and every window of
//! an output that goes or floats no more, gets its opacity back.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chora_kit::float::{self, Float, Floated, Floating};
use chora_scene::{Command, PropsUpdate, ipc::RpcError};
use serde_json::Value;

use super::{Bar, NAME, Wm, decoration};

/// The outputs that float, each with the client that added it, the
/// windows that float as last sent, and the opacity that another client
/// asked for a window (D58), which a window has again when it lands.
#[derive(Default)]
pub(super) struct Floats {
    outputs: BTreeMap<String, Option<String>>,
    sent: BTreeMap<String, Vec<Floating>>,
    asked: HashMap<String, f64>,
}

impl Floats {
    /// Notes the opacity that another client asks for a window. Answers
    /// whether the group takes it now: not while the window floats.
    pub(super) fn ask(&mut self, surface: &str, opacity: f64) -> bool {
        self.asked.insert(surface.to_owned(), opacity);
        !self.floats(surface)
    }

    /// Notes the opacity that a window had before a restart.
    pub(super) fn seed(&mut self, surface: &str, opacity: f64) {
        self.asked.insert(surface.to_owned(), opacity);
    }

    /// Every output floats no more: before a restart, which must leave no
    /// window hidden.
    pub(super) fn land_all(&mut self) {
        self.outputs.clear();
    }

    /// A client went: the outputs that it added float no more, so that
    /// their windows show while the outputs wait for their owner.
    pub(super) fn owner_gone(&mut self, client: &str) {
        self.outputs
            .retain(|_, owner| owner.as_deref() != Some(client));
    }

    /// Forgets a window that went.
    pub(super) fn forget(&mut self, surface: &str) {
        self.asked.remove(surface);
    }

    fn floats(&self, surface: &str) -> bool {
        let mut all = self.sent.values().flatten();
        all.any(|w| w.surface == surface)
    }

    fn landed(&self, surface: &str) -> f64 {
        self.asked.get(surface).copied().unwrap_or(1.0)
    }
}

fn opacity(surface: &str, opacity: f64) -> Command {
    Command::Update {
        id: decoration::group_id(surface),
        props: PropsUpdate {
            opacity: Some(opacity),
            ..PropsUpdate::default()
        },
    }
}

impl Wm {
    /// The method `windows.float`: the windows of a virtual output float
    /// from now on, or no more. Answers the floating windows.
    pub(super) fn float(&mut self, Float { output, on }: Float) -> Result<Value, RpcError> {
        let rejected = |message: String| RpcError::new(RpcError::REJECTED, message);
        let faces = |b: &&Bar| b.output == output && b.is_virtual();
        let Some(bar) = self.bars.iter().find(faces) else {
            return Err(rejected(format!("xp has no virtual output {output}")));
        };
        if on {
            let owner = bar.owner().map(str::to_owned);
            self.floats.outputs.insert(output.clone(), owner);
        } else {
            self.floats.outputs.remove(&output);
        }
        self.apply(Vec::new())
            .map_err(|e| rejected(e.to_string()))?;
        let windows = self.floats.sent.get(&output).cloned().unwrap_or_default();
        serde_json::to_value(Floated { output, windows }).map_err(|e| rejected(e.to_string()))
    }

    /// The windows that float now, by output, the lowest first: those
    /// that show on a floating output.
    fn floating(&self) -> BTreeMap<String, Vec<Floating>> {
        let mut now = BTreeMap::new();
        for output in self.floats.outputs.keys() {
            let Some(bar) = self.bars.iter().find(|b| b.output == *output) else {
                continue;
            };
            let [left, top, ..] = bar.area();
            let on_it = self.windows.iter().filter(|w| {
                let here = self.bar_at(w.rect()).is_some_and(|b| b.output == *output);
                here && w.shown && !w.minimized && w.desktop == self.desktop
            });
            let windows = on_it.map(|w| {
                let [x, y, width, height] = w.bounds();
                Floating {
                    node: decoration::group_id(&w.surface),
                    surface: w.surface.clone(),
                    rect: [x - f64::from(left), y - f64::from(top), width, height],
                }
            });
            now.insert(output.clone(), float::top(windows.collect()));
        }
        now
    }

    /// Brings the groups in line with the windows that float now, with
    /// commands that go after `commands`. Answers the outputs whose
    /// floating windows changed, for the event.
    pub(super) fn sync_floats(&mut self, commands: &mut Vec<Command>) -> Vec<Floated> {
        if self.floats.outputs.is_empty() && self.floats.sent.is_empty() {
            return Vec::new();
        }
        let bars = &self.bars;
        (self.floats.outputs).retain(|o, _| bars.iter().any(|b| b.output == *o && b.is_virtual()));
        let now = self.floating();
        if now == self.floats.sent {
            return Vec::new();
        }
        let floats = |all: &BTreeMap<String, Vec<Floating>>, surface: &str| {
            all.values().flatten().any(|w| w.surface == surface)
        };
        for window in now.values().flatten() {
            if !floats(&self.floats.sent, &window.surface) {
                commands.push(opacity(&window.surface, 0.0));
            }
        }
        for window in self.floats.sent.values().flatten() {
            if !floats(&now, &window.surface) && self.index(&window.surface).is_some() {
                let landed = self.floats.landed(&window.surface);
                commands.push(opacity(&window.surface, landed));
            }
        }
        let outputs: BTreeSet<&String> = now.keys().chain(self.floats.sent.keys()).collect();
        let changed = outputs.into_iter().filter_map(|output| {
            let windows = now.get(output).cloned().unwrap_or_default();
            let before = self.floats.sent.get(output).cloned().unwrap_or_default();
            (windows != before).then(|| Floated {
                output: output.clone(),
                windows,
            })
        });
        let changed = changed.collect();
        self.floats.sent = now;
        changed
    }

    /// Tells the other clients the floating windows that changed.
    pub(super) fn tell_floats(&mut self, changed: Vec<Floated>) {
        for floated in changed {
            let event = serde_json::json!({"event": float::event(NAME), "data": floated});
            if let Err(err) = self.rt.conn.call::<_, Value>("events.emit", event) {
                eprintln!("chora-xp: {err}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floating(surface: &str) -> Floating {
        Floating {
            node: decoration::group_id(surface),
            surface: surface.into(),
            rect: [0.0, 0.0, 10.0, 10.0],
        }
    }

    #[test]
    fn an_asked_opacity_waits_while_the_window_floats() {
        let mut floats = Floats::default();
        assert!(floats.ask("s1", 0.8), "the group takes it");
        assert!((floats.landed("s1") - 0.8).abs() < 1e-9);
        floats.sent.insert("VIRTUAL-1".into(), vec![floating("s1")]);
        assert!(!floats.ask("s1", 0.5), "not while it floats");
        assert!((floats.landed("s1") - 0.5).abs() < 1e-9, "it lands with it");
        assert!((floats.landed("s2") - 1.0).abs() < 1e-9, "nobody asked");
        floats.forget("s1");
        assert!((floats.landed("s1") - 1.0).abs() < 1e-9);
    }
}
