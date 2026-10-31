// The space of chora-construct: a ray from the eye through each pixel;
// the fog is densest at the horizon and thins up and down, with slow,
// soft noise that turns with the view. No grid, no floor. Near the
// leash the fog thickens. It has no time: a still camera costs no frame.
precision highp float;
uniform float alpha;
uniform vec2 view;    // logical width and height of the output
uniform vec2 cam;     // yaw and pitch in radians
uniform float focal;  // in logical pixels
uniform vec3 sky;     // the empty space
uniform vec3 fog;     // the fog at the horizon
uniform float edge;   // 0 at the start point, 1 at the leash
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

void main() {
    vec2 p = v_uv * view;
    float dx = (p.x - view.x * 0.5) / focal;
    float sy = -(p.y - view.y * 0.5) / focal;
    float cp = cos(cam.y), sp = sin(cam.y);
    float dy = cp * sy + sp;
    float dz = -sp * sy + cp;
    float e = dy / length(vec2(dx, dz));   // the slope above the horizon
    float az = cam.x + atan(dx, dz);
    vec2 ring = vec2(cos(az), sin(az)) * 2.5;
    float n = noise(ring + vec2(0.0, e * 4.0))
            + 0.5 * noise(ring * 2.3 - vec2(e * 9.0, 0.0));
    float band = exp(-abs(e) * (e > 0.0 ? 3.5 : 2.5));
    float haze = clamp(band * (0.7 + 0.4 * n / 1.5), 0.0, 1.0);
    haze = mix(haze, 1.0, edge * edge * edge * 0.75);
    gl_FragColor = vec4(mix(sky, fog, haze), 1.0) * alpha;
}
