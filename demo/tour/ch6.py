import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *
from slides import slide

OUT = "/home/user/log/media/2026-11-12/tour-v3"
L = SceneScript("cube", "scene-cube.sh", "light")
S = Session(L.env, L.pids, BIN)
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("6 xp cube", "06-cube")
def front():
    for n in range(4):
        q = S.query(f"id:construct-cube-face-{n}")
        if q and q[0].get("forward"):
            return n
    for n in S.query("*"):
        pass
    return None
try:
    t.jump(900, 330)
    t.wait(2)
    t.act("look", "Look at the cube in use mode", "The front face fills the screen: xp with sunset and chora windows, a STEP BACK Alt+Esc chip", still="06-01-use-mode")
    t.done()
    t.act("key", "Press Alt+Escape (step back)", "The camera steps back: the whole face shows with chips TURN LEFT, TURN RIGHT, STEP IN", settle=3.0, still="06-02-stepped-back")
    t.key("Alt+Escape"); t.done()
    t.act("look", "Look at the floating windows", "The windows float in front of the face on their own planes", settle=0.5, still="06-03-floating")
    t.wait(1.5); t.done()
    t.act("key", "Press Ctrl+Alt+Right (turn by key), look mid-turn", "The cube starts to turn to the next face", settle=0.5, still="06-04-turn-key-mid")
    t.key("Ctrl+Alt+Right"); t.done()
    t.act("look", "Wait for the turn to end", "The next face (desktop 2: wobbly.rs and load) stands in front", settle=2.5, still="06-05-turned-key")
    t.wait(1.5); t.done()
    t.act("pointer", "Drag that starts on a floating window (load), from right to left", "The cube turns (or the window moves)", settle=0.5, still=None)
    t.drag(900, 400, 500, 400, seconds=1.2, hold=0.2, mid_still="06-06-drag-on-window-mid")
    t.wait(0.3); t.done()
    t.act("look", "Look at the face after that drag", "Either the face turned or the window moved on the face", settle=1.0, still="06-07-after-drag-on-window")
    t.wait(1.0); t.done()
    t.act("pointer", "Drag that starts on the empty desktop (top right), right to left", "The cube follows the drag and turns to the next face", settle=0.5, still=None)
    t.drag(1000, 200, 450, 200, seconds=1.2, hold=0.2, mid_still="06-06b-drag-turn-mid")
    t.wait(0.3); t.done()
    t.act("look", "Wait for the turn to settle", "Face 3 (cube.rs and chart) stands in front", settle=2.5, still="06-07b-turned-drag")
    t.wait(2.0); t.done()
    t.act("pointer", "Move the pointer off the face, then onto it", "The edge of the face lights when the pointer is over it", settle=1.0, still="06-08-hover-off")
    t.move(1000, 80, 1.0); t.done()
    t.act("pointer", "Move onto the face", "The edge of the face is lit", settle=1.0, still="06-09-hover-edge")
    t.move(640, 330, 1.0); t.done()
    t.flush("chips")
    cx, cy = S.centre("id:construct-mode-turn-left", "HEADLESS-1")
    t.act("pointer", "Click the TURN LEFT chip", "The cube turns back one face", settle=3.0, still="06-10-turn-chip")
    t.move(cx, cy); t.click(); t.done()
    t.act("key", "Press Ctrl+Alt+Left (turn back)", "The cube is on the first face again (sunset, chora)", settle=2.5, still="06-11-turn-back")
    t.key("Ctrl+Alt+Left"); t.done()
    t.flush("front")
    sx, sy = S.centre("id:construct-mode-step-in", "HEADLESS-1")
    t.act("pointer", "Click the face to step in", "The camera goes into the face: the xp desktop at 1:1 with the STEP BACK chip", settle=3.0, still="06-12-step-in")
    t.move(640, 330, 1.0); t.click(); t.done()
    t.act("key", "Press Alt+Escape (step back again)", "The camera steps back from the face", settle=3.0, still="06-13-step-back-again")
    t.key("Alt+Escape"); t.done()
    t.act("camera glide (method)", "Glide the camera to a view from the side", "The face seen at an angle, windows on it", settle=0.5, still="06-14-stood-off", lead=0.5)
    t.call("construct", "camera.glide", {"x": -360, "z": -600, "yaw": 0.38, "pitch": -0.02, "ms": 1400}); t.wait(1.8); t.done()
    t.act("pointer", "Drag the title bar of the sunset window off the face", "The window leaves the face and becomes a pane in the space, with the label SUNSET", settle=0.5, still=None)
    t.drag(330, 186, 870, 250, seconds=1.6, hold=0.3, mid_still="06-15-pulling"); t.done()
    t.act("look", "Wait for the pane to settle", "The pane floats beside the face, the taskbar button of sunset is gone", settle=2.5, still="06-16-pane")
    t.wait(1.5); t.done()
    t.act("pointer", "Super-drag the pane back onto the face", "While dragged the pane moves over the face", settle=0.3, still=None)
    t.drag(850, 340, 400, 300, seconds=1.6, mods=("Super_L",), hold=0.3, mid_still="06-17-drag-back-mid"); t.done()
    t.act("look", "Wait after the drop", "The pane is an xp window again on the face, its task button is back", settle=2.5, still="06-18-dropped")
    t.wait(1.5); t.done()
    t.wait(1)
    slide(f"{L.dir}/slide.png", 6, "the xp cube", ['Four xp desktops on the faces of a cube.', 'Step back, turn by key and by drag, pull a window out', 'of its face and drop it back.'], accent=(224, 175, 104))
    t.finish("06-cube.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch6.json", "w"), indent=1)
finally:
    L.stop()
