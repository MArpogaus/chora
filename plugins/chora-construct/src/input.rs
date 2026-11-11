//! Pointer, key and drag input: press, move, release, drop, the list
//! keys.

use chora_kit::effects::Target;

use super::{
    Action, BTN_LEFT, ClientError, Command, Construct, Content, Drag, Event, FLY_MS, Grab, Instant,
    LIST, LIST_ROW, LOOK, Moving, NAME, Pane, PressState, VOICE, Voice, WHEEL, give, hud, space,
    window_flown_into,
};

/// The node that the wobble of a pane bends: the group of the pane, in
/// its own coordinates.
fn wobble_target(pane: &Pane) -> Target {
    Target {
        node: pane.node(),
        origin: [0.0; 2],
        size: pane.size,
        set_size: true,
    }
}

impl Construct {
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
        if self.list_input(&event)? || self.modes_input(&event)? {
            return Ok(());
        }
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => match self.settings.value.keys.get(&key) {
                Some(Action::Terminal) => self.start_terminal(),
                Some(Action::Launcher) => self.start_launcher(),
                Some(Action::Close) => return self.close_ahead(),
                Some(Action::Windows) => return self.open_list(),
                Some(Action::Help) => return self.toggle_help(),
                Some(&action) => self.act(action),
                None => {}
            },
            Event::InputKey {
                binding,
                state: PressState::Pressed,
                ..
            } => {
                if let Some(&Grab::Give(to)) = self.rt.action(&binding) {
                    if let Err(err) = give::focused(self, to) {
                        eprintln!("chora-construct: give to {to}: {}", err.message);
                    }
                    return Ok(());
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
                let target = wobble_target(pane);
                (self.effects).grab_3d(&pane.name, &target, [0.0; 3], [u, v, 0.0], [0.0; 2]);
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
    /// the flight into it.
    pub(crate) fn leave_window(&mut self) -> Result<(), ClientError> {
        if self.flown.is_some() {
            return self.fly_out();
        }
        if let Some(into) = self.glide.take().and_then(|g| g.into) {
            self.glide_to(into.back, FLY_MS, None);
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
        if let Some(moving) = self.moving.take() {
            self.effects.release(&moving.name);
            return self.drop_pane(&moving, at);
        }
        let click = self.drag.take().is_some_and(|d| !d.moved && !d.injected);
        if let Some(i) = pane.filter(|_| click) {
            self.fly_in(i);
            return Ok(());
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

    /// The keys and the clicks of the open window list; whether it took
    /// the event. A press on the space closes the list and goes on.
    pub(crate) fn list_input(&mut self, event: &Event) -> Result<bool, ClientError> {
        let Some(sel) = self.list.as_ref().map(|l| l.sel) else {
            return Ok(false);
        };
        match event {
            Event::NodeKey {
                node,
                key,
                state: PressState::Pressed,
                ..
            } if node == LOOK => {
                let toggle = self.settings.value.keys.get(key) == Some(&Action::Windows);
                match key.as_str() {
                    "Escape" => self.close_list(),
                    _ if toggle => self.close_list(),
                    "Up" => self.select(sel.saturating_sub(1)),
                    "Down" => self.select(sel + 1),
                    "Return" | "KP_Enter" => self.choose(sel),
                    "g" => self.give_menu(sel),
                    _ => Ok(()),
                }?;
            }
            Event::PointerPressed { node, .. } if node.starts_with(LIST) => {
                let row =
                    (node.strip_prefix(LIST_ROW)).and_then(|r| r.split('-').next()?.parse().ok());
                if let Some(k) = row {
                    self.choose(k)?;
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
        let target = wobble_target(pane);
        match self.effects.motion_3d(&pane.name, &target, origin) {
            Some(command) => self.rt.conn.apply(vec![command]),
            None => Ok(()),
        }
    }

    /// One frame of the wobbles of the panes.
    pub(crate) fn wobble_frame(&mut self) -> Result<(), ClientError> {
        let panes = &self.panes;
        let frame = self.effects.tick(Instant::now(), |name| {
            panes.iter().find(|p| p.name == name).map(wobble_target)
        });
        if frame.commands.is_empty() {
            return Ok(());
        }
        self.rt.conn.try_apply(frame.commands).map(drop)
    }
}
