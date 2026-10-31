// The mirror image of a pane in a glossy floor: its node texture at a
// reduced size, a little softer the deeper it lies, faded out downwards.
// The transform flips it; the gloss carries the fresnel and the
// distance.
precision mediump float;
uniform sampler2D win;
uniform float alpha;
uniform vec2 size;    // of this node's texture, in pixels
uniform float gloss;  // the strength at the pane's bottom edge
varying vec2 v_uv;

void main() {
    float k = v_uv.y;   // 1 at the pane's bottom edge, nearest the ground
    vec2 o = vec2(0.0, 1.5 * (1.0 - k)) / size;
    vec4 c = 0.5 * texture2D(win, v_uv)
           + 0.25 * (texture2D(win, v_uv + o) + texture2D(win, v_uv - o));
    float soft = smoothstep(0.0, 0.02, v_uv.x) * smoothstep(1.0, 0.98, v_uv.x)
               * smoothstep(1.0, 0.98, k);
    gl_FragColor = c * gloss * soft * k * k * alpha;
}
