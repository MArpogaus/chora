//! The context that picks the HUD folder (`docs/hud.md`): which window
//! manager owns the focused window, the space around it, the focused app,
//! the state of the voice agent and its helpers. The host keeps one record
//! and re-evaluates it [`DEBOUNCE`] after the first change of a burst. It
//! never switches while a question waits or the chat field has the
//! keyboard.

use std::time::{Duration, Instant};

use chora_kit::hud::Context;
use chora_scene::{
    Command, OutputInfo,
    ipc::{ClientError, Filter, Query},
};
use serde_json::Value;

use crate::{Agent, assistant::Assistant, card};

/// How long the host waits after the last change before it re-evaluates.
pub const DEBOUNCE: Duration = Duration::from_millis(150);

/// The longest that a burst of changes can keep the host from evaluating.
pub const MAX_WAIT: Duration = Duration::from_secs(1);

/// The shortest time that a context shows before the next switch.
pub const DWELL: Duration = Duration::from_secs(1);

/// A claim of a window manager: its owner and its area in scene
/// coordinates.
pub type Claim = (String, [f64; 4]);

/// Where the focus is: the fields of the context that the scene tells.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Place {
    /// The window manager of the focused window.
    pub wm: String,
    /// The window manager of the space that shows the window.
    pub space: String,
    /// The `app_id` of the focused window.
    pub app: String,
    /// The output that shows the focused window.
    pub output: String,
    /// A window has the focus.
    pub focused: bool,
    /// The screens: the outputs that are not virtual.
    pub screens: Vec<String>,
}

/// The claims of the nodes of a query: owner and area, for each visible node
/// that claims one.
#[must_use]
pub fn claims_of(nodes: &[Value]) -> Vec<Claim> {
    nodes
        .iter()
        .filter(|node| node["visible"].as_bool().unwrap_or(true))
        .filter_map(|node| {
            let owner = node["owner"].as_str()?.to_owned();
            let area = node["claim"].as_array()?;
            let mut numbers = area.iter().filter_map(Value::as_f64);
            let area = [
                numbers.next()?,
                numbers.next()?,
                numbers.next()?,
                numbers.next()?,
            ];
            Some((owner, area))
        })
        .collect()
}

/// The output that shows most of a window, from the `rects` of its node.
fn output_of(node: &Value) -> Option<String> {
    let area = |rect: &Value| {
        let side = |at: usize| rect[at].as_f64().unwrap_or(0.0);
        side(2) * side(3)
    };
    node["rects"]
        .as_object()?
        .iter()
        .max_by(|(_, a), (_, b)| area(a).total_cmp(&area(b)))
        .map(|(name, _)| name.clone())
}

/// The owner of the largest claim that contains a point: the window
/// manager of the whole space, around those of its panes. Of equal claims,
/// the owner that comes first by name.
fn claimant(claims: &[Claim], [px, py]: [f64; 2]) -> Option<&str> {
    claims
        .iter()
        .filter(|(_, [x, y, w, h])| (*x..x + w).contains(&px) && (*y..y + h).contains(&py))
        .max_by(|(owner_a, a), (owner_b, b)| {
            (a[2] * a[3])
                .total_cmp(&(b[2] * b[3]))
                .then_with(|| owner_b.cmp(owner_a))
        })
        .map(|(owner, _)| owner.as_str())
}

/// The place of the focus, from the focused surface of a query (none: no
/// focus), the claims and the outputs. Without a focus, the window manager
/// is the one that claims the first output and the app is empty; the
/// [`Switcher`] keeps the space of before. The [`Switcher`] keeps the
/// output of the HUD while it is a screen, so that the HUD does not jump
/// between screens with the focus. The output is
/// never a virtual one: its content shows only where a pane of its owner
/// draws it.
#[must_use]
pub fn place_of(focused: Option<&Value>, claims: &[Claim], outputs: &[OutputInfo]) -> Place {
    let named = focused.and_then(output_of);
    let mut screens = outputs.iter().filter(|o| o.is_virtual != Some(true));
    let first = screens.clone().next();
    let output = named
        .as_deref()
        .and_then(|name| screens.find(|o| o.name == name))
        .or(first);
    let space = output
        .and_then(|output| {
            let [x, y, w, h] = output.layout_area().map(f64::from);
            claimant(claims, [x + w / 2.0, y + h / 2.0])
        })
        .unwrap_or_default();
    let wm = focused
        .and_then(|node| node["owner"].as_str())
        .unwrap_or(space);
    Place {
        wm: wm.to_owned(),
        space: space.to_owned(),
        app: focused
            .and_then(|node| node["surface"]["app_id"].as_str())
            .unwrap_or_default()
            .to_owned(),
        output: output.map(|o| o.name.clone()).unwrap_or_default(),
        focused: focused.is_some(),
        screens: (outputs.iter())
            .filter(|o| o.is_virtual != Some(true))
            .map(|o| o.name.clone())
            .collect(),
    }
}

/// `none`, `any`, `working` or `question`, for the states of the helpers.
#[must_use]
pub fn agents_of<'a>(states: impl IntoIterator<Item = &'a str>) -> &'static str {
    states.into_iter().fold("none", |found, state| {
        let rank = |name| {
            ["none", "any", "working", "question"]
                .iter()
                .position(|n| *n == name)
        };
        let this = match state {
            "question" => "question",
            "working" | "thinking" => "working",
            _ => "any",
        };
        if rank(this) > rank(found) {
            this
        } else {
            found
        }
    })
}

/// What a new context does to the look on the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// Nothing.
    Keep,
    /// The values are picked again from the folder, and the figure is built
    /// again if they differ.
    Pick,
    /// The figure is built again on another output.
    Move,
    /// Another folder loads.
    Reload,
}

/// The change for a new context. `now` is the folder on the screen,
/// `wanted` the folder that the rules name and `failed` a folder that did
/// not load, which is not asked for again. `watches` are the fields that
/// the panels of the folder read.
#[must_use]
pub fn change(
    (now, wanted, failed): (&str, &str, Option<&str>),
    watches: &std::collections::BTreeSet<String>,
    (old, new): (&Context, &Context),
) -> Change {
    if wanted != now && failed != Some(wanted) {
        Change::Reload
    } else if old.output != new.output {
        Change::Move
    } else if watches.iter().any(|key| old.get(key) != new.get(key)) {
        Change::Pick
    } else {
        Change::Keep
    }
}

/// The context of the host, and the time of its next re-evaluation.
#[derive(Debug)]
pub struct Switcher {
    applied: Context,
    wanted: Context,
    due: Option<Instant>,
    first: Option<Instant>,
    switched: Option<Instant>,
    focused: Option<String>,
    app: String,
    scene: bool,
    question: bool,
    field: bool,
}

impl Default for Switcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Switcher {
    /// A host that has seen nothing yet: the agent sleeps, no helper works
    /// and no chat is open.
    #[must_use]
    pub fn new() -> Self {
        let context = Context {
            state: "asleep".into(),
            agents: "none".into(),
            chat: "none".into(),
            ..Context::default()
        };
        Self {
            applied: context.clone(),
            wanted: context,
            due: None,
            first: None,
            switched: None,
            focused: None,
            app: String::new(),
            scene: true,
            question: false,
            field: false,
        }
    }

    /// The context that the HUD shows.
    #[must_use]
    pub fn applied(&self) -> &Context {
        &self.applied
    }

    /// Something of the scene changed: re-evaluate [`DEBOUNCE`] after the
    /// last change of the burst, at the latest [`MAX_WAIT`] after its first
    /// and [`DWELL`] after the last switch.
    pub fn touch(&mut self, now: Instant) {
        self.scene = true;
        self.schedule(now);
    }

    fn schedule(&mut self, now: Instant) {
        let first = *self.first.get_or_insert(now);
        let due = (now + DEBOUNCE).min(first + MAX_WAIT);
        self.due = Some(self.switched.map_or(due, |at| due.max(at + DWELL)));
    }

    /// The keyboard focus moved to a surface, or to none.
    pub fn focus_changed(&mut self, surface: Option<&str>, now: Instant) {
        self.focused = surface.map(str::to_owned);
        self.touch(now);
    }

    /// A surface changed its title, size or app id: only the app id of the
    /// focused window, when it differs, is a change of the context.
    pub fn surface_changed(&mut self, surface: &str, app_id: Option<&str>, now: Instant) {
        let app = app_id.unwrap_or_default();
        if self.focused.as_deref() == Some(surface) && self.app != app {
            app.clone_into(&mut self.app);
            self.touch(now);
        }
    }

    /// The state of the voice agent, as the context names it.
    pub fn set_state(&mut self, state: &str, now: Instant) {
        self.set(|context| &mut context.state, state, now);
    }

    /// A chat view shows or not.
    pub fn set_chat(&mut self, open: bool, now: Instant) {
        self.set(
            |context| &mut context.chat,
            if open { "open" } else { "none" },
            now,
        );
    }

    /// The state of the helpers, as the context names it.
    pub fn set_agents(&mut self, agents: &str, now: Instant) {
        self.set(|context| &mut context.agents, agents, now);
    }

    fn set(&mut self, field: fn(&mut Context) -> &mut String, value: &str, now: Instant) {
        let slot = field(&mut self.wanted);
        if slot != value {
            value.clone_into(slot);
            self.schedule(now);
        }
    }

    /// A question waits or not: the context stays while one does.
    pub fn hold_question(&mut self, on: bool, now: Instant) {
        let was = self.held();
        self.question = on;
        self.after_hold(was, now);
    }

    /// The chat field has the keyboard or not: the context stays while it
    /// has.
    pub fn hold_field(&mut self, on: bool, now: Instant) {
        let was = self.held();
        self.field = on;
        self.after_hold(was, now);
    }

    fn held(&self) -> bool {
        self.question || self.field
    }

    fn after_hold(&mut self, was: bool, now: Instant) {
        if was && !self.held() && self.due.is_some() {
            self.first = Some(now);
            self.due = Some(now + DEBOUNCE);
        }
    }

    /// When to re-evaluate; never while a question waits or the field has
    /// the keyboard.
    #[must_use]
    pub fn due(&self) -> Option<Instant> {
        self.due.filter(|_| !self.held())
    }

    /// Whether the scene changed since the last evaluation, so that the
    /// place of the focus is to be read again.
    #[must_use]
    pub fn wants_place(&self) -> bool {
        self.scene
    }

    /// Ends a burst with the place of the focus, when it was read. The new
    /// context, when it differs from the one that shows.
    pub fn resolve(&mut self, place: Option<Place>, now: Instant) -> Option<Context> {
        self.due = None;
        self.first = None;
        self.scene = false;
        if let Some(place) = place {
            self.app.clone_from(&place.app);
            let kept = place.screens.contains(&self.wanted.output);
            if place.focused || !kept {
                self.wanted.wm = place.wm;
                self.wanted.space = place.space;
            }
            if !kept {
                self.wanted.output = place.output;
            } else if !place.focused {
                self.wanted.wm.clone_from(&self.wanted.space);
            }
            self.wanted.app = place.app;
        }
        (self.wanted != self.applied).then(|| {
            self.switched = Some(now);
            self.applied = self.wanted.clone();
            self.applied.clone()
        })
    }
}

impl Agent {
    /// When the context is evaluated next, if something changed.
    pub fn switch_due(&mut self, now: Instant) -> Option<Instant> {
        self.switcher.hold_question(self.card.is_some(), now);
        self.switcher.hold_field(self.chat.keyboard(), now);
        self.switcher.due()
    }

    /// Evaluates the context when its burst of changes has ended.
    pub fn switch_step(&mut self, now: Instant) -> Result<(), ClientError> {
        if !self.deferred.is_empty() && self.card.is_none() {
            self.deferred.clear();
            let commands = self.replace_assistant()?;
            self.apply(commands)?;
            for old in std::mem::take(&mut self.stale) {
                let _ = std::fs::remove_dir_all(old);
            }
        }
        if self.switch_due(now).is_some_and(|due| due <= now) {
            self.evaluate(now)?;
        }
        Ok(())
    }

    /// Reads the place of the focus and, when the context changed, builds
    /// the look of the folder that its rule names. The question card is not
    /// touched: no evaluation runs while one waits.
    pub fn evaluate(&mut self, now: Instant) -> Result<(), ClientError> {
        let place = if self.switcher.wants_place() {
            Some(self.read_place()?)
        } else {
            None
        };
        let Some(context) = self.switcher.resolve(place, now) else {
            return Ok(());
        };
        let commands = self.restyle(&context)?;
        self.apply(commands)
    }

    /// The place of the focus: the focused window and the claims of the
    /// window managers.
    fn read_place(&mut self) -> Result<Place, ClientError> {
        let focused = Query {
            selector: "kind:surface".into(),
            filter: Some(Filter {
                focused: true,
                ..Filter::default()
            }),
        };
        let focused: Vec<Value> = self.rt.conn.call("scene.query", focused)?;
        let groups: Vec<Value> = self.rt.conn.call("scene.query", Query::new("kind:group"))?;
        let outputs = self.rt.outputs()?;
        Ok(place_of(focused.first(), &claims_of(&groups), &outputs))
    }

    /// The commands that show the context: the look of the folder of the
    /// rules when the folder changed, the values picked again when a field
    /// that a panel reads changed, the figure on the output of the focus. A
    /// folder that fails to load leaves the look as it is and is not asked
    /// for again.
    fn restyle(&mut self, context: &Context) -> Result<Vec<Command>, ClientError> {
        let wanted = if self.source.assets.is_some() {
            self.folder.clone()
        } else {
            self.settings.value.folder(context).to_owned()
        };
        let shown = (
            self.folder.as_str(),
            wanted.as_str(),
            self.failed.as_deref(),
        );
        let how = change(shown, &self.values.watches, (&self.look_context, context));
        self.look_context = context.clone();
        match how {
            Change::Keep => return Ok(Vec::new()),
            Change::Move => {}
            Change::Pick => {
                let theme = &self.theme;
                let picked = self.values.repick(context, |sheet| theme.sheet_size(sheet));
                match picked.filter(|picked| *picked != self.values) {
                    Some(picked) => self.values = picked,
                    None => return Ok(Vec::new()),
                }
            }
            Change::Reload => match self.source.load(&wanted, context) {
                Ok((theme, values, cache)) => {
                    self.theme = theme;
                    self.values = values;
                    self.folder = wanted;
                    self.failed = None;
                    let old = std::mem::replace(&mut self.look_cache, cache);
                    let _ = std::fs::remove_dir_all(old);
                    self.watch_slot()?;
                }
                Err(err) => {
                    eprintln!("chora-assistant: hud {wanted}: {err}; the look stays");
                    self.failed = Some(wanted);
                    return Ok(Vec::new());
                }
            },
        }
        let mut commands = Vec::new();
        if self.assistant.take().is_some() {
            commands.push(Assistant::remove());
        }
        if std::mem::take(&mut self.badge) {
            commands.push(Command::Remove {
                id: card::BADGE.into(),
                to: None,
            });
        }
        commands.extend(self.sync_assistant(Instant::now())?);
        commands.extend(self.say_again(Instant::now()));
        commands.extend(self.sync_badge()?);
        commands.extend(self.sync_row()?);
        self.chat_restyle()?;
        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::hud::When;
    use serde_json::json;

    use super::*;
    use crate::config::Config;

    fn output(name: &str, position: [i32; 2]) -> OutputInfo {
        OutputInfo {
            name: name.into(),
            position,
            size: [1280, 800],
            scale: 1.0,
            camera: chora_scene::Transform::default(),
            work_area: [position[0], position[1], 1280, 800],
            default_work_area: [position[0], position[1], 1280, 800],
            is_virtual: None,
            owner: None,
        }
    }

    fn window(owner: &str, app: &str, output: &str) -> Value {
        json!({"id": "s1", "owner": owner, "focused": true,
            "surface": {"app_id": app}, "rects": {output: [10, 10, 300, 200]}})
    }

    fn group(owner: &str, claim: [f64; 4]) -> Value {
        json!({"id": format!("{owner}-root"), "owner": owner, "claim": claim})
    }

    fn rules(yaml: &str) -> Config {
        chora_kit::settings::parse(&format!(
            "enabled: true\nposition: null\nhud: neutral\n{yaml}"
        ))
        .unwrap()
    }

    const SHIPPED: &str = "huds:\n  - {when: {space: construct}, folder: construct}\n  - {when: \
        {wm: mc}, folder: mc}\n  - {when: {wm: xp}, folder: xp}\n";

    fn context(place: &Place) -> Context {
        let mut switcher = Switcher::new();
        switcher.resolve(Some(place.clone()), Instant::now());
        switcher.applied().clone()
    }

    #[test]
    fn focus_on_an_xp_window_picks_xp() {
        let claims = claims_of(&[group("xp", [0.0, 0.0, 1280.0, 800.0])]);
        let place = place_of(
            Some(&window("xp", "notepad", "HEADLESS-1")),
            &claims,
            &[output("HEADLESS-1", [0, 0])],
        );
        assert_eq!(place.wm, "xp");
        assert_eq!(place.space, "xp");
        assert_eq!(place.app, "notepad");
        assert_eq!(place.output, "HEADLESS-1");
        let config = rules(SHIPPED);
        assert_eq!(config.folder(&context(&place)), "xp");
        let tiling = Place {
            wm: "tiling".into(),
            space: "tiling".into(),
            ..place
        };
        assert_eq!(config.folder(&context(&tiling)), "neutral");
    }

    #[test]
    fn a_wm_in_a_construct_pane_keeps_space_construct() {
        let claims = claims_of(&[
            group("construct", [0.0, 0.0, 1280.0, 800.0]),
            group("tiling", [200.0, 100.0, 800.0, 600.0]),
        ]);
        let place = place_of(
            Some(&window("tiling", "foot", "HEADLESS-1")),
            &claims,
            &[output("HEADLESS-1", [0, 0])],
        );
        assert_eq!(
            (place.wm.as_str(), place.space.as_str()),
            ("tiling", "construct")
        );
        assert_eq!(rules(SHIPPED).folder(&context(&place)), "construct");
    }

    #[test]
    fn the_space_is_the_claimant_of_the_output_that_shows_the_window() {
        let claims = claims_of(&[
            group("mc", [0.0, 0.0, 1280.0, 800.0]),
            group("xp", [1280.0, 0.0, 1280.0, 800.0]),
        ]);
        let outputs = [output("A", [0, 0]), output("B", [1280, 0])];
        let on_b = place_of(Some(&window("xp", "x", "B")), &claims, &outputs);
        assert_eq!((on_b.space.as_str(), on_b.output.as_str()), ("xp", "B"));
        let on_a = place_of(Some(&window("mc", "x", "A")), &claims, &outputs);
        assert_eq!(on_a.space, "mc");
    }

    /// M4: the HUD stays on its screen while the focus moves between
    /// window managers on two screens; only the look follows the focus.
    /// When its screen goes, it moves to the screen of the focus.
    #[test]
    fn the_hud_keeps_its_screen_while_the_focus_moves() {
        let claims = claims_of(&[
            group("tiling", [0.0, 0.0, 1280.0, 800.0]),
            group("xp", [1280.0, 0.0, 1280.0, 800.0]),
        ]);
        let both = [output("A", [0, 0]), output("B", [1280, 0])];
        let mut switcher = Switcher::new();
        let now = Instant::now();
        let mut focus = |wm: &str, on: &str, outputs: &[OutputInfo]| {
            switcher.touch(now);
            let place = place_of(Some(&window(wm, "x", on)), &claims, outputs);
            switcher.resolve(Some(place), now);
            let applied = switcher.applied();
            (applied.wm.clone(), applied.output.clone())
        };
        assert_eq!(focus("tiling", "A", &both), ("tiling".into(), "A".into()));
        assert_eq!(focus("xp", "B", &both), ("xp".into(), "A".into()));
        assert_eq!(focus("xp", "B", &both[1..]), ("xp".into(), "B".into()));
    }

    #[test]
    fn without_a_focus_the_claimant_of_the_first_output_stands_in() {
        let claims = claims_of(&[group("mc", [0.0, 0.0, 1280.0, 800.0])]);
        let place = place_of(None, &claims, &[output("HEADLESS-1", [0, 0])]);
        assert_eq!(place.wm, "mc");
        assert_eq!(place.space, "mc");
        assert_eq!(place.app, "");
        assert_eq!(place.output, "HEADLESS-1");
        assert_eq!(place_of(None, &[], &[]), Place::default());
    }

    #[test]
    fn a_burst_of_focus_changes_switches_once() {
        let start = Instant::now();
        let mut switcher = Switcher::new();
        for step in 0..10_u32 {
            switcher.touch(start + Duration::from_millis(u64::from(step) * 10));
        }
        let due = switcher.due().expect("a change is pending");
        assert_eq!(
            due,
            start + Duration::from_millis(90) + DEBOUNCE,
            "the burst waits for its end"
        );
        let xp = Place {
            wm: "xp".into(),
            space: "xp".into(),
            ..Place::default()
        };
        assert!(switcher.resolve(Some(xp.clone()), start).is_some());
        assert_eq!(switcher.due(), None);
        assert!(
            switcher.resolve(Some(xp), start).is_none(),
            "the same place switches nothing"
        );
    }

    #[test]
    fn a_burst_that_ends_where_it_began_switches_nothing() {
        let now = Instant::now();
        let mut switcher = Switcher::new();
        let tiling = Place {
            wm: "tiling".into(),
            ..Place::default()
        };
        assert!(switcher.resolve(Some(tiling.clone()), now).is_some());
        switcher.touch(now);
        assert!(switcher.resolve(Some(tiling), now).is_none());
    }

    #[test]
    fn nothing_switches_while_a_question_waits() {
        let start = Instant::now();
        let mut switcher = Switcher::new();
        switcher.hold_question(true, start);
        switcher.touch(start);
        assert_eq!(switcher.due(), None);
        let later = start + Duration::from_secs(5);
        switcher.hold_question(false, later);
        assert_eq!(switcher.due(), Some(later + DEBOUNCE), "it waits once more");
        let mut idle = Switcher::new();
        idle.hold_question(true, start);
        idle.hold_question(false, later);
        assert_eq!(
            idle.due(),
            None,
            "a hold without a change waits for nothing"
        );
    }

    #[test]
    fn the_context_stays_while_the_field_has_the_keyboard() {
        let start = Instant::now();
        let mut switcher = Switcher::new();
        switcher.hold_field(true, start);
        switcher.touch(start);
        assert_eq!(switcher.due(), None);
        switcher.hold_question(true, start);
        switcher.hold_field(false, start);
        assert_eq!(switcher.due(), None, "a question still holds");
        switcher.hold_question(false, start);
        assert_eq!(switcher.due(), Some(start + DEBOUNCE));
    }

    #[test]
    fn a_state_that_changes_a_rule_is_a_change() {
        let now = Instant::now();
        let mut switcher = Switcher::new();
        switcher.resolve(Some(Place::default()), now);
        switcher.set_state("listening", now);
        assert!(switcher.due().is_some());
        assert!(!switcher.wants_place(), "a state needs no new place");
        switcher.resolve(None, now);
        assert_eq!(switcher.applied().state, "listening");
        switcher.set_state("listening", now);
        assert_eq!(switcher.due(), None, "the same state is no change");
        switcher.set_agents("working", now);
        switcher.resolve(None, now);
        assert_eq!(switcher.applied().agents, "working");
    }

    #[test]
    fn the_agents_field_names_the_busiest_helper() {
        assert_eq!(agents_of([]), "none");
        assert_eq!(agents_of(["idle"]), "any");
        assert_eq!(agents_of(["idle", "working"]), "working");
        assert_eq!(agents_of(["working", "question", "idle"]), "question");
    }

    #[test]
    fn a_rule_reads_each_field_of_the_context() {
        let place = Place {
            wm: "xp".into(),
            space: "construct".into(),
            app: "foot".into(),
            output: "DP-1".into(),
            focused: true,
            screens: vec!["DP-1".into()],
        };
        let context = context(&place);
        for key in chora_kit::hud::CONTEXT_KEYS {
            let value = context.get(key).unwrap();
            let same =
                |value: &str| -> When { serde_json::from_value(json!({ key: value })).unwrap() };
            assert!(same(value).matches(&context), "{key} equals {value:?}");
            assert!(!same("no such value").matches(&context), "{key}");
        }
        assert_eq!(
            (
                context.state.as_str(),
                context.agents.as_str(),
                context.chat.as_str()
            ),
            ("asleep", "none", "none"),
        );
    }

    #[test]
    fn the_host_reads_the_context_at_its_start() {
        let mut switcher = Switcher::new();
        assert!(
            switcher.wants_place(),
            "the first evaluation reads the scene"
        );
        let place = Place {
            wm: "xp".into(),
            space: "xp".into(),
            ..Place::default()
        };
        let context = switcher.resolve(Some(place), Instant::now()).unwrap();
        assert_eq!(rules(SHIPPED).folder(&context), "xp");
    }

    fn alternating(place: u64) -> Place {
        let wm = if place.is_multiple_of(2) { "xp" } else { "mc" };
        Place {
            wm: wm.into(),
            space: wm.into(),
            ..Place::default()
        }
    }

    fn switches(from: u64, to: u64, touching: impl Fn(u64) -> bool) -> Vec<u64> {
        let start = Instant::now();
        let mut switcher = Switcher::new();
        switcher.resolve(Some(Place::default()), start);
        let mut found = Vec::new();
        for ms in (from..to).step_by(10) {
            let now = start + Duration::from_millis(ms);
            if touching(ms) {
                switcher.touch(now);
            }
            if switcher.due().is_some_and(|due| due <= now)
                && switcher.resolve(Some(alternating(ms / 10)), now).is_some()
            {
                found.push(ms);
            }
        }
        found
    }

    #[test]
    fn a_burst_gives_exactly_one_switch() {
        let found = switches(0, 3000, |ms| ms < 100);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0] >= 100 + 150 - 10,
            "{found:?}: after the end of the burst"
        );
    }

    #[test]
    fn a_long_burst_switches_at_most_once_a_second() {
        let found = switches(0, 5000, |ms| ms < 4000);
        assert!(found.len() <= 5, "{found:?}");
        assert!(
            found.windows(2).all(|pair| pair[1] - pair[0] >= 1000),
            "{found:?}: a dwell of a second"
        );
    }

    #[test]
    fn only_the_focused_window_changing_its_app_is_a_change() {
        let now = Instant::now();
        let mut switcher = Switcher::new();
        switcher.focus_changed(Some("s1"), now);
        switcher.resolve(
            Some(Place {
                app: "foot".into(),
                ..Place::default()
            }),
            now,
        );
        switcher.surface_changed("s2", Some("firefox"), now);
        assert_eq!(switcher.due(), None, "another window");
        switcher.surface_changed("s1", Some("foot"), now);
        assert_eq!(switcher.due(), None, "the same app");
        switcher.surface_changed("s1", Some("vim"), now);
        assert!(switcher.due().is_some());
    }

    #[test]
    fn equal_claims_go_to_the_owner_first_by_name_and_a_hidden_claim_is_none() {
        let area = [0.0, 0.0, 1280.0, 800.0];
        let outputs = [output("A", [0, 0])];
        let pick = |claims: Vec<Value>| place_of(None, &claims_of(&claims), &outputs).space;
        let both = vec![group("xp", area), group("mc", area)];
        assert_eq!(pick(both.clone()), "mc");
        assert_eq!(pick(both.into_iter().rev().collect()), "mc");
        let mut hidden = group("mc", area);
        hidden["visible"] = json!(false);
        assert_eq!(pick(vec![hidden, group("xp", area)]), "xp");
    }

    #[test]
    fn without_a_focus_the_output_and_the_space_stay() {
        let now = Instant::now();
        let mut switcher = Switcher::new();
        let on_b = Place {
            wm: "tiling".into(),
            space: "xp".into(),
            app: "foot".into(),
            output: "B".into(),
            focused: true,
            screens: vec!["A".into(), "B".into()],
        };
        switcher.resolve(Some(on_b), now);
        let none = Place {
            wm: "mc".into(),
            space: "mc".into(),
            output: "A".into(),
            screens: vec!["A".into(), "B".into()],
            ..Place::default()
        };
        switcher.touch(now);
        let kept = switcher.resolve(Some(none), now).unwrap();
        assert_eq!((kept.output.as_str(), kept.space.as_str()), ("B", "xp"));
        assert_eq!(kept.app, "");
    }

    fn at(wm: &str, output: &str, state: &str) -> Context {
        Context {
            wm: wm.into(),
            output: output.into(),
            state: state.into(),
            ..Context::default()
        }
    }

    fn reads(keys: &[&str]) -> std::collections::BTreeSet<String> {
        keys.iter().map(|key| (*key).to_owned()).collect()
    }

    #[test]
    fn the_folder_loads_again_only_when_the_folder_changes() {
        let watches = reads(&["wm", "state"]);
        let same = at("xp", "A", "idle");
        let ask = |now, wanted, failed, new: &Context| {
            change((now, wanted, failed), &watches, (&same, new))
        };
        assert_eq!(ask("xp", "xp", None, &at("xp", "A", "idle")), Change::Keep);
        assert_eq!(
            ask("neutral", "xp", None, &at("xp", "A", "idle")),
            Change::Reload
        );
        assert_eq!(
            ask("xp", "xp", None, &at("tiling", "A", "idle")),
            Change::Pick
        );
        assert_eq!(
            ask("xp", "xp", None, &at("xp", "A", "speaking")),
            Change::Pick
        );
        assert_eq!(ask("xp", "xp", None, &at("xp", "B", "idle")), Change::Move);
    }

    #[test]
    fn a_field_that_no_panel_reads_changes_nothing() {
        let old = at("xp", "A", "idle");
        let new = at("xp", "A", "speaking");
        assert_eq!(
            change(("xp", "xp", None), &reads(&["wm"]), (&old, &new)),
            Change::Keep
        );
    }

    #[test]
    fn a_folder_that_failed_is_not_asked_for_again() {
        let old = at("xp", "A", "idle");
        let new = at("tiling", "A", "idle");
        let watches = reads(&[]);
        assert_eq!(
            change(("neutral", "xp", Some("xp")), &watches, (&old, &new)),
            Change::Keep
        );
        assert_eq!(
            change(("neutral", "mc", Some("xp")), &watches, (&old, &new)),
            Change::Reload
        );
    }

    /// A card on a virtual output would show only inside a pane, where a
    /// shader can draw over it: the place is always on a screen.
    #[test]
    fn the_place_is_never_a_virtual_output() {
        let face = OutputInfo {
            is_virtual: Some(true),
            owner: Some("construct".into()),
            ..output("VIRTUAL-1", [-262_144, -262_144])
        };
        let outputs = [face, output("A", [0, 0])];
        let on_the_face = place_of(Some(&window("xp", "x", "VIRTUAL-1")), &[], &outputs);
        assert_eq!(on_the_face.output, "A");
        assert_eq!(place_of(None, &[], &outputs).output, "A");
    }
}
