#!/bin/bash
# Gate check 7 of 0.8: a capture shows the injection mark while a client
# injects input, and a capture later shows it gone. Headless, no GPU; a
# capture still needs the software renderer of Mesa.
#
# Usage: tests/injection-indicator.sh
#   BIN  directory with chora and choractl (default: target/debug)
set -u
repo=$(cd "$(dirname "$0")/.." && pwd)
bin=${BIN:-$repo/target/debug}
out=$(mktemp -d /tmp/chora-mark.XXXXXX)
cleanup() {
    [[ -n "${inject:-}" ]] && kill "$inject" 2>/dev/null
    [[ -n "${session:-}" ]] && kill "$session" 2>/dev/null
    wait 2>/dev/null
    rm -rf "$out"
}
trap cleanup EXIT

"$repo/tests/headless-session.sh" 400x300 >"$out/env" 2>&1 &
session=$!
for _ in $(seq 60); do
    grep -q CHORA_SOCKET "$out/env" && break
    sleep 0.5
done
grep -q CHORA_SOCKET "$out/env" || {
    echo "FAIL: no core" >&2
    cat "$out/env" >&2
    exit 1
}
eval "$(sed -n 's/^\(CHORA_SOCKET\|WAYLAND_DISPLAY\)=\(.*\)/export \1="\2"/p' "$out/env")"
ctl() { "$bin/choractl" --name agent "$@"; }
"$bin/choractl" grant agent input.inject screen.capture || {
    echo "FAIL: the grant" >&2
    exit 1
}

# Keep injecting, so the mark lives across the two captures.
(
    trap 'exit 0' TERM
    for _ in $(seq 100); do
        echo '[{"pointer_move": {"position": [200, 150]}}]' | ctl inject || exit 0
        sleep 0.1
    done
) &
inject=$!
sleep 1
ctl capture "$out/mark.png" || {
    echo "FAIL: the capture" >&2
    exit 1
}
# The mark: a 12x12 yellow rect of the core at 8,8 right of the pointer.
color=$(python3 -I "$repo/tests/png-pixel.py" "$out/mark.png" 210 160)
[[ "$color" == "255 255 0 255" ]] || {
    echo "FAIL: at the pointer: $color" >&2
    exit 1
}
echo "ok: the capture shows the injection mark"

# The mark goes half a second after the last injected event.
kill "$inject"
sleep 1
ctl capture "$out/gone.png" || {
    echo "FAIL: the second capture" >&2
    exit 1
}
color=$(python3 -I "$repo/tests/png-pixel.py" "$out/gone.png" 210 160)
[[ "$color" == "0 0 0 255" ]] || {
    echo "FAIL: after the injection: $color" >&2
    exit 1
}
echo "ok: the mark is gone after half a second"
