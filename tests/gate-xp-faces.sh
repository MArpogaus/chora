#!/bin/bash
# Gate of one xp on four virtual outputs (D69): one headless session with
# one screen. tests/pane-host.py owns four virtual outputs and shows each
# in a forwarding pane on the screen; one chora-xp manages all four with
# `--output`. The cells:
#
#   faces-have-bars      every virtual output has its taskbar and its
#                        wallpaper
#   opens-under-pointer  a new window opens on the virtual output whose
#                        pane is under the pointer, here the second one
#   moves-to-face        `window.move` of xp moves the window to the third
#                        virtual output, where it shows with the focus
#   face-removed         the window is on the fourth virtual output and
#                        that one goes: xp moves the window to another of
#                        its outputs and keeps three taskbars
#
# A cell measures pass or gap; a line `missing` in the table EXPECTED is a
# cell that xp cannot pass yet and that does not run. The gate fails on
# every cell that differs from the table, in both directions, so a fix
# must flip its line. It fails too when the window is lost.
#
# The panes show their outputs at 1:1, two by two; the pointer goes to the
# middle of a pane.
#
# Usage: tests/gate-xp-faces.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs and the capture (default: a temp dir)
set -uo pipefail

EXPECTED='
faces-have-bars pass
opens-under-pointer pass
moves-to-face pass
face-removed pass
'

. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need python3
need_bins chora choractl chora-xp
temp_config
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH
export CHORA_FRAME_BUDGET_US=${CHORA_FRAME_BUDGET_US:-500000}
theme=$repo/flavors/xp/theme

session_start 1280x800
grant input.inject screen.capture methods.call methods.privileged
screen=$(ctl outputs | jq -r '.[0].name')

python3 -I "$repo/tests/pane-host.py" "$screen" \
    600x380+20+10 600x380+660+10 600x380+20+410 600x380+660+410 >"$shots/pane-host.log" 2>&1 &
host=$!
pids+=("$host")
for _ in $(seq 50); do
    grep -qx ready "$shots/pane-host.log" && break
    sleep 0.2
done
grep -qx ready "$shots/pane-host.log" || fail "no pane host: $(tail -n 3 "$shots/pane-host.log")"
mapfile -t faces < <(awk 'NF == 5 {print $1}' "$shots/pane-host.log")
[ "${#faces[@]}" = 4 ] || fail "four virtual outputs: ${faces[*]}"

managed=()
for face in "${faces[@]}"; do managed+=(--output "$face"); done
"$bin/chora-xp" --theme "$theme" "${managed[@]}" >"$shots/xp.log" 2>&1 &
pids+=($!)
ctl wait "id:xp-bar-${faces[3]}" --timeout 20 >/dev/null || fail "xp does not start: $(tail -n 3 "$shots/xp.log")"

inject() { echo "$1" | ctl inject >/dev/null || fail "inject $1"; }
w=

on_face() {
    local shown
    shown=".[0] | .owner == \"xp\" and .parent != null and .focused == true
        and ((.rects[\$face] // [0, 0, 0, 0]) | .[2] > 0 and .[3] > 0)"
    for _ in $(seq "$(awk -v s="$2" 'BEGIN {print s * 5}')"); do
        ctl query "id:$w" 2>/dev/null | jq -e --arg face "$1" "$shown" >/dev/null && return 0
        sleep 0.2
    done
    return 1
}

move_to() { ctl call xp window.move "{\"surface\": \"$w\", \"output\": \"$1\"}" >/dev/null 2>&1; }

cell_faces_have_bars() {
    local face
    for face in "${faces[@]}"; do
        ctl query "id:xp-bar-$face" | jq -e 'length == 1' >/dev/null || return 1
        ctl query "id:xp-wall-$face" | jq -e --arg face "$face" '.[0].rects[$face][2] > 0' >/dev/null || return 1
    done
}

cell_opens_under_pointer() {
    inject '[{"pointer_move": {"position": [960, 200]}}]'
    foot -f "DejaVu Sans Mono:size=10" -a probe sh -c 'sleep 900' >"$shots/foot.log" 2>&1 &
    run_pid $!
    w=$(ctl wait "app_id:probe kind:surface" --timeout 20 | jq -r '.[0].id') || fail "foot does not show"
    on_face "${faces[1]}" 10
}

cell_moves_to_face() {
    move_to "${faces[2]}"
    on_face "${faces[2]}" 10
}

cell_face_removed() {
    local left
    move_to "${faces[3]}"
    on_face "${faces[3]}" 10 || return 1
    kill -USR1 "$host"
    ctl wait "id:xp-bar-${faces[3]}" --gone --timeout 10 >/dev/null || return 1
    for left in "${faces[@]:0:3}"; do
        ctl query "id:xp-bar-$left" | jq -e 'length == 1' >/dev/null || return 1
    done
    on_face "${faces[0]}" 10 || on_face "${faces[1]}" 1 || on_face "${faces[2]}" 1
}

declare -A got
mismatches=()
cell() {
    local id=$1 want result
    want=$(echo "$EXPECTED" | awk -v k="$id" '$1 == k {print $2}')
    [ -n "$want" ] || fail "no EXPECTED line for $id"
    if [ "$want" = missing ]; then
        got[$id]=missing
        return 0
    fi
    if "cell_${id//-/_}"; then result=pass; else result=gap; fi
    got[$id]=$result
    [ "$result" = "$want" ] || mismatches+=("$id: measured $result, expected $want")
    [ -z "$w" ] || [ "$(ctl query "id:$w" | jq -r '.[0].owner // "none"')" = xp ] ||
        fail "the window left xp after $id: $(ctl query "id:$w" | jq -c '.[0] | {owner, rects}')"
}

cells=(faces-have-bars opens-under-pointer moves-to-face face-removed)
for id in "${cells[@]}"; do cell "$id"; done
ctl capture "$shots/faces.png" >/dev/null 2>&1

printf '\n%-22s %s\n' cell measured
for id in "${cells[@]}"; do printf '%-22s %s\n' "$id" "${got[$id]}"; done
echo "run time: ${SECONDS}s"
if [ "${#mismatches[@]}" != 0 ]; then
    printf 'MISMATCH %s\n' "${mismatches[@]}" >&2
    fail "${#mismatches[@]} cells differ from EXPECTED"
fi
ok "gate-xp-faces: every cell as in EXPECTED in ${SECONDS}s"
