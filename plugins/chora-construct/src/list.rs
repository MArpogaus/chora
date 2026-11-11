//! The window list and the give menu: windows without a visible pane, a
//! choice, and giving a window to a manager.

use super::{
    ClientError, Construct, Entry, Filter, Hold, Kind, ListState, NAME, PaneFile, Query, Row,
    RpcError, Value, give, handed_over, hud, space, takes_the_camera, unique,
};

impl give::Windows for Construct {
    fn conn(&mut self) -> &mut chora_kit::conn::Conn {
        &mut self.rt.conn
    }

    fn has(&self, surface: &str) -> bool {
        (self.panes.iter()).any(|p| p.window() == Some(surface) && p.attached)
    }

    fn hand(&mut self, surface: &str, to: &str) -> Result<Value, RpcError> {
        self.give(surface, to)
            .map_err(|e| RpcError::new(RpcError::REJECTED, e.to_string()))
    }
}

impl Construct {
    /// The surface with the keyboard, as the core has it.
    pub(crate) fn focused_surface(&mut self) -> Result<Option<String>, ClientError> {
        let query = Query {
            selector: "kind:surface".into(),
            filter: Some(Filter {
                focused: true,
                ..Filter::default()
            }),
        };
        let nodes: Vec<Value> = self.rt.conn.call("scene.query", query)?;
        Ok((nodes.first()).and_then(|n| Some(n["id"].as_str()?.to_owned())))
    }

    /// All windows (F29): the panes first, then the windows of any
    /// owner that have no pane, but not the launcher's own window. A pane
    /// behind the resting front face of the cube is `covered`.
    pub(crate) fn windows(&mut self) -> Result<Vec<Entry>, ClientError> {
        let nodes = self.rt.conn.query("kind:surface")?;
        let far = self.radius();
        let mut out: Vec<Entry> = (self.panes.iter())
            .filter_map(|p| {
                let surface = p.window()?;
                let node = nodes.iter().find(|n| n.id == surface);
                Some(Entry {
                    name: p.name.clone(),
                    surface: surface.into(),
                    owner: node.and_then(|n| n.owner.clone()),
                    pane: if self
                        .cube
                        .as_ref()
                        .is_some_and(|c| c.covers(&self.pose, p.at))
                    {
                        "covered"
                    } else {
                        space::sight(&self.lens, &self.pose, p.spot(), far)
                    },
                    size: None,
                })
            })
            .collect();
        for node in nodes {
            let paned = out.iter().any(|e| e.surface == node.id);
            if paned || self.launcher_window.as_deref() == Some(&node.id) {
                continue;
            }
            let info = node.surface.unwrap_or_default();
            let base = (info.app_id.or(info.title)).unwrap_or_else(|| node.id.clone());
            out.push(Entry {
                name: unique(&base, |n| out.iter().any(|e| e.name == n)),
                surface: node.id,
                owner: node.owner,
                pane: "none",
                size: info.size.map(|s| s.map(f64::from)),
            });
        }
        Ok(out)
    }

    /// The windows without a visible pane: the rows of the list.
    pub(crate) fn hidden(&mut self) -> Result<Vec<Entry>, ClientError> {
        Ok((self.windows()?.into_iter())
            .filter(|e| e.pane != "shown")
            .collect())
    }

    /// Flies to the pane of a window, or takes the window of another
    /// client as a pane at its own size first (F30). Answers the pane.
    pub(crate) fn show(&mut self, entry: &Entry) -> Result<String, ClientError> {
        if entry.pane == "none" {
            let file = PaneFile {
                kind: Kind::Program,
                command: None,
                plugin: None,
                scene: None,
                at: None,
                yaw: None,
                size: entry.size,
            };
            let hold = match entry.owner.as_deref() {
                None if !self.overlay => Hold::Own,
                Some(NAME) => Hold::Own,
                _ => Hold::Taken,
            };
            self.adopt(&entry.surface, &entry.name, Some(&file), hold, true)?;
        }
        let Some(i) = (self.panes.iter()).position(|p| p.window() == Some(&entry.surface)) else {
            return Err(ClientError::Rpc(RpcError::new(
                RpcError::REJECTED,
                format!("the window {} did not become a pane", entry.name),
            )));
        };
        if self.flown.is_some() {
            self.fly_out()?;
        }
        self.fly_in(i);
        Ok(self.panes[i].name.clone())
    }

    pub(crate) fn open_list(&mut self) -> Result<(), ClientError> {
        if self.list.is_some() {
            return self.close_list();
        }
        self.list = Some(ListState { sel: 0, give: None });
        self.draw_list(true)
    }

    pub(crate) fn select(&mut self, sel: usize) -> Result<(), ClientError> {
        if let Some(list) = &mut self.list {
            list.sel = sel;
        }
        self.draw_list(false)
    }

    /// The rows of the window managers to give the window of row `sel`
    /// to; only a window that the construct owns can go.
    pub(crate) fn give_menu(&mut self, sel: usize) -> Result<(), ClientError> {
        let Some(row) = self.rows()?.into_iter().nth(sel) else {
            return Ok(());
        };
        let own = (self.panes.iter()).any(|p| p.window() == Some(&row.target) && p.attached);
        if let Some(list) = self.list.as_mut().filter(|l| own && l.give.is_none()) {
            *list = ListState {
                sel: 0,
                give: Some(row.target),
            };
            return self.draw_list(false);
        }
        Ok(())
    }

    /// The rows of the open list: the windows without a visible pane, or
    /// the window managers to give a window to.
    pub(crate) fn rows(&mut self) -> Result<Vec<Row>, ClientError> {
        if self.list.as_ref().is_some_and(|l| l.give.is_some()) {
            return Ok((self.managers()?.into_iter())
                .map(|m| Row {
                    label: format!("give to {m}"),
                    note: "window manager".into(),
                    target: m,
                })
                .collect());
        }
        Ok((self.hidden()?.into_iter())
            .map(|e| {
                let owner = e.owner.as_deref().unwrap_or("no owner");
                let place = if e.pane == "none" { "no pane" } else { e.pane };
                Row {
                    label: e.name,
                    note: format!("{owner} \u{b7} {place}"),
                    target: e.surface,
                }
            })
            .collect())
    }

    pub(crate) fn close_list(&mut self) -> Result<(), ClientError> {
        if self.list.take().is_some() {
            let mut commands = hud::list_close();
            commands.extend(self.label_commands());
            self.rt.conn.try_apply(commands)?;
        }
        Ok(())
    }

    /// Closes the list and shows the window of row `k`, or gives the
    /// window to the window manager of row `k`.
    pub(crate) fn choose(&mut self, k: usize) -> Result<(), ClientError> {
        let rows = self.rows()?;
        let give = self.list.as_ref().and_then(|l| l.give.clone());
        self.close_list()?;
        let Some(row) = rows.into_iter().nth(k) else {
            return Ok(());
        };
        let done = match give {
            Some(surface) => self.give(&surface, &row.target).map(drop),
            None => match self.hidden()?.into_iter().find(|e| e.surface == row.target) {
                Some(entry) => self.show(&entry).map(drop),
                None => Ok(()),
            },
        };
        if let Err(err) = done {
            eprintln!("chora-construct: window list: {err}");
        }
        Ok(())
    }

    /// The other window managers: the clients with a claim (D55).
    pub(crate) fn managers(&mut self) -> Result<Vec<String>, ClientError> {
        let mut managers: Vec<String> = (self.rt.conn.query("")?.into_iter())
            .filter(|n| n.props.claim.is_some())
            .filter_map(|n| n.owner)
            .filter(|o| o != NAME)
            .collect();
        managers.sort();
        managers.dedup();
        Ok(managers)
    }

    /// Gives an own window to the client `to`, a window manager or a
    /// plugin such as mc (D55): the core reserves the window for `to`,
    /// with its props back to the defaults, and the pane goes. The core
    /// refuses a `to` without a connection, and the pane and the camera
    /// stay. After a give the keyboard is left to `to`, also when the
    /// camera was in the window.
    pub(crate) fn give(&mut self, surface: &str, to: &str) -> Result<Value, ClientError> {
        self.give_dragged(surface, to, None)
    }

    /// As [`Self::give`]; with `grab` the drag of the window goes too
    /// (`give_grab`), when the core lets it.
    pub(crate) fn give_dragged(
        &mut self,
        surface: &str,
        to: &str,
        grab: Option<chora_scene::GivenGrab>,
    ) -> Result<Value, ClientError> {
        let refuse = |m: String| Err(ClientError::Rpc(RpcError::new(RpcError::REJECTED, m)));
        let Some(pane) = self.panes.iter().find(|p| p.window() == Some(surface)) else {
            return refuse(format!("{surface} is no pane"));
        };
        if !pane.attached {
            return refuse(format!(
                "the window {} belongs to another client, which gives it",
                pane.name
            ));
        }
        let answer = (self.rt.conn)
            .give_dragged(surface, to, grab)
            .map_err(ClientError::Rpc)?;
        self.forget_with(surface, false)?;
        Ok(answer)
    }

    /// How a pane holds a window that the construct started.
    pub(crate) fn started(&self) -> Hold {
        if self.overlay { Hold::Shown } else { Hold::Own }
    }

    /// A window that is free again, or that another window manager gives
    /// to the construct (D55). The flavor takes both; an overlay only a
    /// window given to it, so that it never takes a window on its own
    /// (F30). A window given always gets its pane, and the camera flies
    /// to it, so that it has the keyboard, unless the user is in a
    /// window already. It owns such a window, and its leave frees it again.
    pub(crate) fn detached(&mut self, surface: &str) -> Result<(), ClientError> {
        let node = self.rt.conn.query(&format!("id:{surface}"))?.pop();
        let Some(node) = node else {
            return Ok(());
        };
        if (self.panes.iter()).any(|p| p.window() == Some(surface) && p.attached) {
            return Ok(());
        }
        if self.overlay && node.to.as_deref() != Some(NAME) {
            return Ok(());
        }
        let inside = self.inside();
        let handed = node.to.as_deref() == Some(NAME);
        let info = node.surface.unwrap_or_default();
        let file = handed.then(|| handed_over(info.size.map(|s| s.map(f64::from)), &self.lens));
        let label = info.app_id.or(info.title).unwrap_or_default();
        self.forget(surface)?;
        self.adopt(surface, &label, file.as_ref(), Hold::Own, handed)?;
        if handed && takes_the_camera(inside.as_deref()) {
            self.fly_to_window(surface)?;
        }
        Ok(())
    }

    /// Flies in front of the pane of a window, from inside another pane
    /// too; the window takes the keyboard at the end of the flight.
    pub(crate) fn fly_to_window(&mut self, surface: &str) -> Result<(), ClientError> {
        let Some(i) = (self.panes.iter()).position(|p| p.window() == Some(surface)) else {
            return Ok(());
        };
        if self.flown.is_some() {
            self.fly_out()?;
        }
        self.fly_in(i);
        Ok(())
    }
}
