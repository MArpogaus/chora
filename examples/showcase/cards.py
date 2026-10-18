#!/usr/bin/env python3
"""A card UI drawn by shaders: four glowing cards; each turn lifts one
and pans the camera to it. Run while chora lives, with the environment of
a headless session (`CHORA_SOCKET`):

  python3 examples/showcase/cards.py

The captures land in `$XDG_RUNTIME_DIR/chora-showcase-shots`. The output
capture of the `choractl` calls needs one grant before the first run:
`choractl grant cards screen.capture`.
"""
import json
import os
import subprocess
import sys
import time

CTL = os.environ.get("CHORACTL", "target/debug/choractl")
NAME = "cards"
SHOTS = os.path.join(os.environ.get("XDG_RUNTIME_DIR", os.getcwd()),
                     "chora-showcase-shots")

CARDS = [
    (60, 120, [0.95, 0.55, 0.10, 1.0]),
    (380, 120, [0.10, 0.70, 0.55, 1.0]),
    (700, 120, [0.45, 0.30, 0.90, 1.0]),
    (1020, 120, [0.90, 0.25, 0.35, 1.0]),
]

FRAG = """
precision mediump float;
uniform float time;
uniform vec4 tint;
varying vec2 v_uv;
void main() {
    vec2 p = abs(v_uv - 0.5);
    float frame = smoothstep(0.44, 0.49, max(p.x, p.y));
    float shimmer = 0.5 + 0.5 * sin(time * 2.0 + (v_uv.x + v_uv.y) * 9.0);
    vec3 base = mix(tint.rgb * 0.25, tint.rgb, 0.6 + 0.4 * shimmer);
    vec3 edge = mix(tint.rgb, vec3(1.0), 0.35 + 0.35 * sin(time * 3.0));
    gl_FragColor = vec4(mix(base, edge, frame), 1.0);
}
"""


def ctl(*args: str, inp: str | None = None, quiet: bool = False) -> str:
    r = subprocess.run([CTL, "--name", NAME, *args], input=inp,
                       capture_output=True, text=True)
    if r.returncode and not quiet:
        sys.exit(f"choractl {args[0]}: {r.stdout}{r.stderr}")
    return r.stdout


def main() -> None:
    os.makedirs(SHOTS, exist_ok=True)
    ctl("grant", NAME, "screen.capture", quiet=True)
    # A second run takes the nodes of the first again; they may also be
    # gone already, which the quiet removes tolerate.
    for i in range(len(CARDS)):
        ctl("apply", "-", inp=f'[{{"remove": {{"id": "card{i}"}}}}]', quiet=True)
    path = ctl("asset", "put", "card.frag", "-", inp=FRAG).strip()
    cards = [
        {"create": {"id": f"card{i}", "kind": "shader", "parent": "root",
         "props": {"shader": path, "size": [220, 300],
                   "uniforms": {"tint": tint},
                   "transform": {"translate": [x, y]}}}}
        for i, (x, y, tint) in enumerate(CARDS)
    ]
    ctl("apply", "-", inp=json.dumps(cards))
    for i, (x, y, _) in enumerate(CARDS):
        moves = [
            {"update": {"id": f"card{j}", "props": {"transform": {
                "translate": [nx, ny + (-40 if j == i else 0)]}}}}
            for j, (nx, ny, _) in enumerate(CARDS)
        ]
        moves.append({"camera": {"output": "HEADLESS-1", "duration": 0.4,
                                 "transform": {"translate": [x - 530, -60]}}})
        ctl("apply", "-", inp=json.dumps(moves))
        time.sleep(0.6)
        ctl("capture", f"{SHOTS}/cards-{i}.png", "--output", "HEADLESS-1")
    # Without a transform the core hands the camera back, so the next
    # program gets it even though  resumes these nodes.
    ctl("apply", "-", inp='[{"camera": {"output": "HEADLESS-1"}}]', quiet=True)
    print(f"cards in {SHOTS}/cards-*.png")


if __name__ == "__main__":
    main()
