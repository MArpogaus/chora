import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour6 import Quick, OUT
from tour import Session
from slides import slide

THEME = os.environ.get("MC_THEME", "light")
SCENES = {
    "ingredients": """chapter: {title: Spaghetti}
panels:
- id: lead
  cell: [0, 0, 2, 2]
  style: {base: lead}
  empty: No recipe yet.
- {id: list, cell: [2, 0, 2, 2], rule: faint, empty: No list yet.}
- {id: tip, cell: [0, 2, 4, 1], empty: No tip yet.}
""",
    "boil": """chapter: {title: "Step 1: the water"}
panels:
- id: lead
  cell: [0, 0, 3, 2]
  style: {base: lead}
  empty: Nothing yet.
- {id: side, cell: [3, 0, 1, 2], rule: faint, empty: Nothing yet.}
""",
    "sauce": """chapter: {title: "Step 2: the sauce"}
panels:
- id: lead
  cell: [0, 0, 3, 2]
  style: {base: lead}
  empty: Nothing yet.
- {id: side, cell: [3, 0, 1, 2], rule: faint, empty: Nothing yet.}
""",
}
L = Launcher("mc4")
d = f"{L.dir}/config/chora/mc/scenes"
os.makedirs(d, exist_ok=True)
for name, body in SCENES.items():
    open(f"{d}/{name}.yaml", "w").write(body)
env = L.start([f"{BIN}/chora-mc", "--manage-windows"], flavor="mc",
  extra_config={"chora-mc.yaml": f"theme: {THEME}\ntest_accept_injected_keys: true\n"},
  companions=[[f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(5)
def ps(scene, panel, lines):
    S.call("mc", "panel.set", {"scene": scene, "panel": panel, "lines": lines})
ps("ingredients", "lead", ["Spaghetti with garlic and oil", "Serves two. Twenty minutes from the first pot to the plate."])
ps("ingredients", "list", ["For two", "200 g spaghetti", "4 cloves of garlic", "6 tbsp olive oil"])
ps("ingredients", "tip", ["Tip", "Keep a cup of the pasta water for the sauce."])
ps("boil", "lead", ["Boil the water", "Fill the large pot with two litres of water, add a spoon of salt and bring it to a rolling boil. Then add the spaghetti and cook it for nine minutes."])
ps("boil", "side", ["Timer", "9 minutes once the spaghetti is in."])
ps("sauce", "lead", ["Warm the oil", "Take the oil on low heat. Add the sliced garlic and the chili for two minutes, until golden. Do not let the garlic turn brown."])
ps("sauce", "side", ["Then", "Toss the drained spaghetti in the pan with a splash of the pasta water and the parsley."])
print("SCENES", S.call("mc", "scene.list"), flush=True)
for n, name in enumerate(("ingredients", "boil", "sauce"), 1):
    S.call("mc", "scene.move", {"name": name, "to": n})
print("ORDER", S.call("mc", "scene.list"), flush=True)
S.call("mc", "scene.show", {"name": "ingredients"})
t = Quick(S, OUT, f"{L.dir}/work")
t.begin(f"3 mc: a hands-busy kitchen ({THEME})", f"03-mc-{THEME}")
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
    t.act("look", "The cook has the recipe on the page, hands in the dough", "The recipe page: the lead, the list, a tip; the column with keys at the right", still=f"03-01-recipe-{THEME}", lead=0.1, settle=1.2)
    t.done()
    t.flush("say")
    L.emit("voice.state", {"state": "listening", "task": "", "said": "next step"})
    S.call("mc", "hud.set", {"state": "listening", "text": "next step"})
    t.act("voice event", "By voice: 'next step' (the voice agent is a stand-in)", "The HUD line shows what was heard", still=f"03-02-heard-{THEME}", settle=0.8, lead=0.1)
    t.wait(0.5); t.done()
    S.call("mc", "scene.show", {"name": "boil"})
    t.act("look", "The helper turns the page to the water", "The page changes to Step 1", still=f"03-03-boil-{THEME}", settle=1.2, lead=0.1)
    t.wait(0.6); t.done()
    t.flush("timer")
    S.call("mc", "notify.push", {"title": "Timer: spaghetti, 9 minutes", "body": "Started by voice"})
    t.act("look", "A timer card comes up in the column (a notification of the helper)", "A note with the time and the words in the notifications of the column", still=f"03-04-timer-{THEME}", settle=1.2, lead=0.1)
    t.wait(0.6); t.done()
    t.flush("ask")
    L.emit("voice.state", {"state": "listening", "task": "", "said": "how much garlic"})
    S.call("mc", "hud.set", {"state": "listening", "text": "how much garlic"})
    t.act("voice event", "By voice again: 'how much garlic'", "The HUD line shows the question", still=f"03-04b-asked-{THEME}", settle=0.5, lead=0.1)
    t.wait(0.6); t.done()
    t.flush("answer")
    S.call("mc", "hud.say", {"state": "speaking", "text": "Four cloves, sliced thin.", "duration_ms": 3000})
    t.act("voice event", "The helper answers on the page (and aloud)", "The HUD line shows the answer: four cloves, sliced thin", still=f"03-04c-answer-{THEME}", settle=1.2, lead=0.1)
    t.done()
    x, y = door("mc-r1-footer-next")
    t.act("pointer", "Mouse path: clicks Next in the footer to the sauce", "The page turns to Step 2: the sauce", still=f"03-05-sauce-{THEME}", settle=0.8, lead=0.1)
    t.reach(x, y); t.click(); t.wait(0.9); t.done()
    t.act("key", "Remote: Left twice goes back to the ingredients, Right once forward", "The page of the ingredients, then Step 1", still=f"03-06-back-{THEME}", settle=0.8, lead=0.1)
    t.key("Left"); t.wait(0.7); t.key("Left"); t.wait(0.9); t.key("Right"); t.wait(0.9); t.done()
    t.act("key", "Super+n hides the column: the page is larger for the sofa", "The column goes; the recipe page widens", still=f"03-07-wide-{THEME}", settle=0.8, lead=0.1)
    t.key("Super+n"); t.wait(1.0); t.done()
    t.act("key", "Super+n brings the column back", "The column shows again", still=None, settle=0.4, lead=0.1)
    t.key("Super+n"); t.wait(0.8); t.done()
    t.flush("alarm")
    S.call("mc", "notify.push", {"title": "Spaghetti is ready", "body": "Drain it now", "urgency": "critical"})
    t.act("look", "The timer is up: a critical note", "A red note: Spaghetti is ready", still=f"03-08-alarm-{THEME}", settle=1.5, lead=0.1)
    t.done()
    slide(f"{L.dir}/slide.png", 3, f"mc: a kitchen ({THEME})", ["Hands in the dough: the recipe by voice and by remote,", "a timer card, the page turned from the sofa."], accent=(247, 118, 142))
    t.finish(f"03-mc-{THEME}.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch3-{THEME}.json", "w"), indent=1)
finally:
    L.stop()
