#!/usr/bin/python3
"""Print where the table of keys of the Help window ends in a capture.

Usage: tests/xp-help-rows.py FILE RULE_X TEXT_X TOP HEIGHT
The rows of the table are parted by a rule of 1 px (230 228 219), seen in
the column RULE_X; TEXT_X is the left edge of the column of the words.
Within the pane (TOP, HEIGHT, logical pixels of scale 1) prints "LAST INK BELOW":
the row of the last rule, the number of rows below it with ink (a pixel
darker than a sum of 300) in the words column, and the rows from it to the
bottom of the pane. "none" without a rule.
"""

import importlib.util
import pathlib
import sys

RULE = (230, 228, 219)


def main():
    here = pathlib.Path(__file__).parent
    spec = importlib.util.spec_from_file_location("png_pixel", here / "png-pixel.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    path = sys.argv[1]
    rule_x, text_x, top, height = (int(v) for v in sys.argv[2:6])
    rows = list(module.rows(pathlib.Path(path).read_bytes()))
    px = lambda y, x: tuple(rows[y][x * 4:x * 4 + 3])
    rules = [y for y in range(top, top + height) if px(y, rule_x) == RULE]
    if not rules:
        print("none")
        return
    last = rules[-1]
    ink = sum(1 for y in range(last + 1, top + height)
              if any(sum(px(y, x)) < 300 for x in range(text_x, text_x + 150)))
    print(last, ink, top + height - 1 - last)


main()
