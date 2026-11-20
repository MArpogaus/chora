"""Chapter mc, light (the cook) and dark (the sofa): `run(ctx)` runs both; the rows are `t.act(..., expect=, contract=, cells=, capture=)`."""

import sys, time, os, json, subprocess
from launch import *
import launch
from human import Quick
from tour import Session, checker_for
from harness import Expect
from slides import slide

def light(ctx):
    OUT = ctx.out
    THEME = "light"
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
      xp=True,
      companions=[[f"{BIN}/chora-notify"], [f"{BIN}/chora-assistant"]])
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
    t = Quick(S, OUT, f"{L.dir}/work", checker=checker_for(S, ["mc"], L.dir))
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
        t.act("voice event", "By voice: 'next step' (the voice agent is a stand-in)", "The HUD line shows what was heard", still=f"03-02-heard-{THEME}", settle=0.8, lead=0.1, contract="M1", cells=["voice events"], capture="Q")
        t.wait(0.5); t.done()
        S.call("mc", "scene.show", {"name": "boil"})
        t.act("look", "The helper turns the page to the water", "The page changes to Step 1", still=f"03-03-boil-{THEME}", settle=1.2, lead=0.1)
        t.wait(0.6); t.done()
        t.flush("timer")
        S.call("mc", "notify.push", {"title": "Timer: spaghetti, 9 minutes", "body": "Started by voice"})
        t.act("look", "A timer card comes up in the column (a notification of the helper)", "A note with the time and the words in the notifications of the column", still=f"03-04-timer-{THEME}", settle=1.2, lead=0.1, cells=["act: notify.push of mc shows in chora-notify"], capture="Q", known="the Power glyph of the column lies under the card body (I5; v8 open row, notice over the column)")
        t.wait(0.6); t.done()
        t.flush("ask")
        L.emit("voice.state", {"state": "listening", "task": "", "said": "how much garlic"})
        S.call("mc", "hud.set", {"state": "listening", "text": "how much garlic"})
        t.act("voice event", "By voice again: 'how much garlic'", "The HUD line shows the question", still=f"03-04b-asked-{THEME}", settle=0.5, lead=0.1, known="the Power glyph of the column lies under the card body (I5; v8 open row)")
        t.wait(0.6); t.done()
        t.flush("answer")
        S.call("mc", "hud.say", {"state": "speaking", "text": "Four cloves, sliced thin.", "duration_ms": 3000})
        t.act("voice event", "The helper answers on the page (and aloud)", "The HUD line shows the answer: four cloves, sliced thin", still=f"03-04c-answer-{THEME}", settle=1.2, lead=0.1, contract="T3", cells=["act: the helper answers"], capture="Q+1", known="the Power glyph of the column lies under the card body (I5; v8 open row)")
        t.done()
        x, y = door("mc-r1-footer-next")
        t.act("pointer", "Mouse path: clicks Next in the footer to the sauce", "The page turns to Step 2: the sauce", still=f"03-05-sauce-{THEME}", settle=0.8, lead=0.1)
        t.reach(x, y); t.click(); t.wait(0.9); t.done()
        t.act("key", "Remote: Left twice goes back to the ingredients, Right once forward", "The page of the ingredients, then Step 1", still=f"03-06-back-{THEME}", settle=0.8, lead=0.1, contract="M1", cells=["act: Left of a remote turns back"], capture="Q")
        t.key("Left"); t.wait(0.7); t.key("Left"); t.wait(0.9); t.key("Right"); t.wait(0.9); t.done()
        t.act("key", "Super+n hides the column: the page is larger for the sofa", "The column goes; the recipe page widens", still=f"03-07-wide-{THEME}", settle=0.8, lead=0.1, cells=["doors clock-door"], capture="Q")
        t.key("Super+n"); t.wait(1.0); t.done()
        t.act("key", "Super+n brings the column back", "The column shows again", still=None, settle=0.4, lead=0.1)
        t.key("Super+n"); t.wait(0.8); t.done()
        t.flush("alarm")
        S.call("mc", "notify.push", {"title": "Spaghetti is ready", "body": "Drain it now", "urgency": "critical"})
        t.act("look", "The timer is up: a critical note", "A red note: Spaghetti is ready", still=f"03-08-alarm-{THEME}", settle=1.5, lead=0.1, cells=["act: notify.push of mc shows in chora-notify"], capture="Q")
        t.done()
        slide(f"{L.dir}/slide.png", 3, f"mc: a kitchen ({THEME})", ["Hands in the dough: the recipe by voice and by remote,", "a timer card, the page turned from the sofa."], accent=(247, 118, 142))
        t.finish(f"03-mc-{THEME}.mp4", slide=f"{L.dir}/slide.png")
        json.dump(t.rows, open(f"{OUT}/rows-ch3-{THEME}.json", "w"), indent=1)
    finally:
        L.stop()



def dark(ctx):
    OUT = ctx.out
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
    t = Quick(S, OUT, f"{L.dir}/work", checker=checker_for(S, ["mc"], L.dir))
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
        t.act("key", "Remote: Right turns the page twice", "The page of the morning briefing", still="03-11-tomorrow-dark", settle=0.8, lead=0.1, contract="M1", cells=["act: Right of a remote turns the page in dark mc"], capture="Q")
        t.key("Right"); t.wait(0.9); t.key("Right"); t.wait(0.9); t.done()
        t.act("key", "Remote: Home brings the plan back; Right once", "The plan, then the next page", still=None, settle=0.4, lead=0.1)
        t.key("Super+Home"); t.wait(0.9); t.done()
        x, y = door("mc-r1-right-doors-programs")
        t.act("pointer", "Mouse path: clicks Programs in the column", "The launcher opens over the page", still="03-12-launcher-dark", settle=0.8, lead=0.1, contract="T1", cells=["doors launcher-door"], capture="Q")
        t.reach(x, y); t.click(); t.wait(0.6); t.done()
        t.act("key", "Escape closes the launcher", "The launcher closes", still=None, settle=0.2, lead=0.1)
        t.tap("Escape", 0.2); t.done()
        t.flush("chat")
        L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Reminder", "step": 1, "steps": 1, "seq": 4})
        x, y = door("mc-r1-right-chat-door")
        t.act("pointer", "Mouse path: opens the chat in the column", "The chat opens with the helper list", still="03-13-chat-dark", settle=0.8, lead=0.1, contract="T3", cells=["doors helper-door"], capture="Q")
        t.reach(x, y); t.click(); t.wait(0.6); t.done()
        t.act("key", "Escape closes the chat", "The chat closes", still=None, settle=0.2, lead=0.1, known="the headers of two pages are drawn at once after the chat closes (I5: mc-r0 and mc-r1 overlap)")
        t.tap("Escape", 0.2); t.done()
        slide(f"{L.dir}/slide.png", 3, "mc: the sofa, dark", ["Evening, dark theme: the plan in large print,", "pages turned by the remote, the mouse as a second path."], accent=(247, 118, 142))
        t.finish("03b-mc-dark.mp4", slide=f"{L.dir}/slide.png")
        json.dump(t.rows, open(f"{OUT}/rows-ch3b-dark.json", "w"), indent=1)
    finally:
        L.stop()



def run(ctx):
    light(ctx)
    dark(ctx)
