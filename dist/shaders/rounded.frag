// SPDX-License-Identifier: GPL-3.0-or-later
// An effect of chora: rounds the corners of a layer.
// Uniform: radius, in pixels of the layer texture.
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
uniform vec2 size;
uniform float radius;
varying vec2 v_uv;

void main() {
    vec2 half_size = size * 0.5;
    vec2 corner = abs(v_uv * size - half_size) - (half_size - vec2(radius));
    float distance = length(max(corner, 0.0)) - radius;
    float inside = clamp(0.5 - distance, 0.0, 1.0);
    gl_FragColor = texture2D(tex, v_uv) * inside * alpha;
}
