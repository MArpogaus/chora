// A pane of chora-construct: the live window through its node texture,
// faded into the fog with its distance.
precision mediump float;
uniform sampler2D win;
uniform float alpha;
uniform float haze;   // 0 near, up to 1 far
uniform vec3 fog;
varying vec2 v_uv;

void main() {
    vec4 c = texture2D(win, v_uv);
    gl_FragColor = vec4(mix(c.rgb, fog * c.a, haze), c.a) * alpha;
}
