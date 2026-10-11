#!/bin/bash
# End-to-end test of milestone M1: chora runs nested in a headless sway,
# chora-tiling tiles three foot windows, a crash of the plugin leaves the
# core running, and the emergency reset puts the windows in the grid.
# Further steps check fixed regressions.
#
# The test starts the reset with SIGUSR1. Ctrl+Alt+Escape runs the same
# code, but synthetic keys from wtype carry their own keymap, which a nested
# compositor cannot see.
#
# Usage: tests/nested.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl and chora-tiling
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, python3, pgrep and pkill, and a Mesa driver.
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
# Saves a screenshot as FILE and prints the number of its colors. The
# header of a 1280x800 PPM file has 16 bytes.
colors() {
    grim "$1"
    grim -t ppm - | tail -c +17 | od -An -v -tu1 -w3 | sort -u | wc -l
}
# Succeeds when exactly N surfaces match the jq condition.
count_is() { test "$(surfaces | jq "[.[] | select($2)] | length")" = "$1"; }

step "2. three foot windows appear without overlap"
feet=()
for _ in 1 2 3; do
    WAYLAND_DISPLAY=$inner foot >>"$out/foot.log" 2>&1 &
    feet+=($!)
done
pids+=("${feet[@]}")
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
test "$(colors "$out/tiled.png")" -gt 3 || fail "the screenshot is empty"

step "3. a crash of chora-tiling leaves chora running and detaches the surfaces"
pkill -KILL -x chora-tiling
wait_for count_is 3 '.parent == null and .owner == null'
kill -0 "$chora_pid" || fail "chora died"

# The later steps check fixed regressions.

# Sends a hello as "e2e" and one scene.apply per argument on one
# connection, closes its side, and prints each answer with its duration.
rpc() {
    timeout 20 python3 -I - "$CHORA_SOCKET" "$@" <<'EOF'
import json, socket, sys, time
path, calls = sys.argv[1], [("hello", {"name": "e2e", "keep_nodes": True})]
calls += [("scene.apply", {"commands": json.loads(c)}) for c in sys.argv[2:]]
s = socket.socket(socket.AF_UNIX)
s.settimeout(10)
s.connect(path)
f = s.makefile()
for i, (method, params) in enumerate(calls):
    start = time.monotonic()
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": method,
                          "params": params}).encode() + b"\n")
    if i == len(calls) - 1:
        s.shutdown(socket.SHUT_WR)
    answer = json.loads(f.readline())
    answer["seconds"] = time.monotonic() - start
    print(json.dumps(answer))
EOF
}
# Succeeds when every answer in the file is a result.
all_results() { jq -se 'all(has("result"))' "$1" >/dev/null; }

step "4. a transaction that arrives with the end of the stream still applies"
first=$(surfaces | jq -r '[.[].id] | sort | first')
rpc "[{\"attach\": {\"surface\": \"$first\", \"parent\": \"root\"}},
      {\"configure\": {\"surface\": \"$first\", \"size\": [300, 200]}}]" \
    >"$out/eof.json" || fail "no answers: $out/eof.json"
all_results "$out/eof.json" || fail "answers: $out/eof.json"
count_is 1 '.owner == "e2e"' || fail "the transaction did not apply"

step "5. a client that misses a deadline delays one transaction, not each"
configure() { echo "[{\"configure\": {\"surface\": \"$first\", \"size\": $1}}]"; }
kill -STOP "${feet[@]}"
rpc "$(configure '[310, 210]')" "$(configure '[320, 220]')" >"$out/late.json" || true
kill -CONT "${feet[@]}"
all_results "$out/late.json" || fail "answers: $out/late.json"
jq -se '.[1].seconds >= 0.1 and .[2].seconds < 0.1' "$out/late.json" >/dev/null ||
    fail "waits: $out/late.json"

step "6. a connection without hello closes after 5 s"
timeout 20 python3 -I - "$CHORA_SOCKET" <<'EOF' || fail "the idle connection stayed open"
import socket, sys
s = socket.socket(socket.AF_UNIX)
s.settimeout(8)
s.connect(sys.argv[1])
sys.exit(0 if s.recv(1) == b"" else 1)
EOF

step "7. a camera pans the view of the output"
work_area() { "$bin/choractl" outputs | jq -c '.[0].work_area'; }
echo '[{"camera": {"output": "winit", "transform": {"translate": [-100, -50]}}}]' |
    "$bin/choractl" --name e2e apply || fail "the camera command failed"
test "$(work_area)" = "[100,50,1280,800]" || fail "work area: $(work_area)"
sleep 0.3
test "$(colors "$out/pan.png")" -ge 2 || fail "the panned view is empty"

step "8. the emergency reset puts the three surfaces in the grid"
kill -USR1 "$chora_pid"
wait_for count_is 3 '.parent == "root"'
test "$(work_area)" = "[0,0,1280,800]" || fail "the reset kept the camera: $(work_area)"
surfaces >"$out/grid.json"
jq -e '[.[] | [.transform.matrix[12], .transform.matrix[13]]] | sort == [[0,0],[0,400],[640,0]]' \
    "$out/grid.json" >/dev/null || fail "unexpected grid: $out/grid.json"
grim "$out/grid.png"

step "9. SIGUSR1 resets while the startup command runs, which blocks no signal"
rm -f "$out/sigblk"
# shellcheck disable=SC2016 # $1 expands in the inner shell
"$bin/chora" -- sh -c 'grep SigBlk /proc/self/status >"$1"; exec sleep 60' sh "$out/sigblk" \
    >"$out/chora-signal.log" 2>&1 &
signal_pid=$!
pids+=("$signal_pid")
wait_for grep -q "chora runs" "$out/chora-signal.log"
wait_for test -s "$out/sigblk"
grep -Eq 'SigBlk:[[:space:]]*0+$' "$out/sigblk" || fail "blocked signals: $(cat "$out/sigblk")"
kill -USR1 "$signal_pid"
wait_for grep -q "emergency reset" "$out/chora-signal.log"
kill -0 "$signal_pid" || fail "chora died of SIGUSR1"

echo "PASS (logs and screenshots in $out)"
