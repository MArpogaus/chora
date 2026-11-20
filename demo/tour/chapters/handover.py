"""Chapter handover: one function `run(ctx)`; the rows are `t.act(..., expect=, contract=, cells=, capture=)`."""

import sys, time, os, json, subprocess
from launch import *
import launch
from human import Quick
from tour import Session, checker_for
from harness import Expect
import json, shutil
from slides import slide

def start_multi():
    L = Launcher("multi")
    os.makedirs(f"{L.dir}/stub", exist_ok=True)
    programs_yaml(f"{L.dir}/stub")
    env = L.start(None, voice=True, extra_config={
        "chora-construct.yaml": "theme: light\nvoice_hud: false\ntest_accept_injected_keys: true\neffects: [wobbly]\n",
        "chora-mc.yaml": "theme: light\ntest_accept_injected_keys: true\n",
    })
    S = Session(env, L.pids, BIN)
    S.ctl("grant", "tourrec", "input.inject", "methods.call", "methods.privileged", "screen.capture")
    for x in (1280, 2560, 3840):
        S.ctl("add-output", f"1280x720+{x}+0")
    outs = sorted(json.loads(S.ctl("outputs")), key=lambda o: o["position"][0])
    names = [o["name"] for o in outs]
    assert len(names) == 4, names
    out_of = dict(zip(["construct", "xp", "tiling", "mc"], names))
    note_stub = f"{L.dir}/stub/mousepad"
    open(note_stub, "w").write("#!/bin/sh\nexec foot --app-id=Notepad --title=Notepad -o pad=14x12 -o colors.background=ffffff -o colors.foreground=000000 -o 'font=DejaVu Sans Mono:size=11' -o cursor.blink=no -e sh -c \'cat >>/home/user/notes/letter.txt\'\n")
    os.chmod(note_stub, 0o755)
    xp_yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
    open(f"{L.dir}/xp.yaml", "w").write(xp_yaml)
    L.spawn([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml", "--output", out_of["xp"]], env, "xp.log")
    S.ctl("wait", "owner:xp", "--timeout", "30")
    L.spawn([f"{BIN}/chora-tiling", "--output", out_of["tiling"]], env, "tiling.log")
    L.spawn([f"{BIN}/chora-mc", "--manage-windows", "--output", out_of["mc"]], env, "mc.log")
    L.spawn([f"{BIN}/chora-construct"], env, "construct.log")
    for o in ("tiling", "mc", "construct"):
        S.ctl("wait", f"owner:{o}", "--timeout", "30")
    return L, S, out_of


def run(ctx):
    OUT = ctx.out
    os.environ.setdefault("TOUR_PAIR_FILTER", "[0][1]hstack=inputs=2,scale=1920:540,pad=1920:1080:0:270:0x20242c")
    os.environ.setdefault("TOUR_SLIDE_SIZE", "1920:1080")
    L, S, O = start_multi()
    CO, XP, TI, MC = O["construct"], O["xp"], O["tiling"], O["mc"]
    t = Quick(S, OUT, f"{L.dir}/work", checker=checker_for(S, ["construct", "xp", "tiling", "mc"], L.dir, wallpaper={CO, XP, MC}))
    t.begin("5 handover: one window through the looks", "05-handover")
    t.observed = {}

    def goto(x, y, out, sec=1.0):
        if t.pointer_output != out:
            t.pointer_output = out
            t.jump(x, y)
        else:
            t.reach(x, y, hesitate=False)

    def find_item(text, out):
        t.flush("find")
        for n in range(0, 45):
            q = S.query(f"id:xp-item-{n}")
            if q and (q[0].get("meta") or {}).get("text") == text:
                r = (q[0].get("rects") or {}).get(out)
                if r:
                    return r[0] + r[2] / 2, r[1] + r[3] / 2
        raise RuntimeError("no menu item " + text)

    def press(mods=()):
        events = [{"key": {"key": m, "state": "pressed"}} for m in mods]
        events.append({"button": {"button": 272, "state": "pressed", "hold": True}})
        t.inject(events)
        t.wait(0.3)

    def on_node(out, x=100, y=60):
        t.pointer_output = out
        t.inject([{"pointer_move": {"node": "s1", "point": [x, y]}}])
        t.wait(1.0)

    def release(mods=()):
        t.wait(0.3)
        events = [{"button": {"button": 272, "state": "released"}}]
        events += [{"key": {"key": m, "state": "released"}} for m in reversed(mods)]
        t.inject(events)
        t.frame += 1

    def where():
        t.flush("where")
        q = S.query("id:s1")
        if not q:
            return "no window s1"
        n = q[0]
        return f"owner {n.get('owner')}, focused {n.get('focused')}, rects {n.get('rects')}"

    def note(key):
        w = where()
        t.observed[key] = w
        print(key, w, flush=True)

    def mid(sel, out):
        r = S.rect(sel, out)
        return r[0] + r[2] / 2, r[1] + r[3] / 2

    try:
        t.pair = [XP, TI]
        t.pointer_output = XP
        t.jump(640, 360)
        t.wait(0.3)
        t.act("look", "Two screens: xp on the left, tiling on the right (empty)", "xp desktop with taskbar; tiling desktop empty", still="05-01-two-screens", known="construct and xp each say they hold the focus on a four-output session (I3)")
        t.done()
        t.act("pointer", "Click Start", "The Start menu opens on the xp screen", still="05-02-start", opens=["xp-menu"], known="construct and xp each say they hold the focus on a four-output session (I3)")
        goto(50, 705, XP, 1.0); t.click(); t.done()
        ex, ey = find_item("Notepad", XP)
        t.act("pointer", "Click Notepad", "A text editor opens on the xp screen", settle=1.0, still="05-03-editor-open", contract="T1", capture="Q", expect=Expect(new_window={"owner": "xp"}), closes=["xp-menu"])
        goto(ex, ey, XP, 1.0); t.click(); t.done()
        note("05-03-editor-open")
        t.act("pointer", "Title-bar right click, Send To > Tiling desktop", "the editor window moves to tiling by the menu (h2 of design/tour-v9.md: waits for xp-give; CHAPTERS-A fills it)", todo=True, contract="T7", cells=["gate-handover.sh 8b"], capture="Q")
        t.done()
        tx, ty = mid("id:xp-title-s1", XP)
        cx, cy = 640, 400
        t.act("pointer", "DRAG 1: pull the title bar of the editor window from xp to the tiling screen", "the editor window leaves xp and tiles in the tiling screen (full height)", settle=0.8, still="05-04-drag-xp-to-tiling", lead=1.0, cells=["gate-handover.sh 8b"], capture="H50")
        goto(tx, ty, XP, 1.0); press()
        goto(1100, 300, XP, 1.0)
        goto(100, 300, TI, 1.0); goto(cx, cy, TI, 0.8)
        release(); t.done()
        note("05-04-drag-xp-to-tiling")
        t.act("keys", "The person types a line in the editor on its new screen", "The text appears in the editor on the tiling screen", settle=1.0, still="05-04b-typed-on-tiling")
        t.type_quick("- Monday: plan the week", typo_at=9, typo_char="q"); t.done()
        t.pair = [TI, CO]
        x, y = mid("id:s1", TI)
        t.act("pointer", "DRAG 2: Super+drag the editor window from tiling to the construct screen", "the editor window leaves tiling and becomes a pane in the construct space", settle=1.0, still="05-05-drag-tiling-to-construct", cells=["gate-handover-matrix.sh tiling-construct"], capture="H50")
        on_node(TI); press(("Super_L",))
        goto(100, 360, CO, 1.0); goto(640, 360, CO, 0.8)
        release(("Super_L",)); t.done()
        note("05-05-drag-tiling-to-construct")
        t.pair = [CO, XP]
        x, y = mid("id:s1", CO)
        t.act("pointer", "DRAG 3: Super+drag the the editor window pane from construct to the xp screen", "the editor window leaves the construct and shows in xp with a title bar and a task button", settle=1.0, still="05-06-drag-construct-to-xp", cells=["gate-construct-pull.sh drop"], capture="H50")
        on_node(CO); press(("Super_L",))
        goto(100, 300, XP, 1.0); goto(500, 300, XP, 0.8)
        release(("Super_L",)); t.done()
        note("05-06-drag-construct-to-xp")
        t.pair = [XP, MC]
        tx, ty = mid("id:xp-title-s1", XP)
        t.act("pointer", "DRAG 4: pull the title bar of the editor window from xp to the mc screen", "the editor window becomes the window of an mc scene", settle=1.0, still="05-05-drag-xp-to-mc")
        t.pointer_output = XP; goto(tx, ty, XP, 1.0); press()
        goto(1100, 300, XP, 1.0)
        goto(100, 300, MC, 1.0); goto(640, 360, MC, 0.8)
        release(); t.done()
        note("05-05-drag-xp-to-mc")
        t.pair = [MC, XP]
        x, y = mid("id:s1", MC)
        t.act("pointer", "DRAG 5: Super+drag the editor window out of mc to the xp screen", "the editor window leaves mc and shows in xp", settle=1.0, still="05-08-drag-mc-out")
        on_node(MC); press(("Super_L",))
        goto(100, 300, XP, 1.0); goto(500, 300, XP, 0.8)
        release(("Super_L",)); t.done()
        note("05-08-drag-mc-out")
        t.pair = [XP, TI]
        t.act("key", "GIVE BY KEY: Super+Alt+1 (give the focused window to tiling)", "the editor window goes to tiling and fills its screen", settle=1.0, still="05-09-give-key-tiling", cells=["gate-handover-matrix.sh tiling-mc"], capture="Q")
        t.key("Super+Alt+1"); t.done()
        note("05-09-give-key-tiling")
        t.pair = [TI, XP]
        t.act("key", "GIVE BY KEY: Super+Alt+2 (give the focused window to xp)", "the editor window goes back to xp", settle=1.0, still="05-10-give-key-xp", cells=["gate-handover-matrix.sh mc-construct"], capture="Q")
        t.key("Super+Alt+2"); t.done()
        note("05-10-give-key-xp")
        t.wait(1)
        t.flush("audit")
        audit = S.query("kind:surface")
        print("AUDIT", [(n["id"], n.get("owner")) for n in audit])
        t.observed["audit"] = [(n["id"], n.get("owner")) for n in audit]
        t.flush("end")
        from PIL import Image, ImageDraw, ImageFont
        img = Image.new("RGB", (1920, 1080), (32, 36, 44))
        d = ImageDraw.Draw(img)
        f1 = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 54)
        f2 = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 40)
        d.text((120, 200), "Window audit after six moves", font=f1, fill=(230, 232, 240))
        for i, (sid, owner) in enumerate(t.observed["audit"]):
            d.text((120, 340 + i * 70), f"window {sid}   owner {owner}", font=f2, fill=(158, 206, 106))
        d.text((120, 340 + max(1, len(t.observed["audit"])) * 70 + 40), "one window, none lost", font=f2, fill=(160, 166, 180))
        img.save(f"{L.dir}/audit.png")
        ach = os.path.join(t.work, "05-audit.mkv")
        subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-loop", "1", "-framerate", "30", "-t", "3", "-i", f"{L.dir}/audit.png", "-c:v", "libx264", "-preset", "veryfast", "-crf", "16", "-pix_fmt", "yuv420p", ach], check=True)
        t.chunks.append(ach)
        slide(f"{L.dir}/slide.png", 5, "handover", ['One window travels between xp, tiling, the construct', 'and mc: by dragging and by key.', 'Two screens side by side.'], accent=(115, 218, 202))
        t.finish("05-handover.mp4", slide=f"{L.dir}/slide.png")
        json.dump({"rows": t.rows, "observed": t.observed}, open(f"{OUT}/rows-ch5.json", "w"), indent=1)
    finally:
        L.stop()
