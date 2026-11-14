#!/usr/bin/python3
"""Print the contrast ratio of the text in a rect of an 8-bit RGBA PNG.

The background is the most common colour of the rect, the text is the
colour farthest from it. The ratio is the one of WCAG 2: (L1 + 0.05) /
(L2 + 0.05) with L1 the lighter. Use a rect of one line of text.

Usage: tests/png-contrast.py FILE X Y W H
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


def luminance(rgb):
    """The relative luminance of an sRGB colour."""
    lin = [(c / 255 / 12.92) if c <= 10 else (((c / 255) + 0.055) / 1.055) ** 2.4 for c in rgb]
    return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2]


def contrast(path, x, y, w, h):
    """The contrast ratio of the text in the rect and its colours."""
    lines = list(png.rows(pathlib.Path(path).read_bytes()))
    pixels = [
        tuple(lines[r][c * 4:c * 4 + 3])
        for r in range(y, y + h)
        for c in range(x, x + w)
    ]
    back = Counter(pixels).most_common(1)[0][0]
    fore = max(pixels, key=lambda p: sum((a - b) ** 2 for a, b in zip(p, back)))
    hi, lo = sorted((luminance(back), luminance(fore)), reverse=True)
    return (hi + 0.05) / (lo + 0.05), back, fore


def main():
    """Print the ratio and the two colours."""
    path = sys.argv[1]
    x, y, w, h = (int(v) for v in sys.argv[2:6])
    ratio, back, fore = contrast(path, x, y, w, h)
    print(f"{ratio:.2f} text {fore} on {back}")


if __name__ == "__main__":
    main()
