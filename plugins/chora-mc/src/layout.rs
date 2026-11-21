use super::{
    BAR_SPACE, ClientError, Command, Filter, Geometry, HEAD, HashMap, Ink, Layout, MAX_HIT_PANELS,
    Mc, NAME, NodeKind, Panel, PropsUpdate, Query, Scene, ShaderEffect, Shaders, Type, Uniform,
    Value, chapter_id, create, manage, move_to, page_id, panel_id, panels_id, region, scene,
    title_of, translate, wheel_id,
};

impl Mc {
    /// Builds the pages of all scenes anew, after a side took or gave back
    /// its room, so that the text takes the room that is free.
    pub(crate) fn rebuild_pages(&mut self) -> Result<(), ClientError> {
        let names: Vec<String> = self.scenes.keys().cloned().collect();
        let mut commands = Vec::new();
        for name in &names {
            commands.push(Command::Remove {
                id: page_id(name),
                to: None,
            });
            commands.extend(self.panels(name));
        }
        self.apply(commands)?;
        if let Some(name) = self.current.clone() {
            self.align(&name)?;
        }
        self.take_free_keyboard()
    }

    /// Where the parts of the page of a scene go on the output.
    pub(crate) fn geometry(&self, name: &str) -> Option<Geometry> {
        self.geometry_of(name, self.scenes.get(name)?)
    }

    /// Where the parts of a page go on the output, with the sides of the
    /// scene `name` beside it. The text stays centered, with room for the
    /// sides, so that it never moves when one shows: centered where it
    /// fits, else beside the sides. The margin column of the right keeps
    /// one gutter at the edge and one beside the text, whatever its width.
    pub(crate) fn geometry_of(&self, name: &str, scene: &Scene) -> Option<Geometry> {
        let output = self.output.as_ref()?;
        let [w, h] = output.size.map(f64::from);
        let config = &self.settings.value;
        let (em, b) = (config.scale.base, config.scale.baseline);
        let page = &config.page;
        let (note, gap) = (page.margin * em, (page.gap * em).floor());
        let cols = f64::from(scene.grid()[0]);
        let want = cols * page.measure * em + (cols - 1.0) * (note + 2.0 * gap);
        let [left, right] = self.side_widths(name);
        let need = |side: f64| if side > 0.0 { side + gap } else { 0.0 };
        let lneed = note + gap + need(left);
        let (x, width) = if right > 0.0 {
            let x = (2.0 * b + lneed).round();
            (x, want.min(w - x - right - 4.0 * b).max(b))
        } else {
            let rneed = note + gap;
            let width = want.min(w - lneed - rneed - 4.0 * b).max(b);
            let centered = ((w - width) / 2.0).round();
            let x = centered
                .min(w - 2.0 * b - rneed - width)
                .max(2.0 * b + lneed)
                .round();
            (x, width)
        };
        let top = if Self::has_chapter(scene) {
            HEAD * b + self.chapter_height(scene)
        } else {
            HEAD * b
        };
        let bottom = h - 4.0 * b;
        Some(Geometry {
            text: [x, top, width, (bottom - top).max(b)],
            note,
            gap,
            baseline: b,
        })
    }

    /// The head of a scene: its number, its title and the running head,
    /// as `chapter.layout` sets.
    /// The bar and the title follow the number; `align` moves
    /// them there once the core knows its width.
    pub(crate) fn chapter(&self, name: &str, scene: &Scene) -> Vec<Command> {
        let Some(geo) = self.geometry(name).filter(|_| Self::has_chapter(scene)) else {
            return Vec::new();
        };
        let [x, _, _, _] = geo.text;
        let chapter = &scene.chapter;
        let number = self.number(name);
        let title = chapter.title.clone().unwrap_or_else(|| title_of(name));
        let mut commands = Vec::new();
        match self.layout(scene) {
            Layout::Number => {
                let (number_ty, title_ty) =
                    (self.ty("chapter_number", None), self.ty("chapter", None));
                let chars: f64 = number.chars().map(|_| 1.0).sum();
                let guess = x + 0.6 * number_ty.size * chars;
                commands.push(region::rect(
                    &chapter_id(name, "bar"),
                    &panels_id(name),
                    self.bar(guess),
                    number_ty.color,
                ));
                let parts = [
                    ("number", number_ty, number, x),
                    ("title", title_ty, title, guess),
                ];
                for (part, ty, text, x) in parts {
                    let props = PropsUpdate {
                        transform: Some(translate(x, self.head_y())),
                        ..ty.props(&text)
                    };
                    commands.push(create(
                        &chapter_id(name, part),
                        NodeKind::Text,
                        &panels_id(name),
                        props,
                    ));
                }
                commands.extend(self.head_hit(name));
            }
            Layout::None => {}
        }
        commands
    }

    /// The pointer hit of the head of a page, which opens the page list;
    /// the overview itself has none.
    pub(crate) fn head_hit(&self, name: &str) -> Vec<Command> {
        if name == manage::OVERVIEW {
            return Vec::new();
        }
        let ink = (self.settings.value.colors().get("text").copied()).unwrap_or([0.0; 4]);
        region::hit_nodes(&chapter_id(name, "head"), &panels_id(name), ink).to_vec()
    }

    /// Moves the bar and the title of the head after the number, which is
    /// `width` wide, and sizes the hit of the head to the three. The bar
    /// stands in the middle between the ink of the number and that of
    /// the title.
    pub(crate) fn align_head(
        &self,
        name: &str,
        scene: &Scene,
        geo: &Geometry,
        rect: &dyn Fn(&str) -> Option<[f64; 4]>,
    ) -> Vec<Command> {
        let Some([_, _, width, _]) = rect(&chapter_id(name, "number")) else {
            return Vec::new();
        };
        if !Self::has_chapter(scene) || self.layout(scene) != Layout::Number {
            return Vec::new();
        }
        let title = rect(&chapter_id(name, "title"));
        let space = BAR_SPACE * self.ty("chapter", None).size;
        let words = scene
            .chapter
            .title
            .clone()
            .unwrap_or_else(|| title_of(name));
        let after = self.bearings("chapter_number", &self.number(name))[1];
        let before = self.bearings("chapter", &words)[0];
        let bar = self.bar((geo.text[0] + width - after + space).round());
        let x = bar[0] + bar[2] + space.round() - before;
        let mut commands = vec![
            move_to(chapter_id(name, "bar"), bar[0], bar[1]),
            move_to(chapter_id(name, "title"), x, self.head_y()),
        ];
        commands.extend(self.head_hit_place(name, geo, x, title));
        commands
    }

    /// Sizes the hit of the head to the number, the bar and the title,
    /// whose title now stands at `x` and measures `title`.
    pub(crate) fn head_hit_place(
        &self,
        name: &str,
        geo: &Geometry,
        x: f64,
        title: Option<[f64; 4]>,
    ) -> Vec<Command> {
        let Some([_, _, width, height]) = title.filter(|_| name != manage::OVERVIEW) else {
            return Vec::new();
        };
        let left = geo.text[0];
        let at = [left, self.head_y(), x + width - left, height];
        region::hit_place(
            &chapter_id(name, "head"),
            at,
            left,
            self.ty("chapter", None).ink(),
        )
        .to_vec()
    }

    /// The box of a panel on the screen, without its margin.
    /// As many columns as fit at the least width of a panel; the panels
    /// wrap into new rows, which keep at least the least height. An empty
    /// panel takes no room, unless it says why it is empty: then it takes
    /// one short row for its words, so that every such message shows on a
    /// small output.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a few columns"
    )]
    pub(crate) fn place(&self, name: &str, panel: &Panel) -> Option<[f64; 4]> {
        let scene = self.scenes.get(name)?;
        let geo = self.geometry(name)?;
        let config = &self.settings.value;
        let [min_w, min_h] = config.min_panel.map(|m| m * config.scale.base);
        let gap = [geo.columns(), geo.baseline];
        let [x, y, w, h] = geo.text;
        let [cols, rows] = scene.grid();
        let fit = ((w + gap[0]) / (min_w + gap[0])).floor().max(1.0) as u32;
        if fit >= cols {
            let row = ((h - gap[1] * f64::from(rows - 1)) / f64::from(rows)).max(min_h);
            let shown = f64::from(rows);
            let height = row * shown + gap[1] * (shown - 1.0);
            return Some(scene::place(
                panel.cell,
                [cols, rows],
                [x, y, w, height],
                gap,
            ));
        }
        let cells: Vec<_> = (scene.panels.iter())
            .map(|p| {
                let [x, y, w, rows] = p.cell;
                let rows = match (p.is_empty(), p.empty.is_empty()) {
                    (false, _) => rows,
                    (true, true) => 0,
                    (true, false) => 1,
                };
                [x, y, w, rows]
            })
            .collect();
        let (flowed, flowed_rows) = scene::flow(&cells, fit);
        let at = scene.panels.iter().position(|p| p.id == panel.id)?;
        let [cx, cy, cw, ch] = flowed[at];
        let heights = self.row_heights(scene, &flowed, flowed_rows, (h, gap[1], min_h));
        let [bx, _, bw, _] = scene::place([cx, 0, cw, 1], [fit, 1], [x, y, w, 1.0], gap);
        if ch == 0 {
            let rows = f64::from(panel.cell[3].max(1));
            let by = y + heights.iter().sum::<f64>();
            return Some([bx, by, bw, min_h * rows + gap[1] * (rows - 1.0)]);
        }
        let above: f64 = heights.iter().take(cy as usize).sum();
        let spanned: f64 = heights.iter().skip(cy as usize).take(ch as usize).sum();
        let by = y + above + gap[1] * f64::from(cy);
        Some([bx, by, bw, spanned + gap[1] * f64::from(ch - 1)])
    }

    /// The heights of the rows of a page that wrapped: a row that holds
    /// only panels with their words of emptiness is as high as those
    /// words; the other rows share the rest of the height, and none is
    /// lower than the least height.
    pub(crate) fn row_heights(
        &self,
        scene: &Scene,
        flowed: &[[u32; 4]],
        rows: u32,
        (height, gap, min_h): (f64, f64, f64),
    ) -> Vec<f64> {
        let words = |p: &Panel| p.is_empty() && !p.empty.is_empty();
        let rows_of = |r: u32| {
            scene
                .panels
                .iter()
                .zip(flowed)
                .filter(move |(_, c)| c[3] > 0 && (c[1]..c[1] + c[3]).contains(&r))
                .map(|(p, _)| p)
        };
        let short: Vec<Option<f64>> = (0..rows)
            .map(|r| {
                let mut panels = rows_of(r).peekable();
                panels.peek()?;
                let mut most: f64 = 0.0;
                for p in panels {
                    if !words(p) {
                        return None;
                    }
                    let line = self.ty("body", p.style.as_ref()).line;
                    most = most.max(self.lines_top(p) + 2.0 * line);
                }
                Some(most)
            })
            .collect();
        let taken: f64 = short.iter().flatten().sum();
        let tall = u32::try_from(short.iter().filter(|s| s.is_none()).count()).unwrap_or(1);
        let gaps = gap * f64::from(rows.saturating_sub(1));
        let share = if tall == 0 {
            0.0
        } else {
            ((height - gaps - taken) / f64::from(tall)).max(min_h)
        };
        short.into_iter().map(|s| s.unwrap_or(share)).collect()
    }

    /// The top of the lines of a panel, below its title.
    pub(crate) fn lines_top(&self, panel: &Panel) -> f64 {
        if panel.title.is_empty() {
            return 0.0;
        }
        let title = self.ty("heading", panel.title_style.as_ref()).line;
        title + self.settings.value.scale.baseline
    }

    /// The height of a panel cut to whole lines of its text, so that its
    /// clip never cuts through a line.
    pub(crate) fn whole_lines(&self, panel: &Panel, h: f64) -> f64 {
        let (top, line) = (
            self.lines_top(panel),
            self.ty("body", panel.style.as_ref()).line,
        );
        (top + ((h - top) / line).floor() * line).clamp(0.0, h)
    }

    /// Creates one panel: a group with a clip, its title and its lines,
    /// and the notes in the margin left of it.
    /// One node per row, so that a row wraps and its note stays on
    /// its first line; `align` stacks the rows as the core lays
    /// them out, and moves the notes flush right to the margin.
    pub(crate) fn panel(&self, name: &str, panel: &Panel) -> Vec<Command> {
        let (Some([x, y, w, h]), Some(geo)) = (self.place(name, panel), self.geometry(name)) else {
            return Vec::new();
        };
        let at = Spot {
            name,
            id: panel_id(name, &panel.id),
            indent: geo.note + geo.gap,
            y,
            w,
            h,
        };
        let mut commands = vec![create(
            &at.id,
            NodeKind::Group,
            &panels_id(name),
            PropsUpdate {
                transform: Some(translate(x - at.indent, y)),
                size: Some([w + at.indent, self.whole_lines(panel, h)]),
                clip: Some(Some(chora_scene::Clip { radius: 0.0 })),
                opacity: panel.opacity,
                effects: self.blur_effect(panel).map(|e| vec![e]),
                ..PropsUpdate::default()
            },
        )];
        commands.extend(self.panel_title(&at, panel));
        commands.extend(self.panel_body(&at, panel));
        commands
    }

    /// The title node of a panel, if it has a title.
    fn panel_title(&self, at: &Spot, panel: &Panel) -> Option<Command> {
        if panel.title.is_empty() {
            return None;
        }
        let local = panel
            .title_style
            .as_ref()
            .filter(|_| !panel.lines.is_empty());
        let title = self.ty("heading", local);
        let y = self.lift(at.y, 0.0, &title);
        Some(at.add("title", at.text(&title, &panel.title, y, true)))
    }

    /// The rows, the lines or the empty text of a panel.
    fn panel_body(&self, at: &Spot, panel: &Panel) -> Vec<Command> {
        let top = self.lines_top(panel);
        let body = self.ty("body", panel.style.as_ref());
        if let Some(rows) = panel.columns() {
            self.panel_rows(at, panel, &rows)
        } else if !panel.lines.is_empty() {
            self.panel_lines(at, panel, top, &body)
        } else if !panel.empty.is_empty() {
            self.panel_empty(at, panel, top)
        } else {
            Vec::new()
        }
    }

    fn panel_rows(&self, at: &Spot, panel: &Panel, rows: &[(&str, &str)]) -> Vec<Command> {
        let top = self.lines_top(panel);
        let body = &self.ty("body", panel.style.as_ref());
        let mut commands = Vec::new();
        let note = self.ty("margin", panel.note_style.as_ref());
        let shown = lines_shown(at.h - top, body.line);
        for (i, (time, line)) in rows.iter().take(shown).enumerate() {
            let row = row_top(top, i, body.line);
            let y = self.lift(at.y, row, body);
            let hit = ([at.indent, y, at.w, body.line], body.ink());
            commands.extend(self.row_hit(at.name, &at.id, i, hit));
            let words = at.text(body, line, y, true);
            commands.push(at.add(&format!("line-{i}"), words));
            if !time.is_empty() {
                let y = self.lift(at.y, row, &note);
                let props = at.text_at(&note, time, [0.0, y], false);
                commands.push(at.add(&format!("note-{i}"), props));
            }
        }
        commands
    }

    fn panel_lines(&self, at: &Spot, panel: &Panel, top: f64, body: &Type) -> Vec<Command> {
        let mut commands = Vec::new();
        if at.name == manage::OVERVIEW {
            let hit = [at.indent, top, at.w, body.line];
            commands.extend(self.overview_hits(at.name, &at.id, panel, hit));
        }
        let props = at.text(body, &panel.text(), self.lift(at.y, top, body), true);
        commands.push(at.add("lines", props));
        commands
    }

    fn panel_empty(&self, at: &Spot, panel: &Panel, top: f64) -> Vec<Command> {
        let plain = panel
            .style
            .clone()
            .map(|s| crate::config::Style { base: None, ..s });
        let quiet = self.quiet(&self.ty("body", plain.as_ref()));
        let props = at.text(&quiet, &panel.empty, self.lift(at.y, top, &quiet), true);
        vec![at.add("empty", props)]
    }

    /// The hit of each line of the overview.
    pub(crate) fn overview_hits(
        &self,
        name: &str,
        id: &str,
        panel: &Panel,
        [x, top, w, line]: [f64; 4],
    ) -> Vec<Command> {
        let ink = self.ty("body", panel.style.as_ref()).ink();
        let rows = (0..panel.lines.len()).map(|i| {
            let y = row_top(top, i, line);
            self.row_hit(name, id, i, ([x, y, w, line], ink))
        });
        rows.flatten().collect()
    }

    /// A type in `muted`: the words of an empty panel stay quiet.
    pub(crate) fn quiet(&self, ty: &Type) -> Type {
        let muted = self.settings.value.colors().get("muted").copied();
        let mut quiet = ty.clone();
        quiet.color = muted.unwrap_or(ty.color);
        quiet
    }

    /// Whether the rows of a scene take the pointer: not those of a scene
    /// with very many panels, which would build slowly.
    pub(crate) fn has_hits(&self, scene: &str) -> bool {
        (self.scenes.get(scene)).is_some_and(|s| s.panels.len() <= MAX_HIT_PANELS)
    }

    /// The hit and the focus mark of row `i` of a panel, at a box of the
    /// panel group: the hit spans the margin and the text.
    pub(crate) fn row_hit(
        &self,
        scene: &str,
        panel: &str,
        row: usize,
        (at, caps): ([f64; 4], Ink),
    ) -> Vec<Command> {
        if !self.has_hits(scene) {
            return Vec::new();
        }
        let [x, y, w, h] = at;
        let ink = (self.settings.value.colors().get("text").copied()).unwrap_or([0.0; 4]);
        let base = format!("{panel}-row-{row}");
        let mut commands = region::hit_nodes(&base, panel, ink).to_vec();
        commands.extend(region::hit_place(&base, [0.0, y, x + w, h], x, caps));
        commands
    }

    /// Moves what the core lays out to its place: the title of the head
    /// after the number, the notes of the panels of a scene flush right to
    /// the margin. One query reads the rects of all texts that show.
    /// A title that wraps pushes the lines down by its extra lines.
    /// below the panel
    pub(crate) fn align(&mut self, name: &str) -> Result<(), ClientError> {
        let Some(scene) = self.scenes.get(name).cloned() else {
            return Ok(());
        };
        let Some(geo) = self.geometry(name) else {
            return Ok(());
        };
        let rects = self.text_rects()?;
        let rect = |id: &str| rects.get(id).copied();
        let indent = geo.note + geo.gap;
        let mut commands = Vec::new();
        commands.extend(self.align_head(name, &scene, &geo, &|id| rect(id)));
        for panel in &scene.panels {
            let id = panel_id(name, &panel.id);
            let mut y = self.lines_top(panel);
            if !panel.title.is_empty()
                && let Some([_, _, _, height]) = rect(&format!("{id}-title"))
            {
                let line = self.ty("heading", panel.title_style.as_ref()).line;
                y += (height - line).max(0.0);
            }
            let top_of_panel = self.place(name, panel).map_or(0.0, |p| p[1]);
            let Some(rows) = panel.columns() else {
                if !panel.lines.is_empty() && y > self.lines_top(panel) {
                    let body = self.ty("body", panel.style.as_ref());
                    let at = self.lift(top_of_panel, y, &body);
                    commands.push(move_to(format!("{id}-lines"), indent, at));
                }
                continue;
            };
            let body = self.ty("body", panel.style.as_ref());
            let note = self.ty("margin", panel.note_style.as_ref());
            let wide = self.place(name, panel).map_or(0.0, |p| p[2] + indent);
            for (i, (time, _)) in rows.iter().enumerate() {
                let Some([_, _, _, height]) = rect(&format!("{id}-line-{i}")) else {
                    break;
                };
                let at = self.lift(top_of_panel, y, &body);
                commands.push(move_to(format!("{id}-line-{i}"), indent, at));
                let hit = region::hit_place(
                    &format!("{id}-row-{i}"),
                    [0.0, at, wide, height.max(body.line)],
                    indent,
                    body.ink(),
                );
                commands.extend(hit);
                let note_id = format!("{id}-note-{i}");
                if !time.is_empty()
                    && let Some([_, _, width, _]) = rect(&note_id)
                {
                    let x = (geo.note - width).max(0.0);
                    let at = self.lift(top_of_panel, y, &note);
                    commands.push(move_to(note_id, x, at));
                }
                y += height.max(body.line);
            }
        }
        self.apply(commands)
    }

    /// Scrolls the page of the current scene by `dy`, from its top to its
    /// last panel at the bottom of the text block.
    pub(crate) fn scroll_page(&mut self, node: &str, dy: f64) -> Result<(), ClientError> {
        let over =
            |n: &String| wheel_id(n) == node || node.starts_with(&format!("{}-", panels_id(n)));
        let Some(name) = self.current.clone().filter(over) else {
            return Ok(());
        };
        let (Some(scene), Some(geo)) = (self.scenes.get(&name), self.geometry(&name)) else {
            return Ok(());
        };
        let bottom = (scene.panels.iter())
            .filter_map(|p| self.place(&name, p).map(|[_, y, _, h]| y + h))
            .fold(0.0, f64::max);
        let most = (bottom - geo.text[1] - geo.text[3]).max(0.0);
        let old = self.scroll.get(&name).copied().unwrap_or_default();
        let new = (old + dy).clamp(0.0, most);
        if (new - old).abs() < f64::EPSILON {
            return Ok(());
        }
        self.scroll.insert(name.clone(), new);
        let b = self.settings.value.scale.baseline;
        self.apply(vec![move_to(panels_id(&name), 0.0, -2.0 * b - new)])
    }

    /// The rect that the core lays out a node at: x, y, width, height.
    pub(crate) fn rect(&mut self, id: &str) -> Result<Option<[f64; 4]>, ClientError> {
        let nodes: Vec<Value> =
            (self.rt.conn).call("scene.query", Query::new(format!("id:{id}")))?;
        let rect = nodes.first().and_then(|n| {
            let r = n["rects"].as_object()?.values().next()?.as_array()?;
            let v: Vec<f64> = r.iter().filter_map(Value::as_f64).collect();
            <[f64; 4]>::try_from(v).ok()
        });
        Ok(rect)
    }

    /// The rects that the core lays out the visible texts of the plugin
    /// at, by node, with one query: each query costs the core a walk
    /// over the scene.
    pub(crate) fn text_rects(&mut self) -> Result<HashMap<String, [f64; 4]>, ClientError> {
        let query = Query {
            selector: format!("owner:{NAME} kind:text"),
            filter: Some(Filter {
                visible: true,
                ..Filter::default()
            }),
        };
        let nodes: Vec<Value> = self.rt.conn.call("scene.query", query)?;
        let rects = nodes.iter().filter_map(|n| {
            let r = n["rects"].as_object()?.values().next()?.as_array()?;
            let v: Vec<f64> = r.iter().filter_map(Value::as_f64).collect();
            Some((n["id"].as_str()?.to_owned(), <[f64; 4]>::try_from(v).ok()?))
        });
        Ok(rects.collect())
    }

    /// The rect of an item, or of its hidden copy when `probed`.
    pub(crate) fn measure(&mut self, id: &str, probed: bool) -> Result<[f64; 4], ClientError> {
        let rect = if probed {
            self.rect(&region::probe(id))
        } else {
            self.rect(id)
        };
        Ok(rect?.unwrap_or_default())
    }

    /// The effect of the blur of a panel, if it has one and the watchdog
    /// of the core did not drop the effects.
    pub(crate) fn blur_effect(&self, panel: &Panel) -> Option<ShaderEffect> {
        let radius = (panel.blur).filter(|r| *r > 0.0 && self.shaders == Shaders::On)?;
        Some(ShaderEffect {
            shader: self.blur.to_string_lossy().into_owned(),
            uniforms: [("radius".into(), Uniform::Float(radius))].into(),
        })
    }
}

/// How many lines of a height are shown, the last one cut off as well.
fn lines_shown(height: f64, line: f64) -> usize {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a non-negative line count of the body"
    )]
    let shown = (height / line).ceil().max(0.0) as usize;
    shown
}

/// The top of row `i` of lines that are `line` high, from `top`.
fn row_top(top: f64, i: usize, line: f64) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "rows of one screen")]
    let rows = i as f64;
    top + rows * line
}

/// Where one panel sits: its id, margin indent and box.
struct Spot<'a> {
    name: &'a str,
    id: String,
    indent: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Spot<'_> {
    /// A text node of the panel at the indent.
    fn text(&self, ty: &Type, text: &str, y: f64, wrap: bool) -> PropsUpdate {
        self.text_at(ty, text, [self.indent, y], wrap)
    }

    fn text_at(&self, ty: &Type, text: &str, [x, y]: [f64; 2], wrap: bool) -> PropsUpdate {
        PropsUpdate {
            transform: Some(translate(x, y)),
            max_width: wrap.then_some(self.w),
            ..ty.props(text)
        }
    }

    /// A child text node `part` of the panel group.
    fn add(&self, part: &str, props: PropsUpdate) -> Command {
        create(
            &format!("{}-{part}", self.id),
            NodeKind::Text,
            &self.id,
            props,
        )
    }
}
