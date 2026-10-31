#!/bin/bash
# W12 cross: chora-mc on the first output and chora-tiling with --output
# on the second share a session; then confined agents on client sockets
# try what their capabilities must refuse: settings.set, session.quit,
# updates of window manager nodes, commands through plugin methods, and
# huge transactions.
WF=w12; STOP_ON_DEATH=1; . "$(dirname "$0")/lib.sh"
config_home
pwned=/tmp/bt-$WF-pwned
rm -f "$pwned".*
session_start "1280x800,1280x800+1280+0" sh -c "tests/battle/keep.sh /tmp/bt-$WF-mc.pid $bin/chora-mc & tests/battle/keep.sh /tmp/bt-$WF-tiling.pid $bin/chora-tiling --output HEADLESS-2 & wait" || exit 1
ready mc scene.list '{}' || finding "W12: mc silent at start" wrong "start mc + tiling"
ready tiling get_layout '{}' || finding "W12: tiling silent at start" wrong "start mc + tiling"
rss_note start
c0=$(rss chora)

# 1. new windows with two window managers: each one has one owner
for i in $(seq 6); do open $((i*40)) 200 90 >/dev/null; done
inject '[{"pointer_move": {"position": [600, 400], "output": "HEADLESS-2"}}, {"button": {"state": "pressed"}}, {"button": {"state": "released"}}]' >/dev/null
for i in $(seq 4); do open 90 $((i*40)) 200 >/dev/null; done
wait_count "app_id:chora-probe kind:surface" 10 || finding "W12: $(count_windows) of 10 windows" wrong "10 runs with mc + tiling"
sleep 2
owners=$(ctl query "app_id:chora-probe kind:surface" | python3 -c 'import json,sys,collections
ns=json.load(sys.stdin)
print(dict(collections.Counter(str(n.get("owner")) for n in ns)), sum(1 for n in ns if not n["rects"]))')
step "owners and windows without rects: $owners"
case $owners in *None*) finding "W12: windows without owner with mc + tiling: $owners" wrong "10 windows, mc on HEADLESS-1, tiling --output HEADLESS-2";; esac
health "two WMs"

# 2. confined agents
box0=$(admin clients create box0 | python3 -c 'import json,sys; print(json.load(sys.stdin).get("path",""))')
printf "box1:\n  grants: [methods.call]\n" | admin clients up - >> "$RUN" 2>&1
box1=$(dirname "$CHORA_SOCKET")/clients/box1/ipc.sock
[ -S "$box1" ] || box1=
[ -n "$box0" ] && [ -n "$box1" ] || { finding "W12: clients create failed" wrong "clients create box0, box1 methods.call"; }
win=$(ctl query "owner:tiling app_id:chora-probe kind:surface" | python3 -c 'import json,sys; x=json.load(sys.stdin); print(x[0]["id"] if x else "")')
timeout 120 python3 -I - "$box0" "$box1" "$win" "$pwned" >> "$RUN" 2>&1 <<'EOF'
import json, socket, sys, time
box0, box1, win, pwned = sys.argv[1:5]
leaks = 0
def connect(path, name):
    s = socket.socket(socket.AF_UNIX); s.settimeout(15); s.connect(path)
    f = s.makefile()
    def rpc(m, p=None, i=[0]):
        i[0] += 1
        try:
            s.sendall(json.dumps({"jsonrpc": "2.0", "id": i[0], "method": m, "params": p or {}}).encode() + b"\n")
        except OSError as e:
            return {"error": {"message": f"closed: {e}"}}
        while True:
            try:
                line = f.readline()
            except OSError as e:
                return {"error": {"message": f"closed: {e}"}}
            if not line:
                return {"error": {"message": "closed"}}
            r = json.loads(line)
            if r.get("id") == i[0]:
                return r
    rpc("hello", {"name": name})
    return s, rpc
def must_fail(what, r):
    global leaks
    if "error" in r:
        print(f"refused: {what}: {r['error'].get('message', '')[:90]}")
    else:
        print(f"LEAKED: {what}: {str(r.get('result'))[:90]}")
        leaks += 1
def show(what, r):
    print(f"answer: {what}: {str(r.get('error') or r.get('result'))[:110]}")

_, b0 = connect(box0, "box0")
must_fail("box0 methods.call tiling settings.set", b0("methods.call", {"client": "tiling", "method": "settings.set", "params": {"gap": 40}}))
must_fail("box0 session.quit", b0("session.quit"))
must_fail("box0 methods.call mc app.open", b0("methods.call", {"client": "mc", "method": "app.open", "params": {"command": f"touch {pwned}.box0"}}))
# D58: updates of nodes of window managers
show("box0 update opacity of a tiling window", b0("scene.apply", {"commands": [{"update": {"id": win, "props": {"opacity": 0.1}}}]}))
show("box0 update meta window of a tiling window", b0("scene.apply", {"commands": [{"update": {"id": win, "props": {"meta": {"window": "[]"}}}}]}))
for node in ["tiling", "mc", "mc-bg", "mc-hud", "root"]:
    show(f"box0 update opacity 0 of {node}", b0("scene.apply", {"commands": [{"update": {"id": node, "props": {"opacity": 0.0}}}]}))
must_fail("box0 update size of a tiling window", b0("scene.apply", {"commands": [{"update": {"id": win, "props": {"size": [5, 5]}}}]}))
must_fail("box0 remove a tiling window", b0("scene.apply", {"commands": [{"remove": {"id": win}}]}))
# huge transactions: 10000 creates (0.8 MiB), then a line past 1 MiB
cmds = [{"create": {"id": f"n{i}", "kind": "rect", "parent": "root", "props": {"size": [1, 1]}}} for i in range(10000)]
t = time.time(); show("box0 10000 creates in one transaction", b0("scene.apply", {"commands": cmds})); print(f"  took {time.time()-t:.1f} s")
deep = [{"create": {"id": "d0", "kind": "group", "parent": "root", "props": {}}}] + \
       [{"create": {"id": f"d{i}", "kind": "group", "parent": f"d{i-1}", "props": {}}} for i in range(1, 5000)]
t = time.time(); show("box0 a chain of 5000 nested groups", b0("scene.apply", {"commands": deep})); dt = time.time() - t; print(f"  took {dt:.1f} s" + (" SLOW" if dt > 3 else ""))
big = {"commands": [{"create": {"id": "big", "kind": "text", "parent": "root", "props": {"text": "x" * 1100000}}}]}
show("box0 a line past 1 MiB", b0("scene.apply", big))

_, b1 = connect(box1, "box1")
must_fail("box1 methods.call tiling settings.set", b1("methods.call", {"client": "tiling", "method": "settings.set", "params": {"gap": 40}}))
must_fail("box1 methods.call mc settings.reload", b1("methods.call", {"client": "mc", "method": "settings.reload", "params": {}}))
must_fail("box1 session.quit", b1("session.quit"))
# commands through methods that are not settings.*
r = b1("methods.call", {"client": "mc", "method": "app.open", "params": {"command": f"touch {pwned}.app-open"}})
show("box1 methods.call mc app.open touch", r)
r = b1("methods.call", {"client": "mc", "method": "scene.save", "params": {"name": "evil", "app": f"touch {pwned}.scene-save"}})
show("box1 methods.call mc scene.save app touch", r)
show("box1 methods.call mc scene.show evil", b1("methods.call", {"client": "mc", "method": "scene.show", "params": {"name": "evil"}}))
show("box1 methods.call tiling set_layout", b1("methods.call", {"client": "tiling", "method": "set_layout", "params": {"layout": "max"}}))
time.sleep(3)
sys.exit(1 if leaks else 0)
EOF
# shellcheck disable=SC2181 # the status is that of the here-document command above
[ $? = 0 ] || finding "W12: a confined client got through a refusal" crash "see LEAKED lines in $RUN"
grep -q "SLOW" "$RUN" && finding "W12: a confined client stalls the core with 5000 nested groups ($(grep -o "took [0-9.]* s SLOW" "$RUN" | tail -1))" slow "box0 scene.apply: chain of 5000 nested create group"
for f in "$pwned".*; do
  [ -e "$f" ] && finding "W12: a client with only methods.call ran a command through mc (${f##*.})" crash "clients create box1 methods.call; methods.call mc ${f##*.} {command: touch $f}"
done
health "confined agents"
ready mc scene.list '{}' || finding "W12: mc silent after the confined agents" wrong "confined agents"
ready tiling get_layout '{}' || finding "W12: tiling silent after the confined agents" wrong "confined agents"
OUT=HEADLESS-1 alive_frames frames-mc ctl call mc scene.show '{"name": "agenda"}'
OUT=HEADLESS-2 alive_frames frames-tiling ctl call tiling set_layout '{"layout": "fair"}'

rss_note end
rss_growth "$c0" "$(rss chora)" chora
health "end of W12"
log_scan
session_stop
