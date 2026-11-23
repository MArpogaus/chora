//! What each region shows: the items of its contents and the plan of a
//! scene, built from the settings and the state of the helper.

use super::{
    BTreeMap, Content, Kind, Look, Mc, PropsUpdate, Region, Scene, System, Type, title_of,
};

/// Where the words of a row of the full key list begin, so that
/// `Super+Shift+Escape` fits before them.
const FULL_LIST_WORDS: f64 = 152.0;
/// How many doors stand in a line.
const DOORS_PER_LINE: usize = 3;
/// The space from one door of a line to the next, in pixels.
const DOOR_STEP: f64 = 100.0;

/// The content that names the next entry of the day in the footer.
const NEXT: &str = "next";

/// The content that names the previous page in the footer.
const PREV: &str = "prev";

/// One text node of a region.
pub struct Item {
    pub id: String,
    pub slot: i32,
    /// The props, without the place.
    pub props: PropsUpdate,
    /// Where a stack in a side puts space before it: a new content.
    pub first: bool,
    /// The place across a side, from its left edge.
    pub x: f64,
    pub put: Put,
    /// Space before the item in a stack, in pixels.
    pub before: f64,
    /// What a click on it does: a binding as `key:Super+n`, a helper as
    /// `agent:NAME`; it also shows the focus mark under the pointer.
    pub hit: Option<String>,
    /// A close mark stands before the text: two bars, drawn, not a glyph.
    pub mark: bool,
    /// The item gives way when its side is too full, the last first.
    pub shed: bool,
}

/// Where an item of a side stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Put {
    /// One under the other from the top.
    Stack,
    /// At the foot of the side, above its lower edge.
    Foot,
    /// At the foot, with a hairline over the width of the side above it.
    FootRule,
    /// At the foot, in the line of the item before it.
    FootRow,
    /// At the foot, the first of a line whose items take the pointer by
    /// their own width.
    FootStart,
    /// In the line of the item before it, at `x`.
    Row,
    /// In the line of the item before it, flush right.
    End,
    /// In the line of the item before it, right after it.
    After,
    /// Where the box of the chat begins: it runs to the foot of the side.
    Slot,
}

impl Put {
    pub(super) fn foot(self) -> bool {
        matches!(
            self,
            Self::Foot | Self::FootRule | Self::FootRow | Self::FootStart
        )
    }

    pub(super) fn own_width(self) -> bool {
        matches!(
            self,
            Self::End | Self::After | Self::Row | Self::FootRow | Self::FootStart
        )
    }

    pub(super) fn at_left(self) -> bool {
        matches!(self, Self::End | Self::After)
    }

    pub(super) fn row(self) -> bool {
        matches!(self, Self::FootRow | Self::Row | Self::End | Self::After)
    }
}

impl Item {
    /// A text item that stacks in a side.
    pub fn text(id: String, slot: i32, props: PropsUpdate, first: bool) -> Self {
        Self {
            id,
            slot,
            props,
            first,
            x: 0.0,
            put: Put::Stack,
            before: 0.0,
            hit: None,
            mark: false,
            shed: false,
        }
    }
}

/// The items and the docked app of each region.
pub type Plan = BTreeMap<Region, (Vec<Item>, Option<String>)>;

/// The time and the words of a note, `18:51  Update ready`.
fn split_time(line: &str) -> Option<(&str, &str)> {
    let (time, words) = line.split_once("  ")?;
    let digits = |part: &str| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit());
    let (hour, minute) = time.split_once(':')?;
    (digits(hour) && digits(minute)).then_some((time, words.trim_start()))
}

impl Mc {
    /// The type of a content: its own style on the base of its kind. The
    /// state of the agent shows in the accent, as long as no style says
    /// otherwise.
    pub fn content_type(&self, content: &Content, title: bool) -> Type {
        let base = match content.kind {
            _ if title => "label",
            Kind::Head | Kind::Clock => "running_head",
            Kind::Hud | Kind::Agents => "hud",
            Kind::Chat => "body",
            Kind::State | Kind::Text | Kind::App | Kind::Keys | Kind::Doors => "margin",
        };
        let style = content.style.as_ref().filter(|_| !title);
        let mut ty = self.ty(base, style);
        if content.kind == Kind::State && style.is_none_or(|s| s.color.is_none()) {
            ty.color = (self.settings.value.colors())
                .get("accent")
                .copied()
                .unwrap_or(ty.color);
        }
        ty
    }

    /// The title and the text of a content now in a scene; an empty text
    /// hides it.
    pub(super) fn content_text(
        &self,
        name: &str,
        content: &Content,
        scene: &str,
    ) -> (String, String) {
        match content.kind {
            Kind::Head => (String::new(), self.head(scene)),
            Kind::Clock => (
                String::new(),
                self.clocks.get(name).cloned().unwrap_or_default(),
            ),
            Kind::State => (String::new(), self.hud.state.clone()),
            Kind::Hud => (String::new(), self.hud.text.clone()),
            Kind::Agents => (String::new(), self.agent_line().join("  ")),
            Kind::Text => self
                .texts
                .get(name)
                .map(|(title, lines)| (title.clone(), lines.join("\n")))
                .unwrap_or_default(),
            Kind::App | Kind::Keys | Kind::Chat | Kind::Doors => (String::new(), String::new()),
        }
    }

    /// The running head of a scene: the one of its chapter, or its number
    /// and title.
    pub(super) fn head(&self, name: &str) -> String {
        let Some(scene) = self.scenes.get(name) else {
            return String::new();
        };
        let chapter = &scene.chapter;
        chapter.running_head.clone().unwrap_or_else(|| {
            let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
            format!("{}  {title}", self.number(name))
        })
    }

    /// Whether a region shows in a scene: as the toggle says, else as the
    /// scene says, else as the settings say. The window of a program
    /// fills the text block of its page, so that the column shows beside
    /// it as beside any page, and its doors stay on the screen (mc-42).
    pub(super) fn shown(&self, region: Region, name: &str) -> bool {
        let scene = self.scenes.get(name);
        let side = matches!(region, Region::Left | Region::Right);
        self.toggled
            .get(&region)
            .or_else(|| scene.and_then(|s: &Scene| s.regions.get(&region)))
            .copied()
            .unwrap_or_else(|| {
                let config = &self.settings.value;
                let preset = config.presets.get(&config.preset);
                let shown = (preset.and_then(|p| p.shown.get(&region)).copied())
                    .unwrap_or(config.regions.get(region).shown);
                shown && !(side && scene.is_none())
            })
    }

    /// The binding of an action, as the key list names it.
    fn key_of(&self, action: &crate::config::Action) -> Option<String> {
        (self.settings.value.common.bindings.iter())
            .find(|(_, a)| *a == action)
            .map(|(key, _)| format!("key:{key}"))
    }

    /// The items of the regions of a scene, and the docked app of each.
    /// The list of all keys has its side to itself under the state line,
    /// so that it fits a small screen; the other sides keep their content.
    /// The sides show only where they fit.
    pub fn items(&self, name: &str) -> Plan {
        let contents = &self.settings.value.contents;
        let mut out = Plan::new();
        for (region, slots) in self.placed(name) {
            let dock = (slots.iter().map(|(_, c)| c))
                .find(|c| contents[*c].kind == Kind::App && self.docks.contains_key(*c))
                .cloned();
            let keys = |c: &String| contents[c].kind == Kind::Keys;
            let whole = self.keys == crate::keys::List::All && slots.iter().any(|(_, c)| keys(c));
            let stays = |c: &String| {
                let kind = contents[c].kind;
                !whole || matches!(kind, Kind::State | Kind::Keys | Kind::Doors)
            };
            let items: Vec<Item> = (slots.iter())
                .filter(|(_, c)| stays(c))
                .flat_map(|(slot, c)| self.content_items(region, *slot, c, name))
                .collect();
            if !items.is_empty() || dock.is_some() {
                out.insert(region, (items, dock));
            }
        }
        let [left, right] = self.side_widths(name);
        out.retain(|region, _| match region {
            Region::Left => left > 0.0,
            Region::Right => right > 0.0,
            _ => true,
        });
        out
    }

    /// The text items of a content in a region: one in a line, else its
    /// title and its lines.
    fn content_items(&self, region: Region, slot: i32, name: &str, scene: &str) -> Vec<Item> {
        let content = &self.settings.value.contents[name];
        let side = !region.is_line();
        match content.kind {
            Kind::Agents => self.agent_items(region, slot, name, content),
            Kind::State if side => self.state_items(region, slot, (name, scene)),
            Kind::Chat if side => self.chat_items(slot, name, content, region),
            Kind::Keys if side => self.key_items(slot, name, content, region),
            Kind::Doors if side => self.door_items(slot, name, content, region),
            Kind::App | Kind::Chat | Kind::Keys | Kind::Doors => Vec::new(),
            _ => self.text_items(region, slot, (name, scene)),
        }
    }

    /// The text of a content that has none of its own: a free line for the
    /// state word of the host, the line to the next page in the footer.
    fn filled(
        &self,
        region: Region,
        (name, scene): (&str, &str),
        text: &str,
    ) -> (String, Option<String>) {
        let content = &self.settings.value.contents[name];
        if !text.is_empty() {
            return (text.to_owned(), None);
        }
        if content.kind == Kind::State && !region.is_line() {
            return (" ".to_owned(), None);
        }
        if name == NEXT && region.is_line() {
            let line = self.next_page_line(scene).unwrap_or_default();
            return (line, self.key_of(&Look::NextScene.into()));
        }
        if name == PREV && region.is_line() {
            let line = self.previous_page_line(scene).unwrap_or_default();
            return (line, self.key_of(&Look::PreviousScene.into()));
        }
        (String::new(), None)
    }

    /// The state line of a side: the state word that mc draws, if any,
    /// and the word for the microphone. Where the HUD host draws the
    /// state word at the right, the word for the microphone takes the
    /// free line at the left.
    fn state_items(&self, region: Region, slot: i32, (name, scene): (&str, &str)) -> Vec<Item> {
        let mut items = self.text_items(region, slot, (name, scene));
        let id = format!("{}-{name}", self.region_id(region));
        let blank = |item: &Item| {
            item.props
                .text
                .as_deref()
                .is_none_or(|t| t.trim().is_empty())
        };
        let alone = items.iter().all(blank);
        if let Some(word) = self.mic_item(&id, slot, alone) {
            if alone {
                items.clear();
            }
            items.push(word);
        }
        items
    }

    /// The token of the hit `node`, as its item names it.
    pub(super) fn hit_token(&self, node: &str) -> Option<String> {
        let base = node.strip_suffix("-hit")?;
        let name = self.current.clone().unwrap_or_default();
        (self.items(&name).into_values())
            .flat_map(|(items, _)| items)
            .find(|i| i.id == base)
            .and_then(|i| i.hit)
    }

    /// A press on the word for the microphone; see [`Mc::press_mic`].
    pub fn click_mic(&mut self, node: &str, injected: bool) -> bool {
        (self.hit_token(node)).is_some_and(|token| self.press_mic(&token, injected))
    }

    fn text_items(&self, region: Region, slot: i32, (name, scene): (&str, &str)) -> Vec<Item> {
        let content = &self.settings.value.contents[name];
        let (title, text) = self.content_text(name, content, scene);
        let (mut text, hit) = self.filled(region, (name, scene), &text);
        let id = format!("{}-{name}", self.region_id(region));
        if content.folds && self.chat == crate::Chat::Open && !region.is_line() {
            return self.folded_items(slot, &id, content, (&title, &text));
        }
        let quiet = content.empty && text.is_empty() && !region.is_line();
        if quiet {
            text = crate::keys::nothing_new(chora_kit::words::Language::of_session()).into();
        }
        if content.folds && !region.is_line() && !quiet && !text.is_empty() {
            return self.note_items(slot, &id, content, (&title, &text));
        }
        let item = |id: String, is_title: bool, text: &str, first: bool| {
            let mut props = self.content_type(content, is_title).props(text);
            if quiet && !is_title {
                props.color = self.settings.value.colors().get("muted").copied();
            }
            Item {
                hit: hit.clone(),
                ..Item::text(id, slot, props, first)
            }
        };
        if region.is_line() {
            let text = text.replace('\n', "  ·  ");
            return (!text.is_empty())
                .then(|| item(id, false, &text, true))
                .into_iter()
                .collect();
        }
        let opened = self.toggled.get(&region) == Some(&true) && region != Region::Right;
        let mut items = Vec::new();
        if !title.is_empty() && (opened || !text.is_empty()) {
            items.push(item(format!("{id}-title"), true, &title, true));
        }
        if !text.is_empty() {
            let first = items.is_empty();
            items.push(item(id, false, &text, first));
        }
        items
    }

    /// The chat in a side: while it is closed a door, `Chat with a helper`
    /// and the key of the chat flush right, under a head with the state
    /// word of the helper; while it is open the head with `Esc  Close` and
    /// the place of the box of the host, whose header says so when no
    /// helper is set up. While a helper asks, its card stands over the head
    /// and the door, so that the column has none.
    fn chat_items(&self, slot: i32, name: &str, content: &Content, region: Region) -> Vec<Item> {
        if self.question_pending() && self.chat == crate::Chat::Closed {
            return Vec::new();
        }
        let id = format!("{}-{name}", self.region_id(region));
        let head = self.content_type(content, true).props("Chat");
        let mut items = vec![Item::text(format!("{id}-title"), slot, head, true)];
        let margin = self.ty("margin", None);
        let open = self.chat == crate::Chat::Open;
        let end = |part: &str, text: &str, hit: Option<&str>| Item {
            put: Put::End,
            hit: hit.map(str::to_owned),
            ..Item::text(format!("{id}-{part}"), slot, margin.props(text), false)
        };
        if open {
            items.push(Item {
                put: Put::End,
                ..end("close", "Esc  Close", Some("chat:close"))
            });
            let blank = Item::text(format!("{id}-slot"), slot, margin.props(" "), false);
            items.push(Item {
                put: Put::Slot,
                ..blank
            });
            return items;
        }
        items.push(end("state", &self.helper_state_word(), None));
        let door = self
            .content_type(content, false)
            .props("Chat with a helper ...");
        items.push(Item {
            hit: Some("chat:open".into()),
            ..Item::text(format!("{id}-door"), slot, door, false)
        });
        let key = (self.settings.value.common.bindings.iter())
            .find(|(_, a)| **a == System::Chat.into())
            .map(|(k, _)| chora_kit::keysheet::chord(k));
        items.extend(key.map(|key| end("key", &key, None)));
        items
    }

    /// Whether a helper asks: the HUD host then draws its card over the
    /// chat section of the column.
    fn question_pending(&self) -> bool {
        let named = self.voiced_word.named().map(|(_, raw, _)| raw);
        named.or_else(|| self.agents.first().map(|a| a.state.as_str())) == Some("question")
    }

    /// The state word of the helper door: the state of the helper that the
    /// helper machine of the kit names (KIT7), else of the first helper of
    /// `agent.set`, or that there is none; the label is costume (mc.yaml).
    fn helper_state_word(&self) -> String {
        let machine = &self.voiced_word;
        let named = machine
            .named()
            .filter(|_| machine.state() != chora_kit::helper::State::NoHelper)
            .map(|(_, raw, _)| raw);
        let Some(raw) = named.or_else(|| self.agents.first().map(|a| a.state.as_str())) else {
            let language = chora_kit::words::Language::of_session();
            return chora_kit::words::Word::NoHelperDoor
                .text(language)
                .to_owned();
        };
        let state = self.settings.value.states.get(raw);
        let words = state.and_then(|s| s.label.as_deref()).unwrap_or(raw);
        let mut letters = words.chars();
        letters
            .next()
            .map(|first| first.to_uppercase().chain(letters).collect())
            .unwrap_or_default()
    }

    /// The notes of a list that folds: each on one line, the time in the
    /// margin style and the words in the reading face beside it, a longer
    /// note wrapping under its first word.
    fn note_items(
        &self,
        slot: i32,
        id: &str,
        content: &Content,
        (title, text): (&str, &str),
    ) -> Vec<Item> {
        let mut items = Vec::new();
        if !title.is_empty() {
            let props = self.content_type(content, true).props(title);
            items.push(Item::text(format!("{id}-title"), slot, props, true));
        }
        let baseline = self.settings.value.scale.baseline;
        let note = self.ty("margin", None);
        let body = self.content_type(content, false);
        let x = chora_kit::hud::LABEL_COLUMN;
        let lines = text.lines().filter(|line| !line.trim().is_empty());
        for (i, line) in lines.enumerate() {
            let Some((time, words)) = split_time(line) else {
                let props = body.props(line);
                items.push(Item::text(format!("{id}-{i}"), slot, props, false));
                continue;
            };
            items.push(Item {
                before: if i > 0 { baseline } else { 0.0 },
                ..Item::text(format!("{id}-{i}-time"), slot, note.props(time), false)
            });
            items.push(Item {
                put: Put::Row,
                x,
                ..Item::text(format!("{id}-{i}"), slot, body.props(words), false)
            });
        }
        items
    }

    /// A text that folds while the chat is open: its head with the count
    /// of its lines, and `Show` flush right.
    fn folded_items(
        &self,
        slot: i32,
        id: &str,
        content: &Content,
        (title, text): (&str, &str),
    ) -> Vec<Item> {
        let count = text.lines().filter(|l| !l.trim().is_empty()).count();
        let head = format!("{title}  {count}");
        let props = self.content_type(content, true).props(&head);
        let mut items = vec![Item::text(format!("{id}-title"), slot, props, true)];
        if count > 0 {
            let props = self.ty("margin", None).props("Show");
            items.push(Item {
                put: Put::End,
                hit: Some("chat:close".into()),
                ..Item::text(format!("{id}-show"), slot, props, false)
            });
        }
        items
    }

    /// The rows of the key list: the key in the text color, its words in
    /// the margin color beside it, and a head over them.
    fn key_items(&self, slot: i32, name: &str, content: &Content, region: Region) -> Vec<Item> {
        if self.chat == crate::Chat::Open {
            return Vec::new();
        }
        let config = &self.settings.value;
        let full = self.keys == crate::keys::List::All;
        let plain = self.page_keys_work();
        self.keys_plain.set(plain);
        let rows = if full {
            crate::keys::all(config, plain)
        } else {
            crate::keys::short(config, plain)
        };
        let id = format!("{}-{name}", self.region_id(region));
        let title = if full { "All keys" } else { "Keys" };
        let head = self.content_type(content, true).props(title);
        let mut items = vec![Item {
            put: Put::FootRule,
            ..Item::text(format!("{id}-title"), slot, head, true)
        }];
        let mut ty = self.content_type(content, false);
        let words_x = if full {
            FULL_LIST_WORDS
        } else {
            chora_kit::hud::LABEL_COLUMN
        };
        let words = ty.clone();
        ty.color = config.colors().get("text").copied().unwrap_or(ty.color);
        for (i, row) in rows.iter().enumerate() {
            let mut props = ty.props(&row.key);
            props.weight = Some(500);
            items.push(Item {
                put: Put::Foot,
                hit: row.binding.as_ref().map(|b| format!("key:{b}")),
                ..Item::text(format!("{id}-{i}-key"), slot, props, false)
            });
            items.push(Item {
                put: Put::FootRow,
                x: words_x,
                ..Item::text(
                    format!("{id}-{i}-words"),
                    slot,
                    words.props(&row.words),
                    false,
                )
            });
        }
        items
    }

    /// The doors at the foot of a side: a word for each action that has a
    /// key, three in a line, clicked like its key. They stay while the chat is
    /// open (MH-2): its box ends one baseline above the foot, the doors being
    /// the mouse path of the rc bar.
    fn door_items(&self, slot: i32, name: &str, content: &Content, region: Region) -> Vec<Item> {
        let id = format!("{}-{name}", self.region_id(region));
        let mut ty = self.content_type(content, false);
        ty.color = self
            .settings
            .value
            .colors()
            .get("text")
            .copied()
            .unwrap_or(ty.color);
        let volume = self.volume_words();
        let words = [
            ("programs", "Programs"),
            ("lock", "Lock"),
            ("power", "Power"),
            ("screenshot", "Screenshot"),
            ("volume", volume.as_str()),
        ];
        let mut items = Vec::new();
        for (i, (door, text)) in words.into_iter().enumerate() {
            let column = i % DOORS_PER_LINE;
            #[allow(clippy::cast_precision_loss, reason = "below 3")]
            let x = column as f64 * DOOR_STEP;
            let put = if column == 0 {
                Put::FootStart
            } else {
                Put::FootRow
            };
            items.push(Item {
                put,
                x,
                hit: Some(format!("door:{door}")),
                ..Item::text(format!("{id}-{door}"), slot, ty.props(text), i == 0)
            });
        }
        items
    }

    /// The words of the volume door: the level, or why there is none.
    fn volume_words(&self) -> String {
        let language = chora_kit::words::Language::of_session();
        let title = chora_kit::words::Word::Volume.text(language);
        match self.volume_reading {
            Some(reading @ chora_kit::volume::Reading::Level(volume)) if !volume.muted => {
                format!("{title} {}", reading.words(language))
            }
            Some(reading) => reading.words(language),
            None => title.to_owned(),
        }
    }
}
