import sys, os, time, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from multi import *
L, S, O = start_multi()
print(L.dir, O, L.pids, flush=True)
def inj(ev): S.ctl("inject", stdin=json.dumps(ev))
XP, TI, CO = O["xp"], O["tiling"], O["construct"]
inj([{"pointer_move":{"position":[50,705],"output":XP}},{"click":{}}]); time.sleep(1.5)
inj([{"click":{}}]) if False else None
inj([{"pointer_move":{"position":[100,372],"output":XP}},{"click":{}}]); time.sleep(6)
print(S.query("id:s1"), flush=True)
S.call("xp","window.give",{"surface":"s1","to":"tiling"}); time.sleep(3)
print(S.query("id:s1")[0].get("owner"), flush=True)
# spread super drag
inj([{"pointer_move":{"position":[640,360],"output":TI}}]); time.sleep(0.5)
inj([{"key":{"key":"Super_L","state":"pressed"}}]); time.sleep(0.3)
inj([{"button":{"state":"pressed","hold":True}}]); time.sleep(0.3)
for i in range(10):
    inj([{"pointer_move":{"position":[640+i*60,360],"output":TI}}]); time.sleep(0.1)
inj([{"pointer_move":{"position":[100,360],"output":CO}}]); time.sleep(0.2)
inj([{"pointer_move":{"position":[640,360],"output":CO}}]); time.sleep(0.5)
inj([{"button":{"state":"released"}},{"key":{"key":"Super_L","state":"released"}}]); time.sleep(3)
print("after spread", S.query("id:s1")[0].get("owner"), flush=True)
