import sys, time, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
L = Launcher("xp")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
os.makedirs(f"{L.dir}/config/chora", exist_ok=True)
yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("programs: []", programs_yaml(stub)).replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
open(f"{L.dir}/xp.yaml", "w").write(yaml)
env = L.start([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml"], xp=True,
              companions=[[f"{BIN}/chora-notify", "--assets", f"{REPO}/flavors/xp/assets/notify"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("1 xp", "01-xp")
try:
    def aim(sel, dx=0, dy=0):
        t.flush("aim")
        c = S.centre(sel, "HEADLESS-1")
        if c is None:
            raise RuntimeError("no node " + sel)
        return c[0] + dx, c[1] + dy

    def items():
        t.flush("items")
        out = {}
        for n in S.query("*"):
            if n["id"].startswith("xp-item-") and n.get("rects"):
                out[n["id"]] = n["rects"]["HEADLESS-1"]
        return out

    t.jump(640, 300)
    t.wait(2)
    t.act("pointer", "Look at the empty desktop", "Bliss desktop, taskbar with Start, 4 desktop buttons, volume 40%, clock", still="01-01-desktop")
    t.done()
    sx, sy = aim("id:xp-start-HEADLESS-1")
    t.act("pointer", "Move to Start and click", "The Start menu opens with Terminal, four programs, All Programs", still="01-02-start-menu")
    t.move(sx, sy); t.click(); t.done()
    it = items()
    x, y, w, h = it["xp-item-1"]
    t.act("pointer", "Move over Notes in the menu", "Row Notes is highlighted", still="01-03-hover-notes")
    t.move(x + w/2, y + h/2); t.done()
    t.act("pointer", "Click Notes", "Menu closes, a Notes window opens with a task button", settle=3.0, still="01-04-notes-open")
    t.click(); t.done()
    t.wait(2)
    sx, sy = aim("id:xp-start-HEADLESS-1")
    t.act("pointer", "Start again, move to wobbly.rs", "Menu opens, wobbly.rs row highlighted", still="01-05-menu-again")
    t.move(sx, sy); t.click(); t.wait(0.8)
    t.flush("menu")
    it = items(); x, y, w, h = it["xp-item-2"]
    t.move(x + w/2, y + h/2); t.done()
    t.act("pointer", "Click wobbly.rs", "A second window opens, two task buttons", settle=3.0, still="01-06-two-windows")
    t.click(); t.done()
    t.wait(2)
    t.flush("windows")
    bx, by = aim("id:xp-task-s1-HEADLESS-1")
    t.act("pointer", "Click the Notes button on the taskbar", "Notes comes to the front, its task button looks pressed", still="01-07-taskbar-notes")
    t.move(bx, by); t.click(); t.done()
    vx, vy = 1155, 705
    t.act("pointer", "Click the speaker in the tray", "Volume mutes: the tray says Muted", still="01-08-muted")
    t.move(vx, vy); t.click(); t.done()
    t.act("pointer", "Click the speaker again", "Volume comes back to 40%", still="01-09-unmuted")
    t.click(); t.done()
    t.flush("before voice")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    t.wait(1)
    t.act("voice event", "The fake voice agent says: Good morning! Your notes for Friday are ready.", "The paperclip appears with a balloon of that sentence", settle=2.5, still="01-10-paperclip-balloon", lead=0.5)
    t.flush("emit"); L.emit("voice.say", {"text": "Good morning! Your notes for Friday are ready.", "duration_ms": 9000}); t.done()
    t.flush("emit2"); L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.wait(5)
    px, py = aim("id:hud-figure", 0, 10)
    t.act("pointer", "Click the paperclip", "The balloon becomes the chat: Chat with a helper, Close, 1 Atlas working, a field", settle=2.0, still="01-11-chat-open")
    t.move(px, py); t.click(); t.done()
    t.flush("chat")
    fx, fy = aim("id:hud-chat-close", 150, 105)
    t.act("pointer", "Click Close", "The chat closes, only the paperclip stays (collapsed)", settle=1.5, still="01-12-chat-collapsed")
    cx, cy = aim("id:hud-chat-close")
    t.move(cx, cy); t.click(); t.done()
    px, py = aim("id:hud-figure", 0, 10)
    t.act("pointer", "Click the paperclip again", "The chat opens again", settle=1.5, still="01-13-chat-reopened")
    t.move(px, py); t.click(); t.done()
    pk, pky = aim("id:hud-chat-pick-1")
    t.act("pointer", "Click the row 1 Atlas", "Atlas is picked; the transcript of Atlas shows", settle=2.0, still="01-14-helper-picked")
    t.move(pk, pky); t.click(); t.done()
    t.flush("picked")
    r = S.rect("id:hud-chat", "HEADLESS-1")
    fx, fy = r[0] + r[2] / 2, r[1] + r[3] - 52
    t.act("pointer", "Click into the message field", "The field takes the keyboard (caret)", still="01-15-field-click")
    t.move(fx, fy); t.click(); t.done()
    msg = "Please also add Monday."
    t.act("keys", f"Type '{msg}' at 6 characters a second", "The text shows in the field", settle=0.6, still="01-16-typed")
    t.text(msg); t.done()
    t.act("key", "Press Return", "The message is sent: the field empties", settle=1.5, still="01-17-sent")
    t.key("Return"); t.done()
    t.flush("question")
    L.emit("voice.state", {"name": "Atlas", "state": "question", "task": "Tidy notes", "seq": 5})
    L.emit("voice.confirm", {"id": "51", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/Notes/friday/notes.txt (412 bytes)", "physical": True, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": True})
    t.wait(2)
    t.act("voice event", "The helper asks: Atlas wants to write the file notes.txt", "A question card with Yes and No shows in the chat", settle=1.5, still="01-18-question")
    t.done()
    yx, yy = aim("id:hud-card-yes")
    t.act("pointer", "Click Yes on the card", "The card goes; the helper goes on working", settle=2.0, still="01-19-answered")
    t.move(yx, yy); t.click(); t.done()
    t.flush("answered")
    L.emit("voice.confirm", {"id": "51", "done": True})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 2, "steps": 2, "seq": 6})
    t.act("voice event", "The fake agent takes the answer and closes the question", "The card goes, the chat is back with the field", settle=1.5, still="01-19b-card-closed")
    t.done()
    t.act("key", "Press F1 (key overlay)", "A list of keys shows over the desktop", settle=1.5, still="01-20-f1")
    t.key("F1"); t.done()
    t.act("key", "Press F1 again", "The overlay (if any) goes away", settle=1.0, still="01-21-f1-again")
    t.key("F1"); t.done()
    t.wait(1)
    t.finish("01-xp.mp4")
    import json; json.dump(t.rows, open(f'{OUT}/rows-ch1.json','w'), indent=1)
finally:
    L.stop()
