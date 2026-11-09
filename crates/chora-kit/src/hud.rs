//! The HUD folder format, version 1: a folder with `hud.yaml`, SVG
//! pictures and optional fonts, that the host plugin `chora-assistant`
//! draws. The module is the pure model: the manifest types, the limits of
//! a folder, the context, `when` and the templates. Nothing in a folder
//! runs code, and nothing here draws or talks to a socket.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};

use serde::Deserialize;

/// The only version of the format that this module reads.
pub const VERSION: u32 = 1;
/// The most bytes of `hud.yaml`.
pub const MAX_YAML: u64 = 64 << 10;
/// The most panels of one folder.
pub const MAX_PANELS: usize = 32;
/// The most bytes of one file of a folder.
pub const MAX_FILE: u64 = 1 << 20;
/// The most bytes of all files of a folder.
pub const MAX_FOLDER: u64 = 8 << 20;
/// The most entries (files and folders) in a folder.
pub const MAX_ENTRIES: usize = 256;
/// The deepest level of folders inside a folder; files of the folder
/// itself are at depth 1.
pub const MAX_DEPTH: usize = 4;
/// The most characters of the id of a panel.
pub const MAX_ID: usize = 32;
/// The smallest text size of the chat.
pub const MIN_CHAT_SIZE: u32 = 14;
/// The largest text size of the chat.
pub const MAX_CHAT_SIZE: u32 = 48;
/// The widest chat, in pixels.
pub const MAX_CHAT_WIDTH: u32 = 1600;
/// The most characters of `max_chars`.
pub const MAX_CHARS: u32 = 2000;
/// The most rows of `max_rows`.
pub const MAX_ROWS: u32 = 200;

/// The keys that `when` may compare, and the fields of [`Context`].
pub const CONTEXT_KEYS: [&str; 7] = ["wm", "space", "app", "state", "agents", "chat", "output"];

/// The fields of the context with a closed list of values; the other
/// fields are names that the system gives.
pub const CONTEXT_VALUES: [(&str, &[&str]); 3] = [
    (
        "state",
        &[
            "asleep",
            "listening",
            "thinking",
            "working",
            "question",
            "speaking",
        ],
    ),
    ("agents", &["none", "any", "working", "question"]),
    ("chat", &["open", "none"]),
];

/// A parsed and checked `hud.yaml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hud {
    /// The version of the format.
    pub hud: u32,
    #[serde(default = "default_font")]
    /// The default font family of the folder.
    pub font: String,
    #[serde(default)]
    /// The panels, at most [`MAX_PANELS`].
    pub panels: Vec<Panel>,
    /// The look of the chat; a folder without the block gets the neutral
    /// chat.
    #[serde(default)]
    pub chat: Chat,
}

fn default_font() -> String {
    "sans-serif".into()
}

/// One panel of a folder.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    /// The name that `near` refers to; unique in the folder, 1 to
    /// [`MAX_ID`] characters of `a-z`, `0-9`, `_` and `-`.
    pub id: String,
    /// Where the data comes from.
    pub source: Source,
    /// `CLIENT.EVENT`, for and only for `source: event`.
    pub event: Option<String>,
    #[serde(default)]
    /// The panel shows only when this matches the context.
    pub when: When,
    /// The place in the work area.
    pub anchor: Option<Anchor>,
    /// The gap to the edges, horizontal and vertical.
    pub margin: Option<[i32; 2]>,
    /// The id of another panel that this one stands beside; the chain of
    /// `near` ends at a panel with an anchor.
    pub near: Option<String>,
    /// `FILE#ID`: an SVG file of the folder and an element id in it, a
    /// template.
    pub picture: Option<String>,
    /// What a click does.
    pub action: Option<Action>,
    /// The shape of the text.
    pub style: Option<Style>,
    /// A template for the text.
    pub text: Option<String>,
    /// The most characters of the text, from 1 to [`MAX_CHARS`].
    pub max_chars: Option<u32>,
    /// A template for one row of a list.
    pub rows: Option<String>,
    /// The most rows of a list, from 1 to [`MAX_ROWS`].
    pub max_rows: Option<u32>,
}

/// Where the data of a panel comes from: a closed list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The state of the voice agent.
    Voice,
    /// The sentence of `voice.say`.
    Say,
    /// The helpers of `voice.state`.
    Agents,
    /// The open chats.
    Chat,
    /// A field of the context.
    Context,
    /// The time.
    Clock,
    /// An event of a named client.
    Event,
}

/// The nine places of a panel in the work area of an output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    /// Top left corner.
    TopLeft,
    /// Top edge, centred.
    Top,
    /// Top right corner.
    TopRight,
    /// Left edge, centred.
    Left,
    /// The middle.
    Center,
    /// Right edge, centred.
    Right,
    /// Bottom left corner.
    BottomLeft,
    /// Bottom edge, centred.
    Bottom,
    /// Bottom right corner.
    BottomRight,
}

/// What a click on a panel does; nothing else exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Mutes the microphone; never unmutes.
    Mute,
    /// Cancels the running turn.
    Cancel,
    /// Puts the agent to sleep.
    Sleep,
    /// Wakes the agent.
    Wake,
    /// Opens the chat.
    Chat,
}

/// The shape of the text of a panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    /// A rounded box with an optional tail.
    Bubble,
    /// Text without a box.
    Plain,
}

/// The block `chat:`: values only.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Chat {
    /// The shape of the view.
    pub style: ChatStyle,
    /// Colours are `[r, g, b]`; a colour left out takes the host's.
    pub fill: Option<[u8; 3]>,
    /// Text colour.
    pub text: Option<[u8; 3]>,
    /// Colour of the names of the speakers.
    pub name: Option<[u8; 3]>,
    /// Colour of lines and edges.
    pub border: Option<[u8; 3]>,
    /// Font family; the folder's `font` if left out.
    pub font: Option<String>,
    /// Text size in pixels, from [`MIN_CHAT_SIZE`] to [`MAX_CHAT_SIZE`].
    pub size: u32,
    /// The place; the right edge if left out.
    pub anchor: Option<Anchor>,
    /// The width in pixels, at most [`MAX_CHAT_WIDTH`]; 400 if left out.
    pub width: Option<u32>,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            style: ChatStyle::Plain,
            fill: None,
            text: None,
            name: None,
            border: None,
            font: None,
            size: MIN_CHAT_SIZE,
            anchor: None,
            width: None,
        }
    }
}

/// The shape of the chat view.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatStyle {
    /// Text on the plain background.
    Plain,
    /// Rounded boxes for the lines.
    Bubble,
    /// A glass panel.
    Glass,
}

impl Chat {
    /// The anchor and the width of the view: the right edge, 400 wide, if
    /// left out.
    #[must_use]
    pub fn place(&self) -> (Anchor, u32) {
        (
            self.anchor.unwrap_or(Anchor::Right),
            self.width.unwrap_or(400),
        )
    }

    fn check(&self, errors: &mut Vec<String>) {
        if !(MIN_CHAT_SIZE..=MAX_CHAT_SIZE).contains(&self.size) {
            errors.push(format!(
                "chat.size: {} is not from {MIN_CHAT_SIZE} to {MAX_CHAT_SIZE}",
                self.size
            ));
        }
        if let Some(width) = self.width.filter(|w| !(1..=MAX_CHAT_WIDTH).contains(w)) {
            errors.push(format!(
                "chat.width: {width} is not from 1 to {MAX_CHAT_WIDTH}"
            ));
        }
    }
}

/// The context that `when` compares; see the table of fields in the
/// design. Every field is text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// The window manager.
    pub wm: String,
    /// The space of the focused window.
    pub space: String,
    /// The `app_id` of the focused window.
    pub app: String,
    /// The state of the voice agent.
    pub state: String,
    /// `none`, `any`, `working` or `question`.
    pub agents: String,
    /// `open` or `none`.
    pub chat: String,
    /// The output of the focused window.
    pub output: String,
}

impl Context {
    /// The value of a field, `None` for a name that is no field.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        Some(match key {
            "wm" => &self.wm,
            "space" => &self.space,
            "app" => &self.app,
            "state" => &self.state,
            "agents" => &self.agents,
            "chat" => &self.chat,
            "output" => &self.output,
            _ => return None,
        })
    }
}

/// One value or a list of values.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Match {
    /// A value.
    One(String),
    /// A list of values; one of them matches.
    Many(Vec<String>),
}

/// The condition `when`: each key names a field of the context and must
/// equal the value, or one of a list. All keys must match.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct When(pub BTreeMap<String, Match>);

impl When {
    /// Whether every key matches the context. An empty `when` matches.
    #[must_use]
    pub fn matches(&self, context: &Context) -> bool {
        self.0.iter().all(|(key, wanted)| {
            context.get(key).is_some_and(|value| match wanted {
                Match::One(one) => one == value,
                Match::Many(list) => list.iter().any(|item| item == value),
            })
        })
    }
}

/// Fills `{field}` and `{field.sub}` of a template, in one pass: the text
/// of a value is inserted as it is and never expanded again. A field that
/// `lookup` does not know stays empty.
pub fn expand(template: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let name = after
            .find(['{', '}'])
            .filter(|&end| after.as_bytes()[end] == b'}')
            .map(|end| &after[..end])
            .filter(|name| is_field(name));
        if let Some(name) = name {
            out.push_str(&lookup(name).unwrap_or_default());
            rest = &after[name.len() + 1..];
        } else {
            out.push('{');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

fn is_field(name: &str) -> bool {
    !name.is_empty()
        && name.split('.').all(|part| {
            !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// [`expand`] with fields in a JSON value; `{a.b}` reads `a` and then
/// `b`. Only text, numbers and booleans have a text.
#[must_use]
pub fn render(template: &str, fields: &serde_json::Value) -> String {
    expand(template, |path| {
        let found = path
            .split('.')
            .try_fold(fields, |value, part| value.get(part))?;
        match found {
            serde_json::Value::String(text) => Some(text.clone()),
            serde_json::Value::Number(_) | serde_json::Value::Bool(_) => Some(found.to_string()),
            _ => None,
        }
    })
}

/// Parses and checks the text of a `hud.yaml`. Each error names the key.
///
/// # Errors
///
/// A version other than 1, an unknown key, a wrong value, or a failed
/// check; the list has every failed check.
pub fn parse(text: &str) -> Result<Hud, Vec<String>> {
    #[derive(Deserialize)]
    struct Version {
        hud: Option<u32>,
    }
    let mut budget = serde_saphyr::Budget::default();
    budget.max_aliases = 0;
    budget.max_anchors = 0;
    let mut options = serde_saphyr::Options::default();
    options.budget = Some(budget);
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    let fail = |e: serde_saphyr::Error| vec![format!("hud.yaml: {e}")];
    let version: Version =
        serde_saphyr::from_str_with_options(text, options.clone()).map_err(fail)?;
    match version.hud {
        Some(VERSION) => {}
        Some(other) => {
            return Err(vec![format!(
                "hud.yaml: hud: {other} is not supported, this version reads {VERSION}"
            )]);
        }
        None => {
            return Err(vec![format!(
                "hud.yaml: hud: missing, write `hud: {VERSION}`"
            )]);
        }
    }
    let hud: Hud = serde_saphyr::from_str_with_options(text, options).map_err(fail)?;
    let errors = hud.check();
    if errors.is_empty() {
        Ok(hud)
    } else {
        Err(errors
            .into_iter()
            .map(|e| format!("hud.yaml: {e}"))
            .collect())
    }
}

impl Hud {
    fn check(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.panels.len() > MAX_PANELS {
            errors.push(format!(
                "panels: {} panels, at most {MAX_PANELS}",
                self.panels.len()
            ));
        }
        let ids: BTreeSet<&str> = self.panels.iter().map(|p| p.id.as_str()).collect();
        let mut seen = BTreeSet::new();
        for (index, panel) in self.panels.iter().enumerate() {
            panel.check(&format!("panels[{index}]"), &ids, &mut errors);
            if !seen.insert(panel.id.as_str()) {
                errors.push(format!("panels[{index}].id: {:?} is used twice", panel.id));
            }
        }
        for (index, panel) in self.panels.iter().enumerate() {
            if panel.near.is_some() && !self.reaches_anchor(panel) {
                errors.push(format!(
                    "panels[{index}].near: the chain of near never ends at a panel with an anchor"
                ));
            }
        }
        self.chat.check(&mut errors);
        errors
    }

    fn reaches_anchor(&self, start: &Panel) -> bool {
        let mut panel = start;
        for _ in 0..=self.panels.len() {
            let Some(near) = &panel.near else {
                return panel.anchor.is_some();
            };
            let Some(next) = self.panels.iter().find(|p| &p.id == near) else {
                return false;
            };
            panel = next;
        }
        false
    }
}

fn fail(errors: &mut Vec<String>, at: &str, key: &str, message: &str) {
    errors.push(format!("{at}.{key}: {message}"));
}

impl Panel {
    fn check(&self, at: &str, ids: &BTreeSet<&str>, errors: &mut Vec<String>) {
        self.check_id(at, errors);
        self.check_content(at, errors);
        self.check_place(at, ids, errors);
        self.check_source(at, errors);
        self.check_when(at, errors);
    }

    fn check_id(&self, at: &str, errors: &mut Vec<String>) {
        let plain = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-';
        if !(1..=MAX_ID).contains(&self.id.chars().count()) || !self.id.chars().all(plain) {
            let message = format!(
                "{:?} is not 1 to {MAX_ID} characters of a-z, 0-9, _ and -",
                self.id
            );
            fail(errors, at, "id", &message);
        }
    }

    fn check_content(&self, at: &str, errors: &mut Vec<String>) {
        if self.picture.is_none() && self.text.is_none() && self.rows.is_none() {
            let message = "a panel needs `picture`, `text` or `rows`";
            fail(errors, at, "picture", message);
        }
        let file_name = self.picture.as_deref().and_then(|p| p.split('#').next());
        if file_name.is_some_and(|file| file.contains('{')) {
            let message = "the file name has no `{`; only the id after `#` is a template";
            fail(errors, at, "picture", message);
        }
        for (key, value, most) in [
            ("max_chars", self.max_chars, MAX_CHARS),
            ("max_rows", self.max_rows, MAX_ROWS),
        ] {
            if let Some(value) = value.filter(|v| !(1..=most).contains(v)) {
                fail(errors, at, key, &format!("{value} is not from 1 to {most}"));
            }
        }
    }

    fn check_place(&self, at: &str, ids: &BTreeSet<&str>, errors: &mut Vec<String>) {
        match (self.anchor, &self.near) {
            (None, None) => fail(errors, at, "anchor", "needs `anchor` or `near`"),
            (Some(_), Some(_)) => fail(errors, at, "near", "not together with anchor"),
            (None, Some(near)) if near == &self.id || !ids.contains(near.as_str()) => {
                fail(errors, at, "near", &format!("no other panel {near:?}"));
            }
            _ => {}
        }
    }

    fn check_source(&self, at: &str, errors: &mut Vec<String>) {
        match (&self.event, self.source) {
            (Some(event), Source::Event) => {
                let named = event
                    .split_once('.')
                    .is_some_and(|(client, name)| !client.is_empty() && !name.is_empty());
                if !named {
                    fail(
                        errors,
                        at,
                        "event",
                        &format!("{event:?} is not CLIENT.EVENT"),
                    );
                }
            }
            (None, Source::Event) => {
                fail(errors, at, "event", "source event needs CLIENT.EVENT");
            }
            (Some(_), _) => fail(errors, at, "event", "only for source: event"),
            (None, _) => {}
        }
        if self.action == Some(Action::Mute) && self.source != Source::Voice {
            fail(errors, at, "action", "mute needs source: voice");
        }
    }

    fn check_when(&self, at: &str, errors: &mut Vec<String>) {
        for (key, wanted) in &self.when.0 {
            let key_at = &format!("when.{key}");
            if !CONTEXT_KEYS.contains(&key.as_str()) {
                fail(errors, at, key_at, "not a field of the context");
                continue;
            }
            let Some((_, known)) = CONTEXT_VALUES.iter().find(|(name, _)| name == key) else {
                continue;
            };
            let values = match wanted {
                Match::One(one) => std::slice::from_ref(one),
                Match::Many(list) => list.as_slice(),
            };
            for value in values.iter().filter(|v| !known.contains(&v.as_str())) {
                let message = format!("{value:?} is not one of {}", known.join(", "));
                fail(errors, at, key_at, &message);
            }
        }
    }
}

/// Loads the folder `dir`: the limits, the links, `hud.yaml` and the
/// pictures that it names. `hud.yaml` is parsed only after the scan of
/// the folder passed.
///
/// # Errors
///
/// Each error names its file and, for `hud.yaml`, its key.
pub fn load(dir: &Path) -> Result<Hud, Vec<String>> {
    let errors = scan(dir);
    if !errors.is_empty() {
        return Err(errors);
    }
    let text = file(dir, "hud.yaml")
        .and_then(|data| {
            if data.len() as u64 > MAX_YAML {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("larger than {MAX_YAML} bytes"),
                ));
            }
            String::from_utf8(data)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
        .map_err(|e| vec![format!("hud.yaml: {e}")])?;
    let hud = parse(&text)?;
    let mut errors = Vec::new();
    for (index, panel) in hud.panels.iter().enumerate() {
        let Some(picture) = &panel.picture else {
            continue;
        };
        let name = picture.split('#').next().unwrap_or_default();
        if let Err(e) = file(dir, name) {
            errors.push(format!("panels[{index}].picture: {name}: {e}"));
        }
    }
    if errors.is_empty() {
        Ok(hud)
    } else {
        Err(errors)
    }
}

/// Checks every entry of a folder against the limits and the rule for
/// links: a link must lead to a regular file inside the folder, and a
/// file has one name only.
fn scan(root: &Path) -> Vec<String> {
    let real = match std::fs::canonicalize(root) {
        Ok(real) => real,
        Err(e) => return vec![format!("{}: {e}", root.display())],
    };
    let mut errors = Vec::new();
    let mut total = 0_u64;
    let mut entries_seen = 0_usize;
    let mut pending = vec![(real.clone(), 1_usize)];
    while let Some((dir, depth)) = pending.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                errors.push(format!("{}: {e}", dir.display()));
                continue;
            }
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            entries_seen += 1;
            if entries_seen > MAX_ENTRIES {
                errors.push(format!("folder: more than {MAX_ENTRIES} entries"));
                return errors;
            }
            let name = path
                .strip_prefix(&real)
                .unwrap_or(&path)
                .display()
                .to_string();
            match entry_size(&path, &real) {
                Ok(None) if depth >= MAX_DEPTH => {
                    errors.push(format!("{name}: depth over {MAX_DEPTH}"));
                }
                Ok(None) => pending.push((path, depth + 1)),
                Ok(Some(size)) if size > MAX_FILE => {
                    errors.push(format!("{name}: larger than {MAX_FILE} bytes"));
                }
                Ok(Some(size)) => total = total.saturating_add(size),
                Err(message) => errors.push(format!("{name}: {message}")),
            }
        }
    }
    if total > MAX_FOLDER {
        errors.push(format!("folder: more than {MAX_FOLDER} bytes"));
    }
    errors
}

/// The size of a file, `None` for a folder to enter; an error for a link
/// that leaves the folder or does not lead to a file, and for a device.
fn entry_size(path: &Path, real: &Path) -> Result<Option<u64>, String> {
    let kind = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if kind.is_dir() {
        return Ok(None);
    }
    if kind.is_file() {
        return single_name(&kind).map(|()| Some(kind.len()));
    }
    if !kind.is_symlink() {
        return Err("not a regular file".into());
    }
    let target = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    if !target.starts_with(real) {
        return Err("link points outside the folder".into());
    }
    match std::fs::metadata(&target) {
        Ok(meta) if meta.is_file() => single_name(&meta).map(|()| Some(meta.len())),
        _ => Err("link to something but a file".into()),
    }
}

fn single_name(meta: &std::fs::Metadata) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt as _;
    if meta.nlink() > 1 {
        return Err("file with more than one hard link".into());
    }
    Ok(())
}

/// Reads one file of a folder: a relative path inside it, a regular file
/// of at most [`MAX_FILE`] bytes, also after links are resolved.
///
/// # Errors
///
/// The path leaves the folder, or the file cannot be read.
pub fn file(dir: &Path, name: &str) -> std::io::Result<Vec<u8>> {
    let invalid =
        |message: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, message.to_owned());
    let relative = Path::new(name);
    if name.is_empty()
        || !relative
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(invalid("not a path inside the folder"));
    }
    let root = std::fs::canonicalize(dir)?;
    let real = std::fs::canonicalize(root.join(relative))?;
    if !real.starts_with(&root) {
        return Err(invalid("link points outside the folder"));
    }
    single_name(&std::fs::metadata(&real)?).map_err(|e| invalid(&e))?;
    crate::read_file(&real, MAX_FILE)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const YAML_BYTES: usize = 64 << 10;
    const FILE_BYTES: usize = 1 << 20;
    const MINIMAL: &str = "hud: 1\n";

    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("chora-hud-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn put(&self, name: &str, data: &[u8]) {
            std::fs::write(self.0.join(name), data).unwrap();
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn errors(result: Result<Hud, Vec<String>>) -> String {
        result.unwrap_err().join("\n")
    }

    #[test]
    fn a_full_folder_parses() {
        let hud = parse(
            "hud: 1\nfont: Tahoma\npanels:\n  - {id: figure, source: voice, anchor: bottom_right, \
             margin: [16, 48], picture: 'dot.svg#{state}', action: mute}\n  - {id: line, source: \
             say, near: figure, style: bubble, text: '{text}', max_chars: 160}\n  - {id: ev, \
             source: event, event: mc.chat, anchor: top, text: x}\nchat: {style: glass, size: 16, anchor: \
             right, width: 480}\n",
        )
        .unwrap();
        assert_eq!(hud.panels.len(), 3);
        assert_eq!(hud.chat.place(), (Anchor::Right, 480));
    }

    #[test]
    fn unknown_key_fails() {
        let top = errors(parse("hud: 1\nscript: x\n"));
        assert!(top.contains("script"), "{top}");
        let panel = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, text: x, run: sh}\n",
        ));
        assert!(panel.contains("run"), "{panel}");
        let chat = errors(parse("hud: 1\nchat: {colour: red}\n"));
        assert!(chat.contains("colour"), "{chat}");
        let when = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, text: x, when: {mood: calm}}\n",
        ));
        assert!(when.contains("when.mood"), "{when}");
        let source = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: shell, anchor: top, text: x}\n",
        ));
        assert!(source.contains("shell"), "{source}");
    }

    #[test]
    fn version_2_is_refused() {
        let two = errors(parse("hud: 2\nunknown: 1\n"));
        assert!(two.contains("hud: 2"), "{two}");
        assert!(two.contains('1'), "{two}");
        assert!(errors(parse("panels: []\n")).contains("hud"));
        assert!(parse(MINIMAL).is_ok());
    }

    #[test]
    fn a_symlink_out_of_the_folder_is_refused() {
        let outside = Dir::new("outside");
        outside.put("secret.svg", b"<svg/>");
        let dir = Dir::new("link");
        dir.put("hud.yaml", MINIMAL.as_bytes());
        dir.put("inside.svg", b"<svg/>");
        std::os::unix::fs::symlink(outside.0.join("secret.svg"), dir.0.join("out.svg")).unwrap();
        let refused = errors(load(&dir.0));
        assert!(refused.contains("out.svg"), "{refused}");
        assert!(refused.contains("outside"), "{refused}");
        assert!(file(&dir.0, "out.svg").is_err());
        assert!(file(&dir.0, "../secret.svg").is_err());
        assert!(file(&dir.0, "/etc/passwd").is_err());
        assert!(file(&dir.0, "inside.svg").is_ok());
        std::fs::remove_file(dir.0.join("out.svg")).unwrap();
        std::os::unix::fs::symlink("inside.svg", dir.0.join("alias.svg")).unwrap();
        assert!(load(&dir.0).is_ok());
    }

    #[test]
    fn limits_hold() {
        let big_yaml = format!("hud: 1\n# {}\n", "x".repeat(YAML_BYTES));
        let dir = Dir::new("yaml");
        dir.put("hud.yaml", big_yaml.as_bytes());
        assert!(errors(load(&dir.0)).contains("hud.yaml"));

        let panel = "  - {id: p%, source: clock, anchor: top, text: x}\n";
        let many = |n: usize| {
            let mut text = String::from("hud: 1\npanels:\n");
            for i in 0..n {
                text.push_str(&panel.replace('%', &i.to_string()));
            }
            text
        };
        assert!(parse(&many(MAX_PANELS)).is_ok());
        assert!(errors(parse(&many(MAX_PANELS + 1))).contains("panels"));

        let dir = Dir::new("file");
        dir.put("hud.yaml", MINIMAL.as_bytes());
        dir.put("ok.svg", &vec![b' '; FILE_BYTES]);
        assert!(load(&dir.0).is_ok());
        dir.put("big.svg", &vec![b' '; FILE_BYTES + 1]);
        assert!(errors(load(&dir.0)).contains("big.svg"));

        let dir = Dir::new("folder");
        dir.put("hud.yaml", MINIMAL.as_bytes());
        for i in 0..8 {
            dir.put(&format!("{i}.svg"), &vec![b' '; FILE_BYTES]);
        }
        assert!(errors(load(&dir.0)).contains("folder"));
    }

    #[test]
    fn a_value_is_never_expanded_again() {
        let fields = serde_json::json!({
            "text": "{secret} and {text}",
            "secret": "no",
            "a": {"b": "deep"},
            "n": 3,
        });
        assert_eq!(render("say: {text}", &fields), "say: {secret} and {text}");
        assert_eq!(render("{a.b} {n} {missing} {a.c}.", &fields), "deep 3  .");
        assert_eq!(render("{ unclosed {n", &fields), "{ unclosed {n");
        let lookup = |name: &str| (name == "x").then(|| "{x}".to_owned());
        assert_eq!(expand("{x}{x}", lookup), "{x}{x}");
    }

    #[test]
    fn when_matches_lists_and_all_keys() {
        let context = Context {
            wm: "xp".into(),
            state: "working".into(),
            agents: "any".into(),
            ..Context::default()
        };
        let cases = [
            ("{}", true),
            ("{wm: xp}", true),
            ("{wm: mc}", false),
            ("{wm: [mc, xp]}", true),
            ("{wm: []}", false),
            ("{wm: xp, state: [working, question]}", true),
            ("{wm: xp, state: asleep}", false),
            ("{nothing: x}", false),
            ("{app: ''}", true),
        ];
        for (yaml, expected) in cases {
            let when: When = serde_saphyr::from_str(yaml).unwrap();
            assert_eq!(when.matches(&context), expected, "{yaml}");
        }
        assert_eq!(context.get("agents"), Some("any"));
        assert_eq!(context.get("mood"), None);
        for key in CONTEXT_KEYS {
            assert!(Context::default().get(key).is_some(), "{key}");
        }
    }

    #[test]
    fn the_chat_block_is_values_only() {
        assert!(errors(parse("hud: 1\nchat: {size: 11}\n")).contains("chat.size: 11"));
        assert!(parse("hud: 1\nchat: {size: 14}\n").is_ok());
        let neutral = parse(MINIMAL).unwrap();
        assert_eq!(neutral.chat.place(), (Anchor::Right, 400));
    }

    #[test]
    fn panels_are_checked() {
        let panel = |body: &str| format!("hud: 1\npanels:\n  - {{id: a, text: x, {body}}}\n");
        assert!(errors(parse(&panel("source: say"))).contains("panels[0]"));
        assert!(errors(parse(&panel("source: event, anchor: top"))).contains("event"));
        assert!(errors(parse(&panel("source: clock, anchor: top, event: a.b"))).contains("event"));
        assert!(errors(parse(&panel("source: say, anchor: top, action: mute"))).contains("mute"));
        assert!(errors(parse(&panel("source: say, near: nowhere"))).contains("nowhere"));
        assert!(parse(&panel("source: voice, anchor: top, action: mute")).is_ok());
        let twice = "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, text: x}\n  - {id: a, \
                     source: say, anchor: top, text: x}\n";
        assert!(errors(parse(twice)).contains("twice"));
    }

    #[test]
    fn a_missing_picture_is_an_error() {
        let dir = Dir::new("picture");
        dir.put(
            "hud.yaml",
            b"hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, picture: 'dot.svg#{state}'}\n",
        );
        assert!(errors(load(&dir.0)).contains("dot.svg"));
        dir.put("dot.svg", b"<svg/>");
        assert!(load(&dir.0).is_ok());
    }
    const SHOWN: &str = "picture: 'a.svg#x'";

    fn one_panel(body: &str) -> String {
        format!("hud: 1\npanels:\n  - {{id: a, source: voice, anchor: top, {SHOWN}, {body}}}\n")
    }

    #[test]
    fn an_id_is_a_short_plain_name() {
        let with = |id: &str| {
            format!("hud: 1\npanels:\n  - {{id: '{id}', source: voice, anchor: top, {SHOWN}}}\n")
        };
        for bad in ["", "A", "a b", "a.b", "a/b", "é", &"a".repeat(33)] {
            assert!(
                errors(parse(&with(bad))).contains("panels[0].id"),
                "{bad:?}"
            );
        }
        for good in ["a", "fig_1-b", &"a".repeat(32)] {
            assert!(parse(&with(good)).is_ok(), "{good:?}");
        }
    }

    #[test]
    fn numbers_have_bounds() {
        let chat = |body: &str| format!("hud: 1\nchat: {{{body}}}\n");
        for bad in ["size: 49", "width: 0", "width: 1601"] {
            let key = bad.split(':').next().unwrap();
            assert!(
                errors(parse(&chat(bad))).contains(&format!("chat.{key}")),
                "{bad}"
            );
        }
        for good in ["size: 48", "size: 14", "width: 1600", "width: 1"] {
            assert!(parse(&chat(good)).is_ok(), "{good}");
        }
        for bad in [
            "max_chars: 0",
            "max_chars: 2001",
            "max_rows: 0",
            "max_rows: 201",
        ] {
            let key = bad.split(':').next().unwrap();
            assert!(errors(parse(&one_panel(bad))).contains(key), "{bad}");
        }
        for good in [
            "max_chars: 2000",
            "max_chars: 1",
            "max_rows: 200",
            "max_rows: 1",
        ] {
            assert!(parse(&one_panel(good)).is_ok(), "{good}");
        }
    }

    #[test]
    fn a_folder_cannot_place_the_question() {
        let slot = errors(parse("hud: 1\nchat: {slot: mc}\n"));
        assert!(slot.contains("slot"), "{slot}");
    }

    #[test]
    fn a_picture_file_name_has_no_template() {
        let bad = errors(parse(
            &one_panel("picture: '{state}.svg#a'").replace(SHOWN, "text: x"),
        ));
        assert!(bad.contains("panels[0].picture"), "{bad}");
        let fine = one_panel("text: x").replace(SHOWN, "picture: 'dot.svg#{state}'");
        assert!(parse(&fine).is_ok());
    }

    #[test]
    fn near_must_end_at_an_anchor() {
        let cycle = "hud: 1\npanels:\n  - {id: a, source: say, near: b, text: x}\n  - {id: b, \
                     source: say, near: a, text: x}\n";
        let cycle = errors(parse(cycle));
        assert!(
            cycle.contains("panels[0].near") && cycle.contains("panels[1].near"),
            "{cycle}"
        );
        let chain = "hud: 1\npanels:\n  - {id: a, source: say, near: b, text: x}\n  - {id: b, \
                     source: say, near: c, text: x}\n  - {id: c, source: say, anchor: top, text: x}\n";
        assert!(parse(chain).is_ok());
    }

    #[test]
    fn when_values_are_checked() {
        let when = |body: &str| one_panel(&format!("when: {{{body}}}"));
        let bad = errors(parse(&when("chat: opened")));
        assert!(bad.contains("when.chat") && bad.contains("opened"), "{bad}");
        assert!(errors(parse(&when("state: [working, sleeping]"))).contains("when.state"));
        assert!(errors(parse(&when("agents: many"))).contains("when.agents"));
        assert!(
            parse(&when(
                "chat: open, state: [working, question], agents: any, app: foot"
            ))
            .is_ok()
        );
    }

    #[test]
    fn a_panel_shows_something() {
        let empty = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top}\n",
        ));
        assert!(
            empty.contains("panels[0]") && empty.contains("picture"),
            "{empty}"
        );
        for shown in ["picture: 'a.svg#x'", "text: x", "rows: x"] {
            let text =
                format!("hud: 1\npanels:\n  - {{id: a, source: voice, anchor: top, {shown}}}\n");
            assert!(parse(&text).is_ok(), "{shown}");
        }
    }

    #[test]
    fn aliases_are_refused() {
        let text =
            "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, text: &t x, rows: *t}\n";
        assert!(parse(text).is_err());
    }

    #[test]
    fn the_scan_has_limits() {
        let dir = Dir::new("entries");
        dir.put("hud.yaml", MINIMAL.as_bytes());
        for i in 0..255 {
            dir.put(&format!("{i}.svg"), b"<svg/>");
        }
        assert!(load(&dir.0).is_ok());
        dir.put("one-more.svg", b"<svg/>");
        assert!(errors(load(&dir.0)).contains("entries"));

        let dir = Dir::new("depth");
        dir.put("hud.yaml", MINIMAL.as_bytes());
        std::fs::create_dir_all(dir.0.join("a/b/c")).unwrap();
        dir.put("a/b/c/ok.svg", b"<svg/>");
        assert!(load(&dir.0).is_ok());
        std::fs::create_dir_all(dir.0.join("a/b/c/d")).unwrap();
        dir.put("a/b/c/d/deep.svg", b"<svg/>");
        assert!(errors(load(&dir.0)).contains("depth"));
    }

    #[test]
    fn a_hard_link_is_refused() {
        let dir = Dir::new("nlink");
        dir.put("hud.yaml", MINIMAL.as_bytes());
        dir.put("a.svg", b"<svg/>");
        assert!(load(&dir.0).is_ok());
        std::fs::hard_link(dir.0.join("a.svg"), dir.0.join("b.svg")).unwrap();
        let refused = errors(load(&dir.0));
        assert!(
            refused.contains("a.svg") || refused.contains("b.svg"),
            "{refused}"
        );
        assert!(refused.contains("hard link"), "{refused}");
    }

    #[test]
    fn the_yaml_is_read_only_after_the_scan_passes() {
        let dir = Dir::new("order");
        dir.put("hud.yaml", b"hud: 1\nscript: x\n");
        dir.put("big.svg", &vec![b' '; FILE_BYTES + 1]);
        let refused = errors(load(&dir.0));
        assert!(refused.contains("big.svg"), "{refused}");
        assert!(!refused.contains("script"), "{refused}");
    }

    #[test]
    fn the_yaml_is_read_as_a_file_of_the_folder() {
        let outside = Dir::new("yaml-outside");
        outside.put("hud.yaml", MINIMAL.as_bytes());
        let dir = Dir::new("yaml-link");
        std::os::unix::fs::symlink(outside.0.join("hud.yaml"), dir.0.join("hud.yaml")).unwrap();
        assert!(errors(load(&dir.0)).contains("outside"));
    }
}
