// Digital rain on a black screen.
precision highp float;
uniform float alpha;
uniform vec2 size;
uniform float time;
varying vec2 v_uv;

float hash(vec2 p) { p = fract(p * vec2(0.1031, 0.1030)); p += dot(p, p.yx + 33.33); return fract((p.x + p.y) * p.x); }

void main() {
    vec2 cell = vec2(14.0, 18.0);
    vec2 grid = floor(v_uv * size / cell);
    vec2 inner = fract(v_uv * size / cell);
    float speed = 4.0 + 8.0 * hash(vec2(grid.x, 1.0));
    float head = fract(time * speed / (size.y / cell.y) + hash(vec2(grid.x, 7.0)));
    float row = grid.y / (size.y / cell.y);
    float trail = fract(head - row);
    float glow = pow(1.0 - trail, 6.0);
    float glyph = step(0.35, hash(grid + floor(time * 6.0 * hash(grid))));
    float shape = step(0.15, inner.x) * step(inner.x, 0.85) * step(0.1, inner.y) * step(inner.y, 0.9);
    float bright = glow * glyph * shape;
    vec3 c = vec3(0.1, 1.0, 0.35) * bright + vec3(0.8, 1.0, 0.85) * step(trail, 0.02) * shape * glyph;
    gl_FragColor = vec4(c, 1.0) * alpha;
}
