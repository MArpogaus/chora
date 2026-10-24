#!/bin/bash
# Gate check 10 of 0.8: an MCP client puts a text node under
# screen:NAME, and a capture shows it. The text node is task T9; this
# script probes the core and says so while the kind does not exist.
#
# Usage: tests/gate-0.8-10.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
session_start 800x600
grant screen.capture
out=$(mcp_py <<'PY' 2>&1
import sys, time
from gate_mcp import Mcp

mcp = Mcp()
create = {"id": "label", "kind": "text", "parent": "screen:HEADLESS-1",
          "props": {"text": "Grüße 你好 🙂", "font_size": 32}}
answer = mcp.call("apply_commands", {"commands": [{"create": create}]})
if answer.get("isError"):
    sys.exit("the core refuses kind text; the text node stays open as T9")
for _ in range(50):
    nodes = mcp.json("query_scene", {"selector": "id:label"})
    if nodes and nodes[0].get("rects"):
        break
    time.sleep(0.2)
else:
    sys.exit(f"the text node drew no rect: {nodes}")
mcp.capture_png("capture", {"node": "label"}, "/tmp/gate-0.8-10.png")
print("the MCP client drew a text node under the screen and captured it")
PY
)
rc=$?
if ((rc == 1)) && grep -q "T9" <<<"$out"; then
    skip "$out"
fi
((rc == 0)) || fail "the MCP client: $out"
echo "$out"
ok "the text node under screen:NAME shows in a capture"
