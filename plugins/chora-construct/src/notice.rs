//! The notices of the construct: it draws none, `chora-notify` does.

use chora_kit::notice::{self, Notice, Urgency};

use crate::{Construct, NAME, NOTICE_MS};

impl Construct {
    /// Tells the person `text` in a notice of `chora-notify`, in the place
    /// of the notice of the construct that still shows. Without the
    /// server the person gets none, and the construct goes on.
    pub(crate) fn tell(&mut self, text: &str) {
        let notice = Notice {
            app: NAME,
            icon: "",
            summary: text,
            body: "",
            expire_ms: NOTICE_MS,
            urgency: Urgency::Normal,
        };
        let shown = notice::show_or_log(&mut self.rt.conn, self.notice_id, &notice);
        self.notice_id = shown.or(self.notice_id);
    }
}

#[cfg(test)]
mod tests {
    use chora_kit::notice;
    use serde_json::{Value, json};

    use crate::{config::Action, state::tests::construct};

    /// The calls of `notify.show` that the construct made, as the fake core
    /// recorded them.
    fn shown(core: &chora_kit::testing::FakeCore) -> Vec<Value> {
        let called = core.model().lock().unwrap().called.clone();
        (called.into_iter())
            .filter(|call| call["method"] == notice::SHOW)
            .collect()
    }

    /// A key that did not start its program is a call of `notify.show` of
    /// the app `construct`, not a line that the construct draws.
    #[test]
    fn a_key_that_did_not_start_its_program_gives_a_notice() {
        let (mut c, core) = construct("notice-key", false);
        core.answer_with(notice::SHOW, json!({"id": 3}));
        c.shared_key(Action::Chat).unwrap();
        let calls = shown(&core);
        assert_eq!(calls.len(), 1, "{calls:?}");
        assert_eq!(calls[0]["params"]["app"], "construct");
        assert_eq!(calls[0]["params"]["urgency"], "normal");
        assert_eq!(calls[0]["params"]["expire_ms"], 4000);
    }

    /// A give with no window to give is a call of `notify.show`; the next
    /// replaces it, so that notices do not pile up.
    #[test]
    fn a_refused_give_gives_a_notice_that_replaces_the_last() {
        let (mut c, core) = construct("notice-give", false);
        core.answer_with(notice::SHOW, json!({"id": 3}));
        c.give_key("xp").unwrap();
        c.give_key("xp").unwrap();
        let calls = shown(&core);
        assert_eq!(calls.len(), 2, "{calls:?}");
        assert_eq!(calls[0]["params"]["replaces"], Value::Null);
        assert_eq!(calls[1]["params"]["replaces"], 3);
    }

    /// Typed text that does nothing is one notice, then none for ten
    /// seconds.
    #[test]
    fn typed_text_that_does_nothing_gives_one_notice() {
        let (mut c, core) = construct("notice-walk", false);
        core.answer_with(notice::SHOW, json!({"id": 3}));
        c.keys_walk();
        c.keys_walk();
        assert_eq!(shown(&core).len(), 1);
    }

    /// A notice that finds no server is logged, and the construct goes on.
    #[test]
    fn a_notice_without_a_server_does_not_stop_the_construct() {
        let (mut c, core) = construct("notice-gone", false);
        core.refuse_next("methods.call");
        c.keys_walk();
        c.shared_key(Action::Chat).unwrap();
    }
}
