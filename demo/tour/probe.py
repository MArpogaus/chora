"""Starts the xp session as the tour does, clicks Start and Notepad with no editor installed
and prints what the notification bus saw, so a notice is judged from the bus and not from frames."""
import json, os, subprocess, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import launch
from launch import *
from tour import Session

launch.ENTRIES[:] = []
L = Launcher("probe")
yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
open(f"{L.dir}/xp.yaml", "w").write(yaml)
env = L.start([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml"], flavor="xp", xp=True,
              companions=[[f"{BIN}/chora-notify", "--assets", f"{REPO}/flavors/xp/assets/notify"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)


def inj(events):
    S.ctl("inject", stdin=events)


try:
    time.sleep(6)
    mon = subprocess.Popen(["busctl", "--user", "monitor", "--json=short", "org.freedesktop.Notifications"], env=env, stdout=open(f"{L.dir}/bus.out", "w"), stderr=subprocess.STDOUT)
    time.sleep(1)
    c = S.centre("id:xp-start-HEADLESS-1", "HEADLESS-1")
    inj(json.dumps([{"pointer_move": {"position": list(c)}}, {"button": {"button": 272, "state": "pressed"}}, {"button": {"button": 272, "state": "released"}}]))
    time.sleep(1)
    for n in range(40):
        q = S.query(f"id:xp-item-{n}")
        if q and (q[0].get("meta") or {}).get("text") == "Notepad":
            r = q[0]["rects"]["HEADLESS-1"]
            inj(json.dumps([{"pointer_move": {"position": [r[0] + r[2] / 2, r[1] + r[3] / 2]}}, {"button": {"button": 272, "state": "pressed"}}, {"button": {"button": 272, "state": "released"}}]))
            break
    for k in (1, 3, 6):
        time.sleep(k if k == 1 else 2 if k == 3 else 3)
        S.ctl("capture", f"/tmp/claude-0/probe-{k}.png", "--output", "HEADLESS-1", check=False)
    time.sleep(float(sys.argv[1]) if len(sys.argv) > 1 else 3)
    print("NOTIFY", subprocess.run(["pgrep", "-a", "chora-notify"], capture_output=True, text=True).stdout)
    mon.terminate()
    print(open(f"{L.dir}/bus.out").read()[-3000:])
    print("SURFACES", [(n["id"], n.get("app_id")) for n in S.query("kind:surface")])
finally:
    L.stop()
