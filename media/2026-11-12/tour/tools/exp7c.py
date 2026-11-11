import sys, os, time, json, subprocess
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import BIN
d="/tmp/claude-0/tour/multi.hat0hhk4"
kv = dict(l.split("=",1) for l in open(f"{d}/headless.out").read().splitlines() if "=" in l)
env = dict(os.environ, PATH=f"{BIN}:{os.environ['PATH']}", CHORA_SOCKET=kv["CHORA_SOCKET"], XDG_RUNTIME_DIR=kv["XDG_RUNTIME_DIR"])
def ctl(*a, stdin=None): return subprocess.run([f"{BIN}/choractl", *a], env=env, input=stdin, capture_output=True, text=True).stdout
def inj(ev): print(ctl("inject", stdin=json.dumps(ev)))
def owner(): return json.loads(ctl("query","id:s1"))[0].get("owner")
XP, TI, CO = "HEADLESS-2","HEADLESS-3","HEADLESS-1"
print("start", owner())
inj([{"pointer_move":{"node":"s1","point":[100,60]}},{"key":{"key":"Super_L","state":"pressed"}},{"button":{"state":"pressed","hold":True}},{"pointer_move":{"position":[640,400],"output":CO}},{"button":{"state":"released"}},{"key":{"key":"Super_L","state":"released"}}])
time.sleep(3); print("batch", owner())
def give(to):
    subprocess.run([f"{BIN}/choractl","call","construct","window.give",json.dumps({"surface":"s1","to":to})],env=env,capture_output=True); time.sleep(3)
give("tiling"); print("back in", owner())
inj([{"pointer_move":{"node":"s1","point":[100,60]}},{"key":{"key":"Super_L","state":"pressed"}},{"button":{"state":"pressed","hold":True}}])
time.sleep(1)
inj([{"pointer_move":{"position":[640,400],"output":CO}}]); time.sleep(1)
inj([{"pointer_move":{"position":[700,420],"output":CO}}]); time.sleep(1)
inj([{"button":{"state":"released"}},{"key":{"key":"Super_L","state":"released"}}])
time.sleep(3); print("jump then spread", owner())
