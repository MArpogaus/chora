//! Reference tiling plugin for chora: a master and stack layout with
//! keybindings to start a terminal, close a window and move the focus.

use std::process::ExitCode;

use chora_scene::{
    Binding, Command, Event, NodeKind, OutputInfo, PropsUpdate, ROOT, ToplevelState, Transform,
    ipc::{Apply, Client, ClientError, Grab, Hello, Query, Subscribe},
};
use clap::{Parser, ValueEnum};
use serde_json::Value;

/// The group that holds the tiled surfaces.
const GROUP: &str = "tiling";

/// Tile the windows of chora in a master and stack layout.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The modifier of all bindings.
    #[arg(long = "mod", value_enum, default_value_t = Modifier::Super)]
    modifier: Modifier,
    /// The terminal that MOD+Return starts.
    #[arg(long, default_value = "foot")]
    terminal: String,
    /// The gap around each window, in logical pixels.
    #[arg(long, default_value_t = 4)]
    gap: i32,
}

#[derive(Clone, Copy, ValueEnum)]
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
    windows: Vec<String>,
    focused: usize,
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
        name: "tiling".into(),
        keep_nodes: false,
    };
    let mut tiling = Tiling {
        client: Client::from_env(&hello)?,
        args,
        bindings: Vec::new(),
        area: [0; 4],
        windows: Vec::new(),
        focused: 0,
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
        let group = Command::Create {
            id: GROUP.into(),
            kind: NodeKind::Group,
            parent: ROOT.into(),
            props: PropsUpdate::default(),
        };
        self.apply(vec![group])?;
        let modifier = match self.args.modifier {
            Modifier::Super => "Super",
            Modifier::Alt => "Alt",
        };
        for (key, action) in [
            ("Return", Action::Terminal),
            ("q", Action::Close),
            ("j", Action::FocusNext),
            ("k", Action::FocusPrevious),
        ] {
            let binding: Binding = format!("{modifier}+{key}").parse().expect("valid bindings");
            self.client.call::<_, Value>(
                "input.grab",
                Grab {
                    binding: binding.clone(),
                },
            )?;
            self.bindings.push((binding.to_string(), action));
        }
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
        let free: Vec<Value> = self.client.call(
            "scene.query",
            Query {
                selector: "kind:surface".into(),
            },
        )?;
        for node in free.iter().filter(|n| n["owner"].is_null()) {
            if let Some(id) = node["id"].as_str() {
                self.adopt(id)?;
            }
        }
        self.relayout()
    }

    fn handle(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::SurfaceCreated { surface, .. } => {
                self.adopt(&surface)?;
                self.focused = self.windows.len().saturating_sub(1);
                self.relayout()
            }
            Event::SurfaceDestroyed { surface } | Event::SurfaceDetached { surface } => {
                self.windows.retain(|w| *w != surface);
                self.focused = self.focused.min(self.windows.len().saturating_sub(1));
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
        let n = self.windows.len().max(1);
        match action {
            Action::Terminal => {
                // `&` lets the shell exit at once, so the terminal never
                // becomes a zombie of this plugin.
                std::process::Command::new("sh")
                    .args(["-c", &format!("{} &", self.args.terminal)])
                    .status()?;
                return Ok(());
            }
            Action::Close => {
                if let Some(surface) = self.windows.get(self.focused).cloned() {
                    return self.apply(vec![Command::Close { surface }]);
                }
                return Ok(());
            }
            Action::FocusNext => self.focused = (self.focused + 1) % n,
            Action::FocusPrevious => self.focused = (self.focused + n - 1) % n,
        }
        self.relayout()
    }

    fn adopt(&mut self, surface: &str) -> Result<(), ClientError> {
        let attach = Command::Attach {
            surface: surface.into(),
            parent: GROUP.into(),
        };
        self.apply(vec![attach])?;
        self.windows.push(surface.into());
        Ok(())
    }

    /// Places every window and moves the focus, in one transaction.
    fn relayout(&mut self) -> Result<(), ClientError> {
        let cells = master_stack(self.windows.len(), self.area, self.args.gap);
        let mut commands = Vec::new();
        for (i, (surface, [x, y, w, h])) in self.windows.iter().zip(cells).enumerate() {
            let mut states = vec![ToplevelState::Tiled];
            if i == self.focused {
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
            surface: self.windows.get(self.focused).cloned(),
        });
        self.apply(commands)
    }

    /// Applies a transaction. A rejected transaction only logs: a surface
    /// can disappear between an event and the answer to it.
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
fn master_stack(n: usize, [left, top, width, height]: [i32; 4], gap: i32) -> Vec<[i32; 4]> {
    let inset = |[cx, cy, cw, ch]: [i32; 4]| {
        [
            cx + gap,
            cy + gap,
            (cw - 2 * gap).max(1),
            (ch - 2 * gap).max(1),
        ]
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
}
