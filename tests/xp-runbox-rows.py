#!/usr/bin/python3
"""Print the geometry of the XP Run box (fuzzel) in a capture.

Usage: tests/xp-runbox-rows.py FILE SCALE
Prints "ROW_HEIGHT TEXT_INSET LIST_INSET" in logical pixels, or "none":
the height of the highlighted row (selection colour 49 106 197), the
distance from the inner edge of the 2 px border (0 84 227) to the first
dark pixel of the edit row ("Open:"), and to the left edge of the rows.
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
    bottom = max(y for y, _ in sel)
    left = min(x for _, x in sel)
    mid = (top + bottom) // 2
    edge = left
    while edge > 0 and px(mid, edge - 1) != BORDER:
        edge -= 1
    if edge == 0:
        print("none")
        return
    inner = edge + 2 * scale
    box_top = top
    while box_top > 0 and px(box_top - 1, inner + 4 * scale) != BORDER:
        box_top -= 1
    dark = [x for y in range(box_top + 2 * scale, top) for x in range(inner, left + 400 * scale)
            if x < len(rows[y]) // 4 and sum(px(y, x)) < 200]
    if not dark:
        print("none")
        return
    print((bottom - top + 1) / scale, (min(dark) - inner) / scale, (left - inner) / scale)


main()
