// The shadow of a glass surface of chora-construct (construct HUD spec,
// section 2): three soft layers under a rounded rect, as a CSS box shadow,
// drawn only outside the surface. The node is the surface plus the reach
// on each side. It reads no time, so a still desktop draws nothing.
precision highp float;
uniform float alpha;
uniform vec4 box;    // the surface: width, height, corner radius, reach
uniform vec3 shade;  // the colour of the shadow
uniform vec4 near;   // a layer: offset down, deviation, opacity
uniform vec4 mid;
uniform vec4 far;
varying vec2 v_uv;

// The signed distance to a rounded rect of half size `h` at the origin.
float rounded(vec2 p, vec2 h, float r) {
    vec2 q = abs(p) - h + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

// The error function, to within 5e-4.
float erf(float x) {
    float s = sign(x);
    float a = abs(x);
    float t = 1.0 + (0.278393 + (0.230389 + 0.078108 * a * a) * a) * a;
    t *= t;
    return s - s / (t * t);
}

// The cover of a blurred rounded rect at `p`, moved down by `layer.x`.
float layer(vec2 p, vec2 half_size, float r, vec4 l) {
    float d = rounded(p - vec2(0.0, l.x), half_size, r);
    return l.z * 0.5 * (1.0 - erf(d / (1.41421356 * max(l.y, 0.5))));
}

void main() {
    float reach = box.w;
    vec2 p = v_uv * (box.xy + 2.0 * reach) - reach - box.xy * 0.5;
    vec2 half_size = box.xy * 0.5;
    float inside = rounded(p, half_size, box.z);
    float a = 0.0;
    if (inside > 0.0) {
        float n = layer(p, half_size, box.z, near);
        float m = layer(p, half_size, box.z, mid);
        float f = layer(p, half_size, box.z, far);
        a = 1.0 - (1.0 - n) * (1.0 - m) * (1.0 - f);
    }
    gl_FragColor = vec4(shade * a, a) * alpha;
}
