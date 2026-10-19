// The floor of the construct: white, with a faint perspective grid that
// fades into the horizon at the top edge.
precision mediump float;
uniform float alpha;
uniform vec2 size;
uniform float time;
varying vec2 v_uv;

void main() {
    float y = v_uv.y + 0.02;
    float depth = 1.0 / y;
    vec2 g = vec2((v_uv.x - 0.5) * depth * 6.0, depth * 1.5 - time * 0.15);
    vec2 f = abs(fract(g) - 0.5);
    float line = smoothstep(0.46, 0.5, max(f.x, f.y));
    float fade = smoothstep(0.0, 0.6, v_uv.y);
    float a = line * 0.10 * fade;
    gl_FragColor = vec4(vec3(0.55, 0.6, 0.65) * a, a) * alpha;
}
