// SPDX-License-Identifier: GPL-3.0-or-later
// An effect of chora: shows a layer in gray.
precision mediump float;
uniform sampler2D tex;
uniform float alpha;
varying vec2 v_uv;

void main() {
    vec4 color = texture2D(tex, v_uv);
    float gray = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    gl_FragColor = vec4(vec3(gray), color.a) * alpha;
}
