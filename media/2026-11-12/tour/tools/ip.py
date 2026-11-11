# ip.py SESSION_DIR ACTION...: interactive helper on a live session (explore only)
import sys, os, json, subprocess, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import BIN
d = sys.argv[1]
kv = dict(l.split("=",1) for l in open(f"{d}/headless.out").read().splitlines() if "=" in l) if os.path.exists(f"{d}/headless.out") else dict(l.split("=",1) for l in open(f"{d}/scene.out").read().splitlines() if "=" in l)
env = dict(os.environ, PATH=f"{BIN}:{os.environ['PATH']}", CHORA_SOCKET=kv["CHORA_SOCKET"], XDG_RUNTIME_DIR=kv["XDG_RUNTIME_DIR"], WAYLAND_DISPLAY=kv.get("WAYLAND_DISPLAY",""))
def ctl(*a, stdin=None): return subprocess.run([f"{BIN}/choractl", *a], env=env, input=stdin, capture_output=True, text=True).stdout
def inject(ev): return ctl("inject", stdin=json.dumps(ev))
cmd = sys.argv[2]
if cmd == "cap": ctl("capture", sys.argv[3]); print("ok")
elif cmd == "key": inject([{"shortcut":{"keys":sys.argv[3]}}])
elif cmd == "click": inject([{"pointer_move":{"position":[float(sys.argv[3]),float(sys.argv[4])]}},{"click":{}}])
elif cmd == "move": inject([{"pointer_move":{"position":[float(sys.argv[3]),float(sys.argv[4])]}}])
elif cmd == "text": inject([{"text":{"text":sys.argv[3]}}])
elif cmd == "call": print(ctl("call", sys.argv[3], sys.argv[4], sys.argv[5] if len(sys.argv)>5 else "{}"))
elif cmd == "q":
    for n in json.loads(ctl("query", sys.argv[3])): print(n["id"], n.get("owner"), n.get("rects"), json.dumps(n.get("meta"))[:120])
if cmd == "clicko": inject([{"pointer_move":{"position":[float(sys.argv[3]),float(sys.argv[4])],"output":sys.argv[5]}},{"click":{}}])
if cmd == "capo": ctl("capture", sys.argv[3], "--output", sys.argv[4]); print("ok")
if cmd == "keyo": inject([{"shortcut":{"keys":sys.argv[3]}}])
