#!/bin/bash
# Measures the cost of live panes in a 3D context (D75, section 2.4): 8
# probe windows on a ring, each committing a new color on every frame
# callback, once with a still camera and once while the camera walks. Prints
# the interval of the frame callbacks per round (a core that keeps up shows
# the refresh period) and the number of `layers.dropped` events. Not part of
# CI; run it inside the chora session of the reference hardware with a
# release build.
#
# Usage: tests/3d-bench.sh [BIN_DIR [FRAMES]]
#   BIN_DIR  directory with chora and chora-probe (default: target/debug of
#            the repo)
#   FRAMES   frames per probe and round (default 300)
#
# With CHORA_SOCKET and WAYLAND_DISPLAY set, it uses that session. Without
# them it starts a headless session, which measures the software renderer.
# Exit status: 0 when every round finished and no layer was dropped.
set -eu

repo=$(cd "$(dirname "$0")/.." && pwd)
bin=$(realpath "${1:-$repo/target/debug}")
frames=${2:-300}
out=$(mktemp -d /tmp/3d-bench.XXXXXX)
session=
cleanup() {
    trap - EXIT INT TERM HUP
    [[ -z $session ]] || kill "$session" 2>/dev/null || true
    rm -rf "$out"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

if [[ -z ${CHORA_SOCKET:-} || -z ${WAYLAND_DISPLAY:-} ]]; then
    CHORA_BIN="$bin" "$repo/tests/headless-session.sh" 1280x800 -- sleep 600 \
        >"$out/session.out" 2>"$out/session.log" &
    session=$!
    for _ in $(seq 300); do
        grep -q '^CHORA_SOCKET=' "$out/session.out" 2>/dev/null && break
        kill -0 "$session" 2>/dev/null || break
        sleep 0.1
    done
    grep -q '^CHORA_SOCKET=' "$out/session.out" || {
        echo "3d-bench: no headless session; last lines of its log:" >&2
        tail -n 20 "$out/session.log" >&2 || true
        exit 1
    }
    for name in XDG_RUNTIME_DIR WAYLAND_DISPLAY CHORA_SOCKET; do
        export "$name=$(sed -n "s/^$name=//p" "$out/session.out")"
    done
fi

python3 -I - "$CHORA_SOCKET" "$bin" "$frames" <<'EOF'
import json, math, select, socket, subprocess, sys, time

socket_path, bin_dir, frames = sys.argv[1], sys.argv[2], int(sys.argv[3])
probe = f"{bin_dir}/chora-probe"
conn = socket.socket(socket.AF_UNIX)
conn.connect(socket_path)
reader = conn.makefile()
events = []
serial = 0


def read_message():
    return json.loads(reader.readline())


def call(method, params):
    global serial
    serial += 1
    conn.sendall(json.dumps({"jsonrpc": "2.0", "id": serial, "method": method,
                             "params": params}).encode() + b"\n")
    while True:
        message = read_message()
        if message.get("id") == serial:
            if "error" in message:
                sys.exit(f"3d-bench: {method}: {message['error']}")
            return message["result"]
        events.append(message)


def identity():
    return [1.0 if i % 5 == 0 else 0.0 for i in range(16)]


def times(a, b):
    return [sum(a[k * 4 + r] * b[c * 4 + k] for k in range(4))
            for c in range(4) for r in range(4)]


def translate(x, y, z):
    m = identity()
    m[12:15] = [x, y, z]
    return m


def turn_y(degrees):
    s, c = math.sin(math.radians(degrees)), math.cos(math.radians(degrees))
    m = identity()
    m[0], m[2], m[8], m[10] = c, -s, s, c
    return m


def chain(*matrices):
    result = identity()
    for m in matrices:
        result = times(result, m)
    return result


call("hello", {"name": "bench"})
call("events.subscribe", {"events": ["layers.dropped"]})
outputs = subprocess.run([f"{bin_dir}/choractl", "outputs"],
                         capture_output=True, text=True, check=True)
width, height = json.loads(outputs.stdout)[0]["size"]
eye_distance = float(width)
call("scene.apply", {"commands": [
    {"create": {"id": "bench-space", "kind": "group", "parent": "root",
                "props": {"size": [width, height], "perspective": eye_distance}}},
    {"create": {"id": "bench-eye", "kind": "group", "parent": "bench-space"}},
]})


def ring(index):
    angle = math.radians(index * 45)
    return chain(
        translate(width / 2 + width * 0.3 * math.sin(angle),
                  height * 0.15 + index * height * 0.1,
                  eye_distance * 0.3 * (1 - math.cos(angle))),
        turn_y(20), translate(-100, -100, 0))


def surfaces(count):
    deadline = time.time() + 20
    while time.time() < deadline:
        found = call("scene.query", {"selector": "kind:surface"})
        if len(found) >= count:
            return [node["id"] for node in found][:count]
        time.sleep(0.1)
    sys.exit("3d-bench: the probe windows did not show")


def round_of(label, walk):
    probes = [subprocess.Popen([probe, "live", str(frames)], stdout=subprocess.PIPE,
                               text=True) for _ in range(8)]
    ids = surfaces(8)
    commands = []
    for index, node in enumerate(ids):
        commands.append({"attach": {"surface": node, "parent": "bench-eye"}})
        commands.append({"update": {"id": node, "props": {
            "transform": {"matrix": ring(index)}}}})
    call("scene.apply", {"commands": commands})
    step = 0
    while any(p.poll() is None for p in probes):
        if walk:
            step += 1
            camera = chain(translate(-60 * math.sin(step / 20), 0, 80 * math.sin(step / 33)),
                           turn_y(2 * math.sin(step / 25)))
            call("scene.apply", {"commands": [
                {"update": {"id": "bench-eye", "props": {"transform": {"matrix": camera}}}}]})
        time.sleep(0.016)
    means, worst = [], 0.0
    for p in probes:
        line = p.stdout.read().strip()
        print(f"  {line}")
        fields = line.split()
        if p.returncode != 0 or "mean" not in fields:
            sys.exit(f"3d-bench: a probe failed in round {label}: {line!r}")
        means.append(float(fields[fields.index("mean") + 1]))
        worst = max(worst, float(fields[fields.index("max") + 1]))
    print(f"{label}: mean of the means {sum(means) / len(means):.2f} ms, "
          f"worst max {worst:.2f} ms")
    return worst


round_of("still camera", False)
round_of("walking camera", True)
call("scene.query", {"selector": "id:bench-space"})
dropped = [e for e in events
           if e.get("params", {}).get("event") == "layers.dropped"
           and e["params"].get("nodes")]
print(f"layers.dropped events: {len(dropped)}")
sys.exit(1 if dropped else 0)
EOF
