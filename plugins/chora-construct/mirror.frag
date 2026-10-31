// The mirror image of a pane in a satin ground: its node texture at a
// reduced size, blurred more the deeper it lies, faded out downwards and
// with the distance. The transform flips it.
precision mediump float;
uniform sampler2D win;
uniform float alpha;
uniform vec2 size;    // of this node's texture, in pixels
uniform float gloss;  // the strength at the pane's bottom edge
varying vec2 v_uv;

void main() {
    float k = v_uv.y;   // 1 at the pane's bottom edge, nearest the ground
    vec2 o = vec2(1.0, 1.0 + 3.0 * (1.0 - k)) / size;
    vec4 c = 0.4 * texture2D(win, v_uv)
           + 0.15 * (texture2D(win, v_uv + o) + texture2D(win, v_uv - o)
                   + texture2D(win, v_uv + vec2(o.x, -o.y)) + texture2D(win, v_uv - vec2(o.x, -o.y)));
    float soft = smoothstep(0.0, 0.08, v_uv.x) * smoothstep(1.0, 0.92, v_uv.x)
               * smoothstep(1.0, 0.95, k);
    gl_FragColor = c * gloss * soft * k * k * k * alpha;
}
