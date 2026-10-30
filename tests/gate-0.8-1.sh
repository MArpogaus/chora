#!/bin/bash
# Gate check 1 of 0.8: an MCP client focuses the second terminal and
# requests a move to the right; chora-tiling does both, and a capture
# shows them. Headless: two chora-probe windows stand in for terminals.
#
# Usage: tests/gate-0.8-1.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-tiling chora-probe; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
out=$(mktemp -d /tmp/gate1.XXXXXX)
trap 'rm -rf "$out"' EXIT
session_start 1280x800 -- "$bin/chora-tiling"
# `choractl mcp` is confined: tiling.focus goes through methods.call.
grant screen.capture input.inject methods.call
# On the main socket, run needs no grant (D42); it prints the pid as one JSON line
run_window() {  # run_window <r> <g> <b> <name>
    local pid
    pid=$(ctl run "$bin/chora-probe" window "$1" "$2" "$3" |
        sed -n 's/.*"pid":[[:space:]]*\([0-9]*\).*/\1/p')
    [ -n "$pid" ] || fail "the run of the $4 window"
    run_pid "$pid"
}
run_window 255 0 0 red
run_window 0 0 255 blue
for _ in $(seq 50); do
    test "$(ctl query 'app_id:chora-probe kind:surface' | grep -c '"id"')" = 2 && break
    sleep 0.2
done
mcp_out=$(mcp_py "$out" <<'PY' 2>&1
import os, sys, time
from gate_mcp import Mcp

out = sys.argv[1]
mcp = Mcp()
first, second = sorted(n["id"] for n in mcp.json(
    "query_scene", {"selector": "app_id:chora-probe kind:surface"}))
# The tool of the export itself, not call_method: `client.method` is a
# tool of its own, and its arguments are the params of the method.
for window in (first, second):
    mcp.tool("tiling.focus", {"surface": window})
focused = lambda: next(n for n in mcp.json(
    "query_scene", {"selector": "app_id:chora-probe kind:surface"})
    if n["id"] == second).get("focused")
for _ in range(50):
    if focused():
        break
    time.sleep(0.2)
else:
    sys.exit(f"tiling.focus left the second window {second} unfocused")
mcp.capture_png("capture", {"output": "HEADLESS-1"}, f"{out}/before.png")
# The move request of D31: the agent owns no window, so the core asks
# tiling, and tiling moves the window into the group on the right.
mcp.tool("apply_commands", {"commands": [{"create": {
    "id": "right", "kind": "group", "parent": "root",
    "props": {"transform": {"translate": [400, 0]}}}}]})
mcp.tool("apply_commands", {"commands": [
    {"move": {"id": second, "parent": "right"}}]})
for _ in range(50):
    if next(n for n in mcp.json(
            "query_scene", {"selector": "app_id:chora-probe kind:surface"})
            if n["id"] == second).get("parent") == "right":
        break
    time.sleep(0.2)
else:
    sys.exit(f"tiling never moved the requested window {second}")
time.sleep(0.5)
mcp.capture_png("capture", {"output": "HEADLESS-1"}, f"{out}/after.png")
print(f"the MCP client focused {second} and moved it to the right")
PY
) || fail "the MCP client: $mcp_out"
echo "$mcp_out"
test "$(pixel "$out/before.png" 100 400)" = "0 0 255 255" ||
    fail "before, left: $(pixel "$out/before.png" 100 400)"
test "$(pixel "$out/before.png" 1000 400)" = "255 0 0 255" ||
    fail "before, right: $(pixel "$out/before.png" 1000 400)"
test "$(pixel "$out/after.png" 100 400)" = "255 0 0 255" ||
    fail "after, left: $(pixel "$out/after.png" 100 400)"
test "$(pixel "$out/after.png" 800 400)" = "0 0 255 255" ||
    fail "after, right: $(pixel "$out/after.png" 800 400)"
ok "the MCP focus and the requested move show in the captures"
