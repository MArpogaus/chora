#!/usr/bin/python3
"""Compare two 8-bit RGBA PNG files of the same size.

Print the number of pixels that differ and the largest difference of a
channel. Each `--outside X,Y,W,H` leaves the pixels of that rectangle out,
for an element that shows over one of them only (the resting hint of a
pane, co-76).

Usage: tests/png-same.py A B [--outside X,Y,W,H]...
"""

import importlib.util
import pathlib
import sys

spec = importlib.util.spec_from_file_location(
    "png_pixel", pathlib.Path(__file__).with_name("png-pixel.py")
)
png = importlib.util.module_from_spec(spec)
spec.loader.exec_module(png)


def main():
    """Compare the files of the command line."""
    a, b = (list(png.rows(pathlib.Path(p).read_bytes())) for p in sys.argv[1:3])
    rest = sys.argv[3:]
    if len(rest) % 2 or any(flag != "--outside" for flag in rest[::2]):
        sys.exit("usage: tests/png-same.py A B [--outside X,Y,W,H]...")
    boxes = [[float(v) for v in box.split(",")] for box in rest[1::2]]

    def kept(x, y):
        return not any(bx <= x < bx + bw and by <= y < by + bh for bx, by, bw, bh in boxes)

    if len(a) != len(b) or len(a[0]) != len(b[0]):
        sys.exit(f"sizes differ: {len(a[0]) // 4}x{len(a)} and {len(b[0]) // 4}x{len(b)}")
    pixels, worst = 0, 0
    for y, (ra, rb) in enumerate(zip(a, b)):
        if ra == rb:
            continue
        for i in range(0, len(ra), 4):
            if not kept(i // 4, y):
                continue
            d = max(abs(x - y) for x, y in zip(ra[i : i + 4], rb[i : i + 4]))
            pixels += d > 0
            worst = max(worst, d)
    print(pixels, worst)


if __name__ == "__main__":
    main()
