#!/bin/bash
# A click on a row of the launcher starts the program of that row, in the
# tiling look (the Apps door of the bar) and in the construct (the Programs
# door), when the click comes from tests/persona/act.py, the hands of a
# simulated user (RC shared bar: a visible way to start a program).
#
# The launcher is fuzzel 1.11.1. It takes the first motion of the pointer as
# its start and selects the row under the pointer from the second motion on;
# a pointer_move that enters its surface is an enter, not a motion. A click
# with one move therefore finds the first row highlighted and starts it (the
# helper chat, in the rounds of 2026-11-20), where a hand, which moves the
# pointer many times, starts the row under it. The cell uses the harness only:
# act.py look, click Apps or Programs, click the Mid row. The test rows are
# named Aa, Ab and Ac so that they stand before every row the flavor ships
# (Edit settings, the themes, Screenshot): the Mid row is the second program,
# the third row of the tiling list, where the helper row stands first
# (ti-50), and the second of the construct, but the list is centred, so its y
# follows the number of rows up to the 9 that fuzzel.ini shows (lines=9), and
# the click takes the y of a list that is full. Mid, the second row
# of the list in both looks, must start, and
# Alpha, the highlighted first row, must not.
#
# 1. tests/test_act_pointer.py (judged at the end): act.py click, drag, move and scroll without
#    a session, on the events they send.
# 2. one persona session per look, with a cache folder of its own (fuzzel puts
#    the row it started first in its next list, so the order of the rows is
#    that of the session and not of the machine), with the programs alpha, mid and zebra
#    (foot with their app id) and the launcher of the look.
#
# Usage: tests/gate-launcher-click.sh   BIN: directory with the binaries
#   row LOOK DOOR_X DOOR_Y ROW_X ROW_Y takes the positions of the door and of
#   the Mid row on a capture with the longest side 800, the first look.
# shellcheck disable=SC2016
set -u
. "$(dirname "$0")/gate-lib.sh"
need python3
need foot
need_fuzzel 1.11
need jq
need_bins chora choractl chora-tiling chora-construct chora-assistant chora-probe

unit_log=$(mktemp "${TMPDIR:-/tmp}/launcher-click-unit.XXXXXX")
tmp_dirs+=("$unit_log")
unit_failed=
python3 -I "$repo/tests/test_act_pointer.py" >/dev/null 2>"$unit_log" || unit_failed=1

act() { PERSONA_ENV=$dir/session.env python3 -I "$repo/tests/persona/act.py" "$@"; }
admin() {
    CHORA_SOCKET=$(sed -n 's/^CHORA_SOCKET=//p' "$dir/session.env") \
        XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$dir/session.env") "$bin/choractl" "$@"
}
launcher_up() {
    XDG_RUNTIME_DIR=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$dir/session.env") session_pids -x fuzzel >/dev/null
}
shown() { admin query "app_id:$1 kind:surface" | jq length; }
until_ok() {
    local _
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}

spid=
dir=
stop_session() {
    [ -z "$spid" ] || {
        "$repo/tests/persona/session.sh" --stop "$dir/session.env" >/dev/null 2>&1
        wait "$spid" 2>/dev/null
        spid=
    }
}
trap 'status=$?; stop_session; gate_cleanup $status' EXIT

row() {
    local look=$1 door_x=$2 door_y=$3 row_x=$4 row_y=$5 _
    dir=$(mktemp -d /tmp/launcher-click.XXXXXX)
    tmp_dirs+=("$dir")
    mkdir -p "$dir/data/applications"
    for p in alpha:'Aa Alpha' mid:'Ab Mid' zebra:'Ac Zebra'; do
        printf '[Desktop Entry]\nType=Application\nName=%s\nExec=foot -a %s sh -c "sleep 600"\n' "${p#*:}" "${p%%:*}" \
            >"$dir/data/applications/${p%%:*}.desktop"
    done
    (
        export XDG_DATA_DIRS=$dir/none
        export XDG_CACHE_HOME=$dir/cache
        PERSONA_DIR=$dir CHORA_BIN=$bin exec "$repo/tests/persona/session.sh" "$look" >"$dir.out" 2>"$dir.err"
    ) &
    spid=$!
    until_ok "grep -q '^PERSONA_ENV=' '$dir.out'" 200 || fail "the $look session did not start: $(cat "$dir.err")"
    act look >/dev/null || fail "$look: look"
    local base=$dir/before.png t_click ms n
    admin grant choractl screen.capture >/dev/null || fail "$look: grant of the capture"
    admin capture "$base" >/dev/null || fail "$look: capture before the click"
    t_click=$(date +%s%N)
    act click "$door_x" "$door_y" >/dev/null || fail "$look: click on the door"
    if [ "$look" = tiling ]; then
        local shot=0 shots_ms=() found=
        while :; do
            shot=$((shot + 1))
            admin capture "$dir/after-$shot.png" >/dev/null || fail "$look: capture after the click"
            shots_ms[shot]=$((($(date +%s%N) - t_click) / 1000000))
            [ "${shots_ms[shot]}" -le "${LAUNCHER_MS_MAX:-1500}" ] || break
        done
        for i in $(seq "$shot"); do
            n=$(python3 -I "$repo/tests/capture-diff.py" "$base" "$dir/after-$i.png" 400 150 880 650)
            if [ "$n" != 0 ] && [ "$n" != size ]; then
                found=$i
                break
            fi
        done
        [ -n "$found" ] || fail "$look: no launcher in $shot captures up to ${shots_ms[shot]} ms after the click on the door"
        ms=${shots_ms[found]}
        [ "$ms" -le "${LAUNCHER_MS_MAX:-1500}" ] || fail "$look: the launcher was drawn after $ms ms, not within 1500 ms"
        ok "$look: the launcher is visible $ms ms after the click on the Apps door ($n pixels changed, capture $found of $shot; limit 1500 ms)"
    fi
    until_ok 'launcher_up' 80 || fail "$look: the launcher did not start after a click on its door"
    sleep 2
    act click "$row_x" "$row_y" >/dev/null || fail "$look: click on the Mid row"
    until_ok '[ "$(shown mid)" -gt 0 ]' 120 ||
        fail "$look: a click on the Mid row of the launcher did not start mid (alpha, the first row: $(shown alpha), the chat: $(shown chora-chat))"
    [ "$(shown alpha)" = 0 ] || fail "$look: a click on the Mid row started alpha, the first row"
    ok "$look: a click by act.py on the Mid row of the launcher starts mid and not the first row"
    stop_session
}

row tiling 35 15 400 230
row construct 96 32 400 184
if [ -n "$unit_failed" ]; then
    cat "$unit_log" >&2
    fail "act.py click or drag does not move the pointer as a hand does (tests/test_act_pointer.py)"
fi
ok "act.py click and drag move the pointer as a hand does, move and scroll as they say"
