//! The glass of the construct HUD (docs/construct-style.md, rules 18 to
//! 25): the tokens of the material, the type and the space that the
//! construct and the host `chora-assistant` share, so that a surface of
//! either looks the same. Colours are RGBA, each from 0 to 1.
//!
//! The core has no backdrop blur yet. [`BACKDROP`] is the one switch: while
//! it is off, every surface takes the fallback of the spec, the strong tint
//! raised by [`FALLBACK_LIFT`], and nothing asks the core for a blur.

/// Whether the core blurs what lies behind a surface. Turn it on, and fill
/// [`Material::backdrop`], when the node prop `backdrop` lands in the core.
pub const BACKDROP: bool = false;

/// What the tint gains without a blur behind it.
pub const FALLBACK_LIFT: f64 = 0.08;

/// The grid: every gap is a whole number of units.
pub const UNIT: f64 = 8.0;
/// The margin of a screen-fixed surface from the edges of the output.
pub const MARGIN: f64 = 32.0;
/// The margin on an output below 1024 wide.
pub const NARROW_MARGIN: f64 = 24.0;

/// The radius of panels and cards.
pub const PANEL_RADIUS: f64 = 24.0;
/// The radius of rows, fields and segments: concentric with an inset of 8.
pub const INNER_RADIUS: f64 = 16.0;
/// The radius of the detail well.
pub const WELL_RADIUS: f64 = 12.0;
/// The radius of a keycap.
pub const KEY_RADIUS: f64 = 6.0;

/// The height of the island, the hints and a notice.
pub const CAPSULE: f64 = 40.0;
/// The height of the pane label and the cube face label.
pub const LABEL_CAPSULE: f64 = 32.0;
/// The height of a row of the window list.
pub const ROW: f64 = 56.0;

/// The height and the least width of a keycap.
pub const KEY_HEIGHT: f64 = 24.0;
/// The room at the sides of the name in a keycap.
pub const KEY_PAD: f64 = 7.0;
/// The gap between the caps of a chord.
pub const KEY_GAP: f64 = 4.0;

/// The width of the focus ring in the accent, outside the surface.
pub const RING_WIDTH: f64 = 2.0;
/// The width of the halo around the focus ring.
pub const HALO_WIDTH: f64 = 4.0;
/// The opacity of the accent in the halo.
pub const HALO_ALPHA: f64 = 0.22;

/// The fade in of a surface, in milliseconds.
pub const APPEAR_MS: f64 = 160.0;
/// The fade out of a surface, in milliseconds.
pub const DISAPPEAR_MS: f64 = 120.0;
/// The morph of the island into the chat and back, in milliseconds.
pub const MORPH_MS: f64 = 240.0;

/// The family of labels and text.
pub const SANS: &str = "Inter";
/// The family of data.
pub const MONO: &str = "DejaVu Sans Mono";

/// The four sizes and their roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    /// 11/16, 500, capitals, +0.08 em: state words, heads, scope.
    Label,
    /// 14/20, 400 or 500: key names, the pane label, notes, key lines.
    Small,
    /// `DejaVu Sans Mono` 14/20: paths, details, tool steps, row notes.
    Data,
    /// 16/24, 400: sentences, messages, prompts, row names.
    Text,
    /// 24/32, 300, -0.01 em: the search line of the list and the help.
    Display,
}

/// The size, the line, the weight and the letter spacing of a role.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Face {
    /// The size in logical pixels.
    pub size: f64,
    /// The height of a line.
    pub line: f64,
    /// The weight, 300 to 500.
    pub weight: u16,
    /// The letter spacing in em.
    pub spacing: f64,
    /// In the mono family.
    pub mono: bool,
    /// In capitals.
    pub caps: bool,
}

impl Type {
    /// The face of the role.
    #[must_use]
    pub const fn face(self) -> Face {
        let (size, line, weight, spacing, mono, caps) = match self {
            Self::Label => (11.0, 16.0, 500, 0.08, false, true),
            Self::Small => (14.0, 20.0, 400, 0.0, false, false),
            Self::Data => (14.0, 20.0, 400, 0.0, true, false),
            Self::Text => (16.0, 24.0, 400, 0.0, false, false),
            Self::Display => (24.0, 32.0, 300, -0.01, false, false),
        };
        Face {
            size,
            line,
            weight,
            spacing,
            mono,
            caps,
        }
    }
}

/// A layer of the shadow: its offset down, its blur and its opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    /// The offset down.
    pub y: f64,
    /// The blur radius, twice the deviation.
    pub blur: f64,
    /// The opacity of the colour of the shadow.
    pub alpha: f64,
}

/// What the core blurs behind a surface, once it can.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Backdrop {
    /// The blur radius.
    pub blur: f64,
    /// The saturation of what shows through.
    pub saturation: f64,
}

/// The tokens of one theme.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// The base tint.
    pub tint: [f64; 4],
    /// The strong tint of the text-heavy surfaces.
    pub strong: [f64; 4],
    /// The outer hairline.
    pub edge_out: [f64; 4],
    /// The inner ring.
    pub edge_in: [f64; 4],
    /// The line along the top edge inside.
    pub highlight: [f64; 4],
    /// The colour of the shadow.
    pub shadow_rgb: [f64; 3],
    /// The three layers of the shadow.
    pub shadows: [Shadow; 3],
    /// The text.
    pub text: [f64; 4],
    /// The muted text.
    pub muted: [f64; 4],
    /// The fill under the pointer.
    pub hover: [f64; 4],
    /// The fill of a chosen row.
    pub select: [f64; 4],
    /// The fill of a field, a detail and a segment.
    pub well: [f64; 4],
    /// The fill of a keycap.
    pub cap: [f64; 4],
    /// The ring of a keycap.
    pub cap_edge: [f64; 4],
    /// The line at the bottom of a keycap.
    pub cap_low: [f64; 4],
    /// The scrim behind a command surface.
    pub scrim: [f64; 4],
    /// The dividers and the separators of hints.
    pub separator: [f64; 4],
    /// The mark behind a found word.
    pub mark: [f64; 4],
    /// The bubble of a message of the person.
    pub bubble: [f64; 4],
    /// The accent.
    pub accent: [f64; 4],
    /// The saturation of the blur behind.
    pub saturation: f64,
}

/// The accent, the one hue: rings, focus, selection, caret, lit edges.
pub const ACCENT: [f64; 3] = [rgb(0x26), rgb(0x80), rgb(0xB4)];

const fn rgb(v: u8) -> f64 {
    v as f64 / 255.0
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "a byte of a colour of six hex digits"
)]
const fn hex(v: u32, alpha: f64) -> [f64; 4] {
    [
        rgb((v >> 16) as u8),
        rgb((v >> 8) as u8),
        rgb(v as u8),
        alpha,
    ]
}

const fn accent(alpha: f64) -> [f64; 4] {
    [ACCENT[0], ACCENT[1], ACCENT[2], alpha]
}

const LIGHT: Material = Material {
    tint: hex(0xF8_F9FB, 0.64),
    strong: hex(0xF8_F9FB, 0.82),
    edge_out: hex(0x10_141C, 0.10),
    edge_in: hex(0xFF_FFFF, 0.55),
    highlight: hex(0xFF_FFFF, 0.90),
    shadow_rgb: [rgb(0x10), rgb(0x14), rgb(0x1C)],
    shadows: [
        Shadow {
            y: 1.0,
            blur: 2.0,
            alpha: 0.06,
        },
        Shadow {
            y: 8.0,
            blur: 24.0,
            alpha: 0.08,
        },
        Shadow {
            y: 24.0,
            blur: 64.0,
            alpha: 0.12,
        },
    ],
    text: hex(0x14_181E, 1.0),
    muted: hex(0x4A_5260, 1.0),
    hover: hex(0x14_181E, 0.05),
    select: accent(0.16),
    well: hex(0x14_181E, 0.05),
    cap: hex(0xFF_FFFF, 0.72),
    cap_edge: hex(0x10_141C, 0.12),
    cap_low: hex(0x10_141C, 0.10),
    scrim: hex(0xEC_EFF4, 0.40),
    separator: hex(0x10_141C, 0.08),
    mark: accent(0.24),
    bubble: accent(0.16),
    accent: accent(1.0),
    saturation: 1.8,
};

const DARK: Material = Material {
    tint: hex(0x18_1A20, 0.56),
    strong: hex(0x18_1A20, 0.84),
    edge_out: hex(0x00_0000, 0.60),
    edge_in: hex(0xFF_FFFF, 0.10),
    highlight: hex(0xFF_FFFF, 0.16),
    shadow_rgb: [0.0, 0.0, 0.0],
    shadows: [
        Shadow {
            y: 1.0,
            blur: 2.0,
            alpha: 0.30,
        },
        Shadow {
            y: 8.0,
            blur: 24.0,
            alpha: 0.30,
        },
        Shadow {
            y: 24.0,
            blur: 64.0,
            alpha: 0.50,
        },
    ],
    text: hex(0xEC_EEF2, 1.0),
    muted: hex(0xA3_AAB5, 1.0),
    hover: hex(0xFF_FFFF, 0.07),
    select: accent(0.30),
    well: hex(0x00_0000, 0.22),
    cap: hex(0xFF_FFFF, 0.10),
    cap_edge: hex(0xFF_FFFF, 0.12),
    cap_low: hex(0x00_0000, 0.40),
    scrim: hex(0x00_0000, 0.36),
    separator: hex(0xFF_FFFF, 0.08),
    mark: accent(0.45),
    bubble: accent(0.34),
    accent: accent(1.0),
    saturation: 1.6,
};

impl Material {
    /// The tokens of the light or the dark theme.
    #[must_use]
    pub const fn of(dark: bool) -> Self {
        if dark { DARK } else { LIGHT }
    }

    /// The tint of a surface: `strong` for the text-heavy ones (the
    /// question, the chat, the window list, the help). Without a blur
    /// every surface takes the strong tint, raised by [`FALLBACK_LIFT`].
    #[must_use]
    pub fn fill(&self, strong: bool) -> [f64; 4] {
        let [r, g, b, a] = if strong || !BACKDROP {
            self.strong
        } else {
            self.tint
        };
        if BACKDROP {
            [r, g, b, a]
        } else {
            [r, g, b, (a + FALLBACK_LIFT).min(1.0)]
        }
    }

    /// The blur that a surface asks of the core, none while the core has
    /// no backdrop.
    #[must_use]
    pub fn backdrop(&self) -> Option<Backdrop> {
        BACKDROP.then_some(Backdrop {
            blur: 40.0,
            saturation: self.saturation,
        })
    }
}

/// The margin of a screen-fixed surface on an output `width` wide.
#[must_use]
pub fn margin(width: f64) -> f64 {
    if width < 1024.0 {
        NARROW_MARGIN
    } else {
        MARGIN
    }
}

/// A keycap shows a key as a person reads it: a single letter in capitals,
/// the arrows as arrows and `Escape` as `Esc`.
#[must_use]
pub fn cap(key: &str) -> String {
    match key {
        "Escape" => "Esc".into(),
        "Up" => "↑".into(),
        "Down" => "↓".into(),
        "Left" => "←".into(),
        "Right" => "→".into(),
        "period" => ".".into(),
        "slash" => "/".into(),
        "space" => "Space".into(),
        "Return" | "KP_Enter" => "Return".into(),
        key if key.chars().count() == 1 => key.to_uppercase(),
        key => {
            let mut chars = key.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        }
    }
}

/// The caps of a chord such as `Super+Alt+1`, one cap per key.
#[must_use]
pub fn caps(chord: &str) -> Vec<String> {
    chord
        .split('+')
        .filter(|k| !k.is_empty())
        .map(cap)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_blur_every_surface_takes_the_strong_tint_and_more() {
        const { assert!(!BACKDROP, "the core has no backdrop yet") };
        let light = Material::of(false);
        let dark = Material::of(true);
        for strong in [false, true] {
            assert!((light.fill(strong)[3] - 0.90).abs() < 1e-9);
            assert!((dark.fill(strong)[3] - 0.92).abs() < 1e-9);
        }
        assert_eq!(light.backdrop(), None);
    }

    #[test]
    fn the_tokens_are_those_of_the_spec() {
        let light = Material::of(false);
        assert_eq!(light.text, hex(0x14_181E, 1.0));
        assert!((light.text[0] - 20.0 / 255.0).abs() < 1e-9);
        assert_eq!(Material::of(true).muted, hex(0xA3_AAB5, 1.0));
        assert_eq!(light.select, [ACCENT[0], ACCENT[1], ACCENT[2], 0.16]);
        assert_eq!(Material::of(true).select[3], 0.30);
    }

    #[test]
    fn the_four_sizes_are_eleven_fourteen_sixteen_and_twenty_four() {
        let mut sizes: Vec<f64> = [
            Type::Label,
            Type::Small,
            Type::Data,
            Type::Text,
            Type::Display,
        ]
        .iter()
        .map(|t| t.face().size)
        .collect();
        sizes.sort_by(f64::total_cmp);
        sizes.dedup();
        assert_eq!(sizes, [11.0, 14.0, 16.0, 24.0]);
        assert!(Type::Label.face().caps);
        assert!(Type::Data.face().mono);
    }

    #[test]
    fn a_chord_shows_as_caps_side_by_side() {
        assert_eq!(caps("Super+Alt+1"), ["Super", "Alt", "1"]);
        assert_eq!(caps("Alt+y"), ["Alt", "Y"]);
        assert_eq!(caps("Ctrl+period"), ["Ctrl", "."]);
        assert_eq!(caps("Alt+Escape"), ["Alt", "Esc"]);
        assert_eq!(caps("super+slash"), ["Super", "/"]);
        assert_eq!(margin(800.0), 24.0);
        assert_eq!(margin(1280.0), 32.0);
    }
}
