//! The launcher side of the xp look: the Run box and the fuzzel theme of
//! a start. Following a started program until its window shows is
//! [`chora_kit::launch`].

pub use chora_kit::launch::{Change, Launches};

/// The launcher command in the look of xp: `fuzzel` with the `file` of
/// the flavor as its configuration, unless the command names one or the
/// file is missing. `fuzzel` alone is the Run box, [`RUN_BOX`] of `exe`
/// with the file and the terminal. Another command stays as it is.
#[must_use]
pub fn themed(command: &str, file: &std::path::Path, exe: &str, terminal: &str) -> String {
    let mut words = command.split_whitespace();
    let named = words.clone().any(|w| w == "--config" || w == "-c");
    if words.next() != Some("fuzzel") || named || !file.is_file() {
        return command.to_owned();
    }
    let rest = command.trim_start().trim_start_matches("fuzzel");
    let file = quoted(&file.to_string_lossy());
    if rest.trim().is_empty() {
        return format!("{} {RUN_BOX} {file} {}", quoted(exe), quoted(terminal));
    }
    format!("fuzzel --config {file}{rest}")
}

/// The argument of chora-xp that shows the Run box.
pub const RUN_BOX: &str = "--run-box";

/// A word in single quotes for the shell.
fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// The Run box, `chora-xp --run-box FILE TERMINAL`: fuzzel with the
/// config file of the flavor lists the installed programs, sorted, and
/// says when text finds nothing, as [`chora_kit::launcher`] does.
pub fn run_box(args: &[String]) {
    let [file, terminal] = args else {
        eprintln!("usage: chora-xp {RUN_BOX} FILE TERMINAL");
        return;
    };
    let fuzzel = ["fuzzel", "--config", file].map(String::from);
    chora_kit::launcher::run(&fuzzel, terminal, None);
}

/// The name of the program of a shell command: its first word without
/// quotes and directory.
#[must_use]
pub fn program_name(command: &str) -> String {
    let word = command.split_whitespace().next().unwrap_or_default();
    let word = word.trim_matches(['\'', '"']);
    word.rsplit('/').next().unwrap_or(word).to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUZZEL_INI: &str = include_str!("../../../flavors/xp/theme/fuzzel.ini");

    #[test]
    fn the_name_is_the_program_without_directory_and_quotes() {
        assert_eq!(program_name("libreoffice --writer"), "libreoffice");
        assert_eq!(program_name("/usr/bin/thunar '/'"), "thunar");
        assert_eq!(program_name("'my app' x"), "my");
        assert_eq!(program_name(""), "");
    }

    #[test]
    fn fuzzel_takes_the_look_of_xp() {
        let dir = chora_kit::testing::TempDir::new("chora-xp-themed");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("fuzzel.ini");
        std::fs::write(&file, "[main]\n").unwrap();
        let themed = |command: &str, file: &std::path::Path| {
            super::themed(command, file, "/opt/it's/chora-xp", "foot")
        };
        let line = themed("fuzzel --dmenu", &file);
        assert_eq!(
            line,
            format!("fuzzel --config '{}' --dmenu", file.display())
        );
        assert_eq!(
            themed("fuzzel --config /x.ini", &file),
            "fuzzel --config /x.ini"
        );
        assert_eq!(themed("rofi -show run", &file), "rofi -show run");
        assert_eq!(themed("fuzzel", &dir.join("none.ini")), "fuzzel");
        assert_eq!(
            themed("fuzzel", &file),
            format!(
                "'/opt/it'\\''s/chora-xp' --run-box '{}' 'foot'",
                file.display()
            )
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_shipped_fuzzel_file_is_xp_and_opaque() {
        let text = FUZZEL_INI;
        assert!(text.contains("font=Tahoma:pixelsize=14"), "{text}");
        assert!(text.contains("prompt=\"Open: \""), "{text}");
        for key in ["background", "text", "selection", "border"] {
            let value = text
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{key}=")))
                .unwrap_or_else(|| panic!("{key}"));
            assert!(value.ends_with("ff"), "{key} is not opaque: {value}");
        }
        assert!(text.contains("selection=316ac5ff"), "{text}");
    }

    /// fuzzel reads `line-height` in points (96 dpi: 1 pt is 4/3 px), the
    /// pads in pixels. The row of the Run box is 28 px (xp-v2.md: the edit
    /// box is 28 px, on the grid of 4), so the line height is 21 pt; 28 pt
    /// made rows of 37.3 px (xp-24).
    #[test]
    fn the_row_of_the_run_box_is_28_px() {
        let text = FUZZEL_INI;
        let value = |key: &str| -> f64 {
            text.lines()
                .find_map(|l| l.strip_prefix(&format!("{key}=")))
                .unwrap_or_else(|| panic!("{key}"))
                .parse()
                .unwrap_or_else(|_| panic!("{key} is not a plain number of points"))
        };
        let row_px = value("line-height") * 96.0 / 72.0;
        assert!(
            (row_px - 28.0).abs() < 1e-9,
            "the row is {row_px} px, not 28"
        );
    }
}
