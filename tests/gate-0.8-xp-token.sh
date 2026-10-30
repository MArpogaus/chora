#!/bin/bash
# Gate check 3 of the 0.8 milestone: the start menu of chora-xp starts
# Thunar, and the `surface.created` event of its window carries the
# activation token of `choractl run` (D42). Not tests/gate-0.8-3.sh, which
# is a check of the 0.7 gate. CI does not run this script yet.
#
# Step 13 of tests/xp.sh records the events of the file manager in
# OUT_DIR/fm-created.jsonl. This script reads that file, and runs
# tests/xp.sh into OUT_DIR first when the file does not exist.
#
# Usage: tests/gate-0.8-xp-token.sh [OUT_DIR]   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need thunar
need jq
out=${1:-$(mktemp -d)}
mkdir -p "$out"
log=$out/fm-created.jsonl
if [ ! -f "$log" ]; then
    "$repo/tests/xp.sh" "$bin" "$out" >"$out/xp.log" 2>&1 ||
        fail "tests/xp.sh: $(tail -n 3 "$out/xp.log")"
fi
jq -e 'select(.event == "surface.created" and (.app_id // "" | ascii_downcase) == "thunar"
    and (.token // "") != "")' "$log" >/dev/null ||
    fail "no surface.created of thunar with a token: $log"
ok "the thunar window of the start menu carries its activation token"
