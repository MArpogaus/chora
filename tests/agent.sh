#!/bin/bash
# End-to-end test of milestone M4, the agent interface: chora runs nested
# in a headless sway with chora-tiling and two foot windows. An agent
# calls the methods that chora-tiling exports.
#
# Usage: tests/agent.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl and chora-tiling
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, python3, pgrep and a Mesa driver.
set -euo pipefail

bin=$(realpath "$1")
out=$(realpath "${2:-$(mktemp -d)}")
mkdir -p "$out"
# A short path, because a socket path has at most 107 bytes.
XDG_RUNTIME_DIR=$(mktemp -d /tmp/chora-agent-e2e.XXXXXX)
export XDG_RUNTIME_DIR
pids=()
cleanup() {
    kill "${pids[@]}" 2>/dev/null || true
    rm -rf "$XDG_RUNTIME_DIR"
}
trap cleanup EXIT
fail() {
    echo "FAIL: $*" >&2
    grim "$out/failure.png" 2>/dev/null || true
    "$bin/choractl" query "*" >"$out/failure.json" 2>/dev/null || true
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

step "1. chora -- chora-tiling starts nested with two windows"
"$bin/chora" -- "$bin/chora-tiling" --gap 0 >"$out/chora.log" 2>&1 &
chora_pid=$!
pids+=("$chora_pid")
wait_for compgen -G "$XDG_RUNTIME_DIR/chora/*.sock"
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/*.sock")
export CHORA_SOCKET
inner=$(basename "$CHORA_SOCKET" .sock)
wait_for pgrep -x chora-tiling
ctl() { "$bin/choractl" --name agent "$@"; }
surfaces() { ctl query kind:surface; }
count_is() { test "$(surfaces | jq "[.[] | select($2)] | length")" = "$1"; }
for _ in 1 2; do
    WAYLAND_DISPLAY=$inner foot >>"$out/foot.log" 2>&1 &
    pids+=($!)
done
wait_for count_is 2 '.owner == "tiling"'

# Succeeds when the surfaces stand at the x positions, in any order, for
# example "0 640".
xs_are() {
    test "$(surfaces | jq -r '[.[].transform.matrix[12]] | map(floor) | sort | map(tostring) | join(" ")')" = "$1"
}
ys_are() {
    test "$(surfaces | jq -r '[.[].transform.matrix[13]] | map(floor) | sort | map(tostring) | join(" ")')" = "$1"
}

step "2. tiling.get_layout and tiling.set_layout switch the layout"
ctl call tiling get_layout >"$out/layout.json" || fail "get_layout failed"
jq -e '.layout == "master_stack" and (.windows | length) == 2' "$out/layout.json" >/dev/null ||
    fail "layout: $(cat "$out/layout.json")"
wait_for xs_are "0 640"
ctl call tiling set_layout '{"layout": "rows"}' >"$out/rows.json" || fail "set_layout failed"
jq -e '.layout == "rows"' "$out/rows.json" >/dev/null || fail "rows: $(cat "$out/rows.json")"
wait_for xs_are "0 0"
wait_for ys_are "0 400"
ctl call tiling set_layout '{"layout": "columns"}' >/dev/null || fail "set_layout failed"
wait_for xs_are "0 640"

step "3. bad calls fail with a reason"
if ctl call tiling no_such_method 2>"$out/unknown.err"; then fail "an unknown method answered"; fi
grep -q "no exported method" "$out/unknown.err" || fail "unknown: $(cat "$out/unknown.err")"
if ctl call tiling set_layout '{"layout": "spiral"}' 2>"$out/bad.err"; then
    fail "a bad layout applied"
fi
grep -q "spiral" "$out/bad.err" || fail "bad layout: $(cat "$out/bad.err")"

step "4. a call fails when the exporter gives no answer, or disconnects"
# An exporter that ignores "wait" and exits on "quit".
python3 -I - "$CHORA_SOCKET" <<'EOF' >"$out/exporter.log" 2>&1 &
import json, socket, sys
s = socket.socket(socket.AF_UNIX)
s.connect(sys.argv[1])
f = s.makefile()
def send(i, method, params):
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": method,
                          "params": params}).encode() + b"\n")
send(0, "hello", {"name": "silent"})
send(1, "methods.export", {"names": ["wait", "quit"]})
for line in f:
    message = json.loads(line)
    print(line, end="", flush=True)
    if message.get("params", {}).get("method") == "quit":
        sys.exit(0)
EOF
pids+=($!)
wait_for grep -q '"id":1' "$out/exporter.log"
start=$SECONDS
if ctl call silent wait 2>"$out/wait.err"; then fail "a call without answer succeeded"; fi
grep -q "no answer in time" "$out/wait.err" || fail "wait: $(cat "$out/wait.err")"
test $((SECONDS - start)) -ge 9 || fail "the call ended early"
if ctl call silent quit 2>"$out/quit.err"; then fail "a call to a closed client succeeded"; fi
grep -q "disconnected" "$out/quit.err" || fail "quit: $(cat "$out/quit.err")"
kill -0 "$chora_pid" || fail "chora died"

echo "PASS (logs and screenshots in $out)"
