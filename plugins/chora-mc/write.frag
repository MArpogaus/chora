// SPDX-License-Identifier: GPL-3.0-or-later
// An effect of chora-mc: text writes itself, as Write of manim. A soft
// front draws the outlines line by line, and the fill follows behind.
// `progress` runs from 0 to 1; at 1 the layer shows as it is.
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
uniform vec2 size;
uniform float progress;
// The top of the first line under the title and the height of a line,
// both from 0 to 1 of the layer, and the number of rows with the title.
uniform vec3 rows;
varying vec2 v_uv;

// The width of the front, in lines.
const float SOFT = 0.35;

// How far a front at `f` from 0 to 1 has come at the place `s`.
float reveal(float f, float s) {
    return clamp((f * (rows.z + SOFT) - s) / SOFT, 0.0, 1.0);
}

void main() {
    vec4 color = texture2D(tex, v_uv);
    if (progress >= 1.0) {
        gl_FragColor = color * alpha;
        return;
    }
    // The title is row 0, the lines are the rows below it; a line wraps
    // into the last row.
    float row = v_uv.y < rows.x ? 0.0 : 1.0 + floor((v_uv.y - rows.x) / rows.y);
    float s = min(row, rows.z - 1.0) + v_uv.x;
    // The outline: the strongest neighbour where the pixel itself is weak.
    vec2 px = 1.25 / size;
    vec4 m = color;
    vec4 n;
    n = texture2D(tex, v_uv + vec2(px.x, 0.0)); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv - vec2(px.x, 0.0)); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv + vec2(0.0, px.y)); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv - vec2(0.0, px.y)); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv + px); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv - px); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv + vec2(px.x, -px.y)); if (n.a > m.a) m = n;
    n = texture2D(tex, v_uv - vec2(px.x, -px.y)); if (n.a > m.a) m = n;
    float edge = clamp(m.a - color.a, 0.0, 1.0);
    // Straight from premultiplied, so that the line has the text color.
    vec3 ink = m.rgb / max(m.a, 0.001);
    float drawn = reveal(clamp(progress / 0.6, 0.0, 1.0), s);
    float fill = reveal(clamp((progress - 0.35) / 0.65, 0.0, 1.0), s);
    vec4 outline = vec4(ink, 1.0) * edge * drawn * (1.0 - fill);
    gl_FragColor = (color * fill + outline) * alpha;
}
