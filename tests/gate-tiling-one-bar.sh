#!/bin/bash
# One top bar (ti-47): a tiling session has exactly one waybar on the config
# of its runtime directory, however it starts, and none is left when the
# session is stopped. Three ways in:
#   the product: chora-tiling starts the panel itself (no --no-companions);
#   the unit too: the panel unit (chora-tiling-waybar.service) runs
#     waybar.sh beside the panel that chora-tiling started, as a login
#     session whose target wants both does;
#   the persona harness: tests/persona/session.sh tiling, which starts the
#     companions of the flavor itself and gives the core CHORA_FLAVOR_DIR
#     (a4badd8e), so that the core finds waybar.sh as well.
# A second bar is not a count of one that settles: the cell samples for 10 s
# after the first bar is up, and fails on any sample of two.
#
# Functions: one_bar NAME RUNTIME (one bar comes up and stays one for 10 s),
# none_left NAME RUNTIME (no waybar of that runtime after the stop),
# stop_session (ends the headless session that session_start made last).
# The cells run in the order product, unit (the unit starts after
# chora-tiling.service), persona.
#
# Usage: tests/gate-tiling-one-bar.sh [product|unit|persona ...]
#   BIN: the binaries (default target/debug); the cells to run (default all)
set -u
. "$(dirname "$0")/gate-lib.sh"
need waybar
need jq
need setsid
need dbus-run-session
need dbus-daemon
need_bins chora choractl chora-tiling chora-probe chora-assistant
flavor=$repo/flavors/tiling

bars() { XDG_RUNTIME_DIR=$1 session_pids -x waybar | wc -l; }

one_bar() {
    local name=$1 runtime=$2 n max=0
    until_ok "[ \"\$(bars $runtime)\" -ge 1 ]" 80 || fail "$name: no bar came up on $runtime"
    for _ in $(seq 20); do
        n=$(bars "$runtime")
        [ "$n" -le "$max" ] || max=$n
        pace 0.5
    done
    [ "$max" = 1 ] || fail "$name: $max waybar processes on one config: $(XDG_RUNTIME_DIR=$runtime session_pids -x waybar | tr '\n' ' ')"
    ok "$name: one waybar ($(bars "$runtime")) for 10 s"
}

none_left() {
    until_ok "[ \"\$(bars $2)\" = 0 ]" 40 || fail "$1: $(bars "$2") waybar processes left after the stop"
    ok "$1: none left after the stop"
}

stop_session() {
    local pid=${pids[-1]}
    kill -TERM "$pid" 2>/dev/null
    until_ok "! kill -0 $pid 2>/dev/null" 80 || kill -KILL "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
    pids=()
}

cells=("$@")
[ ${#cells[@]} -gt 0 ] || cells=(product unit persona)
want() { [[ " ${cells[*]} " == *" $1 "* ]]; }

temp_config

if want product || want unit; then
    session_start 1280x800 -- "$bin/chora-tiling"
    runtime=$XDG_RUNTIME_DIR
    export CHORA_FLAVOR_DIR=$flavor
    one_bar "chora-tiling alone" "$runtime"
    if want unit; then
        PATH=$bin:$PATH setsid dbus-run-session -- sh "$flavor/waybar/waybar.sh" "$flavor/waybar" >/dev/null 2>&1 &
        run_pid $!
        pace 3
        one_bar "chora-tiling and the panel unit" "$runtime"
    fi
    stop_session
    none_left "product session" "$runtime"
    unset CHORA_FLAVOR_DIR
fi

if want persona; then
    pdir=$(mktemp -d /tmp/gate-one-bar.XXXXXX)
    tmp_dirs+=("$pdir" "$pdir.out")
    PERSONA_DIR=$pdir CHORA_BIN=$bin "$repo/tests/persona/session.sh" tiling >"$pdir.out" 2>&1 &
    pids+=($!)
    until_ok "grep -q '^PERSONA_ENV=' $pdir.out" 240 || fail "persona session did not start: $(tail -n 3 "$pdir.out")"
    pruntime=$(sed -n 's/^XDG_RUNTIME_DIR=//p' "$pdir/session.env")
    [ -d "$pruntime" ] || fail "no runtime directory of the persona session in $pdir/session.env"
    one_bar "persona session.sh tiling" "$pruntime"
    "$repo/tests/persona/session.sh" --stop "$pdir/session.env" >/dev/null 2>&1
    none_left "persona session" "$pruntime"
fi
