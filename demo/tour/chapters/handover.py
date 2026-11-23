"""Chapter handover: one function `run(ctx)`; the rows are `t.act(..., expect=, contract=, cells=, capture=)`."""

import sys, time, os, json, subprocess, re
from launch import *
import launch
from human import Quick
from tour import Session, checker_for
from harness import Expect
import json, shutil
from slides import slide
from chapters.checks import Diff, letter_is, logged, shown, gone, both, LETTER

LINE1 = "Ward 3: check bed 4 at 14:00"
LINE2 = "- Monday: plan the week"
LOCK = r"(swaylock|loginctl lock-session)"


def owner_is(sid, look):
    """A check: the node of the window says that the look owns it (the window has an owner, none is lost)."""

    def check(after):
        node = after.by_id().get(sid)
        if node is None:
            return [f"no window {sid} in the scene"]
        return [] if node.get("owner") == look else [f"{sid} is owned by {node.get('owner')}, expected {look}"]

    return check


def focus_with(look):
    """A check: the look says that it holds the focus, and no other look does (I3)."""

    def check(after):
        holders = [name for name, state in after.states.items() if state and (state.get("focus") or {}).get("holder") == name]
        return [] if holders == [look] else [f"the focus is with {holders}, expected {look}"]

    return check


def windows_only(*sids):
    """A check: the toplevels are these windows: one window through all four looks, none lost, none new (T7)."""
    return lambda after: [] if after.toplevels == set(sids) else [f"the windows are {sorted(after.toplevels)}, expected {sorted(sids)}"]


def shown_in(look, sid):
    """A check: the state of the look lists the window as shown (a page of mc, a pane of the construct)."""

    def check(after):
        windows = (after.states.get(look) or {}).get("windows") or []
        return [] if any(w.get("surface") == sid and w.get("shown") for w in windows) else [f"{look} does not show {sid}: {windows}"]

    return check


def no_task_button(sid):
    """A check: xp keeps no task button for the window it gave away."""
    return lambda after: [f"xp keeps a task button for {sid}" for n in after.nodes if str(n.get("id")).startswith(f"xp-task-{sid}-")][:1]


def lock_called(calls):
    """A check: the lock command of a look ran (the stand-in `swaylock` or `loginctl` logs its call)."""

    def check(after):
        lines = open(calls).read().splitlines() if os.path.exists(calls) else []
        return [] if any(re.match(LOCK, line) for line in lines) else [f"no swaylock or loginctl lock-session call: {lines}"]

    return check

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
    for what in ("swaylock", "loginctl"):
        open(f"{L.dir}/stub/{what}", "w").write(f"#!/bin/sh\necho {what} \"$@\" >>{L.dir}/calls.log\n")
        os.chmod(f"{L.dir}/stub/{what}", 0o755)
    open(LETTER, "w").write("")
    note_stub = f"{L.dir}/stub/mousepad"
    open(note_stub, "w").write("#!/bin/sh\nexec foot --app-id=Notepad --title=Notepad -o pad=14x12 -o colors.background=ffffff -o colors.foreground=000000 -o 'font=DejaVu Sans Mono:size=11' -o cursor.blink=no -e sh -c \'cat >>/home/user/notes/letter.txt\'\n")
    os.chmod(note_stub, 0o755)
    xp_yaml = open(f"{REPO}/plugins/chora-xp/xp.yaml").read().replace("sounds: true", "sounds: false").replace("test_accept_injected_keys: false", "test_accept_injected_keys: true")
    open(f"{L.dir}/xp.yaml", "w").write(xp_yaml)
    L.spawn([f"{BIN}/chora-xp", "--theme", f"{REPO}/flavors/xp/theme", "--config", f"{L.dir}/xp.yaml", "--output", out_of["xp"]], env, "xp.log")
    S.ctl("wait", "owner:xp", "--timeout", "30")
    L.spawn(tiling_argv(f"{BIN}/chora-tiling", "--output", out_of["tiling"]), env, "tiling.log")
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
        calls = f"{L.dir}/calls.log"
        both_lines = LINE1 + "\n" + LINE2
        t.pair = [XP, TI]
        t.pointer_output = XP
        t.jump(640, 360)
        t.wait(0.3)
        t.act("look", "Two screens: xp on the left, tiling on the right (empty)", "xp desktop with taskbar; tiling desktop empty", scene="h1", still="05-01-two-screens", contract="X1", cells=["gate-handover-matrix.sh"], capture="Q", expect=Expect(verify=windows_only()))
        t.done()
        t.act("pointer", "Click Start", "The Start menu opens on the xp screen", scene="h1", still="05-02-start", opens=["xp-menu"], contract="T1", cells=["doors launcher-door"], capture="Q", known="I3: ca-3 construct and xp each say they hold the focus on a four-output session", expect=Expect(open=["xp-menu"]))
        goto(50, 705, XP, 1.0); t.click(); t.done()
        ex, ey = find_item("Notepad", XP)
        t.act("pointer", "Click Notepad", "A text editor opens on the xp screen", scene="h1", settle=1.0, still="05-03-editor-open", contract="T1", cells=["doors launcher-door"], capture="Q", closes=["xp-menu"], expect=Expect(new_window={"owner": "xp", "focused": True}, open=[]))
        goto(ex, ey, XP, 1.0); t.click(); t.done()
        t.act("keys", "The nurse types the first line in the editor", "Ward 3: check bed 4 at 14:00 shows in the editor", scene="h1", settle=1.0, still="05-03b-typed-on-xp", contract="T1", cells=["gate-xp-notepad-fallback.sh"], capture="Q", expect=Expect(verify=letter_is(LINE1)))
        t.type_quick(LINE1 + "\n"); t.done()
        t.flush("menu")
        tx, ty = mid("id:xp-title-s1", XP)
        t.act("pointer", "Title-bar right click, Send To, Tiling desktop", "The editor window moves to the tiling screen by the menu and tiles there", scene="h2", settle=1.0, still="05-04-send-to-tiling", contract="T7", cells=["gate-handover.sh 8b"], capture="Q", expect=Expect(focus="tiling", verify=both(owner_is("s1", "tiling"), no_task_button("s1"), windows_only("s1"))))
        goto(tx, ty, XP, 1.0); t.click(273); t.wait(0.4)
        t.flush("ctx")
        sx, sy = mid("id:xp-ctx-0-6", XP)
        goto(sx, sy, XP, 0.6); t.wait(0.5)
        t.flush("sub")
        ux, uy = mid("id:xp-ctx-1-0", XP)
        goto(ux, uy, XP, 0.4); t.click(); t.done()
        t.pair = [TI, MC]
        t.act("keys", "The nurse types a line in the editor on its new screen", "The text appears in the editor on the tiling screen", scene="h3", settle=1.0, still="05-05-typed-on-tiling", contract="T7", cells=["gate-handover-matrix.sh"], capture="Q", expect=Expect(verify=both(letter_is(both_lines), owner_is("s1", "tiling"))))
        t.type_quick(LINE2, typo_at=9, typo_char="q"); t.tap("Return", 0.3); t.done()
        t.act("key", "Super+Alt+3 gives the window to mc", "The window leaves tiling and shows in mc as a page", scene="h3", settle=1.0, still="05-06-give-key-mc", contract="T7", cells=["gate-handover-matrix.sh tiling-mc"], capture="Q", expect=Expect(focus="mc", verify=both(owner_is("s1", "mc"), shown_in("mc", "s1"), windows_only("s1"))))
        t.key("Super+Alt+3"); t.done()
        t.pair = [MC, CO]
        t.act("key", "Super+Alt+4 gives the window to the construct", "The window leaves mc and is a pane in the construct", scene="h4", settle=1.0, still="05-07-give-key-construct", contract="C2", cells=["gate-handover-matrix.sh mc-construct", "gate-give-holders.sh"], capture="Q", expect=Expect(verify=both(owner_is("s1", "construct"), windows_only("s1"))))
        t.key("Super+Alt+4"); t.done()
        t.pair = [CO, XP]
        t.act("pointer", "Super+drag the pane of the editor from the construct onto the xp screen", "The window leaves the construct and shows in xp with a title bar and a task button", scene="h5", settle=1.0, still="05-08-drag-construct-to-xp", contract="C2", cells=["gate-construct-pull.sh drop"], capture="H50", expect=Expect(verify=both(owner_is("s1", "xp"), windows_only("s1"))))
        on_node(CO); press(("Super_L",))
        goto(100, 300, XP, 1.0); goto(500, 300, XP, 0.8)
        release(("Super_L",)); t.done()
        t.pair = [XP, TI]
        t.act("look", "xp has the window with all the lines", "The editor window of xp has every line; one window, none lost", scene="h6", settle=1.0, still="05-09-all-lines", contract="T7", cells=["gate-handover-matrix.sh"], capture="Q", expect=Expect(verify=both(owner_is("s1", "xp"), windows_only("s1"), letter_is(both_lines))))
        t.done()
        t.act("key", "Super+Shift+Escape locks every screen", "Every look asks to lock", scene="h7", settle=1.0, still="05-10-lock", contract="T4:lock", cells=["doors lock-key"], capture="Q", expect=Expect(verify=lock_called(calls)))
        t.key("Super+Shift+Escape"); t.wait(0.8); t.done()
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
        json.dump({"rows": t.rows, "observed": t.observed}, open(f"{OUT}/rows-handover.json", "w"), indent=1)
    finally:
        L.stop()
