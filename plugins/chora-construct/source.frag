// The source of a pane of chora-construct that is not its own window: a
// window of another client or a node tree of a plugin, through its node
// texture. The effect pane.frag of the pane group draws the haze and
// the edge.
precision mediump float;

uniform sampler2D win;
uniform float alpha;
varying vec2 v_uv;

void main() {
    gl_FragColor = texture2D(win, v_uv) * alpha;
}
