#!/usr/bin/python3
"""Print the heights of the lines of ink in a rect of an 8-bit RGBA PNG.

The background is the most common colour of the rect; a row is ink when a
pixel of it is far from the background. The heights are those of the runs
of ink rows, from the top, so a line of text with its ascenders and
descenders is one number.

Usage: tests/png-ink.py FILE X Y W H
"""

import importlib.util
import pathlib
import sys
from collections import Counter

spec = importlib.util.spec_from_file_location(
    "png_pixel", pathlib.Path(__file__).with_name("png-pixel.py")
)
png = importlib.util.module_from_spec(spec)
spec.loader.exec_module(png)

FAR = 120


def runs(path, x, y, w, h):
    """The heights of the runs of ink rows in the rect."""
    lines = list(png.rows(pathlib.Path(path).read_bytes()))
    area = [
        [tuple(lines[r][c * 4:c * 4 + 3]) for c in range(x, x + w)]
        for r in range(y, y + h)
    ]
    back = Counter(p for row in area for p in row).most_common(1)[0][0]
    found, run = [], 0
    for row in area:
        ink = any(sum(abs(a - b) for a, b in zip(p, back)) > FAR for p in row)
        if ink:
            run += 1
        elif run:
            found.append(run)
            run = 0
    if run:
        found.append(run)
    return found


def main():
    """Print the heights."""
    path = sys.argv[1]
    x, y, w, h = (int(v) for v in sys.argv[2:6])
    print(*runs(path, x, y, w, h))


if __name__ == "__main__":
    main()
