#!/bin/bash
# Shadows and motion of tiling (tiling-v2, items 8 and 9): a window has a
# shadow on the wallpaper; a layout change slides the windows in 200 ms
# and a tag switch brings them in from the side, each ending at its place
# with the opacity 1; look.animations false makes both instant; an idle
# scene draws no frame.
#
# Usage: tests/gate-tiling-motion.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
need swaybg
need_bins chora choractl chora-tiling chora-probe
temp_config
session_start 1280x800 -- "$bin/chora-tiling"
grant screen.capture
shot=$XDG_CONFIG_HOME/motion.png
start() {
    started=$(ctl run "$@") || fail "the run of $1"
    pid=$(sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p' <<<"$started")
    [ -n "$pid" ] || fail "the run of $1"
    run_pid "$pid"
}
start swaybg -c '#808080' -m solid_color
start "$bin/chora-probe" window 40 44 60
start "$bin/chora-probe" window 70 70 90
for _ in $(seq 60); do
    seen=$(ctl query 'app_id:chora-probe kind:surface') || { sleep 0.5; continue; }
    [ "$(grep -c '"owner": *"tiling"' <<<"$seen")" = 2 ] && break
    sleep 0.5
done
sleep 3
ctl capture --output HEADLESS-1 "$shot" >/dev/null || fail "the capture"
grey=$(pixel "$shot" 1279 400 | cut -d" " -f1)
for spot in "5 400 12" "1270 400 6"; do
    read -r x y least <<<"$spot"
    got=$(pixel "$shot" "$x" "$y" | cut -d' ' -f1)
    [ $((grey - got)) -ge "$least" ] || fail "no shadow at $x,$y: $got against the wallpaper $grey"
done
ok "the focused window has a deep shadow, the other a light one"

groups() {
    local all
    all=$(ctl query 'kind:group') || return 1
    python3 -I -c '
import json, sys
for n in json.load(sys.stdin):
    if n["id"].startswith("tw-"):
        print(n["id"], round(n["transform"]["matrix"][12]), round(n["opacity"], 3))
' <<<"$all" | sort
}
shown=$(groups) || fail "the query of the groups"
[ "$(awk '$3 != 1' <<<"$shown" | wc -l)" = 0 ] || fail "a window stays transparent: $shown"
ok "every window is opaque once it stands"

settled=$(groups) || fail "the query of the groups"
ctl call tiling set_layout '{"layout": "tile_left"}' >/dev/null || fail "set_layout"
moving=$(groups) || fail "the query of the groups"
[ "$moving" != "$settled" ] || fail "the windows did not start to slide"
sleep 1
end=$(groups) || fail "the query of the groups"
[ "$end" != "$settled" ] || fail "the windows did not move"
[ "$(awk '$3 != 1' <<<"$end" | wc -l)" = 0 ] || fail "a window is transparent after its move: $end"
python3 -I - "$moving" "$settled" "$end" <<'PY' || fail "the slide is not between the places"
import sys
rows = [{l.split()[0]: float(l.split()[1]) for l in s.splitlines()} for s in sys.argv[1:]]
moving, before, after = rows
assert any(min(before[k], after[k]) < moving[k] < max(before[k], after[k]) for k in moving if before[k] != after[k]), (moving, before, after)
PY
ok "a layout change slides the windows and they end at their places"

ctl call tiling view_tag '{"tag": 2}' >/dev/null || fail "view_tag 2"
sleep 1
ctl call tiling view_tag '{"tag": 1}' >/dev/null || fail "view_tag 1"
coming=$(groups) || fail "the query of the groups"
sleep 1
there=$(groups) || fail "the query of the groups"
python3 -I - "$coming" "$there" <<'PY' || fail "a tag switch does not bring the windows in from the side"
import sys
a, b = ({l.split()[0]: float(l.split()[1]) for l in s.splitlines()} for s in sys.argv[1:])
assert a and all(abs(a[k] - b[k]) > 200 for k in a), (a, b)
PY
ok "a tag switch brings the windows in from the side"

mkdir -p "$XDG_CONFIG_HOME/chora"
"$bin/chora-tiling" --print-config | sed 's/^  animations: true/  animations: false/' >"$XDG_CONFIG_HOME/chora/chora-tiling.yaml"
ctl call tiling settings.reload >/dev/null || fail "settings.reload"
sleep 1
settled=$(groups) || fail "the query of the groups"
ctl call tiling set_layout '{"layout": "tile"}' >/dev/null || fail "set_layout"
now=$(groups) || fail "the query of the groups"
sleep 1
again=$(groups) || fail "the query of the groups"
[ "$now" = "$again" ] || fail "with animations false the windows still moved over time"
[ "$now" != "$settled" ] || fail "the layout did not change"
ok "animations false makes the change at once"

frames() {
    local stats
    stats=$(ctl render-stats) || return 1
    python3 -I -c 'import json, sys; print(json.load(sys.stdin)["outputs"]["HEADLESS-1"]["damaged_frames"])' <<<"$stats"
}
before=$(frames) || fail "render-stats"
sleep 5
after=$(frames) || fail "render-stats"
[ "$before" = "$after" ] || fail "an idle scene drew $((after - before)) frames in 5 s"
ok "an idle scene draws no frame"
