import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from human import Human
from tour import Session
from slides import slide

OUT = os.environ.get("TOUR_OUT", "/home/user/log/media/2026-11-16/tour-v3")
THEME = os.environ.get("MC_THEME", "light")
L = Launcher("mc")
env = L.start([f"{BIN}/chora-mc", "--manage-windows"], flavor="mc",
  extra_config={"chora-mc.yaml": f"theme: {THEME}\ntest_accept_injected_keys: true\n"},
  companions=[[f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(5)
def ps(scene, panel, lines, title=None):
    p = {"scene": scene, "panel": panel, "lines": lines}
    if title: p["title"] = title
    S.call("mc", "panel.set", p)
ps("agenda", "today", ["08:30\tStand-up with the platform team", "10:00\tReview the compositor merge request", "11:30\tCoffee with Anna", "13:00\tLunch", "14:00\tWrite the release notes for 0.9", "15:30\tDesign review: the new cube", "17:00\tTidy the notes for Friday"])
ps("morning", "lead", ["Release notes are due before lunch", "The merge request for the cube needs one more review. Anna has asked for the release notes by noon, and the design review follows at half past three."])
ps("morning", "weather", ["Weather", "Zurich, 9 degrees, light rain until noon, clear in the evening."])
ps("morning", "day", ["The day", "08:30\tStand-up", "10:00\tReview", "15:30\tDesign review"])
ps("morning", "mail", ["Mail", "Three unread: Anna, the build bot, the newsletter."])
ps("morning", "overnight", ["Overnight", "Atlas tidied the notes for Friday and found three items."])
ps("morning", "reading", ["To read", "A short history of the window manager", "Notes on stepped clocks"])
for name, state, task in (("Atlas", "working", "Tidy notes"), ("Vega", "thinking", "Draft the release notes"), ("Orion", "idle", "")):
    S.call("mc", "agent.set", {"name": name, "state": state, "task": task, "said": "", "plan": ["Read the notes", "Group by day", "Write the summary"] if name == "Atlas" else []})
t = Human(S, OUT, f"{L.dir}/work")
t.begin(f"3 mc ({THEME})", f"03-mc-{THEME}")
O = "HEADLESS-1"
short = THEME == "dark"
try:
    def door(sel):
        t.flush("door")
        for page in ("r1", "r0"):
            c = S.centre("id:" + sel.replace("mc-r1-", f"mc-{page}-"), O)
            if c is not None:
                return c
        raise RuntimeError("no node " + sel)
    t.jump(640, 360)
    t.wait(1.5)
    t.act("look", "The person reads the first scene", "Scene 1 AGENDA: times in the margin, the right column with Notifications, Chat, the keys and the doors", still=f"03-01-start-{THEME}", lead=0.2, settle=3.0)
    t.done()
    n = 2 if short else 4
    for i in range(n):
        x, y = door("mc-r1-footer-next")
        t.act("pointer", "Click Next in the footer", f"The scene fades to number {i + 2}", settle=1.8, still=f"03-{i + 2:02d}-next-scene-{i + 2}-{THEME}")
        t.press(x, y); t.wait(0.8); t.done()
    t.act("key", "Slip: press Super+Left twice, one scene too far back", "The scenes go back; the person goes forward again with the footer", settle=1.2, still=f"03-07-back-{THEME}")
    t.key("Super+Left"); t.wait(1.2); t.key("Super+Left"); t.done()
    if not short:
        t.act("key", "Press Super+o (overview)", "The overview lists the scenes", settle=1.6, still=f"03-08-overview-{THEME}")
        t.key("Super+o"); t.wait(1.5); t.done()
    t.act("key", "Press Super+Home (agenda)", "The agenda again", settle=1.6, still=f"03-09-agenda-{THEME}")
    t.key("Super+Home"); t.done()
    x, y = door("mc-r1-right-doors-programs")
    t.act("pointer", "Click Programs in the column, the visible door", "The launcher opens", settle=1.8, still=f"03-10-launcher-{THEME}")
    t.press(x, y); t.done()
    t.act("key", "Escape closes the launcher; the person opens the terminal scene with Super+Return", "Scene Terminal with a shell", settle=2.5, still=f"03-11-terminal-{THEME}")
    t.wait(1.0); t.tap("Escape", 0.4); t.key("Super+Return"); t.done()
    t.act("keys", "Type ls and cat friday.md with a slip, then open the file in nano", "The shell prints the Friday list; nano shows the file", settle=1.0, still=f"03-12-nano-{THEME}")
    t.type_human("ls\n", typo_at=1, typo_char="k"); t.wait(0.6); t.type_human("cat friday.md\n", think=(0,)); t.wait(0.8); t.type_human("nano friday.md\n", think=(0,)); t.wait(1.2); t.done()
    t.key("ctrl+x"); t.wait(1.0)
    t.act("key", "Press Super+Home", "Back to the agenda", settle=1.6, still=f"03-13-agenda-back-{THEME}")
    t.key("Super+Home"); t.done()
    if not short:
        t.act("key", "Press Super+n (hide the column)", "The right column hides", settle=1.6, still=f"03-14-right-{THEME}")
        t.key("Super+n"); t.wait(1.0); t.done()
        t.act("key", "Press Super+n again", "The column shows again", settle=1.4, still=f"03-14b-right-back-{THEME}")
        t.key("Super+n"); t.done()
    t.flush("voice")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.wait(2.5)
    t.flush("voice2")
    x, y = door("mc-r1-right-chat-door")
    t.act("pointer", "Click the chat door in the column", "The chat opens in the right column with the helper list", settle=2.0, still=f"03-15-chat-drawer-{THEME}")
    t.press(x, y); t.done()
    t.act("key", "Press Escape", "The chat closes", settle=1.4, still=f"03-17-chat-closed-{THEME}")
    t.key("Escape"); t.done()
    t.wait(0.5)
    slide(f"{L.dir}/slide.png", 3, f"mc ({THEME})", ["mc v2: a printed page. Scenes with a head and", "the book serif, a column with notifications, chat", "and the keys, and a terminal scene."], accent=(247, 118, 142))
    t.finish(f"03-mc-{THEME}.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch3-{THEME}.json", "w"), indent=1)
finally:
    L.stop()
