// ball.frag — pulsing gradient on the pong ball, shown through the layer of
// the rounded rect. Premultiplied in and out, like every chora effect.
precision highp float;
uniform sampler2D tex;
uniform float time;
varying vec2 v_uv;

void main() {
    float a = texture2D(tex, v_uv).a;
    float p = 0.5 + 0.5 * sin(time * 6.0);
    vec3 c = mix(vec3(0.25, 0.85, 1.0), vec3(1.0, 0.35, 0.85),
                 clamp(v_uv.y + 0.35 * p, 0.0, 1.0));
    gl_FragColor = vec4(c * a, a);
}
