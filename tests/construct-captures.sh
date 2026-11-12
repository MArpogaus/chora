#!/bin/bash
# Captures of the construct in its six states at one output scale, for a
# pixel review: cube stood back, stood back with a pane beside it,
# stepped into a face, first program flown in, walk with three panes and
# the F1 overlay.
#
# Usage: tests/construct-captures.sh OUT_DIR [SCALE [STATE...]]
#   OUT_DIR  where PREFIX-NAME-Nx.png files go (PREFIX is r4, or $PREFIX)
#   SCALE    1 or 2 (default 1); the output is 1280x800 logical pixels, so
#            1280x800 or 2560x1600 in the PNG
#   STATE    only these states (default: all), by the NAME of the file
#
# A face is slow to paint at 2x: a shot waits until the frame is not
# black, 20 s at most (LIT_TRIES polls of half a second, 40 by default).
#
# Needs the binaries of target/debug (cargo build --workspace). The flags
# --test-injected-press-is-real let the injected keys reach the construct.
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
out=${1:?usage: $0 OUT_DIR [SCALE]}
scale=${2:-1}
shift $(($# < 2 ? $# : 2))
only=" $* "
prefix=${PREFIX:-r4}
mkdir -p "$out"

session=
stop() {
    [[ -n $session ]] || return 0
    kill "$session" 2>/dev/null || true
    wait "$session" 2>/dev/null || true
    session=
}
trap stop EXIT

begin() {
    local log env_file
    log=$(mktemp)
    CHORA_TEST_FLAGS=--test-injected-press-is-real \
        "$repo/tests/persona/session.sh" construct "$((1280 * scale))x$((800 * scale))@$scale" >"$log" 2>&1 &
    session=$!
    for _ in $(seq 1 100); do
        env_file=$(sed -n 's/^PERSONA_ENV=//p' "$log" | head -1)
        [[ -n $env_file && -f $env_file ]] && break
        sleep 0.3
    done
    [[ -n ${env_file:-} && -f $env_file ]] || {
        echo "no session: $(tail -5 "$log")" >&2
        exit 1
    }
    export PERSONA_ENV=$env_file
    sleep 2
}

act() { python3 -I "$repo/tests/persona/act.py" "$@" >/dev/null; }
key() { act key "$1"; sleep "${2:-1.5}"; }
lit() { python3 -I -c 'import sys; from PIL import Image; sys.exit(not sum(Image.open(sys.argv[1]).convert("L").resize((64, 40)).getdata()) > 20 * 64 * 40)' "$1"; }
shot() {
    local path
    for _ in $(seq "${LIT_TRIES:-40}"); do
        path=$(python3 -I "$repo/tests/persona/act.py" look --max-side 4000 | head -1 | cut -d" " -f1)
        lit "$path" && break
        sleep 0.5
    done
    cp "$path" "$out/$prefix-$1-${scale}x.png"
    echo "$out/$prefix-$1-${scale}x.png"
}
want() { [[ $only == "  " || $only == *" $1 "* ]]; }

if want start-card; then
    begin
    shot start-card
    stop
fi

if want cube-stood-back; then
    begin
    key Super+c 2.5
    key Alt+Escape 2.5
    shot cube-stood-back
    stop
fi

if want cube-stood-back-with-pane; then
    begin
    key Super+t 4
    key Alt+Escape 2.5
    key a 1.5
    key a 1.5
    key Super+c 2.5
    key Alt+Escape 2.5
    shot cube-stood-back-with-pane
    stop
fi

if want stepped-into-face; then
    begin
    key Super+c 2.5
    shot stepped-into-face
    stop
fi

if want first-program-flown-in; then
    begin
    key Super+t 4
    shot first-program-flown-in
    stop
fi

if want walk-3-panes || want f1-overlay || want help; then
    begin
    for _ in 1 2 3; do
        key Super+t 3.5
        key Alt+Escape 2
    done
    want walk-3-panes && shot walk-3-panes
    want help && {
        key Super+slash 1.5
        shot help
        key Super+slash 1.5
    }
    want f1-overlay && {
        key F1 1.5
        shot f1-overlay
    }
    stop
fi
