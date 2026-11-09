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
/// The smallest text size of the chat.
pub const MIN_CHAT_SIZE: u32 = 14;

/// The keys that `when` may compare, and the fields of [`Context`].
pub const CONTEXT_KEYS: [&str; 7] = ["wm", "space", "app", "state", "agents", "chat", "output"];

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
    /// The name that `near` refers to; unique in the folder.
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
    /// The id of another panel that this one stands beside.
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
    /// The most characters of the text.
    pub max_chars: Option<u32>,
    /// A template for one row of a list.
    pub rows: Option<String>,
    /// The most rows of a list.
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
    /// Text size in pixels, at least [`MIN_CHAT_SIZE`].
    pub size: u32,
    /// The place; the right edge if left out.
    pub anchor: Option<Anchor>,
    /// The width in pixels; 400 if left out.
    pub width: Option<u32>,
    /// The name of a client that announces a box for the chat.
    pub slot: Option<String>,
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
            slot: None,
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

/// Where the chat view stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place<'a> {
    /// At an anchor of the work area, this wide.
    Anchored(Anchor, u32),
    /// In the box that this client announces.
    Slot(&'a str),
}

impl Chat {
    /// The place of the view: the slot, else the anchor and width, else
    /// the right edge, 400 wide.
    #[must_use]
    pub fn place(&self) -> Place<'_> {
        match &self.slot {
            Some(client) => Place::Slot(client),
            None => Place::Anchored(
                self.anchor.unwrap_or(Anchor::Right),
                self.width.unwrap_or(400),
            ),
        }
    }

    fn check(&self, errors: &mut Vec<String>) {
        if self.size < MIN_CHAT_SIZE {
            errors.push(format!("chat.size: {} is below {MIN_CHAT_SIZE}", self.size));
        }
        if self.width == Some(0) {
            errors.push("chat.width: must be more than 0".into());
        }
        match &self.slot {
            Some(client) if client.is_empty() => {
                errors.push("chat.slot: needs the name of a client".into());
            }
            Some(_) if self.anchor.is_some() || self.width.is_some() => {
                errors.push("chat.slot: not together with anchor or width".into());
            }
            _ => {}
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
    let mut options = serde_saphyr::Options::default();
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
        self.chat.check(&mut errors);
        errors
    }
}

impl Panel {
    fn check(&self, at: &str, ids: &BTreeSet<&str>, errors: &mut Vec<String>) {
        let mut fail = |key: &str, message: String| errors.push(format!("{at}.{key}: {message}"));
        if self.id.is_empty() {
            fail("id", "needs a name".into());
        }
        match (&self.event, self.source) {
            (Some(event), Source::Event) => {
                let named = event
                    .split_once('.')
                    .is_some_and(|(client, name)| !client.is_empty() && !name.is_empty());
                if !named {
                    fail("event", format!("{event:?} is not CLIENT.EVENT"));
                }
            }
            (None, Source::Event) => fail("event", "source event needs CLIENT.EVENT".into()),
            (Some(_), _) => fail("event", "only for source: event".into()),
            (None, _) => {}
        }
        match (self.anchor, &self.near) {
            (None, None) => fail("anchor", "needs `anchor` or `near`".into()),
            (Some(_), Some(_)) => fail("near", "not together with anchor".into()),
            (None, Some(near)) if near == &self.id || !ids.contains(near.as_str()) => {
                fail("near", format!("no other panel {near:?}"));
            }
            _ => {}
        }
        if self.action == Some(Action::Mute) && self.source != Source::Voice {
            fail("action", "mute needs source: voice".into());
        }
        for key in self.when.0.keys() {
            if !CONTEXT_KEYS.contains(&key.as_str()) {
                fail(&format!("when.{key}"), "not a field of the context".into());
            }
        }
    }
}

/// Loads the folder `dir`: the limits, the links, `hud.yaml` and the
/// pictures that it names.
///
/// # Errors
///
/// Each error names its file and, for `hud.yaml`, its key.
pub fn load(dir: &Path) -> Result<Hud, Vec<String>> {
    let mut errors = scan(dir);
    let text = crate::read_text(&dir.join("hud.yaml"), MAX_YAML)
        .map_err(|e| vec![format!("hud.yaml: {e}")]);
    let hud = text.and_then(|text| parse(&text));
    let hud = match hud {
        Ok(hud) if errors.is_empty() => hud,
        Ok(_) => return Err(errors),
        Err(more) => {
            errors.extend(more);
            return Err(errors);
        }
    };
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
/// links: a link must lead to a regular file inside the folder.
fn scan(root: &Path) -> Vec<String> {
    let real = match std::fs::canonicalize(root) {
        Ok(real) => real,
        Err(e) => return vec![format!("{}: {e}", root.display())],
    };
    let mut errors = Vec::new();
    let mut total = 0_u64;
    let mut pending = vec![real.clone()];
    while let Some(dir) = pending.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                errors.push(format!("{}: {e}", dir.display()));
                continue;
            }
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            let name = path
                .strip_prefix(&real)
                .unwrap_or(&path)
                .display()
                .to_string();
            match entry_size(&path, &real) {
                Ok(None) => pending.push(path),
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
        return Ok(Some(kind.len()));
    }
    if !kind.is_symlink() {
        return Err("not a regular file".into());
    }
    let target = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    if !target.starts_with(real) {
        return Err("link points outside the folder".into());
    }
    match std::fs::metadata(&target) {
        Ok(meta) if meta.is_file() => Ok(Some(meta.len())),
        _ => Err("link to something but a file".into()),
    }
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
             source: event, event: mc.chat, anchor: top}\nchat: {style: glass, size: 16, anchor: \
             right, width: 480}\n",
        )
        .unwrap();
        assert_eq!(hud.panels.len(), 3);
        assert_eq!(hud.chat.place(), Place::Anchored(Anchor::Right, 480));
    }

    #[test]
    fn unknown_key_fails() {
        let top = errors(parse("hud: 1\nscript: x\n"));
        assert!(top.contains("script"), "{top}");
        let panel = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, run: sh}\n",
        ));
        assert!(panel.contains("run"), "{panel}");
        let chat = errors(parse("hud: 1\nchat: {colour: red}\n"));
        assert!(chat.contains("colour"), "{chat}");
        let when = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top, when: {mood: calm}}\n",
        ));
        assert!(when.contains("when.mood"), "{when}");
        let source = errors(parse(
            "hud: 1\npanels:\n  - {id: a, source: shell, anchor: top}\n",
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

        let panel = "  - {id: p%, source: clock, anchor: top}\n";
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
        assert!(errors(parse("hud: 1\nchat: {slot: mc, anchor: top}\n")).contains("chat.slot"));
        let slot = parse("hud: 1\nchat: {slot: mc}\n").unwrap();
        assert_eq!(slot.chat.place(), Place::Slot("mc"));
        let neutral = parse(MINIMAL).unwrap();
        assert_eq!(neutral.chat.place(), Place::Anchored(Anchor::Right, 400));
    }

    #[test]
    fn panels_are_checked() {
        let panel = |body: &str| format!("hud: 1\npanels:\n  - {{id: a, {body}}}\n");
        assert!(errors(parse(&panel("source: say"))).contains("panels[0]"));
        assert!(errors(parse(&panel("source: event, anchor: top"))).contains("event"));
        assert!(errors(parse(&panel("source: clock, anchor: top, event: a.b"))).contains("event"));
        assert!(errors(parse(&panel("source: say, anchor: top, action: mute"))).contains("mute"));
        assert!(errors(parse(&panel("source: say, near: nowhere"))).contains("nowhere"));
        assert!(parse(&panel("source: voice, anchor: top, action: mute")).is_ok());
        let twice = "hud: 1\npanels:\n  - {id: a, source: voice, anchor: top}\n  - {id: a, \
                     source: say, anchor: top}\n";
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
}
