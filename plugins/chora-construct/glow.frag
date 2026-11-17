// The focus glow of chora-construct: the soft accent glow around the
// focused pane, as glass.frag draws it around a panel, without the ring
// and the waveform. It reads no time, so a still desktop draws nothing.
precision highp float;
uniform float alpha;
uniform vec4 accent;  // the accent, and the alpha of the glow
uniform vec4 box;     // the pane: width, height, corner radius, glow reach
varying vec2 v_uv;

// The signed distance to a rounded rect of half size `h` at the origin.
float rounded(vec2 p, vec2 h, float r) {
    vec2 q = abs(p) - h + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    float reach = box.w;
    vec2 p = v_uv * (box.xy + 2.0 * reach) - reach;
    vec4 c = vec4(0.0);
    float d = rounded(p - box.xy * 0.5, box.xy * 0.5, box.z);
    if (reach > 0.0 && d > 0.0) {
        float k = 1.0 - clamp(d / reach, 0.0, 1.0);
        float a = accent.a * k * k;
        c = vec4(accent.rgb * a, a);
    }
    gl_FragColor = c * alpha;
}
