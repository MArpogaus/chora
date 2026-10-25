#!/bin/bash
# Gate check 3 of 0.8: wait_events returns while another tool call runs,
# and a cancelled wait_events stops. A cancelled call gets no answer, as
# the protocol and rmcp keep it; the wait still ends, which the server
# reports on stderr. This needs the rmcp server of T6; the script says
# so when the server still answers one request at a time.
#
# Usage: tests/gate-0.8-3.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
session_start 800x600
out=$(mcp_py <<'PY' 2>&1
import json, select, sys, time
from gate_mcp import Mcp

mcp = Mcp()
mcp.tool("subscribe_events", {"events": ["output.changed"]})

def ask(method, params=None):
    mcp.n += 1
    mcp.send({"jsonrpc": "2.0", "id": mcp.n,
              "method": method, "params": params or {}})
    return mcp.n

def answer(wanted, seconds):
    """The answer to the wanted id, or None, within the seconds."""
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        if not select.select([mcp.server.stdout], [], [], 0.01)[0]:
            continue
        line = mcp.server.stdout.readline()
        if not line:
            sys.exit("the server closed")
        message = json.loads(line)
        if message.get("id") == wanted:
            return message
    return None

waiting = ask("tools/call", {"name": "wait_events",
                             "arguments": {"timeout_ms": 5000}})
pinged = ask("ping")
if answer(pinged, 1) is None:
    print("the server answers one request at a time; parallel requests "
          "and cancellation come with rmcp (T6)")
    sys.exit(44)
cancelled = ask("tools/call", {"name": "wait_events",
                               "arguments": {"timeout_ms": 30000}})
mcp.notify("notifications/cancelled", {"requestId": cancelled})
if answer(cancelled, 1) is not None:
    sys.exit("the server answered the cancelled call")
pinged = ask("ping")
start = time.monotonic()
if answer(pinged, 1) is None:
    sys.exit("the server stopped answering after the cancel")
ms = 1000 * (time.monotonic() - start)
if ms > 100:
    sys.exit(f"the call after the cancel answered after {ms:.0f} ms")
print("wait_events runs in parallel; a cancelled call stops without an answer")
PY
)
rc=$?
if ((rc == 44)); then skip "$out"; fi
((rc == 0)) || fail "the MCP client: $out"
grep -q "wait_events cancelled" <<<"$out" ||
    fail "the cancel never reached the wait: $out"
echo "$out"
ok "wait_events runs in parallel and a cancel stops the wait"
