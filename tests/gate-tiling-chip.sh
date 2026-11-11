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
sleep 1.5
grep -q "Tile left" <<<"$(list)" && fail "the chip stays longer than 700 ms"
ok "the chip leaves"
[ "$(ctl call tiling get_layout | grep -c tile_left)" = 1 ] || fail "the layout did not change"

keys Super+space
sleep 1
grep -q fuzzel "$fake/marks" 2>/dev/null || fail "Super+space did not run the launcher"
ok "Super+space runs the launcher"
