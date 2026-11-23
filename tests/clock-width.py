#!/usr/bin/env python3
"""Width bound of the bar clock for tests/gate-tiling-bar-spec.sh.

The clock shows "%a %d %b %H:%M" (flavors/tiling/waybar/config.jsonc), so its
width in Inter 14 px weight 500 follows the date and the time (107 to 124 px).
bounds(times) gives the (low, high) width in px of the text of each time in
TIMES (epoch seconds, the capture may span a minute change), +-3 px for the
rounding of the ink. The selftest holds the widest text of the year, every
minute of every day, in the bound of that very text: the fixed 100-119 of the
first gate misses it (ti-37, "Sun 22 Nov 00:36" is 120.0 wide, the widest is
"Wed 06 May 04:44" at 128.7). The mono face is 134.75 px wide for every text, above the
bound of any date.

Usage: clock-width.py EPOCH...   prints "LOW HIGH"
       clock-width.py --selftest
"""
import sys
import time

INTER = "/usr/share/fonts/opentype/inter/Inter-Medium.otf"
MONO = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
FORMAT = "%a %d %b %H:%M"
SLACK = 3
FIXED_OLD = (100, 119)


def text_width(font_path, text):
    from PIL import ImageFont
    return ImageFont.truetype(font_path, 14).getlength(text)


def bounds(times):
    widths = [text_width(INTER, time.strftime(FORMAT, time.localtime(t))) for t in times]
    return min(widths) - SLACK, max(widths) + SLACK


def selftest():
    bad = 0
    for day in range(0, 366, 1):
        for hour in (0, 11, 20):
            t = time.mktime((2026, 1, 1 + day, hour, 8, 0, 0, 0, -1))
            lo, hi = bounds([t])
            text = time.strftime(FORMAT, time.localtime(t))
            if not lo <= text_width(INTER, text) <= hi:
                print(f"FAIL: Inter {text} is outside {lo:.1f}-{hi:.1f}")
                bad += 1
            if lo <= text_width(MONO, text) <= hi:
                print(f"FAIL: the mono face {text} passes {lo:.1f}-{hi:.1f}")
                bad += 1
    days = [time.localtime(time.mktime((2026, 1, 1 + d, 12, 0, 0, 0, 0, -1))) for d in range(366)]
    date = max((time.strftime("%a %d %b ", d) for d in days), key=lambda x: text_width(INTER, x))
    hour = max((f"{h:02d}:{m:02d}" for h in range(24) for m in range(60)), key=lambda x: text_width(INTER, x))
    widest = date + hour
    t = time.mktime(time.strptime("2026 " + widest, "%Y " + FORMAT))
    lo, hi = FIXED_OLD
    if lo <= text_width(INTER, widest) <= hi:
        print(f"FAIL: the widest text {widest} fits the old fixed {lo}-{hi}")
        bad += 1
    lo, hi = bounds([t])
    if not lo <= text_width(INTER, widest) <= hi:
        print(f"FAIL: the widest text {widest} ({text_width(INTER, widest):.1f}) is outside {lo:.1f}-{hi:.1f}")
        bad += 1
    t = time.mktime((2026, 11, 22, 0, 36, 0, 0, 0, -1))
    lo, hi = bounds([t])
    if not lo <= 120.0 <= hi:
        print(f"FAIL: 120.0 is outside {lo:.1f}-{hi:.1f} for Sun 22 Nov 00:36")
        bad += 1
    print("ok: clock-width bound" if not bad else f"{bad} failures")
    return 1 if bad else 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    lo, hi = bounds([float(a) for a in sys.argv[1:]])
    print(f"{lo} {hi}")
