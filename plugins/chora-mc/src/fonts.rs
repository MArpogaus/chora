//! The family of a style: a list of names, the first that the machine has.
//! The core takes one name and shows a missing family in a sans, which
//! left the page set in two sans faces without TeX Gyre Pagella.

use std::{process::Command, sync::OnceLock};

/// The family names that the machine has: those of fontconfig and of the
/// directories of `CHORA_FONT_DIRS`, one line of comma separated names for
/// each face; none when fontconfig cannot be asked.
fn listing() -> Option<&'static str> {
    static LISTING: OnceLock<Option<String>> = OnceLock::new();
    LISTING
        .get_or_init(|| {
            let mut all = run(Command::new("fc-list").args([":", "family"]))?;
            let dirs = std::env::var("CHORA_FONT_DIRS").unwrap_or_default();
            for dir in dirs.split(':').filter(|dir| !dir.is_empty()) {
                let scan = Command::new("fc-scan")
                    .args(["--format", "%{family}\n", dir])
                    .output();
                if let Some(more) = scan.ok().as_ref().and_then(text_of) {
                    all.push('\n');
                    all.push_str(&more);
                }
            }
            Some(all)
        })
        .as_deref()
}

fn run(command: &mut Command) -> Option<String> {
    command.output().ok().as_ref().and_then(text_of)
}

fn text_of(output: &std::process::Output) -> Option<String> {
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The first name of a list of families that a listing has; the first name
/// when there is no listing or none of them is in it.
#[must_use]
pub fn first_of(list: &str, listing: Option<&str>) -> String {
    let names: Vec<&str> = list.split(',').map(str::trim).collect();
    let first = names.first().copied().unwrap_or_default();
    let Some(listing) = listing else {
        return first.to_owned();
    };
    let has = |name: &str| {
        listing
            .lines()
            .flat_map(|line| line.split(','))
            .any(|family| family.trim() == name)
    };
    names
        .iter()
        .find(|name| has(name))
        .copied()
        .unwrap_or(first)
        .to_owned()
}

/// The family to ask the core for: the first of a list that the machine
/// has.
#[must_use]
pub fn pick(list: &str) -> String {
    if !list.contains(',') {
        return list.to_owned();
    }
    first_of(list, listing())
}

/// The vertical metrics of a face in em: the ascent and the descent that
/// the core centers a line on, and the height of the capitals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Em {
    pub ascent: f64,
    pub descent: f64,
    pub cap: f64,
}

/// What the core assumes of a face that the machine cannot read.
const UNKNOWN: Em = Em {
    ascent: 0.93,
    descent: 0.25,
    cap: 0.7,
};

/// The face file for a family and a weight: a face of the directories of
/// `CHORA_FONT_DIRS` first, else what fontconfig matches.
fn file_of(family: &str, weight: u16) -> Option<String> {
    let dirs = std::env::var("CHORA_FONT_DIRS").unwrap_or_default();
    for dir in dirs.split(':').filter(|dir| !dir.is_empty()) {
        let scan = Command::new("fc-scan")
            .args(["--format", "%{family}\t%{weight}\t%{file}\n", dir])
            .output();
        let Some(text) = scan.ok().as_ref().and_then(text_of) else {
            continue;
        };
        let near = |w: &str| (fc_weight(weight) - w.parse::<f64>().unwrap_or(80.0)).abs();
        let best = text
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\t');
                let (names, w, file) = (parts.next()?, parts.next()?, parts.next()?);
                names
                    .split(',')
                    .any(|n| n.trim() == family)
                    .then(|| (near(w), file.to_owned()))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, file)) = best {
            return Some(file);
        }
    }
    let pattern = format!("{family}:weight={}", fc_weight(weight));
    run(Command::new("fc-match").args(["-f", "%{file}", &pattern])).filter(|f| !f.is_empty())
}

/// The fontconfig weight of a CSS weight.
fn fc_weight(css: u16) -> f64 {
    match css {
        ..=300 => 50.0,
        301..=450 => 80.0,
        451..=550 => 100.0,
        551..=650 => 180.0,
        _ => 200.0,
    }
}

fn face_bytes(family: &str, weight: u16) -> Option<Vec<u8>> {
    std::fs::read(file_of(family, weight)?).ok()
}

/// The metrics of the face for `family` (one name or a list) at a weight
/// (default 400), as the core lays it out.
#[must_use]
pub fn metrics(family: &str, weight: Option<u16>) -> Em {
    type Cache = std::collections::HashMap<(String, u16), Em>;
    static CACHE: OnceLock<std::sync::Mutex<Cache>> = OnceLock::new();
    let weight = weight.unwrap_or(400);
    let name = pick(family);
    let cache = CACHE.get_or_init(Default::default);
    if let Some(em) = cache
        .lock()
        .ok()
        .and_then(|c| c.get(&(name.clone(), weight)).copied())
    {
        return em;
    }
    let em = face_bytes(&name, weight)
        .and_then(|bytes| em_of(&bytes))
        .unwrap_or(UNKNOWN);
    if let Ok(mut cache) = cache.lock() {
        cache.insert((name, weight), em);
    }
    em
}

fn em_of(bytes: &[u8]) -> Option<Em> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let upem = f64::from(face.units_per_em());
    let cap = face.capital_height().map(f64::from).or_else(|| {
        let id = face.glyph_index('H')?;
        face.glyph_bounding_box(id).map(|b| f64::from(b.y_max))
    })?;
    Some(Em {
        ascent: f64::from(face.ascender()) / upem,
        descent: -f64::from(face.descender()) / upem,
        cap: cap / upem,
    })
}

/// The bar that marks a text: from the top of its capitals to its
/// baseline, as an offset from the top of the text box and a height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ink {
    pub top: f64,
    pub height: f64,
}

/// The capitals of a text of `size` pixels in lines of `line` pixels:
/// the core centers the ascent and the descent in the line, and the
/// baseline sits one ascent below that.
#[must_use]
pub fn ink(family: Option<&str>, weight: Option<u16>, size: f64, line: f64) -> Ink {
    let em = family.map_or(UNKNOWN, |f| metrics(f, weight));
    ink_of(em, size, line)
}

#[must_use]
pub fn ink_of(em: Em, size: f64, line: f64) -> Ink {
    let baseline = ((line - (em.ascent + em.descent) * size) / 2.0 + em.ascent * size).round();
    let height = (em.cap * size).round();
    Ink {
        top: baseline - height,
        height,
    }
}

/// The advance of a text in em, without kerning: what the label columns
/// are measured with.
#[cfg(test)]
#[must_use]
pub fn advance(family: &str, weight: Option<u16>, text: &str) -> Option<f64> {
    let name = pick(family);
    let bytes = face_bytes(&name, weight.unwrap_or(400))?;
    let face = ttf_parser::Face::parse(&bytes, 0).ok()?;
    let upem = f64::from(face.units_per_em());
    text.chars()
        .map(|c| {
            let id = face.glyph_index(c)?;
            Some(f64::from(face.glyph_hor_advance(id)?) / upem)
        })
        .sum()
}

/// The ink of the first and the last glyph of a text in em: the space
/// between the edge of its box and the first ink, and between the last ink
/// and the end of its advance.
#[must_use]
pub fn bearings(family: &str, weight: Option<u16>, text: &str) -> Option<[f64; 2]> {
    let name = pick(family);
    let bytes = face_bytes(&name, weight.unwrap_or(400))?;
    let face = ttf_parser::Face::parse(&bytes, 0).ok()?;
    let upem = f64::from(face.units_per_em());
    let side = |c: char| -> Option<[f64; 2]> {
        let id = face.glyph_index(c)?;
        let b = face.glyph_bounding_box(id)?;
        let adv = f64::from(face.glyph_hor_advance(id)?);
        Some([f64::from(b.x_min) / upem, (adv - f64::from(b.x_max)) / upem])
    };
    let first = side(text.chars().next()?)?;
    let last = side(text.chars().next_back()?)?;
    Some([first[0], last[1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "TeX Gyre Pagella, DejaVu Serif";

    #[test]
    fn the_first_family_that_the_machine_has_wins() {
        let both = "Inter,Inter Medium\nTeX Gyre Pagella\nDejaVu Serif,DejaVu Serif Bold";
        assert_eq!(first_of(LIST, Some(both)), "TeX Gyre Pagella");
        let one = "Inter\nDejaVu Serif,DejaVu Serif Bold";
        assert_eq!(first_of(LIST, Some(one)), "DejaVu Serif");
    }

    #[test]
    fn the_capitals_of_a_line_sit_between_the_cap_top_and_the_baseline() {
        let em = Em {
            ascent: 0.9,
            descent: 0.3,
            cap: 0.7,
        };
        let ink = ink_of(em, 40.0, 56.0);
        assert_eq!(ink.height, 28.0);
        assert_eq!(ink.top + ink.height, 44.0 - 4.0, "the baseline");
    }

    #[test]
    fn without_a_listing_or_a_match_the_first_name_stays() {
        assert_eq!(first_of(LIST, None), "TeX Gyre Pagella");
        assert_eq!(first_of(LIST, Some("Inter")), "TeX Gyre Pagella");
        assert_eq!(pick("Inter"), "Inter");
    }
}
