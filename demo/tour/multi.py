"""Four-screen session of the tour: construct, xp, tiling and mc, one
screen each, as tests/gate-handover-matrix.sh starts them."""
import json, os, subprocess, sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import Session


def start_multi():
    L = Launcher("multi")
    os.makedirs(f"{L.dir}/stub", exist_ok=True)
    programs_yaml(f"{L.dir}/stub")
    env = L.start(None, voice=True, extra_config={
        "chora-construct.yaml": "theme: light\nvoice_hud: false\ntest_accept_injected_keys: true\neffects: [wobbly]\n",
        "chora-mc.yaml": "theme: light\ntest_accept_injected_keys: true\n",
    })
    S = Session(env, L.pids, BIN)
    S.ctl("grant", "tourrec", "input.inject", "methods.call", "methods.privileged", "screen.capture")
    for x in (1280, 2560, 3840):
        S.ctl("add-output", f"1280x720+{x}+0")
    outs = sorted(json.loads(S.ctl("outputs")), key=lambda o: o["position"][0])
    names = [o["name"] for o in outs]
    assert len(names) == 4, names
    out_of = dict(zip(["construct", "xp", "tiling", "mc"], names))
    xp_yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("programs: []", programs_yaml(f"{L.dir}/stub")).replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
    open(f"{L.dir}/xp.yaml", "w").write(xp_yaml)
    L.spawn([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml", "--output", out_of["xp"]], env, "xp.log")
    S.ctl("wait", "owner:xp", "--timeout", "30")
    L.spawn([f"{BIN}/chora-tiling", "--output", out_of["tiling"]], env, "tiling.log")
    L.spawn([f"{BIN}/chora-mc", "--manage-windows", "--output", out_of["mc"]], env, "mc.log")
    L.spawn([f"{BIN}/chora-construct"], env, "construct.log")
    for o in ("tiling", "mc", "construct"):
        S.ctl("wait", f"owner:{o}", "--timeout", "30")
    return L, S, out_of
