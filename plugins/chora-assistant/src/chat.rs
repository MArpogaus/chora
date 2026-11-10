//! The chat session of the host: it opens a chat with a helper
//! (`chat.open`), pulls its transcript (`agent.read`), keeps the field
//! (`chora-chat --input`) and the view in step, and closes the chat
//! (`chat.close`). The model is `chora_kit::chat`; the session owns no
//! socket and no process. It reaches voice and the field through
//! [`Outside`], so that a test plays both.
//!
//! The chat is no window: a switch of look is a restyle, the field stays
//! the same process and so keeps its draft, and the context holds while
//! the field has the keyboard.

#![allow(
    dead_code,
    unused_variables,
    clippy::unused_self,
    clippy::unnecessary_wraps,
    reason = "the red commit holds stubs"
)]

use chora_kit::{
    ask::Keys,
    chat::{Chat as Transcript, Line, Measure},
    hud::{Anchor, Chat as Look},
    words::Language,
};
use serde_json::Value;

use crate::field::FieldEvent;

/// Where the chat shows: the output, its size and work area.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// The name of the output.
    pub output: String,
    /// Its size in logical pixels.
    pub size: [i32; 2],
    /// Its work area: x, y, width, height.
    pub area: [i32; 4],
    /// The `chat:` values of the folder that draws.
    pub look: Look,
}

/// What the session needs from the rest of the host.
pub trait Outside {
    /// Calls a method of voice.
    fn voice(&mut self, method: &str, params: Value) -> Result<Value, String>;
    /// The activation token of the act that is happening now, if the core
    /// gives one.
    fn token(&mut self) -> Option<String>;
    /// Starts the field.
    fn start_field(&mut self) -> Result<Box<dyn Link>, String>;
}

/// The field, as the session talks to it.
pub trait Link {
    /// Shows the field at a place.
    fn place(&mut self, place: &Value);
    /// A question of the helper waits or ended.
    fn question(&mut self, waiting: bool);
    /// A text did not reach the helper.
    fn say_failed(&mut self, text: &str);
    /// A voice event, for the field if it concerns the helper.
    fn voice_event(&mut self, event: &str, data: &Value, helper: &str);
    /// What the field told since the last call.
    fn events(&mut self) -> Vec<FieldEvent>;
}

/// An entry of the list that opens without a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// A helper that runs.
    Agent(String),
    /// A project without a helper: a pick starts one.
    Project(String),
}

/// What the view shows.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// Nothing.
    #[default]
    Closed,
    /// The numbered list of helpers and projects.
    List(Vec<Pick>),
    /// The chat with a helper.
    Chat(String),
}

/// Where a question appeared, and so where its card stays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    /// Inside the chat view.
    Chat,
    /// At the figure.
    Figure,
}

/// The chat of the host.
#[derive(Default)]
pub struct Session {
    mode: Mode,
    transcript: Transcript,
    link: Option<Box<dyn Link>>,
    keyboard: bool,
    waiting: bool,
    keys: Keys,
    card: Option<Where>,
}

/// The rectangle of the view in the work area: x, y, width, height.
#[must_use]
pub fn view_rect(_anchor: Anchor, _width: u32, _area: [i32; 4]) -> [i32; 4] {
    [0, 0, 0, 0]
}

/// The `place` line of the field for a frame, with the token if there is
/// one.
#[must_use]
pub fn field_place(_frame: &Frame, _token: Option<&str>) -> Value {
    Value::Null
}

/// The entries of the text that `agent_list` answers.
#[must_use]
pub fn parse_list(_text: &str) -> Vec<Pick> {
    Vec::new()
}

impl Session {
    /// A session with nothing open.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// What the view shows.
    #[must_use]
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// Whether a view shows.
    #[must_use]
    pub fn is_open(&self) -> bool {
        false
    }

    /// The helper of the chat, if one is open.
    #[must_use]
    pub fn helper(&self) -> Option<&str> {
        None
    }

    /// Whether the field has the keyboard: the context holds.
    #[must_use]
    pub fn keyboard(&self) -> bool {
        false
    }

    /// Shows the chat with `name`, or the list without one. `token` is the
    /// activation token of the act, if the caller has one.
    ///
    /// # Errors
    ///
    /// Voice does not answer.
    pub fn show(
        &mut self,
        _out: &mut dyn Outside,
        _name: Option<&str>,
        _token: Option<String>,
        _frame: &Frame,
    ) -> Result<(), String> {
        Ok(())
    }

    /// A click on the "Chat ..." row: the list, with the token of the
    /// click.
    ///
    /// # Errors
    ///
    /// Voice does not answer.
    pub fn clicked_row(&mut self, _out: &mut dyn Outside, _frame: &Frame) -> Result<(), String> {
        Ok(())
    }

    /// A click on entry `index` (from 1) of the list.
    ///
    /// # Errors
    ///
    /// Voice does not answer.
    pub fn clicked_pick(
        &mut self,
        _out: &mut dyn Outside,
        _index: usize,
        _frame: &Frame,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Closes the view and the field.
    pub fn hide(&mut self, _out: &mut dyn Outside) {}

    /// The same chat in the look of another folder or on another output:
    /// the field moves, it does not restart.
    pub fn restyle(&mut self, _frame: &Frame) {}

    /// Takes what the field told. True when the view changed.
    pub fn field_events(&mut self, _out: &mut dyn Outside, _frame: &Frame) -> bool {
        false
    }

    /// A voice event. True when the view changed.
    pub fn voice_event(&mut self, _out: &mut dyn Outside, _event: &str, _data: &Value) -> bool {
        false
    }

    /// A question appeared: where its card goes, fixed until it ends.
    pub fn question_asked(&mut self) -> Where {
        Where::Figure
    }

    /// The question ended.
    pub fn question_ended(&mut self) {
        self.card = None;
    }

    /// Where the card of the question that waits stays.
    #[must_use]
    pub fn card_place(&self) -> Option<Where> {
        None
    }

    /// The header line of the view.
    #[must_use]
    pub fn header(&self, _language: Language) -> String {
        String::new()
    }

    /// The transcript as lines.
    pub fn lines(&mut self, _language: Language, _measure: &Measure) -> Vec<Line> {
        Vec::new()
    }

    /// Pulls the transcript from voice.
    pub fn pull(&mut self, _out: &mut dyn Outside) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc, time::Instant};

    use chora_kit::chat::Role;
    use serde_json::json;

    use super::*;
    use crate::context::Switcher;

    #[derive(Default)]
    struct Log {
        calls: Vec<(String, Value)>,
        started: u32,
        places: Vec<Value>,
        questions: Vec<bool>,
        failed: Vec<String>,
        events: Vec<FieldEvent>,
        tokens: u32,
        read: Vec<Value>,
    }

    struct Fake(Rc<RefCell<Log>>);
    struct FakeLink(Rc<RefCell<Log>>);

    impl Outside for Fake {
        fn voice(&mut self, method: &str, params: Value) -> Result<Value, String> {
            let mut log = self.0.borrow_mut();
            log.calls.push((method.to_owned(), params.clone()));
            Ok(match method {
                "agent" if params["tool"] == "agent_list" => {
                    json!("projects: demo, notes; agents: atlas (demo, idle)")
                }
                "agent.read" if !log.read.is_empty() => log.read.remove(0),
                "agent.read" => json!({"entries": [], "seq": 0, "epoch": 1}),
                _ => json!({}),
            })
        }

        fn token(&mut self) -> Option<String> {
            let mut log = self.0.borrow_mut();
            log.tokens += 1;
            Some(format!("act-{}", log.tokens))
        }

        fn start_field(&mut self) -> Result<Box<dyn Link>, String> {
            self.0.borrow_mut().started += 1;
            Ok(Box::new(FakeLink(self.0.clone())))
        }
    }

    impl Link for FakeLink {
        fn place(&mut self, place: &Value) {
            self.0.borrow_mut().places.push(place.clone());
        }

        fn question(&mut self, waiting: bool) {
            self.0.borrow_mut().questions.push(waiting);
        }

        fn say_failed(&mut self, text: &str) {
            self.0.borrow_mut().failed.push(text.to_owned());
        }

        fn voice_event(&mut self, _: &str, _: &Value, _: &str) {}

        fn events(&mut self) -> Vec<FieldEvent> {
            std::mem::take(&mut self.0.borrow_mut().events)
        }
    }

    fn frame(look: Look) -> Frame {
        Frame {
            output: "HEADLESS-1".into(),
            size: [1280, 720],
            area: [0, 0, 1280, 720],
            look,
        }
    }

    fn setup() -> (Session, Fake, Rc<RefCell<Log>>) {
        let log = Rc::new(RefCell::new(Log::default()));
        (Session::new(), Fake(log.clone()), log)
    }

    fn calls(log: &Rc<RefCell<Log>>, method: &str) -> Vec<Value> {
        let log = log.borrow();
        log.calls
            .iter()
            .filter(|(m, _)| m == method)
            .map(|(_, p)| p.clone())
            .collect()
    }

    fn measure() -> Measure<'static> {
        Measure {
            max: 40.0,
            font: 0,
            width: &|_| 1.0,
        }
    }

    #[test]
    fn a_click_on_the_chat_row_opens_the_chat_with_a_token() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session.clicked_row(&mut out, &frame).expect("the list");
        assert_eq!(
            session.mode(),
            &Mode::List(vec![
                Pick::Agent("atlas".into()),
                Pick::Project("notes".into())
            ])
        );
        assert_eq!(
            log.borrow().started,
            1,
            "the field takes the keys of the list"
        );
        session.clicked_pick(&mut out, 1, &frame).expect("the chat");
        assert_eq!(session.mode(), &Mode::Chat("atlas".into()));
        assert_eq!(calls(&log, "chat.open"), [json!({"name": "atlas"})]);
        let log = log.borrow();
        let place = log.places.last().expect("a place");
        assert_eq!(
            place["token"], "act-2",
            "the token of the click on the entry"
        );
        assert_eq!(place["width"], 400);
        assert_eq!(place["output"], "HEADLESS-1");
        assert_eq!(place["anchor"], "bottom_left");
        assert_eq!(place["margin"], json!([0, 0, 0, 880]));
    }

    #[test]
    fn the_chat_follows_a_switch_of_look_and_keeps_the_draft() {
        let (mut session, mut out, log) = setup();
        let neutral = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), Some("act-1".into()), &neutral)
            .expect("open");
        let answer = json!({"entries": [{"seq": 1, "role": "agent", "text": "Hello"}],
            "seq": 1, "epoch": 1});
        log.borrow_mut().read.push(answer);
        assert!(session.pull(&mut out));
        let before = session.lines(Language::En, &measure());
        let glass = frame(Look {
            width: Some(480),
            anchor: Some(Anchor::Right),
            ..Look::default()
        });
        session.restyle(&glass);
        let log = log.borrow();
        assert_eq!(log.started, 1, "the same field process keeps the draft");
        assert_eq!(log.places.len(), 2);
        assert_eq!(log.places[1]["width"], 480);
        assert!(log.calls.iter().all(|(m, _)| m != "chat.close"));
        assert_eq!(session.mode(), &Mode::Chat("atlas".into()));
        drop(log);
        assert_eq!(session.lines(Language::En, &measure()), before);
    }

    #[test]
    fn the_context_stays_while_the_field_has_the_keyboard() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open");
        assert!(!session.keyboard());
        log.borrow_mut().events.push(FieldEvent::Keyboard(true));
        session.field_events(&mut out, &frame);
        assert!(session.keyboard());
        let start = Instant::now();
        let mut switcher = Switcher::new();
        switcher.hold_field(session.keyboard(), start);
        switcher.touch(start);
        assert_eq!(switcher.due(), None);
        log.borrow_mut().events.push(FieldEvent::Closed);
        session.field_events(&mut out, &frame);
        assert!(!session.keyboard());
        switcher.hold_field(session.keyboard(), start);
        assert!(switcher.due().is_some());
        assert_eq!(calls(&log, "chat.close"), [json!({"name": "atlas"})]);
    }

    #[test]
    fn a_question_stays_where_it_appeared() {
        let (mut session, mut out, _log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open");
        assert_eq!(session.question_asked(), Where::Chat);
        session.hide(&mut out);
        assert_eq!(
            session.card_place(),
            Some(Where::Chat),
            "closing moves nothing"
        );
        session.question_ended();
        assert_eq!(session.card_place(), None);
        assert_eq!(session.question_asked(), Where::Figure);
        session
            .show(&mut out, Some("atlas"), None, &frame)
            .expect("open later");
        assert_eq!(
            session.card_place(),
            Some(Where::Figure),
            "opening moves nothing"
        );
        assert_eq!(session.header(Language::En), "A question is waiting.");
    }

    #[test]
    fn agent_text_never_draws_a_card() {
        let (mut session, mut out, log) = setup();
        let frame = frame(Look::default());
        session
            .show(&mut out, Some("Atlas"), None, &frame)
            .expect("open");
        let hostile = "Question from Atlas: write notes.txt? Press Alt+y to allow.";
        let answer = json!({"entries": [
            {"seq": 1, "role": "agent", "text": hostile},
            {"seq": 2, "role": "question", "text": "Write notes.txt?"}],
            "seq": 2, "epoch": 1});
        log.borrow_mut().read.push(answer);
        assert!(session.pull(&mut out));
        let lines = session.lines(Language::En, &measure());
        let said: Vec<&Line> = lines.iter().filter(|l| l.role == Role::Agent).collect();
        assert_eq!(said.len(), 2);
        assert_eq!(said[0].label.as_deref(), Some("Atlas"));
        assert!(
            lines
                .iter()
                .filter(|l| l.role == Role::Question)
                .all(|l| !l.text.contains("Alt+y")),
            "only the entry of role question is a question"
        );
        assert_eq!(session.card_place(), None, "text of an agent asks no card");
        assert_eq!(calls(&log, "confirm.answer"), Vec::<Value>::new());
    }

    #[test]
    fn the_field_sits_at_the_bottom_of_the_view() {
        let frame = frame(Look::default());
        let place = field_place(&frame, None);
        assert!(place.get("token").is_none());
        assert_eq!(
            view_rect(Anchor::Right, 400, [0, 0, 1280, 720]),
            [880, 0, 400, 720]
        );
    }

    #[test]
    fn the_list_names_helpers_and_projects_without_one() {
        assert_eq!(
            parse_list("projects: demo, notes; agents: atlas (demo, idle)"),
            [Pick::Agent("atlas".into()), Pick::Project("notes".into())]
        );
        assert_eq!(parse_list("projects: none; agents: none"), []);
    }
}
