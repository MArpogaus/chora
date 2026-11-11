import sys, os, time, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *
L = Launcher("c5")
env = L.start([f"{BIN}/chora-construct"], voice=False, extra_config={"chora-construct.yaml": "theme: light\nvoice_hud: false\ntest_accept_injected_keys: true\neffects: [wobbly]\n"})
S = Session(env, L.pids, BIN)
S.ctl("wait","id:construct-look","--timeout","20")
print(S.call("construct","pane.manager",{"manager":"xp","size":[560,400]}))
print(S.call("construct","pane.manager",{"manager":"tiling","size":[560,400]}))
time.sleep(8)
print(json.dumps(S.call("construct","construct.list"),indent=1))
S.ctl("capture", f"{L.dir}/c5.png")
print(L.dir, L.pids)
