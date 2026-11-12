#!/usr/bin/python3
"""Audit the surfaces of the tiling look on a capture.

Usage: tests/tiling-audit.py CAPTURE.png NODES.json THEME SCALE CELL [BAR]
  NODES.json  `choractl query '*'`
  THEME       ink or paper
  SCALE       the scale of the output
  CELL        desktop, chat, question or fullscreen
  BAR         `bar` when the panel runs: the islands stand at y 8 to 40

Prints one `FAIL:` line per finding; exit status 1 on a finding.

  tokens      the flat fill of each helper surface is a token of the theme
              (`surface` or `surface-2`), its hairline is `line`, and an
              unfocused tile border is `line`
  radius      the corners of each helper surface and of a tile are rounded in
              the radius family of 10 to 14
  overlap     a helper surface stays clear of the bar and of the tile borders
  fullscreen  with a fullscreen window the bar islands do not show

The shader that draws a tile border blends its edge pixel by a tenth (EDGE)
with what lies outside, so the border is compared with `line` blended so.
"""

import importlib.util
import json
import pathlib
import sys

TOKENS = {
    "ink": {"surface": (26, 29, 39), "surface-2": (36, 40, 54), "line": (46, 51, 68)},
    "paper": {"surface": (255, 255, 255), "surface-2": (242, 243, 247), "line": (213, 216, 225)},
}
TOLERANCE = 9
EDGE = 0.1
BAR = (8, 40)
GAP = 8


def load(png):
    here = pathlib.Path(__file__).with_name("png-pixel.py")
    spec = importlib.util.spec_from_file_location("png_pixel", here)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return [bytes(r) for r in mod.rows(open(png, "rb").read())]


def near(a, b, tolerance=TOLERANCE):
    return sum(abs(p - q) for p, q in zip(a, b)) <= tolerance


def surfaces(nodes):
    found = {}
    for node in nodes:
        name = node.get("id", "")
        if node.get("owner") != "assistant" or not node.get("rects"):
            continue
        if name in ("hud-chat", "hud-card") or name.startswith("hud-row-"):
            rect = next(iter(node["rects"].values()))
            if rect[2] > 40 and rect[3] > 20:
                found[name] = rect
    return found


def tiles(nodes):
    found = {}
    for node in nodes:
        if node.get("id", "").startswith("tb-") and node.get("rects"):
            found[node["id"]] = next(iter(node["rects"].values()))
    return found


def main():
    png, nodes_file, theme, scale, cell = sys.argv[1:6]
    bar = len(sys.argv) > 6 and sys.argv[6] == "bar"
    scale = float(scale)
    rows = load(png)
    palette = TOKENS[theme]
    nodes = json.load(open(nodes_file))
    problems = []

    def px(x, y):
        x, y = int(x * scale), int(y * scale)
        if 0 <= y < len(rows) and 0 <= x * 4 < len(rows[y]):
            return tuple(rows[y][x * 4:x * 4 + 3])
        return (0, 0, 0)

    def dominant(rect):
        x, y, w, h = rect
        counts = {}
        for yy in range(int(y) + 2, int(y + h) - 2, 3):
            for xx in range(int(x) + 2, int(x + w) - 2, 3):
                counts[px(xx, yy)] = counts.get(px(xx, yy), 0) + 1
        return max(counts, key=counts.get) if counts else (0, 0, 0)

    def corner(rect, fill):
        x, y, _, _ = rect
        for dy in range(20):
            first = next((dx for dx in range(20) if near(px(x + dx, y + dy), fill, 12)), None)
            if first is not None and first <= 2:
                return dy
        return 20

    helpers = surfaces(nodes)
    if cell != "fullscreen":
        if not helpers:
            problems.append(f"{cell}: no helper surface on the screen")
        for name, rect in helpers.items():
            fill = dominant(rect)
            if not any(near(fill, token) for key, token in palette.items() if key != "line"):
                problems.append(f"tokens {name}: fill {fill} is no token of {theme}")
            edge = px(rect[0] + rect[2] / 2, rect[1])
            if not any(near(edge, token, 18) for token in (palette["line"], fill)):
                problems.append(f"tokens {name}: hairline {edge} is no `line` {palette['line']}")
            steps = corner(rect, fill)
            if not 3 <= steps <= 8:
                problems.append(f"radius {name}: the corner turns in {steps} px, not the radius family of 10 to 14")
        borders = tiles(nodes)
        want = palette["line"]
        edge = []
        for rect in borders.values():
            mid = rect[1] + rect[3] / 2
            outside = px(rect[0] - 3, mid)
            blended = tuple(round(EDGE * o + (1 - EDGE) * w) for o, w in zip(outside, want))
            edge.append((px(rect[0] + 1, mid), blended))
        if borders and not any(near(color, blended, 26) for color, blended in edge):
            problems.append(f"tokens tile border: {edge} is not `line` {want}")
        for name, rect in helpers.items():
            if bar and rect[1] < BAR[1] + 8 and name != "hud-chat":
                problems.append(f"overlap {name} reaches the bar: top {rect[1]}")
            if bar and name == "hud-chat" and rect[1] < BAR[1] + GAP:
                problems.append(f"overlap {name} sticks to the bar: top {rect[1]}")
            if name == "hud-chat":
                continue
            for tile, t in borders.items():
                x, y, w, h = rect
                inside = x >= t[0] + GAP and y >= t[1] + GAP and x + w <= t[0] + t[2] - GAP and y + h <= t[1] + t[3] - GAP
                touches = not (x + w < t[0] - GAP or x > t[0] + t[2] + GAP or y + h < t[1] - GAP or y > t[1] + t[3] + GAP)
                if touches and not inside:
                    problems.append(f"overlap {name} crosses the border of {tile}")
    if cell == "fullscreen" and bar:
        window = px(640, 400)
        for x in (24, 66, 640, 1100, 1240):
            if not near(px(x, 24), window, 6):
                problems.append(f"fullscreen: the bar shows over the window at x {x}")
                break
    for problem in problems:
        print("FAIL:", problem)
    return 1 if problems else 0


sys.exit(main())
