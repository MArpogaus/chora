#!/usr/bin/python3
"""Print the rows of a column of an XP edit box in a capture.

Usage: tests/xp-field-rows.py FILE X SCALE
Prints "TOP HEIGHT" in logical pixels: the first row of the column X
whose pixel is the border colour of an XP edit box (127 157 185) and the
height from there to the last such row, or "none".
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
    path, x, scale = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    hits = []
    for r, line in enumerate(module.rows(pathlib.Path(path).read_bytes())):
        at = x * scale * 4
        if tuple(line[at:at + 3]) == BORDER:
            hits.append(r)
    if not hits:
        print("none")
        return
    print(hits[0] // scale, (hits[-1] - hits[0] + 1) // scale)


main()
