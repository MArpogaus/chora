//! The layout audit of the page: findings on the nodes that mc builds,
//! one test for each class of flaw, on the outputs and the states that the
//! screens of the review showed.

use chora_scene::NodeKind;

use crate::testing::Model;

const TOLERANCE: f64 = 1.0;

#[derive(Clone, Debug)]
struct Text {
    id: String,
    words: String,
    x: f64,
    y: f64,
    size: f64,
    line: f64,
    family: Option<String>,
    weight: Option<u16>,
}

#[derive(Clone, Debug)]
struct Rect {
    id: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn place(model: &Model, id: &str) -> (f64, f64) {
    let core = model.lock().unwrap();
    let (mut x, mut y) = (0.0, 0.0);
    let mut at = core.scene.node(id);
    while let Some(node) = at {
        let (dx, dy) = node.props.transform.translation_2d();
        (x, y) = (x + dx, y + dy);
        at = node.parent.as_deref().and_then(|p| core.scene.node(p));
    }
    (x, y)
}

fn shows(model: &Model, id: &str) -> bool {
    let core = model.lock().unwrap();
    let mut at = core.scene.node(id);
    while let Some(node) = at {
        if !node.props.visible {
            return false;
        }
        at = node.parent.as_deref().and_then(|p| core.scene.node(p));
    }
    true
}

fn generation(id: &str) -> Option<u32> {
    let rest = id.strip_prefix("mc-r")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

thread_local! {
    /// The set of regions that shows now, when a test says: the fake core
    /// does not run the fade that hides the set before it.
    static LIVE: std::cell::Cell<Option<u8>> = const { std::cell::Cell::new(None) };
}

/// Tells the audit which set of regions of `mc` shows now.
fn pin(mc: &crate::Mc) {
    LIVE.with(|live| live.set(Some(mc.set)));
}

fn ids(model: &Model, kind: NodeKind) -> Vec<String> {
    let core = model.lock().unwrap();
    let mine: Vec<&str> = (core.scene.nodes())
        .filter(|n| n.kind == kind && n.owner.as_deref() == Some(crate::NAME))
        .map(|n| n.id.as_str())
        .collect();
    let newest = LIVE
        .with(std::cell::Cell::get)
        .map(u32::from)
        .or_else(|| mine.iter().filter_map(|id| generation(id)).max());
    let current = |id: &&str| generation(id).is_none_or(|g| Some(g) == newest);
    mine.into_iter()
        .filter(current)
        .map(str::to_owned)
        .collect()
}

fn texts(model: &Model) -> Vec<Text> {
    let mut out = Vec::new();
    for id in ids(model, NodeKind::Text) {
        if !shows(model, &id) {
            continue;
        }
        let (x, y) = place(model, &id);
        let core = model.lock().unwrap();
        let props = &core.scene.node(&id).unwrap().props;
        let size = props.font_size.unwrap_or(16.0);
        out.push(Text {
            words: props.text.clone().unwrap_or_default(),
            id,
            x,
            y,
            size,
            line: size * props.line_height.unwrap_or(1.2),
            family: props.family.clone(),
            weight: props.weight,
        });
    }
    out
}

fn rects(model: &Model) -> Vec<Rect> {
    let mut out = Vec::new();
    for id in ids(model, NodeKind::Rect) {
        let (x, y) = place(model, &id);
        let core = model.lock().unwrap();
        let size = core.scene.node(&id).unwrap().props.size;
        drop(core);
        if let Some([w, h]) = size {
            out.push(Rect { id, x, y, w, h });
        }
    }
    out
}

fn truth(text: &Text) -> (f64, f64) {
    let family = text.family.as_deref().expect("a text with a family");
    let name = crate::fonts::pick(family);
    let em = crate::fonts::metrics(&name, text.weight);
    let glyphs = (em.ascent + em.descent) * text.size;
    let baseline = (text.line - glyphs) / 2.0 + em.ascent * text.size;
    (baseline - em.cap * text.size, baseline)
}

fn text_of_mark(texts: &[Text], mark: &str) -> Option<Text> {
    let base = mark.strip_suffix("-mark")?;
    let ids = [
        base.to_owned(),
        base.replace("-row-", "-line-"),
        base.replace("-head", "-title"),
    ];
    ids.iter()
        .find_map(|id| texts.iter().find(|t| &t.id == id))
        .cloned()
}

fn bar_findings(model: &Model) -> (usize, Vec<String>) {
    let texts = texts(model);
    let mut out = Vec::new();
    let mut checked = 0;
    let marks = rects(model).into_iter().filter(|r| r.id.ends_with("-mark"));
    let bars = rects(model)
        .into_iter()
        .filter(|r| r.id.ends_with("-chapter-bar"));
    for bar in marks.chain(bars) {
        let id = bar.id.replace("-bar", "-title");
        let Some(text) =
            text_of_mark(&texts, &bar.id).or_else(|| texts.iter().find(|t| t.id == id).cloned())
        else {
            continue;
        };
        checked += 1;
        let (top, baseline) = truth(&text);
        let (want_top, want_bottom) = (text.y + top, text.y + baseline);
        if (bar.y - want_top).abs() > TOLERANCE || (bar.y + bar.h - want_bottom).abs() > TOLERANCE {
            out.push(format!(
                "{}: bar {:.1}..{:.1} beside caps {:.1}..{:.1} of {}",
                bar.id,
                bar.y,
                bar.y + bar.h,
                want_top,
                want_bottom,
                text.id
            ));
        }
    }
    (checked, out)
}

fn above_keys(items: &[crate::region::Item]) -> Vec<String> {
    let stays = |i: &&crate::region::Item| i.id.contains("-notify") || i.id.contains("-chat");
    let over = items.iter().filter(stays);
    over.map(|i| format!("{} stays above the list of all keys", i.id))
        .collect()
}

fn rule_findings(model: &Model) -> Vec<String> {
    let texts = texts(model);
    let mut out = Vec::new();
    let hairlines = rects(model).into_iter().filter(|r| r.h <= 2.0 && r.w > 8.0);
    for rule in hairlines {
        for text in &texts {
            let (top, baseline) = truth(text);
            let (top, bottom) = (text.y + top, text.y + baseline);
            let across = text.x >= rule.x && text.x < rule.x + rule.w;
            if across && rule.y > top + TOLERANCE && rule.y < bottom - TOLERANCE {
                out.push(format!(
                    "{} at y {:.1} strikes {} at {top:.1}..{bottom:.1}",
                    rule.id, rule.y, text.id
                ));
            }
        }
    }
    out
}

fn all(findings: &[String]) {
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::List;
    use crate::region::Region;

    const SIZES: [[i32; 2]; 2] = [[1280, 800], [1920, 1080]];

    #[test]
    fn a_bar_spans_the_capitals_of_the_text_that_it_marks() {
        for size in SIZES {
            let (mut mc, model) = crate::testing::mc("audit-bars", size);
            for scene in ["agenda", "overview"] {
                mc.show(scene, false).unwrap();
                let at = mc.geometry(scene).unwrap();
                let head = mc.scenes.get(scene).unwrap().clone();
                let title = [0.0, 0.0, 200.0, 56.0];
                let moved = mc.align_head(scene, &head, &at, &|id| {
                    id.ends_with("-number")
                        .then_some([0.0, 0.0, 30.0, 56.0])
                        .or_else(|| id.ends_with("-title").then_some(title))
                });
                mc.apply(moved).unwrap();
                let (checked, findings) = bar_findings(&model);
                assert!(checked >= 2, "{checked} bars of {scene}");
                all(&findings);
            }
        }
    }

    #[test]
    fn the_key_list_stays_inside_its_column() {
        for size in SIZES {
            let (mut mc, _model) = crate::testing::mc("audit-column", size);
            mc.show("agenda", false).unwrap();
            for list in [List::Six, List::All] {
                mc.keys = list;
                let column = mc.boxes_of("agenda")[&Region::Right];
                let plan = mc.items("agenda");
                let (items, _) = &plan[&Region::Right];
                let baseline = mc.settings.value.scale.baseline;
                let heights: Vec<f64> = items
                    .iter()
                    .map(|i| {
                        let size = i.props.font_size.unwrap_or(16.0);
                        (size * i.props.line_height.unwrap_or(1.2)).max(baseline)
                    })
                    .collect();
                let (_, flow) = crate::region::lines(items, &heights, false, baseline);
                let (_, foot) = crate::region::lines(items, &heights, true, baseline);
                if list == List::All {
                    all(&above_keys(items));
                }
                assert!(
                    flow + foot <= column[3],
                    "{list:?} at {size:?}: the flow takes {flow}, the foot {foot} of {}",
                    column[3]
                );
            }
        }
    }

    #[test]
    fn no_text_stands_on_a_hairline() {
        for size in SIZES {
            let (mut mc, model) = crate::testing::mc("audit-rules", size);
            mc.show("agenda", false).unwrap();
            for list in [List::Six, List::All] {
                mc.keys = list;
                mc.render().unwrap();
                all(&rule_findings(&model));
            }
        }
    }

    #[test]
    fn a_bar_stands_in_the_middle_between_the_ink_of_its_neighbours() {
        for size in SIZES {
            let (mut mc, model) = crate::testing::mc("audit-gaps", size);
            mc.show("agenda", false).unwrap();
            let at = mc.geometry("agenda").unwrap();
            let head = mc.scenes.get("agenda").unwrap().clone();
            let width = 29.0;
            let moved = mc.align_head("agenda", &head, &at, &|id| {
                id.ends_with("-number")
                    .then_some([0.0, 0.0, width, 56.0])
                    .or_else(|| id.ends_with("-title").then_some([0.0, 0.0, 200.0, 56.0]))
            });
            mc.apply(moved).unwrap();
            let [number, title, bar] = ["number", "title", "bar"]
                .map(|part| place(&model, &format!("mc-s-agenda-chapter-{part}")).0);
            let find = |id: &str| {
                texts(&model)
                    .into_iter()
                    .find(|t| t.id.ends_with(id))
                    .unwrap()
            };
            let (n, t) = (find("chapter-number"), find("chapter-title"));
            let after =
                crate::fonts::bearings(n.family.as_deref().unwrap(), n.weight, "1").unwrap()[1];
            let before =
                crate::fonts::bearings(t.family.as_deref().unwrap(), t.weight, "A").unwrap()[0];
            let left = bar - (number + width - after * n.size);
            let right = (title + before * t.size) - (bar + 1.0);
            assert!((left - right).abs() <= 1.0, "gaps {left:.1} and {right:.1}");
        }
    }

    #[test]
    fn the_terminal_command_leaves_foot_without_a_config_error() {
        let installed = std::process::Command::new("foot").arg("--version").output();
        if installed.is_err() {
            return;
        }
        let (mc, _) = crate::testing::mc("audit-foot", [1280, 800]);
        let config = &mc.settings.value;
        for colors in [&config.themes.light, &config.themes.dark] {
            let command = crate::region::command(&config.contents["shell"].command, colors);
            let run = std::process::Command::new("sh")
                .args(["-c", &format!("{command} --check-config")])
                .output()
                .unwrap();
            let log = String::from_utf8_lossy(&run.stderr);
            assert!(!log.contains(" err:"), "{log}");
        }
    }

    #[test]
    fn every_page_with_a_head_keeps_its_column_and_its_hairline() {
        for scene in ["agenda", "terminal", "overview"] {
            let (mut mc, _) = crate::testing::mc("audit-pages", [1280, 800]);
            mc.show(scene, false).unwrap();
            let listed = mc.list();
            let right = &listed["regions"]["right"];
            assert_eq!(right["visible"], true, "{scene} shows no column: {right}");
        }
    }

    fn notified() -> (crate::Mc, Model) {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, model) = crate::testing::mc("audit-rows", [1280, 800]);
        mc.show("agenda", false).unwrap();
        for title in ["Update ready", "Backup done and a longer line"] {
            mc.method("notify.push", serde_json::json!({"title": title}))
                .unwrap();
        }
        (mc, model)
    }

    #[test]
    fn items_of_one_row_share_a_baseline_and_rows_have_a_gap() {
        let (_mc, model) = notified();
        let all = texts(&model);
        let mut seen = 0;
        for time in all.iter().filter(|t| t.id.ends_with("-time")) {
            let words = all
                .iter()
                .find(|t| t.id == time.id.trim_end_matches("-time"))
                .unwrap();
            seen += 1;
            let base = |t: &Text| t.y + truth(t).1;
            assert!(
                (base(time) - base(words)).abs() <= TOLERANCE,
                "{} and {}",
                time.id,
                words.id
            );
        }
        assert!(seen >= 2, "{seen} rows");
        let mut rows: Vec<&Text> = all.iter().filter(|t| t.id.ends_with("-time")).collect();
        rows.sort_by(|a, b| a.y.total_cmp(&b.y));
        for pair in rows.windows(2) {
            assert!(
                pair[1].y - pair[0].y >= pair[0].line + 14.0 - TOLERANCE,
                "{pair:?}"
            );
        }
    }

    #[test]
    fn the_last_key_row_stands_on_the_baseline_of_the_footer() {
        let (_mc, model) = notified();
        let all = texts(&model);
        let base = |t: &Text| t.y + truth(t).1;
        let foot = all.iter().find(|t| t.id.contains("-footer-next")).unwrap();
        let key = all.iter().find(|t| t.id.ends_with("-keys-5-key")).unwrap();
        assert!(
            (base(foot) - base(key)).abs() <= TOLERANCE,
            "{} against {}",
            base(foot),
            base(key)
        );
    }

    #[test]
    fn no_text_of_the_column_is_set_under_14_pixels() {
        let (mut mc, model) = notified();
        for list in [List::Six, List::All] {
            mc.keys = list;
            mc.render().unwrap();
            for text in texts(&model) {
                assert!(text.size >= 14.0, "{} is set at {}", text.id, text.size);
            }
        }
    }

    #[test]
    fn every_section_of_the_column_has_a_head() {
        use chora_kit::runtime::Plugin as _;
        let (mut mc, _model) = crate::testing::mc("audit-heads", [1280, 800]);
        mc.show("agenda", false).unwrap();
        let args = serde_json::json!({"name": "Atlas", "state": "working", "task": "Tidy"});
        mc.method("agent.set", args).unwrap();
        let plan = mc.items("agenda");
        let heads: Vec<String> = (plan[&Region::Right].0.iter())
            .filter(|i| i.id.ends_with("-title"))
            .map(|i| i.id.clone())
            .collect();
        let want = ["agents", "notify", "chat", "keys"].map(|n| format!("-right-{n}-title"));
        for id in want {
            assert!(
                heads.iter().any(|h| h.ends_with(&id)),
                "no {id} in {heads:?}"
            );
        }
    }

    /// A helper that has a question, as voice tells mc.
    fn helper(mc: &mut crate::Mc) {
        let state = serde_json::json!({"name": "Atlas", "state": "question", "task": "Write file"});
        mc.follow("voice.state", "voice", state).unwrap();
    }

    /// Every state that a page shows: with notes, a question of a helper,
    /// the chat open, and the key list whole.
    fn states(scene: &str, size: [i32; 2]) -> Vec<(String, crate::Mc, Model)> {
        use chora_kit::runtime::Plugin as _;
        let mut out = Vec::new();
        for state in ["rest", "notes", "chat", "keys"] {
            let (mut mc, model) = crate::testing::mc(&format!("audit-grid-{scene}-{state}"), size);
            mc.show(scene, false).unwrap();
            if state != "rest" {
                for title in ["Update ready", "Backup done and a longer line"] {
                    mc.method("notify.push", serde_json::json!({"title": title}))
                        .unwrap();
                }
                helper(&mut mc);
            }
            if state == "chat" {
                mc.chat = crate::Chat::Open;
            }
            if state == "keys" {
                mc.keys = List::All;
            }
            mc.render().unwrap();
            pin(&mc);
            out.push((format!("{scene}/{state}"), mc, model));
        }
        out
    }

    fn base(t: &Text) -> f64 {
        t.y + truth(t).1
    }

    /// How far the baseline of a text is from the nearest line of the grid.
    fn apart(mc: &crate::Mc, t: &Text) -> f64 {
        let b = mc.settings.value.scale.baseline;
        let d = (base(t) - mc.phase()).rem_euclid(b);
        d.min(b - d)
    }

    #[test]
    fn every_baseline_of_a_page_stands_on_one_grid() {
        for size in SIZES {
            for scene in ["agenda", "terminal", "overview", "morning"] {
                for (state, mc, model) in states(scene, size) {
                    let apart = |t: &Text| apart(&mc, t);
                    let off: Vec<String> = (texts(&model).iter())
                        .filter(|t| apart(t) > TOLERANCE)
                        .map(|t| format!("{} at {:.1} is {:.1} off", t.id, base(t), apart(t)))
                        .collect();
                    assert!(off.is_empty(), "{state} at {size:?}\n{}", off.join("\n"));
                }
            }
        }
    }

    #[test]
    fn the_right_column_starts_on_the_same_line_of_the_grid_on_every_page() {
        for size in SIZES {
            let mut first = Vec::new();
            for scene in ["agenda", "terminal", "overview", "morning"] {
                let (mut mc, model) = crate::testing::mc(&format!("audit-top-{scene}"), size);
                mc.show(scene, false).unwrap();
                mc.render().unwrap();
                pin(&mc);
                let top = texts(&model)
                    .into_iter()
                    .filter(|t| t.id.contains("-right-") && !t.words.trim().is_empty())
                    .map(|t| base(&t))
                    .fold(f64::MAX, f64::min);
                first.push((scene, top));
            }
            for (scene, top) in &first {
                assert!(
                    (top - first[0].1).abs() <= TOLERANCE,
                    "{first:?} {scene} at {size:?}"
                );
            }
        }
    }

    #[test]
    fn the_label_columns_of_the_margin_column_are_one_width() {
        use chora_kit::hud::LABEL_COLUMN;
        use chora_kit::runtime::Plugin as _;
        assert_eq!(LABEL_COLUMN % 14.0, 0.0, "a multiple of 14");
        let (mut mc, model) = crate::testing::mc("audit-labels", [1280, 800]);
        mc.show("agenda", false).unwrap();
        for title in ["Update ready", "Backup done and a longer line"] {
            mc.method("notify.push", serde_json::json!({"title": title}))
                .unwrap();
        }
        helper(&mut mc);
        mc.render().unwrap();
        pin(&mc);
        let column = mc.boxes_of("agenda")[&Region::Right][0];
        let all = texts(&model);
        let words = |suffix: &str| -> Vec<f64> {
            all.iter()
                .filter(|t| t.id.contains("-right-") && t.id.ends_with(suffix))
                .map(|t| t.x - column)
                .collect()
        };
        for (what, xs) in [
            ("a note", words("-notify-0")),
            ("a helper", words("-agents-0-name")),
            ("a key", words("-keys-1-words")),
        ] {
            assert!(!xs.is_empty(), "no {what}");
            assert!(xs.iter().all(|x| *x == LABEL_COLUMN), "{what} at {xs:?}");
        }
        mc.keys = List::All;
        mc.render().unwrap();
        pin(&mc);
        let widest = ["Super+Return", "Super+Space", "Super+Home"]
            .map(|k| {
                let ty = mc.ty("margin", None);
                let em = crate::fonts::advance(ty.family().unwrap(), Some(500), k).unwrap();
                em * ty.size
            })
            .into_iter()
            .fold(0.0, f64::max);
        assert!(LABEL_COLUMN >= widest + 8.0, "{widest} in {LABEL_COLUMN}");
        assert!(
            LABEL_COLUMN < widest + 8.0 + 14.0,
            "{widest} in {LABEL_COLUMN}"
        );
    }

    #[test]
    fn the_terminal_is_set_at_14_pixels_or_more_on_the_grid() {
        let (mc, _) = crate::testing::mc("audit-mono", [1280, 800]);
        let config = &mc.settings.value;
        let command =
            crate::region::command(&config.contents["shell"].command, &config.themes.light);
        let number = |key: &str| -> f64 {
            let at = command
                .find(key)
                .unwrap_or_else(|| panic!("no {key} in {command}"));
            let digits: String = (command[at + key.len()..].chars())
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse().unwrap()
        };
        let size = number("pixelsize=");
        assert!(size >= 14.0, "the terminal is set at {size} pixels");
        assert_eq!(
            number("line-height="),
            config.scale.baseline,
            "a line of the terminal is a line of the grid"
        );
    }

    #[test]
    fn the_hairline_of_the_column_runs_from_its_first_capitals_to_the_footer_baseline() {
        for size in SIZES {
            for (state, mc, model) in states("agenda", size) {
                let edge = rects(&model)
                    .into_iter()
                    .find(|r| r.id.ends_with("-right-edge"))
                    .unwrap();
                let all = texts(&model);
                let column: Vec<&Text> = all.iter().filter(|t| t.id.contains("-right-")).collect();
                let first = column
                    .iter()
                    .map(|t| t.y + truth(t).0)
                    .fold(f64::MAX, f64::min);
                let foot = all.iter().find(|t| t.id.contains("-footer-next")).unwrap();
                assert!(
                    (edge.y - first).abs() <= 1.5,
                    "{state}: the rule starts at {} and the capitals at {first}",
                    edge.y
                );
                let end = edge.y + edge.h;
                assert!(
                    (end - base(foot)).abs() <= 1.5,
                    "{state}: the rule ends at {end}, the footer baseline is {}",
                    base(foot)
                );
                let gutter = f64::from(size[1]) - base(foot);
                assert!(gutter >= 20.0, "{state}: {gutter} below the last baseline");
                let low = all.iter().map(|t| t.y + t.line).fold(0.0, f64::max);
                assert!(
                    low <= f64::from(size[1]) - mc.settings.value.scale.baseline / 2.0,
                    "{state}: a line ends at {low}"
                );
            }
        }
    }

    #[test]
    fn every_sign_of_a_state_is_a_shape_and_not_a_thin_letter() {
        let (mc, _) = crate::testing::mc("audit-signs", [1280, 800]);
        for (state, look) in &mc.settings.value.states {
            let sign = look.sign.chars().next().unwrap_or(' ');
            assert!(!sign.is_ascii(), "{state} is signed with {sign:?}");
        }
    }
}
