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
    fn without_a_listing_or_a_match_the_first_name_stays() {
        assert_eq!(first_of(LIST, None), "TeX Gyre Pagella");
        assert_eq!(first_of(LIST, Some("Inter")), "TeX Gyre Pagella");
        assert_eq!(pick("Inter"), "Inter");
    }
}
