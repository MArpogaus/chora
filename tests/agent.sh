#!/bin/bash
# End-to-end test of milestone M4, the agent interface: chora runs nested
# in a headless sway with chora-tiling and two foot windows. An agent
# calls the methods that chora-tiling exports, gets grants, captures the
# output and nodes, types into a terminal, clicks a node and presses a
# shortcut of a plugin. Injected safety keys do nothing, and while the
# session is locked, captures and injections fail.
#
# Usage: tests/agent.sh BIN_DIR [OUT_DIR]
#   BIN_DIR  holds chora, choractl, chora-tiling and chora-probe
#   OUT_DIR  receives the logs and screenshots (default: a temp dir)
# Needs: sway, foot, grim, jq, python3, pgrep, swaylock and a Mesa driver.
set -euo pipefail

bin=$(realpath "$1")
tests=$(realpath "$(dirname "$0")")
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

# Prints the color of a pixel of a PNG file as "r g b a".
png_pixel() { python3 -I "$tests/png-pixel.py" "$@"; }
# Prints the color of a pixel of a screenshot of the host as "r g b". The
# header of a 1280x800 PPM file has 16 bytes.
host_pixel() {
    grim -t ppm "$out/host.ppm"
    od -An -tu1 -j $((16 + ($2 * 1280 + $1) * 3)) -N 3 "$out/host.ppm" | awk '{print $1, $2, $3}'
}
# Succeeds when two colors differ by at most 8 in each of r, g and b.
close_to() {
    awk -v a="$1" -v b="$2" 'BEGIN {
        split(a, x, " "); split(b, y, " ")
        for (i = 1; i <= 3; i++) if (x[i] - y[i] > 8 || y[i] - x[i] > 8) exit 1
    }'
}

step "5. screen.capture needs a grant"
if ctl capture "$out/denied.png" 2>"$out/denied.err"; then fail "a capture without grant worked"; fi
grep -q "needs a grant" "$out/denied.err" || fail "denied: $(cat "$out/denied.err")"
"$bin/choractl" grant agent screen.capture || fail "the grant failed"
"$bin/choractl" grants | jq -e '.agent == ["screen.capture"]' >/dev/null ||
    fail "grants: $("$bin/choractl" grants)"

step "6. a capture of the output shows what the host shows"
ctl capture "$out/screen.png" >"$out/screen.json" || fail "the capture failed"
jq -e '.width == 1280 and .height == 800 and .scale == 1' "$out/screen.json" >/dev/null ||
    fail "picture: $(cat "$out/screen.json")"
for point in "320 400" "960 400" "5 5" "1270 790"; do
    read -r x y <<<"$point"
    inner_color=$(png_pixel "$out/screen.png" "$x" "$y")
    host_color=$(host_pixel "$x" "$y")
    close_to "$inner_color" "$host_color" ||
        fail "pixel $point: $inner_color, the host shows $host_color"
done
ctl capture --max-side 640 "$out/small.png" >"$out/small.json" || fail "the small capture failed"
jq -e '.width == 640 and .height == 400 and .scale == 0.5' "$out/small.json" >/dev/null ||
    fail "small picture: $(cat "$out/small.json")"

step "7. a capture of a node shows the node on its own"
echo '[{"create": {"id": "swatch", "kind": "rect", "parent": "root",
        "props": {"size": [100, 50], "color": [1, 0, 0, 1],
                  "transform": {"matrix": [1,0,0,0, 0,1,0,0, 0,0,1,0, 1100,700,0,1]}}}}]' |
    ctl apply || fail "the swatch failed"
ctl capture --node swatch "$out/swatch.png" >"$out/swatch.json" || fail "the node capture failed"
jq -e '.width == 100 and .height == 50 and .origin == [0, 0]' "$out/swatch.json" >/dev/null ||
    fail "swatch: $(cat "$out/swatch.json")"
test "$(png_pixel "$out/swatch.png" 50 25)" = "255 0 0 255" ||
    fail "swatch pixel: $(png_pixel "$out/swatch.png" 50 25)"
first=$(surfaces | jq -r '[.[].id] | sort | first')
size=$(surfaces | jq -c --arg id "$first" '.[] | select(.id == $id) | .surface.size')
ctl capture --node "$first" "$out/window.png" >"$out/window.json" ||
    fail "the window capture failed"
jq -e --argjson size "$size" '[.width, .height] == $size' "$out/window.json" >/dev/null ||
    fail "window: $(cat "$out/window.json"), the surface has $size"

step "8. a revoke ends the grant"
"$bin/choractl" revoke agent || fail "the revoke failed"
if ctl capture "$out/revoked.png" 2>/dev/null; then fail "a capture after the revoke worked"; fi
kill -0 "$chora_pid" || fail "chora died"

step "9. injected text reaches the focused window"
inject() { echo "$1" | ctl inject; }
if inject '[{"text": {"text": "x"}}]' 2>"$out/inject-denied.err"; then
    fail "an injection without grant worked"
fi
grep -q "needs a grant" "$out/inject-denied.err" || fail "denied: $(cat "$out/inject-denied.err")"
"$bin/choractl" grant agent input.inject || fail "the grant failed"
typed="$out/typed.txt"
inject "[{\"text\": {\"text\": \"echo 'Typed: A-Z!' >$typed\\n\"}}]" || fail "the text failed"
wait_for test -s "$typed"
test "$(cat "$typed")" = "Typed: A-Z!" || fail "typed: $(cat "$typed")"

step "10. an injected click reaches plugins, and a shortcut reaches its grab"
echo '[{"create": {"id": "button", "kind": "rect", "parent": "root",
        "props": {"size": [80, 40], "color": [0, 0, 1, 1], "input": "accept",
                  "transform": {"matrix": [1,0,0,0, 0,1,0,0, 0,0,1,0, 1000,600,0,1]}}}}]' |
    ctl apply || fail "the button failed"
ctl events pointer.pressed pointer.released >"$out/pointer.jsonl" &
pids+=($!)
sleep 0.3
inject '[{"pointer_move": {"position": [1040, 620]}}, {"click": {}}]' || fail "the click failed"
wait_for grep -q '"event":"pointer.released","node":"button"' "$out/pointer.jsonl"
grep -q '"event":"pointer.pressed","node":"button","button":272' "$out/pointer.jsonl" ||
    fail "press: $(cat "$out/pointer.jsonl")"
inject '[{"shortcut": {"keys": "Super+Return"}}]' || fail "the shortcut failed"
wait_for count_is 3 '.owner == "tiling"'

step "11. injected keys never reach the safety layer"
resets() { grep -c "emergency reset" "$out/chora.log" || true; }
before=$(resets)
inject '[{"shortcut": {"keys": "Ctrl+Alt+Escape"}}, {"shortcut": {"keys": "Ctrl+Alt+BackSpace"}}]' ||
    fail "the safety keys failed"
sleep 1
kill -0 "$chora_pid" || fail "an injected Ctrl+Alt+BackSpace stopped chora"
test "$(resets)" = "$before" || fail "an injected Ctrl+Alt+Escape reset"
count_is 3 '.owner == "tiling"' || fail "the windows lost their plugin"

step "12. while the session is locked, no capture and no injection"
"$bin/choractl" grant agent screen.capture || fail "the grant failed"
WAYLAND_DISPLAY=$inner swaylock -c ff0000 >"$out/swaylock.log" 2>&1 &
swaylock_pid=$!
pids+=("$swaylock_pid")
red() { test "$(host_pixel 640 400)" = "255 0 0"; }
wait_for red
if ctl capture "$out/locked.png" 2>"$out/locked.err"; then fail "a capture while locked worked"; fi
grep -q "locked" "$out/locked.err" || fail "capture: $(cat "$out/locked.err")"
if inject '[{"text": {"text": "x"}}]' 2>"$out/locked-inject.err"; then
    fail "an injection while locked worked"
fi
grep -q "locked" "$out/locked-inject.err" || fail "inject: $(cat "$out/locked-inject.err")"
# A new lock client takes over from the killed one and unlocks.
kill -KILL "$swaylock_pid"
wait "$swaylock_pid" 2>/dev/null || true
WAYLAND_DISPLAY=$inner timeout 20 "$bin/chora-probe" take-lock >"$out/probe.log" 2>&1 ||
    fail "the takeover: $out/probe.log"
ctl capture "$out/unlocked.png" >/dev/null || fail "no capture after the unlock"

echo "PASS (logs and screenshots in $out)"
