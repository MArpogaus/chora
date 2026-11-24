#!/bin/bash
# A look that joins a running session does not take the keyboard from the
# look that holds it (ti-38, D55 rule 3 of docs/api.md "Window managers").
# Two looks on one output, 1280x800 (mc-first: two outputs); in the first
# two cells no pointer event and no click: the person only types. Cells:
#
#   tiling-first  tiling starts, xp joins: Super+Return opens a tile of
#                 tiling, not a floating window of xp; once the tile closed,
#                 the next Super+Return opens a tile again
#   xp-first      xp starts, tiling joins: the keyboard stays with xp, and
#                 the window of Super+Return opens in xp
#   xp-window     xp starts and a foot that keeps what it is typed is its
#                 window; tiling joins: Super+x, k, m and q type nothing
#                 into the foot (xp-46), and once xp gave the foot to
#                 tiling, Super+h and Shift+Super+1 of tiling act (rss-2:
#                 the swallow of xp does not take the keys of tiling)
#   mc-first      mc --manage-windows on the first output, tiling joins on
#                 the second, then such a foot is a window page of mc:
#                 Super+x, k, m and q type nothing into the foot (mc-60),
#                 and Super+1 to Super+8 of tiling act (rss-1)
#
# In xp-window the join of tiling leaves no surface focused (stray-super
# r2, found on the way), so a click on the foot gives it the keyboard again.
#
# Each cell prints the `focus` of `choractl state` of both looks before the
# key and asserts the holders (tiling-first: tiling and xp none; xp-first: xp
# holds, tiling none), and the holder after the key; tiling-first ends with a
# click on the tile, which leaves the keyboard with tiling. The looks take injected keys (test_accept_injected_keys).
#
# Usage: tests/gate-join-keys.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs (default: a temp dir)
# expect_holders TAG TILING-HOLDER XP-HOLDER fails when a holder differs.
# shellcheck disable=SC2016
set -uo pipefail
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need_bins chora choractl chora-tiling chora-xp chora-mc
temp_config
for look in tiling xp mc; do
    printf 'test_accept_injected_keys: true\n' >"$XDG_CONFIG_HOME/chora-$look.yaml"
done
shots=${SHOTS:-$XDG_CONFIG_HOME}
mkdir -p "$shots"
export PATH=$bin:$PATH

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
start_look() {
    case $1 in
    tiling) "$bin/chora-tiling" >"$shots/$2-tiling.log" 2>&1 & ;;
    xp) "$bin/chora-xp" --theme "$repo/flavors/xp/theme" >"$shots/$2-xp.log" 2>&1 & ;;
    tiling-on) "$bin/chora-tiling" --output "$3" >"$shots/$2-tiling.log" 2>&1 & ;;
    mc-on) "$bin/chora-mc" --manage-windows --output "$3" >"$shots/$2-mc.log" 2>&1 & ;;
    esac
    pids+=($!)
    local look=${1%-on}
    until_ok "ctl state $look >/dev/null 2>&1" 80 || fail "$2: $look does not answer state.get: $(tail -n 3 "$shots/$2-$look.log")"
}
holder_of() { ctl state "$1" | jq -r '.focus.holder // "none"'; }
expect_holders() {
    [ "$(holder_of tiling)" = "$2" ] && [ "$(holder_of xp)" = "$3" ] ||
        fail "$1: the holders are tiling $(holder_of tiling), xp $(holder_of xp); wanted tiling $2, xp $3"
}
holders() { echo "tiling $(ctl state tiling | jq -c .focus) xp $(ctl state xp | jq -c .focus)"; }
steady() {
    local now
    now=$(holders)
    [ "$now" = "$last_holders" ] && return 0
    last_holders=$now
    return 1
}
surface_owner() { ctl query kind:surface | jq -r '.[0].owner // empty'; }

cell() {
    local first=$1 joins=$2 name=$1-first owner
    CHORA_TEST_FLAGS=--test-injected-press-is-real env_file=$shots/$name.env session_start 1280x800
    grant input.inject methods.call
    start_look "$first" "$name"
    start_look "$joins" "$name"
    ctl wait id:xp-icons --timeout 20 >/dev/null || fail "$name: xp shows no desktop icons"
    last_holders=''
    until_ok steady 40
    echo "$name: before the key: $(holders)"
    if [ "$first" = tiling ]; then expect_holders "$name before the key" none none; else expect_holders "$name before the key" none xp; fi
    echo '[{"shortcut": {"keys": "Super+Return"}}]' | ctl inject >/dev/null || fail "$name: inject Super+Return"
    until_ok '[ -n "$(surface_owner)" ]' 80 || fail "$name: Super+Return opened no window"
    owner=$(surface_owner)
    echo "$name: after the key: $(holders), window of $owner"
    [ "$owner" = "$first" ] ||
        fail "$name: Super+Return opened a window of $owner, not of $first, which held the keyboard before $joins joined"
    if [ "$first" = tiling ]; then expect_holders "$name after the key" tiling none; else expect_holders "$name after the key" none xp; fi
    if [ "$first" = tiling ]; then
        until_ok "ctl state tiling | jq -e '.windows | length == 1 and (.[0].floating | not)' >/dev/null" 40 ||
            fail "$name: tiling shows no tile: $(ctl state tiling | jq -c .windows)"
        mapfile -t left < <(session_pids -x foot)
        kill "${left[@]}" 2>/dev/null
        until_ok '[ -z "$(surface_owner)" ]' 80 || fail "$name: the tile stays after its foot ended"
        echo '[{"shortcut": {"keys": "Super+Return"}}]' | ctl inject >/dev/null || fail "$name: inject Super+Return"
        until_ok '[ -n "$(surface_owner)" ]' 80 || fail "$name: the second Super+Return opened no window"
        owner=$(surface_owner)
        echo "$name: after the tile closed and a second key: $(holders), window of $owner"
        [ "$owner" = tiling ] || fail "$name: after the tile closed, Super+Return opened a window of $owner"
        spot=$(ctl query kind:surface | jq -r '.[0].rects | to_entries[0].value | "\(.[0] + .[2] / 2 | floor) \(.[1] + .[3] / 2 | floor)"') ||
            fail "$name: no rect of the tile"
        echo "[{\"pointer_move\": {\"position\": [${spot% *}, ${spot#* }]}}, {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]" |
            ctl inject >/dev/null || fail "$name: inject the click"
        sleep 0.5
        expect_holders "$name after a click on the tile at $spot" tiling none
        echo "$name: after a click on the tile at $spot: $(holders)"
    fi
    ok "$name: $joins joined and left the keyboard with $first; Super+Return opened a window of $first"
    end_session
}
end_session() {
    local left
    mapfile -t left < <(session_pids -x foot)
    kill "${pids[@]}" "${left[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}

stray_start() {
    stray=$shots/$1.typed
    rm -f "$stray"
    ctl run foot -a stray sh -c "stty raw -echo; cat >$stray" >/dev/null || fail "$1: the run of the typing foot"
    ctl wait --timeout 15 "app_id:stray kind:surface owner:$2" >/dev/null || fail "$1: the typing foot is not a window of $2"
}
stray_focused() { ctl query 'app_id:stray kind:surface' | jq -e '.[0].focused == true' >/dev/null; }
click_stray() {
    local at
    at=$(ctl query 'app_id:stray kind:surface' | jq -r '.[0].rects | to_entries[0] | "\(.value[0] + .value[2] / 2 | floor), \(.value[1] + .value[3] / 2 | floor), \"\(.key)\""') || fail "no rect of the typing foot"
    echo "[{\"pointer_move\": {\"position\": [${at%, *}], \"output\": ${at##*, }}}, {\"button\": {\"state\": \"pressed\"}}, {\"button\": {\"state\": \"released\"}}]" |
        ctl inject >/dev/null || fail "inject the click on the typing foot"
    until_ok stray_focused 40 || fail "a click on the typing foot does not give it the keyboard"
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
typed_text() { echo "[{\"text\": {\"text\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject the text $1"; }
no_stray() {
    local typed
    typed_text start:
    for chord in Super+x Super+k Super+m Super+q; do key "$chord"; done
    typed_text :end
    until_ok '[ "$(tr -d "\r" <"$stray" 2>/dev/null)" = "start::end" ]' 40
    typed=$(tr -d '\r' <"$stray" 2>/dev/null)
    [ "$typed" = "start::end" ] || fail "$1: the window of $2 typed '$typed' and not start::end"
    ok "$1: Super+x, Super+k, Super+m and Super+q type nothing into the window of $2"
}
refusals() { grep -c 'grabbed already' "$shots/$1-tiling.log"; }
tiling_rects() { ctl state tiling | jq -c '[.windows[] | .rect]'; }

cell_xp_window() {
    local name=xp-window before
    CHORA_TEST_FLAGS=--test-injected-press-is-real env_file=$shots/$name.env session_start 1280x800
    grant input.inject methods.call
    start_look xp "$name"
    ctl wait id:xp-icons --timeout 20 >/dev/null || fail "$name: xp shows no desktop icons"
    stray_start "$name" xp
    until_ok stray_focused 40 || fail "$name: the typing foot does not hold the keyboard"
    start_look tiling "$name"
    click_stray
    no_stray "$name" xp
    echo "$name: tiling refused $(refusals "$name") grabs"
    key Super+Alt+1
    until_ok "ctl query 'app_id:stray kind:surface' | jq -e '.[0].owner == \"tiling\"' >/dev/null" 40 || fail "$name: Super+Alt+1 gave the foot not to tiling"
    until_ok stray_focused 40 || fail "$name: the given foot does not hold the keyboard"
    ctl run foot -a second sh -c 'sleep 600' >/dev/null || fail "$name: the run of a second foot"
    until_ok "ctl state tiling | jq -e '[.windows[] | select(.shown)] | length == 2' >/dev/null" 80 || fail "$name: the second foot is no tile: $(ctl state tiling | jq -c .windows)"
    before=$(tiling_rects)
    key Super+h
    until_ok '[ "$(tiling_rects)" != "$before" ]' 40 || fail "$name: Super+h of tiling does nothing beside xp: the rects stay $before"
    ok "$name: Super+h of tiling acts beside xp with a window"
    key Shift+Super+3
    until_ok "ctl state tiling | jq -e '[.windows[] | select(.desks == [3])] | length == 1' >/dev/null" 40 || fail "$name: Shift+Super+3 of tiling does nothing: $(ctl state tiling | jq -c '[.windows[] | .desks]')"
    key Super+3
    until_ok "ctl state tiling | jq -e '[.desks[] | .shown] == [[3]]' >/dev/null" 40 || fail "$name: Super+3 shows not tag 3: $(ctl state tiling | jq -c .desks)"
    key Shift+Super+1
    until_ok "ctl state tiling | jq -e '[.windows[] | select(.desks == [1])] | length == 2' >/dev/null" 40 || fail "$name: Shift+Super+1 of tiling does nothing beside xp: $(ctl state tiling | jq -c '[.windows[] | .desks]')"
    ok "$name: Shift+Super+1 of tiling acts beside xp with a window"
    end_session
}

cell_mc_first() {
    local name=mc-first outs n
    CHORA_TEST_FLAGS=--test-injected-press-is-real env_file=$shots/$name.env session_start 1280x800,1280x800+1280+0
    grant input.inject methods.call
    mapfile -t outs < <(ctl outputs | jq -r 'sort_by(.position[0]) | .[].name')
    [ "${#outs[@]}" = 2 ] || fail "$name: the outputs are ${outs[*]}"
    start_look mc-on "$name" "${outs[0]}"
    ctl wait --timeout 20 'owner:mc meta:pane=mission' >/dev/null || fail "$name: mc shows no page"
    start_look tiling-on "$name" "${outs[1]}"
    ctl wait --timeout 20 owner:tiling >/dev/null || fail "$name: tiling does not start"
    stray_start "$name" mc
    until_ok "ctl call mc scene.list '{}' | jq -e '.current == \"stray\"' >/dev/null" 40 || fail "$name: the typing foot has no page: $(ctl call mc scene.list '{}' | jq -c .current)"
    key Return
    until_ok stray_focused 40 || fail "$name: Return does not give the keyboard to the typing foot"
    no_stray "$name" mc
    echo "$name: tiling refused $(refusals "$name") grabs"
    for n in 2 3 4 5 6 7 8 1; do
        key "Super+$n"
        until_ok "ctl state tiling | jq -e '[.desks[] | .shown] == [[$n]]' >/dev/null" 40 ||
            fail "$name: Super+$n of tiling does nothing beside mc: $(ctl state tiling | jq -c .desks)"
    done
    ok "$name: Super+1 to Super+8 of tiling act beside mc"
    end_session
}

cell tiling xp
cell xp tiling
cell_xp_window
cell_mc_first
ok "gate-join-keys: a joining look leaves the keyboard with the look that holds it (${SECONDS}s)"
