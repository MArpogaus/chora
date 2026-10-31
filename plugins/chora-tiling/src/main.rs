//! chora-tiling: a tiling plugin with the user experience of awesome. Each
//! output has tags, each tag a layout, and the bindings come from a YAML
//! file (see `tiling.yaml`). Other clients, for example an agent, switch
//! the layout and drive the windows through the exported `tiling.*`
//! methods (see `docs/configuration.md`).

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod config;
mod layout;
mod wm;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::ExitCode,
};

use chora_kit::{
    give::{self, Give},
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceRequest, ToplevelState,
    Transform,
    ipc::{ClientError, ExportMethod, PressState, Query, RpcError},
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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

/// Parameters of `tiling.view_tag`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewTag {
    tag: usize,
}

/// Parameters of `tiling.move_to_tag`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveToTag {
    surface: String,
    tag: usize,
}

/// Parameters of `tiling.focus` and `tiling.swap_master`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ForSurface {
    surface: String,
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
    rt: Runtime<Action>,
    wm: Wm,
    /// The configuration, as the file has it.
    settings: settings::File<Config>,
    /// The app ids of windows to leave to other clients.
    ignore: Vec<String>,
    /// What the last transaction set, so that the next one sends changes
    /// only.
    sent: HashMap<String, Placement>,
    order: Vec<String>,
    focus: ShownFocus,
    /// The meta that the core has, by node: what a restart rebuilds from.
    saved: HashMap<String, BTreeMap<String, String>>,
    /// The claim that the core has (D55).
    claimed: Option<[f64; 4]>,
    /// The window that the binding of `drag` holds.
    dragged: Option<String>,
}

#[allow(clippy::print_stdout, reason = "--print-config prints the default")]
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
    runtime::run(NAME, || play(&args))
}

/// One connection to the core: window management until it goes away.
fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-tiling")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let rt = Runtime::start(NAME, args.outputs.clone(), &EVENTS, methods)?;
    let mut tiling = Tiling {
        rt,
        wm: Wm::new(settings.value.clone()),
        settings,
        ignore: args.ignore.clone(),
        sent: HashMap::new(),
        order: Vec::new(),
        focus: ShownFocus::default(),
        saved: HashMap::new(),
        claimed: None,
        dragged: None,
    };
    tiling.start()?;
    Ok(runtime::serve(&mut tiling)?)
}

/// The events that tiling handles.
const EVENTS: [&str; 12] = [
    "surface.created",
    "surface.destroyed",
    "surface.detached",
    "surface.changed",
    "surface.requested",
    "node.requested",
    "output.changed",
    "output.removed",
    "pointer.entered",
    "pointer.pressed",
    "focus.changed",
    "input.button",
];

impl Plugin for Tiling {
    type Action = Action;

    fn runtime(&mut self) -> &mut Runtime<Action> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        self.handle(event)
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        self.run(method, params)
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.reconfigure()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

/// The whole output in scene coordinates, as the default camera shows it.
fn full_area(output: &OutputInfo) -> layout::Rect {
    let ([x, y], [w, h]) = (output.position, output.size);
    [x, y, w, h]
}

impl Tiling {
    /// Whether `--output` names outputs and none of them is there: the
    /// plugin then leaves all windows to other window managers.
    fn without_outputs(&self) -> bool {
        self.rt.filters() && self.wm.outputs.is_empty()
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
        for output in self.rt.outputs()? {
            self.wm
                .set_output(&output.name, output.layout_area(), full_area(&output));
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
    #[allow(clippy::expect_used, reason = "the configuration checks its bindings")]
    fn grab_all(&mut self) -> Result<(), ClientError> {
        let bindings = self.wm.config.bindings.clone();
        for (text, action) in bindings {
            let binding = self
                .wm
                .config
                .binding(&text)
                .expect("the configuration checks its bindings");
            self.rt.grab(binding, action)?;
        }
        Ok(())
    }

    /// Applies the settings and grabs their bindings.
    fn reconfigure(&mut self) -> Result<(), ClientError> {
        self.check_shaders()?;
        self.rt.ungrab_all()?;
        self.wm.reconfigure(self.settings.value.clone());
        self.grab_all()?;
        self.relayout()
    }

    fn query(&mut self, selector: &str) -> Result<Vec<Value>, ClientError> {
        self.rt.conn.call("scene.query", Query::new(selector))
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
            Event::OutputChanged { output } => {
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
            Event::InputButton {
                binding,
                state,
                position,
                ..
            } => self.drag(&binding, state, position),
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
            Event::SurfaceRequested { surface, request } => {
                return self.answer_surface(&surface, request);
            }
            Event::NodeRequested {
                node, action, args, ..
            } => {
                return self.answer_request(&node, &action, &args);
            }
            _ => return Ok(()),
        }
        self.relayout()
    }

    /// Answers a `surface.requested` of one of the windows.
    fn answer_surface(
        &mut self,
        surface: &str,
        request: SurfaceRequest,
    ) -> Result<(), ClientError> {
        if !self.wm.has(surface) {
            return Ok(());
        }
        if request == SurfaceRequest::Close {
            return self.apply(vec![Command::Close {
                surface: surface.into(),
            }]);
        }
        self.wm.request(surface, request);
        self.relayout()
    }

    /// Does what a request of another client asks of one of the windows
    /// (D31). Actions that `answers` does not know stay unanswered.
    fn answer_request(
        &mut self,
        node: &str,
        action: &str,
        args: &Value,
    ) -> Result<(), ClientError> {
        if !self.wm.has(node) {
            return Ok(());
        }
        let before = self.wm.focused().map(String::from);
        let Some(commands) = answers(&mut self.wm, node, action, args) else {
            return Ok(());
        };
        if self.wm.focused() != before.as_deref() {
            self.focus.take();
        }
        if action == "close" {
            return self.apply(commands);
        }
        if !commands.is_empty() {
            let applied = self.apply_report(commands)?;
            if leaves_tiling(applied, action, args) {
                self.forget_window(node);
            }
        }
        self.relayout()
    }

    /// Runs an exported method. Everything but `get_layout` and `get_tags` changes
    /// something, relayouts like the matching binding, and answers with
    /// the layout info of the current tag. A failed relayout fails the
    /// method.
    fn run(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        if method == give::METHOD {
            return self.give(params);
        }
        if method == "get_tags" {
            return Ok(json!({"outputs": self.wm.tags()}));
        }
        if method != "get_layout" {
            let before = self.wm.focused().map(String::from);
            let focuses = self.tune(method, params)?;
            // A method that moves the focus of tiling takes it back, as a
            // binding does.
            if focuses || self.wm.focused() != before.as_deref() {
                self.focus.take();
            }
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
        #[allow(clippy::expect_used, reason = "a layout has string keys only")]
        let value = serde_json::to_value(info).expect("layouts always serialize");
        Ok(value)
    }

    /// Changes what a method other than `window.give` and `get_layout` names.
    /// True when the method focuses a window on its own.
    fn tune(&mut self, method: &str, params: Value) -> Result<bool, RpcError> {
        Ok(match method {
            "set_layout" => {
                let change: SetLayout = runtime::params(params)?;
                if let Some(layout) = change.layout {
                    self.wm.set_layout(layout);
                }
                self.wm.config.gap = change.gap.unwrap_or(self.wm.config.gap);
                false
            }
            "view_tag" => {
                let ViewTag { tag } = runtime::params(params)?;
                self.check_tag(tag)?;
                self.wm.view_tag(tag);
                false
            }
            "move_to_tag" => {
                let MoveToTag { surface, tag } = runtime::params(params)?;
                self.check_tag(tag)?;
                only(self.wm.move_to_tag(&surface, tag), || {
                    format!("tiling has no window {surface}")
                })?;
                false
            }
            "focus" => {
                let ForSurface { surface } = runtime::params(params)?;
                only(self.wm.has(&surface), || {
                    format!("tiling has no window {surface}")
                })?;
                // A window that shows nowhere only gets the urgent mark.
                self.wm.focus(&surface);
                self.wm.focused() == Some(surface.as_str())
            }
            "focus_next" => {
                self.wm.focus_next();
                false
            }
            "focus_prev" => {
                self.wm.focus_prev();
                false
            }
            "swap_master" => {
                let ForSurface { surface } = runtime::params(params)?;
                only(self.wm.swap_master(&surface), || {
                    format!("{surface} shows nowhere")
                })?;
                false
            }
            other => {
                return Err(RpcError::new(
                    RpcError::METHOD_NOT_FOUND,
                    format!("tiling has no method {other}"),
                ));
            }
        })
    }

    /// A tag number of the configuration.
    fn check_tag(&self, tag: usize) -> Result<(), RpcError> {
        if (1..=self.wm.config.tags).contains(&tag) {
            return Ok(());
        }
        Err(RpcError::new(
            RpcError::INVALID_PARAMS,
            format!("no tag {tag}"),
        ))
    }

    /// `window.give`: hands a window, the focused one by default, to
    /// another window manager, and answers with its id.
    fn give(&mut self, params: Value) -> Result<Value, RpcError> {
        let Give { surface, to } = runtime::params(params)?;
        let surface = surface
            .or_else(|| self.wm.focused().map(String::from))
            .ok_or_else(|| RpcError::new(RpcError::REJECTED, "no focused window"))?;
        self.hand(&surface, &to)?;
        Ok(Value::String(surface))
    }

    /// Hands a window to the window manager `to` with a remove with `to`
    /// (D55); the receiver gives it the focus.
    fn hand(&mut self, surface: &str, to: &str) -> Result<(), RpcError> {
        only(self.wm.has(surface), || {
            format!("tiling has no window {surface}")
        })?;
        self.rt.conn.give(surface, to)?;
        self.forget_window(surface);
        self.relayout()
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }

    /// The binding of `drag`: the press takes the window under the
    /// pointer, and a release outside the claim of tiling hands it to the
    /// window manager whose claim holds the point (D55). Without one
    /// there, the window stays.
    fn drag(&mut self, binding: &str, state: PressState, point: [f64; 2]) {
        if !matches!(self.rt.action(binding), Some(Action::Drag)) {
            return;
        }
        if state == PressState::Pressed {
            self.dragged = self.window_at(point);
            return;
        }
        let Some(surface) = self.dragged.take() else {
            return;
        };
        if self.claimed.is_some_and(|c| inside(c, point)) {
            return;
        }
        let given = match self.rt.conn.claimant_at(point) {
            Ok(Some(to)) => self.hand(&surface, &to),
            Ok(None) => Err(RpcError::new(RpcError::REJECTED, "no claim there")),
            Err(err) => Err(err),
        };
        if let Err(err) = given {
            eprintln!("chora-tiling: {surface} stays: {}", err.message);
        }
    }

    /// The topmost shown window at a point in scene coordinates.
    fn window_at(&self, [px, py]: [f64; 2]) -> Option<String> {
        let rect = |id: &String| {
            let placement = self.sent.get(id)?;
            let window = self.wm.windows.iter().find(|w| &w.id == id)?;
            let [x, y] = placement.at?;
            let [w, h] = placement.size.or(window.size)?;
            Some([x, y, w, h].map(f64::from))
        };
        let hit = |id: &&String| rect(id).is_some_and(|r| inside(r, [px, py]));
        self.order.iter().rev().find(hit).cloned()
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
        let Some(action) = self.rt.action(binding).cloned() else {
            return Ok(());
        };
        let before = self.wm.focused().map(String::from);
        let effect = self.wm.act(&action);
        // A binding that moves the focus of tiling takes it back.
        if self.wm.focused() != before.as_deref() {
            self.focus.take();
        }
        match effect {
            Some(Effect::Relayout) => self.relayout(),
            Some(Effect::Run(command)) => {
                chora_kit::run(&command);
                Ok(())
            }
            Some(Effect::Close(surface)) => self.apply(vec![Command::Close { surface }]),
            // The same path as the method `settings.reload`.
            Some(Effect::Reload) => {
                if let Err(err) = runtime::reload_settings(self) {
                    eprintln!("chora-tiling: {}", err.message);
                }
                Ok(())
            }
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
        let nodes = self.query(&format!("id:{surface}"))?;
        let ignored = nodes.iter().any(|node| {
            node["surface"]["app_id"]
                .as_str()
                .is_some_and(|app| self.ignore.iter().any(|i| i == app))
        });
        // The size of a window that comes with one, as from another window
        // manager: it floats at that size, centered, and a drag hits it.
        let size = nodes
            .first()
            .and_then(|n| serde_json::from_value(n["surface"]["size"].clone()).ok());
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
                if let Some(size) = size {
                    self.wm.set_size(surface, size);
                }
                self.focus.take();
                self.style(surface)
            }
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {surface}: {err}");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Gives a new window the style of the settings. A shader that fails
    /// only logs, and the window stays.
    fn style(&mut self, surface: &str) -> Result<(), ClientError> {
        self.apply(vec![styled(&self.wm.config.style, surface)])
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
        // The focus on it is no longer that of tiling: when the window
        // comes back, tiling sends it again.
        if self.focus.surface.as_deref() == Some(id) {
            self.focus.surface = None;
            self.focus.sent = false;
        }
    }

    /// Forgets what the core shows, so that the next transaction sends all.
    fn forget(&mut self) {
        self.sent.clear();
        self.saved.clear();
        self.claimed = None;
        self.order.clear();
        self.focus.sent = false;
    }

    /// Has the core compile the shaders of the style, on a node that the
    /// same transaction removes again, so that `settings.set` refuses a
    /// shader that does not compile instead of each new window.
    fn check_shaders(&mut self) -> Result<(), ClientError> {
        let effects = self.settings.value.style.effects.clone();
        if effects.is_empty() {
            return Ok(());
        }
        let id = "tiling-shader-check";
        self.call_apply(vec![
            Command::Create {
                id: id.into(),
                kind: NodeKind::Rect,
                parent: NAME.into(),
                props: PropsUpdate {
                    effects: Some(effects),
                    ..PropsUpdate::default()
                },
            },
            Command::Remove {
                id: id.into(),
                to: None,
            },
        ])
    }

    fn call_apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        match self.rt.conn.try_apply(commands)? {
            None => Ok(()),
            Some(err) => Err(ClientError::Rpc(err)),
        }
    }

    /// Applies a transaction. A rejected transaction only logs, and the next
    /// one sends everything again: a surface can disappear between an event
    /// and the answer to it, and the event that follows corrects the layout.
    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        self.apply_report(commands).map(|_| ())
    }

    /// Like `apply`, but `Ok(false)` when the core rejected the transaction.
    fn apply_report(&mut self, commands: Vec<Command>) -> Result<bool, ClientError> {
        match self.call_apply(commands) {
            Err(ClientError::Rpc(err)) => {
                if !err.is_gone_surface() {
                    eprintln!("chora-tiling: {err}");
                }
                self.forget();
                Ok(false)
            }
            Ok(()) => Ok(true),
            Err(err) => Err(err),
        }
    }
}

/// Whether an answered request takes the window out of tiling: the core
/// accepted a `move` to a parent that is not the tiling group. A rejected
/// transaction leaves the window managed (D31).
fn leaves_tiling(applied: bool, action: &str, args: &Value) -> bool {
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
/// stays unanswered, as does an `update` with nothing left to change.
fn answers(wm: &mut Wm, node: &str, action: &str, args: &Value) -> Option<Vec<Command>> {
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
            parent: args
                .get("parent")
                .and_then(Value::as_str)
                .map(str::to_owned),
            index: args
                .get("index")
                .and_then(Value::as_u64)
                .and_then(|i| usize::try_from(i).ok()),
        }]),
        "update" => looked(node, args).map(|update| vec![update]),
        _ => None,
    }
}

/// The update of the look of a window that another client asks for
/// (D58): opacity, effects and meta, without the meta key of tiling. No
/// relayout sends opacity or effects, so the look stays across layouts,
/// tags and restarts with resume, and wins over the style.
fn looked(node: &str, args: &Value) -> Option<Command> {
    let asked: PropsUpdate = serde_json::from_value(args.clone()).ok()?;
    let mut meta = asked.meta;
    if let Some(meta) = &mut meta {
        meta.remove(WINDOW_KEY);
    }
    let props = PropsUpdate {
        opacity: asked.opacity,
        effects: asked.effects,
        meta: meta.filter(|m| !m.is_empty()),
        ..PropsUpdate::default()
    };
    (props != PropsUpdate::default()).then(|| Command::Update {
        id: node.into(),
        props,
    })
}

/// The update that gives a new window a style.
fn styled(style: &config::Style, surface: &str) -> Command {
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

/// The layout values that `set_layout` accepts.
fn layout_schema() -> Value {
    json!({
        "enum": [
            "tile", "tile_left", "tile_bottom", "tile_top", "fair",
            "fair_horizontal", "spiral", "dwindle", "max", "fullscreen",
            "magnifier", "floating",
        ],
        "description": "The layout of the current tag.",
    })
}

/// The methods that other clients call, with description and schema of
/// their parameters, as `methods.list` answers them.
fn exported_methods() -> Vec<ExportMethod> {
    let field = |ty: &str, description: &str| json!({"type": ty, "description": description});
    let window = || field("string", "The surface id of the window.");
    let tag = || field("integer", "The tag number, from 1.");
    let on_surface = json!({"surface": window()});
    vec![
        runtime::method(
            "set_layout",
            "Sets the layout and the gap of the current tag.",
            json!({
                "layout": layout_schema(),
                "gap": field("integer", "The gap in logical pixels."),
            }),
            &[],
        ),
        runtime::method(
            "get_layout",
            "Answers the layout of the current tag.",
            json!({}),
            &[],
        ),
        runtime::method(
            "get_tags",
            "Answers the tags of each output: shown, focused, occupied and urgent.",
            json!({}),
            &[],
        ),
        give::method(),
        runtime::method(
            "view_tag",
            "Shows a tag on the current output.",
            json!({"tag": tag()}),
            &["tag"],
        ),
        runtime::method(
            "move_to_tag",
            "Moves a window to a tag.",
            json!({"surface": window(), "tag": tag()}),
            &["surface", "tag"],
        ),
        runtime::method(
            "focus",
            "Focuses a window.",
            on_surface.clone(),
            &["surface"],
        ),
        runtime::method("focus_next", "Focuses the next window.", json!({}), &[]),
        runtime::method("focus_prev", "Focuses the previous window.", json!({}), &[]),
        runtime::method(
            "swap_master",
            "Swaps a window into the master slot.",
            on_surface,
            &["surface"],
        ),
    ]
}

/// Whether a rect `[x, y, w, h]` holds a point.
fn inside([x, y, w, h]: [f64; 4], [px, py]: [f64; 2]) -> bool {
    (x..x + w).contains(&px) && (y..y + h).contains(&py)
}

/// Fails the method unless the model accepted it.
fn only(ok: bool, err: impl FnOnce() -> String) -> Result<(), RpcError> {
    ok.then_some(())
        .ok_or_else(|| RpcError::new(RpcError::REJECTED, err()))
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

    fn tiler(windows: &[&str]) -> Wm {
        let mut wm = Wm::new(settings::parse(crate::config::DEFAULT).unwrap());
        wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        for id in windows {
            wm.add_window(id);
        }
        wm
    }

    #[test]
    fn tiling_answers_a_requested_focus_with_the_model() {
        let mut tiler = tiler(&["s1", "s2"]);
        let commands = answers(&mut tiler, "s1", "focus", &Value::Null).expect("focus answers");
        assert_eq!(commands, Vec::<Command>::new());
        assert_eq!(tiler.focused(), Some("s1"));
    }

    #[test]
    fn tiling_answers_a_requested_move_and_close() {
        let mut tiler = tiler(&["s1"]);
        let args = serde_json::json!({"parent": "other", "index": 2});
        let commands = answers(&mut tiler, "s1", "move", &args).expect("move answers");
        assert!(
            matches!(&commands[..], [Command::Move { id, parent, index }]
                if id == "s1" && parent.as_deref() == Some("other") && *index == Some(2))
        );
        let closed = answers(&mut tiler, "s1", "close", &Value::Null).expect("close answers");
        assert!(
            matches!(&closed[..], [Command::Close { surface }] if surface == "s1"),
            "{closed:?}"
        );
    }

    #[test]
    fn a_window_leaves_tiling_only_after_an_applied_move() {
        let to = |parent| serde_json::json!({"parent": parent});
        assert!(leaves_tiling(true, "move", &to("other")));
        assert!(
            !leaves_tiling(true, "move", &to(NAME)),
            "a move inside tiling keeps the window"
        );
        assert!(
            !leaves_tiling(false, "move", &to("other")),
            "a rejected move keeps the window managed"
        );
        assert!(!leaves_tiling(true, "focus", &to("other")));
    }

    #[test]
    fn a_new_window_gets_the_style_of_the_settings() {
        let mut style = settings::parse::<Config>(config::DEFAULT).unwrap().style;
        style.opacity = 0.5;
        assert!(matches!(
            styled(&style, "s1"),
            Command::Update { id, props }
                if id == "s1" && props.opacity == Some(0.5) && props.effects == Some(Vec::new())
        ));
    }

    #[test]
    fn a_requested_look_applies_without_the_meta_of_tiling() {
        let mut tiler = tiler(&["s1"]);
        let args = serde_json::json!({"opacity": 0.5, "meta": {WINDOW_KEY: "x", "role": "a"}});
        let commands = answers(&mut tiler, "s1", "update", &args).expect("update answers");
        assert!(matches!(
            &commands[..],
            [Command::Update { id, props }] if id == "s1"
                && props.opacity == Some(0.5)
                && props.meta == Some(BTreeMap::from([("role".into(), "a".into())]))
        ));
        let own = serde_json::json!({"meta": {WINDOW_KEY: "x"}});
        assert!(answers(&mut tiler, "s1", "update", &own).is_none());
        // A relayout sends no look, so it keeps the requested one.
        let placed = Placement {
            id: "s1".into(),
            at: Some([0, 0]),
            size: Some([10, 10]),
            states: States::default(),
        };
        assert!(changes(None, &placed).iter().all(|c| !matches!(c,
            Command::Update { props, .. } if props.opacity.is_some() || props.effects.is_some())));
    }

    #[test]
    fn a_requested_resize_stays_unanswered() {
        let mut tiler = tiler(&["s1"]);
        assert!(answers(&mut tiler, "s1", "resize", &Value::Null).is_none());
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
