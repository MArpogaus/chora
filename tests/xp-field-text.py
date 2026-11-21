#!/usr/bin/python3
"""Print the geometry of an XP text field and of the label beside it.

Usage: tests/xp-field-text.py FILE SCALE X Y W H [LX0 LX1]
X Y W H is the field in logical pixels, LX0 LX1 the columns of its label.
Prints "HEIGHT THICKNESS PAD BASELINE LABEL_BASELINE" in logical pixels:
the height from the first to the last row of the border colour (127 157
185) in a column of the field, the thickness of its top edge, the distance
from the inner edge of the border to the first ink of the text, and the
lowest ink row of the text and of the label counted from the top edge of
the field ("-" without a label). Ink is a pixel darker than a sum of 600.
"""

import importlib.util
import pathlib
import sys

BORDER = (127, 157, 185)


def main():
    here = pathlib.Path(__file__).parent
    spec = importlib.util.spec_from_file_location("png_pixel", here / "png-pixel.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    path, scale = sys.argv[1], int(sys.argv[2])
    x, y, w, h = (int(v) for v in sys.argv[3:7])
    label = (int(sys.argv[7]), int(sys.argv[8])) if len(sys.argv) > 8 else None
    rows = list(module.rows(pathlib.Path(path).read_bytes()))
    px = lambda r, c: tuple(rows[r][c * 4:c * 4 + 3])
    col = (x + 3) * scale
    hits = [r for r in range((y - 3) * scale, (y + h + 3) * scale) if px(r, col) == BORDER]
    if not hits:
        print("none")
        return
    top, bottom = hits[0], hits[-1]
    thick = 1
    while px(top + thick, col) == BORDER:
        thick += 1
    ink = lambda r, c: sum(px(r, c)) < 600
    inner = (x * scale) + scale
    text = [(r, c) for r in range(top + scale, bottom - scale + 1)
            for c in range(inner + 1, (x + w - 20) * scale) if ink(r, c)]
    if not text:
        print("none")
        return
    base = (max(r for r, _ in text) + 1 - top) / scale
    pad = (min(c for _, c in text) - inner) / scale
    mark = "-"
    if label:
        found = [r for r in range(top - 3 * scale, bottom + 3 * scale)
                 for c in range(label[0] * scale, label[1] * scale) if ink(r, c)]
        if found:
            mark = (max(found) + 1 - top) / scale
    print((bottom - top + 1) / scale, thick / scale, pad, base, mark)


main()
