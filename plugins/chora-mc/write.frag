// SPDX-License-Identifier: GPL-3.0-or-later
// An effect of chora-mc: text writes itself, as Write of manim. A front
// traces a hairline on the inner edge of the glyphs, line by line, and
// the fill fades in behind it from left to right.
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

// The width of the soft edge of the tracing front, and of the gradient
// of the fill, in lines.
const float TRACE = 0.03;
const float FILL = 0.6;
// The part of the progress that the tracing takes, and the start of the
// fill.
const float TRACE_END = 0.7;
const float FILL_START = 0.2;
// The strength of the hairline: a little under the fill, so that thin
// strokes, all hairline, still show the fill arrive.
const float LINE = 0.75;

// How far a front at `f` from 0 to 1 has come at the place `s`, with an
// edge `soft` wide.
float reveal(float f, float s, float soft) {
    return clamp((f * (rows.z + soft) - s) / soft, 0.0, 1.0);
}

void main() {
    vec4 color = texture2D(tex, v_uv);
    if (progress >= 1.0) {
        gl_FragColor = color * alpha;
        return;
    }
    // The title is row 0, the lines are the rows below it; a line wraps
    // into the last row.
    float title = v_uv.y < rows.x ? 1.0 : 0.0;
    float row = title > 0.5 ? 0.0 : 1.0 + floor((v_uv.y - rows.x) / rows.y);
    float s = min(row, rows.z - 1.0) + v_uv.x;
    // The hairline: the pixel itself, where a neighbour one texel away is
    // empty, so the line lies inside the edge of the glyph.
    vec2 px = 1.0 / size;
    float low = min(
        min(texture2D(tex, v_uv + vec2(px.x, 0.0)).a, texture2D(tex, v_uv - vec2(px.x, 0.0)).a),
        min(texture2D(tex, v_uv + vec2(0.0, px.y)).a, texture2D(tex, v_uv - vec2(0.0, px.y)).a));
    float line = color.a * (1.0 - low);
    // A pseudo path, so that strokes trace rather than wipe: cells about
    // the width of a glyph, each traced around its center. Height and
    // width of a cell in texels.
    float h = (title > 0.5 ? rows.x : rows.y) * size.y;
    float cell = 0.5 * h / size.x;
    float top = title > 0.5 ? 0.0 : rows.x + (row - 1.0) * rows.y;
    vec2 d = vec2((fract(v_uv.x / cell) - 0.5) * cell * size.x,
                  (v_uv.y - top) * size.y - 0.55 * h);
    float turn = atan(d.y, d.x) / 6.2832 + 0.5;
    float traced = reveal(clamp(progress / TRACE_END, 0.0, 1.0), s + cell * (turn - 0.5), TRACE);
    float fill = reveal(clamp((progress - FILL_START) / (1.0 - FILL_START), 0.0, 1.0), s, FILL);
    // Straight from premultiplied, so that the line has the text color.
    vec3 ink = color.rgb / max(color.a, 0.001);
    vec4 outline = vec4(ink, 1.0) * (LINE * line * traced);
    gl_FragColor = (color * fill + outline * (1.0 - fill)) * alpha;
}
