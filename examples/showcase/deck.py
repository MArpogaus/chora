#!/usr/bin/env python3
"""Text-only deck on chora: PIL renders the slides, chora shows them.

Exercises: asset put, image nodes under `screen:NAME`, and asset reload
while a node shows the image. Needs PIL, for example from the venv of
`examples/loader`. Run while chora lives, with the environment of a
headless session (`CHORA_SOCKET`):

  examples/loader/.venv/bin/python examples/showcase/deck.py

The rendered slides land in `$XDG_RUNTIME_DIR/chora-showcase-shots/deck`.
"""
import json, os, subprocess, sys, time
from PIL import Image, ImageDraw, ImageFont

SOCK = os.environ["CHORA_SOCKET"]
CTL = os.environ.get("CHORACTL", "target/debug/choractl")
DECK = os.path.join(os.environ.get("XDG_RUNTIME_DIR", os.getcwd()),
                 "chora-showcase-shots", "deck")
FONT = os.environ.get("LOADER_FONT",
    os.path.expanduser("~/.local/opt/clients/usr/share/fonts/dejavu-sans-fonts/DejaVuSans-Bold.ttf"))

SLIDES = [
    ("CHORA", ["a wayland compositor as a scene graph", "session 2026-10-17 dogfood deck"]),
    ("MODEL", ["nodes in a tree under root", "screen:NAME stays fixed to the output",
               "one client owns the camera"]),
    ("INPUT", ["grab = seat control", "token: 10 s, must still have focus",
               "inject counts as a real press"]),
    ("SAFE", ["no shell in the core", "one grant, one request, one use",
              "revoke flushes at the frame boundary"]),
    ("THE END", ["run this with: examples/showcase/deck.py"]),
]

def render(i, title, lines, path, size=(1280, 800)):
    img = Image.new("RGB", size, (12, 14, 20))
    d = ImageDraw.Draw(img)
    f_big = ImageFont.truetype(FONT, 72)
    f = ImageFont.truetype(FONT, 30)
    d.rectangle([0, 0, size[0], 6], fill=(0, 200, 255))
    d.text((90, 120), title, font=f_big, fill=(235, 240, 250))
    y = 280
    for line in lines:
        d.text((96, y), "·  " + line, font=f, fill=(150, 240, 255))
        y += 60
    d.text((90, size[1] - 70), f"{i + 1} / {len(SLIDES)}", font=f, fill=(90, 100, 120))
    img.save(path)

def ctl(*args):
    return subprocess.run([CTL, "--name", "deck", *args],
                          capture_output=True, text=True)

def apply(*cmds):
    inp = json.dumps(list(cmds))
    r = subprocess.run([CTL, "--name", "deck", "apply", "-"], input=inp,
                       capture_output=True, text=True)
    if r.returncode:
        sys.exit(f"apply failed: {r.stdout}{r.stderr}")

def screen():
    return json.loads(ctl("outputs").stdout)[0]["name"]


def main():
    os.makedirs(DECK, exist_ok=True)
    paths = []
    for i, (t, ls) in enumerate(SLIDES):
        p = f"{DECK}/slide{i}.png"
        render(i, t, ls, p)
        out = subprocess.run([CTL, "--name", "deck", "asset", "put",
                              f"slide{i}.png", p], capture_output=True, text=True)
        if out.returncode:
            sys.exit(f"asset put failed: {out.stderr}")
        paths.append(out.stdout.strip())
    for nid in ("deck-img", "deck-bg", "deck"):
        subprocess.run([CTL, "--name", "deck", "apply", "-"],
                       input=f'[{{"remove": {{"id": "{nid}"}}}}]',
                       capture_output=True, text=True)
    apply({"create": {"id": "deck", "kind": "group", "parent": f"screen:{screen()}",
                      "props": {"size": [1280, 800]}}})
    apply({"create": {"id": "deck-bg", "kind": "rect", "parent": "deck",
                      "props": {"size": [1280, 800], "color": [0.03, 0.04, 0.06, 1]}}})
    apply({"create": {"id": "deck-img", "kind": "image", "parent": "deck",
                      "props": {"image": paths[0]}}})
    ctl("grant", "deck", "input.inject")
    numbers = [a for a in sys.argv[1:] if not a.startswith("-")]
    n = int(numbers[0]) if numbers else 1
    for s in range(n):
        for i in range(len(SLIDES)):
            apply({"update": {"id": "deck-img", "props": {"image": paths[i], "opacity": 1}}})
            print(f"slide {i + 1}")
            time.sleep(3 if s == 0 else 0.4)
    if "--cleanup" in sys.argv:
        apply({"remove": {"id": "deck"}})

if __name__ == "__main__":
    main()
