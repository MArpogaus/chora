import sys, os, time, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *
L = Launcher("tiling")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
programs_yaml(stub)
open(f"{stub}/foot", "w").write(f"""#!/bin/sh
c={L.dir}/count; n=$(cat $c 2>/dev/null || echo 0); echo $((n+1)) >$c
case $((n % 4)) in 0) k=notes;; 1) k=code;; 2) k=palette;; 3) k=page;; esac
args=""; [ $k = code ] && args={REPO}/crates/chora-kit/src/wobbly.rs
exec /usr/bin/foot --app-id=$k --title=$k -o pad=14x12 -o colors.background=1a1b26 -o colors.foreground=c0caf5 -o 'font=DejaVu Sans Mono:size=9' -o cursor.blink=no -e sh -c "python3 -I {CONTENT} $k $args; exec sleep infinity"
""")
os.chmod(f"{stub}/foot", 0o755)
env = L.start([f"{BIN}/chora-tiling"],
  companions=[["swaybg","--color","#2e3440"],["dbus-run-session","--","waybar","--config",f"{REPO}/flavors/tiling/waybar/config.jsonc"],[f"{BIN}/chora-assistant"]])
env2=dict(env)
time.sleep(6)
S = Session(env, L.pids, BIN)
S.ctl("capture", f"{L.dir}/t0.png")
print(L.dir, L.pids)
print(open(L.dir+"/headless.out").read())
