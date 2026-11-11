//! Pointer, key and drag input: press, move, release, drop, the list
//! keys.

use chora_kit::words::{Language, Word, say};

use super::{
    Action, BTN_LEFT, ClientError, Command, Construct, Content, Drag, Event, FLY_MS, Grab, Instant,
    KEYS_WALK_AGAIN, LIST, LIST_ROW, LOOK, Moving, NAME, Pane, PressState, VOICE, Voice, WHEEL,
    cube, give, hud, space, window_flown_into,
};

/// The text of a typed key that a search takes: printable text without
/// Ctrl, Alt or Super.
fn typed(text: Option<&str>, modifiers: &[String]) -> Option<String> {
    let chord = modifiers.iter().any(|m| {
        matches!(
            m.to_lowercase().as_str(),
            "ctrl" | "control" | "alt" | "super" | "logo" | "mod4"
        )
    });
    text.filter(|t| !chord && !t.is_empty() && !t.chars().any(char::is_control))
        .map(str::to_owned)
}

/// Whether the notice of the walk keys may show again, at most once in
/// [`KEYS_WALK_AGAIN`].
pub(crate) fn keys_walk_due(last: Option<Instant>, now: Instant) -> bool {
    last.is_none_or(|at| now.duration_since(at) >= KEYS_WALK_AGAIN)
}

impl Construct {
    /// A typed key did nothing: the HUD says that the keys walk and which
    /// key shows them, at most once in [`KEYS_WALK_AGAIN`].
    fn keys_walk(&mut self) -> Result<(), ClientError> {
        let now = Instant::now();
        if !keys_walk_due(self.keys_walked, now) {
            return Ok(());
        }
        self.keys_walked = Some(now);
        let help = (self.settings.value.keys.iter())
            .find(|(_, action)| **action == Action::Help)
            .map_or("F1", |(key, _)| key.as_str());
        let words = say(Language::of_session(), Word::KeysWalk, &[("key", help)]);
        self.voice.notice(&words, now);
        self.draw_hud()
    }

    /// A give key: gives the focused window, else the window of the pane
    /// ahead, to the window manager `to`. A refusal is a notice.
    fn give_key(&mut self, to: &str) -> Result<(), ClientError> {
        if let Some(sel) = self
            .list
            .as_ref()
            .filter(|l| l.give.is_none())
            .map(|l| l.sel)
        {
            let chosen = self.rows()?.into_iter().nth(sel).map(|row| row.target);
            self.close_list()?;
            if let Some(surface) = chosen
                && let Err(err) = self.give(&surface, to)
            {
                eprintln!("chora-construct: give to {to}: {err}");
                let language = Language::of_session();
                let words = say(language, Word::GiveRefused, &[("manager", to)]);
                self.voice.notice(&words, Instant::now());
                return self.draw_hud();
            }
            return Ok(());
        }
        let focused = self.rt.conn.focused_surface().map_err(ClientError::Rpc)?;
        let ahead = (self.pane_ahead()).and_then(|i| self.panes[i].window().map(String::from));
        let node = match (focused, ahead) {
            (Some(node), _) => Some(node),
            (None, Some(surface)) => self.rt.conn.query(&format!("id:{surface}"))?.pop(),
            (None, None) => None,
        };
        let language = Language::of_session();
        let words = match node {
            None => say(language, Word::NoWindowToGive, &[]),
            Some(node) if node.owner.as_deref() == Some(to) => return Ok(()),
            Some(node) => match give::window(self, Some(&node), to) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    eprintln!("chora-construct: give to {to}: {}", err.message);
                    say(language, Word::GiveRefused, &[("manager", to)])
                }
            },
        };
        self.voice.notice(&words, Instant::now());
        self.draw_hud()
    }

    /// The events of the voice agent and of the watchdog, then the input.
    pub(crate) fn other(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == VOICE => self.follow(&event, data),
            Event::ClientConnected { name } if name == VOICE => {
                self.claim.due = true;
                self.claim_indicator(false);
                Ok(())
            }
            Event::ClientConnected { ref name } | Event::ClientDisconnected { ref name }
                if self.manages(name) =>
            {
                self.manager_client(&event)
            }
            Event::ClientDisconnected { name } if name == VOICE => {
                self.claim.due = true;
                self.voice = Voice::default();
                self.rt
                    .conn
                    .try_apply(vec![hud::fade(hud::HUD, 0.0, hud::FADE_OUT)])?;
                self.draw_ask()
            }
            Event::EffectsDropped { client } if client == NAME => {
                if !self.glass.plain {
                    eprintln!(
                        "chora-construct: the watchdog took the shaders; the panels stay \
                         plain until the settings change"
                    );
                }
                self.glass.plain = true;
                self.effects.dropped();
                self.bends_dropped();
                self.draw_hud()
            }
            Event::LayersDropped { nodes, .. } => {
                if !nodes.is_empty() {
                    eprintln!("chora-construct: layers stay out: {}", nodes.join(" "));
                }
                self.float_dropped(&nodes)?;
                let ours = [
                    "construct-hud",
                    "construct-ask",
                    "construct-list",
                    "construct-label",
                ];
                let hide = nodes.iter().any(|n| ours.iter().any(|o| n.starts_with(o)));
                if hide != self.glass.hidden {
                    self.glass.hidden = hide;
                    self.draw_hud()?;
                    self.draw_ask()?;
                }
                Ok(())
            }
            event => self.input(event),
        }
    }

    /// The keys and the pointer of the walk, and the fly-out binding.
    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    pub(crate) fn input(&mut self, event: Event) -> Result<(), ClientError> {
        if self.start_input(&event)?
            || self.hint_input(&event)?
            || self.help_input(&event)?
            || self.list_input(&event)?
            || self.modes_input(&event)?
        {
            return Ok(());
        }
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                modifiers,
                text,
                ..
            } if node == LOOK => match self.settings.value.action(&key, &modifiers) {
                Some(Action::Terminal) => self.start_terminal(),
                Some(Action::Launcher) => self.start_launcher(),
                Some(Action::Close) => return self.close_ahead(),
                Some(Action::Windows) => return self.open_list(),
                Some(Action::Help) => return self.toggle_help(),
                Some(Action::Cube) => return self.cube_key(),
                Some(action) => self.act(action),
                None if text.is_some() => return self.keys_walk(),
                None => {}
            },
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                ..
            } => {
                if let Some(&Grab::Give(to)) = self.rt.action(&binding) {
                    return self.give_key(to);
                }
                return self.fly_out();
            }
            Event::InputButton {
                state, position, ..
            } => {
                let at = self.local(position);
                let inside = self.inside();
                return match state {
                    PressState::Pressed => match inside {
                        Some(surface) => self.drag_from_window(at, &surface),
                        None => self.press(at, true),
                    },
                    PressState::Released if inside.is_some() => Ok(()),
                    PressState::Released => self.release(at, None),
                };
            }
            Event::InputMoved { position, .. } => return self.moved(self.local(position)),
            Event::PointerPressed {
                node,
                button,
                injected,
                ..
            } if button == BTN_LEFT && self.voice.answer(&node, injected).is_some() => {
                self.answer(&node, injected);
                return Ok(());
            }
            Event::PointerEntered { node, .. } => {
                let hovered = self.own_window(&node).then_some(node);
                if hovered != self.hovered {
                    self.hovered = hovered;
                    return self.frame();
                }
            }
            Event::PointerLeft { node } if self.hovered.as_deref() == Some(node.as_str()) => {
                self.hovered = None;
                return self.frame();
            }
            Event::PointerPressed { node, injected, .. } if self.own_window(&node) => {
                if injected && !self.rt.accepts_injected() {
                    return Ok(());
                }
                return self.rt.conn.apply(vec![Command::Focus {
                    surface: Some(node),
                }]);
            }
            Event::PointerPressed {
                node,
                position,
                modifiers,
                injected,
                ..
            } if node == LOOK || self.texture_pane(&node).is_some() => {
                let held = modifiers.contains(&self.settings.value.modifier);
                self.press(position, held)?;
                if let Some(drag) = &mut self.drag {
                    drag.injected = injected && !self.rt.accepts_injected();
                }
                return Ok(());
            }
            Event::PointerMoved { node, position, .. }
                if node == LOOK || self.texture_pane(&node).is_some() =>
            {
                return self.moved(position);
            }
            Event::PointerReleased { node, position, .. }
                if node == LOOK || self.texture_pane(&node).is_some() =>
            {
                return self.release(position, self.texture_pane(&node));
            }
            Event::PointerScroll { node, dy, .. } if node == hud::ASK_GLASS => {
                if let Some(command) = self.voice.scroll(dy) {
                    self.rt.conn.apply(vec![command])?;
                }
            }
            Event::PointerScroll {
                node,
                position,
                dy,
                modifiers,
                ..
            } if node == LOOK || self.texture_pane(&node).is_some() => {
                let held = modifiers.contains(&self.settings.value.modifier);
                let pane = self.texture_pane(&node).or_else(|| self.pane_at(position));
                match pane.filter(|_| held) {
                    Some(i) => return self.resize_pane(i, (-dy * 0.005).exp()),
                    None => self.walk(-dy * WHEEL, 0.0),
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Whether a node is the window of a pane of the construct, which
    /// takes the input itself.
    pub(crate) fn own_window(&self, node: &str) -> bool {
        (self.panes.iter()).any(|p| p.attached && p.window() == Some(node))
    }

    /// The pane whose shader node of its source is `node`.
    pub(crate) fn texture_pane(&self, node: &str) -> Option<usize> {
        (self.panes.iter()).position(|p| p.texture().as_deref() == Some(node))
    }

    /// A press on the space or on a pane that shows a node texture, at a
    /// point of the output: with the modifier on a pane, it starts to move
    /// the pane; else a drag looks around. The walk takes the keyboard.
    pub(crate) fn press(&mut self, at: [f64; 2], held: bool) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if self.flown.is_none() {
            self.rt.conn.apply(vec![Command::FocusNode {
                node: Some(LOOK.into()),
            }])?;
        }
        if let Some(i) = self.pane_at(at).filter(|_| held) {
            let pane = &self.panes[i];
            self.moving = Some(Moving {
                name: pane.name.clone(),
                from: at,
                at: pane.at,
                yaw: pane.yaw,
            });
            let grab = space::hit(&self.lens, &self.pose, at, pane.at, pane.size, pane.yaw);
            if let Some((_, [u, v])) = grab.filter(|_| pane.attached) {
                let target = crate::wobble::wobble_target(pane);
                self.effects.grab_sail(&pane.name, &target, [u, v, 0.0]);
            }
        } else {
            self.drag = Some(Drag {
                press: at,
                last: at,
                moved: false,
                injected: false,
            });
        }
        Ok(())
    }

    /// The surface that the camera is in or on its way into.
    pub(crate) fn inside(&self) -> Option<String> {
        window_flown_into(self.flown.as_ref(), self.glide.as_ref()).map(String::from)
    }

    /// The drag with the modifier on the window that the camera is in:
    /// the camera flies out, and the pane of the window moves on. The
    /// grabbed drag comes in scene coordinates, also over other outputs.
    pub(crate) fn drag_from_window(
        &mut self,
        at: [f64; 2],
        surface: &str,
    ) -> Result<(), ClientError> {
        let Some(pane) = self.panes.iter().find(|p| p.source == surface) else {
            return Ok(());
        };
        self.moving = Some(Moving {
            name: pane.name.clone(),
            from: at,
            at: pane.at,
            yaw: pane.yaw,
        });
        self.leave_window()
    }

    /// Flies out of the window that the camera is in, or turns back from
    /// the flight into it, and the walk takes the keyboard back.
    pub(crate) fn leave_window(&mut self) -> Result<(), ClientError> {
        if self.flown.is_some() {
            return self.fly_out();
        }
        if let Some(into) = self.glide.take().and_then(|g| g.into) {
            self.glide_to(into.back, FLY_MS, None);
            return self.rt.conn.apply(vec![Command::FocusNode {
                node: Some(LOOK.into()),
            }]);
        }
        Ok(())
    }

    pub(crate) fn moved(&mut self, at: [f64; 2]) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if self.moving.is_some() {
            return self.drag_pane(at);
        }
        self.look(at)
    }

    /// The end of a press: a moved pane drops, a click on a pane that
    /// shows a node texture flies into it. A click on the space moves
    /// nothing.
    pub(crate) fn release(&mut self, at: [f64; 2], pane: Option<usize>) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if self.moving.is_some() && self.cube.is_some() {
            let screen = (self.output.as_ref())
                .map(|o| o.name.clone())
                .unwrap_or_default();
            if self.drop_on_a_face(&screen, at)? {
                return Ok(());
            }
        }
        if let Some(moving) = self.moving.take() {
            self.effects.release(&moving.name);
            return self.drop_pane(&moving, at);
        }
        let click = self.drag.take().is_some_and(|d| !d.moved && !d.injected);
        if let Some(i) = pane.filter(|_| click) {
            return self.fly_in(i);
        }
        let commands = self.label_commands();
        self.rt.conn.apply(commands)
    }

    /// The top left corner of the output in the scene.
    pub(crate) fn origin(&self) -> [f64; 2] {
        (self.output.as_ref()).map_or([0.0; 2], |o| o.position.map(f64::from))
    }

    /// A point of the scene in the coordinates of the output.
    pub(crate) fn local(&self, [x, y]: [f64; 2]) -> [f64; 2] {
        let [ox, oy] = self.origin();
        [x - ox, y - oy]
    }

    /// The drop of a moved pane at a point of the output (D55). Over a
    /// plugin pane, its window goes to that plugin; off the output, the
    /// pane goes back to where it was, and its window to the window
    /// manager whose claim holds the point. A window of another client
    /// goes back to its owner instead. A refused window stays a pane.
    pub(crate) fn drop_pane(&mut self, moving: &Moving, at: [f64; 2]) -> Result<(), ClientError> {
        let Ok(i) = self.pane_named(&moving.name) else {
            return Ok(());
        };
        if self.overlay {
            return Ok(());
        }
        let inside = (0.0..self.lens.w).contains(&at[0]) && (0.0..self.lens.h).contains(&at[1]);
        let to = if inside {
            let plugin =
                |p: &Pane| p.name != moving.name && matches!(p.content, Content::Plugin { .. });
            match self.pane_hit(at, plugin).map(|k| &self.panes[k].content) {
                Some(Content::Plugin { plugin, .. }) => Some(plugin.clone()),
                _ => None,
            }
        } else {
            self.panes[i].at = moving.at;
            self.panes[i].yaw = moving.yaw;
            self.rt.conn.apply(self.placed(&self.panes[i]))?;
            let [ox, oy] = self.origin();
            (self.rt.conn)
                .claimant_at([at[0] + ox, at[1] + oy])
                .map_err(ClientError::Rpc)?
        };
        let pane = &self.panes[i];
        if let (Some(to), Some(surface)) = (to, pane.window().map(String::from)) {
            let given = if pane.taken {
                self.forget(&surface)
            } else {
                self.give(&surface, &to).map(drop)
            };
            if let Err(err) = given {
                eprintln!("chora-construct: {surface} stays: {err}");
            }
        }
        self.frame()
    }

    /// The pointer over a row of the start card fills it, and a click on
    /// it starts what its keys start; whether the card took the event.
    fn start_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        use crate::start::{Act, act, row_of};
        if !self.start_card.up {
            return Ok(false);
        }
        match event {
            Event::PointerPressed {
                node,
                button,
                injected,
                ..
            } if *button == BTN_LEFT && row_of(node).is_some() => {
                if *injected && !self.rt.accepts_injected() {
                    return Ok(true);
                }
                match row_of(node).and_then(act) {
                    Some(Act::Terminal) => self.start_terminal(),
                    Some(Act::Programs) => self.start_launcher(),
                    Some(Act::Cube) => self.cube_key()?,
                    None => {}
                }
                Ok(true)
            }
            Event::PointerEntered { node, .. } | Event::PointerLeft { node }
                if row_of(node).is_some() =>
            {
                let row = row_of(node);
                let over = matches!(event, Event::PointerEntered { .. });
                let hovered = if over {
                    row
                } else {
                    self.start_card.hover.filter(|h| Some(*h) != row)
                };
                if hovered != self.start_card.hover {
                    self.start_card.hover = hovered;
                    let commands = crate::start::hover(&self.styled(), hovered);
                    self.rt.conn.apply(commands)?;
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// A click on a hint, the pointer over a hint, and the pointer at the
    /// bottom of the output; whether a hint took the event.
    fn hint_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        match event {
            Event::PointerPressed {
                node,
                button,
                injected,
                ..
            } if *button == BTN_LEFT
                && node.starts_with(crate::hints::HINT)
                && (!injected || self.rt.accepts_injected()) =>
            {
                self.hint_press(node)
            }
            Event::PointerEntered { node, .. } => self.hint_hovered(node, true),
            Event::PointerLeft { node } => self.hint_hovered(node, false),
            Event::InputMoved { position, .. } => {
                let [_, y] = self.local(*position);
                self.hint_pointer(y)?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// The key of the cube opens one while none is open; a refusal is a
    /// notice.
    pub(crate) fn cube_key(&mut self) -> Result<(), ClientError> {
        if self.cube.is_some() {
            return Ok(());
        }
        if let Err(err) = self.open_cube(cube::FACES) {
            self.voice.notice(&err.to_string(), Instant::now());
            return self.draw_hud();
        }
        Ok(())
    }

    /// The keys and the clicks of the open window list; whether it took
    /// the event. Typed letters filter it, Backspace takes one back, Esc
    /// clears the search and then closes it. A press on the space closes
    /// the list and goes on.
    #[allow(clippy::cognitive_complexity, reason = "one arm per key and event")]
    pub(crate) fn list_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        let Some((sel, searching)) = self.list.as_ref().map(|l| (l.sel, !l.query.is_empty()))
        else {
            return Ok(false);
        };
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                modifiers,
                text,
                ..
            } if node == LOOK => {
                let toggle = self.settings.value.keys.get(key) == Some(&Action::Windows);
                match key.as_str() {
                    "Escape" if searching => self.search_list(String::clear),
                    "Escape" => self.close_list(),
                    _ if toggle => self.close_list(),
                    "Up" => self.select(sel.saturating_sub(1)),
                    "Down" => self.select(sel + 1),
                    "Return" | "KP_Enter" => self.choose(sel),
                    "BackSpace" => self.search_list(|q| {
                        q.pop();
                    }),
                    _ => match typed(text.as_deref(), modifiers) {
                        Some(typed) => self.search_list(|q| q.push_str(&typed)),
                        None => Ok(()),
                    },
                }?;
            }
            Event::PointerPressed { node, .. } if node.starts_with(LIST) => {
                let row =
                    (node.strip_prefix(LIST_ROW)).and_then(|r| r.split('-').next()?.parse().ok());
                if let Some(k) = row {
                    self.choose(k)?;
                }
            }
            Event::PointerEntered { node, .. } | Event::PointerLeft { node }
                if node.starts_with(LIST_ROW) =>
            {
                let over = matches!(event, Event::PointerEntered { .. });
                let row = (node.strip_prefix(LIST_ROW)).and_then(|r| r.parse::<usize>().ok());
                let hovered = if over { row } else { None };
                if let Some(list) = self.list.as_mut().filter(|l| l.hovered != hovered) {
                    list.hovered = hovered;
                    self.draw_list(false)?;
                }
            }
            Event::PointerPressed { node, .. } if node == LOOK => {
                self.close_list()?;
                return Ok(false);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Changes the search of the list, chooses its first row and draws it.
    fn search_list(&mut self, change: impl FnOnce(&mut String)) -> Result<(), ClientError> {
        if let Some(list) = self.list.as_mut() {
            change(&mut list.query);
            list.sel = 0;
        }
        self.draw_list(false)
    }

    /// The keys of the open help; whether it took the event. It takes
    /// every key: typed letters search, Backspace takes one back, the
    /// arrows move, Esc clears the search and then closes the help, and
    /// its own key closes it.
    #[allow(clippy::cognitive_complexity, reason = "one arm per key")]
    pub(crate) fn help_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        if !self.help {
            return Ok(false);
        }
        let Event::NodeKey {
            node,
            key,
            state: PressState::Pressed,
            modifiers,
            text,
            ..
        } = event
        else {
            return Ok(false);
        };
        if node != LOOK {
            return Ok(false);
        }
        let own = self.settings.value.keys.get(key) == Some(&Action::Help);
        match key.as_str() {
            "Escape" if !self.help_query.is_empty() => {
                self.help_query.clear();
                self.help_sel = 0;
            }
            "Escape" => return self.toggle_help().map(|()| true),
            _ if own => return self.toggle_help().map(|()| true),
            "Up" => self.help_sel = self.help_sel.saturating_sub(1),
            "Down" => self.help_sel += 1,
            "BackSpace" => {
                self.help_query.pop();
                self.help_sel = 0;
            }
            _ => match typed(text.as_deref(), modifiers) {
                Some(typed) => {
                    self.help_query.push_str(&typed);
                    self.help_sel = 0;
                }
                None => return Ok(true),
            },
        }
        self.draw_help(false)?;
        Ok(true)
    }

    /// A drag with the modifier moves a pane: sideways along its face,
    /// and up the screen away from the camera, at the pace of the
    /// pointer at the pane's depth.
    pub(crate) fn drag_pane(&mut self, position: [f64; 2]) -> Result<(), ClientError> {
        let Some(moving) = &self.moving else {
            return Ok(());
        };
        let Ok(i) = self.pane_named(&moving.name) else {
            return Ok(());
        };
        let k = space::depth(&self.pose, moving.at).max(self.lens.focal * 0.1) / self.lens.focal;
        let (dx, dy) = (position[0] - moving.from[0], position[1] - moving.from[1]);
        let [fx, fz] = self.pose.forward();
        let (right, ahead) = (dx * k, -dy * k);
        let at = moving.at;
        let to = [
            at[0] + fx * ahead + fz * right,
            at[1] + fz * ahead - fx * right,
        ];
        let (from, yaw) = (moving.at, moving.yaw);
        self.move_pane(i, to)?;
        let pane = &self.panes[i];
        let [dx, dz] = [pane.at[0] - from[0], pane.at[1] - from[1]];
        let (sin, cos) = yaw.sin_cos();
        let origin = [dx * cos - dz * sin, 0.0, dx * sin + dz * cos];
        let target = crate::wobble::wobble_target(pane);
        match self.effects.motion_3d(&pane.name, &target, origin) {
            Some(command) => {
                let commands = self.followed(vec![command]);
                self.rt.conn.apply(commands)
            }
            None => Ok(()),
        }
    }
}
