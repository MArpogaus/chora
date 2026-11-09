//! chora-notify: the notification server of the freedesktop specification
//! on the session bus. It shows each notification as a card in a corner of
//! the first output, drawn into the root `screen:NAME`, and sends the
//! signals of the specification for a click or the end of the time.
//!
//! A block of the XP split (D73, `docs/xp-split.md`). Its look comes from
//! an asset folder: `--assets`, `CHORA_NOTIFY_ASSETS`, the installed
//! `/usr/share/chora/notify/assets`, else `plugins/chora-notify/assets` of
//! the source tree.

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod bus;
mod card;
mod config;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_draw::{
    assets, icons,
    theme::{self, Theme},
};
use chora_kit::{
    icons::Icons,
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
    bus::{Bus, Note, Reason, Request, Signal},
    card::{Card, Queue, Values},
    config::Config,
};

/// The client name of the plugin, the prefix of its methods.
const NAME: &str = "notify";

/// The asset folder of the installed package.
const INSTALLED_ASSETS: &str = "/usr/share/chora/notify/assets";

/// The notification server of chora.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The asset folder, with `values.yaml` and the pictures. Without it,
    /// the installed folder, else the source tree of the executable in
    /// `target/<profile>/`.
    #[arg(long, env = "CHORA_NOTIFY_ASSETS")]
    assets: Option<PathBuf>,
    /// The settings file. Without it, chora-notify reads
    /// $XDG_CONFIG_HOME/chora/chora-notify.yaml, by default in ~/.config,
    /// if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
}

/// The plugin.
struct Notify {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    theme: Theme,
    values: Values,
    icons: Icons,
    bus: Bus,
    queue: Queue,
    card: Option<(Note, Card)>,
}

impl Plugin for Notify {
    type Action = ();

    fn runtime(&mut self) -> &mut Runtime<()> {
        &mut self.rt
    }

    fn event(&mut self, event: Event) -> Result<(), ClientError> {
        let commands = match event {
            Event::PointerPressed { node, .. } if Card::owns(&node) => self.press(&node)?,
            Event::PointerEntered { node, .. } => self.hover(&node, true),
            Event::PointerLeft { node } => self.hover(&node, false),
            Event::OutputChanged { .. } | Event::OutputRemoved { .. } => self.redraw()?,
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
        self.queue.set_times(self.settings.value);
        self.redraw()
            .and_then(|commands| self.apply(commands))
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

impl Notify {
    /// Connects, takes the name on the session bus and exports the
    /// methods of the settings.
    fn start(
        theme: Theme,
        values: Values,
        settings: settings::File<Config>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let bus = Bus::start()?;
        let rt = Runtime::start(
            NAME,
            Vec::new(),
            EVENTS,
            settings::File::<Config>::methods(),
        )?;
        Ok(Self {
            rt,
            queue: Queue::new(settings.value),
            settings,
            theme,
            values,
            icons: Icons::new(icons::THEMES),
            bus,
            card: None,
        })
    }

    /// Applies a transaction, with the logical sizes of the pictures.
    fn apply(&mut self, mut commands: Vec<Command>) -> Result<(), ClientError> {
        if commands.is_empty() {
            return Ok(());
        }
        self.theme.size_images(&mut commands);
        self.rt.conn.apply(commands)
    }

    /// The first output, which shows the card, and the largest scale of
    /// the outputs.
    fn output(&mut self) -> Result<Option<(OutputInfo, f64)>, ClientError> {
        let outputs = self.rt.outputs()?;
        let scale = outputs.iter().map(|o| o.scale).fold(1.0, f64::max);
        Ok(outputs.into_iter().next().map(|o| (o, scale)))
    }

    /// When the loop looks at the notifications again: soon, for the
    /// calls of the bus.
    fn due(&self, now: Instant) -> Instant {
        let poll = now + card::POLL;
        self.queue.due().map_or(poll, |due| due.min(poll))
    }

    /// Takes the calls of the clients and closes an expired card.
    fn step(&mut self, now: Instant) -> Result<(), ClientError> {
        let requests: Vec<Request> = self.bus.requests.try_iter().collect();
        let mut signals = Vec::new();
        for request in requests {
            match request {
                Request::Notify(note) => self.queue.push(note, now),
                Request::Close(id) => signals.extend(self.queue.close(id, Reason::Closed, now)),
            }
        }
        signals.extend(self.queue.expire(now));
        let commands = self.after(&signals)?;
        self.apply(commands)
    }

    /// A click on the close button dismisses the card; one on the card
    /// invokes its default action, too.
    fn press(&mut self, node: &str) -> Result<Vec<Command>, ClientError> {
        let now = Instant::now();
        let signals = match self.queue.current() {
            Some(shown) if Card::is_close(node) => {
                self.queue.close(shown.id, Reason::Dismissed, now)
            }
            _ => self.queue.click(now),
        };
        self.after(&signals)
    }

    fn hover(&mut self, node: &str, on: bool) -> Vec<Command> {
        match self.card.as_mut() {
            Some((_, card)) if Card::owns(node) => card.hover(&mut self.theme, node, on),
            _ => Vec::new(),
        }
    }

    /// Sends the signals and shows the card of the current notification,
    /// if it changed.
    fn after(&mut self, signals: &[Signal]) -> Result<Vec<Command>, ClientError> {
        for signal in signals {
            self.bus.emit(signal);
        }
        let current = self.queue.current();
        if self.card.as_ref().map(|(note, _)| note) == current {
            return Ok(Vec::new());
        }
        let current = current.cloned();
        let mut commands = Vec::new();
        if self.card.take().is_some() {
            commands.push(Card::remove());
        }
        if let Some(note) = current {
            commands.extend(self.show(note)?);
        }
        Ok(commands)
    }

    /// The commands that create the card of a notification.
    fn show(&mut self, note: Note) -> Result<Vec<Command>, ClientError> {
        let Some((output, scale)) = self.output()? else {
            return Ok(Vec::new());
        };
        self.theme.set_scale(scale);
        let icon = Some(note.icon.as_str())
            .filter(|i| !i.is_empty())
            .and_then(|i| self.icons.find(i));
        let corner = self.settings.value.position;
        let card = Card::new(
            &mut self.theme,
            &note,
            icon.as_deref(),
            &self.values,
            (&output, corner),
        );
        let commands = card.create(&mut self.theme);
        self.card = Some((note, card));
        Ok(commands)
    }

    /// Draws the card again, after a change of the outputs or of the
    /// corner.
    fn redraw(&mut self) -> Result<Vec<Command>, ClientError> {
        let Some((note, _)) = self.card.take() else {
            return Ok(Vec::new());
        };
        let mut commands = Vec::new();
        if !self
            .rt
            .conn
            .query(&format!("id:{}", card::GROUP))?
            .is_empty()
        {
            commands.push(Card::remove());
        }
        commands.extend(self.show(note)?);
        Ok(commands)
    }
}

/// The events that the plugin handles.
const EVENTS: &[&str] = &[
    "pointer.pressed",
    "pointer.entered",
    "pointer.left",
    "output.changed",
    "output.removed",
];

fn main() -> ExitCode {
    let args = Args::parse();
    runtime::run(NAME, || serve(&args))
}

/// One connection to the core, until it closes.
fn serve(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-notify")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    let dir = assets::resolve(
        args.assets.clone(),
        Path::new(INSTALLED_ASSETS),
        exe.as_deref(),
        "plugins/chora-notify/assets",
    );
    let (theme, values) = load_assets(&dir, &theme::cache_dir("chora-notify")?)?;
    let mut notify = Notify::start(theme, values, settings)?;
    loop {
        let wait = notify
            .due(Instant::now())
            .saturating_duration_since(Instant::now());
        if let Some(event) = notify
            .rt
            .conn
            .next_event_timeout(wait.max(Duration::from_millis(1)))?
        {
            runtime::dispatch(&mut notify, event)?;
        }
        notify.step(Instant::now())?;
    }
}

/// The values and the pictures of an asset folder. A missing or broken
/// file stops the start with an error that names it.
fn load_assets(dir: &Path, cache: &Path) -> Result<(Theme, Values), String> {
    let values: Values = assets::values(dir)?;
    let theme = Theme::load(dir, cache, &[], &values.font)?;
    assets::check_pictures(&theme, &card::PICTURES)?;
    Ok((theme, values))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache(test: &str) -> PathBuf {
        std::env::temp_dir().join(format!("chora-notify-{test}-{}", std::process::id()))
    }

    #[test]
    fn the_neutral_folder_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let cache = cache("neutral");
        let (_, values) = load_assets(&dir, &cache).expect("the neutral folder loads");
        assert_eq!(values.card.tail, 0, "no tail");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn the_xp_folder_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/assets/notify");
        let cache = cache("xp");
        let (_, values) = load_assets(&dir, &cache).expect("the XP folder loads");
        assert!(values.card.tail > 0, "the tail of XP's balloon");
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_missing_picture_names_its_file() {
        let dir = std::env::temp_dir().join(format!("chora-notify-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("values.yaml"), "{}\n").unwrap();
        let err = load_assets(&dir, &cache("empty")).err().expect("an error");
        assert!(err.contains("close.svg"), "{err}");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
