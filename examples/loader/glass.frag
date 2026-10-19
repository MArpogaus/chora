// Frosted glass behind the donut: the world, blurred and milky, only
// where the mask (the band on screen) is.
precision highp float;
uniform sampler2D world;
uniform sampler2D mask;
uniform vec2 view;     // logical size of the screen
uniform float radius;  // blur radius in logical pixels
uniform float alpha;
varying vec2 v_uv;

void main() {
    float m = texture2D(mask, v_uv).a;
    if (m < 0.004) {
        gl_FragColor = vec4(0.0);
        return;
    }
    // A Gauss kernel on a rotated grid: smooth, without the boxes of
    // a box filter.
    vec2 step = vec2(radius) / view / 4.0;
    mat2 turn = mat2(0.866, 0.5, -0.5, 0.866);
    vec3 sum = vec3(0.0);
    float total = 0.0;
    for (int x = -4; x <= 4; x++) {
        for (int y = -4; y <= 4; y++) {
            vec2 o = turn * vec2(float(x), float(y));
            float w = exp(-dot(o, o) / 8.0);
            sum += texture2D(world, v_uv + o * step).rgb * w;
            total += w;
        }
    }
    vec3 milk = mix(sum / total, vec3(0.96, 0.97, 0.98), 0.35);
    gl_FragColor = vec4(milk, 1.0) * m * alpha;
}
