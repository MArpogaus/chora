#!/bin/bash
# Gate check 6 of 0.8: the tool list of `choractl mcp` matches the
# exported methods of the plugins after each methods.export. One tool
# per exported method is the design of D34 and task T6; this script
# says so while the server still folds them into call_method. It also
# exports a method of its own to check a later export, not only the
# plugins that start with the session.
#
# Usage: tests/gate-0.8-6.sh   BIN: directory with the binaries
set -u
. "$(dirname "$0")/gate-lib.sh"
for what in chora choractl chora-tiling; do
    test -x "$bin/$what" || fail "no $bin/$what; build it first"
done
need python3
session_start 800x600 -- "$bin/chora-tiling"
out=$(mcp_py <<'PY' 2>&1
import json, os, socket, sys
from gate_mcp import Mcp

mcp = Mcp()
# A second exporter that joins after the start, so a later
# methods.export is part of the check.
stream = socket.socket(socket.AF_UNIX)
stream.connect(os.environ["CHORA_SOCKET"])
lines = stream.makefile("w")
for i, (m, p) in enumerate([("hello", {"name": "gate-extra"}),
                            ("methods.export", {"names": [{
                                "name": "ping_pong",
                                "description": "Answers pong.",
                                "input_schema": {"type": "object",
                                                 "properties": {}}}]})]):
    lines.write(json.dumps({"jsonrpc": "2.0", "id": i, "method": m,
                            "params": p}) + "\n")
lines.flush()
tools = {t["name"] for t in mcp.request("tools/list")["tools"]}
listed = {p["name"]: p for p in mcp.json("list_plugins")}
missing = []
for plugin in listed.values():
    methods = [m if isinstance(m, str) else m["name"] for m in plugin["methods"]]
    missing += [f"{plugin['name']}.{m}" for m in methods
                if f"{plugin['name']}.{m}" not in tools]
if not missing:
    print("every exported method has its tool")
    sys.exit(0)
if len(missing) == sum(len(p["methods"]) for p in listed.values()):
    sys.exit("the server still folds plugin methods into call_method; "
             "one tool per method comes with rmcp (T6)")
sys.exit(f"tools missing for the exported methods: {missing}")
PY
)
rc=$?
if ((rc == 1)) && grep -q "rmcp (T6)" <<<"$out"; then
    skip "$out"
fi
((rc == 0)) || fail "the tool list: $out"
ok "the tool list matches the exported methods of the plugins"
