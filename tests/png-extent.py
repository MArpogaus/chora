#!/usr/bin/python3
"""Print the first and the last column of white ink in a rect of an 8-bit RGBA PNG.

White is a pixel whose three channels are all 200 or more (a white caption;
its shadow and its backing are darker). The columns are absolute, in the
coordinates of the PNG. Prints "none" when the rect holds no white.

Usage: tests/png-extent.py FILE X Y W H
"""

import importlib.util
import pathlib
import sys

spec = importlib.util.spec_from_file_location(
    "png_pixel", pathlib.Path(__file__).with_name("png-pixel.py")
)
png = importlib.util.module_from_spec(spec)
spec.loader.exec_module(png)

WHITE = 200


def extent(path, x, y, w, h):
    """The first and last column with white ink in the rect, or None."""
    lines = list(png.rows(pathlib.Path(path).read_bytes()))
    cols = [
        c
        for c in range(x, x + w)
        if any(all(v >= WHITE for v in lines[r][c * 4:c * 4 + 3]) for r in range(y, y + h))
    ]
    return (cols[0], cols[-1]) if cols else None


if __name__ == "__main__":
    file, *rect = sys.argv[1:6]
    found = extent(file, *map(int, rect))
    print("none" if found is None else f"{found[0]} {found[1]}")
