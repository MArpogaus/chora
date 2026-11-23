#!/bin/bash
# repro-sl3: one client stalls the core with one big transaction (sl-3 of
# sim-long r1, W12 step "10000 creates in one transaction"). A confined
# client (box0, as in W12) sends 10000 `create rect` under root in one
# scene.apply; the core must answer within MAX_S seconds and stay alive.
# Red today: the main loop is busy for minutes (debug build) and the hang
# watchdog aborts the core after 30 s. N (default 10000) sets the size,
# MAX_S (default 5) the bound. Exit 1 = red, 0 = green.
# Clean up a run that was cut off with tests/battle/kill-session.sh repro-sl3.
WF=repro-sl3; . "$(dirname "$0")/lib.sh"
RUN=${RUN:-/tmp/bt-$WF.run}; : > "$RUN"
N=${N:-10000}; MAX_S=${MAX_S:-5}
config_home
session_start "1280x800" || exit 1
box0=$(admin clients create box0 | python3 -c 'import json,sys; print(json.load(sys.stdin).get("path",""))')
[ -S "$box0" ] || { echo "no box0 socket" >&2; session_stop; exit 2; }
core=$(cat "/tmp/bt-$WF.core")
rss_note start
timeout 200 python3 -I - "$box0" "$N" "$MAX_S" <<'EOF'
import json, socket, sys, time
path, n, max_s = sys.argv[1], int(sys.argv[2]), float(sys.argv[3])
s = socket.socket(socket.AF_UNIX); s.settimeout(120); s.connect(path); f = s.makefile()
def rpc(i, m, p):
    s.sendall(json.dumps({"jsonrpc": "2.0", "id": i, "method": m, "params": p}).encode() + b"\n")
    while True:
        line = f.readline()
        if not line: return {"error": {"message": "closed"}}
        r = json.loads(line)
        if r.get("id") == i: return r
rpc(1, "hello", {"name": "box0"})
cmds = [{"create": {"id": f"n{i}", "kind": "rect", "parent": "root", "props": {"size": [1, 1]}}} for i in range(n)]
t = time.time()
try:
    r = rpc(2, "scene.apply", {"commands": cmds})
    what = str(r.get("error") or "ok")[:100]
except OSError as e:
    what = f"no answer: {e}"
dt = time.time() - t
print(f"repro-sl3: {n} creates in one scene.apply: {dt:.1f} s ({what})")
sys.exit(1 if dt > max_s else 0)
EOF
rc=$?
grep -a "main loop" "$CHORA_LOG" | sed 's/^/repro-sl3: core log: /' | tail -3
if [ -d "/proc/$core" ] && [ "$(cat "/proc/$core/comm" 2>/dev/null)" = chora ]; then
  rss_note end; echo "repro-sl3: core alive, $(tail -1 "$RUN")"
else
  echo "repro-sl3: core dead: $(grep -a "the core ended with" "$hlenv.err" | tail -1)"; rc=1
fi
session_stop
[ $rc = 0 ] && echo "repro-sl3: GREEN" || echo "repro-sl3: RED"
exit $rc
