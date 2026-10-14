// SPDX-License-Identifier: GPL-3.0-or-later
// An effect of chora: blurs a layer with a 5x5 box.
// Uniform: radius, in pixels of the layer texture.
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
uniform vec2 size;
uniform float radius;
varying vec2 v_uv;

void main() {
    vec2 step = radius / size * 0.5;
    vec4 sum = vec4(0.0);
    for (int x = -2; x <= 2; x++) {
        for (int y = -2; y <= 2; y++) {
            sum += texture2D(tex, v_uv + vec2(float(x), float(y)) * step);
        }
    }
    gl_FragColor = sum / 25.0 * alpha;
}
