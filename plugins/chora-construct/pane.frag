// A pane of chora-construct: the live window through its node texture,
// faded into the fog with its distance. The focused pane has an accent
// hairline along its edge (docs/construct-style.md, rule 16).
precision mediump float;
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
    c = vec4(mix(c.rgb, fog * c.a, haze), c.a);
    if (edge.w > 0.0) {
        vec2 p = v_uv * size;
        float d = min(min(p.x, p.y), min(size.x - p.x, size.y - p.y));
        float line = 0.9 * clamp(edge.w - d + 0.5, 0.0, 1.0);
        c = c * (1.0 - line) + vec4(edge.rgb * line, line);
    }
    gl_FragColor = c * alpha;
}
