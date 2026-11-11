//! The glass of the construct HUD as SVG for the pictures of the host:
//! the surface, its shadow and hairline, keycaps and text in the four
//! sizes, from the tokens of `chora_kit::glass`. A folder with a glass
//! chat draws its island, card and chat with it.
#![allow(
    clippy::many_single_char_names,
    reason = "x, y, w, h and r of shapes, as SVG names them"
)]

use std::fmt::Write as _;

use chora_draw::{
    text::escape,
    theme::{Picture, Theme},
};
use chora_kit::glass::{KEY_GAP, KEY_HEIGHT, KEY_PAD, KEY_RADIUS, MONO, Material, SANS, Type};

/// How far the shadow and the hairline reach out of a surface; the
/// picture of the shadow is this much larger on each side.
pub const REACH: i32 = 120;

/// The share of the size of Inter and of `DejaVu Sans Mono` above the
/// baseline and in the whole line box, for the baseline of a line.
const SANS_METRICS: (f64, f64) = (0.968_75, 1.210_937_5);
const MONO_METRICS: (f64, f64) = (0.928_222_7, 1.164_062_5);

/// An SVG paint of a colour: `fill="rgb(..)" fill-opacity=".."`.
#[must_use]
pub fn paint(attr: &str, [r, g, b, a]: [f64; 4]) -> String {
    let byte = |v: f64| (v * 255.0).round().clamp(0.0, 255.0);
    format!(
        r#"{attr}="rgb({},{},{})" {attr}-opacity="{a:.3}""#,
        byte(r),
        byte(g),
        byte(b)
    )
}

/// The path of a rounded rect.
fn rounded(x: f64, y: f64, w: f64, h: f64, r: f64) -> String {
    let r = r.min(w / 2.0).min(h / 2.0);
    format!(
        "M{l} {y}H{rr}A{r} {r} 0 0 1 {xr} {t}V{bb}A{r} {r} 0 0 1 {rr} {yb}H{l}A{r} {r} 0 0 1 {x} {bb}V{t}A{r} {r} 0 0 1 {l} {y}Z",
        l = x + r,
        rr = x + w - r,
        xr = x + w,
        t = y + r,
        bb = y + h - r,
        yb = y + h,
    )
}

/// A rounded rect in a paint.
#[must_use]
pub fn rect([x, y, w, h]: [f64; 4], r: f64, fill: [f64; 4]) -> String {
    format!(
        r#"<path d="{}" {}/>"#,
        rounded(x, y, w, h, r),
        paint("fill", fill)
    )
}

/// The glass of a surface of `size` with corners of `radius`: the tint,
/// the inner ring and the highlight along the top edge inside.
#[must_use]
pub fn surface([w, h]: [f64; 2], radius: f64, m: &Material, strong: bool) -> String {
    let shape = rounded(0.0, 0.0, w, h, radius);
    let lower = rounded(0.0, 1.0, w, h, radius);
    let ring = rounded(0.5, 0.5, w - 1.0, h - 1.0, (radius - 0.5).max(0.0));
    format!(
        r#"<defs><clipPath id="g"><path d="{shape}"/></clipPath></defs><path d="{shape}" {}/><path d="{ring}" fill="none" stroke-width="1" {}/><path d="{shape}{lower}" fill-rule="evenodd" clip-path="url(#g)" {}/>"#,
        paint("fill", m.fill(strong)),
        paint("stroke", m.edge_in),
        paint("fill", m.highlight),
    )
}

/// The shadow and the outer hairline of a surface of `size`, drawn
/// outside it only, in a picture [`REACH`] larger on each side. `ask`
/// adds the accent hairline and glow of a waiting question.
#[must_use]
pub fn shadow(theme: &mut Theme, [w, h]: [f64; 2], radius: f64, m: &Material, ask: bool) -> String {
    let reach = f64::from(REACH);
    let (pw, ph) = (w + 2.0 * reach, h + 2.0 * reach);
    let shape = rounded(reach, reach, w, h, radius);
    let mut body = format!(
        r#"<defs><mask id="out" maskUnits="userSpaceOnUse" x="0" y="0" width="{pw}" height="{ph}"><rect width="{pw}" height="{ph}" fill="white"/><path d="{shape}" fill="black"/></mask>"#
    );
    for (k, layer) in m.shadows.iter().enumerate() {
        let _ = write!(
            body,
            r#"<filter id="s{k}" filterUnits="userSpaceOnUse" x="0" y="0" width="{pw}" height="{ph}"><feGaussianBlur stdDeviation="{}"/></filter>"#,
            layer.blur / 2.0
        );
    }
    let _ = write!(
        body,
        r#"<filter id="glow" filterUnits="userSpaceOnUse" x="0" y="0" width="{pw}" height="{ph}"><feGaussianBlur stdDeviation="12"/></filter></defs><g mask="url(#out)">"#
    );
    let [r, g, b] = m.shadow_rgb;
    for (k, layer) in m.shadows.iter().enumerate() {
        let moved = rounded(reach, reach + layer.y, w, h, radius);
        let _ = write!(
            body,
            r#"<path d="{moved}" filter="url(#s{k})" {}/>"#,
            paint("fill", [r, g, b, layer.alpha])
        );
    }
    if ask {
        let [ar, ag, ab, _] = m.accent;
        let _ = write!(
            body,
            r#"<path d="{shape}" filter="url(#glow)" {}/>"#,
            paint("fill", [ar, ag, ab, 0.18])
        );
    }
    let edge = if ask {
        let [ar, ag, ab, _] = m.accent;
        [ar, ag, ab, 0.55]
    } else {
        m.edge_out
    };
    let hairline = rounded(reach - 0.5, reach - 0.5, w + 1.0, h + 1.0, radius + 0.5);
    let _ = write!(
        body,
        r#"<path d="{hairline}" fill="none" stroke-width="1" {}/></g>"#,
        paint("stroke", edge)
    );
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a surface is smaller than a screen"
    )]
    let picture = Picture::new(pw.ceil() as i32, ph.ceil() as i32).svg(body);
    theme.render(&picture)
}

/// A line of text in a role, its box at `[x, top]`, the line height of the
/// role; `weight` overrides the weight of the role, `end` puts the right
/// edge of the text at `x`.
#[derive(Clone, Copy)]
pub struct Line<'a> {
    pub text: &'a str,
    pub role: Type,
    pub weight: Option<u16>,
    pub color: [f64; 4],
}

impl<'a> Line<'a> {
    /// A line of `text` in `role`.
    #[must_use]
    pub fn new(text: &'a str, role: Type, color: [f64; 4]) -> Self {
        Self {
            text,
            role,
            weight: None,
            color,
        }
    }

    /// The same line in another weight.
    #[must_use]
    pub fn weight(mut self, weight: u16) -> Self {
        self.weight = Some(weight);
        self
    }

    fn shown(&self) -> String {
        if self.role.face().caps {
            self.text.to_uppercase()
        } else {
            self.text.to_owned()
        }
    }

    /// The `<text>` element with its line box at `[x, top]`; `anchor_end`
    /// sets its right edge at `x`.
    #[must_use]
    pub fn at(&self, [x, top]: [f64; 2], anchor_end: bool) -> String {
        let face = self.role.face();
        let (family, (ascent, content)) = if face.mono {
            (MONO, MONO_METRICS)
        } else {
            (SANS, SANS_METRICS)
        };
        let baseline = top + (face.line - content * face.size) / 2.0 + ascent * face.size;
        let weight = self.weight.unwrap_or(face.weight);
        let spacing = face.spacing * face.size;
        let anchor = if anchor_end {
            r#" text-anchor="end""#
        } else {
            ""
        };
        format!(
            r#"<text x="{x:.2}" y="{baseline:.2}" font-family="{family}" font-size="{}" font-weight="{weight}" letter-spacing="{spacing:.3}"{anchor} {} xml:space="preserve">{}</text>"#,
            face.size,
            paint("fill", self.color),
            escape(&self.shown())
        )
    }

    /// The width of the line.
    #[must_use]
    pub fn width(&self, theme: &Theme) -> f64 {
        theme.svg_width(&self.at([0.0, 0.0], false))
    }
}

/// Keycaps of the keys of a chord, side by side from `[x, top]`, each 24
/// high: the SVG and the width they take.
#[must_use]
pub fn keycaps(theme: &Theme, keys: &[String], [x, top]: [f64; 2], m: &Material) -> (String, f64) {
    let mut body = String::new();
    let mut at = x;
    for (k, key) in keys.iter().enumerate() {
        if k > 0 {
            at += KEY_GAP;
        }
        let line = Line::new(key, Type::Small, m.text).weight(500);
        let ink = line.width(theme);
        let w = (ink + 2.0 * KEY_PAD).round().max(KEY_HEIGHT);
        body += &rect([at, top, w, KEY_HEIGHT], KEY_RADIUS, m.cap);
        let _ = write!(
            body,
            r#"<path d="{}" fill="none" stroke-width="1" {}/>"#,
            rounded(
                at + 0.5,
                top + 0.5,
                w - 1.0,
                KEY_HEIGHT - 1.0,
                KEY_RADIUS - 0.5
            ),
            paint("stroke", m.cap_edge)
        );
        let _ = write!(
            body,
            r#"<path d="M{} {}H{}" stroke-width="1" {}/>"#,
            at + KEY_RADIUS,
            top + KEY_HEIGHT - 0.5,
            at + w - KEY_RADIUS,
            paint("stroke", m.cap_low)
        );
        body += &line.at([at + (w - ink) / 2.0, top + 2.0], false);
        at += w;
    }
    (body, at - x)
}

/// The width of the keycaps of a chord.
#[must_use]
pub fn keycaps_width(theme: &Theme, keys: &[String], m: &Material) -> f64 {
    keycaps(theme, keys, [0.0, 0.0], m).1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_paint_is_an_rgb_and_an_opacity() {
        assert_eq!(
            paint("fill", [20.0 / 255.0, 24.0 / 255.0, 30.0 / 255.0, 0.05]),
            r#"fill="rgb(20,24,30)" fill-opacity="0.050""#
        );
    }

    #[test]
    fn a_label_is_in_capitals_and_its_baseline_sits_in_its_line() {
        let m = Material::of(false);
        let label = Line::new("Question from Atlas", Type::Label, m.text).at([16.0, 16.0], false);
        assert!(label.contains(">QUESTION FROM ATLAS<"), "{label}");
        assert!(label.contains(r#"font-size="11""#), "{label}");
        assert!(label.contains(r#"y="28.00""#), "{label}");
        let data = Line::new("~/notes", Type::Data, m.text).at([0.0, 0.0], false);
        assert!(data.contains("DejaVu Sans Mono"), "{data}");
    }
}
