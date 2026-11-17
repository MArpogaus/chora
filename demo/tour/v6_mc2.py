import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour6 import Quick, OUT
from tour import Session
from slides import slide

L = Launcher("mc4n")
env = L.start([f"{BIN}/chora-mc", "--manage-windows"], flavor="mc",
  extra_config={"chora-mc.yaml": "theme: dark\ntest_accept_injected_keys: true\n"},
  companions=[[f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(5)
def ps(scene, panel, lines):
    S.call("mc", "panel.set", {"scene": scene, "panel": panel, "lines": lines})
ps("agenda", "today", ["18:30\tDinner with the family", "19:30\tNews on the television", "20:00\tCall Anna", "21:00\tTablets", "22:00\tGoodnight"])
ps("morning", "lead", ["Good evening", "Anna will call at eight. The tablets are at nine."])
ps("morning", "weather", ["Tomorrow", "Zurich, 11 degrees, dry."])
ps("morning", "day", ["Tomorrow", "09:00\tNurse visit", "11:00\tWalk"])
ps("morning", "mail", ["Post", "One card from Anna."])
ps("morning", "overnight", ["Tonight", "Nothing to do."])
ps("morning", "reading", ["To read", "The card from Anna"])
t = Quick(S, OUT, f"{L.dir}/work")
t.begin("3b mc: the sofa, evening, dark", "03b-mc-dark")
O = "HEADLESS-1"
try:
    def door(sel):
        t.flush("door")
        for page in ("r1", "r0"):
            c = S.centre("id:" + sel.replace("mc-r1-", f"mc-{page}-"), O)
            if c is not None:
                return c
        raise RuntimeError("no node " + sel)
    t.jump(640, 360)
    t.act("look", "Evening on the sofa, the remote in hand: today's plan, large print, dark", "The agenda in the dark theme; the column with keys", still="03-10-evening-dark", lead=0.1, settle=1.2)
    t.done()
    t.act("key", "Remote: Right turns the page twice", "The page of the morning briefing", still="03-11-tomorrow-dark", settle=0.8, lead=0.1)
    t.key("Right"); t.wait(0.9); t.key("Right"); t.wait(0.9); t.done()
    t.act("key", "Remote: Home brings the plan back; Right once", "The plan, then the next page", still=None, settle=0.4, lead=0.1)
    t.key("Super+Home"); t.wait(0.9); t.done()
    x, y = door("mc-r1-right-doors-programs")
    t.act("pointer", "Mouse path: clicks Programs in the column", "The launcher opens over the page", still="03-12-launcher-dark", settle=0.8, lead=0.1)
    t.reach(x, y); t.click(); t.wait(0.6); t.done()
    t.act("key", "Escape closes the launcher", "The launcher closes", still=None, settle=0.2, lead=0.1)
    t.tap("Escape", 0.2); t.done()
    t.flush("chat")
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Reminder", "step": 1, "steps": 1, "seq": 4})
    x, y = door("mc-r1-right-chat-door")
    t.act("pointer", "Mouse path: opens the chat in the column", "The chat opens with the helper list", still="03-13-chat-dark", settle=0.8, lead=0.1)
    t.reach(x, y); t.click(); t.wait(0.6); t.done()
    t.act("key", "Escape closes the chat", "The chat closes", still=None, settle=0.2, lead=0.1)
    t.tap("Escape", 0.2); t.done()
    slide(f"{L.dir}/slide.png", 3, "mc: the sofa, dark", ["Evening, dark theme: the plan in large print,", "pages turned by the remote, the mouse as a second path."], accent=(247, 118, 142))
    t.finish("03b-mc-dark.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch3b-dark.json", "w"), indent=1)
finally:
    L.stop()
