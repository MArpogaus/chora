// The thumbnail of a window in the window list of chora-construct: its
// picture through its node texture, cut to fill the box like a cover,
// with round corners.
precision mediump float;
uniform sampler2D win;
uniform float alpha;
uniform vec2 size;   // of this node, in pixels
uniform vec4 box;    // width, height, corner radius
uniform vec2 crop;   // the share of the picture that shows, x and y
varying vec2 v_uv;

float rounded(vec2 p, vec2 h, float r) {
    vec2 q = abs(p) - h + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    vec2 uv = 0.5 + (v_uv - 0.5) * crop;
    vec2 p = (v_uv - 0.5) * box.xy;
    float px = box.x / size.x;
    float d = rounded(p, box.xy * 0.5, box.z);
    float cover = clamp(0.5 - d / px, 0.0, 1.0);
    gl_FragColor = texture2D(win, uv) * cover * alpha;
}
