#extension GL_OES_standard_derivatives : enable
// The mirror image of a pane in a glossy floor: its node texture at a
// reduced size, a little softer the deeper it lies, faded out downwards.
// The transform flips it; the gloss carries the fresnel and the
// distance.
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
uniform vec2 size;    // of this node's texture, in pixels
uniform float gloss;  // the strength at the pane's bottom edge
varying vec2 v_uv;

void main() {
    vec2 p = v_uv * size;
    float w = texels(p);
    float d = min(min(p.x, p.y), min(size.x - p.x, size.y - p.y));
    float k = v_uv.y;   // 1 at the pane's bottom edge, nearest the ground
    vec2 o = vec2(0.0, 1.5 * (1.0 - k)) / size;
    vec4 c = 0.5 * texture2D(win, v_uv)
           + 0.25 * (texture2D(win, v_uv + o) + texture2D(win, v_uv - o));
    float soft = smoothstep(0.0, 0.02, v_uv.x) * smoothstep(1.0, 0.98, v_uv.x)
               * smoothstep(1.0, 0.98, k) * smoothstep(0.0, 1.25 * w, d);
    gl_FragColor = c * gloss * soft * k * k * alpha;
}
