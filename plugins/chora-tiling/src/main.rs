//! Reference tiling plugin for chora: a master and stack layout, columns
//! or rows, with keybindings to start a terminal and a launcher, close a
//! window and move the focus. Other clients, for example an agent, switch
//! the layout through the exported methods `tiling.set_layout` and
//! `tiling.get_layout`.

use std::process::ExitCode;

use chora_scene::{
    Binding, Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, SurfaceRequest,
    ToplevelState, Transform,
    ipc::{Apply, Client, ClientError, Export, Grab, Hello, Query, RpcError, Subscribe},
};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The client name, and the group that holds the tiled surfaces.
const NAME: &str = "tiling";

/// Tile the windows of chora in a master and stack layout.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The modifier of all bindings.
    #[arg(long = "mod", value_enum, default_value_t = Modifier::Super)]
    modifier: Modifier,
    /// The shell command that MOD+Return runs, for example "foot -e htop".
    #[arg(long, default_value = "foot")]
    terminal: String,
    /// The shell command that MOD+d runs.
    #[arg(long, default_value = "fuzzel")]
    launcher: String,
    /// The gap around each window, in logical pixels.
    #[arg(long, default_value_t = 4)]
    gap: u16,
    /// The layout at the start.
    #[arg(long, value_enum, default_value_t = Layout::MasterStack)]
    layout: Layout,
}

/// How the windows share the work area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Layout {
    /// The first window takes the left half, the others share the right
    /// half.
    MasterStack,
    /// The windows stand side by side.
    Columns,
    /// The windows lie on top of each other.
    Rows,
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
struct LayoutInfo<'a> {
    layout: Layout,
    gap: u16,
    /// The tiled surfaces, the master first.
    windows: &'a [String],
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Modifier {
    Super,
    Alt,
}

/// What a binding does.
#[derive(Clone, Copy)]
enum Action {
    Terminal,
    Launcher,
    Close,
    FocusNext,
    FocusPrevious,
}

struct Tiling {
    client: Client,
    args: Args,
    bindings: Vec<(String, Action)>,
    area: [i32; 4],
    /// The tiled surfaces; the first one is the master.
    windows: Vec<String>,
    focused: Option<String>,
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chora-tiling: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), ClientError> {
    let hello = Hello {
        name: NAME.into(),
        keep_nodes: false,
    };
    let mut tiling = Tiling {
        client: Client::from_env(&hello)?,
        args,
        bindings: Vec::new(),
        area: [0; 4],
        windows: Vec::new(),
        focused: None,
    };
    tiling.start()?;
    loop {
        let event = tiling.client.next_event()?;
        tiling.handle(event)?;
    }
}

impl Tiling {
    fn start(&mut self) -> Result<(), ClientError> {
        let outputs: Vec<OutputInfo> = self.client.call("output.list", Value::Null)?;
        self.area = outputs.first().map_or([0; 4], area_of);
        if self.query(&format!("id:{NAME}"))?.is_empty() {
            let group = Command::Create {
                id: NAME.into(),
                kind: NodeKind::Group,
                parent: ROOT.into(),
                props: PropsUpdate::default(),
            };
            self.client.call::<_, Value>(
                "scene.apply",
                Apply {
                    commands: vec![group],
                },
            )?;
        }
        self.grab_all()?;
        let names = ["set_layout", "get_layout"].map(String::from).to_vec();
        self.client
            .call::<_, Value>("methods.export", Export { names })?;
        let events = [
            "surface.created",
            "surface.destroyed",
            "surface.detached",
            "surface.requested",
            "output.changed",
        ];
        self.client.call::<_, Value>(
            "events.subscribe",
            Subscribe {
                events: events.map(String::from).to_vec(),
            },
        )?;
        for node in self.query("kind:surface")? {
            let Some(id) = node["id"].as_str() else {
                continue;
            };
            match node["owner"].as_str() {
                None => self.adopt(id)?,
                Some(NAME) => self.windows.push(id.into()),
                Some(_) => {}
            }
        }
        self.focused = self.windows.last().cloned();
        self.relayout()
    }

    fn grab_all(&mut self) -> Result<(), ClientError> {
        for (key, action) in [
            ("Return", Action::Terminal),
            ("d", Action::Launcher),
            ("q", Action::Close),
            ("j", Action::FocusNext),
            ("k", Action::FocusPrevious),
        ] {
            let binding: Binding = format!("{:?}+{key}", self.args.modifier)
                .parse()
                .expect("valid bindings");
            let normal: String = self.client.call("input.grab", Grab { binding })?;
            self.bindings.push((normal, action));
        }
        Ok(())
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
                self.windows.retain(|w| *w != surface);
                self.adopt(&surface)?;
                self.relayout()
            }
            Event::SurfaceDestroyed { surface } => {
                self.windows.retain(|w| *w != surface);
                if self.focused.as_ref() == Some(&surface) {
                    self.focused = self.windows.last().cloned();
                }
                self.relayout()
            }
            Event::OutputChanged { output } => {
                self.area = area_of(&output);
                self.relayout()
            }
            Event::InputKey { binding } => self.act(&binding),
            // For example a click on a notification of the window.
            Event::SurfaceRequested {
                surface,
                request: SurfaceRequest::Activate,
            } if self.windows.contains(&surface) => {
                self.focused = Some(surface);
                self.relayout()
            }
            // For example a click on the close button of a taskbar.
            Event::SurfaceRequested {
                surface,
                request: SurfaceRequest::Close,
            } if self.windows.contains(&surface) => self.apply(vec![Command::Close { surface }]),
            Event::MethodCalled {
                call,
                method,
                params,
                ..
            } => {
                let result = self.method(&method, params);
                // The call may have ended before the answer, for example
                // when its caller left; that is no reason to exit.
                match self.client.answer(call, result) {
                    Err(ClientError::Rpc(err)) => {
                        eprintln!("chora-tiling: {err}");
                        Ok(())
                    }
                    other => other,
                }
            }
            _ => Ok(()),
        }
    }

    /// Runs an exported method. A failed relayout fails the method.
    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let invalid = |e: String| RpcError::new(RpcError::INVALID_PARAMS, e);
        if method == "set_layout" {
            let change: SetLayout =
                serde_json::from_value(params).map_err(|e| invalid(e.to_string()))?;
            self.args.layout = change.layout.unwrap_or(self.args.layout);
            self.args.gap = change.gap.unwrap_or(self.args.gap);
            self.relayout()
                .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
        }
        let info = LayoutInfo {
            layout: self.args.layout,
            gap: self.args.gap,
            windows: &self.windows,
        };
        Ok(serde_json::to_value(info).expect("layouts always serialize"))
    }

    fn act(&mut self, binding: &str) -> Result<(), ClientError> {
        let Some((_, action)) = self.bindings.iter().find(|(b, _)| b == binding) else {
            return Ok(());
        };
        match action {
            Action::Terminal => {
                spawn(&self.args.terminal);
                return Ok(());
            }
            Action::Launcher => {
                spawn(&self.args.launcher);
                return Ok(());
            }
            Action::Close => {
                if let Some(surface) = self.focused.clone() {
                    self.apply(vec![Command::Close { surface }])?;
                }
                return Ok(());
            }
            Action::FocusNext => self.move_focus(1),
            Action::FocusPrevious => self.move_focus(self.windows.len().saturating_sub(1)),
        }
        self.relayout()
    }

    /// Moves the focus `steps` windows forward, wrapping around.
    fn move_focus(&mut self, steps: usize) {
        let n = self.windows.len();
        if n == 0 {
            return;
        }
        let current = self
            .focused
            .as_ref()
            .and_then(|f| self.windows.iter().position(|w| w == f))
            .unwrap_or(0);
        self.focused = Some(self.windows[(current + steps) % n].clone());
    }

    /// Takes a surface without an owner and focuses it. A surface that
    /// another client took first stays out of the layout.
    fn adopt(&mut self, surface: &str) -> Result<(), ClientError> {
        let attach = Command::Attach {
            surface: surface.into(),
            parent: NAME.into(),
        };
        match self.client.call::<_, Value>(
            "scene.apply",
            Apply {
                commands: vec![attach],
            },
        ) {
            Ok(_) => {
                self.windows.push(surface.into());
                self.focused = Some(surface.into());
                Ok(())
            }
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {surface}: {err}");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Places every window and moves the focus, in one transaction.
    fn relayout(&mut self) -> Result<(), ClientError> {
        let n = self.windows.len();
        let gap = i32::from(self.args.gap);
        let cells = match self.args.layout {
            Layout::MasterStack => master_stack(n, self.area, gap),
            Layout::Columns => split(n, self.area, true)
                .into_iter()
                .map(|c| inset(c, gap))
                .collect(),
            Layout::Rows => split(n, self.area, false)
                .into_iter()
                .map(|c| inset(c, gap))
                .collect(),
        };
        let mut commands = Vec::new();
        for (surface, [x, y, w, h]) in self.windows.iter().zip(cells) {
            let mut states = vec![ToplevelState::Tiled];
            if self.focused.as_ref() == Some(surface) {
                states.push(ToplevelState::Activated);
            }
            commands.push(Command::Configure {
                surface: surface.clone(),
                size: Some([w, h]),
                states,
            });
            commands.push(Command::Update {
                id: surface.clone(),
                props: PropsUpdate {
                    transform: Some(Transform::translate(f64::from(x), f64::from(y), 0.0)),
                    ..PropsUpdate::default()
                },
            });
        }
        commands.push(Command::Focus {
            surface: self.focused.clone(),
        });
        self.apply(commands)
    }

    /// Applies a transaction. A rejected transaction only logs: a surface
    /// can disappear between an event and the answer to it, and the event
    /// that follows corrects the layout.
    fn apply(&mut self, commands: Vec<Command>) -> Result<(), ClientError> {
        match self
            .client
            .call::<_, Value>("scene.apply", Apply { commands })
        {
            Err(ClientError::Rpc(err)) => {
                eprintln!("chora-tiling: {err}");
                Ok(())
            }
            other => other.map(|_| ()),
        }
    }
}

fn area_of(output: &OutputInfo) -> [i32; 4] {
    output.work_area
}

/// The cells of a master and stack layout: the first window takes the left
/// half, the others share the right half. Each cell is x, y, width, height.
fn master_stack(n: usize, [left, top, width, height]: [i32; 4], gap: i32) -> Vec<[i32; 4]> {
    let cells = match n {
        0 => Vec::new(),
        1 => vec![[left, top, width, height]],
        _ => {
            let half = width / 2;
            let mut cells = vec![[left, top, half, height]];
            cells.extend(split(
                n - 1,
                [left + half, top, width - half, height],
                false,
            ));
            cells
        }
    };
    cells.into_iter().map(|c| inset(c, gap)).collect()
}

/// Splits an area into `n` equal columns, or rows.
fn split(n: usize, [left, top, width, height]: [i32; 4], columns: bool) -> Vec<[i32; 4]> {
    let n = i32::try_from(n).expect("a few windows");
    (0..n)
        .map(|i| {
            if columns {
                let (start, end) = (width * i / n, width * (i + 1) / n);
                [left + start, top, end - start, height]
            } else {
                let (start, end) = (height * i / n, height * (i + 1) / n);
                [left, top + start, width, end - start]
            }
        })
        .collect()
}

/// Shrinks a cell by a gap on all sides, but never below 1 pixel and never
/// out of the cell.
fn inset([x, y, w, h]: [i32; 4], gap: i32) -> [i32; 4] {
    let (gx, gy) = (gap.min((w - 1) / 2).max(0), gap.min((h - 1) / 2).max(0));
    [x + gx, y + gy, (w - 2 * gx).max(1), (h - 2 * gy).max(1)]
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

#[cfg(test)]
mod tests {
    use super::{master_stack, split};

    #[test]
    fn one_window_fills_the_area() {
        assert_eq!(master_stack(1, [0, 0, 100, 50], 0), vec![[0, 0, 100, 50]]);
    }

    #[test]
    fn the_stack_shares_the_right_half() {
        let cells = master_stack(3, [0, 0, 100, 60], 0);
        assert_eq!(
            cells,
            vec![[0, 0, 50, 60], [50, 0, 50, 30], [50, 30, 50, 30]]
        );
    }

    #[test]
    fn columns_and_rows_share_the_area() {
        assert_eq!(
            split(3, [0, 0, 90, 30], true),
            vec![[0, 0, 30, 30], [30, 0, 30, 30], [60, 0, 30, 30]]
        );
        assert_eq!(
            split(2, [10, 0, 50, 40], false),
            vec![[10, 0, 50, 20], [10, 20, 50, 20]]
        );
    }

    #[test]
    fn gaps_shrink_every_cell() {
        assert_eq!(master_stack(1, [0, 0, 100, 50], 5), vec![[5, 5, 90, 40]]);
    }

    #[test]
    fn a_huge_gap_stays_inside_the_cell() {
        let [[x, y, w, h]] = master_stack(1, [0, 0, 100, 50], 700)[..] else {
            panic!("one cell")
        };
        assert!(x + w <= 100 && y + h <= 50 && w >= 1 && h >= 1);
    }
}
