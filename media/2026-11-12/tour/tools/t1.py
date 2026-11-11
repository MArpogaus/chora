import sys, time
sys.path.insert(0, '.')
from launch import *
L = Launcher("xp")
yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("programs: []", programs_yaml()).replace("test_accept_injected_keys: false","test_accept_injected_keys: true")
open(f"{L.dir}/xp.yaml","w").write(yaml)
env = L.start([f"{BIN}/chora-xp","--theme",f"{REPO}/flavors/xp/theme","--config",f"{L.dir}/xp.yaml"], xp=True,
  companions=[[f"{BIN}/chora-notify","--assets",f"{REPO}/flavors/xp/assets/notify"],[f"{BIN}/chora-assistant"]])
print(L.dir, L.core_env, L.pids)
time.sleep(5)
import subprocess
print(subprocess.run([f"{BIN}/choractl","capture",f"{L.dir}/a.png"],env=env,capture_output=True,text=True))
