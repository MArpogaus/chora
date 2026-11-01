// The light of a glass panel of chora-construct (docs/construct-style.md,
// rules 6 and 7): the glow around the panel, and in the voice HUD the
// state ring and the waveform. The node is the panel plus the reach of
// the glow on each side. The core draws the fill and the hairline as a
// rect under it, so the panel stays readable without this shader.
precision highp float;
uniform float alpha;
uniform vec2 size;    // of this node, in pixels
uniform float time;
uniform vec4 accent;  // the accent, and the alpha of the glow
uniform vec3 muted;
uniform vec4 box;     // the panel: width, height, corner radius, glow reach
uniform vec4 ring;    // the centre x and y in the panel, the diameter (0:
                      // no ring) and the mode: 0 asleep, 1 listening,
                      // 2 thinking, 3 working, 4 question, 5 speaking
uniform float level;  // the waveform: 0 at rest, 1 while a sentence plays
varying vec2 v_uv;

const float TAU = 6.2831853;

// The signed distance to a rounded rect of half size `h` at the origin.
float rounded(vec2 p, vec2 h, float r) {
    vec2 q = abs(p) - h + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float hash(float n) {
    return fract(sin(n) * 43758.5453);
}

// Smooth noise in time, one strand per bar.
float noise(float t, float bar) {
    float i = floor(t);
    float f = fract(t);
    float a = hash(i + bar * 17.0);
    float b = hash(i + 1.0 + bar * 17.0);
    return mix(a, b, f * f * (3.0 - 2.0 * f));
}

void main() {
    float reach = box.w;
    vec2 p = v_uv * (box.xy + 2.0 * reach) - reach;
    // Logical pixels per pixel of the node, for edges of one pixel.
    float px = (box.x + 2.0 * reach) / size.x;
    vec4 c = vec4(0.0);
    float d = rounded(p - box.xy * 0.5, box.xy * 0.5, box.z);
    if (reach > 0.0 && d > 0.0) {
        float k = 1.0 - clamp(d / reach, 0.0, 1.0);
        float a = accent.a * k * k;
        c = vec4(accent.rgb * a, a);
    }
    if (ring.z > 0.0) {
        vec2 q = p - ring.xy;
        float mode = ring.w;
        // A stroke of 2 inside the diameter; asleep a thin one of 1.
        float half_width = mode < 0.5 ? 0.5 : 1.0;
        float r = ring.z * 0.5 - half_width;
        float cover = clamp((half_width - abs(length(q) - r)) / px + 0.5, 0.0, 1.0);
        float a = 1.0;
        if (mode > 1.5 && mode < 3.5) {
            // An arc of 90 degrees that turns, over a faint track.
            float period = mode < 2.5 ? 1.2 : 2.4;
            float head = fract(time / period) * TAU;
            float behind = mod(head - atan(q.y, q.x), TAU);
            float arc = TAU * 0.25;
            a = behind < arc ? 1.0 - 0.6 * behind / arc : 0.16;
        } else if (mode > 3.5 && mode < 4.5) {
            a = 0.8 + 0.2 * cos(time * TAU / 2.0);
        }
        float ink = cover * a;
        if (mode > 4.5) {
            // Five bars of 2 with gaps of 4, centred in the ring.
            float x = q.x + 13.0;
            float bar = floor(x / 6.0);
            if (bar >= 0.0 && bar < 5.0) {
                float h = (0.3 + 0.7 * level * noise(time * 5.0, bar)) * 16.0;
                float bd = rounded(vec2(x - bar * 6.0 - 1.0, q.y), vec2(1.0, h * 0.5), 1.0);
                ink = max(ink, clamp(0.5 - bd / px, 0.0, 1.0));
            }
        }
        vec3 col = mode < 0.5 ? muted : accent.rgb;
        c = c * (1.0 - ink) + vec4(col * ink, ink);
    }
    gl_FragColor = c * alpha;
}
