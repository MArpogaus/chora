// The shadow of a window of chora-tiling: the rounded box `box` in the
// middle of the node, blurred as a Gaussian of the width `blur` (sigma half
// of it) and drawn in `color`. The node is larger than the box by 1.5 blur on
// every side, so that the fall-off ends inside it. `radius`, `blur` and
// `box` are in logical pixels and `logical` is the logical size of the
// node, so that a scaled output draws the same shadow.
precision highp float;
uniform float alpha;
uniform vec2 size;
uniform vec4 color;
uniform float radius;
uniform float blur;
uniform vec2 box;
uniform vec2 logical;
varying vec2 v_uv;

float rounded(vec2 p, vec2 half_size, float r) {
    vec2 q = abs(p) - half_size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    float k = size.x / logical.x;
    vec2 p = (v_uv - 0.5) * size;
    float d = rounded(p, box * 0.5 * k, radius * k);
    float sigma = blur * 0.5 * k;
    float cover = 1.0 / (1.0 + exp(d / (sigma * 0.59)));
    float a = color.a * cover;
    gl_FragColor = vec4(color.rgb * a, a) * alpha;
}
