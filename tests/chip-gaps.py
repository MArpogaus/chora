#!/usr/bin/python3 -I
"""Print "LEFT RIGHT WIDTH" of the layout chip of a bar capture: the gaps in px
between the chip edges and the ink of its word, and the chip width.

Usage: tests/chip-gaps.py FILE R G B   (R G B: the chip colour, surface-2)

The chip is the last run of the chip colour on y 16 and 32 left of the centre
island; its rounded corners cost both sides alike, so the gaps stay symmetric.
"""
import importlib.util
import os
import sys

here = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("png_pixel", os.path.join(here, "png-pixel.py"))
pp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pp)

path, chip = sys.argv[1], tuple(int(v) for v in sys.argv[2:5])
rows = {}
for r, line in enumerate(pp.rows(open(path, "rb").read())):
    if 16 <= r <= 32:
        rows[r] = [tuple(line[i:i + 3]) for i in range(0, len(line), 4)]
width = len(rows[16])
xs = [i for i in range(min(width, 640)) if rows[16][i] == chip and rows[32][i] == chip]
lo = hi = xs[-1]
while lo - 1 in xs:
    lo -= 1
ink = [i for i in range(lo, hi + 1) if any(rows[r][i] != chip for r in range(18, 31))]
print(min(ink) - lo, hi - max(ink), hi - lo + 1)
