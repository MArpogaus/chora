//! Pointer, key and drag input: press, move, release, drop, the list
//! keys.

use crate::camera::take_flight;
use chora_kit::actions::{System, perform::perform};
use chora_kit::words::{Language, Word, say};

use super::{
    Action, BTN_LEFT, ClientError, Construct, Content, Drag, Event, FLY_MS, Grab, Instant,
    KEYS_WALK_AGAIN, LIST, LIST_ROW, LOOK, Moving, NAME, Own, Pane, PressState, VOICE, WHEEL, cube,
    give, space, window_flown_into,
};
use chora_kit::wm::focus;

/// The notice of a window manager that did not take a window, with the
/// reason from the error.
pub(crate) fn refusal(language: Language, manager: &str, error: &str) -> String {
    let reason = crate::names::reason(error, language);
    crate::names::refused(language, manager, &reason)
}

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

/// The row of the window list that a press chooses: a left press of the
/// person, or of `input.inject` while the plugin `accepts_injected`.
pub(crate) fn pressed_row(event: &Event, accepts_injected: bool) -> Option<usize> {
    let Event::PointerPressed {
        node,
        button: BTN_LEFT,
        injected,
        ..
    } = event
    else {
        return None;
    };
    if *injected && !accepts_injected {
        return None;
    }
    (node.strip_prefix(LIST_ROW)).and_then(|r| r.split('-').next()?.parse().ok())
}

/// A key press of the open window list: the key, its modifiers and its text.
pub(crate) type ListKey<'a> = (&'a str, &'a [String], Option<&'a str>);

/// The key of a press on the look that the walk, the cube and the open
/// window list act on: a key of the person, or of `input.inject` while the
/// plugin `accepts_injected`.
pub(crate) fn walk_key(event: &Event, accepts_injected: bool) -> Option<ListKey<'_>> {
    match event {
        Event::NodeKey {
            node,
            key,
            state: PressState::Pressed,
            modifiers,
            text,
            code: _,
            injected,
        } if node == LOOK && (!injected || accepts_injected) => {
            Some((key, modifiers, text.as_deref()))
        }
        _ => None,
    }
}

/// Whether a press, a move or a release of the pointer on the look or on
/// a pane texture acts: one of the person, or of `input.inject` while the
/// plugin `accepts_injected`. The press focuses the look and starts a
/// drag that looks around or moves a pane.
pub(crate) fn look_pointer(event: &Event, accepts_injected: bool) -> bool {
    chora_kit::runtime::from_person(event, accepts_injected)
}

impl Construct {
    /// Whether a node is the look or a pane that shows a node texture.
    fn on_look(&self, node: &str) -> bool {
        node == LOOK || self.texture_pane(node).is_some()
    }

    /// The act of a key press of the walk: the terminal, the close key, the
    /// windows, the cube, the stride keys and the rest of the actions.
    fn walk_press(
        &mut self,
        key: &str,
        modifiers: &[String],
        typed: bool,
    ) -> Result<(), ClientError> {
        match self.settings.value.action(key, modifiers).cloned() {
            Some(Action::System(system)) => return perform(self, &system),
            Some(Action::Look(Own::Windows)) => return self.open_list(),
            Some(Action::Look(Own::Cube)) => self.cube_key(),
            Some(Action::Look(own @ (Own::Forward | Own::Back | Own::Left | Own::Right))) => {
                self.stride_press(key, own);
            }
            Some(Action::Look(own)) => self.act(own),
            None if typed => self.keys_walk(),
            None => {}
        }
        Ok(())
    }

    /// The grab of a chord that the core still sent although a regrab let
    /// it go (the keyboard went back to the walk while the press was on
    /// its way): in the walk it is the key of the walk, so it acts as the
    /// grab would have.
    fn let_go_grab(&self, binding: &str) -> Option<Grab> {
        if self.in_pane() || self.cube.is_some() {
            return None;
        }
        (self.grab_keys().into_iter())
            .find(|(key, _)| {
                key.parse::<chora_scene::Binding>()
                    .is_ok_and(|b| b.to_string() == binding)
            })
            .map(|(_, grab)| grab)
    }

    /// A typed key did nothing: the HUD says that the keys walk and which
    /// key shows them, at most once in [`KEYS_WALK_AGAIN`].
    pub(crate) fn keys_walk(&mut self) {
        let now = Instant::now();
        if !keys_walk_due(self.keys_walked, now) {
            return;
        }
        self.keys_walked = Some(now);
        let help = (self.settings.value.common.bindings.iter())
            .find(|(_, action)| **action == Action::System(System::Keys))
            .map_or("Super+F1", |(key, _)| key.as_str());
        let words = say(Language::of_session(), Word::KeysWalk, &[("key", help)]);
        self.tell(&words);
    }

    /// A give key: gives the focused window, else the window of the pane
    /// ahead, to the window manager `to`. Stood back with the window of a
    /// pane, the key of the face manager puts it into the face at the
    /// front (`put_window`, co-79). A refusal is a notice.
    pub(crate) fn give_key(&mut self, to: &str) -> Result<(), ClientError> {
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
                crate::log::log_warn!("chora-construct: give to {to}: {err}");
                let language = Language::of_session();
                let words = refusal(language, to, &err.to_string());
                self.tell(&words);
                return Ok(());
            }
            return Ok(());
        }
        if let Some(surface) = self.put_window().filter(|_| to == self.face_manager()) {
            return self.put_in_face(&surface, to);
        }
        let focused = self.rt.conn.focused_surface().map_err(ClientError::Rpc)?;
        let before = (self.cube.as_ref()).and_then(crate::cube::Host::noted_focus);
        let ahead = (self.pane_ahead()).and_then(|i| self.panes[i].window().map(String::from));
        let ahead = before.or(ahead);
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
                    crate::log::log_warn!("chora-construct: give to {to}: {}", err.message);
                    refusal(language, to, &err.message)
                }
            },
        };
        self.tell(&words);
        Ok(())
    }

    /// The events of the voice agent and of the watchdog, then the input.
    pub(crate) fn other(&mut self, event: Event) -> Result<(), ClientError> {
        match event {
            Event::Emitted {
                event,
                client,
                data,
            } if client == VOICE => self.follow(&event, &data),
            Event::ClientConnected { ref name } | Event::ClientDisconnected { ref name }
                if self.manages(name) =>
            {
                self.manager_client(&event)
            }
            Event::EffectsDropped { client } if client == NAME => {
                if !self.glass.plain {
                    crate::log::log_info!(
                        "chora-construct: the watchdog took the shaders; the panels stay \
                         plain until the settings change"
                    );
                }
                self.glass.plain = true;
                self.effects.dropped();
                self.bends_dropped();
                Ok(())
            }
            Event::LayersDropped { nodes, .. } => {
                if !nodes.is_empty() {
                    crate::log::log_info!("chora-construct: layers stay out: {}", nodes.join(" "));
                }
                self.float_dropped(&nodes)
            }
            event => self.input(event),
        }
    }

    /// The chat of the HUD host closed and took the keyboard with it: the
    /// walk gets it back. In a pane the window of the pane gets it from
    /// the core, and a cube keeps its own.
    pub(crate) fn keyboard_back(&mut self) -> Result<(), ClientError> {
        if !self.active || self.in_pane() || self.cube.is_some() {
            return Ok(());
        }
        let walk = self.keys_to_the_walk();
        self.rt.conn.apply(vec![walk])
    }

    /// The keys and the pointer of the walk, and the fly-out binding.
    #[allow(clippy::too_many_lines, reason = "one arm per event")]
    pub(crate) fn input(&mut self, event: Event) -> Result<(), ClientError> {
        if let Event::NodeKey {
            node,
            key,
            state: PressState::Released,
            code: _,
            modifiers: _,
            text: _,
            injected,
        } = &event
            && node == LOOK
            && (!injected || self.rt.accepts_injected())
        {
            self.stride_release(key);
        }
        if self.start_input(&event)?
            || self.door_input(&event)?
            || self.hint_input(&event)?
            || self.help_input(&event)?
            || self.list_input(&event)?
            || self.modes_input(&event)?
        {
            return Ok(());
        }
        let accepts = self.rt.accepts_injected();
        if let Some((key, modifiers, text)) = walk_key(&event, accepts) {
            return self.walk_press(key, modifiers, text.is_some());
        }
        match event {
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                .. // injected: the runtime drops an injected binding key unless it accepts them
            } => {
                let grab = (self.rt.action(&binding).cloned()).or_else(|| self.let_go_grab(&binding));
                return match grab {
                    Some(Grab::Give(to)) => self.give_key(to),
                    Some(Grab::Swallow) => Ok(()),
                    Some(Grab::Own(Own::Windows)) => self.open_list(),
                    Some(Grab::Own(_)) => {
                        self.cube_key();
                        Ok(())
                    }
                    Some(Grab::Key(System::Terminal)) => self.terminal_beside(),
                    Some(Grab::Key(system)) => self.grabbed_key(&system),
                    Some(Grab::FlyOut) => self.step_out(),
                    _ => self.fly_out(),
                };
            }
            Event::InputButton {
                state, position, .. // injected: Construct::event drops an injected pointer event first
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
            Event::InputMoved { position, .. } => return self.moved(self.local(position)), // injected: Construct::event drops an injected pointer event first
            Event::PointerEntered { node, .. } => { // injected: Construct::event drops an injected pointer event first
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
                self.walk_keys = false;
                return self.rt.conn.apply(vec![focus::give(node)]);
            }
            Event::PointerPressed {
                node,
                position,
                modifiers,
                .. // injected: look_pointer in the guard drops an injected press
            } if look_pointer(&event, accepts) && self.on_look(&node) => {
                let held = modifiers.contains(&self.settings.value.modifier);
                return self.press(position, held);
            }
            Event::PointerMoved { node, position, .. } // injected: look_pointer in the guard drops an injected move
                if look_pointer(&event, accepts) && self.on_look(&node) =>
            {
                return self.moved(position);
            }
            Event::PointerReleased { node, position, .. } // injected: look_pointer in the guard drops an injected release
                if look_pointer(&event, accepts) && self.on_look(&node) =>
            {
                return self.release(position, self.texture_pane(&node));
            }
            Event::PointerScroll {
                node,
                position,
                dy,
                modifiers,
                .. // injected: Construct::event drops an injected pointer event first
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
    /// the pane; else a drag looks around, unless a cube is open: its view
    /// is fixed, and the cube must not leave it. The walk takes the keyboard.
    pub(crate) fn press(&mut self, at: [f64; 2], held: bool) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if held && self.hand_press(at) {
            return Ok(());
        }
        if self.flown.is_none() {
            let walk = self.keys_to_the_walk();
            self.rt.conn.apply(vec![walk])?;
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
        } else if self.cube.is_none() {
            self.drag = Some(Drag {
                press: at,
                last: at,
                moved: false,
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
        if let Some(into) = take_flight(&mut self.glide) {
            self.glide_to(into.back, FLY_MS, None);
            let walk = self.keys_to_the_walk();
            self.rt.conn.apply(vec![walk])?;
            return self.regrab();
        }
        Ok(())
    }

    pub(crate) fn moved(&mut self, at: [f64; 2]) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if self.hand_moved(at)? {
            return Ok(());
        }
        if self.moving.is_some() {
            return self.drag_pane(at);
        }
        self.look(at)
    }

    /// The end of a press: a moved pane drops, a click on a pane that
    /// shows a node texture flies into it, and so does a click on a pane
    /// while the cube stands back, as Tab and Return do. A click on the
    /// space moves nothing.
    pub(crate) fn release(&mut self, at: [f64; 2], pane: Option<usize>) -> Result<(), ClientError> {
        self.pointer = Some(at);
        if self.hand_released()? {
            return Ok(());
        }
        if let Some(i) = self.clicked_pane_stood_back(at) {
            return self.fly_in(i);
        }
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
        let click = self.drag.take().is_some_and(|d| !d.moved);
        if let Some(i) = pane.filter(|_| click) {
            return self.fly_in(i);
        }
        let commands = self.label_commands();
        self.rt.conn.apply(commands)
    }

    /// The pane of a press that the cube stood back gave to a pane and
    /// that ends as a click, within `CLICK` of the press: the press moves
    /// that pane no more.
    fn clicked_pane_stood_back(&mut self, at: [f64; 2]) -> Option<usize> {
        let stood_back = (self.cube.as_ref())
            .is_some_and(|c| c.view().mode() == chora_kit::modes::Mode::Navigate);
        let moving = self.moving.as_ref().filter(|_| stood_back)?;
        let far = (at[0] - moving.from[0]).hypot(at[1] - moving.from[1]);
        if far > crate::cube::CLICK {
            return None;
        }
        let moving = self.moving.take()?;
        self.effects.release(&moving.name);
        self.pane_named(&moving.name).ok()
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
                crate::log::log_warn!("chora-construct: {surface} stays: {err}");
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
                    Some(Act::Cube) => self.cube_key(),
                    None => {}
                }
                Ok(true)
            }
            Event::PointerEntered { node, .. } | Event::PointerLeft { node } // injected: Construct::event drops an injected pointer event first
                if row_of(node).is_some() =>
            {
                let row = row_of(node);
                let over = matches!(event, Event::PointerEntered { .. }); // injected: Construct::event drops an injected pointer event first
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
            Event::PointerEntered { node, .. } => self.hint_hovered(node, true), // injected: Construct::event drops an injected pointer event first
            Event::PointerLeft { node } => self.hint_hovered(node, false),
            Event::InputMoved { position, .. } => {
                // injected: Construct::event drops an injected pointer event first
                let [_, y] = self.local(*position);
                self.hint_pointer(y)?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// The key of the cube opens one while none is open, stood back, so
    /// that the person sees the cube of desktops and every pane beside it
    /// (co-52, co-54); a refusal is a notice. The window of a pane that
    /// had the keyboard (`cube_window`) is noted as at a step back, though
    /// the cube that covers it took the keyboard, so the give key of the
    /// face manager puts it into the face and the bar names it (co-79).
    pub(crate) fn cube_key(&mut self) {
        if self.cube.is_some() {
            return;
        }
        let had = self.cube_window();
        let opened = (self.open_cube(cube::FACES))
            .and_then(|_| self.mode_act(chora_kit::modes::Act::StepBack, crate::modes::By::Person));
        if let Err(err) = opened {
            self.tell(&err.to_string());
            return;
        }
        if let (Some(host), Some(window)) = (self.cube.as_mut(), had)
            && host.noted_focus().is_none()
        {
            host.note_focus(Some(window));
        }
        if let Err(err) = self.draw_hints() {
            crate::log::log_warn!("chora-construct: the hints after the cube opened: {err}");
        }
    }

    /// The keys and the clicks of the open window list; whether it took
    /// the event. Typed letters filter it, Backspace takes one back, Esc
    /// clears the search and then closes it. A press on the space closes
    /// the list and goes on.
    pub(crate) fn list_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        let Some((sel, searching)) = self.list.as_ref().map(|l| (l.sel, !l.query.is_empty()))
        else {
            return Ok(false);
        };
        match event {
            Event::NodeKey { node, state, .. } if node == LOOK => { // injected: walk_key below drops an injected key, which the list takes
                let Some((key, modifiers, text)) = walk_key(event, self.rt.accepts_injected())
                else {
                    return Ok(*state == PressState::Pressed);
                };
                let toggle = self.settings.value.common.bindings.get(key) == Some(&Action::Look(Own::Windows));
                match key {
                    "Escape" if searching => self.search_list(String::clear),
                    "Escape" => self.close_list(),
                    _ if toggle => self.close_list(),
                    "Up" => self.select(sel.saturating_sub(1)),
                    "Down" => self.select(sel + 1),
                    "Return" | "KP_Enter" => self.choose(sel),
                    "BackSpace" => self.search_list(|q| {
                        q.pop();
                    }),
                    _ => match typed(text, modifiers) {
                        Some(typed) => self.search_list(|q| q.push_str(&typed)),
                        None => Ok(()),
                    },
                }?;
            }
            Event::PointerPressed { node, .. } if node.starts_with(LIST) => { // injected: pressed_row ignores an injected press
                if let Some(k) = pressed_row(event, self.rt.accepts_injected()) {
                    self.choose(k)?;
                }
            }
            Event::PointerEntered { node, .. } | Event::PointerLeft { node } // injected: Construct::event drops an injected pointer event first
                if node.starts_with(LIST_ROW) =>
            {
                let over = matches!(event, Event::PointerEntered { .. }); // injected: Construct::event drops an injected pointer event first
                let row = (node.strip_prefix(LIST_ROW)).and_then(|r| r.parse::<usize>().ok());
                let hovered = if over { row } else { None };
                if let Some(list) = self.list.as_mut().filter(|l| l.hovered != hovered) {
                    list.hovered = hovered;
                    self.draw_list(false)?;
                }
            }
            Event::PointerPressed { node, .. } // injected: look_pointer in the guard drops an injected press
                if node == LOOK && look_pointer(event, self.rt.accepts_injected()) =>
            {
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
    /// its own key closes it, as a press on its Close or outside it.
    pub(crate) fn help_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        if !self.help {
            return Ok(false);
        }
        if let Event::PointerPressed {
            node,
            button: BTN_LEFT,
            injected,
            ..
        } = event
            && node.starts_with(crate::hud::HELP)
            && (node.ends_with("-hit") || node.ends_with("-scrim"))
            && (!injected || self.rt.accepts_injected())
        {
            return self.toggle_help().map(|()| true);
        }
        let Some((key, modifiers, text)) = walk_key(event, self.rt.accepts_injected()) else {
            return Ok(false);
        };
        let own = self.settings.value.action(key, modifiers) == Some(&Action::System(System::Keys));
        match key {
            "Escape" => return self.close_help_by_escape().map(|()| true),
            _ if own => return self.toggle_help().map(|()| true),
            "Up" => self.help_sel = self.help_sel.saturating_sub(1),
            "Down" => self.help_sel += 1,
            "BackSpace" => {
                self.help_query.pop();
                self.help_sel = 0;
            }
            _ => match typed(text, modifiers) {
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

    /// Escape in the open help clears the search, and closes the help when
    /// there is none.
    pub(crate) fn close_help_by_escape(&mut self) -> Result<(), ClientError> {
        if !self.help {
            return Ok(());
        }
        if self.help_query.is_empty() {
            return self.toggle_help();
        }
        self.help_query.clear();
        self.help_sel = 0;
        self.draw_help(false)
    }

    /// A drag with the modifier moves a pane: sideways along its face,
    /// and up the screen away from the camera, at the pace of the
    /// pointer at the pane's depth, turned to where the camera stays at
    /// the end of a glide.
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
        let heads_to = self.target();
        self.move_pane_facing(i, to, [heads_to.x, heads_to.z])?;
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
