#!/bin/bash
# The captures of the tiling look for a review: runs tests/gate-tiling-audit.sh
# with the wallpaper and keeps the capture of each state as
# CELL-THEME-SIZE.png in OUT (desktop, question, chat, fullscreen, keys,
# toast) at each theme and scale, e.g. desktop-ink-1280x800.png and
# desktop-paper-2560x1600@2.png. The audit runs on the same captures.
#
# Usage: tests/capture-tiling.sh OUT
#   SIZES, THEMES, CELLS  as in tests/gate-tiling-audit.sh
set -u
out=${1:?usage: tests/capture-tiling.sh OUT}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
shots=$(mktemp -d /tmp/capture-tiling.XXXXXX)
SHOTS=$shots "$here/gate-tiling-audit.sh"
status=$?
for f in "$shots"/audit-*.png; do
    [ -e "$f" ] || continue
    name=$(basename "$f")
    cp "$f" "$out/${name#audit-}"
done
rm -rf "$shots"
exit $status
