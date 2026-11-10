//! chora-assistant: the state of the voice agent (D65) as a figure on the
//! first output, its speech in a bubble, the question card and the badge
//! of a muted mic. It subscribes to `voice.*` and draws into the root
//! `screen:NAME`. A click on the figure mutes the mic, a click on the badge
//! turns it on again.
//!
//! The host of the HUDs (`docs/hud.md`): the look comes from a HUD folder,
//! named by the setting `hud` and found by [`folders`], which the host reads
//! as untrusted data. The question card and the badge are the host's own.
//! `--assets`, `CHORA_ASSISTANT_ASSETS`, names the trusted asset folder of
//! a flavor instead (D73), until the flavors ship HUD folders.
//!
//! Three commands run without the host: `--check-hud DIR`, `--list-huds`
//! and `--use-hud NAME`.

#![allow(
    clippy::print_stderr,
    reason = "a plugin logs to stderr, which the session keeps"
)]

mod assistant;
mod card;
mod check;
mod config;
mod context;
mod folders;
mod paperclip;
mod voice;

use std::{
    collections::BTreeMap,
    io::Write as _,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use chora_draw::{
    assets,
    theme::{self, Sheet, Theme},
};
use chora_kit::{
    hud::{self, Context},
    runtime::{self, Plugin, Runtime},
    settings::{self, Store},
    words::Language,
};
use chora_scene::{
    Command, Event, OutputInfo,
    ipc::{ClientError, RpcError},
};
use clap::Parser;
use serde_json::Value;

use crate::{
    assistant::Assistant,
    card::Shown,
    config::{Config, DOT, Kind, NEUTRAL, Raw, Values},
    context::Switcher,
};

/// The client name of the plugin, the prefix of its methods.
const NAME: &str = "assistant";

/// The font family of what the host draws itself.
const HOST_FAMILY: &str = chora_draw::text::SANS_SERIF;

/// How long the loop waits for an event while nothing moves.
const IDLE: Duration = Duration::from_secs(60);

/// The events that the plugin handles.
const EVENTS: &[&str] = &[
    "voice.*",
    "focus.changed",
    "surface.changed",
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
    /// The asset folder of a flavor, with `values.yaml` and the pictures,
    /// which replaces the HUD folder of the setting `hud`.
    #[arg(long, env = "CHORA_ASSISTANT_ASSETS")]
    assets: Option<PathBuf>,
    /// The settings file. Without it, chora-assistant reads
    /// $XDG_CONFIG_HOME/chora/chora-assistant.yaml, by default in
    /// ~/.config, if it exists, or the default.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Checks the HUD folder DIR and prints each problem with its file and
    /// key; exits 1 when there is one.
    #[arg(long, value_name = "DIR", conflicts_with_all = ["list_huds", "use_hud"])]
    check_hud: Option<PathBuf>,
    /// Lists the HUD folders by name.
    #[arg(long, conflicts_with = "use_hud")]
    list_huds: bool,
    /// Sets `hud: NAME` in the settings file; `default` takes it out.
    #[arg(long, value_name = "NAME")]
    use_hud: Option<String>,
}

/// Where the look comes from, and where its rendered files go.
struct Source {
    assets: Option<PathBuf>,
    roots: Vec<PathBuf>,
    cache: PathBuf,
    loads: u32,
}

impl Source {
    /// Loads a look into a cache directory of its own, so that the files
    /// of the look on the screen stay until it is replaced. A HUD folder
    /// always goes through `Theme::load_hud`: it is untrusted.
    fn load(&mut self, hud: &str, context: &Context) -> Result<(Theme, Values, PathBuf), String> {
        self.loads += 1;
        let cache = self.cache.join(format!("look-{}", self.loads));
        let (theme, values) = match &self.assets {
            Some(dir) => load_assets(dir, &cache)?,
            None => load_hud(&self.roots, hud, context, &cache)?,
        };
        Ok((theme, values, cache))
    }
}

/// The plugin.
struct Agent {
    rt: Runtime<()>,
    settings: settings::File<Config>,
    source: Source,
    theme: Theme,
    values: Values,
    /// The name of the HUD folder on the screen.
    folder: String,
    /// The context that the context of the screen is evaluated in.
    switcher: Switcher,
    /// The context that the look on the screen was built for.
    look_context: Context,
    /// The state of each helper that the agent named.
    helpers: BTreeMap<String, &'static str>,
    /// The cache directory of the look on the screen.
    look_cache: PathBuf,
    /// The theme of the card and the badge, with the fonts of the host.
    card_theme: Theme,
    language: Language,
    assistant: Option<Assistant>,
    /// The client name of the voice agent and its state, while it is
    /// connected.
    voice: Option<(String, &'static str)>,
    /// The mic is muted.
    muted: bool,
    /// The question that waits, if one does.
    card: Option<Shown>,
    /// The badge of a muted mic shows.
    badge: bool,
    /// The output that has the group of the card and the badge.
    top: Option<String>,
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
            Event::PointerPressed { node, injected, .. } => self.pressed(&node, injected)?,
            Event::OutputChanged { .. } | Event::OutputRemoved { .. } => {
                self.switcher.touch(Instant::now());
                self.replace_assistant()?
            }
            Event::FocusChanged { surface } => {
                self.switcher
                    .focus_changed(surface.as_deref(), Instant::now());
                Vec::new()
            }
            Event::SurfaceChanged {
                surface, app_id, ..
            } => {
                let now = Instant::now();
                self.switcher
                    .surface_changed(&surface, app_id.as_deref(), now);
                Vec::new()
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
        let rejected = |e: String| RpcError::new(RpcError::REJECTED, e);
        let context = self.switcher.applied().clone();
        let folder = self.settings.value.folder(&context).to_owned();
        let (theme, values, cache) = self.source.load(&folder, &context).map_err(rejected)?;
        self.theme = theme;
        self.values = values;
        self.folder = folder;
        self.look_context = context;
        let old = std::mem::replace(&mut self.look_cache, cache);
        let result = self
            .replace_assistant()
            .and_then(|commands| self.apply(commands))
            .map_err(|e| rejected(e.to_string()));
        let _ = std::fs::remove_dir_all(old);
        result
    }
}

impl Agent {
    /// Connects, subscribes to the events of the voice agent and exports
    /// the methods of the settings, and shows the figure if the agent is
    /// awake.
    fn start(
        (mut source, theme, values, look_cache): (Source, Theme, Values, PathBuf),
        (settings, folder): (settings::File<Config>, String),
    ) -> Result<Self, String> {
        let card_theme = card::theme(&source.cache, HOST_FAMILY)?;
        source.loads = source.loads.max(1);
        let rt = Runtime::start(
            NAME,
            Vec::new(),
            EVENTS,
            settings::File::<Config>::methods(),
        )
        .map_err(|e| e.to_string())?;
        let mut agent = Self {
            rt,
            settings,
            source,
            theme,
            values,
            folder,
            switcher: Switcher::new(),
            look_context: Context::default(),
            helpers: BTreeMap::new(),
            look_cache,
            card_theme,
            language: language(),
            assistant: None,
            voice: None,
            muted: false,
            card: None,
            badge: false,
            top: None,
            claim_due: true,
        };
        agent.look_context = agent.switcher.applied().clone();
        agent.evaluate(Instant::now()).map_err(|e| e.to_string())?;
        agent.ask_voice().map_err(|e| e.to_string())?;
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

    /// The output of the focused window, which shows the figure (the first
    /// one before the first evaluation), and the largest scale of the
    /// outputs.
    fn output(&mut self) -> Result<Option<(OutputInfo, f64)>, ClientError> {
        let outputs = self.rt.outputs()?;
        let scale = outputs.iter().map(|o| o.scale).fold(1.0, f64::max);
        let wanted = self.switcher.applied().output.as_str();
        let at = outputs.iter().position(|o| o.name == wanted).unwrap_or(0);
        Ok(outputs.into_iter().nth(at).map(|o| (o, scale)))
    }

    /// A press on the figure mutes the mic, one on the badge turns it on, and
    /// one on a button of the question answers it.
    fn pressed(&mut self, node: &str, injected: bool) -> Result<Vec<Command>, ClientError> {
        let (handled, commands) = self.press_card(node, injected)?;
        if handled {
            return Ok(commands);
        }
        if node == card::BADGE {
            return Ok(self.press_badge(injected));
        }
        if Assistant::owns(node) {
            return Ok(self.press_assistant(injected));
        }
        Ok(Vec::new())
    }
}

/// The language of the words on the screen, from the locale variables.
fn language() -> Language {
    let tag = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());
    tag.map_or(Language::En, |tag| Language::from_tag(&tag))
}

fn main() -> ExitCode {
    let args = Args::parse();
    if let Some(code) = tools(&args) {
        return code;
    }
    runtime::run(NAME, || serve(&args))
}

/// The commands that run without the host, with their exit code.
fn tools(args: &Args) -> Option<ExitCode> {
    let exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    let roots = folders::roots(exe.as_deref());
    let mut out = std::io::stdout();
    if let Some(dir) = &args.check_hud {
        let (text, fine) = check::check_report(dir);
        let _ = out.write_all(text.as_bytes());
        return Some(if fine {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        });
    }
    if args.list_huds {
        let _ = out.write_all(check::list_report(&roots).as_bytes());
        return Some(ExitCode::SUCCESS);
    }
    let name = args.use_hud.as_ref()?;
    let done = settings::File::<Config>::path(args.config.clone(), "chora-assistant")
        .and_then(|file| check::use_hud(name, &roots, file));
    Some(match done {
        Ok(text) => {
            let _ = out.write_all(text.as_bytes());
            ExitCode::SUCCESS
        }
        Err(text) => {
            eprintln!("chora-assistant: {text}");
            ExitCode::FAILURE
        }
    })
}

/// One connection to the core, until it closes.
fn serve(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings::File::<Config>::path(args.config.clone(), "chora-assistant")?;
    let settings = settings::File::<Config>::load(path, config::DEFAULT)?;
    let exe = std::env::current_exe().and_then(std::fs::canonicalize).ok();
    let mut source = Source {
        assets: args.assets.clone(),
        roots: folders::roots(exe.as_deref()),
        cache: theme::cache_dir("chora-assistant")?,
        loads: 0,
    };
    let context = Switcher::new().applied().clone();
    let mut name = settings.value.folder(&context).to_owned();
    let (theme, values, cache) = match source.load(&name, &context) {
        Ok(look) => look,
        Err(err) if source.assets.is_none() && name != NEUTRAL => {
            eprintln!("chora-assistant: hud {name}: {err}; the neutral folder shows");
            NEUTRAL.clone_into(&mut name);
            source.load(NEUTRAL, &context)?
        }
        Err(err) => return Err(err.into()),
    };
    let mut agent = Agent::start((source, theme, values, cache), (settings, name))?;
    loop {
        let now = Instant::now();
        let wait = [agent.assistant_due(), agent.switch_due(now)]
            .into_iter()
            .flatten()
            .min()
            .map_or(IDLE, |due| due.saturating_duration_since(now));
        if let Some(event) = agent
            .rt
            .conn
            .next_event_timeout(wait.max(Duration::from_millis(1)))?
        {
            runtime::dispatch(&mut agent, event)?;
        }
        agent.assistant_step(Instant::now())?;
        agent.switch_step(Instant::now())?;
    }
}

/// The values and the pictures of an asset folder of a flavor, which the
/// flavor ships and the host trusts. A missing or broken file stops the
/// start with an error that names it.
fn load_assets(dir: &Path, cache: &Path) -> Result<(Theme, Values), String> {
    let raw: Raw = assets::values(dir)?;
    let sheets: &[Sheet] = match raw.figure {
        Kind::Dot => &[DOT],
        Kind::Paperclip => &[],
    };
    let theme = Theme::load(dir, cache, sheets, &raw.font)?;
    let values = Values::from_assets(raw, |sheet| theme.sheet_size(sheet));
    Ok((theme, values))
}

/// The values and the pictures of the HUD folder `name`, read as untrusted
/// data: only through `Theme::load_hud`.
fn load_hud(
    roots: &[PathBuf],
    name: &str,
    context: &Context,
    cache: &Path,
) -> Result<(Theme, Values), String> {
    let dir = folders::find(roots, name)
        .ok_or_else(|| format!("no HUD folder named {name}; --list-huds lists them"))?;
    let hud = hud::load(&dir).map_err(|errors| errors.join("; "))?;
    let mut files: Vec<&str> = hud
        .panels
        .iter()
        .filter_map(|panel| panel.picture.as_deref())
        .filter_map(|picture| picture.split('#').next())
        .collect();
    files.sort_unstable();
    files.dedup();
    let sheets: Vec<Sheet> = files
        .into_iter()
        .map(|file| Sheet(Box::leak(file.to_owned().into_boxed_str())))
        .collect();
    let theme = Theme::load_hud(&dir, cache, &sheets, &hud.font)?;
    let values = Values::from_hud(&hud, context, |sheet| theme.sheet_size(sheet));
    Ok((theme, values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Figure;

    fn cache(test: &str) -> PathBuf {
        std::env::temp_dir().join(format!("chora-assistant-{test}-{}", std::process::id()))
    }

    fn huds() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("huds")
    }

    /// The theme and the values of the neutral folder, and the cache.
    pub fn loaded(test: &str) -> (Theme, Values, PathBuf) {
        let cache = cache(test);
        let (theme, values) = load_hud(&[huds()], NEUTRAL, &Context::default(), &cache)
            .expect("the neutral folder loads");
        (theme, values, cache)
    }

    #[test]
    fn the_neutral_folder_loads() {
        let (theme, values, cache) = loaded("neutral");
        assert!(matches!(
            values.figure,
            Figure::Sheet { size: [24, 24], .. }
        ));
        assert_eq!(values.anchor, chora_kit::hud::Anchor::BottomRight);
        assert_eq!(values.margin, [16, 16]);
        assert_eq!(values.speech.tail, 0, "no tail");
        assert!(values.mutes);
        assert!(theme.confined());
        let _ = std::fs::remove_dir_all(cache);
    }

    #[test]
    fn a_hud_folder_is_never_loaded_as_a_trusted_asset_folder() {
        let mut source = Source {
            assets: None,
            roots: vec![huds()],
            cache: cache("source"),
            loads: 0,
        };
        let (theme, ..) = source
            .load(NEUTRAL, &Context::default())
            .expect("the neutral folder loads");
        assert!(theme.confined(), "a HUD folder is read as untrusted");
        assert_eq!(
            source
                .load("nothing", &Context::default())
                .err()
                .map(|e| e.contains("--list-huds")),
            Some(true)
        );
        let xp = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flavors/xp/assets/assistant");
        source.assets = Some(xp);
        let (theme, ..) = source
            .load(NEUTRAL, &Context::default())
            .expect("the XP folder loads");
        assert!(!theme.confined(), "the asset folder of a flavor is trusted");
        let _ = std::fs::remove_dir_all(source.cache);
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

    #[test]
    fn a_broken_folder_names_its_errors_and_the_host_keeps_to_them() {
        let fancy = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/hud");
        let err = load_hud(&[fancy], "fancy", &Context::default(), &cache("fancy"))
            .err()
            .expect("refused");
        assert!(err.contains("link.svg"), "{err}");
    }

    fn write_hostile_folder(dir: &Path) {
        std::fs::create_dir_all(dir).unwrap();
        let panels: String = (0..30)
            .map(|n| format!("- {{id: p{n}, source: say, anchor: center, text: x}}\n"))
            .collect::<Vec<_>>()
            .concat();
        let yaml = format!(
            "hud: 1\npanels:\n- {{id: f, source: voice, anchor: center, \
             picture: 'full.svg#{{state}}'}}\n{panels}"
        );
        std::fs::write(dir.join("hud.yaml"), yaml).unwrap();
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1280" height="800">
            <rect id="idle" width="1280" height="800" fill="red"/></svg>"#;
        std::fs::write(dir.join("full.svg"), svg).unwrap();
    }

    #[test]
    fn a_folder_cannot_hide_the_question() {
        let root = cache("hostile");
        write_hostile_folder(&root.join("huds/hostile"));
        let (mut theme, values) = load_hud(
            &[root.join("huds")],
            "hostile",
            &Context::default(),
            &root.join("cache"),
        )
        .unwrap();
        let output = chora_scene::OutputInfo {
            name: "HEADLESS-1".into(),
            position: [0, 0],
            size: [1280, 800],
            scale: 1.0,
            camera: chora_scene::Transform::default(),
            work_area: [0, 0, 1280, 800],
            default_work_area: [0, 0, 1280, 800],
        };
        let now = Instant::now();
        let (figure, commands) = Assistant::new(
            &mut theme,
            &values,
            (&output, values.anchor),
            ("question", "Question", now),
        );
        let [_, _, w, h] = figure.rect();
        assert!(
            w <= 256 && h <= 256,
            "{w} by {h}: a folder cannot cover the output"
        );
        let host = card::theme(&root.join("cache"), HOST_FAMILY).unwrap();
        let data = serde_json::json!({"id": "1", "prompt": "Write notes.txt?", "buttons": true});
        let question = chora_kit::ask::Question::from_event(&data, None).unwrap();
        let mut waiting = Shown {
            id: question.answer_id.clone(),
            card: chora_kit::ask::Card::new(question, now),
            buttons: true,
            output: output.name.clone(),
            origin: [0, 0],
            notice: false,
            too_long: false,
        };
        assert!(card::fits(&host, &waiting, Language::En, output.size));
        let size = card::size(&host, &waiting, Language::En, output.size);
        waiting.origin = card::place(figure.rect(), size, assistant::work_area(&output));
        let rect = card::rect(&host, &waiting, Language::En, output.size);
        assert!(card::inside(rect, output.size), "{rect:?}");
        let mut host = host;
        let shown = card::show(&mut host, &waiting, Language::En, output.size, true);
        let ours: Vec<&str> = commands
            .iter()
            .filter_map(|c| match c {
                chora_scene::Command::Create { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        assert!(!ours.contains(&card::CARD) && !ours.contains(&card::TOP));
        let last = shown.last().expect("commands");
        assert!(matches!(last, chora_scene::Command::Create { parent, .. } if parent == card::TOP));
        let _ = std::fs::remove_dir_all(root);
    }
}
