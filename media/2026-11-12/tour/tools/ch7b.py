import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from multi import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
L, S, O = start_multi()
CO, XP, TI, MC = O["construct"], O["xp"], O["tiling"], O["mc"]
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("7b give by menu", "07b-menu")
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
    t.act("pointer", "Click Start", "The Start menu opens on the xp screen", still="07b-01-start")
    goto(50, 705, XP, 1.0); t.click(); t.done()
    t.act("pointer", "Click Notes", "A Notes window opens on the xp screen", settle=3.0, still="07b-02-notes-open")
    goto(100, 372, XP, 1.0); t.click(); t.done()
    note("07b-02-notes-open")
    t.act("pointer", "GIVE BY MENU: Start > Control Panel", "The menu opens with Control Panel highlighted", settle=1.0, still="07b-03-menu-control-panel")
    goto(50, 705, XP, 1.0); t.click(); t.wait(0.8); goto(258, 467, XP, 1.0); t.click(); t.done()
    t.act("pointer", "Move right into the submenu and click Keyboard Shortcuts", "The submenu lists Super+Alt+1: window to tiling and the others", settle=1.0, still="07b-04-menu-shortcuts")
    goto(456, 467, XP, 0.8); goto(456, 551, XP, 0.8); t.click(); t.done()
    t.act("pointer", "Click Super+Alt+1: window to tiling", "Notes goes to tiling and the menu closes", settle=3.0, still="07b-05-give-menu-tiling")
    goto(650, 551, XP, 1.0); t.click(); t.done()
    note("07b-05-give-menu-tiling")
    t.wait(1)
    t.flush("audit")
    t.observed["audit"] = [(n["id"], n.get("owner")) for n in S.query("kind:surface")]
    print("AUDIT", t.observed["audit"])
    t.finish("07b-menu-give.mp4")
    json.dump({"rows": t.rows, "observed": t.observed}, open(f"{OUT}/rows-ch7b.json", "w"), indent=1)
finally:
    L.stop()
