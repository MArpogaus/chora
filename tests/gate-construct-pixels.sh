#!/bin/bash
# Gate of the construct pixel review: what only a frame shows, read from the captures of tests/construct-captures.sh.
#
# 1. The panes beside the focus have a label: in the walk with three
#    panes, the strips under the left and the right pane hold the dark
#    pixels of a name.
# 2. A cube stood back with a pane beside it shows the pane, whole and
#    with its label.
# 3. A face stepped into at 2x is not black: the frame has more than a
#    capsule of light in it. Its capsule has one rim and stands clear of
#    the taskbar.
# 4. No pane of the walk or beside a cube is cut by the screen, and the
#    mirror image under a pane fades out at its ends.
#
# Usage: tests/gate-construct-pixels.sh   needs target/debug (cargo build
# --workspace), python3 with PIL. LIT_TRIES=1 turns the wait for the
# paint off: the 2x cell must then fail.
# shellcheck disable=SC2016,SC2034 # cell evaluates its code later
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failed=0
cell() {
    if eval "$2"; then
        echo "ok: $1"
    else
        echo "FAIL: $1"
        failed=1
    fi
}
count() {
    python3 -I -c '
import sys
from PIL import Image
f, x0, y0, x1, y1, m = sys.argv[1], *map(int, sys.argv[2:])
im = Image.open(f).convert("L").crop((x0, y0, x1, y1))
print(sum(1 for p in im.getdata() if p < m))' "$@" 2>/dev/null
}
lum() {
    python3 -I -c '
import sys
from PIL import Image
print(Image.open(sys.argv[1]).convert("L").getpixel((int(sys.argv[2]), int(sys.argv[3]))))' "$@" 2>/dev/null
}
slope() {
    python3 -I -c '
import sys
from PIL import Image
f, x0, x1, y0, y1 = sys.argv[1], *map(int, sys.argv[2:])
im = Image.open(f).convert("L").load()
row = [sum(im[x, y] for y in range(y0, y1)) / (y1 - y0) for x in range(x0, x1)]
print(round(max(abs(row[i + 3] - row[i]) for i in range(len(row) - 3))))' "$@" 2>/dev/null
}
mean() {
    python3 -I -c '
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("L").resize((64, 40))
d = list(im.getdata())
print(round(sum(d) / len(d)))' "$1" 2>/dev/null
}

"$repo/tests/construct-captures.sh" "$work" 1 walk-3-panes cube-stood-back-with-pane stepped-into-face >/dev/null 2>&1
"$repo/tests/construct-captures.sh" "$work" 2 stepped-into-face >/dev/null 2>&1

walk=$work/r4-walk-3-panes-1x.png
cell "the pane at the left has a label under it" '[ "$(count "$walk" 0 493 330 526 150)" -ge 20 ] && [ "$(count "$walk" 0 493 330 526 150)" -le 1000 ]'
cell "the pane at the right has a label under it" '[ "$(count "$walk" 950 493 1280 526 150)" -ge 20 ] && [ "$(count "$walk" 950 493 1280 526 150)" -le 1000 ]'
cell "no pane of the walk is cut by the edge of the screen" '[ "$(count "$walk" 0 200 2 600 130)" = 0 ] && [ "$(count "$walk" 1278 200 1280 600 130)" = 0 ]'
cell "the mirror image under the pane ends in a soft edge (slope $(slope "$walk" 340 440 714 720))" '[ "$(slope "$walk" 340 440 714 720)" -le 5 ]'
beside=$work/r4-cube-stood-back-with-pane-1x.png
cell "a pane stands whole beside the cube stood back" '[ "$(count "$beside" 900 280 1280 480 130)" -ge 5000 ] && [ "$(count "$beside" 0 200 2 600 130)" = 0 ] && [ "$(count "$beside" 1278 200 1280 600 130)" = 0 ]'
cell "the pane beside the cube has its label, under its bottom edge" '[ "$(count "$beside" 900 495 1280 531 150)" -ge 20 ] && [ "$(count "$beside" 900 495 1280 531 150)" -le 1000 ]'
face=$work/r4-stepped-into-face-2x.png
cell "a face stepped into at 2x is not black (mean $(mean "$face"))" '[ "$(mean "$face")" -ge 40 ]'
flown=$work/r4-stepped-into-face-1x.png
cell "the capsule over the face stands clear of the taskbar" '[ "$(lum "$flown" 640 712)" -le 60 ] && [ "$(lum "$flown" 640 756)" -ge 100 ]'
cell "the capsule over the face has one rim: no light line on its top edge" '[ "$(lum "$flown" 640 705)" -le $(($(lum "$flown" 640 712) + 6)) ]'

[ "$failed" = 0 ] || exit 1
echo "PASS: construct pixels"
