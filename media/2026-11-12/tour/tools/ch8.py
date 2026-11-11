import sys, time, os, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from launch import *
from tour import *

OUT = "/home/user/log/media/2026-11-12/tour"
L = Launcher("voice")
stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("programs: []", programs_yaml(stub)).replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
open(f"{L.dir}/xp.yaml", "w").write(yaml)
env = L.start([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml"], xp=True,
              companions=[[f"{BIN}/chora-notify", "--assets", f"{REPO}/flavors/xp/assets/notify"], [f"{BIN}/chora-assistant"]])
S = Session(env, L.pids, BIN)
time.sleep(6)
t = Tour(S, OUT, f"{L.dir}/work")
t.begin("8 voice and chat", "08-voice")
observed = {}
def lines(name):
    p = f"{L.dir}/{name}"
    return open(p).read() if os.path.exists(p) else ""
try:
    t.jump(640, 300)
    t.wait(2)
    t.act("look", "Look at the xp desktop", "Desktop with taskbar, no paperclip yet", still="08-01-desktop")
    t.done()
    t.flush("a")
    L.emit("voice.state", {"state": "listening", "task": "", "said": ""})
    t.wait(1.5)
    t.act("voice event", "The fake voice agent hears 'Open the chat' and says 'Opening the chat.'", "The paperclip says Opening the chat. in a balloon", settle=2.0, still="08-02-voice-said")
    t.flush("b"); L.emit("voice.say", {"text": "Opening the chat.", "duration_ms": 5000}); t.done()
    t.flush("c")
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 1, "steps": 2, "seq": 4})
    t.wait(3)
    t.act("voice action", "The voice agent opens the chat (assistant chat.show)", "The chat opens with the helper list: 1 Atlas working", settle=2.0, still="08-03-chat-by-voice")
    t.call("assistant", "chat.show", {}); t.done()
    t.flush("d")
    r = S.rect("id:hud-chat", "HEADLESS-1")
    fx, fy = r[0] + r[2] / 2, r[1] + r[3] - 52
    t.act("keys", "Type 1 and press Return WITHOUT clicking first", "The helper Atlas is picked (or the keys go to another window)", settle=2.0, still="08-04-pick-by-keys-no-click")
    t.key("1"); t.wait(0.4); t.key("Return"); t.done()
    t.flush("e")
    print("pick state", [(n["id"], n.get("meta")) for n in S.query("*") if n["id"].startswith("hud-chat") and n.get("rects")])
    pk = S.centre("id:hud-chat-pick-1", "HEADLESS-1")
    if pk:
        t.act("pointer", "Click the row 1 Atlas", "Atlas is picked: header with Stop and Close, transcript", settle=2.0, still="08-05-pick-by-click")
        t.move(*pk); t.click(); t.done()
    t.flush("f")
    r = S.rect("id:hud-chat", "HEADLESS-1")
    fx, fy = r[0] + r[2] / 2, r[1] + r[3] - 52
    t.act("pointer", "Click into the message field", "The field takes the keyboard", still="08-06-field")
    t.move(fx, fy); t.click(); t.done()
    msg = "Please also add Monday."
    t.act("keys", f"Type '{msg}' at 6 characters a second", "The text shows in the field", settle=0.6, still="08-07-typed")
    t.text(msg); t.done()
    t.act("key", "Press Return", "The message is sent and the field empties", settle=1.5, still="08-08-sent")
    t.key("Return"); t.done()
    t.flush("g")
    L.emit("voice.state", {"name": "Atlas", "state": "question", "task": "Tidy notes", "seq": 5})
    L.emit("voice.confirm", {"id": "51", "prompt": "Atlas wants to write the file notes.txt.", "detail": "~/Notes/friday/notes.txt (412 bytes)", "physical": True, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": True})
    t.wait(2)
    t.act("voice event", "The helper asks a question that needs a key", "A card: Atlas wants to write the file notes.txt, Alt+Y or Alt+N", settle=1.5, still="08-09-question")
    t.done()
    t.act("key", "Press Alt+y to answer yes", "The question is answered: the card goes", settle=2.5, still="08-10-alt-y")
    t.key("Alt+y"); t.done()
    t.flush("h")
    observed["answers"] = lines("claims.answers")
    observed["keys"] = lines("claims.keys")
    print("ANSWERS", observed["answers"], "KEYS", observed["keys"], flush=True)
    L.emit("voice.confirm", {"id": "51", "done": True})
    L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Tidy notes", "step": 2, "steps": 2, "seq": 6})
    t.wait(2)
    t.flush("pre-stop")
    sp = S.centre("id:hud-chat-stop", "HEADLESS-1")
    print("stop", sp)
    t.act("pointer", "Click Stop", "The helper is stopped (agent.cancel reaches the agent); the header changes", settle=2.5, still="08-11-stop")
    t.move(*sp); t.click(); t.done()
    t.flush("i")
    observed["chat_calls"] = lines("claims.chat")[-600:]
    print("CHAT", observed["chat_calls"], flush=True)
    t.act("look", "Look at the end state", "Chat still open, helper stopped", settle=1.0, still="08-12-end")
    t.done()
    t.wait(1)
    t.finish("08-voice.mp4")
    json.dump({"rows": t.rows, "observed": observed}, open(f"{OUT}/rows-ch8.json", "w"), indent=1)
finally:
    L.stop()
