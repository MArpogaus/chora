//! The commands that run without the host: `--check-hud DIR`,
//! `--list-huds` and `--use-hud NAME`.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use chora_draw::theme::{Sheet, Theme};
use chora_kit::{
    hud::{self, Chat, Hud},
    settings::{self, Store as _},
};
use serde_json::json;

use crate::{
    config::{Config, DEFAULT},
    folders,
};

/// The colours of the neutral chat, which a folder's `chat:` block inherits
/// where it leaves a colour out.
#[must_use]
pub fn host_chat() -> Chat {
    Chat {
        fill: Some([250, 250, 250]),
        text: Some([32, 32, 32]),
        name: Some([0, 80, 160]),
        ..Chat::default()
    }
}

/// Every problem of the folder `dir`, in plain words, each with its file and
/// key: the limits and links, `hud.yaml`, the pictures that it names and the
/// colours of its chat. Empty when the folder is fine.
#[must_use]
pub fn problems(dir: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let text = hud::file(dir, "hud.yaml")
        .ok()
        .and_then(|data| String::from_utf8(data).ok());
    let loaded = hud::load(dir);
    let hud = match loaded {
        Ok(hud) => Some(hud),
        Err(found) => {
            errors.extend(found);
            text.as_deref()
                .and_then(|text| parse_alone(text, &mut errors))
        }
    };
    let files = match (&hud, &text) {
        (Some(hud), _) => picture_files(hud),
        (None, Some(text)) => picture_names_in(text),
        (None, None) => Vec::new(),
    };
    let font = hud.as_ref().map_or("sans-serif", |hud| hud.font.as_str());
    for name in files {
        if !errors.iter().any(|e| e.contains(&name)) {
            check_picture(dir, &name, font, &mut errors);
        }
    }
    if let Some(hud) = &hud {
        errors.extend(hud.chat.contrast_errors(&host_chat()));
    }
    errors
}

/// The text of `hud.yaml` parsed on its own, so that its errors show also
/// when the scan of the folder found others.
fn parse_alone(text: &str, errors: &mut Vec<String>) -> Option<Hud> {
    match hud::parse(text) {
        Ok(hud) => Some(hud),
        Err(found) => {
            for error in found {
                if !errors.contains(&error) {
                    errors.push(error);
                }
            }
            None
        }
    }
}

/// The SVG files that the panels name, each once.
fn picture_files(hud: &Hud) -> Vec<String> {
    let mut files: Vec<String> = hud
        .panels
        .iter()
        .filter_map(|panel| panel.picture.as_deref())
        .filter_map(|picture| picture.split('#').next())
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    files
}

/// The files that the `picture:` keys of a `hud.yaml` name, for a file that
/// does not parse, so that its pictures are checked too.
fn picture_names_in(text: &str) -> Vec<String> {
    let mut files: Vec<String> = text
        .split("picture:")
        .skip(1)
        .filter_map(|rest| {
            let value = rest.trim_start().split(['\n', ',', '}']).next()?;
            let name = value.trim().trim_matches(['\'', '"']).split('#').next()?;
            (!name.is_empty()).then(|| name.to_owned())
        })
        .collect();
    files.sort();
    files.dedup();
    files
}

/// The number of pictures checked so far, for a cache directory of its own.
static CHECKS: AtomicUsize = AtomicUsize::new(0);

/// Reads a picture as the host would and adds what is wrong with it.
fn check_picture(dir: &Path, name: &str, font: &str, errors: &mut Vec<String>) {
    let data = match hud::file(dir, name) {
        Ok(data) => data,
        Err(e) => {
            errors.push(format!("{name}: {e}"));
            return;
        }
    };
    let text = String::from_utf8_lossy(&data);
    for target in outside_links(&text) {
        errors.push(format!(
            "{name}: a picture links to {target}, outside the folder"
        ));
    }
    let cache = std::env::temp_dir().join(format!(
        "chora-assistant-check-{}-{}",
        std::process::id(),
        CHECKS.fetch_add(1, Ordering::Relaxed)
    ));
    let sheet = Sheet(Box::leak(name.to_owned().into_boxed_str()));
    if let Err(e) = Theme::load_hud(dir, &cache, &[sheet], font) {
        errors.push(e);
    }
    let _ = std::fs::remove_dir_all(cache);
}

/// The targets of `href` in SVG text that are neither `data:`, nor a
/// reference inside the document, nor a plain path below the folder.
fn outside_links(svg: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = svg;
    while let Some(at) = rest.find("href=") {
        rest = &rest[at + "href=".len()..];
        let Some(quote) = rest.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
            continue;
        };
        let body = &rest[1..];
        let Some(end) = body.find(quote) else {
            break;
        };
        let target = &body[..end];
        let inside = target.starts_with("data:")
            || target.starts_with('#')
            || !(target.contains(':')
                || target.starts_with('/')
                || target.split('/').any(|part| part == ".."));
        if !inside {
            found.push(target.to_owned());
        }
    }
    found
}

/// The text of `--check-hud`, and whether the folder is fine.
#[must_use]
pub fn check_report(dir: &Path) -> (String, bool) {
    let found = problems(dir);
    if found.is_empty() {
        let name = dir
            .canonicalize()
            .ok()
            .and_then(|real| real.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| dir.display().to_string());
        return (
            format!("OK. Turn it on: chora-assistant --use-hud {name}\n"),
            true,
        );
    }
    let count = found.len();
    let noun = if count == 1 { "problem" } else { "problems" };
    let text = format!(
        "{}\n{count} {noun}; this folder is not used until they are fixed.\n",
        found.join("\n")
    );
    (text, false)
}

/// The text of `--list-huds`: one line for each folder, its name and place.
#[must_use]
pub fn list_report(roots: &[PathBuf]) -> String {
    let lines: Vec<String> = folders::list(roots)
        .into_iter()
        .map(|(name, dir)| format!("{name}  {}\n", dir.display()))
        .collect();
    lines.concat()
}

/// `--use-hud NAME`: writes `hud: NAME` to the settings file at `file`;
/// `default` takes the line out, which gives the neutral folder again.
///
/// # Errors
///
/// No folder of that name, a folder with problems, or a settings file that
/// cannot be read or written; the message says what to do.
pub fn use_hud(name: &str, roots: &[PathBuf], file: PathBuf) -> Result<String, String> {
    let value = if name == "default" {
        json!(null)
    } else {
        let dir = folders::find(roots, name).ok_or_else(|| {
            format!("no HUD folder named {name}; chora-assistant --list-huds lists them")
        })?;
        let found = problems(&dir);
        if !found.is_empty() {
            return Err(format!(
                "{}\nnot used: run chora-assistant --check-hud {}",
                found.join("\n"),
                dir.display()
            ));
        }
        json!(name)
    };
    let mut settings = settings::File::<Config>::load(file, DEFAULT)?;
    settings
        .set(json!({ "hud": value }))
        .map_err(|e| e.to_string())?;
    settings.save().map_err(|e| e.to_string())?;
    Ok(format!(
        "{} is set in {}. A running chora-assistant takes it with: choractl call assistant settings.reload\n",
        if name == "default" {
            "the neutral folder".to_owned()
        } else {
            format!("hud: {name}")
        },
        settings.file().display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/hud")
            .join(name)
    }

    fn neutral() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("huds/neutral")
    }

    fn temp(test: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("chora-assistant-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_neutral_folder_and_a_copy_are_fine() {
        assert_eq!(problems(&neutral()), Vec::<String>::new());
        assert_eq!(problems(&fixture("mine")), Vec::<String>::new());
        let (text, ok) = check_report(&fixture("mine"));
        assert!(
            ok && text.starts_with("OK. Turn it on: chora-assistant --use-hud mine"),
            "{text}"
        );
    }

    #[test]
    fn four_problems_are_named_with_their_files() {
        let (text, ok) = check_report(&fixture("fancy"));
        assert!(!ok);
        for file in ["link.svg", "href.svg", "huge.svg", "hud.yaml: chat.size"] {
            assert!(text.contains(file), "{file} in {text}");
        }
        assert!(!text.contains("root:x:0"), "{text}");
        assert!(text.contains("4 problems"), "{text}");
    }

    #[test]
    fn a_chat_that_is_hard_to_read_is_a_problem() {
        let dir = temp("contrast");
        std::fs::write(
            dir.join("hud.yaml"),
            "hud: 1\nchat:\n  fill: [250, 250, 250]\n  text: [240, 240, 240]\n",
        )
        .unwrap();
        let found = problems(&dir);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].contains("chat.text") && found[0].contains("4.5:1"),
            "{found:?}"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_colours_of_the_host_chat_read_well() {
        assert_eq!(
            host_chat().contrast_errors(&host_chat()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn use_hud_writes_the_setting_and_default_takes_it_out() {
        let dir = temp("use");
        let roots = vec![fixture("")];
        let file = dir.join("chora-assistant.yaml");
        use_hud("mine", &roots, file.clone()).unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.lines().any(|l| l.trim() == "hud: mine"), "{text}");
        use_hud("default", &roots, file.clone()).unwrap();
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(!text.contains("hud: mine"), "{text}");
        let err = use_hud("nothing", &roots, file.clone()).unwrap_err();
        assert!(err.contains("--list-huds"), "{err}");
        let err = use_hud("fancy", &roots, file).unwrap_err();
        assert!(err.contains("chat.size"), "{err}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn links_inside_a_picture_pass_and_others_are_named() {
        let svg = r##"<use href="#a"/><image href="data:image/png;base64,AAAA"/><image xlink:href="x.png"/>
            <image href="/etc/passwd"/><image href='../up.png'/><image href="file:///x"/>"##;
        assert_eq!(
            outside_links(svg),
            ["/etc/passwd", "../up.png", "file:///x"]
        );
    }

    #[test]
    fn pictures_are_found_in_a_file_that_does_not_parse() {
        let text = "panels:\n- picture: a.svg#{state}\n- {id: b, picture: 'b.svg#x', source: voice}\n- picture: \"c.svg\"\n";
        assert_eq!(picture_names_in(text), ["a.svg", "b.svg", "c.svg"]);
    }

    #[test]
    fn the_list_names_each_folder_once() {
        let text = list_report(&[fixture(""), neutral().parent().unwrap().to_path_buf()]);
        for name in ["fancy", "mine", "neutral"] {
            assert_eq!(
                text.lines()
                    .filter(|l| l.split_whitespace().next() == Some(name))
                    .count(),
                1,
                "{text}"
            );
        }
    }
}
