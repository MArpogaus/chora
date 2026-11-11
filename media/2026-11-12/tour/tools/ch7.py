import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from multi import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
L, S, O = start_multi()
CO, XP, TI, MC = O["construct"], O["xp"], O["tiling"], O["mc"]
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("7 window drags between looks", "07-drags")
t.observed = {}

def goto(x, y, out, sec=1.0):
    if t.pointer_output != out:
        t.pointer_output = out
        t.jump(x, y)
    else:
        t.move(x, y, sec)

def press(mods=()):
    events = [{"key": {"key": m, "state": "pressed"}} for m in mods]
    events.append({"button": {"button": 272, "state": "pressed", "hold": True}})
    t.inject(events)
    t.wait(0.3)

def on_node(out, x=100, y=60):
    t.pointer_output = out
    t.inject([{"pointer_move": {"node": "s1", "point": [x, y]}}])
    t.wait(1.0)

def release(mods=()):
    t.wait(0.3)
    events = [{"button": {"button": 272, "state": "released"}}]
    events += [{"key": {"key": m, "state": "released"}} for m in reversed(mods)]
    t.inject(events)
    t.frame += 1

def where():
    t.flush("where")
    q = S.query("id:s1")
    if not q:
        return "no window s1"
    n = q[0]
    return f"owner {n.get('owner')}, focused {n.get('focused')}, rects {n.get('rects')}"

def note(key):
    w = where()
    t.observed[key] = w
    print(key, w, flush=True)

def mid(sel, out):
    r = S.rect(sel, out)
    return r[0] + r[2] / 2, r[1] + r[3] / 2

try:
    t.pair = [XP, TI]
    t.pointer_output = XP
    t.jump(640, 360)
    t.wait(2)
    t.act("look", "Two screens: xp on the left, tiling on the right (empty)", "xp desktop with taskbar; tiling desktop empty", still="07-01-two-screens")
    t.done()
    t.act("pointer", "Click Start", "The Start menu opens on the xp screen", still="07-02-start")
    goto(50, 705, XP, 1.0); t.click(); t.done()
    t.act("pointer", "Click Notes", "A Notes window opens on the xp screen", settle=3.0, still="07-03-notes-open")
    goto(100, 372, XP, 1.0); t.click(); t.done()
    note("07-03-notes-open")
    tx, ty = mid("id:xp-title-s1", XP)
    cx, cy = 640, 400
    t.act("pointer", "DRAG 1: pull the title bar of Notes from xp to the tiling screen", "Notes leaves xp and tiles in the tiling screen (full height)", settle=2.5, still="07-04-drag-xp-to-tiling", lead=1.0)
    goto(tx, ty, XP, 1.0); press()
    goto(1100, 300, XP, 1.0)
    goto(100, 300, TI, 1.0); goto(cx, cy, TI, 0.8)
    release(); t.done()
    note("07-04-drag-xp-to-tiling")
    t.pair = [TI, CO]
    x, y = mid("id:s1", TI)
    t.act("pointer", "DRAG 2: Super+drag Notes from tiling to the construct screen", "Notes leaves tiling and becomes a pane in the construct space", settle=3.0, still="07-05-drag-tiling-to-construct")
    on_node(TI); press(("Super_L",))
    goto(100, 360, CO, 1.0); goto(640, 360, CO, 0.8)
    release(("Super_L",)); t.done()
    note("07-05-drag-tiling-to-construct")
    t.pair = [CO, XP]
    x, y = mid("id:s1", CO)
    t.act("pointer", "DRAG 3: Super+drag the Notes pane from construct to the xp screen", "Notes leaves the construct and shows in xp with a title bar and a task button", settle=3.0, still="07-06-drag-construct-to-xp")
    on_node(CO); press(("Super_L",))
    goto(100, 300, XP, 1.0); goto(500, 300, XP, 0.8)
    release(("Super_L",)); t.done()
    note("07-06-drag-construct-to-xp")
    t.pair = [XP, MC]
    tx, ty = mid("id:xp-title-s1", XP)
    t.act("pointer", "DRAG 4: pull the title bar of Notes from xp to the mc screen", "Notes becomes the window of an mc scene", settle=3.0, still="07-07-drag-xp-to-mc")
    t.pointer_output = XP; goto(tx, ty, XP, 1.0); press()
    goto(1100, 300, XP, 1.0)
    goto(100, 300, MC, 1.0); goto(640, 360, MC, 0.8)
    release(); t.done()
    note("07-07-drag-xp-to-mc")
    t.pair = [MC, XP]
    x, y = mid("id:s1", MC)
    t.act("pointer", "DRAG 5: Super+drag Notes out of mc to the xp screen", "Notes leaves mc and shows in xp", settle=3.0, still="07-08-drag-mc-out")
    on_node(MC); press(("Super_L",))
    goto(100, 300, XP, 1.0); goto(500, 300, XP, 0.8)
    release(("Super_L",)); t.done()
    note("07-08-drag-mc-out")
    t.pair = [XP, TI]
    t.act("key", "GIVE BY KEY: Super+Alt+1 (give the focused window to tiling)", "Notes goes to tiling and fills its screen", settle=3.0, still="07-09-give-key-tiling")
    t.key("Super+Alt+1"); t.done()
    note("07-09-give-key-tiling")
    t.pair = [TI, XP]
    t.act("key", "GIVE BY KEY: Super+Alt+2 (give the focused window to xp)", "Notes goes back to xp", settle=3.0, still="07-10-give-key-xp")
    t.key("Super+Alt+2"); t.done()
    note("07-10-give-key-xp")
    t.pair = [XP, TI]
    t.act("pointer", "GIVE BY MENU: Start > Control Panel", "The menu opens with Control Panel highlighted", settle=1.0, still="07-11-menu-control-panel")
    goto(50, 705, XP, 1.0); t.click(); t.wait(0.8); goto(258, 467, XP, 1.0); t.click(); t.done()
    t.act("pointer", "Click Keyboard Shortcuts", "The submenu lists Super+Alt+1: window to tiling and the others", settle=1.0, still="07-12-menu-shortcuts")
    goto(456, 551, XP, 1.0); t.click(); t.done()
    t.act("pointer", "Click Super+Alt+1: window to tiling", "Notes goes to tiling and the menu closes", settle=3.0, still="07-13-give-menu-tiling")
    goto(650, 551, XP, 1.0); t.click(); t.done()
    note("07-13-give-menu-tiling")
    t.wait(1)
    t.flush("audit")
    audit = S.query("kind:surface")
    print("AUDIT", [(n["id"], n.get("owner")) for n in audit])
    t.observed["audit"] = [(n["id"], n.get("owner")) for n in audit]
    t.finish("07-drags.mp4")
    json.dump({"rows": t.rows, "observed": t.observed}, open(f"{OUT}/rows-ch7.json", "w"), indent=1)
finally:
    L.stop()
