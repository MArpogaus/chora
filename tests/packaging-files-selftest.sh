#!/bin/bash
# Self-test of tests/packaging-files.py: it flags a plugin HUD folder that
# the PKGBUILD of d7be70b9 left out (the tiling HUD) and passes at HEAD.
#
# Usage: tests/packaging-files-selftest.sh
set -u

repo=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo" || exit 2
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

git show d7be70b9:dist/arch/PKGBUILD >"$work/PKGBUILD" || { echo "FAIL: d7be70b9 is not in the history"; exit 1; }
if tests/packaging-files.py --pkgbuild "$work/PKGBUILD" >"$work/old.out" 2>&1; then
    echo "FAIL: packaging-files.py passes the PKGBUILD without the tiling HUD"
    failures=$((failures + 1))
elif ! grep -q 'plugins/chora-assistant/huds/tiling/hud.yaml' "$work/old.out"; then
    echo "FAIL: packaging-files.py does not name the tiling HUD: $(cat "$work/old.out")"
    failures=$((failures + 1))
fi
if ! tests/packaging-files.py >"$work/head.out" 2>&1; then
    echo "FAIL: packaging-files.py fails at HEAD: $(cat "$work/head.out")"
    failures=$((failures + 1))
fi
[ "$failures" = 0 ] || exit 1
echo "ok: packaging-files-selftest"
