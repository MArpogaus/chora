#!/bin/bash
# The bar of the tiling flavor in Ink (tiling-v2, item 1): three floating
# islands, 32 px high, 8 px from the top, 12 px from the sides, radius 10.
# A 1280x800 capture is measured: the islands, their margins and a rounded
# corner. colors.css keeps 4.5:1 for every label on the surface, except the
# empty tags: they are inactive controls at 50 % muted, which WCAG 1.4.3
# exempts.
#
# Usage: tests/gate-tiling-bar.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need_bins chora choractl chora-tiling
flavor=$repo/flavors/tiling

python3 -I - "$flavor/waybar/colors.css" <<'PY' || fail "contrast"
import re, sys
c = dict(re.findall(r"@define-color\s+([\w-]+)\s+(#[0-9a-fA-F]{6})", open(sys.argv[1]).read()))
def lum(h):
    v = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    v = [x / 12.92 if x <= 0.03928 else ((x + 0.055) / 1.055) ** 2.4 for x in v]
    return 0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]
def ratio(a, b):
    x, y = sorted((lum(c[a]), lum(c[b])), reverse=True)
    return (x + 0.05) / (y + 0.05)
for fg, bg in [("text", "surface"), ("muted", "surface"), ("accent", "surface"),
               ("urgent", "surface"), ("on-accent", "accent"), ("text", "surface-2")]:
    r = ratio(fg, bg)
    assert r >= 4.5, f"{fg} on {bg}: {r:.2f}"
PY
ok "every label of colors.css passes 4.5:1"

temp_config
session_start 1280x800 -- "$bin/chora-tiling"
PATH=$bin:$PATH CHORA_FLAVOR_DIR=$flavor setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/tmp/claude-0/wb.log 2>&1 &
run_pid $!
shot=$(mktemp /tmp/gate-bar.XXXXXX.png)
tmp_dirs+=("$shot")
grant screen.capture
for _ in $(seq 40); do
    sleep 0.5
    ctl capture --output HEADLESS-1 "$shot" >/dev/null 2>&1 || continue
    [ "$(pixel "$shot" 640 24)" != "$(pixel "$shot" 640 300)" ] && break
done
bg=$(pixel "$shot" 640 300)
isbar() { [ "$(pixel "$shot" "$1" "$2")" != "$bg" ]; }
isbar 20 24 || fail "no bar island at 20,24 ($(pixel "$shot" 20 24)), background $bg"
isbar 11 24 && fail "the left island starts before 12"
isbar 12 24 || fail "the left island does not start at 12"
isbar 1267 24 || fail "the right island does not end at 1268"
isbar 1268 24 && fail "the right island ends after 1268"
isbar 20 7 && fail "the island starts above 8"
isbar 20 8 || fail "the island does not start at 8"
isbar 20 39 || fail "the island does not reach 39"
isbar 20 40 && fail "the island is higher than 32"
isbar 12 8 && fail "no rounded corner at 12,8"
isbar 640 24 || fail "no centre island"
isbar 400 24 && isbar 450 24 && fail "the gap between the left and the centre island is filled"
ok "three islands, 32 high, 8 from the top, 12 from the sides, corner radius"
