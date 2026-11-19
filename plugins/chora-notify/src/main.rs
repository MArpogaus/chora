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
    bus::{Bus, Note, Reason, Request, Signal, StartError},
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
        if !chora_kit::runtime::hears_pointer(&event, self.rt.accepts_injected()) {
            return Ok(());
        }
        let commands = match event {
            Event::PointerPressed { node, injected, .. }
                if Card::owns(&node) && (!injected || self.rt.accepts_injected()) =>
            {
                self.press(&node)?
            }
            Event::PointerEntered { node, .. } => self.hover(&node, true), // injected: Plugin::event drops an injected pointer event first
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

    fn state(&self) -> Option<Value> {
        let shown = self.card.as_ref().map(|(note, _)| note);
        Some(state_value(shown, &self.theme.texts()))
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.queue.set_times(self.settings.value);
        self.redraw()
            .and_then(|commands| self.apply(commands))
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

/// The answer of `state.get`: the notification on screen with the node
/// of its card, the runs of text of the card, and nothing busy, since a
/// card shows and goes without an animation.
fn state_value(shown: Option<&Note>, texts: &[theme::NodeRun]) -> Value {
    let shown: Vec<Value> = shown
        .map(
            |note| serde_json::json!({"id": note.id, "summary": note.summary, "node": card::GROUP}),
        )
        .into_iter()
        .collect();
    serde_json::json!({"shown": shown, "texts": texts, "busy": []})
}

impl Notify {
    /// Connects, takes the name on the session bus and exports the
    /// methods of the settings. `None` when another server owns the name:
    /// the plugin has nothing to do then, and ends without an error, so
    /// that no unit restarts it.
    fn start(
        theme: Theme,
        values: Values,
        settings: settings::File<Config>,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let bus = match Bus::start() {
            Ok(bus) => bus,
            Err(err @ StartError::Taken) => {
                eprintln!("chora-notify: {err}, which stays its owner");
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        };
        let mut methods = settings::File::<Config>::methods();
        methods.push(runtime::state_method());
        let rt = Runtime::start(NAME, Vec::new(), EVENTS, methods)?;
        Ok(Some(Self {
            rt,
            queue: Queue::new(settings.value),
            settings,
            theme,
            values,
            icons: Icons::new(icons::THEMES),
            bus,
            card: None,
        }))
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
            .and_then(|i| self.icons.for_sender(i));
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
    let Some(mut notify) = Notify::start(theme, values, settings)? else {
        return Ok(());
    };
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

    struct KillOnDrop(std::process::Child);

    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// A notify on a private session bus and a fake core that answers
    /// every request with null and `output.list` with no outputs. `None`
    /// without a `dbus-daemon` to start.
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    fn on_private_bus(tag: &str) -> Option<(Notify, KillOnDrop)> {
        use std::io::{BufRead as _, BufReader, Write as _};
        let daemon = std::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .ok()?;
        let mut daemon = KillOnDrop(daemon);
        let mut address = String::new();
        let stdout = daemon.0.stdout.take().unwrap();
        BufReader::new(stdout).read_line(&mut address).unwrap();
        let connection = zbus::blocking::connection::Builder::address(address.trim())
            .unwrap()
            .build()
            .unwrap();
        let dir = cache(tag);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("core.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = stream.try_clone().unwrap();
            for line in BufReader::new(&stream).lines().map_while(Result::ok) {
                let request: Value = serde_json::from_str(&line).unwrap();
                let result = match request["method"].as_str().unwrap_or_default() {
                    "hello" => serde_json::json!({"version": chora_scene::ipc::VERSION}),
                    "output.list" => serde_json::json!([]),
                    _ => Value::Null,
                };
                let answer =
                    serde_json::json!({"jsonrpc": "2.0", "id": request["id"], "result": result});
                writeln!(writer, "{answer}").unwrap();
            }
        });
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let (theme, values) = load_assets(&assets, &dir.join("cache")).expect("the assets load");
        let settings =
            settings::File::<Config>::load(dir.join("chora-notify.yaml"), config::DEFAULT).unwrap();
        let conn = chora_kit::conn::Conn::connect_at(NAME, &path).unwrap();
        let mut rt = Runtime::with(conn, NAME, Vec::new());
        rt.pin_accepts_injected(false);
        let notify = Notify {
            rt,
            queue: Queue::new(settings.value),
            settings,
            theme,
            values,
            icons: Icons::new(icons::THEMES),
            bus: Bus::on(connection).expect("the name is free"),
            card: None,
        };
        Some((notify, daemon))
    }

    #[test]
    fn an_injected_press_on_the_card_closes_nothing() {
        let Some((mut notify, _daemon)) = on_private_bus("injected-card") else {
            return;
        };
        let note = Note {
            id: 7,
            icon: String::new(),
            summary: "title".into(),
            body: "text".into(),
            actions: Vec::new(),
            expire_ms: 0,
        };
        notify.queue.push(note.clone(), Instant::now());
        let press = |injected| {
            serde_json::from_value::<Event>(serde_json::json!({
                "event": "pointer.pressed", "node": "notify-card-close", "button": 272,
                "position": [0.0, 0.0], "injected": injected}))
            .unwrap()
        };
        notify.event(press(true)).unwrap();
        assert_eq!(notify.queue.current(), Some(&note), "an injected press");
        notify.rt.pin_accepts_injected(true);
        notify.event(press(true)).unwrap();
        assert_eq!(notify.queue.current(), None, "an injected press in a test");
        notify.queue.push(note.clone(), Instant::now());
        notify.rt.pin_accepts_injected(false);
        notify.event(press(false)).unwrap();
        assert_eq!(notify.queue.current(), None, "the press of the person");
    }

    fn note(id: u32, summary: &str) -> Note {
        Note {
            id,
            icon: String::new(),
            summary: summary.into(),
            body: "text".into(),
            actions: Vec::new(),
            expire_ms: 0,
        }
    }

    /// `state.get` lists the notification on screen with the node of its
    /// card, the runs of text of the card, and nothing busy.
    #[test]
    fn the_state_shows_the_card_on_screen() {
        let shown = state_value(Some(&note(7, "Battery low")), &[]);
        assert_eq!(
            shown,
            serde_json::json!({"shown": [{"id": 7, "summary": "Battery low", "node": card::GROUP}],
                "texts": [], "busy": []})
        );
        let empty = state_value(None, &[]);
        assert_eq!(empty["shown"], serde_json::json!([]));
    }

    #[test]
    fn notify_answers_state_get() {
        let Some((notify, _daemon)) = on_private_bus("state") else {
            return;
        };
        let state = notify.state().expect("notify has a state");
        assert_eq!(state["shown"], serde_json::json!([]));
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
