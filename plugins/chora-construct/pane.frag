#extension GL_OES_standard_derivatives : enable
// The effect of a pane of chora-construct over its layer: the window or
// the source of the pane, faded into the fog with its distance. The
// focused pane has no haze and an accent hairline along its edge
// (docs/construct-style.md, rule 16). The core gives `depth` and
// `facing` of the plane in its 3D context, so the haze and the hairline
// follow the camera without updates. The hairline is 1.5 output pixels
// wide: wider on the texture with the distance, and on the sides that
// the turn of the pane narrows.
precision highp float;

// Texture pixels per screen pixel: the screen derivative where the driver
// has it, else 1.
float texels(vec2 p) {
#ifdef GL_OES_standard_derivatives
    return max(max(fwidth(p.x), fwidth(p.y)), 1e-3);
#else
    return 1.0;
#endif
}

uniform sampler2D tex;
uniform float alpha;
uniform vec2 size;     // of the layer texture, in pixels
uniform float scale;   // texture pixels per logical pixel
uniform float depth;   // distance of the eye over the focal length, 1 at 1:1
uniform float facing;  // cosine of the view angle, negative from behind
uniform vec3 fog;
uniform vec4 edge;     // the accent, and 1 for the focused pane
uniform float reach;   // the focal length over the leash radius
uniform vec3 fade;     // the distance where the haze starts, how fast it
                       // grows, and its most
varying vec2 v_uv;

void main() {
    vec4 c = texture2D(tex, v_uv);
    vec2 p = v_uv * size;
    float w = texels(p);
    float d = min(min(p.x, p.y), min(size.x - p.x, size.y - p.y));
    float far = max(depth - 1.0, 0.0) * reach;
    float haze = edge.w > 0.0 ? 0.0 : (1.0 - exp(-max(far - fade.x, 0.0) * fade.y)) * fade.z;
    c = vec4(mix(c.rgb, fog * c.a, haze), c.a);
    if (edge.w > 0.0) {
        float width = max(1.5 * depth, 1.0) * scale;
        float turn = max(abs(facing), 0.2);
        float side = min(min(p.x, size.x - p.x) * turn, min(p.y, size.y - p.y));
        float line = 0.9 * clamp((width - side) / w + 0.5, 0.0, 1.0);
        c = c * (1.0 - line) + vec4(edge.rgb * line, line);
    }
    gl_FragColor = c * alpha * smoothstep(0.0, 1.25 * w, d);
}
