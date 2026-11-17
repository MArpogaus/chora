import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import launch
from launch import *
from tour6 import Quick, OUT
from tour import Session
from slides import slide

launch.ENTRIES[:] = []
L = Launcher("xp4")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("editor:\n", f"editor: {stub}/notepad\n", 1).replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
open(f"{L.dir}/xp.yaml", "w").write(yaml)
open("/home/user/notes/letter.txt", "w").write("")
open(f"{stub}/notepad", "w").write("#!/bin/sh\nexec foot --app-id=Notepad --title=Notepad -o pad=14x12 -o colors.background=ffffff -o colors.foreground=000000 -o 'font=DejaVu Sans Mono:size=11' -o cursor.blink=no -e nano -R /home/user/notes/letter.txt\n")
os.chmod(f"{stub}/notepad", 0o755)
env = L.start([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml"], flavor="xp", xp=True,
              companions=[[f"{BIN}/chora-notify", "--assets", f"{REPO}/flavors/xp/assets/notify"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(6)
t = Quick(S, OUT, f"{L.dir}/work")
t.begin("1 xp: Erika writes a letter", "01-xp")
O = "HEADLESS-1"
try:
    def aim(sel):
        t.flush("aim")
        c = S.centre(sel, O)
        if c is None:
            raise RuntimeError("no node " + sel)
        return c

    def item(text):
        t.flush("item")
        for n in range(0, 60):
            q = S.query(f"id:xp-item-{n}")
            if q and (q[0].get("meta") or {}).get("text") == text:
                r = (q[0].get("rects") or {}).get(O)
                if r:
                    return r[0] + r[2] / 2, r[1] + r[3] / 2
        raise RuntimeError("no menu item " + text)

    t.jump(660, 330)
    t.act("look", "Erika sits down at the xp desktop", "Desktop picture, taskbar with Start, desktop buttons, the paperclip and the volume in the tray", still="01-01-desktop", lead=0.1, settle=1.0)
    t.done()
    sx, sy = aim("id:xp-start-HEADLESS-1")
    t.act("pointer", "Start, then Notepad", "The Start menu opens; the editor opens on an empty letter.txt in a window", still="01-02-start-menu", settle=0.5)
    t.reach(sx, sy); t.click(); t.wait(0.5)
    t.flush("menu")
    x, y = item("Notepad")
    t.reach(x, y); t.click(); t.cut(2.5); t.done()
    t.act("keys", "Types a short letter, one typo corrected, saves with Ctrl+O and Return", "The three lines of the letter show in the editor window; the file is written", still="01-03-letter", settle=0.8)
    t.type_quick("Dear Anna,\nthank you for the lovely visit.\nThe garden looks wonderful.\nLove, Erika", typo_at=14, typo_char="r")
    t.wait(0.3); t.key("ctrl+o"); t.wait(0.5); t.tap("Return", 0.5); t.done()
    t.flush("letter")
    tx, ty = aim("id:xp-title-s1")
    t.act("pointer", "Drags the letter window aside by its title bar", "The window follows the pointer and bends like jelly, then settles", still=None, settle=0.2)
    t.reach(tx, ty)
    t.drag(tx, ty, tx + 260, ty + 70, seconds=1.0, hold=0.1, mid_still="01-04-wobble")
    t.wait(0.5); t.done()
    px, py = aim("id:xp-tray-clip") if S.query("id:xp-tray-clip") else (1172, 705)
    t.act("pointer", "Clicks the paperclip in the tray: she wants to know how to make the sound louder", "The helper's chat opens at the bottom right with an empty transcript", still="01-05-chat-open", settle=0.6)
    t.reach(px, py); t.click(); t.wait(0.5); t.done()
    t.flush("chat")
    fx, fy = 1070, 672
    t.act("keys", "Clicks the field and types her question, sends it with Return", "Her question shows in the transcript", still="01-06-question", settle=0.5)
    t.reach(fx, fy); t.click(); t.wait(0.4)
    t.type_quick("How do I make the sound louder?", typo_at=9, typo_char="x")
    t.tap("Return", 0.3); t.done()
    t.flush("asked")
    json.dump([{"seq": 1, "role": "user", "text": "How do I make the sound louder?"},
               {"seq": 2, "role": "agent", "text": "Open Start, then Control Panel, then Sound, and choose 75%."}], open(f"{L.dir}/entries.json", "w"))
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Answering", "step": 1, "steps": 1, "seq": 2})
    t.wait(0.4)
    t.act("look", "She reads the answer", "The helper answers: Start, Control Panel, Sound, 75%", still="01-07-answer", settle=1.2, lead=0.1)
    t.done()
    cx, cy = aim("id:hud-chat-close")
    t.act("pointer", "Closes the chat with its X button", "The chat closes; the paperclip stays", still=None, settle=0.2)
    t.reach(cx, cy); t.click(); t.wait(0.3); t.done()
    sx, sy = aim("id:xp-start-HEADLESS-1")
    t.act("pointer", "Start, Control Panel, Sound, 75%", "The cascade opens step by step; the tray reads 75% afterwards", still="01-08-sound-menu", settle=0.4)
    t.reach(sx, sy); t.click(); t.wait(0.4)
    t.flush("start2")
    print("MENU after Start:", bool(S.query("id:xp-menu-frame")), flush=True)
    if not S.query("id:xp-menu-frame"):
        t.click(); t.wait(0.4)
    t.reach(260, 542); t.wait(0.8); t.click(); t.wait(0.8)
    t.reach(430, 548, overshoot=False); t.reach(430, 650, overshoot=False); t.wait(0.9)
    t.reach(600, 650, overshoot=False); t.reach(600, 628, overshoot=False); t.wait(0.4); t.still("01-09-sound-levels"); t.click(); t.wait(0.4); t.done()
    t.act("look", "The tray shows 75%", "Volume 75% in the tray", still="01-10-volume-75", settle=0.9, lead=0.1)
    t.done()
    t.act("pointer", "Opens the paperclip again and asks the helper to keep the letter safe", "The chat opens with the first answer; her second request is sent", still="01-11-second-ask", settle=0.4)
    t.reach(1172, 705); t.click(); t.wait(0.5)
    t.reach(1070, 672); t.click(); t.wait(0.4)
    t.type_quick("Please copy my letter to My Documents.", typo_at=12, typo_char="q")
    t.tap("Return", 0.3); t.done()
    t.flush("ask2")
    L.emit("voice.state", {"name": "Atlas", "state": "question", "task": "Copy the letter", "seq": 3})
    L.emit("voice.confirm", {"id": "61", "prompt": "Atlas wants to copy letter.txt to My Documents.", "detail": "~/notes/letter.txt (92 bytes)", "physical": True, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": True})
    t.wait(0.6)
    t.act("look", "The helper asks before it touches a file", "A question card with Yes and No", still="01-12-question", settle=0.8, lead=0.1)
    t.done()
    yx, yy = aim("id:hud-card-yes")
    t.act("pointer", "Erika clicks Yes", "The card goes; the helper reports it is done", still="01-13-done", settle=0.6)
    t.reach(yx, yy); t.click(); t.done()
    t.flush("yes")
    L.emit("voice.confirm", {"id": "61", "done": True})
    json.dump([{"seq": 1, "role": "user", "text": "How do I make the sound louder?"},
               {"seq": 2, "role": "agent", "text": "Open Start, then Control Panel, then Sound, and choose 75%."},
               {"seq": 3, "role": "user", "text": "Please copy my letter to My Documents."},
               {"seq": 4, "role": "agent", "text": "Done. The letter is in My Documents."}], open(f"{L.dir}/entries.json", "w"))
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Done", "step": 1, "steps": 1, "seq": 4})
    t.wait(1.2)
    t.act("look", "She reads that it is done", "The helper says the letter is in My Documents", still="01-14-reply", settle=1.0, lead=0.1)
    t.done()
    slide(f"{L.dir}/slide.png", 1, "xp: a letter for Anna", ["Erika writes a letter, asks the paperclip how to make", "the sound louder, and finds the answer in the Start menu."])
    t.finish("01-xp.mp4", slide=f"{L.dir}/slide.png")
    json.dump(t.rows, open(f"{OUT}/rows-ch1.json", "w"), indent=1)
finally:
    L.stop()
