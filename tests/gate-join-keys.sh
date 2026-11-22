#!/bin/bash
# A look that joins a running session does not take the keyboard from the
# look that holds it (ti-38, D55 rule 3 of docs/api.md "Window managers").
# Two looks on one output, 1280x800, no pointer event and no click: the
# person only types. Cells:
#
#   tiling-first  tiling starts, xp joins: Super+Return opens a tile of
#                 tiling, not a floating window of xp; once the tile closed,
#                 the next Super+Return opens a tile again
#   xp-first      xp starts, tiling joins: the keyboard stays with xp, and
#                 the window of Super+Return opens in xp
#
# Each cell prints the `focus` of `choractl state` of both looks before the
# key. The looks take injected keys (test_accept_injected_keys).
#
# Usage: tests/gate-join-keys.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs (default: a temp dir)
# shellcheck disable=SC2016
set -uo pipefail
. "$(dirname "$0")/gate-lib.sh"
need jq
need foot
need_bins chora choractl chora-tiling chora-xp
temp_config
for look in tiling xp; do
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
    esac
    pids+=($!)
    until_ok "ctl state $1 >/dev/null 2>&1" 80 || fail "$2: $1 does not answer state.get: $(tail -n 3 "$shots/$2-$1.log")"
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
    env_file=$shots/$name.env session_start 1280x800
    grant input.inject methods.call
    start_look "$first" "$name"
    start_look "$joins" "$name"
    ctl wait id:xp-icons --timeout 20 >/dev/null || fail "$name: xp shows no desktop icons"
    last_holders=''
    until_ok steady 40
    echo "$name: before the key: $(holders)"
    echo '[{"shortcut": {"keys": "Super+Return"}}]' | ctl inject >/dev/null || fail "$name: inject Super+Return"
    until_ok '[ -n "$(surface_owner)" ]' 80 || fail "$name: Super+Return opened no window"
    owner=$(surface_owner)
    echo "$name: after the key: $(holders), window of $owner"
    [ "$owner" = "$first" ] ||
        fail "$name: Super+Return opened a window of $owner, not of $first, which held the keyboard before $joins joined"
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

cell tiling xp
cell xp tiling
ok "gate-join-keys: a joining look leaves the keyboard with the look that holds it (${SECONDS}s)"
