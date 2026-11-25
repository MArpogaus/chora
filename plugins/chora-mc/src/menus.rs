//! The launcher and the power menu in the look of mc: fuzzel with a
//! config file that mc writes from the colors of its theme, so that a
//! dark page has a dark box and a light page a light one (rpx1-10). The
//! launcher is `chora-mc --launch fuzzel --config FILE`, the power menu
//! is `power.sh` with `CHORA_MENU` set to the same fuzzel.

use std::path::Path;

use crate::Mc;

/// The name of the config file in the runtime directory of mc.
const FILE: &str = "fuzzel.ini";

/// The byte of a channel from 0 to 1, rounded.
fn byte(channel: f64) -> u8 {
    (0..=255u8)
        .find(|v| f64::from(*v) + 0.5 > channel * 255.0)
        .unwrap_or(255)
}

/// The color of a theme as the `RRGGBBAA` that fuzzel reads.
fn hex(color: [f64; 4]) -> String {
    let [r, g, b, a] = color.map(byte);
    format!("{r:02x}{g:02x}{b:02x}{a:02x}")
}

/// The config of fuzzel for the colors of a theme: the page for the box,
/// the text color for the rows, a hairline border, the faint color behind
/// the chosen row and the accent for what the input matches.
pub(crate) fn config(colors: &std::collections::BTreeMap<String, [f64; 4]>) -> Option<String> {
    let color = |name: &str| colors.get(name).copied().map(hex);
    let [background, text, muted, faint, accent] =
        ["background", "text", "muted", "faint", "accent"].map(color);
    Some(format!(
        "# The launcher and the power menu of mc, written by chora-mc from its theme.\n\
         [colors]\n\
         background={}\n\
         text={text}\n\
         prompt={muted}\n\
         placeholder={muted}\n\
         input={text}\n\
         match={accent}\n\
         selection={faint}\n\
         selection-text={text}\n\
         selection-match={text}\n\
         counter={muted}\n\
         border={faint}\n\
         \n\
         [border]\n\
         width=2\n\
         radius=2\n",
        background?,
        text = text?,
        muted = muted?,
        faint = faint?,
        accent = accent?
    ))
}

impl Mc {
    /// Writes the config of the menus for the current theme; a start and
    /// a change of the settings call it. Without a runtime directory the
    /// menus stay plain fuzzel.
    pub(crate) fn write_menu_theme(&mut self) {
        let file = config(self.settings.value.colors())
            .and_then(|text| chora_kit::runtime_file("chora-mc", FILE, &text).ok())
            .filter(|path| !path.to_string_lossy().contains(char::is_whitespace));
        self.menu = file;
    }

    /// The command in the look of mc: the launcher `fuzzel` is
    /// `chora-mc --launch fuzzel` with the config, the power script
    /// gets the same fuzzel as its menu; another command stays as it is.
    pub(crate) fn themed(&self, command: &str) -> String {
        let Some(file) = self.menu.as_deref().map(Path::to_string_lossy) else {
            return command.to_owned();
        };
        match command.trim() {
            "fuzzel" => format!("chora-mc --launch fuzzel --config {file}"),
            other if other.contains("power.sh") => {
                format!("CHORA_MENU='fuzzel --config {file}' {other}")
            }
            other => other.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The box of the menus has the background of the theme, the rows its
    /// text color, and a light theme gives a light box (rpx1-10).
    #[test]
    fn the_config_of_the_menus_follows_the_theme() {
        let mc = crate::testing::mc("menus-theme", [1280, 800]).0;
        let dark = config(mc.settings.value.colors()).unwrap();
        assert!(dark.contains("background=1c1c1cff"), "{dark}");
        assert!(dark.contains("text=d9d4c9ff"), "{dark}");
        let light = config(&mc.settings.value.themes.light).unwrap();
        assert!(light.contains("background=f7f5edff"), "{light}");
    }

    /// The launcher and the power script get the fuzzel of mc, another
    /// command stays as it is, and without a config nothing changes.
    #[test]
    fn the_commands_get_the_config_of_the_menus() {
        let mut mc = crate::testing::mc("menus-command", [1280, 800]).0;
        mc.menu = Some("/run/chora-mc/fuzzel.ini".into());
        assert_eq!(
            mc.themed("fuzzel"),
            "chora-mc --launch fuzzel --config /run/chora-mc/fuzzel.ini"
        );
        assert_eq!(
            mc.themed("sh '/s/power.sh'"),
            "CHORA_MENU='fuzzel --config /run/chora-mc/fuzzel.ini' sh '/s/power.sh'"
        );
        assert_eq!(mc.themed("foot"), "foot");
        mc.menu = None;
        assert_eq!(mc.themed("fuzzel"), "fuzzel");
    }
}
