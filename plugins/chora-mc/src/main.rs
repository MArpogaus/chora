//! chora-mc, experimental: a voice-first desktop without window
//! management. The whole screen shows one view, a grid of text panels or
//! one app, and a quiet HUD line shows what the voice agent does. The
//! agent and small collector scripts fill the panels through the exported
//! methods; the plugin reads no sources itself.

mod config;
mod view;

use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    process::ExitCode,
};

use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Easing, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, ToplevelState, Transform,
    ipc::{ClientError, ExportMethod, PressState, Query, RpcError},
};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config::{Action, Config},
    view::{Panel, View},
};

/// The client name, and the id of the group that holds the views.
const NAME: &str = "mc";
/// The id of the line of the HUD.
const HUD: &str = "mc-hud";
/// The cross-fade between two views, in milliseconds.
const FADE: f64 = 250.0;

/// One view at a time on the whole screen, driven by voice.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The settings file. Without it, chora-mc reads
    /// $XDG_CONFIG_HOME/chora/chora-mc.yaml if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Print the default settings and exit.
    #[arg(long)]
    print_config: bool,
    /// The directory of the views; `$XDG_CONFIG_HOME/chora/mc/views`
    /// without it.
    #[arg(long)]
    views: Option<PathBuf>,
}

/// Parameters of `view.show`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Show {
    name: String,
}

/// Parameters of `view.save`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    name: String,
    #[serde(flatten)]
    view: View,
}

/// Parameters of `panel.set`; a missing title keeps the title.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetPanel {
    view: String,
    panel: String,
    title: Option<String>,
    lines: Vec<String>,
}

/// Parameters of `hud.set`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetHud {
    #[serde(default)]
    state: String,
    #[serde(default)]
    text: String,
}

/// Parameters of `app.open`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Open {
    command: String,
}

struct Mc {
    rt: Runtime<Action>,
    settings: settings::File<Config>,
    /// The directory of the view files.
    dir: PathBuf,
    views: BTreeMap<String, View>,
    /// The window of an app view, by view.
    windows: HashMap<String, String>,
    /// The view of an app that starts, by its activation token.
    starting: HashMap<String, String>,
    current: Option<String>,
    /// The one output that the plugin fills.
    output: Option<OutputInfo>,
    /// Whether the nodes exist.
    built: bool,
    hud: SetHud,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if args.print_config {
        print!("{}", config::DEFAULT);
        return ExitCode::SUCCESS;
    }
    runtime::run(NAME, || play(&args))
}

fn play(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-mc")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let dir = args
        .views
        .clone()
        .or_else(|| chora_kit::xdg::config_home().map(|d| d.join("chora/mc/views")))
        .ok_or("no $XDG_CONFIG_HOME or $HOME")?;
    let mut methods = exported_methods();
    methods.extend(settings::File::<Config>::methods());
    let rt = Runtime::start(NAME, Vec::new(), &EVENTS, methods)?;
    let mut mc = Mc {
        rt,
        settings,
        views: view::load(&dir),
        dir,
        windows: HashMap::new(),
        starting: HashMap::new(),
        current: None,
        output: None,
        built: false,
        hud: SetHud {
            state: String::new(),
            text: String::new(),
        },
    };
    mc.start()?;
    Ok(runtime::serve(&mut mc)?)
}

const EVENTS: [&str; 5] = [
    "surface.created",
    "surface.destroyed",
    "surface.detached",
    "output.changed",
    "output.removed",
];

fn exported_methods() -> Vec<ExportMethod> {
    let lines = json!({"type": "array", "items": {"type": "string"}});
    let panels = json!({"type": "array", "items": {"type": "object"}, "description":
        "each with id, cell [column, row, width, height], title and lines"});
    vec![
        runtime::method(
            "view.show",
            "Shows a view with a short cross-fade.",
            json!({"name": {"type": "string"}}),
            &["name"],
        ),
        runtime::method(
            "view.list",
            "Answers the views and the one that shows.",
            json!({}),
            &[],
        ),
        runtime::method(
            "view.save",
            "Saves a view in its file and shows the change: a grid of panels, or an app.",
            json!({
                "name": {"type": "string"},
                "grid": {"type": "array", "items": {"type": "integer"}, "description": "columns and rows"},
                "panels": panels,
                "app": {"type": "string", "description": "a shell command whose window fills the screen"},
            }),
            &["name"],
        ),
        runtime::method(
            "panel.set",
            "Sets the title and the lines of a panel of a view, until the next start.",
            json!({
                "view": {"type": "string"},
                "panel": {"type": "string"},
                "title": {"type": "string"},
                "lines": lines,
            }),
            &["view", "panel", "lines"],
        ),
        runtime::method(
            "hud.set",
            "Sets the HUD line: the state of the agent, such as listening, thinking or \
             working, and a short message; both empty hide it.",
            json!({"state": {"type": "string"}, "text": {"type": "string"}}),
            &[],
        ),
        runtime::method(
            "app.open",
            "Starts a shell command as an app in a full-screen view of its own.",
            json!({"command": {"type": "string"}}),
            &["command"],
        ),
    ]
}

impl Plugin for Mc {
    type Action = Action;

    fn runtime(&mut self) -> &mut Runtime<Action> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::SurfaceCreated { to: Some(to), .. } if to != NAME => Ok(()),
            Event::SurfaceCreated {
                surface,
                app_id,
                token,
                ..
            } => self.adopt(&surface, app_id.as_deref(), token.as_deref()),
            Event::SurfaceDetached { surface } => self.adopt(&surface, None, None),
            Event::SurfaceDestroyed { surface } => self.forget(&surface),
            Event::OutputChanged { output }
                if self.output.as_ref().is_none_or(|o| o.name == output.name) =>
            {
                self.output = Some(output);
                self.build()
            }
            Event::OutputRemoved { name }
                if self.output.as_ref().is_some_and(|o| o.name == name) =>
            {
                self.output = None;
                Ok(())
            }
            Event::InputKey {
                binding,
                state: PressState::Pressed,
            } => {
                self.act(&binding);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        match method {
            "view.show" => {
                let Show { name } = runtime::params(params)?;
                self.show(&name)?;
                Ok(Value::Null)
            }
            "view.list" => Ok(json!({
                "views": self.views.keys().collect::<Vec<_>>(),
                "current": self.current,
            })),
            "view.save" => {
                let Save { name, view } = runtime::params(params)?;
                view::save(&self.dir, &name, &view).map_err(invalid)?;
                self.views.insert(name.clone(), view);
                self.build().map_err(rejected)?;
                Ok(Value::Null)
            }
            "panel.set" => {
                let set: SetPanel = runtime::params(params)?;
                self.set_panel(set)?;
                Ok(Value::Null)
            }
            "hud.set" => {
                self.hud = runtime::params(params)?;
                let update = self.hud_update();
                self.apply(vec![update]).map_err(rejected)?;
                Ok(Value::Null)
            }
            "app.open" => {
                let Open { command } = runtime::params(params)?;
                let name = self.app_view(&command)?;
                self.show(&name)?;
                Ok(json!({"view": name}))
            }
            _ => Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.rt
            .ungrab_all()
            .and_then(|()| self.grab_all())
            .and_then(|()| self.build())
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

fn invalid(message: String) -> RpcError {
    RpcError::new(RpcError::INVALID_PARAMS, message)
}

/// The id of the group of a view.
fn view_id(name: &str) -> String {
    format!("mc-v-{name}")
}

/// The id of the group of the panels of a view.
fn panels_id(name: &str) -> String {
    format!("mc-v-{name}-panels")
}

/// A name for a view from an app id or a command: its last path part,
/// with the characters that a name cannot have as `-`.
fn name_of(word: &str) -> String {
    let base = word.rsplit('/').next().unwrap_or(word);
    let name: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-".contains(c) {
                c
            } else {
                '-'
            }
        })
        .take(48)
        .collect();
    let name = name.trim_start_matches('.');
    if name.is_empty() {
        "app".into()
    } else {
        name.into()
    }
}

fn create(id: &str, kind: NodeKind, parent: &str, props: PropsUpdate) -> Command {
    Command::Create {
        id: id.into(),
        kind,
        parent: parent.into(),
        props,
    }
}

fn translate(x: f64, y: f64) -> Transform {
    Transform::translate(x, y, 0.0)
}

impl Mc {
    fn start(&mut self) -> Result<(), ClientError> {
        self.output = self.rt.outputs()?.into_iter().next();
        // The nodes that the core kept over a restart: the windows in them
        // become free and come back as views of their own.
        let kept: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new(format!("owner:{NAME}")))?;
        let old = kept
            .iter()
            .filter_map(|n| n["id"].as_str())
            .filter(|id| *id == NAME || *id == HUD)
            .map(|id| Command::Remove {
                id: id.into(),
                to: None,
            })
            .collect();
        self.apply(old)?;
        self.grab_all()?;
        self.build()?;
        let first = self.views.keys().next().cloned();
        let default = Some(self.settings.value.default_view.clone())
            .filter(|v| self.views.contains_key(v))
            .or(first);
        if let Some(name) = default
            && let Err(err) = self.show(&name)
        {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        let free: Vec<Value> = self
            .rt
            .conn
            .call("scene.query", Query::new("kind:surface"))?;
        for node in free {
            let ours = node["to"].is_null() || node["to"] == NAME;
            if let (Some(id), true, true) = (node["id"].as_str(), node["owner"].is_null(), ours) {
                let app_id = node["surface"]["app_id"].as_str();
                self.adopt(id, app_id, None)?;
            }
        }
        Ok(())
    }

    /// Grabs the bindings of the settings; one that another client holds
    /// only logs.
    fn grab_all(&mut self) -> Result<(), ClientError> {
        for (text, action) in self.settings.value.bindings.clone() {
            let binding = text.parse().expect("the settings check their bindings");
            self.rt.grab(binding, action)?;
        }
        Ok(())
    }

    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        if commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.apply(commands)
    }

    /// Builds all nodes again, for example after a change of the theme or
    /// of the output. The windows wait under the root meanwhile, all in one
    /// transaction, so nothing flickers.
    fn build(&mut self) -> Result<(), ClientError> {
        let Some(output) = self.output.clone() else {
            return Ok(());
        };
        let mut commands = if self.built {
            self.unbuild()
        } else {
            Vec::new()
        };
        let [w, h] = output.size.map(f64::from);
        let [x, y] = output.position.map(f64::from);
        let colors = self.settings.value.colors().clone();
        commands.push(create(
            NAME,
            NodeKind::Group,
            ROOT,
            PropsUpdate {
                transform: Some(translate(x, y)),
                ..PropsUpdate::default()
            },
        ));
        commands.push(create(
            "mc-bg",
            NodeKind::Rect,
            NAME,
            PropsUpdate {
                size: Some([w, h]),
                color: Some(colors.background),
                ..PropsUpdate::default()
            },
        ));
        for name in self.views.keys() {
            let shown = self.current.as_ref() == Some(name);
            commands.push(create(
                &view_id(name),
                NodeKind::Group,
                NAME,
                PropsUpdate {
                    visible: Some(shown),
                    opacity: Some(if shown { 1.0 } else { 0.0 }),
                    ..PropsUpdate::default()
                },
            ));
            commands.extend(self.panels(name));
            if let Some(surface) = self.windows.get(name) {
                commands.extend(self.place_window(name, surface, false));
            }
        }
        commands.push(create(
            HUD,
            NodeKind::Text,
            &format!("screen:{}", output.name),
            self.hud_props(),
        ));
        self.apply(commands)?;
        self.built = true;
        Ok(())
    }

    /// Removes all nodes; the windows wait under the root.
    fn unbuild(&self) -> Vec<Command> {
        let moves = self.windows.values().map(|surface| Command::Move {
            id: surface.clone(),
            parent: Some(ROOT.into()),
            index: None,
        });
        let removes = [NAME, HUD].map(|id| Command::Remove {
            id: id.into(),
            to: None,
        });
        moves.chain(removes).collect()
    }

    /// Creates the panels of a view, under a group of their own that the
    /// next call replaces.
    fn panels(&self, name: &str) -> Vec<Command> {
        let (Some(output), Some(view)) = (&self.output, self.views.get(name)) else {
            return Vec::new();
        };
        let config = &self.settings.value;
        let colors = config.colors();
        let size = config.font.size;
        let margin = size * 3.0;
        let group = panels_id(name);
        let mut commands = vec![create(
            &group,
            NodeKind::Group,
            &view_id(name),
            PropsUpdate::default(),
        )];
        for panel in &view.panels {
            let [x, y, w, h] =
                view::place(panel.cell, view.grid, output.size.map(f64::from), margin);
            let id = format!("{group}-{}", panel.id);
            commands.push(create(
                &id,
                NodeKind::Group,
                &group,
                PropsUpdate {
                    transform: Some(translate(x, y)),
                    size: Some([w, h]),
                    clip: Some(Some(chora_scene::Clip { radius: 0.0 })),
                    ..PropsUpdate::default()
                },
            ));
            let text = |text: String, font_size: f64, color: [f64; 4], top: f64| PropsUpdate {
                text: Some(text),
                family: Some(config.font.family.clone()),
                font_size: Some(font_size),
                max_width: Some(w),
                color: Some(color),
                transform: Some(translate(0.0, top)),
                ..PropsUpdate::default()
            };
            let title_size = size * 1.5;
            if !panel.title.is_empty() {
                let props = text(panel.title.clone(), title_size, colors.accent, 0.0);
                commands.push(create(&format!("{id}-title"), NodeKind::Text, &id, props));
            }
            if !panel.lines.is_empty() {
                let props = text(panel.text(), size, colors.text, title_size * 2.0);
                commands.push(create(&format!("{id}-lines"), NodeKind::Text, &id, props));
            }
        }
        commands
    }

    /// Puts a window into its view at the size of the screen, without a
    /// frame.
    fn place_window(&self, name: &str, surface: &str, attach: bool) -> Vec<Command> {
        let Some(output) = &self.output else {
            return Vec::new();
        };
        let parent = view_id(name);
        let put = if attach {
            Command::Attach {
                surface: surface.into(),
                parent,
            }
        } else {
            Command::Move {
                id: surface.into(),
                parent: Some(parent),
                index: None,
            }
        };
        vec![
            put,
            Command::Configure {
                surface: surface.into(),
                size: Some(output.size),
                states: vec![ToplevelState::Fullscreen],
            },
        ]
    }

    fn hud_text(&self) -> String {
        let SetHud { state, text } = &self.hud;
        match (state.is_empty(), text.is_empty()) {
            (false, false) => format!("{state} · {text}"),
            (false, true) => state.clone(),
            _ => text.clone(),
        }
    }

    fn hud_props(&self) -> PropsUpdate {
        let config = &self.settings.value;
        let text = self.hud_text();
        let size = config.font.size;
        let height = self.output.as_ref().map_or(0.0, |o| f64::from(o.size[1]));
        PropsUpdate {
            visible: Some(!text.is_empty()),
            // A text node needs a text.
            text: Some(if text.is_empty() { " ".into() } else { text }),
            family: Some(config.font.family.clone()),
            font_size: Some(size * 0.8),
            color: Some(config.colors().accent),
            transform: Some(translate(size * 3.0, height - size * 2.0)),
            ..PropsUpdate::default()
        }
    }

    fn hud_update(&self) -> Command {
        Command::Update {
            id: HUD.into(),
            props: self.hud_props(),
        }
    }

    fn show(&mut self, name: &str) -> Result<(), RpcError> {
        let Some(view) = self.views.get(name) else {
            return Err(invalid(format!("no view {name}")));
        };
        let app = view.app.clone();
        let mut commands = Vec::new();
        if self.current.as_deref() != Some(name) {
            if let Some(old) = &self.current {
                commands.push(Command::Animate {
                    id: view_id(old),
                    props: PropsUpdate {
                        opacity: Some(0.0),
                        visible: Some(false),
                        ..PropsUpdate::default()
                    },
                    duration: FADE,
                    easing: Easing::EaseInOut,
                });
            }
            commands.push(Command::Update {
                id: view_id(name),
                props: PropsUpdate {
                    visible: Some(true),
                    ..PropsUpdate::default()
                },
            });
            commands.push(Command::Animate {
                id: view_id(name),
                props: PropsUpdate {
                    opacity: Some(1.0),
                    ..PropsUpdate::default()
                },
                duration: FADE,
                easing: Easing::EaseInOut,
            });
        }
        let window = self.windows.get(name).cloned();
        commands.push(Command::Focus {
            surface: window.clone(),
        });
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        self.current = Some(name.into());
        let waiting = self.starting.values().any(|v| v == name);
        if let (Some(command), None, false) = (app, window, waiting) {
            self.launch(name, &command);
        }
        Ok(())
    }

    /// Starts the app of a view with `choractl run`, whose activation
    /// token finds its window again.
    fn launch(&mut self, name: &str, command: &str) {
        let run = std::process::Command::new("choractl")
            .args(["run", "--", "sh", "-c", command])
            .stdin(std::process::Stdio::null())
            .output();
        let line = match run {
            Ok(out) if out.status.success() => out.stdout,
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                eprintln!("chora-mc: {command}: {}", err.trim());
                return;
            }
            Err(err) => {
                eprintln!("chora-mc: choractl run: {err}");
                return;
            }
        };
        let answer: Value = serde_json::from_slice(&line).unwrap_or_default();
        if let Some(token) = answer["token"].as_str() {
            self.starting.insert(token.into(), name.into());
        }
    }

    /// The view of `app.open`, named after the program.
    fn app_view(&mut self, command: &str) -> Result<String, RpcError> {
        let word = command
            .split_whitespace()
            .next()
            .ok_or_else(|| invalid("an empty command".into()))?;
        let name = name_of(word);
        match self.views.get(&name) {
            Some(view) if view.app.as_deref() == Some(command) => return Ok(name),
            Some(_) => return Err(invalid(format!("a view {name} exists already"))),
            None => {}
        }
        self.views.insert(name.clone(), View::app(command));
        let group = create(
            &view_id(&name),
            NodeKind::Group,
            NAME,
            PropsUpdate {
                visible: Some(false),
                opacity: Some(0.0),
                ..PropsUpdate::default()
            },
        );
        self.apply(vec![group])
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        Ok(name)
    }

    /// Takes a new window into the view that started it, or into a new
    /// view of its own, and shows it.
    fn adopt(
        &mut self,
        surface: &str,
        app_id: Option<&str>,
        token: Option<&str>,
    ) -> Result<(), ClientError> {
        if !self.built {
            return Ok(());
        }
        let started = token.and_then(|t| self.starting.remove(t));
        let mut commands = Vec::new();
        let name = match started {
            Some(name) if !self.windows.contains_key(&name) => name,
            _ => {
                let mut name = name_of(app_id.unwrap_or("app"));
                if self.views.contains_key(&name) {
                    name = format!("{name}-{surface}");
                }
                self.views.insert(name.clone(), View::default());
                commands.push(create(
                    &view_id(&name),
                    NodeKind::Group,
                    NAME,
                    PropsUpdate {
                        visible: Some(false),
                        opacity: Some(0.0),
                        ..PropsUpdate::default()
                    },
                ));
                name
            }
        };
        commands.extend(self.place_window(&name, surface, true));
        self.apply(commands)?;
        self.windows.insert(name.clone(), surface.into());
        if let Err(err) = self.show(&name) {
            eprintln!("chora-mc: {name}: {}", err.message);
        }
        Ok(())
    }

    /// A window closed: a view that only held it goes, and the default
    /// view shows instead of a view without its window.
    fn forget(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some(name) = self
            .windows
            .iter()
            .find(|(_, s)| *s == surface)
            .map(|(n, _)| n.clone())
        else {
            return Ok(());
        };
        self.windows.remove(&name);
        if self.current.as_deref() == Some(&name) {
            let default = self.settings.value.default_view.clone();
            if let Err(err) = self.show(&default) {
                eprintln!("chora-mc: {default}: {}", err.message);
            }
        }
        if self.views[&name].app.is_none() && self.views[&name].panels.is_empty() {
            self.views.remove(&name);
            self.apply(vec![Command::Remove {
                id: view_id(&name),
                to: None,
            }])?;
        }
        Ok(())
    }

    fn set_panel(&mut self, set: SetPanel) -> Result<(), RpcError> {
        let SetPanel {
            view,
            panel,
            title,
            lines,
        } = set;
        let found = self
            .views
            .get_mut(&view)
            .and_then(|v| v.panels.iter_mut().find(|p| p.id == panel))
            .ok_or_else(|| invalid(format!("no panel {panel} in view {view}")))?;
        let new = Panel {
            title: title.unwrap_or_else(|| found.title.clone()),
            lines,
            ..found.clone()
        };
        new.check_text().map_err(invalid)?;
        *found = new;
        let mut commands = vec![Command::Remove {
            id: panels_id(&view),
            to: None,
        }];
        commands.extend(self.panels(&view));
        self.apply(commands)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }

    fn act(&mut self, binding: &str) {
        let Some(action) = self.rt.action(binding).cloned() else {
            return;
        };
        let names: Vec<String> = self.views.keys().cloned().collect();
        let at = self
            .current
            .as_ref()
            .and_then(|c| names.iter().position(|n| n == c))
            .unwrap_or(0);
        let next = match action {
            Action::NextView => names[(at + 1) % names.len()].clone(),
            Action::PreviousView => names[(at + names.len() - 1) % names.len()].clone(),
            Action::View(name) => name,
            Action::Run(command) => {
                chora_kit::run(&command);
                return;
            }
        };
        if let Err(err) = self.show(&next) {
            eprintln!("chora-mc: {next}: {}", err.message);
        }
    }
}
