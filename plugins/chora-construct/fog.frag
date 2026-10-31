// The space of chora-construct: a ray from the eye through each pixel.
// Above the horizon a sky with a thin bright band at the horizon; below
// it a smooth, glossy floor that mirrors the sky, more so at grazing
// angles (fresnel), with a crisp edge between them. A broad soft light,
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
        // The mirrored sky, by fresnel: little looking down, much at
        // grazing angles.
        float cosv = -dy / length(vec3(dx, dy, dz));
        float fresnel = 0.05 + 0.95 * pow(1.0 - max(cosv, 0.0), 5.0);
        vec3 base = mix(ground, heaven(-e), 0.65 * fresnel);
        // The softbox: a broad, soft light low ahead of the start, which
        // glints as a wide band and turns with the view.
        float az = cam.x + atan(dx, dz);
        float da = atan(sin(az), cos(az));
        float de = -e - 0.16;
        base += sheen * 0.22 * exp(-da * da * 1.2 - de * de * 60.0);
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
