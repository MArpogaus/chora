#!/usr/bin/python3
"""Print the baselines of the label and of the typed text of the XP Run box.

Usage: tests/xp-runbox-baseline.py FILE SCALE
The input row of fuzzel (no box, see docs/xp-style.md 13) holds "Open:"
and the text typed after it. Prints "LABEL TEXT" in logical pixels: the
lowest ink row of the first glyph of each (glyphs are runs of ink columns,
the label and the text stand more than 4 px apart), counted from the inner edge of
the top border (0 84 227), or "none". Ink is a pixel darker than a sum
of 200 (the text colour is black on the face colour).
"""

import importlib.util
import pathlib
import sys

BORDER = (0, 84, 227)
SELECTION = (49, 106, 197)


def main():
    here = pathlib.Path(__file__).parent
    spec = importlib.util.spec_from_file_location("png_pixel", here / "png-pixel.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    path, scale = sys.argv[1], int(sys.argv[2])
    rows = list(module.rows(pathlib.Path(path).read_bytes()))
    px = lambda y, x: tuple(rows[y][x * 4:x * 4 + 3])
    sel = [(y, x) for y, r in enumerate(rows) for x in range(len(r) // 4) if px(y, x) == SELECTION]
    if not sel:
        print("none")
        return
    top = min(y for y, _ in sel)
    left = min(x for _, x in sel)
    mid = (top + max(y for y, _ in sel)) // 2
    edge = left
    while edge > 0 and px(mid, edge - 1) != BORDER:
        edge -= 1
    inner = edge + 2 * scale
    box_top = top
    while box_top > 0 and px(box_top - 1, inner + 4 * scale) != BORDER:
        box_top -= 1
    first, last = box_top + 2 * scale, top
    dark = lambda y, x: sum(px(y, x)) < 200
    columns = [x for x in range(inner, left + 400 * scale)
               if x < len(rows[first]) // 4 and any(dark(y, x) for y in range(first, last))]
    if not columns:
        print("none")
        return
    groups, run = [], [columns[0]]
    for x in columns[1:]:
        if x - run[-1] > 1:
            groups.append(run)
            run = []
        run.append(x)
    groups.append(run)
    words, word = [], [groups[0]]
    for g in groups[1:]:
        if g[0] - word[-1][-1] > 4 * scale:
            words.append(word)
            word = []
        word.append(g)
    words.append(word)
    if len(words) < 2:
        print("none")
        return
    low = lambda g: max(y for y in range(first, last) if any(dark(y, x) for x in g)) + 1
    base = lambda g: (low(g) - (box_top + 2 * scale)) / scale
    print(base(words[0][0]), base(words[1][0]))


main()
