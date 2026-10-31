// The space of chora-construct: a ray from the eye through each pixel.
// Above the horizon a sky, below it a ground a little darker and warmer,
// between them a soft bright band with slow noise that turns with the
// view. The ground carries sparse fine dust that moves with the walk,
// and a soft shadow under each pane; both fade into the band with the
// distance. No grid, no lines. Near the leash the fog thickens. It has
// no time: a still camera costs no frame.
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
uniform vec3 dust;
// The panes' shadows: centre x and z, half width, lift above the ground;
// a half width of 0 is no pane. The core passes at most 16 uniforms.
uniform vec4 s0;
uniform vec4 s1;
uniform vec4 s2;
uniform vec4 s3;
uniform vec4 s4;
uniform vec4 s5;
uniform vec4 s6;
uniform vec4 s7;
varying vec2 v_uv;

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

// A soft oval under a pane, across its face; wider, fainter and softer
// the higher the pane floats.
float shadow(vec4 s, vec2 g, vec2 eye, float height) {
    if (s.z <= 0.0) return 0.0;
    vec2 to = s.xy - eye;
    vec2 f = to / max(length(to), 1.0);
    vec2 d = g - s.xy;
    float lift = s.w / height;
    float a = dot(d, vec2(f.y, -f.x)) / (s.z * (0.8 + 0.3 * lift));
    float b = dot(d, f) / (s.z * (0.2 + 0.2 * lift));
    return exp(-(a * a + b * b) * 2.0) / (1.0 + 1.5 * lift);
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
    float az = cam.x + atan(dx, dz);
    vec2 ring = vec2(cos(az), sin(az)) * 2.5;
    float n = noise(ring + vec2(0.0, e * 4.0))
            + 0.5 * noise(ring * 2.3 - vec2(e * 9.0, 0.0));
    float glow = 0.8 + 0.2 * n / 1.5;
    vec3 c;
    if (e >= 0.0) {
        c = mix(sky, fog, exp(-e * 5.0) * glow);
    } else {
        // The point on the ground, its distance and the size of a pixel
        // there, across the view.
        float t = height / -dy;
        float cy = cos(cam.x), sx = sin(cam.x);
        vec2 dir = vec2(cy * dx + sx * dz, -sx * dx + cy * dz);
        vec2 g = pose.zw + dir * t;
        float dist = t * reach;
        float px = scale * t / focal;
        // The dust: one dot in some cells, a disc on the ground that
        // shrinks in depth; below a pixel it fades instead of shrinking.
        vec2 cell = floor(g / 150.0);
        float keep = step(0.5, hash(cell + 3.1));
        vec2 at = (cell + 0.2 + 0.6 * vec2(hash(cell), hash(cell + 17.0))) * 150.0;
        vec2 fwd = dir / length(dir);
        vec2 d = g - at;
        float squash = height / max(dist, height);
        float r = (4.0 + 4.0 * hash(cell + 9.0)) / px;
        vec2 rad = max(vec2(r, r * squash), vec2(0.7));
        vec2 q = vec2(dot(d, vec2(fwd.y, -fwd.x)), dot(d, fwd) * squash) / px / rad;
        float cover = min(1.0, r * r * squash / (rad.x * rad.y));
        float speck = keep * cover * (1.0 - smoothstep(0.35, 1.0, length(q)));
        float shade = shadow(s0, g, pose.zw, height) + shadow(s1, g, pose.zw, height) + shadow(s2, g, pose.zw, height) + shadow(s3, g, pose.zw, height)
                    + shadow(s4, g, pose.zw, height) + shadow(s5, g, pose.zw, height) + shadow(s6, g, pose.zw, height) + shadow(s7, g, pose.zw, height);
        c = mix(ground, dust, speck * 0.75) * (1.0 - 0.28 * min(shade, 1.0));
        // Into the band with the distance, and close to the horizon.
        float far = 1.0 - exp(-dist / 5200.0);
        c = mix(c, fog, max(far * far, exp(e * 14.0) * glow));
    }
    c = mix(c, fog, edge * edge * edge * 0.75);
    gl_FragColor = vec4(c, 1.0) * alpha;
}
