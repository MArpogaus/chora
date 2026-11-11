import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
L = Launcher("c5")
env = L.start([f"{BIN}/chora-construct"], voice=False, extra_config={"chora-construct.yaml": "theme: light\nvoice_hud: false\ntest_accept_injected_keys: true\neffects: [wobbly]\n"})
S = Session(env, L.pids, BIN)
S.ctl("wait", "id:construct-look", "--timeout", "20")
S.call("construct", "pane.manager", {"manager": "xp", "size": [560, 400]})
S.call("construct", "pane.manager", {"manager": "tiling", "size": [560, 400]})
S.ctl("wait", "id:xp-bar-VIRTUAL-1", "--timeout", "30")
S.ctl("wait", "owner:tiling", "--timeout", "30")
time.sleep(5)
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("5 construct with manager panes", "05-manager-panes")
XP, TI = "VIRTUAL-1", "VIRTUAL-2"
def pane_point(manager, out, sel, fx=0.5, fy=0.5):
    t.flush("aim")
    pr = S.rect(f"id:construct-manager-{manager}-source", "HEADLESS-1")
    if sel is None:
        return pr[0] + pr[2] * fx, pr[1] + pr[3] * fy
    r = S.rect(sel, out)
    return pr[0] + (r[0] + r[2] / 2) * pr[2] / 560, pr[1] + (r[1] + r[3] / 2) * pr[3] / 400
try:
    t.jump(900, 600)
    t.wait(2)
    t.act("look", "Look at the construct with two manager panes", "xp is the pane ahead, tiling a pane to the right (black, empty), labels XP and TILING", still="05-01-start")
    t.done()
    t.act("camera glide (method)", "Glide the camera in front of the xp pane", "The xp desktop shows large in the middle of the view", settle=0.5, still="05-02-xp-view", lead=0.5)
    t.call("construct", "camera.glide", {"x": 0, "z": 900, "yaw": 0, "pitch": -0.05, "ms": 2500}); t.wait(2.8); t.done()
    sx, sy = pane_point("xp", XP, "id:xp-start-VIRTUAL-1")
    t.act("pointer", "Move to the Start button in the xp pane and click", "The Start menu opens inside the pane", settle=1.5, still="05-03-xp-start-menu")
    t.move(sx, sy); t.click(); t.done()
    t.flush("menu")
    tx, ty = pane_point("xp", XP, "id:xp-item-0")
    t.act("pointer", "Click Terminal in the menu", "A terminal window opens inside the xp pane", settle=3.5, still="05-04-xp-terminal")
    t.move(tx, ty); t.click(); t.done()
    msg = "echo hello from xp"
    t.act("keys", f"Type '{msg}' and press Return", "The text appears in the xp terminal and the shell answers hello from xp", settle=1.0, still="05-05-xp-typed")
    t.text(msg); t.key("Return"); t.done()
    t.act("camera glide (method)", "Glide the camera to the tiling pane", "The tiling pane (black) fills the middle of the view", settle=0.5, still="05-06-tiling-view", lead=0.5)
    t.call("construct", "camera.glide", {"x": 600, "z": 900, "yaw": 0.37, "pitch": -0.05, "ms": 2500}); t.wait(2.8); t.done()
    cx, cy = pane_point("tiling", TI, None)
    t.act("pointer", "Click into the tiling pane", "The pane takes the keyboard (its edge lights up)", settle=1.0, still="05-07-tiling-click")
    t.move(cx, cy); t.click(); t.done()
    t.act("key", "Press Super+Return in the tiling pane", "tiling opens a terminal that fills its pane", settle=4.0, still="05-08-tiling-terminal")
    t.key("Super+Return"); t.done()
    msg = "echo hello from tiling"
    t.act("keys", f"Type '{msg}' and press Return", "The text appears in the tiling terminal and the shell answers", settle=1.0, still="05-09-tiling-typed")
    t.text(msg); t.key("Return"); t.done()
    t.act("camera glide (method)", "Glide back to the start pose", "Both panes in view again, both with a terminal", settle=0.5, still="05-10-both", lead=0.5)
    t.call("construct", "camera.glide", {"x": 0, "z": -100, "yaw": 0.15, "pitch": -0.07, "ms": 2500}); t.wait(2.8); t.done()
    t.wait(1)
    t.finish("05-manager-panes.mp4")
    json.dump(t.rows, open(f"{OUT}/rows-ch5.json", "w"), indent=1)
finally:
    L.stop()
