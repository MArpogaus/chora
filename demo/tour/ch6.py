import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from human import Human
from tour import Session
from slides import slide

OUT = os.environ.get("TOUR_OUT", "/home/user/log/media/2026-11-16/tour-v3")
L = SceneScript("cube", "scene-cube.sh", "light")
S = Session(L.env, L.pids, BIN)
t = Human(S, OUT, f"{L.dir}/work")
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
    t.reach(1000, 80); t.done()
    t.act("pointer", "Move onto the face", "The edge of the face is lit", settle=1.0, still="06-09-hover-edge")
    t.reach(640, 330); t.done()
    t.flush("chips")
    cx, cy = S.centre("id:construct-mode-turn-left", "HEADLESS-1")
    t.act("pointer", "Click the TURN LEFT chip", "The cube turns back one face", settle=3.0, still="06-10-turn-chip")
    t.press(cx, cy); t.done()
    t.act("key", "Press Ctrl+Alt+Left (turn back)", "The cube is on the first face again (sunset, chora)", settle=2.5, still="06-11-turn-back")
    t.key("Ctrl+Alt+Left"); t.done()
    t.flush("front")
    sx, sy = S.centre("id:construct-mode-step-in", "HEADLESS-1")
    t.act("pointer", "Click the face to step in", "The camera goes into the face: the xp desktop at 1:1 with the STEP BACK chip", settle=3.0, still="06-12-step-in")
    t.press(640, 330); t.done()
    t.flush("inuse")
    edge = json.loads(S.ctl("add-output", "640x720+1280+0"))["name"]
    t.act("pointer", "Drag the title bar of the dusk window over the edge of the face", "The window leaves the face over the edge of the screen and becomes a pane in the space", settle=0.5, still=None)
    t.move(870, 225, 1.0)
    t.inject([{"button": {"button": 272, "state": "pressed", "hold": True}}])
    t.wait(0.3)
    t.move(1270, 260, 1.2)
    t.pointer_output = edge
    t.inject([{"pointer_move": {"position": [260, 340]}}])
    t.wait(0.5)
    t.still("06-15-pulling")
    t.inject([{"button": {"button": 272, "state": "released"}}])
    t.pointer_output = None
    t.done()
    t.act("key", "Press Alt+Escape (step back again)", "The camera steps back from the face; the pulled window is a pane beside the cube", settle=3.0, still="06-16-pane")
    t.key("Alt+Escape"); t.done()
    t.flush("pulled")
    pane = next((n for n in S.query("*") if str(n.get("id", "")).startswith("construct-pane-") and (n.get("rects") or {}).get("HEADLESS-1")), None)
    assert pane is not None, "the pulled window has no pane that is drawn"
    px, py, pw, ph = pane["rects"]["HEADLESS-1"]
    faces = [r for r in (S.rect(f"id:construct-cube-face-{n}", "HEADLESS-1") for n in range(4)) if r]
    fx, fy, fw, fh = max(faces, key=lambda r: r[2] * r[3])
    t.act("pointer", "Super-drag the pane back onto the face", "While dragged the pane moves over the face", settle=0.3, still=None)
    t.drag(px + pw / 2, py + ph / 2, fx + fw * 0.4, fy + fh * 0.5, seconds=1.6, mods=("Super_L",), hold=0.3, mid_still="06-17-drag-back-mid"); t.done()
    t.act("look", "Wait after the drop", "The pane is an xp window again on the face, its task button is back", settle=2.5, still="06-18-dropped")
    t.wait(1.5); t.done()
    t.wait(1)
    slide(f"{L.dir}/slide.png", 6, "the xp cube", ['Four xp desktops on the faces of a cube.', 'Step back, turn by key and by drag, pull a window out', 'of its face and drop it back.'], accent=(224, 175, 104))
    t.finish("06-cube.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch6.json", "w"), indent=1)
finally:
    L.stop()
