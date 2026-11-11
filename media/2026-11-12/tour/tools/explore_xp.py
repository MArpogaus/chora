import sys, os, time, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *
L = Launcher("xp")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
os.makedirs(f"{L.dir}/config/chora", exist_ok=True)
yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("programs: []", programs_yaml(stub)).replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
open(f"{L.dir}/xp.yaml", "w").write(yaml)
env = L.start([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml"], xp=True,
              companions=[[f"{BIN}/chora-notify", "--assets", f"{REPO}/flavors/xp/assets/notify"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(4)
L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
time.sleep(2)
L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4, "said": "Good morning! Your notes for Friday are ready."})
time.sleep(3)
for n in S.query("*"):
    if n["id"].startswith("hud"): print(n["id"], n.get("rects"), json.dumps(n.get("meta"))[:100])
S.ctl("capture", f"{L.dir}/h.png")
print(L.dir, L.pids)
