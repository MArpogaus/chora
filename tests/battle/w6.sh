#!/bin/bash
# W6 sandbox: on a client socket every forbidden try must be refused.
WF=w6; . "$(dirname "$0")/lib.sh"
session_start "1280x800" "$bin/chora-tiling" || exit 1
open 10 200 10; open 200 10 10
foreign=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["id"])')

boxpath=$(admin clients create box | python3 -c 'import json,sys; print(json.load(sys.stdin).get("path",""))')
[ -n "$boxpath" ] || { finding "W6: clients create box failed" wrong "choractl clients create box"; session_stop; exit 1; }

# each try must answer with an error; a result means the sandbox leaked
timeout 60 python3 -I - "$boxpath" "$foreign" >> "$RUN" 2>&1 <<'EOF'
import json, os, socket, sys
path, foreign = sys.argv[1], sys.argv[2]
def tries():
    yield ("focus_node without a press",
           {"focus_node": {"node": foreign}})
    yield ("create under screen:HEADLESS-1",
           {"create": {"id": "bx", "kind": "rect", "parent": "screen:HEADLESS-1", "props": {"size": [50, 50]}}})
    yield ("camera command",
           {"camera": {"output": "HEADLESS-1", "transform": {"translate": [10, 10]}}})
    yield ("attach a foreign window",
           {"attach": {"surface": foreign, "parent": "root"}})
s = socket.socket(socket.AF_UNIX); s.settimeout(10); s.connect(path)
f = s.makefile()
def rpc(i, m, p):
    s.sendall(json.dumps({"jsonrpc":"2.0","id":i,"method":m,"params":p}).encode()+b"\n")
    return json.loads(f.readline())
rpc(0, "hello", {"name": "box"})
leak = 0
for i, (name, cmd) in enumerate(tries()):
    r = rpc(1+i, "scene.apply", {"commands": [cmd]})
    if "error" in r:
        print(f"refused: {name}: {r['error'].get('message','')[:80]}")
    else:
        print(f"LEAKED:  {name} was accepted")
        leak += 1
sys.exit(1 if leak else 0)
EOF
# shellcheck disable=SC2181 # the status is that of the here-document command above
if [ $? != 0 ]; then
  finding "W6: client socket box accepted a forbidden command" crash "see try lines in $RUN"
fi
health "after sandbox probes"
session_stop
