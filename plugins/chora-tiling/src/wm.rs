//! The state of the window manager: outputs with tags, windows with their
//! tags and states, and the focus history. It knows nothing of IPC; the
//! plugin turns its placements into commands.

use std::collections::BTreeMap;

use chora_scene::SurfaceRequest;
use serde::{Deserialize, Serialize};

use crate::{
    config::{Action, Config, Focus, NewWindow, SHARE},
    layout::{self, Layout, Params, Rect},
};

/// The stacking ranks, from the bottom. As in awesome, the last raised
/// window of an output covers the others; a fullscreen window lies below
/// them otherwise; only windows on top cover it.
const FULLSCREEN: u8 = 0;
const TILED: u8 = 1;
/// The shown window of the max, fullscreen and magnifier layouts.
const TILED_TOP: u8 = 2;
const FLOATING: u8 = 3;
const TOP: u8 = 4;
const FULLSCREEN_TOP: u8 = 5;
const ON_TOP: u8 = 6;

/// A tiled or floating window.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)] // independent states, as in awesome
pub struct Window {
    /// The meta of the node holds the rest, so it needs no id.
    #[serde(skip)]
    pub id: String,
    pub output: String,
    /// One bit per tag.
    pub tags: u16,
    pub floating: bool,
    pub fullscreen: bool,
    /// Maximized horizontally and vertically.
    pub maximized: (bool, bool),
    pub minimized: bool,
    pub on_top: bool,
    /// The window asked for attention.
    pub urgent: bool,
    /// The size that the client chose.
    pub size: Option<[i32; 2]>,
    /// Where a drag put a window that does not tile; centered without.
    #[serde(default)]
    pub at: Option<[i32; 2]>,
}

#[derive(Clone, Copy, Debug)]
struct Tag {
    layout: Layout,
    params: Params,
}

/// An output and its tags.
#[derive(Clone, Debug)]
pub struct Output {
    pub name: String,
    /// The work area in scene coordinates.
    pub area: Rect,
    /// The whole output in scene coordinates.
    pub full: Rect,
    /// One bit per shown tag.
    selected: u16,
    previous: u16,
    tags: Vec<Tag>,
}

/// The tags of an output, as `tiling.get_tags` answers them.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct OutputTags {
    pub name: String,
    /// The output has the focus.
    pub focused: bool,
    pub tags: Vec<TagState>,
}

/// The state of one tag of an output.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_excessive_bools)] // independent marks of a panel
pub struct TagState {
    /// The tag number, from 1.
    pub tag: usize,
    /// The output shows the tag.
    pub shown: bool,
    /// The output shows the tag and has the focus.
    pub focused: bool,
    /// A window of the output is on the tag.
    pub occupied: bool,
    /// A window of the output on the tag asked for attention.
    pub urgent: bool,
}

/// Where a window shows, and the size to ask its client for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: String,
    /// `None` hides the window.
    pub at: Option<[i32; 2]>,
    /// The client decides when `None`.
    pub size: Option<[i32; 2]>,
    pub states: States,
}

/// The toplevel states of a window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // the states of xdg-shell
pub struct States {
    pub tiled: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub activated: bool,
}

/// What the plugin must do after an action.
#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Relayout,
    Run(String),
    Close(String),
    Reload,
}

/// The meta key of a window node with its state, see [`Wm::window_meta`].
pub const WINDOW_KEY: &str = "window";
/// The meta key of the group with the current output.
const CURRENT_KEY: &str = "current";
/// The prefix of a meta key of the group with the tags of an output.
const TAGS_KEY: &str = "tags:";

/// A tag in meta: the layout, the master width, the masters and the
/// columns. Short, so that 16 tags fit into the 1 KiB of a meta value.
type KeptTag = (Layout, f64, usize, usize);
/// A window in meta: its states, its place among the windows, and its
/// places in the focus history and in the raise order.
type KeptWindow = (Window, usize, Option<usize>, Option<usize>);

pub struct Wm {
    pub config: Config,
    pub windows: Vec<Window>,
    pub outputs: Vec<Output>,
    /// The index of the output with the focus.
    current: usize,
    /// Window ids, the most recent focus last.
    history: Vec<String>,
    /// Window ids, the last raised one last. A click or a binding raises
    /// the window that it focuses; the focus that follows the pointer does
    /// not.
    raised: Vec<String>,
}

impl Wm {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            windows: Vec::new(),
            outputs: Vec::new(),
            current: 0,
            history: Vec::new(),
            raised: Vec::new(),
        }
    }

    /// Adds an output, or changes its areas. A new output takes the windows
    /// that have no output.
    pub fn set_output(&mut self, name: &str, area: Rect, full: Rect) {
        if let Some(output) = self.outputs.iter_mut().find(|o| o.name == name) {
            (output.area, output.full) = (area, full);
            return;
        }
        let tag = self.new_tag();
        self.outputs.push(Output {
            name: name.into(),
            area,
            full,
            selected: 1,
            previous: 1,
            tags: vec![tag; self.config.tags],
        });
        let names: Vec<String> = self.outputs.iter().map(|o| o.name.clone()).collect();
        for window in &mut self.windows {
            if !names.contains(&window.output) {
                window.output = name.into();
                window.tags = 1;
                window.at = None;
            }
        }
    }

    /// Removes an output; its windows go to the first other output, or wait
    /// for the next new output.
    pub fn remove_output(&mut self, name: &str) {
        let current = self.outputs.get(self.current).map(|o| o.name.clone());
        self.outputs.retain(|o| o.name != name);
        self.current = self
            .outputs
            .iter()
            .position(|o| Some(&o.name) == current.as_ref())
            .unwrap_or(0);
        if let Some(first) = self.outputs.first() {
            let (target, tags) = (first.name.clone(), first.selected.max(1));
            for window in self.windows.iter_mut().filter(|w| w.output == name) {
                window.output.clone_from(&target);
                window.tags = tags;
                window.at = None;
            }
        }
    }

    /// Adds a new window on the shown tags of the current output, and
    /// focuses it.
    pub fn add_window(&mut self, id: &str) {
        let (output, tags) = self
            .outputs
            .get(self.current)
            .map_or((String::new(), 1), |o| (o.name.clone(), o.selected.max(1)));
        let window = Window {
            id: id.into(),
            output,
            tags,
            floating: false,
            fullscreen: false,
            maximized: (false, false),
            minimized: false,
            on_top: false,
            urgent: false,
            size: None,
            at: None,
        };
        match self.config.new_window {
            NewWindow::Master => self.windows.insert(0, window),
            NewWindow::Stack => self.windows.push(window),
        }
        self.focus(id);
    }

    pub fn remove_window(&mut self, id: &str) {
        self.windows.retain(|w| w.id != id);
        self.history.retain(|h| h != id);
        self.raised.retain(|h| h != id);
    }

    pub fn has(&self, id: &str) -> bool {
        self.window(id).is_some()
    }

    /// Notes the size that a client chose. True when a window that does not
    /// tile must move to keep its place.
    pub fn set_size(&mut self, id: &str, size: [i32; 2]) -> bool {
        let floats = self.window(id).is_some_and(|w| !self.tiles(w));
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.size != Some(size);
        window.size = Some(size);
        changed && floats
    }

    /// Puts a window that does not tile at a point, for a drag. False for a
    /// window that tiles.
    pub fn place(&mut self, id: &str, at: [i32; 2]) -> bool {
        if self.window(id).is_none_or(|w| self.tiles(w)) {
            return false;
        }
        self.window_mut(id)
            .is_some_and(|w| w.at.replace(at) != Some(at))
    }

    /// The focused window: the top window of the current output.
    pub fn focused(&self) -> Option<&str> {
        self.last_on(self.current, &self.history)
    }

    /// Focuses and raises a window, and focuses its output. A window that
    /// shows nowhere only gets the urgent mark, as when it asks for
    /// attention.
    pub fn focus(&mut self, id: &str) {
        if !self.window(id).is_some_and(|w| self.shows(w)) {
            if let Some(window) = self.window_mut(id) {
                window.urgent = true;
            }
            return;
        }
        if self.has(id) {
            self.raised.retain(|h| h != id);
            self.raised.push(id.into());
        }
        self.focus_only(id);
    }

    /// Focuses a window and its output, without a raise.
    fn focus_only(&mut self, id: &str) {
        let Some(window) = self.window_mut(id) else {
            return;
        };
        window.urgent = false;
        let output = window.output.clone();
        self.history.retain(|h| h != id);
        self.history.push(id.into());
        if let Some(index) = self.outputs.iter().position(|o| o.name == output) {
            self.current = index;
        }
    }

    /// Focuses a window under the pointer, as the configuration says. A
    /// pointer that stood still focuses nothing. `lost`: another window
    /// manager has the focus, so the focused window takes it again.
    pub fn pointer_focus(&mut self, id: &str, clicked: bool, moved: bool, lost: bool) -> bool {
        let follow = self.config.focus == Focus::FollowMouse
            && moved
            && self
                .current_tag()
                .is_some_and(|t| t.layout != Layout::Magnifier);
        let shown = self.window(id).is_some_and(|w| self.shows(w));
        let index = self.current;
        let done = !lost
            && if clicked {
                self.last_on(index, &self.raised) == Some(id) && self.focused() == Some(id)
            } else {
                self.focused() == Some(id)
            };
        if !shown || !(clicked || follow) || done {
            return false;
        }
        if clicked {
            self.focus(id);
        } else {
            self.focus_only(id);
        }
        true
    }

    /// A request of a client, for example from its title bar.
    pub fn request(&mut self, id: &str, request: SurfaceRequest) {
        if request == SurfaceRequest::Activate {
            self.focus(id);
            return;
        }
        let Some(window) = self.window_mut(id) else {
            return;
        };
        match request {
            SurfaceRequest::Fullscreen => window.fullscreen = true,
            SurfaceRequest::Unfullscreen => window.fullscreen = false,
            SurfaceRequest::Maximize => window.maximized = (true, true),
            SurfaceRequest::Unmaximize => window.maximized = (false, false),
            SurfaceRequest::Minimize => window.minimized = true,
            SurfaceRequest::Activate | SurfaceRequest::Close => {}
        }
    }

    /// Runs an action of a binding.
    pub fn act(&mut self, action: &Action) -> Option<Effect> {
        match action {
            Action::Run(command) => return Some(Effect::Run(command.clone())),
            Action::Close => return self.focused().map(|id| Effect::Close(id.into())),
            Action::Reload => return Some(Effect::Reload),
            // The plugin handles the button events of a drag itself.
            Action::Drag => return None,
            Action::FocusNext => self.focus_next(),
            Action::FocusPrevious => self.focus_prev(),
            Action::FocusBack => self.focus_back(),
            Action::FocusUrgent => self.focus_urgent(),
            Action::SwapNext => self.swap(1),
            Action::SwapPrevious => self.swap(-1),
            Action::MoveToMaster => {
                if let Some(id) = self.focused().map(String::from) {
                    self.swap_master(&id);
                }
            }
            Action::ToggleFloating => self.toggle(|w| &mut w.floating),
            Action::ToggleFullscreen => self.toggle(|w| &mut w.fullscreen),
            Action::ToggleMaximized => self.with_focused(toggle_maximized),
            Action::ToggleMaximizedHorizontal => self.toggle(|w| &mut w.maximized.0),
            Action::ToggleMaximizedVertical => self.toggle(|w| &mut w.maximized.1),
            Action::ToggleOnTop => self.toggle(|w| &mut w.on_top),
            Action::Minimize => self.with_focused(|w| w.minimized = true),
            Action::RestoreMinimized => self.restore_minimized(),
            Action::MoveToTag(n) => {
                if let Some(id) = self.focused().map(String::from) {
                    self.move_to_tag(&id, *n);
                }
            }
            Action::ToggleWindowTag(n) => self.with_focused(|w| toggle_tag(w, *n)),
            Action::IncMasterWidth(d) => self.with_tag(|t, _| t.params.master_width = share(t, *d)),
            Action::IncMasters(d) => {
                self.with_tag(|t, _| t.params.masters = add(t.params.masters, *d, 0));
            }
            Action::IncColumns(d) => {
                self.with_tag(|t, _| t.params.columns = add(t.params.columns, *d, 1));
            }
            Action::NextLayout => self.with_tag(|t, all| t.layout = cycle(all, t.layout, 1)),
            Action::PreviousLayout => self.with_tag(|t, all| t.layout = cycle(all, t.layout, -1)),
            Action::FocusNextOutput => self.focus_output(1),
            Action::FocusPreviousOutput => self.focus_output(-1),
            Action::MoveToNextOutput => self.move_to_next_output(),
            Action::ViewTag(n) => self.view_tag(*n),
            Action::ToggleTagView(n) => self.view(|tags, _| tags ^ bit(*n)),
            Action::ViewNext => self.view(|tags, count| 1 << wrap(first_tag(tags), 1, count)),
            Action::ViewPrevious => self.view(|tags, count| 1 << wrap(first_tag(tags), -1, count)),
            Action::ViewBack => self.view_back(),
        }
        Some(Effect::Relayout)
    }

    /// Changes the layout of the current tag, from the exported method.
    pub fn set_layout(&mut self, layout: Layout) {
        self.with_tag(|t, _| t.layout = layout);
    }

    /// Shows only this tag on the current output, from `tiling.view_tag`.
    pub fn view_tag(&mut self, tag: usize) {
        self.view(|_, _| bit(tag));
    }

    /// Puts a window on this tag only, as `move_to_tag` does for the
    /// focused window. False when tiling has no such window.
    pub fn move_to_tag(&mut self, id: &str, tag: usize) -> bool {
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        window.tags = bit(tag);
        true
    }

    /// Focuses the next window that shows on the current output.
    pub fn focus_next(&mut self) {
        self.focus_step(1);
    }

    /// Focuses the previous window that shows on the current output.
    pub fn focus_prev(&mut self) {
        self.focus_step(-1);
    }

    /// Swaps a window with the master of its output, as `move_to_master`
    /// does for the focused window. False when the window shows nowhere.
    pub fn swap_master(&mut self, id: &str) -> bool {
        let Some(output) = self.window(id).map(|w| w.output.clone()) else {
            return false;
        };
        let Some(index) = self.outputs.iter().position(|o| o.name == output) else {
            return false;
        };
        let shown = self.shown_on(index);
        let Some(at) = shown.iter().position(|&i| self.windows[i].id == id) else {
            return false;
        };
        self.windows.swap(shown[at], shown[0]);
        true
    }

    /// The layout and the parameters of the current tag.
    pub fn current(&self) -> Option<(Layout, Params)> {
        self.current_tag().map(|t| (t.layout, t.params))
    }

    /// The windows that show on the current output, in their order.
    pub fn shown_windows(&self) -> Vec<String> {
        self.shown_on(self.current)
            .into_iter()
            .map(|i| self.windows[i].id.clone())
            .collect()
    }

    /// The tags of each output, for a panel.
    pub fn tags(&self) -> Vec<OutputTags> {
        self.outputs
            .iter()
            .enumerate()
            .map(|(index, output)| OutputTags {
                name: output.name.clone(),
                focused: index == self.current,
                tags: (1..=self.config.tags)
                    .map(|tag| {
                        let on = |w: &&Window| w.output == output.name && w.tags & bit(tag) != 0;
                        let shown = output.selected & bit(tag) != 0;
                        TagState {
                            tag,
                            shown,
                            focused: shown && index == self.current,
                            occupied: self.windows.iter().any(|w| on(&w)),
                            urgent: self.windows.iter().filter(on).any(|w| w.urgent),
                        }
                    })
                    .collect(),
            })
            .collect()
    }

    /// Takes a new configuration; tags beyond the new count go.
    pub fn reconfigure(&mut self, config: Config) {
        let count = config.tags;
        let mask = u16::MAX >> (16 - count);
        self.config = config;
        let tag = self.new_tag();
        for output in &mut self.outputs {
            output.tags.resize(count, tag);
            output.selected = nonzero(output.selected & mask);
            output.previous = nonzero(output.previous & mask);
        }
        for window in &mut self.windows {
            window.tags = nonzero(window.tags & mask);
        }
    }

    /// Where each window shows, from the bottom to the top.
    pub fn arrange(&self) -> Vec<Placement> {
        let mut shown: Vec<(u8, Placement)> = (0..self.outputs.len())
            .flat_map(|index| self.arrange_output(index))
            .collect();
        // A stable sort keeps the order of the layout within a rank.
        shown.sort_by_key(|(rank, _)| *rank);
        let hidden: Vec<Placement> = self
            .windows
            .iter()
            .filter(|w| !shown.iter().any(|(_, p)| p.id == w.id))
            .map(|w| Placement {
                id: w.id.clone(),
                at: None,
                size: None,
                states: States::default(),
            })
            .collect();
        hidden
            .into_iter()
            .chain(shown.into_iter().map(|(_, p)| p))
            .collect()
    }

    /// The placements of the windows of one output, with their rank in the
    /// stacking order.
    fn arrange_output(&self, index: usize) -> Vec<(u8, Placement)> {
        let output = &self.outputs[index];
        let tag = output.tags[first_tag(output.selected)];
        let focused = self.focused();
        let top = self.last_on(index, &self.raised);
        let windows: Vec<&Window> = self
            .shown_on(index)
            .into_iter()
            .map(|i| &self.windows[i])
            .collect();
        let tiled: Vec<&Window> = windows.iter().copied().filter(|w| self.tiles(w)).collect();
        let top_at = self.raised_among(&tiled);
        let stacks = matches!(
            tag.layout,
            Layout::Max | Layout::Fullscreen | Layout::Magnifier
        );
        let cells = layout::arrange(
            tag.layout,
            tiled.len(),
            (output.area, output.full),
            Some(top_at),
            tag.params,
        );
        let gap = if tag.layout == Layout::Fullscreen {
            0
        } else {
            i32::from(self.config.gap)
        };
        let mut placed = Vec::new();
        for (i, (window, cell)) in tiled.iter().zip(cells).enumerate() {
            let [x, y, w, h] = layout::inset(cell, gap);
            let states = States {
                tiled: true,
                activated: Some(&*window.id) == focused,
                ..States::default()
            };
            let rank = if window.on_top {
                ON_TOP
            } else if Some(&*window.id) == top {
                TOP
            } else if stacks && i == top_at {
                TILED_TOP
            } else {
                TILED
            };
            placed.push((rank, placement(window, [x, y], Some([w, h]), states)));
        }
        for window in windows.iter().filter(|w| !self.tiles(w)) {
            let id = Some(&*window.id);
            placed.push(float(window, output, id == top, id == focused));
        }
        placed
    }

    fn tiles(&self, window: &Window) -> bool {
        let layout = self
            .outputs
            .iter()
            .find(|o| o.name == window.output)
            .map(|o| o.tags[first_tag(o.selected)].layout);
        layout.is_some_and(|l| l != Layout::Floating)
            && !window.floating
            && !window.fullscreen
            && window.maximized == (false, false)
    }

    /// Whether a window shows on its output.
    fn shows(&self, window: &Window) -> bool {
        !window.minimized
            && self
                .outputs
                .iter()
                .any(|o| o.name == window.output && o.selected & window.tags != 0)
    }

    /// The indices of the windows that show on an output, in their order.
    fn shown_on(&self, output: usize) -> Vec<usize> {
        let Some(name) = self.outputs.get(output).map(|o| &o.name) else {
            return Vec::new();
        };
        (0..self.windows.len())
            .filter(|&i| &self.windows[i].output == name && self.shows(&self.windows[i]))
            .collect()
    }

    /// The position of the last raised window among windows, or 0. The max,
    /// fullscreen and magnifier layouts show this tiled window.
    fn raised_among(&self, windows: &[&Window]) -> usize {
        self.raised
            .iter()
            .rev()
            .find_map(|id| windows.iter().position(|w| w.id == *id))
            .unwrap_or(0)
    }

    /// The last window of a history that shows on an output.
    fn last_on<'a>(&'a self, output: usize, history: &'a [String]) -> Option<&'a str> {
        let name = &self.outputs.get(output)?.name;
        history
            .iter()
            .rev()
            .find(|id| {
                self.window(id)
                    .is_some_and(|w| &w.output == name && self.shows(w))
            })
            .map(String::as_str)
    }

    fn focus_step(&mut self, step: isize) {
        let shown = self.shown_on(self.current);
        if shown.is_empty() {
            return;
        }
        let at = self.focused_position(&shown).unwrap_or(0);
        let id = self.windows[shown[wrap(at, step, shown.len())]].id.clone();
        self.focus(&id);
    }

    fn focus_back(&mut self) {
        let focused = self.focused().map(String::from);
        let back = self
            .history
            .iter()
            .rev()
            .filter(|id| Some(*id) != focused.as_ref())
            .find(|id| self.window(id).is_some_and(|w| self.shows(w)))
            .cloned();
        if let Some(id) = back {
            self.focus(&id);
        }
    }

    /// Shows a window that asked for attention, with its tag, and focuses
    /// it; a minimized one comes back.
    fn focus_urgent(&mut self) {
        let Some(window) = self.windows.iter_mut().find(|w| w.urgent) else {
            return;
        };
        window.minimized = false;
        let (id, name, tags) = (window.id.clone(), window.output.clone(), window.tags);
        if let Some(output) = self.outputs.iter_mut().find(|o| o.name == name)
            && output.selected & tags == 0
        {
            output.previous = output.selected;
            output.selected = 1 << first_tag(tags);
        }
        self.focus(&id);
    }

    /// Swaps the focused window with the next or previous one that shows.
    fn swap(&mut self, step: isize) {
        let shown = self.shown_on(self.current);
        if let Some(at) = self.focused_position(&shown) {
            self.windows
                .swap(shown[at], shown[wrap(at, step, shown.len())]);
        }
    }

    /// The position of the focused window among the indices of windows.
    fn focused_position(&self, shown: &[usize]) -> Option<usize> {
        let focused = self.focused()?;
        shown.iter().position(|&i| self.windows[i].id == focused)
    }

    fn focus_output(&mut self, step: isize) {
        if self.outputs.is_empty() {
            return;
        }
        self.current = wrap(self.current, step, self.outputs.len());
    }

    fn move_to_next_output(&mut self) {
        let Some(id) = self.focused().map(String::from) else {
            return;
        };
        let next = wrap(self.current, 1, self.outputs.len());
        if next == self.current {
            return;
        }
        let next = &self.outputs[next];
        let (name, tags) = (next.name.clone(), nonzero(next.selected));
        let Some(window) = self.window_mut(&id) else {
            return;
        };
        (window.output, window.tags, window.at) = (name, tags, None);
        self.focus(&id);
    }

    /// Shows the most recently minimized window of the current tags.
    fn restore_minimized(&mut self) {
        let Some(output) = self.outputs.get(self.current) else {
            return;
        };
        let (name, selected) = (output.name.clone(), output.selected);
        let restored = self
            .history
            .iter()
            .rev()
            .chain(self.windows.iter().map(|w| &w.id))
            .find(|id| {
                self.window(id)
                    .is_some_and(|w| w.minimized && w.output == name && w.tags & selected != 0)
            })
            .cloned();
        if let Some(id) = restored
            && let Some(window) = self.window_mut(&id)
        {
            window.minimized = false;
            self.focus(&id);
        }
    }

    /// Flips a state of the focused window.
    fn toggle(&mut self, state: impl FnOnce(&mut Window) -> &mut bool) {
        self.with_focused(|w| *state(w) ^= true);
    }

    fn with_focused(&mut self, change: impl FnOnce(&mut Window)) {
        if let Some(id) = self.focused().map(String::from)
            && let Some(window) = self.window_mut(&id)
        {
            change(window);
        }
    }

    /// Changes the current tag of the current output.
    fn with_tag(&mut self, change: impl FnOnce(&mut Tag, &[Layout])) {
        let layouts = &self.config.layouts;
        if let Some(output) = self.outputs.get_mut(self.current) {
            let first = first_tag(output.selected);
            change(&mut output.tags[first], layouts);
        }
    }

    /// Shows other tags on the current output, from the shown tags and their
    /// count. At least one tag stays shown, or new windows would get no tag
    /// that shows.
    fn view(&mut self, select: impl FnOnce(u16, usize) -> u16) {
        if let Some(output) = self.outputs.get_mut(self.current) {
            let selected = select(output.selected, output.tags.len());
            if selected != 0 && selected != output.selected {
                output.previous = output.selected;
                output.selected = selected;
            }
        }
    }

    fn view_back(&mut self) {
        if let Some(output) = self.outputs.get_mut(self.current) {
            (output.selected, output.previous) = (output.previous, output.selected);
        }
    }

    fn current_tag(&self) -> Option<&Tag> {
        let output = self.outputs.get(self.current)?;
        output.tags.get(first_tag(output.selected))
    }

    fn new_tag(&self) -> Tag {
        Tag {
            layout: self.config.layouts[0],
            params: Params {
                master_width: self.config.master_width,
                masters: self.config.masters,
                columns: self.config.columns,
            },
        }
    }

    /// The claim of the plugin (D55): the box around the work areas of
    /// its outputs. ponytail: one box, so the gaps between outputs of
    /// another shape count too; a claim per output if that ever matters.
    pub fn claim(&self) -> Option<[f64; 4]> {
        let areas = self.outputs.iter().map(|o| o.area);
        let [x0, y0] = [
            areas.clone().map(|a| a[0]).min()?,
            areas.clone().map(|a| a[1]).min()?,
        ];
        let x1 = areas.clone().map(|a| a[0] + a[2]).max()?;
        let y1 = areas.map(|a| a[1] + a[3]).max()?;
        Some([x0, y0, x1 - x0, y1 - y0].map(f64::from))
    }

    /// What a restart needs of the outputs, as meta of the group: the
    /// current output, and the shown tags and the tags of each output.
    pub fn group_meta(&self) -> BTreeMap<String, String> {
        let mut meta = BTreeMap::new();
        if let Some(output) = self.outputs.get(self.current) {
            meta.insert(CURRENT_KEY.into(), output.name.clone());
        }
        for o in &self.outputs {
            let tags: Vec<KeptTag> = o
                .tags
                .iter()
                .map(|t| {
                    (
                        t.layout,
                        t.params.master_width,
                        t.params.masters,
                        t.params.columns,
                    )
                })
                .collect();
            meta.insert(
                format!("{TAGS_KEY}{}", o.name),
                json(&(o.selected, o.previous, tags)),
            );
        }
        meta
    }

    /// What a restart needs of each window, as the meta value of
    /// [`WINDOW_KEY`] by window id.
    pub fn window_meta(&self) -> Vec<(String, String)> {
        let rank = |list: &[String], id: &str| list.iter().position(|h| h == id);
        self.windows
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let kept = (w, i, rank(&self.history, &w.id), rank(&self.raised, &w.id));
                (w.id.clone(), json(&kept))
            })
            .collect()
    }

    /// Rebuilds the model after a restart from the meta of the group and
    /// of the windows by id, once the outputs are known. Meta that does not
    /// parse is skipped; the configuration may have other tags now.
    pub fn restore(&mut self, group: &BTreeMap<String, String>, windows: &[(String, String)]) {
        for output in &mut self.outputs {
            let kept = group
                .get(&format!("{TAGS_KEY}{}", output.name))
                .and_then(|t| serde_json::from_str::<(u16, u16, Vec<KeptTag>)>(t).ok());
            if let Some((selected, previous, tags)) = kept {
                (output.selected, output.previous) = (selected, previous);
                output.tags = tags
                    .into_iter()
                    .map(|(layout, master_width, masters, columns)| Tag {
                        layout,
                        params: Params {
                            master_width,
                            masters,
                            columns,
                        },
                    })
                    .collect();
            }
        }
        let mut kept: Vec<KeptWindow> = windows
            .iter()
            .filter_map(|(id, text)| {
                let mut window: KeptWindow = serde_json::from_str(text).ok()?;
                window.0.id.clone_from(id);
                Some(window)
            })
            .collect();
        kept.sort_by_key(|k| k.1);
        let by_rank = |rank: fn(&KeptWindow) -> Option<usize>| {
            let mut ranked: Vec<(usize, String)> = kept
                .iter()
                .filter_map(|k| Some((rank(k)?, k.0.id.clone())))
                .collect();
            ranked.sort();
            ranked.into_iter().map(|(_, id)| id).collect()
        };
        self.history = by_rank(|k| k.2);
        self.raised = by_rank(|k| k.3);
        self.windows = kept.into_iter().map(|k| k.0).collect();
        self.current = group
            .get(CURRENT_KEY)
            .and_then(|name| self.outputs.iter().position(|o| &o.name == name))
            .unwrap_or(0);
        self.reconfigure(self.config.clone());
    }

    fn window(&self, id: &str) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    fn window_mut(&mut self, id: &str) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }
}

#[allow(clippy::expect_used, reason = "the model has string keys only")]
fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("the model always serializes")
}

/// The placement and the rank of a window that does not tile.
fn float(window: &Window, output: &Output, top: bool, activated: bool) -> (u8, Placement) {
    let rank = match (window.on_top, top) {
        (true, _) => ON_TOP,
        (false, true) => TOP,
        (false, false) => FLOATING,
    };
    if window.fullscreen {
        let [x, y, w, h] = output.full;
        let states = States {
            fullscreen: true,
            activated,
            ..States::default()
        };
        let rank = match (window.on_top, top) {
            (true, _) => ON_TOP,
            (false, true) => FULLSCREEN_TOP,
            (false, false) => FULLSCREEN,
        };
        return (rank, placement(window, [x, y], Some([w, h]), states));
    }
    let [ax, ay, aw, ah] = output.area;
    let [sw, sh] = window.size.unwrap_or([aw / 2, ah / 2]);
    let (mx, my) = window.maximized;
    let (w, h) = (if mx { aw } else { sw }, if my { ah } else { sh });
    let at = match window.at {
        Some(at) if !(mx || my) => at,
        _ => [ax + (aw - w) / 2, ay + (ah - h) / 2],
    };
    let states = States {
        maximized: mx && my,
        activated,
        ..States::default()
    };
    (
        rank,
        placement(window, at, (mx || my).then_some([w, h]), states),
    )
}

/// Adds or removes a tag of a window, but keeps its last tag.
fn toggle_tag(window: &mut Window, n: usize) {
    if window.tags != bit(n) {
        window.tags ^= bit(n);
    }
}

fn toggle_maximized(window: &mut Window) {
    let both = window.maximized != (true, true);
    window.maximized = (both, both);
}

/// The master share of a tag plus a change, within its range.
fn share(tag: &Tag, delta: f64) -> f64 {
    (tag.params.master_width + delta).clamp(*SHARE.start(), *SHARE.end())
}

/// A count plus a change, at least `least`.
fn add(count: usize, delta: isize, least: usize) -> usize {
    count.saturating_add_signed(delta).max(least)
}

/// The layout `step` places after a layout in a list.
fn cycle(layouts: &[Layout], layout: Layout, step: isize) -> Layout {
    let index = layouts.iter().position(|l| *l == layout).unwrap_or(0);
    layouts[wrap(index, step, layouts.len())]
}

fn placement(window: &Window, at: [i32; 2], size: Option<[i32; 2]>, states: States) -> Placement {
    Placement {
        id: window.id.clone(),
        at: Some(at),
        size,
        states,
    }
}

/// The bit of a tag by its number, from 1.
fn bit(n: usize) -> u16 {
    1 << (n - 1)
}

/// The index of the first tag of a set, or 0 for no tag.
fn first_tag(tags: u16) -> usize {
    if tags == 0 {
        0
    } else {
        tags.trailing_zeros() as usize
    }
}

fn nonzero(tags: u16) -> u16 {
    if tags == 0 { 1 } else { tags }
}

/// An index `step` places further, wrapping around.
fn wrap(index: usize, step: isize, len: usize) -> usize {
    let step_len = step.unsigned_abs() % len;
    if step < 0 {
        (index + len - step_len) % len
    } else {
        (index + step_len) % len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT;

    #[test]
    fn wrap_goes_around_both_ways() {
        assert_eq!(wrap(2, 1, 3), 0);
        assert_eq!(wrap(0, -1, 3), 2);
        assert_eq!(wrap(1, -4, 3), 0);
        assert_eq!(wrap(1, 5, 3), 0);
    }

    fn wm(windows: &[&str]) -> Wm {
        let mut wm = Wm::new(chora_kit::settings::parse::<Config>(DEFAULT).unwrap());
        wm.config.gap = 0;
        wm.set_output("A", [0, 0, 100, 60], [0, 0, 100, 80]);
        for id in windows {
            wm.add_window(id);
        }
        wm
    }

    fn shown(wm: &Wm) -> Vec<(String, Option<[i32; 2]>)> {
        wm.arrange().into_iter().map(|p| (p.id, p.at)).collect()
    }

    #[test]
    fn a_new_window_becomes_the_focused_master() {
        let wm = wm(&["a", "b"]);
        assert_eq!(wm.shown_windows(), ["b", "a"]);
        assert_eq!(wm.focused(), Some("b"));
        // The focused master lies on top.
        let master = wm.arrange().pop().unwrap();
        assert_eq!((master.id.as_str(), master.at), ("b", Some([0, 0])));
        assert_eq!(master.size, Some([50, 60]));
        assert!(master.states.tiled && master.states.activated);
    }

    #[test]
    fn tags_hide_and_show_windows() {
        let mut wm = wm(&["a"]);
        wm.act(&Action::MoveToTag(2));
        assert_eq!(shown(&wm), [("a".into(), None)]);
        assert_eq!(wm.focused(), None);
        wm.act(&Action::ToggleTagView(2));
        assert_eq!(wm.focused(), Some("a"));
        // The last shown tag stays shown.
        wm.act(&Action::ToggleTagView(1));
        wm.act(&Action::ToggleTagView(2));
        assert_eq!(wm.shown_windows(), ["a"]);
        wm.act(&Action::ToggleTagView(1));
        wm.act(&Action::ViewTag(3));
        wm.act(&Action::ViewBack);
        assert_eq!(wm.shown_windows(), ["a"]);
        // From tags 1 and 2, the next tag is 2, then 3.
        wm.act(&Action::ViewNext);
        assert_eq!(wm.shown_windows(), ["a"]);
        wm.act(&Action::ViewNext);
        assert_eq!(wm.shown_windows(), Vec::<String>::new());
    }

    #[test]
    fn focus_moves_back_and_swaps() {
        let mut wm = wm(&["a", "b", "c"]);
        wm.act(&Action::FocusNext);
        assert_eq!(wm.focused(), Some("b"));
        wm.act(&Action::FocusBack);
        assert_eq!(wm.focused(), Some("c"));
        wm.act(&Action::SwapNext);
        assert_eq!(wm.shown_windows(), ["b", "c", "a"]);
        // As in awesome, the focused window swaps with the master.
        wm.act(&Action::FocusNext);
        wm.act(&Action::MoveToMaster);
        assert_eq!(wm.shown_windows(), ["a", "c", "b"]);
    }

    #[test]
    fn view_tag_shows_only_that_tag_on_the_current_output() {
        let mut wm = wm(&["a"]);
        wm.act(&Action::MoveToTag(2));
        assert_eq!(wm.shown_windows(), Vec::<String>::new());
        wm.view_tag(2);
        assert_eq!(wm.shown_windows(), ["a"]);
        wm.view_tag(3);
        assert_eq!(wm.shown_windows(), Vec::<String>::new());
    }

    #[test]
    fn move_to_tag_puts_a_window_on_one_tag_only() {
        let mut wm = wm(&["a", "b"]);
        assert!(wm.move_to_tag("a", 2));
        assert!(!wm.move_to_tag("x", 2));
        assert_eq!(wm.shown_windows(), ["b"], "a leaves the shown tag");
        wm.view_tag(2);
        assert_eq!(wm.shown_windows(), ["a"]);
    }

    #[test]
    fn tags_tell_shown_focused_occupied_and_urgent() {
        let mut wm = wm(&["a", "b"]);
        wm.set_output("B", [100, 0, 100, 60], [100, 0, 100, 80]);
        assert!(wm.move_to_tag("a", 2));
        wm.act(&Action::Minimize); // b goes away, but stays on tag 1
        wm.focus("b"); // b shows nowhere, so it gets the urgent mark
        let tags = wm.tags();
        let marks = |o: &OutputTags| -> Vec<_> {
            o.tags
                .iter()
                .map(|t| (t.tag, t.shown, t.focused, t.occupied, t.urgent))
                .collect()
        };
        assert_eq!(tags.len(), 2);
        assert_eq!((tags[0].name.as_str(), tags[0].focused), ("A", true));
        assert_eq!(
            &marks(&tags[0])[..3],
            [
                (1, true, true, true, true),
                (2, false, false, true, false),
                (3, false, false, false, false),
            ]
        );
        assert_eq!(marks(&tags[0]).len(), wm.config.tags);
        assert_eq!((tags[1].name.as_str(), tags[1].focused), ("B", false));
        assert_eq!(marks(&tags[1])[0], (1, true, false, false, false));
    }

    #[test]
    fn focus_focuses_and_raises_a_window() {
        let mut wm = wm(&["a", "f"]);
        wm.act(&Action::ToggleFloating);
        wm.focus("a");
        assert_eq!(wm.focused(), Some("a"));
        assert_eq!(wm.arrange().pop().unwrap().id, "a", "focus raises");
    }

    #[test]
    fn focusing_a_window_that_shows_nowhere_marks_it_urgent() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::Minimize); // b, the focused window, goes away
        assert_eq!(wm.focused(), Some("a"));
        wm.focus("b");
        assert_eq!(wm.focused(), Some("a"), "a hidden window gains no focus");
        assert_eq!(wm.arrange().last().unwrap().id, "a", "and no raise");
        assert!(wm.windows.iter().any(|w| w.id == "b" && w.urgent));
    }

    #[test]
    fn focus_next_and_prev_step_through_the_shown_windows() {
        let mut wm = wm(&["a", "b", "c"]);
        wm.focus_next();
        assert_eq!(wm.focused(), Some("b"));
        wm.focus_prev();
        assert_eq!(wm.focused(), Some("c"));
        wm.focus_prev();
        assert_eq!(wm.focused(), Some("a"), "stepping wraps around");
    }

    #[test]
    fn swap_master_swaps_a_window_with_the_master_of_its_output() {
        let mut wm = wm(&["a", "b", "c"]);
        assert!(!wm.swap_master("x"));
        assert!(wm.swap_master("b"));
        assert_eq!(wm.shown_windows(), ["b", "c", "a"]);
        wm.act(&Action::MoveToTag(2));
        assert!(!wm.swap_master("c"), "c shows nowhere");
    }

    #[test]
    fn states_take_a_window_out_of_the_tiles() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::ToggleFullscreen);
        let top = wm.arrange().pop().unwrap();
        assert_eq!(
            (top.id.as_str(), top.at, top.size),
            ("b", Some([0, 0]), Some([100, 80]))
        );
        assert!(top.states.fullscreen);
        assert_eq!(wm.arrange()[0].size, Some([100, 60]), "a tiles alone");
        wm.act(&Action::ToggleFullscreen);
        wm.act(&Action::Minimize);
        assert_eq!(wm.focused(), Some("a"));
        wm.act(&Action::RestoreMinimized);
        assert_eq!(wm.focused(), Some("b"));
    }

    #[test]
    fn layouts_and_parameters_belong_to_the_tag() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::NextLayout);
        wm.act(&Action::IncMasterWidth(0.1));
        assert_eq!(wm.current().unwrap().0, Layout::TileLeft);
        wm.act(&Action::ViewTag(2));
        assert_eq!(wm.current().unwrap().0, Layout::Tile);
        wm.act(&Action::PreviousLayout);
        assert_eq!(wm.current().unwrap().0, Layout::Floating);
        wm.act(&Action::ViewTag(1));
        let (layout, params) = wm.current().unwrap();
        assert_eq!(layout, Layout::TileLeft);
        assert!((params.master_width - 0.6).abs() < 1e-9);
    }

    #[test]
    fn urgent_windows_and_other_outputs() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::MoveToTag(2));
        wm.focus("b");
        assert_eq!(wm.focused(), Some("a"));
        wm.act(&Action::FocusUrgent);
        assert_eq!(wm.focused(), Some("b"));
        wm.set_output("B", [100, 0, 100, 60], [100, 0, 100, 60]);
        wm.act(&Action::MoveToNextOutput);
        assert_eq!(wm.focused(), Some("b"));
        assert_eq!(wm.arrange().last().unwrap().at, Some([100, 0]));
        wm.remove_output("B");
        assert_eq!(wm.windows.iter().filter(|w| w.output == "A").count(), 2);
    }

    #[test]
    fn floating_windows_center_at_their_size() {
        let mut wm = wm(&["a"]);
        wm.act(&Action::ToggleFloating);
        assert!(wm.set_size("a", [40, 20]));
        let placed = wm.arrange().pop().unwrap();
        assert_eq!((placed.at, placed.size), (Some([30, 20]), None));
    }

    #[test]
    fn a_drag_places_only_a_window_that_does_not_tile() {
        let mut wm = wm(&["a"]);
        assert!(!wm.place("a", [5, 6]), "a tiles");
        wm.act(&Action::ToggleFloating);
        assert!(wm.place("a", [5, 6]));
        assert!(!wm.place("a", [5, 6]), "no change");
        assert_eq!(wm.arrange().pop().unwrap().at, Some([5, 6]));
    }

    #[test]
    fn a_fullscreen_window_covers_others_only_with_the_focus() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::ToggleFullscreen);
        wm.act(&Action::FocusNext);
        assert_eq!(wm.focused(), Some("a"));
        assert_eq!(wm.arrange().pop().unwrap().id, "a");
    }

    #[test]
    fn windows_survive_a_change_of_all_outputs() {
        let mut wm = wm(&["a"]);
        wm.remove_output("A");
        wm.add_window("b");
        wm.set_output("B", [0, 0, 100, 60], [0, 0, 100, 60]);
        assert_eq!(wm.shown_windows(), ["b", "a"]);
    }

    #[test]
    fn the_current_output_stays_after_another_goes() {
        let mut wm = wm(&[]);
        wm.set_output("B", [100, 0, 100, 60], [100, 0, 100, 60]);
        wm.set_output("C", [200, 0, 100, 60], [200, 0, 100, 60]);
        wm.act(&Action::FocusNextOutput);
        wm.add_window("b");
        wm.remove_output("A");
        assert_eq!(wm.focused(), Some("b"));
    }

    #[test]
    fn the_shown_tag_again_keeps_the_history() {
        let mut wm = wm(&["a"]);
        wm.act(&Action::ViewTag(2));
        wm.act(&Action::ViewTag(1));
        wm.act(&Action::ViewTag(1));
        wm.act(&Action::ViewBack);
        assert_eq!(wm.shown_windows(), Vec::<String>::new());
        wm.act(&Action::ViewBack);
        wm.act(&Action::ToggleWindowTag(1));
        assert_eq!(wm.windows[0].tags, 1, "the last tag stays");
    }

    #[test]
    fn an_urgent_minimized_window_comes_back() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::Minimize);
        wm.request("b", SurfaceRequest::Activate);
        wm.act(&Action::FocusUrgent);
        assert_eq!(wm.focused(), Some("b"));
        assert_eq!(wm.shown_windows(), ["b", "a"]);
    }

    #[test]
    fn the_focused_window_covers_others_and_on_top_covers_all() {
        let mut wm = wm(&["a", "b"]);
        wm.act(&Action::ToggleMaximized);
        wm.act(&Action::FocusNext);
        assert_eq!(
            wm.arrange().pop().unwrap().id,
            "a",
            "tiled a covers maximized b"
        );
        wm.act(&Action::ToggleOnTop);
        wm.act(&Action::FocusNext);
        wm.act(&Action::ToggleFullscreen);
        assert_eq!(
            wm.arrange().pop().unwrap().id,
            "a",
            "on top covers fullscreen"
        );
    }

    #[test]
    fn one_output_keeps_the_tags_of_a_moved_window() {
        let mut wm = wm(&["a"]);
        wm.act(&Action::ToggleWindowTag(2));
        wm.act(&Action::MoveToNextOutput);
        assert_eq!(wm.windows[0].tags, 0b11);
    }

    #[test]
    fn the_focus_that_follows_the_pointer_raises_nothing() {
        let mut wm = wm(&["a", "f"]);
        wm.act(&Action::ToggleFloating);
        wm.pointer_focus("a", false, true, false);
        assert_eq!(wm.focused(), Some("a"));
        assert_eq!(wm.arrange().pop().unwrap().id, "f");
        wm.pointer_focus("a", true, true, false);
        assert_eq!(wm.arrange().pop().unwrap().id, "a", "a click raises");
    }

    #[test]
    fn the_magnifier_shows_its_window_behind_a_floating_one() {
        let mut wm = wm(&["a", "b", "c"]);
        wm.set_layout(Layout::Magnifier);
        wm.act(&Action::ToggleFloating);
        let ids: Vec<String> = wm.arrange().into_iter().map(|p| p.id).collect();
        assert_eq!(
            ids,
            ["a", "b", "c"],
            "b, the first tiled window, lies over a"
        );
    }

    #[test]
    fn max_shows_the_last_raised_tiled_window() {
        let mut wm = wm(&["a", "b", "c"]);
        wm.set_layout(Layout::Max);
        wm.act(&Action::FocusNext);
        assert_eq!(wm.focused(), Some("b"));
        wm.add_window("f");
        wm.act(&Action::ToggleFloating);
        let ids: Vec<String> = wm.arrange().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["c", "a", "b", "f"], "b stays behind the floating f");
    }

    #[test]
    fn a_fullscreen_window_rises_with_a_click_only() {
        let mut wm = wm(&["a", "b", "o"]);
        wm.act(&Action::ToggleOnTop);
        wm.act(&Action::FocusNext);
        wm.act(&Action::ToggleFullscreen);
        wm.pointer_focus("o", false, true, false);
        let ids: Vec<String> = wm.arrange().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, ["a", "b", "o"], "the fullscreen b stays over a");
        wm.pointer_focus("a", true, true, false);
        wm.pointer_focus("b", false, true, false);
        assert_eq!(wm.arrange()[1].id, "a", "the pointer does not raise b");
        wm.pointer_focus("b", true, true, false);
        assert_eq!(wm.arrange()[1].id, "b", "a click does");
    }

    #[test]
    fn a_restart_rebuilds_the_model_from_meta() {
        let mut old = wm(&["a", "b", "c"]);
        old.act(&Action::MoveToTag(2));
        old.act(&Action::ViewTag(2));
        old.act(&Action::NextLayout);
        old.act(&Action::ToggleFloating);
        old.act(&Action::ViewTag(1));
        let mut new = wm(&[]);
        new.restore(&old.group_meta(), &old.window_meta());
        assert_eq!(new.arrange(), old.arrange());
        assert_eq!(new.focused(), old.focused());
        new.act(&Action::ViewTag(2));
        old.act(&Action::ViewTag(2));
        assert_eq!(new.current(), old.current());
        assert_eq!(new.arrange(), old.arrange());
    }

    #[test]
    fn the_claim_holds_the_work_areas_of_all_outputs() {
        let mut wm = wm(&[]);
        assert_eq!(wm.claim(), Some([0.0, 0.0, 100.0, 60.0]));
        wm.set_output("B", [100, 10, 50, 40], [100, 0, 50, 50]);
        assert_eq!(wm.claim(), Some([0.0, 0.0, 150.0, 60.0]));
    }
}
