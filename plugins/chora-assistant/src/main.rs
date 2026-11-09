//! chora-assistant: the state of the voice agent (D65) as a figure above a
//! corner of the first output, and its speech in a bubble. It subscribes to
//! `voice.*`, drawn into the root `screen:NAME`, and a click on the figure
//! mutes or unmutes the mic.
//!
//! A block of the XP split (D73, `docs/xp-split.md`). Its look comes from
//! an asset folder: `--assets`, `CHORA_ASSISTANT_ASSETS`, the installed
//! `/usr/share/chora/assistant/assets`, else `plugins/chora-assistant/assets`
//! of the source tree.

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod assistant;
mod config;
mod paperclip;
mod voice;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_draw::{
    assets,
    theme::{self, Sheet, Theme},
};
use chora_kit::{
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Event, OutputInfo,
    ipc::{ClientError, RpcError},
};
use clap::Parser;
use serde_json::Value;

use crate::{
    assistant::{Assistant, Pose},
    config::{Config, Figure, Values},
};

/// The client name of the plugin, the prefix of its methods.
const NAME: &str = "assistant";

/// The asset folder of the installed package.
const INSTALLED_ASSETS: &str = "/usr/share/chora/assistant/assets";

/// How long the loop waits for an event while nothing moves.
const IDLE: Duration = Duration::from_secs(60);

/// The events that the plugin handles.
const EVENTS: &[&str] = &[
    "voice.*",
    "client.connected",
    "client.disconnected",
    "pointer.pressed",
    "output.changed",
    "output.removed",
];

/// The assistant of the voice agent.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The asset folder, with `values.yaml` and the pictures. Without it,
    /// the installed folder, else the source tree of the executable in
    /// `target/<profile>/`.
    #[arg(long, env = "CHORA_ASSISTANT_ASSETS")]
    assets: Option<PathBuf>,
    /// The settings file. Without it, chora-assistant reads
    /// $XDG_CONFIG_HOME/chora/chora-assistant.yaml, by default in
    /// ~/.config, if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
}

/// The plugin.
struct Agent {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    theme: Theme,
    values: Values,
    assistant: Option<Assistant>,
    /// The client name of the voice agent and its pose, while it is
    /// connected.
    voice: Option<(String, Pose)>,
    /// Whether the plugin claims the state indicator of the voice agent at
    /// its next chance.
    claim_due: bool,
}

impl Plugin for Agent {
    type Action = ();

    fn runtime(&mut self) -> &mut Runtime<()> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        let commands = match event {
            Event::PointerPressed { node, .. } => self.pressed(&node),
            Event::OutputChanged { .. } | Event::OutputRemoved { .. } => {
                self.replace_assistant()?
            }
            Event::Emitted {
                event,
                client,
                data,
            } => self.voice_event(&event, &client, &data)?,
            Event::ClientConnected { name } => {
                self.voice_connected(&name);
                Vec::new()
            }
            Event::ClientDisconnected { name } => self.voice_gone(&name)?,
            _ => Vec::new(),
        };
        self.apply(commands)
    }

    fn method(&mut self, method: &str, _: Value) -> Result<Value, RpcError> {
        Err(RpcError::new(
            RpcError::METHOD_NOT_FOUND,
            format!("{NAME} has no method {method}"),
        ))
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.claim_indicator(false);
        self.replace_assistant()
            .and_then(|commands| self.apply(commands))
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

impl Agent {
    /// Connects, subscribes to the events of the voice agent and exports
    /// the methods of the settings, and shows the figure if the agent is
    /// awake.
    fn start(
        theme: Theme,
        values: Values,
        settings: settings::File<Config>,
    ) -> Result<Self, ClientError> {
        let rt = Runtime::start(
            NAME,
            Vec::new(),
            EVENTS,
            settings::File::<Config>::methods(),
        )?;
        let mut agent = Self {
            rt,
            settings,
            theme,
            values,
            assistant: None,
            voice: None,
            claim_due: true,
        };
        agent.ask_voice()?;
        Ok(agent)
    }

    /// Applies a transaction, with the logical sizes of the pictures.
    fn apply(&mut self, mut commands: Vec<Command>) -> Result<(), ClientError> {
        if commands.is_empty() {
            return Ok(());
        }
        self.theme.size_images(&mut commands);
        self.rt.conn.apply(commands)
    }

    /// The first output, which shows the figure, and the largest scale of
    /// the outputs.
    fn output(&mut self) -> Result<Option<(OutputInfo, f64)>, ClientError> {
        let outputs = self.rt.outputs()?;
        let scale = outputs.iter().map(|o| o.scale).fold(1.0, f64::max);
        Ok(outputs.into_iter().next().map(|o| (o, scale)))
    }

    /// A press on the figure mutes the mic or unmutes it; one on a button
    /// of a question answers it.
    fn pressed(&mut self, node: &str) -> Vec<Command> {
        if self.press_answer(node) {
            return Vec::new();
        }
        if Assistant::owns(node) {
            return self.press_assistant();
        }
        Vec::new()
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    runtime::run(NAME, || serve(&args))
}

/// One connection to the core, until it closes.
fn serve(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-assistant")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    let dir = assets::resolve(
        args.assets.clone(),
        Path::new(INSTALLED_ASSETS),
        exe.as_deref(),
        "plugins/chora-assistant/assets",
    );
    let (theme, values) = load_assets(&dir, &theme::cache_dir("chora-assistant")?)?;
    let mut agent = Agent::start(theme, values, settings)?;
    loop {
        let now = Instant::now();
        let wait = agent
            .assistant_due()
            .map_or(IDLE, |due| due.saturating_duration_since(now));
        if let Some(event) = agent
            .rt
            .conn
            .next_event_timeout(wait.max(Duration::from_millis(1)))?
        {
            runtime::dispatch(&mut agent, event)?;
        }
        agent.assistant_step(Instant::now())?;
    }
}

/// The values and the pictures of an asset folder. A missing or broken
/// file stops the start with an error that names it.
fn load_assets(dir: &Path, cache: &Path) -> Result<(Theme, Values), String> {
    let values: Values = assets::values(dir)?;
    let sheets: &[Sheet] = match values.figure {
        Figure::Dot => &[assistant::DOT],
        Figure::Paperclip => &[],
    };
    let theme = Theme::load(dir, cache, sheets, &values.font)?;
    Ok((theme, values))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache(test: &str) -> PathBuf {
        std::env::temp_dir().join(format!("chora-assistant-{test}-{}", std::process::id()))
    }

    /// The theme and the values of the neutral folder, and the cache.
    pub fn loaded(test: &str) -> (Theme, Values, PathBuf) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let cache = cache(test);
        let (theme, values) = load_assets(&dir, &cache).expect("the neutral folder loads");
        (theme, values, cache)
    }

    #[test]
    fn the_neutral_folder_loads() {
        let (_, values, cache) = loaded("neutral");
        assert_eq!(values.figure, Figure::Dot);
        assert_eq!(values.speech.tail, 0, "no tail");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_xp_folder_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/assets/assistant");
        let cache = cache("xp");
        let (_, values) = load_assets(&dir, &cache).expect("the XP folder loads");
        assert_eq!(values.figure, Figure::Paperclip);
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_missing_picture_names_its_file() {
        let dir =
            std::env::temp_dir().join(format!("chora-assistant-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("values.yaml"), "{}\n").unwrap();
        let err = load_assets(&dir, &cache("empty")).err().expect("an error");
        assert!(err.contains("dot.svg"), "{err}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
