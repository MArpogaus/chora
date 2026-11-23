//! The methods of the regions, the agents and the follow-up of events,
//! split from the root of the plugin.

use chora_kit::runtime;
use chora_scene::ipc::{ClientError, RpcError};
use serde_json::{Value, json};

use crate::{
    ASSISTANT, AddLine, Chat, Holder, Look, Mc, NAME, Put, Query, SetContent, Show, SoloParams,
    Toggle, VOICE, agent, config, invalid, keys, mic, page_id, region, region::MAX_LINES,
};

impl Mc {
    /// The methods of the regions and their contents.
    pub(crate) fn region_method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let rejected = |e: ClientError| RpcError::new(RpcError::REJECTED, e.to_string());
        match method {
            "pane.focus" => {
                let Show { name } = runtime::params(params)?;
                // A scene that is not shown takes no keyboard (review
                // rc3 round 4, 2.7).
                if self.current.as_deref() != Some(name.as_str()) {
                    return Err(RpcError::new(
                        RpcError::REJECTED,
                        format!("the scene {name} is not the current one"),
                    ));
                }
                self.focus_scene(&name)
                    .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))?;
                Ok(Value::Null)
            }
            "region.list" => Ok(self.list()),
            "region.toggle" => {
                let Toggle { region, shown } = runtime::params(params)?;
                let shown = self.toggle(region, shown, false).map_err(rejected)?;
                Ok(json!({"shown": shown}))
            }
            "region.put" => {
                let put: Put = runtime::params(params)?;
                self.put_content(put)?;
                self.render().map_err(rejected)?;
                Ok(Value::Null)
            }
            "content.set" => {
                let SetContent {
                    content,
                    title,
                    lines,
                } = runtime::params(params)?;
                self.set_text(&content, title, |_| lines)?;
                Ok(Value::Null)
            }
            "content.add" => {
                let AddLine { content, line } = runtime::params(params)?;
                self.set_text(&content, None, |mut lines| {
                    lines.push(line);
                    let extra = lines.len().saturating_sub(MAX_LINES);
                    lines.split_off(extra)
                })?;
                Ok(Value::Null)
            }
            "region.solo" => {
                let SoloParams {
                    content,
                    region,
                    shown,
                } = runtime::params(params)?;
                let found = (self.settings.value.contents.get(&content))
                    .ok_or_else(|| invalid(format!("no content {content}")))?;
                region::put_check(found, region).map_err(invalid)?;
                let shown = (self.solo(&content, region, shown, false)).map_err(rejected)?;
                Ok(json!({"shown": shown}))
            }
            _ => self.agent_method(method, params),
        }
    }

    /// An event of the voice agent (D65). The HUD host
    /// draws the state, the sentences and the questions of the agent, so
    /// mc takes from its own state only the state of the microphone;
    /// `voice.state` with a `name` sets that helper as `agent.set` does,
    /// and `voice.remove` removes it. Wrong data only logs, since the
    /// emitter gets no answer; other events do nothing.
    pub(crate) fn follow(
        &mut self,
        event: &str,
        client: &str,
        mut data: Value,
    ) -> Result<(), ClientError> {
        let kind = event.strip_prefix(client).and_then(|e| e.strip_prefix('.'));
        let helper = data.get("name").is_some();
        if kind == Some("state") && !helper && self.mic_follows(mic::Mic::of(&data)) {
            self.render()?;
        }
        let done = match kind {
            Some("state") if helper => {
                if let Some(object) = data.as_object_mut() {
                    object.retain(|key, _| agent::STATE_FIELDS.contains(&key.as_str()));
                }
                self.voiced(&data);
                runtime::params(data).and_then(|set| self.set_agent(set))
            }
            Some("remove") => runtime::params(data).and_then(|r| self.remove_agent(&r)),
            _ => Ok(()),
        };
        if let Err(e) = done {
            tracing::warn!("{event}: {e}");
        }
        Ok(())
    }

    /// An event of a client (D65): of the voice agent, or `assistant.chat`
    /// and `assistant.ready` of the HUD host. `voice.*` also matches a client named `voice.evil`:
    /// only the agent itself shows its state (review 4.9).
    pub(crate) fn emitted(
        &mut self,
        event: &str,
        client: &str,
        data: Value,
    ) -> Result<(), ClientError> {
        match client {
            VOICE => self.follow(event, client, data),
            ASSISTANT if event == "assistant.chat" => self.chat_drawer(&data),
            ASSISTANT if event == "assistant.chat.keyboard" => {
                if data["on"] == true {
                    self.focus.lose();
                }
                Ok(())
            }
            ASSISTANT if event == "assistant.ready" => self.name_theme(),
            _ => Ok(()),
        }
    }

    /// Whether input is a person's: not from `input.inject`, or mc takes
    /// that as the person's.
    fn by_person(&self, injected: bool) -> bool {
        !injected || self.rt.accepts_injected()
    }

    /// A press of the pointer: on a column of a helper it opens the chat,
    /// else it is a click. The columns and the hits ignore a press of
    /// `input.inject`, as the bindings do.
    pub(crate) fn press(
        &mut self,
        node: &str,
        at: [f64; 2],
        injected: bool,
    ) -> Result<(), ClientError> {
        let person = self.by_person(injected);
        if (person && self.click_agent(node, at))
            || self.click_mic(node, injected)
            || (person && self.click_hit(node))
        {
            return Ok(());
        }
        self.click(node, at, injected)
    }

    /// `assistant.chat` of the HUD host: while it shows the chat on the
    /// output of mc, the margin column holds it: the notifications fold,
    /// the key list goes, and mc emits the box under the head `Chat` as
    /// `mc.chat.slot {output, rect}`; `rect` is `null` when it closes.
    fn chat_drawer(&mut self, data: &Value) -> Result<(), ClientError> {
        let Some(output) = self.output.as_ref().map(|o| o.name.clone()) else {
            return Ok(());
        };
        let here = data.get("output").and_then(Value::as_str) == Some(output.as_str());
        let open = here && data.get("open").and_then(Value::as_bool) == Some(true);
        if open == (self.chat == Chat::Open) {
            return Ok(());
        }
        self.chat = if open { Chat::Open } else { Chat::Closed };
        self.render()?;
        let still_open = data.get("open").and_then(Value::as_bool) == Some(true);
        let mut ours = self.windows.values().chain(self.docks.values());
        let taken = (self.core_focus.as_ref()).is_some_and(|s| !ours.any(|o| o == s));
        if open || still_open || taken {
            return Ok(());
        }
        let commands = self.keyboard_back();
        self.apply(commands)
    }

    /// Gives the keyboard to the page that shows, when no window and no
    /// node has it: at the start, after the chat of the host closed and
    /// after a panel built the page anew, so that the plain keys of a page
    /// work without a click.
    pub(crate) fn take_free_keyboard(&mut self) -> Result<(), ClientError> {
        let Some(name) = self.current.clone() else {
            return Ok(());
        };
        if self.page_keys == keys::PageKeys::Bound || self.windows.contains_key(&name) {
            return Ok(());
        }
        let query = Query {
            selector: "*".into(),
            filter: Some(chora_scene::ipc::Filter {
                focused: true,
                ..chora_scene::ipc::Filter::default()
            }),
        };
        let focused: Vec<Value> = self.rt.conn.call("scene.query", query)?;
        if !focused.is_empty() {
            return Ok(());
        }
        let focus = self.focus.to_node(Some(page_id(&name)));
        self.apply(vec![focus])
    }

    /// The voice agent came or went: the state of its microphone is
    /// unknown until its next state.
    pub(crate) fn voice_changed(&mut self) -> Result<(), ClientError> {
        if self.mic_follows(mic::Mic::Unknown) {
            self.render()?;
        }
        Ok(())
    }

    /// A key without a modifier while a page has the keyboard: Right and
    /// Left turn the page, Home and `XF86HomePage` go to the first page,
    /// Escape goes back one step.
    pub(crate) fn page_key(&mut self, key: &str, modifiers: &[String], injected: bool) {
        let look = match key {
            _ if !modifiers.is_empty() || !self.by_person(injected) => return,
            "Home" | "XF86HomePage" => Look::Scene(self.settings.value.default_scene.clone()),
            "Right" => Look::NextScene,
            "Left" => Look::PreviousScene,
            "Escape" => {
                self.plain_turn = true;
                self.back();
                self.plain_turn = false;
                return;
            }
            "Return" => return self.enter_window(),
            _ => return,
        };
        self.plain_turn = true;
        self.turn(look);
        self.plain_turn = false;
    }

    /// Draws the key list anew when the plain keys of the page began or
    /// ended to work, as when the keyboard moves between a window and its
    /// page.
    pub(crate) fn sync_keys(&mut self) -> Result<(), ClientError> {
        let named = self.chat == Chat::Closed && self.built && self.current.is_some();
        if named && self.page_keys_work() != self.keys_plain.get() {
            self.render()?;
        }
        Ok(())
    }

    /// Whether the plain keys of the page turn it now: the page of a window
    /// holds them only while the page has the keyboard.
    pub(crate) fn page_keys_work(&self) -> bool {
        let page = (self.current.as_deref()).is_some_and(|scene| !self.is_app(scene));
        let holds_page = matches!(self.focus.holder(NAME, |_| false), Holder::Node(_));
        self.page_keys == keys::PageKeys::Plain && (page || holds_page)
    }

    /// Return on a page that stands over a window gives the window the
    /// keyboard: the keys of the person then belong to the program, and
    /// only the bindings with Super turn the page (mc-42).
    fn enter_window(&mut self) {
        let window = self
            .current
            .as_ref()
            .and_then(|c| self.windows.get(c))
            .cloned();
        if let Some(window) = window
            && self.chat == Chat::Closed
        {
            let focus = self.focus.to(Some(window));
            if let Err(err) = self.apply(vec![focus]).and_then(|()| self.sync_keys()) {
                tracing::warn!("enter window: {err}");
            }
        }
    }

    /// The binding `back`, `XF86Back` of a remote: a window that holds the
    /// keyboard leaves it to its page, which keeps it as after a plain turn;
    /// on a page it is one step back as Escape (mc-46).
    pub(crate) fn step_back(&mut self) {
        let current = self.current.clone();
        let window = current.as_ref().and_then(|c| self.windows.get(c)).cloned();
        let holds = window.as_ref().is_some_and(|w| {
            matches!(self.focus.holder(NAME, |s| s == w), Holder::Window(_))
                && self.chat == Chat::Closed
        });
        self.plain_turn = true;
        match current {
            Some(name) if holds => {
                let commands = self.keyboard_to(&name, window);
                if let Err(err) = self.apply(commands).and_then(|()| self.sync_keys()) {
                    tracing::warn!("leave window: {err}");
                }
            }
            _ => self.back(),
        }
        self.plain_turn = false;
    }

    /// One step back: the full key list closes; else the page that showed
    /// before shows again, and at the end of that trail the first page.
    fn back(&mut self) {
        if self.keys == keys::List::All {
            self.keys = keys::List::Six;
            if let Err(err) = self.render() {
                tracing::warn!("keys: {err}");
            }
            return;
        }
        let current = self.current.clone();
        let known =
            |name: &String| self.scenes.contains_key(name) && Some(name) != current.as_ref();
        let mut trail = std::mem::take(&mut self.trail);
        let back = std::iter::from_fn(|| trail.pop()).find(known);
        let first = Some(self.settings.value.default_scene.clone()).filter(known);
        if let Some(name) = back.or(first)
            && let Err(err) = self.show(&name, true)
        {
            chora_kit::runtime::report(crate::NAME, &format_args!("{}: {}", name, err.message));
        }
        self.trail = trail;
    }

    /// Names the theme of mc to the HUD host, which colours its look by it.
    pub(crate) fn name_theme(&mut self) -> Result<(), ClientError> {
        let dark = self.settings.value.theme == config::Theme::Dark;
        let emit = chora_kit::hud::theme_event(NAME, dark);
        self.rt.conn.call::<_, Value>("events.emit", emit).map(drop)
    }

    /// The methods of the agents and the notifications.
    fn agent_method(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        match method {
            "agent.set" => self.set_agent(runtime::params(params)?),
            "agent.remove" => self.remove_agent(&runtime::params(params)?),
            "agent.list" => return Ok(self.list_agents()),
            "agent.focus" => self.focus_agent(&runtime::params(params)?),
            "notify.push" => self.push(&runtime::params(params)?),
            "notify.clear" => self.clear_notices(),
            _ => Err(RpcError::new(RpcError::METHOD_NOT_FOUND, method)),
        }
        .map(|()| Value::Null)
    }
}
