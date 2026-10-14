//! chora-tiling: a tiling plugin with the user experience of awesome. Each
//! output has tags, each tag a layout, and the bindings come from a YAML
//! file (see `tiling.yaml`). Other clients, for example an agent, switch
//! the layout through the exported methods `tiling.set_layout` and
//! `tiling.get_layout`.

mod config;
mod layout;
mod wm;

use std::{collections::HashMap, path::PathBuf, process::ExitCode};

use chora_scene::{
    Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceRequest, ToplevelState,
    Transform,
    ipc::{
        Apply, Client, ClientError, Export, Grab, Hello, PressState, Query, RpcError, Subscribe,
    },
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    config::{Action, Config},
    layout::Layout,
    wm::{Effect, Placement, Wm},
};

/// The client name, and the group that holds the windows.
const NAME: &str = "tiling";

/// Tile the windows of chora with tags and layouts in the style of awesome.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The configuration file. Without it, chora-tiling reads
    /// ~/.config/chora/tiling.yaml if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default configuration and exit.
    #[arg(long)]
    print_config: bool,
}

/// Parameters of `tiling.set_layout`; a missing field keeps its value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetLayout {
    layout: Option<Layout>,
    gap: Option<u16>,
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
    client: Client,
    wm: Wm,
    /// The configuration file, if there is one.
    path: Option<PathBuf>,
    /// The grabbed bindings, as the core names them.
    grabs: Vec<(String, Action)>,
    /// What the last transaction set, so that the next one sends changes
    /// only.
    sent: HashMap<String, Placement>,
    order: Vec<String>,
    focus: Option<String>,
    /// Whether the core has `focus`.
    focus_sent: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
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
fn config_path(arg: Option<PathBuf>) -> Option<PathBuf> {
    arg.or_else(|| {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("chora/tiling.yaml")).filter(|p| p.exists())
    })
}

fn load(path: Option<&PathBuf>) -> Result<Config, String> {
    path.map_or_else(|| Config::parse(config::DEFAULT), |p| Config::read(p))
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path(args.config);
    let config = load(path.as_ref())?;
    let hello = Hello {
        name: NAME.into(),
        keep_nodes: false,
    };
    let mut tiling = Tiling {
        client: Client::from_env(&hello)?,
        wm: Wm::new(config),
        path,
        grabs: Vec::new(),
        sent: HashMap::new(),
        order: Vec::new(),
        focus: None,
        focus_sent: false,
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
    fn start(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.client.call("output.list", Value::Null)?;
        for output in &outputs {
            self.wm
                .set_output(&output.name, output.layout_area(), full_area(output));
        }
        if self.query(&format!("id:{NAME}"))?.is_empty() {
            let group = Command::Create {
                id: NAME.into(),
                kind: NodeKind::Group,
                parent: ROOT.into(),
                props: PropsUpdate::default(),
            };
            self.call_apply(vec![group])?;
        }
        self.grab_all()?;
        let names = ["set_layout", "get_layout"].map(String::from).to_vec();
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
        ];
        self.client.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: events.map(String::from).to_vec(),
            },
        )?;
        for node in self.query("kind:surface")? {
            if let (Some(id), None) = (node["id"].as_str(), node["owner"].as_str()) {
                self.adopt(id)?;
            }
        }
        self.relayout()
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
        let config = match load(self.path.as_ref()) {
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
            // A detached surface has no owner any more, so it is free to take.
            Event::SurfaceCreated { surface, .. } | Event::SurfaceDetached { surface } => {
                self.wm.remove_window(&surface);
                self.adopt(&surface)?;
            }
            Event::SurfaceDestroyed { surface } => {
                self.wm.remove_window(&surface);
                self.sent.remove(&surface);
            }
            Event::SurfaceChanged {
                surface,
                size: Some(size),
                ..
            } if self.wm.set_size(&surface, size) => {}
            Event::OutputChanged { output } => {
                self.wm
                    .set_output(&output.name, output.layout_area(), full_area(&output));
            }
            Event::OutputRemoved { name } => self.wm.remove_output(&name),
            Event::InputKey {
                binding,
                state: PressState::Pressed,
            } => return self.act(&binding),
            Event::PointerEntered { node, .. } if self.wm.pointer_focus(&node, false) => {}
            Event::PointerPressed { node, .. } if self.wm.pointer_focus(&node, true) => {}
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
            } => {
                let result = self.method(&method, params);
                // The call may have ended before the answer, for example
                // when its caller left; that is no reason to exit.
                return match self.client.answer(call, result) {
                    Err(ClientError::Rpc(err)) => {
                        eprintln!("chora-tiling: {err}");
                        Ok(())
                    }
                    other => other,
                };
            }
            _ => return Ok(()),
        }
        self.relayout()
    }

    /// Runs an exported method. A failed relayout fails the method.
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
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

    fn act(&mut self, binding: &str) -> Result<(), ClientError> {
        let Some((_, action)) = self.grabs.iter().find(|(b, _)| b == binding) else {
            return Ok(());
        };
        match self.wm.act(&action.clone()) {
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

    /// Takes a surface without an owner. A surface that another client took
    /// first stays out of the layout.
    fn adopt(&mut self, surface: &str) -> Result<(), ClientError> {
        let attach = Command::Attach {
            surface: surface.into(),
            parent: NAME.into(),
        };
        match self.call_apply(vec![attach]) {
            Ok(()) => {
                self.wm.add_window(surface);
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
        if !self.focus_sent || self.focus != focus {
            commands.push(Command::Focus {
                surface: focus.clone(),
            });
        }
        if commands.is_empty() {
            return Ok(());
        }
        self.sent = placements.into_iter().map(|p| (p.id.clone(), p)).collect();
        (self.order, self.focus, self.focus_sent) = (order, focus, true);
        self.apply(commands)
    }

    /// Forgets what the core shows, so that the next transaction sends all.
    fn forget(&mut self) {
        self.sent.clear();
        self.order.clear();
        self.focus_sent = false;
    }

    fn call_apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        self.client
            .call::<_, Value>("scene.apply", Apply { commands })
            .map(drop)
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
        let hide = Command::Update {
            id: id.clone(),
            props: PropsUpdate {
                visible: Some(false),
                ..PropsUpdate::default()
            },
        };
        return if sent.is_some_and(|s| s.at.is_none()) {
            Vec::new()
        } else {
            vec![hide]
        };
    };
    let mut commands = Vec::new();
    if sent.is_none_or(|s| s.size != placement.size || s.states != placement.states) {
        let states = placement.states;
        let states = [
            (states.tiled, ToplevelState::Tiled),
            (states.maximized, ToplevelState::Maximized),
            (states.fullscreen, ToplevelState::Fullscreen),
            (states.activated, ToplevelState::Activated),
        ];
        commands.push(Command::Configure {
            surface: id.clone(),
            size: placement.size,
            states: states
                .into_iter()
                .filter(|(on, _)| *on)
                .map(|(_, s)| s)
                .collect(),
        });
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
