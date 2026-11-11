// The ring of the voice island of the host in a busy state (construct HUD
// spec, section 4): a pulse while it listens or a question waits, an arc
// that turns while it thinks (1.2 s a turn) or works (2.4 s a turn). A
// still state shows a picture instead, so a still screen draws nothing.
precision highp float;
uniform float alpha;
uniform float time;
uniform vec2 size;    // of this node, in pixels
uniform vec3 accent;
uniform float mode;   // 1 listening, 2 thinking, 3 working, 4 question
varying vec2 v_uv;

const float TAU = 6.2831853;

// The cover of a ring of radius `r` and width `w` at distance `d`.
float ring(float d, float r, float w, float px) {
    return clamp((w * 0.5 - abs(d - r)) / px + 0.5, 0.0, 1.0);
}

float disc(float d, float r, float px) {
    return clamp((r - d) / px + 0.5, 0.0, 1.0);
}

void main() {
    vec2 p = (v_uv - 0.5) * 24.0;
    float px = 24.0 / size.x;
    float d = length(p);
    float a = 0.0;
    if (mode > 1.5 && mode < 3.5) {
        float period = mode < 2.5 ? 1.2 : 2.4;
        float head = fract(time / period) * TAU;
        float behind = mod(head - atan(p.x, -p.y), TAU);
        float arc = behind < TAU * 0.25 ? 1.0 : 0.22;
        a = ring(d, 9.0, 2.5, px) * arc;
    } else {
        float pulse = 0.8 + 0.2 * cos(time * TAU / 2.0);
        if (mode > 3.5) {
            a = max(disc(d, 11.0, px) * 0.18, ring(d, 8.0, 2.0, px) * pulse);
        } else {
            a = ring(d, 9.0, 2.0, px) * 0.35;
        }
        a = max(a, disc(d, 4.0, px) * pulse);
    }
    gl_FragColor = vec4(accent * a, a) * alpha;
}
