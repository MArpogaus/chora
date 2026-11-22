"""Chapter construct: one function `run(ctx)`; the rows are `t.act(..., expect=, contract=, cells=, capture=)`."""

import sys, time, os, json, subprocess
from launch import *
import launch
from human import Quick
from tour import Session, checker_for
from harness import Expect
from slides import slide

def pane_and_face(S, O):
    """The rects of the pane of s1 and of the face it goes back to; an error when one is missing,
    so the act is not skipped silently (rt-5)."""
    pane = S.rect("id:construct-pane-s1", O)
    face = S.rect("id:construct-cube-face-0", O)
    if not pane:
        raise RuntimeError("no pane: id:construct-pane-s1")
    if not face:
        raise RuntimeError("no face: id:construct-cube-face-0")
    return pane, face


def window_back_on_face(after):
    """`Expect.verify`: s1 is a window of xp again, no longer a pane of construct."""
    owners = [n.get("owner") for n in after.nodes if n.get("id") == "s1"]
    if not owners:
        return ["no node s1"]
    return [f"s1 is still owned by construct: {owners}"] if "construct" in owners else []


def window_is_pane(after):
    """`Expect.verify`: s1 is a pane of construct, no longer a window of xp."""
    owners = [n.get("owner") for n in after.nodes if n.get("id") == "s1"]
    if not owners:
        return ["no node s1"]
    return [] if "construct" in owners else [f"s1 is not a pane of construct: {owners}"]


def cube_is(**fields):
    """An `Expect` on the answer of `cube.list` (`faces` by its count, `front`, `mode`, `turning`)."""
    return Expect(cube=fields)


def front_now(S):
    """The front face of the cube now."""
    return S.call("construct", "cube.list").get("front")


def sheet_open(shown):
    """An `Expect` on the key sheet of construct: open or shut (the id the doors gate reads)."""
    return Expect(open=["construct-help"] if shown else [])


def plain(t, name):
    """A key pressed and released with no modifier, as the acts of tests/acts/looks/construct.py send it."""
    t.inject([{"key": {"key": name, "state": "pressed"}}, {"key": {"key": name, "state": "released"}}])
    t.frame += 1


SHEET_OVERLAP = "I5: cb-2 the key sheet of the stood-back cube overlaps the chips Turn right and Step in (row 28, Volume down)"


def run(ctx):
    OUT = ctx.out
    L = SceneScript("cube4", "scene-cube.sh", "light")
    S = Session(L.env, L.pids, BIN)
    edge = json.loads(S.ctl("add-output", "640x720+1280+0"))["name"]
    t = Quick(S, OUT, f"{L.dir}/work", checker=checker_for(S, ["construct", "xp"], L.dir, wallpaper={"HEADLESS-1"}))
    t.begin("4 construct: a demo watcher at the cube", "04-construct")
    O = "HEADLESS-1"
    try:
        t.jump(900, 330)
        t.act("look", "A watcher at a demo stands in front of the first face", "The front face fills the screen: xp with sunset and chora windows, the chip STEP BACK", scene="c1", still="04-01-use-mode", lead=0.1, settle=1.2, cells=["doors launcher-key"], capture="Q", expect=cube_is(faces=4, front=0, mode="use"))
        t.done()
        t.flush("pull-prep")
        cx0, cy0 = S.centre("id:xp-title-s1", "VIRTUAL-1") or (200, 50)
        t.act("pointer", "Drags a window by its title bar over the edge of the face", "The window leaves the face and floats as a pane in front of the cube", scene="c5", still=None, settle=0.2, lead=0.1, contract="C2", cells=["gate-construct-pull.sh"], capture="H50", expect=Expect(verify=window_is_pane))
        t.reach(cx0, cy0, hesitate=False)
        t.inject([{"button": {"button": 272, "state": "pressed", "hold": True}}]); t.wait(0.3)
        t.move(700, 300, 0.7)
        t.inject([{"pointer_move": {"position": [200, 300], "output": edge}}]); t.wait(0.5); t.still("04-10-pull-over-edge")
        t.inject([{"pointer_move": {"position": [260, 340], "output": edge}}]); t.wait(0.4)
        t.inject([{"button": {"button": 272, "state": "released"}}]); t.cut(0.5); t.wait(1.0); t.done()
        t.act("look", "The window is a pane of the space, in front of the face", "A pane with the sunset picture floats in front of the xp face", scene="c5", still="04-11-pane-in-space", settle=1.0, lead=0.1, contract="C2", capture="Q", expect=Expect(verify=window_is_pane))
        t.done()
        t.act("key", "Alt+Escape: step back to see the pane in the space", "The cube with the pane beside it", scene="c2", still="04-12-pane-stepped-back", settle=0.8, lead=0.1, contract="C1", cells=["act: the cube opens with one xp on four faces", "act: the key of the step back stands back"], capture="Q", expect=cube_is(faces=4, mode="navigate"))
        t.key("Alt+Escape"); t.cut(0.6); t.wait(0.8); t.done()
        t.flush("back-prep")
        pr, fr = pane_and_face(S, O)
        t.act("pointer", "Drags the pane back over the face with a plain drag (v6 held Super: a script slip)", "The pane lands on the face and is a window of xp again", scene="c5", still=None, settle=0.2, lead=0.1, contract="C2", cells=["gate-construct-pull.sh", "act: a pulled window goes back to the xp of its face when its pane goes"], capture="H50", expect=Expect(verify=window_back_on_face))
        px, py = pr[0] + pr[2] / 2, pr[1] + pr[3] / 2
        t.reach(px, py, hesitate=False)
        t.inject([{"button": {"button": 272, "state": "pressed", "hold": True}}]); t.wait(0.3)
        t.move(fr[0] + fr[2] * 0.4, fr[1] + fr[3] * 0.5, 0.8); t.wait(0.3)
        t.inject([{"button": {"button": 272, "state": "released"}}]); t.cut(0.5); t.wait(1.0); t.done()
        t.act("look", "The window is back on its face", "The cube with the window on the face, no pane", scene="c5", still="04-13-back-on-face", settle=1.0, lead=0.1, contract="C2", capture="Q", expect=Expect(verify=window_back_on_face))
        t.done()
        t.act("key", "Ctrl+Alt+Right: turns the cube by key", "The cube turns to the second face", scene="c3", still=None, settle=0.1, lead=0.1, cells=["act: the turn key turns the cube", "act: the cube is held halfway through a turn"], capture="H50", expect=cube_is(front=1, turning=False))
        t.key("Ctrl+Alt+Right"); t.wait(0.5); t.still("04-03-turn-mid"); t.cut(0.5); t.wait(0.6); t.still("04-04-face-2"); t.done()
        t.act("key", "Ctrl+Alt+Right again: the third face", "The cube turns to the third face", scene="c3", still="04-06-face-3", settle=0.4, lead=0.1, cells=["act: the turn key turns the cube"], capture="Q", expect=cube_is(front=2, turning=False))
        t.key("Ctrl+Alt+Right"); t.cut(0.4); t.wait(0.6); t.done()
        t.flush("chips")
        c = S.centre("id:construct-mode-turn-left", O)
        t.act("pointer", "Clicks the TURN LEFT chip", "The cube turns back one face", scene="c3", still="04-07-chip", settle=0.4, lead=0.1, cells=["act: the pager turns the cube"], capture="Q", expect=cube_is(front=1, turning=False))
        if c:
            t.reach(*c); t.click()
        t.cut(0.6); t.wait(0.5); t.done()
        t.act("key", "Ctrl+Alt+Left: back to the first face", "The first face (sunset and chora) is in front", scene="c3", still="04-08-face-1", settle=0.5, lead=0.1, cells=["act: the turn key turns the cube"], capture="Q", expect=cube_is(front=0, turning=False))
        t.key("Ctrl+Alt+Left"); t.cut(0.6); t.wait(0.5); t.done()
        t.act("key", "Super+F1: the key overlay", "A list of the keys that work now", scene="c4", still="04-09-keys", settle=1.0, lead=0.1, contract="T2", cells=["doors keys-key"], capture="Q", opens=["construct-help"], expect=sheet_open(True), known=SHEET_OVERLAP)
        t.key("Super+F1"); t.done()
        t.act("key", "Escape closes the overlay", "The overlay closes", scene="c4", still=None, settle=0.2, lead=0.1, contract="T2", cells=["doors keys-key"], capture="Q", closes=["construct-help"], expect=sheet_open(False))
        plain(t, "Escape"); t.wait(0.3); t.done()
        t.act("key", "Return: steps into the face", "The camera goes into the face: the xp desktop at 1:1 with the chip STEP BACK", scene="c5", still="04-14-step-in", settle=0.5, lead=0.1, contract="C2", cells=["gate-construct-pull.sh"], capture="Q", expect=cube_is(mode="use"))
        plain(t, "Return"); t.cut(0.8); t.wait(0.5); t.done()
        t.act("key", "Alt+Escape: steps back out", "The whole cube again", scene="c2", still="04-15-out", settle=0.4, lead=0.1, contract="C1", cells=["act: the key of the step back stands back"], capture="Q", expect=cube_is(faces=4, mode="navigate"))
        t.key("Alt+Escape"); t.cut(0.8); t.wait(0.6); t.done()
        t.flush("drag-prep")
        front_before = front_now(S)
        t.act("pointer", "Drags the cube itself to turn it by hand", "The cube follows the pointer and turns to the third face", scene="c3", still=None, settle=0.1, lead=0.1, cells=["act: a drag on a face turns the cube"], capture="H50", expect=Expect(cube={"mode": "navigate", "turning": False}, verify=lambda after: [] if (after.cube or {}).get("front") != front_before else [f"the cube is still on face {front_before}"]))
        t.drag(780, 360, 80, 360, seconds=0.9, hold=0.1, mid_still="04-05-drag-turn")
        t.cut(0.4); t.wait(0.5); t.still("04-06-face-3"); t.done()

        t.act("look", "Stepped back, the cube at rest (0 degrees)", "The cube from outside, one face square on", scene="c3", still="04-20-seam-0", settle=0.8, lead=0.1, cells=["act: the cube is held halfway through a turn"], capture="Q", expect=cube_is(mode="navigate", turning=False))
        t.done()
        t.act("pointer", "Drags the cube half way and holds: about 45 degrees", "The cube stands between two faces, the seam between them in view", scene="c3", still=None, settle=0.1, lead=0.1, cells=["act: the cube is held halfway through a turn"], capture="H50", expect=cube_is(mode="navigate", turning=False))
        t.move(640, 360, 0.6)
        t.inject([{"button": {"button": 272, "state": "pressed", "hold": True}}]); t.wait(0.3)
        t.move(500, 360, 0.5); t.wait(0.6); t.still("04-21-seam-45a")
        t.move(460, 360, 0.3); t.wait(0.6); t.still("04-21-seam-45b")
        t.inject([{"button": {"button": 272, "state": "released"}}]); t.cut(0.5); t.wait(0.8); t.done()
        t.flush("turn-prep")
        front_at_rest = front_now(S)
        t.act("key", "Ctrl+Alt+Right: one face turned (90 degrees)", "The cube rests on the next face", scene="c3", still="04-22-seam-90", settle=0.3, lead=0.1, cells=["act: the turn key turns the cube"], capture="Q", expect=Expect(cube={"mode": "navigate", "turning": False}, verify=lambda after: [] if (after.cube or {}).get("front") == (front_at_rest + 1) % 4 else [f"the front is {(after.cube or {}).get('front')}, expected {(front_at_rest + 1) % 4}"]))
        t.key("Ctrl+Alt+Right"); t.cut(0.6); t.wait(0.8); t.done()
        slide(f"{L.dir}/slide.png", 4, "construct: the cube", ["A watcher steps back, turns the cube by key and by chip,", "steps into a face and out, and drags the cube by hand."], accent=(187, 154, 247))
        t.finish("04-construct.mp4", slide=f"{L.dir}/slide.png")
        json.dump(t.rows, open(f"{OUT}/rows-construct.json", "w"), indent=1)
    finally:
        L.stop()
