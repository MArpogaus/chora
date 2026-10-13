// The floor of the walk: a ray from the eye through each pixel meets the
// plane y = 0; grid lines on it fade with the distance. Above the horizon,
// a soft white sky.
precision highp float;
uniform float alpha;
uniform vec2 size;
uniform vec2 view;    // logical width and height of the screen
uniform vec4 cam;     // x, z, yaw, pitch
uniform float eye;    // height of the eye
uniform float focal;  // in logical pixels
uniform float grid;   // spacing of the lines
varying vec2 v_uv;

void main() {
    vec2 p = v_uv * view;
    float dx = (p.x - view.x * 0.5) / focal;
    float sy = -(p.y - view.y * 0.5) / focal;
    // Undo the pitch: the ray in the frame of the walker.
    float cp = cos(cam.w), sp = sin(cam.w);
    float dy = cp * sy + sp;
    float dz = -sp * sy + cp;
    vec3 sky = mix(vec3(0.985), vec3(0.93, 0.95, 0.97), clamp(1.0 - dy / dz * 3.0, 0.0, 1.0));
    if (dy > -0.002 * dz) {
        gl_FragColor = vec4(sky, 1.0) * alpha;
        return;
    }
    float t = -eye / dy;
    float xc = dx * t;
    float zc = dz * t;
    float c = cos(cam.z), s = sin(cam.z);
    vec2 w = vec2(cam.x + c * xc + s * zc, cam.y - s * xc + c * zc);
    vec2 f = abs(fract(w / grid) - 0.5) * grid;
    float width = zc * 1.2 / focal;
    float line = 1.0 - smoothstep(0.0, width, min(f.x, f.y) - width * 0.5);
    float fade = exp(-zc / (grid * 30.0));
    vec3 floor_color = mix(vec3(0.97), vec3(0.6, 0.66, 0.72), line * 0.5 * fade);
    float haze = smoothstep(-0.002, -0.06, dy / dz);
    gl_FragColor = vec4(mix(sky, floor_color, haze), 1.0) * alpha;
}
