import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
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
t.begin("3 mc", f"03-mc-{THEME}")
try:
    t.jump(640, 360)
    t.wait(2)
    t.act("look", "Look at mc at start", "The first scene fills the screen, header with the running head", still=f"03-01-start-{THEME}")
    t.done()
    names = ["agenda", "morning", "agents", "mission", "overview"]
    for i in range(6):
        t.act("key", "Press Super+Right (next scene)", f"The scene fades to number {i + 2}", settle=2.0, still=f"03-{i + 2:02d}-next-scene-{i + 2}-{THEME}")
        t.key("Super+Right"); t.done()
    t.act("key", "Press Super+o (overview)", "The overview lists the scenes as N | Title", settle=2.0, still=f"03-08-overview-{THEME}")
    t.key("Super+o"); t.done()
    t.act("key", "Press Super+a (agenda)", "The agenda with times in the margin", settle=2.0, still=f"03-09-agenda-{THEME}")
    t.key("Super+a"); t.done()
    t.act("key", "Press Super+Return (drawer)", "A terminal drawer slides in", settle=2.5, still=f"03-10-drawer-{THEME}")
    t.key("Super+Return"); t.done()
    t.act("key", "Press Super+Return again", "The drawer closes", settle=2.0, still=f"03-11-drawer-closed-{THEME}")
    t.key("Super+Return"); t.done()
    t.act("key", "Press Super+n (right side)", "The right side with notes shows", settle=2.0, still=f"03-12-right-{THEME}")
    t.key("Super+n"); t.done()
    t.act("key", "Press Super+n again", "The right side hides", settle=1.5, still=f"03-13-right-hidden-{THEME}")
    t.key("Super+n"); t.done()
    t.flush("voice")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.wait(3)
    print([(n["id"], n.get("rects")) for n in S.query("*") if n["id"].startswith("hud") and n.get("rects")])
    fig = S.centre("id:hud-row-chat", "HEADLESS-1")
    print("fig", fig)
    t.act("pointer", "Click the HUD figure to open the chat", "The chat opens in a drawer", settle=2.5, still=f"03-14-chat-drawer-{THEME}")
    t.move(*fig); t.click(); t.done()
    t.act("key", "Press Escape", "The chat drawer closes", settle=2.0, still=f"03-15-chat-closed-{THEME}")
    t.key("Escape"); t.done()
    t.wait(1)
    t.finish(f"03-mc-{THEME}.mp4")
    json.dump(t.rows, open(f"{OUT}/rows-ch3-{THEME}.json", "w"), indent=1)
finally:
    L.stop()
