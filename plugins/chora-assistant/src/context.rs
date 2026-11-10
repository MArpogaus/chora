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

/// How long the host waits after a change before it re-evaluates.
pub const DEBOUNCE: Duration = Duration::from_millis(150);

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
}

/// The claims of the nodes of a query: owner and area, for each node that
/// claims one.
#[must_use]
pub fn claims_of(nodes: &[Value]) -> Vec<Claim> {
    nodes
        .iter()
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
/// manager of the whole space, around those of its panes.
fn claimant(claims: &[Claim], [px, py]: [f64; 2]) -> Option<&str> {
    claims
        .iter()
        .filter(|(_, [x, y, w, h])| (*x..x + w).contains(&px) && (*y..y + h).contains(&py))
        .max_by(|(_, a), (_, b)| (a[2] * a[3]).total_cmp(&(b[2] * b[3])))
        .map(|(owner, _)| owner.as_str())
}

/// The place of the focus, from the focused surface of a query (none: no
/// focus), the claims and the outputs. Without a focus, the window manager
/// is the one that claims the first output and the app is empty.
#[must_use]
pub fn place_of(focused: Option<&Value>, claims: &[Claim], outputs: &[OutputInfo]) -> Place {
    let named = focused.and_then(output_of);
    let output = named
        .as_deref()
        .and_then(|name| outputs.iter().find(|o| o.name == name))
        .or_else(|| outputs.first());
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

/// The context of the host, and the time of its next re-evaluation.
#[derive(Debug)]
pub struct Switcher {
    applied: Context,
    wanted: Context,
    due: Option<Instant>,
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
            scene: false,
            question: false,
            field: false,
        }
    }

    /// The context that the HUD shows.
    #[must_use]
    pub fn applied(&self) -> &Context {
        &self.applied
    }

    /// Something of the scene changed: re-evaluate after [`DEBOUNCE`]. A
    /// change inside a pending burst joins it.
    pub fn touch(&mut self, now: Instant) {
        self.scene = true;
        self.due.get_or_insert(now + DEBOUNCE);
    }

    /// The state of the voice agent, as the context names it.
    pub fn set_state(&mut self, state: &str, now: Instant) {
        self.set(|context| &mut context.state, state, now);
    }

    /// The state of the helpers, as the context names it.
    pub fn set_agents(&mut self, agents: &str, now: Instant) {
        self.set(|context| &mut context.agents, agents, now);
    }

    fn set(&mut self, field: fn(&mut Context) -> &mut String, value: &str, now: Instant) {
        let slot = field(&mut self.wanted);
        if slot != value {
            value.clone_into(slot);
            self.due.get_or_insert(now + DEBOUNCE);
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
    #[allow(
        dead_code,
        reason = "the chat of H7 calls it when the field takes the keyboard"
    )]
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
    pub fn resolve(&mut self, place: Option<Place>) -> Option<Context> {
        self.due = None;
        self.scene = false;
        if let Some(place) = place {
            self.wanted.wm = place.wm;
            self.wanted.space = place.space;
            self.wanted.app = place.app;
            self.wanted.output = place.output;
        }
        (self.wanted != self.applied).then(|| {
            self.applied = self.wanted.clone();
            self.applied.clone()
        })
    }
}

impl Agent {
    /// When the context is evaluated next, if something changed.
    pub fn switch_due(&mut self, now: Instant) -> Option<Instant> {
        self.switcher.hold_question(self.card.is_some(), now);
        self.switcher.due()
    }

    /// Evaluates the context when its burst of changes has ended.
    pub fn switch_step(&mut self, now: Instant) -> Result<(), ClientError> {
        if self.switch_due(now).is_some_and(|due| due <= now) {
            self.evaluate()?;
        }
        Ok(())
    }

    /// Reads the place of the focus and, when the context changed, builds
    /// the look of the folder that its rule names. The question card is not
    /// touched: no evaluation runs while one waits.
    pub fn evaluate(&mut self) -> Result<(), ClientError> {
        let place = if self.switcher.wants_place() {
            Some(self.read_place()?)
        } else {
            None
        };
        let Some(context) = self.switcher.resolve(place) else {
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
    /// rules when the folder or a field that it reads changed, the figure
    /// on the output of the focus. A folder that fails to load leaves the
    /// look as it is.
    fn restyle(&mut self, context: &Context) -> Result<Vec<Command>, ClientError> {
        let folder = self.settings.value.folder(context).to_owned();
        let read = |field: &String| self.look_context.get(field) != context.get(field);
        let rebuilt = self.source.assets.is_none()
            && (folder != self.folder || self.values.watches.iter().any(read));
        let moved = self.look_context.output != context.output;
        if rebuilt {
            match self.source.load(&folder, context) {
                Ok((theme, values, cache)) => {
                    self.theme = theme;
                    self.values = values;
                    self.folder = folder;
                    let old = std::mem::replace(&mut self.look_cache, cache);
                    let _ = std::fs::remove_dir_all(old);
                }
                Err(err) => eprintln!("chora-assistant: hud {folder}: {err}; the look stays"),
            }
        }
        self.look_context = context.clone();
        if !(rebuilt || moved) || self.card.is_some() {
            return Ok(Vec::new());
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
        commands.extend(self.sync_badge()?);
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
        switcher.resolve(Some(place.clone()));
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
            start + DEBOUNCE,
            "the burst waits from its first change"
        );
        let xp = Place {
            wm: "xp".into(),
            space: "xp".into(),
            ..Place::default()
        };
        assert!(switcher.resolve(Some(xp.clone())).is_some());
        assert_eq!(switcher.due(), None);
        assert!(
            switcher.resolve(Some(xp)).is_none(),
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
        assert!(switcher.resolve(Some(tiling.clone())).is_some());
        switcher.touch(now);
        assert!(switcher.resolve(Some(tiling)).is_none());
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
        switcher.set_state("listening", now);
        assert!(switcher.due().is_some());
        assert!(!switcher.wants_place(), "a state needs no new place");
        switcher.resolve(None);
        assert_eq!(switcher.applied().state, "listening");
        switcher.set_state("listening", now);
        assert_eq!(switcher.due(), None, "the same state is no change");
        switcher.set_agents("working", now);
        switcher.resolve(None);
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
    fn a_fresh_context_matches_every_field_of_a_rule() {
        let context = Switcher::new().applied().clone();
        for key in chora_kit::hud::CONTEXT_KEYS {
            assert!(context.get(key).is_some(), "{key}");
        }
        let chat_none: When =
            serde_json::from_str(r#"{"chat": "none", "agents": "none"}"#).unwrap();
        assert!(chat_none.matches(&context));
    }
}
