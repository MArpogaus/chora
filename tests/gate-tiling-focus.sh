#!/bin/bash
# The focus mark of tiling (tiling-v2, item 5): a 2 px border around each
# window, outside it, with the radius 12; the focused window has the accent
# gradient, the others the line colour. After Super+j only the new window
# carries the accent. Over a white and a black window the border is
# unbroken, at the scale 1 and at the scale 2.
#
# Usage: tests/gate-tiling-focus.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need_bins chora choractl chora-tiling chora-probe
temp_config
scale=${1:-1}
session_start "$((1280 * scale))x$((800 * scale))@$scale" -- "$bin/chora-tiling"
grant screen.capture input.inject
shot=$XDG_CONFIG_HOME/focus.png

window() {
    started=$(ctl run "$bin/chora-probe" window "$@") || fail "the run of the window"
    pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
    [ -n "$pid" ] || fail "the run of the window"
    run_pid "$pid"
}
px() { pixel "$shot" $(($1 * scale)) $(($2 * scale)); }
snap() {
    sleep $((2 * scale + 1))
    ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
}
accented() {
    read -r r _ b _ < <(px "$1" "$2")
    [ "$b" -gt 150 ] && [ "$r" -gt 60 ]
}
lined() {
    read -r r _ b _ < <(px "$1" "$2")
    [ "$r" -lt 80 ] && [ "$b" -gt 40 ] && [ "$b" -lt 110 ]
}
window 255 255 255
window 0 0 0
for _ in $(seq 60); do
    seen=$(ctl query 'app_id:chora-probe kind:surface') || { sleep 0.5; continue; }
    [ "$(grep -c '"owner": *"tiling"' <<<"$seen")" = 2 ] && break
    sleep 0.5
done
snap
accented 12 400 || fail "the focused window has no accent border at 12,400: $(px 12 400)"
accented 300 12 || fail "no accent border above the focused window: $(px 300 12)"
lined 646 400 || fail "the other window has no line border at 646,400: $(px 646 400)"
ok "only the focused window carries the accent"

echo '[{"shortcut": {"keys": "Super+j"}}]' | ctl inject - >/dev/null || fail "Super+j"
snap
lined 12 400 || fail "after Super+j the old window keeps the accent at 12,400: $(px 12 400)"
accented 646 400 || fail "after Super+j the new window has no accent at 646,400: $(px 646 400)"
ok "after Super+j only the new window carries the accent"

python3 -I - "$repo/tests" "$shot" "$scale" <<'PY' || fail "the border is broken"
import sys
sys.path.insert(0, sys.argv[1])
import importlib.util
spec = importlib.util.spec_from_file_location("png_pixel", sys.argv[1] + "/png-pixel.py")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
k = int(sys.argv[3])
rows = [bytes(r) for r in mod.rows(open(sys.argv[2], "rb").read())]
def lit(x, y):
    return rows[y * k][x * k * 4:x * k * 4 + 3] != b"\0\0\0"
broken = [(x, y) for y in range(30, 770, 4) for x in (12, 634, 645, 1267) if not lit(x, y)]
broken += [(x, y) for x in list(range(30, 620, 4)) + list(range(660, 1250, 4)) for y in (12, 787) if not lit(x, y)]
assert not broken, f"broken at {broken[:5]}"
PY
ok "the border is unbroken at the scale $scale"
