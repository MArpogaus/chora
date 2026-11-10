#extension GL_OES_standard_derivatives : enable
// The mirror image of a pane in a glossy floor: its node texture at a
// reduced size, a little softer the deeper it lies, faded out downwards.
// The transform flips it. The gloss of the floor falls with the haze of
// the distance and rises with the fresnel of a grazing view, from the
// `depth` that the core gives in its 3D context.
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
uniform float gloss;  // the strength of the floor
uniform float depth;  // distance of the eye over the focal length
uniform float reach;  // the focal length over the leash radius
uniform float ground; // the height of the eye over the focal length
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
    float haze = (1.0 - exp(-max(depth - 1.0, 0.0) * reach * 1.5)) * 0.9;
    float grazing = 1.0 - ground / length(vec2(ground, depth));
    float fresnel = 0.05 + 0.95 * pow(grazing, 5.0);
    float strength = gloss * (0.4 + 0.6 * fresnel) * (1.0 - haze);
    gl_FragColor = c * strength * soft * k * k * alpha;
}
