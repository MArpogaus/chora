import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *
from slides import slide

OUT = "/home/user/log/media/2026-11-12/tour-v2"
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
t = Tour(S, OUT, f"{L.dir}/work")
t.begin(f"3 mc ({THEME})", f"03-mc-{THEME}")
O = "HEADLESS-1"
short = THEME == "dark"
try:
    t.jump(640, 360)
    t.wait(1.2)
    t.act("look", "mc at start", "Scene 1 AGENDA: times in the margin, header with the helpers", still=f"03-01-start-{THEME}")
    t.done()
    n = 3 if short else 6
    for i in range(n):
        t.act("key", "Press Super+Right (next scene)", f"The scene fades to number {i + 2}", settle=1.4, still=f"03-{i + 2:02d}-next-scene-{i + 2}-{THEME}")
        t.key("Super+Right"); t.done()
    if not short:
        t.act("key", "Press Super+o (overview)", "The overview lists the scenes", settle=1.4, still=f"03-08-overview-{THEME}")
        t.key("Super+o"); t.done()
    t.act("key", "Press Super+a (agenda)", "The agenda again", settle=1.4, still=f"03-09-agenda-{THEME}")
    t.key("Super+a"); t.done()
    t.act("key", "Press Super+Return (drawer)", "A terminal drawer slides in", settle=2.0, still=f"03-10-drawer-{THEME}")
    t.key("Super+Return"); t.done()
    t.act("keys", "Type cat friday.md in the drawer", "The shell prints the Friday list in the drawer", settle=0.8, still=f"03-11-drawer-cat-{THEME}")
    t.text("cat friday.md\n"); t.done()
    t.act("key", "Press Super+Return again", "The drawer closes", settle=1.4, still=f"03-12-drawer-closed-{THEME}")
    t.key("Super+Return"); t.done()
    if not short:
        t.act("key", "Press Super+n (right side)", "The right side shows", settle=1.4, still=f"03-13-right-{THEME}")
        t.key("Super+n"); t.wait(0.3); t.done()
        t.key("Super+n"); t.wait(0.3)
    t.flush("voice")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.wait(2.5)
    t.flush("voice2")
    fig = S.centre("id:hud-row-chat", O)
    t.act("pointer", "Click the Chat ... pill", "The chat drawer opens with the helper list", settle=2.0, still=f"03-14-chat-drawer-{THEME}")
    t.move(*fig); t.click(); t.done()
    t.act("keys", "Type 1 and press Return (pick Atlas)", "Atlas is picked in the drawer", settle=1.4, still=f"03-15-picked-{THEME}")
    t.key("1"); t.wait(0.3); t.key("Return"); t.done()
    t.act("key", "Press Escape", "The drawer closes", settle=1.4, still=f"03-16-chat-closed-{THEME}")
    t.key("Escape"); t.done()
    t.wait(0.5)
    slide(f"{L.dir}/slide.png", 3, f"mc ({THEME})", ["A page, not windows: scenes with headings and", "text in the margin. Agenda, the morning, agents,", "a terminal drawer and the chat drawer."], accent=(247, 118, 142))
    t.finish(f"03-mc-{THEME}.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch3-{THEME}.json", "w"), indent=1)
finally:
    L.stop()
