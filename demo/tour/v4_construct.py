import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour4 import Quick, OUT
from tour import Session
from slides import slide

L = SceneScript("cube4", "scene-cube.sh", "light")
S = Session(L.env, L.pids, BIN)
t = Quick(S, OUT, f"{L.dir}/work")
t.begin("4 construct: a demo watcher at the cube", "04-construct")
O = "HEADLESS-1"
try:
    t.jump(900, 330)
    t.act("look", "A watcher at a demo stands in front of the first face", "The front face fills the screen: xp with sunset and chora windows, the chip STEP BACK", still="04-01-use-mode", lead=0.1, settle=1.2)
    t.done()
    t.act("key", "Alt+Escape: steps back to see the whole cube", "The camera steps back; chips TURN LEFT, TURN RIGHT, STEP IN", still="04-02-stepped-back", settle=0.4, lead=0.1)
    t.key("Alt+Escape"); t.cut(0.4); t.wait(0.9); t.done()
    t.act("key", "Ctrl+Alt+Right: turns the cube by key", "The cube turns to the second face", still=None, settle=0.1, lead=0.1)
    t.key("Ctrl+Alt+Right"); t.wait(0.5); t.still("04-03-turn-mid"); t.cut(0.5); t.wait(0.6); t.still("04-04-face-2"); t.done()
    t.act("key", "Ctrl+Alt+Right again: the third face", "The cube turns to the third face", still="04-06-face-3", settle=0.4, lead=0.1)
    t.key("Ctrl+Alt+Right"); t.cut(0.4); t.wait(0.6); t.done()
    t.flush("chips")
    c = S.centre("id:construct-mode-turn-left", O)
    t.act("pointer", "Clicks the TURN LEFT chip", "The cube turns back one face", still="04-07-chip", settle=0.4, lead=0.1)
    if c:
        t.reach(*c); t.click()
    t.cut(0.6); t.wait(0.5); t.done()
    t.act("key", "Ctrl+Alt+Left: back to the first face", "The first face (sunset and chora) is in front", still="04-08-face-1", settle=0.5, lead=0.1)
    t.key("Ctrl+Alt+Left"); t.cut(0.6); t.wait(0.5); t.done()
    t.act("key", "F1: the key overlay", "A list of the keys that work now", still="04-09-keys", settle=1.0, lead=0.1)
    t.key("F1"); t.done()
    t.act("key", "Escape closes the overlay", "The overlay closes", still=None, settle=0.2, lead=0.1)
    t.tap("Escape", 0.2); t.done()
    t.act("key", "Return: steps into the face", "The camera goes into the face: the xp desktop at 1:1 with the chip STEP BACK", still="04-14-step-in", settle=0.5, lead=0.1)
    t.key("Return"); t.cut(0.8); t.wait(0.5); t.done()
    t.act("key", "Alt+Escape: steps back out", "The whole cube again", still="04-15-out", settle=0.4, lead=0.1)
    t.key("Alt+Escape"); t.cut(0.8); t.wait(0.6); t.done()
    t.act("pointer", "Drags the cube itself to turn it by hand", "The cube follows the pointer and turns to the third face", still=None, settle=0.1, lead=0.1)
    t.drag(640, 360, 380, 360, seconds=0.9, hold=0.1, mid_still="04-05-drag-turn")
    t.cut(0.4); t.wait(0.5); t.still("04-06-face-3"); t.done()

    slide(f"{L.dir}/slide.png", 4, "construct: the cube", ["A watcher steps back, turns the cube by key and by chip,", "steps into a face and out, and drags the cube by hand."], accent=(187, 154, 247))
    t.finish("04-construct.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch4.json", "w"), indent=1)
finally:
    L.stop()
