// The space of chora-construct: a ray from the eye through each pixel.
// Above the horizon a sky with a thin bright band at the horizon; below
// it a floor of polished marble that mirrors the sky, more so at grazing
// angles (fresnel), with a crisp edge between them. The marble is calm:
// faint warped veins in the ground, micro bumps that bend what it
// mirrors, and fine scratches that show only where the light glints.
// Each detail fades out where a pixel covers more floor than the detail
// is wide, so that a walk does not shimmer. A broad soft light,
// as of a studio softbox, glints in the floor and turns with the view;
// a soft shadow lies under each pane. The floor softens a little into
// the horizon far away. Near the leash the fog thickens. It has no
// time: a still camera costs no frame.
precision highp float;
uniform float alpha;
uniform vec2 view;    // logical width and height of the output
uniform vec4 pose;    // yaw and pitch in radians, x and z on the ground
uniform vec4 lens;    // the focal length and the height of the eye in
                      // logical pixels, logical pixels per pixel of this
                      // node, and the leash: 0 at the start, 1 at R
uniform vec3 sky;
uniform vec3 fog;     // the band at the horizon
uniform vec3 ground;
uniform vec3 sheen;   // the light that glints in the floor
uniform vec4 marble;  // the strength of the veins, the bumps and the
                      // scratches, from 0 to 1, and the size of the
                      // pattern of the veins in logical pixels
// The panes' shadows: centre x and z, half width, and the lift above the
// ground in whole pixels plus the yaw of the pane as the fraction
// (yaw / 2pi + 0.5) * 0.99; a half width of 0 is no pane. The core
// passes at most 16 uniforms.
uniform vec4 s0;
uniform vec4 s1;
uniform vec4 s2;
uniform vec4 s3;
uniform vec4 s4;
uniform vec4 s5;
uniform vec4 s6;
uniform vec4 s7;
varying vec2 v_uv;

// A soft oval under a pane, across its face; wider, fainter and softer
// the higher the pane floats.
float shadow(vec4 s, vec2 g, float height) {
    if (s.z <= 0.0) return 0.0;
    float yaw = (fract(s.w) / 0.99 - 0.5) * 6.2831853;
    vec2 f = vec2(sin(yaw), cos(yaw));
    vec2 d = g - s.xy;
    float lift = floor(s.w) / height;
    float a = dot(d, vec2(f.y, -f.x)) / (s.z * (0.8 + 0.3 * lift));
    float b = dot(d, f) / (s.z * (0.2 + 0.2 * lift));
    return exp(-(a * a + b * b) * 2.0) / (1.0 + 1.5 * lift);
}

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

float fbm(vec2 p) {
    float sum = 0.0;
    float a = 0.5;
    for (int i = 0; i < 4; i++) {
        sum += a * noise(p);
        p = p * 2.03 + 17.1;
        a *= 0.5;
    }
    return sum;
}

// A thin vein along the lines t = n (n whole), as a cover from 0 to 1,
// widened and faded by the floor that a pixel covers, `foot`, in units
// of t, with a soft halo around it.
float vein(float t, float width, float foot) {
    float d = abs(fract(t + 0.5) - 0.5);
    float w = width + foot;
    float core = (1.0 - smoothstep(0.0, w, d)) * width / w;
    float halo = exp(-d * d / (16.0 * w * w)) * 0.3;
    return core + halo;
}

// The veins of the marble at the ground point g: long meandering ones
// that come and go, finer ones across them, and a faint cloudiness;
// from about -0.1 to 1.
float veins(vec2 g, float foot) {
    vec2 q = g / marble.w;
    float f = foot / marble.w;
    float warp = (fbm(q * 0.8) - 0.5) * 1.4 + (fbm(q * 3.1 + 5.0) - 0.5) * 0.3;
    float main = vein((q.x * 0.55 + q.y * 0.83 + warp) * 0.6, 0.02, f * 0.6)
               * smoothstep(0.3, 0.6, fbm(q * 0.5 + 3.0));
    float fine = vein((q.x * 1.3 - q.y * 0.9 + warp * 1.3) * 1.4, 0.015, f * 1.4)
               * smoothstep(0.4, 0.7, fbm(q * 0.9 + 11.0));
    float cloud = (fbm(q * 2.5 + 7.0) - 0.5) * 0.25;
    return main + 0.6 * fine + cloud;
}

// Fine scratches at the ground point g: in cells of 28 pixels, some
// hold a short straight line at a random angle.
float scratches(vec2 g, float foot) {
    float sum = 0.0;
    for (int layer = 0; layer < 2; layer++) {
        vec2 c = g / 28.0 + float(layer) * 0.5;
        vec2 cell = floor(c);
        float h = hash(cell + float(layer) * 41.0);
        if (h < 0.55) {
            float a = h * 11.42;
            vec2 dir = vec2(cos(a), sin(a));
            vec2 d = fract(c) - 0.5 - (vec2(hash(cell + 7.0), hash(cell + 13.0)) - 0.5) * 0.4;
            float across = abs(dot(d, vec2(-dir.y, dir.x)));
            float along = abs(dot(d, dir));
            float w = 0.03 + foot / 28.0;
            sum += (1.0 - smoothstep(0.0, w, across)) * (0.03 / w)
                 * (1.0 - smoothstep(0.25, 0.45, along));
        }
    }
    return sum;
}

// The sky at the slope e above the horizon: the band is thin.
vec3 heaven(float e) {
    return mix(sky, fog, exp(-e * 120.0));
}

void main() {
    vec2 cam = pose.xy;
    float focal = lens.x, height = lens.y, scale = lens.z, edge = lens.w;
    vec2 p = v_uv * view;
    float dx = (p.x - view.x * 0.5) / focal;
    float sy = -(p.y - view.y * 0.5) / focal;
    float cp = cos(cam.y), sp = sin(cam.y);
    float dy = cp * sy + sp;
    float dz = -sp * sy + cp;
    float reach = length(vec2(dx, dz));
    float e = dy / reach;   // the slope above the horizon
    // One pixel of this node as a slope, for a crisp, smooth edge.
    float pixel = scale / focal;
    vec3 c = heaven(max(e, 0.0));
    if (e < pixel) {
        float t = height / max(-dy, 1e-6);
        float cy = cos(cam.x), sx = sin(cam.x);
        vec2 dir = vec2(cy * dx + sx * dz, -sx * dx + cy * dz);
        vec2 g = pose.zw + dir * t;
        float dist = t * reach;
        // The floor that a pixel covers, longer along the view at a
        // grazing angle.
        float foot = dist * pixel / max(-e, 0.02);
        // The micro bumps tilt the floor a little; they fade out where a
        // pixel covers them.
        vec2 b = g / 36.0;
        float n0 = noise(b);
        vec2 tilt = vec2(noise(b + vec2(0.5, 0.0)) - n0, noise(b + vec2(0.0, 0.5)) - n0);
        tilt *= marble.y * 0.012 * (1.0 - smoothstep(4.0, 24.0, foot));
        float eb = e + tilt.y;
        // The veins in the ground: grey in a light floor, a warm white in
        // a dark one.
        float light = step(0.5, dot(ground, vec3(0.2126, 0.7152, 0.0722)));
        vec3 veined = mix(ground + vec3(0.30, 0.29, 0.27), ground * 0.8, light);
        vec3 stone = mix(ground, veined, marble.x * 0.55 * clamp(veins(g, foot), -0.2, 1.0));
        // The mirrored sky, by fresnel: little looking down, much at
        // grazing angles.
        float cosv = -dy / length(vec3(dx, dy, dz)) + tilt.y;
        float fresnel = 0.05 + 0.95 * pow(1.0 - clamp(cosv, 0.0, 1.0), 5.0);
        vec3 base = mix(stone, heaven(-eb), 0.65 * fresnel);
        // The softbox: a broad, soft light low ahead of the start, which
        // glints as a wide band and turns with the view; the scratches
        // catch it.
        float az = cam.x + atan(dx, dz) + tilt.x;
        float da = atan(sin(az), cos(az));
        float de = -eb - 0.16;
        float glint = exp(-da * da * 1.2 - de * de * 60.0);
        float scratched = 1.0 + marble.z * 2.5 * min(scratches(g, foot), 1.0);
        base = mix(base, sheen, min(0.22 * glint * scratched, 1.0));
        float shade = shadow(s0, g, height) + shadow(s1, g, height) + shadow(s2, g, height) + shadow(s3, g, height)
                    + shadow(s4, g, height) + shadow(s5, g, height) + shadow(s6, g, height) + shadow(s7, g, height);
        base *= 1.0 - 0.3 * min(shade, 1.0);
        // A little softer far away, into the edge of the floor.
        float far = 1.0 - exp(-dist / 60000.0);
        base = mix(base, mix(ground, fog, 0.65), far * 0.35);
        c = mix(base, c, smoothstep(-pixel, pixel, e));
    }
    c = mix(c, fog, edge * edge * edge * 0.75);
    gl_FragColor = vec4(c, 1.0) * alpha;
}
