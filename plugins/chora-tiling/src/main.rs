//! chora-tiling: a tiling plugin with the user experience of awesome. Each
//! output has tags, each tag a layout, and the bindings come from a YAML
//! file (see `tiling.yaml`). Other clients, for example an agent, switch
//! the layout through the exported methods `tiling.set_layout` and
//! `tiling.get_layout`.

mod config;
mod layout;
mod wm;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::ExitCode,
};

use chora_kit::conn::Conn;
use chora_scene::{
    Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceRequest, ToplevelState,
    Transform,
    ipc::{ClientError, Export, Grab, PressState, Query, RpcError, Subscribe},
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    config::{Action, Config},
    layout::Layout,
    wm::{Effect, Placement, States, WINDOW_KEY, Wm},
};

/// The client name, and the group that holds the windows.
const NAME: &str = "tiling";

/// Tile the windows of chora with tags and layouts in the style of awesome.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The configuration file. Without it, chora-tiling reads
    /// $XDG_CONFIG_HOME/chora/chora-tiling.yaml, by default in ~/.config,
    /// if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default configuration and exit.
    #[arg(long)]
    print_config: bool,
    /// The app id of windows to leave to other clients; repeatable.
    #[arg(long, value_name = "APP_ID")]
    ignore: Vec<String>,
    /// Manages only this output, by name, and leaves the others to other
    /// window managers; all outputs without it. Repeatable.
    #[arg(long = "output", value_name = "NAME")]
    outputs: Vec<String>,
}

/// Parameters of `tiling.set_layout`; a missing field keeps its value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetLayout {
    layout: Option<Layout>,
    gap: Option<u16>,
}

/// Parameters of `tiling.give`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Give {
    to: String,
}

/// The answer of `tiling.set_layout` and `tiling.get_layout`.
#[derive(Serialize)]
struct LayoutInfo {
    layout: Layout,
    gap: u16,
    master_width: f64,
    masters: usize,
    columns: usize,
    /// The windows that show on the current output, in their order.
    windows: Vec<String>,
}

struct Tiling {
    client: Conn,
    wm: Wm,
    /// The configuration file of the command line.
    arg: Option<PathBuf>,
    /// The app ids of windows to leave to other clients.
    ignore: Vec<String>,
    /// The outputs that `--output` leaves to this plugin; all when empty.
    outputs: Vec<String>,
    /// The grabbed bindings, as the core names them.
    grabs: Vec<(String, Action)>,
    /// What the last transaction set, so that the next one sends changes
    /// only.
    sent: HashMap<String, Placement>,
    order: Vec<String>,
    focus: ShownFocus,
    /// The meta that the core has, by node: what a restart rebuilds from.
    saved: HashMap<String, BTreeMap<String, String>>,
    /// The claim that the core has (D55).
    claimed: Option<[f64; 4]>,
}

fn main() -> ExitCode {
    let args = Args::try_parse().unwrap_or_else(|err| {
        if err.kind() == clap::error::ErrorKind::UnknownArgument {
            eprintln!(
                "chora-tiling: --mod, --terminal, --launcher, --gap and --layout moved into \
                 the configuration file; chora-tiling --print-config prints it"
            );
        }
        err.exit()
    });
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chora-tiling: {err}");
            ExitCode::FAILURE
        }
    }
}

/// The configuration file to read: the argument, or the file of the user.
fn config_path(arg: Option<&PathBuf>) -> Option<PathBuf> {
    arg.cloned().or_else(|| {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("chora/chora-tiling.yaml")).filter(|p| p.exists())
    })
}

/// Reads the configuration file, or the default without one.
fn load(arg: Option<&PathBuf>) -> Result<Config, String> {
    config_path(arg).map_or_else(|| Config::parse(config::DEFAULT), |p| Config::read(&p))
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let config = load(args.config.as_ref())?;
    let mut tiling = Tiling {
        client: Conn::connect(NAME)?,
        wm: Wm::new(config),
        arg: args.config,
        ignore: args.ignore,
        outputs: args.outputs,
        grabs: Vec::new(),
        sent: HashMap::new(),
        order: Vec::new(),
        focus: ShownFocus::default(),
        saved: HashMap::new(),
        claimed: None,
    };
    tiling.start()?;
    loop {
        let event = tiling.client.next_event()?;
        tiling.handle(event)?;
    }
}

/// The whole output in scene coordinates, as the default camera shows it.
fn full_area(output: &OutputInfo) -> layout::Rect {
    let ([x, y], [w, h]) = (output.position, output.size);
    [x, y, w, h]
}

impl Tiling {
    /// Whether `--output` leaves an output to this plugin.
    fn manages(&self, output: &str) -> bool {
        self.outputs.is_empty() || self.outputs.iter().any(|o| o == output)
    }

    /// Whether `--output` names outputs and none of them is there: the
    /// plugin then leaves all windows to other window managers.
    fn without_outputs(&self) -> bool {
        !self.outputs.is_empty() && self.wm.outputs.is_empty()
    }

    /// Gives all windows away with a remove without `to`, so that another
    /// window manager adopts them, when the last output of `--output`
    /// goes.
    fn give_all_away(&mut self) -> Result<(), ClientError> {
        let ids: Vec<String> = self.wm.windows.iter().map(|w| w.id.clone()).collect();
        let commands = ids
            .iter()
            .map(|id| Command::Remove {
                id: id.clone(),
                to: None,
            })
            .collect();
        self.apply(commands)?;
        for id in &ids {
            self.forget_window(id);
        }
        Ok(())
    }

    fn start(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.client.call("output.list", Value::Null)?;
        for output in &outputs {
            if self.manages(&output.name) {
                self.wm
                    .set_output(&output.name, output.layout_area(), full_area(output));
            }
        }
        // The nodes that the core kept over a restart with resume.
        let kept = self.query(&format!("owner:{NAME}"))?;
        if kept.iter().any(|n| n["id"] == NAME) {
            self.restore(&kept);
        } else {
            let group = Command::Create {
                id: NAME.into(),
                kind: NodeKind::Group,
                parent: ROOT.into(),
                props: PropsUpdate::default(),
            };
            self.call_apply(vec![group])?;
        }
        self.grab_all()?;
        let names = ["set_layout", "get_layout", "give"]
            .map(String::from)
            .to_vec();
        self.client
            .call::<_, Value>("methods.export", Export { names })?;
        let events = [
            "surface.created",
            "surface.destroyed",
            "surface.detached",
            "surface.changed",
            "surface.requested",
            "output.changed",
            "output.removed",
            "pointer.entered",
            "pointer.pressed",
            "focus.changed",
        ];
        self.client.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: events.map(String::from).to_vec(),
            },
        )?;
        // Free surfaces, and those the core reserved for tiling, for example
        // during the grace time of a restart.
        for node in self.query("kind:surface")? {
            let ours = node["to"].is_null() || node["to"] == NAME;
            if let (Some(id), None, true) = (node["id"].as_str(), node["owner"].as_str(), ours) {
                self.adopt(id)?;
            }
        }
        self.relayout()
    }

    /// Rebuilds the model from the meta of the kept nodes. A kept window
    /// without meta comes back as a new one.
    fn restore(&mut self, kept: &[Value]) {
        let meta = |node: &Value| {
            serde_json::from_value::<BTreeMap<String, String>>(node["meta"].clone())
                .unwrap_or_default()
        };
        let group = kept
            .iter()
            .find(|n| n["id"] == NAME)
            .map(meta)
            .unwrap_or_default();
        let windows: Vec<(String, String)> = kept
            .iter()
            .filter(|n| n["kind"] == "surface")
            .filter_map(|n| {
                let text = meta(n).remove(WINDOW_KEY).unwrap_or_default();
                Some((n["id"].as_str()?.to_owned(), text))
            })
            .collect();
        self.wm.restore(&group, &windows);
        for (id, _) in &windows {
            if !self.wm.has(id) {
                self.wm.add_window(id);
            }
        }
    }

    /// Grabs every binding of the configuration. A binding that another
    /// client holds only logs.
    fn grab_all(&mut self) -> Result<(), ClientError> {
        let bindings = self.wm.config.bindings.clone();
        for (text, action) in bindings {
            let binding = self
                .wm
                .config
                .binding(&text)
                .expect("the configuration checks its bindings");
            match self
                .client
                .call::<_, String>("input.grab", Grab { binding })
            {
                Ok(normal) => self.grabs.push((normal, action)),
                Err(ClientError::Rpc(err)) => eprintln!("chora-tiling: {text}: {err}"),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    /// Reads the configuration again and grabs its bindings.
    fn reload(&mut self) -> Result<(), ClientError> {
        let config = match load(self.arg.as_ref()) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("chora-tiling: {err}");
                return Ok(());
            }
        };
        for (normal, _) in std::mem::take(&mut self.grabs) {
            let binding = normal.parse().expect("the core names valid bindings");
            self.client
                .call::<_, Value>("input.ungrab", Grab { binding })?;
        }
        self.wm.reconfigure(config);
        self.grab_all()?;
        self.relayout()
    }

    fn query(&mut self, selector: &str) -> Result<Vec<Value>, ClientError> {
        self.client.call(
            "scene.query",
            Query {
                selector: selector.into(),
            },
        )
    }

    fn handle(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            // The start adopts surfaces that the events then announce.
            Event::SurfaceCreated { surface, .. } if self.wm.has(&surface) => return Ok(()),
            // The core reserved it for another window manager.
            Event::SurfaceCreated { to: Some(to), .. } if to != NAME => return Ok(()),
            // A detached surface has no owner any more, so it is free to take.
            Event::SurfaceCreated { surface, .. } | Event::SurfaceDetached { surface } => {
                self.wm.remove_window(&surface);
                self.adopt(&surface)?;
            }
            Event::SurfaceDestroyed { surface } => {
                self.forget_window(&surface);
            }
            Event::SurfaceChanged {
                surface,
                size: Some(size),
                ..
            } => {
                if !self.wm.set_size(&surface, size) {
                    return Ok(());
                }
            }
            Event::OutputChanged { output } if self.manages(&output.name) => {
                self.wm
                    .set_output(&output.name, output.layout_area(), full_area(&output));
            }
            Event::FocusChanged { surface } => {
                self.focus_changed(surface.as_deref());
                return Ok(());
            }
            Event::OutputRemoved { name } => self.output_removed(&name)?,
            Event::InputKey {
                binding,
                state: PressState::Pressed,
            } => return self.act(&binding),
            Event::PointerEntered { node, moved } => {
                if !self.pointer_focus(&node, false, moved) {
                    return Ok(());
                }
            }
            Event::PointerPressed { node, .. } => {
                if !self.pointer_focus(&node, true, true) {
                    return Ok(());
                }
            }
            Event::SurfaceRequested { surface, request } if self.wm.has(&surface) => {
                if request == SurfaceRequest::Close {
                    return self.apply(vec![Command::Close { surface }]);
                }
                self.wm.request(&surface, request);
            }
            Event::MethodCalled {
                call,
                method,
                params,
                ..
            } => return self.answer(call, &method, params),
            _ => return Ok(()),
        }
        self.relayout()
    }

    /// Answers a call of an exported method. The call may have ended
    /// before the answer, for example when its caller left; that is no
    /// reason to exit.
    fn answer(&mut self, call: u64, method: &str, params: Value) -> Result<(), ClientError> {
        let result = self.method(method, params);
        match self.client.answer(call, result) {
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {err}");
                Ok(())
            }
            other => other,
        }
    }

    /// Runs an exported method. A failed relayout fails the method.
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        if method == "give" {
            return self.give(params);
        }
        if method == "set_layout" {
            let change: SetLayout = serde_json::from_value(params)
                .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))?;
            if let Some(layout) = change.layout {
                self.wm.set_layout(layout);
            }
            self.wm.config.gap = change.gap.unwrap_or(self.wm.config.gap);
            self.relayout()
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        let (layout, params) = self
            .wm
            .current()
            .ok_or_else(|| RpcError::new(RpcError::REJECTED, "no output"))?;
        let info = LayoutInfo {
            layout,
            gap: self.wm.config.gap,
            master_width: params.master_width,
            masters: params.masters,
            columns: params.columns,
            windows: self.wm.shown_windows(),
        };
        Ok(serde_json::to_value(info).expect("layouts always serialize"))
    }

    /// Hands the focused window to another client with a remove with `to`,
    /// and answers with its id.
    fn give(&mut self, params: Value) -> Result<Value, RpcError> {
        let Give { to } = serde_json::from_value(params)
            .map_err(|e| RpcError::new(RpcError::INVALID_PARAMS, e.to_string()))?;
        let surface = self
            .wm
            .focused()
            .ok_or_else(|| RpcError::new(RpcError::REJECTED, "no focused window"))?
            .to_owned();
        let give = Command::Remove {
            id: surface.clone(),
            to: Some(to),
        };
        self.call_apply(vec![give])
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        self.forget_window(&surface);
        self.relayout()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(Value::String(surface))
    }

    fn focus_changed(&mut self, surface: Option<&str>) {
        self.focus.moved(surface);
    }

    /// An output goes; without one of `--output` the windows go too.
    fn output_removed(&mut self, name: &str) -> Result<(), ClientError> {
        self.wm.remove_output(name);
        if self.without_outputs() {
            self.give_all_away()?;
        }
        Ok(())
    }

    /// Focuses a window under the pointer. While another window manager
    /// has the focus, the focused window of tiling takes it again.
    fn pointer_focus(&mut self, node: &str, clicked: bool, moved: bool) -> bool {
        let took = self.wm.pointer_focus(node, clicked, moved, self.focus.lost);
        if took {
            self.focus.take();
        }
        took
    }

    fn act(&mut self, binding: &str) -> Result<(), ClientError> {
        let Some((_, action)) = self.grabs.iter().find(|(b, _)| b == binding) else {
            return Ok(());
        };
        let action = action.clone();
        let before = self.wm.focused().map(String::from);
        let effect = self.wm.act(&action);
        // A binding that moves the focus of tiling takes it back.
        if self.wm.focused() != before.as_deref() {
            self.focus.take();
        }
        match effect {
            Some(Effect::Relayout) => self.relayout(),
            Some(Effect::Spawn(command)) => {
                spawn(&command);
                Ok(())
            }
            Some(Effect::Close(surface)) => self.apply(vec![Command::Close { surface }]),
            Some(Effect::Reload) => self.reload(),
            None => Ok(()),
        }
    }

    /// Passes a surface that the core reserved for tiling on to other
    /// window managers while tiling has no output: it takes it and lets
    /// it go at once, without `to`.
    fn pass_on_reserved(&mut self, surface: &str) -> Result<(), ClientError> {
        let reserved = self
            .query(&format!("id:{surface}"))?
            .iter()
            .any(|node| node["to"] == NAME);
        if !reserved {
            return Ok(());
        }
        self.apply(pass_on(surface))
    }

    /// Takes a surface without an owner. A surface that another client took
    /// first, or whose app id `--ignore` names, stays out of the layout.
    fn adopt(&mut self, surface: &str) -> Result<(), ClientError> {
        if self.without_outputs() {
            return self.pass_on_reserved(surface);
        }
        let ignored = self.query(&format!("id:{surface}"))?.iter().any(|node| {
            node["surface"]["app_id"]
                .as_str()
                .is_some_and(|app| self.ignore.iter().any(|i| i == app))
        });
        if ignored {
            return self.pass_on_reserved(surface);
        }
        let attach = Command::Attach {
            surface: surface.into(),
            parent: NAME.into(),
        };
        match self.call_apply(vec![attach]) {
            Ok(()) => {
                // A new window of tiling takes the focus.
                self.wm.add_window(surface);
                self.focus.take();
                Ok(())
            }
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {surface}: {err}");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Sends what changed since the last transaction: sizes and states,
    /// positions, visibility, the stacking order and the focus.
    fn relayout(&mut self) -> Result<(), ClientError> {
        let placements = self.wm.arrange();
        let mut commands = Vec::new();
        for placement in &placements {
            commands.extend(changes(self.sent.get(&placement.id), placement));
        }
        let order: Vec<String> = placements.iter().map(|p| p.id.clone()).collect();
        if order != self.order {
            commands.extend(order.iter().enumerate().map(|(index, id)| Command::Move {
                id: id.clone(),
                parent: None,
                index: Some(index),
            }));
        }
        let focus = self.wm.focused().map(String::from);
        let send_focus = self.focus.sends(focus.as_deref());
        if send_focus {
            commands.push(Command::Focus {
                surface: focus.clone(),
            });
        }
        let saved = self.meta();
        for (id, meta) in &saved {
            commands.extend(meta_change(self.saved.get(id), id, meta));
        }
        let claim = self.wm.claim();
        if claim != self.claimed {
            commands.push(Command::Update {
                id: NAME.into(),
                props: PropsUpdate {
                    // No outputs: no claim, so `null`.
                    claim: Some(claim),
                    ..PropsUpdate::default()
                },
            });
        }
        if commands.is_empty() {
            return Ok(());
        }
        self.saved = saved;
        self.claimed = claim;
        self.sent = placements.into_iter().map(|p| (p.id.clone(), p)).collect();
        self.order = order;
        if send_focus {
            self.focus.sent(focus);
        }
        self.apply(commands)
    }

    /// The meta of the group and of each window, by node.
    fn meta(&self) -> HashMap<String, BTreeMap<String, String>> {
        let windows = self.wm.window_meta().into_iter();
        windows
            .map(|(id, text)| (id, BTreeMap::from([(WINDOW_KEY.to_owned(), text)])))
            .chain([(NAME.to_owned(), self.wm.group_meta())])
            .collect()
    }

    /// Forgets a window that left tiling: its place in the model, and
    /// what the core shows of it.
    fn forget_window(&mut self, id: &str) {
        self.wm.remove_window(id);
        self.sent.remove(id);
        self.saved.remove(id);
    }

    /// Forgets what the core shows, so that the next transaction sends all.
    fn forget(&mut self) {
        self.sent.clear();
        self.saved.clear();
        self.claimed = None;
        self.order.clear();
        self.focus.sent = false;
    }

    fn call_apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        match self.client.try_apply(commands)? {
            None => Ok(()),
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    /// Applies a transaction. A rejected transaction only logs, and the next
    /// one sends everything again: a surface can disappear between an event
    /// and the answer to it, and the event that follows corrects the layout.
    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        match self.call_apply(commands) {
            Err(ClientError::Rpc(err)) => {
                if !err.is_gone_surface() {
                    eprintln!("chora-tiling: {err}");
                }
                self.forget();
                Ok(())
            }
            other => other,
        }
    }
}

/// The commands that bring a window from what was sent to its placement.
fn changes(sent: Option<&Placement>, placement: &Placement) -> Vec<Command> {
    let id = &placement.id;
    let Some([x, y]) = placement.at else {
        return hide(sent, id);
    };
    let mut commands = Vec::new();
    if sent.is_none_or(|s| s.size != placement.size || s.states != placement.states) {
        commands.push(configure(id, placement.size, placement.states));
    }
    if sent.is_none_or(|s| s.at != placement.at) {
        commands.push(Command::Update {
            id: id.clone(),
            props: PropsUpdate {
                transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
                visible: Some(true),
                ..PropsUpdate::default()
            },
        });
    }
    commands
}

/// The update from the saved meta of a node to its new meta. A key that
/// went stays in the core; the restore skips it.
fn meta_change(
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
fn hide(sent: Option<&Placement>, id: &str) -> Vec<Command> {
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
        id: id.into(),
        props: PropsUpdate {
            visible: Some(false),
            ..PropsUpdate::default()
        },
    });
    commands
}

fn configure(id: &str, size: Option<[i32; 2]>, states: States) -> Command {
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

/// Runs a shell command. `&` lets the shell exit at once, so the command
/// never becomes a zombie of this plugin.
fn spawn(command: &str) {
    let spawned = std::process::Command::new("sh")
        .args(["-c", &format!("{command} &")])
        .status();
    if let Err(err) = spawned {
        eprintln!("chora-tiling: cannot run {command:?}: {err}");
    }
}

/// Takes a surface and lets it go at once, without `to`: the core then
/// announces it as detached, free for any window manager.
fn pass_on(surface: &str) -> Vec<Command> {
    vec![
        Command::Attach {
            surface: surface.into(),
            parent: NAME.into(),
        },
        Command::Remove {
            id: surface.into(),
            to: None,
        },
    ]
}

/// The keyboard focus as tiling shows it to the core.
#[derive(Debug, Default)]
struct ShownFocus {
    /// The focus of the last transaction that sent one.
    surface: Option<String>,
    /// Whether the core has `surface`.
    sent: bool,
    /// Another window manager took the focus since. Tiling sends none
    /// until one of its windows takes the focus back, so that no relayout
    /// takes it.
    lost: bool,
}

impl ShownFocus {
    /// The core moved the focus. An empty focus is no loss: it happens
    /// when the focused window of tiling itself exits, and tiling must
    /// then still focus its next window.
    fn moved(&mut self, surface: Option<&str>) {
        match surface {
            None => {}
            Some(s) if Some(s) == self.surface.as_deref() => self.lost = false,
            Some(_) => self.lost = true,
        }
    }

    /// A window of tiling takes the focus back: by the pointer, a binding
    /// or as a new window. The next transaction sends it, even when it is
    /// the one sent before.
    fn take(&mut self) {
        if self.lost {
            self.lost = false;
            self.sent = false;
        }
    }

    /// Whether a transaction sends a focus.
    fn sends(&self, focus: Option<&str>) -> bool {
        !self.lost && (!self.sent || self.surface.as_deref() != focus)
    }

    /// A transaction sent a focus.
    fn sent(&mut self, focus: Option<String>) {
        self.surface = focus;
        self.sent = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_passed_on_is_free_for_any_window_manager() {
        assert!(matches!(
            &pass_on("s1")[..],
            [
                Command::Attach { surface, parent },
                Command::Remove { id, to: None },
            ] if surface == "s1" && parent == NAME && id == "s1"
        ));
    }

    #[test]
    fn a_relayout_takes_no_focus_from_another_window_manager() {
        let mut shown = ShownFocus::default();
        assert!(shown.sends(Some("s3")));
        shown.sent(Some("s3".into()));
        assert!(!shown.sends(Some("s3")));
        // chora-xp takes the focus: a relayout, even with a new focus of
        // tiling, as after a window exits, sends nothing.
        shown.moved(Some("s1"));
        assert!(!shown.sends(Some("s3")));
        assert!(!shown.sends(Some("s4")));
        // The pointer on s3 takes it back, though tiling sent s3 before.
        shown.take();
        assert!(shown.sends(Some("s3")));
        shown.sent(Some("s3".into()));
        // The core confirms the focus of tiling.
        shown.moved(Some("s3"));
        assert!(!shown.lost && !shown.sends(Some("s3")));
    }

    #[test]
    fn a_focus_emptied_by_a_closed_window_is_no_loss() {
        let mut shown = ShownFocus::default();
        shown.sent(Some("s3".into()));
        // The focused window of tiling exits: the core empties the focus.
        shown.moved(None);
        assert!(!shown.lost, "an empty focus is not another owner");
        assert!(shown.sends(Some("s4")), "the next window gets the focus");
        // Another window manager still takes it with its own window.
        shown.moved(Some("s9"));
        assert!(shown.lost && !shown.sends(Some("s4")));
    }

    #[test]
    fn a_window_new_to_the_core_hides_too() {
        let hidden = Placement {
            id: "s1".into(),
            at: None,
            size: None,
            states: States::default(),
        };
        let commands = changes(None, &hidden);
        assert!(matches!(
            &commands[..],
            [Command::Update { props, .. }] if props.visible == Some(false)
        ));
        assert_eq!(changes(Some(&hidden), &hidden).len(), 0);
    }
}
