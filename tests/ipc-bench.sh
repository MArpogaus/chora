#!/bin/bash
# Measures the IPC round trip of transactions that change a mesh, as a
# plugin does in every frame for wobbly windows (D28). Not part of CI.
# Usage: tests/ipc-bench.sh BIN_DIR [OUT_DIR]; use a release build.
set -eu
bin=$(realpath "$1")
out=$(realpath "${2:-$(mktemp -d)}")
mkdir -p "$out"
XDG_RUNTIME_DIR=$(mktemp -d /tmp/bench.XXXXXX); export XDG_RUNTIME_DIR
pids=()
trap 'kill "${pids[@]}" 2>/dev/null; rm -rf "$XDG_RUNTIME_DIR"' EXIT
printf 'output HEADLESS-1 resolution 1280x800\ndefault_border none\n' >"$out/sway.conf"
WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1 \
    sway -c "$out/sway.conf" >"$out/sway.log" 2>&1 &
pids+=($!)
sleep 1
export WAYLAND_DISPLAY=wayland-1
"$bin/chora" -- sleep 120 >"$out/chora.log" 2>&1 &
pids+=($!)
sleep 1.5
CHORA_SOCKET=$(compgen -G "$XDG_RUNTIME_DIR/chora/*.sock"); export CHORA_SOCKET
python3 -I - "$CHORA_SOCKET" <<'EOF'
import json, socket, sys, time, math, statistics
s = socket.socket(socket.AF_UNIX); s.connect(sys.argv[1]); f = s.makefile()
n = 0
def call(method, params):
    global n; n += 1
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": n, "method": method, "params": params}).encode() + b"\n")
    while True:
        msg = json.loads(f.readline())
        if msg.get("id") == n: return msg
call("hello", {"name": "bench"})
call("scene.apply", {"commands": [{"create": {"id": "w", "kind": "rect", "parent": "root",
     "props": {"size": [400, 300], "color": [0.2, 0.4, 0.8, 1], "transform": {"translate": [100, 100]}}}}]})
for cells in (4, 8, 16):
    times = []
    for i in range(300):
        pts = [[x * 400 / cells + 10 * math.sin(i / 10 + y), y * 300 / cells] for y in range(cells + 1) for x in range(cells + 1)]
        t = time.perf_counter()
        r = call("scene.apply", {"commands": [{"update": {"id": "w", "props": {"mesh": {"cols": cells, "rows": cells, "points": pts}}}}]})
        times.append((time.perf_counter() - t) * 1000)
        assert "result" in r, r
    times.sort()
    print(f"mesh {cells}x{cells}: p50 {statistics.median(times):.3f} ms, p99 {times[int(len(times)*0.99)]:.3f} ms, max {times[-1]:.3f} ms")
EOF
