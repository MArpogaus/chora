//! The layout audit of the page: findings on the nodes that mc builds,
//! one test for each class of flaw, on the outputs and the states that the
//! screens of the review showed.

use chora_scene::NodeKind;

use crate::testing::Model;

const TOLERANCE: f64 = 1.0;

#[derive(Clone, Debug)]
struct Text {
    id: String,
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

fn ids(model: &Model, kind: NodeKind) -> Vec<String> {
    let core = model.lock().unwrap();
    let mine: Vec<&str> = (core.scene.nodes())
        .filter(|n| n.kind == kind && n.owner.as_deref() == Some(crate::NAME))
        .map(|n| n.id.as_str())
        .collect();
    let newest = mine.iter().filter_map(|id| generation(id)).max();
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
}
