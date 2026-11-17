#!/bin/bash
# Gate of a window that goes to mc (rv5-2 of the tour review): a headless
# session with xp, tiling and mc, each on its own output, and a foot with
# a red background that xp shows. In each cell the window shows on the
# page of mc within 1 s, and no window is lost:
#
#   - give: `choractl give mc` of the focused window;
#   - key: Super+Alt+3 of xp;
#   - drag-held: the title bar of xp pulled over the mc output with the
#     button still down; the window follows the pointer over the page;
#   - drag-dropped: the button goes up there; mc owns the window.
#
# "Shows" is a red pixel (the window) in the middle of the rectangle that
# the scene reports for the window on the mc output, in a capture.
# Captures go to SHOTS as mc-handover-CELL.png.
#
# Usage: tests/gate-mc-handover.sh   BIN: directory with the binaries
#   SHOTS: directory for the captures (default: a temp dir)
set -uo pipefail
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need python3
need_bins chora choractl chora-tiling chora-xp chora-mc
temp_config
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
theme=$repo/flavors/xp/theme
within_ms=${WITHIN_MS:-1000}

session_start 1280x800
grant input.inject screen.capture methods.call
ctl add-output 1280x800+1280+0 >/dev/null || fail "add-output"
ctl add-output 1280x800+2560+0 >/dev/null || fail "add-output"
mapfile -t outs < <(ctl outputs | jq -r 'sort_by(.position[0]) | .[].name')
[ "${#outs[@]}" = 3 ] || fail "three outputs: ${outs[*]}"
xp_out=${outs[0]} tiling_out=${outs[1]} mc_out=${outs[2]}

plugin() {
    local name=$1
    shift
    "$bin/chora-$name" "$@" >"$shots/$name.log" 2>&1 &
    pids+=($!)
}
plugin tiling --output "$tiling_out"
plugin xp --theme "$theme" --output "$xp_out"
plugin mc --manage-windows --output "$mc_out"
for c in tiling xp mc; do
    ctl wait "owner:$c" --timeout 20 >/dev/null || fail "$c does not start: $(tail -n 3 "$shots/$c.log")"
done

inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
now_ms() { date +%s%3N; }
owner_of() { ctl query "id:$w" | jq -r '.[0].owner // "none"'; }
surface_count() { ctl query kind:surface | jq 'length'; }

inject "[{\"pointer_move\": {\"position\": [640, 300], \"output\": \"$xp_out\"}}]"
foot -f "DejaVu Sans Mono:size=10" -o colors.background=ff0000 -o colors.alpha=1 \
    sh -c 'sleep 900' >"$shots/foot.log" 2>&1 &
run_pid $!
w=$(ctl wait "app_id:foot kind:surface owner:xp" --timeout 20 | jq -r '.[0].id') ||
    fail "foot does not show in xp"
count=$(surface_count)

shows_red() {
    local rect
    rect=$(ctl query "id:$w" | jq -c --arg o "$mc_out" '.[0].rects[$o] // empty')
    [ -n "$rect" ] || return 1
    ctl capture "$shots/mc-handover-$1.png" --output "$mc_out" >/dev/null || return 1
    python3 -I - "$shots/mc-handover-$1.png" "$rect" <<'PY'
import json, sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
x, y, w, h = json.loads(sys.argv[2])
px = (min(max(int(x + w / 2), 0), im.width - 1), min(max(int(y + h / 2), 0), im.height - 1))
r, g, b = im.getpixel(px)
sys.exit(0 if r > 200 and g < 60 and b < 60 else 1)
PY
}

drawn() {
    local start now
    start=$(now_ms)
    while :; do
        now=$(now_ms)
        if [ "$(owner_of)" = "$2" ] && shows_red "$1"; then
            [ "$(surface_count)" = "$count" ] || fail "$1: $count surfaces before, $(surface_count) after"
            ok "$1: the window shows on the page of mc after $((now - start)) ms, owner $2"
            return 0
        fi
        [ $((now - start)) -lt "$within_ms" ] || break
        sleep 0.05
    done
    fail "$1: the window is not drawn on the mc output within $within_ms ms (owner $(owner_of)); capture $shots/mc-handover-$1.png"
}

has_focus() {
    for _ in $(seq 50); do
        ctl query "id:$w" | jq -e '.[0].focused == true' >/dev/null && return 0
        sleep 0.1
    done
    return 1
}
settle_with() {
    ctl wait "id:$w owner:$1" --timeout 20 >/dev/null || fail "the window does not come to $1"
    has_focus || fail "the window has no focus with $1"
    sleep 1
}

settle_with xp
ctl give mc >/dev/null || fail "choractl give mc"
drawn give mc

ctl call mc window.give "{\"surface\": \"$w\", \"to\": \"xp\"}" >/dev/null || fail "mc window.give to xp"
settle_with xp
key Super+Alt+3
drawn key mc

ctl call mc window.give "{\"surface\": \"$w\", \"to\": \"xp\"}" >/dev/null || fail "mc window.give to xp"
settle_with xp
inject "[{\"pointer_move\": {\"node\": \"xp-title-$w\", \"point\": [60, 10]}},
  {\"button\": {\"state\": \"pressed\", \"hold\": true}},
  {\"pointer_move\": {\"position\": [900, 300], \"output\": \"$xp_out\"}},
  {\"pointer_move\": {\"position\": [100, 300], \"output\": \"$mc_out\"}},
  {\"pointer_move\": {\"position\": [640, 400], \"output\": \"$mc_out\"}}]"
drawn drag-held xp
inject "[{\"button\": {\"state\": \"released\"}}]"
drawn drag-dropped mc
