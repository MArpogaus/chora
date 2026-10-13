// A futuristic progress bar: a dim track, a glowing fill with a moving
// shimmer, and scanlines. Uniform: progress from 0 to 1.
precision mediump float;
uniform float alpha;
uniform vec2 size;
uniform float time;
uniform float progress;
varying vec2 v_uv;

void main() {
    vec2 uv = v_uv;
    float edge = smoothstep(0.0, 0.5, 0.5 - abs(uv.y - 0.5));
    vec3 cyan = vec3(0.15, 0.95, 1.0);
    vec3 pink = vec3(1.0, 0.2, 0.85);
    // The track: thin ticks every 2 % of the width.
    float tick = step(0.85, fract(uv.x * 50.0));
    vec4 color = vec4(cyan * 0.25, 0.25 + 0.2 * tick) * edge;
    if (uv.x < progress) {
        vec3 fill = mix(cyan, pink, uv.x / max(progress, 0.001));
        float shimmer = pow(max(0.0, sin((uv.x * 6.0 - time * 1.6) * 3.14159)), 24.0);
        float scan = 0.85 + 0.15 * sin(uv.x * size.x * 0.9 - time * 20.0);
        float head = exp(-abs(uv.x - progress) * size.x * 0.05);
        vec3 c = fill * scan + vec3(shimmer * 0.8) + vec3(head);
        color = vec4(c, 1.0) * (0.35 + 0.65 * edge);
    }
    gl_FragColor = vec4(color.rgb * color.a, color.a) * alpha;
}
