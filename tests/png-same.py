#!/usr/bin/python3
"""Compare two 8-bit RGBA PNG files of the same size.

Print the number of pixels that differ and the largest difference of a
channel.

Usage: tests/png-same.py A B
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
    if len(a) != len(b) or len(a[0]) != len(b[0]):
        sys.exit(f"sizes differ: {len(a[0]) // 4}x{len(a)} and {len(b[0]) // 4}x{len(b)}")
    pixels, worst = 0, 0
    for ra, rb in zip(a, b):
        if ra == rb:
            continue
        for i in range(0, len(ra), 4):
            d = max(abs(x - y) for x, y in zip(ra[i : i + 4], rb[i : i + 4]))
            pixels += d > 0
            worst = max(worst, d)
    print(pixels, worst)


if __name__ == "__main__":
    main()
