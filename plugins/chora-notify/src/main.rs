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
    notice as kit_notice,
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
};
use chora_scene::{
    Command, Event, OutputInfo,
    ipc::{ClientError, RpcError},
};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    bus::{Bus, Ids, Note, Reason, Request, Signal, StartError, next_id},
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

/// The parameters of `notify.show`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Show {
    #[serde(default)]
    #[allow(
        dead_code,
        reason = "the caller names itself; the card does not show it"
    )]
    app: String,
    #[serde(default)]
    replaces: Option<u32>,
    #[serde(default)]
    icon: String,
    summary: String,
    #[serde(default)]
    body: String,
    #[serde(default = "default_expire")]
    expire_ms: i32,
    #[serde(default)]
    urgency: kit_notice::Urgency,
}

const fn default_expire() -> i32 {
    -1
}

/// The parameters of `notify.close`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Close {
    id: u32,
}

/// The plugin.
struct Notify {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    theme: Theme,
    values: Values,
    icons: Icons,
    bus: Option<Bus>,
    ids: Ids,
    pending: Vec<Request>,
    queue: Queue,
    card: Option<(Note, Card)>,
    /// The output and the box of the open chat of the HUD host, which the
    /// card does not cover.
    chat: Option<(String, [i32; 4])>,
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
            Event::Emitted {
                event,
                client,
                data,
            } if client == HOST && event == CHAT => {
                let chat = chat_box(&data);
                if chat == self.chat {
                    Vec::new()
                } else {
                    self.chat = chat;
                    self.redraw()?
                }
            }
            _ => Vec::new(),
        };
        self.apply(commands)
    }

    fn method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        match method {
            kit_notice::SHOW => {
                let call: Show = runtime::params(params)?;
                let id = next_id(&self.ids, call.replaces.unwrap_or(0));
                let critical = call.urgency == kit_notice::Urgency::Critical;
                self.pending.push(Request::Notify(Note {
                    id,
                    icon: call.icon,
                    summary: call.summary,
                    body: call.body,
                    actions: Vec::new(),
                    expire_ms: if critical { 0 } else { call.expire_ms },
                }));
                Ok(json!({"id": id}))
            }
            kit_notice::CLOSE => {
                let call: Close = runtime::params(params)?;
                self.pending.push(Request::Close(call.id));
                Ok(Value::Null)
            }
            _ => Err(RpcError::new(
                RpcError::METHOD_NOT_FOUND,
                format!("{NAME} has no method {method}"),
            )),
        }
    }

    fn settings(&mut self) -> Option<&mut dyn Store> {
        Some(&mut self.settings)
    }

    fn state(&self) -> Option<Value> {
        let shown = self.card.as_ref().map(|(note, card)| (note, card.rect()));
        let mut state = state_value(shown, &self.queue, &self.theme.texts());
        state["chat"] = json!(self.chat.as_ref().map(|(_, rect)| rect));
        Some(state)
    }

    fn settings_changed(&mut self) -> Result<(), RpcError> {
        self.queue.set_times(self.settings.value);
        self.redraw()
            .and_then(|commands| self.apply(commands))
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

/// The answer of `state.get`: the notification on screen with the node
/// and the box of its card on the output, the ids that the server
/// accepted, that wait and that closed, the runs of text of the card, and
/// nothing busy, since a card shows and goes without an animation.
fn state_value(shown: Option<(&Note, [i32; 4])>, queue: &Queue, texts: &[theme::NodeRun]) -> Value {
    let shown: Vec<Value> = shown
        .map(|(note, rect)| {
            json!({"id": note.id, "summary": note.summary, "body": note.body,
                "node": card::BODY, "rect": rect})
        })
        .into_iter()
        .collect();
    serde_json::json!({
        "shown": shown,
        "accepted": queue.accepted_ids(),
        "waiting": queue.waiting_ids(),
        "closed": queue.closed_ids(),
        "texts": texts,
        "busy": [],
    })
}

impl Notify {
    /// Connects, takes the name on the session bus and exports the
    /// methods of the settings; see [`Self::on`].
    fn start(
        theme: Theme,
        values: Values,
        settings: settings::File<Config>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ids = Ids::default();
        let bus = Bus::start(ids.clone());
        let mut rt = Runtime::start(NAME, Vec::new(), EVENTS, Self::methods())?;
        let ready = json!({"event": READY, "data": {}});
        if let Err(err) = rt.conn.call::<_, Value>("events.emit", ready) {
            eprintln!("chora-notify: {READY}: {err}");
        }
        Ok(Self::on(rt, bus, ids, (theme, values), settings))
    }

    /// The plugin on a connection, with the outcome of taking the bus
    /// name. Without the name, as when another server owns it or there is
    /// no session bus, the plugin serves `notify.show` and `notify.close`
    /// to the plugins all the same: their notices show as cards, while
    /// the clients of the bus reach the other server.
    fn on(
        rt: Runtime<()>,
        bus: Result<Bus, StartError>,
        ids: Ids,
        (theme, values): (Theme, Values),
        settings: settings::File<Config>,
    ) -> Self {
        let bus = match bus {
            Ok(bus) => Some(bus),
            Err(err @ StartError::Taken) => {
                eprintln!(
                    "chora-notify: {err}, which stays its owner; only the plugins are served"
                );
                None
            }
            Err(err) => {
                eprintln!("chora-notify: no session bus ({err}); only the plugins are served");
                None
            }
        };
        Self {
            rt,
            queue: Queue::new(settings.value),
            settings,
            theme,
            values,
            icons: Icons::new(icons::THEMES),
            bus,
            ids,
            pending: Vec::new(),
            card: None,
            chat: None,
        }
    }

    /// The methods of the settings, `state.get` and the live API.
    fn methods() -> Vec<chora_scene::ipc::ExportMethod> {
        let mut methods = settings::File::<Config>::methods();
        methods.push(runtime::state_method());
        methods.push(runtime::method(
            kit_notice::SHOW,
            "Shows a notice, or replaces the one named by `replaces`; answers its id.",
            json!({
                "app": {"type": "string"},
                "replaces": {"type": ["integer", "null"]},
                "icon": {"type": "string"},
                "summary": {"type": "string"},
                "body": {"type": "string"},
                "expire_ms": {"type": "integer"},
                "urgency": {"enum": ["normal", "critical"]},
            }),
            &["summary"],
        ));
        methods.push(runtime::method(
            kit_notice::CLOSE,
            "Dismisses the notice with an id.",
            json!({"id": {"type": "integer"}}),
            &["id"],
        ));
        methods
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
        let mut requests = std::mem::take(&mut self.pending);
        if let Some(bus) = &self.bus {
            requests.extend(bus.requests.try_iter());
        }
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
        if let Some(bus) = &self.bus {
            for signal in signals {
                bus.emit(signal);
            }
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
        let chat = (self.chat.as_ref())
            .filter(|(on, _)| *on == output.name)
            .map(|(_, rect)| *rect);
        let card = Card::new(
            &mut self.theme,
            &note,
            icon.as_deref(),
            &self.values,
            (&output, corner, chat),
        );
        let commands = card.create(&mut self.theme);
        self.card = Some((note, card));
        Ok(commands)
    }

    /// Draws the card again, after a change of the outputs, of the corner
    /// or of the chat.
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
    CHAT,
];

/// The client name of the HUD host, which draws the chat.
const HOST: &str = "assistant";

/// The event of the plugin once it listens, so that the HUD host says
/// where its open chat is.
const READY: &str = "notify.ready";

/// The event of the HUD host when its chat opens, closes or moves.
const CHAT: &str = "assistant.chat";

/// The output and the box of the chat that `assistant.chat {open, output,
/// rect}` names; `None` when the chat closed or names no box.
fn chat_box(data: &Value) -> Option<(String, [i32; 4])> {
    if data["open"] != true {
        return None;
    }
    let output = data["output"].as_str()?.to_owned();
    let rect: [i32; 4] = serde_json::from_value(data["rect"].clone()).ok()?;
    Some((output, rect))
}

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

    struct KillOnDrop(std::process::Child);

    impl Drop for KillOnDrop {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// A private session bus: its address and the daemon. `None` without
    /// a `dbus-daemon` to start.
    fn private_bus() -> Option<(String, KillOnDrop)> {
        use std::io::{BufRead as _, BufReader};
        let daemon = std::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .ok()?;
        let mut daemon = KillOnDrop(daemon);
        let mut address = String::new();
        let stdout = daemon.0.stdout.take().unwrap();
        BufReader::new(stdout).read_line(&mut address).unwrap();
        Some((address.trim().to_owned(), daemon))
    }

    fn connect(address: &str) -> zbus::blocking::Connection {
        zbus::blocking::connection::Builder::address(address)
            .unwrap()
            .build()
            .unwrap()
    }

    /// A notify with the outcome of taking the bus name, on a fake core
    /// that answers every request with null and `output.list` with no
    /// outputs.
    #[allow(
        clippy::disallowed_methods,
        reason = "the test connects to its fake core"
    )]
    fn on_fake_core(tag: &str, bus: Result<Bus, StartError>, ids: Ids) -> Notify {
        use std::io::{BufRead as _, BufReader, Write as _};
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
        let assets = load_assets(&assets, &dir.join("cache")).expect("the assets load");
        let settings =
            settings::File::<Config>::load(dir.join("chora-notify.yaml"), config::DEFAULT).unwrap();
        let conn = chora_kit::conn::Conn::connect_at(NAME, &path).unwrap();
        let mut rt = Runtime::with(conn, NAME, Vec::new());
        rt.pin_accepts_injected(false);
        Notify::on(rt, bus, ids, assets, settings)
    }

    /// A notify on a private session bus whose name it owns.
    fn on_private_bus(tag: &str) -> Option<(Notify, KillOnDrop)> {
        let (address, daemon) = private_bus()?;
        let ids = Ids::default();
        let bus = Bus::on(connect(&address), ids.clone());
        assert!(bus.is_ok(), "the name is free");
        let notify = on_fake_core(tag, bus, ids);
        Some((notify, daemon))
    }

    /// Another server owns the name on the bus, as mako when its
    /// activation file wins: notify keeps serving `notify.show` and
    /// `notify.close` to the plugins, without the bus.
    #[test]
    fn a_taken_name_leaves_the_live_api_served() {
        let Some((address, _daemon)) = private_bus() else {
            return;
        };
        let owner = Bus::on(connect(&address), Ids::default()).expect("the name is free");
        let ids = Ids::default();
        let taken = Bus::on(connect(&address), ids.clone());
        assert!(matches!(taken, Err(StartError::Taken)), "the name is taken");
        let mut notify = on_fake_core("taken", taken, ids);
        assert!(notify.bus.is_none(), "no bus");
        let shown = (notify.method(kit_notice::SHOW, json!({"summary": "Saved"})))
            .expect("notify.show answers");
        notify.step(Instant::now()).unwrap();
        let summary = notify.queue.current().map(|n| n.summary.clone());
        assert_eq!(summary.as_deref(), Some("Saved"));
        (notify.method(kit_notice::CLOSE, json!({"id": shown["id"]})))
            .expect("notify.close answers");
        notify.step(Instant::now()).unwrap();
        assert_eq!(notify.queue.current(), None);
        drop(owner);
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

    /// A plugin shows a notice through the live API: the id comes at
    /// once, the notice shows after the next step, a second call with the
    /// id replaces it, a critical one waits for a click, and a close
    /// removes it. `state.get` lists what shows.
    #[test]
    fn the_methods_of_the_live_api_show_replace_and_close() {
        let Some((mut notify, _daemon)) = on_private_bus("methods") else {
            return;
        };
        let call = |notify: &mut Notify, method: &str, params: Value| {
            notify.method(method, params).expect("the call answers")
        };
        let summary = |notify: &Notify| notify.queue.current().map(|n| n.summary.clone());
        let first = call(
            &mut notify,
            kit_notice::SHOW,
            json!({"app": "probe", "summary": "One", "body": "text"}),
        );
        assert_eq!(first, json!({"id": 1}));
        notify.step(Instant::now()).unwrap();
        assert_eq!(summary(&notify).as_deref(), Some("One"));
        let again = call(
            &mut notify,
            kit_notice::SHOW,
            json!({"summary": "Two", "replaces": 1, "urgency": "critical"}),
        );
        assert_eq!(again, json!({"id": 1}));
        notify.step(Instant::now()).unwrap();
        assert_eq!(summary(&notify).as_deref(), Some("Two"));
        assert_eq!(notify.queue.current().map(|n| n.expire_ms), Some(0));
        call(&mut notify, kit_notice::CLOSE, json!({"id": 1}));
        notify.step(Instant::now()).unwrap();
        assert_eq!(summary(&notify), None);
        let err = notify
            .method(kit_notice::SHOW, json!({"body": "no title"}))
            .unwrap_err();
        assert_eq!(err.code, RpcError::INVALID_PARAMS);
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
    /// card, the ids that were accepted, that wait and that closed, the
    /// runs of text of the card, and nothing busy.
    #[test]
    fn the_state_shows_the_card_on_screen() {
        let new = || Queue::new(chora_kit::settings::parse(config::DEFAULT).unwrap());
        let mut queue = new();
        let now = Instant::now();
        queue.push(note(7, "Battery low"), now);
        queue.push(note(8, "Update"), now);
        queue.push(note(9, "Mail"), now);
        queue.close(9, Reason::Closed, now);
        let shown = state_value(queue.current().map(|n| (n, [8, 9, 300, 80])), &queue, &[]);
        assert_eq!(
            shown,
            serde_json::json!({
                "shown": [{"id": 7, "summary": "Battery low", "body": "text", "node": card::BODY,
                    "rect": [8, 9, 300, 80]}],
                "accepted": [7, 8, 9], "waiting": [8], "closed": [9],
                "texts": [], "busy": []})
        );
        let empty = state_value(None, &new(), &[]);
        assert_eq!(empty["shown"], serde_json::json!([]));
        assert_eq!(empty["accepted"], serde_json::json!([]));
    }

    /// `assistant.chat` of the HUD host names the box of its open chat;
    /// a closed chat or one without a box leaves the card where it is.
    #[test]
    fn the_chat_of_the_host_names_its_box() {
        let open = json!({"open": true, "name": "coder", "output": "HEADLESS-1",
            "rect": [856, 24, 400, 752]});
        assert_eq!(
            chat_box(&open),
            Some(("HEADLESS-1".to_owned(), [856, 24, 400, 752]))
        );
        let closed = json!({"open": false, "name": null, "output": "HEADLESS-1", "rect": null});
        assert_eq!(chat_box(&closed), None);
        assert_eq!(
            chat_box(&json!({"open": true, "output": "HEADLESS-1"})),
            None
        );
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
