// The border of a window of chora-tiling: a ring of the width `width`
// inside the rounded box of the node, in a gradient at 135 degrees from
// `from` at the top left to `to` at the bottom right. Equal colours give
// a solid border. The window sits inside the ring, so nothing of it is
// covered. `radius` and `width` are in logical pixels, `logical` is the logical size
// of the node, so that a scaled output draws the same border.
precision highp float;
uniform float alpha;
uniform vec2 size;
uniform vec4 from;
uniform vec4 to;
uniform float radius;
uniform float width;
uniform vec2 logical;
varying vec2 v_uv;

float box(vec2 p, vec2 half_size, float r) {
    vec2 q = abs(p) - half_size + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    float k = size.x / logical.x;
    vec2 p = (v_uv - 0.5) * size;
    float d = box(p, size * 0.5, radius * k);
    float outer = clamp(0.5 - d, 0.0, 1.0);
    float inner = clamp(0.5 + d + width * k, 0.0, 1.0);
    float cover = outer * inner;
    float t = clamp((p.x + p.y) / (size.x + size.y) + 0.5, 0.0, 1.0);
    vec4 color = mix(from, to, t);
    gl_FragColor = vec4(color.rgb * color.a * cover, color.a * cover) * alpha;
}
