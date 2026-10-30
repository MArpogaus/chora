// SPDX-License-Identifier: GPL-3.0-or-later
// An effect of chora-mc: a soft blur of a panel, for context that steps
// back. Two rings of samples around each pixel; `radius` in pixels.
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
uniform vec2 size;
uniform float radius;
varying vec2 v_uv;

void main() {
    vec4 sum = texture2D(tex, v_uv) * 2.0;
    float weight = 2.0;
    for (int i = 0; i < 12; i++) {
        float a = float(i) * 0.5236;
        vec2 d = vec2(cos(a), sin(a)) * radius / size;
        sum += texture2D(tex, v_uv + d) + texture2D(tex, v_uv + d * 0.5) * 1.5;
        weight += 2.5;
    }
    gl_FragColor = sum / weight * alpha;
}
