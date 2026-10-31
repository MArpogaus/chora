#extension GL_OES_standard_derivatives : enable
// A pane of chora-construct: the live window through its node texture,
// faded into the fog with its distance. The focused pane has an accent
// hairline along its edge (docs/construct-style.md, rule 16).
precision highp float;

// Texture pixels per screen pixel: the screen derivative where the driver
// has it, else 1 (a pane at 1:1, which the walk leaves nearly alone).
float texels(vec2 p) {
#ifdef GL_OES_standard_derivatives
    return max(max(fwidth(p.x), fwidth(p.y)), 1e-3);
#else
    return 1.0;
#endif
}

uniform sampler2D win;
uniform float alpha;
uniform float haze;   // 0 near, up to 1 far
uniform vec3 fog;
uniform vec2 size;    // of this node's texture, in pixels
uniform vec4 edge;    // the accent, and the width of the hairline in
                      // pixels of the texture; 0: none
varying vec2 v_uv;

void main() {
    vec4 c = texture2D(win, v_uv);
    vec2 p = v_uv * size;
    float w = texels(p);
    float d = min(min(p.x, p.y), min(size.x - p.x, size.y - p.y));
    c = vec4(mix(c.rgb, fog * c.a, haze), c.a);
    if (edge.w > 0.0) {
        float line = 0.9 * clamp((edge.w - d) / w + 0.5, 0.0, 1.0);
        c = c * (1.0 - line) + vec4(edge.rgb * line, line);
    }
    // The outer edge fades over 1.25 screen pixels: no jagged rim.
    gl_FragColor = c * alpha * smoothstep(0.0, 1.25 * w, d);
}
