//! The state of the window manager: outputs with tags, windows with their
//! tags and states, and the focus history. It knows nothing of IPC; the
//! plugin turns its placements into commands.

use chora_scene::SurfaceRequest;

use crate::{
    config::{Action, Config, Focus, NewWindow},
    layout::{self, Layout, Params, Rect},
};

/// A tiled or floating window.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)] // independent states, as in awesome
pub struct Window {
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
    Spawn(String),
    Close(String),
    Reload,
}

pub struct Wm {
    pub config: Config,
    pub windows: Vec<Window>,
    pub outputs: Vec<Output>,
    /// The index of the output with the focus.
    current: usize,
    /// Window ids, the most recent focus last.
    history: Vec<String>,
}

impl Wm {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            windows: Vec::new(),
            outputs: Vec::new(),
            current: 0,
            history: Vec::new(),
        }
    }

    /// Adds an output, or changes its areas.
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
            tags: vec![tag; self.config.tags.len()],
        });
    }

    /// Removes an output; its windows go to the first other output.
    pub fn remove_output(&mut self, name: &str) {
        self.outputs.retain(|o| o.name != name);
        self.current = self.current.min(self.outputs.len().saturating_sub(1));
        if let Some(first) = self.outputs.first() {
            let (target, tags) = (first.name.clone(), first.selected.max(1));
            for window in self.windows.iter_mut().filter(|w| w.output == name) {
                window.output.clone_from(&target);
                window.tags = tags;
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
    }

    pub fn has(&self, id: &str) -> bool {
        self.windows.iter().any(|w| w.id == id)
    }

    /// Notes the size that a client chose. True when a floating window must
    /// move to keep its place.
    pub fn set_size(&mut self, id: &str, size: [i32; 2]) -> bool {
        let floats = self.floats(id);
        let Some(window) = self.window_mut(id) else {
            return false;
        };
        let changed = window.size != Some(size);
        window.size = Some(size);
        changed && floats
    }

    /// The focused window: the most recent one that shows on the current
    /// output.
    pub fn focused(&self) -> Option<&str> {
        let output = &self.outputs.get(self.current)?.name;
        self.history
            .iter()
            .rev()
            .find(|id| {
                self.window(id)
                    .is_some_and(|w| &w.output == output && self.shows(w))
            })
            .map(String::as_str)
    }

    /// Focuses a window and its output.
    pub fn focus(&mut self, id: &str) {
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

    /// Focuses a window under the pointer, as the configuration says.
    pub fn pointer_focus(&mut self, id: &str, clicked: bool) -> bool {
        let follow = self.config.focus == Focus::FollowMouse
            && self
                .current_tag()
                .is_some_and(|t| t.layout != Layout::Magnifier);
        let shown = self.window(id).is_some_and(|w| self.shows(w));
        if !shown || !(clicked || follow) || self.focused() == Some(id) {
            return false;
        }
        self.focus(id);
        true
    }

    /// A window asked for attention: it gets the focus when it shows.
    pub fn activate(&mut self, id: &str) {
        let shows = self.window(id).is_some_and(|w| self.shows(w));
        if shows {
            self.focus(id);
        } else if let Some(window) = self.window_mut(id) {
            window.urgent = true;
        }
    }

    /// A request of a client, for example from its title bar.
    pub fn request(&mut self, id: &str, request: SurfaceRequest) {
        if request == SurfaceRequest::Activate {
            self.activate(id);
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
            _ => {}
        }
    }

    /// Runs an action of a binding.
    pub fn act(&mut self, action: &Action) -> Option<Effect> {
        match action {
            Action::Spawn(command) => return Some(Effect::Spawn(command.clone())),
            Action::Close => return self.focused().map(|id| Effect::Close(id.into())),
            Action::Reload => return Some(Effect::Reload),
            Action::FocusNext => self.focus_step(1),
            Action::FocusPrevious => self.focus_step(-1),
            Action::FocusBack => self.focus_back(),
            Action::FocusUrgent => self.focus_urgent(),
            Action::SwapNext => self.swap(1),
            Action::SwapPrevious => self.swap(-1),
            Action::MoveToMaster => self.move_to_master(),
            Action::FocusNextOutput => self.focus_output(1),
            Action::FocusPreviousOutput => self.focus_output(-1),
            Action::MoveToNextOutput => self.move_to_next_output(),
            _ => {
                self.window_action(action);
                self.tag_action(action);
            }
        }
        Some(Effect::Relayout)
    }

    /// The actions on the states of the focused window.
    fn window_action(&mut self, action: &Action) {
        let restore = matches!(action, Action::RestoreMinimized);
        if restore {
            self.restore_minimized();
        }
        let Some(id) = self.focused().map(String::from).filter(|_| !restore) else {
            return;
        };
        let window = self.window_mut(&id).expect("the focused window exists");
        match action {
            Action::ToggleFloating => window.floating ^= true,
            Action::ToggleFullscreen => window.fullscreen ^= true,
            Action::ToggleMaximized => {
                let both = !(window.maximized.0 && window.maximized.1);
                window.maximized = (both, both);
            }
            Action::ToggleMaximizedHorizontal => window.maximized.0 ^= true,
            Action::ToggleMaximizedVertical => window.maximized.1 ^= true,
            Action::ToggleOnTop => window.on_top ^= true,
            Action::Minimize => window.minimized = true,
            Action::MoveToTag(n) => window.tags = bit(*n),
            Action::ToggleWindowTag(n) if window.tags ^ bit(*n) != 0 => window.tags ^= bit(*n),
            _ => {}
        }
    }

    /// The actions on the tags and the layout of the current output.
    fn tag_action(&mut self, action: &Action) {
        let layouts = self.config.layouts.clone();
        let Some(output) = self.outputs.get_mut(self.current) else {
            return;
        };
        let count = output.tags.len();
        let first = first_tag(output.selected);
        let view = match action {
            Action::ViewTag(n) => bit(*n),
            Action::ToggleTagView(n) => output.selected ^ bit(*n),
            Action::ViewNext => 1 << wrap(first, 1, count),
            Action::ViewPrevious => 1 << wrap(first, -1, count),
            Action::ViewBack => output.previous,
            _ => {
                change_tag(&mut output.tags[first], action, &layouts);
                return;
            }
        };
        output.previous = output.selected;
        output.selected = view;
    }

    /// Changes the layout of the current tag, from the exported method.
    pub fn set_layout(&mut self, layout: Layout) {
        if let Some(output) = self.outputs.get_mut(self.current) {
            let first = first_tag(output.selected);
            output.tags[first].layout = layout;
        }
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

    /// Takes a new configuration; tags beyond the new count go.
    pub fn reconfigure(&mut self, config: Config) {
        let count = config.tags.len();
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
        let focused = self.focused();
        let mut hidden = Vec::new();
        let mut shown = Vec::new();
        for (index, output) in self.outputs.iter().enumerate() {
            shown.extend(self.arrange_output(index, output, focused));
        }
        for window in &self.windows {
            if !shown
                .iter()
                .any(|(_, p): &(u8, Placement)| p.id == window.id)
            {
                hidden.push(Placement {
                    id: window.id.clone(),
                    at: None,
                    size: None,
                    states: States::default(),
                });
            }
        }
        // A stable sort keeps the order of the layout within a rank.
        shown.sort_by_key(|(rank, _)| *rank);
        hidden
            .into_iter()
            .chain(shown.into_iter().map(|(_, p)| p))
            .collect()
    }

    /// The placements of the windows of one output, with their rank in the
    /// stacking order.
    fn arrange_output(
        &self,
        index: usize,
        output: &Output,
        focused: Option<&str>,
    ) -> Vec<(u8, Placement)> {
        let tag = output.tags[first_tag(output.selected)];
        let windows: Vec<&Window> = self
            .shown_on(index)
            .into_iter()
            .map(|i| &self.windows[i])
            .collect();
        let tiled: Vec<&Window> = windows
            .iter()
            .copied()
            .filter(|w| tiles(w, tag.layout))
            .collect();
        let focus_at = tiled.iter().position(|w| Some(&*w.id) == focused);
        let cells = layout::arrange(
            tag.layout,
            tiled.len(),
            (output.area, output.full),
            focus_at,
            tag.params,
        );
        let gap = if tag.layout == Layout::Fullscreen {
            0
        } else {
            i32::from(self.config.gap)
        };
        let stacks = matches!(
            tag.layout,
            Layout::Max | Layout::Fullscreen | Layout::Magnifier
        );
        let mut placed = Vec::new();
        for (window, cell) in tiled.iter().zip(cells) {
            let [x, y, w, h] = layout::inset(cell, gap);
            let activated = Some(&*window.id) == focused;
            let states = States {
                tiled: true,
                activated,
                ..States::default()
            };
            let rank = u8::from(stacks && activated);
            placed.push((rank, placement(window, [x, y], Some([w, h]), states)));
        }
        for window in windows.iter().filter(|w| !tiles(w, tag.layout)) {
            placed.push(float(window, output, focused));
        }
        placed
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

    fn focus_step(&mut self, step: isize) {
        let shown = self.shown_on(self.current);
        if shown.is_empty() {
            return;
        }
        let at = self
            .focused()
            .and_then(|f| shown.iter().position(|&i| self.windows[i].id == f))
            .unwrap_or(0);
        let next = shown[wrap(at, step, shown.len())];
        let id = self.windows[next].id.clone();
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

    fn focus_urgent(&mut self) {
        let Some(window) = self.windows.iter().find(|w| w.urgent && !w.minimized) else {
            return;
        };
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
        let Some(at) = self
            .focused()
            .and_then(|f| shown.iter().position(|&i| self.windows[i].id == f))
        else {
            return;
        };
        self.windows
            .swap(shown[at], shown[wrap(at, step, shown.len())]);
    }

    fn move_to_master(&mut self) {
        let shown = self.shown_on(self.current);
        let Some(id) = self.focused().map(String::from) else {
            return;
        };
        let from = self
            .windows
            .iter()
            .position(|w| w.id == id)
            .expect("focused");
        let window = self.windows.remove(from);
        self.windows.insert(shown[0].min(from), window);
    }

    fn focus_output(&mut self, step: isize) {
        if self.outputs.is_empty() {
            return;
        }
        self.current = wrap(self.current, step, self.outputs.len());
        if let Some(id) = self.focused().map(String::from) {
            self.focus(&id);
        }
    }

    fn move_to_next_output(&mut self) {
        let Some(id) = self.focused().map(String::from) else {
            return;
        };
        let next = wrap(self.current, 1, self.outputs.len());
        let (name, tags) = (self.outputs[next].name.clone(), self.outputs[next].selected);
        let window = self.window_mut(&id).expect("focused");
        window.output = name;
        window.tags = nonzero(tags);
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
        if let Some(id) = restored {
            self.window_mut(&id).expect("found").minimized = false;
            self.focus(&id);
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

    fn floats(&self, id: &str) -> bool {
        self.window(id).is_some_and(|w| {
            let layout = self
                .outputs
                .iter()
                .find(|o| o.name == w.output)
                .map(|o| o.tags[first_tag(o.selected)].layout);
            !layout.is_some_and(|l| tiles(w, l))
        })
    }

    fn window(&self, id: &str) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    fn window_mut(&mut self, id: &str) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }
}

/// The placement of a window that does not tile.
fn float(window: &Window, output: &Output, focused: Option<&str>) -> (u8, Placement) {
    let activated = Some(&*window.id) == focused;
    if window.fullscreen {
        let [x, y, w, h] = output.full;
        let states = States {
            fullscreen: true,
            activated,
            ..States::default()
        };
        return (5, placement(window, [x, y], Some([w, h]), states));
    }
    let [ax, ay, aw, ah] = output.area;
    let [sw, sh] = window.size.unwrap_or([aw / 2, ah / 2]);
    let (mx, my) = window.maximized;
    let (w, h) = (if mx { aw } else { sw }, if my { ah } else { sh });
    let at = [ax + (aw - w) / 2, ay + (ah - h) / 2];
    let size = (mx || my).then_some([w, h]);
    let states = States {
        maximized: mx && my,
        activated,
        ..States::default()
    };
    let rank = 2 + u8::from(activated) + 2 * u8::from(window.on_top);
    (rank, placement(window, at, size, states))
}

fn tiles(window: &Window, layout: Layout) -> bool {
    layout != Layout::Floating
        && !window.floating
        && !window.fullscreen
        && window.maximized == (false, false)
}

/// Changes the layout or a parameter of a tag.
fn change_tag(tag: &mut Tag, action: &Action, layouts: &[Layout]) {
    let index = layouts.iter().position(|l| *l == tag.layout).unwrap_or(0);
    let delta = |d: &i32| isize::try_from(*d).unwrap_or_default();
    match action {
        Action::MasterWidth(d) => {
            tag.params.master_width = (tag.params.master_width + d).clamp(0.05, 0.95);
        }
        Action::Masters(d) => {
            tag.params.masters = tag.params.masters.saturating_add_signed(delta(d));
        }
        Action::Columns(d) => {
            tag.params.columns = tag.params.columns.saturating_add_signed(delta(d)).max(1);
        }
        Action::NextLayout => tag.layout = layouts[wrap(index, 1, layouts.len())],
        Action::PreviousLayout => tag.layout = layouts[wrap(index, -1, layouts.len())],
        _ => {}
    }
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
    let len = isize::try_from(len).expect("a few windows");
    let index = isize::try_from(index).expect("a few windows");
    usize::try_from((index + step).rem_euclid(len)).expect("rem_euclid is not negative")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT;

    fn wm(windows: &[&str]) -> Wm {
        let mut wm = Wm::new(Config::parse(DEFAULT).unwrap());
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
        let placed = wm.arrange();
        assert_eq!(placed[0].at, Some([0, 0]));
        assert_eq!(placed[0].size, Some([50, 60]));
        assert!(placed[0].states.tiled && placed[0].states.activated);
    }

    #[test]
    fn tags_hide_and_show_windows() {
        let mut wm = wm(&["a"]);
        wm.act(&Action::MoveToTag(2));
        assert_eq!(shown(&wm), [("a".into(), None)]);
        assert_eq!(wm.focused(), None);
        wm.act(&Action::ToggleTagView(2));
        assert_eq!(wm.focused(), Some("a"));
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
        wm.act(&Action::MoveToMaster);
        assert_eq!(wm.shown_windows(), ["c", "b", "a"]);
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
        wm.act(&Action::MasterWidth(0.1));
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
        wm.activate("b");
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
}
