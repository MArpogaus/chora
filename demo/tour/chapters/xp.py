"""Chapter xp: Erika writes a letter: one function `run(ctx)`; the rows are `t.act(..., expect=, contract=, cells=, capture=)`."""

import sys, time, os, json, subprocess, dataclasses, re, shutil, importlib.util
from launch import *
import launch
from human import Quick
from tour import Session, checker_for
from harness import Expect
from slides import slide
from chapters.checks import *

TEXT = "Dear Anna,\nthank you for the lovely visit.\nThe garden looks wonderful.\nLove, Erika"
ASK = "How do I make the sound louder?"
ANSWER = "Open Start, then Control Panel, then Sound, and choose 75%."
ASK2 = "Please copy my letter to My Documents."
DONE = "Done. The letter is in My Documents."
NOSUCH = "nosuchprogram"
O = "HEADLESS-1"


def looks_xp():
    """The checks of the acts of xp (tests/acts/looks/xp.py): the pane of Help, the key list."""
    here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "tests", "acts", "looks")
    sys.path.insert(0, here)
    spec = importlib.util.spec_from_file_location("looks_xp", os.path.join(here, "xp.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def clock_shown(after):
    """The clock of xp is drawn: a node whose id says clock, or a tray text that is a time."""
    for n in after.nodes:
        text = str((n.get("meta") or {}).get("text") or n.get("text") or "")
        if "clock" in str(n.get("id")) or (str(n.get("id")).startswith("xp-tray-") and re.search(r"\d:\d\d", text)):
            return []
    return ["no clock node in the scene"]


def tooltip_says(want):
    """A check: the tooltip on screen has these words."""

    def check(after):
        text = ((after.by_id().get("xp-tooltip") or {}).get("meta") or {}).get("text")
        return [] if text == want else [f"the tooltip says {text!r}, expected {want!r}"]

    return check


def desk_is(number):
    """A check: the desktop of the cube that xp shows on its output (`desks` of its state)."""

    def check(after):
        shown = [d.get("shown") for d in (after.states.get("xp") or {}).get("desks", [])]
        return [] if shown == [[number]] else [f"xp shows the desktops {shown}, expected [[{number}]]"]

    return check


def notice_says(word):
    """F-1: a notice of chora-notify that is on screen says the word (summary or body; a bus-level
    check: the card of the frame is looked at in the RECORD run, launch.notify_config keeps it up)."""

    def check(after):
        block = (after.blocks or {}).get("notify") or {}
        texts = [f"{n.get('summary')} {n.get('body')}" for n in block.get("shown") or [] if isinstance(n, dict)]
        return [] if any(word.lower() in t.lower() for t in texts) else [f"no notice that says {word!r} in chora-notify: {texts}"]

    return check


def run(ctx):
    OUT = ctx.out
    launch.ENTRIES[:] = []
    L = Launcher("xp4")
    stub = f"{L.dir}/stub"; os.makedirs(stub, exist_ok=True)
    yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
    open(f"{L.dir}/xp.yaml", "w").write(yaml)
    open("/home/user/notes/letter.txt", "w").write("")
    fuzzel_log(L.dir, stub)
    env = L.start([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml"], flavor="xp", xp=True,
                  companions=[[f"{BIN}/chora-notify", "--assets", f"{REPO}/flavors/xp/assets/notify"], [f"{BIN}/chora-assistant"]])
    S = Session(env, L.pids, BIN)
    time.sleep(6)
    t = Quick(S, OUT, f"{L.dir}/work", checker=checker_for(S, ["xp"], L.dir))
    t.begin("1 xp: Erika writes a letter", "01-xp")
    L.emit("voice.state", {"name": "Atlas", "state": "idle", "task": "", "seq": 0})
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

        wpctl_state = f"{stub}/wpctl.state"
        claims = f"{L.dir}/claims"
        Q = "id:xp-start-HEADLESS-1"

        def open_start(sx, sy):
            t.reach(sx, sy); t.click(); t.wait(0.5)
            t.flush("menu")
            if not S.query("id:xp-menu-frame"):
                print("START did not open: second click", flush=True)
                t.click(); t.wait(0.5); t.flush("menub")

        t.jump(660, 330)
        t.act("look", "Erika sits down at the xp desktop", "Desktop picture, taskbar with Start, desktop buttons, the paperclip and the volume in the tray", scene="x1", still="01-01-desktop", lead=0.1, settle=1.0, contract="X1", cells=["doors clock-door"], capture="Q", expect=Expect(focus="xp", verify=clock_shown))
        t.done()
        sx, sy = aim(Q)
        t.act("pointer", "Start, then Notepad", "The Start menu opens; the editor opens on an empty letter.txt in a window", scene="x2", still="01-02-start-menu", settle=0.5, contract="T1", cells=["act: the Start button opens the Start menu", "act: a click on Notepad starts it"], capture="Q", expect=Diff(open=[], delta=notice_is_new))
        open_start(sx, sy)
        x, y = item("Notepad")
        t.reach(x, y); t.click(); t.wait(6.0); t.done()
        t.act("look", "No text editor is installed: Notepad says so", "A notice names the missing editor (mousepad); no terminal editor opens", scene="x2", still=None, settle=0.2, lead=0.1, contract="T6", cells=["act: the balloon goes when the window of Notepad shows"], capture="Q", expect=Expect(open=[], verify=notice_says("mousepad")))
        t.wait(1.0); t.still("01-02b-no-editor"); t.wait(1.5); t.done()
        t.flush("card gone")
        S.dismiss_notices()
        open(f"{stub}/mousepad", "w").write("#!/bin/sh\nexec foot --app-id=Notepad --title=Notepad -o pad=14x12 -o colors.background=ffffff -o colors.foreground=000000 -o 'font=DejaVu Sans Mono:size=11' -o cursor.blink=no -e sh -c 'cat >>/home/user/notes/letter.txt'\n")
        os.chmod(f"{stub}/mousepad", 0o755)
        print("SET", S.ctl("call", "xp", "settings.set", json.dumps({"editor": "mousepad"})), flush=True)
        t.flush("set")
        sx, sy = aim(Q)
        t.act("pointer", "Start, Notepad again, now with a stand-in GUI editor on PATH", "The editor opens in a window", scene="x2", still="01-02c-notepad-gui", settle=0.5, contract="T1", cells=["gate-xp-notepad-fallback.sh"], capture="Q", expect=Expect(new_window={"owner": "xp", "focused": True}, open=[]))
        open_start(sx, sy)
        x, y = item("Notepad")
        t.reach(x, y); t.click(); t.cut(2.5); t.done()
        t.act("keys", "Types a short letter, one typo corrected, saves with Ctrl+D (stand-in)", "The three lines of the letter show in the editor window; the file is written", scene="x3", still="01-03-letter", settle=0.8, cells=["gate-xp-notepad-fallback.sh"], capture="Q", expect=Expect(verify=letter_is(TEXT)))
        t.type_quick(TEXT, typo_at=14, typo_char="r")
        t.wait(0.3); t.key("ctrl+d"); t.wait(0.5); t.done()
        t.flush("letter")
        print("SURFACES", [(n["id"], n.get("app_id")) for n in S.query("kind:surface")], flush=True)
        tx, ty = aim("id:xp-title-s1")
        t.act("pointer", "Drags the letter window aside by its title bar", "The window follows the pointer and bends like jelly, then settles", scene="x4", still=None, settle=0.2, contract="X2", cells=["act: a drag of a window shuts the Start menu"], capture="H50", expect=Diff(open=[], delta=moved("xp")))
        t.reach(tx, ty)
        t.drag(tx, ty, tx + 260, ty + 70, seconds=1.0, hold=0.1, mid_still="01-04-wobble")
        t.wait(0.5); t.done()
        px, py = aim("id:hud-figure")
        t.act("pointer", "Clicks the paperclip on the desktop (node hud-figure): she wants to know how to make the sound louder", "The helper's chat opens at the bottom right with an empty transcript", scene="x5", still="01-05-chat-open", settle=0.6, contract="T3", cells=["act: the chat of the helper opens"], capture="Q", expect=Expect(helper="idle", verify=shown("hud-chat-close")))
        t.reach(px, py); t.click(); t.wait(0.5); t.done()
        t.flush("chat")
        def field():
            """The text field of the chat: left of the Send button, on its line."""
            t.flush("field")
            r = S.rect("id:hud-chat-send", O)
            if r is None:
                raise RuntimeError("no chat to type in")
            return r[0] - 100, r[1] + r[3] / 2

        fx, fy = field()
        t.act("keys", "Clicks the field and types her question, sends it with Return", "Her question shows in the transcript", scene="x5", still="01-06-question", settle=0.5, contract="T3", cells=["act: the helper answers"], capture="Q", expect=Expect(verify=logged(f"{claims}.chat", ASK)))
        t.reach(fx, fy); t.click(); t.wait(0.4)
        t.type_quick(ASK, typo_at=9, typo_char="x")
        t.tap("Return", 0.3); t.done()
        t.flush("asked")
        L.emit("voice.state", {"name": "Atlas", "state": "working", "task": "Answering", "step": 1, "steps": 1, "seq": 1})
        t.wait(0.4)
        t.act("look", "The helper works on it", "The helper word says it works", scene="x5", still=None, settle=0.5, lead=0.1, contract="T3", cells=["act: the helper answers"], capture="Q", expect=Expect(helper="working"))
        t.done()
        json.dump([{"seq": 1, "role": "user", "text": ASK}, {"seq": 2, "role": "agent", "text": ANSWER}], open(f"{L.dir}/entries.json", "w"))
        L.emit("voice.state", {"name": "Atlas", "state": "done", "task": "Answering", "step": 1, "steps": 1, "seq": 2})
        t.wait(0.4)
        t.act("look", "She reads the answer", "The helper answers: Start, Control Panel, Sound, 75%", scene="x5", still="01-07-answer", settle=1.2, lead=0.1, contract="T3", cells=["act: the helper answers"], capture="Q+1", expect=Expect(helper="answered", verify=shown("hud-chat-close")))
        t.done()
        cx, cy = aim("id:hud-chat-close")
        t.act("pointer", "Closes the chat with its X button", "The chat closes; the paperclip stays", scene="x5", still=None, settle=0.2, contract="T3", cells=["act: the chat of the helper shuts"], capture="Q", expect=Expect(verify=gone("hud-chat-close")))
        t.reach(cx, cy); t.click(); t.wait(0.3); t.done()
        sx, sy = aim(Q)
        t.act("pointer", "Start, Control Panel, Sound, 75%", "The cascade opens step by step; the tray reads 75% afterwards", scene="x7", still="01-08-sound-menu", settle=0.4, contract="T4:volume", cells=["act: a click on 75 percent sets the level and shuts the popup"], capture="Q", expect=Expect(open=[], verify=volume_is(75, wpctl_state)))
        t.reach(sx, sy); t.click(); t.wait(0.4)
        t.flush("start2")
        print("MENU after Start:", bool(S.query("id:xp-menu-frame")), flush=True)
        if not S.query("id:xp-menu-frame"):
            t.click(); t.wait(0.4)
        t.reach(260, 542); t.wait(0.8); t.click(); t.wait(0.8)
        t.reach(430, 548, overshoot=False); t.reach(430, 650, overshoot=False); t.wait(0.9)
        t.reach(600, 650, overshoot=False); t.reach(600, 628, overshoot=False); t.wait(0.4); t.still("01-09-sound-levels"); t.click(); t.wait(0.4); t.done()
        spx, spy = aim("id:xp-speaker-HEADLESS-1")
        t.act("pointer", "The pointer rests on the speaker: the tray says 75%", "The tooltip reads Volume 75%", scene="x7", still="01-10-volume-75", settle=1.4, lead=0.1, contract="T4:volume", cells=["act: the tooltip of the speaker says the level that the popup set"], capture="Q", expect=Expect(verify=tooltip_says("Volume 75%")))
        t.reach(spx, spy); t.wait(1.2); t.done()
        t.act("key", "Murat presses Print Screen (Super+Print)", "A notice says the screenshot is saved, or says why not", scene="x8", still="01-10b-screenshot", settle=1.0, lead=0.1, contract="T4:screenshot", cells=["doors screenshot-key"], capture="Q+0.5", expect=Diff(delta=notice_is_new))
        t.key("Super+Print"); t.wait(1.2); t.done()
        t.flush("shot")
        S.dismiss_notices()
        sx, sy = aim(Q)
        t.act("pointer", "Start, Run..., types a name that is no program", "The Run box answers Nothing found for the name", scene="x9", still="01-10c-run", settle=0.6, lead=0.1, contract="T6", cells=["gate-xp-run.sh", "doors launcher-door"], capture="Q", expect=Expect(open=[], verify=box_says(L.dir, NOSUCH)))
        open_start(sx, sy)
        x, y = item("Run...")
        t.reach(x, y); t.click(); t.wait(1.5)
        t.type_quick(NOSUCH); t.tap("Return", 1.0); t.done()
        t.act("key", "Escape shuts the Run box", "The Run box closes", scene="x9", still=None, settle=0.4, lead=0.1, contract="T6", cells=["gate-xp-run.sh"], capture="Q", expect=Expect(open=[]))
        t.tap("Escape", 0.4); t.done()
        sx, sy = aim(Q)
        looks = looks_xp()
        t.act("pointer", "Start, Help and Support", "Help opens on a filled pane", scene="x10", still="01-10d-help", settle=0.8, lead=0.1, contract="T2", cells=["act: Help and Support of the Start menu opens Help on a filled pane", "doors keys-door"], capture="Q", opens=[looks.HELP], expect=Expect(open=[looks.HELP], verify=looks.pane_says("Click Start in the bottom left corner and click a program.")))
        open_start(sx, sy)
        x, y = item("Help and Support")
        t.reach(x, y); t.click(); t.wait(0.8); t.done()
        hx, hy = aim(f"id:{looks.HELP}-content-c-topic-4")
        t.act("pointer", "Clicks the topic Keyboard shortcuts", "Every key of the list shows inside the window", scene="x10", still="01-10e-keys", settle=0.6, lead=0.1, contract="T2", cells=["act: the topic Keyboard shortcuts lists every key inside the window", "doors keys-door"], capture="Q", known="I6,expect: ca-1 the key list of Help at 1280x720 ends below its pane (the acts hold it at 1280x800)", expect=Expect(verify=looks.key_table_whole))
        t.reach(hx, hy); t.click(); t.wait(0.5); t.done()
        t.act("key", "Escape shuts Help", "Help closes", scene="x10", still=None, settle=0.3, lead=0.1, contract="T2", cells=["act: Escape shuts Help"], capture="Q", closes=[looks.HELP], expect=Expect(open=[]))
        t.tap("Escape", 0.4); t.done()
        sx, sy = aim(Q)
        t.act("pointer", "Start, Turn Off Computer", "The dialog Turn Off Computer opens", scene="x11", still="01-10f-turn-off", settle=0.6, lead=0.1, contract="T4:power", cells=["doors power-door", "act: Super+Shift+e opens Turn Off Computer"], capture="Q", opens=["xp-session"], expect=Expect(open=["xp-session"]))
        open_start(sx, sy)
        x, y = item("Turn Off Computer")
        t.reach(x, y); t.click(); t.wait(0.8); t.done()
        cx, cy = aim("id:xp-session-button-5")
        t.act("pointer", "Murat thinks again and clicks Cancel", "The dialog closes, nothing was turned off", scene="x11", still=None, settle=0.4, lead=0.1, contract="T4:power", cells=["act: Escape shuts Turn Off Computer"], capture="Q", closes=["xp-session"], expect=Expect(open=[]))
        t.reach(cx, cy); t.click(); t.wait(0.4); t.done()
        trect = S.rect("id:xp-tray-HEADLESS-1", O)
        tx2, ty2 = (trect[0] + 10, trect[1] + trect[3] / 2) if trect else (1172, 705)
        t.act("pointer", "Opens the paperclip again and asks the helper to keep the letter safe", "The chat opens with the first answer; her second request is sent", scene="x6", still="01-11-second-ask", settle=0.4, contract="T3", cells=["act: the chat of the helper opens"], capture="Q", expect=Expect(verify=logged(f"{claims}.chat", ASK2)))
        t.reach(tx2, ty2); t.click(); t.wait(0.5)
        t.flush("again")
        fx, fy = field()
        t.reach(fx, fy); t.click(); t.wait(0.4)
        t.type_quick(ASK2, typo_at=12, typo_char="q")
        t.tap("Return", 0.3); t.done()
        t.flush("ask2")
        L.emit("voice.state", {"name": "Atlas", "state": "question", "task": "Copy the letter", "seq": 3})
        L.emit("voice.confirm", {"id": "61", "prompt": "Atlas wants to copy letter.txt to My Documents.", "detail": "~/notes/letter.txt (92 bytes)", "physical": True, "keys": {"yes": "Alt+y", "no": "Alt+n"}, "buttons": True})
        t.wait(0.6)
        t.act("look", "The helper asks before it touches a file", "A question card with Yes and No", scene="x6", still="01-12-question", settle=0.8, lead=0.1, contract="T3", cells=["act: the helper asks", "gate-xp-question.sh"], capture="Q", expect=Expect(helper="answered", verify=shown("hud-card-yes")))
        t.done()
        yx, yy = aim("id:hud-card-yes")
        t.act("pointer", "Erika clicks Yes", "The card goes; the helper reports it is done", scene="x6", still="01-13-done", settle=0.6, contract="T3", cells=["gate-xp-question.sh"], capture="Q", expect=Expect(verify=both(gone("hud-card-yes"), logged(f"{claims}.answers", '"yes": true'))))
        t.reach(yx, yy); t.click(); t.done()
        t.flush("yes")
        L.emit("voice.confirm", {"id": "61", "done": True})
        json.dump([{"seq": 1, "role": "user", "text": ASK}, {"seq": 2, "role": "agent", "text": ANSWER},
                   {"seq": 3, "role": "user", "text": ASK2}, {"seq": 4, "role": "agent", "text": DONE}], open(f"{L.dir}/entries.json", "w"))
        L.emit("voice.state", {"name": "Atlas", "state": "done", "task": "Done", "step": 1, "steps": 1, "seq": 4})
        t.wait(1.2)
        t.act("look", "She reads that it is done", "The helper says the letter is in My Documents", scene="x6", still="01-14-reply", settle=1.0, lead=0.1, contract="T3", cells=["act: the helper answers"], capture="Q+1", expect=Expect(helper="answered", verify=shown("hud-chat-close")))
        t.done()
        cx, cy = aim("id:hud-chat-close")
        t.act("pointer", "Closes the chat again", "The chat closes", scene="x6", still=None, settle=0.2, contract="T3", cells=["act: the chat of the helper shuts"], capture="Q", expect=Expect(verify=gone("hud-chat-close")))
        t.reach(cx, cy); t.click(); t.wait(0.4); t.done()
        t.flush("tray")
        trect = S.rect("id:xp-tray-HEADLESS-1", O); hrect = S.rect("id:xp-helper-HEADLESS-1", O)
        print("TRAY", trect, "HELPER", hrect, flush=True)
        tx2, ty2 = (trect[0] + 10, trect[1] + trect[3] / 2) if trect else (1172, 705)
        t.act("pointer", "Clicks the paperclip icon in the tray (xp-helper)", "The helper chat opens", scene="x5", still="01-15-tray-icon", settle=0.6, contract="T3", cells=["doors helper-door"], capture="Q", expect=Expect(verify=shown("hud-chat-close")))
        t.reach(tx2, ty2); t.click(); t.wait(0.6); t.done()
        t.flush("tray2")
        print("CHAT after tray click:", bool(S.query("id:hud-chat-close")), flush=True)
        cx, cy = aim("id:hud-chat-close")
        t.act("pointer", "Closes the chat with its X button", "The chat closes", scene="x5", still=None, settle=0.2, contract="T3", cells=["act: the chat of the helper shuts"], capture="Q", expect=Expect(verify=gone("hud-chat-close")))
        t.reach(cx, cy); t.click(); t.wait(0.4); t.done()
        t.act("key", "Ctrl+Alt+Right: turns the xp desktop cube by key", "The desktop turns like a cube to the next desktop", scene="x12", still=None, settle=0.1, lead=0.1, cells=["doors keys-door"], capture="H50", expect=Expect(verify=desk_is(2)))
        t.key("Ctrl+Alt+Right"); t.wait(0.5); t.still("01-16-cube-mid"); t.cut(0.5); t.wait(0.7); t.still("01-17-cube-desktop2"); t.done()
        t.act("key", "Ctrl+Alt+Left: back to the first desktop", "The first desktop", scene="x12", still=None, settle=0.1, lead=0.1, cells=["doors keys-door"], capture="Q", expect=Expect(verify=desk_is(1)))
        t.key("Ctrl+Alt+Left"); t.cut(0.6); t.wait(0.7); t.still("01-18-cube-back"); t.done()
        t.act("pointer", "Super held, drags on the desktop to turn the cube", "The desktop follows the pointer like a cube", scene="x12", still=None, settle=0.1, lead=0.1, cells=["doors keys-door"], capture="H50", known="I4: ca-4 Super released after the drag opens the Start menu", expect=Expect(verify=desk_is(2)))
        t.drag(1000, 250, 200, 250, seconds=0.9, mods=("Super_L",), hold=0.1, mid_still="01-19-super-drag-mid")
        t.cut(0.4); t.wait(0.7); t.still("01-20-super-drag-end"); t.done()
        slide(f"{L.dir}/slide.png", 1, "xp: a letter for Anna", ["Erika writes a letter, asks the paperclip how to make", "the sound louder, and finds the answer in the Start menu."])
        t.finish("01-xp.mp4", slide=f"{L.dir}/slide.png")
        json.dump(t.rows, open(f"{OUT}/rows-xp.json", "w"), indent=1)
    finally:
        L.stop()
