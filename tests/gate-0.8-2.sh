#!/bin/bash
# Gate check 2 of 0.8: under a camera with pan and zoom, a click on a
# node through the MCP tools hits that node in 20 of 20 tries. The click
# goes in node coordinates; the core turns it into screen coordinates.
#
# Usage: tests/gate-0.8-2.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
session_start 1280x800
grant screen.capture input.inject
out=$(mcp_py <<'PY' 2>&1
import sys, time
from gate_mcp import Mcp

mcp = Mcp()
mcp.tool("subscribe_events", {"events": ["pointer.pressed"]})
mcp.tool("apply_commands", {"commands": [{"create": {
    "id": "target", "kind": "rect", "parent": "root",
    "props": {"size": [120, 80], "color": [1, 1, 0, 1], "input": "accept",
              "transform": {"translate": [200, 200]}}}}]})
# Pan and zoom the camera of the one output.
mcp.tool("apply_commands", {"commands": [{"camera": {
    "output": "HEADLESS-1",
    "transform": {"translate": [-140, -60], "scale": 1.5}}}]})
hits = 0
for _ in range(20):
    mcp.tool("inject_input", {"events": [
        {"pointer_move": {"node": "target", "point": [60, 40]}},
        {"click": {}}]})
    for _ in range(5):
        batch = mcp.json("wait_events", {"timeout_ms": 1000})
        if batch["ended"]:
            sys.exit(f"the event stream ended: {batch}")
        if any(e["event"] == "pointer.pressed" and e.get("node") == "target"
               for e in batch["events"]):
            hits += 1
            break
if hits != 20:
    sys.exit(f"only {hits} of 20 clicks hit the node")
print("the MCP client hit the node 20 of 20 times")
PY
) || fail "the MCP client: $out"
echo "$out"
ok "20 of 20 node clicks landed on the node under pan and zoom"
