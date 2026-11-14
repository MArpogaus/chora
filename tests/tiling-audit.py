#!/usr/bin/python3
"""Audit the surfaces of the tiling look on a capture.

Usage: tests/tiling-audit.py CAPTURE.png NODES.json THEME SCALE CELL [BAR] [WALL]
  NODES.json  `choractl query '*'`
  THEME       ink or paper
  SCALE       the scale of the output
  CELL        desktop, chat, question, toast, keys, layout, tooltip, launcher,
              fullscreen, chatq (chat and question card) or chattoast
              (chat and a notification)
  REF         `ref=CHAT.png`: the capture of the chat alone (cell chattoast)
  BAR         `bar` when the panel runs: the islands stand at y 8 to 40
  WALL        `wall` when the wallpaper runs behind the tiles (cell desktop)

Prints one `FAIL:` line per finding; exit status 1 on a finding.

  tokens      the flat fill of each helper surface is a token of the theme
              (`surface` or `surface-2`), its hairline is `line`, and an
              unfocused tile border is `line`
  radius      the corners of each helper surface and of a tile are rounded in
              the radius family of 10 to 14
  overlap     a helper surface stays clear of the bar and of the tile borders
  bar         the layout word stands in a pill of `surface-2`, close to the
              right edge of the left island
  terminal    the fill of a tile with a terminal is the `terminal` token
  toast       a notification stands in the lower right corner, 24 from the
              edges, on `surface` with a hairline of `line`, rounded, clear
              of the dot of the helper
  keys        the top of the key list is on the grid of 4, the title and the
              hint stand on one baseline, and the keycaps of a column end at
              one x
  surface     the fill of the bar and of the key list is `surface`, as it is
              on every helper surface
  chip        the state word of the helper stands on a token fill, is high
              enough for text of 14 px, and its orb is one of the colours of
              the palette
  chat        the state word and the orb of the helper stay clear of the
              panel, the Close button stands at the right edge of the chat panel,
              inside its padding, and the corners of the input field are
              round
  launcher    fuzzel stands opaque in the `surface` fill
  fullscreen  with a fullscreen window the bar islands do not show
  icons       the icons of the bar are drawn at the output scale: at a scale
              above 1 their edge has few mid-tones, as the text has
  chatq       the question card stands clear of the chat panel, the state word
              and the orb, and the card shows
  chattoast   the notification stands clear of the chat panel: the panel
              looks as it does without one, and the notification shows
  scrim       the key list's scrim is tinted, not grey, and the bar and the
              surfaces under the scrim stay within a few levels of each other
  look        the wallpaper shows at the corners of the screen, the focused
              tile has a glow of its own colour on the outer side, and an
              unfocused tile casts a shadow

The shader that draws a tile border dims it by up to a fifth, so the border
is compared with `line` within that range; the exact token is checked in
colors.css by the gate.
"""

import importlib.util
import json
import pathlib
import sys

TOKENS = {
    "ink": {"surface": (26, 29, 39), "surface-2": (36, 40, 54), "line": (46, 51, 68)},
    "paper": {"surface": (255, 255, 255), "surface-2": (242, 243, 247), "line": (213, 216, 225)},
}
ORBS = [(167, 139, 250), (94, 234, 212), (248, 113, 113), (154, 161, 180), (36, 40, 54)]
CHIP_HEIGHT = 26
TEXT = {"ink": (230, 232, 239), "paper": (27, 30, 39)}
TERMINAL = {"ink": (22, 25, 34), "paper": (255, 255, 255)}
PAD = 14
TOLERANCE = 9
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


def wallpaper(theme):
    name = "wallpaper.png" if theme == "ink" else "wallpaper-paper.png"
    return load(pathlib.Path(__file__).resolve().parent.parent / "flavors" / "tiling" / name)


def look(rows, nodes, theme, scale, cell):
    """The findings of the look cell: the capture against the wallpaper."""
    wall = wallpaper(theme)
    height, width = len(wall), len(wall[0]) // 4
    out_w, out_h = len(rows[0]) // 4, len(rows)
    fill = max(out_w / width, out_h / height)

    def at(grid, x, y, factor):
        x, y = int(x * factor), int(y * factor)
        y = min(max(y, 0), len(grid) - 1)
        x = min(max(x, 0), len(grid[0]) // 4 - 1)
        return tuple(grid[y][x * 4:x * 4 + 3])

    def behind(x, y):
        wx = (x * scale - (out_w - width * fill) / 2) / fill
        wy = (y * scale - (out_h - height * fill) / 2) / fill
        return at(wall, wx, wy, 1)

    def shown(x, y):
        return at(rows, x, y, scale)

    def gap(a, b):
        return sum(abs(p - q) for p, q in zip(a, b))

    problems = []
    w, h = out_w / scale, out_h / scale
    if cell == "keys":
        for x, y in ((2, 2), (w - 3, h - 3)):
            chroma = lambda c: max(c) - min(c)
            if chroma(behind(x, y)) >= 12 and chroma(shown(x, y)) < 0.6 * chroma(behind(x, y)):
                problems.append(f"scrim: {shown(x, y)} at {int(x)},{int(y)} turned grey, the wallpaper there is {behind(x, y)}")
        return problems
    for x, y in ((2, 2), (w - 3, h - 3)):
        if gap(shown(x, y), behind(x, y)) > 24:
            problems.append(f"look backdrop: {shown(x, y)} at {int(x)},{int(y)} is not the wallpaper {behind(x, y)}")
    focus = [n for n in nodes if n.get("id", "").startswith("tg-") and n.get("opacity") == 1.0 and n.get("rects")]
    for node in focus:
        x, y, tw, th = next(iter(node["rects"].values()))
        px = x - 4 if x < w - (x + tw) else x + tw + 3
        py = y + th / 2
        if gap(shown(px, py), behind(px, py)) < 20:
            problems.append(f"look glow: {shown(px, py)} at {int(px)},{int(py)} beside the focused tile is the wallpaper {behind(px, py)}")
    for node in nodes:
        if not node.get("id", "").startswith("tb-") or not node.get("rects"):
            continue
        x, y, tw, th = next(iter(node["rects"].values()))
        if any(f["id"][3:] == node["id"][3:] for f in focus):
            continue
        px, py = x + tw / 2, y + th + 4
        if gap(shown(px, py), behind(px, py)) < 4:
            problems.append(f"look shadow: {shown(px, py)} at {int(px)},{int(py)} under an unfocused tile is the wallpaper {behind(px, py)}")
    return problems


def main():
    png, nodes_file, theme, scale, cell = sys.argv[1:6]
    flags = sys.argv[6:]
    bar = "bar" in flags
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

    def corner(rect):
        """The row from which the hairline of a corner stands at the edge; 0 for a square corner."""
        x, y, _, _ = rect
        for dy in range(3, 20):
            first = next((dx for dx in range(20) if near(px(x + dx, y + dy), palette["line"], 30)), None)
            if first is not None and first <= 2:
                return dy if dy > 3 or first == 2 else 0
        return 20

    named = {n["id"]: next(iter(n["rects"].values())) for n in nodes if n.get("rects") and n["id"].startswith("hud-")}
    named_all = {n["id"]: next(iter(n["rects"].values())) for n in nodes if n.get("rects")}
    word, orb = (named.get("hud-word"), named.get("hud-figure")) if cell in ("desktop", "question") else (None, None)
    if word:
        fill = dominant(word)
        if not any(near(fill, token) for key, token in palette.items() if key != "line"):
            problems.append(f"chip: the state word fill {fill} is no token of {theme}")
        if word[3] < CHIP_HEIGHT:
            problems.append(f"chip: the state word is {word[3]} px high, under the {CHIP_HEIGHT} of text of 14 px")
    if orb:
        colour = px(orb[0] + orb[2] / 2, orb[1] + orb[3] / 2)
        if not any(near(colour, token, 12) for token in ORBS):
            problems.append(f"chip: the orb {colour} is no colour of the palette")
    panel, close = named.get("hud-chat"), named.get("hud-chat-close")
    if cell in ("chat", "chatq") and panel:
        for part, rect in (("state word", named.get("hud-word")), ("orb", named.get("hud-figure"))):
            if not rect:
                problems.append(f"chat: the {part} is not on the screen")
            elif rect[0] + rect[2] > panel[0] and rect[0] < panel[0] + panel[2] and rect[1] + rect[3] > panel[1] and rect[1] < panel[1] + panel[3]:
                problems.append(f"chat: the panel covers the {part} {rect}")
    if cell == "chat" and panel and close:
        right = panel[0] + panel[2] - PAD
        if abs(close[0] + close[2] - right) > 2:
            problems.append(f"chat: Close ends at {close[0] + close[2]}, not at the padding {right} of the panel")
    if cell == "chat" and panel:
        fill = dominant(panel)
        spot = px(panel[0] + PAD + 1, panel[1] + panel[3] - PAD - 2)
        if not near(spot, fill, 6):
            problems.append(f"chat: the corner of the input field {spot} is not clear, so the field is square")
    def meets(a, b):
        return a[0] < b[0] + b[2] and b[0] < a[0] + a[2] and a[1] < b[1] + b[3] and b[1] < a[1] + a[3]

    card = named.get("hud-card")
    if cell == "chatq":
        if not (panel and card):
            problems.append(f"chatq: the chat {panel} and the card {card} must both show")
        elif meets(panel, card):
            problems.append(f"chatq: the card {card} covers or is covered by the chat panel {panel}")
    if cell == "chattoast":
        ref = next((f[4:] for f in flags if f.startswith("ref=")), None)
        if not (panel and ref):
            problems.append("chattoast: no chat panel or no reference capture")
        else:
            before = load(ref)

            def changed(x0, x1):
                found = 0
                for yy in range(int(660 * scale), int(736 * scale), 2):
                    for xx in range(int(x0 * scale), int(x1 * scale), 2):
                        a, b = rows[yy][xx * 4:xx * 4 + 3], before[yy][xx * 4:xx * 4 + 3]
                        found += sum(abs(p - q) for p, q in zip(a, b)) > 40
                return found

            under = changed(panel[0], panel[0] + panel[2])
            if under > 40:
                problems.append(f"chattoast: the notification changes {under} samples of the chat panel {panel}")
            beside = changed(500, panel[0])
            if beside < 100:
                problems.append(f"chattoast: no notification beside the chat panel, {beside} samples changed")
    if cell == "toast":
        left, right, bottom = 896, 1256, 738
        fill = px(right - 16, bottom - 6)
        if not near(fill, palette["surface"], 6):
            problems.append(f"toast: no notification in the corner, {fill} at {right - 16},{bottom - 6}")
        else:
            top = bottom - 6
            while top > 40 and near(px(right - 16, top - 1), palette["surface"], 6):
                top -= 1
            edge = px((left + right) / 2, bottom - 1)
            if not near(edge, palette["line"], 24):
                problems.append(f"toast: the hairline {edge} is no `line` {palette['line']}")
            steps = corner([left, top, right - left, bottom - top])
            if not 3 <= steps <= 8:
                problems.append(f"toast: the corner turns in {steps} px, not the radius family of 10 to 14")
            if bottom - top < 40:
                problems.append(f"toast: only {bottom - top} px high")
    helpers = surfaces(nodes)
    if bar:
        island = px(570, 24)
        if not near(island, palette["surface"]):
            problems.append(f"tokens bar: the clock island fill {island} is not `surface` {palette['surface']}")
    if bar and cell == "desktop" and scale > 1:
        text = TEXT[theme]
        mid = full = 0
        for yy in range(int(14 * scale), int(34 * scale)):
            for xx in range(int(22 * scale), int(44 * scale)):
                p = tuple(rows[yy][xx * 4:xx * 4 + 3])
                if near(p, palette["surface"], 6):
                    continue
                if near(p, text, 6):
                    full += 1
                else:
                    mid += 1
        if full < 20 or mid > 0.6 * full:
            problems.append(f"icons: the Apps icon has {mid} mid-tone and {full} solid pixels, it is a raster scaled up")
    if bar:
        pill = [x for x in range(12, 560) if near(px(x, 24), palette["surface-2"], 3)]
        edge = [x for x in range(12, 560) if near(px(x, 24), palette["line"], 14)]
        if not pill:
            problems.append("bar: no layout pill of `surface-2` in the left island")
        elif edge and edge[-1] - pill[-1] > 18:
            problems.append(f"bar: the left island ends at {edge[-1]}, {edge[-1] - pill[-1]} px after its last pill")
    keys = named_all.get("tiling-keys-panel")
    if cell == "keys":
        if not keys:
            problems.append("keys: no key list on the screen")
        else:
            if keys[1] % 4:
                problems.append(f"keys: the top of the key list is at {keys[1]}, off the grid of 4")
            back = px(keys[0] + 5, keys[1] + 5)

            def baseline(left, right):
                rows_ = [y for y in range(int(keys[1]) + 10, int(keys[1]) + 50) if any(not near(px(x, y), back, 150) for x in range(int(keys[0]) + left, int(keys[0]) + right))]
                return rows_[-1] if rows_ else None

            title, hint = baseline(24, 36), baseline(120, 128)
            if title is None or hint is None or abs(title - hint) > 1:
                problems.append(f"keys: the title ends at row {title} and the hint at {hint}, not on one baseline")
            caps = sorted(r[0] + r[2] for i, r in named_all.items() if i.startswith("tiling-keys-body") and i.endswith("-cap"))
            columns = {}
            for edge in caps:
                columns.setdefault(round(edge / 400), []).append(edge)
            for edges in columns.values():
                if max(edges) - min(edges) > 1:
                    problems.append(f"keys: the keycaps of a column end at {min(edges)} to {max(edges)}")
            fill = dominant(keys)
            if not near(fill, palette["surface"]):
                problems.append(f"tokens keys: the key list fill {fill} is not `surface` {palette['surface']}")
    if cell not in ("fullscreen", "keys", "toast", "layout", "tooltip", "launcher", "chattoast"):
        if not helpers:
            problems.append(f"{cell}: no helper surface on the screen")
        for name, rect in helpers.items():
            fill = dominant(rect)
            if not any(near(fill, token) for key, token in palette.items() if key != "line"):
                problems.append(f"tokens {name}: fill {fill} is no token of {theme}")
            edge = px(rect[0] + rect[2] / 2, rect[1])
            if not any(near(edge, token, 18) for token in (palette["line"], fill)):
                problems.append(f"tokens {name}: hairline {edge} is no `line` {palette['line']}")
            steps = corner(rect)
            if not 3 <= steps <= 8:
                problems.append(f"radius {name}: the corner turns in {steps} px, not the radius family of 10 to 14")
        borders = tiles(nodes)
        want = palette["line"]
        edge = [px(x, rect[1] + rect[3] / 2) for rect in borders.values() for x in (rect[0] + 1, rect[0] + rect[2] - 2)]
        low = tuple(0.8 * c - 6 for c in want)
        high = tuple((1.1 if scale == int(scale) else 1.3) * c + 6 for c in want)
        if borders and cell not in ("chat", "chatq") and not any(all(a <= c <= b for a, c, b in zip(low, color, high)) for color in edge):
            problems.append(f"tokens tile border: {edge} is not `line` {want}")
        for name, rect in helpers.items():
            if bar and rect[1] < BAR[1] + 8 and name != "hud-chat":
                problems.append(f"overlap {name} reaches the bar: top {rect[1]}")
            if bar and name == "hud-chat" and rect[1] < BAR[1] + GAP:
                problems.append(f"overlap {name} sticks to the bar: top {rect[1]}")
            if name == "hud-chat" or cell == "chatq":
                continue
            for tile, t in borders.items():
                x, y, w, h = rect
                inside = x >= t[0] + GAP and y >= t[1] + GAP and x + w <= t[0] + t[2] - GAP and y + h <= t[1] + t[3] - GAP
                touches = not (x + w < t[0] - GAP or x > t[0] + t[2] + GAP or y + h < t[1] - GAP or y > t[1] + t[3] + GAP)
                if touches and not inside:
                    problems.append(f"overlap {name} crosses the border of {tile}")
    if cell == "launcher":
        hits = sum(px(x, y) == palette["surface"] for x in range(500, 780, 4) for y in range(300, 500, 4))
        if hits < 100:
            problems.append(f"launcher: only {hits} samples of the middle are the `surface` fill: no opaque launcher in the palette")
    if cell == "fullscreen" and bar:
        window = px(640, 400)
        for x in (33, 66, 640, 1100, 1240):
            if not near(px(x, 24), window, 6):
                problems.append(f"fullscreen: the bar shows over the window at x {x}")
                break
    if cell == "desktop":
        for tile, rect in tiles(nodes).items():
            fill = px(rect[0] + rect[2] / 2, rect[1] + rect[3] / 2)
            if not near(fill, TERMINAL[theme], 6):
                problems.append(f"terminal {tile}: the fill {fill} is not the `terminal` token {TERMINAL[theme]}")
    if "wall" in flags and cell in ("desktop", "keys"):
        problems.extend(look(rows, nodes, theme, scale, cell))
    if cell == "keys" and bar and "hud-row-chat" in named:
        below, island = dominant(named["hud-row-chat"]), px(570, 24)
        if sum(abs(a - b) for a, b in zip(below, island)) > 14:
            problems.append(f"scrim: the bar {island} and the surface below the key list {below} differ, the scrim does not treat them alike")
    for problem in problems:
        print("FAIL:", problem)
    return 1 if problems else 0


sys.exit(main())
