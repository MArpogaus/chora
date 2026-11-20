#!/usr/bin/python3
"""Print how many pixels differ between two captures in a region.

Usage: tests/capture-diff.py A.png B.png X0 Y0 X1 Y1
The region is in pixels of the captures; the captures need the same size.
"""

import importlib.util
import pathlib
import sys


def main():
    here = pathlib.Path(__file__).parent
    spec = importlib.util.spec_from_file_location("png_pixel", here / "png-pixel.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    a, b = sys.argv[1:3]
    x0, y0, x1, y1 = (int(v) for v in sys.argv[3:7])
    rows_a = list(module.rows(pathlib.Path(a).read_bytes()))
    rows_b = list(module.rows(pathlib.Path(b).read_bytes()))
    if len(rows_a) != len(rows_b):
        print("size")
        return
    count = 0
    for y in range(y0, min(y1, len(rows_a))):
        ra, rb = rows_a[y], rows_b[y]
        count += sum(1 for x in range(x0, x1) if ra[x * 4:x * 4 + 3] != rb[x * 4:x * 4 + 3])
    print(count)


main()
