// Shows a node texture: a live window on a panel of the walk.
precision mediump float;
uniform sampler2D win;
uniform float alpha;
varying vec2 v_uv;

void main() {
    gl_FragColor = texture2D(win, v_uv) * alpha;
}
