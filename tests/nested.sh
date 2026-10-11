#!/bin/bash
# End-to-end test of milestone M1: chora runs nested in a headless sway,
# chora-tiling tiles three foot windows, a crash of the plugin leaves the
# core running, and the emergency reset puts the windows in the grid. A
# second chora checks SIGUSR1 while its startup command runs.
#
# The test starts the reset with SIGUSR1. Ctrl+Alt+Escape runs the same
# code, but synthetic keys from wtype carry their own keymap, which a nested
# compositor cannot see.
#
# Usage: tests/nested.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl and chora-tiling
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, python3.
set -euo pipefail

bin=$(realpath "$1")
out=$(realpath "${2:-$(mktemp -d)}")
mkdir -p "$out"
# A short path, because a socket path has at most 107 bytes.
XDG_RUNTIME_DIR=$(mktemp -d /tmp/chora-e2e.XXXXXX)
export XDG_RUNTIME_DIR
pids=()
cleanup() {
    kill "${pids[@]}" 2>/dev/null || true
    rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    exit 1
}
step() { echo "== $*"; }

# Waits up to 10 s for a command to succeed.
wait_for() {
    for _ in $(seq 50); do
        if "$@" >/dev/null 2>&1; then return 0; fi
        sleep 0.2
    done
    fail "timeout: $*"
}

step "host: headless sway"
printf 'output HEADLESS-1 resolution 1280x800\ndefault_border none\n' >"$out/sway.conf"
WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c "$out/sway.conf" >"$out/sway.log" 2>&1 &
pids+=($!)
wait_for test -S "$XDG_RUNTIME_DIR/wayland-1"
export WAYLAND_DISPLAY=wayland-1

step "1. chora -- chora-tiling starts nested"
"$bin/chora" -- "$bin/chora-tiling" >"$out/chora.log" 2>&1 &
chora_pid=$!
pids+=("$chora_pid")
wait_for compgen -G "$XDG_RUNTIME_DIR/chora/*.sock"
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/*.sock")
export CHORA_SOCKET
inner=$(basename "$CHORA_SOCKET" .sock)
wait_for pgrep -x chora-tiling

surfaces() { "$bin/choractl" query kind:surface; }
# Succeeds when exactly N surfaces match the jq condition.
count_is() { test "$(surfaces | jq "[.[] | select($2)] | length")" = "$1"; }

step "2. three foot windows appear without overlap"
for _ in 1 2 3; do
    WAYLAND_DISPLAY=$inner foot >>"$out/foot.log" 2>&1 &
    pids+=($!)
done
wait_for count_is 3 '.owner == "tiling"'
sleep 0.5
surfaces >"$out/tiled.json"
jq -e '
  [.[] | {x: .transform.matrix[12], y: .transform.matrix[13],
          w: .surface.size[0], h: .surface.size[1]}] as $r
  | [range(0; $r | length) as $i | range($i + 1; $r | length) as $j
     | ($r[$i]) as $a | ($r[$j]) as $b
     | ($a.x < $b.x + $b.w and $b.x < $a.x + $a.w and
        $a.y < $b.y + $b.h and $b.y < $a.y + $a.h)]
  | any | not' "$out/tiled.json" >/dev/null || fail "windows overlap: $out/tiled.json"
grim "$out/tiled.png"
# The header of a 1280x800 PPM file has 16 bytes.
grim -t ppm - | tail -c +17 | od -An -v -tu1 -w3 | sort -u | wc -l >"$out/tiled.colors"
test "$(cat "$out/tiled.colors")" -gt 3 || fail "the screenshot is empty"

step "3. a crash of chora-tiling leaves chora running and detaches the surfaces"
pkill -KILL -x chora-tiling
wait_for count_is 3 '.parent == null and .owner == null'
kill -0 "$chora_pid" || fail "chora died"

step "4. a transaction that arrives with the end of the stream still applies"
first=$(surfaces | jq -r '[.[].id] | sort | first')
python3 -I - "$CHORA_SOCKET" "$first" >"$out/eof.json" <<'EOF'
import json, socket, sys
path, surface = sys.argv[1:]
calls = [
    ("hello", {"name": "e2e", "keep_nodes": True}),
    ("scene.apply", {"commands": [
        {"attach": {"surface": surface, "parent": "root"}},
        {"configure": {"surface": surface, "size": [300, 200]}},
    ]}),
]
s = socket.socket(socket.AF_UNIX)
s.connect(path)
s.sendall(b"".join(
    json.dumps({"jsonrpc": "2.0", "id": i, "method": m, "params": p}).encode() + b"\n"
    for i, (m, p) in enumerate(calls)))
s.shutdown(socket.SHUT_WR)
print(s.makefile().read(), end="")
EOF
jq -se 'map(.result) == [null, null]' "$out/eof.json" >/dev/null || fail "answers: $out/eof.json"
count_is 1 '.owner == "e2e"' || fail "the transaction did not apply"

step "5. the emergency reset puts the three surfaces in the grid"
kill -USR1 "$chora_pid"
wait_for count_is 3 '.parent == "root"'
surfaces >"$out/grid.json"
jq -e '[.[] | [.transform.matrix[12], .transform.matrix[13]]] | sort == [[0,0],[0,400],[640,0]]' \
    "$out/grid.json" >/dev/null || fail "unexpected grid: $out/grid.json"
grim "$out/grid.png"

step "6. SIGUSR1 resets while the startup command runs, which blocks no signal"
"$bin/chora" -- sh -c "grep SigBlk /proc/self/status >'$out/sigblk'; exec sleep 600" \
    >"$out/chora-signal.log" 2>&1 &
signal_pid=$!
pids+=("$signal_pid")
wait_for test -s "$out/sigblk"
grep -Eq 'SigBlk:\s*0+$' "$out/sigblk" || fail "blocked signals: $(cat "$out/sigblk")"
kill -USR1 "$signal_pid"
sleep 1
kill -0 "$signal_pid" || fail "chora died of SIGUSR1"

echo "PASS (logs and screenshots in $out)"
