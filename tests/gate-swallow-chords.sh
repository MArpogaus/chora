#!/bin/bash
# No chord of Super with any other modifier reaches a client (rpt2-1): with
# a foot that keeps what it is typed as the window of a look, Super+Alt+9,
# Super+Alt+x, Super+Ctrl+Shift+x, Super+Alt+Shift+x and Super+Ctrl+Alt+x
# type nothing into it, a plain text still does, and a chord that the look
# binds still acts. One cell for each look that swallows with a window:
#
#   tiling  the foot is a tile; Super+2 views tag 2
#   xp      the foot is a window of xp; Super+d shows the desktop and back
#   mc      the foot is a window page of mc, with the keyboard; Super+F1
#           shows the key list
#
# The construct swallows the same list for a window in a pane: its cell is
# the co-25 check of tests/gate-construct-cube.sh, which sends the same
# chords to the probe window of the front face.
#
# Usage: tests/gate-swallow-chords.sh   BIN: directory with the binaries
#   SHOTS: directory for the logs (default: a temp dir)
# shellcheck disable=SC2016 # until_ok evaluates its code later
set -uo pipefail
export CHORA_TEST_FLAGS=--test-injected-press-is-real
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

CHORDS=(Super+Alt+9 Super+Alt+x Super+Ctrl+Shift+x Super+Alt+Shift+x Super+Ctrl+Alt+x)

until_ok() {
    for _ in $(seq "${2:-40}"); do
        eval "$1" && return 0
        sleep 0.25
    done
    return 1
}
key() { echo "[{\"shortcut\": {\"keys\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject $1"; }
typed_text() { echo "[{\"text\": {\"text\": \"$1\"}}]" | ctl inject >/dev/null || fail "inject the text $1"; }
stray_focused() { ctl query 'app_id:stray kind:surface' | jq -e '.[0].focused == true' >/dev/null; }
end_session() {
    local left
    mapfile -t left < <(session_pids -x foot)
    kill "${pids[@]}" "${left[@]}" 2>/dev/null
    wait 2>/dev/null
    pids=()
}
start_session() {
    CHORA_TEST_FLAGS=--test-injected-press-is-real env_file=$shots/$1.env session_start 1280x800
    grant input.inject methods.call
    case $1 in
    tiling) "$bin/chora-tiling" >"$shots/$1.log" 2>&1 & ;;
    xp) "$bin/chora-xp" --theme "$repo/flavors/xp/theme" >"$shots/$1.log" 2>&1 & ;;
    mc) "$bin/chora-mc" --manage-windows >"$shots/$1.log" 2>&1 & ;;
    esac
    pids+=($!)
    until_ok "ctl state $1 >/dev/null 2>&1" 80 || fail "$1: no state.get answer: $(tail -n 3 "$shots/$1.log")"
}
start_stray() {
    stray=$shots/$1.typed
    rm -f "$stray"
    ctl run foot -a stray sh -c "stty raw -echo; cat >$stray" >/dev/null || fail "$1: the run of the typing foot"
    ctl wait --timeout 15 "app_id:stray kind:surface owner:$1" >/dev/null || fail "$1: the typing foot is no window of $1"
}
no_stray() {
    local typed chord
    typed_text start:
    for chord in "${CHORDS[@]}"; do key "$chord"; done
    typed_text :end
    until_ok '[ "$(tr -d "\r" <"$stray" 2>/dev/null)" = "start::end" ]' 40
    typed=$(tr -d '\r' <"$stray" 2>/dev/null)
    [ "$typed" = "start::end" ] ||
        fail "$1: the window typed '$typed' and not start::end, after ${CHORDS[*]}"
    ok "$1: ${CHORDS[*]} type nothing into the window, a plain text does"
}

cell_tiling() {
    start_session tiling
    start_stray tiling
    until_ok stray_focused 40 || fail "tiling: the typing foot does not hold the keyboard"
    no_stray tiling
    key Super+2
    until_ok "ctl state tiling | jq -e '[.desks[] | .shown] == [[2]]' >/dev/null" 40 ||
        fail "tiling: Super+2 does not view tag 2: $(ctl state tiling | jq -c .desks)"
    ok "tiling: a chord that tiling binds still acts"
    end_session
}
cell_xp() {
    start_session xp
    ctl wait id:xp-icons --timeout 20 >/dev/null || fail "xp: no desktop icons"
    start_stray xp
    until_ok stray_focused 40 || fail "xp: the typing foot does not hold the keyboard"
    no_stray xp
    key Super+d
    until_ok "ctl state xp | jq -e '[.windows[] | select(.shown)] | length == 0' >/dev/null" 40 ||
        fail "xp: Super+d puts no window away: $(ctl state xp | jq -c '[.windows[] | .shown]')"
    key Super+d
    until_ok "ctl state xp | jq -e '[.windows[] | select(.shown)] | length == 1' >/dev/null" 40 ||
        fail "xp: a second Super+d does not bring the window back"
    ok "xp: a chord that xp binds still acts"
    end_session
}
cell_mc() {
    start_session mc
    ctl wait --timeout 20 'owner:mc meta:pane=mission' >/dev/null || fail "mc: no page"
    start_stray mc
    until_ok "ctl call mc scene.list '{}' | jq -e '.current == \"stray\"' >/dev/null" 40 ||
        fail "mc: the typing foot has no page: $(ctl call mc scene.list '{}' | jq -c .current)"
    key Return
    until_ok stray_focused 40 || fail "mc: Return does not give the keyboard to the typing foot"
    no_stray mc
    key Super+F1
    until_ok "ctl query 'owner:mc kind:text' | jq -e 'any(.[]; .text == \"Show overview\" and .visible)' >/dev/null" 40 ||
        fail "mc: Super+F1 shows no key list"
    ok "mc: a chord that mc binds still acts"
    end_session
}

for look in ${LOOKS:-tiling xp mc}; do "cell_$look"; done
ok "gate-swallow-chords: Super with any modifier reaches no window of tiling, xp or mc (${SECONDS}s)"
