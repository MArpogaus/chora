#!/bin/bash
# Captures of the construct in its six states at one output scale, for a
# pixel review: cube stood back, stood back with a pane beside it,
# stepped into a face, first program flown in, walk with three panes and
# the F1 overlay.
#
# Usage: tests/construct-captures.sh OUT_DIR [SCALE]
#   OUT_DIR  where r3-NAME-Nx.png files go
#   SCALE    1 or 2 (default 1); the output is 1280x800 logical pixels, so
#            1280x800 or 2560x1600 in the PNG
#
# Needs the binaries of target/debug (cargo build --workspace). The flags
# --test-injected-press-is-real let the injected keys reach the construct.
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
out=${1:?usage: $0 OUT_DIR [SCALE]}
scale=${2:-1}
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
shot() {
    local path
    path=$(python3 -I "$repo/tests/persona/act.py" look --max-side 4000 | head -1 | cut -d" " -f1)
    cp "$path" "$out/r3-$1-${scale}x.png"
    echo "$out/r3-$1-${scale}x.png"
}

begin
key Super+c 2.5
key Alt+Escape 2.5
shot cube-stood-back
stop

begin
key Super+t 4
key Alt+Escape 2.5
key q 1
key Super+c 2.5
key Alt+Escape 2.5
shot cube-stood-back-with-pane
stop

begin
key Super+c 2.5
shot stepped-into-face
stop

begin
key Super+t 4
shot first-program-flown-in
stop

begin
for _ in 1 2 3; do
    key Super+t 3.5
    key Alt+Escape 2
done
shot walk-3-panes
key F1 1.5
shot f1-overlay
stop
