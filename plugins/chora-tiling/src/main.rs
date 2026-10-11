//! Reference tiling plugin for chora: a master and stack layout with
//! keybindings to start a terminal, close a window and move the focus.

use std::process::ExitCode;

use chora_scene::{
    Binding, Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, ToplevelState, Transform,
    ipc::{Apply, Client, ClientError, Grab, Hello, Query, Subscribe},
};
use clap::{Parser, ValueEnum};
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
    /// The gap around each window, in logical pixels.
    #[arg(long, default_value_t = 4)]
    gap: u16,
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
        let events = [
            "surface.created",
            "surface.destroyed",
            "surface.detached",
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
            _ => Ok(()),
        }
    }

    fn act(&mut self, binding: &str) -> Result<(), ClientError> {
        let Some((_, action)) = self.bindings.iter().find(|(b, _)| b == binding) else {
            return Ok(());
        };
        match action {
            Action::Terminal => {
                // `&` lets the shell exit at once, so the terminal never
                // becomes a zombie of this plugin.
                let spawned = std::process::Command::new("sh")
                    .args(["-c", &format!("{} &", self.args.terminal)])
                    .status();
                if let Err(err) = spawned {
                    eprintln!("chora-tiling: cannot run {:?}: {err}", self.args.terminal);
                }
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
        let cells = master_stack(self.windows.len(), self.area, i32::from(self.args.gap));
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
    let ([x, y], [w, h]) = (output.position, output.size);
    [x, y, w, h]
}

/// The cells of a master and stack layout: the first window takes the left
/// half, the others share the right half. Each cell is x, y, width, height.
/// A gap shrinks each cell on all sides, but never below 1 pixel and never
/// out of its cell.
fn master_stack(n: usize, [left, top, width, height]: [i32; 4], gap: i32) -> Vec<[i32; 4]> {
    let inset = |[cx, cy, cw, ch]: [i32; 4]| {
        let (gx, gy) = (gap.min((cw - 1) / 2).max(0), gap.min((ch - 1) / 2).max(0));
        [cx + gx, cy + gy, (cw - 2 * gx).max(1), (ch - 2 * gy).max(1)]
    };
    match n {
        0 => Vec::new(),
        1 => vec![inset([left, top, width, height])],
        _ => {
            let half = width / 2;
            let stack = i32::try_from(n - 1).expect("a few windows");
            let mut cells = vec![inset([left, top, half, height])];
            for i in 0..stack {
                let (start, end) = (height * i / stack, height * (i + 1) / stack);
                cells.push(inset([left + half, top + start, width - half, end - start]));
            }
            cells
        }
    }
}

#[cfg(test)]
mod tests {
    use super::master_stack;

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
