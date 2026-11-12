#extension GL_OES_standard_derivatives : enable
// The mirror image of a pane in a floor of polished marble: its node
// texture at a reduced size, faded out downwards. The roughness of the
// polish blurs it: sharp where the pane meets the floor, softer the
// farther the reflected point lies from that line, and stretched along
// the view, as a reflection on a polished floor is. Micro bumps of the
// polish waver it by a pixel or two. The transform flips it. The gloss
// of the floor falls with the haze of the distance and rises with the
// fresnel of a grazing view, from the `depth` that the core gives in its
// 3D context. It reads no clock: a still pane costs no frame.
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
uniform vec3 fade;    // the haze of the pane: where it starts, how fast
                      // it grows, and its most
uniform float ground; // the height of the eye over the focal length
uniform float fall;    // how far the image reaches below the pane, in texture pixels
uniform float feather; // the soft edge at the sides and the top, in texture pixels
uniform vec4 rough;   // the blur in texture pixels at the contact line
                      // and at the top, its stretch along the view, and
                      // the waver of the bumps in texture pixels
varying vec2 v_uv;

float hash(vec2 p) {
    vec3 q = fract(vec3(p.xyx) * 0.1031);
    q += dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    vec2 u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

// The texture blurred by an ellipse of radii r over 13 taps on a golden
// spiral, weighted as a Gauss.
vec4 blurred(vec2 uv, vec2 r) {
    vec4 sum = texture2D(win, uv);
    float total = 1.0;
    for (int i = 1; i < 13; i++) {
        float t = float(i) / 12.0;
        float a = float(i) * 2.3999632;
        vec2 o = vec2(cos(a), sin(a)) * sqrt(t) * r / size;
        float w = exp(-2.0 * t);
        sum += texture2D(win, uv + o) * w;
        total += w;
    }
    return sum / total;
}

void main() {
    vec2 p = v_uv * size;
    float w = texels(p);
    float d = min(min(p.x, p.y), min(size.x - p.x, size.y - p.y));
    float k = v_uv.y;   // 1 at the pane's bottom edge, nearest the ground
    float r = mix(rough.y, rough.x, k);
    float waver = (noise(p / vec2(48.0, 14.0)) - 0.5) * 2.0 * rough.w;
    vec2 uv = v_uv + vec2(waver / size.x, 0.0);
    vec4 c = blurred(uv, vec2(r, r * rough.z));
    // The side edges are feathered by `feather`: 3 px for the faces of a
    // cube, so that they sit on each other's edges, more for a pane, so
    // that no hard slab stands on the floor; the image ends `fall` px
    // under the pane.
    float side = feather / size.x;
    float down = smoothstep(1.0 - fall / size.y, 1.0, k);
    float soft = smoothstep(0.0, side, v_uv.x) * smoothstep(1.0, 1.0 - side, v_uv.x)
               * smoothstep(1.0, 1.0 - max(feather / size.y, 0.02), k) * smoothstep(0.0, 1.25 * w, d) * down;
    float haze = (1.0 - exp(-max(max(depth - 1.0, 0.0) * reach - fade.x, 0.0) * fade.y)) * fade.z;
    float grazing = 1.0 - ground / length(vec2(ground, depth));
    float fresnel = 0.05 + 0.95 * pow(grazing, 5.0);
    float strength = gloss * (0.4 + 0.6 * fresnel) * (1.0 - haze);
    gl_FragColor = c * strength * soft * k * k * alpha;
}
