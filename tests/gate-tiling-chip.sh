#!/bin/bash
# The layout chip of tiling (tiling-v2, item 7): Super+Shift+space goes to
# the next layout and a chip names it, "Tile left", for 700 ms; the same
# for the method and the click on the layout word. Super+space runs the
# launcher in every look (PM, taste decisions), so no key of the two looks
# dead.
#
# Usage: tests/gate-tiling-chip.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling
fake=$(mktemp -d /tmp/gate-chip.XXXXXX)
printf '#!/bin/sh\necho fuzzel >>"%s/marks"\n' "$fake" >"$fake/fuzzel"
chmod +x "$fake/fuzzel"
export PATH=$fake:$bin:$PATH
temp_config
tmp_dirs+=("$fake")
session_start 1280x800 -- "$bin/chora-tiling"
grant input.inject
keys() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject - >/dev/null || fail "the key $1"; }
list() { ctl query 'kind:text' 2>/dev/null; }

keys Super+Shift+space
sleep 0.3
grep -q "Tile left" <<<"$(list)" || fail "no chip names the new layout: $(list | head -c 300)"
ok "Super+Shift+space shows the chip 'Tile left'"
ctl query 'owner:tiling' | python3 -I -c '
import json, sys
by = {n["id"]: next(iter(n["rects"].values())) for n in json.load(sys.stdin) if n.get("rects") and n["id"].startswith("tiling-chip")}
glass, word = by["tiling-chip-glass"], by["tiling-chip-word"]
for axis, name in ((0, "horizontally"), (1, "vertically")):
    off = (word[axis] + word[axis + 2] / 2) - (glass[axis] + glass[axis + 2] / 2)
    if abs(off) > 1.0:
        sys.exit("the word is %.1f px off the centre of the chip, %s" % (off, name))
' || fail "the word is not centred in the chip"
ok "the word is centred in the chip, on both axes"
sleep 1.5
grep -q "Tile left" <<<"$(list)" && fail "the chip stays longer than 700 ms"
ok "the chip leaves"
[ "$(ctl call tiling get_layout | grep -c tile_left)" = 1 ] || fail "the layout did not change"

keys Super+space
sleep 1
grep -q fuzzel "$fake/marks" 2>/dev/null || fail "Super+space did not run the launcher"
ok "Super+space runs the launcher"
